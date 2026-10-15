# Accessibility Statement for `rust-recursive-deps-updater` (`rrdu`)

`rust-recursive-deps-updater` (`rrdu`) is committed to ensuring digital accessibility for people with disabilities. We continually improve the user experience for everyone and apply relevant accessibility standards to our command-line interface, terminal abstractions, and documentation.

---

## 1. Keyboard Navigation & Motor Accessibility

`rrdu` is 100% operable via keyboard without requiring any pointing device or mouse input.

### Keybindings Reference

| Keystroke | Scope | Function |
| :--- | :--- | :--- |
| `↑` / `k` | List & Detail Views | Move selection or scroll up by one line |
| `↓` / `j` | List & Detail Views | Move selection or scroll down by one line |
| `←` / `h` | List & Detail Views | Switch to previous page or tab |
| `→` / `l` | List & Detail Views | Switch to next page or tab |
| `Home` | List & Detail Views | Jump directly to the first item, page, or line |
| `End` | List & Detail Views | Jump directly to the last item, page, or line |
| `PageUp` | List & Detail Views | Scroll up by one full page |
| `PageDown` | List & Detail Views | Scroll down by one full page |
| `Enter` / `Space` | List Views | Select highlighted item and open child view |
| `u` | List & Detail Views | Execute manifest dependency update on highlighted/active crate |
| `Backspace` | All Views | Navigate back up to the parent view level |
| `Esc` / `Ctrl+C` | All Views | Cleanly restore terminal state and exit |

---

## 2. Terminal State & Screen Restoration

- **Alternate Screen Buffer:** In interactive mode, `rrdu` enters an alternate screen buffer (`\x1b[?1049h`). Upon exit, completion, or user cancellation (`Esc` / `Ctrl+C`), the alternate buffer is closed (`\x1b[?1049l`), completely restoring the user's previous terminal buffer.
- **Cursor & Raw Mode Restoration:** The terminal cursor is automatically unhidden and raw mode is cleanly deactivated upon exit or signal interruption (`SIGINT`), preventing corrupted or unresponsive terminal sessions.
- **Defensive TTY Fallback:** If `rrdu` is executed in a non-interactive environment (pipes, redirected streams, or CI pipelines without a TTY), it detects this condition proactively and exits with an informative error message recommending `rrdu --headless` rather than risking terminal corruption.

---

## 3. Visual Accessibility & Screen Reader Support

### Color Independence
Information is never conveyed solely through color:
- Status classifications include explicit textual indicators: `[No migration needed]`, `[Need manual migration]`, `*Update`, `[Excluded]`, `[Error]`.
- Active selections are delimited by high-contrast ASCII markers (`|-> 1  <-|`, `[ TabName ]`).
- Pagination bars provide explicit numerical indicators (`Highlighted: 1`, `Items: 1-10 / 45`).

### Headless & Screen Reader Friendly Mode (`--headless`)
For users relying on screen readers or terminal accessibility utilities, `rrdu --headless` provides:
- Clean, non-interactive Unicode/ASCII table output directly to `stdout`.
- Complete elimination of in-place line overwriting, cursor repositioning escape codes, and animation spinners.
- Full compatibility with piped streams, terminal speech synthesizers, and braille displays.

---

## 4. Cognitive Accessibility & Predictable Layouts

- **Hierarchical Breadcrumb Navigation:** Menus follow a predictable hierarchical mental model: `Workspace` $\rightarrow$ `Project` $\rightarrow$ `Section` $\rightarrow$ `Dependency`.
- **Cognitive Load Reduction:** Screen clutter is bounded by configurable pagination (`updater.max_lines` in `.rrduconfig`), ensuring that large dependency trees are presented in digestible chunks.
- **Explicit Instructions:** Every view level displays a contextual instruction prompt (e.g. `Select project:`, `Select dependency section:`).

---

## 5. Feedback & Reporting Accessibility Barriers

We welcome feedback on the accessibility of `rrdu`. If you encounter accessibility barriers or have suggestions for improvement, please open an issue on GitHub:
- **Repository Issues:** [https://github.com/AnnoDomine/rust-recursive-deps-updater/issues](https://github.com/AnnoDomine/rust-recursive-deps-updater/issues)
