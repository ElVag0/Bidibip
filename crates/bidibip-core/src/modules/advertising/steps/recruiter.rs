use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::{create_multi_button_options, create_text_input_options, ButtonDescription, Location};
use crate::modules::advertising::{Advertising, Step};
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildChannel, Message};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RecruiterInfos {
    step: Step,
    location: Option<Location>,
    studio: Option<String>,
    responsibilities: Option<String>,
    qualifications: Option<String>,
}

impl RecruiterInfos {
    pub async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {

        if self.location.is_none() {
            if self.step.test_or_set("LOCATION") { return Ok(false); }
            create_multi_button_options::<Advertising>(&ctx.http, &thread, "Quelles sont les modalités de travail ?", vec![
                ButtonDescription::new("location_remote", "🌍 Distanciel"),
                ButtonDescription::new("location_flex", "🤷‍♀️ Télétravail possible"),
                ButtonDescription::new("location_onsite", "🏣 Présentiel uniquement")
            ]).await?;
            return Ok(false);
        }

        if self.studio.is_none() {
            if self.step.test_or_set("STUDIO") { return Ok(false); }
            create_text_input_options::<Advertising>(&ctx.http, &thread, "Quel est le nom de ton entreprise / studio ?", Some("studio")).await?;
            return Ok(false);
        }

        if self.responsibilities.is_none() {
            if self.step.test_or_set("RESPONSIBILITIES") { return Ok(false); }
            create_text_input_options::<Advertising>(&ctx.http, &thread, "Quelles sont les responsabilitées demandées ?", Some("responsibilities")).await?;
            return Ok(false);
        }

        if self.qualifications.is_none() {
            if self.step.test_or_set("QUALIFICATIONS") { return Ok(false); }
            create_text_input_options::<Advertising>(&ctx.http, &thread, "Quelles sont les compétences requises ?", Some("qualifications")).await?;
            return Ok(false);
        }

        self.step.test_or_set("finished");

        Ok(true)
    }

    pub fn receive_message(&mut self, message: &Message) {
        match self.step.value() {
            "STUDIO" => {
                self.studio = Some(message.content.clone());
            }
            "QUALIFICATIONS" => {
                self.qualifications = Some(message.content.clone());
            }
            "RESPONSIBILITIES" => {
                self.responsibilities = Some(message.content.clone());
            }
            _ => {}
        }
    }

    pub fn clicked_button(&mut self, action: &str) {
        match action {
            /***************************/
            "location_remote" => {
                self.location = Some(Location::Remote);
            }
            "location_flex" => {
                self.location = Some(Location::OnSiteFlex(None));
            }
            "location_onsite" => {
                self.location = Some(Location::OnSite(None));
            }
            /***************************/
            "edit_responsibilities" => {
                self.responsibilities = None;
            }
            "edit_qualifications" => {
                self.qualifications = None;
            }
            &_ => {}
        }
    }
}