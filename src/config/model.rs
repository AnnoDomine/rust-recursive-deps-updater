use std::{collections::HashMap, env, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{constants::*, enums::*, errors::*, functions::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ExcludeConfig {
    /// Project wide excluded dependencies
    #[serde(default)]
    pub project: Vec<String>,
    /// Section specified excluded dependencies
    #[serde(default)]
    pub section: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectConfig {
    /// Name of the project.
    /// Can be:
    /// - `name` key from Cargo.toml
    /// - Last folder from the discovery, if it is a sub-config project
    pub project: String,
    /// Path to `Cargo.toml` of `.rrduconfig` if sub-config
    #[serde(alias = "toml")]
    pub path: String,
    /// List of dependencies excluded from updating
    #[serde(default)]
    pub exclude: ExcludeConfig,
    /// Identifier, if the project is a `.rrduconfig` link
    #[serde(default)]
    pub sub_config: SubConfig,
}

impl ProjectConfig {
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

    pub fn set_sub_config(&mut self, is_sub_config: bool) {
        self.sub_config = SubConfig::Bool(is_sub_config);
    }

    pub fn define_sub_config(&mut self, sub_config: SubConfig) {
        self.sub_config = sub_config;
    }

    pub fn get_sub_config_path(&self) -> Result<PathBuf, FileError> {
        match validate_path_traversal(&PathBuf::from(&self.path)) {
            Ok(p) => validate_path_and_file_name(&p, RRDUCONFIG_FILE_NAME),
            Err(e) => Err(FileError::ConfigError(e)),
        }
    }

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UpdaterConfig {
    /// List of excluded projects from the workspace list
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Automatic update all dependencies (excludes these ones from the project defined excluded and excluded projects)
    #[serde(default = "default_auto_update")]
    pub auto_update: String,
    /// Automatic scan for updates of dependencies (excludes these ones from the project defined excluded and excluded projects)
    #[serde(default = "default_auto_scan")]
    pub auto_scan: bool,
    /// Scroll area for interactive CLI (does not affect CI mode)
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    /// Version where the `.rrduconfig` was created.
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
    50
}
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
        assert_eq!(updater.max_lines, 50);
        assert!(updater.exclude.is_empty());

        updater.exclude_project("crates/vendor".to_string());
        assert_eq!(updater.exclude, vec!["crates/vendor"]);
    }
}
