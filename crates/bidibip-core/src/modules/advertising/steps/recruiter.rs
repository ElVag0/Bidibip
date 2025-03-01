use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::{ButtonOption, TextOption};
use crate::modules::advertising::steps::{ResetStep, SubStep};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, GuildChannel, Http, Message};


#[derive(Serialize, Deserialize, Clone)]
pub enum Location {
    Remote,
    OnSiteFlex(TextOption),
    OnSite(TextOption),
}

#[serenity::async_trait]
impl ResetStep for Location {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        match self {
            Location::Remote => { Ok(()) }
            Location::OnSiteFlex(val) => { val.delete(http, thread).await }
            Location::OnSite(val) => { val.delete(http, thread).await }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RecruiterInfos {
    location: ButtonOption<Location>,
    studio: TextOption,
    responsibilities: TextOption,
    qualifications: TextOption,
}

#[serenity::async_trait]
impl ResetStep for RecruiterInfos {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        self.location.delete(http, thread).await?;
        self.studio.delete(http, thread).await?;
        self.responsibilities.delete(http, thread).await?;
        self.qualifications.delete(http, thread).await?;
        Ok(())
    }
}

#[serenity::async_trait]
impl SubStep for RecruiterInfos {
    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.location.is_unset() {
            self.location.try_init(&ctx.http, thread, "Quelles sont les modalités de travail ?", vec![
                ("🌍 Distanciel", Location::Remote),
                ("🤷‍♀️ Télétravail possible", Location::OnSiteFlex(TextOption::default())),
                ("🏣 Présentiel uniquement", Location::OnSiteFlex(TextOption::default())),
            ]).await?;
            return Ok(false);
        }

        if let Some(value) = self.location.value_mut() {
            match value {
                Location::Remote => {}
                Location::OnSiteFlex(val) => {
                    if val.is_unset() {
                        val.try_init(&ctx.http, thread, "Quelle est ta ville / région ?").await?;
                        return Ok(false);
                    }
                }
                Location::OnSite(val) => {
                    if val.is_unset() {
                        val.try_init(&ctx.http, thread, "Quelle est ta ville / région ?").await?;
                        return Ok(false);
                    }
                }
            }
        }

        if self.studio.is_unset() {
            self.studio.try_init(&ctx.http, thread, "Quel est le nom de ton entreprise / studio ?").await?;
            return Ok(false);
        }

        if self.responsibilities.is_unset() {
            self.responsibilities.try_init(&ctx.http, thread, "Quelles sont les responsabilitées demandées ?").await?;
            return Ok(false);
        }

        if self.qualifications.is_unset() {
            self.qualifications.try_init(&ctx.http, thread, "Quelles sont les compétences requises ?").await?;
            return Ok(false);
        }

        Ok(true)
    }

    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<(), BidibipError> {
        self.studio.try_set(&ctx.http, thread, message).await?;
        self.qualifications.try_set(&ctx.http, thread, message).await?;
        self.responsibilities.try_set(&ctx.http, thread, message).await?;

        if let Some(value) = self.location.value_mut() {
            match value {
                Location::Remote => {}
                Location::OnSiteFlex(val) => { val.try_set(&ctx.http, thread, message).await?; }
                Location::OnSite(val) => { val.try_set(&ctx.http, thread, message).await?; }
            }
        }
        Ok(())
    }

    async fn clicked_button(&mut self, ctx: &Context, thread: &ChannelId, action: &str) -> Result<(), BidibipError> {
        self.studio.reset(&ctx.http, thread, action).await?;
        self.qualifications.reset(&ctx.http, thread, action).await?;
        self.responsibilities.reset(&ctx.http, thread, action).await?;
        self.location.try_set(&ctx.http, thread, action).await?;
        Ok(())
    }
}