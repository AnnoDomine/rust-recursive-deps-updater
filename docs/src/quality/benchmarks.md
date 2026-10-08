# Benchmarks

Performance is a key objective of `rust-recursive-deps-updater`. Benchmarks ensure that discovery, manifest parsing, SemVer checks, and table rendering remain fast and regression-free.

---

## Tooling & Methodology

Benchmarks are implemented using [`criterion`](https://crates.io/crates/criterion) under the `benches/` directory.

- **Statistical Analysis:** Criterion provides outlier detection, mean/median execution times, and standard deviation calculations.
- **Flamegraphs & HTML Reports:** Generated under `target/criterion/report/index.html` during benchmark runs.

---

## Target Benchmark Scenarios

1. **CLI Table Formatting & Rendering:**
   - Benchmarks `rrdu::cli::table` across small (5-10 rows), medium (50 rows), and large (500+ rows) dependency sets using `comfy-table-inline`.
   - Evaluates dynamic column width calculation, alignment, and pagination slicing.
2. **Workspace Discovery & Ignore Traversal:**
   - Measures traversal throughput across deep directory trees with `.gitignore` and `.rrduignore` rules.
3. **AST Manifest Manipulation (`toml_edit`):**
   - Assesses in-place parsing and modification of complex multi-crate `Cargo.toml` files while preserving comments.

---

## Running Benchmarks Locally

```sh
cargo bench
```
