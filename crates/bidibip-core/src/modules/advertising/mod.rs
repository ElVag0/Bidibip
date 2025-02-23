mod ad_config;

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, ChannelType, CommandInteraction, Context, CreateThread, EditThread, Message, UserId};
use tokio::sync::RwLock;
use crate::core::config::Config;
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::error::BidibipError;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::core::utilities::Username;
use crate::modules::{BidibipModule, LoadModule};
use crate::{assert_some, on_fail};
use crate::core::message_reference::MessageReference;

pub struct Advertising {
    config: Arc<Config>,
    ad_config: RwLock<AdvertisingConfig>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AdData {
    thread: ChannelId,
    ad_message: MessageReference,
}

#[derive(Serialize, Deserialize, Default)]
struct AdvertisingConfig {
    ad_creation_channel: ChannelId,
    ad_create_threads: HashMap<UserId, ChannelId>,
    ad_threads: HashMap<ChannelId, UserId>,
    ad_list: HashMap<UserId, Vec<AdData>>,
}

impl Advertising {
    async fn start_procedure(&self) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn receive_message(&self, ctx: Context, message: Message) -> Result<(), BidibipError> {
        Ok(())
    }
}

#[serenity::async_trait]
impl BidibipModule for Advertising {
    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        match name {
            "name" => {
                let mut ad_config = self.ad_config.write().await;

                let mut channel = if let Some(channel_id) = ad_config.ad_create_threads.get(&command.user.id) {
                    let mut channel = assert_some!(on_fail!(channel_id.to_channel(&ctx.http).await, "Failed to get channel")?.guild(), "Failed to get guild_channel")?;
                    on_fail!(channel.edit_thread(&ctx.http, EditThread::new().archived(false).locked(false)).await, "failed to edit thread")?;
                    Some(channel)
                } else { None };

                let channel = match channel {
                    None => {
                        let new_channel = on_fail!(ad_config.ad_creation_channel.create_thread( &ctx.http,
                            CreateThread::new(format !("Annonce de {}", Username::from_user( & command.user).safe_full())).kind(ChannelType::PrivateThread)).await, "Failed to create thread")?;
                        ad_config.ad_create_threads.insert(command.user.id, new_channel.id);
                        new_channel
                    }
                    Some(channel) => { channel }
                };
                on_fail!(channel.id.add_thread_member(&ctx.http, command.user.id).await, "Failed to add member to thread")?;

                self.start_procedure().await?;
            }
            _ => {}
        }
        Ok(())
    }

    fn fetch_commands(&self, _: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("annonce").description("Créer une annonce d'offre ou de recherche d'emploi")]
    }

    async fn message(&self, ctx: Context, message: Message) -> Result<(), BidibipError> {
        if self.ad_config.read().await.ad_threads.contains_key(&message.channel_id) {
            self.receive_message(ctx, message).await?;
        }
        Ok(())
    }
}

impl LoadModule<Advertising> for Advertising {
    fn name() -> &'static str {
        "advertising"
    }

    fn description() -> &'static str {
        "Créer une annonce d'offre ou de recherche d'emploi"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Advertising, Error> {
        let module = Self { config: shared_data.config.clone(), ad_config: Default::default() };
        let warn_config = shared_data.config.load_module_config::<Advertising, AdvertisingConfig>()?;
        *module.ad_config.write().await = warn_config;
        Ok(module)
    }
}