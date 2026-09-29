//! This file handles every client codes.
//! Beside the default http status codes, rrdu includes an own status code which use following patterns:
//! <MODULE NUMBER ID><STATUC NUMBER>
//! Modules have there own identification numbers which are always 2 digits:
//! - Configuration => 1YXX
//! - Workspace => 2YXX
//! - Discovery => 3YXX
//! - Registry Client => 4YXX (41XX - 45XX => equivalent to HTTP status codes)
//! - Updater => 5YXX
//!
//! Status codes which have '9' at the second place are alway security relevant status codes (excluded 41XX - 45XX as these codes are responsed from http requests).
//! Security relevant codes are counted by levels:
//! - 0 => Low
//! - 2 => Relative
//! - 5 => Medium
//! - 7 => High
//! - 9 => Extrem

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Module {
    CONFIGURATION,
    WORKSPACE,
    DISCOVERY,
    REGISTRYCLIENT,
    UPDATER,
    CACHE,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Module::CONFIGURATION => write!(f, "RRDU Configuration Module"),
            Module::WORKSPACE => write!(f, "RRDU Workspace Module"),
            Module::DISCOVERY => write!(f, "RRDU Discovery Module"),
            Module::REGISTRYCLIENT => write!(f, "RRDU Registry Client Module"),
            Module::UPDATER => write!(f, "RRDU Updater Module"),
            Module::CACHE => write!(f, "RRDU Cache Module"),
        }
    }
}

impl Module {
    pub fn get_module_code(&self) -> u8 {
        match self {
            Module::CONFIGURATION => 1u8,
            Module::WORKSPACE => 2u8,
            Module::DISCOVERY => 3u8,
            Module::REGISTRYCLIENT => 4u8,
            Module::UPDATER => 5u8,
            Module::CACHE => 6u8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCodeSchema<T = ()> {
    pub module: Module,
    pub code: u32,
    pub message: String,
    pub meta: Option<T>,
}

impl<T> Display for StatusCodeSchema<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formated_status_code = format!(
            "{:} [{:}{:}]",
            self.module,
            self.module.get_module_code(),
            self.code
        );
        write!(f, "{formated_status_code} | {:}", self.message)
    }
}

impl<T> StatusCodeSchema<T> {
    pub fn with_meta(module: Module, code: u32, message: String, meta: T) -> Self {
        Self {
            module,
            code,
            message,
            meta: Some(meta),
        }
    }
}
impl<T> StatusCodeSchema<T> {
    pub fn simple(module: Module, code: u32, message: impl Into<String>) -> Self {
        Self {
            module,
            code,
            message: message.into(),
            meta: None,
        }
    }
}
