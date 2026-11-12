//! Configuration loading, schema representation, and validation for `rrdu`.

/// Migration logic from legacy `.rrduconfig` to modern `rrdu.toml`.
pub mod migrate;
pub mod model;
pub mod rrdu_config;
/// TOML configuration schema, data models, and parser for `rrdu.toml`.
pub mod rrdu_toml;
