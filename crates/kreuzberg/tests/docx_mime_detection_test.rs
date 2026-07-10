//! Test that DOCX files are correctly detected as DOCX, not ZIP.
//!
//! This tests the fix for https://github.com/kreuzberg-dev/kreuzberg-lts/issues/350

use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_docx_detected_from_bytes_not_zip() {
    let mut file = NamedTempFile::new().unwrap();

    let docx_content: &[u8] = &[
        0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, b'w', b'o', b'r', b'd', b'/', b'd',
        b'o', b'c', b'u', b'm', b'e', b'n', b't', b'.', b'x', b'm', b'l',
    ];

    file.write_all(docx_content).unwrap();
    file.flush().unwrap();

    let content = std::fs::read(file.path()).unwrap();
    let mime = kreuzberg::core::mime::detect_mime_type_from_bytes(&content).unwrap();

    assert_eq!(
        mime, "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "DOCX file should be detected as DOCX MIME type, not ZIP"
    );
}

#[test]
fn test_xlsx_detected_from_bytes_not_zip() {
    let xlsx_content: &[u8] = &[
        0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, b'x', b'l', b'/', b'w', b'o', b'r',
        b'k', b'b', b'o', b'o', b'k', b'.', b'x', b'm', b'l',
    ];

    let mime = kreuzberg::core::mime::detect_mime_type_from_bytes(xlsx_content).unwrap();

    assert_eq!(
        mime, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "XLSX file should be detected as XLSX MIME type, not ZIP"
    );
}

#[test]
fn test_pptx_detected_from_bytes_not_zip() {
    let pptx_content: &[u8] = &[
        0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, b'p', b'p', b't', b'/', b'p', b'r',
        b'e', b's', b'e', b'n', b't', b'a', b't', b'i', b'o', b'n', b'.', b'x', b'm', b'l',
    ];

    let mime = kreuzberg::core::mime::detect_mime_type_from_bytes(pptx_content).unwrap();

    assert_eq!(
        mime, "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "PPTX file should be detected as PPTX MIME type, not ZIP"
    );
}

#[test]
fn test_plain_zip_still_detected_as_zip() {
    let plain_zip_content: &[u8] = &[
        0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, b't', b'e', b's', b't', b'.', b't',
        b'x', b't',
    ];

    let mime = kreuzberg::core::mime::detect_mime_type_from_bytes(plain_zip_content).unwrap();

    assert_eq!(mime, "application/zip", "Plain ZIP should remain as application/zip");
}
