//! Modern TOML configuration schema and loader for `rrdu.toml`.
//!
//! Provides [`RrduTomlConfig`](crate::config::rrdu_toml::RrduTomlConfig) representing the root `rrdu.toml` file,
//! model types for workspaces and exclude patterns, and loading/saving routines.

#![allow(deprecated)]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    config::{
        migrate::MigrateYamlToTomlConfig,
        model::{ExcludeConfig, ProjectConfig},
    },
    constants::{RRDU_TOML_FILE_NAME, RRDUCONFIG_FILE_NAME},
    enums::{ConfigCompatibility, DependencyEntry, ExcludeReason, SubConfig},
    errors::ConfigError,
    functions::{create_absolute_path, validate_path_traversal},
    logger::status_codes::Module,
    meta_status, simple_status,
    workspace::discovery::Discovery,
};

/// Exclusion rules for workspace dependencies in `rrdu.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TomlWorkspaceExclude {
    /// Optional list of project-wide excluded dependency names.
    #[serde(default)]
    pub project: Option<Vec<String>>,
    /// Optional mapping of section names to lists of excluded dependencies.
    #[serde(default)]
    pub section: Option<HashMap<String, Vec<String>>>,
}

impl TomlWorkspaceExclude {
    /// Adds a dependency name to the project-wide exclusion list.
    pub fn add_project_dep(&mut self, dep: &str) {
        match &mut self.project {
            Some(ed) => {
                if !ed.contains(&dep.to_string()) {
                    ed.push(dep.to_string())
                }
            }
            None => {
                self.project = Some(vec![dep.to_string()]);
            }
        }
    }

    /// Removes a dependency name from the project-wide exclusion list.
    pub fn remove_project_dep(&mut self, dep: &str) {
        if let Some(ed) = &mut self.project
            && let Some(idx) = ed.iter().position(|d| *d == dep)
        {
            ed.remove(idx);
            if ed.is_empty() {
                self.project = None;
            }
        }
    }

    /// Adds a section key to the section exclusion map.
    pub fn add_section(&mut self, sec: &str) {
        match &mut self.section {
            Some(sections) => {
                sections.entry(sec.to_string()).or_default();
            }
            None => self.section = Some(HashMap::from([(sec.to_string(), Vec::new())])),
        }
    }

    /// Adds a dependency name to a specific section exclusion list.
    pub fn add_section_dep(&mut self, sec: &str, dep: &str) {
        if let Some(sections) = &mut self.section {
            if let Some(s) = sections.get_mut(&sec.to_string()) {
                if !s.contains(&dep.to_string()) {
                    s.push(dep.to_string());
                }
            } else {
                sections.insert(sec.to_string(), vec![dep.to_string()]);
            }
        } else {
            self.add_section(sec);
            self.add_section_dep(sec, dep);
        }
    }

    /// Removes a section key from the section exclusion map.
    pub fn remove_section(&mut self, sec: &str) {
        if let Some(sections) = &mut self.section {
            sections.remove_entry(sec);
            if sections.is_empty() {
                self.section = None;
            }
        }
    }

    /// Removes a specific dependency name from a section exclusion list.
    pub fn remove_section_dep(&mut self, sec: &str, dep: &str) {
        if let Some(sections) = &mut self.section
            && let Some(section) = sections.get_mut(sec)
        {
            if let Some(idx) = section.iter().position(|d| *d == dep) {
                section.remove(idx);
            }
            if section.is_empty() {
                self.remove_section(sec);
            }
        }
    }
}

/// Representation of a single workspace entry in `rrdu.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TomlWorkspace {
    /// Optional display label for the workspace.
    #[serde(default)]
    pub label: Option<String>,
    /// Path to `Cargo.toml` or workspace folder.
    pub path: String,
    /// Optional exclusion configuration for this workspace.
    #[serde(default)]
    pub exclude: Option<TomlWorkspaceExclude>,
}

impl TomlWorkspace {
    /// Creates a new workspace configuration for the specified path.
    pub fn new(path: &str) -> Self {
        Self {
            label: None,
            path: path.to_string(),
            exclude: None,
        }
    }

    /// Sets the display label for this workspace.
    pub fn set_label(&mut self, label: &str) {
        self.label = Some(label.to_string())
    }

    /// Adds a project-wide excluded dependency.
    pub fn add_exclude_deps_in_project(&mut self, dep: &str) {
        if let Some(exc) = &mut self.exclude {
            exc.add_project_dep(dep);
        } else {
            let mut exc = TomlWorkspaceExclude::default();
            exc.add_project_dep(dep);
            self.exclude = Some(exc);
        }
    }

