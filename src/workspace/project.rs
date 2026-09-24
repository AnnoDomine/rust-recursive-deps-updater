//! Representation of prjects and dependencies
//! The is not the representation for the config.
//! It holds the information as a struct to handle navigation to the CLI and updating the Cargo.toml

use crate::config::model::ProjectConfig;
use std::collections::HashMap;
use std::path::PathBuf;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetDepKind {
    Normal,
    Dev,
    Build,
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
    pub version: String,
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
    pub version: String,
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
    pub version: String,
}

impl DependencyEntry {
    pub fn package(&self) -> &str {
        match self {
            Self::Simple(d) => &d.toml_key,
            Self::Inline(d) => &d.package,
            Self::Table(d) => &d.package,
        }
    }

    pub fn toml_key(&self) -> &str {
        match self {
            Self::Simple(d) => &d.toml_key,
            Self::Inline(d) => &d.toml_key,
            Self::Table(d) => &d.toml_key,
        }
    }

    pub fn version(&self) -> &str {
        match self {
            Self::Simple(d) => &d.version,
            Self::Inline(d) => &d.version,
            Self::Table(d) => &d.version,
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
