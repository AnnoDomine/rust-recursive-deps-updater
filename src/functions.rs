use std::path::{Component, Path, PathBuf};

use crate::errors::{ConfigError, FileError};

/// Validates a path and file name
///
/// Rise an error if file name is not the requested.
/// If the path does not have a file attached, add the requested file name and return the full path incl file name.
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

/// Validates the provided path does not includes traversal or is called from root.
///
/// Return:
/// * PathBuf: If the path is valid
/// * Error: If the path is called from root or includes traversal
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

/// Simple global function to retreive the path constructed from the execution path
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
