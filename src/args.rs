use clap::Parser;

#[derive(Parser, Debug, Copy, Clone)]
#[command(version, about, long_about = None)]
/// A fast, zero-privilege CLI tool and GitHub Action to inspect, navigate, and recursively update dependencies across Rust workspaces and multi-crate repositories.
pub struct Args {
    /// Initalize an .rrduconfig file
    #[arg(long, default_value_t = false)]
    pub init: bool,
    /// Headless scan with audit table. This can be piped (e.g. `rrdu --headless | tee rrdu_headless.txt`)
    #[arg(long, default_value_t = false)]
    pub headless: bool,
    /// Create report output for GitHub issue. This can be piped (e.g. `rrdu --report | tee rrdu_report.txt`)
    #[arg(long, default_value_t = false)]
    pub report: bool,
    /// Enable trace file
    #[arg(long, default_value_t = false)]
    pub trace: bool,
}
