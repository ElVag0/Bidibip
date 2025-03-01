mod ad_utils;
mod steps;

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{   ChannelId, ChannelType, CommandInteraction, ComponentInteractionDataKind, Context, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateThread,  GuildChannel, Http, Interaction, Mentionable, Message, User, UserId};
use serenity::builder::{CreateActionRow, CreateButton};
use tokio::sync::RwLock;
use crate::core::config::Config;
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::error::BidibipError;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::core::utilities::Username;
use crate::modules::{BidibipModule, LoadModule};
use crate::{assert_some, on_fail};
use crate::core::interaction_utils::{make_custom_id, InteractionUtils};
use crate::core::message_reference::MessageReference;
use crate::modules::advertising::steps::main::MainSteps;
use crate::modules::advertising::steps::SubStep;

pub struct Advertising {
    ad_config: RwLock<AdvertisingConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct StoredAdData {
    thread: ChannelId,
    ad_message: MessageReference,
    description: MainSteps,
}

#[derive(Serialize, Deserialize)]
struct AdvertisingConfig {
    in_progress_ad_channel: ChannelId,
    max_ad_per_user: u64,
    stored_adds: HashMap<UserId, HashMap<ChannelId, StoredAdData>>,
    in_progress_ad: HashMap<UserId, (ChannelId, MainSteps)>,
}

impl Default for AdvertisingConfig {
    fn default() -> Self {
        Self {
            in_progress_ad_channel: Default::default(),
            max_ad_per_user: 2,
            stored_adds: Default::default(),
            in_progress_ad: Default::default(),
        }
    }
}

impl Advertising {
    async fn start_procedure(&self, ctx: &Context, thread: GuildChannel, user: &User, config: &mut MainSteps) -> Result<(), BidibipError> {
        on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content(format!("# Bienvenue dans le formulaire de création d'annonce {} !", user.name))).await, "Failed to send welcome message")?;
        config.advance(ctx, &thread).await?;
        Ok(())
    }

    async fn remove_in_progress_ad(&self, http: &Http, user: UserId) -> Result<bool, BidibipError> {
        let mut config = self.ad_config.write().await;
        if let Some((edition_thread, _)) = config.in_progress_ad.get(&user) {
            #[allow(unused)]
            edition_thread.delete(http).await;
            config.in_progress_ad.remove(&user);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[serenity::async_trait]
impl BidibipModule for Advertising {
    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        match name {
            "annonce" => {
                let removed_old = self.remove_in_progress_ad(&ctx.http, command.user.id).await?;
                let mut ad_config = self.ad_config.write().await;

                let new_channel = on_fail!(ad_config.in_progress_ad_channel.create_thread( &ctx.http, CreateThread::new(format !("Annonce de {}", Username::from_user( & command.user).safe_full())).kind(ChannelType::PrivateThread)).await, "Failed to create thread")?;
                let new_channel_id = new_channel.id;
                let mut in_progress_data = MainSteps::default();

                on_fail!(new_channel.id.add_thread_member(&ctx.http, command.user.id).await, "Failed to add member to thread")?;

                // Can fail if sending /annonce in announcement channel (the channel will be deleted)
                #[allow(unused)]
                command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true)
                    .content(format!("Bien reçu, la suite se passe ici :arrow_right: {}{}", new_channel.mention(), if removed_old { "\n> Note : ta précédente annonce en cours de création a été supprimée" } else { "" })))).await;

                if let Some(stored_add) = ad_config.stored_adds.get(&command.user.id) {
                    if stored_add.len() > 0 {
                        on_fail!(new_channel.send_message(&ctx.http, CreateMessage::new().content("Tu as déjà des annonces ouvertes")).await, "failed to send existing message 1")?;

                        for (channel, data) in stored_add {
                            let title = match &data.description.title.value() {
                                None => { "Annonce sans titre" }
                                Some(title) => { title.as_str() }
                            };

                            on_fail!(new_channel.send_message(&ctx.http, CreateMessage::new()
                                .content(format!("**{}** : {}", title, data.ad_message.link(Config::get().server_id)))
                            .components(vec![CreateActionRow::Buttons(vec![
                                        CreateButton::new(make_custom_id::<Advertising>("edit-ad", channel)).label("Modifier"),
                                        CreateButton::new(make_custom_id::<Advertising>("delete-ad", channel)).label("Supprimer")
                                    ])])).await, "failed to send existing message 1")?;
                        }
                        if stored_add.len() as u64 >= ad_config.max_ad_per_user {
                            on_fail!(new_channel.send_message(&ctx.http, CreateMessage::new().content(":warning: Tu as atteint le nombre maximal d'annonces simultanées")).await, "failed to send existing message 1")?;
                        } else {
                            on_fail!(new_channel.send_message(&ctx.http, CreateMessage::new().content("# Crée une nouvelle annonce")
                                .components(vec![CreateActionRow::Buttons(vec![
                                        CreateButton::new(make_custom_id::<Advertising>("create-ad", new_channel.id)).label("Nouvelle annonce")
                                    ])])).await, "failed to send existing message 1")?;
                        }
                    } else {
                        self.start_procedure(&ctx, new_channel, &command.user, &mut in_progress_data).await?;
                    }
                } else {
                    self.start_procedure(&ctx, new_channel, &command.user, &mut in_progress_data).await?;
                }

                ad_config.in_progress_ad.insert(command.user.id, (new_channel_id, in_progress_data));

                on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "failed to save config")?;
            }
            _ => {}
        }
        Ok(())
    }

    fn fetch_commands(&self, _: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("annonce").description("Créer une annonce d'offre ou de recherche d'emploi")]
    }

    async fn message(&self, ctx: Context, message: Message) -> Result<(), BidibipError> {
        let config = self.ad_config.write();
        if let Some((edition_thread, ad_config)) = config.await.in_progress_ad.get_mut(&message.author.id) {
            // Wrong channel
            if *edition_thread != message.channel_id { return Ok(()); }

            let mut items : Vec<&mut dyn SubStep> = vec![ad_config];
            while let Some(item) = items.pop() {
                item.receive_message(&ctx, edition_thread, &message).await?;
                items.append(&mut item.get_dependencies());
            }

            // Move to next step
            let guild_channel = assert_some!(on_fail!(edition_thread.to_channel(&ctx.http).await, "Failed to get channel data")?.guild(), "Invalid guild thread data")?;
            ad_config.advance(&ctx, &guild_channel).await?;
        }

        Ok(())
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> Result<(), BidibipError> {
        if let Interaction::Component(component) = interaction {
            if let ComponentInteractionDataKind::Button = component.data.kind {
                if component.data.get_custom_id_data::<Advertising>("create-ad").is_some() {
                    let mut ad_config = self.ad_config.write().await;
                    if let Some((edition_thread, in_progress)) = ad_config.in_progress_ad.get_mut(&component.user.id) {
                        if *edition_thread != component.channel_id {
                            return Ok(());
                        }
                        let channel = assert_some!(on_fail!(component.channel_id.to_channel(&ctx.http).await, "failed to get channel")?.guild(), "Failed to get guild_channel")?;
                        self.start_procedure(&ctx, channel, &component.user, in_progress).await?;
                        on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "Failed to save ad_config")?;
                    }
                }
                if let Some((action, _)) = component.data.get_custom_id_action::<Advertising>()
                {
                    let mut ad_config = self.ad_config.write().await;
                    if let Some((edition_thread, in_progress)) = ad_config.in_progress_ad.get_mut(&component.user.id) {

                        on_fail!(component.defer(&ctx.http).await, "failed to defer interaction")?;

                        let mut items : Vec<&mut dyn SubStep> = vec![in_progress];
                        while let Some(item) = items.pop() {
                            item.clicked_button(&ctx, edition_thread, action.as_str()).await?;
                            items.append(&mut item.get_dependencies());
                        }

                        // Move to next step
                        let guild_channel = assert_some!(on_fail!(edition_thread.to_channel(&ctx.http).await, "Failed to get channel data")?.guild(), "Invalid guild thread data")?;
                        in_progress.advance(&ctx, &guild_channel).await?;
                    }

                    // Save modifications
                    on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&mut ad_config), "Failed to save ad_config")?;
                }
            }
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

    async fn load(_: &Arc<BidibipSharedData>) -> Result<Advertising, Error> {
        let module = Self { ad_config: Default::default() };
        let warn_config = Config::get().load_module_config::<Advertising, AdvertisingConfig>()?;
        *module.ad_config.write().await = warn_config;
        Ok(module)
    }
}