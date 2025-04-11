use std::sync::{Arc};
use anyhow::Error;
use serenity::all::{AuditLogEntry, ChannelId, CommandInteraction, Context, GuildChannel, GuildId, GuildMemberUpdateEvent, Interaction, Member, Message, MessageId, MessageUpdateEvent, PartialGuildChannel, Ready, User};
use tracing::{error};
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::error::BidibipError;
use crate::core::global_interface::{BidibipSharedData, PermissionData};

mod say;
mod warn;
mod log;
mod history;
mod modo;
mod help;
mod utilities;
mod welcome;
mod reglement;
mod repost;
mod advertising;
mod user_count;
mod anti_spam;

#[serenity::async_trait]
pub trait BidibipModule: Sync + Send {
    // When one of the specified command is executed
    async fn execute_command(&self, _: Context, _: &str, _: CommandInteraction) -> Result<(), BidibipError> { Ok(()) }
    // Get a list of available commands for this module
    fn fetch_commands(&self, _: &PermissionData) -> Vec<CreateCommandDetailed> { vec![] }

    async fn channel_create(&self, _: Context, _: GuildChannel) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn channel_delete(&self, _: Context, _: GuildChannel, _: Option<Vec<Message>>) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn guild_audit_log_entry_create(&self, _: Context, _: AuditLogEntry, _: GuildId) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn guild_ban_addition(&self, _: Context, _: GuildId, _: User) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn guild_ban_removal(&self, _: Context, _: GuildId, _: User) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn guild_member_addition(&self, _: Context, _: Member) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn guild_member_removal(&self, _: Context, _: GuildId, _: User, _: Option<Member>) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn guild_member_update(&self, _: Context, _: Option<Member>, _: Option<Member>, _: GuildMemberUpdateEvent) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn message(&self, _: Context, _: Message) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn message_delete(&self, _: Context, _: ChannelId, _: MessageId, _: Option<GuildId>) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn message_delete_bulk(&self, _: Context, _: ChannelId, _: Vec<MessageId>, _: Option<GuildId>) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn message_update(&self, _: Context, _: Option<Message>, _: Option<Message>, _: MessageUpdateEvent) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn ready(&self, _: Context, _: Ready) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn interaction_create(&self, _: Context, _: Interaction) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn thread_create(&self, _: Context, _: GuildChannel) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn thread_delete(&self, _: Context, _: PartialGuildChannel, _: Option<GuildChannel>) -> Result<(), BidibipError> {
        Ok(())
    }
}

pub trait LoadModule<T: BidibipModule> {
    // Module display name
    fn name() -> &'static str;
    // Module display name
    fn description() -> &'static str;
    // Module constructor
    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<T, Error>;
}

async fn load_module_helper<T: 'static + LoadModule<T> + BidibipModule>(shared_data: &Arc<BidibipSharedData>) {
    match T::load(shared_data).await {
        Ok(module) => {
            shared_data.register_module(module).await;
        }
        Err(err) => { error!("Failed to load module {} : {}", T::name(), err) }
    }
}

pub async fn load_modules(shared_data: &Arc<BidibipSharedData>) {
    load_module_helper::<say::Say>(shared_data).await;
    load_module_helper::<warn::Warn>(shared_data).await;
    load_module_helper::<log::Log>(shared_data).await;
    load_module_helper::<history::History>(shared_data).await;
    load_module_helper::<help::Help>(shared_data).await;
    load_module_helper::<modo::Modo>(shared_data).await;
    load_module_helper::<utilities::Utilities>(shared_data).await;
    load_module_helper::<welcome::Welcome>(shared_data).await;
    load_module_helper::<reglement::Reglement>(shared_data).await;
    load_module_helper::<repost::Repost>(shared_data).await;
    load_module_helper::<advertising::Advertising>(shared_data).await;
    load_module_helper::<user_count::UserCount>(shared_data).await;
    load_module_helper::<anti_spam::AntiSpam>(shared_data).await;
}