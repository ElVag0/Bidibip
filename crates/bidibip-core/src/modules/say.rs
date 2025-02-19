use serde::{Deserialize, Serialize};
use crate::modules::{BidibipModule};
use serenity::all::{CommandInteraction, CommandOptionType, CommandType, Context, CreateCommand, CreateCommandOption, EventHandler, ResolvedValue};
use crate::core::utilities::{json_to_message, CommandHelper, OptionHelper, ResultDebug};

#[derive(Serialize, Deserialize)]
pub struct Say {
}

#[serenity::async_trait]
impl BidibipModule for Say {
    fn name(&self) -> &'static str {
        "Say"
    }

    fn fetch_commands(&self) -> Vec<(String, CreateCommand)> {
        vec![("say".to_string(),
              CreateCommand::new("say")
                  .description("Ma parole sera la votre")
                  .kind(CommandType::ChatInput)
                  .add_option(CreateCommandOption::new(CommandOptionType::String, "message", "Que dois-je dire à votre place ?"))
                  .add_option(CreateCommandOption::new(CommandOptionType::Attachment, "fichier", "Fichier json pour afficher un message formaté")))]
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
                    let message = String::from_utf8(attachment.download().await.unwrap()).expect("Our bytes should be valid utf8");
                    for message in json_to_message(message) {
                        command.channel_id.send_message(&ctx.http, message).await.on_fail("Failed to send message in channel");
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