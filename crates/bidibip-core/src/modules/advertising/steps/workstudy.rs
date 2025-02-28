use crate::core::error::BidibipError;
use crate::modules::advertising::ad_utils::create_text_input_options;
use crate::modules::advertising::{Advertising, Step};
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildChannel, Message};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WorkStudyInfos {
    step: Step,
    pub duration: Option<String>,
    pub compensation: Option<String>,
}

impl WorkStudyInfos {
    pub async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError> {
        if self.duration.is_none() {
            if self.step.test_or_set("DURATION") { return Ok(false); }
            create_text_input_options::<Advertising>(&ctx.http, &thread, "Durée du contrat", Some("duration")).await?;
            return Ok(false);
        }
        if self.compensation.is_none() {
            if self.step.test_or_set("COMPENSATION") { return Ok(false); }
            create_text_input_options::<Advertising>(&ctx.http, &thread, "Rémunération", Some("compensation")).await?;
            return Ok(false);
        }
        self.step.test_or_set("finished");
        Ok(true)
    }

    pub fn receive_message(&mut self, message: &Message) {
        match self.step.value() {
            "DURATION" => {
                self.duration = Some(message.content.clone())
            }
            "COMPENSATION" => {
                self.compensation = Some(message.content.clone())
            }
            _ => {}
        }
    }

    pub fn clicked_button(&mut self, action: &str) {
        match action {
            "edit_compensation" => {
                self.compensation = None;
            }
            "edit_duration" => {
                self.duration = None;
            }
            &_ => {}
        }
    }
}