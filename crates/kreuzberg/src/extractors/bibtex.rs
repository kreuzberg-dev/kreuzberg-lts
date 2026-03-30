//! BibTeX bibliography extractor.
//!
//! Extracts and parses BibTeX bibliography files, providing structured access
//! to bibliography entries, metadata, and author information.

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::{ExtractionResult, Metadata};
use ahash::AHashMap;
use async_trait::async_trait;
use std::borrow::Cow;
use std::collections::HashSet;

#[cfg(feature = "office")]
use biblatex::{Bibliography, ChunksExt};

/// BibTeX bibliography extractor.
///
/// Parses BibTeX files and extracts structured bibliography data including
/// entries, authors, publication years, and entry type distribution.
pub struct BibtexExtractor;

impl BibtexExtractor {
    /// Create a new BibTeX extractor.
    pub fn new() -> Self {
        Self
    }
}

impl Default for BibtexExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for BibtexExtractor {
    fn name(&self) -> &str {
        "bibtex-extractor"
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    fn description(&self) -> &str {
        "Extracts and parses BibTeX bibliography files with structured metadata"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg(feature = "office")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for BibtexExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let bibtex_str = String::from_utf8_lossy(content);
        let wants_structure = config.include_document_structure;

        let mut entries_vec = Vec::new();
        let mut authors_set = HashSet::new();
        let mut years_set = HashSet::new();
        let mut entry_types_map: AHashMap<String, i32> = AHashMap::new();
        let mut formatted_entries = String::new();
        let mut citation_pairs: Vec<(String, String, AHashMap<String, String>)> = Vec::new();

        match Bibliography::parse(&bibtex_str) {
            Ok(bib) => {
                for entry in bib.iter() {
                    let key = entry.key.clone();
                    let entry_type = entry.entry_type.clone();

                    // Track start position for document structure citation text
                    let entry_start = formatted_entries.len();

                    // Collect all entry fields as attributes
                    let mut entry_fields: AHashMap<String, String> = AHashMap::new();
                    entry_fields.insert("entry_type".to_string(), entry_type.to_string());

                    // Format as @type{key, with key on the same line
                    formatted_entries.push_str(&format!("@{}{{{},\n", entry_type, key));

                    for (field_name, field_chunks) in &entry.fields {
                        let field_text = field_chunks.format_verbatim();
                        formatted_entries.push_str(&format!("  {} = {{{}}},\n", field_name, field_text));

                        // Store every field as an attribute
                        entry_fields.insert(field_name.to_lowercase(), field_text.clone());

                        if field_name.to_lowercase() == "author" {
                            for author in field_text.split(" and ") {
                                let trimmed_author = author.trim().to_string();
                                if !trimmed_author.is_empty() {
                                    authors_set.insert(trimmed_author);
                                }
                            }
                        }

                        if field_name.to_lowercase() == "year"
                            && let Ok(year) = field_text.parse::<u32>()
                        {
                            years_set.insert(year);
                        }
                    }

                    formatted_entries.push_str("}\n\n");

                    // Reuse the already-formatted entry text for the citation node
                    if wants_structure {
                        let citation_text = formatted_entries[entry_start..].trim().to_string();
                        citation_pairs.push((key.clone(), citation_text, entry_fields.clone()));
                    }

                    // Store per-entry fields in additional metadata
                    let fields_json: serde_json::Map<String, serde_json::Value> = entry_fields
                        .iter()
                        .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                        .collect();
                    // Only add non-empty field maps
                    if !fields_json.is_empty() {
                        // We'll aggregate these into "entries" below
                    }

                    *entry_types_map
                        .entry(entry_type.to_string().to_lowercase())
                        .or_insert(0) += 1;

                    entries_vec.push((key, fields_json));
                }
            }
            Err(_err) => {
                #[cfg(feature = "otel")]
                tracing::warn!("BibTeX parsing failed, returning raw content: {}", _err);
                formatted_entries = bibtex_str.to_string();
            }
        }

        let mut additional: AHashMap<Cow<'static, str>, serde_json::Value> = AHashMap::new();

        additional.insert(Cow::Borrowed("entry_count"), serde_json::json!(entries_vec.len()));

        // Collect citation keys (just the key strings)
        let citation_keys: Vec<&str> = entries_vec.iter().map(|(k, _)| k.as_str()).collect();
        additional.insert(Cow::Borrowed("citation_keys"), serde_json::json!(citation_keys));

        // Store per-entry field maps as structured metadata
        let entries_metadata: Vec<serde_json::Value> = entries_vec
            .iter()
            .map(|(key, fields)| {
                let mut entry_obj = serde_json::Map::new();
                entry_obj.insert("key".to_string(), serde_json::json!(key));
                for (k, v) in fields {
                    entry_obj.insert(k.clone(), v.clone());
                }
                serde_json::Value::Object(entry_obj)
            })
            .collect();
        additional.insert(Cow::Borrowed("entries"), serde_json::json!(entries_metadata));

        let mut authors_list: Vec<String> = authors_set.into_iter().collect();
        authors_list.sort();
        additional.insert(Cow::Borrowed("authors"), serde_json::json!(authors_list));

        if !years_set.is_empty() {
            let min_year = years_set.iter().min().copied().unwrap_or(0);
            let max_year = years_set.iter().max().copied().unwrap_or(0);
            additional.insert(
                Cow::Borrowed("year_range"),
                serde_json::json!({
                    "min": min_year,
                    "max": max_year,
                    "years": years_set.into_iter().collect::<Vec<_>>()
                }),
            );
        }

        if !entry_types_map.is_empty() {
            let mut entry_types_json = serde_json::json!({});
            for (entry_type, count) in entry_types_map {
                entry_types_json[entry_type] = serde_json::json!(count);
            }
            additional.insert(Cow::Borrowed("entry_types"), entry_types_json);
        }

        let document = if wants_structure && !citation_pairs.is_empty() {
            use crate::types::builder::DocumentStructureBuilder;
            let mut builder = DocumentStructureBuilder::new().source_format("bibtex");
            for (key, citation_text, fields) in citation_pairs {
                let node_idx = builder.push_citation(&key, &citation_text, None);
                // Set all entry fields as node attributes
                if !fields.is_empty() {
                    builder.set_attributes(node_idx, fields);
                }
            }
            Some(builder.build())
        } else {
            None
        };

        Ok(ExtractionResult {
            content: formatted_entries,
            mime_type: mime_type.to_string().into(),
            metadata: Metadata {
                additional,
                ..Default::default()
            },
            pages: None,
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: None,
            djot_content: None,
            elements: None,
            ocr_elements: None,
            document,
            #[cfg(any(feature = "keywords-yake", feature = "keywords-rake"))]
            extracted_keywords: None,
            quality_score: None,
            processing_warnings: Vec::new(),
            annotations: None,
            children: None,
        })
    }

    fn supported_mime_types(&self) -> &[&str] {
        &["application/x-bibtex", "text/x-bibtex", "application/x-biblatex"]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(all(test, feature = "office"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_can_extract_bibtex_mime_types() {
        let extractor = BibtexExtractor::new();
        let supported = extractor.supported_mime_types();

        assert!(supported.contains(&"application/x-bibtex"));
        assert!(supported.contains(&"text/x-bibtex"));
        assert!(supported.contains(&"application/x-biblatex"));
        assert_eq!(supported.len(), 3);
    }

    #[tokio::test]
    async fn test_extract_simple_bibtex() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@article{key2023,
    title = {Sample Title},
    author = {John Doe},
    year = {2023}
}"#;

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await;

        assert!(result.is_ok());
        let result = result.expect("Should extract valid BibTeX entry");

        assert!(result.content.contains("@article"));
        assert!(result.content.contains("key2023"));
        assert!(result.content.contains("Sample Title"));

        let metadata = &result.metadata;
        assert_eq!(
            metadata.additional.get(&Cow::Borrowed("entry_count")),
            Some(&serde_json::json!(1))
        );
    }

    #[tokio::test]
    async fn test_extract_multiple_entries() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@article{first2020,
    title = {First Paper},
    author = {Author One},
    year = {2020},
    journal = {Test Journal}
}

@book{second2021,
    title = {Test Book},
    author = {Author Two},
    year = {2021},
    publisher = {Test Publisher}
}

@inproceedings{third2022,
    title = {Conference Paper},
    author = {Author Three},
    year = {2022}
}"#;

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await;

        assert!(result.is_ok());
        let result = result.expect("Should extract valid BibTeX entries");

        let metadata = &result.metadata;

        assert_eq!(
            metadata.additional.get(&Cow::Borrowed("entry_count")),
            Some(&serde_json::json!(3))
        );

        if let Some(keys) = metadata.additional.get(&Cow::Borrowed("citation_keys"))
            && let Some(keys_array) = keys.as_array()
        {
            assert_eq!(keys_array.len(), 3);
        }

        if let Some(types) = metadata.additional.get(&Cow::Borrowed("entry_types")) {
            assert!(types.get("article").is_some());
            assert!(types.get("book").is_some());
            assert!(types.get("inproceedings").is_some());
        }
    }

    #[tokio::test]
    async fn test_extract_article_entry() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@article{einstein1905,
    author = {Albert Einstein},
    title = {On the Electrodynamics of Moving Bodies},
    journal = {Annalen der Physik},
    year = {1905},
    volume = {17},
    pages = {891-921}
}"#;

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await;

        assert!(result.is_ok());
        let result = result.expect("Should extract valid article entry");

        assert!(result.content.contains("@article"));
        assert!(result.content.contains("einstein1905"));
        assert!(result.content.contains("On the Electrodynamics of Moving Bodies"));
        assert!(result.content.contains("Annalen der Physik"));

        let metadata = &result.metadata;
        if let Some(authors) = metadata.additional.get("authors")
            && let Some(authors_array) = authors.as_array()
        {
            assert!(!authors_array.is_empty());
            assert!(authors_array[0].as_str().unwrap_or("").contains("Einstein"));
        }
    }

    #[tokio::test]
    async fn test_extract_book_entry() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@book{knuth1984,
    author = {Donald E. Knuth},
    title = {The TeXbook},
    publisher = {Addison-Wesley},
    year = {1984}
}"#;

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await;

        assert!(result.is_ok());
        let result = result.expect("Should extract valid book entry");

        assert!(result.content.contains("@book"));
        assert!(result.content.contains("knuth1984"));
        assert!(result.content.contains("The TeXbook"));

        let metadata = &result.metadata;
        assert_eq!(
            metadata.additional.get(&Cow::Borrowed("entry_count")),
            Some(&serde_json::json!(1))
        );

        if let Some(year_range) = metadata.additional.get("year_range") {
            assert_eq!(year_range.get("min"), Some(&serde_json::json!(1984)));
            assert_eq!(year_range.get("max"), Some(&serde_json::json!(1984)));
        }
    }

