use std::collections::{HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc};
use serenity::all::{AuditLogEntry, ChannelId, Command, Context, GuildChannel, GuildId, GuildMemberUpdateEvent, Interaction, Member, Message, MessageId, MessageUpdateEvent, PartialGuildChannel, Ready, User};
use serenity::model::Permissions;
use serenity::prelude::EventHandler;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use crate::core::config::Config;
use crate::core::logger::DiscordLogConnector;
use crate::modules::{load_modules, BidibipModule};

pub struct GlobalInterface {
    log_connector: Arc<DiscordLogConnector>,
    shared_data: Arc<BidibipSharedData>,
}

pub struct ModuleData {
    pub module: Box<dyn BidibipModule>,
    pub command_names: HashSet<String>,
    pub name: String,
    #[allow(unused)]
    pub description: String,
}

#[derive(Default, Clone)]
pub struct PermissionData {
    member_permissions: Permissions,
    helper_permissions: Permissions,
    administrator_permissions: Permissions,
}

impl PermissionData {
    pub fn at_least_admin(&self) -> Permissions {
        self.administrator_permissions
    }
    pub fn at_least_helper(&self) -> Permissions {
        self.helper_permissions.intersection(self.administrator_permissions)
    }
    pub fn at_least_member(&self) -> Permissions {
        self.member_permissions.intersection(self.helper_permissions.intersection(self.administrator_permissions))
    }
}

pub struct BidibipSharedData {
    pub config: Arc<Config>,
    pub modules: RwLock<Vec<ModuleData>>,
    pub permissions: RwLock<PermissionData>,
}

impl GlobalInterface {
    // Default constructor
    // The log connector is used to provide the log channel to the logger
    pub async fn new(config: Arc<Config>, log_connector: Arc<DiscordLogConnector>) -> Self {
        let shared_data = Arc::new(BidibipSharedData { config, modules: Default::default(), permissions: Default::default() });
        load_modules(&shared_data).await;
        Self { shared_data, log_connector }
    }

    pub async fn fetch_roles(&self, ctx: &Context) {
        let roles = match GuildId::from(self.shared_data.config.server_id).roles(&ctx.http).await {
            Ok(roles) => { roles }
            Err(err) => {
                return error!("Failed to fetch roles : {}", err);
            }
        };

        let member_role = match roles.get(&self.shared_data.config.roles.member) {
            None => {
                return error!("Member role with id {} does not exists", self.shared_data.config.roles.member);
            }
            Some(role) => { role }
        };

        let admin_role = match roles.get(&self.shared_data.config.roles.administrator) {
            None => {
                return error!("Administrator role with id {} does not exists", self.shared_data.config.roles.administrator);
            }
            Some(role) => { role }
        };

        let helper_role = match roles.get(&self.shared_data.config.roles.helper) {
            None => {
                return error!("Helper role with id {} does not exists", self.shared_data.config.roles.helper);
            }
            Some(role) => { role }
        };

        let mut permissions = self.shared_data.permissions.write().await;

        permissions.member_permissions = member_role.permissions;
        permissions.administrator_permissions = admin_role.permissions;
        permissions.helper_permissions = helper_role.permissions;

        if member_role.permissions == admin_role.permissions {
            permissions.member_permissions = Permissions::all();
            permissions.administrator_permissions = Permissions::all();
            error!("Member and admin permissions are the same !!!")
        }

        if helper_role.permissions == admin_role.permissions {
            permissions.helper_permissions = Permissions::all();
            permissions.administrator_permissions = Permissions::all();
            error!("Helper and admin permissions are the same !!!")
        }

        if member_role.permissions == helper_role.permissions {
            permissions.member_permissions = Permissions::all();
            permissions.helper_permissions = Permissions::all();
            error!("Member and helper permissions are the same !!!")
        }
    }

