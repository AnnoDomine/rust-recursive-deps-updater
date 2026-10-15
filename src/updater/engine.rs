//! Manifest update engine implementation.

use std::{fmt::Display, fs, io, path::PathBuf};

use toml_edit::{DocumentMut, Formatted, Item, Value};

use crate::{
    constants::CARGO_TOML_FILE_NAME,
    enums::{DependencyEntry, DependencySection},
    functions::create_absolute_path,
    meta_status, simple_status,
};

/// Metadata required by [`UpdateEngine`] to update a dependency in a manifest file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdaterInformation {
    /// Manifest section where the dependency is located.
    pub section: DependencySection,
    /// TOML key identifying the dependency within the section.
    pub toml_key: String,
    /// Optional SemVer operator prefix (e.g. `^`, `~`, `=`) to preserve.
    pub prefix: Option<String>,
    /// Target latest version to upgrade to.
    pub latest: Option<String>,
}

impl Display for UpdaterInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Section: {:} - Toml-Key: {:} - Prefix: {:?} - Latest: {:?}",
            self.section, self.toml_key, self.prefix, self.latest
        )
    }
}

impl UpdaterInformation {
    /// Sets the target latest version string.
    ///
    /// # Arguments
    /// * `latest` - Target version string.
    pub fn set_latest(&mut self, latest: &str) {
        self.latest = Some(latest.to_string());
    }

    /// Returns the target version string formatted with optional SemVer operator prefix.
    ///
    /// # Returns
    /// An `Option<String>` containing the prefixed version or `None` if unset.
    pub fn get_latest(&self) -> Option<String> {
        if let Some(l) = &self.latest {
            if let Some(p) = &self.prefix {
                Some(format!("{p}{l}"))
            } else {
                Some(l.to_string())
            }
        } else {
            None
        }
    }
}

/// Engine coordinating updates to `Cargo.toml` manifests using `toml_edit`.
#[derive(Debug, Clone)]
pub struct UpdateEngine {
    #[allow(dead_code)]
    pure_toml: DocumentMut,
    toml_path: PathBuf,
}

impl PartialEq for UpdateEngine {
    fn eq(&self, other: &Self) -> bool {
        self.toml_path == other.toml_path
    }
}

impl Eq for UpdateEngine {}

impl UpdateEngine {
    /// Creates a new `UpdateEngine` instance.
    ///
    /// # Arguments
    /// * `toml` - Parsed mutable TOML document.
    /// * `path` - Filesystem path to the `Cargo.toml` manifest.
    ///
    /// # Returns
    /// An initialized `UpdateEngine` instance.
    pub fn new(toml: DocumentMut, path: PathBuf) -> Self {
        Self {
            pure_toml: toml,
            toml_path: path,
        }
    }

    /// Generates updater information for a given dependency entry and section.
    ///
    /// # Arguments
    /// * `section` - Target dependency section.
    /// * `entry` - Dependency entry to extract prefix and key from.
    ///
    /// # Returns
    /// An initialized `UpdaterInformation` payload.
    pub fn get_updater_information(
        &self,
        section: DependencySection,
        entry: &DependencyEntry,
    ) -> UpdaterInformation {
        let prefix = entry.get_version_prefix();
        simple_status!(
            log::LevelFilter::Trace,
            crate::logger::status_codes::Module::UPDATER,
            100,
            format!("Create updater information for {:?}", entry.toml_key())
        );
        UpdaterInformation {
            section,
            toml_key: entry.toml_key().to_string(),
            prefix,
            latest: None,
        }
    }

    /// Atomically persists the modified TOML document to disk.
    ///
    /// # Errors
    /// Returns [`io::Error`] if file writing or atomic replacement fails.
    pub fn save(&self) -> io::Result<()> {
        if let Ok(mut path) = create_absolute_path(&self.toml_path, None) {
            path.push(CARGO_TOML_FILE_NAME);
            let content = self.pure_toml.to_string();
            let tmp_path = path.with_extension("tmp");

            simple_status!(
                log::LevelFilter::Trace,
                crate::logger::status_codes::Module::UPDATER,
                200,
                "Updating Cargo.toml!"
            );

            fs::write(&tmp_path, content)?;
            fs::rename(&tmp_path, &path)?;

            simple_status!(
                log::LevelFilter::Trace,
                crate::logger::status_codes::Module::UPDATER,
                200,
                "Cargo.toml updated!"
            );
        }
        Ok(())
    }

