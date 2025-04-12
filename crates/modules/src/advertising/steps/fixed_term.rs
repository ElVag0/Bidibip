use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateEmbed, GuildChannel, Http, Interaction, Message};
use crate::advertising::ad_utils::TextOption;
use crate::advertising::steps::{ResetStep, SubStep};
use utils::error::BidibipError;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FixedTermInfos {
    pub duration: TextOption,
    pub compensation: TextOption,
}

#[serenity::async_trait]
impl ResetStep for FixedTermInfos {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        self.duration.delete(http, thread).await?;
        self.compensation.delete(http, thread).await?;
        Ok(())
    }

    fn clean_for_storage(&mut self) {
        self.duration.clean_for_storage();
        self.compensation.clean_for_storage()
    }
}

#[serenity::async_trait]
impl SubStep for FixedTermInfos {
    fn fill_message(&self, main_fields: &mut Vec<(String, String, bool)>, _: &mut Vec<CreateEmbed>) {
        main_fields.push(("Durée".to_string(), match self.duration.value() {
            None => { "[Donnée manquante]".to_string() }
            Some(value) => { value.clone() }
        }, true));
        main_fields.push(("Rémunération".to_string(), match self.compensation.value() {
            None => { "[Donnée manquante]".to_string() }
            Some(value) => { value.clone() }
        }, true));
    }

    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.duration.is_unset() {
            self.duration.try_init(&ctx.http, thread, "Durée du contrat", false).await?;
            return Ok(false);
        }

        if self.compensation.is_unset() {
            if self.compensation.try_init(&ctx.http, thread, "Rémunération", false).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<bool, BidibipError> {
        Ok(self.duration.try_set(&ctx.http, thread, message).await? ||
            self.compensation.try_set(&ctx.http, thread, message).await?)
    }

    async fn on_interaction(&mut self, ctx: &Context, component: &Interaction) -> Result<bool, BidibipError> {
        Ok(self.duration.try_edit(&ctx.http, component).await? || self.compensation.try_edit(&ctx.http, component).await?)
    }
}