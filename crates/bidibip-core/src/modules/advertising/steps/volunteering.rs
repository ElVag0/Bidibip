use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildChannel, Message};
use crate::core::error::BidibipError;
use crate::modules::advertising::Step;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VolunteeringInfos {
    step: Step,
}


impl VolunteeringInfos {
    pub async fn advance(&mut self, _: &Context, _: &GuildChannel) -> Result<bool, BidibipError> {
        Ok(true)
    }

    pub fn receive_message(&mut self, _: &Message) {
        match self.step.value() {
            _ => {}
        }
    }

    pub fn clicked_button(&mut self, action: &str) {
        match action {
            &_ => {}
        }
    }
}