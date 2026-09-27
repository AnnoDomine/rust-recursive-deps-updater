# Changelog

All notable changes to `rust-recursive-deps-updater` (`rrdu`) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and [Conventional Commits](https://www.conventionalcommits.org/).

## [v0.0.3] - 2026-09-27

### Added
- **Workspace & Standalone Crate Discovery:** Native recursive filesystem traversal discovering all `Cargo.toml` manifests across cargo workspaces and nested standalone crates.
- **Hierarchical Ignore Engine:** Scoped ignore handling supporting `.gitignore`, `.rrduignore`, hidden directory exclusions (`.*`), and automatic crate build/source folder filtering (`target`, `src`).
- **Dependency AST Representation:** Support for all Cargo dependency definitions (simple strings, inline tables, explicit table sections, workspace dependencies, and target-specific dependencies).
- **SemVer Classification Engine:** Built-in Cargo SemVer comparison categorizing updates into compatible bumps (`[No migration needed]`) and breaking bumps (`[Need manual migration]`).
- **Granular Exclusion Schema:** Support for project-wide and section-specific dependency exclusions in `.rrduconfig`.
- **Sub-Config Support:** Ability to delegate sub-crate configuration via `sub-config: true`.
- **Zero External Traversal Dependencies:** Eliminated `walkdir` in favor of pure standard library `std::fs::read_dir`.
- **Unit Test Coverage:** 23 passing unit tests verifying discovery, path resolution, ignore scoping, and dependency parsing.

### Changed
- Migrated `.rrduconfig` schema: `toml` renamed to `path`, introduced granular `exclude.project` and `exclude.section` mapping, added `updater.version`.

## [v0.0.2] - 2026-09-21

### Added
- Configuration engine with YAML parsing via `noyalib` (`#![forbid(unsafe_code)]`).
- Path traversal defense rejecting parent references (`../`) and root paths (`/`).
- Initial CLI configuration generator (`rrdu --init`).
- Domain-specific typed error hierarchy (`ConfigError`).

## [v0.0.1] - 2026-09-20

### Added
- Project foundation, governance, GitHub Actions CI/CD workflows, and GPL-3.0 licensing.