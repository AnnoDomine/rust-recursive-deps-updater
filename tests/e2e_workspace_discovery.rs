use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::Builder;

use rrdu::{
    enums::ProjectDependency,
    workspace::{discovery::Discovery, project::Workspace},
};

#[tokio::test]
async fn test_e2e_workspace_discovery_with_ignores() {
    // 1. Create a temporary workspace root inside current directory
    let temp = Builder::new()
        .prefix("test_e2e_disc_")
        .tempdir_in(".")
        .expect("create temporary directory");
    let root_path = temp.path();

    // 2. Create workspace structure
    let core_dir = root_path.join("crates").join("core");
    let utils_dir = root_path.join("crates").join("utils");
    let ignored_dir = root_path.join("crates").join("ignored_crate");
    fs::create_dir_all(&core_dir).expect("create core dir");
    fs::create_dir_all(&utils_dir).expect("create utils dir");
    fs::create_dir_all(&ignored_dir).expect("create ignored dir");

    // 2.1 Root Cargo.toml
    fs::write(
        root_path.join("Cargo.toml"),
        r#"[workspace]
members = [
    "crates/core",
    "crates/utils",
]
"#,
    )
    .expect("write root Cargo.toml");

    // 2.2 crates/core/Cargo.toml
    fs::write(
        core_dir.join("Cargo.toml"),
        r#"[package]
name = "crate-core"
version = "0.1.0"
edition = "2024"

[dependencies]
tokio = "1.35.0"
"#,
    )
    .expect("write core Cargo.toml");

    // 2.3 crates/utils/Cargo.toml
    fs::write(
        utils_dir.join("Cargo.toml"),
        r#"[package]
name = "crate-utils"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = "1.0.190"
"#,
    )
    .expect("write utils Cargo.toml");

    // 2.4 crates/ignored_crate/Cargo.toml
    fs::write(
        ignored_dir.join("Cargo.toml"),
        r#"[package]
name = "crate-ignored"
version = "0.1.0"
edition = "2024"

[dependencies]
regex = "1.10.0"
"#,
    )
    .expect("write ignored Cargo.toml");

    // 2.5 Add .rrduignore at workspace root ignoring ignored_crate
    fs::write(root_path.join(".rrduignore"), "crates/ignored_crate\n").expect("write .rrduignore");

    // 3. Test Discovery directly on root
    let discovery = Discovery::new(root_path.to_path_buf(), Vec::new())
        .expect("discovery scan on root must succeed");

    assert!(discovery.has_toml, "Root should contain Cargo.toml");
    assert!(
        discovery.found_project.is_some(),
        "Root workspace Cargo.toml should be detected"
    );

    // Verify ignore list collected from .rrduignore using PathBuf
    #[cfg(windows)]
    {
        let expected_ignored = PathBuf::from("crates\\ignored_crate");
        assert!(
            discovery
                .ignore
                .iter()
                .any(|p| p.ends_with(&expected_ignored)),
            "crates\\ignored_crate must be registered in discovery ignore list on Windows"
        );
    }

    #[cfg(not(windows))]
    {
        let expected_ignored = PathBuf::from("crates/ignored_crate");
        assert!(
            discovery
                .ignore
                .iter()
                .any(|p| p.ends_with(&expected_ignored)),
            "crates/ignored_crate must be registered in discovery ignore list on Unix"
        );
    }
}

