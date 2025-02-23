use std::fmt::Display;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serenity::all::{CommandInteraction, ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage, Http, Mentionable, ModalInteraction, ResolvedOption, ResolvedValue, User, UserId};
use tracing::error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Username {
    handle: String,
    server_name: String,
    id: UserId,
}

impl Username {
    pub fn from_user(user: &User) -> Self {
        let mut handle = user.name.clone();
        if let Some(discriminator) = &user.discriminator {
            handle += format!("#{discriminator}").as_str();
        }

        Self {
            handle: handle.clone(),
            server_name: if let Some(global) = &user.global_name { global.clone() } else { handle },
            id: user.id,
        }
    }

    pub fn full(&self) -> String {
        format!("{} `{} | {}`", self.id.mention().to_string(), self.server_name, self.handle)
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn safe_full(&self) -> String {
        format!("{} | {}", self.server_name, self.handle)
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

pub trait TruncateText {
    fn truncate_text(&self, max: usize) -> String;
}

impl<T: Display> TruncateText for T {
    fn truncate_text(&self, max: usize) -> String {
        let string = format!("{self}");
        if string.len() > max {
            format!("{}..", string[0..max - 2].to_string())
        } else {
            string
        }
    }
}

impl CommandHelper for ComponentInteraction {
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

pub trait ModalHelper {
    async fn _respond_user_error<T: Display>(&self, http: &Arc<Http>, message: T);
}
impl ModalHelper for ModalInteraction {
    async fn _respond_user_error<T: Display>(&self, http: &Arc<Http>, message: T) {
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
