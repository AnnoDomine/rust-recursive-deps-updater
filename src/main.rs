#![forbid(unsafe_code)]
use clap::Parser;

pub mod config;
pub mod globals;
pub mod workspace;

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
        config::model::RrduConfig::create_config();
        return;
    }
    println!("This application is currently in an early stage and not ready for use!");
    let _config = config::model::RrduConfig::new();
}
