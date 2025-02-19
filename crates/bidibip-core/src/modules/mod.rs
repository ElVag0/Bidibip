use crate::core::config::Config;
use crate::modules::say::Say;
use crate::modules::warn::Warn;
use serenity::all::{CreateCommand, EventHandler};
use std::sync::Arc;

mod say;
mod warn;

pub trait Module: Sync + Send + EventHandler {
    fn name(&self) -> &'static str;
    fn fetch_command(&self) -> Vec<(String, CreateCommand)> { vec![] }
}

pub fn load_modules(config: Arc<Config>) -> Vec<Box<dyn Module>> {
    vec![Box::new(Say {}), Box::new(Warn {})]
}