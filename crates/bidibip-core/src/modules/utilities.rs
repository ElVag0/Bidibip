use std::ops::Deref;
use std::sync::Arc;
use anyhow::Error;
use serenity::all::{CommandInteraction, CommandType, Context, CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage};
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::error::BidibipError;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::modules::{BidibipModule, LoadModule};
use crate::core::utilities::CommandHelper;
use crate::on_fail;

pub struct Utilities {
    shared_data: Arc<BidibipSharedData>,
}

impl LoadModule<Utilities> for Utilities {
    fn name() -> &'static str {
        "utilities"
    }

    fn description() -> &'static str {
        "Utilitaires de modération"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Utilities, Error> {
        Ok(Utilities { shared_data: shared_data.clone() })
    }
}

#[serenity::async_trait]
impl BidibipModule for Utilities {
    async fn execute_command(&self, ctx: Context, cmd: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        if cmd == "modules" {
            let modules = self.shared_data.modules.read().await;

            let mut actions = vec![];
            for module in modules.deref() {
                actions.push(CreateActionRow::Buttons(vec![CreateButton::new("test").label(module.name.clone())]))
            }


            on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("{} modules disponibles", modules.len()))
                    .ephemeral(true)
                    .components(actions)
            )).await, "Failed to create response")?;

            command.skip(&ctx.http).await;
        }
        Ok(())
    }

    fn fetch_commands(&self, config: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("modules")
                 .description("Informations sur les modules")
                 .kind(CommandType::ChatInput)
                 .default_member_permissions(config.at_least_admin()),
             /*
             CreateCommandDetailed::new("settings")
                 .description("Panneau de configuration")
                 .kind(CommandType::ChatInput)
                 .default_member_permissions(config.at_least_admin())*/
        ]
    }
}