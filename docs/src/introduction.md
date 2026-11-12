# Introduction

**Rust Recursive Dependency Updater (`rrdu`)** is a fast, safe, zero-privilege CLI tool and GitHub Action designed to inspect, navigate, and recursively update dependencies across Rust workspaces and multi-crate repositories.

---

## Mission & Core Objectives

1. **Workspace & Crate Discovery:** Automatically locates all `Cargo.toml` manifests across cargo workspaces and nested standalone crates.
2. **Selective crates.io Auditing:** Checks dependencies exclusively against `crates.io`, intentionally ignoring git and path dependencies.
3. **Accurate SemVer Classification:** Categorizes updates according to official Cargo SemVer rules into compatible non-breaking bumps (`[No migration needed]`) and breaking updates (`[Need manual migration]`).
4. **Formatting-Preserving Manifest Updates:** Modifies manifests via `toml_edit` to ensure that 100% of formatting, indentation, whitespace, and user comments are preserved.
5. **Zero-Privilege & Pure User-Space:** Runs with `#![forbid(unsafe_code)]` across all internal logic and never requests elevated root or administrator permissions.

---

## Architectural Highlights

- **Dual-Mode Execution:** Interactive terminal interface with pagination, visual spinners, in-place overwriting, and keyboard navigation, alongside headless CI execution (`--headless`). All dependency updates require interactive validation by the user.
- **Two-Tier Registry Strategy:** Fast static queries via the crates.io Sparse Index for CI gates, paired with rich metadata queries strictly throttled to 1 request per second in interactive mode adhering to the crates.io Data Access Policy.
- **Tri-Cache Layer:** Persistent disk cache in `~/.rrdu/cache/` (Index, Web API, RustSec advisories, and Self-Update metadata) with TTL expiration checks.
- **In-Place Binary Self-Update:** Built-in self-update mechanism (`rrdu --self-update`) comparing the local build against GitHub Releases and crates.io.
- **Modern TOML Configuration & Schema:** Structured configuration via `rrdu.toml` backed by Taplo-compatible JSON Schema validation and an automated migration tool (`rrdu --migrate`).
- **Asynchronous Trace Logging & Structured Diagnostics:** Pure safe Rust async logging engine (`src/logger/`) with typed module status codes (`1YXX` - `8YXX`) backed by Tokio MPSC queuing (`mpsc::channel(5000)`). Log entries are written in the background to `./audit/rrdu.log` with automatic 1 MB file splitting and timestamped archive rotation (`rrdu_yyyy_mm_dd_hh_mm_ss_ms.log`) when `--trace` or `--report` is enabled.
- **Terminal Safety & Execution Boundaries:** 
  - In **Headless Mode (`--headless`)**, the tool completely avoids unsafe code and low-level TTY interactions, utilizing `comfy-table-inline` with `Table::force_no_tty` to eliminate unsafe `ioctl` terminal calls.
  - In **Interactive Mode**, rich terminal interactions rely on audited dependencies (`dialoguer`, `indicatif`). Potential risks stemming from low-level FFI/unsafe in their underlying libraries (`console`, `libc`, `windows-sys`) are actively minimized through defensive fallbacks: verifying TTY attachments (`Term::is_term()`) to prevent `ioctl` errors in non-interactive pipes, and implementing signal trapping (`Ctrl+C`) to cleanly restore terminal state and exit with code `2`.

---

## Current State and Version 0.2.0 Vision

`rrdu` (version `{{RRDU_VERSION}}`) provides production-ready recursive workspace discovery, scoped ignore rules, registry index queries, tri-caching, SemVer classification, interactive terminal navigation, binary self-updates, and comment-preserving manifest updates (`toml_edit`).

Looking forward to the upcoming major version **v0.2.0**:
- **Cargo Crate Core Refactoring:** The core workspace traversal and dependency graph engine is being refactored to embed the official `cargo` crate directly as a library dependency, delivering maximum execution performance, native Cargo resolution accuracy, and reduced memory overhead.
- **Interactive Workspace Dependency Management:** Directly enabled by the `cargo` crate integration, developers will be able to add and delete dependencies across workspace crates interactively (`Add/delete deps in workspace in interactive mode`).
- **Desktop Graphical UI via Dioxus (`--ui`):** A modern desktop GUI built with Dioxus on top of the interactive core, delivering rich workspace tree visualization and elevated UX/UI and digital accessibility.

> Detailed information regarding completed milestones and upcoming implementations can be found in the [Roadmap](roadmap.md).

