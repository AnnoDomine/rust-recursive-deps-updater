# Configuration (`rrdu.toml`)

`rrdu` uses an optional TOML configuration file named `rrdu.toml` located in the root of your workspace or project repository.

> [!WARNING]
> **Deprecation Notice:** The legacy YAML format (`.rrduconfig`) is **deprecated** and will be removed in a future release. While existing `.rrduconfig` files continue to be supported transparently in-memory, you should migrate to `rrdu.toml` via `rrdu --migrate`. See [Migration Guide](history/migration.md).

---

## Initializing Configuration

Generate a tailored `rrdu.toml` automatically by scanning the workspace:

```sh
rrdu --init
```

If an `rrdu.toml` file already exists, `rrdu` will prompt for confirmation before overwriting.

---

## Migrating Legacy Configuration

To migrate an existing `.rrduconfig` file located at the current directory level to modern `rrdu.toml`:

```sh
rrdu --migrate
```

> [!CAUTION]
> The legacy `.rrduconfig` file is irrevocably deleted once `rrdu.toml` is written. We recommend creating your own backup prior to running `--migrate`.

Running `--migrate` converts legacy settings to TOML format and preserves all project boundaries.

---

## Configuration Schema

```toml
# Schema for editor autocompletion and validation (Taplo / Even Better TOML)
#:schema https://annodomine.github.io/rust-recursive-deps-updater/schemas/rrdu.schema.json

# List single 'Cargo.toml' workspaces
[workspaces]
rust-recursive-deps-updater = { label = "rrdu", path = "./", exclude = { project = ["comfy-table-inline", "rustls"], section = { dev-dependencies = ["tempfile"] } } }

# List single 'rrdu.toml' or legacy ".rrduconfig" sub-workspaces
[configs]
# workspace_crate = "crates/workspace_crate"

# 'rrdu' workspace configuration
[rrdu]
max-lines = 10
version = "0.1.0"
exclude = [
  "target",
  "vendor"
]
```

---

## Field Reference

### `[workspaces]`
- `<project-key>.path`: Relative path to the project directory containing `Cargo.toml` or directly to `Cargo.toml`. Paths must stay within `./` or subdirectories; directory traversal (`../`) and absolute root paths (`/`) are strictly rejected.
- `<project-key>.label`: Optional human-readable display label. Defaults to the table key if omitted.
- `<project-key>.exclude.project`: List of crate names excluded across all sections of this project.
- `<project-key>.exclude.section`: Section-scoped exclusions. Maps section names (e.g., `dev-dependencies`, `dependencies.clap`) to lists of crate names, or empty list `[]` to exclude the entire section.

### `[configs]`
- `<label> = "<path>"`: Sub-project directory or path providing its own `rrdu.toml` (or legacy `.rrduconfig`). When configured, `rrdu` loads the sub-configuration independently.

### `[rrdu]`
- `max-lines`: Number of dependency rows per page during interactive CLI navigation *(default: 10)*.
- `version`: Version of the schema with which the configuration file was created or migrated.
- `exclude`: Directory names ignored during recursive filesystem traversal.

---

## Editor Autocompletion & Validation (JSON Schema)

`rrdu.toml` supports real-time syntax checking, hover documentation, and auto-completion in modern editors (VS Code, Cursor, RustRover, NeoVim) through the official JSON Schema:

```toml
#:schema https://annodomine.github.io/rust-recursive-deps-updater/schemas/rrdu.schema.json
```
