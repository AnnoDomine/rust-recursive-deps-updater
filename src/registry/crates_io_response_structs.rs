use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CratesIOResponse {
    #[serde(rename = "crate")]
    pub crate_field: Crate,
    pub versions: Vec<Version>,
    pub keywords: Vec<Keyword>,
    pub categories: Vec<Category>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crate {
    pub id: String,
    pub name: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    pub versions: Vec<i64>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub badges: Vec<Value>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub downloads: i64,
    #[serde(rename = "recent_downloads")]
    pub recent_downloads: i64,
    #[serde(rename = "default_version")]
    pub default_version: String,
    #[serde(rename = "num_versions")]
    pub num_versions: i64,
    pub yanked: bool,
    #[serde(rename = "max_version")]
    pub max_version: String,
    #[serde(rename = "newest_version")]
    pub newest_version: String,
    #[serde(rename = "max_stable_version")]
    pub max_stable_version: String,
    pub description: String,
    pub homepage: String,
    pub documentation: Value,
    pub repository: String,
    pub links: Links,
    #[serde(rename = "exact_match")]
    pub exact_match: bool,
    #[serde(rename = "trustpub_only")]
    pub trustpub_only: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    #[serde(rename = "version_downloads")]
    pub version_downloads: String,
    pub versions: Value,
    pub owners: String,
    #[serde(rename = "owner_team")]
    pub owner_team: String,
    #[serde(rename = "owner_user")]
    pub owner_user: String,
    #[serde(rename = "reverse_dependencies")]
    pub reverse_dependencies: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: i64,
    #[serde(rename = "crate")]
    pub crate_field: String,
    pub num: String,
    #[serde(rename = "dl_path")]
    pub dl_path: String,
    #[serde(rename = "readme_path")]
    pub readme_path: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub downloads: i64,
    pub features: Features,
    pub yanked: bool,
    #[serde(rename = "yank_message")]
    pub yank_message: Value,
    #[serde(rename = "lib_links")]
    pub lib_links: Value,
    pub license: String,
    pub links: Links2,
    #[serde(rename = "crate_size")]
    pub crate_size: i64,
    #[serde(rename = "published_by")]
    pub published_by: PublishedBy,
    #[serde(rename = "audit_actions")]
    pub audit_actions: Vec<AuditAction>,
    pub checksum: String,
    #[serde(rename = "rust_version")]
    pub rust_version: Value,
    #[serde(rename = "has_lib")]
    pub has_lib: bool,
    #[serde(rename = "bin_names")]
    pub bin_names: Vec<String>,
    pub edition: String,
    pub description: String,
    pub homepage: String,
    pub documentation: Value,
    pub repository: String,
    #[serde(rename = "trustpub_data")]
    pub trustpub_data: Value,
    pub linecounts: Linecounts,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Features {}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links2 {
    pub dependencies: String,
    #[serde(rename = "version_downloads")]
    pub version_downloads: String,
    pub authors: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishedBy {
    pub id: i64,
    pub login: String,
    pub name: String,
    pub avatar: String,
    pub url: String,
    #[serde(rename = "github_username_matches")]
    pub github_username_matches: bool,
    #[serde(rename = "created_at")]
    pub created_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditAction {
    pub action: String,
    pub user: User,
    pub time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub login: String,
    pub name: String,
    pub avatar: String,
    pub url: String,
    #[serde(rename = "github_username_matches")]
    pub github_username_matches: bool,
    #[serde(rename = "created_at")]
    pub created_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Linecounts {
    pub languages: Languages,
    #[serde(rename = "total_code_lines")]
    pub total_code_lines: i64,
    #[serde(rename = "total_comment_lines")]
    pub total_comment_lines: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Languages {
    #[serde(rename = "Rust")]
    pub rust: Rust,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rust {
    #[serde(rename = "code_lines")]
    pub code_lines: i64,
    #[serde(rename = "comment_lines")]
    pub comment_lines: i64,
    pub files: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keyword {
    pub id: String,
    pub keyword: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "crates_cnt")]
    pub crates_cnt: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub category: String,
    pub slug: String,
    pub description: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "crates_cnt")]
    pub crates_cnt: i64,
}
