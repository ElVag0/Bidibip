use std::ops::Deref;
use std::sync::Arc;
use anyhow::Error;
use crate::modules::{BidibipModule, CreateCommandDetailed, LoadModule};
use serenity::all::{Colour, CommandInteraction, CommandType, Context, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::builder::CreateEmbed;
use crate::core::error::BidibipError;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::{assert_some, on_fail};
use crate::core::utilities::TruncateText;

pub struct Help {
    shared_data: Arc<BidibipSharedData>,
}

impl LoadModule<Help> for Help {
    fn name() -> &'static str {
        "help"
    }

    fn description() -> &'static str {
        "Voir la liste des commandes disponibles"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Help, Error> {
        Ok(Help { shared_data: shared_data.clone() })
    }
}

#[serenity::async_trait]
impl BidibipModule for Help {
    async fn execute_command(&self, ctx: Context, _: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        let mut embed = CreateEmbed::new().title("Aide de Bidibip").description("Liste des commandes disponibles :").color(Colour::DARK_GREEN);

        for module in self.shared_data.modules.read().await.deref() {
            let permissions = self.shared_data.permissions.read().await.clone();
            for found_command in module.module.fetch_commands(&permissions) {
                let member = assert_some!(command.member.clone(), "Failed to get member data")?;
                let permissions = assert_some!(member.permissions, "Failed to get user permissions")?;

                if let Some(perms) = found_command.default_member_permissions {
                    if !permissions.contains(perms) {
                        continue;
                    }
                }
                if let Some(kind) = found_command.kind {
                    if kind == CommandType::ChatInput {
                        embed = embed.field(found_command.name.clone().truncate_text(256), found_command.description.unwrap_or_default().truncate_text(1024), false);
                    }
                } else {
                    embed = embed.field(found_command.name.clone().truncate_text(256), found_command.description.unwrap_or_default().truncate_text(1024), false);
                }
            }
        }
        on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed).ephemeral(true))).await, "Failed to print command list")?;
        Ok(())
    }

    fn fetch_commands(&self, _: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("help").description("Liste des commandes disponibles")]
    }
}