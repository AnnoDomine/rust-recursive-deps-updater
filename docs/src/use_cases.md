# Use-Cases

`rrdu` accommodates workflows ranging from interactive local development to automated CI/CD pipelines.

---

## 1. Interactive Developer Workflow

Run `rrdu` in your repository root without flags:

```sh
rrdu
```

- **Interactive Selection:** Inspect individual crates or select `*` to view all.
- **Selective Upgrades:**
  - `*`: Apply all safe, non-breaking updates (`[No migration needed]`).
  - `*-force`: Apply all available updates including breaking major versions (`[Need manual migration]`).
  - `<index>` / `<name>`: Update an individual dependency interactively.
- **Navigation:** Paginate using `/next`, `/prev`, arrow keys, `/back` to return to previous menus, or `/exit` to quit.

---

## 2. Headless CI / Pull Request Gate (`--run=scan`)

Automate dependency health checks on Pull Requests:

```sh
rrdu --run=scan
```

- Renders a clean 5-column tabular report formatted via `comfy-table-inline`.
- **Exit Code 0:** All dependencies are up to date.
- **Exit Code 1:** Outdated dependencies were discovered (or a validation error occurred).
- Fails the CI pipeline if dependencies require updating, preventing dependency drift.

---

## 3. Automated Dependency Upgrades (`--run=full`)

Execute unattended updates based on the rules configured in `.rrduconfig`:

```sh
rrdu --run=full
```

- Updates manifests in place using `toml_edit`.
- Preserves all whitespace, comments, and structure.
- Exits with code `0` on success.

---

## 4. Multi-Crate Monorepos and Nested Crates

In repositories containing mixed workspace members and standalone nested crates:
- `rrdu` identifies the root workspace members as well as independent sub-crates.
- Central `[workspace.dependencies]` are detected and updated, and member crates inheriting from workspace dependencies (`dep = { workspace = true }`) remain properly linked.
