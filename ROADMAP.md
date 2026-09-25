# Project Roadmap & Technical Architecture for `rrdu`

This document serves as the central engineering specification, architecture manual, and active implementation roadmap for `rust-recursive-deps-updater` (`rrdu`). It tracks progress across development phases and provides the blueprint for design decisions.

---

## 1. Vision & Core Mission

`rust-recursive-deps-updater` (`rrdu`) is a fast, safe, zero-privilege CLI tool and GitHub Action tailored for developers and teams managing complex Rust workspaces and multi-crate repositories.

### Primary Objectives

1. **Transparent Discovery:** Automatically discover all `Cargo.toml` manifests across cargo workspaces and nested standalone crates.
2. **Deterministic Configuration:** Support `.rrduconfig` (YAML) to define project boundaries and fine-grained exclude lists.
3. **Targeted Registry Queries:** Check dependencies exclusively against `crates.io` (git and path dependencies are intentionally ignored and skipped).
4. **Accurate SemVer Classification:** Categorize updates following Rust Cargo SemVer conventions:
   - `latest version` (up to date)
   - `[No migration needed]` (compatible patch or minor bump)
   - `[Need manual migration]` (breaking major bump or pre-1.0 breaking bump)
5. **Formatting Preservation:** Update `Cargo.toml` manifests via `toml_edit`, preserving 100% of indentation, inline tables, comments, and structure.
6. **Dual Execution Modes:**
   - Interactive terminal UI with pagination, keyboard navigation, and selective updates.
   - Headless CI/CD execution (`rrdu --run=scan`, `rrdu --run=full`).

---

## 2. Technical Architecture & Component Specification

### 2.1 Configuration Layer (`.rrduconfig` & GitHub Action)

- **Format:** YAML 1.2, parsed via `noyalib` (pure Rust, `#![forbid(unsafe_code)]`, maintained drop-in replacement for `serde_yaml`).
- **Standard Location:** Workspace root (`./.rrduconfig`).
- **Custom Configuration:** The CLI accepts `--config-path=<path>` (strictly validated against path traversal).
- **GitHub Action Integration:**
  - `action.yml` supports custom config files via `config: <path>`.
  - If omitted, the action defaults to `./.rrduconfig` in the workspace root.
- **Security & Path Defense:**
  - Strict path validation: Paths in `.rrduconfig` must point to the current directory (`./`) or internal subdirectories (e.g., `crates/...`).
  - Parent directory references (`../`) or absolute root paths (`/`) are rejected with `ConfigError::InsecurePath` and exit code `1`. No panics!
- **Specification:**

```yaml
workspace:
  - project: Root
    path: ./
    exclude:
      project: # Excludes all named dependencies project wide
        - tokio
      section: # Excludes all named dependencies from specific sections
        dev-dependencies: # Only exclude named dependencies from [dev-dependencies] section
          - serde
        dependencies.clap: [] # Exclude [dependencies.clap] section in total. No values needed in the list, as the whole section is a single dependency.
  - project: WorkspaceCrate
    path: crates/workspace_crate
    sub-config: true

updater:
  exclude:
    - excluded_project
  auto-update: none # Options: "none", "*", "*-force"
  auto-scan: true # Options: true, false, or list of project names
  max-lines: 50 # Maximum entries per page during interactive pagination
  version: 0.0.3 # Version where the config file was created
```

- `workspace.project`: Human-readable name of the project.
- `workspace.path`: Relative path to the project directory or `Cargo.toml` (e.g., `./`, `crates/...`).
- `workspace.exclude`: Fine-grained exclusion configuration with project-wide (project) and section-specific (section) scopes.
- `workspace.sub-config`: Optional boolean flag indicating whether the sub-project provides its own `.rrduconfig` (default: `false`).
- `updater.exclude`: Optional folder paths ignored during recursive discovery.
- `updater.auto-update`: Automated update strategy (`none`, `*`, `*-force`). Default: `"none"`.
- `updater.auto-scan`: Automatic scan on interactive CLI startup (`true`, `false`, or list of project names). Default: `true`.
- `updater.max-lines`: Maximum entries per page during interactive pagination. Default: `50`.
- `updater.version`: Schema version of the `.rrduconfig` used for compatibility checks and legacy configuration detection.

### 2.2 Workspace & Crate Discovery Engine

