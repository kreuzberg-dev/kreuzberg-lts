//! Feature processing logic.
//!
//! This module handles feature-specific processing like chunking,
//! embedding generation, and language detection.

use crate::Result;
use crate::core::config::ExtractionConfig;
#[cfg(feature = "chunking")]
use crate::types::PageBoundary;
use crate::types::{ExtractionResult, ProcessingWarning};
use std::borrow::Cow;

/// Recompute page boundaries against the rendered `content` string.
///
/// `PageBoundary` offsets produced during extraction are computed against raw
/// pdfium/source text, but `result.content` is produced by `render_plain` which
/// may have different byte lengths (e.g. normalised whitespace, Unicode
/// normalisation, dropped control characters).  This function re-derives the
/// boundaries by locating each page's rendered content inside the combined
/// `content` string, so that the byte offsets passed to the chunker are valid
/// indices into `result.content`.
///
/// Pages whose content cannot be found are silently skipped (the chunker will
/// still produce output, just without page-range metadata for those pages).
#[cfg(feature = "chunking")]
fn recompute_boundaries_from_pages(content: &str, pages: &[crate::types::PageContent]) -> Vec<PageBoundary> {
    let mut boundaries = Vec::with_capacity(pages.len());
    let mut search_offset = 0usize;

    for page in pages {
        if page.content.trim().is_empty() {
            boundaries.push(PageBoundary {
                page_number: page.page_number,
                byte_start: search_offset,
                byte_end: search_offset,
            });
            continue;
        }

        let normalized: String = page
            .content
            .split("\n\n")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");

        if let Some(pos) = content[search_offset..].find(normalized.as_str()) {
            let byte_start = search_offset + pos;
            let byte_end = content.floor_char_boundary(byte_start + normalized.len());
            boundaries.push(PageBoundary {
                page_number: page.page_number,
                byte_start,
                byte_end,
            });
            search_offset = byte_end;
            continue;
        }

        if let Some(line) = page.content.lines().find(|l| !l.trim().is_empty()).map(|l| l.trim())
            && let Some(pos) = content[search_offset..].find(line)
        {
            let byte_start = search_offset + pos;
            let raw_end = (byte_start + normalized.len()).min(content.len());
            let byte_end = content.floor_char_boundary(raw_end);
            boundaries.push(PageBoundary {
                page_number: page.page_number,
                byte_start,
                byte_end,
            });
            search_offset = byte_end;
            continue;
        }

        tracing::debug!(
            page = page.page_number,
            "Could not locate page content in rendered text — skipping page boundary"
        );
    }

    boundaries
}

/// Map TSLP `CodeChunk`s directly to kreuzberg `Chunk`s, bypassing text-splitter.
///
/// When the extraction result contains code intelligence with non-empty chunks,
/// those chunks already represent semantically meaningful code boundaries produced
/// by tree-sitter. Using text-splitter would break these boundaries.
#[cfg(feature = "tree-sitter")]
fn try_code_chunks(result: &ExtractionResult) -> Option<Vec<crate::types::extraction::Chunk>> {
    use crate::types::extraction::{Chunk, ChunkMetadata, ChunkType, HeadingContext, HeadingLevel};

    let code_chunks = match &result.metadata.format {
        Some(crate::types::metadata::FormatMetadata::Code(pr)) if !pr.chunks.is_empty() => &pr.chunks,
        _ => return None,
    };

    let total_chunks = code_chunks.len();
    let chunks: Vec<Chunk> = code_chunks
        .iter()
        .enumerate()
        .map(|(i, cc)| {
            let chunk_type = ChunkType::CodeBlock;

            let heading_context = if cc.metadata.context_path.is_empty() {
                None
            } else {
                Some(HeadingContext {
                    headings: cc
                        .metadata
                        .context_path
                        .iter()
                        .enumerate()
                        .map(|(depth, name)| HeadingLevel {
                            level: (depth as u8).saturating_add(2).min(6),
                            text: name.clone(),
                        })
                        .collect(),
                })
            };

            Chunk {
                content: cc.content.clone(),
                chunk_type,
                embedding: None,
                metadata: ChunkMetadata {
                    byte_start: cc.start_byte,
                    byte_end: cc.end_byte,
                    token_count: None,
                    chunk_index: i,
                    total_chunks,
                    first_page: None,
                    last_page: None,
                    heading_context,
                },
            }
        })
        .collect();

    Some(chunks)
}

