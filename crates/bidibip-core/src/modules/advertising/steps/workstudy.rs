use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::{TextOption};
use crate::modules::advertising::steps::{ResetStep, SubStep};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, GuildChannel, Http, Message};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WorkStudyInfos {
    pub duration: TextOption,
    pub compensation: TextOption,
}

#[serenity::async_trait]
impl ResetStep for WorkStudyInfos {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        self.duration.delete(http, thread).await?;
        self.compensation.delete(http, thread).await?;
        Ok(())
    }
}

#[serenity::async_trait]
impl SubStep for WorkStudyInfos {
    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.duration.is_unset() {
            self.duration.try_init(&ctx.http, thread, "Durée du contrat").await?;
            return Ok(false);
        }

        if self.compensation.is_unset() {
            self.compensation.try_init(&ctx.http, thread, "Rémunération").await?;
            return Ok(false);
        }
        Ok(true)
    }

    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<(), BidibipError> {
        self.duration.try_set(&ctx.http, thread, message).await?;
        self.compensation.try_set(&ctx.http, thread, message).await?;
        Ok(())
    }

    async fn clicked_button(&mut self, ctx: &Context, thread: &ChannelId, action: &str) -> Result<(), BidibipError> {
        self.duration.reset(&ctx.http, thread, action).await?;
        self.compensation.reset(&ctx.http, thread, action).await?;
        Ok(())
    }
}