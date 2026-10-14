//! Manifest update engine implementation.

use std::path::PathBuf;

use toml_edit::DocumentMut;

/// Engine coordinating updates to `Cargo.toml` manifests using `toml_edit`.
#[derive(Debug, Clone)]
pub struct UpdateEngine {
    #[allow(dead_code)]
    pure_toml: DocumentMut,
    toml_path: PathBuf,
}

impl PartialEq for UpdateEngine {
    fn eq(&self, other: &Self) -> bool {
        self.toml_path == other.toml_path
    }
}

impl Eq for UpdateEngine {}

impl UpdateEngine {
    /// Creates a new `UpdateEngine` instance.
    ///
    /// # Arguments
    /// * `toml` - Parsed mutable TOML document.
    /// * `path` - Filesystem path to the `Cargo.toml` manifest.
    ///
    /// # Returns
    /// An initialized `UpdateEngine` instance.
    pub fn new(toml: DocumentMut, path: PathBuf) -> Self {
        Self {
            pure_toml: toml,
            toml_path: path,
        }
    }
}
