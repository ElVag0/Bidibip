use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use serenity::all::{AuditLogEntry, ChannelId, Context, GuildId, GuildMemberUpdateEvent, Interaction, Member, Message, MessageId, MessageUpdateEvent, Ready, User};
use serenity::prelude::EventHandler;
use tracing::{error, info};
use crate::core::config::Config;
use crate::core::logger::DiscordLogConnector;
use crate::modules::{load_modules, BidibipModule};

pub struct GlobalInterface {
    config: Arc<Config>,
    modules: Vec<ModuleData>,
    log_connector: Arc<DiscordLogConnector>,
}

struct ModuleData {
    module: Box<dyn BidibipModule>,
    commands: HashSet<String>,
}

impl GlobalInterface {
    // Default constructor
    // The log connector is used to provide the log channel to the logger
    pub async fn new(config: Arc<Config>, log_connector: Arc<DiscordLogConnector>) -> Self {
        let mut modules = vec![];
        for module in load_modules(config.clone()).await {
            let mut commands = HashSet::new();
            for command in module.fetch_commands() {
                commands.insert(command.0);
            }

            modules.push(ModuleData { module, commands })
        }

        Self { config: config.clone(), modules, log_connector }
    }

    pub async fn update_commands(&self, ctx: &Context) {
        let mut commands = HashMap::new();

        for module in &self.modules {
            for (name, command) in module.module.fetch_commands() {
                commands.insert(name.clone(), command.name(name));
            }
        }

        let guild_id = GuildId::new(self.config.server_id);

        for command in guild_id.get_commands(&ctx.http).await.unwrap() {
            if commands.contains_key(&command.name) {
                commands.remove(&command.name);
            } else {
                match guild_id.delete_command(&ctx.http, command.id).await {
                    Ok(_) => {}
                    Err(err) => { error!("Failed to remove outdated command {err}") }
                };
            }
        }

        for command in commands {
            match guild_id.create_command(&ctx.http, command.1).await {
                Ok(command) => { info!("Registered new command {}", command.name) }
                Err(err) => { error!("Failed to register new command {err}") }
            };
        }
    }
}

#[serenity::async_trait]
impl EventHandler for GlobalInterface {
    async fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, banned_user: User) {
        for module in &self.modules {
            module.module.guild_ban_addition(ctx.clone(), guild_id, banned_user.clone()).await
        }
    }

    async fn guild_ban_removal(&self, ctx: Context, guild_id: GuildId, unbanned_user: User) {
        for module in &self.modules {
            module.module.guild_ban_removal(ctx.clone(), guild_id, unbanned_user.clone()).await
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        for module in &self.modules {
            module.module.guild_member_addition(ctx.clone(), new_member.clone()).await
        }
    }

    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
        for module in &self.modules {
            module.module.guild_member_removal(ctx.clone(), guild_id, user.clone(), member_data_if_available.clone()).await
        }
    }

    async fn guild_member_update(&self, ctx: Context, old_if_available: Option<Member>, new: Option<Member>, event: GuildMemberUpdateEvent) {
        for module in &self.modules {
            module.module.guild_member_update(ctx.clone(), old_if_available.clone(), new.clone(), event.clone()).await
        }
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        for module in &self.modules {
            module.module.message_delete(ctx.clone(), channel_id, deleted_message_id, guild_id).await;
        }
    }

    async fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId>) {
        for module in &self.modules {
            module.module.message_delete_bulk(ctx.clone(), channel_id, multiple_deleted_messages_ids.clone(), guild_id).await;
        }
    }

    async fn message_update(&self, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {
        for module in &self.modules {
            module.module.message_update(ctx.clone(), old_if_available.clone(), new.clone(), event.clone()).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        self.log_connector.init_for_channel(ChannelId::new(self.config.channels.log_channel), ctx.http.clone());

        self.update_commands(&ctx).await;

        for module in &self.modules {
            module.module.ready(ctx.clone(), ready.clone()).await;
            info!("Initialized module {}", module.module.name());
        }

        info!("Je suis prêt à botter des culs ! >:)");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        for module in &self.modules {
            module.module.interaction_create(ctx.clone(), interaction.clone()).await;
        }

        if let Interaction::Command(command) = interaction {
            for module in &self.modules {
                if module.commands.contains(&command.data.name) {
                    module.module.execute_command(ctx.clone(), command.data.name.as_str(), command.clone()).await;
                }
            }
        }
    }

    async fn guild_audit_log_entry_create(&self, ctx: Context, entry: AuditLogEntry, guild_id: GuildId) {
        for module in &self.modules {
            module.module.guild_audit_log_entry_create(ctx.clone(), entry.clone(), guild_id).await;
        }
    }
}