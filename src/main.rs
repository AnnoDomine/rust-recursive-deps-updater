//! `rust-recursive-deps-updater` (`rrdu`)
//!
//! A fast, safe, zero-privilege CLI and GitHub Action tool to recursively discover
//! Rust crates across workspaces, query `crates.io` for up-to-date versions, categorize
//! breaking SemVer changes, and update `Cargo.toml` manifests with full comment and formatting preservation.

#![forbid(unsafe_code)]
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use rrdu::{
    cli::templates::Template, config::rrdu_config::RrduConfig, simple_status, status_codes::Module,
    workspace::project::Workspace,
};

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
        RrduConfig::create_config();
        return;
    }
    simple_status!(
        LevelFilter::Info,
        Module::CONFIGURATION,
        100,
        "This application is currently in an early stage and not ready for use!"
    );
    let mut w = Workspace::new();
    let _ = w.read_rrdu_config(None);
    w.fetch_latest_versions();
    Template::workspace_table(w);
}
