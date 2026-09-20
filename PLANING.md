# Rust Recursive Dependency Updater (`rrdu`) - Planungs- & Architektur-Dokument

Dieses Dokument dient als zentrale Wissensbasis und Fahrplan für die Entwicklung von `rust-recursive-deps-updater`. Es wird bei jedem Planungsschritt und jeder architektonischen Entscheidung synchron gehalten.

---

## 1. Vision & Zielsetzung

`rust-recursive-deps-updater` (`rrdu`) ist ein eigenständiges CLI-Tool für Entwickler und Teams, die Rust-Workspaces und Multi-Crate-Repositories verwalten. Es ermöglicht:
1. Das transparente Erfassen aller Abhängigkeiten über den Root-Workspace und alle Sub-Crates hinweg.
2. Die Konfiguration über eine optionale Datei `.rrduconfig` (YAML) zur Definition von Projekten und Ausnahmelisten (`exclude`).
3. Den automatischen Abgleich mit den neuesten stabilen Versionen auf crates.io.
4. Eine intelligente SemVer-Einstufung in:
   - **Aktuell** (`latest version`)
   - **Kompatibles Update** (`[No migration needed]`)
   - **Major / Breaking Update** (`[Need manual migration]`)
5. Ein interaktives Terminal-Interface mit flexiblen Update-Befehlen (`*`, `*-force`, Crate-spezifisch) und schonender Aktualisierung der `Cargo.toml`-Dateien (Formatierung und Kommentare bleiben erhalten).

---

## 2. Spezifikation der Kernkomponenten

### 2.1 Konfiguration (`.rrduconfig` & GitHub Action)
- **Format:** YAML
- **Standardpfad:** Im Root-Verzeichnis des Workspaces (`./.rrduconfig`).
- **Optionaler Pfad:** Über CLI kann optional `--config-path=<path>` angegeben werden (strikt validiert: nur Unterordner/Workspace-intern, kein Path-Traversal).
- **Inline-Konfiguration in GitHub Action:**
  - Die GitHub Action (`action.yml`) unterstützt einen Parameter `config: | ...`, womit Nutzer die Konfiguration direkt inline in ihrem GitHub Workflow definieren können, ohne eine Datei `.rrduconfig` im Repository anlegen zu müssen.
  - Alternativ unterstützt die Action `config-path: <path>` für abweichende Ablageorte.
  - Fehlen beide, greift die Action automatisch auf die `./.rrduconfig` im Workspace-Root zu.
- **Verhalten bei Fehlen:** Automatisches Discovery über die `[workspace]`-Sektion in der Root-`Cargo.toml` sowie rekursive Suche nach Sub-Crates.
- **Sicherheits-Grundsatz:** Keine dynamischen Injektions-Flags. Alle Einstellungen erfolgen ausschließlich über validierte YAML-Strukturen (Datei oder validierte Action-Config).
- **Struktur:**
  ```yaml
  workspace:
    - project: "Root"
      toml: "./"
      exclude:
        - "tokio"
        - "serde"
    - project: "WorkspaceCrate"
      toml: "./crates/workspace_crate/"
  
  updater:
    exclude:
      - "./excluded_folder/"
    auto-update: "none"
    auto-scan: true
    max_lines: 50
  ```
_`workspace.exclude`, `updater.exclude` und `updater.max_lines` sind optional._

- `workspace.project.exclude`: Liste von Crate-Namen, die für dieses Projekt vom Scan und Update ausgeschlossen werden. (Optional)
- `updater.exclude`: Ordnerpfade, die beim rekursiven Dateiscan vollständig ignoriert werden. (Optional)
- `updater.auto-update`: Update-Strategie für den automatisierten Modus oder Projektauswahl (`none`, `*`, `*-force`). Default: `"none"`.
- `updater.auto-scan`: Automatischer Scan bei Start der CLI (`true`, `false` oder Liste von Projektnamen). Default: `true`.
- `updater.max_lines`: Maximale Anzahl an Zeilen/Einträgen pro Seite bei der interaktiven Paginierung. (Optional, Default: `50`).


