//! Self-update controller and terminal user interface.
//!
//! Provides interactive notification and menu rendering for updating `rrdu`.

use std::{
    io::{self, Write},
    process::exit,
};

use comfy_table::{ContentArrangement, InlineTable, Table, presets::UTF8_FULL};
use dialoguer::console::{Key, Term};

use crate::{
    constants::TERMINAL_MAX_WIDTH,
    errors::RrduConfigResult,
    functions::wait_for_enter,
    logger::logger::trace,
    self_updater::engine::{RrduReleaseStatus, SelfUpdateEngine},
};

/// Controller managing self-update notification and interactive update screens.
#[derive(Default, Debug)]
pub struct SelfUpdateController {
    /// Underlying self-update engine managing remote version checks.
    pub engine: Option<SelfUpdateEngine>,
}

impl SelfUpdateController {
    /// Initializes a new `SelfUpdateController` with version information.
    ///
    /// # Returns
    /// An initialized `SelfUpdateController` instance.
    pub fn new() -> Self {
        let engine = match SelfUpdateEngine::new() {
            Ok(eng) => {
                // println!("Engine loaded successfully: {:#?}", eng);
                Some(eng)
            }
            Err(err) => {
                println!("Engine failed with error: {:#?}", err);
                None
            }
        };
        Self { engine }
    }

    fn render_git_hub_table(&self, table: &mut Table) {
        if let Some(engine) = &self.engine
            && let Some(status) = &engine.releases.gh
        {
            let mut git_hub_inline = InlineTable::new();
            let header_outdated = format!(
                "RRDU is {}outdated",
                if engine.git_hub { "" } else { "not " }
            );
            git_hub_inline.set_header(["GitHub", header_outdated.as_str()]);

            if let RrduReleaseStatus::Updated(release) = status {
                git_hub_inline.add_row(["Name", &release.name]);
                git_hub_inline.add_row(["Version", &release.version]);
                git_hub_inline.add_row(["Date", &release.date]);
                if let Some(body) = &release.body {
                    git_hub_inline.add_row(["Body", body]);
                }
                if let Some(release_notes_url) = &release.release_notes_url {
                    git_hub_inline.add_row(["Note URL", release_notes_url]);
                }

                if !release.assets.is_empty() {
                    let total = release.assets.len();
                    git_hub_inline.add_row(["Assets", format!("Total: {total}").as_str()]);
                    for asset in &release.assets {
                        let mut asset_inline = InlineTable::new();
                        asset_inline.set_header(["Field", "Value"]);
                        asset_inline.add_row(["Name", &asset.name]);
                        asset_inline.add_row(["URL", &asset.download_url]);
                        asset_inline
                            .add_row(["Digits", asset.digest.as_ref().map_or_default(|_| "None")]);
                        git_hub_inline.add_inline_table(asset_inline);
                    }
                }
            }

            table.add_inline_table(git_hub_inline);
        }
    }

    fn render_crate_io_table(&self, table: &mut Table) {
        if let Some(engine) = &self.engine
            && let Some(release) = &engine.releases.ci
        {
            let mut crates_io_inline = InlineTable::new();
            let header_outdated = format!(
                "RRDU is {}outdated",
                if engine.crates_io { "" } else { "not " }
            );
            crates_io_inline.set_header(["crates.io", header_outdated.as_str()]);
            crates_io_inline.add_row(["Name", &release.name]);
            crates_io_inline.add_row(["Version", &release.default_version]);
            crates_io_inline.add_row(["Date", &release.updated_at]);

            if let Some(value) = &release.description {
                crates_io_inline.add_row(["Description", value]);
            }

            if let Some(value) = &release.homepage {
                crates_io_inline.add_row(["Homepage Link", value]);
            }

            if let Some(value) = &release.documentation {
                crates_io_inline.add_row(["Documentation Link", value]);
            }

            if let Some(value) = &release.repository {
                crates_io_inline.add_row(["Repositiory Link", value]);
            }

            table.add_inline_table(crates_io_inline);
        }
    }

    fn render_information(&self) {
        let mut table = Table::new();

        table
            .force_no_tty()
            .set_width(TERMINAL_MAX_WIDTH)
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .load_style(UTF8_FULL)
            .set_header(["RRDU self-updater"]);

        self.render_crate_io_table(&mut table);
        self.render_git_hub_table(&mut table);

        trace().print_and_save_log(log::LevelFilter::Trace, &table.to_string());
    }

