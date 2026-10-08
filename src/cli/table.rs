use std::{collections::HashMap, fmt::Display};

use comfy_table::{
    CellAlignment, ContentArrangement, Table as ComfyTable,
    Width::{self, Fixed, Percentage},
};

use crate::{constants::TERMINAL_MAX_WIDTH, simple_status, status_codes::Module};

/// Cell value variants supported by table rendering.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CellValue {
    /// Empty or missing cell value.
    None,
    /// String cell value.
    String(String),
    /// Boolean cell value.
    Boolean(bool),
    /// Numeric cell value.
    Number(usize),
}

impl Display for CellValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellValue::None => write!(f, ""),
            CellValue::String(s) => write!(f, "{s}"),
            CellValue::Boolean(b) => write!(f, "{b}"),
            CellValue::Number(n) => write!(f, "{n}"),
        }
    }
}

/// Mapping definitions for conditional cell value transformations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueHandling {
    /// List of value conditional rules.
    pub handling_map: Vec<ValueHandlingMap>,
}

/// Single value transformation mapping rule.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValueHandlingMap<T = CellValue> {
    /// Condition matching cell value.
    pub when: T,
    /// Output text when condition matches.
    pub then: String,
}

impl ValueHandling {
    /// Creates a new value handling container with an initial rule.
    pub fn new(handling: ValueHandlingMap) -> Self {
        Self {
            handling_map: vec![handling],
        }
    }

    fn compare_none(&self, handling: &ValueHandlingMap) -> Option<String> {
        if let CellValue::None = &handling.when {
            return Some(handling.then.clone());
        };
        None
    }

    fn compare_string(&self, value: &String, handling: &ValueHandlingMap) -> Option<String> {
        if let CellValue::String(w_s) = &handling.when
            && value == w_s
        {
            return Some(handling.then.clone());
        };
        None
    }

    fn compare_bool(&self, value: &bool, handling: &ValueHandlingMap) -> Option<String> {
        if let CellValue::Boolean(w_b) = &handling.when
            && value == w_b
        {
            return Some(handling.then.clone());
        };
        None
    }

    fn compare_number(&self, value: &usize, handling: &ValueHandlingMap) -> Option<String> {
        if let CellValue::Number(w_n) = &handling.when
            && value == w_n
        {
            return Some(handling.then.clone());
        };
        None
    }

    fn value_is_or(&self, value: &CellValue, handling: &ValueHandlingMap) -> Option<String> {
        match value {
            CellValue::None => self.compare_none(handling),
            CellValue::String(s) => self.compare_string(s, handling),
            CellValue::Boolean(b) => self.compare_bool(b, handling),
            CellValue::Number(n) => self.compare_number(n, handling),
        }
    }

    /// Transforms a cell value based on configured handling rules.
    pub fn handle_value(&self, value: &CellValue) -> String {
        for handling in &self.handling_map {
            if let Some(v) = self.value_is_or(value, handling) {
                return v;
            }
        }
        String::new()
    }

    /// Appends a new handling rule to the map.
    pub fn add_handling(&mut self, handling: ValueHandlingMap) {
        self.handling_map.push(handling);
    }
}

/// Styling and layout configuration for terminal table rendering.
pub struct TableStyles {
    /// Maximum width of the rendered table.
    pub table_width: u16,
    /// Define the column alignment for all rows based by column header value. (Default: left aligned if not defined)
    pub column_alignments: HashMap<String, CellAlignment>,
    /// Define the output of a cell based on cell value.
    /// e.g. If value is boolean - true then "Yes" else "No"
    /// Undefined column value handlings will fallback to stringified value
    pub column_value_handling: HashMap<String, ValueHandling>,
    /// Column width constraints.
    pub column_width: Vec<Option<Width>>,
}

impl Default for TableStyles {
    fn default() -> Self {
        Self {
            table_width: TERMINAL_MAX_WIDTH,
            column_alignments: HashMap::new(),
            column_value_handling: HashMap::new(),
            column_width: Vec::new(),
        }
    }
}

impl TableStyles {
    /// Sets alignment for a specific column header.
    pub fn set_alignment(&mut self, column: String, alignment: CellAlignment) {
        self.column_alignments.insert(column, alignment);
    }

    /// Adds a value handling rule for a column.
    pub fn add_value_handling(&mut self, column: String, handling: ValueHandlingMap) {
        match self.column_value_handling.get_mut(&column) {
            Some(col) => {
                col.add_handling(handling);
            }
            None => {
                self.column_value_handling
                    .insert(column, ValueHandling::new(handling));
            }
        };
    }

