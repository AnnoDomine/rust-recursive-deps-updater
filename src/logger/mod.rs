//! Trace logging and diagnostic reporting module for `rrdu`.

#[allow(clippy::module_inception)]
/// Core asynchronous logger and background worker thread implementation.
pub mod logger;
/// Structured status code definitions and logging macros.
pub mod status_codes;
