use anyhow::Error;
use serde::{Deserialize, Serialize};
use crate::modules::{BidibipModule, CreateCommandDetailed};
use serenity::all::{CommandInteraction, CommandOptionType, CommandType, Context, CreateCommand, CreateCommandOption, CreateMessage, EventHandler, ResolvedValue};
use tracing::error;
use crate::core::utilities::{json_to_message, CommandHelper, OptionHelper, ResultDebug};

#[derive(Serialize, Deserialize)]
pub struct Say {}

#[serenity::async_trait]
impl BidibipModule for Say {
    fn name(&self) -> &'static str {
        "Say"
    }

    fn fetch_commands(&self) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("say")
                  .description("Ma parole sera la votre")
                  .kind(CommandType::ChatInput)
                  .add_option(CreateCommandOption::new(CommandOptionType::String, "message", "Que dois-je dire à votre place ?"))
                  .add_option(CreateCommandOption::new(CommandOptionType::Attachment, "fichier", "Fichier json pour afficher un message formaté"))]
    }

    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) {
        if name == "say" {
            if let Some(option) = command.data.options().find("message") {
                if let ResolvedValue::String(str) = option {
                    command.channel_id.say(&ctx.http, str).await.on_fail("Failed to send message in channel");
                    command.skip(&ctx.http).await;
                }
            } else if let Some(option) = command.data.options().find("fichier") {
                if let ResolvedValue::Attachment(attachment) = option {
                    let message = String::from_utf8(match attachment.download().await {
                        Ok(download) => { download }
                        Err(err) => { return error!("Failed to download attachment : {}", err) }
                    }).expect("Our bytes should be valid utf8");
                    match json_to_message(message) {
                        Ok(message) => {
                            for message in message {
                                command.channel_id.send_message(&ctx.http, message).await.on_fail("Failed to send message in channel");
                            }
                        }
                        Err(err) => { error!("Invalid json_to_message : {}", err) }
                    }
                    command.skip(&ctx.http).await;
                }
            } else {
                command.respond_user_error(&ctx.http, "Tu n'as pas précisé ce que je dois annoncer !").await;
            }
        }
    }
}
impl EventHandler for Say {}