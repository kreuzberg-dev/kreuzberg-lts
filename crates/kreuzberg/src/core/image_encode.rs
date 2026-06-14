//! Centralised image re-encoding helper.
//!
//! This module provides [`re_encode`], which converts an [`ExtractedImage`] in place
//! to a caller-selected target format.  It is designed to be called from the extraction
//! pipeline after OCR has run and before post-processors that consume `data` + `format`.

use std::borrow::Cow;
use std::io::Cursor;

use bytes::Bytes;
use image::{DynamicImage, ImageFormat};
use tracing::warn;

use crate::core::config::extraction::ImageOutputFormat;
use crate::types::ExtractedImage;

// ── Public warning type ──────────────────────────────────────────────────────

/// Describes why a re-encode attempt was skipped or failed.
///
/// The pipeline converts `Err(EncodeWarning)` into a `ProcessingWarning` and leaves
/// the image bytes untouched — the caller is never left with a partially-written image.
#[derive(Debug)]
pub(crate) enum EncodeWarning {
    /// The source format cannot be decoded by any available decoder (vector/metafile formats).
    Undecodable {
        /// Format string of the source image (e.g. `"svg"`, `"emf"`).
        source_format: String,
    },
    /// The source bytes failed to decode despite the format being nominally supported.
    DecodeFailed {
        /// Format string that was attempted.
        source_format: String,
        /// Underlying error message from the decoder.
        message: String,
    },
    /// The decoded image could not be encoded to the target format.
    EncodeFailed {
        /// Name of the target format (e.g. `"jpeg"`, `"webp"`).
        target_format: &'static str,
        /// Underlying error message from the encoder.
        message: String,
    },
    /// The encoder for the target format is not available at runtime.
    EncoderUnavailable {
        /// Name of the target format.
        target_format: &'static str,
        /// Details about why the encoder is unavailable.
        message: String,
    },
}

impl std::fmt::Display for EncodeWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeWarning::Undecodable { source_format } => {
                write!(f, "cannot re-encode format '{source_format}': no decoder available")
            }
            EncodeWarning::DecodeFailed { source_format, message } => {
                write!(f, "failed to decode '{source_format}' image: {message}")
            }
            EncodeWarning::EncodeFailed { target_format, message } => {
                write!(f, "failed to encode image as {target_format}: {message}")
            }
            EncodeWarning::EncoderUnavailable { target_format, message } => {
                write!(f, "encoder for {target_format} is unavailable: {message}")
            }
        }
    }
}

// ── Public entry point ───────────────────────────────────────────────────────

/// Re-encode `image` in place to the requested `target` format.
///
/// Returns `Ok(true)` when the image was re-encoded and both `data` and `format`
/// have been updated.  Returns `Ok(false)` when no work was necessary (either the
/// target is [`ImageOutputFormat::Native`] or the image is already in the target
/// format).  On `Err(EncodeWarning)` the image is left completely untouched.
pub(crate) fn re_encode(image: &mut ExtractedImage, target: ImageOutputFormat) -> Result<bool, EncodeWarning> {
    // Fast exit 1: caller said Native — do nothing.
    if target == ImageOutputFormat::Native {
        return Ok(false);
    }

    // Fast exit 2: source is already the target format — no-op.
    if target_matches_format(target, &image.format) {
        return Ok(false);
    }

    // Reject untranslatable vector / metafile formats.
    if is_untranslatable(&image.format) {
        return Err(EncodeWarning::Undecodable {
            source_format: image.format.to_string(),
        });
    }

    // Decode source bytes to a DynamicImage.
    let dynamic = decode_source(image)?;

    // Encode the DynamicImage to the target format.
    let (new_bytes, new_format) = encode_to_target(&dynamic, target)?;

    // Commit the result — only reached on full success.
    image.data = Bytes::from(new_bytes);
    image.format = Cow::Borrowed(new_format);

    Ok(true)
}

// ── Format helpers ───────────────────────────────────────────────────────────

