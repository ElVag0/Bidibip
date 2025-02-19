use serenity::all::{ComponentInteractionDataKind, Context, EventHandler, Interaction, Mentionable, ResolvedValue};
use tracing::{info};
use crate::core::utilities::Username;
use crate::modules::BidibipModule;

pub struct Log {}

#[serenity::async_trait]
impl EventHandler for Log {
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
    fn name(&self) -> &'static str {
        "Log"
    }
}