- **Root Manifest Inspection:** Evaluates root `Cargo.toml` for `[workspace]` definitions and resolves `[workspace.members]` patterns (including globs like `crates/*`).
- **Recursive Fallback:** If no root workspace is declared, recursively scans the directory tree to discover standalone sub-crates.
- **Workspace Dependencies:** Resolves central `[workspace.dependencies]` and detects inherited dependencies in member crates (`dep = { workspace = true }`).
- **Non-Workspace Projects:** Correctly categorizes and processes nested standalone crates that are not part of the root workspace.

### 2.3 Registry Client & SemVer Classification Engine

- **Target Scope:** Strictly direct `crates.io` dependencies.
- Git dependencies (`git = "..."`) and path dependencies (`path = "..."`) are explicitly skipped and ignored.
- **Registry Communication:**
- Uses `ureq` with `rustls` (synchronous, lightweight, memory-safe, no OpenSSL dependencies).
- Outgoing HTTP queries enforce a 10-second timeout, retry limits, and a compliant `User-Agent: rust-recursive-deps-updater/<version> (<url>)`.
- In-memory caching ensures identical crates across multiple manifests are queried only once per session.
- Yanked releases are filtered out and ignored.
- **SemVer Rules for Migration Necessity:**
- **Version >= 1.0.0:**
  - Patch or Minor bump (e.g., `1.2.0` -> `1.3.1`): `[No migration needed]`
  - Major bump (e.g., `1.8.0` -> `2.0.0`): `[Need manual migration]`
- **Version < 1.0.0 (Pre-1.0 Cargo Convention):**
  - `0.x.y` where `x` changes (e.g., `0.1.2` -> `0.2.0`): Breaking change -> `[Need manual migration]`
  - `0.x.y` where only `y` changes (e.g., `0.5.3` -> `0.5.4`): Compatible -> `[No migration needed]`

### 2.4 CLI Binary & Execution Modes

Binary executable: `rrdu` (Unix) and `rrdu.exe` (Windows).

#### Mode A: Interactive Terminal Interface (`rrdu`)

- **Header & Version Notification:**
- Displays current working directory and configuration status.
- Asynchronously checks GitHub Releases for new `rrdu` versions. If an update is available, displays a non-blocking notification:
  `A new version of rrdu is available (vX.Y.Z -> vA.B.C). Run 'rrdu self-update' to update.`
- **Pagination & Navigation:**
- Projects and crates are paginated based on `updater.max-lines` (default 50).
- Navigation commands: `/next`, `/prev`, arrow keys (Up/Down/Left/Right).
- Global navigation: `/back` (return to previous view), `/exit` or `/quit` (exit process with code 2).
- Contextual Help: Typing `/?`, `/h`, or `/help` displays available commands and keyboard shortcuts.
- **Update Execution:**
- `*`: Updates all compatible dependencies across the selected project (`[No migration needed]`).
- `*-force`: Updates all dependencies including breaking versions (`[Need manual migration]`).
- `<crate>` / `<index>`: Updates an individual crate interactively.

#### Mode B: Headless Scan (`rrdu --run=scan`)

- Designed for CI/CD pipelines.
- Requires `.rrduconfig`. If missing, terminates with exit code `1` and suggests running `rrdu init`.
- Scans all configured projects and renders a clean 5-column table:
  `Project` | `Dependency` | `Current version` | `Latest version` | `Migration necessary` (`true` / `false` / empty)
- Table is rendered natively in `src/cli/table.rs` using `std::fmt` (**Zero external dependencies**).
- **Exit Codes:**
- `0`: All dependencies are up to date.
- `1`: Outdated dependencies found or execution error.

#### Mode C: Headless Full Update (`rrdu --run=full`)

- Executes non-interactive updates according to `updater.auto-update` in `.rrduconfig`.
- Applies updates to manifests and exits with code `0` on success or `1` on error.

#### Mode D: Configuration Generator (`rrdu init`)

- Recursively scans the current directory, detects all `Cargo.toml` manifests, and generates a fully-commented `.rrduconfig` template.
- If `.rrduconfig` already exists, prompts `Override [y/n]` to prevent accidental data loss.

#### Mode E: Binary Self-Update (`rrdu self-update`)

- Queries GitHub Releases API for the latest binary release matching the current OS architecture.
- Replaces the running binary in user space without requiring root or administrator privileges.

#### Mode F: Privacy-Sanitized Diagnostics (`rrdu report`)

- Generates a sanitized diagnostic markdown summary for issue reporting:
- System architecture, OS, Rust edition, `rrdu` version.
- Discovered project names and dependency counts.
- **Privacy Guarantee:** All absolute system paths, usernames, and sensitive home directory tokens are masked or stripped.

