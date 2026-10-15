use std::{
    io::{self, Write},
    process,
};

use dialoguer::console::{Key, Term};
use semver::Version;
use time::error::Format;

use crate::{
    constants::*,
    enums::{DependencyCollectionVersion, DependencyEntry, RegistryClientState},
    logger::status_codes::Module,
    registry::{
        client::Client,
        crates_index_response_structs::CratesIndexItem,
        crates_io_response_structs::{CratesIOResponse, Version as IOVersion},
        rust_sec_json_response_structs::{Affected, RustsecItem},
    },
    simple_status,
    updater::engine::UpdaterInformation,
};

/// Represents the outcome of an interactive terminal navigation or selection action.
pub enum TerminalReturn {
    /// Navigate back to the previous view level.
    PrevView,
    /// An item was selected by index.
    SelectItem(usize),
    /// An update command was invoked for the item at the specified index.
    Update(Option<UpdaterInformation>),
}

/// Identifies supported keyboard and navigation actions available in a terminal view.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SpecialInputs {
    /// Navigate to the previous page. (In dependecy view, scroll one page up)
    PrevPage,
    /// Navigate to the next page. (In dependecy view, scroll one page down)
    NextPage,
    /// Move cursor to the previous item. (In dependecy view, scroll one line up)
    PrevItem,
    /// Move cursor to the next item. (In dependecy view, scroll one line down)
    NextItem,
    /// Only available in dependency view. Select prev Tab.
    TabLeft,
    /// Only available in dependency view. Select next Tab.
    TabRight,
    /// Select the currently highlighted item.
    Select,
    /// Trigger an update on the highlighted item.
    Updateable,
    /// View identificator, if update is available in view
    Update,
    /// Return to the previous menu level.
    ViewBack,
}

/// Stores pre-rendered diagnostic and registry information lines separated by tab categories.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ClientInformationLines {
    index: Vec<String>,
    api: Vec<String>,
    audit: Vec<String>,
}

/// Interactive terminal interface manager providing paginated navigation and key handling.
pub struct Terminal {
    /// Navigation breadcrumbs path displayed at the top of the terminal.
    pub menu_path: Vec<String>,
    /// Formatted keyboard navigation hint lines rendered at the bottom.
    pub input_lines: Vec<String>,
    /// Contextual instruction text displayed beneath the breadcrumb path.
    pub instruction_line: String,
    /// Set of interactive actions enabled for the current view.
    pub available_inputs: Vec<SpecialInputs>,
    /// Maximum number of list items rendered per page.
    pub max_items: usize,
}

impl Terminal {
    /// Width allocation for fixed-size status columns in pagination headers.
    pub const STATIC_ELEMENT_SIZE: usize = 30;

    /// Initializes a new interactive terminal controller.
    ///
    /// # Arguments
    /// * `max_items` - Maximum items visible per page.
    pub fn new(max_items: usize) -> Self {
        Self {
            menu_path: Vec::new(),
            available_inputs: Vec::new(),
            instruction_line: String::new(),
            input_lines: Vec::new(),
            max_items,
        }
    }

    /// Sets the list of formatted keybinding instruction lines.
    ///
    /// # Arguments
    /// * `input_lines` - Lines describing available keyboard shortcuts.
    pub fn set_input_lines(&mut self, input_lines: Vec<String>) {
        self.input_lines = input_lines;
    }

    /// Generates default navigation and operation hints (Backspace, Update, Select, Esc).
    ///
    /// # Arguments
    /// * `available` - List of allowed special input actions.
    ///
    /// # Returns
    /// Vector of formatted shortcut hint strings.
    fn generate_default_input_lines(&mut self, available: Vec<SpecialInputs>) -> Vec<String> {
        let mut special_selection_line: Vec<String> = Vec::new();
        if available.contains(&SpecialInputs::ViewBack) {
            special_selection_line.push("Backspace = Prev View".to_string());
        }
        if available.contains(&SpecialInputs::Update)
            && available.contains(&SpecialInputs::Updateable)
        {
            special_selection_line.push("Key 'u' = Update".to_string());
        }
        if available.contains(&SpecialInputs::Select) {
            special_selection_line.push("Enter/Return = Select".to_string());
        }
        special_selection_line.push("Esc = Exit".to_string());
        special_selection_line
    }

    /// Generates keybinding instruction rows based on enabled navigation actions.
    ///
    /// # Arguments
    /// * `available` - Slice of enabled special input actions.
    pub fn generate_input_lines(&mut self, available: Vec<SpecialInputs>) {
        let mut input_lines: Vec<String> = Vec::new();
        let mut navigation_page_prev_line: Vec<String> = Vec::new();
        let mut navigation_page_next_line: Vec<String> = Vec::new();
        if available.contains(&SpecialInputs::PrevItem) {
            navigation_page_prev_line.push(String::from("↑ = Prev Item"));
        }
        if available.contains(&SpecialInputs::NextItem) {
            navigation_page_next_line.push(String::from("↓ = Next Item"));
        }
        if available.contains(&SpecialInputs::PrevPage) {
            navigation_page_prev_line.push(String::from("Page ↑ = Scroll to prev page"));
            navigation_page_prev_line.push(String::from("Pos 1 = Scroll to beginn"));
        }
        if available.contains(&SpecialInputs::NextPage) {
            navigation_page_next_line.push(String::from("Page ↓ = Scroll to next page"));
            navigation_page_next_line.push(String::from("End = Scroll to end"));
        }
        input_lines.push(navigation_page_prev_line.join(" | "));
        input_lines.push(navigation_page_next_line.join(" | "));
        input_lines.push(self.generate_default_input_lines(available).join(" | "));
        self.set_input_lines(input_lines);
    }

    /// Generates keybinding instruction rows for the dependency detail view with tab navigation and scrolling.
    ///
    /// # Arguments
    /// * `available` - List of enabled special input actions.
    pub fn generate_dependency_input_lines(&mut self, available: Vec<SpecialInputs>) {
        let mut input_lines: Vec<String> = Vec::new();
        let mut navigation_tab_line: Vec<String> = Vec::new();
        let mut navigation_page_prev_line: Vec<String> = Vec::new();
        let mut navigation_page_next_line: Vec<String> = Vec::new();
        if available.contains(&SpecialInputs::PrevItem) {
            navigation_page_prev_line.push(String::from("↑ = Scroll up"));
        }
        if available.contains(&SpecialInputs::NextItem) {
            navigation_page_next_line.push(String::from("↓ = Scroll down"));
        }
        if available.contains(&SpecialInputs::PrevPage) {
            navigation_page_prev_line.push(String::from("Page ↑ = Scroll to prev page"));
            navigation_page_prev_line.push(String::from("Pos 1 = Scroll to beginn"));
        }
        if available.contains(&SpecialInputs::NextPage) {
            navigation_page_next_line.push(String::from("Page ↓ = Scroll to next page"));
            navigation_page_next_line.push(String::from("End = Scroll to end"));
        }
        if available.contains(&SpecialInputs::TabLeft) {
            navigation_tab_line.push(String::from("← = Prev Tab"));
        }
        if available.contains(&SpecialInputs::TabRight) {
            navigation_tab_line.push(String::from("→ = Next Tab"));
        }
        input_lines.push(navigation_tab_line.join(" | "));
        input_lines.push(navigation_page_prev_line.join(" | "));
        input_lines.push(navigation_page_next_line.join(" | "));
        input_lines.push(self.generate_default_input_lines(available).join(" | "));
        self.set_input_lines(input_lines);
    }

    /// Configures the active interactive input capabilities for the current view.
    ///
    /// # Arguments
    /// * `available` - List of allowed `SpecialInputs`.
    pub fn set_available_input(&mut self, available: Vec<SpecialInputs>) {
        self.available_inputs = available;
    }

