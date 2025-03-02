use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::TextOption;
use crate::modules::advertising::steps::{ResetStep, SubStep};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateEmbed, GuildChannel, Http, Message};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OpenEndedInfos {
    pub compensation: TextOption,
}

#[serenity::async_trait]
impl ResetStep for OpenEndedInfos {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        self.compensation.delete(http, thread).await?;
        Ok(())
    }
}

#[serenity::async_trait]
impl SubStep for OpenEndedInfos {
    fn fill_message(&self, main_fields: &mut Vec<(String, String, bool)>, _: &mut Vec<CreateEmbed>) {
        main_fields.push(("Rémunération".to_string(), match self.compensation.value() {
            None => { "[Donnée manquante]".to_string() }
            Some(value) => { value.clone() }
        }, true));
    }
    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.compensation.is_unset() {
            self.compensation.try_init(&ctx.http, thread, "Rémunération").await?;
            return Ok(false);
        }
        Ok(true)
    }

    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<(), BidibipError> {
        self.compensation.try_set(&ctx.http, thread, message).await?;
        Ok(())
    }

    async fn clicked_button(&mut self, ctx: &Context, thread: &ChannelId, action: &str) -> Result<(), BidibipError> {
        self.compensation.reset(&ctx.http, thread, action).await?;
        Ok(())
    }
}