use utils::error::BidibipError;
use serde::{Deserialize, Serialize};
use serenity::all::{Context, GuildChannel};
use crate::advertising::steps::{ResetStep, SubStep};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VolunteeringInfos {
}

#[serenity::async_trait]
impl ResetStep for VolunteeringInfos {
    fn clean_for_storage(&mut self) {}
}

#[serenity::async_trait]
impl SubStep for VolunteeringInfos {
    async fn advance(&mut self, _: &Context, _: &GuildChannel) -> Result<bool, BidibipError> {
        Ok(true)
    }
}