    /// Sets the contextual instruction text displayed for this menu view.
    ///
    /// # Arguments
    /// * `instruction` - Instructional message text.
    pub fn set_instruction(&mut self, instruction: String) {
        self.instruction_line = instruction;
    }

    /// Appends a segment to the navigation breadcrumbs path.
    ///
    /// # Arguments
    /// * `path` - Breadcrumb segment label.
    pub fn add_path(&mut self, path: &str) {
        self.menu_path.push(path.to_string());
    }

    /// Replaces the current navigation path with the provided breadcrumb segments.
    ///
    /// # Arguments
    /// * `paths` - Sequence of navigation level labels.
    pub fn set_path(&mut self, paths: Vec<String>) {
        let mut menu_paths = vec!["RRDU Interactive Terminal".to_string()];
        if !paths.is_empty() {
            menu_paths.append(&mut paths.clone());
        }
        self.menu_path = menu_paths;
    }

    /// Removes the first matching breadcrumb segment from the navigation path.
    ///
    /// # Arguments
    /// * `path` - Breadcrumb label to remove.
    pub fn remove_path(&mut self, path: &str) {
        if let Some(path_idx) = self.menu_path.iter().position(|p| p == path) {
            self.menu_path.remove(path_idx);
        };
    }

    /// Renders a formatted selection row for a given indexed item.
    ///
    /// # Arguments
    /// * `item` - Tuple of 1-based display index, label string, and updateable status.
    /// * `highlighted` - Currently selected 0-based index.
    ///
    /// # Returns
    /// The formatted terminal row string.
    pub fn get_selection_line(
        &self,
        item: &(usize, String, Option<UpdaterInformation>),
        highlighted: usize,
    ) -> String {
        let (prefix, suffix) = if item.0 == highlighted + 1 {
            (format!("|-> {:} ", item.0), " <-|")
        } else {
            (format!("|   {:} ", item.0), "   |")
        };
        let label = format!(" {:}", item.1);
        let fill_amount = (TERMINAL_MAX_WIDTH as usize).saturating_sub(prefix.len() + suffix.len());
        format!("{prefix}{label:.>fill_amount$}{suffix}")
    }

    /// Checks whether a specific navigation action is enabled in the current view.
    ///
    /// # Arguments
    /// * `input` - The `SpecialInputs` action to test.
    ///
    /// # Returns
    /// `true` if the action is enabled.
    pub fn is_input_included(&self, input: SpecialInputs) -> bool {
        self.available_inputs.contains(&input)
    }

    /// Computes enabled pagination and navigation actions for the current page slice and updates instruction lines.
    ///
    /// # Arguments
    /// * `first` - Index of the first visible item on the page.
    /// * `last` - Index of the last visible item on the page.
    /// * `current_pos` - Index of the currently highlighted item.
    /// * `items_len` - Total count of available items.
    /// * `is_highlighted_updateable` - Whether the currently highlighted item can be updated.
    pub fn prepare_paginated_input_line(
        &mut self,
        first: usize,
        last: usize,
        current_pos: usize,
        items_len: usize,
        is_highlighted_updateable: bool,
    ) {
        let mut available = self.available_inputs.clone();
        if current_pos > 0 {
            available.push(SpecialInputs::PrevItem);
        };
        if current_pos < items_len - 1 {
            available.push(SpecialInputs::NextItem);
        };
        if first > 0 {
            available.push(SpecialInputs::PrevPage);
        };
        if last < items_len {
            available.push(SpecialInputs::NextPage);
        };
        if is_highlighted_updateable {
            available.push(SpecialInputs::Updateable);
        }
        self.generate_input_lines(available);
    }

    /// Generates a horizontal separator line spanning the maximum terminal width.
    ///
    /// # Returns
    /// Separator line string.
    fn get_terminal_section_seperator(&self) -> String {
        "-".repeat(TERMINAL_MAX_WIDTH.into()).to_string()
    }

    /// Generates the pagination status bar line showing currently highlighted item and page range.
    ///
    /// # Arguments
    /// * `highlighted` - Zero-based index of the currently highlighted item.
    /// * `first_item` - Zero-based index of the first visible item on this page.
    /// * `last_item` - Zero-based index of the last visible item on this page.
    /// * `items_len` - Total number of available items.
    ///
    /// # Returns
    /// Formatted pagination bar line fitting the terminal width.
    fn generate_pagination_line(
        &self,
        highlighted: usize,
        first_item: usize,
        last_item: usize,
        items_len: usize,
    ) -> String {
        let selected = format!(" {:}", highlighted + 1);
        let items = format!(" {:}-{:} / {:}", first_item + 1, last_item, items_len);
        let selected_label = "Highlighted: ";
        let items_label = "Items: ";
        let selected_fill_size = Self::STATIC_ELEMENT_SIZE.saturating_sub(selected_label.len());
        let items_fill_size = Self::STATIC_ELEMENT_SIZE.saturating_sub(items_label.len());
        let selected_element = format!("| {selected_label}{selected:.>selected_fill_size$}");
        let items_element = format!("{items_label}{items:.>items_fill_size$} |");
        let fill_amount = (TERMINAL_MAX_WIDTH as usize).saturating_sub(selected_element.len());
        format!("{selected_element}{items_element:>fill_amount$}")
    }

    /// Generates an empty spacer line bounded by terminal vertical borders.
    ///
    /// # Returns
    /// Formatted empty row string.
    fn generate_item_fill_line(&self) -> String {
        let fill_amount = (TERMINAL_MAX_WIDTH as usize).saturating_sub(1);
        format!("|{:>fill_amount$}", "|")
    }

    /// Wraps a text line with standard terminal outer borders and padding.
    ///
    /// # Arguments
    /// * `value` - String slice to render inside borders.
    fn generate_dependency_single_line(&self, value: &str) -> String {
        let fill_amount = (TERMINAL_MAX_WIDTH as usize).saturating_sub(4);
        format!("| {value:<fill_amount$} |")
    }

    /// Leaves the alternate screen buffer, unhides the cursor, and flushes stdout.
    pub fn leave_alternate_screen() -> io::Result<()> {
        let term = Term::stdout();
        term.show_cursor()?;
        print!("\x1b[?1049l");
        io::stdout().flush()?;
        Ok(())
    }

    /// Checks if the currently highlighted list item is updateable.
    ///
    /// # Arguments
    /// * `highlighted` - Zero-based index of highlighted item.
    /// * `items` - Slices of available items with updateability flag.
    fn is_updateable_highlighted(
        &self,
        highlighted: &usize,
        items: &[(usize, String, Option<UpdaterInformation>)],
    ) -> bool {
        items
            .iter()
            .find(|(idx, _, updateable)| *idx == (highlighted + 1) && updateable.is_some())
            .is_some()
    }

    /// Formats a key-value pair into a dot-padded label-value string.
    ///
    /// # Arguments
    /// * `key` - Property name.
    /// * `value` - Property value.
    fn generate_dependency_information_line(&self, key: &str, value: &str) -> String {
        let key_string = format!("{:} ", key);
        let value_string = format!(" {:}", value);
        format!("{key_string:.<20}{value_string:.>20}")
    }

    /// Generates metadata rows for the installed local dependency.
    ///
    /// # Arguments
    /// * `item` - The local dependency entry.
    fn generate_entry_inforamtion(&self, item: DependencyEntry) -> Vec<String> {
        vec![
            self.generate_dependency_information_line("Dependency:", item.package()),
            self.generate_dependency_information_line(
                "Current version:",
                &item.version().to_string(),
            ),
        ]
    }