#### Mode G: Reference Help (`rrdu help`, `rrdu --help`, `rrdu -h`)

- Prints comprehensive CLI command documentation and flag reference.

### 2.5 Comment-Preserving Manifest Updater (`toml_edit`)

- Utilizes `toml_edit` to perform in-place AST modifications on `Cargo.toml`.
- Preserves 100% of user comments, formatting, indentation, and structure.
- Handles standard string versions (`dep = "1.0"`), inline tables (`dep = { version = "1.0", features = [...] }`), and workspace dependencies (`[workspace.dependencies]`).

---

## 3. Engineering & Security Standards

### 3.1 Security & Zero-Privilege

- **User-Space Operation:** Never requests or requires root or administrator privileges.
- **100% Safe Rust:** `#![forbid(unsafe_code)]` enforced at crate root (`src/main.rs`). Zero unsafe code.
- **Path Traversal Defense:** All input paths are canonicalized and checked against project root boundaries.
- **Static Configuration:** No arbitrary CLI execution flags to prevent injection attacks.
- **Network Hardening:** Strict HTTPS, TLS 1.2/1.3, 10s timeouts, verified crates.io crawler User-Agent.

### 3.2 Code Quality & Error Handling

- **Toolchain:** Rust Edition 2024, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`.
- **Error Types:** Domain-specific errors via `thiserror` (`ConfigError`, `WorkspaceError`, `RegistryError`, `UpdateError`).
- **No Panics:** Production code paths forbid `unwrap()` and `expect()`.
- **Exit Codes:**
- `0`: Success (up to date or update completed).
- `1`: Failure / outdated dependencies during `--run=scan`.
- `2`: User cancellation (`/exit`, Ctrl+C).

### 3.3 Governance & Anti-Vibe-Coding Policy

- **License:** GNU General Public License v3.0 (`GPL-3.0-or-later`).
- **Conventional Commits:** Enforced on PR titles (`lint_pr.yml`) and local commits (`.githooks/commit-msg`).
- **Anti-Bot Rule:** Autonomous bot PRs or unsolicited automated reviews are strictly prohibited.
- **Anti-Vibe-Coding Policy:**
- AI may assist with fixing `cargo check` errors, drafting documentation, or small integrations up to 4 lines.
- Any larger full integration or bugfix generated by AI must terminate every single generated line with:
  `// I am an AI and i am dumb`

---

## 4. Module & Directory Layout

```

rust-recursive-deps-updater/
├── .github/
│ ├── workflows/
│ │ ├── ci.yml # Format, clippy, tests (Linux/Win), cargo-audit
│ │ ├── lint*pr.yml # Semantic PR title validation with sticky comments
│ │ └── release.yml # Cross-platform matrix release & crates.io publishing
│ ├── ISSUE_TEMPLATE/
│ │ ├── bug_report.md # Bug template with rrdu report integration
│ │ ├── feature_request.md # Feature template with duplicate check
│ │ └── config.yml # Community links & blank issue disabling
│ ├── copilot-instructions.md # Copilot pointer to AGENTS.md
│ └── pull_request_template.md # PR template with Human-In-The-Middle verification
├── .githooks/
│ ├── pre-commit # Local formatting, linting, and test guard
│ └── commit-msg # Local Conventional Commits validator
├── src/
│ ├── main.rs # #![forbid(unsafe_code)], CLI dispatcher & exit codes
│ ├── cli/ # Interactive UI, prompts, table formatter, pagination
│ │ ├── mod.rs
│ │ ├── prompt.rs # Interactive commands (*, \_-force, crate selection)
│ │ ├── display.rs # Colored console output & progress spinners
│ │ ├── table.rs # Zero-dependency table renderer for --run=scan
│ │ └── pagination.rs # Pagination & scroll logic (/next, /prev, arrow keys)
│ ├── config/ # .rrduconfig data models & noyalib YAML parser
│ │ ├── mod.rs
│ │ └── model.rs # Config structs, exclude evaluation, path validation
│ ├── workspace/ # Cargo workspace & standalone crate discovery
│ │ ├── mod.rs
│ │ ├── discovery.rs # Manifest search, glob resolution for workspace members
│ │ └── project.rs # Project models & dependency representation
│ ├── registry/ # crates.io client & SemVer comparison
│ │ ├── mod.rs
│ │ ├── client.rs # ureq HTTP client (rustls) with in-memory caching
│ │ └── semver_check.rs # SemVer comparison & breaking change classification
│ ├── self_update/ # In-place binary self-update
│ │ ├── mod.rs
│ │ └── client.rs # GitHub Releases querying & atomic binary replacement
│ └── updater/ # Manifest modification
│ ├── mod.rs
│ └── toml_writer.rs # toml_edit writer preserving comments & layout
├── tests/ # Integration test suite & fixtures
│ ├── config_tests.rs # YAML parsing & path traversal validation tests
│ ├── updater_tests.rs # toml_edit comment preservation tests
│ └── fixtures/ # Mock workspaces and multi-crate fixtures
├── action.yml # Composite Action for GitHub Marketplace
├── .cursorrules # Cursor IDE directive (points to AGENTS.md)
├── .gitignore # Ignore rules for Cargo, OS, IDEs
├── .windsurfrules # Windsurf IDE directive (points to AGENTS.md)
├── AGENTS.md # Single Source of Truth for AI assistants & developers
├── CLAUDE.md # Claude Code directive (points to AGENTS.md)
├── Cargo.toml # Crate metadata, dependencies, release profile
├── CODE_OF_CONDUCT.md # Contributor Covenant v2.1
├── CONTRIBUTING.md # Contributor workflow & anti-vibe-coding policy
├── GEMINI.md # Gemini CLI directive (points to AGENTS.md)
├── LICENSE # GNU General Public License v3.0 text
├── README.md # Official user manual & documentation
├── ROADMAP.md # This living roadmap & architecture specification
└── SECURITY.md # Security policy & vulnerability reporting

```

