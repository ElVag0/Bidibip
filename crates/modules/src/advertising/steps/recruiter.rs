use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Colour, Context, CreateEmbed, GuildChannel, Http, Interaction, Message};

use utils::error::BidibipError;
use crate::advertising::ad_utils::{ButtonOption, TextOption};
use crate::advertising::steps::{ResetStep, SubStep};

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

    fn clean_for_storage(&mut self) {
        match self {
            Location::Remote => {}
            Location::OnSiteFlex(v) => { v.clean_for_storage() }
            Location::OnSite(v) => { v.clean_for_storage() }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RecruiterInfos {
    pub location: ButtonOption<Location>,
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

    fn clean_for_storage(&mut self) {
        self.location.clean_for_storage();
        self.studio.clean_for_storage();
        self.responsibilities.clean_for_storage();
        self.qualifications.clean_for_storage();
    }
}

#[serenity::async_trait]
impl SubStep for RecruiterInfos {
    fn fill_message(&self, main_fields: &mut Vec<(String, String, bool)>, other_categories: &mut Vec<CreateEmbed>) {
        other_categories.push(
            CreateEmbed::new()
                .color(Colour::PURPLE)
                .title("Qualifications")
                .description(match self.qualifications.value() {
                    None => { "[Donnée manquante]" }
                    Some(value) => { value.as_str() }
                }));
        other_categories.push(
            CreateEmbed::new()
                .color(Colour::PURPLE)
                .title("Responsabilités")
                .description(match self.responsibilities.value() {
                    None => { "[Donnée manquante]" }
                    Some(value) => { value.as_str() }
                }));

        main_fields.push(("Emplacement".to_string(), match self.location.value() {
            None => { "[Donnée manquante]".to_string() }
            Some(value) => {
                match value {
                    Location::Remote => { "🌍 Distanciel uniquement".to_string() }
                    Location::OnSiteFlex(location) => {
                        format!("{} (🤷‍♀️ Télétravail possible)", match location.value() {
                            None => { "[Donnée manquante]" }
                            Some(location) => { location.as_str() }
                        })
                    }
                    Location::OnSite(location) => {
                        format!("{} (🏣 sur site)", match location.value() {
                            None => { "[Donnée manquante]" }
                            Some(location) => { location.as_str() }
                        })
                    }
                }
            }
        }, true));

        main_fields.push(("Entreprise".to_string(), match self.studio.value() {
            None => { "[Donnée manquante]".to_string() }
            Some(value) => { value.clone() }
        }, true));
    }
    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.location.is_unset() {
            if self.location.try_init(&ctx.http, thread, "Quelles sont les modalités de travail ?", vec![
                ("remote", "🌍 Distanciel", Location::Remote),
                ("flex", "🤷‍♀️ Télétravail possible", Location::OnSiteFlex(TextOption::default())),
                ("on_site", "🏣 Présentiel uniquement", Location::OnSite(TextOption::default())),
            ]).await? {
                return Ok(false);
            }
        }

        if let Some(value) = self.location.value_mut() {
            match value {
                Location::Remote => {}
                Location::OnSiteFlex(val) => {
                    if val.is_unset() {
                        if val.try_init(&ctx.http, thread, "Quelle est ta ville / région ?", false).await? {
                            return Ok(false);
                        }
                    }
                }
                Location::OnSite(val) => {
                    if val.is_unset() {
                        if val.try_init(&ctx.http, thread, "Quelle est ta ville / région ?", false).await? {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        if self.studio.is_unset() {
            if self.studio.try_init(&ctx.http, thread, "Quel est le nom de ton entreprise / studio ?", false).await? {
                return Ok(false);
            }
        }

        if self.responsibilities.is_unset() {
            if self.responsibilities.try_init(&ctx.http, thread, "Quelles sont les responsabilitées demandées ?", false).await? {
                return Ok(false);
            }
        }

        if self.qualifications.is_unset() {
            if self.qualifications.try_init(&ctx.http, thread, "Quelles sont les compétences requises ?", false).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<bool, BidibipError> {
        if let Some(value) = self.location.value_mut() {
            match value {
                Location::Remote => {}
                Location::OnSiteFlex(val) => { if val.try_set(&ctx.http, thread, message).await? { return Ok(true); } }
                Location::OnSite(val) => { if val.try_set(&ctx.http, thread, message).await? { return Ok(true); } }
            }
        }
        Ok(self.studio.try_set(&ctx.http, thread, message).await? ||
            self.qualifications.try_set(&ctx.http, thread, message).await? ||
            self.responsibilities.try_set(&ctx.http, thread, message).await?)
    }

    async fn on_interaction(&mut self, ctx: &Context, component: &Interaction) -> Result<bool, BidibipError> {
        Ok(self.studio.try_edit(&ctx.http, component).await? ||
            self.qualifications.try_edit(&ctx.http, component).await? ||
            self.responsibilities.try_edit(&ctx.http, component).await? ||
            self.location.try_set(&ctx.http, component).await?)
    }
}