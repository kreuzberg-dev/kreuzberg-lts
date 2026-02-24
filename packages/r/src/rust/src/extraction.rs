//! File and bytes extraction functions (sync + async via Tokio)

use crate::config::parse_config;
use crate::error::to_r_error;
use crate::result::extraction_result_to_list;
use extendr_api::prelude::*;

pub fn extract_file_sync_impl(path: &str, mime_type: Nullable<&str>, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let mime = match mime_type {
        Nullable::NotNull(m) => Some(m),
        Nullable::Null => None,
    };
    let result = kreuzberg::extract_file_sync(path, mime, &config).map_err(to_r_error)?;
    extraction_result_to_list(result)
}

pub fn extract_file_impl(path: &str, mime_type: Nullable<&str>, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let mime = match mime_type {
        Nullable::NotNull(m) => Some(m),
        Nullable::Null => None,
    };
    let runtime = tokio::runtime::Runtime::new().map_err(to_r_error)?;
    let result = runtime
        .block_on(async { kreuzberg::extract_file(path, mime, &config).await })
        .map_err(to_r_error)?;
    extraction_result_to_list(result)
}

pub fn extract_bytes_sync_impl(data: Raw, mime_type: &str, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let bytes = data.as_slice();
    let result = kreuzberg::extract_bytes_sync(bytes, mime_type, &config).map_err(to_r_error)?;
    extraction_result_to_list(result)
}

pub fn extract_bytes_impl(data: Raw, mime_type: &str, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let bytes = data.as_slice();
    let runtime = tokio::runtime::Runtime::new().map_err(to_r_error)?;
    let result = runtime
        .block_on(async { kreuzberg::extract_bytes(bytes, mime_type, &config).await })
        .map_err(to_r_error)?;
    extraction_result_to_list(result)
}
