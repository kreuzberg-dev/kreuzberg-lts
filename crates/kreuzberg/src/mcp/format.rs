//! MCP response formatting and configuration helpers.
//!
//! This module provides utilities for formatting extraction results and building configurations.

use crate::{ExtractionConfig, ExtractionResult as KreuzbergResult};

/// Build extraction config from MCP parameters.
///
/// Merges the provided config JSON (if any) with the default config.
pub(super) fn build_config(
    default_config: &ExtractionConfig,
    config_json: Option<serde_json::Value>,
) -> Result<ExtractionConfig, String> {
    let mut config = default_config.clone();

    if let Some(json) = config_json {
        // Attempt to merge the provided config JSON with the default
        match serde_json::from_value::<ExtractionConfig>(json) {
            Ok(provided_config) => {
                // Merge: override default settings with provided config
                config = provided_config;
            }
            Err(e) => {
                return Err(format!("Invalid extraction config: {}", e));
            }
        }
    }

    Ok(config)
}

/// Format extraction result as human-readable text.
pub(super) fn format_extraction_result(result: &KreuzbergResult) -> String {
    let mut response = String::new();

    response.push_str(&format!("Content ({} characters):\n", result.content.len()));
    response.push_str(&result.content);
    response.push_str("\n\n");

    response.push_str("Metadata:\n");
    response.push_str(&serde_json::to_string_pretty(&result.metadata).unwrap_or_default());
    response.push_str("\n\n");

    if !result.tables.is_empty() {
        response.push_str(&format!("Tables ({}):\n", result.tables.len()));
        for (i, table) in result.tables.iter().enumerate() {
            response.push_str(&format!("\nTable {} (page {}):\n", i + 1, table.page_number));
            response.push_str(&table.markdown);
            response.push('\n');
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_with_no_config() {
        let default_config = ExtractionConfig::default();

        let config = build_config(&default_config, None).unwrap();
        assert_eq!(config.use_cache, default_config.use_cache);
    }

    #[test]
    fn test_build_config_with_config_json() {
        let default_config = ExtractionConfig::default();
        let config_json = serde_json::json!({
            "use_cache": false
        });

        let config = build_config(&default_config, Some(config_json)).unwrap();
        assert!(!config.use_cache);
    }

    #[test]
    fn test_build_config_with_invalid_config_json() {
        let default_config = ExtractionConfig::default();
        let config_json = serde_json::json!({
            "invalid_field": "value"
        });

        let result = build_config(&default_config, Some(config_json));
        assert!(result.is_err());
    }

    #[test]
    fn test_build_config_preserves_default_config_settings() {
        let default_config = ExtractionConfig {
            use_cache: false,
            ..Default::default()
        };

        let config = build_config(&default_config, None).unwrap();

        assert!(!config.use_cache);
    }

    #[test]
    fn test_build_config_overrides_default_settings() {
        let default_config = ExtractionConfig {
            use_cache: true,
            ..Default::default()
        };

        let config_json = serde_json::json!({
            "use_cache": false
        });

        let config = build_config(&default_config, Some(config_json)).unwrap();
        assert!(!config.use_cache);
    }

    #[test]
    fn test_format_extraction_result_with_content() {
        let result = KreuzbergResult {
            content: "Sample extracted text".to_string(),
            mime_type: "text/plain".to_string(),
            metadata: crate::Metadata::default(),
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: None,
            pages: None,
            elements: None,
            djot_content: None,
        };

        let formatted = format_extraction_result(&result);

        assert!(formatted.contains("Content (21 characters)"));
        assert!(formatted.contains("Sample extracted text"));
        assert!(formatted.contains("Metadata:"));
    }

    #[test]
    fn test_format_extraction_result_with_tables() {
        let result = KreuzbergResult {
            content: "Document with tables".to_string(),
            mime_type: "application/pdf".to_string(),
            metadata: crate::Metadata::default(),
            tables: vec![
                crate::Table {
                    cells: vec![
                        vec!["Col1".to_string(), "Col2".to_string()],
                        vec!["A".to_string(), "B".to_string()],
                    ],
                    page_number: 1,
                    markdown: "| Col1 | Col2 |\n|------|------|\n| A    | B    |".to_string(),
                },
                crate::Table {
                    cells: vec![
                        vec!["X".to_string(), "Y".to_string()],
                        vec!["1".to_string(), "2".to_string()],
                    ],
                    page_number: 2,
                    markdown: "| X | Y |\n|---|---|\n| 1 | 2 |".to_string(),
                },
            ],
            detected_languages: None,
            chunks: None,
            images: None,
            pages: None,
            elements: None,
            djot_content: None,
        };

        let formatted = format_extraction_result(&result);

        assert!(formatted.contains("Tables (2)"));
        assert!(formatted.contains("Table 1 (page 1)"));
        assert!(formatted.contains("Table 2 (page 2)"));
        assert!(formatted.contains("| Col1 | Col2 |"));
        assert!(formatted.contains("| X | Y |"));
    }

    #[test]
    fn test_format_extraction_result_empty_content() {
        let result = KreuzbergResult {
            content: String::new(),
            mime_type: "text/plain".to_string(),
            metadata: crate::Metadata::default(),
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: None,
            pages: None,
            elements: None,
            djot_content: None,
        };

        let formatted = format_extraction_result(&result);

        assert!(formatted.contains("Content (0 characters)"));
        assert!(formatted.contains("Metadata:"));
    }

    #[test]
    fn test_format_extraction_result_no_tables() {
        let result = KreuzbergResult {
            content: "Simple text".to_string(),
            mime_type: "text/plain".to_string(),
            metadata: crate::Metadata::default(),
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: None,
            pages: None,
            elements: None,
            djot_content: None,
        };

        let formatted = format_extraction_result(&result);

        assert!(formatted.contains("Simple text"));
        assert!(!formatted.contains("Tables"));
    }
}
