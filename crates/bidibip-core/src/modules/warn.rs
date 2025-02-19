use crate::modules::{BidibipModule, OptionHelper, ResultDebug};
use serenity::all::{CommandInteraction, CommandOptionType, CommandType, Context, CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateModal, EventHandler, InputTextStyle, ResolvedValue, User};
use serenity::builder::{CreateActionRow, CreateInputText};
use tracing::error;

pub struct Warn {}

impl Warn {
    async fn warn_command(&self, ctx: Context, user: User, name: &str, command: CommandInteraction) {
        let title = match name {
            "warn" => { format!("Warn de {}", user.name) }
            "ban du vocal" => { format!("Exclusion du vocal de {}", user.name) }
            "kick" => { format!("Kick de {}", user.name) }
            "ban" => { format!("Ban de {}", user.name) }
            val => { panic!("Unhandled command {}", val) }
        };

        command.create_response(&ctx.http, CreateInteractionResponse::Modal(
            CreateModal::new("ModalId", title).components(vec![
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Paragraph, "Raison", "reason")
                        .required(true)
                        .placeholder("Ce message sera transmis à la personne concernée")),
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Paragraph, "Autres informations", "other")
                        .required(false)
                        .placeholder("Informations complémentaires pour l'historique")),
            ]))).await.on_fail("Failed to create interaction modal");
    }
}

#[serenity::async_trait]
impl BidibipModule for Warn {
    fn name(&self) -> &'static str {
        "warn"
    }

    fn fetch_commands(&self) -> Vec<(String, CreateCommand)> {
        vec![
            ("warn".to_string(), CreateCommand::new("warn").kind(CommandType::User)),
            ("ban du vocal".to_string(), CreateCommand::new("ban du vocal").kind(CommandType::User)),
            ("kick".to_string(), CreateCommand::new("kick").kind(CommandType::User)),
            ("ban".to_string(), CreateCommand::new("ban").kind(CommandType::User)),
            ("sanction".to_string(), CreateCommand::new("sanction")
                .description("Sanctionne un utilisateur")
                .add_option(CreateCommandOption::new(CommandOptionType::User, "cible", "utilisateur à sanctionner").required(true))
                .add_option(CreateCommandOption::new(CommandOptionType::String, "action", "sanction à appliquer")
                    .required(true)
                    .add_string_choice("warn", "warn")
                    .add_string_choice("ban du vocal", "ban du vocal")
                    .add_string_choice("kick", "kick")
                    .add_string_choice("ban", "ban")
                )
            ),
        ]
    }

    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) {
        let action = if name == "sanction" {
            match command.data.options().find("action") {
                None => {
                    error!("Missing action value");
                    return;
                }
                Some(option) => {
                    if let ResolvedValue::String(val) = option {
                        val.to_string()
                    } else {
                        error!("Wrong action value");
                        return;
                    }
                }
            }
        } else { name.to_string() };
        if let Some(target) = command.data.target_id {
            match target.to_user_id().to_user(&ctx.http).await {
                Ok(user) => {
                    self.warn_command(ctx, user, action.as_str(), command).await;
                }
                Err(err) => { error!("Failed to fetch user data : {err}") }
            }
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Warn {

}