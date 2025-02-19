use serenity::all::{CommandOptionType, CommandType, CreateCommand, CreateCommandOption, EventHandler};
use crate::modules::Module;

pub struct Warn {

}

impl EventHandler for Warn {

}

impl Module for Warn {
    fn name(&self) -> &'static str {
        "warn"
    }

    fn fetch_command(&self) -> Vec<(String, CreateCommand)> {
        vec![("test".to_string(), CreateCommand::new("test").kind(CommandType::User))]
    }
}