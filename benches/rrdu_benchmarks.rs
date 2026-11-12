//! Micro-benchmark suite for `rrdu` core operations and data structures.
//!
//! Evaluates the algorithmic performance and scalability of:
//! - SemVer classification and row generation (`check_if_latest`, `get_dependency_row_valus`)
//! - Interactive terminal sliding-window pagination and row styling (`prepare_paginated_input_line`, `get_selection_line`)
//! - Configuration serialization and deserialization via `noyalib` (`RrduConfig`)
//! - Dependency tree aggregation and metrics filtering (`ProjectDependency`)
//!
//! Benchmark workloads are parameterized across sample sizes of:
//! 10, 20, 50, 100, 500, and 1000 entries.

#![forbid(unsafe_code)]
#![allow(deprecated)]

use std::{collections::BTreeMap, hint::black_box, time::Duration};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use noyalib::{ParserConfig, SerializerConfig};
use rrdu::{
    config::{
        model::*,
        rrdu_config::RrduConfig,
        rrdu_toml::{RrduTomlConfig, TomlWorkspace},
    },
    enums::*,
    interactivity::terminal::Terminal,
    updater::engine::UpdaterInformation,
};

/// Evaluation sample sizes requested for performance profiling.
const SIZES: &[usize] = &[10, 20, 50, 100, 500, 1000];

/// Generates synthetic dependency fixtures with realistic SemVer bump patterns.
///
/// Cycles through major bumps, minor updates, patch updates, pre-1.0 increments,
/// and equal (up-to-date) versions.
fn generate_semver_fixtures(size: usize) -> Vec<(DependencyEntry, String)> {
    let patterns = [
        ("serde", "1.0.100", "2.0.0"),
        ("tokio", "1.2.0", "1.3.0"),
        ("rand", "0.8.4", "0.8.5"),
        ("log", "0.4.17", "0.4.18"),
        ("hyper", "0.14.20", "1.0.0"),
        ("syn", "2.0.1", "2.0.1"),
        ("zerocopy", "0.7.0", "0.8.0"),
        ("bitflags", "2.4.0", "2.4.1"),
        ("anyhow", "1.0.70", "1.0.75"),
        ("clap", "4.4.0", "4.5.0"),
    ];

    (0..size)
        .map(|i| {
            let (name, current, latest) = patterns[i % patterns.len()];
            let dep_name = format!("{}_{}", name, i);
            let entry = DependencyEntry::Simple(SimpleDependency {
                toml_key: dep_name,
                version: DependencyVersion::Supported(current.to_string()),
            });
            (entry, latest.to_string())
        })
        .collect()
}

/// Benchmarks SemVer difference comparison and row values calculation.
///
/// Evaluates [`DependencyEntry::check_if_latest`] and [`DependencyEntry::get_dependency_row_valus`]
/// across parametric sizes of 10, 20, 50, 100, 500, and 1000 dependency entries.
fn bench_semver_classification(c: &mut Criterion) {
    let mut group = c.benchmark_group("semver_classification");
    group.warm_up_time(Duration::from_millis(400));
    group.measurement_time(Duration::from_millis(800));
    group.sample_size(30);

    for &size in SIZES {
        let fixtures = generate_semver_fixtures(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &fixtures, |b, fix| {
            b.iter(|| {
                for (entry, latest) in fix {
                    let (is_latest, needs_migration) = entry.check_if_latest(black_box(latest));
                    let row = entry.get_dependency_row_valus(Some(latest.clone()));
                    black_box((is_latest, needs_migration, row));
                }
            });
        });
    }
    group.finish();
}

/// Generates indexed terminal display items for pagination testing.
fn generate_terminal_items(size: usize) -> Vec<(usize, String, Option<UpdaterInformation>)> {
    (0..size)
        .map(|i| {
            (
                i + 1,
                format!("dependency_crate_{:04} v1.{}.0", i, i % 10),
                None,
            )
        })
        .collect()
}

