//! Response structures for the crates.io sparse index line-delimited JSON format.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::meta_status;

/// Parsed list of version release items from a crates.io sparse index file.
pub type CratesIndexResponseParsed = Vec<CratesIndexItem>;

/// Represents a single version release record in the crates.io sparse index.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CratesIndexItem {
    /// Name of the crate.
    pub name: String,
    /// Release version string.
    pub vers: String,
    /// List of crate dependencies.
    pub deps: Vec<Dep>,
    /// SHA-256 checksum of the packaged `.crate` tarball.
    pub cksum: String,
    /// Map of available feature flags and their enabled sub-features.
    pub features: HashMap<String, Vec<String>>,
    /// Whether this crate release has been yanked from crates.io.
    pub yanked: bool,
    /// Publication timestamp string (ISO 8601 / RFC 3339).
    pub pubtime: String,
}

impl CratesIndexItem {
    fn parse_item(item: &str) -> Option<CratesIndexItem> {
        if item.is_empty() {
            return None;
        }
        match serde_json::de::from_str::<CratesIndexItem>(item) {
            Ok(parsed_item) => Some(parsed_item),
            Err(serde_json_error) => {
                meta_status!(
                    log::LevelFilter::Error,
                    crate::status_codes::Module::REGISTRYCLIENT,
                    400,
                    "Error while parsing index item.".to_string(),
                    serde_json_error
                );
                None
            }
        }
    }

    /// Parses newline-delimited JSON entries from the sparse index into a collection of [`CratesIndexItem`].
    ///
    /// # Arguments
    /// * `response` - Raw text content returned from the sparse index endpoint.
    ///
    /// # Returns
    /// Vector of successfully deserialized index items.
    pub fn parse_response(response: &str) -> CratesIndexResponseParsed {
        response
            .split("\n")
            .flat_map(CratesIndexItem::parse_item)
            .collect()
    }
}

/// Dependency specification entry in the sparse index.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dep {
    /// Dependency crate name.
    pub name: String,
    /// Version requirement string (e.g. `^1.0`).
    pub req: String,
    /// Features explicitly enabled for this dependency.
    pub features: Vec<String>,
    /// Whether the dependency is optional.
    pub optional: bool,
    /// Whether default features of the dependency are enabled.
    #[serde(rename = "default_features")]
    pub default_features: bool,
    /// Target platform filter expression, or `null`.
    pub target: Value,
    /// Dependency category kind (`normal`, `dev`, or `build`).
    pub kind: String,
}
