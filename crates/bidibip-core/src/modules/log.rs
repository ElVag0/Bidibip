use std::sync::Arc;
use anyhow::Error;
use serenity::all::{ComponentInteractionDataKind, Context, EventHandler, GuildId, Interaction, Member, Mentionable, ResolvedValue, User};
use tracing::{info};
use crate::core::module::BidibipSharedData;
use crate::core::utilities::Username;
use crate::modules::{BidibipModule, LoadModule};

pub struct Log {}

impl LoadModule<Log> for Log {
    fn name() -> &'static str {
        "log"
    }

    fn description() -> &'static str {
        "logs du serveur dans un channel dédié"
    }

    async fn load(_: &Arc<BidibipSharedData>) -> Result<Log, Error> {
        Ok(Log{})
    }
}

#[serenity::async_trait]
impl EventHandler for Log {
    async fn guild_member_addition(&self, _: Context, new_member: Member) {
        info!("{} a rejoint le serveur", Username::from_user(&new_member.user).full());
    }

    async fn guild_member_removal(&self, _: Context, _: GuildId, user: User, _: Option<Member>) {
        info!("{} a quitté le serveur", Username::from_user(&user).full());
    }

    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command_interaction) => {
                let mut options = String::new();
                for option in command_interaction.data.options() {
                    match option.value {
                        ResolvedValue::Boolean(b) => { options += format!("{} = {}, ", option.name, b).as_str(); }
                        ResolvedValue::Integer(i) => { options += format!("{} = {i}, ", option.name).as_str() }
                        ResolvedValue::Number(n) => { options += format!("{} = {n}, ", option.name).as_str() }
                        ResolvedValue::String(s) => { options += format!("{} = {s}, ", option.name).as_str() }
                        ResolvedValue::Attachment(att) => { options += format!("{} = {}, ", option.name, att.url).as_str() }
                        ResolvedValue::Channel(chan) => { options += format!("{} = {}, ", option.name, chan.name.clone().unwrap_or(String::from("Unknown"))).as_str() }
                        ResolvedValue::Role(role) => { options += format!("{} = {}, ", option.name, role.mention()).as_str() }
                        ResolvedValue::User(user, _) => { options += format!("{} = {}, ", option.name, Username::from_user(user).full()).as_str() }
                        _ => { options += format!("{} = ?, ", option.name).as_str() }
                    }
                }
                info!("User {} sent command {} with options {}", Username::from_user(&command_interaction.user).safe_full(), command_interaction.data.name, options)
            }
            Interaction::Component(component_interaction) => {
                if let ComponentInteractionDataKind::Button = component_interaction.data.kind {
                    info!("User {} clicked on button {}", Username::from_user(&component_interaction.user).safe_full(), component_interaction.data.custom_id);
                }
            }
            Interaction::Modal(modal_interaction) => {
                info!("User {} sent modal #{}", Username::from_user(&modal_interaction.user).safe_full(), modal_interaction.data.custom_id);
            }
            _ => { }
        }
    }
}

impl BidibipModule for Log {
}