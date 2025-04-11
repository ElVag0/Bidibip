use std::collections::{HashMap};
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use serenity::all::{ActionRowComponent, ButtonKind, ButtonStyle, ChannelId, CreateActionRow, CreateButton, CreateInputText, CreateInteractionResponse, CreateMessage, CreateModal, EditMessage, GuildChannel, Http, InputTextStyle, Interaction, Message, MessageId};
use crate::core::config::ButtonId;
use crate::core::error::BidibipError;
use crate::core::interaction_utils::{make_custom_id, InteractionUtils};
use crate::modules::advertising::Advertising;
use crate::{assert_some, on_fail, on_fail_warn};
use crate::core::utilities::{TruncateText};
use crate::modules::advertising::steps::ResetStep;

fn default_none_value<T: Clone + ResetStep>() -> Option<(T, String)> {
    None
}

fn default_empty_map<T: Clone + ResetStep>() -> HashMap<String, T> {
    HashMap::new()
}

fn default_none_question<T: Clone + ResetStep>() -> Option<QuestionOptions<T>> {
    None
}

#[derive(Serialize, Deserialize, Clone)]
struct QuestionOptions<T: Clone + ResetStep> {
    message: MessageId,

    // ButtonLabel, Value
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default = "default_empty_map::<T>")]
    items: HashMap<String, T>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ButtonOption<T: Clone + ResetStep> {
    // Value / edit button
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "default_none_value")]
    value: Option<(T, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "default_none_question")]
    question_options: Option<QuestionOptions<T>>,
}

impl<T: Clone + ResetStep> Default for ButtonOption<T> {
    fn default() -> Self {
        Self {
            value: None,
            question_options: None,
        }
    }
}

#[serenity::async_trait]
impl<T: Clone + ResetStep + Send + Sync> ResetStep for ButtonOption<T> {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        if let Some(value) = &mut self.value {
            value.0.delete(http, thread).await?;
        }
        if let Some(options) = &self.question_options {
            on_fail!(on_fail!(thread.message(http, options.message).await, "failed to retrieve message")?.delete(http).await, "Failed to delete message")?;
        }
        self.value = None;
        self.question_options = None;
        Ok(())
    }

    fn clean_for_storage(&mut self) {
        if let Some(value) = &mut self.value {
            value.0.clean_for_storage();
        }
        self.question_options = None;
    }
}

