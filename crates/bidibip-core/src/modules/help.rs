use std::ops::Deref;
use std::sync::Arc;
use anyhow::Error;
use crate::modules::{BidibipModule, CreateCommandDetailed, LoadModule};
use serenity::all::{Colour, CommandInteraction, CommandType, Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler};
use serenity::builder::CreateEmbed;
use tracing::error;
use crate::core::module::{BidibipSharedData, PermissionData};

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
impl EventHandler for Help {}

#[serenity::async_trait]
impl BidibipModule for Help {
    async fn execute_command(&self, ctx: Context, _: &str, command: CommandInteraction) {
        let mut embed = CreateEmbed::new().title("Aide de Bidibip").description("Liste des commandes disponibles :").color(Colour::DARK_GREEN);

        for module in self.shared_data.modules.read().await.deref() {
            let permissions = self.shared_data.permissions.read().await.clone();
            for found_command in module.module.fetch_commands(&permissions) {
                if let Some(member) = command.member.clone() {
                    if let Some(user_permissions) = member.permissions {
                        if let Some(perms) = found_command.default_member_permissions {
                            if !user_permissions.contains(perms) {
                                continue;
                            }
                        }


                        if let Some(kind) = found_command.kind {
                            if kind == CommandType::ChatInput {
                                embed = embed.field(found_command.name.clone(), found_command.description.unwrap_or_default(), false);
                            }
                        } else {
                            embed = embed.field(found_command.name.clone(), found_command.description.unwrap_or_default(), false);
                        }
                    } else {
                        error!("Failed to get user permissions");
                    }
                } else {
                    error!("Failed to get member data");
                }
            }
        }

        if let Err(err) = command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed).ephemeral(true))).await {
            error!("Failed to print command list : {}", err);
        }
    }

    fn fetch_commands(&self, _: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("help").description("Liste des commandes disponibles")]
    }
}