    /// Sets the overall table width.
    pub fn set_table_width(&mut self, width: u16) {
        self.table_width = width;
    }

    fn is_precentage_width(&self, width: &Width) -> bool {
        match width {
            Fixed(_) => false,
            Percentage(_) => true,
        }
    }

    fn return_width(&self, width: &Width) -> u16 {
        match width {
            Percentage(p) => *p,
            Fixed(f) => *f,
        }
    }

    fn collect_fixed_width(&self) -> Vec<u16> {
        self.column_width
            .iter()
            .flatten()
            .filter(|width| !self.is_precentage_width(width))
            .map(|w| self.return_width(w))
            .collect::<Vec<u16>>()
    }

    fn sum_fixed_width(&self) -> u16 {
        self.collect_fixed_width().iter().sum()
    }

    /// Re-aligns overall table width if fixed column widths exceed current table width.
    pub fn realign_column_width(&mut self) {
        let total_fixed_width = self.sum_fixed_width();
        if self.table_width < total_fixed_width {
            self.table_width = total_fixed_width;
        }
    }

    /// Sets a fixed width constraint for a specific column index.
    pub fn set_column_width(&mut self, idx: usize, size: u16) {
        self.column_width.insert(idx, Some(Fixed(size)));
    }

    /// Resizes the column width constraint vector to match the column count.
    pub fn initialize_column_width_vec(&mut self, column_len: usize) {
        self.column_width.resize_with(column_len, || None);
    }
}

/// Pagination controller and state tracker for paginated terminal tables.
pub struct TablePagination {
    /// Current active page number (1-based).
    pub current_page: usize,
    /// Maximum number of items per page, if pagination is enabled.
    pub page_size: Option<usize>,
    /// Total number of items tracked across all pages.
    pub item_amount: usize,
    /// Flag indicating whether the current page is the last page.
    pub is_last_page: bool,
    /// Flag indicating whether the current page is the first page.
    pub is_first_page: bool,
}

impl TablePagination {
    fn new(size: Option<usize>) -> Self {
        Self {
            current_page: 1,
            page_size: size,
            item_amount: 0,
            is_last_page: true,
            is_first_page: true,
        }
    }

    /// Records an added item and updates pagination boundary flags.
    pub fn add_item(&mut self) {
        let new_item_amount = self.item_amount + 1;
        if let Some(size) = self.page_size {
            self.is_last_page = new_item_amount > size;
        }
        self.item_amount = new_item_amount;
    }

    /// Advances to the next page if available.
    pub fn next_page(&mut self) {
        if let Some(size) = self.page_size {
            let max_pages = self.item_amount / size;
            let next_page = self.current_page + 1;
            if next_page <= max_pages {
                self.is_last_page = next_page == max_pages;
                self.current_page = next_page;
            };
        }
    }

    /// Navigates to the previous page if available.
    pub fn prev_page(&mut self) {
        let prev_page = self.current_page - 1;
        if prev_page > 0 {
            self.current_page -= 1;
        };
    }

    /// Returns a sliced subset of rows corresponding to the active page.
    pub fn return_page_rows(&self, items: &[Vec<String>]) -> Vec<Vec<String>> {
        match self.page_size {
            Some(size) => {
                let min = (self.current_page.saturating_sub(1)) * size;
                let max = std::cmp::min(items.len(), min + size);
                if min >= items.len() {
                    return Vec::new();
                }
                items[min..max].to_vec()
            }
            None => items.to_vec(),
        }
    }
}

/// Safe table wrapper using `comfy-table-inline` with styling and pagination support.
pub struct Table {
    /// Table rows containing cell values.
    pub rows: Vec<Vec<CellValue>>,
    /// Column header names.
    pub headers: Vec<String>,
    /// Style and layout configuration.
    pub style: TableStyles,
    /// Underlying comfy-table instance.
    pub table: ComfyTable,
    /// Pagination controller.
    pub pagination: TablePagination,
}

impl Table {
    /// Creates a new table instance with optional pagination page size.
    pub fn new(page_size: Option<usize>) -> Self {
        let mut table = Self {
            rows: Vec::new(),
            headers: Vec::new(),
            table: ComfyTable::new(),
            style: TableStyles::default(),
            pagination: TablePagination::new(page_size),
        };
        // Force no TTY for all tables
        table.table.force_no_tty();
        table.table.set_width(table.style.table_width);
        table
            .table
            .set_content_arrangement(ContentArrangement::DynamicFullWidth);
        table
    }

