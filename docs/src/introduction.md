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
- **Tri-Cache Layer:** Persistent disk cache in `~/.rrdu/cache/` (Index, Web API, and RustSec advisories) with TTL expiration checks.
- **Terminal Safety & Execution Boundaries:** 
  - In **Headless Mode (`--headless`)**, the tool completely avoids unsafe code and low-level TTY interactions, utilizing `comfy-table-inline` with `Table::force_no_tty` to eliminate unsafe `ioctl` terminal calls.
  - In **Interactive Mode**, rich terminal interactions rely on audited dependencies (`dialoguer`, `indicatif`). Potential risks stemming from low-level FFI/unsafe in their underlying libraries (`console`, `libc`, `windows-sys`) are actively minimized through defensive fallbacks: verifying TTY attachments (`Term::is_term()`) to prevent `ioctl` errors in non-interactive pipes, and implementing signal trapping (`Ctrl+C`) to cleanly restore terminal state and exit with code `2`.

---

## Current State and Integrated Handlings

`rrdu` is currently under active pre-release development. Fundamental components including recursive workspace discovery, scoped ignore rules, registry index queries, tri-caching, and SemVer classification are fully implemented and covered by unit tests. Interactive terminal navigation and comment-preserving manifest updates are actively being integrated.

> Detailed information regarding completed milestones and upcoming implementations can be found in the [Pre-Release Roadmap](roadmap.md).

