use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::{
    cli::headless::Headless, config::rrdu_config::RrduConfig, workspace::project::Workspace,
};

/// Diagnostic report generator inspecting system environment, configuration, and workspace status.
pub struct Report;

impl Report {
    /// Collects and prints diagnostic report information including system specs, configuration, and workspace dependencies.
    pub async fn collect() {
        println!("{:#^150}", " REPORT START ");
        Self::list_rrdu_config();
        Self::rrdu_infos();
        println!("{:#^150}", " START SYSTEM ");
        Self::system();
        Self::load_workspace().await;
        println!("{:#^150}", " END SYSTEM ");
        Self::system();
        println!("{:#^150}", " REPORT END ");
    }

    fn system_information(label: &str, info: &str) {
        let info_string = format!(" {info}");
        println!("{:.<100}{:.>50}", label, info_string)
    }

    fn system() {
        if let Some(os_name) = System::name() {
            Self::system_information("OS Name: ", &os_name);
        }
        if let Some(os_version) = System::os_version() {
            Self::system_information("OS Version: ", &os_version);
        }
        Self::system_information("Kernel: ", &System::kernel_long_version());

        let mut s = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );

        s.refresh_cpu_usage();
        Self::system_information("CPU usage: ", &(s.global_cpu_usage()).to_string());

        s.refresh_memory();
        let total_memory = s.total_memory() as f64 / 1024.0 / 1024.0;
        let used_memory = s.used_memory() as f64 / 1024.0 / 1024.0;
        let free_memory = s.free_memory() as f64 / 1024.0 / 1024.0;
        Self::system_information("Total Memory: ", &format!("{total_memory:.2} MB"));
        Self::system_information("Used Memory: ", &format!("{used_memory:.2} MB"));
        Self::system_information("Available Memory: ", &format!("{free_memory:.2} MB"));
    }

    fn rrdu_infos() {
        println!("{:#^150}", " RRDU BINARY ");
        let version = env!("CARGO_PKG_VERSION");
        Self::system_information("RRDU version: ", version);
    }

    fn list_rrdu_config() {
        println!("{:#^150}", " RRDU CONFIG ");
        let config = RrduConfig::new();
        println!("{config}");
    }

    async fn load_workspace() {
        println!("{:#^150}", " RRDU WORKSPACE ");
        let mut workspace = Workspace::new();
        let _ = workspace.read_rrdu_config(None);
        // Fetch latest without progress bars
        workspace.fetch_latest_versions(true).await;
        // Load the table outside headless becouse error code
        Headless::new(workspace).headless_workspace_table(false);
    }
}
