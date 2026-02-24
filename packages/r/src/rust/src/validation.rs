//! Validation function wrappers

use extendr_api::prelude::*;

pub fn validate_ocr_backend_impl(backend: &str) -> extendr_api::Result<bool> {
    let code = unsafe { kreuzberg_ffi::kreuzberg_validate_ocr_backend(backend.as_ptr() as *const std::os::raw::c_char) };
    Ok(code == 0)
}

pub fn validate_language_code_impl(code_str: &str) -> extendr_api::Result<bool> {
    let code = unsafe { kreuzberg_ffi::kreuzberg_validate_language_code(code_str.as_ptr() as *const std::os::raw::c_char) };
    Ok(code == 0)
}

pub fn validate_output_format_impl(format: &str) -> extendr_api::Result<bool> {
    let code = unsafe { kreuzberg_ffi::kreuzberg_validate_output_format(format.as_ptr() as *const std::os::raw::c_char) };
    Ok(code == 0)
}
