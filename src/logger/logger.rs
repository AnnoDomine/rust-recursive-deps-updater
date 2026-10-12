//! Asynchronous trace file logger with background Tokio worker and log rotation.

use clap::Parser;
use log::LevelFilter;
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use time::UtcDateTime;
use tokio::{
    fs::{File, OpenOptions, create_dir_all, read_dir, rename},
    io::AsyncWriteExt,
    sync::mpsc,
};

use crate::{args::Args, constants::RRDU_LOG, functions::create_absolute_path};

/// Background file worker handling path resolution, file writing, and log rotation.
#[derive(Clone)]
pub struct LogWorker {
    /// Base directory path where log files and archives are stored.
    pub path: PathBuf,
    /// Maximum file size in bytes before rotating the log file.
    pub max_size: u64,
    /// Optional cached count of existing archived log files.
    pub archive_amount: Option<usize>,
}

/// Asynchronous logger dispatching log entries via an in-memory Tokio MPSC channel.
#[derive(Clone)]
pub struct Logger {
    /// The associated worker configuration and file manager.
    pub worker: LogWorker,
    sender: mpsc::Sender<String>,
}

impl Default for LogWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl LogWorker {
    /// Creates a new `LogWorker` with default audit path and 1 MB max file size.
    ///
    /// # Returns
    /// A new `LogWorker` instance.
    pub fn new() -> Self {
        let path = Path::new("audit");

        // 1 MB
        let max_size: u64 = 1024 * 1024;

        // Generate absolut path
        let log_file = match create_absolute_path(path, None) {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        Self {
            path: log_file,
            max_size,
            archive_amount: None,
        }
    }

    async fn get_log_archive_amount(&self) -> usize {
        let log_file_string = "rrdu";
        let log_path = &self.path;
        let dir_content = read_dir(log_path).await;
        match dir_content {
            Err(_) => 0,
            Ok(mut entries) => {
                let mut count = 0;
                while let Ok(Some(e)) = entries.next_entry().await {
                    if let Some(f) = e.file_name().to_str()
                        && f != RRDU_LOG
                        && f.starts_with(log_file_string)
                    {
                        count += 1;
                    }
                }
                count
            }
        }
    }

    async fn create_log_file_archive_name(&self) -> String {
        let count = match &self.archive_amount {
            Some(c) => c + 1,
            None => self.get_log_archive_amount().await + 1,
        };
        format!("rrdu_{count}.log")
    }

    async fn split_log(&self, file: File) -> std::io::Result<File> {
        let file_size = file.metadata().await?.len();

        if file_size > self.max_size {
            let log_file = self.get_log_file_path().await;
            file.sync_all().await?;
            drop(file);
            let archive_name = self.get_log_file_archive_path().await;
            rename(&log_file, archive_name).await?;
            Ok(OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .await?)
        } else {
            Ok(file)
        }
    }

    async fn get_log_file_archive_path(&self) -> PathBuf {
        let mut p = PathBuf::from(&self.path);
        let file_name = self.create_log_file_archive_name().await;
        p.push(file_name);
        p
    }

    async fn get_log_file_path(&self) -> PathBuf {
        let mut p = PathBuf::from(&self.path);
        p.push(RRDU_LOG);
        p
    }

    async fn get_log_file(&self) -> std::io::Result<File> {
        let path = Path::new(&self.path);
        if !path.exists() {
            create_dir_all(path).await?;
        }
        let log_file = self.get_log_file_path().await;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .await?;

        self.split_log(file).await
    }

    /// Sets the base log directory path.
    ///
    /// # Arguments
    /// * `path` - New base directory path for log files.
    ///
    /// # Returns
    /// A reference to `Self`.
    pub async fn set_log_path(&mut self, path: PathBuf) -> &Self {
        self.path = match create_absolute_path(&path, None) {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        self
    }

    /// Sets the maximum log file size in bytes before file rotation is triggered.
    ///
    /// # Arguments
    /// * `max_size` - Maximum log file size in bytes.
    ///
    /// # Returns
    /// A reference to `Self`.
    pub async fn set_max_size(&mut self, max_size: u64) -> &Self {
        self.max_size = max_size;

        self
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    /// Creates a new `Logger` and spawns its background log writer task.
    ///
    /// # Returns
    /// A new `Logger` instance.
    pub fn new() -> Self {
        // Generate worker and clone for tokio spawn
        let worker = LogWorker::new();
        let worker_clone = worker.clone();

        // Spawn task and create mpsc channel
        let sender = Self::spawn(worker_clone);

        Self { worker, sender }
    }

    fn spawn(worker: LogWorker) -> mpsc::Sender<String> {
        let (sender, mut receiver) = mpsc::channel::<String>(5000);
        let worker_clone = worker.clone();

        tokio::spawn(async move {
            let task: std::io::Result<()> = async {
                let mut log = worker_clone.get_log_file().await?;

                while let Some(entry) = receiver.recv().await {
                    log.write_all(entry.as_bytes()).await?;
                }

                Ok(())
            }
            .await;

            if let Err(e) = task {
                eprintln!("Error in background logger: {:?}", e);
            }
        });
        sender
    }

    /// Generates a standardized trace message with UTC timestamp and severity level.
    ///
    /// # Arguments
    /// * `level` - Severity level of the log entry.
    /// * `message` - Log message payload.
    ///
    /// # Returns
    /// A formatted trace string.
    pub fn generate_trace_message(level: LevelFilter, message: &str) -> String {
        let timestamp = UtcDateTime::now();
        format!("[{timestamp}] {level} {message}\n")
    }

    /// Asynchronously enqueues a log message to the background file writer if trace or report mode is active.
    ///
    /// # Arguments
    /// * `level` - Severity level of the log entry.
    /// * `message` - Log message payload.
    pub fn save_log(&self, level: LevelFilter, message: &str) {
        // Only trace to file, if arg trace or report is setted.
        let args = Args::try_parse();
        if let Ok(a) = args
            && (a.trace || a.report)
        {
            let entry = Self::generate_trace_message(level, message);
            if let Err(e) = self.sender.try_send(entry.clone()) {
                eprintln!("Error while logging: {:?}", e);
            }
        }
    }

    /// Prints a formatted log entry to stdout and enqueues it to the trace file writer.
    ///
    /// # Arguments
    /// * `level` - Severity level of the log entry.
    /// * `message` - Log message payload.
    pub fn print_and_save_log(&self, level: LevelFilter, message: &str) {
        // Print the entry and save trace file.
        let entry = Self::generate_trace_message(level, message);
        println!("{:}", entry);
        self.save_log(level, message);
    }
}

/// Global thread-safe singleton instance of `Logger`.
pub static LOGGER: OnceLock<Logger> = OnceLock::new();

/// Retrieves or initializes the global `Logger` singleton instance.
///
/// # Returns
/// A cloned handle to the active `Logger`.
pub fn trace() -> Logger {
    LOGGER.get_or_init(Logger::new).clone()
}
