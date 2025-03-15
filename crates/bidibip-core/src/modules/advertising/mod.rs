mod ad_utils;
mod steps;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, ChannelType, CommandInteraction, ComponentInteractionDataKind, Context, CreateEmbed, CreateForumPost, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage, CreateThread, ForumTagId, GuildChannel, Http, Interaction, Mentionable, Message, User, UserId};
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
use crate::modules::advertising::steps::{ResetStep, SubStep};

pub struct Advertising {
    ad_config: RwLock<AdvertisingConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct StoredAdData {
    ad_message: MessageReference,
    description: MainSteps,
}


#[derive(Serialize, Deserialize, Default)]
struct AdvertisingTags {
    freelance: ForumTagId,
    volunteer: ForumTagId,
    paid: ForumTagId,
    unpaid: ForumTagId,
    internship: ForumTagId,
    fixed_term: ForumTagId,
    open_ended: ForumTagId,
    work_study: ForumTagId,
    worker: ForumTagId,
    recruiter: ForumTagId,
    remote: ForumTagId,
    on_site: ForumTagId,
    on_site_flex: ForumTagId,
}

#[derive(Serialize, Deserialize)]
struct AdvertisingConfig {
    tags: AdvertisingTags,
    ad_forum: ChannelId,
    in_progress_ad_channel: ChannelId,
    max_ad_per_user: u64,
    stored_adds: HashMap<UserId, HashMap<ChannelId, StoredAdData>>,
    in_progress_ad: HashMap<UserId, (ChannelId, MainSteps)>,
}

impl Default for AdvertisingConfig {
    fn default() -> Self {
        Self {
            tags: Default::default(),
            ad_forum: Default::default(),
            in_progress_ad_channel: Default::default(),
            max_ad_per_user: 2,
            stored_adds: Default::default(),
            in_progress_ad: Default::default(),
        }
    }
}

impl Advertising {
    async fn remove_in_progress_ad(&self, http: &Http, config: &mut AdvertisingConfig, user: UserId) -> Result<bool, BidibipError> {
        if let Some((edition_thread, _)) = config.in_progress_ad.get(&user) {
            #[allow(unused)]
            edition_thread.delete(http).await;
            config.in_progress_ad.remove(&user);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn advance_or_print(&self, config: &mut MainSteps, ctx: &Context, thread: &GuildChannel, user: &User) -> Result<(), BidibipError> {
        if config.advance(ctx, thread).await? {
            config.send_test_message_to_channel(ctx, &thread.id, &user).await?;
        };
        Ok(())
    }

    async fn init_channel_with_data(&self, ctx: &Context, config: &mut AdvertisingConfig, user: &User, mut data: MainSteps) -> Result<CreateInteractionResponse, BidibipError> {
        let removed_old = self.remove_in_progress_ad(&ctx.http, config, user.id).await?;
        let new_channel = on_fail!(config.in_progress_ad_channel.create_thread( & ctx.http, CreateThread::new(format ! ("Annonce de {}", Username::from_user(&user).safe_full())).kind(ChannelType::PrivateThread)).await, "Failed to create thread")?;
        on_fail!(new_channel.id.add_thread_member( & ctx.http, user.id).await, "Failed to add member to thread")?;

        config.in_progress_ad.insert(user.id, (new_channel.id, data));

        on_fail!(new_channel.send_message(&ctx.http, CreateMessage::new().content(format!("# Bienvenue dans le formulaire de création d'annonce {} !", user.name))).await, "Failed to send welcome message")?;
        self.advance_or_print(&mut config.in_progress_ad.get_mut(&user.id).unwrap().1, ctx, &new_channel, user).await?;

        on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&config), "failed to save config")?;
        Ok(CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true)
            .content(format!("Bien reçu, la suite se passe ici :arrow_right: {}{}", new_channel.mention(), if removed_old { "\n> Note : ta précédente annonce en cours de création a été supprimée" } else { "" }))))
    }
}

