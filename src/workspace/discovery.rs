//! The module holds the full discovery logic
//!
//! It integrates:
//! - Crawling to the workspace and the sub folders to find the Cargo.toml
//! - Ignores hidden folders (e.g. .git) and 'target' by default
//! - It respects the .gitignore and collect the values to ignore from discovery
//!
//! The Discovery flow is:
//! - Enter a folder
//! - Check for '.rrduconfig'
//! - If found, read config and include it. Skip the rest as the config should be the 'single source of true'
//! - Search for 'Cargo.toml'
//! - If found, read it and collect the project dependencies
//! - Check for '.gitignore'
//! - If found, read '.gitignore' and collect all values for this folder discovery to skip them when discover
//! - Check for '.rrduignore'
//! - If found, read '.rrduignore' and collect all values for this folder discovery to skip them when discover
//! - Enter the next folder if it is not ignored
//! - Repeat till no sub-folder are present

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discovery {
    ignore: Vec<String>,
    has_toml: bool,
}

impl Default for Discovery {
    fn default() -> Self {
        Self::new()
    }
}

impl Discovery {
    pub fn new() -> Self {
        let mut discovery = Self {
            ignore: Vec::new(),
            has_toml: false,
        };
        discovery.scan_for_toml();
        discovery.ignore_by_default();
        discovery
    }

    fn scan_for_toml(&mut self) {}

    fn list_ignore_by_default_folders(&mut self) {
        // Run to folder in same level and collect every folder which starts with a "."

        if self.has_toml {
            // Ignore folders which are hidden as source code folder or build folder from rust
            // Only add them, if a Cargo.toml is present
            self.ignore.push("target".to_string());
            self.ignore.push("src".to_string());
        }
    }

    fn parse_ignore_files(&mut self) {}

    fn ignore_by_default(&mut self) {
        self.list_ignore_by_default_folders();
        self.parse_ignore_files();
    }
}
