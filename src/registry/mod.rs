//! Registry communication, cache management, and response models.
//!
//! Provides the HTTP client for crates.io index/API and RustSec queries, disk-backed caching,
//! and strongly typed response deserialization structures.

/// Disk and memory caching registry.
pub mod cache_registry;
/// Crates.io and RustSec HTTP client.
pub mod client;

/// Deserialization schemas for crates.io sparse index records.
pub mod crates_index_response_structs;
/// Deserialization schemas for crates.io REST API JSON responses.
pub mod crates_io_response_structs;
/// Deserialization schemas for RustSec OSV/JSON advisory responses.
pub mod rust_sec_json_response_structs;
