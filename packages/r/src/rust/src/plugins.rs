//! Plugin registration FFI wrappers

use crate::error::to_r_error;
use extendr_api::prelude::*;

// Post-processor plugins
pub fn register_post_processor_impl(_name: &str, _callback: Robj) -> extendr_api::Result<()> {
    // TODO: Implement R callback bridge for post-processors
    Err(extendr_api::Error::Other("Post-processor registration from R not yet supported".to_string()))
}

pub fn unregister_post_processor_impl(name: &str) -> extendr_api::Result<()> {
    kreuzberg::unregister_post_processor(name);
    Ok(())
}

pub fn list_post_processors_impl() -> extendr_api::Result<Strings> {
    let names = kreuzberg::list_post_processors();
    let r_strings: Vec<String> = names.into_iter().map(|s| s.to_string()).collect();
    Ok(r_strings.into())
}

pub fn clear_post_processors_impl() -> extendr_api::Result<()> {
    kreuzberg::clear_post_processors();
    Ok(())
}

// Validator plugins
pub fn register_validator_impl(_name: &str, _callback: Robj) -> extendr_api::Result<()> {
    Err(extendr_api::Error::Other("Validator registration from R not yet supported".to_string()))
}

pub fn unregister_validator_impl(name: &str) -> extendr_api::Result<()> {
    kreuzberg::unregister_validator(name);
    Ok(())
}

pub fn list_validators_impl() -> extendr_api::Result<Strings> {
    let names = kreuzberg::list_validators();
    let r_strings: Vec<String> = names.into_iter().map(|s| s.to_string()).collect();
    Ok(r_strings.into())
}

pub fn clear_validators_impl() -> extendr_api::Result<()> {
    kreuzberg::clear_validators();
    Ok(())
}

// OCR backend plugins
pub fn register_ocr_backend_impl(_name: &str, _callback: Robj) -> extendr_api::Result<()> {
    Err(extendr_api::Error::Other("OCR backend registration from R not yet supported".to_string()))
}

pub fn unregister_ocr_backend_impl(name: &str) -> extendr_api::Result<()> {
    kreuzberg::unregister_ocr_backend(name);
    Ok(())
}

pub fn list_ocr_backends_impl() -> extendr_api::Result<Strings> {
    let names = kreuzberg::list_ocr_backends();
    let r_strings: Vec<String> = names.into_iter().map(|s| s.to_string()).collect();
    Ok(r_strings.into())
}

pub fn clear_ocr_backends_impl() -> extendr_api::Result<()> {
    kreuzberg::clear_ocr_backends();
    Ok(())
}

// Document extractor plugins
pub fn list_document_extractors_impl() -> extendr_api::Result<Strings> {
    let names = kreuzberg::list_document_extractors();
    let r_strings: Vec<String> = names.into_iter().map(|s| s.to_string()).collect();
    Ok(r_strings.into())
}

pub fn unregister_document_extractor_impl(name: &str) -> extendr_api::Result<()> {
    kreuzberg::unregister_document_extractor(name);
    Ok(())
}

pub fn clear_document_extractors_impl() -> extendr_api::Result<()> {
    kreuzberg::clear_document_extractors();
    Ok(())
}