    /// Generates summary rows for the fetched latest version status.
    ///
    /// # Arguments
    /// * `latest` - The fetched latest version status, if available.
    fn generate_dependency_inforamtion(
        &self,
        latest: Option<&DependencyCollectionVersion>,
    ) -> Vec<String> {
        let mut information: Vec<String> = Vec::new();

        // Fetched dependency information
        match latest {
            Some(DependencyCollectionVersion::Latest(l)) => {
                information.push(self.generate_dependency_information_line("Latest version:", l))
            }
            Some(DependencyCollectionVersion::Excluded) => {
                information.push("Dependency is excluded!".to_string())
            }
            Some(DependencyCollectionVersion::Error(e)) => information.push(e.to_string()),
            _ => {}
        }

        information
    }

    /// Formats an individual index entry with version, features count, deps count, and yanked status.
    ///
    /// # Arguments
    /// * `line` - Crates index item representation.
    fn parse_index_version_line(&self, line: &CratesIndexItem) -> String {
        format!(
            "{} | Features: {:>4} | Dependencies: {:>4} | Yenked: {:>5}",
            self.generate_dependency_information_line("Version:", &line.vers),
            line.features.len(),
            line.deps.len(),
            line.yanked
        )
    }

    /// Extracts and sorts all versions from crates.io sparse index into display lines in descending SemVer order.
    ///
    /// # Arguments
    /// * `index` - Slice of sparse index items.
    fn get_registry_index_information(&self, index: &[CratesIndexItem]) -> Vec<String> {
        let mut parsed_version_index: Vec<CratesIndexItem> = index.to_vec();
        parsed_version_index.sort_by_cached_key(|entry| Version::parse(&entry.vers).ok());
        parsed_version_index
            .iter()
            .map(|i| self.parse_index_version_line(i))
            .rev()
            .collect()
    }

    /// Formats a key and value into a padded line fitting the inner width.
    ///
    /// # Arguments
    /// * `label` - Descriptive label.
    /// * `value` - Formatted value.
    fn generate_registry_line_with_fillings(&self, label: &str, value: &str) -> String {
        let label_str = format!("{:} ", label);
        let value_str = format!(" {:}", value);
        let fill_length =
            (TERMINAL_MAX_WIDTH as usize).saturating_sub(label_str.chars().count() + 4);

        format!("{label_str}{value_str:.>fill_length$}")
    }

    /// Checks if a string value matches the character length of a pattern string.
    ///
    /// # Arguments
    /// * `value` - Input string to check.
    /// * `pattern` - Target pattern mask.
    fn check_value_pattern(&self, value: &str, pattern: &str) -> bool {
        if value.chars().count() != pattern.chars().count() {
            return false;
        };
        true
    }

    /// Validates date and time component lengths against expected patterns.
    ///
    /// # Arguments
    /// * `dt_vec` - Slice of date/time tokens.
    fn check_dt_pattern(&self, dt_vec: &[&str]) -> time::Result<[String; 6]> {
        let [
            year_pattern,
            month_pattern,
            day_pattern,
            hours_pattern,
            min_pattern,
            sec_pattern,
        ] = ["dddd", "dd", "dd", "dd", "dd", "dd"];
        let [year, month, day, hours, min, sec]: [String; 6] = [
            dt_vec[0].to_string(),
            dt_vec[1].to_string(),
            dt_vec[2].to_string(),
            dt_vec[3].to_string(),
            dt_vec[4].to_string(),
            dt_vec[5].to_string(),
        ];
        if self.check_value_pattern(&year, year_pattern)
            && self.check_value_pattern(&month, month_pattern)
            && self.check_value_pattern(&day, day_pattern)
            && self.check_value_pattern(&hours, hours_pattern)
            && self.check_value_pattern(&min, min_pattern)
            && self.check_value_pattern(&sec, sec_pattern)
        {
            return Ok([year, month, day, hours, min, sec]);
        };
        Err(time::Error::Format(Format::InvalidComponent(
            "Invalid pattern",
        )))
    }

    /// Parse the responsed timestamps from ISO to human readable.
    ///
    /// Only parse '2026-07-18T20:59:37.656167Z' possible.
    /// Returns '2026-07-18 18:20:59 UTC'
    fn parse_dt(&self, dt: String) -> time::Result<String> {
        let dt_clone = dt.clone();
        let binding = &dt_clone.replace("Z", "");
        let dt_vec: Vec<&str> = binding.split(&['T', '-', ':', '.'][..]).collect();
        if dt_vec.len() < 6 {
            return Err(time::Error::Format(Format::InvalidComponent(
                "Invalid format",
            )));
        }
        let [year, month, day, hours, min, sec] = self.check_dt_pattern(&dt_vec)?;
        Ok(format!("{year}-{month}-{day} {hours}:{min}:{sec}"))
    }

    /// Parses an ISO timestamp into a human-readable string or returns error text.
    ///
    /// # Arguments
    /// * `timestamp` - Raw timestamp string.
    fn parse_timestamp(&self, timestamp: &str) -> String {
        match self.parse_dt(timestamp.to_string()) {
            Ok(dt) => dt,
            Err(e) => e.to_string(),
        }
    }

    /// Formats up to 4 elements from a string vector into a comma-separated list.
    ///
    /// # Arguments
    /// * `vec` - Slice of strings.
    fn parse_registry_vec(&self, vec: &[String]) -> String {
        vec[0..vec.len().clamp(0, 4)].join(", ").to_string()
    }

    /// Wraps long single-line text descriptions across multiple lines fitting the terminal width.
    ///
    /// # Arguments
    /// * `desc` - Long description text.
    fn parse_description(&self, desc: &str) -> Vec<String> {
        let max_line_length = (TERMINAL_MAX_WIDTH as usize).saturating_sub(4);
        let splitted_desc = desc.split(" ");

        let mut line_temp = String::new();
        let mut lines: Vec<String> = Vec::new();

        for word in splitted_desc {
            if line_temp
                .chars()
                .count()
                .saturating_add(word.chars().count())
                > max_line_length
            {
                lines.push(line_temp.trim_end().to_string());
                line_temp = String::new();
            }
            line_temp += word;
            line_temp += " ";
        }
        if !line_temp.is_empty() {
            lines.push(line_temp.trim_end().to_string());
        }

        lines
    }

    /// Parses multi-line descriptions with embedded newlines into wrapped terminal rows.
    ///
    /// # Arguments
    /// * `desc` - Raw description string with linebreaks.
    fn parse_description_with_linebreaks(&self, desc: &str) -> Vec<String> {
        let splitted_by_linebreak = desc.split("\n");
        let mut lines = Vec::new();
        for line in splitted_by_linebreak {
            lines.append(&mut self.parse_description(line));
        }
        lines
    }

    /// Generates an inner section separator line fitting the terminal content area.
    fn generate_registry_seperator(&self) -> String {
        let max_line_length = (TERMINAL_MAX_WIDTH as usize).saturating_sub(4);
        "-".repeat(max_line_length)
    }

    /// Formats a version entry row with yanked indicators.
    ///
    /// # Arguments
    /// * `version` - crates.io Version entry.
    fn generate_registry_versions_line(&self, version: IOVersion) -> String {
        let mut version_num = version.num;
        if version.yanked {
            version_num += " [YANKED]";
        }
        self.generate_registry_line_with_fillings("Version:", &version_num)
    }

    /// Formats a centered text header row.
    ///
    /// # Arguments
    /// * `value` - Header text.
    fn generate_centered_header(&self, value: &str) -> String {
        let max_line_length = (TERMINAL_MAX_WIDTH as usize).saturating_sub(4);
        format!("{value:^max_line_length$}")
    }

    /// Extracts and formats rich crates.io web API metadata into display rows.
    ///
    /// # Arguments
    /// * `api` - crates.io web API response model.
    fn get_registry_api_information(&self, api: &CratesIOResponse) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();

        // From field 'crate'
        let crate_field = &api.crate_field;

        lines.push("Crate information:".to_string());
        lines.push(self.generate_registry_seperator());