/// Execute chunking if configured.
pub(super) fn execute_chunking(result: &mut ExtractionResult, config: &ExtractionConfig) -> Result<()> {
    #[cfg(feature = "chunking")]
    if let Some(ref chunking_config) = config.chunking {
        #[cfg(feature = "tree-sitter")]
        if let Some(code_chunks) = try_code_chunks(result) {
            result.chunks = Some(code_chunks);

            let resolved_config = chunking_config.resolve_preset();
            #[cfg(feature = "embeddings")]
            if let Some(ref embedding_config) = resolved_config.embedding
                && let Some(ref mut chunks) = result.chunks
                && let Err(e) = crate::embeddings::generate_embeddings_for_chunks(chunks, embedding_config)
            {
                tracing::warn!("Embedding generation failed: {e}. Check that ONNX Runtime is installed.");
                result.processing_warnings.push(ProcessingWarning {
                    source: Cow::Borrowed("embedding"),
                    message: Cow::Owned(e.to_string()),
                });
            }

            #[cfg(not(feature = "embeddings"))]
            if resolved_config.embedding.is_some() {
                tracing::warn!(
                    "Embedding config provided but embeddings feature is not enabled. Recompile with --features embeddings."
                );
                result.processing_warnings.push(ProcessingWarning {
                    source: Cow::Borrowed("embedding"),
                    message: Cow::Borrowed("Embeddings feature not enabled"),
                });
            }

            return Ok(());
        }

        let resolved_config = chunking_config.resolve_preset();
        let chunking_config = &resolved_config;

        let (chunk_input, heading_source) = if config.output_format != crate::core::config::OutputFormat::Plain {
            (
                result.formatted_content.as_deref().unwrap_or(result.content.as_str()),
                None,
            )
        } else {
            (result.content.as_str(), result.formatted_content.as_deref())
        };

        let recomputed_boundaries: Option<Vec<PageBoundary>> = result
            .pages
            .as_deref()
            .map(|pages| recompute_boundaries_from_pages(chunk_input, pages))
            .filter(|boundaries| !boundaries.is_empty());

        let page_boundaries: Option<&[PageBoundary]> = recomputed_boundaries
            .as_deref()
            .or_else(|| result.metadata.pages.as_ref().and_then(|ps| ps.boundaries.as_deref()));

        match crate::chunking::chunk_text_with_heading_source(
            chunk_input,
            chunking_config,
            page_boundaries,
            heading_source,
        ) {
            Ok(chunking_result) => {
                result.chunks = Some(chunking_result.chunks);

                #[cfg(feature = "embeddings")]
                if let Some(ref embedding_config) = chunking_config.embedding
                    && let Some(ref mut chunks) = result.chunks
                    && let Err(e) = crate::embeddings::generate_embeddings_for_chunks(chunks, embedding_config)
                {
                    tracing::warn!("Embedding generation failed: {e}. Check that ONNX Runtime is installed.");
                    result.processing_warnings.push(ProcessingWarning {
                        source: Cow::Borrowed("embedding"),
                        message: Cow::Owned(e.to_string()),
                    });
                }

                #[cfg(not(feature = "embeddings"))]
                if chunking_config.embedding.is_some() {
                    tracing::warn!(
                        "Embedding config provided but embeddings feature is not enabled. Recompile with --features embeddings."
                    );
                    result.processing_warnings.push(ProcessingWarning {
                        source: Cow::Borrowed("embedding"),
                        message: Cow::Borrowed("Embeddings feature not enabled"),
                    });
                }
            }
            Err(e) => {
                result.processing_warnings.push(ProcessingWarning {
                    source: Cow::Borrowed("chunking"),
                    message: Cow::Owned(e.to_string()),
                });
            }
        }
    }

    #[cfg(not(feature = "chunking"))]
    if config.chunking.is_some() {
        result.processing_warnings.push(ProcessingWarning {
            source: Cow::Borrowed("chunking"),
            message: Cow::Borrowed("Chunking feature not enabled"),
        });
    }

    Ok(())
}

/// Execute language detection if configured.
pub(super) fn execute_language_detection(result: &mut ExtractionResult, config: &ExtractionConfig) -> Result<()> {
    #[cfg(feature = "language-detection")]
    if let Some(ref lang_config) = config.language_detection {
        match crate::language_detection::detect_languages(&result.content, lang_config) {
            Ok(detected) => {
                result.detected_languages = detected;
            }
            Err(e) => {
                result.processing_warnings.push(ProcessingWarning {
                    source: Cow::Borrowed("language_detection"),
                    message: Cow::Owned(e.to_string()),
                });
            }
        }
    }

    #[cfg(not(feature = "language-detection"))]
    if config.language_detection.is_some() {
        result.processing_warnings.push(ProcessingWarning {
            source: Cow::Borrowed("language_detection"),
            message: Cow::Borrowed("Language detection feature not enabled"),
        });
    }

    Ok(())
}

