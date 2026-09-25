use std::{
    collections::HashMap,
    env,
    path::{Component, Path, PathBuf},
};

use noyalib::{ParserConfig, SerializerConfig};
use serde::{Deserialize, Serialize};

use crate::{config::ConfigError, globals::*};

/// Identifier if a sub project configuration is present.
///
/// Possible values:
/// - true -> Sub config file is in same folder as Cargo.toml
/// - false -> No sub config file present, but Cargo.toml
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum SubConfig {
    #[default]
    None,
    Bool(bool),
}

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
    pub exclude: ExcludeConfig,
    #[serde(default)]
    /// Identifier, if the project is a `.rrduconfig` link
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

    pub fn define_sub_config(&mut self, sub_config: SubConfig) {
        self.sub_config = sub_config;
    }

    pub fn get_sub_config_path(&self) -> Option<PathBuf> {
        let path = match &self.sub_config {
            SubConfig::Bool(true) => PathBuf::from(&self.path).join(".rrduconfig"),
            _ => return None,
        };

        if Self::validate_path(&path) {
            Some(path)
        } else {
            None
        }
    }

    pub fn exclude_dep(&mut self, area: ExcludeArea, dep: String) {
        match area {
            ExcludeArea::Project => {
                self.exclude.project.push(dep);
            }
            ExcludeArea::Section(sec) => self.format_section_dep(sec, dep),
        }
    }

    fn format_section_dep(&mut self, sec: DependencySection, dep: String) {
        match sec {
            DependencySection::Table { .. } => {
                self.exclude.section.entry(sec.to_string()).or_default();
            }
            _ => {
                self.exclude
                    .section
                    .entry(sec.to_string())
                    .or_default()
                    .push(dep);
            }
        }
    }

    pub fn validate_project_path(&self) -> bool {
        let path = Path::new(&self.path);
        Self::validate_path(path)
    }

    pub fn validate_path(path: &Path) -> bool {
        for path_comp in path.components() {
            match path_comp {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return false;
                }
                _ => {}
            }
        }
        true
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RrduConfig {
    /// List of workspaces based on the `.rrduconfig` fodler
    #[serde(default = "default_root_project")]
    pub workspace: Vec<ProjectConfig>,
    /// Updater configuration
    #[serde(default)]
    pub updater: UpdaterConfig,
}

fn default_root_project() -> Vec<ProjectConfig> {
    vec![]
}

pub enum ConfigCompatibility {
    /// Version of config is compatible with the current running/installed version
    Compatible,
    /// Config have no version specified
    LegacyMissingVersion,
    /// Version of config is outdated
    OutdatedVersion(semver::Version, semver::Version),
    /// Version of running rrdu is older than config version
    IncompatibleFutureVersion(semver::Version),
}

