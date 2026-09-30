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
    /// Excluded dependency section in project defined in `workspace.exclude.section` (e.g. `[dependencies.serde]`)
    ExcludedSection,
    /// Excluded DEPENDENCY project wide defined in `workspace.exclude.project`
    ExcludedInProject,
    /// Excluded DEPENDENCY in project section defined in `workspace.exclude.section` (e.g. `[dev-dependencies`)
    ExcludedInSection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyCollectionVersion {
    /// Chack not fulfilled
    None,
    /// Check fulfilled
    Latest(String),
    /// Check error
    Error(CollectionVersionError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DEPENDENCYSectionMap {
    /// Map of the dependencies from the section
    Map(HashMap<String, DependencyEntry>),
    /// Excluded section defined in `workspace.exclude.section` if complete section is excluded.
    /// Used for sections which are dependency specific.
    ExcludedSection,
    /// State if no DEPENDENCY found
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDEPENDENCY {
    /// Map of the dependencies from the project
    Map(HashMap<DependencySection, DEPENDENCYSectionMap>),
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
pub type DEPENDENCYRow = (Boolean, Boolean, String);

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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateResponses {
    pub index: Option<CratesIndexResponseParsed>,
    pub api: Option<Box<CratesIOResponse>>,
    pub audit: Option<RustsecJsonResponse>,
}

/// Client request states for a crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryClientState {
    /// The client request has not yet started.
    Uninitialised,
    /// The clinet make a request to 'crates.io' to get the latest versions.
    Loading,
    /// The request were succesfully finished and the crate was found. The latest version were responsed.
    Succesed(CrateResponses),
    /// The request runs into an error and could not get fulfilled.
    Errored(CollectionVersionError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheResponse {
    // Crates io index response
    CratesIOIndexResponse(HashMap<String, CacheEntry<CratesIndexResponseParsed>>),
    // Crate io api response
    CratesIOAPIResponse(HashMap<String, CacheEntry<Box<CratesIOResponse>>>),
    // Rustsec Json Response
    RustSecJsonResponse(HashMap<String, CacheEntry<RustsecJsonResponse>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub timestamp: SystemTime,
    pub response: T,
}

impl<T: Clone + Display + Debug> CacheEntry<T> {
    pub fn get_ttl(&self) -> SystemTime {
        self.timestamp
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheType {
    CratesIOIndex,
    CratesIOApi,
    RustsecJsonResponse,
}