/// Execute token reduction if configured.
pub(super) fn execute_token_reduction(result: &mut ExtractionResult, config: &ExtractionConfig) -> Result<()> {
    #[cfg(feature = "quality")]
    if let Some(ref tr_config) = config.token_reduction {
        let level = crate::text::token_reduction::ReductionLevel::from(tr_config.mode.as_str());

        if !matches!(level, crate::text::token_reduction::ReductionLevel::Off) {
            let impl_config = crate::text::token_reduction::TokenReductionConfig {
                level,
                ..Default::default()
            };

            let lang_hint: Option<&str> = result
                .detected_languages
                .as_deref()
                .and_then(|langs| langs.first().map(|s| s.as_str()));

            match crate::text::token_reduction::reduce_tokens(&result.content, &impl_config, lang_hint) {
                Ok(reduced) => {
                    result.content = reduced;
                }
                Err(e) => {
                    result.processing_warnings.push(ProcessingWarning {
                        source: Cow::Borrowed("token_reduction"),
                        message: Cow::Owned(e.to_string()),
                    });
                }
            }
        }
    }

    #[cfg(not(feature = "quality"))]
    if config.token_reduction.is_some() {
        result.processing_warnings.push(ProcessingWarning {
            source: Cow::Borrowed("token_reduction"),
            message: Cow::Borrowed("Token reduction requires the quality feature"),
        });
    }

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "chunking")]
mod tests {
    use super::*;
    use crate::core::config::{ChunkerType, ChunkingConfig, OutputFormat};
    use crate::types::PageContent;

    fn make_page(page_number: usize, content: &str) -> PageContent {
        PageContent {
            page_number,
            content: content.to_string(),
            tables: Vec::new(),
            images: Vec::new(),
            hierarchy: None,
            is_blank: None,
            layout_regions: None,
        }
    }

    fn markdown_chunking_config() -> ExtractionConfig {
        ExtractionConfig {
            output_format: OutputFormat::Markdown,
            chunking: Some(ChunkingConfig {
                max_characters: 2000,
                overlap: 0,
                trim: true,
                chunker_type: ChunkerType::Markdown,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn chunks_content_is_markdown_when_output_format_is_markdown() {
        let mut result = ExtractionResult {
            content: "SH-001 Luca Bianchi Common Germany 3500000".to_string(),
            formatted_content: Some("| SH-001 | Luca Bianchi | Common | Germany | 3,500,000 |".to_string()),
            mime_type: Cow::Borrowed("application/pdf"),
            ..Default::default()
        };

        execute_chunking(&mut result, &markdown_chunking_config()).unwrap();

        let chunks = result.chunks.expect("chunks must be populated");
        assert!(!chunks.is_empty());
        assert!(chunks.iter().any(|chunk| chunk.content.contains('|')));
        assert!(chunks.iter().all(|chunk| !chunk.content.starts_with("SH-001 Luca")));
        assert!(result.formatted_content.is_some());
    }

    #[test]
    fn markdown_chunks_preserve_page_metadata_when_formatted_pages_match() {
        let mut result = ExtractionResult {
            content: "Page one text\n\nPage two text".to_string(),
            formatted_content: Some("# Page one\n\nPage one text\n\n# Page two\n\nPage two text".to_string()),
            pages: Some(vec![make_page(1, "Page one text"), make_page(2, "Page two text")]),
            mime_type: Cow::Borrowed("application/pdf"),
            ..Default::default()
        };

        execute_chunking(&mut result, &markdown_chunking_config()).unwrap();

        let chunks = result.chunks.expect("chunks must be populated");
        assert!(!chunks.is_empty());
        assert!(chunks.iter().any(|chunk| chunk.metadata.first_page.is_some()));
        assert!(chunks.iter().any(|chunk| chunk.metadata.last_page.is_some()));
    }

    #[test]
    fn recompute_boundaries_trailing_space_pages_all_resolve() {
        let p1_raw = "Heading \n\nBody paragraph one. ";
        let p2_raw = "Second heading \n\nBody paragraph two. ";
        let p3_raw = "Conclusion. ";
        let p1_norm = "Heading\n\nBody paragraph one.";
        let p2_norm = "Second heading\n\nBody paragraph two.";
        let p3_norm = "Conclusion.";
        let content = format!("{p1_norm}\n\n{p2_norm}\n\n{p3_norm}");

        let pages = vec![make_page(1, p1_raw), make_page(2, p2_raw), make_page(3, p3_raw)];
        let boundaries = recompute_boundaries_from_pages(&content, &pages);

        assert_eq!(boundaries.len(), 3);
        assert_eq!(&content[boundaries[0].byte_start..boundaries[0].byte_end], p1_norm);
        assert_eq!(&content[boundaries[1].byte_start..boundaries[1].byte_end], p2_norm);
        assert_eq!(&content[boundaries[2].byte_start..boundaries[2].byte_end], p3_norm);
    }
}
