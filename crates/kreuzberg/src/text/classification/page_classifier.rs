//! Per-page LLM classification driver.
//!
//! Iterates the page boundaries attached to the extraction result, renders a
//! Minijinja prompt for each page, and asks the configured LLM to return a
//! single-label or multi-label classification using
//! [`crate::llm::structured::complete_with_json_schema`].

use serde_json::{Value, json};

use crate::core::config::PageClassificationConfig;
use crate::types::classification::{ClassificationLabel, PageClassification};
use crate::types::{ExtractionResult, LlmUsage};

/// Default Jinja2 template used when `PageClassificationConfig::prompt_template`
/// is `None`. Variables: `labels` (joined comma-separated list), `page_text`,
/// `multi_label` (bool).
pub const DEFAULT_CLASSIFICATION_TEMPLATE: &str = "\
You are a precise document classification system. Classify the page text below \
using ONLY labels from this list: {{ labels }}.

{% if multi_label %}Return every label that applies. Order is not significant.\
{% else %}Return exactly one label — the single best fit.{% endif %}

Page text:
{{ page_text }}

Respond as JSON that matches the provided schema. Do not invent labels not in \
the list. If no label fits, return an empty array{% if not multi_label %} or omit `label`{% endif %}.";

/// Build the JSON schema the LLM is constrained to obey.
///
/// Single-label schema: `{"label": str, "confidence": float?}`. Multi-label
/// schema: `{"labels": [{"label": str, "confidence": float?}]}`.
fn build_schema(labels: &[String], multi_label: bool) -> Value {
    let label_enum: Vec<Value> = labels.iter().map(|l| Value::String(l.clone())).collect();
    let label_object = json!({
        "type": "object",
        "properties": {
            "label": { "type": "string", "enum": label_enum },
            "confidence": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
        },
        "required": ["label"],
        "additionalProperties": false,
    });

    if multi_label {
        json!({
            "type": "object",
            "properties": {
                "labels": {
                    "type": "array",
                    "items": label_object,
                },
            },
            "required": ["labels"],
            "additionalProperties": false,
        })
    } else {
        label_object
    }
}

/// Slice `content` on the supplied byte boundaries and return `(page_number, text)` tuples.
///
/// Defensively clamps each boundary to the next valid `char` boundary so that a
/// page boundary computed mid-codepoint cannot trigger a slice panic. In
/// practice extractors emit boundaries on line ends, so this is a safety net.
fn split_pages<'a>(content: &'a str, boundaries: &[crate::types::page::PageBoundary]) -> Vec<(u32, &'a str)> {
    let len = content.len();
    boundaries
        .iter()
        .filter_map(|b| {
            let start = clamp_to_char_boundary_floor(content, b.byte_start.min(len));
            let end = clamp_to_char_boundary_floor(content, b.byte_end.min(len));
            if end <= start {
                return None;
            }
            Some((b.page_number, &content[start..end]))
        })
        .collect()
}

