# Quality & Verification

`rrdu` enforces strict engineering standards to ensure zero panics, complete memory safety, and dependable performance.

---

## Standards Overview

- **100% Safe Rust:** `#![forbid(unsafe_code)]` at the crate root.
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
