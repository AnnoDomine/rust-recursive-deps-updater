//! Response structures for the crates.io REST API (`/api/v1/crates/<crate>`).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top-level response returned by the crates.io crate metadata API.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CratesIOResponse {
    /// Root crate metadata object.
    #[serde(rename = "crate")]
    pub crate_field: Crate,
    /// List of published release versions.
    pub versions: Vec<Version>,
    /// Associated search keywords.
    pub keywords: Vec<Keyword>,
    /// Associated categories.
    pub categories: Vec<Category>,
}

/// Metadata describing a crate on crates.io.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crate {
    /// Unique crate identifier.
    pub id: String,
    /// Package name.
    pub name: String,
    /// Timestamp when the crate was last updated.
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    /// Internal version IDs.
    pub versions: Vec<i64>,
    /// Keyword tags.
    pub keywords: Vec<String>,
    /// Category slugs.
    pub categories: Vec<String>,
    /// Badges configured for the crate.
    pub badges: Vec<Value>,
    /// Timestamp when the crate was first published.
    #[serde(rename = "created_at")]
    pub created_at: String,
    /// Total all-time download count.
    pub downloads: i64,
    /// Download count over the last 90 days.
    #[serde(rename = "recent_downloads")]
    pub recent_downloads: i64,
    /// Default version string.
    #[serde(rename = "default_version")]
    pub default_version: String,
    /// Total number of versions released.
    #[serde(rename = "num_versions")]
    pub num_versions: i64,
    /// Whether the crate has been yanked.
    pub yanked: bool,
    /// Highest version string released.
    #[serde(rename = "max_version")]
    pub max_version: String,
    /// Most recently published version string.
    #[serde(rename = "newest_version")]
    pub newest_version: String,
    /// Highest stable non-prerelease version string.
    #[serde(rename = "max_stable_version")]
    pub max_stable_version: String,
    /// Short crate description.
    pub description: Option<String>,
    /// Project homepage URL.
    pub homepage: Option<String>,
    /// Documentation URL.
    pub documentation: Option<String>,
    /// Source repository URL.
    pub repository: Option<String>,
    /// Navigation endpoints for this crate.
    pub links: Links,
    /// Whether search query matched this crate exactly.
    #[serde(rename = "exact_match")]
    pub exact_match: bool,
    /// Whether crate is restricted to trusted publishing.
    #[serde(rename = "trustpub_only")]
    pub trustpub_only: bool,
}

/// API navigation endpoints for a crate.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    /// Version downloads API endpoint.
    #[serde(rename = "version_downloads")]
    pub version_downloads: String,
    /// Versions API endpoint.
    pub versions: Value,
    /// Crate owners API endpoint.
    pub owners: String,
    /// Owner team API endpoint.
    #[serde(rename = "owner_team")]
    pub owner_team: String,
    /// Owner user API endpoint.
    #[serde(rename = "owner_user")]
    pub owner_user: String,
    /// Reverse dependencies API endpoint.
    #[serde(rename = "reverse_dependencies")]
    pub reverse_dependencies: String,
}

/// Metadata for a specific release version of a crate.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    /// Internal version record identifier.
    pub id: i64,
    /// Crate name.
    #[serde(rename = "crate")]
    pub crate_field: String,
    /// SemVer version string.
    pub num: String,
    /// Download path for the package tarball.
    #[serde(rename = "dl_path")]
    pub dl_path: String,
    /// Path to rendered README.
    #[serde(rename = "readme_path")]
    pub readme_path: String,
    /// Last update timestamp for this version.
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    /// Publication timestamp.
    #[serde(rename = "created_at")]
    pub created_at: String,
    /// Download count for this specific version.
    pub downloads: i64,
    /// Feature flag configuration.
    pub features: HashMap<String, Vec<String>>,
    /// Whether this specific release has been yanked.
    pub yanked: bool,
    /// Optional rationale message when yanked.
    #[serde(rename = "yank_message")]
    pub yank_message: Value,
    /// Shared library linkage data.
    #[serde(rename = "lib_links")]
    pub lib_links: Value,
    /// SPDX license expression.
    pub license: Option<String>,
    /// Navigation endpoints for this version.
    pub links: Links2,
    /// Package file size in bytes.
    #[serde(rename = "crate_size")]
    pub crate_size: i64,
    /// Publisher user information.
    #[serde(rename = "published_by")]
    pub published_by: Option<PublishedBy>,
    /// Audit history events.
    #[serde(rename = "audit_actions")]
    pub audit_actions: Vec<AuditAction>,
    /// SHA-256 package checksum.
    pub checksum: String,
    /// Minimum Supported Rust Version (MSRV).
    #[serde(rename = "rust_version")]
    pub rust_version: Option<String>,
    /// Whether the package contains a library target.
    #[serde(rename = "has_lib")]
    pub has_lib: Option<bool>,
    /// Binary target names provided.
    #[serde(rename = "bin_names")]
    pub bin_names: Option<Vec<Value>>,
    /// Rust edition used.
    pub edition: Option<String>,
    /// Release-specific description.
    pub description: Option<String>,
    /// Homepage URL.
    pub homepage: Option<String>,
    /// Documentation URL.
    pub documentation: Option<String>,
    /// Source code repository URL.
    pub repository: Option<String>,
    /// Trusted publisher provenance information.
    #[serde(rename = "trustpub_data")]
    pub trustpub_data: Value,
    /// Source code line count statistics.
    pub linecounts: Linecounts,
}

