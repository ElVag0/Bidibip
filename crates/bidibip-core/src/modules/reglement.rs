use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, ComponentInteractionDataKind, Context, GetMessages, GuildId, Interaction, Message};
use crate::core::config::Config;
use crate::core::error::BidibipError;
use crate::core::json_to_message::json_to_message;
use crate::core::global_interface::BidibipSharedData;
use crate::modules::{BidibipModule, LoadModule};
use crate::{on_fail, on_fail_warn};

pub struct Reglement {
    reglement_config: ReglementConfig,
}

#[derive(Serialize, Deserialize, Default)]
struct ReglementConfig {
    reglement_channel: ChannelId,
}

#[serenity::async_trait]
impl BidibipModule for Reglement {
    async fn message(&self, ctx: Context, new_message: Message)   -> Result<(), BidibipError> {
        if new_message.channel_id == self.reglement_config.reglement_channel {
            if let Some(file) = new_message.attachments.first() {
                let data = on_fail!(String::from_utf8(on_fail!(file.download().await, "Failed to download reglement json")?), "Sent json is not a valid utf8 file")?;

                let messages = on_fail!(json_to_message(data), "Failed to convert json to message")?;
                let old_messages = on_fail!(new_message.channel_id.messages(&ctx.http, GetMessages::new().limit(100)).await, "Failed to get old messages")?;
                for message in old_messages {
                    on_fail!(message.delete(&ctx.http).await, "Failed to delete old message")?;
                }
                for message in messages {
                    on_fail!(new_message.channel_id.send_message(&ctx.http, message).await, "Failed to send new reglement message")?;
                }
            }
        }
        Ok(())
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction)   -> Result<(), BidibipError> {
        match interaction {
            Interaction::Component(component) => {
                if component.data.custom_id != "reglement_approval" {
                    return Ok(());
                }
                match component.data.kind {
                    ComponentInteractionDataKind::Button => {
                        let member = on_fail!(GuildId::from(Config::get().server_id).member(&ctx.http, component.user.id).await, "Failed to get member data")?;
                        on_fail!(member.add_role(&ctx.http, Config::get().roles.member).await, "Failed to give member role")?;
                        on_fail_warn!(component.defer(&ctx.http).await, "Failed to defer command interaction");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl LoadModule<Reglement> for Reglement {
    fn name() -> &'static str {
        "reglement"
    }

    fn description() -> &'static str {
        "Outil de mise à jour automatique du réglement"
    }

    async fn load(_: &Arc<BidibipSharedData>) -> Result<Reglement, Error> {
        let config = Config::get().load_module_config::<Reglement, ReglementConfig>()?;
        if config.reglement_channel == 0 {
            return Err(Error::msg("Invalid reglement channel id"));
        }
        Ok(Reglement { reglement_config: config })
    }
}