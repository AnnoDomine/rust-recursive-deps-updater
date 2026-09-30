//! Path manipulation and validation helper functions.
//!
//! Provides functions to validate manifest file names, check against path traversal
//! vulnerabilities, and construct absolute paths relative to the current working directory.

use std::path::{Component, Path, PathBuf};

use crate::errors::{ConfigError, FileError};

/// Validates a path and expected file name.
///
/// If `path` is a directory or has no file component, appends `file_name` and returns the path.
/// If `path` already points to a file, verifies that its filename matches `file_name`.
///
/// # Arguments
/// * `path` - The path to validate.
/// * `file_name` - The expected file name (e.g. `Cargo.toml`).
///
/// # Returns
/// The normalized `PathBuf` terminating in `file_name`.
///
/// # Errors
/// Returns [`FileError::InvalidFileName`] if the path references an unexpected file name.
pub fn validate_path_and_file_name(path: &Path, file_name: &str) -> Result<PathBuf, FileError> {
    let p = if path.is_file() {
        if path.file_name().and_then(|n| n.to_str()) == Some(file_name) {
            path.to_path_buf()
        } else {
            return Err(FileError::InvalidFileName {
                path: path.to_path_buf(),
                file_name: file_name.to_string(),
            });
        }
    } else if path.file_name().and_then(|n| n.to_str()) == Some(file_name) {
        path.to_path_buf()
    } else if path.extension().is_some() && !path.is_dir() {
        return Err(FileError::InvalidFileName {
            path: path.to_path_buf(),
            file_name: file_name.to_string(),
        });
    } else {
        path.join(file_name)
    };
    Ok(p)
}

/// Validates that the provided path does not contain traversal components or root references.
///
/// Rejects any path component matching `..`, `/` (root), or Windows drive prefixes (`C:\`).
///
/// # Arguments
/// * `path` - The path to inspect for traversal attempts.
///
/// # Returns
/// A safe `PathBuf` if validation succeeds.
///
/// # Errors
/// Returns [`ConfigError::InsecurePath`] if any traversal or root component is detected.
pub fn validate_path_traversal(path: &Path) -> Result<PathBuf, ConfigError> {
    for path_comp in path.components() {
        match path_comp {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ConfigError::InsecurePath(path.to_path_buf()));
            }
            _ => {}
        }
    }
    Ok(path.to_path_buf())
}

/// Constructs an absolute path from the current working directory.
///
/// Optionally appends and validates the provided `file_name`.
///
/// # Arguments
/// * `path` - Relative path to resolve against the working directory.
/// * `file_name` - Optional expected file name to attach and validate.
///
/// # Returns
/// The resolved absolute `PathBuf`.
///
/// # Errors
/// Returns [`FileError::Io`] if retrieving the current working directory fails.
/// Returns [`FileError::InvalidFileName`] if `file_name` validation fails.
pub fn create_absolute_path(path: &Path, file_name: Option<&str>) -> Result<PathBuf, FileError> {
    let mut abolute_path = match std::env::current_dir() {
        Ok(current) => current,
        Err(e) => {
            return Err(FileError::Io {
                path: path.to_path_buf(),
                source: e,
            });
        }
    };
    abolute_path.push(path);
    match file_name {
        Some(f) => validate_path_and_file_name(&abolute_path, f),
        None => Ok(abolute_path),
    }
}

#[cfg(test)]
mod test_path_validation {
    use super::*;

    #[test]
    fn test_validate_path_valid_relative_paths() {
        let valid_paths = [
            "./",
            "./Cargo.toml",
            "./crates/my_crate",
            "crates/sub_project/Cargo.toml",
            "sub_project",
        ];

        for path in valid_paths {
            assert!(
                validate_path_traversal(Path::new(path)).is_ok(),
                "Expected valid path '{path}' to pass validation"
            );
        }
    }

    #[test]
    fn test_validate_path_rejects_parent_traversal() {
        let traversal_paths = [
            "../",
            "../Cargo.toml",
            "./crates/../../secret",
            "crates/../..",
            "crates/sub/../../../etc",
        ];

        for path in traversal_paths {
            assert!(
                validate_path_traversal(Path::new(path)).is_err(),
                "Expected traversal path '{path}' to fail validation"
            );
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_validate_path_rejects_root_and_prefix() {
        let absolute_paths = ["/", "/etc/passwd", "/home/user/project"];

        for path in absolute_paths {
            assert!(
                validate_path_traversal(Path::new(path)).is_err(),
                "Expected absolute/root path '{path}' to fail validation"
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_validate_path_rejects_root_and_prefix() {
        let absolute_paths = ["C:\\", "C:\\Windows\\System32", "D:\\"];

        for path in absolute_paths {
            assert!(
                validate_path_traversal(Path::new(path)).is_err(),
                "Expected absolute/root path '{path}' to fail validation"
            );
        }
    }

    #[test]
    fn test_validate_path_and_file_name() {
        let dir = PathBuf::from("./crates/foo");
        assert_eq!(
            validate_path_and_file_name(&dir, "Cargo.toml").unwrap(),
            PathBuf::from("./crates/foo/Cargo.toml")
        );

        let file = PathBuf::from("./Cargo.toml");
        assert_eq!(
            validate_path_and_file_name(&file, "Cargo.toml").unwrap(),
            PathBuf::from("./Cargo.toml")
        );

        let invalid = PathBuf::from("./other.toml");
        assert!(validate_path_and_file_name(&invalid, "Cargo.toml").is_err());
    }

    #[test]
    fn test_create_absolute_path_with_tempfile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let temp_path = temp.path();

        let resolved =
            create_absolute_path(temp_path, Some("Cargo.toml")).expect("create absolute path");
        assert_eq!(resolved, temp_path.join("Cargo.toml"));

        let without_file =
            create_absolute_path(temp_path, None).expect("create absolute path without file");
        assert_eq!(without_file, temp_path);
    }
}
