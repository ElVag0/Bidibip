use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, CreateActionRow, CreateButton, CreateMessage, GuildChannel, Http};
use crate::core::error::BidibipError;
use crate::core::interaction_utils::make_custom_id;
use crate::modules::{BidibipModule, LoadModule};
use crate::on_fail;

#[derive(Serialize, Deserialize, Clone)]
pub enum Contact {
    Discord,
    Other(Option<String>),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Location {
    Remote,
    OnSiteFlex(Option<String>),
    OnSite(Option<String>),
}

pub struct ButtonDescription {
    id: String,
    text: String,
}

impl ButtonDescription {
    pub fn new(id: impl ToString, text: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
        }
    }
}

pub async fn create_multi_button_options<T: BidibipModule + LoadModule<T>>(http: &Http, thread: &GuildChannel, title: &str, buttons: Vec<ButtonDescription>) -> Result<(), BidibipError> {
    let mut components = vec![];
    let mut current_row_content = vec![];
    let mut cnt = 0;
    for button in buttons {
        if cnt >= 3 {
            components.push(CreateActionRow::Buttons(current_row_content.clone()));
            current_row_content.clear();
            cnt = 0;
        } else {
            cnt += 1;
        }
        current_row_content.push(CreateButton::new(make_custom_id::<T>(button.id.as_str(), thread.id)).label(button.text.as_str()));
    }

    if !current_row_content.is_empty() {
        components.push(CreateActionRow::Buttons(current_row_content.clone()));
    }

    on_fail!(thread.send_message(http, CreateMessage::new().content(format!("## ▶  {}", title)).components(components)).await, "Failed to create buttons")?;
    Ok(())
}

pub async fn create_text_input_options<T: BidibipModule + LoadModule<T>>(http: &Http, thread: &GuildChannel, title: &str, edit: Option<&str>) -> Result<(), BidibipError> {
    let mut components = vec![];
    if let Some(edit) = edit {
        components.push(CreateActionRow::Buttons(vec![CreateButton::new(make_custom_id::<T>(format!("edit_{}", edit).as_str(), thread.id)).style(ButtonStyle::Secondary).label("modifier")]));
    };
    on_fail!(thread.send_message(http, CreateMessage::new()
                    .content(format!("## ▶  {title}\n> *Écris ta réponse sous ce message*"))
                    .components(components)
                ).await, "Failed to send message")?;
    Ok(())
}