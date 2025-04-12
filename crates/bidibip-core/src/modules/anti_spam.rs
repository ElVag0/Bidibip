use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, ChannelId, Context, CreateActionRow, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction, Mentionable, Message, MessageId, RoleId, UserId};
use serenity::all::Interaction::Component;
use serenity::builder::{CreateButton, CreateMessage};
use tokio::sync::RwLock;
use tracing::log::warn;
use crate::core::config::{ButtonId, Config};
use crate::core::error::BidibipError;
use crate::core::global_interface::BidibipSharedData;
use crate::core::message_reference::MessageReference;
use crate::modules::{BidibipModule, LoadModule};
use crate::{on_fail};

#[derive(Default)]
struct LastMessage {
    content: String,
    occurrences: Vec<(DateTime<Utc>, MessageReference)>,
    warned: bool,
}

#[derive(Deserialize, Serialize)]
struct SpammerContext {
    kick_button: ButtonId,
    pardon_button: ButtonId,
    spammer: UserId,
}

#[derive(Deserialize, Serialize)]
struct AntiSpamConfig {
    min_occurrences: usize,
    max_delay_ms: i64,
    mute_role: RoleId,
    moderation_channel: ChannelId,
    spammers: HashMap<MessageId, SpammerContext>,
}

impl Default for AntiSpamConfig {
    fn default() -> Self {
        Self {
            min_occurrences: 3,
            max_delay_ms: 60000,
            mute_role: Default::default(),
            moderation_channel: Default::default(),
            spammers: Default::default(),
        }
    }
}

pub struct AntiSpam {
    history: RwLock<HashMap<UserId, LastMessage>>,
    anti_spam_config: RwLock<AntiSpamConfig>,
}

impl LoadModule<AntiSpam> for AntiSpam {
    fn name() -> &'static str {
        "anti-spam"
    }

    fn description() -> &'static str {
        "Protection contre les spams potentiels"
    }

    async fn load(_: &Arc<BidibipSharedData>) -> Result<AntiSpam, Error> {
        Ok(AntiSpam { history: Default::default(), anti_spam_config: RwLock::new(Config::get().load_module_config::<AntiSpam, AntiSpamConfig>()?) })
    }
}

#[serenity::async_trait]
impl BidibipModule for AntiSpam {
    async fn message(&self, ctx: Context, msg: Message) -> Result<(), BidibipError> {
        let mut history = self.history.write().await;

        let entry = history.entry(msg.author.id).or_default();

        if entry.content == msg.content {
            let mut spam_messages = vec![];

            // Ensure the message was sent in different channels each time. (the same message sent in the same channel multiple times will not be detected)
            for (_, channel) in &entry.occurrences {
                if channel.channel() == msg.channel_id {
                    return Ok(());
                }
            }

            // Ensure all the messages was sent in a delay < max_delay_ms
            for (date, message) in &entry.occurrences {
                let elapsed = Utc::now() - *date;
                if elapsed < Duration::milliseconds(self.anti_spam_config.read().await.max_delay_ms) {
                    spam_messages.push(message.clone());
                }
            }

            // If we reached the threshold of min_occurrences, consider the user as a spammer
            if spam_messages.len() >= self.anti_spam_config.read().await.min_occurrences {
                if entry.warned {
                    return Ok(());
                }
                entry.warned = true;

                let mut config = self.anti_spam_config.write().await;

                let member = on_fail!(Config::get().server_id.member(&ctx.http, msg.author.id).await, "Not a member")?;
                on_fail!(member.add_role(&ctx.http, config.mute_role).await, "Failed to mute potential spammer")?;

                let kick_button = ButtonId::new()?;
                let pardon_button = ButtonId::new()?;

                let modo_message = on_fail!(config.moderation_channel.send_message(&ctx.http, CreateMessage::new()
                    .content(format!("@everyone Spam potentiel de {} : `{}`", msg.author.mention(), msg.content))
                .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(kick_button.custom_id::<AntiSpam>()).style(ButtonStyle::Danger).label("Kick"),
                        CreateButton::new(pardon_button.custom_id::<AntiSpam>()).style(ButtonStyle::Success).label("Pardonner")
                    ])])).await, "Failed to send warn message in modo channel")?;

                for message in spam_messages {
                    match message.message(&ctx.http).await {
                        Ok(message) => {
                            on_fail!(message.delete(&ctx.http).await, "Failed to delete spam message")?;
                        }
                        Err(err) => { warn!("Failed to get message to delete : {}", err) }
                    }
                }

                config.spammers.insert(modo_message.id, SpammerContext {
                    kick_button,
                    pardon_button,
                    spammer: msg.author.id,
                });
                Config::get().save_module_config::<AntiSpam, AntiSpamConfig>(&config)?;
            }
        } else {
            entry.content = msg.content.clone();
            entry.occurrences.clear();
            entry.warned = false;
        }
        entry.occurrences.push((Utc::now(), MessageReference::from(&msg)));
        Ok(())
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> Result<(), BidibipError> {
        if let Component(component) = interaction {
            let mut config = self.anti_spam_config.write().await;
            let mute_role = config.mute_role;
            if let Some(infos) = config.spammers.get_mut(&component.message.id) {
                if infos.kick_button.custom_id::<AntiSpam>().to_string() == component.data.custom_id {
                    infos.kick_button.free()?;
                    infos.pardon_button.free()?;
                    let member = on_fail!(Config::get().server_id.member(&ctx.http, infos.spammer).await, "Not a member")?;
                    on_fail!(member.kick_with_reason(&ctx, "Spam détecté").await, "Failed to kick spammer")?;
                    on_fail!(component.message.delete(&ctx.http).await, "Failed to delete anti spam message")?;
                    on_fail!(component.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!("{} a été kick par {} pour cause de spam", infos.spammer.mention(), component.user.mention())))).await, "Failed to send response")?;
                    config.spammers.remove(&component.message.id);
                    Config::get().save_module_config::<AntiSpam, AntiSpamConfig>(&config)?;
                } else if infos.pardon_button.custom_id::<AntiSpam>().to_string() == component.data.custom_id {
                    infos.kick_button.free()?;
                    infos.pardon_button.free()?;
                    let member = on_fail!(Config::get().server_id.member(&ctx.http, infos.spammer).await, "Not a member")?;
                    on_fail!(member.remove_role(&ctx.http, mute_role).await, "Failed to remove mute role")?;
                    on_fail!(component.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!("{} a été pardonné par {}", infos.spammer.mention(), component.user.mention())))).await, "Failed to send response")?;
                    on_fail!(component.message.delete(&ctx.http).await, "Failed to delete anti spam message")?;
                    self.history.write().await.remove(&infos.spammer);
                    config.spammers.remove(&component.message.id);
                    Config::get().save_module_config::<AntiSpam, AntiSpamConfig>(&config)?;
                }
            }
        }
        Ok(())
    }
}