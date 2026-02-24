//! Batch extraction functions

use crate::config::parse_config;
use crate::error::to_r_error;
use crate::result::extraction_result_to_list;
use extendr_api::prelude::*;

pub fn batch_extract_files_sync_impl(paths: Strings, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let path_vec: Vec<String> = paths.iter().map(|s| s.to_string()).collect();
    let path_refs: Vec<&str> = path_vec.iter().map(|s| s.as_str()).collect();
    let results = kreuzberg::batch_extract_files_sync(&path_refs, &config).map_err(to_r_error)?;
    let r_results: Vec<Robj> = results.into_iter()
        .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
        .collect::<extendr_api::Result<Vec<_>>>()?;
    Ok(List::from_values(r_results))
}

pub fn batch_extract_files_impl(paths: Strings, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let path_vec: Vec<String> = paths.iter().map(|s| s.to_string()).collect();
    let path_refs: Vec<&str> = path_vec.iter().map(|s| s.as_str()).collect();
    let runtime = tokio::runtime::Runtime::new().map_err(to_r_error)?;
    let results = runtime
        .block_on(async { kreuzberg::batch_extract_files(&path_refs, &config).await })
        .map_err(to_r_error)?;
    let r_results: Vec<Robj> = results.into_iter()
        .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
        .collect::<extendr_api::Result<Vec<_>>>()?;
    Ok(List::from_values(r_results))
}

pub fn batch_extract_bytes_sync_impl(data_list: List, mime_types: Strings, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let bytes_vec: Vec<Vec<u8>> = data_list.values()
        .map(|v| {
            let raw = Raw::try_from(v).map_err(to_r_error)?;
            Ok(raw.as_slice().to_vec())
        })
        .collect::<extendr_api::Result<Vec<_>>>()?;
    let bytes_refs: Vec<&[u8]> = bytes_vec.iter().map(|b| b.as_slice()).collect();
    let mime_vec: Vec<String> = mime_types.iter().map(|s| s.to_string()).collect();
    let mime_refs: Vec<&str> = mime_vec.iter().map(|s| s.as_str()).collect();
    let results = kreuzberg::batch_extract_bytes_sync(&bytes_refs, &mime_refs, &config).map_err(to_r_error)?;
    let r_results: Vec<Robj> = results.into_iter()
        .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
        .collect::<extendr_api::Result<Vec<_>>>()?;
    Ok(List::from_values(r_results))
}

pub fn batch_extract_bytes_impl(data_list: List, mime_types: Strings, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let bytes_vec: Vec<Vec<u8>> = data_list.values()
        .map(|v| {
            let raw = Raw::try_from(v).map_err(to_r_error)?;
            Ok(raw.as_slice().to_vec())
        })
        .collect::<extendr_api::Result<Vec<_>>>()?;
    let bytes_refs: Vec<&[u8]> = bytes_vec.iter().map(|b| b.as_slice()).collect();
    let mime_vec: Vec<String> = mime_types.iter().map(|s| s.to_string()).collect();
    let mime_refs: Vec<&str> = mime_vec.iter().map(|s| s.as_str()).collect();
    let runtime = tokio::runtime::Runtime::new().map_err(to_r_error)?;
    let results = runtime
        .block_on(async { kreuzberg::batch_extract_bytes(&bytes_refs, &mime_refs, &config).await })
        .map_err(to_r_error)?;
    let r_results: Vec<Robj> = results.into_iter()
        .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
        .collect::<extendr_api::Result<Vec<_>>>()?;
    Ok(List::from_values(r_results))
}
