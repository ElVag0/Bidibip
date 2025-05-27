mod ad_utils;
mod steps;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, ChannelId, ChannelType, CommandInteraction, ComponentInteractionDataKind, Context, CreateForumPost, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage, CreateModal, CreateThread, EditMessage, ForumTagId, GetMessages, GuildChannel, InputTextStyle, Interaction, Mentionable, Message, RoleId, User, UserId};
use serenity::all::ActionRowComponent::InputText;
use serenity::builder::{CreateActionRow, CreateButton, CreateInputText};
use tokio::sync::RwLock;
use utils::module::{BidibipModule, LoadModule};
use utils::global_interface::BidibipSharedData;
use utils::error::BidibipError;
use utils::{on_fail, on_fail_warn, assert_some, assert_warn_some};
use crate::advertising::steps::main::{Contract, MainSteps};
use crate::advertising::steps::{ResetStep, SubStep};
use utils::utilities::{TruncateText, Username};
use utils::interaction_utils::{make_custom_id, InteractionUtils};
use utils::create_command_detailed::CreateCommandDetailed;
use utils::config::Config;
use utils::message_reference::MessageReference;
use utils::global_interface::PermissionData;

pub struct Advertising {
    ad_config: RwLock<AdvertisingConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
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
    reviewer_roles: Vec<RoleId>,
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
            reviewer_roles: vec![],
            in_progress_ad_channel: Default::default(),
            max_ad_per_user: 2,
            stored_adds: Default::default(),
            in_progress_ad: Default::default(),
        }
    }
}

