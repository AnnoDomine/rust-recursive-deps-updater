# Quality & Verification

`rrdu` enforces strict engineering standards to ensure zero panics, complete memory safety, and dependable performance.

---

## Standards Overview

- **100% Safe Rust Core Logic:** `#![forbid(unsafe_code)]` enforced at the crate root (`src/main.rs` and `src/lib.rs`).
- **Zero-Unsafe Headless Execution:** Completely eliminates unsafe TTY code and `ioctl` calls in `--headless` CI mode via `Table::force_no_tty`.
- **Defensive Interactive Fallbacks:** Isolates terminal dependencies (`dialoguer`, `indicatif`) to interactive mode, mitigating dependency-level FFI risks with proactive TTY checks and signal trapping (`Ctrl+C` cleanup).
- **Zero-Privilege:** Strictly user-space execution.
- **Continuous Integration:** Every commit is validated across:
  - Formatting (`cargo fmt --check`)
  - Linter warnings (`cargo clippy --all-targets -- -D warnings`)
  - Unit and integration tests (`cargo test`)
  - Cross-platform builds (Linux GNU/musl, Windows, macOS)
  - Security audit (`cargo audit`)

---

## Sections

- [Test Results](test_results.md): Overview of unit testing, integration tests, and CI test matrices.
- [Benchmarks](benchmarks.md): Performance benchmarks and Criterion test results.
