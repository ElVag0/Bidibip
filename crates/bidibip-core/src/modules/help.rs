use crate::modules::{BidibipModule, CreateCommandDetailed};
use serenity::all::{CommandInteraction, Context, CreateMessage, EventHandler};
use serenity::builder::CreateEmbed;

pub struct Help {}

impl Help {

}

#[serenity::async_trait]
impl EventHandler for Help {}

#[serenity::async_trait]
impl BidibipModule for Help {
    fn name(&self) -> &'static str {
        "help"
    }

    fn fetch_commands(&self) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("help").description("Liste des commandes disponibles")]
    }

    async fn execute_command(&self, ctx: Context, _: &str, command: CommandInteraction) {
        let embed = CreateEmbed::new().title("Aide de Bidibip").description("Liste des commandes disponibles :");


        command.channel_id.send_message(&ctx.http, CreateMessage::new().embed(embed)).await;
    }
}