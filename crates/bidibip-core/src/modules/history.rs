use std::sync::Arc;
use serenity::all::{audit_log, AuditLogEntry, ChannelId, Colour, Context, CreateMessage, EventHandler, GuildId, Member, Message, MessageAction, MessageId, MessageUpdateEvent, User, UserId};
use serenity::all::audit_log::Action;
use serenity::builder::CreateEmbed;
use tracing::{error, info, warn};
use tracing::log::log;
use crate::core::config::Config;
use crate::core::utilities::{ResultDebug, Username};
use crate::modules::BidibipModule;

pub struct History {
    config: Arc<Config>,
}

impl History {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}


#[serenity::async_trait]
impl EventHandler for History {
    async fn guild_audit_log_entry_create(&self, ctx: Context, entry: AuditLogEntry, guild_id: GuildId) {
        if let Action::Message(message) = entry.action {
            match message {
                MessageAction::Delete => {
                    warn!("AUDIT delete message");
                }
                MessageAction::BulkDelete => {
                    warn!("AUDIT delete message bulk");
                }
                _ => {}
            }
        }
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        if let Some(deleted) = ctx.cache.message(channel_id, deleted_message_id) {
            info!("OK J'AI LE CACHE {:?}", deleted.content);
        }


        let mut user = None;
        let mut from = None;
        let mut text = String::new();


        if let Some(guild) = guild_id {
            match guild.audit_logs(&ctx.http, Some(Action::Message(MessageAction::Delete)), None, None, Some(1)).await {
                Ok(res) => {
                    if let Some(entry) = res.entries.first() {
                        match UserId::from(entry.user_id.get()).to_user(&ctx.http).await {
                            Ok(found_user) => {
                                from = Some(found_user);
                            }
                            Err(err) => { error!("Failed to get deleted message user : {}", err) }
                        }
                        warn!("test : {:?}", entry);

                        if let Some(target) = entry.target_id {
                            match UserId::from(target.get()).to_user(&ctx.http).await {
                                Ok(found_user) => {
                                    user = Some(found_user);
                                }
                                Err(err) => { error!("Failed to get deleted message action user : {}", err) }
                            }
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

        let from_name = match &from {
            None => { "Unknown user".to_string() }
            Some(user) => {
                format!("{} ({})", Username::from_user(user).safe_full(), user.id)
            }
        };


        let embed = CreateEmbed::new()
            .color(Colour::RED)
            .title(&user_name)
            .description(format!("Message supprimé par {}", from_name));

        ChannelId::from(self.config.channels.log_channel).send_message(
            &ctx.http,
            CreateMessage::new().embed(embed)).await.on_fail("Failed to print message rename log");

        info!(target: "log","Message de {} supprimé par {}", user_name, from_name);
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

        ChannelId::from(self.config.channels.log_channel).send_message(
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

impl BidibipModule for History {
    fn name(&self) -> &'static str {
        "History"
    }
}