### 2.2 Workspace- & Crate-Erkennung
- **Root-Projekt:** Prüfung auf Vorhandensein von `Cargo.toml`.
- **Mitglieder-Erkennung:**
  - Auswertung von `[workspace.members]` im Root-Manifest (inklusive Globs wie `crates/*`).
  - Rekursiver Fallback-Scan, falls keine Workspace-Definition vorliegt, aber verschachtelte Crates existieren.
- **Workspace-Dependencies:**
  - Erkennung von zentralen `[workspace.dependencies]` in modernen Rust-Projekten.
  - Erkennung von vererbten Abhängigkeiten in Sub-Crates (`dep = { workspace = true }`).
- **Nicht in Workspace integrierte Projekte:**
  - Erkennung eigenständiger "Non-Workspace Projects" anhand im Dateibaum gefundener `Cargo.toml`.

### 2.3 Crates.io Versionsabfrage & SemVer-Klassifizierung
- **Unterstützte Abhängigkeiten:**
  - **Ausschließlich direkte `crates.io`-Abhängigkeiten:** Git-Dependencies (`git = "..."`) und Path-Dependencies (`path = "..."`) werden bewusst nicht unterstützt bzw. übersprungen und nicht bei crates.io angefragt.
- **Registry-Schnittstelle:**
  - crates.io Web-API (`GET https://crates.io/api/v1/crates/{crate}`) oder Sparse-Index (`https://index.crates.io/...`).
  - Caching der Abfrageergebnisse innerhalb eines Durchlaufs (damit identische Crates in verschiedenen Sub-Projekten nicht mehrfach angefragt werden).
  - Berücksichtigung von `yanked`-Versionen (diese werden ignoriert).
- **SemVer-Regeln für Migrationsbedarf:**
  - **Version >= 1.0.0:**
    - Patch- oder Minor-Sprung (z. B. `1.2.0` -> `1.3.1`): `[No migration needed]`
    - Major-Sprung (z. B. `1.8.0` -> `2.0.0`): `[Need manual migration]`
  - **Version < 1.0.0 (Pre-1.0 SemVer-Konvention in Rust):**
    - `0.x.y` mit Änderung von `x` (z. B. `0.12.5` -> `0.20.0` oder `0.1.2` -> `0.2.0`): Gilt im Rust-Ökosystem als Breaking Change -> `[Need manual migration]`
    - `0.x.y` mit reiner Änderung von `y` (z. B. `0.5.3` -> `0.5.4`): `[No migration needed]`
  - **Wildcards/Tilde/Caret:**
    - Berücksichtigung gängiger Versionsanforderungen (`^`, `~`, exact `=`).

### 2.4 CLI-Befehl & Betriebsmodi

Das Tool wird als Binary mit dem Namen `rrdu` (bzw. `rrdu.exe` unter Windows) kompiliert und bereitgestellt.

##### A. Interaktiver Modus (`rrdu` ohne Argumente)
- **Workflow:**
  1. Begrüßung und Anzeige des aktuellen Arbeitsverzeichnisses.
     - **Versionsprüfung im Header:** Prüfung gegen die GitHub Releases Page im Hintergrund. Falls eine neuere Version verfügbar ist, wird ein Hinweis angezeigt:
       `💡 Eine neuere Version von rrdu ist verfügbar (vX.Y.Z -> vA.B.C). Führe 'rrdu self-update' aus, um zu aktualisieren.`
  2. Information über gefundene oder fehlende `.rrduconfig`.
  3. Auflistung aller erkannten Projekte/Crates mit Pfaden:
     - **Paginierung / Page-by-Page-Navigation:** Bei umfangreichen Workspaces erfolgt die Darstellung seitenweise (z. B. 10–15 Einträge pro Seite), um das Terminal übersichtlich zu halten.
     - Scrollen mit Pfeiltasten (Hoch/Runter) oder Befehlseingabe:
       - `/next` : Zur nächsten Seite blättern
       - `/prev` : Zur vorherigen Seite blättern
  4. Auswahl:
     - Einzelnes Projekt (z. B. `Root`, `1`, etc.)
     - `*` für alle Projekte nacheinander / gesammelt
  5. Detailanzeige je Projekt:
     - Anzahl gefundener und ausgeschlossener Abhängigkeiten.
     - Farbige Statusauflistung aller Crates (ebenfalls mit Paginierung bei langen Listen).
  6. Update-Aufforderung:
     - `*` : Aktualisiert nur Abhängigkeiten ohne Migrationsbedarf (`[No migration needed]`).
     - `*-force` : Aktualisiert ausnahmslos alle veralteten Abhängigkeiten.
     - `<crate_name>` : Aktualisiert selektiv nur die angegebene Crate.
  7. Modifikation & Bestätigung:
     - Saubere Anpassung der `Cargo.toml`.
     - Optionales Ausführen von `cargo check` oder Ausgabe einer Zusammenfassung.
  8. Navigation & Interaktive Hilfebefehle:
     - Mit `/?`, `/h` oder `/help` kann sich der Nutzer an jedem Prompt die verfügbaren generischen Steuerbefehle anzeigen lassen (`/back`, `/exit`, `/quit`, `/next`, `/prev`, `*`, `*-force`).
     - Spezifische Werte (wie individuelle Projekt- oder Crate-Namen) werden separat als erklärende Infozeile dargestellt.
     - Mit `/back` gelangt der Nutzer immer einen Schritt zurück im Menü.
     - Mit `/exit` oder `/quit` beendet er die CLI von überall.

