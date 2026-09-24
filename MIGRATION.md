# Migration Guide

As the project may introduce breaking changes during early development, this document provides a guide for updating your configuration files between versions.

---

## Version 0.0.1/0.0.2 to 0.0.3

### Changes in `.rrduconfig`

1. **Renamed `toml` to `path`:**  
   Starting from version 0.0.3, sub-project `.rrduconfig` files are supported. Therefore, the field `toml` in the `workspace` array has been renamed to `path`.
2. **Added `sub-config` field (boolean):**  
   - `false` (default): No dedicated `.rrduconfig` file for this sub-project. `path` points directly to the directory / `Cargo.toml`.
   - `true`: `path` points to a sub-project directory containing its own `.rrduconfig`.
3. **Added `updater.version` field:**  
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
      - tokio
      - serde
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