# Installation

`rrdu` is available as a pre-compiled standalone binary, via Cargo, and as a GitHub Marketplace Action.

---

## 1. Pre-Compiled Binaries (Linux & Windows)

Download the latest pre-compiled binary matching your system architecture directly from the [GitHub Releases](https://github.com/AnnoDomine/rust-recursive-deps-updater/releases) page:

- **Linux (x86_64 glibc):** `rrdu-x86_64-unknown-linux-gnu.tar.gz`
- **Linux (x86_64 musl static):** `rrdu-x86_64-unknown-linux-musl.tar.gz`
- **Windows (x86_64 MSVC):** `rrdu-x86_64-pc-windows-msvc.zip`
- **macOS (Apple Silicon & Intel):** `rrdu-aarch64-apple-darwin.tar.gz` / `rrdu-x86_64-apple-darwin.tar.gz`

Extract the binary and place it in your `$PATH` (e.g. `~/.local/bin` or `/usr/local/bin`).

---

## 2. Via Cargo

Install `rrdu` using Cargo from `crates.io`:

```sh
cargo install rust-recursive-deps-updater
```

To update an existing installation:

```sh
cargo install rust-recursive-deps-updater --force
```

---

## 3. GitHub Actions Composite Action

You can integrate `rrdu` directly into your GitHub Actions CI/CD workflows:

```yaml
- name: Check Rust Dependencies
  uses: AnnoDomine/rust-recursive-deps-updater@v1
  with:
    run: "scan"
```

For security, pinning to full immutable commit SHAs is recommended.
