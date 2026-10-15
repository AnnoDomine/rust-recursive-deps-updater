# Changelog

All notable changes to `rust-recursive-deps-updater` (`rrdu`) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and [Conventional Commits](https://www.conventionalcommits.org/).

## [v0.0.6] - 2026-10-15

### Added
- **Comment-Preserving Manifest Updates:** In-place `Cargo.toml` dependency updating engine (`src/updater/engine.rs`) powered by `toml_edit`, supporting standard version strings, inline table dependencies (`{ version = "...", features = [...] }`), and workspace-level dependencies (`[workspace.dependencies]`) with 100% preservation of formatting, indentation, and comments.
- **Interactive Update Trigger:** Wired keyboard input `u` in terminal views to interactively update selected dependency requirements with automatic cascading history updates.
- **Accessibility Statement (`ACCESSIBILITY.md`):** Comprehensive digital accessibility documentation covering keyboard navigation standards, screen reader compatibility via `--headless`, cognitive predictability, and feedback reporting.
- **Dynamic mdBook Versioning:** Integrated `mdbook-variables` preprocessor supporting dynamic `{{RRDU_VERSION}}` placeholder substitution from the build environment.

### Changed
- **Composite Action Parameter Cleanup:** Removed the legacy `config` input from `action.yml`, standardizing on directory-local `.rrduconfig` manifests or in-memory auto-discovery.

### Security
- **Remediated Transitive `rustls` Advisories:** Upgraded `rustls` to `0.23.46` in `Cargo.lock` addressing RUSTSEC-2026-0346, RUSTSEC-2026-0347, RUSTSEC-2026-0348, and RUSTSEC-2026-0349.

## [v0.0.5] - 2026-10-11

### Added
- **Interactive TUI & Sliding-Window Pagination:** Terminal interface with directional pagination (`←`/`→`), item navigation (`↑`/`↓`), line filler padding calculation, and keyboard navigation controls (`Enter`, `Backspace`, `u`, `Esc`).
- **Headless Audit Mode (`--headless`):** Pure-safe non-interactive CI execution displaying formatted dependency audit tables with exit code `0` (clean) or `1` (updates pending/error).
- **Diagnostics Reporting (`--report`):** Hardware and OS diagnostic markdown report using `sysinfo` with privacy sanitization of user home directory tokens.
- **CLI Reference Help (`--help` / `-h`):** Detailed CLI argument help texts powered by Clap derive.
- **Criterion Benchmark Suite:** Comprehensive performance benchmarks (`benches/rrdu_benchmarks.rs`) across sample sizes of 10, 20, 50, 100, 500, and 1000 items.
- **CI Documentation & Benchmark Pipeline:** Automated GitHub Actions workflow (`pages.yml`) building mdBook and embedding interactive Criterion benchmark reports on GitHub Pages.
- **Non-TTY Stream Protection:** Proactively detects non-interactive streams in interactive mode and exits with a guidance message pointing to `--headless`.

### Changed
- **Legacy Configuration Cleanup:** Removed `updater.auto-update` and `updater.auto-scan` from config models with backwards-compatible, non-breaking deserialization.
- **Default Viewport Size:** Reduced default `updater.max-lines` from 50 to 10 entries for responsive terminal rendering.

## [v0.0.4] - 2026-09-30

### Added
- **Two-Tier Registry Client:** High-performance crates.io integration querying the sparse index (`https://index.crates.io/`) for crate availability and web API (`/api/v1/crates/<crate>`) for detailed metadata.
- **Security Vulnerability Auditing:** Direct integration with RustSec OSV advisory database (`https://rustsec.org/packages/<crate>.json`) to identify security advisories and vulnerable dependency versions.
- **Persistent Tri-Cache Engine:** Generic disk-backed caching layer (`CacheRegistry<T>`) stored in `~/.rrdu/cache/` with individual TTL policies (24 hours for index/API metadata, 6 hours for RustSec advisories).
- **Rate Limiting & Compliant HTTP:** Outgoing crates.io request throttling with a 1-second cooldown (`TIME_BETWEEN_REQUESTS`) and compliant `User-Agent` headers.
- **Standardized Structured Logging:** Structured logging architecture with standardized 4-digit status codes (`1YXX` - `6YXX`) and typed status macros (`simple_status!`, `meta_status!`) backed by the `log` facade and `simple_logger`.
- **Rustdoc Documentation Enforcement:** 100% Rustdoc documentation coverage across all public APIs, strictly enforced via `.githooks/pre-commit` and GitHub Actions `docs` CI workflow.
- **Real-World Test Fixtures:** Comprehensive offline test suite featuring captured index, API, and RustSec responses across 14 diverse crates.

### Changed
- **Zero Raw Console Output:** Replaced all raw `println!` and `eprintln!` calls across production code paths with structured status logging macros to prevent terminal output pollution and CI reporting corruption.
- **Configuration Schema Update:** Updated `updater.auto-update` options in `.rrduconfig` to support `none`, `semver-safe`, and `full`.
- **Crate Metadata Links:** Added explicit `homepage` (releases) and `documentation` links to `Cargo.toml`.

### Fixed
- **Nested Sub-Config Discovery Recursion:** Prevented recursive traversal from entering subdirectories containing their own `.rrduconfig`, respecting sub-project boundaries (fixes #5).
- **Sparse Index 3-Letter Path Resolution:** Corrected sparse index directory prefix resolution for 3-letter crate names (`3/<first-char>/<crate>`).
- **Defensive Deserialization:** Made nullable and missing API and RustSec response fields optional (`Option<T>`) to prevent deserialization failures on edge cases.

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