    /// Removes a project-wide excluded dependency.
    pub fn remove_exclude_deps_in_project(&mut self, dep: &str) {
        if let Some(exc) = &mut self.exclude {
            exc.remove_project_dep(dep);
            if exc.project.is_none() && exc.section.is_none() {
                self.exclude = None;
            }
        }
    }

    /// Adds an excluded dependency under a section, or excludes the whole section if `dep` is `None`.
    pub fn add_exclude_deps_in_sec(&mut self, sec: &str, dep: Option<&str>) {
        if let Some(exc) = &mut self.exclude {
            if let Some(d) = dep {
                exc.add_section_dep(sec, d);
            } else {
                exc.add_section(sec);
            }
        } else {
            let mut exc = TomlWorkspaceExclude::default();
            if let Some(d) = dep {
                exc.add_section_dep(sec, d);
            } else {
                exc.add_section(sec);
            }
            self.exclude = Some(exc);
        }
    }

    /// Removes an excluded dependency or section exclusion.
    pub fn remove_exclude_deps_in_sec(&mut self, sec: &str, dep: Option<&str>) {
        if let Some(exc) = &mut self.exclude {
            if let Some(d) = dep {
                exc.remove_section_dep(sec, d);
            } else {
                exc.remove_section(sec);
            }
            if exc.project.is_none() && exc.section.is_none() {
                self.exclude = None;
            }
        }
    }

    /// Migrates legacy YAML exclusion rules into the TOML exclusion model.
    pub fn parse_legacy_exclude(&mut self, legacy: ExcludeConfig) {
        let migrated = MigrateYamlToTomlConfig::migrate_workspace_exclude(legacy);
        self.exclude = migrated;
    }

    /// Checks whether a dependency entry is excluded at project or section level and marks it accordingly.
    ///
    /// # Arguments
    /// * `entry` - The dependency entry to evaluate.
    /// * `section` - Optional section name containing the dependency.
    pub fn is_dep_excluded(&self, entry: &mut DependencyEntry, section: Option<String>) {
        if let Some(exc) = &self.exclude {
            let in_project: Vec<String> = if let Some(p) = &exc.project {
                p.to_vec()
            } else {
                Vec::new()
            };
            let in_section: HashMap<String, Vec<String>> = if let Some(s) = &exc.section {
                s.clone()
            } else {
                HashMap::new()
            };
            let key = entry.package();
            if let Some(section_name) = section
                && let Some(sec) = in_section.get(&section_name)
                && sec.contains(&key.to_string())
            {
                entry.exclude_dep(&ExcludeReason::ExcludedInSection);
            } else if in_project.contains(&key.to_string()) {
                entry.exclude_dep(&ExcludeReason::ExcludedInProject);
            };
        };
    }
}

/// Global runner and interactive CLI configuration settings in `rrdu.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TomlRrdu {
    /// Maximum line display limit for pagination in interactive CLI.
    #[serde(
        rename = "max-lines",
        alias = "max_lines",
        default = "default_max_lines"
    )]
    pub max_lines: usize,
    /// Schema version string of the configuration file.
    #[serde(default = "get_rrdu_version")]
    pub version: String,
    /// List of excluded projects or folders.
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
}

fn default_max_lines() -> usize {
    10
}

/// Returns the current running `rrdu` package version string.
pub fn get_rrdu_version() -> String {
    let current = env!("CARGO_PKG_VERSION");
    String::from(current)
}

impl Default for TomlRrdu {
    fn default() -> Self {
        Self {
            max_lines: default_max_lines(),
            version: get_rrdu_version(),
            exclude: None,
        }
    }
}

/// Root configuration representation corresponding to `rrdu.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RrduTomlConfig {
    /// Mapping of project keys to workspace configurations.
    #[serde(default)]
    pub workspaces: HashMap<String, TomlWorkspace>,
    /// Mapping of config labels to sub-configuration file or directory paths.
    #[serde(default)]
    pub configs: HashMap<String, String>,
    /// Global runner settings.
    #[serde(default)]
    pub rrdu: TomlRrdu,
}

impl std::fmt::Display for RrduTomlConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(toml) = self.to_toml_string() {
            write!(f, "{toml}")
        } else {
            write!(
                f,
                "Workspaces: {}, Configs: {}",
                self.workspaces.len(),
                self.configs.len()
            )
        }
    }
}

