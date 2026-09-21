use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when loading, parsing, or validating configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Rejection of parent paths (`..`) or root paths (`/`, `C:\`).
    #[error(
        "Insecure path detected: '{0}'. Only relative paths within the workspace are permitted."
    )]
    InsecurePath(PathBuf),

    /// I/O error when reading or creating the configuration file.
    #[error("Configuration I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing or serialization failure from noyalib.
    #[error("YAML configuration error: {0}")]
    Yaml(#[from] noyalib::Error),
}
