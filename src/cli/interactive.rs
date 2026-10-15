use std::{collections::HashMap, fmt::Display};

use crate::{
    enums::*,
    interactivity::terminal::{SpecialInputs, Terminal, TerminalReturn},
    logger::status_codes::Module,
    meta_status,
    registry::client::Client,
    simple_status,
    updater::engine::{UpdateEngine, UpdaterInformation},
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
    Workspace(Box<Projects>),
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
    Project(Box<Projects>),
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
        /// Updater information to retreive the information for updating a dependency. None if the dependency is not updateable.
        updater_information: Option<UpdaterInformation>, // String is placeholder till logic is integrated
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
            Views::Dependecy {
                key,
                entry: _,
                updater_information: _,
            } => write!(f, "{key}"),
        }
    }
}

impl Views {
    /// Extracts selectable child items from the current view.
    ///
    /// # Returns
    /// A vector of `ViewItem` elements for menu display.
    pub fn retreive_select_items(&self, update_engine: &Option<UpdateEngine>) -> Vec<ViewItem> {
        simple_status!(
            log::LevelFilter::Trace,
            Module::INTERACTIVITY,
            100,
            "Generate view items"
        );
        let mut map: Vec<ViewItem> = Vec::new();
        match self {
            Views::Workspace(workspace) => {
                for p in &workspace.projects {
                    if p.is_valid_project() {
                        map.push(ViewItem {
                            idx: map.len() + 1,
                            label: p.name.to_string(),
                            view: Some(Views::Project(Box::new(p.clone()))),
                            meta: ViewItemMeta::Workspace(Box::new(p.clone())),
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
                section,
                section_dependencies: DependencySectionMap::Map(dependencies),
            } => {
                for (dependency, entry) in dependencies {
                    if entry.is_support() {
                        let updater_information: Option<UpdaterInformation> = update_engine
                            .as_ref()
                            .map(|ue| ue.get_updater_information(section.clone(), entry));

                        simple_status!(
                            log::LevelFilter::Trace,
                            Module::INTERACTIVITY,
                            100,
                            format!(
                                "Generate section item\n{:?}\n{:?}",
                                update_engine, updater_information
                            )
                        );
                        map.push(ViewItem {
                            idx: map.len() + 1,
                            label: dependency.to_string(),
                            view: Some(Views::Dependecy {
                                key: dependency.to_string(),
                                entry: entry.clone(),
                                updater_information,
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
    /// Map of fetched dependency collection versions keyed by package name.
    pub dependencies: HashMap<String, DependencyCollectionVersion>,
    /// Optional registry client instance used to fetch index, API, and audit information.
    pub client: Option<Client>,
    /// Project integrated update engine
    active_update_engine: Option<UpdateEngine>,
}

impl Display for Interactive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Workspace: {:#?}\nCurrent view: {:#?}\nView history: {:#?}",
            self.workspace, self.current_view, self.view_history
        )
    }
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
            dependencies: workspace.collected_deps.clone(),
            client: workspace.client.clone(),
            active_update_engine: None,
        }
    }

    /// Constructs the current navigation breadcrumb trail from the view history.
    ///
    /// # Returns
    /// Vector of string labels representing the hierarchical path.
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

    fn set_update_engine(&mut self, view: &Views) {
        match view {
            Views::Workspace(_) => self.active_update_engine = None,
            Views::Project(p) => self.active_update_engine = p.updater.clone(),
            _ => {}
        }
    }

    /// Transitions to a child view, saving the active view onto the history stack.
    ///
    /// # Arguments
    /// * `next` - The destination view to activate.
    fn change_view(&mut self, next: Views) {
        self.view_history.push(self.current_view.clone());
        self.current_view = next.clone();
        self.set_update_engine(&next);
    }

    /// Pops the previous view from history and restores it as the current view.
    fn back_to_prev_view(&mut self) {
        if let Some(last) = self.view_history.clone().last()
            && let Some(_) = self.view_history.pop()
        {
            self.current_view = last.clone();
            self.set_update_engine(last);
        }
    }

    /// Computes the valid interactive keystrokes and operations supported by the current view.
    ///
    /// # Returns
    /// Vector of enabled `SpecialInputs` for this view level.
    fn get_available_inputs(&self) -> Vec<SpecialInputs> {
        match &self.current_view {
            Views::Workspace(_) => vec![SpecialInputs::Select],
            Views::Project(_) => vec![SpecialInputs::Select, SpecialInputs::ViewBack],
            Views::Section {
                section: _,
                section_dependencies: _,
            } => vec![
                SpecialInputs::ViewBack,
                SpecialInputs::Select,
                SpecialInputs::Update,
            ],
            Views::Dependecy {
                key: _,
                entry: _,
                updater_information,
            } => {
                if updater_information.is_some() {
                    vec![SpecialInputs::ViewBack, SpecialInputs::Update]
                } else {
                    vec![SpecialInputs::ViewBack]
                }
            }
        }
    }

    /// Returns the prompt instruction text appropriate for the current view.
    ///
    /// # Returns
    /// An instruction string describing the active selection context.
    fn get_instruction(&self) -> String {
        match self.current_view {
            Views::Workspace(_) => String::from("Select project:"),
            Views::Project(_) => String::from("Select dependency section:"),
            Views::Section {
                section: _,
                section_dependencies: _,
            } => String::from("Select dependency:"),
            Views::Dependecy {
                key: _,
                entry: _,
                updater_information: _,
            } => String::new(),
        }
    }

    /// Truncates a string to a maximum character length, appending an ellipsis if truncated.
    ///
    /// # Arguments
    /// * `string` - The input string slice to format.
    /// * `max_length` - Maximum permitted character count.
    ///
    /// # Returns
    /// Formatted string fitted within the length constraint.
    fn format_string_with_ellipsis(&self, string: &str, max_length: usize) -> String {
        if string.chars().count() > max_length {
            // We remove 4 chars. Else we have an UI break.
            let shorten: String = string.chars().take(max_length - 4).collect();
            format!("{}...", shorten.trim_end())
        } else {
            string.to_string()
        }
    }

    /// Formats a workspace-level project row showing section counts and total dependencies.
    ///
    /// # Arguments
    /// * `label` - Project name label.
    /// * `deps` - Parsed project dependency collection.
    ///
    /// # Returns
    /// Formatted row string padded to the column layout.
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

    /// Formats a project-level section row showing supported and total dependency counts.
    ///
    /// # Arguments
    /// * `label` - Section name label.
    /// * `deps` - Dependency map for this section.
    ///
    /// # Returns
    /// Formatted row string padded to the column layout.
    fn generate_sections_row(&self, label: &str, deps: DependencySectionMap) -> String {
        let element_size: usize = 35;
        let total_deps_label = " Total dependencies: ";
        let deps_counter = format!(" {:} / {:}", deps.count_supported_deps(), deps.len());
        let deps_fill_amount = element_size.saturating_sub(total_deps_label.len());
        let label = format!("{:} ", label);
        format!("{label}|{total_deps_label}{deps_counter:.>deps_fill_amount$}")
    }

    /// Formats a dependency row displaying current version, latest version, SemVer status, and update markers.
    ///
    /// # Arguments
    /// * `entry` - The dependency entry from Cargo.toml.
    /// * `is_latest` - Whether the dependency is already at the newest version.
    /// * `needs_migration` - Whether upgrading requires a major/breaking SemVer migration.
    /// * `message` - Formatted status or error message.
    /// * `latest_value` - Latest version string or exclusion/error note.
    ///
    /// # Returns
    /// A formatted row string padded to the terminal width.
    fn generate_dependency_row(
        &self,
        entry: DependencyEntry,
        is_latest: Boolean,
        needs_migration: Boolean,
        message: String,
        latest_value: String,
    ) -> String {
        let mut entry_value = if entry.package() == entry.toml_key() {
            entry.package().to_string()
        } else {
            format!("{:} [Key: {:}]", entry.package(), entry.toml_key())
        };
        let mut update_information: Vec<&str> = Vec::new();
        if is_latest.is_false() {
            entry_value.push('*');
            update_information.push("Update");
        }
        if needs_migration.is_true() {
            update_information.push("Migration");
        }
        if update_information.is_empty() {
            update_information.push("No action")
        }
        let current_version_information =
            format!(" {:}", self.format_string_with_ellipsis(&message, 23));
        let latest_version_information =
            format!(" {:}", self.format_string_with_ellipsis(&latest_value, 23));
        let update_version_information = format!(" {:} required", update_information.join(" & "));
        format!(
            "{:} | {:.>25} | {:.>25} | {:.>30}",
            entry_value,
            current_version_information,
            latest_version_information,
            update_version_information
        )
    }

    fn update_in_history(&mut self, updater_information: UpdaterInformation) {
        let Some(latest) = updater_information.get_latest() else {
            return;
        };

        let mut updated_section: Option<DependencySectionMap> = None;
        let mut updated_project: Option<Projects> = None;

        match &mut self.current_view {
            Views::Dependecy { entry, .. } => {
                entry.update_in_entry(&latest);
            }
            Views::Section {
                section,
                section_dependencies,
            } => {
                if let DependencySectionMap::Map(deps) = section_dependencies
                    && let Some(entry) = deps.get_mut(&updater_information.toml_key)
                    && section == &updater_information.section
                {
                    entry.update_in_entry(&latest);
                    updated_section = Some(section_dependencies.clone());
                }
            }
            _ => {}
        }

        for view in self.view_history.iter_mut().rev() {
            match view {
                Views::Project(p) => {
                    if let Some(sec_deps) = &updated_section {
                        p.deps.update_in_project(
                            updater_information.section.clone(),
                            sec_deps.clone(),
                        );
                    }
                    if let Some(engine) = &self.active_update_engine {
                        p.updater = Some(engine.clone());
                    }
                    updated_project = Some((**p).clone());
                }
                Views::Workspace(w) => {
                    if let Some(proj) = &updated_project {
                        w.update_in_workspace(proj);
                    }
                }
                Views::Section {
                    section,
                    section_dependencies,
                } => {
                    if let DependencySectionMap::Map(deps) = section_dependencies
                        && let Some(entry) = deps.get_mut(&updater_information.toml_key)
                        && section == &updater_information.section
                    {
                        entry.update_in_entry(&latest);
                        updated_section = Some(section_dependencies.clone());
                    }
                }
                _ => {}
            }
        }
        if let Some(proj) = &updated_project {
            self.workspace.update_in_workspace(proj);
        }
    }

    /// Converts view items into selectable row tuples with update capability flags.
    ///
    /// # Arguments
    /// * `items` - Slice of view items for the active menu level.
    ///
    /// # Returns
    /// Vector of `(index, formatted_label, is_updateable)` tuples for terminal rendering.
    fn parse_updateable_dependencies(
        &self,
        items: &Vec<ViewItem>,
    ) -> Vec<(usize, String, Option<UpdaterInformation>)> {
        simple_status!(
            log::LevelFilter::Trace,
            Module::INTERACTIVITY,
            100,
            "Parse items"
        );
        let mut selectable_items: Vec<(usize, String, Option<UpdaterInformation>)> = Vec::new();
        let deps = &self.dependencies;
        for item in items {
            match item.meta.clone() {
                ViewItemMeta::Workspace(w) => selectable_items.push((
                    item.idx,
                    self.generate_projects_row(&item.label, w.deps),
                    None,
                )),
                ViewItemMeta::Project(p) => selectable_items.push((
                    item.idx,
                    self.generate_sections_row(&item.label, p),
                    None,
                )),
                ViewItemMeta::Section(s) => {
                    let latest = deps.get(s.package());
                    let latest_value: String;
                    let (is_latest, needs_migration, message) = match latest {
                        Some(DependencyCollectionVersion::Latest(l)) => {
                            latest_value = l.to_string();
                            s.get_dependency_row_valus(Some(l.to_string()))
                        }
                        Some(DependencyCollectionVersion::Error(e)) => {
                            latest_value = e.to_string();
                            s.get_dependency_row_valus(None)
                        }
                        Some(DependencyCollectionVersion::Excluded) => {
                            latest_value = "Excluded".to_string();
                            s.get_dependency_row_valus(None)
                        }
                        _ => {
                            latest_value = "Not requested".to_string();
                            s.get_dependency_row_valus(None)
                        }
                    };
                    let update_information: Option<UpdaterInformation> =
                        if let Some(Views::Dependecy {
                            updater_information,
                            ..
                        }) = &item.view
                            && let Some(information) = &updater_information
                            && is_latest.is_false()
                            && let Some(DependencyCollectionVersion::Latest(l)) = latest
                            && let Some(version) = DependencyEntry::parse_base_version(l)
                        {
                            let mut info = information.clone();
                            info.set_latest(&version.to_string());
                            Some(info)
                        } else {
                            None
                        };
                    simple_status!(
                        log::LevelFilter::Trace,
                        Module::INTERACTIVITY,
                        100,
                        format!(
                            "Generate dependency item: {:}\n{:?}",
                            s.package(),
                            update_information
                        )
                    );
                    selectable_items.push((
                        item.idx,
                        self.generate_dependency_row(
                            s,
                            is_latest,
                            needs_migration,
                            message,
                            latest_value,
                        ),
                        update_information,
                    ));
                }
            };
        }
        selectable_items
    }

    /// Dispatches the active view to the terminal renderer and handles user interaction returns.
    fn apply_view(&mut self) {
        if let Some(config) = &self.workspace.config {
            let items = self
                .current_view
                .retreive_select_items(&self.active_update_engine);
            let select_items: Vec<(usize, String, Option<UpdaterInformation>)> =
                self.parse_updateable_dependencies(&items);
            let mut terminal = Terminal::new(config.updater.max_lines);
            terminal.set_path(self.get_history_path());
            terminal.set_instruction(self.get_instruction());
            terminal.set_available_input(self.get_available_inputs());
            let selection = match &self.current_view {
                Views::Dependecy {
                    key: _,
                    entry,
                    updater_information,
                } => terminal.render_dependency(
                    entry.clone(),
                    self.dependencies.get(entry.package()),
                    self.client.clone(),
                    &mut updater_information.clone(),
                ),
                _ => terminal.render(&select_items),
            };
            match selection {
                Ok(select) => match select {
                    TerminalReturn::SelectItem(idx) => {
                        let item = items[idx].clone();
                        if let Some(v) = item.view {
                            self.change_view(v);
                        }
                    }
                    TerminalReturn::PrevView => self.back_to_prev_view(),
                    TerminalReturn::Update(information) => {
                        if let Some(engine) = &mut self.active_update_engine
                            && let Some(info) = &information
                        {
                            engine.update_dependency(info);
                            match engine.save() {
                                Ok(_) => {
                                    simple_status!(
                                        log::LevelFilter::Trace,
                                        Module::INTERACTIVITY,
                                        200,
                                        "Dependency updated. Starting view updates."
                                    );
                                    self.update_in_history(info.clone());
                                }
                                Err(e) => {
                                    meta_status!(
                                        log::LevelFilter::Error,
                                        Module::INTERACTIVITY,
                                        500,
                                        "Dependency updated failes.",
                                        e
                                    );
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    meta_status!(
                        log::LevelFilter::Error,
                        Module::CLI,
                        501,
                        "Terminal rendering failed",
                        e
                    );
                }
            }
            self.apply_view();
        }
    }

    /// Enters alternate terminal screen buffer and runs the interactive menu navigation loop.
    pub fn render(&mut self) {
        print!("\x1b[?1049h");
        simple_status!(
            log::LevelFilter::Debug,
            Module::INTERACTIVITY,
            100,
            format!("Render interactive CLI: {:#?}", &self.workspace.config)
        );
        self.apply_view();
    }
}
