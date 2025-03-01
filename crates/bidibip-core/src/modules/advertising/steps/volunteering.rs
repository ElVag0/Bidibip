use crate::core::error::BidibipError;
use crate::modules::advertising::steps::{ResetStep, SubStep};
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildChannel};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VolunteeringInfos {
}

#[serenity::async_trait]
impl ResetStep for VolunteeringInfos {}

#[serenity::async_trait]
impl SubStep for VolunteeringInfos {
    async fn advance(&mut self, _: &Context, _: &GuildChannel) -> Result<bool, BidibipError> {
        Ok(true)
    }
}