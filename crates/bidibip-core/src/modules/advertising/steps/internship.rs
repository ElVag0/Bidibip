use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::{create_multi_button_options, create_text_input_options, ButtonDescription};
use crate::modules::advertising::{Advertising, Step};
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildChannel, Message};

#[derive(Serialize, Deserialize, Clone)]
pub enum Compensation {
    None,
    Unset,
    Set(String)
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct InternshipInfos {
    step: Step,
    pub duration: Option<String>,
    pub compensation: Option<Compensation>, // Paid or not
}

impl InternshipInfos {
    pub async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.duration.is_none() {
            if self.step.test_or_set("DURATION") { return Ok(false); }
            create_text_input_options::<Advertising>(&ctx.http, &thread, "Durée du stage", Some("duration")).await?;
            return Ok(false);
        }

        match &self.compensation {
            None => {
                if self.step.test_or_set("REMUNERATION") { return Ok(false); }
                create_multi_button_options::<Advertising>(&ctx.http, &thread, "Le stage est-il rémunéré ?", vec![
                    ButtonDescription::new("remuneration_yes", "oui"),
                    ButtonDescription::new("remuneration_no", "non"),
                ]).await?;
                return Ok(false);
            }
            Some(value) => {
                match value {
                    Compensation::Unset => {
                        if self.step.test_or_set("REMUNERATION_VALUE") { return Ok(false); }
                        create_text_input_options::<Advertising>(&ctx.http, &thread, "Quelle est la gratification ? (4,35€/h minimum pour un stage de plus de 10 semaines)", Some("compensation")).await?;
                        return Ok(false);}
                    _ => {}
                }
            }
        }

        self.step.test_or_set("finished");
        Ok(true)
    }


    pub fn receive_message(&mut self, message: &Message) {
        match self.step.value() {
            "DURATION" => {
                self.duration = Some(message.content.clone())
            }
            "REMUNERATION_VALUE" => {
                self.compensation = Some(Compensation::Set(message.content.clone()))
            }
            _ => {}
        }
    }

    pub fn clicked_button(&mut self, action: &str) {
        match action {
            "remuneration_yes" => {
                self.compensation = Some(Compensation::Unset);
            }
            "remuneration_no" => {
                self.compensation = Some(Compensation::None);
            }
            "edit_compensation" => {
                self.compensation = Some(Compensation::Unset);
            }
            "edit_duration" => {
                self.duration = None;
            }
            &_ => {}
        }
    }
}