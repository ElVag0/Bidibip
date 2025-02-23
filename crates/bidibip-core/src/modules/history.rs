use std::sync::Arc;
use anyhow::Error;
use serenity::all::{ChannelId, Colour, Context, CreateMessage, EventHandler, GuildId, Message, MessageAction, MessageId, MessageUpdateEvent};
use serenity::all::audit_log::Action;
use serenity::builder::CreateEmbed;
use tracing::{error, info};
use crate::core::config::Config;
use crate::core::module::BidibipSharedData;
use crate::core::utilities::{ResultDebug, Username};
use crate::modules::{BidibipModule, LoadModule};

pub struct History {
    config: Arc<Config>,
}

impl LoadModule<History> for History {
    fn name() -> &'static str {
        "history"
    }

    fn description() -> &'static str {
        "Historique des messages modifiés et supprimés"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<History, Error> {
        Ok(History { config: shared_data.config.clone() })
    }
}

impl BidibipModule for History {}

#[serenity::async_trait]
impl EventHandler for History {
    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        let date = deleted_message_id.created_at().format("%d %B %Y");
        let mut old_message_content = format!("Ancien message : {}", deleted_message_id.link(channel_id, guild_id));
        let mut user = None;
        if let Some(deleted) = ctx.cache.message(channel_id, deleted_message_id) {

            // Skip self
            if deleted.author.id.get() == self.config.application_id.get() {
                return;
            }

            if !deleted.content.is_empty() {
                old_message_content = deleted.content.clone()
            } else if !deleted.attachments.is_empty() {
                let mut str = String::new();
                for attachment in &deleted.attachments {
                    str += format!("{} ", attachment.url).as_str();
                }
                old_message_content = str;
            }
            user = Some(deleted.author.clone());
        }

        if user.is_some() {
            let mut by = None;
            if let Some(guild) = guild_id {
                match guild.audit_logs(&ctx.http, Some(Action::Message(MessageAction::Delete)), None, None, Some(1)).await {
                    Ok(res) => {
                        if let Some(entry) = res.entries.first() {
                            match entry.user_id.to_user(&ctx.http).await {
                                Ok(found_user) => {
                                    by = Some(found_user);
                                }
                                Err(err) => { error!("Failed to get deleted message user : {}", err) }
                            }
                        }
                    }
                    Err(error) => { error!("Failed to fetch audit logs {}", error) }
                }
            }


            let user_name = match &user {
                None => { "Unknown user".to_string() }
                Some(user) => {
                    format!("{} ({})", Username::from_user(user).safe_full(), user.id)
                }
            };

            let from_name = match &by {
                None => { "Unknown user".to_string() }
                Some(user) => {
                    format!("{} ({})", Username::from_user(user).safe_full(), user.id)
                }
            };
            self.config.channels.log_channel.send_message(
                &ctx.http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .color(Colour::RED)
                        .title(format!("Message du {} supprimé par {}", date, from_name))
                        .description(deleted_message_id.link(channel_id, guild_id))
                        .field(format!("de : {}", &user_name), &old_message_content, false))).await.on_fail("Failed to print message rename log");

            info!(target: "log","Message {} de {} du {} supprimé par {} : {}", deleted_message_id.link(channel_id, guild_id), user_name, date, from_name, old_message_content);
        } else {
            self.config.channels.log_channel.send_message(
                &ctx.http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .color(Colour::RED)
                        .title(format!("Ancien message du {} supprimé", date))
                        .description(deleted_message_id.link(channel_id, guild_id)))).await.on_fail("Failed to print message rename log");

            info!(target: "log","Ancien message du {} supprimé : {}", date, deleted_message_id.link(channel_id, guild_id));
        }
    }

    async fn message_update(&self, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {
        let mut user = event.author;
        let mut new_url = event.id.link(event.channel_id, event.guild_id);
        let mut old_text = String::new();
        let mut new_text = String::new();

        if let Some(old) = old_if_available {
            if user.is_none() {
                user = Some(old.author.clone());
            }
            old_text = old.content;
            if old_text.is_empty() {
                for attachment in &old.attachments {
                    old_text += format!("{} ", attachment.url).as_str();
                }
            }
        }

        if let Some(new) = new {

            // Skip self
            if new.author.id.get() == self.config.application_id.get() {
                return;
            }

            if user.is_none() {
                user = Some(new.author.clone());
            }
            new_text = new.content.clone();
            if new_text.is_empty() {
                for attachment in &new.attachments {
                    new_text += format!("{} ", attachment.url).as_str();
                }
            }
            if new_url.is_empty() {
                new_url = new.link();
            }
        }

        if new_text.is_empty() {
            if let Some(content) = event.content {
                new_text = content;
            }
            if new_text.is_empty() {
                if let Some(attachments) = event.attachments {
                    for attachment in &attachments {
                        new_text += format!("{} ", attachment.url).as_str();
                    }
                }
            }
        }


        let mut embed = CreateEmbed::new()
            .color(Colour::ORANGE)
            .title(match &user {
                None => { "Unknown user".to_string() }
                Some(user) => {
                    format!("{} ({})", Username::from_user(user).safe_full(), user.id)
                }
            })
            .description(format!("Message modifié : {}", new_url));
        if !old_text.is_empty() {
            embed = embed.field("ancien", &old_text, false);
        }
        if !new_text.is_empty() {
            embed = embed.field("nouveau", &new_text, false);
        }

        self.config.channels.log_channel.send_message(
            &ctx.http,
            CreateMessage::new().embed(embed)).await.on_fail("Failed to print message rename log");

        info!(target: "log","Message de {} modifié : [[FROM]] {} [[TO]] {}", match &user {
                    None => {
                       "Unknown user".to_string()
                    }
                    Some(user) => {
                        Username::from_user(&user).full()
                    }
                }, old_text, new_text);
    }
}