/// Returns `true` when `target` already matches the source `format` string,
/// meaning no re-encode is needed.
fn target_matches_format(target: ImageOutputFormat, format: &str) -> bool {
    match target {
        ImageOutputFormat::Native => true, // guarded at call site; included for exhaustiveness
        ImageOutputFormat::Png => format.eq_ignore_ascii_case("png"),
        ImageOutputFormat::Jpeg { .. } => format.eq_ignore_ascii_case("jpeg") || format.eq_ignore_ascii_case("jpg"),
        ImageOutputFormat::WebP { .. } => format.eq_ignore_ascii_case("webp"),
        #[cfg(feature = "heic")]
        ImageOutputFormat::Heif { .. } => {
            format.eq_ignore_ascii_case("heif")
                || format.eq_ignore_ascii_case("heic")
                || format.eq_ignore_ascii_case("HEIF")
                || format.eq_ignore_ascii_case("HEIC")
        }
    }
}

/// Returns `true` for formats that cannot be decoded by any available decoder.
///
/// These are vector / Windows-metafile formats for which no raster pixel data
/// is accessible.  Returning `Err(EncodeWarning::Undecodable)` signals the
/// pipeline to skip re-encoding and emit a warning instead.
fn is_untranslatable(format: &str) -> bool {
    matches!(
        format.to_ascii_lowercase().as_str(),
        "svg" | "emf" | "wmf" | "jpeg2000" | "jp2" | "j2k"
    )
}

// ── Decode ───────────────────────────────────────────────────────────────────

/// Decode the source bytes inside `image` to a [`DynamicImage`].
///
/// The dispatch order is:
/// 1. Known format strings → `image::load_from_memory_with_format`
/// 2. `"heic"` / `"heif"` / `"HEIC"` / `"HEIF"` → `kreuzberg-libheif` (feature `heic`)
/// 3. `"unknown"` or anything else → `image::load_from_memory` (magic-byte auto-detect)
fn decode_source(image: &ExtractedImage) -> Result<DynamicImage, EncodeWarning> {
    let format_lc = image.format.to_ascii_lowercase();

    // Delegate heic/heif to kreuzberg-libheif when available.
    #[cfg(feature = "heic")]
    if matches!(format_lc.as_str(), "heic" | "heif") {
        return decode_heic(&image.data, &format_lc);
    }

    // For heic/heif without the feature flag, return Undecodable.
    #[cfg(not(feature = "heic"))]
    if matches!(format_lc.as_str(), "heic" | "heif") {
        return Err(EncodeWarning::Undecodable {
            source_format: image.format.to_string(),
        });
    }

    // Map known format strings to image::ImageFormat.
    let maybe_fmt: Option<ImageFormat> = match format_lc.as_str() {
        "jpeg" | "jpg" => Some(ImageFormat::Jpeg),
        "png" => Some(ImageFormat::Png),
        "webp" => Some(ImageFormat::WebP),
        "gif" => Some(ImageFormat::Gif),
        "bmp" => Some(ImageFormat::Bmp),
        "tiff" | "tif" => Some(ImageFormat::Tiff),
        "pnm" | "pbm" | "pgm" | "ppm" => Some(ImageFormat::Pnm),
        _ => None,
    };

    match maybe_fmt {
        Some(fmt) => image::load_from_memory_with_format(&image.data, fmt).map_err(|err| EncodeWarning::DecodeFailed {
            source_format: image.format.to_string(),
            message: err.to_string(),
        }),
        None => {
            // Unknown format — try magic-byte auto-detection.
            image::load_from_memory(&image.data).map_err(|_err| EncodeWarning::Undecodable {
                source_format: image.format.to_string(),
            })
        }
    }
}

