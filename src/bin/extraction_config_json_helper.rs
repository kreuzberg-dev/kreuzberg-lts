//! Helper binary to serialize ExtractionConfig to JSON.
//!
//! This binary is used for cross-language serialization testing.
//! It accepts a JSON string as input and outputs the serialized ExtractionConfig.

use std::io::{self, Read};

use kreuzberg::core::config::ExtractionConfig;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read input from stdin or command-line argument
    let input = if std::env::args().len() > 1 {
        // Read from command-line argument
        std::env::args().nth(1).unwrap()
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    // Parse input JSON
    let input_json: serde_json::Value = serde_json::from_str(&input)?;

    // Create ExtractionConfig from JSON
    let config: ExtractionConfig = serde_json::from_value(input_json)?;

    // Serialize config to JSON
    let output = serde_json::to_value(&config)?;

    // Pretty-print output
    let output_string = serde_json::to_string_pretty(&output)?;
    println!("{}", output_string);

    Ok(())
}
