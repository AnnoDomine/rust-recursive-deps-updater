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
    version: 'latest' # Optional: pin to a release tag like v0.0.6
```

For security, pinning to full immutable commit SHAs is recommended.

---

## 4. In-Place Binary Self-Update (`--self-update`)

Once installed, you can update `rrdu` directly in-place to the newest GitHub Release without needing root or administrator privileges:

```sh
rrdu --self-update
```

- **Safety:** Replaces the binary atomically in user space.
- **Upstream Sources:** Checks GitHub Releases for new pre-compiled binaries and compares versions with `crates.io`.
- **Caching:** Self-update metadata is cached in `~/.rrdu/cache/rrdu_self_update_cache.bin` with a 24-hour TTL to prevent redundant network calls.