impl Advertising {
    /// Create an ad edition thread with input data (allow using existing data to edit it)
    async fn init_channel_with_data(&self, ctx: &Context, config: &mut AdvertisingConfig, interaction: &Interaction, data: MainSteps) -> Result<(), BidibipError> {
        // Get user who did the action
        let user = match interaction {
            Interaction::Command(cmd) => { &cmd.user }
            Interaction::Component(cmp) => { &cmp.user }
            _ => { return Err(BidibipError::msg("Unhandled")) }
        };

        // Delete existing edition thread if it already exists
        let removed_old = if let Some((edition_thread, _)) = config.in_progress_ad.get(&user.id) {
            on_fail_warn!(edition_thread.delete(&ctx.http).await, "Failed to delete thread ad edition thread");
            config.in_progress_ad.remove(&user.id);
            true
        } else {
            false
        };

        // Create edition thread
        let edition_thread = on_fail!(config.in_progress_ad_channel.create_thread( & ctx.http, CreateThread::new(format ! ("Annonce de {}", Username::from_user(&user).safe_full())).kind(ChannelType::PrivateThread)).await, "Failed to create thread")?;
        on_fail!(edition_thread.send_message(&ctx.http, CreateMessage::new().content(format!("# Bienvenue dans le formulaire de création d'annonce {} !", user.name))).await, "Failed to send welcome message")?;
        config.in_progress_ad.insert(user.id, (edition_thread.id, data));

        // Add user to edition thread
        on_fail!(edition_thread.id.add_thread_member( & ctx.http, user.id).await, "Failed to add member to thread")?;

        // Step in
        self.advance_or_print(&mut assert_some!(config.in_progress_ad.get_mut(&user.id), "Failed to get main step data")?.1, ctx, &edition_thread, user).await?;

        // Invite the user to see the edition thread
        let response = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true)
            .content(format!("Bien reçu, la suite se passe ici :arrow_right: {}{}", edition_thread.mention(), if removed_old { "\n> Note : ta précédente annonce en cours de création a été supprimée" } else { "" })));
        on_fail_warn!(match interaction {
            Interaction::Command(cmd) => { cmd.create_response(&ctx.http, response).await }
            Interaction::Component(cmp) => { cmp.create_response(&ctx.http, response).await }
            _ => { return Err(BidibipError::msg("Unhandled")) }
        }, "Failed to send invitation response for ad edition thread");

        Ok(())
    }

    /// Advance to the next step (ask next question or print preview message)
    async fn advance_or_print(&self, config: &mut MainSteps, ctx: &Context, thread: &GuildChannel, user: &User) -> Result<(), BidibipError> {
        if config.advance(ctx, thread).await? {
            if let Err(err) = config.print_preview_message_in_channel(ctx, &thread.id, &user).await {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content(format!(":no_entry:Impossible de formater l'annonce :no_entry: \n> {}", err.to_string()))).await, format!("Failed to send error reason : {}", err.to_string()))?;
            }
        };
        Ok(())
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

                            on_fail!(command.create_followup(&ctx.http, CreateInteractionResponseFollowup::new().content(format!("**{}** : {}", title.truncate_text(300), data.ad_message.link(Config::get().server_id)))
                                .ephemeral(true)
                            .components(vec![CreateActionRow::Buttons(vec![
                                        CreateButton::new(make_custom_id::<Advertising>("edit-ad", channel)).label("Modifier"),
                                        CreateButton::new(make_custom_id::<Advertising>("delete-ad", channel)).label("Supprimer")
                                    ])])).await, "Failed to send interaction response")?;
                        }
                    } else {
                        self.init_channel_with_data(&ctx, &mut ad_config, &Interaction::Command(command), MainSteps::default()).await?;
                    }
                } else {
                    self.init_channel_with_data(&ctx, &mut ad_config, &Interaction::Command(command), MainSteps::default()).await?;
                }
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
                if item.receive_message(&ctx, edition_thread, &message).await? {

                    // Move to next step
                    let guild_channel = assert_some!(on_fail!(edition_thread.to_channel(&ctx.http).await, "Failed to get channel data")?.guild(), "Invalid guild thread data")?;
                    self.advance_or_print(ad_config, &ctx, &guild_channel, &message.author).await?;
                    // Save modifications
                    on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&config), "failed to save config")?;
                    break;
                }
                items.append(&mut item.get_dependencies());
            }
        }

        Ok(())
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> Result<(), BidibipError> {
        match &interaction {
            Interaction::Component(component) => {
                if let ComponentInteractionDataKind::Button = component.data.kind {

                    // Create new ad
                    if component.data.get_custom_id_data::<Advertising>("create-ad").is_some() {
                        let mut ad_config = self.ad_config.write().await;
                        self.init_channel_with_data(&ctx, &mut ad_config, &interaction, MainSteps::default()).await?;
                    }

                    // Edit existing ad
                    else if let Some(channel) = component.data.get_custom_id_data::<Advertising>("edit-ad") {
                        let mut ad_config = self.ad_config.write().await;
                        if let Some(user_ads) = ad_config.stored_adds.get(&component.user.id) {
                            let edited_data_channel = ChannelId::new(u64::from_str(channel.as_str())?);
                            let data = if let Some(data) = user_ads.get(&edited_data_channel) {
                                Some(data.clone())
                            } else { None };
                            if let Some(mut data) = data {
                                data.description.edited_post = Some(data.ad_message.clone());
                                self.init_channel_with_data(&ctx, &mut ad_config, &interaction, data.description).await?;
                            }
                        }
                    }

                    // Delete existing ad
                    else if let Some(channel) = component.data.get_custom_id_data::<Advertising>("delete-ad") {
                        let mut ad_config = self.ad_config.write().await;
                        if let Some(user_ads) = ad_config.stored_adds.get_mut(&component.user.id) {
                            let removed_data_channel = ChannelId::new(u64::from_str(channel.as_str())?);
                            on_fail_warn!(removed_data_channel.delete(&ctx.http).await, "Failed to remove ad channel");
                            assert_warn_some!(user_ads.remove(&removed_data_channel), "Ad data was empty, nothing to remove");
                            on_fail_warn!(component.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content("Ton annonce a bien été supprimée !"))).await, "Failed to delete interaction message");
                        }
                        on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "Failed to save ad_config")?;
                    } else if component.data.get_custom_id_data::<Advertising>("pre-publish").is_some() {
                        let ad_config = self.ad_config.read().await;
                        on_fail!(component.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content("Bien reçu, nous allons passer en revue ton annonce"))).await, "Failed to send confirmation message")?;

                        let mut message = format!("{} a terminé son annonce, nous allons procéder à quelques vérifications avant de la publier.\n", component.user.mention());
                        for role in &ad_config.reviewer_roles {
                            message += role.mention().to_string().as_str();
                        }
                        on_fail!(component.channel_id.send_message(&ctx.http, CreateMessage::new().content(message)).await, "Failed to send confirmation message")?;
                        on_fail!(component.message.clone().edit(&ctx.http, EditMessage::new().components(vec![
                            CreateActionRow::Buttons(vec![CreateButton::new(make_custom_id::<Advertising>("validate", "")).label("Publier").style(ButtonStyle::Success),
                            CreateButton::new(make_custom_id::<Advertising>("deny", "")).label("Révoquer").style(ButtonStyle::Danger)]),
                        ])).await, "Failed to edit message")?;
                    } else if component.data.get_custom_id_data::<Advertising>("deny").is_some() {
                        on_fail!(
                            component.create_response(&ctx.http, CreateInteractionResponse::Modal(CreateModal::new("deny_modal", "Contenu problématique")
                            .components(vec![CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Paragraph, "Raison", component.message.id.to_string()).required(true))]))).await, "Failed to create modal")?;
                    } else if component.data.get_custom_id_data::<Advertising>("validate").is_some() {
                        let mut ad_config = self.ad_config.write().await;

                        let member = on_fail!(Config::get().server_id.member(&ctx.http, component.user.id).await, "Failed to get member data")?;

                        let mut can_review = false;
                        for role in member.roles {
                            if ad_config.reviewer_roles.contains(&role) {
                                can_review = true;
                            }
                        }
                        if !can_review {
                            on_fail!(component.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content("Tu n'as pas l'autorisation requise pour faire ceci !"))).await, "Failed to send response")?;
                            return Ok(());
                        }

                        let mut initial_user = None;
                        for data in &ad_config.in_progress_ad {
                            if data.1.0 == component.channel_id {
                                initial_user = Some(on_fail!(data.0.to_user(&ctx.http).await, "Failed to get user data")?);
                            }
                        }
                        let initial_user = assert_some!(initial_user, "Failed to get initial user")?;

                        let mut data = if let Some((edition_thread, in_progress)) = ad_config.in_progress_ad.get_mut(&initial_user.id) {
                            if *edition_thread != component.channel_id {
                                return Ok(());
                            }
                            in_progress.clone()
                        } else {
                            return Ok(())
                        };
                        let edited_post = data.edited_post.take();

                        data.clean_for_storage();

                        let post = if let Some(edited_post) = edited_post {
                            on_fail!(edited_post.channel().edit_message(&ctx.http, edited_post.id(), data.edit_message(&initial_user)).await, "Failed to edit initial ad message")?;
                            edited_post
                        } else {
                            let message = data.create_message(&initial_user);

                            let mut title = assert_some!(data.title.value(), "Invalid title")?.clone();

                            if let Some(contract) = data.kind.value() {
                                title = format!("{} {}", match contract {
                                    Contract::Volunteering(_) => { "🤝" }
                                    Contract::Internship(_) => { "🪂" }
                                    Contract::Freelance(_) => { "🧐" }
                                    Contract::WorkStudy(_) => { "🤓" }
                                    Contract::FixedTerm(_) => { "😎" }
                                    Contract::OpenEnded(_) => { "🤯" }
                                }, title.truncate_text(100));
                            }

                            let new_post = on_fail!(ad_config.ad_forum.create_forum_post(&ctx.http, CreateForumPost::new(title, message.clone()).set_applied_tags(data.get_tags(&ad_config.tags))).await, "Failed to create forum post")?;
                            let messages = on_fail!(new_post.messages(&ctx.http, GetMessages::new().limit(10)).await, "Failed to get post first messages")?;
                            MessageReference::from(assert_some!(messages.first(), "There is no message in this thread")?)
                        };

                        ad_config.stored_adds.entry(initial_user.id).or_default().insert(post.channel(), StoredAdData {
                            ad_message: post,
                            description: data,
                        });
                        ad_config.in_progress_ad.remove(&initial_user.id);
                        on_fail!(component.channel_id.delete(&ctx.http).await, "Failed to delete edition channel")?;
                        on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "Failed to save ad_config")?;
                    }
                    // Clicked on option button
                    else if let Some(_) = component.data.get_custom_id_action::<Advertising>()
                    {
                        let mut ad_config = self.ad_config.write().await;
                        if let Some((edition_thread, in_progress)) = ad_config.in_progress_ad.get_mut(&component.user.id) {
                            let mut items: Vec<&mut dyn SubStep> = vec![in_progress];
                            while let Some(item) = items.pop() {
                                if item.on_interaction(&ctx, &interaction).await? {
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
            Interaction::Modal(modal) => {
                let mut ad_config = self.ad_config.write().await;

                if modal.data.custom_id == "deny_modal" {
                    if let Some(action_row) = modal.data.components.first() {
                        if let Some(InputText(text)) = action_row.components.first() {
                            if let Some(value) = &text.value {

                                let mut demo_message =  None;

                                let mut user = None;
                                for in_progress in &mut ad_config.in_progress_ad {
                                    if in_progress.1.0 == modal.channel_id {
                                        user = Some(in_progress.0.clone());
                                        demo_message = in_progress.1.1.demo_message.take();
                                        break;
                                    }
                                }
                                let user = assert_some!(user, "Failed to find initial user")?;
                                let demo_message = assert_some!(demo_message, "Failed to find demo_message")?;
                                on_fail!(modal.channel_id.send_message(&ctx.http, CreateMessage::new().content(format!("{}, ton annonce n'a pas été validée pour la raison suivante :\n{}\n\nTu peux encore modifier ton annonce avant de la ressoumettre pour qu'elle soit conforme aux prérequis.", user.mention(), value))).await, "Failed to send reason")?;
                                let mut demo_message = on_fail!(modal.channel_id.message(&ctx.http, demo_message).await, "Failed to get demo message")?;
                                on_fail!(demo_message.edit(&ctx.http, EditMessage::new().components(vec![])).await, "Failed to remove buttons")?;
                                on_fail_warn!(modal.defer(&ctx.http).await, "Faield to defer modal");
                            }
                        }
                    }
                } else {
                    if let Some((edition_thread, in_progress)) = ad_config.in_progress_ad.get_mut(&modal.user.id) {
                        let mut items: Vec<&mut dyn SubStep> = vec![in_progress];
                        while let Some(item) = items.pop() {
                            if item.on_interaction(&ctx, &interaction).await? {
                                break;
                            }
                            items.append(&mut item.get_dependencies());
                        }

                        // Move to next step
                        let guild_channel = assert_some!(on_fail!(edition_thread.to_channel(&ctx.http).await, "Failed to get channel data")?.guild(), "Invalid guild thread data")?;
                        self.advance_or_print(in_progress, &ctx, &guild_channel, &modal.user).await?;
                    }

                    // Save modifications
                    on_fail!(Config::get().save_module_config::<Advertising, AdvertisingConfig>(&mut ad_config), "Failed to save ad_config")?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[serenity::async_trait]
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