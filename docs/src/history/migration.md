# Migration Guide

As the project may introduce breaking changes during early development, this document provides a guide for updating your configuration files between versions.

---

## Version 0.1.0 to 0.1.1

### Transition from `.rrduconfig` (YAML) to `rrdu.toml` (TOML)

> [!WARNING]
> **Deprecation Notice:** `.rrduconfig` (YAML) is now **deprecated**. While full backwards compatibility is maintained in this release (legacy configurations continue to work transparently in-memory without breaking existing workflows), `.rrduconfig` will be completely removed in a future release. We strongly recommend migrating to `rrdu.toml`.

#### 1. Migration Command (`--migrate`)
You can migrate an existing `.rrduconfig` file located at the current folder level to `rrdu.toml` using:

```sh
rrdu --migrate
```

* **Scope:** Migrates the configuration file located at the current working directory level.
* **Deletion & Backups:** The legacy `.rrduconfig` file is irrevocably deleted after `rrdu.toml` is written. We recommend creating your own backup beforehand.
* **Necessity:** Running `--migrate` is **optional for now**. If no `rrdu.toml` is present, `rrdu` automatically falls back to `.rrduconfig` and processes it in-memory. However, as legacy `.rrduconfig` will be dropped in a subsequent version, updating to `rrdu.toml` ensures long-term compatibility.

#### 2. Configuration Schema Differences

##### Legacy `.rrduconfig` (YAML - Deprecated):
```yaml
workspace:
  - project: my-crate
    path: ./
    exclude:
      project:
        - serde
      section:
        dev-dependencies:
          - tokio
  - project: sub-crate
    path: crates/sub
    sub-config: true

updater:
  exclude:
    - vendor
  max-lines: 10
  version: 0.1.0
```

##### Modern `rrdu.toml` (TOML - Active Standard):
```toml
# Schema for editor autocompletion and validation (Taplo / Even Better TOML)
#:schema https://annodomine.github.io/rust-recursive-deps-updater/schemas/rrdu.schema.json

[workspaces]
my-crate = { label = "my-crate", path = "./", exclude = { project = ["serde"], section = { dev-dependencies = ["tokio"] } } }

[configs]
sub-crate = "crates/sub"

[rrdu]
max-lines = 10
version = "0.1.0"
exclude = ["vendor"]
```

---

## Version 0.0.6 to 0.1.0

### Changes in `.rrduconfig`

1. **Full Backwards Compatibility:**  
   Version 0.1.0 stabilizes the end-to-end integration and verification suites with zero breaking schema changes. All existing `.rrduconfig` configuration files continue to work without modification.

---

## Version 0.0.5 to 0.0.6

### Changes in `.rrduconfig`

1. **No Breaking Schema Changes:**  
   Version 0.0.6 introduces in-place manifest updates via `toml_edit` without modifying the `.rrduconfig` format. Existing configuration files remain 100% compatible.

---

## Version 0.0.4 to 0.0.5

### Changes in `.rrduconfig`

1. **Removed fields `updater.auto-update` and `updater.auto-scan`:**
   As `updater.auto-update` and `updater.auto-scan` are no more needed, we removed them from `.rrduconfig`. These are no breaking changes.

---

## Version 0.0.3 to 0.0.4

### Changes in `.rrduconfig`

1. **Change value of `updater.auto-update`:**
   For better readability we changed the possible values of the `updater.auto-update` to `none`, `semver-safe` and `full`. As this key currently is not used, it is not breaking the application.

---

## Version 0.0.1/0.0.2 to 0.0.3

### Changes in `.rrduconfig`

1. **Renamed `toml` to `path`:**  
   Starting from version 0.0.3, sub-project `.rrduconfig` files are supported. Therefore, the field `toml` in the `workspace` array has been renamed to `path`.
2. **Added `sub-config` field (boolean):**
   - `false` (default): No dedicated `.rrduconfig` file for this sub-project. `path` points directly to the directory / `Cargo.toml`.
   - `true`: `path` points to a sub-project directory containing its own `.rrduconfig`.
3. **Changed schema of `exclude`:**
   `exclude` was before a simple list with project wide excluded dependencies. With the 0.0.3 we changed it to a more granular schema which differs between project wide exclusion and section specified exclusion to give more control.
4. **Added `updater.version` field:**  
   Specifies the configuration schema version for easier compatibility checks.

> **Rationale:** These changes avoid unnecessary discoveries of projects in deep child folders and keep the root `.rrduconfig` concise.

#### Configuration Diff

```yaml
# Before (0.0.1):
workspace:
  - project: Root
    toml: ./
    exclude:
      - tokio
      - serde
  - project: WorkspaceCrate
    toml: crates/workspace_crate

updater:
  exclude:
    - excluded_folder
  auto-update: none
  auto-scan: true
  max-lines: 50

# After (0.0.3):
workspace:
  - project: Root
    path: ./
    exclude:
      project: # Excludes all named dependencies project wide
        - tokio
      section: # Excludes all named dependencies from specific sections
        dev-dependencies: # Only exclude named dependencies from [dev-dependencies] section
          - serde
        dependencies.clap: [] # Exclude [dependencies.clap] section in total.
  - project: WorkspaceCrate
    path: crates/workspace_crate
    sub-config: true

updater:
  exclude:
    - excluded_folder
  auto-update: none
  auto-scan: true
  max-lines: 50
  version: 0.0.3
```
