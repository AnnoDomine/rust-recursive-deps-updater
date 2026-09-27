use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::*;

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

/// Primitve boolean enum for better readability
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Boolean {
    True,
    False,
}

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
pub enum ExcludeReason {
    /// Excluded dependency section in project defined in `workspace.exclude.section` (e.g. `[dependecies.serde]`)
    ExcludedSection,
    /// Excluded dependecy project wide defined in `workspace.exclude.project`
    ExcludedInProject,
    /// Excluded dependecy in project section defined in `workspace.exclude.section` (e.g. `[dev-dependencies`)
    ExcludedInSection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepndencyCollectionVersion {
    /// Chack not fulfilled
    None,
    /// Check fulfilled
    Latest(String),
    /// Check error
    Error(CollectionVersionError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependecySectionMap {
    /// Map of the dependencies from the section
    Map(HashMap<String, DependencyEntry>),
    /// Excluded section defined in `workspace.exclude.section` if complete section is excluded.
    /// Used for sections which are dependency specific.
    ExcludedSection,
    /// State if no dependecy found
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDependecy {
    /// Map of the dependencies from the project
    Map(HashMap<DependencySection, DependecySectionMap>),
    /// Excluded project defined in `updater.exclude`
    ExcludedProject,
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
    /// Excluded dependency incl. reason
    Excluded(ExcludeReason),
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
    /// `[dependencies.my_serde]` or `[target.x86.dependencies.clap]` or `[workspace.dependencies.serde]` or `[target.'<target>'.build-dependencies.tokio]`
    Table {
        parent: Box<DependencySection>,
        toml_key: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcludeArea {
    /// Project wide excluded dependencies
    Project,
    /// Section specified excluded dependencies
    Section(DependencySection),
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
