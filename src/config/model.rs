//! Configuration data models and serialization schemas.
//!
//! Defines [`ProjectConfig`], [`ExcludeConfig`], and [`UpdaterConfig`] models matching
//! the `.rrduconfig` YAML format, including path resolution and dependency exclusion helpers.

use std::{collections::HashMap, env, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{constants::*, enums::*, errors::*, functions::*};

/// Dependency exclusion rules configured for a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ExcludeConfig {
    /// Project-wide excluded dependency names.
    #[serde(default)]
    pub project: Vec<String>,
    /// Section-specific excluded dependencies mapped by section header name.
    #[serde(default)]
    pub section: HashMap<String, Vec<String>>,
}

/// Configuration settings for an individual project or crate within a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectConfig {
    /// Name of the project.
    /// Can be:
    /// - `name` key from Cargo.toml
    /// - Last folder from the discovery, if it is a sub-config project
    pub project: String,
    /// Path to `Cargo.toml` or `.rrduconfig` if sub-config.
    #[serde(alias = "toml")]
    pub path: String,
    /// List of dependencies excluded from updating.
    #[serde(default)]
    pub exclude: ExcludeConfig,
    /// Identifier indicating if the project delegates to a sub-project `.rrduconfig`.
    #[serde(default)]
    pub sub_config: SubConfig,
}

impl ProjectConfig {
    /// Creates a new project configuration.
    ///
    /// # Arguments
    /// * `project` - The name of the project or crate.
    /// * `path` - Relative path to the project manifest or sub-config.
    pub fn new(project: String, path: String) -> Self {
        Self {
            project,
            path,
            exclude: ExcludeConfig {
                project: Vec::new(),
                section: HashMap::new(),
            },
            sub_config: SubConfig::default(),
        }
    }

    /// Sets whether this project delegates to a sub-configuration file.
    ///
    /// # Arguments
    /// * `is_sub_config` - Boolean flag indicating sub-config status.
    pub fn set_sub_config(&mut self, is_sub_config: bool) {
        self.sub_config = SubConfig::Bool(is_sub_config);
    }

    /// Sets the sub-configuration variant for this project.
    ///
    /// # Arguments
    /// * `sub_config` - The `SubConfig` enum variant.
    pub fn define_sub_config(&mut self, sub_config: SubConfig) {
        self.sub_config = sub_config;
    }

    /// Resolves and validates the path to the sub-project's `.rrduconfig` file.
    ///
    /// # Returns
    /// Validated `PathBuf` pointing to `.rrduconfig`.
    ///
    /// # Errors
    /// Returns [`FileError::ConfigError`] if path traversal is detected.
    /// Returns [`FileError::InvalidFileName`] if the filename is invalid.
    pub fn get_sub_config_path(&self) -> Result<PathBuf, FileError> {
        match validate_path_traversal(&PathBuf::from(&self.path)) {
            Ok(p) => validate_path_and_file_name(&p, RRDUCONFIG_FILE_NAME),
            Err(e) => Err(FileError::ConfigError(e)),
        }
    }

    /// Adds a dependency name to the exclusion rules.
    ///
    /// # Arguments
    /// * `area` - The scope of exclusion (project-wide or specific section).
    /// * `dep` - Optional dependency name to exclude.
    pub fn exclude_dep(&mut self, area: ExcludeArea, dep: Option<String>) {
        match area {
            ExcludeArea::Project => {
                if let Some(named_dep) = dep {
                    self.exclude.project.push(named_dep);
                }
            }
            ExcludeArea::Section(sec) => self.format_section_dep(sec, dep),
        }
    }

    fn format_section_dep(&mut self, sec: DependencySection, dep: Option<String>) {
        match sec {
            DependencySection::Table { .. } => {
                self.exclude.section.entry(sec.to_string()).or_default();
            }
            _ => match dep {
                Some(named_dep) => {
                    self.exclude
                        .section
                        .entry(sec.to_string())
                        .or_default()
                        .push(named_dep);
                }
                None => {
                    self.exclude.section.entry(sec.to_string()).or_default();
                }
            },
        }
    }

