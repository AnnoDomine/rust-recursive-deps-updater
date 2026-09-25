//! Representation of prjects and dependencies
//! The is not the representation for the config.
//! It holds the information as a struct to handle navigation to the CLI and updating the Cargo.toml

use crate::{config::model::ProjectConfig, globals::Boolean};
use std::collections::HashMap;
use std::path::PathBuf;
use toml_edit::{InlineTable, Item, Table, Value};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetDepKind {
    /// `[dependencies]`
    Normal,
    /// `[dev-dependencies]`
    Dev,
    /// `[build-dependencies]`
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyVersion {
    /// Supported crates.io dependency version requirement (e.g. "1.0.0", "^3.1", "~0.4")
    Supported(String),
    /// Contains one or more unsupported configuration keys (e.g. ["git", "path"])
    UnsupportedKeys { keys: Vec<String> },
    /// One or more required fields are missing or invalid (e.g. ["version"])
    MissingRequired { fields: Vec<String> },
    /// Value of version is not supported
    UnsupportedValue(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyEntry {
    /// E.g.
    /// ```toml
    /// colored = "3.1.1"
    /// ```
    Simple(SimpleDependency),
    /// E.g.
    /// ```toml
    /// web-sys = { version = "0.3", features = [...] }
    /// ```
    Inline(InlineDependency),
    /// E.g.
    /// ```toml
    /// [dependencies.serde]
    /// version = "3.0.0"
    /// # OR
    /// [dependencies.my_serde]
    /// version = "3.0.0"
    /// package = "serde"
    /// ```
    Table(TableDependency),
}

/// Schema of a simple key-value dependency definition
///
/// E.g.
/// ```toml
/// colored = "3.1.1"
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleDependency {
    pub toml_key: String,
    pub version: DependencyVersion,
}

/// Schema of a object like dependency definition
///
/// E.g.
/// ```toml
/// web-sys = { version = "0.3", features = [...] }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineDependency {
    pub toml_key: String,
    pub package: String,
    pub version: DependencyVersion,
}

/// Schema of a section specified dependency definition
///
/// E.g.
/// ```toml
/// [dependencies.serde]
/// version = "3.0.0"
/// # OR
/// [dependencies.my_serde]
/// version = "3.0.0"
/// package = "serde"
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDependency {
    pub toml_key: String,
    pub package: String,
    pub version: DependencyVersion,
}

/// Values to generate CI and CLI dependency table row
///
/// (is latest, needs migration, version string)
pub type DependecyRow = (Boolean, Boolean, String);

impl DependencyEntry {
    const UNSUPPORTED: [&'static str; 4] = ["git", "path", "registry", "workspace"];
    // Currently we only require version, so we can hardcode.
    #[allow(dead_code)]
    const MIN_REQUIRED: [&'static str; 1] = ["version"];

    /// Entry function for parsing the dependecy.
    ///
    /// This function internal handles all posible schemas.
    /// If a schema is not supported or wrong defined (e.g. no version key) it returns None to not getting mapped.
    ///
    /// * We only support dependencies which are downloadable and updateable from crate.io
    /// * Additional at least the verion needs to be defined as a semver supported string ("MAJOR.MINOR.PATCH")
    pub fn new(key: &str, item: &Item) -> Option<Self> {
        match item {
            Item::Value(val @ Value::String(_)) => Some(Self::Simple(SimpleDependency {
                toml_key: key.to_string(),
                version: Self::validate_version_definition(val),
            })),
            Item::Value(Value::InlineTable(inline)) => Some(Self::parse_inline(key, inline)),
            Item::Table(table) => Some(Self::parse_table(key, table)),
            _ => None,
        }
    }

    /// Function to parse inline dependecy definitions.
    ///
    /// ```toml
    /// web-sys = { version = "0.3", features = [...] }
    /// ```
    fn parse_inline(key: &str, item: &InlineTable) -> Self {
        let keys: Vec<String> = item.iter().map(|(k, _v)| k.to_string()).collect();
        let version = match Self::is_unsupported(&keys) {
            Some(def) => def,
            None => Self::validate_version_key(item.get("version")),
        };
        let package = match item.get("package").and_then(|p| p.as_str()) {
            Some(p) => p.to_string(),
            None => key.to_string(),
        };
        Self::Inline(InlineDependency {
            toml_key: key.to_string(),
            package,
            version,
        })
    }

    /// Function to parse array table dependency definitions.
    ///
    /// ```toml
    /// [dependencies.serde]
    /// version = "3.0.0"
    /// # OR
    /// [dependencies.my_serde]
    /// version = "3.0.0"
    /// package = "serde"
    /// ```
    fn parse_table(key: &str, table: &Table) -> Self {
        let keys: Vec<String> = table.iter().map(|(k, _v)| k.to_string()).collect();
        let version = match Self::is_unsupported(&keys) {
            Some(def) => def,
            None => Self::validate_version_key(table.get("version").and_then(|i| i.as_value())),
        };
        let package = match table.get("package").and_then(|p| p.as_str()) {
            Some(p) => p.to_string(),
            None => key.to_string(),
        };
        Self::Table(TableDependency {
            toml_key: key.to_string(),
            package,
            version,
        })
    }

    /// Validate the keys does not includes a unsuported dependency definition (git, path,...).
    fn is_unsupported(keys: &[String]) -> Option<DependencyVersion> {
        let mut unsupported_list: Vec<String> = Vec::new();
        for key in keys {
            if Self::UNSUPPORTED.contains(&key.as_str()) {
                unsupported_list.push(key.clone());
            }
        }
        if !unsupported_list.is_empty() {
            Some(DependencyVersion::UnsupportedKeys {
                keys: unsupported_list,
            })
        } else {
            None
        }
    }

    /// Validate the key 'version' is present in the definition
    fn validate_version_key(key: Option<&Value>) -> DependencyVersion {
        match key {
            Some(v) => Self::validate_version_definition(v),
            None => DependencyVersion::MissingRequired {
                fields: vec!["version".to_string()],
            },
        }
    }

    /// Validate the version value is supported (type: str)
    fn validate_version_definition(value: &Value) -> DependencyVersion {
        match value.as_str() {
            Some(s) => DependencyVersion::Supported(s.to_string()),
            None => DependencyVersion::UnsupportedValue(value.to_string()),
        }
    }

    /// Return the dependency name (from create.io)
    pub fn package(&self) -> &str {
        match self {
            Self::Simple(d) => &d.toml_key,
            Self::Inline(d) => &d.package,
            Self::Table(d) => &d.package,
        }
    }

    /// Return the key of a dependency defined in Cargo.toml
    pub fn toml_key(&self) -> &str {
        match self {
            Self::Simple(d) => &d.toml_key,
            Self::Inline(d) => &d.toml_key,
            Self::Table(d) => &d.toml_key,
        }
    }

    /// Return the version value based on DependencyVersion enum
    pub fn version(&self) -> &DependencyVersion {
        match self {
            Self::Simple(d) => &d.version,
            Self::Inline(d) => &d.version,
            Self::Table(d) => &d.version,
        }
    }

    /// Return the values needed for the dependecy row inside the CI and the CLI
    ///
    /// Return:
    /// * is_latest: Boolean
    /// * needs_migration: Boolean
    /// * message: String
    ///
    /// `(Boolean, Boolean, String)`
    pub fn get_dependency_row_valus(&self, latest: String) -> DependecyRow {
        let version = self.version();
        match version {
            DependencyVersion::Supported(v) => {
                let (is_latest, needs_migration) = self.check_if_latest(&latest);
                let message = match &is_latest {
                    Boolean::True => v.to_string(),
                    Boolean::False => format!("{:} -> {:}", v, latest),
                };
                (is_latest, needs_migration, message)
            }
            // All invalid versions are latest by default and do not need a dependency migration.
            DependencyVersion::UnsupportedKeys { keys } => {
                let key_string = if keys.len() > 1 { "keys" } else { "key" };
                (
                    Boolean::True,
                    Boolean::False,
                    format!("Unsupported {:}: {:}", key_string, keys.join(", ")),
                )
            }
            DependencyVersion::MissingRequired { fields } => (
                Boolean::True,
                Boolean::False,
                format!("Missing: {:}", fields.join(", ")),
            ),
            DependencyVersion::UnsupportedValue(v) => (
                Boolean::True,
                Boolean::False,
                format!("Unsupported value: {:}", v),
            ),
        }
    }

    /// Check if the current version is the latest and when, if the latest could be need a migration
    ///
    /// Return:
    /// * is_latest: Boolean
    /// * needs_migration: Boolean
    ///
    /// `(Boolean, Boolean)`
    pub fn check_if_latest(&self, _latest: &str) -> (Boolean, Boolean) {
        // TODO: Check if installed version is latest

        // Fallback
        (Boolean::False, Boolean::False)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencySection {
    /// `[dependencies]`
    Normal,
    /// `[dev-dependencies]`
    Dev,
    /// `[build-dependencies]`
    Build,
    /// `[workspace.dependencies]`
    Workspace,
    /// `[target.'<target>'.dependencies | dev-dependencies | build-dependencies]`
    Target { target: String, kind: TargetDepKind },
    /// `[dependencies.my_serde]` or `[target.x86.dependencies.my_serde]`
    Table {
        parent: Box<DependencySection>,
        toml_key: String,
    },
}

impl DependencySection {
    pub fn get_table_mut<'a>(
        &self,
        doc: &'a mut toml_edit::DocumentMut,
    ) -> Result<&'a mut dyn toml_edit::TableLike, String> {
        match self {
            Self::Normal => doc["dependencies"]
                .as_table_like_mut()
                .ok_or("no dependencies table".into()),
            Self::Dev => doc["dev-dependencies"]
                .as_table_like_mut()
                .ok_or("no dev-dependencies table".into()),
            Self::Build => doc["build-dependencies"]
                .as_table_like_mut()
                .ok_or("no build-dependencies table".into()),
            Self::Workspace => doc["workspace"]["dependencies"]
                .as_table_like_mut()
                .ok_or("no workspace dependencies".into()),
            Self::Target { target, kind } => {
                let kind_str = match kind {
                    TargetDepKind::Normal => "dependencies",
                    TargetDepKind::Dev => "dev-dependencies",
                    TargetDepKind::Build => "build-dependencies",
                };
                doc["target"][target][kind_str]
                    .as_table_like_mut()
                    .ok_or("no target table".into())
            }
            Self::Table { parent, toml_key } => {
                let parent_table = parent.get_table_mut(doc)?;
                parent_table
                    .get_mut(toml_key)
                    .and_then(|item| item.as_table_like_mut())
                    .ok_or_else(|| format!("table section {} not found", toml_key))
            }
        }
    }
}

pub struct Projects {
    pub name: String,
    pub path: PathBuf,
    pub deps: HashMap<DependencySection, HashMap<String, DependencyEntry>>,
    pub config: ProjectConfig,
}

pub struct Workspace {
    pub projects: Vec<Projects>,
    pub collected_deps: HashMap<String, Option<String>>,
}
