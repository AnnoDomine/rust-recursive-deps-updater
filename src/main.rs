#![forbid(unsafe_code)]
use clap::Parser;

/// Modules
pub mod config;
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
    let args = Args::parse();

    if args.init {
        config::rrdu_config::RrduConfig::create_config();
        return;
    }
    println!("This application is currently in an early stage and not ready for use!");
    let mut w = workspace::project::Workspace::new();
    let _ = w.read_rrdu_config(None);
    println!("Workspace: {:#?}", w);
}
