#![forbid(unsafe_code)]
use clap::Parser;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use crate::status_codes::Module;

/// Modules
pub mod config;
pub mod registry;
pub mod workspace;

/// Globals
pub mod constants;
pub mod enums;
pub mod errors;
pub mod functions;
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
