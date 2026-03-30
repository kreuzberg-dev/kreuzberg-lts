//! Image extractors for various image formats.

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::extraction::image::extract_image_metadata;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::{ExtractionResult, Metadata};
use async_trait::async_trait;

/// Image extractor for various image formats.
///
/// Supports: PNG, JPEG, WebP, BMP, TIFF, GIF.
/// Extracts dimensions, format, and EXIF metadata.
/// Optionally runs OCR when configured.
/// When layout detection is also enabled, uses per-region OCR with
/// markdown formatting based on detected layout classes.
pub struct ImageExtractor;

impl ImageExtractor {
    /// Create a new image extractor.
    pub fn new() -> Self {
        Self
    }

    /// Extract text from image using OCR with optional page tracking for multi-frame TIFFs.
    #[cfg(any(feature = "ocr", feature = "ocr-wasm"))]
    async fn extract_with_ocr(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        use crate::plugins::registry::get_ocr_backend_registry;

        let ocr_config = config.ocr.as_ref().ok_or_else(|| crate::KreuzbergError::Parsing {
            message: "OCR config required for image OCR".to_string(),
            source: None,
        })?;

        let backend = {
            let registry = get_ocr_backend_registry();
            let registry = registry.read();
            registry.get(&ocr_config.backend)?
        };

        // Thread output_format from ExtractionConfig to OcrConfig
        let mut ocr_config_with_format = ocr_config.clone();
        ocr_config_with_format.output_format = Some(config.output_format);

        let ocr_result = backend.process_image(content, &ocr_config_with_format).await?;

        // Full OCR with TIFF multi-frame support (requires tiff crate)
        #[cfg(feature = "ocr")]
        {
            let ocr_text = ocr_result.content.clone();
            let ocr_extraction_result = crate::extraction::image::extract_text_from_image_with_ocr(
                content,
                mime_type,
                ocr_text,
                config.pages.as_ref(),
            )?;

            let mut result = ocr_result;
            result.content = ocr_extraction_result.content;
            result.pages = ocr_extraction_result.page_contents;

            Ok(result)
        }

        // Simplified OCR path for WASM (no TIFF multi-frame support)
        #[cfg(not(feature = "ocr"))]
        {
            let _ = mime_type;
            Ok(ocr_result)
        }
    }

