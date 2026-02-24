//! MIME type detection functions

use crate::error::to_r_error;
use extendr_api::prelude::*;

pub fn detect_mime_type_impl(data: Raw) -> extendr_api::Result<String> {
    let bytes = data.as_slice();
    Ok(kreuzberg::detect_mime_type(bytes))
}

pub fn detect_mime_type_from_path_impl(path: &str) -> extendr_api::Result<String> {
    Ok(kreuzberg::detect_mime_type_from_path(path))
}

pub fn get_extensions_for_mime_impl(mime_type: &str) -> extendr_api::Result<Strings> {
    let extensions = kreuzberg::get_extensions_for_mime(mime_type);
    let r_strings: Vec<String> = extensions.into_iter().map(|s| s.to_string()).collect();
    Ok(r_strings.into())
}

pub fn validate_mime_type_impl(mime_type: &str) -> extendr_api::Result<bool> {
    Ok(kreuzberg::validate_mime_type(mime_type))
}
