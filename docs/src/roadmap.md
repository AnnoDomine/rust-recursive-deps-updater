# Pre-Release Roadmap

This document outlines the milestones, current progress, and upcoming engineering phases leading to the official `v0.1.0` release of `rust-recursive-deps-updater` (`rrdu`).

---

## Milestone Progress

### Phase 0: Setup, Governance, CI/CD & Marketplace Infrastructure
- [x] GNU General Public License v3.0 setup (`LICENSE`).
- [x] Governance documentation (`CODE_OF_CONDUCT.md`, `SECURITY.md`, `CONTRIBUTING.md`).
- [x] AI agent directives (`AGENTS.md`) and anti-vibe-coding standards.
- [x] CI workflows with cross-platform matrix testing (Linux GNU/musl, Windows) and PR title linting.
- [x] Immutable SHA pinning for all GitHub Actions.
- [x] Composite Action implementation (`action.yml`).

### Phase 1: Configuration Layer (`.rrduconfig`) & `init` Command
- [x] Configuration data models (`RrduConfig`, `ProjectConfig`, `UpdaterConfig`).
- [x] YAML parsing via pure-safe `noyalib`.
- [x] Path canonicalization and strict traversal validation (rejecting `../` and `/`).
- [x] Interactive template generator (`rrdu --init`) with overwrite protection.

### Phase 2: Workspace & Sub-Crate Discovery Engine (`workspace`)
- [x] Manifest traversal and dependency mapping.
- [x] Scoped `.gitignore` and `.rrduignore` filtering via standard library traversal.
- [x] Fine-grained exclusions (`workspace.exclude` and `updater.exclude`).
- [x] Filtration of `git` and `path` dependencies (strictly checking `crates.io`).

### Phase 3: crates.io Registry Client, Security & SemVer Engine (`registry`)
- [x] Sub-config boundaries for nested `.rrduconfig` manifests (`sub-config: true`).
- [x] Hierarchical module status codes (`1YXX` - `8YXX`) and structured logging.
- [x] Persistent tri-cache (`~/.rrdu/cache/`) with TTL validation (Index, Web API, RustSec).
- [x] Two-tier querying: fast sparse index queries for CI and throttled Web API queries (1 req/s) for interactive mode.
- [x] Cargo SemVer comparison engine with pre-1.0 and post-1.0 breaking change detection.

### Phase 4: Interactive CLI, Table Formatter & Headless Modes (`cli`)
- [x] Split architecture into `rrdu` core library and thin CLI binary.
- [x] Integration of `comfy-table-inline` (custom `comfy-table` edition with inline table support and `force_no_tty` safety).
- [x] Asynchronous trace file logging (`audit/rrdu.log`) with Tokio MPSC queuing, 1 MB splitting, and timestamped archive rotation (`rrdu_yyyy_mm_dd_hh_mm_ss_ms.log`).
- [x] Filter out yanked versions from registry responses.
- [x] Headless CI check (`rrdu --headless`) with hierarchical audit tables.
- [x] Interactive pagination, spinners, and keyboard navigation (arrow keys, Backspace, Esc).
- [x] Privacy-sanitized diagnostics report (`rrdu --report`).
- [x] CLI documentation and argument help display (`rrdu --help`).
- [x] Integration of the first benchmarks.

### Phase 5: Comment-Preserving Manifest Updater (`updater`)
- [x] In-place AST modifications via `toml_edit`.
- [x] Full preservation of comments, whitespace, indentation, and inline tables.
- [x] Workspace-level dependency updates (`[workspace.dependencies]`).
- [x] Re-enable interactive dependency selection (`SpecialInputs::Select` in `Views::Dependecy`) and wire key `u`.
- [x] Add accessibility statement (`ACCESSIBILITY.md`).

### Phase 6: End-to-End Verification, Fixtures & Release v0.1.0
- [x] Integration test suite in `tests/` using `tempfile`.
- [x] End-to-end headless CI verification across Linux and Windows.
- [x] Criterion benchmark suite in `benches/`.
- [x] Clean security audit (`cargo audit` / `--trace`).
- [x] Official `v0.1.0` release on GitHub and `crates.io`.

### Phase 7: Post-v0.1.0 Enhancements
- [ ] Transition crates.io publishing to Trusted Publishing (OIDC).
- [ ] Publish composite action to GitHub Marketplace.
- [x] Implement binary self-update (`rrdu self-update` / `rrdu --self-update`) against GitHub Releases API with persistent user-space caching.
- [x] Migrate configuration format from legacy `.rrduconfig` to modern `rrdu.toml` (`rrdu --migrate`) with JSON schema validation.

### Phase 8: Major Milestone v0.2.0 (Cargo Crate Core Refactor & Dioxus Desktop UI)
- [ ] **Core Cargo Crate Refactoring:**
  - [ ] Embed the official `cargo` binary/crate as a direct library dependency for workspace discovery and package dependency graph traversal.
  - [ ] Replace custom manifest walking with native Cargo workspace metadata resolution for maximum performance and native toolchain fidelity.
- [ ] **Interactive Dependency Management:**
  - [ ] Support adding new dependencies to workspace crates interactively (`Add/delete deps in workspace in interactive mode`).
  - [ ] Support removing existing dependencies across workspace crates interactively.
- [ ] **Desktop GUI via Dioxus (`--ui`):**
  - [ ] Implement an optional native Desktop GUI flag (`rrdu --ui`) built with Dioxus Desktop.
  - [ ] Rich graphical workspace tree visualization, interactive dependency inspector, and batch update trigger.
  - [ ] Advanced UX/UI design focusing on digital accessibility (keyboard accessibility, OS screen reader integration, dynamic text scaling, and high-contrast color palettes).