    /// Applies in-place version modification to the internal TOML document.
    ///
    /// # Arguments
    /// * `information` - Updater metadata identifying the target dependency and version.
    pub fn update_dependency(&mut self, information: &UpdaterInformation) {
        meta_status!(
            log::LevelFilter::Trace,
            crate::logger::status_codes::Module::UPDATER,
            100,
            "Start updating dependecy",
            information
        );
        if let Some(latest) = &information.get_latest()
            && let Some(toml) = information.section.get_table_mut(&mut self.pure_toml)
        {
            if information.section.is_dependency_section().is_some()
                && let Some(field) = toml.get_mut("version")
            {
                meta_status!(
                    log::LevelFilter::Trace,
                    crate::logger::status_codes::Module::UPDATER,
                    100,
                    "Start updating dependecy section",
                    &mut *field
                );
                if let Item::Value(val) = field {
                    let decor = val.decor().clone();
                    *val = Value::String(Formatted::new(latest.to_string()));
                    *val.decor_mut() = decor;
                } else {
                    *field = Item::Value(Value::String(Formatted::new(latest.to_string())));
                }
            } else if let Some(entry) = toml.get_mut(&information.toml_key) {
                meta_status!(
                    log::LevelFilter::Trace,
                    crate::logger::status_codes::Module::UPDATER,
                    100,
                    "Updating dependecy in section",
                    &mut *entry
                );
                match entry {
                    Item::Value(val @ Value::String(_)) => {
                        let decor = val.decor().clone();
                        *val = Value::String(Formatted::new(latest.to_string()));
                        *val.decor_mut() = decor;
                    }
                    Item::Value(Value::InlineTable(inline)) => {
                        if let Some(v_key) = inline.get_mut("version") {
                            let decor = v_key.decor().clone();
                            *v_key = Value::String(Formatted::new(latest.to_string()));
                            *v_key.decor_mut() = decor;
                        }
                    }
                    Item::Table(table) => {
                        if let Some(field) = table.get_mut("version") {
                            if let Item::Value(val) = field {
                                let decor = val.decor().clone();
                                *val = Value::String(Formatted::new(latest.to_string()));
                                *val.decor_mut() = decor;
                            } else {
                                *field =
                                    Item::Value(Value::String(Formatted::new(latest.to_string())));
                            }
                        };
                    }
                    _ => {}
                }
            }
            simple_status!(
                log::LevelFilter::Trace,
                crate::logger::status_codes::Module::UPDATER,
                100,
                "toml updated! Save."
            );
        }
    }