/// Benchmarks terminal pagination state resolution and row formatting.
///
/// Measures computation of visible window boundaries in [`Terminal::prepare_paginated_input_line`]
/// and row styling in [`Terminal::get_selection_line`].
fn bench_terminal_pagination_and_lines(c: &mut Criterion) {
    let mut group = c.benchmark_group("terminal_pagination_and_lines");
    group.warm_up_time(Duration::from_millis(400));
    group.measurement_time(Duration::from_millis(800));
    group.sample_size(30);

    for &size in SIZES {
        let items = generate_terminal_items(size);
        let mut terminal = Terminal::new(10);
        group.bench_with_input(BenchmarkId::from_parameter(size), &items, |b, itms| {
            b.iter(|| {
                let current_pos = size / 2;
                let first = current_pos.saturating_sub(5);
                let last = (first + 10).min(size);
                terminal.prepare_paginated_input_line(first, last, current_pos, size, false);
                for item in &itms[first..last] {
                    let line = terminal.get_selection_line(black_box(item), current_pos);
                    black_box(line);
                }
            });
        });
    }
    group.finish();
}

/// Generates an in-memory [`RrduConfig`] with $N$ workspace crate definitions.
fn generate_config(size: usize) -> RrduConfig {
    let workspace = (0..size)
        .map(|i| {
            ProjectConfig::new(
                format!("crate_{:04}", i),
                format!("crates/crate_{:04}/Cargo.toml", i),
            )
        })
        .collect();
    RrduConfig {
        workspace,
        updater: UpdaterConfig::default(),
    }
}

/// Benchmarks YAML configuration serialization and deserialization via `noyalib`.
///
/// Measures encoding and decoding of [`RrduConfig`] across 10 to 1000 project entries.
fn bench_config_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_roundtrip");
    group.warm_up_time(Duration::from_millis(400));
    group.measurement_time(Duration::from_millis(1000));
    group.sample_size(30);

    let serializer_config = SerializerConfig::new().quote_all(false);
    let deserializer_config = ParserConfig::new();

    for &size in SIZES {
        let config = generate_config(size);
        let yaml_str = noyalib::to_string_with_config(&config, &serializer_config)
            .expect("serialization should succeed");

        group.bench_with_input(BenchmarkId::new("serialize", size), &config, |b, cfg| {
            b.iter(|| {
                let out = noyalib::to_string_with_config(black_box(cfg), &serializer_config);
                black_box(out)
            });
        });

        group.bench_with_input(
            BenchmarkId::new("deserialize", size),
            &yaml_str,
            |b, yml| {
                b.iter(|| {
                    let parsed: Result<RrduConfig, _> =
                        noyalib::from_str_with_config(black_box(yml), &deserializer_config);
                    black_box(parsed)
                });
            },
        );
    }
    group.finish();
}

/// Generates an in-memory [`RrduTomlConfig`] with $N$ workspace definitions and exclusion patterns.
fn generate_toml_config(size: usize) -> RrduTomlConfig {
    let mut config = RrduTomlConfig::default();
    config.rrdu.max_lines = 10;
    config.rrdu.version = String::from("0.1.1");

    for i in 0..size {
        let name = format!("crate_{:04}", i);
        let path = format!("crates/crate_{:04}", i);

        if i % 10 == 0 {
            config.configs.insert(name, path);
        } else {
            let mut ws = TomlWorkspace::new(&path);
            ws.set_label(&name);
            if i % 3 == 0 {
                ws.add_exclude_deps_in_project("serde");
            }
            if i % 5 == 0 {
                ws.add_exclude_deps_in_sec("dev-dependencies", Some("tokio"));
            }
            config.workspaces.insert(name, ws);
        }
    }

    config
}