    fn stringify_cell(&self, value: &CellValue, handling: Option<&ValueHandling>) -> String {
        match handling {
            Some(ha) => ha.handle_value(value),
            None => value.to_string(),
        }
    }

    fn stringify_row(&self, row: &[CellValue]) -> Vec<String> {
        let mut stringified_row: Vec<String> = Vec::new();
        for (idx, item) in row.iter().enumerate().take(self.headers.len()) {
            let header = &self.headers[idx];
            let cell = item;
            let stringified =
                self.stringify_cell(cell, self.style.column_value_handling.get(header));
            stringified_row.push(stringified);
        }
        stringified_row
    }

    fn stringify_rows(&self) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|r| self.stringify_row(r))
            .collect::<Vec<Vec<String>>>()
    }

    fn construct_table(&self) -> ComfyTable {
        let mut table = self.table.clone();
        let page_rows = self.pagination.return_page_rows(&self.stringify_rows());
        table.set_header(&self.headers);
        table.add_rows(page_rows);
        table.clone()
    }

    /// Renders the configured table with active pagination into a `comfy_table::Table` instance.
    pub fn render(&self) -> ComfyTable {
        self.construct_table()
    }

    /// Sets the table header columns and prepares width tracking.
    ///
    /// # Arguments
    /// * `headers` - Vector of column header names.
    pub fn set_headers(&mut self, headers: Vec<String>) {
        self.headers = headers.clone();
        self.style.initialize_column_width_vec(headers.len());
    }

    /// Sets a fixed width for the specified column name.
    ///
    /// # Arguments
    /// * `column` - The target column header.
    /// * `width` - The fixed character width.
    pub fn set_column_width(&mut self, column: String, width: u16) {
        if self.is_column_defined(&column, String::from("column width"))
            && let Some(idx) = self.headers.iter().position(|c| c == &column)
        {
            self.style.set_column_width(idx, width);
        }
    }

    /// Adds a value transformation rule for a named column.
    ///
    /// # Arguments
    /// * `column` - The target column header.
    /// * `handling` - The value handling mapping.
    pub fn add_value_handling(&mut self, column: String, handling: ValueHandlingMap) {
        if self.is_column_defined(&column, String::from("value handling")) {
            self.style.add_value_handling(column, handling);
        }
    }

    /// Sets cell text alignment for a named column.
    ///
    /// # Arguments
    /// * `column` - The target column header.
    /// * `alignment` - The desired cell text alignment.
    pub fn add_column_alignment(&mut self, column: String, alignment: CellAlignment) {
        if self.is_column_defined(&column, String::from("cell alignment")) {
            self.style.set_alignment(column, alignment);
        }
    }

    /// Sets both text alignment and value handling for a named column.
    ///
    /// # Arguments
    /// * `column` - The target column header.
    /// * `handling` - The value handling mapping.
    /// * `alignment` - The desired cell text alignment.
    pub fn add_column_alignment_and_value_handling(
        &mut self,
        column: String,
        handling: ValueHandlingMap,
        alignment: CellAlignment,
    ) {
        self.add_column_alignment(column.clone(), alignment);
        self.add_value_handling(column.clone(), handling);
    }

    fn is_column_defined(&self, column: &String, handling: String) -> bool {
        if !self.headers.contains(column) {
            simple_status!(
                log::LevelFilter::Error,
                Module::CLI,
                404,
                format!(
                    "Can not set {handling} for column which is not defined. Please first define '{column}'!"
                )
            );
            return false;
        };
        true
    }

    fn get_column_len(&self) -> usize {
        self.headers.len()
    }

    fn is_row_fits(&self, row_length: usize) -> bool {
        let column_len = self.get_column_len();
        match column_len {
            0 => {
                simple_status!(
                    log::LevelFilter::Warn,
                    Module::CLI,
                    404,
                    format!("No headers defined. Current: 0 - Required: {row_length}")
                );
                false
            }
            _ if row_length != column_len => {
                simple_status!(
                    log::LevelFilter::Error,
                    Module::CLI,
                    400,
                    format!(
                        "The given row size does not fit to the column amount. Current: {column_len} - Required: {row_length}"
                    )
                );
                false
            }
            _ => true,
        }
    }

    /// Validates column count and appends a row of cell values to the table.
    ///
    /// # Arguments
    /// * `row` - Vector of cell values corresponding to each column header.
    pub fn add_row(&mut self, row: Vec<CellValue>) {
        let row_len = row.len();
        if self.is_row_fits(row_len) {
            self.rows.push(row);
        };
    }
}
