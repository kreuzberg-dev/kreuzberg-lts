//! Image parsing and format detection.
//!
//! This module handles image-related parsing from slide XML and
//! detection of image formats from file data.

use std::borrow::Cow;

pub(super) fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub(super) fn detect_image_format(data: &[u8]) -> Cow<'static, str> {
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Cow::Borrowed("jpeg")
    } else if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        Cow::Borrowed("png")
    } else if data.starts_with(b"GIF") {
        Cow::Borrowed("gif")
    } else if data.starts_with(b"BM") {
        Cow::Borrowed("bmp")
    } else if data.starts_with(b"<svg") || data.starts_with(b"<?xml") {
        Cow::Borrowed("svg")
    } else if data.starts_with(b"II\x2A\x00") || data.starts_with(b"MM\x00\x2A") {
        Cow::Borrowed("tiff")
    } else {
        Cow::Borrowed("unknown")
    }
}

pub(super) fn get_slide_rels_path(slide_path: &str) -> String {
    let parts: Vec<&str> = slide_path.rsplitn(2, '/').collect();
    if parts.len() == 2 {
        format!("{}/_rels/{}.rels", parts[1], parts[0])
    } else {
        format!("_rels/{}.rels", slide_path)
    }
}

pub(super) fn get_full_image_path(slide_path: &str, image_target: &str) -> String {
    if image_target.starts_with("..") {
        let parts: Vec<&str> = slide_path.rsplitn(3, '/').collect();
        if parts.len() >= 3 {
            format!("{}/{}", parts[2], &image_target[3..])
        } else {
            format!("ppt/{}", &image_target[3..])
        }
    } else {
        let parts: Vec<&str> = slide_path.rsplitn(2, '/').collect();
        if parts.len() == 2 {
            format!("{}/{}", parts[1], image_target)
        } else {
            format!("ppt/slides/{}", image_target)
        }
    }
}
