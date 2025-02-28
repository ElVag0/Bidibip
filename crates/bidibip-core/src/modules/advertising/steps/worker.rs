use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::{create_multi_button_options, create_text_input_options, ButtonDescription};
use crate::modules::advertising::{Advertising, Step};
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildChannel, Message};

#[derive(Serialize, Deserialize, Clone)]
enum Location {
    Remote,
    Anywhere(Option<String>),
    OnSite(Option<String>),
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WorkerInfos {
    step: Step,
    location: Option<Location>,
    skills: Option<String>,
}

impl WorkerInfos {
    pub async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        match &self.location {
            None => {
                if self.step.test_or_set("LOCATION") { return Ok(false); }
                create_multi_button_options::<Advertising>(&ctx.http, &thread, "Souhaites-tu travailler à distance ou en présentiel ?", vec![
                    ButtonDescription::new("location_remote", "🌍 Distanciel"),
                    ButtonDescription::new("location_flex", "🤷‍♀️ Télétravail possible"),
                    ButtonDescription::new("location_onsite", "🏣 Présentiel uniquement")
                ]).await?;
                return Ok(false);
            }
            Some(value) => {
                match value {
                    Location::Remote => {}
                    Location::Anywhere(location) => {
                        if location.is_none() {
                            if self.step.test_or_set("LOCATION_ANYWHERE") { return Ok(false); }
                            create_text_input_options::<Advertising>(&ctx.http, &thread, "Indique ta ville / région", Some("location_anywhere")).await?;
                            return Ok(false);
                        }
                    }
                    Location::OnSite(location) => {
                        if location.is_none() {
                            if self.step.test_or_set("LOCATION_ON_SITE") { return Ok(false); }
                            create_text_input_options::<Advertising>(&ctx.http, &thread, "Indique ta ville / région", Some("location_on_site")).await?;
                            return Ok(false);
                        }
                    }
                }
            }
        }


        if self.skills.is_none() {
            if self.step.test_or_set("SKILLS") { return Ok(false); }
            create_text_input_options::<Advertising>(&ctx.http, &thread, "Quelles sont tes compétences ?", Some("skills")).await?;
            return Ok(false);
        }


        self.step.test_or_set("finished");

        Ok(true)
    }

    pub fn receive_message(&mut self, message: &Message) {
        match self.step.value() {
            "LOCATION_ANYWHERE" => {
                self.location = Some(Location::Anywhere(Some(message.content.clone())));
            }
            "LOCATION_ON_SITE" => {
                self.location = Some(Location::OnSite(Some(message.content.clone())));
            }
            "SKILLS" => {
                self.skills = Some(message.content.clone());
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
                self.location = Some(Location::Anywhere(None));
            }
            "location_onsite" => {
                self.location = Some(Location::OnSite(None));
            }
            &_ => {}
        }
    }
}