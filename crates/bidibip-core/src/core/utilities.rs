use std::fmt::Display;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serenity::all::{CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, Http, Mentionable, ModalInteraction, ResolvedOption, ResolvedValue, User, UserId};
use tracing::error;

pub fn json_to_message(_json: String) -> Vec<CreateMessage> {
    vec![]
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Username {
    handle: String,
    server_name: String,
    id: u64,
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
            id: user.id.get(),
        }
    }

    pub fn full(&self) -> String {
        format!("{} `{} | {}`", UserId::from(self.id).mention().to_string(), self.server_name, self.handle)
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn safe_full(&self) -> String {
        format!("{} | {}", self.server_name, self.handle)
    }

    pub fn mention(&self) -> String {
        UserId::from(self.id).mention().to_string()
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


pub trait ModalHelper {
    async fn close(&self, http: &Arc<Http>) -> bool;
    async fn respond_user_error<T: Display>(&self, http: &Arc<Http>, message: T);
}
impl ModalHelper for ModalInteraction {
    async fn close(&self, http: &Arc<Http>) -> bool {
        if self.defer(http).await.on_fail("Failed to close modal interaction") {
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