#### B. Headless CI- & Automatisierungs-Modus (`rrdu --run=<mode>`)
Ein einzelnes Argument `--run` steuert die Headless-Ausführung (keine Paginierung, vollständige lineare Ausgabe für CI-Logs):
- **`rrdu --run=scan` (Reiner Analyse- & Prüflauf):**
  - Führt einen nicht-interaktiven Scan basierend auf `.rrduconfig` (oder Action-Inline-Config) durch.
  - **Tabellarische Ausgabe:** Formatierte Konsolentabelle mit fest definierten Spalten:
    | Spalte | Beschreibung | Wert / Format |
    |---|---|---|
    | **Project** | Name des Projekts / Crates | z. B. `Root`, `WorkspaceCrate` |
    | **Dependency** | Name der Abhängigkeit | z. B. `tokio`, `clap` |
    | **Current version** | Aktuell deklarierte Version | z. B. `0.12.5`, `1.2.0` |
    | **Latest version** | Neueste Version auf crates.io | z. B. `0.20.0`, `1.3.1` |
    | **Migration necessary** | Einstufung des Migrationsbedarfs | `true` (bei Breaking Change), `false` (bei kompatiblem Update), **leer** (wenn `latest == current`) |
  - **Voraussetzung:** Eine Konfiguration muss vorhanden sein (`.rrduconfig` oder Action-Config). Fehlt diese, bricht `rrdu` mit einer Fehlermeldung und Exit-Code 1 ab.
  - **Exit-Codes:**
    - `0`: Alle Abhängigkeiten sind aktuell.
    - `1`: Veraltete Abhängigkeiten gefunden oder Fehler aufgetreten (perfekt für CI-Gates / Pull Request Checks).
- **`rrdu --run=full` (Automatisierter Update-Lauf):**
  - Führt Scan und Update vollautomatisch basierend auf der Einstellung `updater.auto-update` aus `.rrduconfig` (`none`, `*`, `*-force`) durch.
  - **Voraussetzung:** Eine Konfiguration muss vorhanden sein. Fehlt diese, bricht `rrdu` mit einer Fehlermeldung und Exit-Code 1 ab.
  - **Exit-Codes:**
    - `0`: Updates erfolgreich durchgeführt oder keine Updates notwendig.
    - `1`: Fehler beim Parsen, Schreiben oder Netzwerkfehler.

#### C. Scanning- und Visualisierungs-Verhalten
1. Basierend darauf, ob eine Konfiguration vorhanden ist, wird beim Start der CLI gescannt und nach dem Willkommens-Header informiert ('no updates found' oder 'updates found').
2. Wenn keine vorhanden ist, wird bei Projektauswahl gescannt.
3. Der Scan sowie das Update werden in der CLI optisch visualisiert (Ladeschnecke / Spinner via `indicatif`).

