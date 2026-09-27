//! This file holds every global constant values

pub const RRDUCONFIG_FILE_NAME: &str = ".rrduconfig";
pub const CARGO_TOML_FILE_NAME: &str = "Cargo.toml";
pub const RRDUIGNORE_FILE_NAME: &str = ".rrduignore";
pub const GITIGNORE_FILE_NAME: &str = ".gitignore";

pub const UNSUPPORTED_DEPENDECY_KEYS: [&str; 4] = ["git", "path", "registry", "workspace"];
pub const MIN_REQUIRED_DEPENDECY_KEYS: [&str; 1] = ["version"];

pub const DEPENDECY_SECTION_SUFFIX: [&str; 3] =
    ["dependecies", "dev-dependecies", "build-dependecies"];
pub const DEPENDECY_SECTION_PREFIX: [&str; 5] = [
    "workspace",
    "target",
    "dependecies",
    "dev-dependecies",
    "build-dependecies",
];
