use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, ComponentInteractionDataKind, Context, EventHandler, GetMessages, GuildId, Interaction, Message};
use tracing::error;
use crate::core::config::Config;
use crate::core::json_to_message::json_to_message;
use crate::core::module::BidibipSharedData;
use crate::core::utilities::{ResultDebug};
use crate::modules::{BidibipModule, LoadModule};

pub struct Reglement {
    config: Arc<Config>,
    reglement_config: ReglementConfig,
}

#[derive(Serialize, Deserialize, Default)]
struct ReglementConfig {
    reglement_channel: ChannelId,
}

#[serenity::async_trait]
impl EventHandler for Reglement {
    async fn message(&self, ctx: Context, new_message: Message) {
        if new_message.channel_id == self.reglement_config.reglement_channel {
            if let Some(file) = new_message.attachments.first() {
                let data = match file.download().await {
                    Ok(data) => {
                        match String::from_utf8(data) {
                            Ok(data) => { data }
                            Err(err) => { return error!("Sent json is not a valid utf8 file : {}", err) }
                        }
                    }
                    Err(err) => { return error!("Failed to download reglement json : {}", err) }
                };

                let messages = match json_to_message(data) {
                    Ok(message) => { message }
                    Err(err) => { return error!("Failed to convert json to message : {}", err) }
                };
                let old_messages = match new_message.channel_id.messages(&ctx.http, GetMessages::new().limit(100)).await {
                    Ok(old_messages) => { old_messages }
                    Err(err) => { return error!("Failed to get old messages : {}", err) }
                };
                for message in old_messages {
                    if let Err(err) = message.delete(&ctx.http).await {
                        return error!("Failed to delete old message : {}", err);
                    }
                }
                for message in messages {
                    if let Err(err) = new_message.channel_id.send_message(&ctx.http, message).await {
                        return error!("Failed to send new reglement message : {}", err);
                    }
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Component(component) => {
                if component.data.custom_id != "reglement_approval" {
                    return;
                }
                match component.data.kind {
                    ComponentInteractionDataKind::Button => {
                        let member = match GuildId::from(self.config.server_id).member(&ctx.http, component.user.id).await {
                            Ok(member) => {member}
                            Err(err) => { return error!("Failed to get member data : {}", err) }
                        };
                        if let Err(err) = member.add_role(&ctx.http, self.config.roles.member).await {
                            error!("Failed to give member role : {}", err)
                        }
                        component.defer(&ctx.http).await.on_fail("Failed to defer command interaction");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl BidibipModule for Reglement {}

impl LoadModule<Reglement> for Reglement {
    fn name() -> &'static str {
        "reglement"
    }

    fn description() -> &'static str {
        "Outil de mise à jour automatique du réglement"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Reglement, Error> {
        let config = shared_data.config.load_module_config::<Reglement, ReglementConfig>()?;
        if config.reglement_channel == 0 {
            return Err(Error::msg("Invalid reglement channel id"));
        }
        Ok(Reglement { config: shared_data.config.clone(), reglement_config: config })
    }
}