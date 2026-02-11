use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc};
use serenity::all::{AuditLogEntry, ChannelId, Command, Context, GuildChannel, GuildId, GuildMemberUpdateEvent, Interaction, Member, Message, MessageId, MessageUpdateEvent, PartialGuildChannel, Ready, User};
use serenity::model::Permissions;
use serenity::prelude::EventHandler;
use tokio::sync::{RwLock};
use tracing::{error, info, warn};
use crate::config::Config;
use crate::logger::DiscordLogConnector;
use crate::module::{BidibipModule, LoadModule};

pub struct GlobalInterface {
    log_connector: Arc<DiscordLogConnector>,
    shared_data: Arc<BidibipSharedData>,
}

pub struct ModuleData {
    pub module: Box<dyn BidibipModule>,
    command_names: RwLock<HashSet<String>>,
    pub name: String,
    #[allow(unused)]
    pub description: String,
}

impl ModuleData {
    pub fn new(name: String, module: Box<dyn BidibipModule>, description: String) -> Self {
        Self {
            module,
            command_names: Default::default(),
            name,
            description,
        }
    }
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
    available_modules: RwLock<HashMap<String, Arc<ModuleData>>>,
    enabled_modules: RwLock<HashMap<String, Arc<ModuleData>>>,
    disabled_modules: RwLock<HashSet<String>>,
    pub permissions: RwLock<PermissionData>,
}

