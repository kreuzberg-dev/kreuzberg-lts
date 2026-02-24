//! R list -> ExtractionConfig conversion

use crate::error::to_r_error;
use extendr_api::prelude::*;

/// Parse a JSON config string into an ExtractionConfig
pub fn parse_config(config_json: Nullable<&str>) -> extendr_api::Result<kreuzberg::ExtractionConfig> {
    match config_json {
        Nullable::NotNull(json_str) => {
            let config: kreuzberg::ExtractionConfig =
                serde_json::from_str(json_str).map_err(to_r_error)?;
            Ok(config)
        }
        Nullable::Null => Ok(kreuzberg::ExtractionConfig::default()),
    }
}