impl<T: Clone + ResetStep + Send + Sync> ButtonOption<T> {
    /// Return true if value was modified. To be modified you should have called init() before
    pub async fn try_set(&mut self, http: &Http, interaction: &Interaction) -> Result<bool, BidibipError> {
        if let Interaction::Component(component) = interaction {
            if let Some(options) = &self.question_options {
                if let Some((action, _)) = component.data.get_custom_id_action::<Advertising>() {
                    if let Some(option) = options.items.get(&action) {
                        on_fail_warn!(component.defer(&http).await, "failed to defer interaction");

                        if let Some(value) = &mut self.value {
                            value.0.delete(http, &component.channel_id).await?;
                        }

                        self.value = Some((option.clone(), action.clone()));
                        let mut question_message = on_fail!(component.channel_id.message(http, options.message).await, "Failed to get question message".to_string())?;
                        let mut components = vec![];
                        for row in &question_message.components {
                            let mut new_buttons = vec![];
                            for component in &row.components {
                                if let ActionRowComponent::Button(button) = component {
                                    if let ButtonKind::NonLink { custom_id, style: _style } = &button.data {
                                        let label = assert_some!(button.label.clone(), "this button doesn't have a valid label")?;
                                        new_buttons.push(CreateButton::new(custom_id).label(label).style(if custom_id == make_custom_id::<Advertising>(action.as_str(), "").as_str() { ButtonStyle::Success } else { ButtonStyle::Secondary }))
                                    }
                                }
                            }
                            components.push(CreateActionRow::Buttons(new_buttons));
                        }

                        on_fail!(question_message.edit(http, EditMessage::new().components(components)).await,  "Failed to edit question button".to_string())?;
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn is_none(&self) -> bool {
        self.value.is_none() && self.question_options.is_none()
    }

    pub fn is_unset(&self) -> bool {
        self.value.is_none() || self.question_options.is_none()
    }

    #[allow(unused)]
    pub fn value(&self) -> Option<&T> {
        match self.value.as_ref() {
            None => { None }
            Some(v) => { Some(&v.0) }
        }
    }

    pub fn value_mut(&mut self) -> Option<&mut T> {
        match self.value.as_mut() {
            None => { None }
            Some(v) => { Some(&mut v.0) }
        }
    }

    /// Write the question in the given channel. Also write the current value if not null
    /// returns false if the question value is not null
    pub async fn try_init(&mut self, http: &Http, thread: &GuildChannel, title: impl Display, options: Vec<(&str, impl ToString, T)>) -> Result<bool, BidibipError> {
        if self.question_options.is_some() {
            return Ok(false);
        }
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

            current_row_content.push(
                CreateButton::new(make_custom_id::<Advertising>(button.0.to_string().as_str(), ""))
                    .label(button.1.to_string())
                    .style(if let Some(value) = &self.value {
                        if value.1 == button.0.to_string() { ButtonStyle::Success } else { ButtonStyle::Secondary }
                    } else { ButtonStyle::Primary })
            );
            out_options.insert(button.0.to_string(), button.2);
        }

        if !current_row_content.is_empty() {
            components.push(CreateActionRow::Buttons(current_row_content.clone()));
        }

        let message = on_fail!(thread.send_message(http, CreateMessage::new()
                    .content(format!("## ▶  {title}"))
                    .components(components)
                ).await, "Failed to send message")?;

        self.question_options = Some(QuestionOptions { message: message.id, items: out_options });
        Ok(self.value.is_none())
    }
}
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TextOption {
    // Value / edit button id
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    value: Option<String>,

    #[serde(default)]
    skipped: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    edit_button: Option<ButtonId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    clear_button: Option<ButtonId>,

    // Question message / title / optional
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    question_message: Option<(MessageId, String, bool)>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    waiting_edition_id: Option<ButtonId>,
}

#[serenity::async_trait]
impl ResetStep for TextOption {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        if let Some((message, _, _)) = &self.question_message {
            let message = on_fail!(thread.message(http, message).await, "Failed to get question message")?;
            on_fail!(message.delete(http).await, "Failed to delete question message")?;
        }
        if let Some(edit_button) = &mut self.edit_button {
            edit_button.free()?;
        }
        if let Some(clear_button) = &mut self.clear_button {
            clear_button.free()?;
        }
        self.value = None;
        self.question_message = None;
        Ok(())
    }

    fn clean_for_storage(&mut self) {
        self.edit_button = None;
        self.clear_button = None;
        self.waiting_edition_id = None;
        self.question_message = None;
    }
}

impl TextOption {
    pub fn is_none(&self) -> bool {
        (self.value.is_none() || self.skipped) && self.question_message.is_none()
    }

    // Return true if value was modified. To be modified you should have called init() before
    pub async fn try_set(&mut self, http: &Http, channel: &ChannelId, message: &Message) -> Result<bool, BidibipError> {
        if let Some((question_message, question, optional)) = &self.question_message {
            if self.value.is_some() || self.skipped {
                return Ok(false);
            }

            let edit_button = ButtonId::new()?;
            let clear_button = ButtonId::new()?;
            let mut buttons = vec![CreateButton::new(edit_button.custom_id::<Advertising>()).style(ButtonStyle::Secondary).label("Modifier")];
            if *optional {
                buttons.push(CreateButton::new(clear_button.custom_id::<Advertising>()).style(ButtonStyle::Secondary).label("Supprimer"));
            }

            let mut question_message = on_fail!(channel.message(http, question_message).await, format!("Failed to get question message for {}", question))?;
            on_fail!(question_message.edit(http, EditMessage::new().content(format!("## ▶  {question}\n`{}`", message.content.truncate_text(1900))).components(vec![CreateActionRow::Buttons(buttons)])).await,  format!("Failed to edit question message for {}", question))?;
            on_fail!(message.delete(http).await, "failed to delete response")?;

            if let Some(clear_button) = &mut self.clear_button {
                clear_button.free()?;
            }
            self.clear_button = Some(clear_button);
            self.edit_button = Some(edit_button);
            self.value = Some(message.content.clone());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    pub async fn try_edit(&mut self, http: &Http, interaction: &Interaction) -> Result<bool, BidibipError> {
        match interaction {
            Interaction::Component(component) => {
                if let Some(clear_button_id) = &self.clear_button {
                    if let Some((action, _)) = component.data.get_custom_id_action::<Advertising>() {
                        if clear_button_id.raw().to_string().as_str() == action {
                            if let Some((question_message, title, _)) = &self.question_message {
                                if let Some(edit_btn) = &mut self.edit_button {
                                    edit_btn.free()?;
                                }

                                if let Some(clear_btn) = &mut self.clear_button {
                                    clear_btn.free()?;
                                }

                                let edit_button = ButtonId::new()?;
                                let clear_button = ButtonId::new()?;

                                self.value = None;
                                self.skipped = true;
                                let mut question_message = on_fail!(component.channel_id.message(http, question_message).await, format!("Failed to get question message for {}", title))?;
                                on_fail!(question_message.edit(http, EditMessage::new()
                                    .content(format!("## ▶  {}\n:negative_squared_cross_mark:", title.truncate_text(300)))
                                    .components(vec![CreateActionRow::Buttons(vec![
                                        CreateButton::new(edit_button.custom_id::<Advertising>()).style(ButtonStyle::Secondary).label("Modifier")])])
                                ).await, format!("Failed to edit question message for {}", title))?;

                                self.clear_button = Some(clear_button);
                                self.edit_button = Some(edit_button);
                                on_fail_warn!(component.defer(http).await, "Failed to acknowledge button");
                                return Ok(true);
                            }
                        }
                    }
                }

                if let Some(edit_button_id) = &self.edit_button {
                    if let Some((action, _)) = component.data.get_custom_id_action::<Advertising>() {
                        if edit_button_id.raw().to_string().as_str() == action {
                            let (_, title, optional) = assert_some!(&self.question_message, "Question message should be none")?;

                            let value = match &self.value {
                                None => { String::new() }
                                Some(val) => { val.clone() }
                            };

                            let button = ButtonId::new()?;
                            on_fail!(component.create_response(http,
                                CreateInteractionResponse::Modal(CreateModal::new(button.custom_id::<Advertising>(), title.truncate_text(45))
                                    .components(vec![CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Paragraph, "Nouveau contenu", "text").placeholder("Nouveau contenu").required(!optional).value(value))]))).await, "Failed to create edit modal")?;
                            self.waiting_edition_id = Some(button);
                            return Ok(true);
                        }
                    }
                }
            }
            Interaction::Modal(modal) => {
                if let Some(edition_id) = &self.waiting_edition_id {
                    if edition_id.custom_id::<Advertising>() == modal.data.custom_id {
                        for component in &modal.data.components {
                            for component in &component.components {
                                if let ActionRowComponent::InputText(text) = component {
                                    let question = assert_some!(&self.question_message, "Failed to get question message")?;

                                    let content = if text.value.is_none() || text.value.as_ref().unwrap().len() == 0 {
                                        self.skipped = true;
                                        self.value = None;
                                        String::from(":negative_squared_cross_mark:")
                                    } else {
                                        self.value = Some(assert_some!(text.value.clone(), "empty modal text")?);
                                        format!("`{}`", text.value.clone().unwrap_or_default())
                                    };

                                    on_fail_warn!(modal.defer(http).await, "Failed to close edition modal");
                                    edition_id.clone().free()?;
                                    self.waiting_edition_id = None;

                                    let mut question_message = on_fail!(modal.channel_id.message(http, question.0).await, format!("Failed to get question message for {}", question.1))?;
                                    on_fail!(question_message.edit(http, EditMessage::new().content(format!("## ▶  {}\n{}", question.1.truncate_text(300), content.truncate_text(1650)))).await,  format!("Failed to edit question message for {}", question.1))?;

                                    return Ok(true);
                                }
                            }
                            break;
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(false)
    }

    pub fn is_unset(&self) -> bool {
        (self.value.is_none() && !self.skipped) || self.question_message.is_none()
    }

    /// Write the question in the given channel. Also write the current value if not null
    /// returns false if the question value is not null
    pub async fn try_init(&mut self, http: &Http, thread: &GuildChannel, title: impl Display, optional: bool) -> Result<bool, BidibipError> {
        if self.question_message.is_some() { return Ok(self.value.is_none()); }
        let message = match &mut self.value {
            None => {
                let skip_button = ButtonId::new()?;
                self.clear_button = Some(skip_button.clone());
                let mut message = CreateMessage::new().content(format!("## ▶  {}\n> *Écris ta réponse sous ce message*", title.truncate_text(300)));
                if optional {
                    message = message.components(vec![CreateActionRow::Buttons(vec![CreateButton::new(skip_button.custom_id::<Advertising>()).label("Ignorer").style(ButtonStyle::Secondary)])]);
                }
                on_fail!(thread.send_message(http,message).await, "Failed to send message")?
            }
            Some(value) => {
                let edit_button = ButtonId::new()?;
                let skip_button = ButtonId::new()?;

                let mut buttons = vec![CreateButton::new(edit_button.custom_id::<Advertising>()).style(ButtonStyle::Secondary).label("Modifier")];
                if optional {
                    buttons.push(CreateButton::new(skip_button.custom_id::<Advertising>()).style(ButtonStyle::Secondary).label("Supprimer"));
                }

                let message = on_fail!(thread.send_message(http, CreateMessage::new()
                            .content(format!("## ▶  {}\n`{}`", title.truncate_text(300), value.truncate_text(1650)))
                            .components(vec![CreateActionRow::Buttons(buttons)])).await, "Failed to send message")?;
                self.edit_button = Some(edit_button);
                self.clear_button = Some(skip_button.clone());
                message
            }
        };
        self.question_message = Some((message.id, title.to_string(), optional));
        Ok(self.value.is_none())
    }
}