/// Decode a HEIC/HEIF image via `kreuzberg-libheif` into a [`DynamicImage`].
///
/// The decoded output is always RGBA8 so that the subsequent encode step has a
/// uniform input regardless of the source chroma.
#[cfg(feature = "heic")]
fn decode_heic(data: &[u8], source_format: &str) -> Result<DynamicImage, EncodeWarning> {
    use kreuzberg_libheif::{ColorSpace, HeifContext, LibHeif, RgbChroma};

    let context = HeifContext::read_from_bytes(data).map_err(|err| EncodeWarning::DecodeFailed {
        source_format: source_format.to_string(),
        message: format!("{err:?}"),
    })?;

    let handle = context
        .primary_image_handle()
        .map_err(|err| EncodeWarning::DecodeFailed {
            source_format: source_format.to_string(),
            message: format!("{err:?}"),
        })?;

    let lib = LibHeif::new();
    let heif_img = lib
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None)
        .map_err(|err| EncodeWarning::DecodeFailed {
            source_format: source_format.to_string(),
            message: format!("{err:?}"),
        })?;

    let planes = heif_img.planes();
    let plane = planes.interleaved.as_ref().ok_or_else(|| EncodeWarning::DecodeFailed {
        source_format: source_format.to_string(),
        message: "HEIF image has no interleaved plane".to_string(),
    })?;

    let width = heif_img.width();
    let height = heif_img.height();

    // The plane may have stride padding — collect only the valid pixels row-by-row.
    let row_size = (width as usize) * 4; // RGBA = 4 bytes/pixel
    let mut rgba_bytes: Vec<u8> = Vec::with_capacity((width as usize) * (height as usize) * 4);
    for row in plane.data.chunks(plane.stride) {
        rgba_bytes.extend_from_slice(&row[..row_size.min(row.len())]);
    }

    let rgba_img =
        image::RgbaImage::from_raw(width, height, rgba_bytes).ok_or_else(|| EncodeWarning::DecodeFailed {
            source_format: source_format.to_string(),
            message: format!("RGBA buffer does not fit {width}×{height} image"),
        })?;

    Ok(DynamicImage::ImageRgba8(rgba_img))
}

// ── Encode ───────────────────────────────────────────────────────────────────

/// Encode `img` into `target` format and return the raw bytes plus the canonical
/// format name string.
///
/// Returns `Err(EncodeWarning)` if the encode step fails.
fn encode_to_target(img: &DynamicImage, target: ImageOutputFormat) -> Result<(Vec<u8>, &'static str), EncodeWarning> {
    match target {
        ImageOutputFormat::Native => {
            // Guarded at call site; should never be reached.
            unreachable!("Native target must be handled before encode dispatch")
        }
        ImageOutputFormat::Png => {
            let bytes = encode_png(img)?;
            Ok((bytes, "png"))
        }
        ImageOutputFormat::Jpeg { quality } => {
            let clamped = clamp_quality(quality, "jpeg");
            let bytes = encode_jpeg(img, clamped)?;
            Ok((bytes, "jpeg"))
        }
        ImageOutputFormat::WebP { quality: _ } => {
            // The `image` 0.25 crate's built-in WebP encoder supports lossless
            // encoding only (VP8L).  Lossy WebP would require an additional
            // dependency (`webp` crate / libwebp FFI), which is not included to
            // avoid pulling in a C library.  We emit lossless WebP regardless of
            // the quality field and document that the quality knob is ignored
            // until a lossy encoder is wired in.
            let bytes = encode_webp_lossless(img)?;
            Ok((bytes, "webp"))
        }
        #[cfg(feature = "heic")]
        ImageOutputFormat::Heif { quality } => {
            let clamped = clamp_quality(quality, "heif");
            let bytes = encode_heif(img, clamped)?;
            Ok((bytes, "heif"))
        }
    }
}

/// Clamp a quality value to `1..=100` and emit a warning when clamping occurs.
fn clamp_quality(quality: u8, format_name: &'static str) -> u8 {
    // u8 cannot be negative; upper bound is the only relevant check.
    if quality == 0 {
        warn!(
            target: "kreuzberg::image_encode",
            quality,
            format = format_name,
            "quality 0 is out of range (1–100); clamped to 1"
        );
        return 1;
    }
    if quality > 100 {
        warn!(
            target: "kreuzberg::image_encode",
            quality,
            format = format_name,
            "quality {quality} is out of range (1–100); clamped to 100"
        );
        return 100;
    }
    quality
}

/// Encode `img` as PNG (lossless).
fn encode_png(img: &DynamicImage) -> Result<Vec<u8>, EncodeWarning> {
    let mut buf: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|err| EncodeWarning::EncodeFailed {
            target_format: "png",
            message: err.to_string(),
        })?;
    Ok(buf)
}

