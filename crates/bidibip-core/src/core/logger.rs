use chrono::{DateTime, Utc};
use serenity::all::{ChannelId, Http, Mentionable, RoleId};
use std::fmt::{Debug};
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::field::Field;
use tracing::{Level, Subscriber};
use tracing_subscriber::field::Visit;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, Layer};
use crate::core::config::Config;

pub struct DiscordLogConnector {
    connected_log_channel: RwLock<Option<(Arc<Http>, ChannelId)>>,
}

impl DiscordLogConnector {
    pub fn new() -> Self {
        Self {
            connected_log_channel: Default::default(),
        }
    }

    pub fn init_for_channel(&self, channel: ChannelId, http: Arc<Http>) {
        *self.connected_log_channel.write().unwrap() = Some((http, channel));
    }
}

pub struct ChannelWriter {
    connector: Arc<DiscordLogConnector>,
    config: Arc<Config>,
}

pub struct FieldMessageVisitor(String);
impl Visit for FieldMessageVisitor {
    fn record_debug(&mut self, _: &Field, value: &dyn Debug) {
        self.0 = format!("{value:?}");
    }
}

impl<S> Layer<S> for ChannelWriter
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some((http, channel)) = &*self.connector.connected_log_channel.read().unwrap() {
            let level = *event.metadata().level();

            if level == Level::INFO || level == Level::WARN || level == Level::ERROR {
                let target = event.metadata().target().to_string();
                let line = match event.metadata().line() {
                    None => { String::new() }
                    Some(line) => { format!(":{line}") }
                };

                let mut visitor = FieldMessageVisitor(String::new());
                event.record(&mut visitor);
                let http = http.clone();
                let channel = *channel;
                let support_role = self.config.roles.support;
                tokio::spawn(async move {
                    match level {
                        Level::INFO => {
                            channel.say(&http, format!(":green_circle: `{target}{line}` {}", &visitor.0[0..std::cmp::min(900, visitor.0.len())])).await.expect("Failed to send info log");
                        }
                        Level::WARN => {
                            channel.say(&http, format!(":yellow_circle: `{target}{line}` {}", &visitor.0[0..std::cmp::min(900, visitor.0.len())])).await.expect("Failed to send warning log");
                        }
                        Level::ERROR => {
                            channel.say(&http, format!(":red_circle: `{target}{line}` {} {}", RoleId::from(support_role).mention(), &visitor.0[0..std::cmp::min(900, visitor.0.len())])).await.expect("Failed to send error log");
                        }
                        _ => {}
                    }
                });
            }
        }
    }
}

pub fn init_logger(config: Arc<Config>) -> Arc<DiscordLogConnector> {
    let log_directory = &config.log_directory;

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

    // Create log files and channels
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
    let connector = Arc::new(DiscordLogConnector::new());

    // Setup logger
    let subscriber = tracing_subscriber::Registry::default()
        .with(
            // stdout layer, to view everything in the console
            fmt::layer()
                .compact()
                .with_ansi(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(Level::INFO))
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
        )
        .with(
            // log to discord channels
            ChannelWriter { connector: connector.clone(), config }
        );

    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize tracing subscriber");

    connector
}