#[serenity::async_trait]
impl BidibipModule for Advertising {
    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        match name {
            "annonce" => {
                let mut ad_config = self.ad_config.write().await;
                if let Some(stored_add) = ad_config.stored_adds.get(&command.user.id) {
                    if stored_add.len() > 0 {
                        if stored_add.len() as u64 >= ad_config.max_ad_per_user {
                            on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                                .content("# :warning: Tu as déjà des annonces ouvertes !\n> Note : Tu as atteint le nombre maximal d'annonces simultanées")
                            .ephemeral(true))).await, "Failed to send interaction response")?;
                        } else {
                            on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                                .content("# :warning: Tu as déjà des annonces ouvertes !")
                                .ephemeral(true)
                                .components(vec![CreateActionRow::Buttons(vec![
                                        CreateButton::new(make_custom_id::<Advertising>("create-ad", "")).label("Créer une nouvelle annonce")
                                    ])]))).await, "Failed to send interaction response")?;
                        }


                        for (channel, data) in stored_add {
                            let title = match &data.description.title.value() {
                                None => { "Annonce sans titre" }
                                Some(title) => { title.as_str() }
                            };

                            on_fail!(command.create_followup(&ctx.http, CreateInteractionResponseFollowup::new().content(format!("**{}** : {}", title, data.ad_message.link(Config::get().server_id)))
                                .ephemeral(true)
                            .components(vec![CreateActionRow::Buttons(vec![
                                        CreateButton::new(make_custom_id::<Advertising>("edit-ad", channel)).label("Modifier"),
                                        CreateButton::new(make_custom_id::<Advertising>("delete-ad", channel)).label("Supprimer")
                                    ])])).await, "Failed to send interaction response")?;
                        }

                        return Ok(());
                    }
                }
                #[allow(unused)]
                command.create_response(&ctx.http, on_fail!(self.init_channel_with_data(&ctx, &mut ad_config, &command.user, MainSteps::default()).await, "Failed to create form channel")?).await;
            }
            _ => {}
        }
        Ok(())
    }

    fn fetch_commands(&self, _: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("annonce").description("Créer une annonce d'offre ou de recherche d'emploi")]
    }

    async fn message(&self, ctx: Context, message: Message) -> Result<(), BidibipError> {
        let mut config = self.ad_config.write().await;
        if let Some((edition_thread, ad_config)) = config.in_progress_ad.get_mut(&message.author.id) {
            // Wrong channel
            if *edition_thread != message.channel_id { return Ok(()); }

            let mut items: Vec<&mut dyn SubStep> = vec![ad_config];
            while let Some(item) = items.pop() {
                item.receive_message(&ctx, edition_thread, &message).await?;
                items.append(&mut item.get_dependencies());
            }

            // Move to next step
            let guild_channel = assert_some!(on_fail!(edition_thread.to_channel(&ctx.http).await, "Failed to get channel data")?.guild(), "Invalid guild thread data")?;
            self.advance_or_print(ad_config, &ctx, &guild_channel, &message.author).await?;
            // Save modifications
            on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&config), "failed to save config")?;
        }

        Ok(())
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> Result<(), BidibipError> {
        if let Interaction::Component(component) = interaction {
            if let ComponentInteractionDataKind::Button = component.data.kind {

                // Create new ad
                if component.data.get_custom_id_data::<Advertising>("create-ad").is_some() {
                    let mut ad_config = self.ad_config.write().await;
                    #[allow(unused)]
                    component.create_response(&ctx.http, self.init_channel_with_data(&ctx, &mut ad_config, &component.user, MainSteps::default()).await?).await;
                }

                // Edit existing ad
                if let Some(id) = component.data.get_custom_id_data::<Advertising>("edit-ad") {
                    let mut ad_config = self.ad_config.write().await;

                    if let Some(user_data) = ad_config.stored_adds.get_mut(&component.user.id) {
                        let channel = ChannelId::from(u64::from_str(id.as_str())?);

                        if let Some(existing_data) = user_data.get(&channel) {

                            let mut data = existing_data.description.clone();
                            data.edited_post = Some((component.user.id.clone(), channel.clone()));

                            #[allow(unused)]
                            component.create_response(&ctx.http, self.init_channel_with_data(&ctx, &mut ad_config, &component.user, data).await?).await;
                        }
                    }
                }

                // Delete existing ad
                if let Some(id) = component.data.get_custom_id_data::<Advertising>("delete-ad") {
                    let mut ad_config = self.ad_config.write().await;
                    let is_empty = if let Some(user_data) = ad_config.stored_adds.get_mut(&component.user.id) {
                        let channel = ChannelId::from(u64::from_str(id.as_str())?);

                        if user_data.contains_key(&channel) {
                            user_data.remove(&channel);
                            on_fail!(channel.delete(&ctx.http).await, "Failed to delete old ad channel")?;

                            #[allow(unused)]
                            component.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                                .ephemeral(true)
                                .content("Ton annonce a bien été supprimée !"))).await;
                        }

                        user_data.is_empty()
                    } else { false };
                    if is_empty {
                        ad_config.stored_adds.remove(&component.user.id);
                    }
                    on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "Failed to save ad_config")?;
                }

                // Publish button
                if component.data.get_custom_id_data::<Advertising>("publish").is_some() {
                    let mut ad_config = self.ad_config.write().await;

                    let mut data = if let Some((edition_thread, in_progress)) = ad_config.in_progress_ad.get_mut(&component.user.id) {
                        if *edition_thread != component.channel_id {
                            return Ok(());
                        }
                        in_progress.clone()
                    } else {
                        return Ok(())
                    };
                    data.clean_for_storage();

                    if let Some((user, edited_post)) = &data.edited_post {
                        todo!()
                    } else {
                        let message = data.create_message(&component.user);

                        let title = assert_some!(data.title.value(), "Invalid title")?;

                        let new_post = on_fail!(ad_config.ad_forum.create_forum_post(&ctx.http, CreateForumPost::new(title, message.clone()).set_applied_tags(data.get_tags(&ad_config.tags))).await, "Failed to create forum post")?;

                        ad_config.stored_adds.entry(component.user.id).or_default().insert(new_post.id, StoredAdData {
                            ad_message: MessageReference::default(),
                            description: data,
                        });
                        ad_config.in_progress_ad.remove(&component.user.id);
                        on_fail!(component.channel_id.delete(&ctx.http).await, "Failed to delete channel")?;

                        on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "Failed to save ad_config")?;
                    }
                }

                if let Some(_) = component.data.get_custom_id_action::<Advertising>()
                {
                    let mut ad_config = self.ad_config.write().await;
                    if let Some((edition_thread, in_progress)) = ad_config.in_progress_ad.get_mut(&component.user.id) {
                        let mut items: Vec<&mut dyn SubStep> = vec![in_progress];
                        while let Some(item) = items.pop() {
                            if item.clicked_button(&ctx, &component).await? {
                                break;
                            }
                            items.append(&mut item.get_dependencies());
                        }

                        // Move to next step
                        let guild_channel = assert_some!(on_fail!(edition_thread.to_channel(&ctx.http).await, "Failed to get channel data")?.guild(), "Invalid guild thread data")?;
                        self.advance_or_print(in_progress, &ctx, &guild_channel, &component.user).await?;
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