impl BidibipSharedData {
    pub async fn register_module<T: 'static + LoadModule<T> + BidibipModule>(&self, module: T) {
        let module = Arc::new(ModuleData::new(T::name().to_string(), Box::new(module), T::description().to_string()));
        if !self.disabled_modules.read().await.contains(&T::name().to_string()) {
            self.enabled_modules.write().await.insert(T::name().to_string(), module.clone());
        }
        self.available_modules.write().await.insert(T::name().to_string(), module);
    }

    pub async fn available_modules(&self) -> HashSet<String> {
        let mut modules = HashSet::<String>::new();
        for module in self.available_modules.read().await.keys() {
            modules.insert(module.clone());
        }
        modules
    }

    pub async fn set_module_enabled(&self, ctx: &Context, name: &str, enabled: bool, update_commands: bool) {
        if enabled {
            self.disabled_modules.write().await.remove(&name.to_string());
            if let Some(module) = self.available_modules.read().await.get(&name.to_string()) {
                self.enabled_modules.write().await.insert(name.to_string(), module.clone());
            }
        } else {
            self.disabled_modules.write().await.insert(name.to_string());
            self.enabled_modules.write().await.remove(&name.to_string());
        }
        if update_commands {
            self.update_commands(ctx).await;
        }
    }

    pub async fn get_enabled_modules(&self) -> Vec<Arc<ModuleData>> {
        let mut modules = vec![];
        for module in self.enabled_modules.read().await.deref() {
            modules.push(module.1.clone())
        }
        modules
    }

    pub async fn get_disabled_modules(&self) -> Vec<Arc<ModuleData>> {
        let mut modules = vec![];
        for module in self.available_modules.read().await.deref() {
            if self.disabled_modules.read().await.contains(module.0) {
                modules.push(module.1.clone())
            }
        }
        modules
    }

    /// Update command list, and register or remove updated commands
    pub async fn update_commands(&self, ctx: &Context) {
        let mut outdated_commands = HashSet::new();
        let mut command_list = vec![];

        let permissions = self.permissions.read().await.clone();
        for module in &*self.get_enabled_modules().await {
            for command in module.module.fetch_commands(&permissions) {
                outdated_commands.insert(command.name.clone());
                command_list.push(command.into());
            }
        }

        let guild_id = Config::get().server_id;

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


impl GlobalInterface {
    /// Default constructor
    /// The log connector is used to provide the log channel to the logger
    pub async fn new(log_connector: Arc<DiscordLogConnector>) -> Self {
        let shared_data = Arc::new(BidibipSharedData { available_modules: Default::default(), enabled_modules: Default::default(), permissions: Default::default(), disabled_modules: Default::default() });
        Self { shared_data, log_connector }
    }

    pub fn shared_data(&self) -> &Arc<BidibipSharedData> {
        &self.shared_data
    }

    /// Update roles from configured roles id (used to handle permissions)
    async fn fetch_roles(&self, ctx: &Context) {
        let roles = match GuildId::from(Config::get().server_id).roles(&ctx.http).await {
            Ok(roles) => { roles }
            Err(err) => {
                return error!("Failed to fetch roles : {}", err);
            }
        };

        let member_role = match roles.get(&Config::get().roles.member) {
            None => {
                return error!("Member role with id {} does not exists", Config::get().roles.member);
            }
            Some(role) => { role }
        };

        let admin_role = match roles.get(&Config::get().roles.administrator) {
            None => {
                return error!("Administrator role with id {} does not exists", Config::get().roles.administrator);
            }
            Some(role) => { role }
        };

        let helper_role = match roles.get(&Config::get().roles.helper) {
            None => {
                return error!("Helper role with id {} does not exists", Config::get().roles.helper);
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
}

#[serenity::async_trait]
impl EventHandler for GlobalInterface {
    async fn channel_create(&self, ctx: Context, channel: GuildChannel) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.channel_create(ctx.clone(), channel.clone()).await;
        }
    }

    async fn channel_delete(&self, ctx: Context, channel: GuildChannel, messages: Option<Vec<Message>>) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.channel_delete(ctx.clone(), channel.clone(), messages.clone()).await;
        }
    }

    async fn guild_audit_log_entry_create(&self, ctx: Context, entry: AuditLogEntry, guild_id: GuildId) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.guild_audit_log_entry_create(ctx.clone(), entry.clone(), guild_id).await;
        }
    }

    async fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, banned_user: User) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.guild_ban_addition(ctx.clone(), guild_id, banned_user.clone()).await;
        }
    }

    async fn guild_ban_removal(&self, ctx: Context, guild_id: GuildId, unbanned_user: User) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.guild_ban_removal(ctx.clone(), guild_id, unbanned_user.clone()).await;
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.guild_member_addition(ctx.clone(), new_member.clone()).await;
        }
    }

    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.guild_member_removal(ctx.clone(), guild_id, user.clone(), member_data_if_available.clone()).await;
        }
    }

    async fn guild_member_update(&self, ctx: Context, old_if_available: Option<Member>, new: Option<Member>, event: GuildMemberUpdateEvent) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.guild_member_update(ctx.clone(), old_if_available.clone(), new.clone(), event.clone()).await;
        }
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.message(ctx.clone(), new_message.clone()).await;
        }
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.message_delete(ctx.clone(), channel_id, deleted_message_id, guild_id).await;
        }
    }

    async fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId>) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.message_delete_bulk(ctx.clone(), channel_id, multiple_deleted_messages_ids.clone(), guild_id).await;
        }
    }

    async fn message_update(&self, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.message_update(ctx.clone(), old_if_available.clone(), new.clone(), event.clone()).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {

        //migrate(&ctx).await;


        self.log_connector.init_for_channel(Config::get().channels.log_channel, ctx.http.clone());

        self.fetch_roles(&ctx).await;

        let permissions = self.shared_data.permissions.read().await.clone();
        for module in self.shared_data.get_enabled_modules().await {
            let mut command_names = HashSet::new();
            for command in module.module.fetch_commands(&permissions) {
                command_names.insert(command.name);
            }
            *module.command_names.write().await = command_names;
        }

        let mut init_message = String::new();

        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.ready(ctx.clone(), ready.clone()).await;
            init_message += format!("{}, ", module.name).as_str();
        }
        info!("Initialized modules {}", init_message);

        self.shared_data.update_commands(&ctx).await;

        info!("Je suis prêt à botter des culs ! >:)");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.interaction_create(ctx.clone(), interaction.clone()).await;
        }

        if let Interaction::Command(command) = interaction {
            for module in self.shared_data.get_enabled_modules().await {
                if module.command_names.read().await.contains(&command.data.name) {
                    #[allow(unused)]
                    module.module.execute_command(ctx.clone(), command.data.name.as_str(), command.clone()).await;
                }
            }
        }
    }

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.thread_create(ctx.clone(), thread.clone()).await;
        }
    }

    async fn thread_delete(&self, ctx: Context, thread: PartialGuildChannel, full_thread_data: Option<GuildChannel>) {
        for module in self.shared_data.get_enabled_modules().await {
            #[allow(unused)]
            module.module.thread_delete(ctx.clone(), thread.clone(), full_thread_data.clone()).await;
        }
    }
}