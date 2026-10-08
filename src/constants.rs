//! Global constant definitions for `rrdu`.
//!
//! Includes default filenames, cache filenames, TTL durations, registry API endpoints,
//! User-Agent header templates, and dependency key filters.

use std::time::Duration;

/// Configuration filename for `rrdu`.
pub const RRDUCONFIG_FILE_NAME: &str = ".rrduconfig";

/// Cargo manifest filename.
pub const CARGO_TOML_FILE_NAME: &str = "Cargo.toml";

/// Ignore pattern filename for `rrdu`.
pub const RRDUIGNORE_FILE_NAME: &str = ".rrduignore";

/// Git ignore pattern filename.
pub const GITIGNORE_FILE_NAME: &str = ".gitignore";

/// Cache binary filename for crates.io API metadata.
pub const RRDU_REGISTRY_CACHE_CRATES_IO_API: &str = "rrdu_crates_io_api_cache.bin";

/// Cache binary filename for crates.io sparse index data.
pub const RRDU_REGISTRY_CACHE_CRATES_IO_INDEX: &str = "rrdu_crates_io_index_cache.bin";

/// Cache binary filename for RustSec advisory data.
pub const RRDU_REGISTRY_CACHE_RUSTSEC_API_IDS: &str = "rrdu_rustsec_cache.bin";

/// Default log filename for `rrdu`.
pub const RRDU_LOG: &str = "rrdu.log";

/// Cache TTL for crates.io API entries (24 hours).
///
/// See <https://crates.io/data-access> for crates.io crawl policy.
pub const CACHE_ENTRY_CRATES_IO_API_TTL: Duration = Duration::from_millis(1000 * 60 * 60 * 24);

/// Cache TTL for crates.io sparse index entries (24 hours).
pub const CACHE_ENTRY_CRATES_IO_INDEX_TTL: Duration = Duration::from_millis(1000 * 60 * 60 * 24);

/// Cache TTL for RustSec advisory entries (6 hours).
pub const CACHE_ENTRY_RUSTSEC_API_TTL: Duration = Duration::from_millis(1000 * 60 * 60 * 6);

/// Mandatory delay between consecutive registry requests to respect rate limits.
pub const TIME_BETWEEN_REQUESTS: Duration = Duration::from_millis(1000);

/// Base URL for the crates.io sparse index.
pub const CRATE_IO_INDEX_URL: &str = "https://index.crates.io";

/// Base URL for the crates.io REST API.
pub const CRATE_IO_API_URL: &str = "https://crates.io/api/v1/crates";

/// Base URL for the RustSec vulnerability database.
pub const RUSTSEC_API_URL: &str = "https://rustsec.org/packages";

/// User-Agent URL pointing to `rrdu` on crates.io.
pub const DEFAULT_USER_AGENT_HEADER_CRATES_IO_URL: &str =
    "https://crates.io/crates/rust-recursive-deps-updater";

/// User-Agent URL pointing to the `rrdu` GitHub repository.
pub const DEFAULT_USER_AGENT_HEADER_GITHUB_URL: &str =
    "https://github.com/AnnoDomine/rust-recursive-deps-updater";

/// User-Agent application prefix.
pub const DEFAULT_USER_AGENT_HEADER_APPLICATION: &str = "rust-recursive-dependency-updater@";

/// User-Agent purpose suffix for index availability checks.
pub const DEFAULT_USER_AGENT_HEADER_TYPE_AVAILABILITY: &str = " - check dependency availability ";

/// User-Agent purpose suffix for full metadata queries.
pub const DEFAULT_USER_AGENT_HEADER_TYPE_FULL_INFORMATION: &str = " - get dependency information ";

/// User-Agent purpose suffix for security audit queries.
pub const DEFAULT_USER_AGENT_HEADER_TYPE_AUDIT: &str = " - get dependency audit information ";

/// Manifest keys that indicate non-crates.io or unsupported dependencies.
pub const UNSUPPORTED_DEPENDENCY_KEYS: [&str; 4] = ["git", "path", "registry", "workspace"];

/// Minimum keys required for a dependency entry to be valid.
pub const MIN_REQUIRED_DEPENDENCY_KEYS: [&str; 1] = ["version"];

/// Valid suffix names for dependency sections in `Cargo.toml`.
pub const DEPENDENCY_SECTION_SUFFIX: [&str; 3] =
    ["dependencies", "dev-dependencies", "build-dependencies"];

/// Valid prefix names for dependency sections in `Cargo.toml`.
pub const DEPENDENCY_SECTION_PREFIX: [&str; 5] = [
    "workspace",
    "target",
    "dependencies",
    "dev-dependencies",
    "build-dependencies",
];

/// Terminal max width
pub const TERMINAL_MAX_WIDTH: u16 = 100;