pub fn check_config_compatibility(config: &RrduConfig) -> ConfigCompatibility {
    let Some(raw_version) = &config.updater.version else {
        return ConfigCompatibility::LegacyMissingVersion;
    };

    let Ok(config_ver) = semver::Version::parse(raw_version) else {
        return ConfigCompatibility::LegacyMissingVersion;
    };

    let min_supported = semver::Version::new(0, 0, 3);
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

impl Default for RrduConfig {
    fn default() -> Self {
        Self {
            workspace: default_root_project(),
            updater: UpdaterConfig::default(),
        }
    }
}

impl RrduConfig {
    pub const CONFIG_FILE_NAME: &'static str = ".rrduconfig";

    pub fn new() -> Self {
        // Check if a config is already generated.
        // If no config could be loaded, return the initialised config without save it.
        Self::load_config().unwrap_or_else(|_| Self::init())
    }

    pub fn init() -> Self {
        let mut initial_default: Self = Self::default();
        initial_default.discover_workspace();
        initial_default
    }

    pub fn create_config() -> Self {
        let new_config = Self::init();
        let res = new_config.save_config();
        if res.is_err() {
            println!("Error: {:?}", res);
        }
        new_config
    }

    pub fn add_workspace(&mut self, project: String, path: String) {
        let _ = &self.workspace.push(ProjectConfig::new(project, path));
    }

    fn discover_workspace(&mut self) {
        // Discover the workspace for Cargo.toml files
        // TODO: Implement discovery and apply it here to auto fill workspace for initialisation
    }

    fn retreive_config_path() -> Result<PathBuf, ConfigError> {
        let mut config_file = std::env::current_dir()?;
        config_file.push(Self::CONFIG_FILE_NAME);
        Ok(config_file)
    }

    fn save_config(&self) -> Result<(), ConfigError> {
        // Stores the configuration
        let serializer_config: SerializerConfig = SerializerConfig::new().quote_all(false);
        let path = Self::retreive_config_path()?;
        if path.exists() {
            println!(
                "Config file already exists. To create a new one, delete it and run 'rrdu --init'."
            );
            return Ok(());
        }
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        noyalib::to_writer_with_config(writer, self, &serializer_config)?;
        Ok(())
    }

    /// Loads and parses an `.rrduconfig` from a specific file path (relative or absolute).
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(content);
        let deserializer_config = ParserConfig::new();
        let yaml = noyalib::from_reader_with_config::<std::io::BufReader<std::fs::File>, Self>(
            reader,
            &deserializer_config,
        )?;

        for project in &yaml.workspace {
            if !project.validate_project_path() {
                return Err(ConfigError::InsecurePath(PathBuf::from(&project.path)));
            }
        }

        match check_config_compatibility(&yaml) {
            ConfigCompatibility::Compatible => Ok(yaml),
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
        let path = Self::retreive_config_path()?;
        Self::load_from_path(path)
    }
}

#[cfg(test)]
mod test_project_config {
    use super::*;

    #[test]
    fn test_validate_path_valid_relative_paths() {
        let valid_paths = [
            "./",
            "./Cargo.toml",
            "./crates/my_crate",
            "crates/sub_project/Cargo.toml",
            "sub_project",
        ];

        for path in valid_paths {
            let project = ProjectConfig::new("test".to_string(), path.to_string());
            assert!(
                project.validate_project_path(),
                "Expected valid path '{path}' to pass validation"
            );
        }
    }

    #[test]
    fn test_validate_path_rejects_parent_traversal() {
        let traversal_paths = [
            "../",
            "../Cargo.toml",
            "./crates/../../secret",
            "crates/../..",
            "crates/sub/../../../etc",
        ];

        for path in traversal_paths {
            let project = ProjectConfig::new("test".to_string(), path.to_string());
            assert!(
                !project.validate_project_path(),
                "Expected traversal path '{path}' to fail validation"
            );
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_validate_path_rejects_root_and_prefix() {
        let absolute_paths = ["/", "/etc/passwd", "/home/user/project"];

        for path in absolute_paths {
            let project = ProjectConfig::new("test".to_string(), path.to_string());
            assert!(
                !project.validate_project_path(),
                "Expected absolute/root path '{path}' to fail validation"
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_validate_path_rejects_root_and_prefix() {
        let absolute_paths = ["C:\\", "C:\\Windows\\System32", "D:\\"];

        for path in absolute_paths {
            let project = ProjectConfig::new("test".to_string(), path.to_string());
            assert!(
                !project.validate_project_path(),
                "Expected absolute/root path '{path}' to fail validation"
            );
        }
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

        project.exclude_dep(ExcludeArea::Project, "tokio".to_string());
        project.exclude_dep(ExcludeArea::Project, "serde".to_string());

        project.exclude_dep(
            ExcludeArea::Section(DependencySection::Dev),
            "clap".to_string(),
        );
        project.exclude_dep(
            ExcludeArea::Section(DependencySection::Dev),
            "tokio".to_string(),
        );
        project.exclude_dep(
            ExcludeArea::Section(DependencySection::Table {
                parent: Box::new(DependencySection::Dev),
                toml_key: "clap".to_string(),
            }),
            "clap".to_string(),
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

    #[test]
    fn test_rrdu_config_add_workspace() {
        let mut config = RrduConfig::default();
        assert!(config.workspace.is_empty());

        config.add_workspace("App".to_string(), "./".to_string());
        assert_eq!(config.workspace.len(), 1);
        assert_eq!(config.workspace[0].project, "App");
        assert_eq!(config.workspace[0].path, "./");
    }

    #[test]
    fn test_yaml_in_memory_roundtrip() {
        let mut original = RrduConfig::default();
        original.add_workspace("Root".to_string(), "./".to_string());
        original.updater.exclude_project("test_dir".to_string());

        let mut buffer = Vec::new();
        let serializer_cfg = SerializerConfig::new().quote_all(false);
        noyalib::to_writer_with_config(&mut buffer, &original, &serializer_cfg)
            .expect("Failed to serialize config into buffer");

        let parser_cfg = ParserConfig::new();
        let parsed: RrduConfig = noyalib::from_reader_with_config(&buffer[..], &parser_cfg)
            .expect("Failed to deserialize config from buffer");

        assert_eq!(original, parsed);
    }
}