    /// Extract text from image using layout detection + per-region OCR.
    ///
    /// Runs layout detection to identify document regions (headings, text,
    /// code, formulas, etc.), then OCRs each region individually and
    /// assembles the results into structured markdown.
    #[cfg(all(feature = "layout-detection", any(feature = "ocr", feature = "ocr-wasm")))]
    async fn extract_with_layout_ocr(&self, content: &[u8], config: &ExtractionConfig) -> Result<ExtractionResult> {
        use crate::layout::LayoutClass;
        use crate::plugins::registry::get_ocr_backend_registry;
        use image::ImageEncoder;
        use std::io::Cursor;

        let layout_config = config.layout.as_ref().ok_or_else(|| crate::KreuzbergError::Parsing {
            message: "Layout config required for layout-enhanced OCR".to_string(),
            source: None,
        })?;

        let ocr_config = config.ocr.as_ref().ok_or_else(|| crate::KreuzbergError::Parsing {
            message: "OCR config required for layout-enhanced OCR".to_string(),
            source: None,
        })?;

        // 1. Decode image
        let img = image::load_from_memory(content).map_err(|e| crate::KreuzbergError::Parsing {
            message: format!("Failed to decode image for layout detection: {e}"),
            source: None,
        })?;
        let rgb = img.to_rgb8();

        // 2. Run layout detection (reuse cached engine when available)
        let mut engine = crate::layout::take_or_create_engine(layout_config)
            .map_err(|e| crate::KreuzbergError::Other(format!("Layout engine init failed: {e}")))?;

        let detection = engine
            .detect(&rgb)
            .map_err(|e| crate::KreuzbergError::Other(format!("Layout detection failed: {e}")))?;

        // Return engine to cache immediately — we're done with inference
        crate::layout::return_engine(engine);

        tracing::info!(
            detections = detection.detections.len(),
            img_width = rgb.width(),
            img_height = rgb.height(),
            "Layout detection completed for image"
        );

        if detection.detections.is_empty() {
            tracing::debug!("No layout regions detected, falling back to whole-image OCR");
            return self.extract_with_ocr(content, "image/png", config).await;
        }

        // 3. Sort detections by reading order (top-to-bottom, left-to-right)
        let mut detections = detection.detections.clone();
        // Quantize y-centers into discrete rows to ensure transitive ordering.
        let row_threshold = (rgb.height() as f32 * 0.05).max(1.0);
        detections.sort_by(|a, b| {
            let ay = (a.bbox.y1 + a.bbox.y2) / 2.0;
            let by = (b.bbox.y1 + b.bbox.y2) / 2.0;
            let a_row = (ay / row_threshold) as i64;
            let b_row = (by / row_threshold) as i64;
            a_row.cmp(&b_row).then_with(|| {
                let ax = (a.bbox.x1 + a.bbox.x2) / 2.0;
                let bx = (b.bbox.x1 + b.bbox.x2) / 2.0;
                ax.total_cmp(&bx)
            })
        });

        // 4. Get OCR backend
        let backend = {
            let registry = get_ocr_backend_registry();
            let registry = registry.read();
            registry.get(&ocr_config.backend)?
        };

        // Use plain text for per-region OCR (we build markdown structure ourselves)
        let mut region_ocr_config = ocr_config.clone();
        region_ocr_config.output_format = Some(crate::core::config::OutputFormat::Plain);

        // 5. Per-region OCR + formatting
        let mut markdown_parts = Vec::new();
        let img_width = rgb.width();
        let img_height = rgb.height();

        for det in &detections {
            // Skip picture regions (OCR on an embedded image is not useful)
            if det.class == LayoutClass::Picture {
                continue;
            }

            // Crop region (clamp to image bounds)
            let x1 = (det.bbox.x1.max(0.0) as u32).min(img_width.saturating_sub(1));
            let y1 = (det.bbox.y1.max(0.0) as u32).min(img_height.saturating_sub(1));
            let x2 = (det.bbox.x2.max(0.0).ceil() as u32).min(img_width);
            let y2 = (det.bbox.y2.max(0.0).ceil() as u32).min(img_height);

            let crop_w = x2.saturating_sub(x1);
            let crop_h = y2.saturating_sub(y1);
            if crop_w < 4 || crop_h < 4 {
                continue; // Too small to OCR meaningfully
            }

            let crop = image::imageops::crop_imm(&rgb, x1, y1, crop_w, crop_h).to_image();

            // Encode crop as PNG for OCR backend
            let mut png_buf = Cursor::new(Vec::new());
            image::codecs::png::PngEncoder::new(&mut png_buf)
                .write_image(
                    crop.as_raw(),
                    crop.width(),
                    crop.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| crate::KreuzbergError::Other(format!("Failed to encode crop as PNG: {e}")))?;
            let crop_bytes = png_buf.into_inner();

            // OCR the cropped region
            let ocr_result = backend.process_image(&crop_bytes, &region_ocr_config).await?;
            let text = ocr_result.content.trim().to_string();
            if text.is_empty() {
                continue;
            }

            tracing::trace!(
                class = ?det.class,
                confidence = det.confidence,
                text_len = text.len(),
                "OCR result for layout region"
            );

            // Format based on layout class
            let formatted = match det.class {
                LayoutClass::Title => format!("# {text}"),
                LayoutClass::SectionHeader => format!("## {text}"),
                LayoutClass::Code => format!("```\n{text}\n```"),
                LayoutClass::Formula => format!("$$\n{text}\n$$"),
                LayoutClass::ListItem => format!("- {text}"),
                LayoutClass::Caption => format!("*{text}*"),
                LayoutClass::Footnote => format!("[^]: {text}"),
                LayoutClass::Table => text,
                LayoutClass::PageHeader | LayoutClass::PageFooter => continue,
                LayoutClass::CheckboxSelected => format!("- [x] {text}"),
                LayoutClass::CheckboxUnselected => format!("- [ ] {text}"),
                _ => text,
            };

            markdown_parts.push(formatted);
        }

        let content_text = markdown_parts.join("\n\n");

        Ok(ExtractionResult {
            content: content_text,
            mime_type: "image/png".to_string().into(),
            metadata: Metadata {
                output_format: Some("markdown".to_string()),
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
            document: None,
            #[cfg(any(feature = "keywords-yake", feature = "keywords-rake"))]
            extracted_keywords: None,
            quality_score: None,
            processing_warnings: Vec::new(),
            annotations: None,
            children: None,
        })
    }
}

/// Build a simple `DocumentStructure` for an image extraction result.
///
/// If OCR text is available, pushes it as a paragraph. Always pushes
/// the image itself as an `Image` node.
fn build_image_document_structure(ocr_text: Option<&str>) -> crate::types::document_structure::DocumentStructure {
    use crate::types::builder::DocumentStructureBuilder;

    let mut builder = DocumentStructureBuilder::new().source_format("image");
    if let Some(text) = ocr_text
        && !text.trim().is_empty()
    {
        builder.push_paragraph(text.trim(), vec![], None, None);
    }
    builder.push_image(None, Some(0), None, None);
    builder.build()
}

impl Default for ImageExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ImageExtractor {
    fn name(&self) -> &str {
        "image-extractor"
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
        "Extracts dimensions, format, and EXIF data from images (PNG, JPEG, WebP, BMP, TIFF, GIF)"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for ImageExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let extraction_metadata = extract_image_metadata(content)?;

        let image_metadata = crate::types::ImageMetadata {
            width: extraction_metadata.width,
            height: extraction_metadata.height,
            format: extraction_metadata.format.clone(),
            exif: extraction_metadata.exif_data,
        };

        if config.ocr.is_some() {
            // Layout-enhanced OCR: when both OCR and layout detection are configured,
            // run layout detection first, then OCR each detected region individually
            // and assemble into structured markdown.
            #[cfg(all(feature = "layout-detection", any(feature = "ocr", feature = "ocr-wasm")))]
            if config.layout.is_some() {
                match self.extract_with_layout_ocr(content, config).await {
                    Ok(mut result) => {
                        result.metadata.format = Some(crate::types::FormatMetadata::Image(image_metadata));
                        result.mime_type = mime_type.to_string().into();
                        if config.include_document_structure && result.document.is_none() {
                            result.document = Some(build_image_document_structure(Some(&result.content)));
                        }
                        return Ok(result);
                    }
                    Err(e) => {
                        tracing::warn!("Layout-enhanced OCR failed, falling back to regular OCR: {e}");
                        // Fall through to regular OCR below
                    }
                }
            }

            #[cfg(any(feature = "ocr", feature = "ocr-wasm"))]
            {
                let mut ocr_result = self.extract_with_ocr(content, mime_type, config).await?;

                ocr_result.metadata.format = Some(crate::types::FormatMetadata::Image(image_metadata));
                ocr_result.mime_type = mime_type.to_string().into();

                if config.include_document_structure && ocr_result.document.is_none() {
                    ocr_result.document = Some(build_image_document_structure(Some(&ocr_result.content)));
                }

                return Ok(ocr_result);
            }
            #[cfg(not(any(feature = "ocr", feature = "ocr-wasm")))]
            {
                let content_text = format!(
                    "Image: {} {}x{}",
                    extraction_metadata.format, extraction_metadata.width, extraction_metadata.height
                );

                let document = if config.include_document_structure {
                    Some(build_image_document_structure(None))
                } else {
                    None
                };

                return Ok(ExtractionResult {
                    content: content_text,
                    mime_type: mime_type.to_string().into(),
                    metadata: Metadata {
                        format: Some(crate::types::FormatMetadata::Image(image_metadata)),
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
                });
            }
        }

        let document = if config.include_document_structure {
            Some(build_image_document_structure(None))
        } else {
            None
        };

        Ok(ExtractionResult {
            content: format!(
                "Image: {} {}x{}",
                extraction_metadata.format, extraction_metadata.width, extraction_metadata.height
            ),
            mime_type: mime_type.to_string().into(),
            metadata: Metadata {
                format: Some(crate::types::FormatMetadata::Image(image_metadata)),
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
        &[
            "image/png",
            "image/jpeg",
            "image/jpg",
            "image/pjpeg",
            "image/webp",
            "image/bmp",
            "image/x-bmp",
            "image/x-ms-bmp",
            "image/tiff",
            "image/x-tiff",
            "image/gif",
            "image/jp2",
            "image/jpx",
            "image/jpm",
            "image/mj2",
            "image/x-jbig2",
            "image/x-portable-anymap",
            "image/x-portable-bitmap",
            "image/x-portable-graymap",
            "image/x-portable-pixmap",
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
    async fn test_image_extractor_invalid_image() {
        let extractor = ImageExtractor::new();
        let invalid_bytes = vec![0, 1, 2, 3, 4, 5];
        let config = ExtractionConfig::default();

        let result = extractor.extract_bytes(&invalid_bytes, "image/png", &config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_image_plugin_interface() {
        let extractor = ImageExtractor::new();
        assert_eq!(extractor.name(), "image-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert!(extractor.supported_mime_types().contains(&"image/png"));
        assert!(extractor.supported_mime_types().contains(&"image/jpeg"));
        assert!(extractor.supported_mime_types().contains(&"image/webp"));
        assert_eq!(extractor.priority(), 50);
    }

    #[test]
    fn test_image_extractor_default() {
        let extractor = ImageExtractor;
        assert_eq!(extractor.name(), "image-extractor");
    }

    #[test]
    fn test_image_extractor_supports_alias_mime_types() {
        let extractor = ImageExtractor::new();
        let supported = extractor.supported_mime_types();
        assert!(supported.contains(&"image/pjpeg"));
        assert!(supported.contains(&"image/x-bmp"));
        assert!(supported.contains(&"image/x-ms-bmp"));
        assert!(supported.contains(&"image/x-tiff"));
        assert!(supported.contains(&"image/x-portable-anymap"));
    }
}
