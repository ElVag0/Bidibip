use crate::core::error::BidibipError;
use serenity::all::{ChannelId, ComponentInteraction, Context, CreateEmbed, GuildChannel, Http, Message};

pub mod main;
mod internship;
mod recruiter;
mod worker;
mod volunteering;
mod freelance;
mod open_ended;
mod workstudy;
mod fixed_term;

#[serenity::async_trait]
pub trait SubStep: Sync + Send + ResetStep {
    async fn advance(&mut self, ctx: &Context, thread: &GuildChannel) -> Result<bool, BidibipError>;
    #[allow(unused)]
    async fn receive_message(&mut self, ctx: &Context, thread: &ChannelId, message: &Message) -> Result<(), BidibipError> { Ok(()) }
    #[allow(unused)]
    async fn clicked_button(&mut self, ctx: &Context, component: &ComponentInteraction) -> Result<bool, BidibipError> { Ok(false) }
    fn get_dependencies(&mut self) -> Vec<&mut dyn SubStep> { vec![] }
    #[allow(unused)]
    fn fill_message(&self, main_fields: &mut Vec<(String, String, bool)>, other_categories: &mut Vec<CreateEmbed>) {}
}

#[serenity::async_trait]
pub trait ResetStep {
    // Used to remove old messages
    #[allow(unused)]
    async fn delete(&mut self, http: &Http, thread: &ChannelId) -> Result<(), BidibipError> { Ok(()) }
}