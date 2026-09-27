//! The module holds the full discovery logic
//!
//! It integrates:
//! - Crawling to the workspace and the sub folders to find the Cargo.toml
//! - Ignores hidden folders (e.g. .git) and 'target' by default
//! - It respects the .gitignore and collect the values to ignore from discovery
//!
//! The Discovery flow is:
//! - Enter a folder
//! - Check for '.rrduconfig'
//! - If found, read config and include it. Skip the rest as the config should be the 'single source of true'
//! - Search for 'Cargo.toml'
//! - If found, read it and collect the project dependencies
//! - Check for '.gitignore'
//! - If found, read '.gitignore' and collect all values for this folder discovery to skip them when discover
//! - Check for '.rrduignore'
//! - If found, read '.rrduignore' and collect all values for this folder discovery to skip them when discover
//! - Enter the next folder if it is not ignored
//! - Repeat till no sub-folder are present

use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use toml_edit::{Item, Value};

use crate::{
    config::model::ProjectConfig,
    constants::{
        CARGO_TOML_FILE_NAME, GITIGNORE_FILE_NAME, RRDUCONFIG_FILE_NAME, RRDUIGNORE_FILE_NAME,
    },
    errors::FileError,
    functions::{create_absolute_path, validate_path_traversal},
    workspace::project::Projects,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discovery {
    pub ignore: Vec<PathBuf>,
    pub has_toml: bool,
    pub current_folder: PathBuf,
    pub next_depth: Vec<PathBuf>,
    pub dir_entries: Vec<PathBuf>,
    pub found_project: Option<ProjectConfig>,
}

impl Discovery {
    pub fn new(current: PathBuf, ignored_before: Vec<PathBuf>) -> Result<Self, FileError> {
        let mut discovery = Self {
            ignore: Self::prepare_ignored(&current, ignored_before),
            has_toml: false,
            current_folder: current,
            next_depth: Vec::new(),
            dir_entries: Vec::new(),
            found_project: None,
        };
        discovery.scan_dir()?;
        discovery.scan_for_rrduconfig()?;
        discovery.generate_next_depth();
        Ok(discovery)
    }

    fn set_has_toml(&mut self, has_toml: bool) {
        self.has_toml = has_toml;
    }

    fn add_to_ignore(&mut self, path: PathBuf) {
        self.ignore.push(path);
    }

    fn remove_from_ignore(&mut self, path: PathBuf) {
        self.ignore.retain(|e| e != &path && !e.starts_with(&path));
    }

    fn scan_dir(&mut self) -> Result<(), FileError> {
        let absolute = self.retreive_absolute_path_to_scan()?;
        if let Ok(dirs) = read_dir(absolute) {
            self.dir_entries = dirs
                .filter(|e| e.is_ok())
                .map(|e| PathBuf::from(e.unwrap().file_name()))
                .collect()
        };
        Ok(())
    }

    fn prepare_ignored(current: &Path, ignored_before: Vec<PathBuf>) -> Vec<PathBuf> {
        let Some(folder_name) = current.file_name() else {
            return ignored_before;
        };

        let mut new_ignore = Vec::new();
        for ign in ignored_before {
            if let Ok(stripped) = ign.strip_prefix(folder_name)
                && !stripped.as_os_str().is_empty()
            {
                new_ignore.push(stripped.to_path_buf());
            }
        }
        new_ignore
    }

    fn add_folder(&mut self, folder: &PathBuf) {
        let mut current = self.current_folder.clone();
        current.push(folder);
        self.next_depth.push(current.clone());
    }

    fn generate_next_depth(&mut self) {
        let Ok(abs_current) = self.retreive_absolute_path_to_scan() else {
            return;
        };

        let subfolders: Vec<PathBuf> = self
            .dir_entries
            .iter()
            .filter(|entry| {
                let full_path = abs_current.join(entry);
                full_path.is_dir() && !self.ignore.contains(entry)
            })
            .cloned()
            .collect();

        for folder in subfolders {
            self.add_folder(&folder);
        }
    }

    fn scan_for_rrduconfig(&mut self) -> Result<(), FileError> {
        if self
            .dir_entries
            .contains(&PathBuf::from(RRDUCONFIG_FILE_NAME))
        {
            if let Some(name) = self.current_folder.to_str() {
                self.apply_found_project(name.to_string(), name.to_string(), true);
            };
        } else {
            return self.scan_for_toml();
        };
        Ok(())
    }

    fn parse_cargo_toml(&mut self) -> Result<ProjectConfig, FileError> {
        let mut toml_path = self.retreive_absolute_path_to_scan()?;
        toml_path.push(CARGO_TOML_FILE_NAME);
        let toml_doc = Projects::get_toml_content_from_path(toml_path)?;

        let project_path = if self.current_folder.as_os_str().is_empty() {
            "./".to_string()
        } else {
            self.current_folder.to_string_lossy().to_string()
        };

        let fallback_name = if self.current_folder.as_os_str().is_empty() {
            "Root".to_string()
        } else {
            self.current_folder
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string()
        };

        let name = match toml_doc.get("package").and_then(|p| p.get("name")) {
            Some(Item::Value(Value::String(val))) => val.value().to_string(),
            _ => fallback_name,
        };

        Ok(ProjectConfig::new(name, project_path))
    }

    fn scan_for_toml(&mut self) -> Result<(), FileError> {
        if self
            .dir_entries
            .contains(&PathBuf::from(CARGO_TOML_FILE_NAME))
        {
            self.set_has_toml(true);
            let project = self.parse_cargo_toml()?;
            self.found_project = Some(project);
        }
        self.list_ignore_by_default_folders()
    }

    fn list_ignore_by_default_folders(&mut self) -> Result<(), FileError> {
        // Run to folder in same level and collect every folder which starts with a "."
        let abs_current = self.retreive_absolute_path_to_scan()?;

        let hidden_folders: Vec<PathBuf> = self
            .dir_entries
            .iter()
            .filter(|e| {
                let is_dot = e.to_str().map(|s| s.starts_with('.')).unwrap_or(false);
                is_dot && abs_current.join(e).is_dir()
            })
            .cloned()
            .collect();

        self.ignore.extend(hidden_folders);

        if self.has_toml {
            self.add_to_ignore(PathBuf::from("target"));
            self.add_to_ignore(PathBuf::from("src"));
        }

        self.parse_gitignore()
    }

    fn get_ignore_file_content(&mut self, file: String) -> Result<String, FileError> {
        let mut path = self.retreive_absolute_path_to_scan()?;
        path.push(&file);
        std::fs::read_to_string(&path).map_err(|source| FileError::Io {
            path: path.clone(),
            source,
        })
    }

    fn get_splitted_ignore_content(&mut self, file: String) -> Result<Vec<String>, FileError> {
        let content = self.get_ignore_file_content(file.clone())?;
        Ok(content
            .split("\n")
            .map(|l| l.trim().trim_end_matches('/').to_string())
            .collect::<Vec<String>>())
    }

    fn parse_ignore_file(&mut self, content: Vec<String>) {
        for line in content {
            if !line.starts_with("#")
                && !line.is_empty()
                && let Some(pur) = line
                    .replace("!", "")
                    .split("*")
                    .collect::<Vec<&str>>()
                    .first()
                && let Ok(clean) = validate_path_traversal(Path::new(pur))
                && let Ok(absolut) = self.retreive_absolute_path_to_scan()
            {
                let unignore = line.starts_with("!");
                let mut path = absolut;
                path.push(&clean);
                if path.is_dir() && path.exists() && clean.iter().next().is_some() {
                    if unignore {
                        self.remove_from_ignore(clean);
                    } else {
                        self.add_to_ignore(clean);
                    };
                };
            }
        }
    }

    /// Parse the .gitignore file.
    /// It check every entry, if it is a folder and applies it to the ignore list.
    fn parse_gitignore(&mut self) -> Result<(), FileError> {
        if self
            .dir_entries
            .contains(&PathBuf::from(GITIGNORE_FILE_NAME))
        {
            let content = self.get_splitted_ignore_content(GITIGNORE_FILE_NAME.to_string())?;
            self.parse_ignore_file(content);
        }
        self.parse_rrduignore()
    }

    /// Parse the .rrduignore file.
    /// The file contains folders which are ignored or get removed from the ignore list when prefixed with '!'.
    fn parse_rrduignore(&mut self) -> Result<(), FileError> {
        if self
            .dir_entries
            .contains(&PathBuf::from(RRDUIGNORE_FILE_NAME))
        {
            let content = self.get_splitted_ignore_content(RRDUIGNORE_FILE_NAME.to_string())?;
            self.parse_ignore_file(content);
        }
        self.unique_ignored();
        Ok(())
    }

    fn unique_ignored(&mut self) {
        let mut unique: Vec<PathBuf> = Vec::new();
        for igno in self.ignore.clone() {
            if !unique.contains(&igno) {
                unique.push(igno);
            }
        }
        self.ignore = unique;
    }

    fn apply_found_project(&mut self, name: String, path: String, sub_config: bool) {
        let mut project = ProjectConfig::new(name, path);
        project.set_sub_config(sub_config);
        self.found_project = Some(project);
    }

    fn retreive_absolute_path_to_scan(&self) -> Result<PathBuf, FileError> {
        create_absolute_path(&self.current_folder, None)
    }
}

#[cfg(test)]
mod test_discovery {
    use super::*;

    #[test]
    fn test_prepare_ignored_root_returns_unchanged() {
        let ignored_before = vec![
            PathBuf::from(".git"),
            PathBuf::from("target"),
            PathBuf::from("src"),
        ];
        let result = Discovery::prepare_ignored(Path::new(""), ignored_before.clone());
        assert_eq!(result, ignored_before);
    }

    #[test]
    fn test_prepare_ignored_strips_matching_prefix() {
        let ignored_before = vec![
            PathBuf::from("tests/fixtures"),
            PathBuf::from("tests/fixtures/sub"),
            PathBuf::from("crates/other"),
            PathBuf::from("tests"),
            PathBuf::from("target"),
        ];
        let result = Discovery::prepare_ignored(Path::new("tests"), ignored_before);
        assert_eq!(
            result,
            vec![PathBuf::from("fixtures"), PathBuf::from("fixtures/sub"),]
        );
    }

    #[test]
    fn test_prepare_ignored_multi_level_subfolder() {
        let ignored_before = vec![
            PathBuf::from("fixtures/sub_project"),
            PathBuf::from("other_dir"),
        ];
        let result = Discovery::prepare_ignored(Path::new("tests/fixtures"), ignored_before);
        assert_eq!(result, vec![PathBuf::from("sub_project")]);
    }

    #[test]
    fn test_remove_from_ignore() {
        let mut discovery = Discovery {
            ignore: vec![
                PathBuf::from("target"),
                PathBuf::from("src"),
                PathBuf::from("tests/fixtures"),
            ],
            has_toml: false,
            current_folder: PathBuf::from(""),
            next_depth: Vec::new(),
            dir_entries: Vec::new(),
            found_project: None,
        };

        discovery.remove_from_ignore(PathBuf::from("src"));
        assert_eq!(
            discovery.ignore,
            vec![PathBuf::from("target"), PathBuf::from("tests/fixtures")]
        );

        discovery.remove_from_ignore(PathBuf::from("tests"));
        assert_eq!(discovery.ignore, vec![PathBuf::from("target")]);
    }

    #[test]
    fn test_unique_ignored() {
        let mut discovery = Discovery {
            ignore: vec![
                PathBuf::from("target"),
                PathBuf::from("src"),
                PathBuf::from("target"),
                PathBuf::from("src"),
            ],
            has_toml: false,
            current_folder: PathBuf::from(""),
            next_depth: Vec::new(),
            dir_entries: Vec::new(),
            found_project: None,
        };

        discovery.unique_ignored();
        assert_eq!(
            discovery.ignore,
            vec![PathBuf::from("target"), PathBuf::from("src")]
        );
    }

    #[test]
    fn test_discovery_root_scan() {
        let discovery =
            Discovery::new(PathBuf::from(""), Vec::new()).expect("root discovery should succeed");
        assert!(discovery.has_toml);
        assert!(discovery.found_project.is_some());
        let project = discovery.found_project.unwrap();
        assert_eq!(project.project, "rust-recursive-deps-updater");
        assert_eq!(project.path, "./");

        // Verifies hidden, default, and .rrduignore folders are ignored
        assert!(discovery.ignore.contains(&PathBuf::from(".git")));
        assert!(discovery.ignore.contains(&PathBuf::from("target")));
        assert!(discovery.ignore.contains(&PathBuf::from("src")));
        assert!(discovery.ignore.contains(&PathBuf::from("images")));

        // Verifies next_depth contains tests and excludes ignored folders
        assert!(discovery.next_depth.contains(&PathBuf::from("tests")));
        assert!(!discovery.next_depth.contains(&PathBuf::from(".git")));
        assert!(!discovery.next_depth.contains(&PathBuf::from("target")));
        assert!(!discovery.next_depth.contains(&PathBuf::from("images")));
    }

    #[test]
    fn test_discovery_sub_project_scan() {
        let discovery = Discovery::new(PathBuf::from("tests/fixtures"), Vec::new())
            .expect("fixtures discovery should succeed");
        assert!(discovery.has_toml);
        assert!(discovery.found_project.is_some());
        let project = discovery.found_project.unwrap();
        assert_eq!(project.project, "test-project");
        assert_eq!(project.path, "tests/fixtures");
        assert!(discovery.next_depth.is_empty());
    }

    #[test]
    fn test_discovery_intermediate_folder_scan() {
        let discovery = Discovery::new(PathBuf::from("tests"), Vec::new())
            .expect("tests folder discovery should succeed");
        assert!(!discovery.has_toml);
        assert!(discovery.found_project.is_none());
        assert_eq!(discovery.next_depth, vec![PathBuf::from("tests/fixtures")]);
    }
}
