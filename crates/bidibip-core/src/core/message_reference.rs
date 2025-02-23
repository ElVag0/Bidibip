use std::hash::{Hash};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Http, Message, MessageId};
use serenity::Error;

#[derive(Default, Copy, Clone, Serialize, Deserialize, Hash, Eq)]
pub struct MessageReference {
    id: MessageId,
    channel: ChannelId
}

impl From<Message> for MessageReference {
    fn from(value: Message) -> Self {
        Self {
            id: value.id,
            channel: value.channel_id
        }
    }
}

impl MessageReference {
    #[allow(unused)]
    pub fn new(id: MessageId, channel: ChannelId) -> Self {
        Self {
            id,
            channel,
        }
    }

    #[allow(unused)]
    pub fn id(&self) -> MessageId {
        self.id
    }

    #[allow(unused)]
    pub fn channel(&self) -> ChannelId {
        self.channel
    }


    pub async fn message(&self, http: &Arc<Http>) -> Result<Message, Error> {
        Ok(self.channel.message(http, self.id).await?)
    }
}

impl PartialEq for MessageReference {
    fn eq(&self, other: &Self) -> bool {
        self.channel == other.channel && self.id == other.id
    }
}