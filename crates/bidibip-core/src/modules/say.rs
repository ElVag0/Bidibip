use crate::modules::Module;
use serenity::all::{CommandOptionType, CommandType, Context, CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, Interaction, ResolvedOption, ResolvedValue};
use tracing::error;

pub struct Say;
#[serenity::async_trait]
impl EventHandler for Say {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.data.name.as_str() == "say" {
                for option in &command.data.options() {
                    if option.name == "message" {
                        if let ResolvedValue::String(str) = option.value {
                            command.channel_id.say(&ctx.http, str).await;
                        }
                    } else if option.name == "fichier" {
                        if let ResolvedValue::Attachment(attachment) = option.value {
                            let message = String::from_utf8(attachment.download().await.unwrap()).expect("Our bytes should be valid utf8");
                            command.channel_id.say(&ctx.http, message).await;
                        }
                    }
                }

                command.defer(&ctx.http).await;
                command.delete_response(&ctx.http).await;
            }
        }
    }
}

impl Module for Say {
    fn name(&self) -> &'static str {
        "Say"
    }

    fn fetch_command(&self) -> Vec<(String, CreateCommand)> {
        vec![("say".to_string(),
              CreateCommand::new("say")
                  .description("Ma parole sera la votre")
                  .kind(CommandType::ChatInput)
                  .add_option(CreateCommandOption::new(CommandOptionType::String, "message", "Que dois-je dire à votre place ?"))
                  .add_option(CreateCommandOption::new(CommandOptionType::Attachment, "fichier", "Fichier json pour afficher un message formaté")))]
    }
}