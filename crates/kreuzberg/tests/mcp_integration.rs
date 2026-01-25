//! MCP integration tests for API consistency and breaking changes.
//!
//! This test suite validates that:
//! 1. MCP parameters properly handle extraction configuration
//! 2. MCP parameter deserialization is consistent
//! 3. Various config combinations work correctly
//!
//! Note: These tests verify the parameter structures used by MCP.
//! The build_config function in the MCP server should accept
//! a config JSON field instead of separate enable_ocr/force_ocr flags
//! to align with the new API consistency approach.

#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::field_reassign_with_default)]

use serde_json::json;

/// Test that parameter structures can handle various JSON configurations
#[test]
fn test_extraction_config_parameter_structure() {
    // This demonstrates the new approach: config JSON instead of separate flags
    let config_json = json!({
        "use_cache": true,
        "force_ocr": true,
        "output_format": "markdown",
    });

    let config: kreuzberg::core::config::ExtractionConfig =
        serde_json::from_value(config_json).expect("Failed to parse config");

    assert_eq!(config.use_cache, true);
    assert_eq!(config.force_ocr, true);
    assert_eq!(config.output_format, kreuzberg::core::config::OutputFormat::Markdown);
}

#[test]
fn test_mcp_style_params_with_config() {
    // This demonstrates how MCP params should accept full config JSON
    let mcp_request = json!({
        "path": "/test.pdf",
        "mime_type": "application/pdf",
        "config": {
            "use_cache": false,
            "force_ocr": true,
            "output_format": "markdown",
        }
    });

    // The config field should be parseable as ExtractionConfig
    let config_obj = mcp_request.get("config").expect("Should have config field");
    let config: kreuzberg::core::config::ExtractionConfig =
        serde_json::from_value(config_obj.clone()).expect("Failed to parse config");

    assert_eq!(config.force_ocr, true);
    assert_eq!(config.use_cache, false);
}

#[test]
fn test_mcp_params_backward_compatibility_minimal() {
    // Minimal MCP params structure
    let params = json!({
        "path": "/test.pdf",
    });

    // Should be deserializable
    let path = params.get("path").expect("Should have path");
    assert_eq!(path, "/test.pdf");
}

#[test]
fn test_mcp_params_with_all_fields() {
    // Complete MCP params with config
    let params = json!({
        "path": "/test.pdf",
        "mime_type": "application/pdf",
        "config": {
            "use_cache": true,
            "enable_quality_processing": true,
            "force_ocr": false,
            "output_format": "plain",
        }
    });

    // Extract and validate config
    if let Some(config_obj) = params.get("config") {
        let config: kreuzberg::core::config::ExtractionConfig =
            serde_json::from_value(config_obj.clone()).expect("Failed to parse");

        assert_eq!(config.use_cache, true);
        assert_eq!(config.force_ocr, false);
        assert_eq!(config.output_format, kreuzberg::core::config::OutputFormat::Plain);
    }
}

#[test]
fn test_batch_extraction_params_structure() {
    // Batch extraction params with paths and config
    let batch_params = json!({
        "paths": ["/file1.pdf", "/file2.pdf", "/file3.pdf"],
        "config": {
            "force_ocr": true,
            "max_concurrent_extractions": 4,
        }
    });

    let paths = batch_params.get("paths").expect("Should have paths");
    assert!(paths.is_array());
    assert_eq!(paths.as_array().unwrap().len(), 3);

    if let Some(config_obj) = batch_params.get("config") {
        let config: kreuzberg::core::config::ExtractionConfig =
            serde_json::from_value(config_obj.clone()).expect("Failed to parse");
        assert_eq!(config.force_ocr, true);
        assert_eq!(config.max_concurrent_extractions, Some(4));
    }
}

#[test]
fn test_config_merge_in_mcp_context() {
    // Simulate default config being merged with request config
    let mut default_config = kreuzberg::core::config::ExtractionConfig::default();
    default_config.use_cache = false;

    // Request provides partial config override
    let request_config_json = json!({
        "force_ocr": true,
    });

    let request_config: kreuzberg::core::config::ExtractionConfig =
        serde_json::from_value(request_config_json).expect("Failed to parse");

    // In MCP context, request config overrides defaults
    let final_config = request_config;

    // The new request config replaces defaults
    assert_eq!(final_config.force_ocr, true);
    // Other fields get their defaults
    assert_eq!(final_config.use_cache, true);
}

#[test]
fn test_config_json_flexibility() {
    // Config JSON can have any combination of fields
    let configs = vec![
        json!({}),                                                             // Empty = all defaults
        json!({"force_ocr": true}),                                            // Single field
        json!({"force_ocr": true, "use_cache": false}),                        // Multiple fields
        json!({"output_format": "markdown", "max_concurrent_extractions": 8}), // Various types
    ];

    for config_json in configs {
        let config: Result<kreuzberg::core::config::ExtractionConfig, _> = serde_json::from_value(config_json);
        assert!(config.is_ok(), "Config should deserialize successfully");
    }
}

#[test]
fn test_extraction_config_serialization_for_mcp() {
    // MCP should be able to serialize config back to JSON
    let mut config = kreuzberg::core::config::ExtractionConfig::default();
    config.force_ocr = true;
    config.output_format = kreuzberg::core::config::OutputFormat::Markdown;

    let json = serde_json::to_value(&config).expect("Failed to serialize");

    // Verify it round-trips
    let restored: kreuzberg::core::config::ExtractionConfig =
        serde_json::from_value(json).expect("Failed to deserialize");

    assert_eq!(config.force_ocr, restored.force_ocr);
    assert_eq!(config.output_format, restored.output_format);
}