/// Navigation endpoints for a specific crate release.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links2 {
    /// Dependencies API endpoint.
    pub dependencies: String,
    /// Version downloads API endpoint.
    #[serde(rename = "version_downloads")]
    pub version_downloads: String,
    /// Authors API endpoint.
    pub authors: String,
}

/// User who published a crate release.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishedBy {
    /// User account ID.
    pub id: i64,
    /// User login username.
    pub login: String,
    /// User display name.
    pub name: Option<String>,
    /// Avatar image URL.
    pub avatar: Option<String>,
    /// Profile URL.
    pub url: Option<String>,
    /// Whether GitHub username matches login.
    #[serde(rename = "github_username_matches")]
    pub github_username_matches: bool,
    /// Account creation timestamp.
    #[serde(rename = "created_at")]
    pub created_at: String,
}

/// Audit event action logged for a release.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditAction {
    /// Action performed (e.g. `publish`, `yank`).
    pub action: String,
    /// User who performed the action.
    pub user: User,
    /// Action timestamp.
    pub time: String,
}

/// User profile details on crates.io.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// User account ID.
    pub id: i64,
    /// Login handle.
    pub login: String,
    /// Display name.
    pub name: Option<String>,
    /// Avatar image URL.
    pub avatar: Option<String>,
    /// Profile URL.
    pub url: Option<String>,
    /// Whether GitHub username matches login.
    #[serde(rename = "github_username_matches")]
    pub github_username_matches: bool,
    /// Account creation timestamp.
    #[serde(rename = "created_at")]
    pub created_at: String,
}

/// Source line count metrics for a crate package.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linecounts {
    /// Language-specific line count breakdowns.
    pub languages: HashMap<String, Languages>,
    /// Total lines of code across all files.
    #[serde(rename = "total_code_lines")]
    pub total_code_lines: i64,
    /// Total lines of comments across all files.
    #[serde(rename = "total_comment_lines")]
    pub total_comment_lines: i64,
}

/// Line counts for a single programming language.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Languages {
    /// Number of code lines.
    #[serde(rename = "code_lines")]
    pub code_lines: i64,
    /// Number of comment lines.
    #[serde(rename = "comment_lines")]
    pub comment_lines: i64,
    /// Number of files written in this language.
    pub files: i64,
}

/// Search keyword associated with a crate.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keyword {
    /// Unique keyword ID.
    pub id: String,
    /// Keyword string.
    pub keyword: String,
    /// Creation timestamp.
    #[serde(rename = "created_at")]
    pub created_at: String,
    /// Number of crates tagged with this keyword.
    #[serde(rename = "crates_cnt")]
    pub crates_cnt: i64,
}

/// Category taxonomy assigned to a crate.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    /// Unique category ID.
    pub id: String,
    /// Human-readable category title.
    pub category: String,
    /// Category URL slug.
    pub slug: String,
    /// Category description.
    pub description: String,
    /// Creation timestamp.
    #[serde(rename = "created_at")]
    pub created_at: String,
    /// Number of crates categorized under this tag.
    #[serde(rename = "crates_cnt")]
    pub crates_cnt: i64,
}