    pub async fn update_commands(&self, ctx: &Context) {
        let mut outdated_commands = HashSet::new();
        let mut command_list = vec![];

        let permissions = self.shared_data.permissions.read().await.clone();
        for module in &*self.shared_data.modules.read().await {
            for command in module.module.fetch_commands(&permissions) {
                outdated_commands.insert(command.name.clone());
                command_list.push(command.into());
            }
        }

        let guild_id = self.shared_data.config.server_id;

        let glob = Command::get_global_commands(&ctx.http).await.unwrap();
        if !glob.is_empty() {
            match Command::set_global_commands(&ctx.http, vec![]).await {
                Ok(_) => { warn!("Cleaned up old global commands"); }
                Err(err) => { error!("Failed to cleanup old global commands : {}", err); }
            };
        }

        let mut outdated = false;
        for command in guild_id.get_commands(&ctx.http).await.unwrap() {
            if outdated_commands.contains(&command.name) {
                outdated_commands.remove(&command.name);
            } else {
                outdated = true;
            }
        }
        if !outdated_commands.is_empty() {
            outdated = true;
        }


        if outdated {
            match guild_id.set_commands(&ctx.http, command_list).await {
                Ok(_) => {
                    warn!("Updated command list");
                }
                Err(err) => {
                    error!("Failed to update command list : {}", err);
                }
            };
        }
    }
}

#[serenity::async_trait]
impl EventHandler for GlobalInterface {
    async fn channel_create(&self, ctx: Context, channel: GuildChannel) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.channel_create(ctx.clone(), channel.clone()).await;
        }
    }

    async fn channel_delete(&self, ctx: Context, channel: GuildChannel, messages: Option<Vec<Message>>) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.channel_delete(ctx.clone(), channel.clone(), messages.clone()).await;
        }
    }

    async fn guild_audit_log_entry_create(&self, ctx: Context, entry: AuditLogEntry, guild_id: GuildId) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.guild_audit_log_entry_create(ctx.clone(), entry.clone(), guild_id).await;
        }
    }

    async fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, banned_user: User) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.guild_ban_addition(ctx.clone(), guild_id, banned_user.clone()).await;
        }
    }

    async fn guild_ban_removal(&self, ctx: Context, guild_id: GuildId, unbanned_user: User) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.guild_ban_removal(ctx.clone(), guild_id, unbanned_user.clone()).await;
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.guild_member_addition(ctx.clone(), new_member.clone()).await;
        }
    }

    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.guild_member_removal(ctx.clone(), guild_id, user.clone(), member_data_if_available.clone()).await;
        }
    }

    async fn guild_member_update(&self, ctx: Context, old_if_available: Option<Member>, new: Option<Member>, event: GuildMemberUpdateEvent) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.guild_member_update(ctx.clone(), old_if_available.clone(), new.clone(), event.clone()).await;
        }
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.message(ctx.clone(), new_message.clone()).await;
        }
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.message_delete(ctx.clone(), channel_id, deleted_message_id, guild_id).await;
        }
    }

    async fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId>) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.message_delete_bulk(ctx.clone(), channel_id, multiple_deleted_messages_ids.clone(), guild_id).await;
        }
    }

    async fn message_update(&self, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.message_update(ctx.clone(), old_if_available.clone(), new.clone(), event.clone()).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        self.log_connector.init_for_channel(self.shared_data.config.channels.log_channel, ctx.http.clone());

        self.fetch_roles(&ctx).await;

        self.update_commands(&ctx).await;


        let permissions = self.shared_data.permissions.read().await.clone();
        for module in self.shared_data.modules.write().await.deref_mut() {
            let mut command_names = HashSet::new();
            for command in module.module.fetch_commands(&permissions) {
                command_names.insert(command.name);
            }
            module.command_names = command_names;
        }

        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.ready(ctx.clone(), ready.clone()).await;
            info!("Initialized module {}", module.name);
        }

        info!("Je suis prêt à botter des culs ! >:)");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.interaction_create(ctx.clone(), interaction.clone()).await;
        }

        if let Interaction::Command(command) = interaction {
            for module in self.shared_data.modules.read().await.deref() {
                if module.command_names.contains(&command.data.name) {
                    #[allow(unused)]
                    module.module.execute_command(ctx.clone(), command.data.name.as_str(), command.clone()).await;
                }
            }
        }
    }

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.thread_create(ctx.clone(), thread.clone()).await;
        }
    }

    async fn thread_delete(&self, ctx: Context, thread: PartialGuildChannel, full_thread_data: Option<GuildChannel>) {
        for module in self.shared_data.modules.read().await.deref() {
            #[allow(unused)]
            module.module.thread_delete(ctx.clone(), thread.clone(), full_thread_data.clone()).await;
        }
    }
}