    /// Renders an in-terminal notification banner in interactive mode when an update is available.
    pub fn render_interactive(&self) {
        if let Some(engine) = &self.engine
            && (engine.git_hub || engine.crates_io)
        {
            self.render_information();
            trace().print_and_save_log(log::LevelFilter::Trace, "Your version of rrdu is outdated. Please run 'rrdu --self-update' for update to latest version!");
            trace().print_and_save_log(log::LevelFilter::Trace, "Press 'Enter' to continue!");

            // Await user press return to continue
            let _ = wait_for_enter();
        }
    }

    fn render_self_update_border_filling(
        &self,
        chars: &str,
        pref: &str,
        suff: &str,
        value: Option<&str>,
    ) -> String {
        let width: usize = 100;
        let val = value.map_or_else(|| "".to_string(), |v| format!(" {v} "));
        let filling = if chars.is_empty() { " " } else { chars };
        format!(
            "{pref}{val}{}{suff}",
            filling.to_string().repeat(width - val.len())
        )
    }

    fn leave_alternate_screen() -> RrduConfigResult<()> {
        let term = Term::stdout();
        term.show_cursor()?;
        print!("\x1b[?1049l");
        io::stdout().flush()?;
        Ok(())
    }

    /// Renders the interactive full-screen self-update selection menu.
    ///
    /// # Returns
    /// `Ok(())` upon successful execution, exit, or cancellation.
    ///
    /// # Errors
    /// Returns [`RrduConfigResult::Err`] if terminal operations or updates fail.
    pub async fn render_self_update(&self) -> RrduConfigResult<()> {
        if let Some(engine) = &self.engine
            && (engine.git_hub || engine.crates_io)
        {
            let mut engine_clone = engine.clone();

            loop {
                let terminal = Term::stdout();
                terminal.clear_screen()?;
                // HEADER
                terminal
                    .write_line(&self.render_self_update_border_filling("═", "╔", "╗", None))?;
                terminal.write_line(&self.render_self_update_border_filling(
                    " ",
                    "║",
                    "║",
                    Some("RUST-RECURSIVE-DEPS-UPDATER SELF-UPDATER"),
                ))?;
                terminal
                    .write_line(&self.render_self_update_border_filling("═", "╠", "╣", None))?;

                // PROVIDER
                terminal.write_line(&self.render_self_update_border_filling(
                    " ",
                    "║",
                    "║",
                    Some("We found a new version provided by:"),
                ))?;
                terminal
                    .write_line(&self.render_self_update_border_filling(" ", "║", "║", None))?;
                if engine.git_hub {
                    terminal.write_line(&self.render_self_update_border_filling(
                        " ",
                        "║",
                        "║",
                        Some("GitHub (via self_update crate)"),
                    ))?;
                }
                if engine.crates_io {
                    terminal.write_line(&self.render_self_update_border_filling(
                        " ",
                        "║",
                        "║",
                        Some("crates.io (via cargo)"),
                    ))?;
                }
                terminal
                    .write_line(&self.render_self_update_border_filling(" ", "║", "║", None))?;

                // Inputs
                if engine.git_hub {
                    terminal.write_line(&self.render_self_update_border_filling(
                        " ",
                        "║",
                        "║",
                        Some("Press 'g' to update via GitHub"),
                    ))?;
                }
                if engine.crates_io {
                    terminal.write_line(&self.render_self_update_border_filling(
                        " ",
                        "║",
                        "║",
                        Some("Press 'c' to update via cargo from crates.io"),
                    ))?;
                }
                terminal.write_line(&self.render_self_update_border_filling(
                    " ",
                    "║",
                    "║",
                    Some("Press 'q' to abort"),
                ))?;
                terminal
                    .write_line(&self.render_self_update_border_filling("═", "╚", "╝", None))?;

                let key = terminal.read_key()?;

                match key {
                    Key::Char('g') => {
                        if engine.git_hub {
                            engine_clone.update_with_github()?;
                            exit(0);
                        }
                    }
                    Key::Char('c') => {
                        if engine.crates_io {
                            engine_clone.update_with_cargo()?;
                            exit(0);
                        }
                    }
                    // Clean quit
                    Key::CtrlC => {
                        Self::leave_alternate_screen()?;
                        exit(2);
                    }
                    // Clean exit
                    Key::Escape | Key::Char('q') => {
                        Self::leave_alternate_screen()?;
                        exit(0);
                    }
                    _ => {}
                }
            }
        }
        Self::leave_alternate_screen()?;
        Ok(())
    }
}