        lines.push(self.generate_registry_line_with_fillings(
            "Created at:",
            &self.parse_timestamp(&crate_field.created_at),
        ));
        lines.push(self.generate_registry_line_with_fillings(
            "Last updated:",
            &self.parse_timestamp(&crate_field.updated_at),
        ));
        lines.push(
            self.generate_registry_line_with_fillings(
                "Latest version:",
                &crate_field.newest_version,
            ),
        );
        if let Some(d) = &crate_field.description {
            lines.push(self.generate_registry_seperator());
            lines.append(&mut self.parse_description(d));
        }
        if crate_field.homepage.is_some()
            || crate_field.documentation.is_some()
            || crate_field.repository.is_some()
        {
            lines.push(self.generate_registry_seperator());
            if let Some(h) = &crate_field.homepage {
                lines.push(self.generate_registry_line_with_fillings("Homepage:", h));
            }
            if let Some(doc) = &crate_field.documentation {
                lines.push(self.generate_registry_line_with_fillings("Documentation:", doc));
            }
            if let Some(repo) = &crate_field.repository {
                lines.push(self.generate_registry_line_with_fillings("Repository:", repo));
            }
        }
        if !crate_field.keywords.is_empty() || !crate_field.categories.is_empty() {
            lines.push(self.generate_registry_seperator());
            if !crate_field.categories.is_empty() {
                lines.push(self.generate_registry_line_with_fillings(
                    "Categories:",
                    &self.parse_registry_vec(&crate_field.categories),
                ));
            }
            if !crate_field.keywords.is_empty() {
                lines.push(self.generate_registry_line_with_fillings(
                    "Keywords:",
                    &self.parse_registry_vec(&crate_field.keywords),
                ));
            }
        }
        if crate_field.yanked {
            lines.push(self.generate_registry_seperator());
            lines.push(
                self.generate_registry_line_with_fillings(
                    "Yanked:",
                    &crate_field.yanked.to_string(),
                ),
            );
        }

        // Versions
        if !api.versions.is_empty() {
            lines.push(self.generate_registry_seperator());
            lines.push("Versions:".to_string());
            let versions = &api.versions;
            for version in versions {
                lines.push(self.generate_registry_seperator());
                lines.push(self.generate_registry_versions_line(version.clone()));
                lines.push(self.generate_registry_line_with_fillings(
                    "Created at:",
                    &self.parse_timestamp(&version.created_at),
                ));
                lines.push(self.generate_registry_line_with_fillings(
                    "Last update:",
                    &self.parse_timestamp(&version.updated_at),
                ));
                if let Some(license) = &version.license {
                    lines.push(self.generate_registry_line_with_fillings("License:", license));
                }
                if let Some(rust_version) = &version.rust_version {
                    lines.push(self.generate_registry_line_with_fillings(
                        "Minimum required Rust version:",
                        rust_version,
                    ));
                }
            }
        }

