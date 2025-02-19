use serenity::all::{ChannelId, Context, EventHandler, GuildId, Message, MessageId, MessageUpdateEvent};
use crate::modules::BidibipModule;

pub struct History {}

#[serenity::async_trait]
impl EventHandler for History{
    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId>) {
        todo!()
    }

    async fn message_delete_bulk(&self, ctx: Context, channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId>) {
        todo!()
    }

    async fn message_update(&self, ctx: Context, old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent) {
        todo!()
    }
}

impl BidibipModule for History {
    fn name(&self) -> &'static str {
        "Log"
    }
}