use std::collections::{HashMap};
use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use serenity::all::{ActionRowComponent, ButtonKind, ButtonStyle, ChannelId, CreateActionRow, CreateButton, CreateMessage, EditMessage, GuildChannel, Http, Message, MessageId};
use crate::core::config::ButtonId;
use crate::core::error::BidibipError;
use crate::core::interaction_utils::make_custom_id;
use crate::modules::advertising::Advertising;
use crate::{assert_some, on_fail};
use crate::modules::advertising::steps::ResetStep;

#[derive(Serialize, Deserialize, Clone)]
pub struct ButtonOption<T: Clone + ResetStep> {
    // Value / edit button
    value: Option<T>,
    // ButtonId, Value
    question_options: Option<(MessageId, HashMap<u64, T>)>,
}

#[serenity::async_trait]
impl<T: Clone + ResetStep + Send + Sync> ResetStep for ButtonOption<T> {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        if let Some(value) = &mut self.value {
            value.delete(http, thread).await?;
        }
        if let Some((message, buttons)) = &self.question_options {
            on_fail!(on_fail!(thread.message(http, message).await, "failed to retrieve message")?.delete(http).await, "Failed to delete message")?;

            for button in buttons.keys() {
                ButtonId::from(*button).free()?;
            }

        }
        self.value = None;
        self.question_options = None;
        Ok(())
    }
}

impl<T: Clone + ResetStep> Default for ButtonOption<T> {
    fn default() -> Self {
        Self {
            value: None,
            question_options: None,
        }
    }
}


impl<T: Clone + ResetStep + Send + Sync> ButtonOption<T> {
    // Return true if value was modified. To be modified you should have called init() before
    pub async fn try_set(&mut self, http: &Http, channel: &ChannelId, action: &str) -> Result<bool, BidibipError> {
        if let Some((question_message, options)) = &self.question_options {

            let id = match u64::from_str(action) {
                Ok(val) => {val}
                Err(_) => {return Ok(false)}
            };

            if let Some(option) = options.get(&id) {

                if let Some(value) = &mut self.value {
                    value.delete(http, channel).await?;
                }


                self.value = Some(option.clone());
                let mut question_message = on_fail!(channel.message(http, question_message).await, "Failed to get question message".to_string())?;
                let mut components = vec![];
                for row in &question_message.components {
                    let mut new_buttons = vec![];
                    for component in &row.components {
                        if let ActionRowComponent::Button(button) = component {
                            if let ButtonKind::NonLink { custom_id, style: _style } = &button.data {
                                let label = assert_some!(button.label.clone(), "this button doesn't have a valid label")?;
                                new_buttons.push(CreateButton::new(custom_id).label(label).style(if custom_id == make_custom_id::<Advertising>(action, "").as_str() { ButtonStyle::Success } else { ButtonStyle::Secondary }))
                            }
                        }
                    }
                    components.push(CreateActionRow::Buttons(new_buttons));
                }

                on_fail!(question_message.edit(http, EditMessage::new().components(components)).await,  "Failed to edit question button".to_string())?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn is_unset(&self) -> bool {
        self.value.is_none()
    }

    #[allow(unused)]
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn value_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }

    pub async fn try_init(&mut self, http: &Http, thread: &GuildChannel, title: impl Display, options: Vec<(impl ToString, T)>) -> Result<(), BidibipError> {
        if self.question_options.is_some() { return Ok(()); }

        let mut out_options = HashMap::new();

        let mut components = vec![];
        let mut current_row_content = vec![];
        let mut cnt = 0;
        for button in options {
            if cnt >= 3 {
                components.push(CreateActionRow::Buttons(current_row_content.clone()));
                current_row_content.clear();
                cnt = 0;
            } else {
                cnt += 1;
            }
            let id = ButtonId::new()?;

            current_row_content.push(CreateButton::new(id.custom_id::<Advertising>()).label(button.0.to_string()));
            out_options.insert(id.raw(), button.1);
        }

        if !current_row_content.is_empty() {
            components.push(CreateActionRow::Buttons(current_row_content.clone()));
        }

        let message = on_fail!(thread.send_message(http, CreateMessage::new()
                    .content(format!("## ▶  {title}"))
                    .components(components)
                ).await, "Failed to send message")?;

        self.question_options = Some((message.id, out_options));
        Ok(())
    }
}
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TextOption {
    // Value / edit button id
    value: Option<(String, ButtonId)>,
    // Question message / title
    question_message: Option<(MessageId, String)>,
}

#[serenity::async_trait]
impl ResetStep for TextOption {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        if let Some((message, _)) = &self.question_message {
            let message = on_fail!(thread.message(http, message).await, "Failed to get question message")?;
            on_fail!(message.delete(http).await, "Failed to delete question message")?;
        }
        if let Some((_, edit_button)) = &mut self.value {
            edit_button.free()?;
        }

        self.value = None;
        self.question_message = None;
        Ok(())
    }
}

impl TextOption {
    // Return true if value was modified. To be modified you should have called init() before
    pub async fn try_set(&mut self, http: &Http, channel: &ChannelId, message: &Message) -> Result<bool, BidibipError> {
        if let Some((question_message, question)) = &self.question_message {
            if self.value.is_some() {
                return Ok(false);
            }
            //ctx.http
            let id = ButtonId::new()?;
            let mut question_message = on_fail!(channel.message(http, question_message).await, format!("Failed to get question message for {}", question))?;
            on_fail!(question_message.edit(http, EditMessage::new().content(format!("## ▶  {question}\n`{}`", message.content)).components(vec![
                CreateActionRow::Buttons(vec![CreateButton::new(id.custom_id::<Advertising>()).style(ButtonStyle::Secondary).label("Modifier")])
            ])).await,  format!("Failed to edit question message for {}", question))?;
            on_fail!(message.delete(http).await, "failed to delete response")?;

            self.value = Some((message.content.clone(), id));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn value(&self) -> Option<&String> {
        match &self.value {
            None => { None }
            Some((value, _)) => { Some(value) }
        }
    }

    pub async fn reset(&mut self, http: &Http, thread: &ChannelId, action: &str) -> Result<(), BidibipError> {
        if let Some((_, reset_button_id)) = &self.value {
            if reset_button_id.raw().to_string().as_str() == action {
                self.delete(http, thread).await?;
            }
        }
        Ok(())
    }

    pub fn is_unset(&self) -> bool {
        self.value.is_none()
    }

    pub async fn try_init(&mut self, http: &Http, thread: &GuildChannel, title: impl Display) -> Result<(), BidibipError> {
        if self.question_message.is_some() { return Ok(()); }
        let message = on_fail!(thread.send_message(http, CreateMessage::new()
                    .content(format!("## ▶  {title}\n> *Écris ta réponse sous ce message*"))
                ).await, "Failed to send message")?;
        self.question_message = Some((message.id, title.to_string()));
        Ok(())
    }
}