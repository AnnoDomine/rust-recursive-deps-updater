use std::fmt::Display;

use crate::{
    enums::*,
    interactivity::terminal::{SpecialInputs, Terminal, TerminalReturn},
    workspace::project::{Projects, Workspace},
};

/// Selectable menu row item representing a workspace, project, section, or dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewItem {
    /// 1-based display index of the item.
    pub idx: usize,
    /// Human-readable label displayed in the list row.
    pub label: String,
    /// Optional child view navigated to upon selecting this item.
    pub view: Option<Views>,
    /// Associated metadata payload holding underlying domain data.
    pub meta: ViewItemMeta,
}

/// Underlying entity metadata attached to a selectable view item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewItemMeta {
    /// Contains project metadata at the workspace level.
    Workspace(Projects),
    /// Contains dependency section map metadata at the project level.
    Project(DependencySectionMap),
    /// Contains dependency entry metadata at the section level.
    Section(DependencyEntry),
}

/// Represents the active hierarchical navigation view in the interactive CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Views {
    /// Top-level workspace overview displaying all member projects.
    Workspace(Box<Workspace>),
    /// Project-level view displaying all dependency sections within a project.
    Project(Projects),
    /// Section-level view displaying all dependencies in a specific section.
    Section {
        /// The dependency section category.
        section: DependencySection,
        /// Parsed dependencies within this section.
        section_dependencies: DependencySectionMap,
    },
    /// Detail view of a single dependency entry.
    Dependecy {
        /// TOML key identifying the dependency.
        key: String,
        /// Parsed dependency configuration and version details.
        entry: DependencyEntry,
    },
}

impl Display for Views {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Views::Workspace(_) => write!(f, ""),
            Views::Project(p) => write!(f, "{:}", p.name),
            Views::Section {
                section,
                section_dependencies: _,
            } => write!(f, "{section}"),
            Views::Dependecy { key, entry: _ } => write!(f, "{key}"),
        }
    }
}

impl Views {
    /// Extracts selectable child items from the current view.
    ///
    /// # Returns
    /// A vector of `ViewItem` elements for menu display.
    pub fn retreive_select_items(&self) -> Vec<ViewItem> {
        let mut map: Vec<ViewItem> = Vec::new();
        match self {
            Views::Workspace(workspace) => {
                for p in &workspace.projects {
                    if p.is_valid_project() {
                        map.push(ViewItem {
                            idx: map.len() + 1,
                            label: p.name.to_string(),
                            view: Some(Views::Project(p.clone())),
                            meta: ViewItemMeta::Workspace(p.clone()),
                        });
                    }
                }
            }
            Views::Project(project) => {
                if let ProjectDependency::Map(sections) = &project.deps {
                    for (section, dependencies) in sections {
                        if let DependencySectionMap::Map(_deps) = dependencies
                            && dependencies.is_valid_section()
                        {
                            map.push(ViewItem {
                                idx: map.len() + 1,
                                label: section.to_string(),
                                view: Some(Views::Section {
                                    section: section.clone(),
                                    section_dependencies: dependencies.clone(),
                                }),
                                meta: ViewItemMeta::Project(dependencies.clone()),
                            });
                        };
                    }
                }
            }
            Views::Section {
                section_dependencies: DependencySectionMap::Map(dependencies),
                ..
            } => {
                for (dependency, entry) in dependencies {
                    if entry.is_support() {
                        map.push(ViewItem {
                            idx: map.len() + 1,
                            label: dependency.to_string(),
                            view: Some(Views::Dependecy {
                                key: dependency.to_string(),
                                entry: entry.clone(),
                            }),
                            meta: ViewItemMeta::Section(entry.clone()),
                        });
                    }
                }
            }
            _ => {}
        };
        map
    }
}

/// Interactive CLI controller coordinating hierarchical menu navigation and terminal rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interactive {
    /// Associated workspace containing projects and dependency manifests.
    pub workspace: Workspace,
    current_view: Views,
    view_history: Vec<Views>,
}

impl Interactive {
    /// Initializes a new interactive CLI session starting at the root workspace view.
    ///
    /// # Arguments
    /// * `workspace` - The scanned and evaluated workspace model.
    pub fn new(workspace: &Workspace) -> Self {
        Self {
            workspace: workspace.clone(),
            current_view: Views::Workspace(Box::new(workspace.clone())),
            view_history: Vec::new(),
        }
    }

    fn get_history_path(&self) -> Vec<String> {
        let mut history: Vec<String> = self
            .view_history
            .iter()
            .map(|i| i.to_string())
            .filter(|i| !i.is_empty())
            .collect();
        if !self.current_view.to_string().is_empty() {
            history.push(self.current_view.to_string());
        }
        history
    }

    fn change_view(&mut self, next: Views) {
        self.view_history.push(self.current_view.clone());
        self.current_view = next.clone();
    }

    fn back_to_prev_view(&mut self) {
        if let Some(last) = self.view_history.clone().last()
            && let Some(_) = self.view_history.pop()
        {
            self.current_view = last.clone();
        }
    }

    fn get_available_inputs(&self) -> Vec<SpecialInputs> {
        match &self.current_view {
            Views::Workspace(_) => vec![SpecialInputs::Select],
            Views::Project(_) => vec![SpecialInputs::Select, SpecialInputs::ViewBack],
            Views::Section {
                section: _,
                section_dependencies: _,
            } => vec![SpecialInputs::Select, SpecialInputs::ViewBack],
            Views::Dependecy { key: _, entry: _ } => {
                vec![SpecialInputs::Select, SpecialInputs::ViewBack]
            }
        }
    }

