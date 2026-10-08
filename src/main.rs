//! `rust-recursive-deps-updater` (`rrdu`)
//!
//! A fast, safe, zero-privilege CLI and GitHub Action tool to recursively discover
//! Rust crates across workspaces, query `crates.io` for up-to-date versions, categorize
//! breaking SemVer changes, and update `Cargo.toml` manifests with full comment and formatting preservation.

#![forbid(unsafe_code)]

use std::process;

use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use rrdu::{
    cli::headless::Headless, config::rrdu_config::RrduConfig, simple_status, status_codes::Module,
    workspace::project::Workspace,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Initalize an .rrduconfig file
    #[arg(long, default_value_t = false)]
    init: bool,
    // Headless scan with audit table
    #[arg(long, default_value_t = false)]
    headless: bool,
}

fn main() {
    let args = Args::parse();
    // Init simple logger
    let _ = SimpleLogger::new()
        .with_level(LevelFilter::Off)
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

    if args.init {
        RrduConfig::create_config();
        process::exit(0);
    }
    simple_status!(
        LevelFilter::Info,
        Module::CONFIGURATION,
        100,
        "This application is currently in an early stage and not ready for use!"
    );
    let mut workspace = Workspace::new();
    let _ = workspace.read_rrdu_config(None);

    if args.headless {
        workspace.fetch_latest_versions(true);
        Headless::new(workspace).headless_workspace_table();
        process::exit(0);
    }

    workspace.fetch_latest_versions(false);
    Headless::new(workspace).headless_workspace_table();
    process::exit(0);
}