#### D. Selbst-Update (`rrdu self-update`)
- Der Befehl `rrdu self-update` aktualisiert das CLI-Tool automatisch:
  1. Fragt die offizielle GitHub Release API (`AnnoDomine/rust-recursive-deps-updater`) nach der neuesten Version ab.
  2. Vergleicht sie mit der aktuell laufenden Programmversion (`env!("CARGO_PKG_VERSION")`).
  3. Falls eine neuere Version vorliegt: Lädt das passende Release-Binary für das aktuelle OS (Linux/Windows) sicher herunter.
  4. Ersetzt die aktuelle Binärdatei im User-Space und gibt eine Erfolgsmeldung aus.
  5. Benötigt keine Root- oder Administrationsrechte.

#### E. Diagnose- & Bugreport-Generator (`rrdu report`)
- Der Befehl `rrdu report` generiert einen formatierten Markdown-Diagnosebericht zur direkten Übernahme in GitHub Issues:
  - **Datenschutz & Privacy-by-Design:**
    - Absolute System- und Benutzerpfade (z. B. `/home/<username>/...` oder `C:\Users\<username>\...`) werden vollständig maskiert und nur als relative Workspace-Pfade (`<workspace_root>/...`) ausgegeben.
    - Keine Offenlegung von privaten Benutzernamen, sensiblen Ordnerstrukturen oder Umgebungsvariablen.
  - **Enthaltene Diagnose-Daten:**
    - `rrdu`-Version und Build-Architektur.
    - Betriebssystem und Kernel-Plattform (z. B. `Linux x86_64`, `Windows 11 x86_64`).
    - Toolchain-Informationen (`rustc`/`cargo`-Version falls vorhanden).
    - Workspace-Metrik: Anzahl gefundener Projekte, Vorhandensein von `.rrduconfig` (Validierungsstatus).
    - Konnektivitätsprüfung zu `crates.io` (HTTPS-Handshake, Statuscode, Latenz).
  - Der Bericht kann vom Nutzer direkt per Copy & Paste in die GitHub Bug Report Vorlage eingefügt werden.

#### F. Hilfe & Befehlsübersicht (`rrdu help`)
- Der Befehl `rrdu help` (sowie `--help` und `-h`) gibt eine sauber formatierte Übersicht aller verfügbaren Befehle, Modi, Optionen und Beschreibungen im Terminal aus:
  - Übersicht über interaktiven Modus, `init`, `--run=scan`, `--run=full`, `self-update`, `report` und Konfigurationsoptionen.
  - Ermöglicht dem Nutzer schnelles Nachschlagen direkt in der Konsole ohne Blick in die Dokumentation.

### 2.5 Initialisierung (`rrdu init`)
- Der Befehl `rrdu init` dient der einfachen Erstkonfiguration.
- **Verhalten:** Wenn das Argument `init` übergeben wird, werden alle weiteren Argumente ignoriert.
- Der aktuelle Ordner sowie ALLE Unterordner werden rekursiv nach `Cargo.toml`-Dateien durchsucht (sowohl `[workspace]`-Mitglieder als auch eigenständige Crates).
- Es wird eine Vorlage für `.rrduconfig` mit dem `updater:`-Block generiert.
- Alle optionalen Optionen (`exclude`, `updater.exclude`, `auto-update`, `auto-scan`) werden auskommentiert inklusive erklärender Kommentare eingefügt.
- Existenzprüfung: Falls bereits eine `.rrduconfig` vorhanden ist, erfolgt eine Sicherheitsabfrage:
  `.rrduconfig found. Override [y/n] (default: n): `

### 2.6 Schonende Modifikation von `Cargo.toml`
- Verwendung von `toml_edit` anstelle von standardmäßigem `toml` Serde-Parsing, um:
  - Kommentare zu erhalten
  - Leerzeilen und Einrückungsstile beizubehalten
  - Inline-Tabellen (z. B. `{ version = "1.0", features = [...] }`) nicht zu zerstören.

---

## 3. Entwicklungs- & Qualitätsstandards

Um eine hohe Codequalität, Zuverlässigkeit, Sicherheit und Wartbarkeit von Beginn an zu garantieren, gelten für `rust-recursive-deps-updater` folgende Standards:

