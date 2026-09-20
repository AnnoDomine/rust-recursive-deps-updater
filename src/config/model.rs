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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    #[serde(default)]
    pub workspace: Vec<ProjectConfig>,
    #[serde(default)]
    pub updater: UpdaterConfig,
}

impl ModelConfig {
    pub const CONFIG_FILE_NAME: &'static str = ".rrduconfig";

    pub fn new() -> Self {
        // Check if a config is already generated.
        // TODO: Implement load config

        // If no config is present, loading the default config
        let mut default_config: Self = Self::default();
        default_config.discover_workspace();
        default_config
    }

    pub fn init() -> Self {
        let mut initial_default: Self = Self::default();
        initial_default.discover_workspace();
        initial_default.save_config();
        initial_default
    }

    fn discover_workspace(&mut self) {
        // Discover the workspace for Cargo.toml files
        // TODO: Implement discovery and apply it here to auto fill workspace for non file configurated workspace
    }

    fn save_config(&self) {
        // Stores the configuration
        println!("{:}", Self::CONFIG_FILE_NAME);
        // TODO: Implement save logic
    }
}
