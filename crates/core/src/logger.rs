use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use chrono::{DateTime, Utc};
use tracing::Level;
use tracing_subscriber::{fmt, Layer};
use tracing_subscriber::layer::SubscriberExt;

pub fn init_logger(log_directory: &Path) {

    // Create log directory
    fs::create_dir_all(log_directory).expect("Failed to create log directories");
    let log_file = Path::join(log_directory, "log.log");
    let error_file = Path::join(log_directory, "error.log");

    // Archive previous log files
    if fs::exists(&error_file).expect("Cannot determine if previous error file exists") {
        let last_write_time: DateTime<Utc> = fs::metadata(&error_file).expect("Failed to get error metadata").modified().expect("Failed to get error modified infos").into();
        let last_write_time = format!("{last_write_time}").replace(":", "-").replace(" ", "_");
        fs::rename(&error_file, Path::join(log_directory, format!("error_{}.log", last_write_time))).expect("Failed to rename old error file");
    }
    if fs::exists(&log_file).expect("Cannot determine if previous log file exists") {
        let last_write_time: DateTime<Utc> = fs::metadata(&log_file).expect("Failed to get log metadata").modified().expect("Failed to get log modified infos").into();
        let last_write_time = format!("{last_write_time}").replace(":", "-").replace(" ", "_");
        fs::rename(&log_file, Path::join(log_directory, format!("log_{}.log", last_write_time))).expect("Failed to rename old log file");
    }

    // Create log files
    let err_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(error_file)
        .unwrap();
    let debug_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_file)
        .unwrap();

    // Setup logger
    let subscriber = tracing_subscriber::Registry::default()
        .with(
            // stdout layer, to view everything in the console
            fmt::layer()
                .compact()
                .with_ansi(true)
        )
        .with(
            // log-error file, to log the errors that arise
            fmt::layer()
                .json()
                .with_writer(err_file)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::WARN))
        )
        .with(
            // log-debug file, to log the debug
            fmt::layer()
                .json()
                .with_writer(debug_file)
        );

    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing subscriber");
}