    /// Checks whether a dependency entry is excluded at project or section level and marks it accordingly.
    ///
    /// # Arguments
    /// * `entry` - The dependency entry to evaluate.
    /// * `section` - Optional section name containing the dependency.
    pub fn is_dep_excluded(&self, entry: &mut DependencyEntry, section: Option<String>) {
        let in_project = &self.exclude.project;
        let in_section = &self.exclude.section;
        let key = entry.package();
        if let Some(section_name) = section
            && let Some(sec) = in_section.get(&section_name)
            && sec.contains(&key.to_string())
        {
            entry.exclude_dep(&ExcludeReason::ExcludedInSection);
        } else if in_project.contains(&key.to_string()) {
            entry.exclude_dep(&ExcludeReason::ExcludedInProject);
        };
    }
}

/// Global updater settings controlling automated scans, updates, and UI display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UpdaterConfig {
    /// List of excluded projects from the workspace list.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Automatic update mode (`none`, `semver-safe`, or `full`).
    #[serde(default = "default_auto_update")]
    pub auto_update: String,
    /// Whether to automatically scan for outdated dependencies on start.
    #[serde(default = "default_auto_scan")]
    pub auto_scan: bool,
    /// Maximum line display limit for pagination in interactive CLI.
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    /// Schema version string of the configuration file.
    #[serde(default)]
    pub version: Option<String>,
}

/// Default value for updater
fn default_auto_update() -> String {
    "none".to_string()
}
fn default_auto_scan() -> bool {
    true
}
fn default_max_lines() -> usize {
    10
}

/// Returns the current running `rrdu` package version string.
pub fn get_rrdu_version() -> String {
    let current = env!("CARGO_PKG_VERSION");
    String::from(current)
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            auto_update: default_auto_update(),
            auto_scan: default_auto_scan(),
            max_lines: default_max_lines(),
            version: Some(get_rrdu_version()),
        }
    }
}

impl UpdaterConfig {
    /// Adds a project name to the updater exclusion list.
    ///
    /// # Arguments
    /// * `project` - The name of the project to exclude from updates.
    pub fn exclude_project(&mut self, project: String) {
        self.exclude.push(project);
    }
}

#[cfg(test)]
mod test_rrdu_config {
    use super::*;

    #[test]
    fn test_project_config_exclude_dep() {
        let mut project = ProjectConfig::new("Core".to_string(), "./crates/core".to_string());
        assert!(project.exclude.project.is_empty());
        assert!(project.exclude.section.is_empty());

        project.exclude_dep(ExcludeArea::Project, Some("tokio".to_string()));
        project.exclude_dep(ExcludeArea::Project, Some("serde".to_string()));

        project.exclude_dep(
            ExcludeArea::Section(DependencySection::Dev),
            Some("clap".to_string()),
        );
        project.exclude_dep(
            ExcludeArea::Section(DependencySection::Dev),
            Some("tokio".to_string()),
        );
        project.exclude_dep(
            ExcludeArea::Section(DependencySection::Table {
                parent: Box::new(DependencySection::Dev),
                toml_key: "clap".to_string(),
            }),
            Some("clap".to_string()),
        );

        assert_eq!(project.exclude.project, vec!["tokio", "serde"]);
        assert_eq!(
            project.exclude.section[&DependencySection::Dev.to_string()],
            vec!["clap", "tokio"]
        );
        assert_eq!(
            project.exclude.section[&DependencySection::Table {
                parent: Box::new(DependencySection::Dev),
                toml_key: "clap".to_string(),
            }
            .to_string()],
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_updater_config_defaults_and_exclude() {
        let mut updater = UpdaterConfig::default();
        assert_eq!(updater.auto_update, "none");
        assert!(updater.auto_scan);
        assert_eq!(updater.max_lines, 10);
        assert!(updater.exclude.is_empty());

        updater.exclude_project("crates/vendor".to_string());
        assert_eq!(updater.exclude, vec!["crates/vendor"]);
    }

    #[test]
    fn test_project_config_get_sub_config_path() {
        let valid_project = ProjectConfig::new("sub".to_string(), "crates/sub".to_string());
        assert_eq!(
            valid_project.get_sub_config_path().unwrap(),
            PathBuf::from("crates/sub").join(RRDUCONFIG_FILE_NAME)
        );

        let insecure_project = ProjectConfig::new("sub".to_string(), "../sub".to_string());
        assert!(insecure_project.get_sub_config_path().is_err());
    }
}