/// Evaluates compatibility between the loaded configuration schema version and the running binary.
///
/// # Arguments
/// * `config` - The loaded configuration to check.
///
/// # Returns
/// A [`ConfigCompatibility`] variant indicating compatibility status.
pub fn check_config_compatibility(config: &RrduTomlConfig) -> ConfigCompatibility {
    let raw_version = &config.rrdu.version;

    let Ok(config_ver) = semver::Version::parse(raw_version) else {
        return ConfigCompatibility::LegacyMissingVersion;
    };

    let min_supported = semver::Version::new(0, 0, 6);
    let installed = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| semver::Version::new(0, 0, 0));

    if config_ver < min_supported {
        ConfigCompatibility::OutdatedVersion(config_ver, min_supported)
    } else if config_ver > installed {
        ConfigCompatibility::IncompatibleFutureVersion(config_ver)
    } else {
        ConfigCompatibility::Compatible
    }
}

impl RrduTomlConfig {
    /// Loads an existing configuration from the current working directory, or initializes a new in-memory config.
    pub fn new() -> Self {
        match Self::load_config() {
            Ok(c) => c,
            Err(_) => Self::init(),
        }
    }

    /// Initializes a default configuration and executes workspace discovery without persisting to disk.
    pub fn init() -> Self {
        let mut initial_default: Self = Self::default();
        initial_default.discover_workspace(None, None);
        initial_default
    }

    /// Discovers projects across the workspace and saves a new `rrdu.toml` to disk.
    pub fn create_config() -> Self {
        let new_config = Self::init();
        let _ = new_config.save_config();
        new_config
    }

    fn add_workspace(&mut self, project: ProjectConfig) {
        if let SubConfig::Bool(true) = project.sub_config {
            self.configs.insert(project.project, project.path);
        } else {
            let mut p = TomlWorkspace::new(&project.path);
            p.set_label(&project.project);
            p.parse_legacy_exclude(project.exclude);
            self.workspaces.insert(project.project, p);
        }
    }

    fn discover_workspace(&mut self, path: Option<PathBuf>, ignore_before: Option<Vec<PathBuf>>) {
        let discover_path = path.unwrap_or_else(|| PathBuf::from(""));
        let last_ignored = ignore_before.unwrap_or_default();
        simple_status!(
            log::LevelFilter::Info,
            Module::CONFIGURATION,
            100,
            format!("Start discover path: {:#?}", discover_path)
        );
        let discovery = Discovery::new(discover_path.clone(), last_ignored);
        match discovery {
            Ok(disc) => {
                if let Some(p) = disc.found_project {
                    self.add_workspace(p)
                };
                let next = disc.next_depth;
                for n in next {
                    self.discover_workspace(Some(n), Some(disc.ignore.clone()));
                }
            }
            Err(e) => meta_status!(
                log::LevelFilter::Error,
                Module::CONFIGURATION,
                500,
                format!("Error while discover '{:#?}'", discover_path),
                e
            ),
        };
    }

    /// Saves the configuration to `./rrdu.toml`.
    pub fn save_config(&self) -> Result<(), ConfigError> {
        let config_path = match create_absolute_path(Path::new(""), Some(RRDU_TOML_FILE_NAME)) {
            Ok(f) => f,
            Err(_) => return Err(ConfigError::InsecurePath(Path::new("").to_path_buf())),
        };
        if config_path.exists() {
            simple_status!(
                log::LevelFilter::Warn,
                Module::CONFIGURATION,
                200,
                "Config file already exists. To create a new one, delete it and run 'rrdu --init'."
            );
            return Ok(());
        }
        let toml_string = self.to_toml_string()?;
        std::fs::write(&config_path, toml_string)?;
        simple_status!(
            log::LevelFilter::Info,
            Module::CONFIGURATION,
            200,
            format!("Configuration initialized and saved to '{:?}'", config_path)
        );
        Ok(())
    }

    /// Formats the configuration into a clean TOML string with compact workspace tables and schema comment.
    pub fn to_toml_string(&self) -> Result<String, ConfigError> {
        let raw = toml_edit::ser::to_string_pretty(self)?;
        let mut doc = raw.parse::<toml_edit::DocumentMut>()?;
        if let Some(workspaces_item) = doc.get_mut("workspaces")
            && let Some(workspaces_table) = workspaces_item.as_table_mut()
        {
            let keys: Vec<String> = workspaces_table
                .iter()
                .map(|(k, _)| k.to_string())
                .collect();
            for key in keys {
                if let Some(item) = workspaces_table.remove(&key)
                    && let Ok(tbl) = item.into_table()
                {
                    let inline_tbl = tbl.into_inline_table();
                    workspaces_table.insert(
                        &key,
                        toml_edit::Item::Value(toml_edit::Value::InlineTable(inline_tbl)),
                    );
                }
            }
        }
        let formatted = format!(
            "#:schema https://annodomine.github.io/rust-recursive-deps-updater/schemas/rrdu.schema.json\n\n{doc}"
        );
        Ok(formatted)
    }

