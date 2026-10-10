//! `rust-recursive-deps-updater` (`rrdu`)
//!
//! A fast, safe, zero-privilege CLI and GitHub Action tool to recursively discover
//! Rust crates across workspaces, query `crates.io` for up-to-date versions, categorize
//! breaking SemVer changes, and update `Cargo.toml` manifests with full comment and formatting preservation.

#![forbid(unsafe_code)]

use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{io::IsTerminal, process};

use rrdu::{
    args::Args,
    cli::{headless::Headless, interactive::Interactive},
    config::rrdu_config::RrduConfig,
    report::report::Report,
    simple_status,
    status_codes::Module,
    workspace::project::Workspace,
};

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
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

    // Only allow pipe if not headless or report mode
    if !args.headless
        && !args.report
        && (!std::io::stdin().is_terminal() || !std::io::stdout().is_terminal())
    {
        simple_status!(
            LevelFilter::Error,
            Module::INTERACTIVITY,
            500,
            "Interactive mode requires an interactive terminal (TTY) and cannot be piped. Use 'rrdu --headless' instead."
        );
        process::exit(1);
    }

    simple_status!(
        LevelFilter::Trace,
        Module::CONFIGURATION,
        100,
        "Start rust-recursive-deps-updater"
    );

    // Report mode
    if args.report {
        Report::collect().await;
        process::exit(0);
    }

    // Init mode
    if args.init {
        RrduConfig::create_config();
        process::exit(0);
    }

    // Preload workspace
    let mut workspace = Workspace::new();
    let _ = workspace.read_rrdu_config(None);
    workspace.fetch_latest_versions(args.headless).await;

    // Headless mode
    if args.headless {
        Headless::new(workspace).headless_workspace_table(args.headless);
        process::exit(0);
    }

    // Interactive mode
    let mut interactive = Interactive::new(&workspace);
    interactive.render();
    process::exit(0);
}
