use std::fs;
use tempfile::Builder;

use rrdu::{
    cli::headless::Headless,
    config::model::ProjectConfig,
    enums::{Boolean, DependencyCollectionVersion, DependencySection},
    registry::crates_index_response_structs::CratesIndexItem,
    workspace::project::{Projects, Workspace},
};

#[tokio::test]
async fn test_e2e_offline_fixtures_and_semver_audit() {
    // 1. Copy fixture Cargo.toml to a safe temporary directory
    let temp = Builder::new()
        .prefix("test_e2e_audit_")
        .tempdir_in(".")
        .expect("create temporary directory");
    let rel_dir = temp.path().file_name().unwrap().to_str().unwrap();

    let fixture_manifest =
        fs::read_to_string("tests/fixtures/Cargo.toml").expect("read fixture Cargo.toml");
    let target_toml = temp.path().join("Cargo.toml");
    fs::write(&target_toml, &fixture_manifest).expect("write temp Cargo.toml");

    // 2. Parse project
    let config = ProjectConfig::new("test-project".to_string(), rel_dir.to_string());
    let mut project = Projects::new(config, Boolean::False);
    project.parse_project();

    // 3. Parse offline sparse index fixtures
    let serde_index_raw =
        fs::read_to_string("tests/fixtures/serde_index.txt").expect("read serde index fixture");
    let serde_index = CratesIndexItem::parse_response(&serde_index_raw);
    assert!(!serde_index.is_empty(), "serde index items must be parsed");

    let newest_serde = serde_index
        .iter()
        .filter(|i| !i.yanked && semver::Version::parse(&i.vers).is_ok_and(|v| v.pre.is_empty()))
        .max_by_key(|i| semver::Version::parse(&i.vers).ok())
        .expect("newest stable serde version");

    assert_eq!(
        newest_serde.vers, "1.0.229",
        "Newest serde in index fixture should be 1.0.229"
    );

    // 4. SemVer compatibility evaluation
    if let rrdu::enums::ProjectDependency::Map(sec) = &project.deps {
        let normal_deps = sec.get(&DependencySection::Normal).expect("normal section");
        if let rrdu::enums::DependencySectionMap::Map(deps) = normal_deps {
            let serde_entry = deps.get("serde").expect("serde entry");

            // Current: "1.0.190" vs Latest: "1.0.229"
            let (is_latest, needs_migration) = serde_entry.check_if_latest(&newest_serde.vers);
            assert_eq!(
                is_latest,
                Boolean::False,
                "serde 1.0.190 is not latest when 1.0.229 exists"
            );
            assert_eq!(
                needs_migration,
                Boolean::False,
                "serde 1.0.190 to 1.0.229 is within same major version (no migration needed)"
            );

            // Breaking major change test: e.g. target "2.0.0"
            let (is_latest_major, needs_migration_major) = serde_entry.check_if_latest("2.0.0");
            assert_eq!(is_latest_major, Boolean::False);
            assert_eq!(
                needs_migration_major,
                Boolean::True,
                "Bumping from 1.x to 2.x requires manual migration"
            );
        } else {
            panic!("Expected DependencySectionMap::Map");
        }
    } else {
        panic!("Expected ProjectDependency::Map");
    }

    // 5. Populate workspace with resolved offline fixture versions
    let mut workspace = Workspace::new();
    workspace.projects.push(project);
    workspace.collected_deps.insert(
        "serde".to_string(),
        DependencyCollectionVersion::Latest(newest_serde.vers.clone()),
    );
    workspace.collected_deps.insert(
        "tokio".to_string(),
        DependencyCollectionVersion::Latest("1.53.3".to_string()),
    );
    workspace.collected_deps.insert(
        "colored".to_string(),
        DependencyCollectionVersion::Latest("3.1.1".to_string()),
    );
    workspace.collected_deps.insert(
        "clap".to_string(),
        DependencyCollectionVersion::Latest("4.6.7".to_string()),
    );
    workspace.collected_deps.insert(
        "cc".to_string(),
        DependencyCollectionVersion::Latest("1.2.0".to_string()),
    );
    workspace.collected_deps.insert(
        "tempfile".to_string(),
        DependencyCollectionVersion::Latest("3.27.0".to_string()),
    );

    // 6. Test Headless table rendering in non-exiting mode (is_headless = false avoids process::exit)
    let mut headless = Headless::new(workspace);
    headless.headless_workspace_table(false);

    assert!(
        headless.found_outdated_dependency,
        "Headless audit must flag outdated dependencies (e.g. serde 1.0.190 vs 1.0.229)"
    );
}
