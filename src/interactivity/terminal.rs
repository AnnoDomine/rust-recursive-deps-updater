use std::{io, process};

use dialoguer::console::{Key, Term};

use crate::constants::TERMINAL_MAX_WIDTH;

/// Represents the outcome of an interactive terminal navigation or selection action.
pub enum TerminalReturn {
    /// Navigate back to the previous view level.
    PrevView,
    /// An item was selected by index.
    SelectItem(usize),
    /// An update command was invoked for the item at the specified index.
    Update(usize),
}

/// Identifies supported keyboard and navigation actions available in a terminal view.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SpecialInputs {
    /// Navigate to the previous page.
    PrevPage,
    /// Navigate to the next page.
    NextPage,
    /// Move cursor to the previous item.
    PrevItem,
    /// Move cursor to the next item.
    NextItem,
    /// Select the currently highlighted item.
    Select,
    /// Trigger an update on the highlighted item.
    Update,
    /// Return to the previous menu level.
    ViewBack,
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

    /// Generates keybinding instruction rows based on enabled navigation actions.
    ///
    /// # Arguments
    /// * `available` - Slice of enabled special input actions.
    pub fn generate_input_lines(&mut self, available: Vec<SpecialInputs>) {
        let mut input_lines: Vec<String> = Vec::new();
        let mut navigation_row_line: Vec<String> = Vec::new();
        let mut navigation_page_line: Vec<String> = Vec::new();
        if available.contains(&SpecialInputs::PrevItem) {
            navigation_page_line.push(String::from("↑ = Prev Item"));
        }
        if available.contains(&SpecialInputs::NextItem) {
            navigation_page_line.push(String::from("↓ = Next Item"));
        }
        if available.contains(&SpecialInputs::PrevPage) {
            navigation_page_line.push(String::from("← = Prev Page"));
        }
        if available.contains(&SpecialInputs::NextPage) {
            navigation_page_line.push(String::from("→ = Next Page"));
        }
        navigation_page_line.append(&mut navigation_row_line);
        input_lines.push(navigation_page_line.join(" | "));
        let mut special_selection_line: Vec<String> = Vec::new();
        if available.contains(&SpecialInputs::ViewBack) {
            special_selection_line.push("Backspace = Prev View".to_string());
        }
        if available.contains(&SpecialInputs::Update) {
            special_selection_line.push("Key 'u' = Update".to_string());
        }
        if available.contains(&SpecialInputs::Select) {
            special_selection_line.push("Enter/Return = Select".to_string());
        }
        special_selection_line.push("Esc = Exit".to_string());
        input_lines.push(special_selection_line.join(" | "));
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
    /// * `item` - Tuple of 1-based display index and label string.
    /// * `highlighted` - Currently selected 0-based index.
    ///
    /// # Returns
    /// The formatted terminal row string.
    pub fn get_selection_line(&self, item: &(usize, String), highlighted: usize) -> String {
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
    pub fn prepare_paginated_input_line(
        &mut self,
        first: usize,
        last: usize,
        current_pos: usize,
        items_len: usize,
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
        self.generate_input_lines(available);
    }

    fn get_terminal_section_seperator(&self) -> String {
        "-".repeat(TERMINAL_MAX_WIDTH.into()).to_string()
    }

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

    fn generate_item_fill_line(&self) -> String {
        let fill_amount = (TERMINAL_MAX_WIDTH as usize).saturating_sub(1);
        format!("|{:>fill_amount$}", "|")
    }

    /// Renders the interactive terminal interface loop, handling user keystrokes, pagination, and item selection.
    ///
    /// # Arguments
    /// * `items` - Slice of indexed items to display.
    ///
    /// # Returns
    /// The resulting `TerminalReturn` action upon selection or view navigation back.
    ///
    /// # Errors
    /// Returns [`io::Error`] if terminal I/O fails.
    pub fn render(&mut self, items: &[(usize, String)]) -> io::Result<TerminalReturn> {
        let selection: TerminalReturn;
        let items_len = items.len();
        let mut highlighted: usize = 0;
        let mut first_item: usize = 0;
        let mut last_item = self.max_items.clamp(0, items_len);
        loop {
            self.prepare_paginated_input_line(first_item, last_item, highlighted, items_len);
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
            // Fill missing selectable lines, to align navigation over every view
            for _ in 0..(self.max_items.saturating_sub(last_item)) {
                term.write_line(&self.generate_item_fill_line())?;
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
                    }
                }
                // // Prev page
                // Key::ArrowLeft => {}
                // // Next page
                // Key::ArrowRight => {
                //     if highlighted < items_len - 1 {
                //         let mut new_highlighted: usize = highlighted.saturating_add(self.max_items);
                //         highlighted = new_highlighted;
                //         // scroll down if we are over last item
                //         if new_highlighted == last_item {
                //             first_item = first_item.saturating_add(1);
                //             last_item = last_item.saturating_add(1);
                //         }
                //     }
                // }
                // Go one view back
                Key::Char('b') => {
                    if self.is_input_included(SpecialInputs::ViewBack) {
                        selection = TerminalReturn::PrevView;
                        break;
                    }
                }
                Key::Backspace => {
                    if self.is_input_included(SpecialInputs::ViewBack) {
                        selection = TerminalReturn::PrevView;
                        break;
                    }
                }
                // Update selected
                Key::Char('u') => {
                    if self.is_input_included(SpecialInputs::Update) {
                        selection = TerminalReturn::Update(highlighted);
                        break;
                    }
                }
                // Clean quit
                Key::CtrlC => {
                    term.clear_screen()?;
                    process::exit(2);
                }
                // Clean exit
                Key::Escape => {
                    term.clear_screen()?;
                    process::exit(0);
                }
                _ => {}
            }
            term.clear_screen()?;
        }
        Ok(selection)
    }
}
