//! `rrdu` Core Library

#![forbid(unsafe_code)]

/// CLI related structs and impls for render and controll interactive and headless CLI
pub mod cli;
/// Workspace configuration loading, generation, and validation.
pub mod config;
/// Crates.io index/API communication, cache, and response models.
pub mod registry;
/// Workspace project discovery, TOML parsing, and dependency mapping.
pub mod workspace;

/// Global application constants.
pub mod constants;
/// Domain enums and data types.
pub mod enums;
/// Error types and error codes.
pub mod errors;
/// Path and validation utility functions.
pub mod functions;
/// Structured status code definitions and logging macros.
pub mod status_codes;