---

## 5. Crate Dependencies Overview

| Crate                 | Version Target    | Purpose              | Rationale                                                                              |
| --------------------- | ----------------- | -------------------- | -------------------------------------------------------------------------------------- |
| `clap`                | `~4.5` (derive)   | CLI Argument Parsing | Industry standard for robust CLI parsing (`init`, `--run=scan`, `--run=full`)          |
| `serde`               | `~1.0` (derive)   | Data Serialization   | Struct serialization/deserialization                                                   |
| `noyalib`             | `latest`          | YAML Parser          | Pure Rust YAML 1.2 with `#![forbid(unsafe_code)]`, maintained drop-in for `serde_yaml` |
| `toml_edit`           | `latest`          | Manifest Editor      | Official Cargo-team AST parser preserving comments, whitespace, and formatting         |
| `semver`              | `~1.0`            | SemVer Logic         | Rust core SemVer parser for accurate version comparison and breaking change checks     |
| `ureq`                | `latest` (rustls) | HTTP Client          | Synchronous, lightweight, memory-safe TLS without OpenSSL system dependencies          |
| `colored`             | `latest`          | Terminal Colors      | Clean, intuitive ANSI color output for terminal interfaces                             |
| `indicatif`           | `latest`          | Progress Indicators  | Smooth spinners and progress indicators during background network queries              |
| `walkdir`             | `latest`          | Filesystem Discovery | Fast recursive filesystem traversal for standalone `Cargo.toml` discovery              |
| `thiserror`           | `latest`          | Error Handling       | Ergonomic typed domain error hierarchies without panics                                |
| _(native `std::fmt`)_ | Built-in          | Table Formatter      | Zero-dependency 5-column table renderer in `src/cli/table.rs`                          |
| `tempfile`            | `latest` _(dev)_  | Integration Testing  | Isolated temporary workspaces for manifest modification verification                   |

---

## 6. Implementation Roadmap & Phases

### Phase 0: Setup, Governance, CI/CD & Marketplace Infrastructure

- [x] Create and clean Git repository.
- [x] Configure GNU General Public License v3.0 (`LICENSE`).
- [x] Establish governance documentation: `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CONTRIBUTING.md`.
- [x] Configure AI agent directives: `AGENTS.md`, `.cursorrules`, `.windsurfrules`, `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`.
- [x] Set up Semantic PR title validation workflow (`.github/workflows/lint_pr.yml`) with sticky comments and auto-deletion.
- [x] Set up CI workflow (`.github/workflows/ci.yml`) for `cargo fmt`, `cargo clippy`, Linux/Windows matrix tests, and `cargo-audit`.
- [x] Set up multi-platform release workflow (`.github/workflows/release.yml`) for Linux (GNU/musl) and Windows with SHA256 checksums and crates.io publishing.
- [x] Pin all GitHub Actions across all workflows to immutable full-length commit SHAs.
- [x] Implement GitHub Marketplace Composite Action (`action.yml`).
- [x] Configure native Git hooks in `.githooks/` (`pre-commit` and `commit-msg`).
- [x] Configure SSH commit signing and GitHub branch protection rules.
- [x] Finalize official user documentation in `README.md` and project assets.

