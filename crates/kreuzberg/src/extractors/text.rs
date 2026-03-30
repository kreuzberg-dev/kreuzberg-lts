//! Plain text extractor.

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::ExtractionResult;
use async_trait::async_trait;

/// Plain text extractor.
///
/// Extracts content from plain text files (.txt).
pub struct PlainTextExtractor;

impl PlainTextExtractor {
    /// Create a new plain text extractor.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlainTextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PlainTextExtractor {
    fn name(&self) -> &str {
        "plain-text-extractor"
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
        "Extracts content from plain text files"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for PlainTextExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let text = String::from_utf8_lossy(content).into_owned();
        let text = text.trim_end_matches('\n').trim_end_matches('\r').to_string();
        let line_count = text.lines().count();
        let word_count = text.split_whitespace().count();
        let character_count = text.len();

        let document = if config.include_document_structure {
            use crate::types::builder::DocumentStructureBuilder;
            let mut builder = DocumentStructureBuilder::new().source_format("text");
            for paragraph in text.split("\n\n") {
                let trimmed = paragraph.trim();
                if !trimmed.is_empty() {
                    builder.push_paragraph(trimmed, vec![], None, None);
                }
            }
            Some(builder.build())
        } else {
            None
        };

        Ok(ExtractionResult {
            content: text,
            mime_type: mime_type.to_string().into(),
            metadata: crate::types::Metadata {
                format: Some(crate::types::FormatMetadata::Text(crate::types::TextMetadata {
                    line_count,
                    word_count,
                    character_count,
                    headers: None,
                    links: None,
                    code_blocks: None,
                })),
                ..Default::default()
            },
            pages: None,
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: None,
            elements: None,
            djot_content: None,
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
        &[
            "text/plain",
            "text/troff",
            "text/x-mdoc",
            "text/x-pod",
            "text/x-dokuwiki",
        ]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plain_text_extractor() {
        let extractor = PlainTextExtractor::new();
        let content = b"Hello, World!\nThis is a test.";
        let config = ExtractionConfig::default();

        let result = extractor.extract_bytes(content, "text/plain", &config).await.unwrap();

        assert_eq!(result.mime_type, "text/plain");
        assert!(result.content.contains("Hello, World!"));
        assert!(result.metadata.format.is_some());
        let text_meta = match result.metadata.format.as_ref().unwrap() {
            crate::types::FormatMetadata::Text(meta) => meta,
            _ => panic!("Expected Text metadata"),
        };
        assert_eq!(text_meta.line_count, 2);
        assert_eq!(text_meta.word_count, 6);
    }

    #[test]
    fn test_plain_text_plugin_interface() {
        let extractor = PlainTextExtractor::new();
        assert_eq!(extractor.name(), "plain-text-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(
            extractor.supported_mime_types(),
            &[
                "text/plain",
                "text/troff",
                "text/x-mdoc",
                "text/x-pod",
                "text/x-dokuwiki",
            ]
        );
        assert_eq!(extractor.priority(), 50);
    }
}
