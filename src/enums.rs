//! Domain enums, models, and type definitions for `rrdu`.
//!
//! Encompasses dependency types, version representations, configuration compatibility,
//! section discriminants, registry client states, and cache responses.

use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::{
    errors::*,
    registry::{
        crates_index_response_structs::CratesIndexResponseParsed,
        crates_io_response_structs::CratesIOResponse,
        rust_sec_json_response_structs::RustsecJsonResponse,
    },
};

/// Identifier if a sub project configuration is present.
///
/// Possible values:
/// - true -> Sub config file is in same folder as Cargo.toml
/// - false -> No sub config file present, but Cargo.toml
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum SubConfig {
    /// No sub-configuration file is present.
    #[default]
    None,
    /// Explicit boolean indicating sub-configuration status.
    Bool(bool),
}

/// Primitive boolean enum for clear readability in table and classification displays.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Boolean {
    /// True value.
    True,
    /// False value.
    False,
}

impl Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Boolean::True => write!(f, "True"),
            Boolean::False => write!(f, "False"),
        }
    }
}

impl Boolean {
    /// Returns `true` if this boolean value is `Boolean::False`.
    pub fn is_false(&self) -> bool {
        *self == Boolean::False
    }

    /// Returns `true` if this boolean value is `Boolean::True`.
    pub fn is_true(&self) -> bool {
        *self == Boolean::True
    }
}

/// Target dependency section classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetDepKind {
    /// `[dependencies]`
    Normal,
    /// `[dev-dependencies]`
    Dev,
    /// `[build-dependencies]`
    Build,
}

/// Rationale for excluding a dependency from analysis and updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcludeReason {
    /// Excluded dependency section in project defined in `workspace.exclude.section` (e.g. `[dependencies.serde]`)
    ExcludedSection,
    /// Excluded DEPENDENCY project wide defined in `workspace.exclude.project`
    ExcludedInProject,
    /// Excluded DEPENDENCY in project section defined in `workspace.exclude.section` (e.g. `[dev-dependencies`)
    ExcludedInSection,
}

impl Display for ExcludeReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExcludeReason::ExcludedInProject => write!(f, "Excluded project wide!"),
            ExcludeReason::ExcludedInSection => write!(f, "Excluded in section!"),
            ExcludeReason::ExcludedSection => write!(f, "Section excluded!"),
        }
    }
}

/// State of a collected dependency version from the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyCollectionVersion {
    /// Check not fulfilled or pending.
    None,
    /// Check fulfilled with latest version string.
    Latest(String),
    /// Encountered an error during registry query.
    Error(CollectionVersionError),
    /// Encounter the dependency as ignored by config.
    Excluded,
}

/// Representation of dependencies within a specific section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySectionMap {
    /// Map of dependency names to parsed entries.
    Map(HashMap<String, DependencyEntry>),
    /// Entire section was excluded via configuration.
    ExcludedSection,
    /// No dependencies found in this section.
    Empty,
}

impl Display for DependencySectionMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencySectionMap::Empty => write!(f, "No dependencies"),
            DependencySectionMap::ExcludedSection => write!(f, "Section is excluded"),
            DependencySectionMap::Map(deps) => {
                write!(
                    f,
                    "{:}",
                    deps.iter()
                        .map(|d| format!("- {:}", d.0))
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
        }
    }
}

impl DependencySectionMap {
    /// Returns the number of dependencies in this section map.
    pub fn len(&self) -> usize {
        match self {
            DependencySectionMap::Map(deps) => deps.len(),
            _ => 0,
        }
    }

    /// Returns `true` if this dependency section map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Map of all dependency sections in a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDependency {
    /// Map of sections to their dependency entries.
    Map(HashMap<DependencySection, DependencySectionMap>),
    /// Project was entirely excluded via configuration.
    ExcludedProject,
}

impl ProjectDependency {
    /// Returns the number of dependency sections in this project.
    pub fn len(&self) -> usize {
        match self {
            ProjectDependency::ExcludedProject => 0,
            ProjectDependency::Map(sec) => sec.len(),
        }
    }

    /// Returns `true` if the project has no dependency sections.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total count of dependency entries across all sections.
    pub fn total_values(&self) -> usize {
        match self {
            ProjectDependency::ExcludedProject => 0,
            ProjectDependency::Map(sec) => sec.iter().map(|e| e.1.len()).sum(),
        }
    }
}

/// Classification of a dependency's version specification in `Cargo.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyVersion {
    /// Supported crates.io dependency version requirement (e.g. "1.0.0", "^3.1", "~0.4")
    Supported(String),
    /// Contains one or more unsupported configuration keys (e.g. ["git", "path"])
    UnsupportedKeys {
        /// List of unsupported keys detected in the dependency definition.
        keys: Vec<String>,
    },
    /// One or more required fields are missing or invalid (e.g. ["version"])
    MissingRequired {
        /// List of required fields that were absent.
        fields: Vec<String>,
    },
    /// Value of version is not supported
    UnsupportedValue(String),
    /// Excluded dependency incl. reason
    Excluded(ExcludeReason),
}

