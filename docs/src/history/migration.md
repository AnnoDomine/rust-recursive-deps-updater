# Migration Guide

As `rrdu` evolves during early development, breaking changes to the `.rrduconfig` configuration schema are documented here with migration instructions.

---

## Version 0.0.3 to 0.0.4

### Changes in `.rrduconfig`

1. **Enum value updates for `updater.auto-update`:**
   Possible values are `none`, `semver-safe`, and `full`.

---

## Version 0.0.1/0.0.2 to 0.0.3

### Changes in `.rrduconfig`

1. **Renamed `toml` to `path`:**  
   Sub-project `.rrduconfig` files are supported. Therefore, `toml` in the `workspace` list is renamed to `path`.
2. **Added `sub-config` field (boolean):**
   - `false` (default): Points directly to the directory / `Cargo.toml`.
   - `true`: Points to a sub-project directory containing its own `.rrduconfig`.
3. **Changed schema of `exclude`:**
   Changed to a granular schema differentiating between `project` and `section` exclusions.
4. **Added `updater.version` field:**  
   Specifies the configuration schema version for compatibility checks.

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
      project:
        - tokio
      section:
        dev-dependencies:
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
