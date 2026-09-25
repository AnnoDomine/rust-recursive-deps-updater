//! Representation of prjects and dependencies
//! The is not the representation for the config.
//! It holds the information as a struct to handle navigation to the CLI and updating the Cargo.toml

use crate::{config::model::ProjectConfig, globals::*};
use std::collections::HashMap;
use std::path::PathBuf;
use toml_edit::{InlineTable, Item, Table, Value};

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
            None => DependencyVersion::UnsupportedValue(value.to_string().trim().to_string()),
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

    /// Helper to extract clean numeric SemVer from strings like "^1.2.3", "~0.3", "1.0"
    fn parse_base_version(raw: &str) -> Option<semver::Version> {
        let trimmed = raw.trim().trim_start_matches(|c: char| !c.is_ascii_digit());
        let dot_count = trimmed.chars().filter(|&c| c == '.').count();
        let normalized = match dot_count {
            0 => format!("{}.0.0", trimmed),
            1 => format!("{}.0", trimmed),
            _ => trimmed.to_string(),
        };
        semver::Version::parse(&normalized).ok()
    }

    /// Check if the current version is the latest and when, if the latest could be need a migration
    ///
    /// Return:
    /// * is_latest: Boolean
    /// * needs_migration: Boolean
    ///
    /// `(Boolean, Boolean)`
    pub fn check_if_latest(&self, latest: &str) -> (Boolean, Boolean) {
        let current_raw = match self.version() {
            DependencyVersion::Supported(v) => v.as_str(),
            _ => return (Boolean::True, Boolean::False),
        };

        let (Ok(req), Ok(latest_ver)) = (
            semver::VersionReq::parse(current_raw),
            semver::Version::parse(latest),
        ) else {
            return (Boolean::True, Boolean::False);
        };

        // If latest satisfies the current requirement, Cargo already resolves it
        if req.matches(&latest_ver) {
            return (Boolean::True, Boolean::False);
        }

        let Some(current_ver) = Self::parse_base_version(current_raw) else {
            return (Boolean::True, Boolean::False);
        };

        if current_ver >= latest_ver {
            return (Boolean::True, Boolean::False);
        }

        // Installed requirement does not cover latest -> is_latest = False
        let needs_migration = if current_ver.major >= 1 {
            latest_ver.major > current_ver.major
        } else if current_ver.minor >= 1 {
            latest_ver.major > 0 || latest_ver.minor > current_ver.minor
        } else {
            // In 0.0.x every bump is breaking
            latest_ver != current_ver
        };

        let migration_bool = if needs_migration {
            Boolean::True
        } else {
            Boolean::False
        };

        (Boolean::False, migration_bool)
    }
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

#[cfg(test)]
mod test_dependency_entry {
    use super::*;
    use toml_edit::DocumentMut;

    #[test]
    fn test_simple_dependency() {
        let doc: DocumentMut = "colored = \"3.1.1\"".parse().expect("valid toml");
        let item = &doc["colored"];
        let entry = DependencyEntry::new("colored", item).expect("should parse");
        assert_eq!(entry.toml_key(), "colored");
        assert_eq!(entry.package(), "colored");
        assert_eq!(
            entry.version(),
            &DependencyVersion::Supported("3.1.1".to_string())
        );
    }

    #[test]
    fn test_inline_dependencies() {
        let toml = r#"
    web-sys = { version = "0.3", features = ["Window"] }
    my_serde = { package = "serde", version = "1.0.190" }
    "#;
        let doc: DocumentMut = toml.parse().expect("valid toml");
        let entry1 = DependencyEntry::new("web-sys", &doc["web-sys"]).expect("should parse");
        assert_eq!(entry1.toml_key(), "web-sys");
        assert_eq!(entry1.package(), "web-sys");
        assert_eq!(
            entry1.version(),
            &DependencyVersion::Supported("0.3".to_string())
        );

        let entry2 = DependencyEntry::new("my_serde", &doc["my_serde"]).expect("should parse");
        assert_eq!(entry2.toml_key(), "my_serde");
        assert_eq!(entry2.package(), "serde");
        assert_eq!(
            entry2.version(),
            &DependencyVersion::Supported("1.0.190".to_string())
        );
    }

    #[test]
    fn test_table_dependencies() {
        let toml = r#"
    [dependencies.serde]
    version = "1.0.0"
    [dependencies.my_tokio]
    package = "tokio"
    version = "1.30.0"
    "#;
        let doc: DocumentMut = toml.parse().expect("valid toml");
        let deps = &doc["dependencies"];
        let entry1 = DependencyEntry::new("serde", &deps["serde"]).expect("should parse");
        assert_eq!(entry1.toml_key(), "serde");
        assert_eq!(entry1.package(), "serde");
        assert_eq!(
            entry1.version(),
            &DependencyVersion::Supported("1.0.0".to_string())
        );

        let entry2 = DependencyEntry::new("my_tokio", &deps["my_tokio"]).expect("should parse");
        assert_eq!(entry2.toml_key(), "my_tokio");
        assert_eq!(entry2.package(), "tokio");
        assert_eq!(
            entry2.version(),
            &DependencyVersion::Supported("1.30.0".to_string())
        );
    }

