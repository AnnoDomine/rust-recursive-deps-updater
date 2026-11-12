//! Domain error types for `rrdu`.
//!
//! Provides typed error hierarchies for configuration parsing, manifest I/O,
//! file path validation, and registry version collection.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Combined error type representing either a configuration or manifest file error.
pub enum CombinedConfigFileError {
    /// Error originating from configuration loading or validation.
    ConfigError,
    /// Error originating from manifest I/O or TOML parsing.
    FileError,
}

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

    /// TOML parsing or deserialization failure from toml_edit.
    #[error("TOML configuration deserialization error: {0}")]
    TomlDe(#[from] toml_edit::de::Error),

    /// TOML serialization failure from toml_edit.
    #[error("TOML configuration serialization error: {0}")]
    TomlSer(#[from] toml_edit::ser::Error),

    /// TOML document parsing error from toml_edit.
    #[error("TOML document parsing error: {0}")]
    Toml(#[from] toml_edit::TomlError),

    /// SemVer error
    #[error("SemVer error: {0}")]
    SemVer(#[from] semver::Error),

    /// 'self_update' error
    #[error("self_update error: {0}")]
    SelfUpdate(#[from] self_update::errors::Error),

    /// YAML rrduconfig version incompatible error
    #[error(
        "Incompatible configuration version: found '{found}', required at least '{required}' or newer. See MIGRATION.md."
    )]
    IncompatibleVersion {
        /// Version string found in the configuration file.
        found: String,
        /// Minimum version string required by the running `rrdu` binary.
        required: String,
    },

    /// YAML rrduconfig version missing error
    #[error("Legacy configuration detected without version field. See MIGRATION.md.")]
    LegacyConfiguration,

    /// Rust-std lib error
    #[error("Std error: {0}")]
    StdErr(#[from] Box<dyn std::error::Error>),

    /// Ureq error
    #[error("UREQ error: {0}")]
    Ureq(#[from] ureq::Error),
}

/// Errors that can occur when reading the 'Cargo.toml' files
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    /// IFile read error
    #[error("Failed to read manifest at '{path}': {source}")]
    Io {
        /// Path to the manifest file that failed to read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// TOML parse error
    #[error("Failed to parse manifest at '{path}': {source}")]
    Toml {
        /// Path to the manifest file containing invalid TOML.
        path: PathBuf,
        /// The underlying TOML parse error from `toml_edit`.
        #[source]
        source: toml_edit::TomlError,
    },

    /// Rise error if a file is applied to the path and it is not the 'Cargo.toml'.
    #[error("Your provided file in the path '{path}' is not valid. Required '{file_name}'")]
    InvalidFileName {
        /// Path containing the unexpected file name.
        path: PathBuf,
        /// Expected file name.
        file_name: String,
    },

    /// Rise if an error occure from the config file.
    #[error("{0}")]
    ConfigError(ConfigError),
}

/// Reserved placeholder enum for typed `rrdu` numeric error codes.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum RrduErrorCodes {}

/// Errors encountered while collecting dependency versions from `crates.io`.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize, Ord, PartialOrd)]
pub enum CollectionVersionError {
    /// Dependency not found error
    #[error("Dependency could not found on 'crates.io'")]
    NotFound,

    /// Other errors from fetching
    #[error("Error while try to get version from 'crates.io': {0}")]
    Other(String),
}

/// Result type with attached `ConfigError` as `Err(..)`
pub type RrduConfigResult<T> = Result<T, ConfigError>;
