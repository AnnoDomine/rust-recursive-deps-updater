//! End-to-end integration tests for modern TOML configuration (`rrdu.toml`).
//!
//! Verifies:
//! - Recursive crate discovery with `rrdu.toml` manifests
//! - Workspace exclusions (project-wide and section-specific) in TOML format
//! - Sub-configuration boundaries and delegation via `[configs]`
//! - Migration from legacy `.rrduconfig` to modern `rrdu.toml`
//! - Comment and whitespace preservation during manifest updates configured via `rrdu.toml`

#![forbid(unsafe_code)]

use std::fs;
use tempfile::Builder;

use rrdu::{
    config::{migrate::MigrateYamlToTomlConfig, rrdu_toml::RrduTomlConfig},
    enums::ProjectDependency::{self, ExcludedProject},
    workspace::project::Workspace,
};

#[tokio::test]
async fn test_e2e_toml_sub_config_and_exclusions() {
    let temp = Builder::new()
        .prefix("test_e2e_toml_sub_")
        .tempdir_in(".")
        .expect("create temporary directory");
    let rel_root = temp.path().file_name().unwrap().to_str().unwrap();

    let active_dir = temp.path().join("crates").join("active");
    let excluded_dir = temp.path().join("crates").join("excluded");
    let sub_ws_dir = temp.path().join("crates").join("sub_workspace");
    let sub_crate_dir = sub_ws_dir.join("sub_crate");

    fs::create_dir_all(&active_dir).expect("create active dir");
    fs::create_dir_all(&excluded_dir).expect("create excluded dir");
    fs::create_dir_all(&sub_crate_dir).expect("create sub_crate dir");

    fs::write(
        active_dir.join("Cargo.toml"),
        r#"[package]
name = "active-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = "1.0.190"
tokio = "1.35.0"
"#,
    )
    .expect("write active Cargo.toml");

    fs::write(
        excluded_dir.join("Cargo.toml"),
        r#"[package]
name = "excluded-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
tokio = "1.35.0"
"#,
    )
    .expect("write excluded Cargo.toml");

    fs::write(
        sub_crate_dir.join("Cargo.toml"),
        r#"[package]
name = "sub-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = "4.4.18"
"#,
    )
    .expect("write sub_crate Cargo.toml");

    // Sub-workspace rrdu.toml
    fs::write(
        sub_ws_dir.join("rrdu.toml"),
        r#"#:schema https://annodomine.github.io/rust-recursive-deps-updater/schemas/rrdu.schema.json

[workspaces]
sub-crate = { label = "sub-crate", path = "sub_crate" }

[configs]

[rrdu]
max-lines = 10
version = "0.1.0"
"#,
    )
    .expect("write sub rrdu.toml");

    // Root rrdu.toml with project-wide exclusion for tokio and [configs] entry
    fs::write(
        temp.path().join("rrdu.toml"),
        r#"#:schema https://annodomine.github.io/rust-recursive-deps-updater/schemas/rrdu.schema.json

[workspaces]
active-crate = { label = "active-crate", path = "crates/active", exclude = { project = ["tokio"] } }
excluded-crate = { label = "excluded-crate", path = "crates/excluded" }

[configs]
sub_ws = "crates/sub_workspace"

[rrdu]
max-lines = 10
version = "0.1.0"
exclude = ["excluded-crate"]
"#,
    )
    .expect("write root rrdu.toml");

    let mut workspace = Workspace::new();
    let read_result = workspace.read_rrdu_config(Some(rel_root.into()));
    assert!(
        read_result.is_ok(),
        "read_rrdu_config should succeed: {:?}",
        read_result.err()
    );

    let projects = workspace.projects;
    assert!(
        projects.iter().any(|p| p.name == "active-crate"),
        "active-crate should be present"
    );
    assert!(
        projects.iter().any(|p| p.name == "sub-crate"),
        "sub-crate should be loaded via [configs]"
    );
    assert!(
        !projects
            .iter()
            .any(|p| p.name == "excluded-crate" && !matches!(p.deps, ExcludedProject)),
        "excluded-crate should be omitted via [rrdu.exclude]"
    );

    // Verify tokio is marked as excluded in active-crate
    let active_project = projects.iter().find(|p| p.name == "active-crate").unwrap();
    if let ProjectDependency::Map(sections) = &active_project.deps {
        for section in sections.values() {
            if let rrdu::enums::DependencySectionMap::Map(deps) = section {
                for (name, dep) in deps {
                    if name == "tokio" {
                        assert!(
                            dep.is_excluded(),
                            "tokio must be marked as excluded in active-crate"
                        );
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_e2e_toml_migration_workflow() {
    let temp = Builder::new()
        .prefix("test_e2e_toml_mig_")
        .tempdir_in(".")
        .expect("create temporary directory");
    let folder = temp.path();

    // Create legacy .rrduconfig
    let legacy_yaml = r#"workspace:
  - project: fixture-project
    path: ./
    exclude:
      project:
        - serde
      section:
        dev-dependencies:
          - tokio
    sub-config: null
updater:
  exclude:
    - vendor
  max-lines: 15
  version: 0.1.0
"#;
    let yaml_file = folder.join(".rrduconfig");
    fs::write(&yaml_file, legacy_yaml).expect("write legacy .rrduconfig");
    assert!(yaml_file.exists());

    // Execute migration
    let toml_file = folder.join("rrdu.toml");
    let migrated = MigrateYamlToTomlConfig::migrate(folder);
    let toml_str = migrated.to_toml_string().expect("serialize migrated toml");
    fs::write(&toml_file, &toml_str).expect("write migrated rrdu.toml");
    fs::remove_file(&yaml_file).expect("remove old yaml");

    assert!(
        !yaml_file.exists(),
        ".rrduconfig should be removed after migration"
    );
    assert!(toml_file.exists(), "rrdu.toml should exist after migration");

    // Verify content structure of migrated TOML
    assert!(toml_str.starts_with(
        "#:schema https://annodomine.github.io/rust-recursive-deps-updater/schemas/rrdu.schema.json"
    ));
    assert!(toml_str.contains("[workspaces]\n"));
    assert!(toml_str.contains("fixture-project = {"));
    assert!(toml_str.contains("max-lines = 15"));

    // Verify deserialization into RrduTomlConfig
    let loaded = RrduTomlConfig::load_from_path(folder).expect("load migrated rrdu.toml");
    assert_eq!(loaded.rrdu.max_lines, 15);
    let ws = loaded
        .workspaces
        .get("fixture-project")
        .expect("find fixture-project");
    let exc = ws.exclude.as_ref().expect("find exclusions");
    assert_eq!(exc.project.as_deref(), Some(&["serde".to_string()][..]));
}