    #[test]
    fn test_unsupported_keys() {
        let toml = r#"
    git_dep = { git = "https://github.com/foo/bar" }
    path_dep = { path = "../local_lib" }
    registry_dep = { registry = "custom", version = "1.0" }
    workspace_dep = { workspace = true }
    multi_dep = { git = "https://example.com", path = "../foo" }
    "#;
        let doc: DocumentMut = toml.parse().expect("valid toml");
        let g = DependencyEntry::new("git_dep", &doc["git_dep"]).expect("should parse");
        assert_eq!(
            g.version(),
            &DependencyVersion::UnsupportedKeys {
                keys: vec!["git".to_string()]
            }
        );
        let p = DependencyEntry::new("path_dep", &doc["path_dep"]).expect("should parse");
        assert_eq!(
            p.version(),
            &DependencyVersion::UnsupportedKeys {
                keys: vec!["path".to_string()]
            }
        );
        let r = DependencyEntry::new("registry_dep", &doc["registry_dep"]).expect("should parse");
        assert_eq!(
            r.version(),
            &DependencyVersion::UnsupportedKeys {
                keys: vec!["registry".to_string()]
            }
        );
        let w = DependencyEntry::new("workspace_dep", &doc["workspace_dep"]).expect("should parse");
        assert_eq!(
            w.version(),
            &DependencyVersion::UnsupportedKeys {
                keys: vec!["workspace".to_string()]
            }
        );
        let m = DependencyEntry::new("multi_dep", &doc["multi_dep"]).expect("should parse");
        assert_eq!(
            m.version(),
            &DependencyVersion::UnsupportedKeys {
                keys: vec!["git".to_string(), "path".to_string()]
            }
        );
    }

    #[test]
    fn test_missing_required_and_unsupported_value() {
        let toml = r#"
    missing_ver = { features = ["Window"] }
    invalid_val = { version = 123 }
    "#;
        let doc: DocumentMut = toml.parse().expect("valid toml");
        let m = DependencyEntry::new("missing_ver", &doc["missing_ver"]).expect("should parse");
        assert_eq!(
            m.version(),
            &DependencyVersion::MissingRequired {
                fields: vec!["version".to_string()]
            }
        );
        let inv = DependencyEntry::new("invalid_val", &doc["invalid_val"]).expect("should parse");
        assert_eq!(
            inv.version(),
            &DependencyVersion::UnsupportedValue("123".to_string())
        );
    }

    #[test]
    fn test_check_if_latest_semver_rules() {
        let make_dep = |ver: &str| -> DependencyEntry {
            DependencyEntry::Simple(SimpleDependency {
                toml_key: "test".to_string(),
                version: DependencyVersion::Supported(ver.to_string()),
            })
        };

        assert_eq!(
            make_dep("1.2.0").check_if_latest("1.2.0"),
            (Boolean::True, Boolean::False)
        );

        assert_eq!(
            make_dep("^1.2.0").check_if_latest("1.2.5"),
            (Boolean::True, Boolean::False)
        );
        assert_eq!(
            make_dep("^1.2.0").check_if_latest("1.3.0"),
            (Boolean::True, Boolean::False)
        );

        assert_eq!(
            make_dep("^1.2.0").check_if_latest("2.0.0"),
            (Boolean::False, Boolean::True)
        );

        assert_eq!(
            make_dep("^0.1.2").check_if_latest("0.1.5"),
            (Boolean::True, Boolean::False)
        );
        assert_eq!(
            make_dep("^0.1.2").check_if_latest("0.2.0"),
            (Boolean::False, Boolean::True)
        );

        assert_eq!(
            make_dep("0.0.2").check_if_latest("0.0.3"),
            (Boolean::False, Boolean::True)
        );

        assert_eq!(
            make_dep("~1.2.0").check_if_latest("1.2.4"),
            (Boolean::True, Boolean::False)
        );
        assert_eq!(
            make_dep("~1.2.0").check_if_latest("1.3.0"),
            (Boolean::False, Boolean::False)
        );
        assert_eq!(
            make_dep("~1.2.0").check_if_latest("2.0.0"),
            (Boolean::False, Boolean::True)
        );
    }

    #[test]
    fn test_get_dependency_row_valus() {
        let make_dep = |ver: DependencyVersion| -> DependencyEntry {
            DependencyEntry::Simple(SimpleDependency {
                toml_key: "test".to_string(),
                version: ver,
            })
        };

        let up_to_date = make_dep(DependencyVersion::Supported("1.0.0".to_string()));
        assert_eq!(
            up_to_date.get_dependency_row_valus("1.0.0".to_string()),
            (Boolean::True, Boolean::False, "1.0.0".to_string())
        );

        let outdated = make_dep(DependencyVersion::Supported("1.0.0".to_string()));
        assert_eq!(
            outdated.get_dependency_row_valus("2.0.0".to_string()),
            (Boolean::False, Boolean::True, "1.0.0 -> 2.0.0".to_string())
        );

        let unsupported = make_dep(DependencyVersion::UnsupportedKeys {
            keys: vec!["git".to_string()],
        });
        assert_eq!(
            unsupported.get_dependency_row_valus("any".to_string()),
            (
                Boolean::True,
                Boolean::False,
                "Unsupported key: git".to_string()
            )
        );

        let missing = make_dep(DependencyVersion::MissingRequired {
            fields: vec!["version".to_string()],
        });
        assert_eq!(
            missing.get_dependency_row_valus("any".to_string()),
            (
                Boolean::True,
                Boolean::False,
                "Missing: version".to_string()
            )
        );
    }
}