    fn migrate_legacy_config<P: AsRef<Path>>(path: P) -> Option<Self> {
        let config_path = match create_absolute_path(path.as_ref(), Some(RRDUCONFIG_FILE_NAME)) {
            Ok(f) => f,
            Err(_) => return None,
        };
        if config_path.exists() {
            return Some(MigrateYamlToTomlConfig::migrate(path));
        };
        None
    }

    /// Loads and parses an `rrdu.toml` from a specific file or directory path.
    ///
    /// # Arguments
    /// * `path` - Path pointing to `rrdu.toml` or a folder containing it.
    ///
    /// # Returns
    /// Validated [`RrduTomlConfig`] instance.
    ///
    /// # Errors
    /// Returns [`ConfigError::InsecurePath`] if path traversal is detected.
    /// Returns [`ConfigError::Io`] if file access fails.
    /// Returns [`ConfigError::TomlDe`] if TOML deserialization fails.
    /// Returns [`ConfigError::IncompatibleVersion`] or [`ConfigError::LegacyConfiguration`] if version checks fail.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let toml = if let Some(c) = Self::migrate_legacy_config(path.as_ref()) {
            c
        } else {
            let config_path = match create_absolute_path(path.as_ref(), Some(RRDU_TOML_FILE_NAME)) {
                Ok(f) => f,
                Err(_) => return Err(ConfigError::InsecurePath(path.as_ref().to_path_buf())),
            };
            simple_status!(
                log::LevelFilter::Debug,
                Module::CONFIGURATION,
                100,
                format!("Loading configuration from '{:?}'", config_path)
            );
            let content = std::fs::read_to_string(config_path)?;
            toml_edit::de::from_str::<Self>(&content)?
        };

        for project in toml.workspaces.values() {
            validate_path_traversal(Path::new(&project.path))?;
        }

        for path in toml.configs.values() {
            validate_path_traversal(Path::new(path))?;
        }

        match check_config_compatibility(&toml) {
            ConfigCompatibility::Compatible => {
                simple_status!(
                    log::LevelFilter::Debug,
                    Module::CONFIGURATION,
                    200,
                    format!(
                        "Configuration loaded and verified compatible ({} workspace project(s))",
                        toml.workspaces.len()
                    )
                );
                Ok(toml)
            }
            ConfigCompatibility::LegacyMissingVersion => Err(ConfigError::LegacyConfiguration),
            ConfigCompatibility::OutdatedVersion(found, required) => {
                Err(ConfigError::IncompatibleVersion {
                    found: found.to_string(),
                    required: required.to_string(),
                })
            }
            ConfigCompatibility::IncompatibleFutureVersion(found) => {
                Err(ConfigError::IncompatibleVersion {
                    found: found.to_string(),
                    required: format!("<= {}", env!("CARGO_PKG_VERSION")),
                })
            }
        }
    }

    fn load_config() -> Result<Self, ConfigError> {
        Self::load_from_path(Path::new(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_toml() {
        let mut config = RrduTomlConfig::default();
        let mut ws = TomlWorkspace::new("./");
        ws.set_label("my-crate");
        ws.add_exclude_deps_in_project("serde");
        ws.add_exclude_deps_in_sec("dev-dependencies", Some("tokio"));
        config.workspaces.insert("my-crate".to_string(), ws);
        config
            .configs
            .insert("sub-crate".to_string(), "crates/sub".to_string());

        let toml_output = config.to_toml_string().unwrap();
        println!("Formatted TOML:\n{}", toml_output);
        assert!(toml_output.starts_with("#:schema "));
        assert!(toml_output.contains("[workspaces]\n"));
        assert!(toml_output.contains("my-crate = {"));
        assert!(toml_output.contains("max-lines = 10"));

        let roundtrip: RrduTomlConfig = toml_edit::de::from_str(&toml_output).unwrap();
        assert_eq!(roundtrip.workspaces.len(), 1);
        assert_eq!(roundtrip.configs.len(), 1);
        assert_eq!(roundtrip.rrdu.max_lines, 10);
    }
}