/// Round `offset` down to the nearest `char` boundary in `content`. Returns
/// `offset` unchanged when it already is one (or equals `content.len()`).
fn clamp_to_char_boundary_floor(content: &str, mut offset: usize) -> usize {
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

/// Convert the parsed LLM JSON into a [`PageClassification`] entry.
fn parse_response(page_number: u32, value: &Value, multi_label: bool) -> PageClassification {
    let mut labels = Vec::new();
    if multi_label {
        if let Some(arr) = value.get("labels").and_then(|v| v.as_array()) {
            for entry in arr {
                if let Some(label) = entry.get("label").and_then(|v| v.as_str()) {
                    labels.push(ClassificationLabel {
                        label: label.to_string(),
                        confidence: entry.get("confidence").and_then(|v| v.as_f64()).map(|f| f as f32),
                    });
                }
            }
        }
    } else if let Some(label) = value.get("label").and_then(|v| v.as_str()) {
        labels.push(ClassificationLabel {
            label: label.to_string(),
            confidence: value.get("confidence").and_then(|v| v.as_f64()).map(|f| f as f32),
        });
    }
    PageClassification { page_number, labels }
}

/// Resolve the page chunks the classifier should operate on. When the
/// extraction has no page boundary metadata, the whole content is treated as a
/// single page (`page_number = 1`).
fn page_slices(result: &ExtractionResult) -> Vec<(u32, &str)> {
    let boundaries = result
        .metadata
        .pages
        .as_ref()
        .and_then(|p| p.boundaries.as_deref());

    match boundaries {
        Some(b) if !b.is_empty() => split_pages(&result.content, b),
        _ => {
            if result.content.is_empty() {
                Vec::new()
            } else {
                vec![(1, result.content.as_str())]
            }
        }
    }
}

/// Run page classification against an extraction result.
///
/// Mutates `result.page_classifications` with one entry per non-empty page and
/// appends every LLM call's usage to `result.llm_usage`.
///
/// # Errors
///
/// Returns the first error encountered when rendering the prompt or calling the
/// LLM. Partially produced classifications are discarded so callers do not see
/// a half-populated vector.
pub async fn classify_pages(result: &mut ExtractionResult, config: &PageClassificationConfig) -> crate::Result<()> {
    if config.labels.is_empty() {
        return Err(crate::KreuzbergError::validation(
            "PageClassificationConfig.labels must contain at least one entry",
        ));
    }

    let pages = page_slices(result);
    if pages.is_empty() {
        return Ok(());
    }

    let template = config
        .prompt_template
        .as_deref()
        .unwrap_or(DEFAULT_CLASSIFICATION_TEMPLATE);
    let labels_joined = config.labels.join(", ");
    let schema = build_schema(&config.labels, config.multi_label);
    let schema_name = if config.multi_label {
        "page_classification_multi"
    } else {
        "page_classification_single"
    };

    let mut classifications: Vec<PageClassification> = Vec::with_capacity(pages.len());
    let mut usages: Vec<LlmUsage> = Vec::new();

    for (page_number, page_text) in pages {
        let ctx = minijinja::context! {
            labels => &labels_joined,
            page_text => page_text,
            multi_label => config.multi_label,
        };
        let prompt = crate::llm::prompts::render_template(template, &ctx)?;

        let (value, usage) = crate::llm::structured::complete_with_json_schema(
            &config.llm,
            &prompt,
            schema_name,
            &schema,
            "page_classification",
        )
        .await?;

        classifications.push(parse_response(page_number, &value, config.multi_label));
        if let Some(u) = usage {
            usages.push(u);
        }
    }

    if !classifications.is_empty() {
        result.page_classifications = Some(classifications);
    }
    if !usages.is_empty() {
        result.llm_usage.get_or_insert_with(Vec::new).extend(usages);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_schema_single_label_uses_label_field() {
        let labels = vec!["invoice".to_string(), "memo".to_string()];
        let schema = build_schema(&labels, false);
        assert_eq!(schema["properties"]["label"]["type"], "string");
        assert!(schema["properties"]["labels"].is_null());
    }

    #[test]
    fn build_schema_multi_label_uses_labels_array() {
        let labels = vec!["invoice".to_string(), "memo".to_string()];
        let schema = build_schema(&labels, true);
        assert_eq!(schema["properties"]["labels"]["type"], "array");
        assert_eq!(
            schema["properties"]["labels"]["items"]["properties"]["label"]["type"],
            "string"
        );
    }

    #[test]
    fn parse_response_single_label_extracts_label_and_confidence() {
        let payload = json!({ "label": "invoice", "confidence": 0.92 });
        let parsed = parse_response(3, &payload, false);
        assert_eq!(parsed.page_number, 3);
        assert_eq!(parsed.labels.len(), 1);
        assert_eq!(parsed.labels[0].label, "invoice");
        assert_eq!(parsed.labels[0].confidence, Some(0.92));
    }

    #[test]
    fn parse_response_multi_label_yields_every_entry() {
        let payload = json!({
            "labels": [
                {"label": "invoice", "confidence": 0.8},
                {"label": "memo"},
            ]
        });
        let parsed = parse_response(2, &payload, true);
        assert_eq!(parsed.page_number, 2);
        assert_eq!(parsed.labels.len(), 2);
        assert_eq!(parsed.labels[0].confidence, Some(0.8));
        assert_eq!(parsed.labels[1].confidence, None);
    }

    #[test]
    fn split_pages_respects_boundaries() {
        let content = "alpha\nbeta\ngamma";
        let boundaries = vec![
            crate::types::page::PageBoundary {
                byte_start: 0,
                byte_end: 6,
                page_number: 1,
            },
            crate::types::page::PageBoundary {
                byte_start: 6,
                byte_end: 11,
                page_number: 2,
            },
        ];
        let slices = split_pages(content, &boundaries);
        assert_eq!(slices.len(), 2);
        assert_eq!(slices[0].0, 1);
        assert_eq!(slices[0].1, "alpha\n");
        assert_eq!(slices[1].0, 2);
        assert_eq!(slices[1].1, "beta\n");
    }
}
