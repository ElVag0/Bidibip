mod ad_config;

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, ChannelType, CommandInteraction, ComponentInteractionDataKind, Context, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateThread, EditThread, GuildChannel, Http, Interaction, Mentionable, Message, User, UserId};
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
use crate::modules::advertising::ad_config::AdDescription;

pub struct Advertising {
    config: Arc<Config>,
    ad_config: RwLock<AdvertisingConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct StoredAdData {
    thread: ChannelId,
    ad_message: MessageReference,
    description: AdDescription,
}

#[derive(Serialize, Deserialize)]
enum Step {
    None,
    What,
    Kind,
    Location,
    Title,
    Description,
    Duration,
    Qualifications,
    Contact,
    Other
}

#[derive(Serialize, Deserialize)]
pub struct InProgressAdData {
    step: Step,
    edition_thread: ChannelId,
    description: AdDescription,
}

#[derive(Serialize, Deserialize)]
struct AdvertisingConfig {
    in_progress_ad_channel: ChannelId,
    max_ad_per_user: u64,
    stored_adds: HashMap<UserId, HashMap<ChannelId, StoredAdData>>,
    in_progress_ad: HashMap<UserId, InProgressAdData>,
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
    async fn start_procedure(&self, ctx: &Context, thread: GuildChannel, user: User, config: &mut InProgressAdData) -> Result<(), BidibipError> {
        on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content(format!("# Bienvenue dans le formulaire de création d'annonce {} !", user.name))).await, "Failed to send welcome message")?;
        self.advance(ctx, thread, config).await?;
        Ok(())
    }

    async fn branch_apply(&self, ctx: &Context, thread: GuildChannel, config: &InProgressAdData) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn branch_search(&self, ctx: &Context, thread: GuildChannel, config: &InProgressAdData) -> Result<(), BidibipError> {
        Ok(())
    }

    async fn advance(&self, ctx: &Context, thread: GuildChannel, config: &mut InProgressAdData) -> Result<(), BidibipError> {
        let is_searching = match &config.description.is_searching {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("▶  Que cherches tu ?")
                .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(make_custom_id::<Advertising>("looking_job", thread.id)).label("👩‍🔧 Je cherche du travail"),
                        CreateButton::new(make_custom_id::<Advertising>("looking_employee", thread.id)).label("🕵️‍♀️ Je recrute"),
                    ])])).await, "Failed to send message")?;
                config.step = Step::What;
                return Ok(())
            }
            Some(is_searching) => {is_searching}
        };

        let kind = match &config.description.kind {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Quel type de contrat recherches-tu ?")
                .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(make_custom_id::<Advertising>("job_freelance", thread.id)).label("🧐 Freelance"),
                        CreateButton::new(make_custom_id::<Advertising>("job_volunteering", thread.id)).label("🤝 Bénévolat (non rémunéré)"),
                        CreateButton::new(make_custom_id::<Advertising>("job_free_internship", thread.id)).label("🪂 Stage (non rémunéré)"),
                        CreateButton::new(make_custom_id::<Advertising>("job_paid_internship", thread.id)).label("👨‍🎓 Stage (rémunéré)"),
                        CreateButton::new(make_custom_id::<Advertising>("job_workstudy", thread.id)).label("🤓 Alternance (rémunéré)"),
                        CreateButton::new(make_custom_id::<Advertising>("job_fixedterm", thread.id)).label("😎 CDD (rémunéré)"),
                        CreateButton::new(make_custom_id::<Advertising>("job_openended", thread.id)).label("🤯 CDI (rémunéré)"),
                    ])])).await, "Failed to send message")?;
                config.step = Step::Kind;
                return Ok(())
            }
            Some(kind) => {kind}
        };

        let location = match &config.description.location {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Souhaites-tu travailler à distance ou en présentiel ?")
                  .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(make_custom_id::<Advertising>("location_remote", thread.id)).label("🌍 Distanciel"),
                        CreateButton::new(make_custom_id::<Advertising>("location_any", thread.id)).label("🤷‍♀️ Au choix"),
                        CreateButton::new(make_custom_id::<Advertising>("location_onsite", thread.id)).label("🏣 Présentiel uniquement"),
                    ])])).await, "Failed to send message")?;
                config.step = Step::Location;
                return Ok(())
            }
            Some(location) => {location}
        };

        let title = match &config.description.title {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Donne un titre à ton annonce")).await, "Failed to send message")?;
                config.step = Step::Title;
                return Ok(())
            }
            Some(title) => {title}
        };

        let description = match &config.description.description {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Écrit une description pour cette offre")).await, "Failed to send message")?;
                config.step = Step::Description;
                return Ok(())
            }
            Some(description) => {description}
        };

        let duration = match &config.description.duration {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Quelle est la durée idéale du contrat ?")).await, "Failed to send message")?;
                config.step = Step::Duration;
                return Ok(())
            }
            Some(duration) => {duration}
        };

        let responsibilities = match &config.description.responsibilities {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Quelles sont les responsabilitées demandées ?")).await, "Failed to send message")?;
                //@TODO config.step = Step::Title;
                return Ok(())
            }
            Some(responsibilities) => {responsibilities}
        };

        let qualifications = match &config.description.qualifications {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Quelles sont les compétences requises ?")).await, "Failed to send message")?;
                config.step = Step::Qualifications;
                return Ok(())
            }
            Some(qualifications) => {qualifications}
        };

        let apply_at = match &config.description.apply_at {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Comment peut-on te contacter ?")).await, "Failed to send message")?;
                config.step = Step::Contact;
                return Ok(())
            }
            Some(apply_at) => {apply_at}
        };

        let other_urls = match &config.description.other_urls {
            None => {
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Ajoutes un lien vers ton CV")).await, "Failed to send message")?;
                config.step = Step::Other;
                return Ok(())
            }
            Some(other_urls) => {other_urls}
        };

        Ok(())
    }

    async fn remove_in_progress_ad(&self, http: &Http, user: UserId) -> Result<bool, BidibipError> {
        let mut config = self.ad_config.write().await;
        if let Some(ad) = config.in_progress_ad.get(&user) {
            on_fail!(ad.edition_thread.delete(http).await, "Failed to delete old thread")?;
            config.in_progress_ad.remove(&user);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn receive_message(&self, ctx: Context, message: Message) -> Result<(), BidibipError> {

        Ok(())
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

                ad_config.in_progress_ad.insert(command.user.id, InProgressAdData { step: Step::None, edition_thread: new_channel.id, description: AdDescription::default() });

                let mut in_progress_data = ad_config.in_progress_ad.get_mut(&command.user.id).unwrap();

                on_fail!(new_channel.id.add_thread_member(&ctx.http, command.user.id).await, "Failed to add member to thread")?;

                on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true)
                    .content(format!("Bien reçu, la suite se passe ici :arrow_right: {}{}", new_channel.mention(), if removed_old {"\n> Note : ta précédente annonce en cours de création a été supprimée"} else {""})))).await, "failed to send response")?;

                if let Some(stored_add) = ad_config.stored_adds.get(&command.user.id) {
                    if stored_add.len() > 0 {
                        on_fail!(new_channel.send_message(&ctx.http, CreateMessage::new().content("Tu as déjà des annonces ouvertes")).await, "failed to send existing message 1")?;

                        for (channel, data) in stored_add {

                            let title = match &data.description.title {
                                None => {"Annonce sans titre"}
                                Some(title) => {title.as_str()}
                            };

                            on_fail!(new_channel.send_message(&ctx.http, CreateMessage::new()
                                .content(format!("**{}** : {}", title, data.ad_message.link(self.config.server_id)))
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
                        self.start_procedure(&ctx, new_channel, command.user, in_progress_data).await?;
                    }
                } else {
                    self.start_procedure(&ctx, new_channel, command.user, in_progress_data).await?;
                }

                on_fail!(self.config.save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "failed to save config");
            }
            _ => {}
        }
        Ok(())
    }

    fn fetch_commands(&self, _: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("annonce").description("Créer une annonce d'offre ou de recherche d'emploi")]
    }

    async fn message(&self, ctx: Context, message: Message) -> Result<(), BidibipError> {
        /*
        if self.ad_config.read().await.ad_threads.contains_key(&message.channel_id) {
            self.receive_message(ctx, message).await?;
        }

         */
        Ok(())
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> Result<(), BidibipError> {
        if let Interaction::Component(component) = interaction {
            if let ComponentInteractionDataKind::Button = component.data.kind  {
                if component.data.get_custom_id_data::<Advertising>("create-ad").is_some() {
                    let mut ad_config = self.ad_config.write().await;
                    if let Some(in_progress) = ad_config.in_progress_ad.get_mut(&component.user.id) {
                        if in_progress.edition_thread != component.channel_id {
                            return Ok(());
                        }
                        let channel = assert_some!(on_fail!(component.channel_id.to_channel(&ctx.http).await, "failed to get channel")?.guild(), "Failed to get guild_channel")?;
                        self.start_procedure(&ctx, channel, component.user, in_progress).await?;
                        on_fail!(self.config.save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "Failed to save ad_config")?;
                    }
                }
                if component.data.get_custom_id_data::<Advertising>("looking_job").is_some() {
                    todo!()
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

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Advertising, Error> {
        let module = Self { config: shared_data.config.clone(), ad_config: Default::default() };
        let warn_config = shared_data.config.load_module_config::<Advertising, AdvertisingConfig>()?;
        *module.ad_config.write().await = warn_config;
        Ok(module)
    }
}