/// Benchmarks TOML configuration serialization (with compact inline tables) and deserialization via `toml_edit`.
///
/// Measures encoding via [`RrduTomlConfig::to_toml_string`] and decoding via [`toml_edit::de::from_str`]
/// across 10 to 1000 project entries.
fn bench_toml_config_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("toml_config_roundtrip");
    group.warm_up_time(Duration::from_millis(400));
    group.measurement_time(Duration::from_millis(1000));
    group.sample_size(30);

    for &size in SIZES {
        let config = generate_toml_config(size);
        let toml_str = config
            .to_toml_string()
            .expect("serialization should succeed");

        group.bench_with_input(
            BenchmarkId::new("serialize_compact", size),
            &config,
            |b, cfg| {
                b.iter(|| {
                    let out = black_box(cfg).to_toml_string();
                    black_box(out)
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("deserialize", size), &toml_str, |b, s| {
            b.iter(|| {
                let parsed: Result<RrduTomlConfig, _> = toml_edit::de::from_str(black_box(s));
                black_box(parsed)
            });
        });
    }
    group.finish();
}

/// Generates a [`ProjectDependency`] hierarchy partitioned across 3 sections.
fn generate_project_dependency(size: usize) -> ProjectDependency {
    let mut normal_map = BTreeMap::new();
    let mut dev_map = BTreeMap::new();
    let mut build_map = BTreeMap::new();

    for i in 0..size {
        let name = format!("dep_{:04}", i);
        let entry = DependencyEntry::Simple(SimpleDependency {
            toml_key: name.clone(),
            version: DependencyVersion::Supported(format!("1.{}.0", i % 10)),
        });
        match i % 3 {
            0 => {
                normal_map.insert(name, entry);
            }
            1 => {
                dev_map.insert(name, entry);
            }
            _ => {
                build_map.insert(name, entry);
            }
        }
    }

    let mut map = BTreeMap::new();
    map.insert(
        DependencySection::Normal,
        DependencySectionMap::Map(normal_map),
    );
    map.insert(DependencySection::Dev, DependencySectionMap::Map(dev_map));
    map.insert(
        DependencySection::Build,
        DependencySectionMap::Map(build_map),
    );
    ProjectDependency::Map(map)
}

/// Benchmarks section traversal, dependency filtering, and count aggregations.
///
/// Evaluates [`ProjectDependency::total_values`], [`ProjectDependency::count_valid_sections`],
/// and [`ProjectDependency::count_supported_total_values`] across 10 to 1000 items.
fn bench_dependency_tree_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_tree_metrics");
    group.warm_up_time(Duration::from_millis(400));
    group.measurement_time(Duration::from_millis(800));
    group.sample_size(30);

    for &size in SIZES {
        let proj_dep = generate_project_dependency(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &proj_dep, |b, dep| {
            b.iter(|| {
                let total = dep.total_values();
                let valid = dep.count_valid_sections();
                let supported = dep.count_supported_total_values();
                black_box((total, valid, supported))
            });
        });
    }
    group.finish();
}

/// Benchmarks self-updater version comparison and cache serialization.
fn bench_self_updater_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("self_updater_operations");
    group.warm_up_time(Duration::from_millis(300));
    group.measurement_time(Duration::from_millis(600));
    group.sample_size(30);

    let versions = [
        ("0.1.0", "0.1.1"),
        ("1.0.0", "1.0.0"),
        ("1.2.3", "2.0.0"),
        ("0.2.1", "0.2.0"),
        ("0.1.0+build1", "0.1.0+build2"),
    ];

    group.bench_function("version_comparison_batch", |b| {
        b.iter(|| {
            for (curr, lat) in &versions {
                black_box(
                    rrdu::self_updater::engine::SelfUpdateEngine::is_version_outdated(curr, lat),
                );
            }
        });
    });

    let engine = rrdu::self_updater::engine::SelfUpdateEngine {
        crates_io: true,
        git_hub: true,
        releases: rrdu::self_updater::engine::ReleaseNotes {
            gh: Some(rrdu::self_updater::engine::RrduReleaseStatus::UpToDate),
            ci: None,
        },
    };

    group.bench_function("cache_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&engine).unwrap();
            let deserialized: rrdu::self_updater::engine::SelfUpdateEngine =
                serde_json::from_str(&serialized).unwrap();
            black_box(deserialized);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_semver_classification,
    bench_terminal_pagination_and_lines,
    bench_config_roundtrip,
    bench_toml_config_roundtrip,
    bench_dependency_tree_metrics,
    bench_self_updater_operations,
);
criterion_main!(benches);
