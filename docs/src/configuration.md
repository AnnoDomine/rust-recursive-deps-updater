# Configuration (`.rrduconfig`)

`rrdu` uses an optional YAML configuration file named `.rrduconfig` located in the root of your workspace or project repository.

---

## Initializing Configuration

Generate a tailored `.rrduconfig` automatically by running:

```sh
rrdu --init
```

If a `.rrduconfig` file already exists, `rrdu` will prompt for confirmation before overwriting.

---

## Configuration Schema

```yaml
workspace:
  - project: Root
    path: ./
    exclude:
      project: # Excludes all named dependencies project-wide
        - tokio
      section: # Excludes named dependencies from specific sections
        dev-dependencies: # Exclude serde only from [dev-dependencies]
          - serde
        dependencies.clap: [] # Exclude [dependencies.clap] section completely
  - project: WorkspaceCrate
    path: crates/workspace_crate
    sub-config: true

updater:
  exclude:
    - target
    - vendor
  auto-update: none # Options: `none`, `semver-safe`, or `full`
  auto-scan: true # Options: true, false, or list of project names
  max-lines: 50 # Entries per page during interactive pagination
  version: 0.0.4 # Schema version
```

---

## Field Reference

### `workspace`
- `project`: Human-readable identifier for the crate or workspace root.
- `path`: Relative path to the project directory containing `Cargo.toml`. Paths must stay within `./` or subdirectories; parent traversal (`../`) and root paths (`/`) are strictly rejected.
- `exclude.project`: List of crate names excluded from all sections.
- `exclude.section`: Section-scoped exclusions.
- `sub-config`: When `true`, stops recursive traversal and uses the sub-project's own `.rrduconfig`.

### `updater`
- `exclude`: Directory names ignored during recursive filesystem traversal.
- `auto-update`: Automated update strategy in `--run=full` mode (`none`, `semver-safe`, or `full`).
- `auto-scan`: Automatic scan on interactive CLI startup (`true`, `false`, or list of project names).
- `max-lines`: Number of dependency rows per page during interactive CLI navigation.
- `version`: Version of the schema for backward compatibility checks.
