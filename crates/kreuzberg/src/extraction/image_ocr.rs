//! Centralized image OCR processing.
//!
//! Provides a shared function for processing extracted images with OCR,
//! used by DOCX, PPTX, Jupyter, Markdown, and other extractors.
//!
//! # Recursion Prevention
//!
//! The OCR results produced here set `images: None` to prevent any
//! downstream consumer from triggering further image extraction on
//! OCR output. This breaks the potential cycle:
//! document → extract images → OCR images → (no further image extraction).
//!
//! # Concurrency
//!
//! Image OCR tasks are processed with a bounded concurrency limit
//! derived from the configured thread budget to prevent resource
//! exhaustion when documents contain many embedded images.

#[cfg(all(feature = "ocr", feature = "tokio-runtime"))]
use crate::ocr::OcrProcessor;
use crate::types::{ExtractedImage, ExtractionResult};

#[cfg(all(feature = "ocr", feature = "tokio-runtime"))]
fn is_ocr_decodable_image(image: &ExtractedImage) -> std::result::Result<(), String> {
    if image.data.is_empty() {
        return Err("image data is empty".to_string());
    }

    let format = image.format.as_ref().to_ascii_lowercase();
    if matches!(
        format.as_str(),
        "raw" | "ccitt" | "jbig2" | "jpeg2000" | "jpx" | "unknown"
    ) {
        return Err(format!("unsupported image format for OCR: {format}"));
    }

    let cursor = std::io::Cursor::new(image.data.as_ref());
    image::ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| format!("image format probe failed: {e}"))?
        .into_dimensions()
        .map_err(|e| format!("image dimensions could not be decoded: {e}"))?;

    Ok(())
}

/// Process extracted images with OCR if configured.
///
/// For each image, spawns a blocking OCR task and stores the result
/// in `image.ocr_result`. If OCR is not configured or fails for an
/// individual image, that image's `ocr_result` remains `None`.
///
/// This function is the single shared implementation used by all
/// document extractors (DOCX, PPTX, Jupyter, Markdown, etc.).
///
/// # Recursion Safety
///
/// The produced `ExtractionResult` for each image explicitly sets
/// `images: None`, preventing further image extraction cycles when
/// OCR results are consumed by archive or recursive extraction paths.
///
/// # Concurrency
///
/// Concurrency is bounded by the configured thread budget
/// using a semaphore to prevent resource exhaustion.
#[cfg(all(feature = "ocr", feature = "tokio-runtime"))]
pub async fn process_images_with_ocr(
    mut images: Vec<ExtractedImage>,
    config: &crate::core::config::ExtractionConfig,
    warnings: &mut Vec<crate::types::ProcessingWarning>,
) -> crate::Result<Vec<ExtractedImage>> {
    if images.is_empty() || config.ocr.is_none() {
        return Ok(images);
    }

    let ocr_config = config.ocr.as_ref().unwrap();
    let tess_config = ocr_config.tesseract_config.as_ref().cloned().unwrap_or_default();
    let output_format = config.output_format.clone();

    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    let max_tasks = crate::core::config::concurrency::resolve_thread_budget(config.concurrency.as_ref());
    let semaphore = Arc::new(Semaphore::new(max_tasks));

    type OcrTaskResult = (
        usize,
        Result<Result<crate::types::OcrExtractionResult, crate::ocr::error::OcrError>, tokio::task::JoinError>,
    );
    let mut join_set: JoinSet<OcrTaskResult> = JoinSet::new();

    for (idx, image) in images.iter().enumerate() {
        if let Err(reason) = is_ocr_decodable_image(image) {
            warnings.push(crate::types::ProcessingWarning {
                source: std::borrow::Cow::Borrowed("image_ocr"),
                message: std::borrow::Cow::Owned(format!("Image {} skipped before OCR: {}", idx, reason)),
            });
            continue;
        }

        let image_data = image.data.clone();
        let tess_config_clone = tess_config.clone();
        let span = tracing::Span::current();
        let permit = Arc::clone(&semaphore);
        let output_format = output_format.clone();

        join_set.spawn(async move {
            let _permit = permit.acquire().await.expect("semaphore should not be closed");

            let blocking_result = tokio::task::spawn_blocking(move || {
                let _guard = span.entered();
                let cache_dir = std::env::var("KREUZBERG_CACHE_DIR").ok().map(std::path::PathBuf::from);

                let proc = OcrProcessor::new(cache_dir)?;
                let ocr_tess_config: crate::ocr::types::TesseractConfig = (&tess_config_clone).into();
                proc.process_image_with_format(&image_data, &ocr_tess_config, output_format)
            })
            .await;
            (idx, blocking_result)
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        let (idx, blocking_result) = join_result.map_err(|e| crate::KreuzbergError::Ocr {
            message: format!("OCR task panicked: {}", e),
            source: None,
        })?;

        let ocr_result = blocking_result.map_err(|e| crate::KreuzbergError::Ocr {
            message: format!("OCR blocking task panicked: {}", e),
            source: None,
        })?;

        match ocr_result {
            Ok(ocr_extraction) => {
                let extraction_result = ExtractionResult {
                    content: ocr_extraction.content,
                    mime_type: ocr_extraction.mime_type.into(),
                    ocr_elements: ocr_extraction.ocr_elements,
                    ..Default::default()
                };
                images[idx].ocr_result = Some(Box::new(extraction_result));
            }
            Err(e) => {
                warnings.push(crate::types::ProcessingWarning {
                    source: std::borrow::Cow::Borrowed("image_ocr"),
                    message: std::borrow::Cow::Owned(format!("Image {} OCR failed: {}", idx, e)),
                });
                images[idx].ocr_result = None;
            }
        }
    }

    Ok(images)
}

#[cfg(test)]
#[cfg(all(feature = "ocr", feature = "tokio-runtime"))]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::borrow::Cow;

    fn image_with(format: &'static str, data: Bytes) -> ExtractedImage {
        ExtractedImage {
            data,
            format: Cow::Borrowed(format),
            image_index: 0,
            page_number: Some(1),
            width: None,
            height: None,
            colorspace: None,
            bits_per_component: None,
            is_mask: false,
            description: None,
            ocr_result: None,
            bounding_box: None,
            source_path: None,
        }
    }

    #[test]
    fn raw_pdf_image_stream_is_skipped_before_ocr() {
        let image = image_with("raw", Bytes::from_static(b"not a raster"));
        let reason = is_ocr_decodable_image(&image).expect_err("raw streams must be rejected");
        assert!(reason.contains("unsupported image format"));
    }

    #[test]
    fn empty_image_is_skipped_before_ocr() {
        let image = image_with("png", Bytes::new());
        let reason = is_ocr_decodable_image(&image).expect_err("empty images must be rejected");
        assert!(reason.contains("empty"));
    }
}
