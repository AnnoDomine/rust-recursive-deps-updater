use rrdu::{
    enums::CacheType,
    registry::cache_registry::CacheRegistry,
    self_updater::{
        controller::SelfUpdateController,
        engine::{
            ReleaseNotes, RrduRelease, RrduReleaseAsset, RrduReleaseStatus, SelfUpdateEngine,
        },
    },
};

#[tokio::test]
async fn test_e2e_self_updater_cache_and_controller() {
    let mut cache: CacheRegistry<SelfUpdateEngine> = CacheRegistry::new(CacheType::SelfUpdate);

    let release = RrduRelease {
        name: "v0.1.1".to_string(),
        version: "0.1.1".to_string(),
        date: "2026-11-12T00:00:00Z".to_string(),
        body: Some("Test release body".to_string()),
        release_notes_url: Some(
            "https://github.com/AnnoDomine/rust-recursive-deps-updater/releases/tag/v0.1.1"
                .to_string(),
        ),
        assets: vec![RrduReleaseAsset {
            name: "rrdu-x86_64-unknown-linux-gnu.tar.gz".to_string(),
            download_url: "https://example.com/asset.tar.gz".to_string(),
            digest: Some("testdigest123".to_string()),
        }],
    };

    let engine = SelfUpdateEngine {
        crates_io: true,
        git_hub: true,
        releases: ReleaseNotes {
            gh: Some(RrduReleaseStatus::Updated(release)),
            ci: None,
        },
    };

    // 1. Add entry to cache and persist
    cache.add_cache_entry("self_update", engine.clone());
    assert!(cache.get_cache_entry("self_update").is_some());

    // 2. Validate cache entry retrieved matches persisted
    let cached_entry = cache
        .get_cache_entry("self_update")
        .expect("cache entry should exist");
    assert_eq!(&engine, &cached_entry);
    assert!(cached_entry.git_hub);
    assert!(cached_entry.crates_io);

    // 3. Initialize controller with the cached engine
    let controller = SelfUpdateController {
        engine: Some(cached_entry.clone()),
    };

    assert!(controller.engine.is_some());

    // 4. Verify SemVer outdated logic
    assert!(SelfUpdateEngine::is_version_outdated("0.1.0", "0.1.1"));
    assert!(!SelfUpdateEngine::is_version_outdated("0.1.1", "0.1.0"));
    assert!(!SelfUpdateEngine::is_version_outdated("0.1.0", "0.1.0"));
}
