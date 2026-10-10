#![cfg(feature = "dynamic-linking")]

use kreuzberg_tesseract::{Pix, TessPageSegMode, TesseractAPI};

#[test]
fn should_expose_ocr_api_when_using_system_libraries() {
    let _: Option<TesseractAPI> = None;
    let _: Option<Pix> = None;
    assert_eq!(TessPageSegMode::PSM_AUTO as i32, 3);
}
