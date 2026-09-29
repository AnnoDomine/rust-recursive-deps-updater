//! This file holds every global constant values

// Static file names included extension

use std::time::Duration;

// RRDU Config file
pub const RRDUCONFIG_FILE_NAME: &str = ".rrduconfig";
// Cargo toml file
pub const CARGO_TOML_FILE_NAME: &str = "Cargo.toml";
// RRDU ignore file
pub const RRDUIGNORE_FILE_NAME: &str = ".rrduignore";
// GitHub ignore file
pub const GITIGNORE_FILE_NAME: &str = ".gitignore";
// RRDU cache files
pub const RRDU_REGISTRY_CACHE_CRATES_IO_API: &str = "rrdu_crates_io_api_cache.bin";
pub const RRDU_REGISTRY_CACHE_CRATES_IO_INDEX: &str = "rrdu_crates_io_index_cache.bin";
pub const RRDU_REGISTRY_CACHE_RUSTSEC_API_IDS: &str = "rrdu_rustsec_cache.bin";
// RRDU log file
pub const RRDU_LOG: &str = "rrdu.log";

// crates.io provides a guideline to make requests to there API (See: https://crates.io/data-access)
// Cache TTL 1 day as crates.io dumps db every 24 hours.
pub const CACHE_ENTRY_CRATES_IO_API_TTL: Duration = Duration::from_millis(1000 * 60 * 60 * 24);
pub const CACHE_ENTRY_CRATES_IO_INDEX_TTL: Duration = Duration::from_millis(1000 * 60 * 60 * 24);
pub const CACHE_ENTRY_RUSTSEC_API_TTL: Duration = Duration::from_millis(1000 * 60 * 60 * 6);
// Request timeout till next fetch (1 sec)
pub const TIME_BETWEEN_REQUESTS: Duration = Duration::from_millis(1000);
// Endpoints
pub const CRATE_IO_INDEX_URL: &str = "https://index.crates.io";
pub const CRATE_IO_API_URL: &str = "https://crates.io/api/v1/crates";
pub const RUSTSEC_API_URL: &str = "https://rustsec.org/packages";
// Default user-agent header
pub const DEFAULT_USER_AGENT_HEADER_CRATES_IO_URL: &str =
    "https://crates.io/crates/rust-recursive-deps-updater";
pub const DEFAULT_USER_AGENT_HEADER_GITHUB_URL: &str =
    "https://github.com/AnnoDomine/rust-recursive-deps-updater";
pub const DEFAULT_USER_AGENT_HEADER_APPLICATION: &str = "rust-recursive-dependency-updater@";
pub const DEFAULT_USER_AGENT_HEADER_TYPE_AVAILABILITY: &str = " - check dependency availability ";
pub const DEFAULT_USER_AGENT_HEADER_TYPE_FULL_INFORMATION: &str = " - get dependency information ";
pub const DEFAULT_USER_AGENT_HEADER_TYPE_AUDIT: &str = " - get dependency audit information ";

pub const UNSUPPORTED_DEPENDENCY_KEYS: [&str; 4] = ["git", "path", "registry", "workspace"];
pub const MIN_REQUIRED_DEPENDENCY_KEYS: [&str; 1] = ["version"];

pub const DEPENDENCY_SECTION_SUFFIX: [&str; 3] =
    ["dependencies", "dev-dependencies", "build-dependencies"];
pub const DEPENDENCY_SECTION_PREFIX: [&str; 5] = [
    "workspace",
    "target",
    "dependencies",
    "dev-dependencies",
    "build-dependencies",
];
