use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::{ButtonOption, TextOption};
use crate::modules::advertising::steps::{ResetStep, SubStep};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateEmbed, GuildChannel, Http, Message};

#[derive(Serialize, Deserialize, Clone)]
pub enum Compensation {
    No,
    Yes(TextOption),
}
#[serenity::async_trait]
impl ResetStep for Compensation {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        match self {
            Compensation::No => { Ok(()) }
            Compensation::Yes(obj) => { obj.delete(http, thread).await }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct InternshipInfos {
    pub duration: TextOption,
    pub compensation: ButtonOption<Compensation>, // Paid or not
}

#[serenity::async_trait]
impl ResetStep for InternshipInfos {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        self.duration.delete(http, thread).await?;
        self.compensation.delete(http, thread).await?;
        Ok(())
    }
}

#[serenity::async_trait]
impl SubStep for InternshipInfos {
    fn fill_message(&self, main_fields: &mut Vec<(String, String, bool)>, _: &mut Vec<CreateEmbed>) {
        main_fields.push(("Durée".to_string(), match self.duration.value() {
            None => { "[Donnée manquante]".to_string() }
            Some(value) => { value.clone() }
        }, true));
        match self.compensation.value() {
            None => { main_fields.push(("Rémunération".to_string(), "[Donnée manquante]".to_string(), true)); }
            Some(value) => {
                match value {
                    Compensation::No => {}
                    Compensation::Yes(value) => {
                        main_fields.push(("Rémunération".to_string(), match value.value() {
                            None => { "[Donnée manquante]".to_string() }
                            Some(value) => { value.clone() }
                        }, true));
                    }
                }
            }
        }
    }

    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.duration.is_unset() {
            self.duration.try_init(&ctx.http, thread, "Durée du stage").await?;
            return Ok(false);
        }

        if self.compensation.is_unset() {
            self.compensation.try_init(&ctx.http, thread, "Le stage est-il rémunéré ?", vec![
                ("Oui", Compensation::Yes(TextOption::default())),
                ("No", Compensation::No),
            ]).await?;
            return Ok(false);
        }
        if let Some(compensation) = self.compensation.value_mut() {
            if let Compensation::Yes(value) = compensation {
                if value.is_unset() {
                    value.try_init(&ctx.http, thread, "Quelle est la gratification ? (4,35€/h minimum pour un stage de plus de 10 semaines)").await?;
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }


    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<(), BidibipError> {
        self.duration.try_set(&ctx.http, thread, message).await?;
        if let Some(compensation) = self.compensation.value_mut() {
            if let Compensation::Yes(value) = compensation {
                if value.is_unset() {
                    value.try_set(&ctx.http, thread, message).await?;
                }
            }
        }
        Ok(())
    }

    async fn clicked_button(&mut self, ctx: &Context, thread: &ChannelId, action: &str) -> Result<(), BidibipError> {
        self.duration.reset(&ctx.http, thread, action).await?;
        self.compensation.try_set(&ctx.http, thread, action).await?;
        if let Some(compensation) = self.compensation.value_mut() {
            if let Compensation::Yes(value) = compensation {
                if value.is_unset() {
                    value.reset(&ctx.http, thread, action).await?;
                }
            }
        }
        Ok(())
    }
}