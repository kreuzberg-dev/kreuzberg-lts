//! Helper binary to serialize ExtractionConfig to JSON.
//!
//! This binary is used for cross-language serialization testing.
//! It accepts a JSON string as input and outputs the serialized ExtractionConfig.

use std::io::{self, Read};

use kreuzberg::core::config::ExtractionConfig;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = if std::env::args().len() > 1 {
        std::env::args().nth(1).unwrap()
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let input_json: serde_json::Value = serde_json::from_str(&input)?;

    let config: ExtractionConfig = serde_json::from_value(input_json)?;

    let output = serde_json::to_value(&config)?;

    let output_string = serde_json::to_string_pretty(&output)?;
    println!("{}", output_string);

    Ok(())
}
