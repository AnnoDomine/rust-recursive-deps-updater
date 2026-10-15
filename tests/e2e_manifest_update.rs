use std::fs;
use tempfile::Builder;

use rrdu::{
    config::model::ProjectConfig,
    enums::{Boolean, DependencySection, TargetDepKind},
    updater::engine::UpdaterInformation,
    workspace::project::Projects,
};

#[tokio::test]
async fn test_e2e_manifest_update_and_comment_preservation() {
    // 1. Create a safe temporary directory inside current directory
    let temp = Builder::new()
        .prefix("test_e2e_manifest_")
        .tempdir_in(".")
        .expect("create temporary directory");
    let rel_dir = temp.path().file_name().unwrap().to_str().unwrap();

    // 2. Read fixture Cargo.toml and copy to temp directory
    let fixture_path = "tests/fixtures/Cargo.toml";
    let fixture_content = fs::read_to_string(fixture_path).expect("fixture Cargo.toml must exist");
    let target_toml_path = temp.path().join("Cargo.toml");
    fs::write(&target_toml_path, &fixture_content).expect("write target Cargo.toml");

    // 3. Initialize and parse project via rrdu
    let config = ProjectConfig::new("test-project".to_string(), rel_dir.to_string());
    let mut project = Projects::new(config, Boolean::False);
    project.parse_project();

    assert!(
        project.updater.is_some(),
        "UpdateEngine should be initialized on parsed project"
    );

    // 4. Update various dependency types using UpdateEngine
    let updater = project.updater.as_mut().unwrap();

    // 4.1 Update simple string dependency: serde = "1.0.190" -> "1.0.229"
    let mut serde_info = UpdaterInformation {
        section: DependencySection::Normal,
        toml_key: "serde".to_string(),
        prefix: None,
        latest: None,
    };
    serde_info.set_latest("1.0.229");
    updater.update_dependency(&serde_info);

    // 4.2 Update inline table dependency with features: tokio = { version = "1.35.0", features = ["full"] } -> "1.53.3"
    let mut tokio_info = UpdaterInformation {
        section: DependencySection::Normal,
        toml_key: "tokio".to_string(),
        prefix: None,
        latest: None,
    };
    tokio_info.set_latest("1.53.3");
    updater.update_dependency(&tokio_info);

    // 4.3 Update renamed package inline table: my_colored = { package = "colored", version = "2.1.0" } -> "3.1.1"
    let mut colored_info = UpdaterInformation {
        section: DependencySection::Normal,
        toml_key: "my_colored".to_string(),
        prefix: None,
        latest: None,
    };
    colored_info.set_latest("3.1.1");
    updater.update_dependency(&colored_info);

    // 4.4 Update dev-dependency: tempfile = "3.10.0" -> "3.27.0"
    let mut tempfile_info = UpdaterInformation {
        section: DependencySection::Dev,
        toml_key: "tempfile".to_string(),
        prefix: None,
        latest: None,
    };
    tempfile_info.set_latest("3.27.0");
    updater.update_dependency(&tempfile_info);

    // 4.5 Update build-dependency: cc = "1.0.83" -> "1.2.0"
    let mut cc_info = UpdaterInformation {
        section: DependencySection::Build,
        toml_key: "cc".to_string(),
        prefix: None,
        latest: None,
    };
    cc_info.set_latest("1.2.0");
    updater.update_dependency(&cc_info);

    // 4.6 Update target-specific dependency: windows-sys = "0.52.0" -> "0.59.0"
    let mut windows_info = UpdaterInformation {
        section: DependencySection::Target {
            target: "cfg(windows)".to_string(),
            kind: TargetDepKind::Normal,
        },
        toml_key: "windows-sys".to_string(),
        prefix: None,
        latest: None,
    };
    windows_info.set_latest("0.59.0");
    updater.update_dependency(&windows_info);

    // 4.7 Update workspace dependency: serde_json = "1.0.0" -> "1.0.154"
    let mut serde_json_info = UpdaterInformation {
        section: DependencySection::Workspace,
        toml_key: "serde_json".to_string(),
        prefix: None,
        latest: None,
    };
    serde_json_info.set_latest("1.0.154");
    updater.update_dependency(&serde_json_info);

    // 4.8 Update explicit table dependency: [dependencies.clap] version = "4.4.18" -> "4.6.7"
    let mut clap_info = UpdaterInformation {
        section: DependencySection::Table {
            parent: Box::new(DependencySection::Normal),
            toml_key: "clap".to_string(),
        },
        toml_key: "clap".to_string(),
        prefix: None,
        latest: None,
    };
    clap_info.set_latest("4.6.7");
    updater.update_dependency(&clap_info);

    // 5. Save updated manifest atomically to disk
    updater.save().expect("manifest save must succeed");

    // 6. Read back updated manifest from disk
    let updated_content =
        fs::read_to_string(&target_toml_path).expect("read updated Cargo.toml from disk");

    // 7. Verification: Verify all 8 original comments are 100% intact
    assert!(
        updated_content.contains("# 1. Einfache String-Dependency (Simple)"),
        "Comment 1 must be preserved"
    );
    assert!(
        updated_content.contains("# 2. Inline-Table mit Features (Inline)"),
        "Comment 2 must be preserved"
    );
    assert!(
        updated_content.contains("# 3. Umbenanntes Package (Inline mit 'package')"),
        "Comment 3 must be preserved"
    );
    assert!(
        updated_content.contains(
            "# 4. Nicht unterstützte Keys (z.B. Git / Path -> landet in UnsupportedKeys)"
        ),
        "Comment 4 must be preserved"
    );
    assert!(
        updated_content.contains("# 6. Dev-Dependencies"),
        "Comment 6 must be preserved"
    );
    assert!(
        updated_content.contains("# 7. Build-Dependencies"),
        "Comment 7 must be preserved"
    );
    assert!(
        updated_content.contains("# 8. Target-spezifische Dependencies"),
        "Comment 8 must be preserved"
    );
    assert!(
        updated_content.contains("# 5. Explizite Tabellensektion (Table)"),
        "Comment 5 must be preserved"
    );

    // 8. Verification: Verify version updates on disk
    assert!(
        updated_content.contains(r#"serde = "1.0.229""#),
        "serde must be updated to 1.0.229 on disk"
    );
    assert!(
        updated_content.contains(r#"version = "1.53.3""#),
        "tokio version must be updated to 1.53.3 on disk"
    );
    assert!(
        updated_content.contains(r#"features = ["full"]"#),
        "tokio features must be preserved"
    );
    assert!(
        updated_content.contains(r#"version = "3.1.1""#),
        "my_colored version must be updated to 3.1.1 on disk"
    );
    assert!(
        updated_content.contains(r#"package = "colored""#),
        "my_colored package attribute must be preserved"
    );
    assert!(
        updated_content.contains(r#"tempfile = "3.27.0""#),
        "tempfile must be updated to 3.27.0 on disk"
    );
    assert!(
        updated_content.contains(r#"cc = "1.2.0""#),
        "cc must be updated to 1.2.0 on disk"
    );
    assert!(
        updated_content.contains(r#"windows-sys = "0.59.0""#),
        "windows-sys must be updated to 0.59.0 on disk"
    );
    assert!(
        updated_content.contains(r#"serde_json = "1.0.154""#),
        "serde_json must be updated to 1.0.154 on disk"
    );
    assert!(
        updated_content.contains(r#"version = "4.6.7""#),
        "clap version must be updated to 4.6.7 on disk"
    );
    assert!(
        updated_content.contains(r#"features = ["derive"]"#),
        "clap features must be preserved"
    );

    // 9. Verification: Unsupported keys must remain completely untouched
    assert!(
        updated_content.contains(r#"local_dep = { path = "../local_lib" }"#),
        "local_dep path dependency must remain untouched"
    );

    // 10. Re-parse the updated file with Projects to verify complete roundtrip
    let mut reloaded_project = Projects::new(
        ProjectConfig::new("test-project".to_string(), rel_dir.to_string()),
        Boolean::False,
    );
    reloaded_project.parse_project();
    let mut collected = std::collections::HashMap::new();
    reloaded_project.list_all_deps(&mut collected);

    assert!(collected.contains_key("serde"));
    assert!(collected.contains_key("tokio"));
    assert!(collected.contains_key("colored"));
    assert!(collected.contains_key("tempfile"));
    assert!(collected.contains_key("cc"));
    assert!(collected.contains_key("windows-sys"));
    assert!(collected.contains_key("serde_json"));
    // local_dep has a path key and must NOT be collected for crates.io querying
    assert!(
        !collected.contains_key("local_dep"),
        "Path dependencies must not be registered for crates.io queries"
    );

    // Verify local_dep is still present in project.deps and classified as UnsupportedKeys
    if let rrdu::enums::ProjectDependency::Map(sec) = &reloaded_project.deps {
        let normal_deps = sec
            .get(&DependencySection::Normal)
            .expect("normal deps section");
        if let rrdu::enums::DependencySectionMap::Map(map) = normal_deps {
            let local_entry = map.get("local_dep").expect("local_dep entry");
            assert!(
                matches!(
                    local_entry.version(),
                    rrdu::enums::DependencyVersion::UnsupportedKeys { .. }
                ),
                "local_dep must be classified as UnsupportedKeys"
            );
        } else {
            panic!("Expected DependencySectionMap::Map");
        }
    } else {
        panic!("Expected ProjectDependency::Map");
    }
}