impl Display for DependencyVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyVersion::Excluded(reason) => write!(f, "{reason}"),
            DependencyVersion::Supported(version) => write!(f, "{version}"),
            DependencyVersion::UnsupportedKeys { keys } => write!(f, "{:}", keys.join(", ")),
            DependencyVersion::MissingRequired { fields } => write!(f, "{:}", fields.join(", ")),
            DependencyVersion::UnsupportedValue(value) => write!(f, "{value}"),
        }
    }
}

/// A parsed dependency entry representing one of the valid `Cargo.toml` declaration formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyEntry {
    /// Simple string version dependency.
    ///
    /// E.g.
    /// ```toml
    /// colored = "3.1.1"
    /// ```
    Simple(SimpleDependency),
    /// Inline table dependency.
    ///
    /// E.g.
    /// ```toml
    /// web-sys = { version = "0.3", features = [...] }
    /// ```
    Inline(InlineDependency),
    /// Multi-line table dependency.
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
    /// The TOML key identifying the dependency.
    pub toml_key: String,
    /// The version requirement specification.
    pub version: DependencyVersion,
}

/// Schema of an object-like inline dependency definition
///
/// E.g.
/// ```toml
/// web-sys = { version = "0.3", features = [...] }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineDependency {
    /// The TOML key identifying the dependency.
    pub toml_key: String,
    /// The crate package name on crates.io.
    pub package: String,
    /// The version requirement specification.
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
    /// The TOML key identifying the dependency.
    pub toml_key: String,
    /// The crate package name on crates.io.
    pub package: String,
    /// The version requirement specification.
    pub version: DependencyVersion,
}

/// Values to generate CI and CLI dependency table row
///
/// (is latest, needs migration, version string)
pub type DependencyRow = (Boolean, Boolean, String);

/// Identifies a dependency section within `Cargo.toml`.
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
    Target {
        /// Target triple or platform expression.
        target: String,
        /// Target dependency section kind.
        kind: TargetDepKind,
    },
    /// `[dependencies.my_serde]` or `[target.x86.dependencies.clap]` or `[workspace.dependencies.serde]` or `[target.'<target>'.build-dependencies.tokio]`
    Table {
        /// Parent section of this table dependency.
        parent: Box<DependencySection>,
        /// TOML key under the parent section.
        toml_key: String,
    },
}

/// Scope of dependency exclusion in configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcludeArea {
    /// Project wide excluded dependencies
    Project,
    /// Section specified excluded dependencies
    Section(DependencySection),
}

/// Compatibility assessment between `.rrduconfig` and the running `rrdu` binary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigCompatibility {
    /// Version of config is compatible with the current running/installed version
    Compatible,
    /// Config has no version specified (legacy format)
    LegacyMissingVersion,
    /// Version of config is outdated and requires migration
    OutdatedVersion(semver::Version, semver::Version),
    /// Version of running rrdu is older than config version
    IncompatibleFutureVersion(semver::Version),
}

/// Aggregate response bundle holding registry index, API metadata, and audit records for a crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateResponses {
    /// Parsed crates.io sparse index response.
    pub index: Option<CratesIndexResponseParsed>,
    /// Crates.io web API metadata response.
    pub api: Option<Box<CratesIOResponse>>,
    /// RustSec advisory database response.
    pub audit: Option<RustsecJsonResponse>,
}

/// Client request states for a crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryClientState {
    /// The client request has not yet started.
    Uninitialised,
    /// The client makes a request to 'crates.io' to get the latest versions.
    Loading,
    /// The request was successfully finished and the crate was found.
    Succesed(CrateResponses),
    /// The request ran into an error and could not be fulfilled.
    Errored(CollectionVersionError),
}

/// Cached responses organized by cache type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheResponse {
    /// Crates.io index cache responses keyed by crate name.
    CratesIOIndexResponse(HashMap<String, CacheEntry<CratesIndexResponseParsed>>),
    /// Crates.io API cache responses keyed by crate name.
    CratesIOAPIResponse(HashMap<String, CacheEntry<Box<CratesIOResponse>>>),
    /// RustSec JSON advisory cache responses keyed by crate name.
    RustSecJsonResponse(HashMap<String, CacheEntry<RustsecJsonResponse>>),
}

/// A timestamped cache entry wrapping a cached response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// System time when the cache entry was created or fetched.
    pub timestamp: SystemTime,
    /// The cached response payload.
    pub response: T,
}

impl<T: Clone + Display + Debug> CacheEntry<T> {
    /// Returns the creation timestamp for TTL calculation.
    pub fn get_ttl(&self) -> SystemTime {
        self.timestamp
    }
}

/// Discriminant for the supported cache storage types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheType {
    /// Sparse crates.io index cache.
    CratesIOIndex,
    /// Crates.io REST API cache.
    CratesIOApi,
    /// RustSec vulnerability advisory cache.
    RustsecJsonResponse,
}
