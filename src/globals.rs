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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExcludeArea {
    /// Project wide excluded dependencies
    Project,
    /// Section specified excluded dependencies
    Section(DependencySection),
}

impl std::fmt::Display for DependencySection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "dependencies"),
            Self::Dev => write!(f, "dev-dependencies"),
            Self::Build => write!(f, "build-dependencies"),
            Self::Workspace => write!(f, "workspace.dependencies"),
            Self::Target { target, kind } => {
                let kind_str = match kind {
                    TargetDepKind::Normal => "dependencies",
                    TargetDepKind::Dev => "dev-dependencies",
                    TargetDepKind::Build => "build-dependencies",
                };
                write!(f, "target.{target}.{kind_str}")
            }
            Self::Table { parent, toml_key } => {
                write!(f, "{parent}.{toml_key}")
            }
        }
    }
}
