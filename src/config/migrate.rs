//! Migration routines from legacy `.rrduconfig` (YAML) to `rrdu.toml` (TOML).
//!
//! Provides [`MigrateYamlToTomlConfig`](crate::config::migrate::MigrateYamlToTomlConfig) for transforming legacy models,
//! mapping exclusion rules, and persisting the migrated configuration to disk.

#![allow(deprecated)]

use std::{fs::remove_file, path::Path};

use crate::{
    config::{
        model::ExcludeConfig,
        rrdu_config::RrduConfig,
        rrdu_toml::{RrduTomlConfig, TomlWorkspace, TomlWorkspaceExclude},
    },
    constants::{RRDU_TOML_FILE_NAME, RRDUCONFIG_FILE_NAME},
    enums::SubConfig,
    functions::create_absolute_path,
};

/// Helper providing migration methods from legacy YAML configuration to modern TOML schema.
pub struct MigrateYamlToTomlConfig;

impl MigrateYamlToTomlConfig {
    /// Loads a legacy configuration from the given path or returns a default configuration.
    pub fn load_legacy_config<P: AsRef<Path>>(path: P) -> RrduConfig {
        RrduConfig::load_from_path(path).unwrap_or_default()
    }

    /// Migrates legacy dependency exclusion rules into TOML workspace exclusion settings.
    pub fn migrate_workspace_exclude(ws_exc: ExcludeConfig) -> Option<TomlWorkspaceExclude> {
        let mut exc = TomlWorkspaceExclude::default();
        if ws_exc.project.is_empty() && ws_exc.section.is_empty() {
            return None;
        };
        for p in ws_exc.project {
            exc.add_project_dep(&p);
        }
        for s in ws_exc.section {
            let sec = s.0;
            let deps = s.1;
            exc.add_section(&sec);
            for dep in deps {
                exc.add_section_dep(&sec, &dep);
            }
        }
        Some(exc)
    }

    /// Transforms a legacy [`RrduConfig`] instance into a modern [`RrduTomlConfig`].
    pub fn migrate_config(legacy: RrduConfig) -> RrduTomlConfig {
        let mut toml = RrduTomlConfig::default();

        for project in legacy.workspace {
            if let SubConfig::Bool(true) = project.sub_config {
                toml.configs.insert(project.project, project.path);
            } else {
                let mut ws = TomlWorkspace::new(&project.path);
                ws.set_label(&project.project);
                ws.exclude = MigrateYamlToTomlConfig::migrate_workspace_exclude(project.exclude);
                toml.workspaces.insert(project.project, ws);
            }
        }

        toml.rrdu.max_lines = legacy.updater.max_lines;
        if !legacy.updater.exclude.is_empty() {
            toml.rrdu.exclude = Some(legacy.updater.exclude);
        }

        toml
    }

    /// Loads and migrates a legacy configuration from the specified path.
    pub fn migrate<P: AsRef<Path>>(path: P) -> RrduTomlConfig {
        let legacy = MigrateYamlToTomlConfig::load_legacy_config(path);
        MigrateYamlToTomlConfig::migrate_config(legacy)
    }

    /// Migrates the local legacy `.rrduconfig` file, writes `rrdu.toml`, and removes the old config.
    pub fn save_to_new_toml() {
        let config_path = match create_absolute_path(Path::new("./"), Some(RRDUCONFIG_FILE_NAME)) {
            Ok(f) => f,
            Err(_) => return,
        };
        if !config_path.exists() {
            return;
        }
        let toml_path = match create_absolute_path(Path::new("./"), Some(RRDU_TOML_FILE_NAME)) {
            Ok(f) => f,
            Err(_) => return,
        };
        let legacy = MigrateYamlToTomlConfig::load_legacy_config(&config_path);
        let migrated = MigrateYamlToTomlConfig::migrate_config(legacy);
        if let Ok(content) = migrated.to_toml_string()
            && std::fs::write(&toml_path, content).is_ok()
        {
            let _ = remove_file(config_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_config_to_compact_toml() {
        let legacy_yaml = r#"
workspace:
  - project: my-crate
    path: ./
    exclude:
      project:
        - serde
      section:
        dev-dependencies:
          - tokio
  - project: sub-crate
    path: crates/sub
    sub-config: true
updater:
  exclude:
    - vendor
  max-lines: 10
  version: 0.1.0
"#;
        let legacy: RrduConfig =
            noyalib::from_str_with_config(legacy_yaml, &noyalib::ParserConfig::default()).unwrap();
        let migrated = MigrateYamlToTomlConfig::migrate_config(legacy);
        let toml_str = migrated.to_toml_string().unwrap();

        assert!(toml_str.starts_with("#:schema https://annodomine.github.io/rust-recursive-deps-updater/schemas/rrdu.schema.json"));
        assert!(toml_str.contains("[workspaces]\n"));
        assert!(toml_str.contains("my-crate = {"));
        assert!(toml_str.contains("[configs]\nsub-crate = \"crates/sub\""));
        assert!(toml_str.contains("max-lines = 10"));
        assert!(!toml_str.contains("[workspaces.my-crate]"));
        assert!(!toml_str.contains("[workspaces.my-crate.exclude]"));
    }
}