    #[tokio::test]
    async fn test_extract_metadata() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@article{paper1,
    author = {Alice Smith and Bob Jones},
    title = {Title 1},
    year = {2020}
}

@article{paper2,
    author = {Charlie Brown},
    title = {Title 2},
    year = {2021}
}

@book{book1,
    author = {David Lee},
    title = {Book Title},
    year = {2019}
}"#;

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await;

        assert!(result.is_ok());
        let result = result.expect("Should extract valid metadata");
        let metadata = &result.metadata;

        assert_eq!(
            metadata.additional.get(&Cow::Borrowed("entry_count")),
            Some(&serde_json::json!(3))
        );

        if let Some(authors) = metadata.additional.get("authors")
            && let Some(authors_array) = authors.as_array()
        {
            assert!(authors_array.len() >= 4);
        }

        if let Some(year_range) = metadata.additional.get("year_range") {
            assert_eq!(year_range.get("min"), Some(&serde_json::json!(2019)));
            assert_eq!(year_range.get("max"), Some(&serde_json::json!(2021)));
        }

        if let Some(types) = metadata.additional.get(&Cow::Borrowed("entry_types")) {
            assert_eq!(types.get("article"), Some(&serde_json::json!(2)));
            assert_eq!(types.get("book"), Some(&serde_json::json!(1)));
        }
    }

    #[tokio::test]
    async fn test_empty_bibliography() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = b"";

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await;

        assert!(result.is_ok());
        let result = result.expect("Should extract empty bibliography");
        let metadata = &result.metadata;

        assert_eq!(
            metadata.additional.get(&Cow::Borrowed("entry_count")),
            Some(&serde_json::json!(0))
        );
    }

    #[tokio::test]
    async fn test_malformed_entry() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@article{incomplete
    title = {Missing fields}

Some random text that's not valid BibTeX"#;

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await;

        assert!(result.is_ok());
        let result = result.expect("Should extract malformed entry as raw content");

        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_authors_extraction() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@article{collab2022,
    author = {First Author and Second Author and Third Author},
    title = {Collaborative Work},
    year = {2022}
}"#;

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await;

        assert!(result.is_ok());
        let result = result.expect("Should extract multiple authors");
        let metadata = &result.metadata;

        if let Some(authors) = metadata.additional.get("authors")
            && let Some(authors_array) = authors.as_array()
        {
            assert!(authors_array.len() >= 3);
        }
    }

    #[tokio::test]
    async fn test_bibtex_extractor_plugin_interface() {
        let extractor = BibtexExtractor::new();
        assert_eq!(extractor.name(), "bibtex-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(extractor.priority(), 50);
        assert!(!extractor.supported_mime_types().is_empty());
    }

    #[test]
    fn test_bibtex_extractor_default() {
        let extractor = BibtexExtractor;
        assert_eq!(extractor.name(), "bibtex-extractor");
    }

    #[tokio::test]
    async fn test_bibtex_extractor_initialize_shutdown() {
        let extractor = BibtexExtractor::new();
        assert!(extractor.initialize().is_ok());
        assert!(extractor.shutdown().is_ok());
    }

    #[tokio::test]
    async fn test_bibtex_entry_fields_extraction() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@article{einstein1905,
    author = {Albert Einstein},
    title = {On the Electrodynamics of Moving Bodies},
    journal = {Annalen der Physik},
    year = {1905},
    volume = {17},
    pages = {891-921},
    doi = {10.1002/andp.19053220806},
    publisher = {Wiley}
}"#;

        let config = ExtractionConfig::default();
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await
            .expect("Should extract entry fields");

        let metadata = &result.metadata;

        // Check that entries metadata contains all fields
        let entries = metadata.additional.get(&Cow::Borrowed("entries"));
        assert!(entries.is_some(), "Should have entries metadata");
        let entries_array = entries
            .expect("entries key should be present")
            .as_array()
            .expect("entries should be an array");
        assert_eq!(entries_array.len(), 1);

        let entry = &entries_array[0];
        assert_eq!(
            entry
                .get("key")
                .expect("key field")
                .as_str()
                .expect("key should be string"),
            "einstein1905"
        );
        assert_eq!(
            entry
                .get("entry_type")
                .expect("entry_type field")
                .as_str()
                .expect("entry_type should be string"),
            "article"
        );
        assert!(entry.get("journal").is_some(), "Should have journal field");
        assert!(entry.get("volume").is_some(), "Should have volume field");
        assert!(entry.get("pages").is_some(), "Should have pages field");
        assert!(entry.get("doi").is_some(), "Should have doi field");
        assert!(entry.get("publisher").is_some(), "Should have publisher field");
    }

    #[tokio::test]
    async fn test_bibtex_document_structure_attributes() {
        let extractor = BibtexExtractor::new();
        let bibtex_content = br#"@book{knuth1984,
    author = {Donald E. Knuth},
    title = {The TeXbook},
    publisher = {Addison-Wesley},
    year = {1984},
    isbn = {0-201-13447-0}
}"#;

        let config = ExtractionConfig {
            include_document_structure: true,
            ..Default::default()
        };
        let result = extractor
            .extract_bytes(bibtex_content, "application/x-bibtex", &config)
            .await
            .expect("Should extract with document structure");

        assert!(result.document.is_some(), "Should have document structure");
        let doc = result.document.expect("document structure should be present");
        // The document should have citation nodes with attributes
        assert!(!doc.nodes.is_empty(), "Document should have nodes");
    }
}
