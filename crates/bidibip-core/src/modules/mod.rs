use std::sync::Arc;
use anyhow::Error;
use serenity::all::{CommandInteraction, Context, CreateCommand, EventHandler};
use tracing::error;
use crate::core::config::Config;
use crate::modules::warn::Warn;

mod say;
mod warn;
mod log;
mod history;

#[serenity::async_trait]
pub trait BidibipModule: Sync + Send + EventHandler {
    // Module display name
    fn name(&self) -> &'static str;

    // Get a list of available commands for this module
    fn fetch_commands(&self) -> Vec<(String, CreateCommand)> { vec![] }

    // When one of the specified command is executed
    async fn execute_command(&self, _ctx: Context, _name: &str, _command: CommandInteraction) {}
}

pub async fn load_modules(config: Arc<Config>) -> Vec<Box<dyn BidibipModule>> {
    let mut modules: Vec<Box<dyn BidibipModule>> = vec![];

    // SAY
    modules.push(Box::new(say::Say {}));

    // WARN
    match Warn::new(config.clone()).await {
        Ok(module) => {
            modules.push(Box::new(module))
        }
        Err(err) => { error!("Failed to load warn module : {err}") }
    }

    // LOG
    modules.push(Box::new(log::Log {}));

    // HISTORY
    modules.push(Box::new(history::History::new(config.clone())));

    modules
}