//! Representation of prjects and dependencies
//! The is not the representation for the config.
//! It holds the information as a struct to handle navigation to the CLI and updating the Cargo.toml

use crate::{
    config::{model::*, rrdu_config::RrduConfig},
    constants::*,
    enums::*,
    errors::*,
    functions::*,
};
use std::collections::HashMap;
use std::path::PathBuf;
use toml_edit::*;

impl DependencyEntry {
    /// Entry function for parsing the DEPENDENCY.
    ///
    /// This function internal handles all posible schemas.
    /// If a schema is not supported or wrong defined (e.g. no version key) it returns None to not getting mapped.
    ///
    /// * We only support dependencies which are downloadable and updateable from crates.io
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

    /// Function to parse inline DEPENDENCY definitions.
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
            if UNSUPPORTED_DEPENDENCY_KEYS.contains(&key.as_str()) {
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

    /// Return the dependency name (from crates.io)
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

    /// Return the 'crates.io' package name, if the dependency us supported. else None
    pub fn is_dep_included(&self) -> Result<&str, DEPENDENCYRow> {
        match &self.version() {
            DependencyVersion::Supported(_) => Ok(self.package()),
            _ => Err(self.get_dependency_row_valus(None)),
        }
    }

    /// Return the values needed for the DEPENDENCY row inside the CI and the CLI
    ///
    /// Return:
    /// * is_latest: Boolean
    /// * needs_migration: Boolean
    /// * message: String
    ///
    /// `(Boolean, Boolean, String)`
    pub fn get_dependency_row_valus(&self, latest: Option<String>) -> DEPENDENCYRow {
        let version = self.version();
        match latest {
            Some(l) => {
                match version {
                    DependencyVersion::Supported(v) => {
                        let (is_latest, needs_migration) = self.check_if_latest(&l);
                        let message = match &is_latest {
                            Boolean::True => v.to_string(),
                            Boolean::False => format!("{:} -> {:}", v, l),
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
                    DependencyVersion::Excluded(r) => {
                        (Boolean::True, Boolean::False, self.get_exclude_reason(r))
                    }
                }
            }
            None => (
                Boolean::True,
                Boolean::False,
                format!("Scan not fullfilled for DEPENDENCY '{:}'", self.package()),
            ),
        }
    }

    /// Returns the reason why a dependency is excluded
    fn get_exclude_reason(&self, reason: &ExcludeReason) -> String {
        match reason {
            ExcludeReason::ExcludedInProject => "Excluded project wide!".to_string(),
            ExcludeReason::ExcludedInSection => "Excluded in section!".to_string(),
            ExcludeReason::ExcludedSection => "Section excluded!".to_string(),
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

    /// Mark the dependency as excluded with a provided reason.
    pub fn exclude_dep(&mut self, reason: &ExcludeReason) {
        match self {
            Self::Simple(d) => d.version = DependencyVersion::Excluded(reason.clone()),
            Self::Inline(d) => d.version = DependencyVersion::Excluded(reason.clone()),
            Self::Table(d) => d.version = DependencyVersion::Excluded(reason.clone()),
        };
    }
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

impl DependencySection {
    pub fn new(section: &str) -> Option<Self> {
        match section {
            "dependencies" => Some(Self::Normal),
            "dev-dependencies" => Some(Self::Dev),
            "build-dependencies" => Some(Self::Build),
            "workspace.dependencies" => Some(Self::Workspace),
            _ => match section.rsplit_once(".") {
                Some((p, s)) => match Self::get_target_kind(s) {
                    Some(k) => p.strip_prefix("target.").map(|target| Self::Target {
                        target: target.to_string(),
                        kind: k,
                    }),
                    None => Self::new(p).map(|parent| Self::Table {
                        parent: Box::new(parent),
                        toml_key: (&s).to_string(),
                    }),
                },
                None => None,
            },
        }
    }

    pub fn get_target_kind(kind: &str) -> Option<TargetDepKind> {
        match kind {
            "dependencies" => Some(TargetDepKind::Normal),
            "dev-dependencies" => Some(TargetDepKind::Dev),
            "build-dependencies" => Some(TargetDepKind::Build),
            _ => None,
        }
    }

    pub fn get_table_mut<'a>(&self, doc: &'a mut DocumentMut) -> Option<&'a mut dyn TableLike> {
        match self {
            Self::Normal => doc["dependencies"].as_table_like_mut(),
            Self::Dev => doc["dev-dependencies"].as_table_like_mut(),
            Self::Build => doc["build-dependencies"].as_table_like_mut(),
            Self::Workspace => doc["workspace"]["dependencies"].as_table_like_mut(),
            Self::Target { target, kind } => {
                let kind_str = match kind {
                    TargetDepKind::Normal => "dependencies",
                    TargetDepKind::Dev => "dev-dependencies",
                    TargetDepKind::Build => "build-dependencies",
                };
                doc["target"][target][kind_str].as_table_like_mut()
            }
            Self::Table { parent, toml_key } => {
                let parent_table = parent.get_table_mut(doc)?;
                parent_table
                    .get_mut(toml_key)
                    .and_then(|item| item.as_table_like_mut())
            }
        }
    }

    pub fn get_item<'a>(&self, doc: &'a DocumentMut) -> Option<&'a Item> {
        match self {
            Self::Normal => Some(&doc["dependencies"]),
            Self::Dev => Some(&doc["dev-dependencies"]),
            Self::Build => Some(&doc["build-dependencies"]),
            Self::Workspace => Some(&doc["workspace"]["dependencies"]),
            Self::Target { target, kind } => {
                let kind_str = match kind {
                    TargetDepKind::Normal => "dependencies",
                    TargetDepKind::Dev => "dev-dependencies",
                    TargetDepKind::Build => "build-dependencies",
                };
                Some(&doc["target"][target][kind_str])
            }
            Self::Table { parent, toml_key } => {
                let parent_table = parent.get_item(doc)?;
                Some(&parent_table[toml_key])
            }
        }
    }

    pub fn is_dependency_section(&self) -> Option<&String> {
        match self {
            Self::Table { toml_key, .. } => Some(toml_key),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projects {
    pub name: String,
    pub path: PathBuf,
    pub deps: ProjectDEPENDENCY,
    pub config: ProjectConfig,
}

impl Projects {
    pub fn new(config: ProjectConfig, is_excuded: Boolean) -> Self {
        Self {
            name: config.project.clone(),
            path: PathBuf::from(config.path.clone()),
            deps: Self::define_deps(is_excuded),
            config: config.clone(),
        }
    }

    fn define_deps(is_excuded: Boolean) -> ProjectDEPENDENCY {
        match is_excuded {
            Boolean::True => ProjectDEPENDENCY::ExcludedProject,
            Boolean::False => ProjectDEPENDENCY::Map(HashMap::new()),
        }
    }

    pub fn parse_project(&mut self) {
        match self.deps {
            ProjectDEPENDENCY::ExcludedProject => {}
            ProjectDEPENDENCY::Map(_) => {
                match self.get_toml_content() {
                    Ok(toml) => self.collect_sections(toml),
                    Err(e) => println!("{:}", e),
                };
            }
        }
    }

    pub fn get_toml_content_from_path(toml_path: PathBuf) -> Result<DocumentMut, FileError> {
        let file = std::fs::read_to_string(&toml_path).map_err(|source| FileError::Io {
            path: toml_path.clone(),
            source,
        })?;
        file.parse::<DocumentMut>()
            .map_err(|source| FileError::Toml {
                path: toml_path.clone(),
                source,
            })
    }

    /// This function read and retreive the content of 'Cargo.toml'.
    fn get_toml_content(&self) -> Result<DocumentMut, FileError> {
        let toml_path = create_absolute_path(&self.path, Some(CARGO_TOML_FILE_NAME))?;
        Self::get_toml_content_from_path(toml_path)
    }

    pub fn set_deps(&mut self, deps: ProjectDEPENDENCY) {
        self.deps = deps;
    }

    fn map_deps(&self, section: DependencySection, doc: DocumentMut) -> DEPENDENCYSectionMap {
        let mut map: HashMap<String, DependencyEntry> = HashMap::new();
        if let Some(t) = section.get_item(&doc) {
            match section.is_dependency_section() {
                Some(key) => {
                    if let Some(dep) = DependencyEntry::new(key, t) {
                        map.insert(key.to_string(), dep);
                    };
                }
                None => {
                    if let Some(table_items) = t.as_table() {
                        for (key, item) in table_items {
                            if !item.is_table()
                                && let Some(dep) = DependencyEntry::new(key, item)
                            {
                                map.insert(key.to_string(), dep);
                            };
                        }
                    };
                }
            };
        };
        if map.is_empty() {
            return DEPENDENCYSectionMap::Empty;
        }
        DEPENDENCYSectionMap::Map(map)
    }

    pub fn search_sections(
        &self,
        section: Option<String>,
        key: &str,
        item: Item,
        collector: &mut HashMap<DependencySection, Option<DEPENDENCYSectionMap>>,
    ) {
        match &item {
            Item::Table(t) => {
                for (k_def, v_def) in t
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect::<Vec<(String, Item)>>()
                {
                    let sec = match &section {
                        Some(s) => format!("{:}.{:}", s, key.to_owned()),
                        None => key.to_owned(),
                    };
                    self.search_sections(Some(sec), &k_def.to_string(), v_def.clone(), collector);
                }
            }
            _ => {
                let sec = section.unwrap_or(key.to_owned());
                let dependency_section = DependencySection::new(&sec);
                if let Some(mapable_section) = dependency_section {
                    let excluded = self.config.exclude.clone();
                    // If the section is inside the section excluded in the project definition from the config, means the hole section is excluded.
                    match excluded.section.get(&sec) {
                        Some(excluded_section_values) if excluded_section_values.is_empty() => {
                            collector.insert(
                                mapable_section,
                                Some(DEPENDENCYSectionMap::ExcludedSection),
                            );
                        }
                        _ => {
                            collector.insert(mapable_section, None);
                        }
                    };
                }
            }
        }
    }

    fn collect_sections(&mut self, toml_doc: DocumentMut) {
        let k = &toml_doc
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<Vec<(String, Item)>>();
        let mut collector: HashMap<DependencySection, Option<DEPENDENCYSectionMap>> =
            HashMap::new();
        for (k_def, v_def) in k {
            self.search_sections(None, &k_def.to_string(), v_def.clone(), &mut collector);
        }
        let mut mapped_deps: HashMap<DependencySection, DEPENDENCYSectionMap> = HashMap::new();
        for (sec, deps) in collector {
            if deps.is_none() {
                mapped_deps.insert(sec.clone(), self.map_deps(sec, toml_doc.clone()));
            }
        }
        self.set_deps(ProjectDEPENDENCY::Map(mapped_deps));
    }

    /// Takes a mutatable hashmap to collect the included and supported dependencies to scan for newer versions
    pub fn list_all_deps(&self, collected_deps: &mut HashMap<String, DependencyCollectionVersion>) {
        if let ProjectDEPENDENCY::Map(sec_map) = &self.deps {
            for deps in sec_map.values() {
                if let DEPENDENCYSectionMap::Map(dep_map) = deps {
                    for dep in dep_map.values() {
                        if let Ok(p) = dep.is_dep_included() {
                            collected_deps.insert(p.to_string(), DependencyCollectionVersion::None);
                        };
                    }
                }
            }
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub projects: Vec<Projects>,
    pub collected_deps: HashMap<String, DependencyCollectionVersion>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            collected_deps: HashMap::new(),
        }
    }

    /// Load the .rrduconfig file to parse the projects.
    /// Trigger Self::collect_projects if config could loaded (is supported by the executed version of 'rrdu' and is present).
    /// Automatic respects config key 'workspace.sub-config'.
    ///
    /// Arguments:
    /// * path: Path to .rrduconfig yaml.
    ///   * If provided, loads a config from a specific path (Calls 'RrduConfig::load_from_path(path)')
    ///   * None if is not provided to run the initial call (Calls 'RrduConfig::new()' as it loads the rrdu config from the executed path)
    pub fn read_rrdu_config(&mut self, path: Option<PathBuf>) -> Result<(), FileError> {
        let config = match &path {
            Some(p) => RrduConfig::load_from_path(p),
            None => Ok(RrduConfig::new()),
        };
        match config {
            Ok(c) => {
                self.collect_projects(c)?;
            }
            Err(e) => return Err(FileError::ConfigError(e)),
        };
        Ok(())
    }

    /// Collects the projects from a config and applies excluded definition.
    fn collect_projects(&mut self, config: RrduConfig) -> Result<(), FileError> {
        for project in config.workspace {
            if config.updater.exclude.contains(&project.project) {
                self.projects.push(Projects::new(project, Boolean::True));
            } else {
                self.add_project(project)?;
            }
        }
        Ok(())
    }

    /// Reads the .rrduconfig yaml and parse the workspace.
    ///
    /// Additional handles sub-config definition.
    fn add_project(&mut self, project: ProjectConfig) -> Result<(), FileError> {
        match project.sub_config {
            SubConfig::Bool(true) => {
                let sub_config_path = project.get_sub_config_path()?;
                self.read_rrdu_config(Some(sub_config_path))?;
            }
            _ => {
                let mut p = Projects::new(project, Boolean::False);
                println!("{:#?}", p);
                p.parse_project();
                p.list_all_deps(&mut self.collected_deps);
                self.projects.push(p);
            }
        };
        Ok(())
    }

    /// Fetch the dependency versions from 'crates.io'
    pub fn fetch_latest_versions(&mut self) {
        todo!("Add fetch logic as part of registry")
    }
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
            up_to_date.get_dependency_row_valus(Some("1.0.0".to_string())),
            (Boolean::True, Boolean::False, "1.0.0".to_string())
        );

        let outdated = make_dep(DependencyVersion::Supported("1.0.0".to_string()));
        assert_eq!(
            outdated.get_dependency_row_valus(Some("2.0.0".to_string())),
            (Boolean::False, Boolean::True, "1.0.0 -> 2.0.0".to_string())
        );

        let unsupported = make_dep(DependencyVersion::UnsupportedKeys {
            keys: vec!["git".to_string()],
        });
        assert_eq!(
            unsupported.get_dependency_row_valus(Some("any".to_string())),
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
            missing.get_dependency_row_valus(Some("any".to_string())),
            (
                Boolean::True,
                Boolean::False,
                "Missing: version".to_string()
            )
        );

        // Unscanned dependency
        let unscanned = make_dep(DependencyVersion::Supported("1.0.0".to_string()));
        assert_eq!(
            unscanned.get_dependency_row_valus(None),
            (
                Boolean::True,
                Boolean::False,
                "Scan not fullfilled for DEPENDENCY 'test'".to_string()
            )
        );

        // Excluded dependencies
        let excluded_by_project = make_dep(DependencyVersion::Excluded(
            ExcludeReason::ExcludedInProject,
        ));
        assert_eq!(
            excluded_by_project.get_dependency_row_valus(Some("2.0.0".to_string())),
            (
                Boolean::True,
                Boolean::False,
                "Excluded project wide!".to_string()
            )
        );

        let excluded_by_section = make_dep(DependencyVersion::Excluded(
            ExcludeReason::ExcludedInSection,
        ));
        assert_eq!(
            excluded_by_section.get_dependency_row_valus(Some("2.0.0".to_string())),
            (
                Boolean::True,
                Boolean::False,
                "Excluded in section!".to_string()
            )
        );

        let excluded_section =
            make_dep(DependencyVersion::Excluded(ExcludeReason::ExcludedSection));
        assert_eq!(
            excluded_section.get_dependency_row_valus(Some("2.0.0".to_string())),
            (
                Boolean::True,
                Boolean::False,
                "Section excluded!".to_string()
            )
        );
    }
}
