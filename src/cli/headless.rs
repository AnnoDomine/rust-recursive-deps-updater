use std::{collections::HashMap, process};

use comfy_table::{
    ColumnConstraint::Absolute, ContentArrangement, InlineTable, Table, Width::Percentage,
    presets::UTF8_FULL,
};

use crate::{
    constants::TERMINAL_MAX_WIDTH, enums::*, simple_status, workspace::project::Workspace,
};

/// Headless rendering helpers for CLI and workspace outputs.
pub struct Headless {
    /// Associated workspace to inspect and render.
    pub workspace: Workspace,
    /// Tracks whether at least one outdated dependency was discovered.
    pub found_outdated_dependency: bool,
}

impl Headless {
    /// Creates a new `Headless` renderer instance.
    ///
    /// # Arguments
    /// * `workspace` - The workspace to inspect.
    pub fn new(workspace: Workspace) -> Self {
        Self {
            workspace,
            found_outdated_dependency: false,
        }
    }
    /// Renders all dependency sections within a project.
    ///
    /// # Arguments
    /// * `sections` - Map of dependency sections and their respective entries.
    fn headless_project_dependency_section_table(
        &mut self,
        sections: HashMap<DependencySection, DependencySectionMap>,
    ) -> InlineTable {
        let mut section_table = InlineTable::new();
        section_table.set_header(["Section", "Dependencies"]);
        section_table.set_constraints(vec![Absolute(Percentage(85)), Absolute(Percentage(15))]);
        for (section, section_deps) in sections {
            section_table.add_row([
                format!("[{:}]", section),
                format!("{:}", section_deps.len()),
            ]);
            match section_deps {
                DependencySectionMap::Empty => {
                    section_table.add_row(["Empty section", ""]);
                }
                DependencySectionMap::ExcludedSection => {
                    section_table.add_row(["Excluded section", ""]);
                }
                DependencySectionMap::Map(deps) => {
                    let mut dependency_inline_table = InlineTable::new();
                    dependency_inline_table.set_header(vec![
                        "Dependency",
                        "Version",
                        "Latest",
                        "Is latest",
                        "Needs Migration",
                    ]);
                    dependency_inline_table.set_constraints(vec![
                        Absolute(Percentage(35)),
                        Absolute(Percentage(20)),
                        Absolute(Percentage(15)),
                        Absolute(Percentage(15)),
                        Absolute(Percentage(15)),
                    ]);
                    for (_, entry) in deps {
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
                        if is_latest.is_false() {
                            self.found_outdated_dependency = true;
                        }
                        let entry_value = if entry.package() == entry.toml_key() {
                            entry.package().to_string()
                        } else {
                            format!("{:} [Key: {:}]", entry.package(), entry.toml_key())
                        };
                        dependency_inline_table.add_row(vec![
                            entry_value,
                            message.to_string(),
                            latest_value,
                            is_latest.to_string(),
                            needs_migration.to_string(),
                        ]);
                    }
                    section_table.add_inline_table(dependency_inline_table);
                }
            };
        }
        section_table
    }

    /// Renders a list of projects and their dependency tables.
    ///
    /// # Arguments
    /// * `projects` - Vector of projects to display.
    fn headless_project_table(&mut self) -> InlineTable {
        let projects = self.workspace.projects.clone();
        let mut project_table = InlineTable::new();
        project_table.set_header(["Project", "Sections", "Dependencies"]);
        project_table.set_constraints(vec![
            Absolute(Percentage(70)),
            Absolute(Percentage(15)),
            Absolute(Percentage(15)),
        ]);
        for project in projects {
            project_table.add_row([
                project.name.to_string(),
                format!("{:}", project.deps.len()),
                format!("{:}", project.deps.total_values()),
            ]);
            match project.deps {
                ProjectDependency::ExcludedProject => {
                    project_table.add_row(["Excluded project", "", ""]);
                }
                ProjectDependency::Map(sections) => {
                    project_table
                        .add_inline_table(self.headless_project_dependency_section_table(sections));
                }
            };
        }
        project_table
    }

    /// Renders the entire workspace and its member projects.
    ///
    /// # Arguments
    /// * `w` - The workspace to display.
    pub fn headless_workspace_table(&mut self) {
        let mut table = Table::new();
        table
            .force_no_tty()
            .set_width(TERMINAL_MAX_WIDTH)
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .load_style(UTF8_FULL)
            .set_header(["rrdu workspace dependency audit"]);
        let projects = self.headless_project_table();
        table.add_inline_table(projects);
        println!("{table}");
        if self.found_outdated_dependency {
            simple_status!(
                log::LevelFilter::Error,
                crate::status_codes::Module::CLI,
                400,
                "Found outdated dependencies. Please Check the audit!"
            );
            process::exit(1);
        }
    }
}
