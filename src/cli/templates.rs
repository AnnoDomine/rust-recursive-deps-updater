use std::collections::HashMap;

use crate::{
    cli::table::{CellValue, Table},
    enums::*,
    workspace::project::{Projects, Workspace},
};

/// Template rendering helpers for CLI and workspace outputs.
pub struct Template;

impl Template {
    /// Renders a single-line header table to stdout.
    ///
    /// # Arguments
    /// * `value` - The text header to display in the table.
    pub fn render_single_line_table(value: String) {
        let mut table = Table::new(None);
        table.set_headers(vec![value]);
        let render = table.render();
        println!("{render}");
    }

    /// Renders a table of dependencies for a specific section.
    ///
    /// # Arguments
    /// * `dependencies` - The mapped dependency section.
    pub fn project_dependencies_table(dependencies: DependencySectionMap) {
        match dependencies {
            DependencySectionMap::Empty => {
                Template::render_single_line_table("Empty section".to_string())
            }
            DependencySectionMap::ExcludedSection => {
                Template::render_single_line_table("Excluded section".to_string())
            }
            DependencySectionMap::Map(deps) => {
                let mut table = Table::new(None);
                table.set_headers(vec!["Dependency".to_string(), "Version".to_string()]);
                for (dep, entry) in deps {
                    table.add_row(vec![
                        CellValue::String(dep),
                        CellValue::String(entry.version().to_string()),
                    ]);
                }
                let render = table.render();
                println!("{render}");
            }
        }
    }

    /// Renders all dependency sections within a project.
    ///
    /// # Arguments
    /// * `sections` - Map of dependency sections and their respective entries.
    pub fn project_dependency_section_table(
        sections: HashMap<DependencySection, DependencySectionMap>,
    ) {
        for (section, deps) in sections {
            Template::render_single_line_table(format!("[{section}]"));
            Template::project_dependencies_table(deps);
        }
    }

    /// Renders project-level dependencies or excluded status.
    ///
    /// # Arguments
    /// * `deps` - The project dependency representation.
    pub fn project_dependency_table(deps: ProjectDependency) {
        match deps {
            ProjectDependency::ExcludedProject => {
                Template::render_single_line_table("Excluded project".to_string())
            }
            ProjectDependency::Map(sections) => {
                Template::project_dependency_section_table(sections)
            }
        }
    }

    /// Renders a list of projects and their dependency tables.
    ///
    /// # Arguments
    /// * `projects` - Vector of projects to display.
    pub fn project_table(projects: Vec<Projects>) {
        for project in projects {
            Template::render_single_line_table(project.name.to_string());
            Template::project_dependency_table(project.deps);
        }
    }

    /// Renders the entire workspace and its member projects.
    ///
    /// # Arguments
    /// * `w` - The workspace to display.
    pub fn workspace_table(w: Workspace) {
        Template::project_table(w.projects);
    }
}
