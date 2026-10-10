use std::{collections::HashMap, sync::Arc};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::meta_status;

/// Manages interactive progress bars and multi-progress displays using `indicatif`.
#[derive(Clone)]
pub struct Progress {
    is_headless: bool,
    bar_wrapper: Arc<MultiProgress>,
    bars: HashMap<String, ProgressBar>,
    sty: ProgressStyle,
}

impl Progress {
    /// Initializes a new progress tracker.
    ///
    /// # Arguments
    /// * `is_headless` - Flag indicating whether the application runs in headless CI mode.
    ///   When `true`, all progress rendering is safely disabled.
    pub fn new(is_headless: bool) -> Self {
        let style: ProgressStyle = match ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:100.cyan/blue} {pos:>2}/{len:2} {msg}",
        ) {
            Ok(sty) => sty.progress_chars("█▓░"),
            Err(e) => {
                if !is_headless {
                    meta_status!(
                        log::LevelFilter::Error,
                        crate::status_codes::Module::INTERACTIVITY,
                        500,
                        "An error occured while style the progress bars.",
                        e
                    );
                }
                ProgressStyle::default_bar()
            }
        };
        Self {
            is_headless,
            bar_wrapper: Arc::new(MultiProgress::new()),
            bars: HashMap::new(),
            sty: style,
        }
    }

    /// Adds multiple progress bars concurrently.
    ///
    /// # Arguments
    /// * `ids` - Identifiers for each progress bar.
    /// * `steps` - Total number of tick steps for each bar.
    /// * `initial_state` - Initial status description text.
    pub fn add_multi(&mut self, ids: Vec<String>, steps: u64, initial_state: &str) {
        if !self.is_headless {
            for id in ids {
                self.add(&id, steps, initial_state, None);
            }
        }
    }

    /// Sets a shared progress style across multiple bars.
    ///
    /// # Arguments
    /// * `ids` - Target bar identifiers.
    /// * `style` - The `indicatif` progress style to apply.
    pub fn set_multi_style(&mut self, ids: Vec<String>, style: ProgressStyle) {
        if !self.is_headless {
            for id in ids {
                self.set_style(&id, style.clone());
            }
        }
    }

    /// Registers a single progress bar within the multi-progress display.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the bar.
    /// * `steps` - Total number of tick steps.
    /// * `initial_state` - Initial status label.
    /// * `after_id` - Optional identifier of an existing bar after which this one should be inserted.
    pub fn add(&mut self, id: &String, steps: u64, initial_state: &str, after_id: Option<String>) {
        if !self.is_headless {
            let pb = ProgressBar::new(steps);
            let bar = if let Some(below) = after_id
                && let Some(p_b) = self.bars.get(&below)
            {
                self.bar_wrapper.insert_after(p_b, pb)
            } else {
                self.bar_wrapper.add(pb)
            };
            bar.set_style(self.sty.clone());
            bar.set_prefix(id.clone());
            bar.set_message(format!("{id}: {initial_state}"));
            self.bars.insert(id.clone(), bar);
        }
    }

    /// Increments a progress bar by one step and updates its message.
    ///
    /// # Arguments
    /// * `id` - Bar identifier.
    /// * `new_state` - Updated message string.
    pub fn increment_bar(&mut self, id: &str, new_state: &str) {
        if !self.is_headless
            && let Some(bar) = self.bars.get_mut(id)
        {
            bar.inc(1);
            bar.set_message(format!("{id}: {new_state}"));
        }
    }

    /// Increments a progress bar while appending the new state to the existing message.
    ///
    /// # Arguments
    /// * `id` - Bar identifier.
    /// * `next_state` - State string to append.
    pub fn increment_bar_with_assigned_state(&mut self, id: &str, next_state: &str) {
        if !self.is_headless
            && let Some(bar) = self.bars.get_mut(id)
        {
            let current_state = bar.message();
            self.increment_bar(id, format!("{current_state}, {next_state}").as_str());
        }
    }

    /// Sets the display style for a specific progress bar.
    ///
    /// # Arguments
    /// * `id` - Bar identifier.
    /// * `style` - The `ProgressStyle` to assign.
    pub fn set_style(&mut self, id: &str, style: ProgressStyle) {
        if !self.is_headless
            && let Some(bar) = self.bars.get_mut(id)
        {
            bar.set_style(style);
        }
    }

    /// Updates the message displayed on a specific progress bar.
    ///
    /// # Arguments
    /// * `id` - Bar identifier.
    /// * `msg` - New message string.
    pub fn set_bar_message(&mut self, id: &str, msg: &str) {
        if !self.is_headless
            && let Some(bar) = self.bars.get_mut(id)
        {
            bar.set_message(format!("{id}: {msg}"));
        }
    }

    /// Marks a progress bar as finished and displays a completion message.
    ///
    /// # Arguments
    /// * `id` - Bar identifier.
    pub fn finish_bar(&mut self, id: &str) {
        if !self.is_headless
            && let Some(bar) = self.bars.get_mut(id)
        {
            bar.finish_with_message(format!("{id}: Done!"));
        }
    }

    /// Finishes and removes a progress bar from the active terminal view.
    ///
    /// # Arguments
    /// * `id` - Bar identifier.
    pub fn remove(&mut self, id: &str) {
        if !self.is_headless
            && let Some(bar) = self.bars.get_mut(id)
        {
            bar.finish_and_clear();
        }
    }

    /// Clears and removes all progress bars from the terminal display.
    pub fn clean(&mut self) {
        if !self.is_headless {
            let _ = self.bar_wrapper.clear();
        }
    }
}
