use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::{ButtonOption, TextOption};
use crate::modules::advertising::steps::{ResetStep, SubStep};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Colour, Context, CreateEmbed, GuildChannel, Http, Interaction, Message};

#[derive(Serialize, Deserialize, Clone)]
pub enum Location {
    Remote,
    Anywhere(TextOption),
    OnSite(TextOption),
}
#[serenity::async_trait]
impl ResetStep for Location {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        match self {
            Location::Remote => { Ok(()) }
            Location::Anywhere(obj) => { obj.delete(http, thread).await }
            Location::OnSite(obj) => { obj.delete(http, thread).await }
        }
    }

    fn clean_for_storage(&mut self) {
        match self {
            Location::Remote => {}
            Location::Anywhere(v) => {v.clean_for_storage()}
            Location::OnSite(v) => {v.clean_for_storage()}
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WorkerInfos {
    pub location: ButtonOption<Location>,
    skills: TextOption,
}

#[serenity::async_trait]
impl ResetStep for WorkerInfos {
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> {
        self.location.delete(http, thread).await?;
        self.skills.delete(http, thread).await?;
        Ok(())
    }

    fn clean_for_storage(&mut self) {
        self.location.clean_for_storage();
        self.skills.clean_for_storage();
    }
}

#[serenity::async_trait]
impl SubStep for WorkerInfos {
    fn fill_message(&self, main_fields: &mut Vec<(String, String, bool)>, other_categories: &mut Vec<CreateEmbed>) {
        other_categories.push(
            CreateEmbed::new()
                .color(Colour::PURPLE)
                .title("Compétences")
                .description(match self.skills.value() {
                    None => { "[Donnée manquante]" }
                    Some(value) => { value.as_str() }
                }));

        main_fields.push(("Emplacement".to_string(), match self.location.value() {
            None => { "[Donnée manquante]".to_string() }
            Some(value) => {
                match value {
                    Location::Remote => { "🌍 Distanciel uniquement".to_string() }
                    Location::Anywhere(location) => {
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
    }

    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.location.is_unset() {
            if self.location.try_init(&ctx.http, thread, "Souhaites-tu travailler à distance ou en présentiel ?", vec![
                ("remote", "🌍 Distanciel", Location::Remote),
                ("any", "🤷‍♀️ Télétravail possible", Location::Anywhere(TextOption::default())),
                ("on_site", "🏣 Présentiel uniquement", Location::OnSite(TextOption::default()))]).await? {
                return Ok(false);
            }
        }

        if let Some(location) = self.location.value_mut() {
            match location {
                Location::Remote => {}
                Location::Anywhere(loc) => {
                    if loc.is_unset() {
                        if loc.try_init(&ctx.http, thread, "Indique ta ville / région").await? {
                            return Ok(false);
                        }
                    }
                }
                Location::OnSite(loc) => {
                    if loc.is_unset() {
                        if loc.try_init(&ctx.http, thread, "Indique ta ville / région").await? {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        if self.skills.is_unset() {
            if self.skills.try_init(&ctx.http, thread, "Quelles sont tes compétences ?").await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<(), BidibipError> {
        if self.skills.try_set(&ctx.http, thread, message).await? { return Ok(()); }

        if let Some(location) = self.location.value_mut() {
            match location {
                Location::Remote => {}
                Location::Anywhere(loc) => { loc.try_set(&ctx.http, thread, message).await?; }
                Location::OnSite(loc) => { loc.try_set(&ctx.http, thread, message).await?; }
            }
        }
        Ok(())
    }

    async fn on_interaction(&mut self, ctx: &Context, component: &Interaction) -> Result<bool, BidibipError> {
        Ok(self.location.try_set(&ctx.http, component).await? || self.skills.try_edit(&ctx.http, component).await?)
    }
}