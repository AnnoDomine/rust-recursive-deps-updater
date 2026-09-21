use std::path::{Component, Path, PathBuf};

use noyalib::{ParserConfig, SerializerConfig};
use serde::{Deserialize, Serialize};

use crate::config::ConfigError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: String,
    pub toml: String,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl ProjectConfig {
    pub fn new(project: String, toml: String) -> Self {
        Self {
            project,
            toml,
            exclude: Vec::new(),
        }
    }

    pub fn exclude_dep(&mut self, dep: String) {
        self.exclude.push(dep);
    }

    pub fn validate_path(&self) -> bool {
        let path = Path::new(&self.toml);
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
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default = "default_auto_update")]
    pub auto_update: String,
    #[serde(default = "default_auto_scan")]
    pub auto_scan: bool,
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
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

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            auto_update: default_auto_update(),
            auto_scan: default_auto_scan(),
            max_lines: default_max_lines(),
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
    #[serde(default = "default_root_project")]
    pub workspace: Vec<ProjectConfig>,
    #[serde(default)]
    pub updater: UpdaterConfig,
}

fn default_root_project() -> Vec<ProjectConfig> {
    vec![]
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

    pub fn add_workspace(&mut self, project: String, toml: String) {
        let _ = &self.workspace.push(ProjectConfig::new(project, toml));
    }

    fn discover_workspace(&mut self) {
        // Discover the workspace for Cargo.toml files
        // TODO: Implement discovery and apply it here to auto fill workspace for non file configurated workspace
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
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        let _ = noyalib::to_writer_with_config(writer, self, &serializer_config);
        Ok(())
    }

    fn load_config() -> Result<Self, ConfigError> {
        let path = Self::retreive_config_path()?;
        // Load the .rrduconfig file and return it as a string
        let content = std::fs::File::open(path)?;
        let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(content);
        let deserializer_config = ParserConfig::new();
        let yaml = noyalib::from_reader_with_config::<std::io::BufReader<std::fs::File>, Self>(
            reader,
            &deserializer_config,
        )?;
        for project in &yaml.workspace {
            if !project.validate_path() {
                return Err(ConfigError::InsecurePath(PathBuf::from(&project.toml)));
            }
        }
        Ok(yaml)
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
                project.validate_path(),
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
                !project.validate_path(),
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
                !project.validate_path(),
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
                !project.validate_path(),
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
        assert!(project.exclude.is_empty());

        project.exclude_dep("tokio".to_string());
        project.exclude_dep("serde".to_string());

        assert_eq!(project.exclude, vec!["tokio", "serde"]);
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
        assert_eq!(config.workspace[0].toml, "./");
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
