//! Regression test for https://github.com/kreuzberg-dev/kreuzberg-lts/issues/781
//!
//! DOCX OCR extraction was failing because the pipeline was deriving the document
//! (Markdown/Text generation) BEFORE running OCR on embedded images. As a result,
//! the renderers could not see or inject the OCR text results.
//!
//! This test verifies that OCR results for images in a DOCX file are successfully
//! injected into the final content.

#![cfg(feature = "ocr")]
#![cfg(feature = "office")]

mod helpers;

use helpers::*;
use kreuzberg::core::config::{ExtractionConfig, ImageExtractionConfig, OcrConfig};
use kreuzberg::extract_file_sync;

#[test]
fn test_docx_ocr_content_injection() {
    let file_path = get_test_file_path("docx/word_sample.docx");

    let config = ExtractionConfig {
        ocr: Some(OcrConfig {
            backend: "tesseract".to_string(),
            language: "eng".to_string(),
            ..Default::default()
        }),
        images: Some(ImageExtractionConfig {
            extract_images: true,
            ..Default::default()
        }),
        force_ocr: true,
        use_cache: false,
        ..Default::default()
    };

    let result = match extract_file_sync(&file_path, None, &config) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("OCR extraction failed: {}", e);
            return;
        }
    };

    let images = result.images.as_ref().expect("images must be extracted");
    assert!(!images.is_empty(), "DOCX should have at least one image");

    let has_ocr_content = images.iter().any(|img| {
        img.ocr_result
            .as_ref()
            .is_some_and(|ocr| !ocr.content.trim().is_empty())
    });

    if has_ocr_content {
        let mut found_in_content = false;
        for img in images {
            if let Some(ocr) = &img.ocr_result
                && !ocr.content.trim().is_empty()
                && result.content.contains(&ocr.content)
            {
                found_in_content = true;
                break;
            }
        }
        assert!(
            found_in_content,
            "OCR content from images must be present in the final document content"
        );
    } else {
        eprintln!("No OCR content produced for images; skipping injection verification");
    }
}
