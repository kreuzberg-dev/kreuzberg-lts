//! Email message extractor.

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::extractors::SyncExtractor;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::{EmailMetadata, ExtractionResult, Metadata};
use ahash::AHashMap;
use async_trait::async_trait;
use std::borrow::Cow;
#[cfg(feature = "tokio-runtime")]
use std::path::Path;

/// Email message extractor.
///
/// Supports: .eml, .msg
pub struct EmailExtractor;

impl Default for EmailExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for EmailExtractor {
    fn name(&self) -> &str {
        "email-extractor"
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
}

impl SyncExtractor for EmailExtractor {
    fn extract_sync(&self, content: &[u8], mime_type: &str, config: &ExtractionConfig) -> Result<ExtractionResult> {
        let fallback_codepage = config.email.as_ref().and_then(|e| e.msg_fallback_codepage);
        let email_result = crate::extraction::email::extract_email_content(content, mime_type, fallback_codepage)?;

        let text = crate::extraction::email::build_email_text_output(&email_result);

        let attachment_names: Vec<String> = email_result
            .attachments
            .iter()
            .filter_map(|att| att.filename.clone().or_else(|| att.name.clone()))
            .collect();

        // Filter out keys already represented in EmailMetadata to avoid
        // flattened field conflicts (e.g. "attachments" as string vs Vec).
        const EMAIL_STRUCT_KEYS: &[&str] = &[
            "from_email",
            "from_name",
            "to_emails",
            "cc_emails",
            "bcc_emails",
            "message_id",
            "attachments",
            "subject",
            "date",
            "email_from",
            "email_to",
            "email_cc",
            "email_bcc",
        ];
        let mut additional = AHashMap::new();
        for (key, value) in &email_result.metadata {
            if !EMAIL_STRUCT_KEYS.contains(&key.as_str()) {
                additional.insert(Cow::Owned(key.clone()), serde_json::json!(value));
            }
        }

        // Build document structure while email_result is still fully owned,
        // borrowing fields that will be moved into EmailMetadata/Metadata afterward.
        let document = if config.include_document_structure {
            use crate::types::builder::DocumentStructureBuilder;
            let mut builder = DocumentStructureBuilder::new().source_format("email");

            // Push email headers as a metadata block
            let mut header_entries = Vec::new();
            if let Some(ref subject) = email_result.subject {
                header_entries.push(("Subject".to_string(), subject.clone()));
            }
            if let Some(ref from) = email_result.from_email {
                header_entries.push(("From".to_string(), from.clone()));
            }
            if !email_result.to_emails.is_empty() {
                header_entries.push(("To".to_string(), email_result.to_emails.join(", ")));
            }
            if !email_result.cc_emails.is_empty() {
                header_entries.push(("CC".to_string(), email_result.cc_emails.join(", ")));
            }
            if let Some(ref date) = email_result.date {
                header_entries.push(("Date".to_string(), date.clone()));
            }
            if !header_entries.is_empty() {
                builder.push_metadata_block(header_entries, None);
            }

            // Push body content: if HTML body is available, use the HTML
            // structure walker for rich annotations (bold, italic, links, etc.);
            // otherwise fall back to plain text paragraph splitting.
            if let Some(ref html) = email_result.html_content {
                let html_doc = crate::extraction::html::structure::build_document_structure(html);
                // Merge HTML structure nodes into the email builder.
                for node in &html_doc.nodes {
                    // Only merge root-level body nodes (skip the HTML wrapper structure).
                    if node.parent.is_none() {
                        match &node.content {
                            crate::types::NodeContent::Paragraph { text } => {
                                let trimmed = text.trim();
                                if !trimmed.is_empty() {
                                    builder.push_paragraph(trimmed, node.annotations.clone(), None, None);
                                }
                            }
                            crate::types::NodeContent::Heading { level, text } => {
                                builder.push_heading(*level, text.as_str(), None, None);
                            }
                            crate::types::NodeContent::List { ordered } => {
                                let list_idx = builder.push_list(*ordered, None);
                                // Collect list item children from the HTML doc
                                for &child_idx in &node.children {
                                    if let Some(child) = html_doc.nodes.get(child_idx.0 as usize)
                                        && let crate::types::NodeContent::ListItem { text } = &child.content
                                    {
                                        builder.push_list_item(list_idx, text.as_str(), None);
                                    }
                                }
                            }
                            crate::types::NodeContent::Code { text, language } => {
                                builder.push_code(text.as_str(), language.as_deref(), None);
                            }
                            _ => {
                                // For other node types, extract text if available
                                // and push as paragraph
                                if let Some(text) = node.content.text() {
                                    let trimmed = text.trim();
                                    if !trimmed.is_empty() {
                                        builder.push_paragraph(trimmed, node.annotations.clone(), None, None);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                for paragraph in email_result.cleaned_text.split("\n\n") {
                    let trimmed = paragraph.trim();
                    if !trimmed.is_empty() {
                        builder.push_paragraph(trimmed, vec![], None, None);
                    }
                }
            }
            Some(builder.build())
        } else {
            None
        };

        // Move fields out of email_result now that all borrows above are complete.
        let subject = email_result.subject;
        let created_at = email_result.date;
        let email_metadata = EmailMetadata {
            from_email: email_result.from_email,
            from_name: None,
            to_emails: email_result.to_emails,
            cc_emails: email_result.cc_emails,
            bcc_emails: email_result.bcc_emails,
            message_id: email_result.message_id,
            attachments: attachment_names,
        };

        Ok(ExtractionResult {
            content: text,
            mime_type: mime_type.to_string().into(),
            metadata: Metadata {
                format: Some(crate::types::FormatMetadata::Email(email_metadata)),
                subject,
                created_at,
                additional,
                ..Default::default()
            },
            tables: vec![],
            detected_languages: None,
            chunks: None,
            images: None,
            pages: None,
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
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for EmailExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        self.extract_sync(content, mime_type, config)
    }

    #[cfg(feature = "tokio-runtime")]
    async fn extract_file(&self, path: &Path, mime_type: &str, config: &ExtractionConfig) -> Result<ExtractionResult> {
        let bytes = tokio::fs::read(path).await?;
        self.extract_bytes(&bytes, mime_type, config).await
    }

    fn supported_mime_types(&self) -> &[&str] {
        &["message/rfc822", "application/vnd.ms-outlook"]
    }

    fn priority(&self) -> i32 {
        50
    }

    fn as_sync_extractor(&self) -> Option<&dyn crate::extractors::SyncExtractor> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_extractor_plugin_interface() {
        let extractor = EmailExtractor::new();
        assert_eq!(extractor.name(), "email-extractor");
        assert!(extractor.initialize().is_ok());
        assert!(extractor.shutdown().is_ok());
    }

    #[test]
    fn test_email_extractor_supported_mime_types() {
        let extractor = EmailExtractor::new();
        let mime_types = extractor.supported_mime_types();
        assert_eq!(mime_types.len(), 2);
        assert!(mime_types.contains(&"message/rfc822"));
        assert!(mime_types.contains(&"application/vnd.ms-outlook"));
    }

    #[test]
    fn test_email_extractor_uses_config() {
        use crate::core::config::EmailConfig;

        // Extractor with email config set should not panic or error on invalid data
        let config = ExtractionConfig {
            email: Some(EmailConfig {
                msg_fallback_codepage: Some(1251),
            }),
            ..Default::default()
        };
        let extractor = EmailExtractor::new();
        // Empty data returns a validation error — config is still used without panic
        let result = extractor.extract_sync(b"", "application/vnd.ms-outlook", &config);
        assert!(result.is_err());
    }
}