### 3.1 Sicherheit & Privilegien (Zero-Privilege / Sandboxed Mentality)
- **Keine Root-/Admin-Rechte erforderlich:** Die CLI agiert rein im User-Space. Sie benötigt und verlangt niemals erweiterte Systemrechte.
- **100 % Safe Rust (`#![forbid(unsafe_code)]`):** Das Projekt verbietet `unsafe`-Blöcke vollständig.
- **Path Traversal Schutz:** Alle Pfade (aus Konfiguration oder Benutzereingabe) werden validiert und kanonisiert. Es darf niemals aus dem deklarierten Workspace ausgebrochen werden (`../`-Angriffe abwehren).
  - Strikte Pfad-Validierung: Pfade dürfen ausschließlich auf denselben Ordner (`./`) oder Unterordner (z. B. `crates/...`) verweisen. Niemals Parent-Ordner (`../`) oder absolute Root-Pfade (`/`).
  - Unzulässige Pfade führen zu einem sauberen `ConfigError::InsecurePath` mit verständlicher Fehlermeldung und kontrolliertem Exit-Code 1 (kein Panic!).
- **Netzwerk-Sicherheit:**
  - Abfragen gegen crates.io erfolgen ausschließlich über HTTPS (TLS 1.2 / TLS 1.3).
  - Fester, normgerechter User-Agent gemäß crates.io Crawler-Richtlinie: `rust-recursive-deps-updater/<version> (<repository-url>)`.
  - Harte HTTP-Timeouts (z. B. 10 Sekunden) und Retry-Limits, um Hänger bei Netzwerkproblemen zu verhindern.

### 3.2 Rust Code-Qualität & Toolchain
- **Rust Edition:** Edition 2024.
- **Formatting:** `cargo fmt` ist verpflichtend. CI prüft mit `cargo fmt --check`.
- **Linter (Clippy):** 
  - CI prüft mit `cargo clippy --all-targets -- -D warnings`.
  - Ergänzend Aktivierung von sinnvollen Pedantic-Lints.
- **Error Handling (Keine Panics):**
  - Strikter Verzicht auf `unwrap()` und `expect()` im Produktivcode.
  - Fehler werden mit typsicheren Enums via `thiserror` abgebildet (`ConfigError`, `WorkspaceError`, `RegistryError`, `UpdateError`).
  - Anwenderfreundliche Fehlerausgabe in der Konsole ohne unleserliche Stacktraces (außer bei explizitem `RUST_BACKTRACE=1` oder `--verbose`).
  - Standardisierte Exit-Codes: `0` (Erfolg / Alles aktuell), `1` (Fehler oder veraltete Crates bei `--scan`), `2` (Benutzerabbruch).
- **Git Hooks (`.githooks/`):**
  - Aktivierung via `git config core.hooksPath .githooks` (keine Python- oder Node-Abhängigkeiten).
  - `pre-commit`: Führt automatisch `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` und `cargo test` aus.
  - `commit-msg`: Validiert, dass die Commit-Message strikt den Conventional Commits entspricht (`feat:`, `fix:`, `docs:`, etc.).

### 3.3 Dokumentations- & Open-Source-Governance-Standards
- **Rustdoc:** Alle öffentlichen Module, Structs, Enums, Traits und Funktionen werden mit normgerechten `///`-Dokumentationskommentaren auf Englisch versehen (inklusive `# Arguments`, `# Returns`, `# Errors` und Code-Beispielen wo sinnvoll).
- **Benutzerdokumentation:** Vollständige `README.md`, verständliche Fehlertexte und optionale GitHub Wiki-Seiten für erweiterte Konfigurationen.
- **Repository-Governance & Community-Standards:**
  - `LICENSE`: Volltext der **GNU General Public License v3.0** (`GPL-3.0-or-later`).
  - `SECURITY.md`: Sicherheitsrichtlinie, Meldeverfahren für Schwachstellen und unterstützte Versionen.
  - `CONTRIBUTING.md`: Leitfaden für Mitwirkende (Git-Workflow, Test-Voraussetzungen, PR-Ablauf).
  - `CODE_OF_CONDUCT.md`: Contributor Covenant Verhaltenskodex v2.1.
  - `AGENTS.md`: Leitfaden und Single Source of Truth für AI-Agenten (inkl. strikter Anti-Vibe-Coding & Anti-Credit-Crawling Policy: KI darf nur für `cargo check`, Docs oder max. 4-Zeilen-Integrationsanfragen ohne Kennzeichnung genutzt werden; größere Integrationen/Bugfixes müssen jede Zeile mit `"I am an AI and i am dumb"` abschließen).