#[tokio::test]
async fn test_e2e_workspace_sub_config_and_exclusions() {
    let temp = Builder::new()
        .prefix("test_e2e_sub_")
        .tempdir_in(".")
        .expect("create temporary directory");
    let rel_root = temp.path().file_name().unwrap().to_str().unwrap();

    // Setup:
    // root/
    //   .rrduconfig
    //   crates/
    //     active/
    //       Cargo.toml
    //     excluded/
    //       Cargo.toml
    //     sub_workspace/
    //       .rrduconfig
    //       sub_crate/
    //         Cargo.toml
    let active_dir = temp.path().join("crates").join("active");
    let excluded_dir = temp.path().join("crates").join("excluded");
    let sub_ws_dir = temp.path().join("crates").join("sub_workspace");
    let sub_crate_dir = sub_ws_dir.join("sub_crate");

    fs::create_dir_all(&active_dir).expect("create active dir");
    fs::create_dir_all(&excluded_dir).expect("create excluded dir");
    fs::create_dir_all(&sub_crate_dir).expect("create sub_crate dir");

    // Write active Cargo.toml
    fs::write(
        active_dir.join("Cargo.toml"),
        r#"[package]
name = "active-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = "1.0.190"
"#,
    )
    .expect("write active Cargo.toml");

    // Write excluded Cargo.toml
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

    // Write sub_crate Cargo.toml
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

    // Write sub_workspace/.rrduconfig
    let sub_config_yaml = r#"workspace:
  - project: sub-crate
    path: sub_crate
updater:
  version: "0.1.0"
"#;
    fs::write(sub_ws_dir.join(".rrduconfig"), sub_config_yaml).expect("write sub .rrduconfig");

    // Write root/.rrduconfig with relative project paths
    let root_config_yaml = r#"workspace:
  - project: active-crate
    path: crates/active
  - project: excluded-crate
    path: crates/excluded
  - project: sub-ws
    path: crates/sub_workspace
    sub-config: true
updater:
  version: "0.1.0"
  exclude:
    - excluded-crate
"#;
    let root_config_path = temp.path().join(".rrduconfig");
    fs::write(&root_config_path, root_config_yaml).expect("write root .rrduconfig");

    // Run Workspace::read_rrdu_config with relative path from current directory
    let mut workspace = Workspace::new();
    workspace
        .read_rrdu_config(Some(PathBuf::from(format!("{}/.rrduconfig", rel_root))))
        .expect("read_rrdu_config must succeed");

    // Assert projects count: active-crate, excluded-crate, sub-crate (delegated from sub-ws)
    assert_eq!(
        workspace.projects.len(),
        3,
        "Workspace should contain 3 projects (including delegated sub-config crate)"
    );

    // 1. active-crate must have parsed dependencies
    let active_proj = workspace
        .projects
        .iter()
        .find(|p| p.name == "active-crate")
        .expect("active-crate must be in workspace");
    assert!(
        matches!(active_proj.deps, ProjectDependency::Map(_)),
        "active-crate should have parsed dependency map"
    );

    // 2. excluded-crate must be ExcludedProject
    let excluded_proj = workspace
        .projects
        .iter()
        .find(|p| p.name == "excluded-crate")
        .expect("excluded-crate must be in workspace");
    assert_eq!(
        excluded_proj.deps,
        ProjectDependency::ExcludedProject,
        "excluded-crate must be marked as ExcludedProject"
    );

    // 3. sub-crate must have aligned path and parsed dependencies
    let sub_proj = workspace
        .projects
        .iter()
        .find(|p| p.name == "sub-crate")
        .expect("sub-crate must be discovered via sub-config delegation");
    assert!(
        matches!(sub_proj.deps, ProjectDependency::Map(_)),
        "sub-crate should have parsed dependency map"
    );
    #[cfg(windows)]
    {
        let expected_path = Path::new(rel_root)
            .join("crates")
            .join("sub_workspace")
            .join("sub_crate");
        assert_eq!(
            sub_proj.path, expected_path,
            "sub-crate path on Windows must match expected PathBuf"
        );
    }

    #[cfg(not(windows))]
    {
        let expected_path = Path::new(rel_root)
            .join("crates")
            .join("sub_workspace")
            .join("sub_crate");
        assert_eq!(
            sub_proj.path, expected_path,
            "sub-crate path on Unix must match expected PathBuf"
        );
    }

    // 4. Verify collected dependencies:
    // Should contain `serde` (from active-crate) and `clap` (from sub-crate),
    // but NOT `tokio` (from excluded-crate)
    assert!(
        workspace.collected_deps.contains_key("serde"),
        "serde must be collected from active-crate"
    );
    assert!(
        workspace.collected_deps.contains_key("clap"),
        "clap must be collected from delegated sub-crate"
    );
    assert!(
        !workspace.collected_deps.contains_key("tokio"),
        "tokio must not be collected from excluded-crate"
    );
}
