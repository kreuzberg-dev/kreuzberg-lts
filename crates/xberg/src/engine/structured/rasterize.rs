//! PDF / image rasterization for the structured-extraction mechanism.
//!
//! For PDF input, pages are rendered at a caller-supplied DPI via the crate's
//! [`crate::pdf::render`] helpers. For image MIMEs, the input is decoded and
//! re-encoded as a single PNG. The module is lazy: when the call mode decides
//! `Skip` or `TextOnly`, no rasterization happens at all.
//!
//! Unlike the enterprise worker this port came from, the mechanism has no
//! notion of object-storage persistence: there is no `PersistContext` and no
//! storage write loop. Persisting rasters is a caller concern, layered on top
//! of the returned [`PageImage`] values.

use std::io::Cursor;

use thiserror::Error;

// Imported from heuristics — the only call-mode source of truth.
use crate::heuristics::StructuredCallMode;

/// A rendered page ready for inline-base64 transport to the vision model.
#[derive(Debug, Clone)]
pub struct PageImage {
    /// 1-based page number (matches the citation envelope convention).
    pub page_number: u32,
    /// PNG-encoded page bytes.
    pub png_bytes: Vec<u8>,
}

/// Errors produced while rendering pages.
#[derive(Debug, Error)]
pub enum RasterizeError {
    /// PDF parsing or page rendering failed.
    #[error("pdf rendering failed: {0}")]
    Pdf(String),
    /// Image decoding or PNG re-encoding failed.
    #[error("image decoding failed: {0}")]
    Image(String),
    /// The supplied MIME type is neither `application/pdf` nor `image/*`.
    #[error("unsupported mime type: {0}")]
    UnsupportedMime(String),
}

/// Render pages for the given call mode. Returns an empty vec when the
/// heuristic does not require vision (`Skip` / `TextOnly` /
/// `TextOnlyWithVisionFallback`).
///
/// `dpi` is supplied by the caller (the enterprise worker uses 200 DPI); the
/// mechanism does not impose a default. `TextOnlyWithVisionFallback` is handled
/// by the orchestrator — rasterization only happens if a fallback escalates.
pub async fn pages_for_call(
    bytes: &[u8],
    mime: &str,
    mode: StructuredCallMode,
    dpi: u32,
) -> Result<Vec<PageImage>, RasterizeError> {
    let pages = match mode {
        StructuredCallMode::Skip | StructuredCallMode::TextOnly | StructuredCallMode::TextOnlyWithVisionFallback => {
            // TextOnlyWithVisionFallback is handled by the orchestrator — rasterize only if fallback escalates.
            Vec::new()
        }
        StructuredCallMode::VisionOnly | StructuredCallMode::TextPlusVision => {
            let mime_lc = mime.to_ascii_lowercase();
            if mime_lc == "application/pdf" {
                render_pdf(bytes, dpi)?
            } else if mime_lc.starts_with("image/") {
                render_image(bytes)?
            } else {
                return Err(RasterizeError::UnsupportedMime(mime.into()));
            }
        }
    };

    Ok(pages)
}

fn render_pdf(bytes: &[u8], dpi: u32) -> Result<Vec<PageImage>, RasterizeError> {
    let page_count = crate::pdf::render::pdf_page_count(bytes, None)
        .map_err(|e| RasterizeError::Pdf(format!("failed to read page count: {e}")))?;

    let mut pages = Vec::with_capacity(page_count);

    for page_idx in 0..page_count {
        let png_bytes = crate::pdf::render::render_pdf_page_to_png(bytes, page_idx, Some(dpi as i32), None)
            .map_err(|e| RasterizeError::Pdf(format!("failed to render page {}: {e}", page_idx + 1)))?;

        pages.push(PageImage {
            page_number: (page_idx + 1) as u32,
            png_bytes,
        });
    }

    Ok(pages)
}

fn render_image(bytes: &[u8]) -> Result<Vec<PageImage>, RasterizeError> {
    // Decode + re-encode as PNG via the `image` crate. Single-page result.
    let img =
        image::load_from_memory(bytes).map_err(|e| RasterizeError::Image(format!("failed to decode image: {e}")))?;

    let mut png_bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| RasterizeError::Image(format!("failed to encode PNG: {e}")))?;

    Ok(vec![PageImage {
        page_number: 1,
        png_bytes,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_pixel_png() -> Vec<u8> {
        // Minimal 1x1 PNG built via the image crate.
        let img = image::RgbImage::new(1, 1);
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[tokio::test]
    async fn skip_mode_returns_no_pages() {
        let pages = pages_for_call(&[], "application/pdf", StructuredCallMode::Skip, 200)
            .await
            .unwrap();
        assert!(pages.is_empty());
    }

    #[tokio::test]
    async fn text_only_mode_returns_no_pages() {
        let pages = pages_for_call(&[], "application/pdf", StructuredCallMode::TextOnly, 200)
            .await
            .unwrap();
        assert!(pages.is_empty());
    }

    #[tokio::test]
    async fn image_mime_returns_single_page() {
        let png = one_pixel_png();
        let pages = pages_for_call(&png, "image/png", StructuredCallMode::VisionOnly, 200)
            .await
            .unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].page_number, 1);
        assert!(pages[0].png_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]), "PNG magic");
    }

    #[tokio::test]
    async fn unsupported_mime_errors() {
        let res = pages_for_call(&[], "application/zip", StructuredCallMode::VisionOnly, 200).await;
        assert!(matches!(res, Err(RasterizeError::UnsupportedMime(_))));
    }

    #[tokio::test]
    async fn text_only_with_vision_fallback_returns_no_pages_initially() {
        let pages = pages_for_call(
            &[],
            "application/pdf",
            StructuredCallMode::TextOnlyWithVisionFallback,
            200,
        )
        .await
        .unwrap();
        assert!(pages.is_empty());
    }
}
