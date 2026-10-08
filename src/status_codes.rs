//! This file handles every client codes.
//! Beside the default http status codes, rrdu includes an own status code which use following patterns:
//! `<MODULE NUMBER ID><STATUS NUMBER>`
//! Modules have there own identification numbers which are always 2 digits:
//! - Configuration => 1YXX
//! - Workspace => 2YXX
//! - Discovery => 3YXX
//! - Registry Client => 4YXX (41XX - 45XX => equivalent to HTTP status codes)
//! - Updater => 5YXX
//! - Cache => 6YXX
//!
//! Status codes which have '9' at the second place are alway security relevant status codes (excluded 41XX - 45XX as these codes are responsed from http requests).
//! Security relevant codes are counted by levels:
//! - 0 => Low
//! - 2 => Relative
//! - 5 => Medium
//! - 7 => High
//! - 9 => Extrem

use std::fmt::Display;

use log::LevelFilter;
use serde::{Deserialize, Serialize};

/// Represents the sub-modules within `rrdu` that report status codes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Module {
    /// Configuration parsing, validation, and serialization (`1YXX`).
    CONFIGURATION,
    /// Workspace and manifest handling (`2YXX`).
    WORKSPACE,
    /// Filesystem directory crawling and project discovery (`3YXX`).
    DISCOVERY,
    /// Crates.io and registry network operations (`4YXX`).
    REGISTRYCLIENT,
    /// Cargo.toml dependency updating and toml_edit writing (`5YXX`).
    UPDATER,
    /// Cache operations and disk storage (`6YXX`).
    CACHE,
    /// CLI render and generating (`7YXX`).
    CLI,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Module::CONFIGURATION => write!(f, "RRDU Configuration Module"),
            Module::WORKSPACE => write!(f, "RRDU Workspace Module"),
            Module::DISCOVERY => write!(f, "RRDU Discovery Module"),
            Module::REGISTRYCLIENT => write!(f, "RRDU Registry Client Module"),
            Module::UPDATER => write!(f, "RRDU Updater Module"),
            Module::CACHE => write!(f, "RRDU Cache Module"),
            Module::CLI => write!(f, "RRDU CLI Module"),
        }
    }
}

impl Module {
    /// Returns the numeric module identifier (1 through 6).
    pub fn get_module_code(&self) -> u8 {
        match self {
            Module::CONFIGURATION => 1u8,
            Module::WORKSPACE => 2u8,
            Module::DISCOVERY => 3u8,
            Module::REGISTRYCLIENT => 4u8,
            Module::UPDATER => 5u8,
            Module::CACHE => 6u8,
            Module::CLI => 7u8,
        }
    }
}

/// Structured status code entry containing module identification, status code, message, and optional metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCodeSchema<T = ()> {
    /// The module issuing this status code.
    pub module: Module,
    /// The specific status code within the module's range.
    pub code: u32,
    /// Descriptive message associated with this status event.
    pub message: String,
    /// Optional metadata payload providing contextual details.
    pub meta: Option<T>,
}

impl<T: Display> Display for StatusCodeSchema<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formated_status_code = format!(
            "{:} [{:}{:}]",
            self.module,
            self.module.get_module_code(),
            self.code
        );
        let meta = match &self.meta {
            Some(m) => format!("\n{:#?}", m.to_string()),
            None => "".to_string(),
        };
        write!(f, "{formated_status_code} | {:}{:}", self.message, meta)
    }
}

impl<T: Display> StatusCodeSchema<T> {
    /// Creates a new status code entry with attached metadata.
    ///
    /// # Arguments
    /// * `module` - The originating `rrdu` module.
    /// * `code` - The specific status code.
    /// * `message` - Informational status message.
    /// * `meta` - Additional metadata payload.
    pub fn with_meta(module: Module, code: u32, message: impl Into<String>, meta: T) -> Self {
        Self {
            module,
            code,
            message: message.into(),
            meta: Some(meta),
        }
    }

    /// Creates a simple status code entry without metadata.
    ///
    /// # Arguments
    /// * `module` - The originating `rrdu` module.
    /// * `code` - The specific status code.
    /// * `message` - Informational status message.
    pub fn simple(module: Module, code: u32, message: impl Into<String>) -> Self {
        Self {
            module,
            code,
            message: message.into(),
            meta: None,
        }
    }

    /// Dispatches this status code event to the `log` facade at the specified log level.
    ///
    /// # Arguments
    /// * `level` - The target `log::LevelFilter` (e.g. `LevelFilter::Info`, `LevelFilter::Debug`).
    pub fn log(&self, level: LevelFilter) {
        match level {
            LevelFilter::Error => log::error!("{:}", self),
            LevelFilter::Warn => log::warn!("{:}", self),
            LevelFilter::Info => log::info!("{:}", self),
            LevelFilter::Debug => log::debug!("{:}", self),
            LevelFilter::Trace => log::trace!("{:}", self),
            LevelFilter::Off => {}
        }
    }
}

/// Logs a simple status code message without additional metadata.
///
/// # Arguments
/// * `$level` - The `log::LevelFilter` (e.g. `LevelFilter::Info`).
/// * `$module` - The `Module` identifier (e.g. `Module::DISCOVERY`).
/// * `$code` - The numeric status code.
/// * `$message` - The log message string.
#[macro_export]
macro_rules! simple_status {
    ($level:expr, $module:expr, $code:expr, $message:expr) => {{
        let status =
            $crate::status_codes::StatusCodeSchema::<String>::simple($module, $code, $message);
        status.log($level);
    }};
}

/// Logs a status code message with contextual metadata.
///
/// # Arguments
/// * `$level` - The `log::LevelFilter` (e.g. `LevelFilter::Warn`).
/// * `$module` - The `Module` identifier (e.g. `Module::CACHE`).
/// * `$code` - The numeric status code.
/// * `$message` - The log message string.
/// * `$meta` - Metadata implementing `Display`.
#[macro_export]
macro_rules! meta_status {
    ($level:expr, $module:expr, $code:expr, $message:expr, $meta:expr) => {{
        let status =
            $crate::status_codes::StatusCodeSchema::with_meta($module, $code, $message, $meta);
        status.log($level);
    }};
}