        lines
    }

    /// Capitalizes the first letter of a string and converts remaining letters to lowercase.
    ///
    /// # Arguments
    /// * `value` - Input string slice.
    fn parse_first_uppercase_string(&self, value: &str) -> String {
        let first_letter = value[0..1].to_ascii_uppercase();
        let rest = value[1..value.chars().count()].to_lowercase();
        format!("{first_letter}{rest}")
    }

    /// Formats affected package details (ecosystem, OS, affected functions, SemVer ranges) into display rows.
    ///
    /// # Arguments
    /// * `affected` - RustSec affected package details.
    fn get_audit_affected_lines(&self, affected: &Affected) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();

        lines.push(
            self.generate_registry_line_with_fillings("Ecosystem:", &affected.package.ecosystem),
        );
        if !affected.ecosystem_specific.affects.os.is_empty()
            || !affected.ecosystem_specific.affects.functions.is_empty()
        {
            lines.push(self.generate_registry_seperator());
            if !affected.ecosystem_specific.affects.os.is_empty() {
                lines.push(self.generate_centered_header("Affected OS"));
                lines.append(
                    &mut self.parse_description(&affected.ecosystem_specific.affects.os.join(", ")),
                );
            }
            if !affected.ecosystem_specific.affects.functions.is_empty() {
                lines.push(self.generate_registry_seperator());
                lines.push(self.generate_centered_header("Affected Functions"));
                lines.append(
                    &mut self.parse_description(
                        &affected.ecosystem_specific.affects.functions.join(", "),
                    ),
                );
            }
        }

        if !affected.ranges.is_empty() {
            for range in &affected.ranges {
                if !range.events.is_empty() {
                    lines.push(self.generate_registry_seperator());
                    lines.push(self.generate_centered_header(
                        &self.parse_first_uppercase_string(&range.type_field),
                    ));
                    for event in &range.events {
                        if let Some(introduced) = &event.introduced {
                            lines.push(self.generate_registry_seperator());
                            lines.push(self.generate_registry_line_with_fillings(
                                "Introduced in:",
                                introduced,
                            ));
                        }
                        if let Some(fixed_in) = &event.fixed {
                            lines.push(self.generate_registry_seperator());
                            lines.push(
                                self.generate_registry_line_with_fillings("Fixed in:", fixed_in),
                            );
                        }
                    }
                }
            }
        }

        lines
    }

    /// Formats RustSec vulnerability advisory entries into display lines.
    ///
    /// # Arguments
    /// * `audit` - Vector of RustSec advisory items.
    fn get_registry_audit_information(&self, audit: &Vec<RustsecItem>) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push("Published Security Audits:".to_string());

        for sec in audit {
            lines.push(self.generate_registry_seperator());
            lines.push(self.generate_centered_header(&sec.id));
            lines.push(self.generate_registry_line_with_fillings("Published:", &sec.published));
            lines.push(self.generate_registry_line_with_fillings("Updated:", &sec.modified));
            lines.push(self.generate_registry_seperator());
            lines.push(self.generate_centered_header("Description"));
            lines.append(&mut self.parse_description(&sec.summary));
            lines.push(self.generate_registry_seperator());
            lines.append(&mut self.parse_description_with_linebreaks(&sec.details));
            if !sec.affected.is_empty() {
                lines.push(self.generate_registry_seperator());
                lines.push(self.generate_centered_header("Affected"));
                for affected in &sec.affected {
                    lines.append(&mut self.get_audit_affected_lines(affected));
                }
            }
            if !sec.severity.is_empty() {
                lines.push(self.generate_registry_seperator());
                lines.push(self.generate_centered_header("Severities"));
                for severity in &sec.severity {
                    lines.push(self.generate_registry_line_with_fillings(
                        &severity.type_field,
                        &severity.score,
                    ));
                }
            }
            if !sec.references.is_empty() {
                lines.push(self.generate_registry_seperator());
                lines.push(self.generate_centered_header("References"));
                for references in &sec.references {
                    lines.push(self.generate_registry_line_with_fillings(
                        &self.parse_first_uppercase_string(&references.type_field),
                        &references.url,
                    ));
                }
            }
        }

        lines
    }

    /// Compiles all registry information categories (index, api, audit) for a given crate.
    ///
    /// # Arguments
    /// * `dep` - Package name.
    /// * `client` - Optional registry client instance.
    fn generate_client_information(
        &self,
        dep: &str,
        client: Option<Client>,
    ) -> ClientInformationLines {
        let mut idx_information: Vec<String> = Vec::new();
        let mut api_information: Vec<String> = Vec::new();
        let mut audit_information: Vec<String> = Vec::new();

        if let Some(c) = client
            && let Some(crate_entry) = c.crates.get(dep)
            && let RegistryClientState::Succesed(registry) = crate_entry
        {
            if let Some(idx) = &registry.index
                && !idx.is_empty()
            {
                idx_information.append(&mut self.get_registry_index_information(idx));
            };
            if let Some(api) = &registry.api {
                api_information.append(&mut self.get_registry_api_information(api));
            };
            if let Some(audit) = &registry.audit
                && !audit.is_empty()
            {
                audit_information.append(&mut self.get_registry_audit_information(audit));
            }
        }

        ClientInformationLines {
            index: idx_information,
            api: api_information,
            audit: audit_information,
        }
    }

    /// Formats an individual tab label button with selection brackets.
    ///
    /// # Arguments
    /// * `page` - Tab name.
    /// * `is_highlighted` - Whether this tab is currently active.
    fn generate_page_item(&self, page: &str, is_highlighted: bool) -> String {
        if is_highlighted {
            format!("[{page:^10}]")
        } else {
            format!(" {page:^10} ")
        }
    }

    /// Generates the horizontal tab bar string for active dependency metadata categories.
    ///
    /// # Arguments
    /// * `dep` - Package name.
    /// * `client` - Optional registry client instance.
    /// * `highlightes_page` - Identifier of the currently active tab.
    fn generate_pages(
        &self,
        dep: &str,
        client: Option<Client>,
        highlightes_page: String,
    ) -> String {
        if let Some(c) = client
            && let Some(crate_entry) = c.crates.get(dep)
            && let RegistryClientState::Succesed(registry) = crate_entry
        {
            let mut page_strings: Vec<String> = Vec::new();
            if let Some(idx) = &registry.index
                && !idx.is_empty()
            {
                page_strings.push(self.generate_page_item("Index", highlightes_page == "index"));
            };
            if let Some(_api) = &registry.api {
                page_strings.push(self.generate_page_item("crates.io", highlightes_page == "api"));
            };
            if let Some(audit) = &registry.audit
                && !audit.is_empty()
            {
                page_strings.push(self.generate_page_item("RustSec", highlightes_page == "audit"));
            };
            page_strings.join(" | ").to_string()
        } else {
            String::new()
        }
    }

    /// Retrieves line content for a requested tab identifier.
    ///
    /// # Arguments
    /// * `page` - Target tab identifier (`index`, `api`, or `audit`).
    /// * `client_information` - Compiled client information structure.
    fn retreive_page(&self, page: &str, client_information: ClientInformationLines) -> Vec<String> {
        match page {
            "index" => client_information.index,
            "api" => client_information.api,
            "audit" => client_information.audit,
            _ => Vec::new(),
        }
    }

    /// Resolves the preceding available tab identifier.
    ///
    /// # Arguments
    /// * `current` - Active tab identifier.
    /// * `client_lines` - Available content across tab categories.
    fn get_prev_dependency_page(
        &self,
        current: &str,
        client_lines: ClientInformationLines,
    ) -> String {
        let has_index = !client_lines.index.is_empty();
        let has_api = !client_lines.api.is_empty();

        match current {
            "api" => {
                if has_index {
                    String::from("index")
                } else {
                    current.to_string()
                }
            }
            "audit" => {
                if has_api {
                    String::from("api")
                } else if has_index {
                    String::from("index")
                } else {
                    current.to_string()
                }
            }
            _ => current.to_string(),
        }
    }

    /// Resolves the succeeding available tab identifier.
    ///
    /// # Arguments
    /// * `current` - Active tab identifier.
    /// * `client_lines` - Available content across tab categories.
    fn get_next_dependency_page(
        &self,
        current: &str,
        client_lines: ClientInformationLines,
    ) -> String {
        let has_api = !client_lines.api.is_empty();
        let has_audit = !client_lines.audit.is_empty();

        match current {
            "index" => {
                if has_api {
                    String::from("api")
                } else if has_audit {
                    String::from("audit")
                } else {
                    current.to_string()
                }
            }
            "api" => {
                if has_audit {
                    String::from("audit")
                } else {
                    current.to_string()
                }
            }
            _ => current.to_string(),
        }
    }

    /// Updates interactive navigation capabilities based on scroll offset and tab availability.
    ///
    /// # Arguments
    /// * `first_item` - First visible line offset.
    /// * `last_item` - Last visible line offset.
    /// * `total_lines` - Total content line count.
    /// * `current_page` - Active tab identifier.
    /// * `client_lines` - Content availability across categories.
    fn prepare_dependency_pagination_info(
        &mut self,
        first_item: usize,
        last_item: usize,
        total_lines: usize,
        current_page: &str,
        client_lines: ClientInformationLines,
        is_updateable: bool,
    ) {
        let mut available = self.available_inputs.clone();
        let has_prev_tab =
            self.get_prev_dependency_page(current_page, client_lines.clone()) != current_page;
        let has_next_tab =
            self.get_next_dependency_page(current_page, client_lines.clone()) != current_page;
        if first_item > 0 {
            available.push(SpecialInputs::PrevItem);
            available.push(SpecialInputs::PrevPage);
        };
        if last_item < total_lines {
            available.push(SpecialInputs::NextItem);
            available.push(SpecialInputs::NextPage);
        };
        if has_prev_tab {
            available.push(SpecialInputs::TabLeft);
        };
        if has_next_tab {
            available.push(SpecialInputs::TabRight);
        };
        if is_updateable {
            available.push(SpecialInputs::Updateable);
        }
        self.generate_dependency_input_lines(available);
    }

    /// Renders the interactive dependency detail view with metadata tabs and scrollable content.
    ///
    /// # Arguments
    /// * `item` - The dependency entry to display.
    /// * `latest` - The fetched latest version status, if available.
    /// * `client` - The registry client containing cached index, API, and audit details.
    ///
    /// # Returns
    /// The resulting `TerminalReturn` action upon navigation back or item update.
    ///
    /// # Errors
    /// Returns [`io::Error`] if terminal I/O fails.
    pub fn render_dependency(
        &mut self,
        item: DependencyEntry,
        latest: Option<&DependencyCollectionVersion>,
        client: Option<Client>,
        update_information: &mut Option<UpdaterInformation>,
    ) -> io::Result<TerminalReturn> {
        let selection: TerminalReturn;
        let entry_lines = self.generate_entry_inforamtion(item.clone());
        let dependency_lines = self.generate_dependency_inforamtion(latest);
        let client_information = self.generate_client_information(item.package(), client.clone());

        let mut selected_page = String::from("index");

        let (is_latest, need_migration, msg) = match latest {
            Some(DependencyCollectionVersion::Latest(l)) => {
                item.get_dependency_row_valus(Some(l.to_string()))
            }
            Some(DependencyCollectionVersion::Error(_)) => item.get_dependency_row_valus(None),
            Some(DependencyCollectionVersion::Excluded) => item.get_dependency_row_valus(None),
            _ => item.get_dependency_row_valus(None),
        };

        if let Some(information) = update_information
            && is_latest.is_false()
            && let Some(DependencyCollectionVersion::Latest(l)) = latest
            && let Some(version) = DependencyEntry::parse_base_version(l)
        {
            information.set_latest(&version.to_string());
        }

        let entry_info = format!(
            "Information: {:} | Updateable: {:5} | Possible requires migration: {:5}",
            msg,
            is_latest.is_false(),
            need_migration.is_true(),
        );

        // Inside dependecy render we scroll inside the client information.
        let mut first_item: usize = 0;
        let mut client_lines = self.retreive_page(&selected_page, client_information.clone());
        let mut scrollable_len = client_lines.len();
        let mut last_item = first_item.saturating_add(
            self.max_items
                .clamp(0, scrollable_len.saturating_sub(first_item)),
        );
        let term = Term::stdout();
        term.hide_cursor()?;
        loop {
            term.clear_screen()?;
            self.prepare_dependency_pagination_info(
                first_item,
                last_item,
                scrollable_len,
                &selected_page,
                client_information.clone(),
                update_information.is_some() && is_latest.is_false(),
            );
            term.write_line(&self.get_terminal_section_seperator())?;
            term.write_line(&self.menu_path.join(" -> "))?;
            term.write_line(&self.get_terminal_section_seperator())?;
            term.write_line(&self.generate_dependency_single_line(&self.generate_pages(
                item.package(),
                client.clone(),
                selected_page.to_string(),
            )))?;
            term.write_line(&self.get_terminal_section_seperator())?;
            for entry_item in &entry_lines {
                term.write_line(&self.generate_dependency_single_line(entry_item))?;
            }
            for dependency_item in &dependency_lines {
                term.write_line(&self.generate_dependency_single_line(dependency_item))?;
            }
            term.write_line(&self.get_terminal_section_seperator())?;
            term.write_line(&self.generate_dependency_single_line(&entry_info))?;
            term.write_line(&self.get_terminal_section_seperator())?;
            let client_line = &client_lines[first_item..last_item];
            for client_item in client_line {
                term.write_line(&self.generate_dependency_single_line(client_item))?;
            }
            // Fill missing selectable lines only when total items count is less than max_items
            if scrollable_len < self.max_items {
                for _ in 0..(self.max_items.saturating_sub(scrollable_len)) {
                    term.write_line(&self.generate_item_fill_line())?;
                }
            }
            term.write_line(&self.get_terminal_section_seperator())?;
            for input_line in &self.input_lines {
                if !input_line.is_empty() {
                    term.write_line(input_line)?;
                }
            }
            term.write_line(&self.get_terminal_section_seperator())?;

            let key = term.read_key()?;

            match key {
                // Scroll to beginn
                Key::Home => {
                    if first_item != 0 {
                        first_item = 0;
                        last_item = first_item.saturating_add(
                            self.max_items
                                .clamp(0, scrollable_len.saturating_sub(first_item)),
                        );
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            211,
                            "Scrollable view scrolled to top (Home)"
                        );
                    }
                }
                // Scroll to end
                Key::End => {
                    if last_item != scrollable_len {
                        first_item = scrollable_len.saturating_sub(self.max_items);
                        last_item = first_item.saturating_add(
                            self.max_items
                                .clamp(0, scrollable_len.saturating_sub(first_item)),
                        );
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            212,
                            format!("Scrollable view scrolled to end (End): offset {first_item}")
                        );
                    }
                }
                // Prev page
                Key::PageUp => {
                    if first_item != 0 {
                        first_item = first_item
                            .saturating_sub(self.max_items)
                            .clamp(0, scrollable_len.saturating_sub(self.max_items));
                        last_item = first_item.saturating_add(
                            self.max_items
                                .clamp(0, scrollable_len.saturating_sub(first_item)),
                        );
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            213,
                            format!(
                                "Scrollable view scrolled page up (PageUp): offset {first_item}"
                            )
                        );
                    }
                }
                // Next Page
                Key::PageDown => {
                    if last_item != scrollable_len {
                        first_item = first_item
                            .saturating_add(self.max_items)
                            .clamp(0, scrollable_len.saturating_sub(self.max_items));
                        last_item = first_item.saturating_add(
                            self.max_items
                                .clamp(0, scrollable_len.saturating_sub(first_item)),
                        );
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            214,
                            format!(
                                "Scrollable view scrolled page down (PageDown): offset {first_item}"
                            )
                        );
                    }
                }
                // Scroll up.
                Key::ArrowUp => {
                    // scroll up if we are udner first item
                    if first_item != 0 {
                        first_item = first_item.saturating_sub(1);
                        last_item = first_item.saturating_add(
                            self.max_items
                                .clamp(0, scrollable_len.saturating_sub(first_item)),
                        );
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            215,
                            format!("Scrollable view scrolled up (ArrowUp): offset {first_item}")
                        );
                    }
                }
                //  Scroll down.
                Key::ArrowDown => {
                    // scroll down if we are over last item
                    if last_item != scrollable_len {
                        first_item = first_item.saturating_add(1);
                        last_item = first_item.saturating_add(
                            self.max_items
                                .clamp(0, scrollable_len.saturating_sub(first_item)),
                        );
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            216,
                            format!(
                                "Scrollable view scrolled down (ArrowDown): offset {first_item}"
                            )
                        );
                    }
                }
                // Prev Tab (←)
                Key::ArrowLeft => {
                    first_item = 0;
                    selected_page =
                        self.get_prev_dependency_page(&selected_page, client_information.clone());
                    client_lines = self.retreive_page(&selected_page, client_information.clone());
                    scrollable_len = client_lines.len();
                    last_item = first_item.saturating_add(
                        self.max_items
                            .clamp(0, scrollable_len.saturating_sub(first_item)),
                    );
                    simple_status!(
                        log::LevelFilter::Trace,
                        Module::INTERACTIVITY,
                        217,
                        format!("Switched to prev tab (ArrowLeft): {selected_page:?}")
                    );
                }
                // Next Tab (→)
                Key::ArrowRight => {
                    first_item = 0;
                    selected_page =
                        self.get_next_dependency_page(&selected_page, client_information.clone());
                    client_lines = self.retreive_page(&selected_page, client_information.clone());
                    scrollable_len = client_lines.len();
                    last_item = first_item.saturating_add(
                        self.max_items
                            .clamp(0, scrollable_len.saturating_sub(first_item)),
                    );
                    simple_status!(
                        log::LevelFilter::Trace,
                        Module::INTERACTIVITY,
                        218,
                        format!("Switched to next tab (ArrowRight): {selected_page:?}")
                    );
                }
                // Go one view back
                Key::Backspace => {
                    if self.is_input_included(SpecialInputs::ViewBack) {
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            219,
                            "Navigating back from scrollable view (Backspace)"
                        );
                        selection = TerminalReturn::PrevView;
                        break;
                    }
                }
                // Update selected
                Key::Char('u') => {
                    simple_status!(
                        log::LevelFilter::Trace,
                        Module::INTERACTIVITY,
                        209,
                        format!(
                            "Update requested for item {:} (Key 'u')\n{:?}",
                            item.toml_key(),
                            update_information
                        )
                    );
                    if self.is_input_included(SpecialInputs::Update) && update_information.is_some()
                    {
                        selection = TerminalReturn::Update(update_information.clone());
                        term.clear_screen()?;
                        break;
                    }
                }
                // Clean quit
                Key::CtrlC => {
                    Self::leave_alternate_screen()?;
                    process::exit(2);
                }
                // Clean exit
                Key::Escape => {
                    Self::leave_alternate_screen()?;
                    process::exit(0);
                }
                _ => {}
            }
        }
        Ok(selection)
    }

    /// Renders the interactive terminal interface loop, handling user keystrokes, pagination, and item selection.
    ///
    /// # Arguments
    /// * `items` - Slice of tuples containing 1-based display index, formatted label string, and updateability flag.
    ///
    /// # Returns
    /// The resulting `TerminalReturn` action upon selection or view navigation back.
    ///
    /// # Errors
    /// Returns [`io::Error`] if terminal I/O fails.
    pub fn render(
        &mut self,
        items: &[(usize, String, Option<UpdaterInformation>)],
    ) -> io::Result<TerminalReturn> {
        let selection: TerminalReturn;
        let items_len = items.len();
        let mut highlighted: usize = 0;
        let mut first_item: usize = 0;
        let mut last_item = self.max_items.clamp(0, items_len);
        loop {
            self.prepare_paginated_input_line(
                first_item,
                last_item,
                highlighted,
                items_len,
                self.is_updateable_highlighted(&highlighted, items),
            );
            let term = Term::stdout();
            term.clear_screen()?;
            term.write_line(&self.get_terminal_section_seperator())?;
            term.write_line(&self.menu_path.join(" -> "))?;
            term.write_line(&self.get_terminal_section_seperator())?;
            term.write_line(&self.instruction_line)?;
            term.write_line(&self.get_terminal_section_seperator())?;
            let rendered_items = &items[first_item..last_item];
            for rendered_item in rendered_items {
                term.write_line(&self.get_selection_line(rendered_item, highlighted))?;
            }
            // Fill missing selectable lines only when total items count is less than max_items
            if items_len < self.max_items {
                for _ in 0..(self.max_items.saturating_sub(items_len)) {
                    term.write_line(&self.generate_item_fill_line())?;
                }
            }
            term.write_line(&self.get_terminal_section_seperator())?;
            term.write_line(&self.generate_pagination_line(
                highlighted,
                first_item,
                last_item,
                items_len,
            ))?;
            term.write_line(&self.get_terminal_section_seperator())?;
            for input_line in &self.input_lines {
                if !input_line.is_empty() {
                    term.write_line(input_line)?;
                }
            }
            term.write_line(&self.get_terminal_section_seperator())?;

            let key = term.read_key()?;

            match key {
                // Select item
                Key::Enter => {
                    if self.is_input_included(SpecialInputs::Select) {
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            201,
                            format!("Item at index {highlighted} selected (Enter)")
                        );
                        selection = TerminalReturn::SelectItem(highlighted);
                        break;
                    }
                }
                // Highlight prev item. Scroll if needed.
                Key::ArrowUp => {
                    if highlighted > 0 {
                        let new_highlighted: usize = highlighted.saturating_sub(1);
                        highlighted = new_highlighted;
                        // scroll up if we are udner first item
                        if new_highlighted < first_item && first_item != 0 {
                            first_item = first_item.saturating_sub(1);
                            last_item = last_item.saturating_sub(1);
                        }
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            202,
                            format!("Moved highlight up to item {highlighted}")
                        );
                    }
                }
                // Highlight next item. Scroll if needed.
                Key::ArrowDown => {
                    let new_highlighted: usize = highlighted.saturating_add(1);
                    if new_highlighted < items_len {
                        highlighted = new_highlighted;
                        // scroll down if we are over last item
                        if new_highlighted == last_item {
                            first_item = first_item.saturating_add(1);
                            last_item = last_item.saturating_add(1);
                        }
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            203,
                            format!("Moved highlight down to item {highlighted}")
                        );
                    }
                }
                // Prev page
                Key::PageUp => {
                    if first_item > 0 {
                        let old_first = first_item;
                        first_item = first_item.saturating_sub(self.max_items);
                        let shift = old_first - first_item;
                        last_item = (first_item + self.max_items).min(items_len);
                        highlighted = highlighted
                            .saturating_sub(shift)
                            .clamp(first_item, last_item.saturating_sub(1));
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            204,
                            format!(
                                "Paged up: window {first_item}..{last_item}, highlighted {highlighted}"
                            )
                        );
                    }
                }
                // Next page
                Key::PageDown => {
                    if last_item < items_len {
                        let old_last = last_item;
                        let new_last = (last_item + self.max_items).min(items_len);
                        let shift = new_last - old_last;
                        last_item = new_last;
                        first_item = last_item.saturating_sub(self.max_items);
                        highlighted =
                            (highlighted + shift).clamp(first_item, last_item.saturating_sub(1));
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            205,
                            format!(
                                "Paged down: window {first_item}..{last_item}, highlighted {highlighted}"
                            )
                        );
                    }
                }
                Key::Home => {
                    if items_len > 0 && highlighted != 0 {
                        highlighted = 0;
                        first_item = 0;
                        last_item = self.max_items.clamp(0, items_len);
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            206,
                            "Jumped to first item (Home)"
                        );
                    }
                }
                Key::End => {
                    if items_len > 0 && highlighted < items_len.saturating_sub(1) {
                        highlighted = items_len.saturating_sub(1);
                        last_item = items_len;
                        first_item = items_len.saturating_sub(self.max_items);
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            207,
                            format!("Jumped to last item (End): {highlighted}")
                        );
                    }
                }
                // Go one view back
                Key::Backspace => {
                    if self.is_input_included(SpecialInputs::ViewBack) {
                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            208,
                            "Navigating back to previous view (Backspace)"
                        );
                        selection = TerminalReturn::PrevView;
                        break;
                    }
                }
                // Update selected
                Key::Char('u') => {
                    simple_status!(
                        log::LevelFilter::Trace,
                        Module::INTERACTIVITY,
                        209,
                        format!(
                            "Update requested for item {highlighted} (Key 'u')\n{:?}",
                            items[highlighted].2
                        )
                    );
                    if self.is_input_included(SpecialInputs::Update)
                        && let Some(information) = &items[highlighted].2
                    {
                        selection = TerminalReturn::Update(Some(information.clone()));
                        break;
                    }
                }
                // Clean quit
                Key::CtrlC => {
                    Self::leave_alternate_screen()?;
                    process::exit(2);
                }
                // Clean exit
                Key::Escape => {
                    Self::leave_alternate_screen()?;
                    process::exit(0);
                }
                _ => {}
            }
            term.clear_screen()?;
        }
        Ok(selection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_terminal_initialization() {
        let term = Terminal::new(10);
        assert_eq!(term.max_items, 10);
        assert!(term.menu_path.is_empty());
        assert!(term.available_inputs.is_empty());
        assert!(term.instruction_line.is_empty());
        assert!(term.input_lines.is_empty());
    }

    #[tokio::test]
    async fn test_path_manipulation() {
        let mut term = Terminal::new(10);
        term.set_path(vec!["Workspace".to_string(), "Crate A".to_string()]);
        assert_eq!(
            term.menu_path,
            vec![
                "RRDU Interactive Terminal".to_string(),
                "Workspace".to_string(),
                "Crate A".to_string(),
            ]
        );

        term.add_path("Section");
        assert_eq!(term.menu_path.len(), 4);
        assert_eq!(term.menu_path[3], "Section");

        term.remove_path("Workspace");
        assert_eq!(
            term.menu_path,
            vec![
                "RRDU Interactive Terminal".to_string(),
                "Crate A".to_string(),
                "Section".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn test_set_instruction() {
        let mut term = Terminal::new(10);
        term.set_instruction("Press Enter to select".to_string());
        assert_eq!(term.instruction_line, "Press Enter to select");
    }

    #[tokio::test]
    async fn test_generate_input_lines() {
        let mut term = Terminal::new(10);
        term.generate_input_lines(vec![
            SpecialInputs::PrevItem,
            SpecialInputs::NextItem,
            SpecialInputs::PrevPage,
            SpecialInputs::NextPage,
            SpecialInputs::Select,
            SpecialInputs::Update,
            SpecialInputs::ViewBack,
        ]);
        assert_eq!(term.input_lines.len(), 3);
        assert!(term.input_lines[0].contains("↑ = Prev Item"));
        assert!(term.input_lines[1].contains("↓ = Next Item"));
        assert!(term.input_lines[0].contains("Page ↑ = Scroll to prev page"));
        assert!(term.input_lines[1].contains("Page ↓ = Scroll to next page"));
        assert!(term.input_lines[0].contains("Pos 1 = Scroll to beginn"));
        assert!(term.input_lines[1].contains("End = Scroll to end"));
        assert!(term.input_lines[2].contains("Backspace = Prev View"));
        assert!(term.input_lines[2].contains("Enter/Return = Select"));
        assert!(term.input_lines[2].contains("Esc = Exit"));
    }

    #[tokio::test]
    async fn test_prepare_paginated_input_line_boundaries() {
        let mut term = Terminal::new(10);
        term.set_available_input(vec![SpecialInputs::Select, SpecialInputs::ViewBack]);

        // First page of multi-page list
        term.prepare_paginated_input_line(0, 10, 0, 16, false);
        assert!(term.input_lines[0].is_empty());
        assert!(term.input_lines[1].contains("↓ = Next Item"));
        assert!(term.input_lines[1].contains("Page ↓ = Scroll to next page"));
        assert!(term.input_lines[1].contains("End = Scroll to end"));

        // Last page of multi-page list
        term.prepare_paginated_input_line(6, 16, 15, 16, false);
        assert!(term.input_lines[0].contains("↑ = Prev Item"));
        assert!(term.input_lines[0].contains("Page ↑ = Scroll to prev page"));
        assert!(term.input_lines[0].contains("Pos 1 = Scroll to beginn"));
        assert!(term.input_lines[1].is_empty());

        // Single item list
        term.prepare_paginated_input_line(0, 1, 0, 1, false);
        assert!(term.input_lines[0].is_empty());
        assert!(term.input_lines[1].is_empty());
    }

    #[tokio::test]
    async fn test_get_selection_line_formatting() {
        let term = Terminal::new(10);
        let item = (1, "serde = 1.0.197".to_string(), None);

        let highlighted_line = term.get_selection_line(&item, 0);
        assert!(highlighted_line.starts_with("|-> 1 "));
        assert!(highlighted_line.ends_with(" <-|"));

        let unhighlighted_line = term.get_selection_line(&item, 1);
        assert!(unhighlighted_line.starts_with("|   1 "));
        assert!(unhighlighted_line.ends_with("   |"));
    }

    #[tokio::test]
    async fn test_section_separator_and_fill_line() {
        let term = Terminal::new(10);
        let sep = term.get_terminal_section_seperator();
        assert_eq!(sep.len(), TERMINAL_MAX_WIDTH as usize);
        assert!(sep.chars().all(|c| c == '-'));

        let fill = term.generate_item_fill_line();
        assert_eq!(fill.len(), TERMINAL_MAX_WIDTH as usize);
        assert!(fill.starts_with('|'));
        assert!(fill.ends_with('|'));
    }

    #[tokio::test]
    async fn test_sliding_window_pagination_math() {
        let items_len: usize = 16;
        let max_items: usize = 10;

        let mut first_item: usize = 0;
        let mut last_item: usize = max_items.min(items_len);
        let mut highlighted: usize = 0;

        assert_eq!(first_item, 0);
        assert_eq!(last_item, 10);
        assert_eq!(last_item - first_item, 10);

        // Advance to next page (ArrowRight)
        if last_item < items_len {
            let old_last = last_item;
            let new_last = (last_item + max_items).min(items_len);
            let shift = new_last - old_last;
            last_item = new_last;
            first_item = last_item.saturating_sub(max_items);
            highlighted = (highlighted + shift).clamp(first_item, last_item.saturating_sub(1));
        }

        // On the last page, full max_items must be displayed: items 6..16 (count = 10)
        assert_eq!(last_item, 16);
        assert_eq!(first_item, 6);
        assert_eq!(last_item - first_item, 10);
        assert_eq!(highlighted, 6);

        // Fill lines condition: only when total items < max_items
        let fill_needed = items_len < max_items;
        assert!(!fill_needed);

        // Rewind to previous page (ArrowLeft)
        if first_item > 0 {
            let old_first = first_item;
            first_item = first_item.saturating_sub(max_items);
            let shift = old_first - first_item;
            last_item = (first_item + max_items).min(items_len);
            highlighted = highlighted
                .saturating_sub(shift)
                .clamp(first_item, last_item.saturating_sub(1));
        }

        assert_eq!(first_item, 0);
        assert_eq!(last_item, 10);
        assert_eq!(last_item - first_item, 10);
        assert_eq!(highlighted, 0);
    }

    #[tokio::test]
    async fn test_fill_lines_only_when_total_less_than_max_items() {
        let max_items: usize = 10;

        let small_items_len: usize = 4;
        let mut small_lines_drawn = 0;
        if small_items_len < max_items {
            for _ in 0..(max_items.saturating_sub(small_items_len)) {
                small_lines_drawn += 1;
            }
        }
        assert_eq!(small_lines_drawn, 6);

        let large_items_len: usize = 16;
        let mut large_lines_drawn = 0;
        if large_items_len < max_items {
            for _ in 0..(max_items.saturating_sub(large_items_len)) {
                large_lines_drawn += 1;
            }
        }
        assert_eq!(large_lines_drawn, 0);
    }

    #[tokio::test]
    async fn test_home_and_end_selection_logic() {
        let items_len: usize = 25;
        let max_items: usize = 10;

        let mut first_item: usize = 10;
        let mut last_item: usize = 20;
        let mut highlighted: usize = 15;

        // Press Home -> jump to first item
        if items_len > 0 && highlighted != 0 {
            highlighted = 0;
            first_item = 0;
            last_item = max_items.clamp(0, items_len);
        }
        assert_eq!(highlighted, 0);
        assert_eq!(first_item, 0);
        assert_eq!(last_item, 10);

        // Press End -> jump to last item
        if items_len > 0 && highlighted < items_len.saturating_sub(1) {
            highlighted = items_len.saturating_sub(1);
            last_item = items_len;
            first_item = items_len.saturating_sub(max_items);
        }
        assert_eq!(highlighted, 24);
        assert_eq!(first_item, 15);
        assert_eq!(last_item, 25);

        // Edge case: list shorter than max_items
        let short_len: usize = 4;
        let mut short_first: usize = 0;
        let mut short_last: usize = short_len;
        let mut short_highlighted: usize = 2;

        // End on short list
        if short_len > 0 && short_highlighted < short_len.saturating_sub(1) {
            short_highlighted = short_len.saturating_sub(1);
            short_last = short_len;
            short_first = short_len.saturating_sub(max_items);
        }
        assert_eq!(short_highlighted, 3);
        assert_eq!(short_first, 0);
        assert_eq!(short_last, 4);

        // Home on short list
        if short_len > 0 && short_highlighted != 0 {
            short_highlighted = 0;
            short_first = 0;
            short_last = max_items.clamp(0, short_len);
        }
        assert_eq!(short_highlighted, 0);
        assert_eq!(short_first, 0);
        assert_eq!(short_last, 4);
    }

    #[tokio::test]
    async fn test_generate_dependency_input_lines() {
        let mut term = Terminal::new(10);
        term.generate_dependency_input_lines(vec![
            SpecialInputs::PrevItem,
            SpecialInputs::NextItem,
            SpecialInputs::PrevPage,
            SpecialInputs::NextPage,
            SpecialInputs::TabLeft,
            SpecialInputs::TabRight,
            SpecialInputs::ViewBack,
        ]);

        assert_eq!(term.input_lines.len(), 4);
        assert!(term.input_lines[0].contains("← = Prev Tab"));
        assert!(term.input_lines[0].contains("→ = Next Tab"));
        assert!(term.input_lines[1].contains("↑ = Scroll up"));
        assert!(term.input_lines[1].contains("Page ↑ = Scroll to prev page"));
        assert!(term.input_lines[1].contains("Pos 1 = Scroll to beginn"));
        assert!(term.input_lines[2].contains("↓ = Scroll down"));
        assert!(term.input_lines[2].contains("Page ↓ = Scroll to next page"));
        assert!(term.input_lines[2].contains("End = Scroll to end"));
        assert!(term.input_lines[3].contains("Backspace = Prev View"));
        assert!(term.input_lines[3].contains("Esc = Exit"));
    }

    #[tokio::test]
    async fn test_scrollable_home_and_end_logic() {
        let scrollable_len: usize = 50;
        let max_items: usize = 10;

        let mut first_item: usize = 20;
        let mut last_item: usize = 30;

        // Home in scrollable view
        if first_item != 0 {
            first_item = 0;
            last_item = first_item
                .saturating_add(max_items.clamp(0, scrollable_len.saturating_sub(first_item)));
        }
        assert_eq!(first_item, 0);
        assert_eq!(last_item, 10);

        // End in scrollable view
        if last_item != scrollable_len {
            first_item = scrollable_len.saturating_sub(max_items);
            last_item = first_item
                .saturating_add(max_items.clamp(0, scrollable_len.saturating_sub(first_item)));
        }
        assert_eq!(first_item, 40);
        assert_eq!(last_item, 50);
    }
}
