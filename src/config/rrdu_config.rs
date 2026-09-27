use std::path::{Path, PathBuf};

use noyalib::{ParserConfig, SerializerConfig};
use serde::{Deserialize, Serialize};

use crate::{
    config::model::*, constants::*, enums::*, errors::*, functions::*,
    workspace::discovery::Discovery,
};

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

impl Default for RrduConfig {
    fn default() -> Self {
        Self {
            workspace: default_root_project(),
            updater: UpdaterConfig::default(),
        }
    }
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

impl RrduConfig {
    pub fn new() -> Self {
        // Check if a config is already generated.
        // If no config could be loaded, return the initialised config without save it.
        match Self::load_config() {
            Ok(c) => c,
            Err(e) => {
                println!("{:}", e);
                Self::init()
            }
        }
    }

    pub fn init() -> Self {
        let mut initial_default: Self = Self::default();
        initial_default.discover_workspace(None, None);
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

    fn add_workspace(&mut self, project: ProjectConfig) {
        self.workspace.push(project);
    }

    fn discover_workspace(&mut self, path: Option<PathBuf>, ignore_before: Option<Vec<PathBuf>>) {
        let discover_path = path.unwrap_or_else(|| PathBuf::from(""));
        let last_ignored = ignore_before.unwrap_or_default();
        // Discover the workspace for Cargo.toml files
        println!("Start discover path: {:#?}", discover_path);
        let discovery = Discovery::new(discover_path.clone(), last_ignored);
        println!("{:#?}", discovery);
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
            Err(e) => println!("Error while discover '{:#?}': {:#?}", discover_path, e),
        };
    }

    fn save_config(&self) -> Result<(), ConfigError> {
        // Stores the configuration
        let serializer_config: SerializerConfig = SerializerConfig::new().quote_all(false);
        let config_path = match create_absolute_path(Path::new(""), Some(RRDUCONFIG_FILE_NAME)) {
            Ok(f) => f,
            Err(_) => return Err(ConfigError::InsecurePath(Path::new("").to_path_buf())),
        };
        if config_path.exists() {
            println!(
                "Config file already exists. To create a new one, delete it and run 'rrdu --init'."
            );
            return Ok(());
        }
        let file = std::fs::File::create(config_path)?;
        let writer = std::io::BufWriter::new(file);
        noyalib::to_writer_with_config(writer, self, &serializer_config)?;
        Ok(())
    }

    /// Loads and parses an `.rrduconfig` from a specific file path (relative or absolute).
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config_path = match create_absolute_path(path.as_ref(), Some(RRDUCONFIG_FILE_NAME)) {
            Ok(f) => f,
            Err(_) => return Err(ConfigError::InsecurePath(path.as_ref().to_path_buf())),
        };
        let content = std::fs::File::open(config_path)?;
        let reader = std::io::BufReader::new(content);
        let deserializer_config = ParserConfig::new();
        let yaml = noyalib::from_reader_with_config::<std::io::BufReader<std::fs::File>, Self>(
            reader,
            &deserializer_config,
        )?;

        for project in &yaml.workspace {
            validate_path_traversal(Path::new(&project.path))?;
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
        Self::load_from_path(Path::new(""))
    }
}

#[cfg(test)]
mod test_rrdu_config {
    use super::*;

    #[test]
    fn test_rrdu_config_add_workspace() {
        let mut config = RrduConfig::default();
        assert!(config.workspace.is_empty());

        config.add_workspace(ProjectConfig::new("App".to_string(), "./".to_string()));
        assert_eq!(config.workspace.len(), 1);
        assert_eq!(config.workspace[0].project, "App");
        assert_eq!(config.workspace[0].path, "./");
    }

    #[test]
    fn test_yaml_in_memory_roundtrip() {
        let mut original = RrduConfig::default();
        original.add_workspace(ProjectConfig::new("Root".to_string(), "./".to_string()));
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
