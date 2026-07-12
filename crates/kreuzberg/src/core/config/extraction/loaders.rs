//! Configuration file loading with caching support.
//!
//! This module provides methods for loading extraction configuration from various
//! file formats (TOML, YAML, JSON) with automatic caching based on file modification times.

use crate::{KreuzbergError, Result};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::SystemTime;

use super::core::ExtractionConfig;

static CONFIG_CACHE: LazyLock<DashMap<PathBuf, (SystemTime, Arc<ExtractionConfig>)>> = LazyLock::new(DashMap::new);

impl ExtractionConfig {
    /// Load configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML file
    ///
    /// # Errors
    ///
    /// Returns `KreuzbergError::Validation` if file doesn't exist or is invalid TOML.
    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let metadata = std::fs::metadata(path)
            .map_err(|e| KreuzbergError::validation(format!("Failed to read config file {}: {}", path.display(), e)))?;
        let mtime = metadata.modified().map_err(|e| {
            KreuzbergError::validation(format!("Failed to get modification time for {}: {}", path.display(), e))
        })?;

        if let Some(entry) = CONFIG_CACHE.get(path)
            && entry.0 == mtime
        {
            return Ok((*entry.1).clone());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| KreuzbergError::validation(format!("Failed to read config file {}: {}", path.display(), e)))?;

        let config: Self = toml::from_str(&content)
            .map_err(|e| KreuzbergError::validation(format!("Invalid TOML in {}: {}", path.display(), e)))?;

        let config_arc = Arc::new(config);
        CONFIG_CACHE.insert(path.to_path_buf(), (mtime, config_arc.clone()));

