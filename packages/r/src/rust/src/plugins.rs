//! Plugin registration FFI wrappers

use crate::error::kreuzberg_error;
use extendr_api::prelude::*;

use kreuzberg::plugins::{
    clear_extractors as kz_clear_extractors, clear_ocr_backends as kz_clear_ocr_backends,
    clear_validators as kz_clear_validators, list_extractors as kz_list_extractors,
    list_ocr_backends as kz_list_ocr_backends, list_post_processors as kz_list_post_processors,
    list_validators as kz_list_validators, unregister_extractor as kz_unregister_extractor,
    unregister_ocr_backend as kz_unregister_ocr_backend, unregister_validator as kz_unregister_validator,
};

pub fn register_post_processor_impl(_name: &str, _callback: Robj) -> extendr_api::Result<()> {
    Err(extendr_api::Error::Other(
        "Post-processor registration from R not yet supported".to_string(),
    ))
}

pub fn unregister_post_processor_impl(name: &str) -> extendr_api::Result<()> {
    let registry = kreuzberg::get_post_processor_registry();
    registry.write().remove(name).map_err(kreuzberg_error)?;
    Ok(())
}

pub fn list_post_processors_impl() -> extendr_api::Result<Strings> {
    let names = kz_list_post_processors().map_err(kreuzberg_error)?;
    Ok(Strings::from_values(names))
}

pub fn clear_post_processors_impl() -> extendr_api::Result<()> {
    let registry = kreuzberg::get_post_processor_registry();
    registry.write().shutdown_all().map_err(kreuzberg_error)?;
    Ok(())
}

pub fn register_validator_impl(_name: &str, _callback: Robj) -> extendr_api::Result<()> {
    Err(extendr_api::Error::Other(
        "Validator registration from R not yet supported".to_string(),
    ))
}

pub fn unregister_validator_impl(name: &str) -> extendr_api::Result<()> {
    kz_unregister_validator(name).map_err(kreuzberg_error)
}

pub fn list_validators_impl() -> extendr_api::Result<Strings> {
    let names = kz_list_validators().map_err(kreuzberg_error)?;
    Ok(Strings::from_values(names))
}

pub fn clear_validators_impl() -> extendr_api::Result<()> {
    kz_clear_validators().map_err(kreuzberg_error)
}

pub fn register_ocr_backend_impl(_name: &str, _callback: Robj) -> extendr_api::Result<()> {
    Err(extendr_api::Error::Other(
        "OCR backend registration from R not yet supported".to_string(),
    ))
}

pub fn unregister_ocr_backend_impl(name: &str) -> extendr_api::Result<()> {
    kz_unregister_ocr_backend(name).map_err(kreuzberg_error)
}

pub fn list_ocr_backends_impl() -> extendr_api::Result<Strings> {
    let names = kz_list_ocr_backends().map_err(kreuzberg_error)?;
    Ok(Strings::from_values(names))
}

pub fn clear_ocr_backends_impl() -> extendr_api::Result<()> {
    kz_clear_ocr_backends().map_err(kreuzberg_error)
}

pub fn list_document_extractors_impl() -> extendr_api::Result<Strings> {
    let names = kz_list_extractors().map_err(kreuzberg_error)?;
    Ok(Strings::from_values(names))
}

pub fn unregister_document_extractor_impl(name: &str) -> extendr_api::Result<()> {
    kz_unregister_extractor(name).map_err(kreuzberg_error)
}

pub fn clear_document_extractors_impl() -> extendr_api::Result<()> {
    kz_clear_extractors().map_err(kreuzberg_error)
}
