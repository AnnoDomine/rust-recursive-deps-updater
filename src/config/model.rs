use std::path::PathBuf;

use noyalib::{ParserConfig, SerializerConfig};
use serde::{Deserialize, Serialize};

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

    pub fn serialize_deps(&self) -> String {
        // TODO: Serializing logic
        let serializer_config: SerializerConfig = SerializerConfig::new().quote_all(false);
        let serialized_project: String =
            noyalib::to_string_with_config(self, &serializer_config).unwrap();
        serialized_project
    }

    pub fn deserialize_deps(&mut self) {
        // TODO: Deserializing logic
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(loaded: Self) -> Self {
        loaded
    }

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
        Self::load_config()
    }

    pub fn init() -> Self {
        let mut initial_default: Self = Self::default();
        initial_default.discover_workspace();
        initial_default
    }

    pub fn create_config() -> Self {
        let new_config = Self::init();
        new_config.save_config();
        new_config
    }

    pub fn add_workspace(&mut self, project: String, toml: String) {
        let _ = &self.workspace.push(ProjectConfig::new(project, toml));
    }

    fn discover_workspace(&mut self) {
        // Discover the workspace for Cargo.toml files
        // TODO: Implement discovery and apply it here to auto fill workspace for non file configurated workspace
    }

    fn retreive_config_path() -> PathBuf {
        let mut config_file = match std::env::current_dir() {
            Ok(p) => p,
            // We take use of the default error from rust when retreive the current directory.
            // Possible errors are:
            // - Directory does not exist.
            // - No permission to acces the current directory.
            Err(e) => panic!("{:?}", e),
        };
        config_file.push(Self::CONFIG_FILE_NAME);
        config_file
    }

    fn save_config(&self) {
        // Stores the configuration
        let serializer_config: SerializerConfig = SerializerConfig::new().quote_all(false);
        let file = match std::fs::File::create(Self::retreive_config_path()) {
            Ok(w) => w,
            Err(_) => panic!("Cound not create config file!"),
        };
        let writer = std::io::BufWriter::new(file);
        let _ = noyalib::to_writer_with_config(writer, self, &serializer_config);
    }

    fn load_config() -> Self {
        // Load the .rrduconfig file and return it as a string
        match std::fs::File::open(Self::retreive_config_path()) {
            Ok(content) => {
                // unwarp as error already tracked.
                let reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(content);
                let deserializer_config = ParserConfig::new();
                match noyalib::from_reader_with_config::<std::io::BufReader<std::fs::File>, Self>(
                    reader,
                    &deserializer_config,
                ) {
                    Ok(yaml) => yaml,
                    Err(e) => {
                        println!(
                            "Could not deserialize yaml file. {:#?}\nPlease review your .rrduconfig.",
                            e
                        );
                        Self::init()
                    }
                }
            }
            Err(_) => {
                println!("No config file found. Using default!");
                Self::init()
            }
        }
    }
}