    fn get_instruction(&self) -> String {
        match self.current_view {
            Views::Workspace(_) => String::from("Select project:"),
            Views::Project(_) => String::from("Select dependency section:"),
            Views::Section {
                section: _,
                section_dependencies: _,
            } => String::from("Select dependency:"),
            Views::Dependecy { key: _, entry: _ } => String::new(),
        }
    }

    fn format_string_with_ellipsis(&self, string: &str, max_length: usize) -> String {
        if string.chars().count() > max_length {
            let shorten: String = string.chars().take(max_length - 3).collect();
            format!("{}...", shorten)
        } else {
            string.to_string()
        }
    }

    fn generate_projects_row(&self, label: &str, deps: ProjectDependency) -> String {
        let element_size: usize = 35;
        let section_label = " Sections: ";
        let total_deps_label = " Total dependencies: ";
        let section_counter = format!(" {:} / {:} ", deps.count_valid_sections(), deps.len());
        let deps_counter = format!(
            " {:} / {:}",
            deps.count_supported_total_values(),
            deps.total_values()
        );
        let sections_fill_amount = element_size.saturating_sub(section_label.len());
        let deps_fill_amount = element_size.saturating_sub(total_deps_label.len());
        let label = format!("{:} ", label);
        format!(
            "{label}|{section_label}{section_counter:.>sections_fill_amount$}|{total_deps_label}{deps_counter:.>deps_fill_amount$}"
        )
    }

    fn generate_sections_row(&self, label: &str, deps: DependencySectionMap) -> String {
        let element_size: usize = 35;
        let total_deps_label = " Total dependencies: ";
        let deps_counter = format!(" {:} / {:}", deps.count_supported_deps(), deps.len());
        let deps_fill_amount = element_size.saturating_sub(total_deps_label.len());
        let label = format!("{:} ", label);
        format!("{label}|{total_deps_label}{deps_counter:.>deps_fill_amount$}")
    }

    fn generate_dependency_row(&self, entry: DependencyEntry) -> String {
        let latest = self.workspace.collected_deps.get(entry.package());
        let latest_value: String;
        let (is_latest, needs_migration, message) = match latest {
            Some(DependencyCollectionVersion::Latest(l)) => {
                latest_value = l.to_string();
                entry.get_dependency_row_valus(Some(l.to_string()))
            }
            Some(DependencyCollectionVersion::Error(e)) => {
                latest_value = e.to_string();
                entry.get_dependency_row_valus(None)
            }
            Some(DependencyCollectionVersion::Excluded) => {
                latest_value = "Excluded".to_string();
                entry.get_dependency_row_valus(None)
            }
            _ => {
                latest_value = "Not requested".to_string();
                entry.get_dependency_row_valus(None)
            }
        };
        let entry_value = if entry.package() == entry.toml_key() {
            entry.package().to_string()
        } else {
            format!("{:} [Key: {:}]", entry.package(), entry.toml_key())
        };
        let mut update_information: Vec<&str> = Vec::new();
        if is_latest.is_false() {
            update_information.push("Update");
        }
        if needs_migration.is_true() {
            update_information.push("Migration");
        }
        if update_information.is_empty() {
            update_information.push("No action")
        }
        let current_version_information =
            format!(" {:}", self.format_string_with_ellipsis(&message, 20));
        let latest_version_information =
            format!(" {:}", self.format_string_with_ellipsis(&latest_value, 20));
        let update_version_information = format!(" {:} required", update_information.join(" & "));
        format!(
            "{:} | {:.>20} | {:.>20} | {:.>30}",
            entry_value,
            current_version_information,
            latest_version_information,
            update_version_information
        )
    }

    fn generate_item_label(&self, item: ViewItem) -> String {
        match item.meta {
            ViewItemMeta::Workspace(w) => self.generate_projects_row(&item.label, w.deps),
            ViewItemMeta::Project(p) => self.generate_sections_row(&item.label, p),
            ViewItemMeta::Section(s) => self.generate_dependency_row(s),
        }
    }

    fn apply_view(&mut self) {
        if let Some(config) = &self.workspace.config {
            let items = self.current_view.retreive_select_items();
            let select_items: Vec<(usize, String)> = items
                .clone()
                .iter()
                .map(|i| (i.idx, self.generate_item_label(i.clone())))
                .collect();
            let mut terminal = Terminal::new(config.updater.max_lines);
            terminal.set_path(self.get_history_path());
            terminal.set_instruction(self.get_instruction());
            terminal.set_available_input(self.get_available_inputs());
            let selection = terminal.render(&select_items);
            match selection {
                Ok(select) => match select {
                    TerminalReturn::SelectItem(idx) => {
                        let item = items[idx].clone();
                        if let Some(v) = item.view {
                            self.change_view(v);
                        }
                    }
                    TerminalReturn::PrevView => self.back_to_prev_view(),
                    TerminalReturn::Update(_) => {}
                },
                Err(e) => {
                    println!("{e}");
                }
            }
            self.apply_view();
        }
    }

    /// Enters alternate terminal screen buffer and runs the interactive menu navigation loop.
    pub fn render(&mut self) {
        print!("\x1b[?1049h");
        self.apply_view();
    }
}
