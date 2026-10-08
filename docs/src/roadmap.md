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
- [x] Hierarchical module status codes (`1YXX` - `7YXX`) and structured logging.
- [x] Persistent tri-cache (`~/.rrdu/cache/`) with TTL validation (Index, Web API, RustSec).
- [x] Two-tier querying: fast sparse index queries for CI and throttled Web API queries (1 req/s) for interactive mode.
- [x] Cargo SemVer comparison engine with pre-1.0 and post-1.0 breaking change detection.

### Phase 4: Interactive CLI, Table Formatter & Headless Modes (`cli`)
- [x] Split architecture into `rrdu` core library and thin CLI binary.
- [x] Integration of `comfy-table-inline` (custom `comfy-table` edition with inline table support and `force_no_tty` safety).
- [ ] Dual-mode logging (file-based `./rrdu.log` for interactive CLI, stderr for CI).
- [ ] Interactive pagination, spinners, and navigation (`/next`, `/prev`, `/back`, `/exit`).
- [ ] Headless CI check (`rrdu --run=scan`) and unattended upgrades (`rrdu --run=full`).
- [ ] Privacy-sanitized diagnostics report (`rrdu --report`).

### Phase 5: Comment-Preserving Manifest Updater (`updater`)
- [ ] In-place AST modifications via `toml_edit`.
- [ ] Full preservation of comments, whitespace, indentation, and inline tables.
- [ ] Workspace-level dependency updates (`[workspace.dependencies]`).

### Phase 6: End-to-End Verification, Fixtures & Release v0.1.0
- [ ] Integration test suite in `tests/` using `tempfile`.
- [ ] End-to-end headless CI verification across Linux and Windows.
- [ ] Criterion benchmark suite in `benches/`.
- [ ] Clean security audit (`cargo audit`).
- [ ] Official `v0.1.0` release on GitHub and `crates.io`.