        Ok((*config_arc).clone())
    }

    /// Load configuration from a YAML file.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let metadata = std::fs::metadata(path)
            .map_err(|e| KreuzbergError::validation(format!("Failed to read config file {}: {}", path.display(), e)))?;
        let mtime = metadata.modified().map_err(|e| {
            KreuzbergError::validation(format!("Failed to get modification time for {}: {}", path.display(), e))
        })?;

        if let Some(entry) = CONFIG_CACHE.get(path)
            && entry.0 == mtime
        {
            return Ok((*entry.1).clone());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| KreuzbergError::validation(format!("Failed to read config file {}: {}", path.display(), e)))?;

        let config: Self = serde_yaml_ng::from_str(&content)
            .map_err(|e| KreuzbergError::validation(format!("Invalid YAML in {}: {}", path.display(), e)))?;

        let config_arc = Arc::new(config);
        CONFIG_CACHE.insert(path.to_path_buf(), (mtime, config_arc.clone()));

        Ok((*config_arc).clone())
    }

    /// Load configuration from a JSON file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let metadata = std::fs::metadata(path)
            .map_err(|e| KreuzbergError::validation(format!("Failed to read config file {}: {}", path.display(), e)))?;
        let mtime = metadata.modified().map_err(|e| {
            KreuzbergError::validation(format!("Failed to get modification time for {}: {}", path.display(), e))
        })?;

        if let Some(entry) = CONFIG_CACHE.get(path)
            && entry.0 == mtime
        {
            return Ok((*entry.1).clone());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| KreuzbergError::validation(format!("Failed to read config file {}: {}", path.display(), e)))?;

        let config: Self = serde_json::from_str(&content)
            .map_err(|e| KreuzbergError::validation(format!("Invalid JSON in {}: {}", path.display(), e)))?;

        let config_arc = Arc::new(config);
        CONFIG_CACHE.insert(path.to_path_buf(), (mtime, config_arc.clone()));

        Ok((*config_arc).clone())
    }

    /// Load configuration from a file, auto-detecting format by extension.
    ///
    /// Supported formats:
    /// - `.toml` - TOML format
    /// - `.yaml` - YAML format
    /// - `.json` - JSON format
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Errors
    ///
    /// Returns `KreuzbergError::Validation` if:
    /// - File doesn't exist
    /// - File extension is not supported
    /// - File content is invalid for the detected format
    ///
    /// # Example
    ///
    /// ```rust
    /// use kreuzberg::core::config::ExtractionConfig;
    ///
    /// // Auto-detects TOML format
    /// // let config = ExtractionConfig::from_file("kreuzberg.toml")?;
    ///
    /// // Auto-detects YAML format
    /// // let config = ExtractionConfig::from_file("kreuzberg.yaml")?;
    /// ```
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let metadata = std::fs::metadata(path)
            .map_err(|e| KreuzbergError::validation(format!("Failed to read config file {}: {}", path.display(), e)))?;
        let mtime = metadata.modified().map_err(|e| {
            KreuzbergError::validation(format!("Failed to get modification time for {}: {}", path.display(), e))
        })?;

        if let Some(entry) = CONFIG_CACHE.get(path)
            && entry.0 == mtime
        {
            return Ok((*entry.1).clone());
        }

        let extension = path.extension().and_then(|ext| ext.to_str()).ok_or_else(|| {
            KreuzbergError::validation(format!(
                "Cannot determine file format: no extension found in {}",
                path.display()
            ))
        })?;

        let config = match extension.to_lowercase().as_str() {
            "toml" => Self::from_toml_file(path)?,
            "yaml" | "yml" => Self::from_yaml_file(path)?,
            "json" => Self::from_json_file(path)?,
            _ => {
                return Err(KreuzbergError::validation(format!(
                    "Unsupported config file format: .{}. Supported formats: .toml, .yaml, .json",
                    extension
                )));
            }
        };

        let config_arc = Arc::new(config);
        CONFIG_CACHE.insert(path.to_path_buf(), (mtime, config_arc.clone()));

        Ok((*config_arc).clone())
    }

    /// Discover a configuration file.
    ///
    /// Resolution order:
    /// 1. `kreuzberg.toml` in the current directory, then each parent up to the
    ///    filesystem root (project-local config wins).
    /// 2. The user-level config directory as a fallback, resolved
    ///    platform-natively via [`dirs::config_dir`] (same approach as the cache
    ///    dir): `$XDG_CONFIG_HOME/kreuzberg/kreuzberg.toml` or
    ///    `~/.config/kreuzberg/kreuzberg.toml` on Linux,
    ///    `~/Library/Application Support/kreuzberg/kreuzberg.toml` on macOS,
    ///    `%APPDATA%\kreuzberg\kreuzberg.toml` on Windows.
    ///
    /// # Returns
    ///
    /// - `Some(config)` if found
    /// - `None` if no config file found
    pub fn discover() -> Result<Option<Self>> {
        let cwd = std::env::current_dir().map_err(KreuzbergError::Io)?;
        let global = dirs::config_dir().map(|dir| dir.join("kreuzberg").join("kreuzberg.toml"));
        Self::discover_from(&cwd, global.as_deref())
    }

    /// Walk up from `start` looking for `kreuzberg.toml`, then fall back to the
    /// user-level `global` config path if provided.
    ///
    /// Split out from [`discover`](Self::discover) so the resolution order is
    /// unit-testable without mutating process-global state (the working
    /// directory or `HOME`).
    fn discover_from(start: &Path, global: Option<&Path>) -> Result<Option<Self>> {
        let mut current = start.to_path_buf();

        loop {
            let kreuzberg_toml = current.join("kreuzberg.toml");
            if kreuzberg_toml.exists() {
                return Ok(Some(Self::from_toml_file(kreuzberg_toml)?));
            }

            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        if let Some(global) = global
            && global.exists()
        {
            return Ok(Some(Self::from_toml_file(global)?));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod discover_tests {
    use super::*;
    use std::fs;

    const TOML: &str = "[ocr]\nenabled = true\n";

    #[test]
    fn project_config_takes_precedence_over_global() {
        // Project config has an [ocr] section; the global fallback does not, so
        // `ocr.is_some()` proves the project-local file was the one loaded.
        let project = tempfile::tempdir().expect("tempdir");
        fs::write(project.path().join("kreuzberg.toml"), TOML).expect("write project");
        let global = tempfile::tempdir().expect("tempdir");
        let global_file = global.path().join("kreuzberg.toml");
        fs::write(&global_file, "output_format = \"markdown\"\n").expect("write global");

        let found = ExtractionConfig::discover_from(project.path(), Some(&global_file))
            .expect("discover")
            .expect("some config");
        assert!(
            found.ocr.is_some(),
            "project-local config must win over the global fallback"
        );
    }

    #[test]
    fn falls_back_to_global_when_no_project_config() {
        let empty = tempfile::tempdir().expect("tempdir");
        let global = tempfile::tempdir().expect("tempdir");
        let global_file = global.path().join("kreuzberg.toml");
        fs::write(&global_file, TOML).expect("write global");

        let found = ExtractionConfig::discover_from(empty.path(), Some(&global_file)).expect("discover");
        assert!(found.is_some(), "should fall back to the global config file");
    }

    #[test]
    fn returns_none_when_no_config_anywhere() {
        let empty = tempfile::tempdir().expect("tempdir");
        let missing = empty.path().join("nope").join("kreuzberg.toml");

        let found = ExtractionConfig::discover_from(empty.path(), Some(&missing)).expect("discover");
        assert!(found.is_none(), "no project or global config should yield None");
    }
}
