# Third-Party Dependencies & Licenses

`rust-recursive-deps-updater` (`rrdu`) is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**.

All dependencies used in this project are strictly sourced from [crates.io](https://crates.io/) and adhere to permissive open-source licenses compatible with the GPL-3.0.

---

## Direct Dependencies

| Crate | Version | License (SPDX) | Role / Purpose | Notes |
| :--- | :--- | :--- | :--- | :--- |
| [`clap`](https://crates.io/crates/clap) | `4.6.7` | `Apache-2.0 OR MIT` | Command-line argument parsing (`derive`) | Standard robust CLI flag parser |
| [`colored`](https://crates.io/crates/colored) | `3.1.1` | `MPL-2.0` | ANSI terminal color styling | Intuitive terminal text coloring |
| [`comfy-table-inline`](https://crates.io/crates/comfy-table-inline) (aliased as `comfy-table`) | `8.0.2` | `MPL-2.0` | Terminal table rendering | **Custom release:** Based on `comfy-table`, integrating native inline table support while configuring `Table::force_no_tty` by default for safe Rust |
| [`indicatif`](https://crates.io/crates/indicatif) | `0.18.6` | `MIT` | Progress indicators & spinners | Smooth progress feedback during network queries |
| [`log`](https://crates.io/crates/log) | `0.4.34` | `Apache-2.0 OR MIT` | Standard logging facade | Modular status codes and diagnostics |
| [`noyalib`](https://crates.io/crates/noyalib) | `0.0.55` | `Apache-2.0 OR MIT` | YAML 1.2 parser | Pure safe Rust YAML parser (`#![forbid(unsafe_code)]`) |
| [`semver`](https://crates.io/crates/semver) | `1.0.28` | `Apache-2.0 OR MIT` | SemVer parsing & comparison | Official Cargo SemVer comparison logic |
| [`serde`](https://crates.io/crates/serde) | `1.0.229` | `Apache-2.0 OR MIT` | Generic data serialization (`derive`) | Struct serialization framework |
| [`serde_json`](https://crates.io/crates/serde_json) | `1.0.151` | `Apache-2.0 OR MIT` | JSON & NDJSON parsing | Parsing sparse index lines, API metadata, and tri-cache |
| [`simple_logger`](https://crates.io/crates/simple_logger) | `5.2.0` | `MIT` | Logger implementation | Timestamped console and file logging |
| [`thiserror`](https://crates.io/crates/thiserror) | `2.0.21` | `Apache-2.0 OR MIT` | Ergonomic domain errors | Typed error hierarchies without panics |
| [`time`](https://crates.io/crates/time) | `0.3.55` | `Apache-2.0 OR MIT` | Timestamp formatting | Precise formatted timestamps for logging and caching |
| [`toml_edit`](https://crates.io/crates/toml_edit) | `0.25.16` | `Apache-2.0 OR MIT` | TOML AST parser and writer | Format-preserving manifest updater keeping all comments and indentation intact |
| [`ureq`](https://crates.io/crates/ureq) | `3.4.2` | `Apache-2.0 OR MIT` | Synchronous HTTP client (`rustls`, `json`) | Memory-safe HTTPS network client without OpenSSL dependencies |

---

## Development Dependencies

| Crate | Version | License (SPDX) | Role / Purpose |
| :--- | :--- | :--- | :--- |
| [`criterion`](https://crates.io/crates/criterion) | `0.8.2` | `Apache-2.0 OR MIT` | Micro-benchmarking framework with statistics and HTML reports |
| [`tempfile`](https://crates.io/crates/tempfile) | `3.27.0` | `Apache-2.0 OR MIT` | Isolated temporary directories for manifest and filesystem integration tests |

---

## License Notice for `comfy-table-inline`

`rrdu` utilizes `comfy-table-inline`, a customized edition of `comfy-table` published on crates.io by Dominic Seel:
- **Repository:** [https://github.com/AnnoDomine/comfy-table](https://github.com/AnnoDomine/comfy-table)
- **License:** Mozilla Public License 2.0 (MPL-2.0)
- **Modifications:** Extended the upstream table engine with first-class inline table rendering capabilities for cleaner multi-column dependency views, while strictly maintaining `Table::force_no_tty` to ensure 100% safe Rust without OS-level `ioctl` calls.