/// Encode `img` as JPEG at the given quality (1–100).
fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, EncodeWarning> {
    use image::codecs::jpeg::JpegEncoder;
    let mut buf: Vec<u8> = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    encoder.encode_image(img).map_err(|err| EncodeWarning::EncodeFailed {
        target_format: "jpeg",
        message: err.to_string(),
    })?;
    Ok(buf)
}

/// Encode `img` as lossless WebP using the `image` crate's built-in VP8L encoder.
///
/// The `quality` field from [`ImageOutputFormat::WebP`] is intentionally ignored:
/// `image` 0.25 exposes only lossless WebP (VP8L) via `WebPEncoder::new_lossless`.
/// Lossy encoding would require the `webp` crate (libwebp FFI) or a future `image`
/// release that exposes a quality knob on its VP8 encode path.
fn encode_webp_lossless(img: &DynamicImage) -> Result<Vec<u8>, EncodeWarning> {
    let mut buf: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP)
        .map_err(|err| EncodeWarning::EncodeFailed {
            target_format: "webp",
            message: err.to_string(),
        })?;
    Ok(buf)
}

/// Encode `img` as HEIF/HEVC using `kreuzberg-libheif`.
///
/// The pixel data is first converted to RGBA8 (via `DynamicImage::to_rgba8`)
/// and then written into a libheif interleaved plane before encoding.
#[cfg(feature = "heic")]
fn encode_heif(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, EncodeWarning> {
    use kreuzberg_libheif::{
        Channel, ColorSpace, CompressionFormat, EncoderQuality, HeifContext, Image, LibHeif, RgbChroma,
    };

    // Convert to RGBA8 — handles any input format uniformly.
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Create an output HEIF context.
    let mut context = HeifContext::new().map_err(|err| EncodeWarning::EncoderUnavailable {
        target_format: "heif",
        message: format!("HeifContext::new failed: {err:?}"),
    })?;

    // Create a libheif Image with RGBA8 interleaved layout.
    let mut heif_img =
        Image::new(width, height, ColorSpace::Rgb(RgbChroma::Rgba)).map_err(|err| EncodeWarning::EncodeFailed {
            target_format: "heif",
            message: format!("Image::new failed: {err:?}"),
        })?;

    // Add the single interleaved RGBA plane at 8 bits per channel.
    heif_img
        .create_plane(Channel::Interleaved, width, height, 8)
        .map_err(|err| EncodeWarning::EncodeFailed {
            target_format: "heif",
            message: format!("create_plane failed: {err:?}"),
        })?;

    // Copy pixel data row-by-row respecting the plane's stride.
    {
        let mut planes = heif_img.planes_mut();
        let plane = planes.interleaved.as_mut().ok_or(EncodeWarning::EncodeFailed {
            target_format: "heif",
            message: "interleaved plane missing after create_plane".to_string(),
        })?;
        let row_size = (width as usize) * 4; // RGBA = 4 bytes/pixel
        for (dst_row, src_row) in plane.data.chunks_mut(plane.stride).zip(rgba.chunks_exact(row_size)) {
            dst_row[..row_size].copy_from_slice(src_row);
        }
    }

    // Obtain the HEVC encoder (highest-priority plugin for HEIF containers).
    let lib = LibHeif::new();
    let mut encoder =
        lib.encoder_for_format(CompressionFormat::Hevc)
            .map_err(|err| EncodeWarning::EncoderUnavailable {
                target_format: "heif",
                message: format!("no HEVC encoder available: {err:?}"),
            })?;

    // Set lossy quality (clamped to u8 range; quality is already 1–100 here).
    encoder
        .set_quality(EncoderQuality::Lossy(quality))
        .map_err(|err| EncodeWarning::EncodeFailed {
            target_format: "heif",
            message: format!("set_quality failed: {err:?}"),
        })?;

    // Encode the image into the context.
    context
        .encode_image(&heif_img, &mut encoder, None)
        .map_err(|err| EncodeWarning::EncodeFailed {
            target_format: "heif",
            message: format!("encode_image failed: {err:?}"),
        })?;

    // Serialise the context to raw HEIF bytes.
    context.write_to_bytes().map_err(|err| EncodeWarning::EncodeFailed {
        target_format: "heif",
        message: format!("write_to_bytes failed: {err:?}"),
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal valid 4×4 PNG image as `Bytes` for use in tests.
    fn make_png_bytes() -> Bytes {
        let img = image::RgbImage::new(4, 4);
        let mut buf: Vec<u8> = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
            .expect("test PNG encode");
        Bytes::from(buf)
    }

    /// Create a minimal valid 4×4 JPEG image as `Bytes` for use in tests.
    fn make_jpeg_bytes() -> Bytes {
        use image::codecs::jpeg::JpegEncoder;
        let img = image::RgbImage::new(4, 4);
        let mut buf: Vec<u8> = Vec::new();
        JpegEncoder::new_with_quality(&mut buf, 85)
            .encode_image(&DynamicImage::ImageRgb8(img))
            .expect("test JPEG encode");
        Bytes::from(buf)
    }

    /// Create a minimal `ExtractedImage` from raw bytes and a format string.
    fn make_image(data: Bytes, format: &'static str) -> ExtractedImage {
        ExtractedImage {
            data,
            format: Cow::Borrowed(format),
            ..Default::default()
        }
    }

    // ── No-op paths ──────────────────────────────────────────────────────────

    #[test]
    fn native_target_no_op() {
        let original_data = make_png_bytes();
        let mut image = make_image(original_data.clone(), "png");
        let result = re_encode(&mut image, ImageOutputFormat::Native);
        assert!(matches!(result, Ok(false)), "Native must return Ok(false)");
        assert_eq!(image.data, original_data, "bytes must be untouched");
        assert_eq!(image.format.as_ref(), "png", "format must be untouched");
    }

    #[test]
    fn same_format_no_op() {
        let original_data = make_png_bytes();
        let mut image = make_image(original_data.clone(), "png");
        let result = re_encode(&mut image, ImageOutputFormat::Png);
        assert!(matches!(result, Ok(false)), "already-PNG → Png must return Ok(false)");
        assert_eq!(image.data, original_data, "bytes must be untouched");
    }

    // ── Successful re-encode paths ────────────────────────────────────────────

    #[test]
    fn png_to_jpeg() {
        let mut image = make_image(make_png_bytes(), "png");
        let result = re_encode(&mut image, ImageOutputFormat::Jpeg { quality: 85 });
        assert!(
            matches!(result, Ok(true)),
            "png→jpeg must return Ok(true); got {result:?}"
        );
        assert_eq!(image.format.as_ref(), "jpeg");
        let guessed = image::guess_format(&image.data).expect("should detect valid JPEG");
        assert_eq!(guessed, ImageFormat::Jpeg);
    }

    #[test]
    fn jpeg_to_png() {
        let mut image = make_image(make_jpeg_bytes(), "jpeg");
        let result = re_encode(&mut image, ImageOutputFormat::Png);
        assert!(
            matches!(result, Ok(true)),
            "jpeg→png must return Ok(true); got {result:?}"
        );
        assert_eq!(image.format.as_ref(), "png");
        let guessed = image::guess_format(&image.data).expect("should detect valid PNG");
        assert_eq!(guessed, ImageFormat::Png);
    }

    #[test]
    fn png_to_webp() {
        let mut image = make_image(make_png_bytes(), "png");
        let result = re_encode(&mut image, ImageOutputFormat::WebP { quality: 80 });
        assert!(
            matches!(result, Ok(true)),
            "png→webp must return Ok(true); got {result:?}"
        );
        assert_eq!(image.format.as_ref(), "webp");
        let guessed = image::guess_format(&image.data).expect("should detect valid WebP");
        assert_eq!(guessed, ImageFormat::WebP);
    }

    // ── Error paths ───────────────────────────────────────────────────────────

    #[test]
    fn untranslatable_svg_warns() {
        let svg_bytes = Bytes::from_static(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>");
        let original_data = svg_bytes.clone();
        let mut image = make_image(svg_bytes, "svg");
        let result = re_encode(&mut image, ImageOutputFormat::Png);
        assert!(
            matches!(result, Err(EncodeWarning::Undecodable { ref source_format }) if source_format == "svg"),
            "svg must return Err(Undecodable); got {result:?}",
        );
        // Original bytes must be untouched.
        assert_eq!(image.data, original_data);
        assert_eq!(image.format.as_ref(), "svg");
    }

    #[test]
    fn corrupt_png_decode_fails() {
        let corrupt = Bytes::from_static(b"\x89PNG\r\n\x1a\ncorrupt garbage bytes here");
        let original_data = corrupt.clone();
        let mut image = make_image(corrupt, "png");
        let result = re_encode(&mut image, ImageOutputFormat::Jpeg { quality: 85 });
        assert!(
            matches!(result, Err(EncodeWarning::DecodeFailed { ref source_format, .. }) if source_format == "png"),
            "corrupt PNG must return Err(DecodeFailed); got {result:?}",
        );
        assert_eq!(image.data, original_data, "bytes must be untouched on decode failure");
    }

    #[test]
    fn unknown_format_auto_detects() {
        // bytes are valid PNG but the stored format is "unknown"
        let png_bytes = make_png_bytes();
        let mut image = make_image(png_bytes, "unknown");
        let result = re_encode(&mut image, ImageOutputFormat::Jpeg { quality: 85 });
        assert!(
            matches!(result, Ok(true)),
            "unknown-format valid PNG→jpeg must return Ok(true); got {result:?}"
        );
        assert_eq!(image.format.as_ref(), "jpeg");
    }

    #[test]
    fn quality_out_of_range_clamps() {
        // quality 200 is beyond u8::MAX (255) but the type is u8, so the
        // maximum representable out-of-range value is 101–255.  Test with 200.
        let mut image = make_image(make_png_bytes(), "png");
        // quality: 200 — should clamp to 100 and still encode successfully.
        let result = re_encode(&mut image, ImageOutputFormat::Jpeg { quality: 200 });
        assert!(
            matches!(result, Ok(true)),
            "quality 200 should clamp and encode; got {result:?}"
        );
        assert_eq!(image.format.as_ref(), "jpeg");
    }

    // ── Conditional HEIF tests ─────────────────────────────────────────────

    #[cfg(feature = "heic")]
    #[test]
    fn png_to_heif_round_trip() {
        let mut image = make_image(make_png_bytes(), "png");
        let result = re_encode(&mut image, ImageOutputFormat::Heif { quality: 80 });
        assert!(
            matches!(result, Ok(true)),
            "png→heif must return Ok(true); got {result:?}"
        );
        assert_eq!(image.format.as_ref(), "heif");
        // Verify the output is parseable as HEIF.
        let context =
            kreuzberg_libheif::HeifContext::read_from_bytes(&image.data).expect("output should be valid HEIF");
        let handle = context.primary_image_handle().expect("should have primary image");
        assert_eq!(handle.width(), 4);
        assert_eq!(handle.height(), 4);
    }

    #[cfg(feature = "heic")]
    #[test]
    fn heif_same_format_no_op() {
        // We cannot easily construct HEIF bytes in a unit test without a full encode,
        // so just verify the format-match short-circuit works by checking the return value
        // when format == "heif".  The bytes are arbitrary; the shortcut fires before decode.
        let mut image = make_image(Bytes::from_static(b"placeholder"), "heif");
        let result = re_encode(&mut image, ImageOutputFormat::Heif { quality: 80 });
        assert!(matches!(result, Ok(false)), "heif→heif must return Ok(false)");
    }

    #[cfg(feature = "heic")]
    #[test]
    fn heic_format_string_matches() {
        // "heic" (Apple branding) should also match the Heif target.
        let mut image = make_image(Bytes::from_static(b"placeholder"), "heic");
        let result = re_encode(&mut image, ImageOutputFormat::Heif { quality: 80 });
        assert!(matches!(result, Ok(false)), "heic→Heif must return Ok(false)");
    }
}