### Phase 1: Configuration Layer (`.rrduconfig`) & `init` Command

- [x] Implement `src/config/model.rs` defining `RrduConfig`, `ProjectConfig`, and `UpdaterConfig`.
- [x] Implement YAML parsing via `noyalib` with comprehensive error mapping to `ConfigError`.
- [x] Implement path canonicalization and path traversal validation (rejecting `../` and root `/`).
- [x] Implement `rrdu init` generator creating a fully-commented `.rrduconfig` template with overwrite protection.
- [x] Unit tests for configuration parsing, default values, and path validation.

### Phase 2: Workspace & Sub-Crate Discovery Engine (`workspace`)

- [ ] Implement `src/workspace/project.rs` for crate and dependency representation.
- [ ] Implement `src/workspace/discovery.rs` parsing root `Cargo.toml` and resolving `[workspace.members]`.
- [ ] Implement recursive directory search fallback for non-workspace crates using `walkdir`.
- [ ] Implement exclusion filtering for `workspace.exclude` and `updater.exclude`.
- [ ] Parse `dependencies`, `dev-dependencies`, and `build-dependencies`.
- [ ] Filter out and ignore `git` and `path` dependencies (process direct crates.io dependencies only).
- [ ] Unit tests for workspace discovery and member globbing.

### Phase 3: crates.io Registry Client & SemVer Engine (`registry`)

- [ ] Implement `src/registry/client.rs` using `ureq` with `rustls` and strict timeouts.
- [ ] Implement in-memory cache to avoid duplicate network queries.
- [ ] Filter out yanked versions from registry responses.
- [ ] Implement `src/registry/semver_check.rs` with exact Cargo SemVer comparison:
  - Post-1.0: Major changes = breaking (`[Need manual migration]`), Minor/Patch = compatible (`[No migration needed]`).
  - Pre-1.0: `0.x` change = breaking (`[Need manual migration]`), `0.x.y` patch = compatible (`[No migration needed]`).
- [ ] Unit tests covering diverse SemVer comparison cases.

### Phase 4: Interactive CLI, Table Formatter & Headless Modes (`cli`)

- [ ] Implement interactive banner, status presentation, and `indicatif` spinner.
- [ ] Implement native zero-dependency table renderer in `src/cli/table.rs`.
- [ ] Implement interactive pagination (`/next`, `/prev`, arrow keys, `updater.max-lines`).
- [ ] Implement interactive commands: `*`, `*-force`, crate selection, `/back`, `/exit`, `/quit`.
- [ ] Implement in-app generic help (`/?`, `/h`, `/help`).
- [ ] Implement headless mode `rrdu --run=scan` (5-column formatted table output, exit code 0/1).
- [ ] Implement headless mode `rrdu --run=full` (automated update execution).
- [ ] Implement `rrdu report` diagnostic report generator with privacy sanitization.
- [ ] Implement `rrdu help` / `--help` CLI documentation display.

### Phase 5: Comment-Preserving Manifest Updater (`updater`)

- [ ] Implement `src/updater/toml_writer.rs` using `toml_edit`.
- [ ] Support standard string versions (`dep = "1.0"`).
- [ ] Support inline table dependencies (`dep = { version = "1.0", features = [...] }`).
- [ ] Support central `[workspace.dependencies]` updates.
- [ ] Ensure 100% preservation of comments, whitespaces, and indentation.
- [ ] Unit and fixture tests verifying comment preservation.

### Phase 6: End-to-End Verification, Fixtures & Release v0.1.0

- [ ] Implement integration tests in `tests/` using `tempfile` against fixture workspaces.
- [ ] Test headless CI runs (`--run=scan`, `--run=full`) on Linux and Windows.
- [ ] Verify clean `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
- [ ] Verify `cargo-audit` passes with zero vulnerabilities.
- [ ] Update crate metadata in `Cargo.toml` to `version = "0.1.0"`.
- [ ] Tag `v0.1.0` and trigger automated release workflow.

### Phase 7: Post-v0.1.0 Enhancements

- [ ] Transition crates.io publishing to Trusted Publishing (OIDC) after the initial release.
- [ ] Publish composite action to GitHub Marketplace.
- [ ] Implement binary self-update (`rrdu self-update`) against GitHub Releases API.

```

```
