use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use crate::modules::{BidibipModule, CreateCommandDetailed, LoadModule};
use serenity::all::{CommandInteraction, CommandOptionType, CommandType, Context, CreateCommandOption, ResolvedValue};
use crate::core::error::BidibipError;
use crate::core::json_to_message::json_to_message;
use crate::core::global_interface::{BidibipSharedData, PermissionData};
use crate::core::utilities::{CommandHelper, OptionHelper, ResultDebug};
use crate::on_fail;

#[derive(Serialize, Deserialize)]
pub struct Say {}

impl LoadModule<Say> for Say {
    fn name() -> &'static str {
        "say"
    }

    fn description() -> &'static str {
        "Fait parler bidibip"
    }

    async fn load(_: &Arc<BidibipSharedData>) -> Result<Say, Error> {
        Ok(Say {})
    }
}

#[serenity::async_trait]
impl BidibipModule for Say {
    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        if name == "say" {
            if let Some(option) = command.data.options().find("message") {
                if let ResolvedValue::String(str) = option {
                    command.channel_id.say(&ctx.http, str).await.on_fail("Failed to send message in channel");
                    command.skip(&ctx.http).await;
                }
            } else if let Some(option) = command.data.options().find("fichier") {
                if let ResolvedValue::Attachment(attachment) = option {
                    on_fail!(command.defer(&ctx.http).await, "Failed to defer say command")?;

                    let message = on_fail!(String::from_utf8(on_fail!(attachment.download().await, "Failed to download attachment")?), "Our bytes should be valid utf8")?;
                    let message = on_fail!(json_to_message(message), "Invalid json_to_message")?;
                    for message in message {
                        command.channel_id.send_message(&ctx.http, message).await.on_fail("Failed to send message in channel");
                    }
                    command.delete_response(&ctx.http).await.on_fail("Failed to delete command interaction");
                }
            } else {
                command.respond_user_error(&ctx.http, "Tu n'as pas précisé ce que je dois annoncer !").await;
            }
        }
        Ok(())
    }

    fn fetch_commands(&self, config: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("say")
            .default_member_permissions(config.at_least_member())
            .description("Ma parole sera la votre")
            .kind(CommandType::ChatInput)
            .add_option(CreateCommandOption::new(CommandOptionType::String, "message", "Que dois-je dire à votre place ?"))
            .add_option(CreateCommandOption::new(CommandOptionType::Attachment, "fichier", "Fichier json pour afficher un message formaté"))]
    }
}