use std::fmt::Display;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonKind, ButtonStyle, CommandInteraction, CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, Http, Mentionable, ModalInteraction, ResolvedOption, ResolvedValue, User, UserId};
use serenity::builder::CreateEmbed;
use tracing::error;

#[derive(Deserialize, Debug)]
struct JsonToMessageMessageInteraction {
    #[serde(rename = "type")]
    button_type: String,
    texte: String,
    identifiant: String,
}

#[derive(Deserialize, Debug)]
struct JsonToMessageMessageEmbed {
    title: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct JsonToMessageMessage {
    textes: Vec<String>,
    embed: Vec<JsonToMessageMessageEmbed>,
    interactions: Vec<JsonToMessageMessageInteraction>,
}

#[derive(Deserialize, Debug)]
struct JsonToMessageBase {
    messages: Vec<JsonToMessageMessage>,
}

pub fn json_to_message(json: String) -> Result<Vec<CreateMessage>, Error> {
    let data: JsonToMessageBase = serde_json::from_str(json.as_str())?;
    let mut messages = vec![];

    for message in data.messages {
        let mut data = CreateMessage::new();

        if message.textes.is_empty() && message.embed.is_empty() {
            return Err(Error::msg("Chaque message doit contenir au moins message ou au moins un embed"));
        }
        if !message.textes.is_empty() {
            let mut full_text = String::new();
            for text in message.textes {
                full_text += format!("{}\n", text).as_str();
            }
            data = data.content(full_text);
        }
        for embed in message.embed {
            data = data.embed(CreateEmbed::new().title(embed.title).description(embed.description));
        }
        let mut components = vec![];
        for interaction in message.interactions {

            let button_type = match interaction.button_type.as_str() {
                "Primary" => { ButtonStyle::Primary }
                "Secondary" => { ButtonStyle::Secondary }
                "Success" => { ButtonStyle::Success }
                "Danger" => { ButtonStyle::Danger }
                &_ => { ButtonStyle::Primary }
            };
            components.push(CreateActionRow::Buttons(vec![CreateButton::new(interaction.identifiant).label(interaction.texte).style(button_type)]))
        }
        data = data.components(components);
        messages.push(data);
    }
    Ok(messages)
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
    async fn _respond_user_error<T: Display>(&self, http: &Arc<Http>, message: T);
}
impl ModalHelper for ModalInteraction {
    async fn close(&self, http: &Arc<Http>) -> bool {
        if self.defer(http).await.on_fail("Failed to close modal interaction") {
            false
        } else { true }
    }

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

