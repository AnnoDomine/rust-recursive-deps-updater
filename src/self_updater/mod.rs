//! Self-updater subsystem for querying newer versions and updating the `rrdu` binary.

/// Core self-update engine, GitHub Releases metadata, and release status tracking.
pub mod engine;

/// Version check and update controller
pub mod controller;
