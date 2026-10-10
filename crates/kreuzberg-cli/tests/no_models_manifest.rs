#![cfg(not(any(feature = "paddle-ocr", feature = "layout-detection")))]

use std::process::Command;

#[test]
fn should_return_empty_manifest_when_model_features_are_disabled() {
    let output = Command::new(env!("CARGO_BIN_EXE_kreuzberg"))
        .args(["cache", "manifest", "--format", "json"])
        .output()
        .expect("cache manifest command should run");
    assert_eq!(output.status.code(), Some(0));
    let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).expect("manifest should be valid JSON");
    assert_eq!(manifest["model_count"], 0);
    assert_eq!(manifest["total_size_bytes"], 0);
    assert_eq!(manifest["models"], serde_json::json!([]));
}