### 3.4 Git, CI/CD & Marketplace-Standards
- **Conventional Commits:** Alle Commits folgen der Konvention (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `ci:`, `chore:`).
- **Semantic Versioning (SemVer):** Versionierung strikt nach `MAJOR.MINOR.PATCH`.
- **Semantic Pull Request Validation (`lint_pr.yml`):**
  - Automatisierte PR-Titel-Prüfung via `amannn/action-semantic-pull-request@v5` auf Conventional Commits.
- **CI/CD Pipeline (`ci.yml`):**
  - Automatische Checks bei jedem Push/PR (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`).
  - Automatisierter Sicherheits-Scan via `cargo-audit` (RustSec Advisory Database).
- **Multi-Platform Release Pipeline (`release.yml`):**
  - Matrix Builds für Linux (`x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`) und Windows (`x86_64-pc-windows-msvc`).
  - Automatische GitHub Releases mit vorkompilierten Binaries (`rrdu` bzw. `rrdu.exe`) bei Release-Tags.
  - Automatisches Publishing zu crates.io via GitHub Secret (`CARGO_REGISTRY_TOKEN`).
- **GitHub Marketplace Action (`action.yml`):**
  - Bereitstellung als offizielle GitHub Composite Action im Root-Verzeichnis (`action.yml`).
  - Ermöglicht Entwicklern weltweit die direkte Einbindung in ihre CI/CD-Pipelines (z. B. `uses: AnnoDomine/rust-recursive-deps-updater@v1` mit `run: 'scan'`).
  - Lädt automatisch das vorkompilierte Release-Binary für die jeweilige Runner-Plattform (Linux/Windows) herunter – ohne zeitraubende Cargo-Builds in Fremd-Pipelines.

### 3.5 Test- & Verifikationsstrategie
- **Unit Tests:**
  - SemVer-Einstufungslogik (Tests für Pre-1.0 und Post-1.0 Regeln).
  - YAML Config Deserialisierung und Defaults.
  - Glob- und Filter-Matching (Exclude-Regeln).
- **Integrationstests (Fixtures):**
  - Einsatz von `tempfile` für temporäre isolierte Workspace-Umgebungen.
  - End-to-End-Test von `toml_edit`: Sicherstellen, dass nach einem Update Kommentare, Einrückungen und Whitespaces 100 % erhalten bleiben.

---

## 4. Architektur- & Repository-Struktur

```
rust-recursive-deps-updater/
├── .github/
│   └── workflows/
│       ├── ci.yml               # cargo test, fmt, clippy, cargo-audit
│       ├── lint_pr.yml          # Semantic Pull Request Titel-Check
│       └── release.yml          # Multi-Platform Binary Build & Release (Linux / Windows)
├── .githooks/
│   ├── pre-commit               # Lokaler Guard: Formatierung, Linter, Tests, Audit
│   └── commit-msg               # Lokaler Guard: Strikte Conventional Commits Prüfung
├── src/
│   ├── main.rs                  # #![forbid(unsafe_code)], CLI-Einstieg & Error-Handling
│   ├── cli/                     # Terminal-UI, Argument-Parsing, Prompts, Spinner
│   │   ├── mod.rs
│   │   ├── prompt.rs            # Interaktive Abfragen (*, *-force, Menüauswahl)
│   │   ├── display.rs           # Formatierte Konsolenausgabe mit Farben/Status
│   │   ├── table.rs             # Zero-Dependency Tabellen-Renderer für --run=scan
│   │   └── pagination.rs        # Paginierung & Scroll-Logik (/next, /prev, Pfeiltasten)
│   ├── config/                  # Konfigurationsdateien (.rrduconfig) via noyalib
│   │   ├── mod.rs
│   │   └── model.rs             # RrduConfig, ProjectConfig, Exclude-Logik & Pfad-Validierung
│   ├── workspace/               # Cargo-Manifest & Workspace-Erkennung
│   │   ├── mod.rs
│   │   ├── discovery.rs         # Traversierung, Glob-Auflösung von members
│   │   └── project.rs           # Datenstrukturen für Projekte und deklarierte Abhängigkeiten
│   ├── registry/                # Versionsabfragen & SemVer
│   │   ├── mod.rs
│   │   ├── client.rs            # HTTP-Client (ureq mit rustls) & In-Memory Caching
│   │   └── semver_check.rs      # SemVer-Vergleich & Breaking-Change-Erkennung
│   ├── self_update/             # Selbst-Update & Release-Versionsabgleich
│   │   ├── mod.rs
│   │   └── client.rs            # GitHub Releases API Abfrage & Binär-Austausch
│   └── updater/                 # Manifest-Modifikation
│       ├── mod.rs
│       └── toml_writer.rs       # toml_edit basierte Aktualisierung von Versionsstrings
├── tests/                       # Dedizierte Integrationstests & Test-Fixtures
│   ├── config_tests.rs          # End-to-End Tests für YAML-Parsing & Validation
│   ├── updater_tests.rs         # End-to-End Tests für toml_edit Erhaltung
│   └── fixtures/                # Beispiel-Workspaces für Integrationsläufe
├── action.yml                   # GitHub Composite Action für GitHub Marketplace
├── .cursorrules                 # Cursor IDE Direktive (Single Source: AGENTS.md)
├── .windsurfrules               # Windsurf IDE Direktive (Single Source: AGENTS.md)
├── AGENTS.md                    # Dokumentation & Richtlinien für AI-Agenten (Single Source of Truth)
├── CLAUDE.md                    # Claude Code Direktive (Single Source: AGENTS.md)
├── Cargo.toml                   # Metadaten, GPL-3.0-or-later, Linter-Konfiguration
├── CODE_OF_CONDUCT.md           # Contributor Covenant Verhaltenskodex v2.1
├── CONTRIBUTING.md               # Richtlinien für Beiträge & Git-Workflow
├── GEMINI.md                    # Gemini CLI Direktive (Single Source: AGENTS.md)
├── LICENSE                      # Offizieller GNU General Public License v3.0 Volltext
├── PLANING.md                   # Zentrales Architektur- & Planungsdokument
├── README.md                    # Offizielles Benutzer-Handbuch
└── SECURITY.md                  # Sicherheitsrichtlinie & Reporting-Verfahren
```

**Test-Organisation (Rust-Standard):**
- **Unit-Tests:** Werden idiomatisch in `src/` direkt am Code platziert (`#[cfg(test)] mod tests`), um auch private Hilfsfunktionen isoliert testen zu können.
- **Integrationstests:** Dedizierter Top-Level-Ordner `tests/` für End-to-End-Szenarien, Dummy-Workspaces (Fixtures) und Modifikations-Tests mit `tempfile`.

---

## 5. Geplante Crate-Abhängigkeiten

| Crate | Zweck | Begründung | Dev Note |
|---|---|---|--|
| `clap` | CLI-Argumente | Industriestandard für Rust-CLIs (für `init` und `--run=scan\|full`) | Klares interaktives CLI ohne dynamische Injektions-Flags |
| `serde`, `noyalib` | Config-Parsing | Deserialisierung von `.rrduconfig` im YAML-Format | Modernes, sicheres Pure-Rust YAML 1.2 mit voller Serde-Unterstützung und `#![forbid(unsafe_code)]` |
| `toml_edit` | Manifest-Bearbeitung | Erhält Formatierung, Kommentare und Struktur in `Cargo.toml` | Referenzbibliothek des Cargo-Teams |
| `semver` | SemVer-Logik | Sichere Prüfung von Versionen und Breaking-Change-Klassifizierung | Standardparser aus dem Rust-Core |
| `ureq` | HTTP-Client | Abfrage der crates.io API | Schlankes synchrones Binary, `#![forbid(unsafe_code)]`, memory-safe mit `rustls` |
| `colored` / `owo-colors` | Terminal-Design | Farbliche Kennzeichnung im Terminal (grün, gelb, rot) | |
| `indicatif` | Spinner / Progress | Ladeschnecke & Fortschrittsbalken für Scan- und Update-Vorgänge | |
| `dialoguer` oder `inquire` | Interaktion | Ermöglicht elegante interaktive Eingaben und Menüs | |
| `glob` / `walkdir` | Dateisuche | Suche nach verschachtelten Cargo.toml und Member-Auflösung | |
| `thiserror` | Error Handling | Typsichere, saubere Fehlerkataloge ohne Panics | |
| *(keine externe Crate)* | Tabellen-Formatierung | 5-Spalten-Tabelle für `--run=scan` wird nativ in `src/cli/table.rs` umgesetzt | **Zero-Dependency**, 0 Angriffsfläche, kein Wartungsrisiko |

---

## 6. Implementierungsphasen
- [ ] **Phase 0: Vorbereitung, Governance, CI/CD & Marketplace-Setup**
  - Öffentliches GitHub Repository sauber aufsetzen.
  - Vollständige Lizenzierung mit **GNU General Public License v3.0** (`LICENSE`).
  - Community- & Governance-Dateien anlegen: `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CONTRIBUTING.md`.
  - Workflow `lint_pr.yml` mit `amannn/action-semantic-pull-request@v5` für PR-Titel.
  - Workflow `ci.yml` für automatische Tests, Linter (`fmt`, `clippy`) und Sicherheits-Audits (`cargo-audit`).
  - Workflow `release.yml` für Multi-Platform Matrix Builds (Linux/Windows) und crates.io Veröffentlichung.
  - Erstellung der `action.yml` für die Distribution über den GitHub Actions Marketplace.
  - Lokale Git-Hooks in `.githooks/` (`pre-commit` und `commit-msg`) einrichten.
  - Dokumentation aller manuellen Nutzerschnittstellen (crates.io Token, GitHub Secrets `CARGO_REGISTRY_TOKEN`).
- [ ] **Phase 1: Konfigurationsschicht & `init`**
  - Implementierung des Datenmodells für `.rrduconfig` mit `workspace`- und `updater`-Blöcken via `noyalib`.
  - Strikte Pfad-Validierung gegen Path-Traversal (`ConfigError::InsecurePath`).
  - `rrdu init` Generator mit Sicherheitswarnung bei existierender Konfiguration.
- [ ] **Phase 2: Workspace-Discovery (`workspace`)**
  - Auffinden von Root-`Cargo.toml` und Auflösen von `[workspace.members]`.
  - Rekursiver Fallback-Scan bei Workspace-losen Verzeichnisbäumen.
  - Parsen von `dependencies`, `dev-dependencies`, `build-dependencies`.
  - Filtern: Nur direkte crates.io Abhängigkeiten (Ignorieren von `git = "..."` und `path = "..."`).
  - Berücksichtigung von Exclude-Listen (`workspace.exclude` und `updater.exclude`).
- [ ] **Phase 3: Registry-Client & SemVer-Analyzer (`registry`)**
  - Anbindung an crates.io API via `ureq` mit User-Agent und In-Memory-Cache.
  - Exakte SemVer-Einstufung (Pre-1.0 vs. Post-1.0).
- [ ] **Phase 4: Interaktive CLI & Headless-Modi (`cli`)**
  - Anzeige des Begrüßungsbildschirms, Projektübersicht und Spinner (`indicatif`).
  - Eingabeauswertung für Projektauswahl und Befehle (`*`, `*-force`, Crate-Name, `/back`, `/exit`).
  - Headless-Modus `rrdu --run=scan` mit nativer 5-Spalten-Tabelle.
  - Headless-Modus `rrdu --run=full` für automatisiertes Update.
- [ ] **Phase 5: Formatierungserhaltender Updater (`updater`)**
  - Aktualisieren der Versionsangaben in `Cargo.toml` mit `toml_edit`.
  - Unterstützung für einfache String-Versionen (`dep = "1.0"`) und Inline-Tabellen (`dep = { version = "1.0", ... }`).
  - Unterstützung für `[workspace.dependencies]`.
- [ ] **Phase 6: Verifikation & Tests**
  - Unit-Tests für SemVer-Logik, Exclude-Regeln und Config-Parsing.
  - Integrationstests an echten Dummy-Workspaces in `tests/fixtures/`.
  - Verifikation des Erhalts von Kommentaren und Formatierungen.
  - Finale Dokumentationsprüfung (`README.md`, `AGENTS.md`, rustdoc).
