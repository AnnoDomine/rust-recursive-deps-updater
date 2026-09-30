//! `rust-recursive-deps-updater` (`rrdu`)
//!
//! A fast, safe, zero-privilege CLI and GitHub Action tool to recursively discover
//! Rust crates across workspaces, query `crates.io` for up-to-date versions, categorize
//! breaking SemVer changes, and update `Cargo.toml` manifests with full comment and formatting preservation.

#![forbid(unsafe_code)]
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use crate::status_codes::Module;

/// Workspace configuration loading, generation, and validation.
pub mod config;
/// Crates.io index/API communication, cache, and response models.
pub mod registry;
/// Workspace project discovery, TOML parsing, and dependency mapping.
pub mod workspace;

/// Global application constants.
pub mod constants;
/// Domain enums and data types.
pub mod enums;
/// Error types and error codes.
pub mod errors;
/// Path and validation utility functions.
pub mod functions;
/// Structured status code definitions and logging macros.
pub mod status_codes;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Initalize an .rrduconfig file
    #[arg(long, default_value_t = false)]
    init: bool,
}

fn main() {
    // Init simple logger
    let _ = SimpleLogger::new()
        .env()
        .with_colors(true)
        .with_timestamp_format(time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init();

    simple_status!(
        LevelFilter::Trace,
        Module::CONFIGURATION,
        100,
        "Start rust-recursive-deps-updater"
    );
    let args = Args::parse();

    if args.init {
        config::rrdu_config::RrduConfig::create_config();
        return;
    }
    simple_status!(
        LevelFilter::Info,
        Module::CONFIGURATION,
        100,
        "This application is currently in an early stage and not ready for use!"
    );
    let mut w = workspace::project::Workspace::new();
    let _ = w.read_rrdu_config(None);
    w.fetch_latest_versions();
}