    /// Returns the parsed TOML document formatted as a string.
    ///
    /// # Returns
    /// Formatted TOML string representation.
    pub fn toml_string(&self) -> String {
        self.pure_toml.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{DependencyVersion, SimpleDependency, TargetDepKind};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_update_simple_dependency() {
        let manifest = r#"[package]
name = "test_pkg"
version = "0.1.0"

[dependencies]
# Top comment
colored = "3.1.1" # Inline comment
serde = "1.0.0"
"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, PathBuf::from("./"));

        let mut info = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "colored".to_string(),
            prefix: None,
            latest: None,
        };
        info.set_latest("3.2.0");

        engine.update_dependency(&info);
        let updated = engine.toml_string();

        assert!(updated.contains(r#"colored = "3.2.0""#));
        assert!(updated.contains("# Top comment"));
        assert!(updated.contains("# Inline comment"));
        assert!(updated.contains(r#"serde = "1.0.0""#));
    }

    #[tokio::test]
    async fn test_update_simple_dependency_with_prefix() {
        let manifest = r#"[dependencies]
colored = "^3.1.1"
tokio = "~1.53.0"
"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, PathBuf::from("./"));

        let mut info_caret = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "colored".to_string(),
            prefix: Some("^".to_string()),
            latest: None,
        };
        info_caret.set_latest("3.2.0");
        engine.update_dependency(&info_caret);

        let mut info_tilde = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "tokio".to_string(),
            prefix: Some("~".to_string()),
            latest: None,
        };
        info_tilde.set_latest("1.54.0");
        engine.update_dependency(&info_tilde);

        let updated = engine.toml_string();
        assert!(updated.contains(r#"colored = "^3.2.0""#));
        assert!(updated.contains(r#"tokio = "~1.54.0""#));
    }

    #[tokio::test]
    async fn test_update_inline_table_dependency() {
        let manifest = r#"[dependencies]
# Inline table dep
serde = { version = "1.0.100", features = ["derive"] } # Trailing comment
"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, PathBuf::from("./"));

        let mut info = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "serde".to_string(),
            prefix: None,
            latest: None,
        };
        info.set_latest("1.0.218");
        engine.update_dependency(&info);

        let updated = engine.toml_string();
        assert!(updated.contains(r#"version = "1.0.218""#));
        assert!(updated.contains(r#"features = ["derive"]"#));
        assert!(updated.contains("# Inline table dep"));
        assert!(updated.contains("# Trailing comment"));
    }

    #[tokio::test]
    async fn test_update_table_dependency() {
        let manifest = r#"[dependencies.serde]
# Version comment
version = "1.0.100"
features = ["derive"]
"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, PathBuf::from("./"));

        let mut info = UpdaterInformation {
            section: DependencySection::Table {
                parent: Box::new(DependencySection::Normal),
                toml_key: "serde".to_string(),
            },
            toml_key: "serde".to_string(),
            prefix: None,
            latest: None,
        };
        info.set_latest("1.0.218");
        engine.update_dependency(&info);

        let updated = engine.toml_string();
        assert!(updated.contains(r#"version = "1.0.218""#));
        assert!(updated.contains(r#"features = ["derive"]"#));
        assert!(updated.contains("# Version comment"));
    }

    #[tokio::test]
    async fn test_update_nested_table_in_section() {
        let manifest = r#"[dependencies]
other = "0.1.0"

[dependencies.sub]
version = "0.2.0"
"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, PathBuf::from("./"));

        let mut info = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "sub".to_string(),
            prefix: None,
            latest: None,
        };
        info.set_latest("0.3.0");
        engine.update_dependency(&info);

        let updated = engine.toml_string();
        assert!(updated.contains(r#"version = "0.3.0""#));
        assert!(updated.contains(r#"other = "0.1.0""#));
    }

    #[tokio::test]
    async fn test_update_dev_and_workspace_dependencies() {
        let manifest = r#"[dev-dependencies]
pretty_assertions = "1.0.0"

[workspace.dependencies]
tokio = "1.0.0"
"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, PathBuf::from("./"));

        let mut dev_info = UpdaterInformation {
            section: DependencySection::Dev,
            toml_key: "pretty_assertions".to_string(),
            prefix: None,
            latest: None,
        };
        dev_info.set_latest("1.4.0");
        engine.update_dependency(&dev_info);

        let mut ws_info = UpdaterInformation {
            section: DependencySection::Workspace,
            toml_key: "tokio".to_string(),
            prefix: None,
            latest: None,
        };
        ws_info.set_latest("1.50.0");
        engine.update_dependency(&ws_info);

        let updated = engine.toml_string();
        assert!(updated.contains(r#"pretty_assertions = "1.4.0""#));
        assert!(updated.contains(r#"tokio = "1.50.0""#));
    }

    #[tokio::test]
    async fn test_update_target_dependencies() {
        let manifest = r#"[target.'cfg(unix)'.dependencies]
nix = "0.26.0"
"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, PathBuf::from("./"));

        let mut info = UpdaterInformation {
            section: DependencySection::Target {
                target: "cfg(unix)".to_string(),
                kind: TargetDepKind::Normal,
            },
            toml_key: "nix".to_string(),
            prefix: None,
            latest: None,
        };
        info.set_latest("0.27.0");
        engine.update_dependency(&info);

        let updated = engine.toml_string();
        assert!(updated.contains(r#"nix = "0.27.0""#));
    }

    #[tokio::test]
    async fn test_updater_information_prefix_and_latest() {
        let mut info = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "test_dep".to_string(),
            prefix: Some("~".to_string()),
            latest: None,
        };
        assert_eq!(info.get_latest(), None);

        info.set_latest("1.2.3");
        assert_eq!(info.get_latest(), Some("~1.2.3".to_string()));

        info.prefix = None;
        assert_eq!(info.get_latest(), Some("1.2.3".to_string()));

        let display_output = format!("{info}");
        assert!(display_output.contains("Toml-Key: test_dep"));
    }

    #[tokio::test]
    async fn test_get_updater_information_from_entry() {
        let manifest = r#"[dependencies]
colored = "^3.1.1"
"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let engine = UpdateEngine::new(doc, PathBuf::from("./"));

        let entry = DependencyEntry::Simple(SimpleDependency {
            toml_key: "colored".to_string(),
            version: DependencyVersion::Supported("^3.1.1".to_string()),
        });

        let info = engine.get_updater_information(DependencySection::Normal, &entry);
        assert_eq!(info.toml_key, "colored");
        assert_eq!(info.section, DependencySection::Normal);
        assert_eq!(info.prefix, Some("^".to_string()));
        assert_eq!(info.latest, None);
    }

    #[tokio::test]
    async fn test_save_to_disk() {
        let dir = tempdir().expect("temporary directory creation");
        let cargo_toml_path = dir.path().join("Cargo.toml");
        let initial_content = r#"[package]
name = "atomic_test"
version = "0.1.0"

[dependencies]
colored = "1.0.0"
"#;
        fs::write(&cargo_toml_path, initial_content).expect("initial write");

        let doc: DocumentMut = initial_content.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, dir.path().to_path_buf());

        let mut info = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "colored".to_string(),
            prefix: None,
            latest: None,
        };
        info.set_latest("2.0.0");

        engine.update_dependency(&info);
        engine.save().expect("save succeeds");

        let saved_content = fs::read_to_string(&cargo_toml_path).expect("read saved file");
        assert!(saved_content.contains(r#"colored = "2.0.0""#));

        let tmp_path = cargo_toml_path.with_extension("tmp");
        assert!(!tmp_path.exists());
    }

    #[tokio::test]
    async fn test_big_toml() {
        let manifest = r#"[package]
name = 'test-project'
version = '0.1.0'
edition = '2024'

[dependencies]
# 1. Einfache String-Dependency (Simple)
serde = '1.0.190'

# 2. Inline-Table mit Features (Inline)
tokio = { version = '1.35.0', features = ['full'] }

# 3. Umbenanntes Package (Inline mit 'package')
my_colored = { package = 'colored', version = '2.1.0' }

# 4. Nicht unterstützte Keys (z.B. Git / Path -> landet in UnsupportedKeys)
local_dep = { path = '../local_lib' }

# 6. Dev-Dependencies
[dev-dependencies]
tempfile = '3.10.0'

# 7. Build-Dependencies
[build-dependencies]
cc = '^1.0.83'

# 8. Target-spezifische Dependencies
[target.'cfg(windows)'.dependencies]
windows-sys = '0.52.0'

# 5. Explizite Tabellensektion (Table)
[dependencies.clap]
version = '4.4.18'
features = ['derive']

[workspace.dependencies]
serde_json = '1.0.0'"#;
        let doc: DocumentMut = manifest.parse().expect("valid toml");
        let mut engine = UpdateEngine::new(doc, PathBuf::from("./"));

        // 1. Update serde in [dependencies]
        let mut info_serde = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "serde".to_string(),
            prefix: None,
            latest: None,
        };
        info_serde.set_latest("1.0.218");
        engine.update_dependency(&info_serde);

        // 2. Update tokio in [dependencies] (inline table)
        let mut info_tokio = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "tokio".to_string(),
            prefix: None,
            latest: None,
        };
        info_tokio.set_latest("1.38.0");
        engine.update_dependency(&info_tokio);

        // 3. Update my_colored (renamed package) in [dependencies]
        let mut info_colored = UpdaterInformation {
            section: DependencySection::Normal,
            toml_key: "my_colored".to_string(),
            prefix: None,
            latest: None,
        };
        info_colored.set_latest("2.2.0");
        engine.update_dependency(&info_colored);

        // 4. Update tempfile in [dev-dependencies]
        let mut info_tempfile = UpdaterInformation {
            section: DependencySection::Dev,
            toml_key: "tempfile".to_string(),
            prefix: None,
            latest: None,
        };
        info_tempfile.set_latest("3.12.0");
        engine.update_dependency(&info_tempfile);

        // 5. Update cc in [build-dependencies] incl. prefix
        let mut info_cc = UpdaterInformation {
            section: DependencySection::Build,
            toml_key: "cc".to_string(),
            prefix: Some("^".to_string()),
            latest: None,
        };
        info_cc.set_latest("1.1.0");
        engine.update_dependency(&info_cc);

        // 6. Update windows-sys in [target.'cfg(windows)'.dependencies]
        let mut info_windows = UpdaterInformation {
            section: DependencySection::Target {
                target: "cfg(windows)".to_string(),
                kind: TargetDepKind::Normal,
            },
            toml_key: "windows-sys".to_string(),
            prefix: None,
            latest: None,
        };
        info_windows.set_latest("0.59.0");
        engine.update_dependency(&info_windows);

        // 7. Update clap in [dependencies.clap]
        let mut info_clap = UpdaterInformation {
            section: DependencySection::Table {
                parent: Box::new(DependencySection::Normal),
                toml_key: "clap".to_string(),
            },
            toml_key: "clap".to_string(),
            prefix: None,
            latest: None,
        };
        info_clap.set_latest("4.5.10");
        engine.update_dependency(&info_clap);

        // 8. Update serde_json in [workspace.dependencies]
        let mut info_serde_json = UpdaterInformation {
            section: DependencySection::Workspace,
            toml_key: "serde_json".to_string(),
            prefix: None,
            latest: None,
        };
        info_serde_json.set_latest("1.0.120");
        engine.update_dependency(&info_serde_json);

        let updated = engine.toml_string();

        // Assertions: All updated versions
        assert!(updated.contains(r#"serde = "1.0.218""#));
        assert!(updated.contains(r#"version = "1.38.0""#));
        assert!(updated.contains(r#"features = ['full']"#));
        assert!(updated.contains(r#"package = 'colored'"#));
        assert!(updated.contains(r#"version = "2.2.0""#));
        assert!(updated.contains(r#"tempfile = "3.12.0""#));
        assert!(updated.contains(r#"cc = "^1.1.0""#));
        assert!(updated.contains(r#"windows-sys = "0.59.0""#));
        assert!(updated.contains(r#"version = "4.5.10""#));
        assert!(updated.contains(r#"features = ['derive']"#));
        assert!(updated.contains(r#"serde_json = "1.0.120""#));

        // Unsupported dep is untouched
        assert!(updated.contains(r#"local_dep = { path = '../local_lib' }"#));

        // Comments are preserved
        assert!(updated.contains("# 1. Einfache String-Dependency (Simple)"));
        assert!(updated.contains("# 2. Inline-Table mit Features (Inline)"));
        assert!(updated.contains("# 3. Umbenanntes Package (Inline mit 'package')"));
        assert!(updated.contains("# 4. Nicht unterstützte Keys"));
        assert!(updated.contains("# 6. Dev-Dependencies"));
        assert!(updated.contains("# 7. Build-Dependencies"));
        assert!(updated.contains("# 8. Target-spezifische Dependencies"));
        assert!(updated.contains("# 5. Explizite Tabellensektion (Table)"));
    }
}
