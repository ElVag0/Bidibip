use std::fmt::Display;
use crate::core::config::Config;
use crate::modules::say::Say;
use crate::modules::warn::Warn;
use serenity::all::{CommandInteraction, Context, CreateCommand, CreateInteractionResponseMessage, EventHandler, Http, ResolvedOption, ResolvedValue};
use std::sync::Arc;
use serenity::builder::CreateInteractionResponse;
use tracing::error;

mod say;
mod warn;

#[serenity::async_trait]
pub trait BidibipModule: Sync + Send + EventHandler {
    // Module display name
    fn name(&self) -> &'static str;
    // Get a list of available commands for this module
    fn fetch_commands(&self) -> Vec<(String, CreateCommand)> { vec![] }
    // When one of the specified command is executed
    async fn execute_command(&self, _ctx: Context, _name: &str, _command: CommandInteraction) {}
}


pub trait ResultDebug<T, E: std::fmt::Debug> {
    fn on_fail<M: Display>(&self, message: M) -> bool;
}

impl<T, E: std::fmt::Debug> ResultDebug<T, E> for Result<T, E> {
    fn on_fail<M: Display>(&self, message: M) -> bool {
        if let Err(err) = self {
            error!("{} : {:?}", message, err);
            false
        } else {
            true
        }
    }
}

pub trait CommandHelper {
    async fn skip(&self, http: &Arc<Http>) -> bool;
    async fn respond_user_error<T: Display>(&self, http: &Arc<Http>, message: T);
}

impl CommandHelper for CommandInteraction {
    async fn skip(&self, http: &Arc<Http>) -> bool {
        if self.defer(http).await.on_fail("Failed to defer command interaction") {
            self.delete_response(http).await.on_fail("Failed to delete command interaction");
            false
        } else { true }
    }

    async fn respond_user_error<T: Display>(&self, http: &Arc<Http>, message: T) {
        self.create_response(http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content(format!(":boom: **Mince alors !**\n{message}")))).await.on_fail("Failed to send command user error response");
    }
}

pub trait OptionHelper<'a> {
    fn find(&self, option: &str) -> Option<ResolvedValue<'a>>;
}

impl<'a> OptionHelper<'a> for Vec<ResolvedOption<'a>> {
    fn find(&self, option: &str) -> Option<ResolvedValue<'a>> {
        for opt in self {
            if opt.name == option {
                return Some(opt.value.clone());
            }
        }
        None
    }
}

pub fn load_modules(_config: Arc<Config>) -> Vec<Box<dyn BidibipModule>> {
    vec![Box::new(Say {}), Box::new(Warn {})]
}