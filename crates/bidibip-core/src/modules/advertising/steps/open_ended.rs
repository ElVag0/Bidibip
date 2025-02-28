
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildChannel, Message};
use crate::core::error::BidibipError;
use crate::modules::advertising::{Advertising, Step};
use crate::modules::advertising::ad_utils::create_text_input_options;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OpenEndedInfos {
    step: Step,
    pub compensation: Option<String>,
}


impl OpenEndedInfos {
    pub async fn advance(&mut self, ctx: &Context, thread:& GuildChannel) -> Result<bool, BidibipError> {
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
            &_ => {}
        }
    }
}