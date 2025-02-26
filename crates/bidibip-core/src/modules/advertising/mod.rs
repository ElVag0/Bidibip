mod ad_config;

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ActionRowComponent, ButtonKind, ButtonStyle, ChannelId, ChannelType, CommandInteraction, ComponentInteractionDataKind, Context, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateThread, EditMessage, GuildChannel, Http, Interaction, Mentionable, Message, User, UserId};
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
use crate::modules::advertising::ad_config::{AdDescription, Contact, Contract, FixedTermInfos, FreelanceInfos, InternshipInfos, Location, OpenEndedInfos, WorkStudyInfos};

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

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
enum Step {
    None,
    What,
    Kind,
    Location,
    Title,
    Description,
    Duration,
    Responsibilities,
    Qualifications,
    Contact,
    ContactOther,
    Urls,
    Done,
}

#[derive(Serialize, Deserialize, Clone)]
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
    async fn start_procedure(&self, ctx: &Context, thread: GuildChannel, user: &User, config: &mut InProgressAdData) -> Result<(), BidibipError> {
        on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content(format!("# Bienvenue dans le formulaire de création d'annonce {} !", user.name))).await, "Failed to send welcome message")?;
        self.advance(ctx, thread, config, user).await?;
        Ok(())
    }

    async fn advance(&self, ctx: &Context, thread: GuildChannel, config: &mut InProgressAdData, user: &User) -> Result<(), BidibipError> {
        match &config.description.is_searching {
            None => {
                if config.step == Step::What { return Ok(()); }
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("▶  Que cherches tu ?")
                .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(make_custom_id::<Advertising>("looking_job", thread.id)).label("👩‍🔧 Je cherche du travail"),
                        CreateButton::new(make_custom_id::<Advertising>("looking_employee", thread.id)).label("🕵️‍♀️ Je recrute"),
                    ])])).await, "Failed to send message")?;
                config.step = Step::What;

                return Ok(());
            }
            Some(_) => {}
        };

        match &config.description.kind {
            None => {
                if config.step == Step::Kind { return Ok(()); }
                on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("## ▶  Quel type de contrat recherches-tu ?")
                .components(vec![
                        CreateActionRow::Buttons(vec![
                            CreateButton::new(make_custom_id::<Advertising>("job_freelance", thread.id)).label("🧐 Freelance"),
                            CreateButton::new(make_custom_id::<Advertising>("job_volunteering", thread.id)).label("🤝 Bénévolat (non rémunéré)"),
                            CreateButton::new(make_custom_id::<Advertising>("job_internship", thread.id)).label("🪂 Stage")
                        ]),
                        CreateActionRow::Buttons(vec![
                            CreateButton::new(make_custom_id::<Advertising>("job_workstudy", thread.id)).label("🤓 Alternance (rémunéré)"),
                            CreateButton::new(make_custom_id::<Advertising>("job_fixedterm", thread.id)).label("😎 CDD (rémunéré)"),
                            CreateButton::new(make_custom_id::<Advertising>("job_openended", thread.id)).label("🤯 CDI (rémunéré)"),
                        ])
                    ])).await, "Failed to send message")?;
                config.step = Step::Kind;
                return Ok(());
            }
            Some(_) => {}
        };

        match &config.description.location {
            None => {
                if config.step == Step::Location { return Ok(()); }
                on_fail!(thread.send_message( & ctx.http, CreateMessage::new().content("## ▶  Souhaites-tu travailler à distance ou en présentiel ?")
                    .components(vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(make_custom_id::< Advertising > ("location_remote", thread.id)).label("🌍 Distanciel"),
                        CreateButton::new(make_custom_id::< Advertising > ("location_flex", thread.id)).label("🤷‍♀️ Télétravail possible"),
                        CreateButton::new(make_custom_id::< Advertising >("location_onsite", thread.id)).label("🏣 Présentiel uniquement"),
                    ])])).await, "Failed to send message")?;
                config.step = Step::Location;
                return Ok(());
            }
            Some(_) => {}
        };

        match &config.description.title {
            None => {
                on_fail!(thread.send_message( & ctx.http, CreateMessage::new()
                    .content("## ▶  Donne un titre à ton annonce\n> *Écris ta réponse sous ce message*")
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(make_custom_id::<Advertising>("edit_title", thread.id)).style(ButtonStyle::Secondary).label("modifier")])])
                ).await, "Failed to send message")?;
                config.step = Step::Title;
                return Ok(());
            }
            Some(_) => {}
        };

        match &config.description.description {
            None => {
                if config.step == Step::Description { return Ok(()); }
                on_fail!(thread.send_message( & ctx.http, CreateMessage::new()
                    .content("## ▶  Écrit une description pour cette offre")
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(make_custom_id::<Advertising>("edit_description", thread.id)).style(ButtonStyle::Secondary).label("modifier")])])).await, "Failed to send message")?;
                config.step = Step::Description;
                return Ok(());
            }
            Some(_) => {}
        };

        match &config.description.responsibilities {
            None => {
                if config.step == Step::Responsibilities { return Ok(()); }
                on_fail!(thread.send_message( & ctx.http, CreateMessage::new().content("## ▶  Quelles sont les responsabilitées demandées ?")
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(make_custom_id::<Advertising>("edit_responsibilities", thread.id)).style(ButtonStyle::Secondary).label("modifier")])])).await, "Failed to send message")?;
                config.step = Step::Responsibilities;
                return Ok(());
            }
            Some(_) => {}
        };

        match &config.description.qualifications {
            None => {
                if config.step == Step::Qualifications { return Ok(()); }
                on_fail!(thread.send_message( & ctx.http, CreateMessage::new().content("## ▶  Quelles sont les compétences requises ?")
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(make_custom_id::<Advertising>("edit_qualifications", thread.id)).style(ButtonStyle::Secondary).label("modifier")])])).await, "Failed to send message")?;
                config.step = Step::Qualifications;
                return Ok(());
            }
            Some(_) => {}
        };

        match &config.description.contact {
            None => {
                if config.step == Step::Contact { return Ok(()); }
                on_fail!(thread.send_message( & ctx.http, CreateMessage::new().content("## ▶  Comment peut-on te contacter ?")
                    .components(vec![
                        CreateActionRow::Buttons(vec![
                            CreateButton::new(make_custom_id::<Advertising>("contact_discord", thread.id)).label("Discord"),
                            CreateButton::new(make_custom_id::<Advertising>("contact_other", thread.id)).label("Autre"),
                        ])])).await, "Failed to send message")?;
                config.step = Step::Contact;
                return Ok(());
            }
            Some(apply_at) => {
                match apply_at {
                    Contact::Discord => {}
                    Contact::Other(location) => {
                        match location {
                            Some(_) => {}
                            None => {
                                if config.step == Step::ContactOther { return Ok(()); }
                                on_fail!(thread.send_message( & ctx.http, CreateMessage::new().content("## ▶  Indique au moins un moyen de contact (mail etc...)")).await, "Failed to send message")?;
                                config.step = Step::ContactOther;
                                return Ok(());
                            }
                        }
                    }
                }
            }
        };

        match &config.description.other_urls {
            None => {
                if config.step == Step::Urls { return Ok(()); }
                on_fail!(thread.send_message( & ctx.http, CreateMessage::new().content("## ▶  Précises d'autres liens utils")
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(make_custom_id::<Advertising>("edit_urls", thread.id)).style(ButtonStyle::Secondary).label("modifier")])])).await, "Failed to send message")?;
                config.step = Step::Urls;
                return Ok(());
            }
            Some(_) => {}
        };


        if config.step == Step::Done { return Ok(()); }
        config.step = Step::Done;
        on_fail!(thread.send_message(&ctx.http, self.create_message(&config.description, &Username::from_user(user))).await, "Failed to send final message")?;
        Ok(())
    }

    fn create_message(&self, data: &AdDescription, author: &Username) -> CreateMessage {
        let mut message = CreateMessage::new();


        let title = match &data.title {
            None => { "[Titre manquant]" }
            Some(title) => { title.as_str() }
        };

        let description = match &data.description {
            None => { "[Aucune description]" }
            Some(title) => { title.as_str() }
        };

        let mut main_embed = CreateEmbed::new()
            .title(title)
            .description(format!("Annonce de {}:\n{}", author.full(), description));

        if let Some(kind) = &data.kind {
            match kind {
                Contract::Volunteering => {}
                Contract::Internship(_) => {}
                Contract::Freelance(infos) => {
                    main_embed = main_embed.field("Durée", match &infos.duration {
                        None => { "[non spécifié]" }
                        Some(duration) => { duration.as_str() }
                    }, false);
                }
                Contract::WorkStudy(_) => {}
                Contract::FixedTerm(_) => {}
                Contract::OpenEnded(_) => {}
            }
        }

        message = message.embed(main_embed);


        message
    }


    async fn remove_in_progress_ad(&self, http: &Http, user: UserId) -> Result<bool, BidibipError> {
        let mut config = self.ad_config.write().await;
        if let Some(ad) = config.in_progress_ad.get(&user) {
            #[allow(unused)]
            ad.edition_thread.delete(http).await;
            config.in_progress_ad.remove(&user);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn update_buttons(&self, ctx: &Context, message: &mut Box<Message>, clicked_element: &str) -> Result<(), BidibipError> {
        let mut component = vec![];
        for row in &message.components {
            let mut buttons = vec![];
            for component in &row.components {
                if let ActionRowComponent::Button(button) = component {
                    if let ButtonKind::NonLink { custom_id, style: _style } = &button.data {
                        buttons.push(CreateButton::new(custom_id.clone()).label(assert_some!(button.label.clone(), "invalid label")?).style(if custom_id.contains(clicked_element) { ButtonStyle::Success } else { ButtonStyle::Secondary }));
                    }
                }
            }
            component.push(CreateActionRow::Buttons(buttons));
        }
        on_fail!(message.edit(&ctx.http, EditMessage::new().components(component)).await, "Failed to update buttons")?;
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

                let mut in_progress_data = InProgressAdData { step: Step::None, edition_thread: new_channel.id, description: AdDescription::default() };

                on_fail!(new_channel.id.add_thread_member(&ctx.http, command.user.id).await, "Failed to add member to thread")?;

                // Can fail if sending /annonce in announcement channel (the channel will be deleted)
                #[allow(unused)]
                command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true)
                    .content(format!("Bien reçu, la suite se passe ici :arrow_right: {}{}", new_channel.mention(), if removed_old { "\n> Note : ta précédente annonce en cours de création a été supprimée" } else { "" })))).await;

                if let Some(stored_add) = ad_config.stored_adds.get(&command.user.id) {
                    if stored_add.len() > 0 {
                        on_fail!(new_channel.send_message(&ctx.http, CreateMessage::new().content("Tu as déjà des annonces ouvertes")).await, "failed to send existing message 1")?;

                        for (channel, data) in stored_add {
                            let title = match &data.description.title {
                                None => { "Annonce sans titre" }
                                Some(title) => { title.as_str() }
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
                        self.start_procedure(&ctx, new_channel, &command.user, &mut in_progress_data).await?;
                    }
                } else {
                    self.start_procedure(&ctx, new_channel, &command.user, &mut in_progress_data).await?;
                }

                ad_config.in_progress_ad.insert(command.user.id, in_progress_data);

                on_fail!(self.config.save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "failed to save config")?;
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
        if let Some(ad_config) = config.await.in_progress_ad.get_mut(&message.author.id) {
            // Wrong channel
            if ad_config.edition_thread != message.channel_id { return Ok(()); }

            match ad_config.step {
                Step::Title => {
                    ad_config.description.title = Some(message.content);
                }
                Step::Description => {
                    ad_config.description.description = Some(message.content);
                }
                Step::Qualifications => {
                    ad_config.description.qualifications = Some(message.content);
                }
                Step::Responsibilities => {
                    ad_config.description.responsibilities = Some(message.content);
                }
                Step::ContactOther => {
                    ad_config.description.contact = Some(Contact::Other(Some(message.content)));
                }
                Step::Urls => {
                    ad_config.description.other_urls = Some(vec![message.content]);
                }
                _ => {
                    return Ok(())
                }
            }
            // Move to next step
            let guild_channel = assert_some!(on_fail!(ad_config.edition_thread.to_channel(&ctx.http).await, "Failed to get channel data")?.guild(), "Invalid guild thread data")?;
            self.advance(&ctx, guild_channel, ad_config, &message.author).await?;
        }

        Ok(())
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> Result<(), BidibipError> {
        if let Interaction::Component(mut component) = interaction {
            if let ComponentInteractionDataKind::Button = component.data.kind {
                if component.data.get_custom_id_data::<Advertising>("create-ad").is_some() {
                    let mut ad_config = self.ad_config.write().await;
                    if let Some(in_progress) = ad_config.in_progress_ad.get_mut(&component.user.id) {
                        if in_progress.edition_thread != component.channel_id {
                            return Ok(());
                        }
                        let channel = assert_some!(on_fail!(component.channel_id.to_channel(&ctx.http).await, "failed to get channel")?.guild(), "Failed to get guild_channel")?;
                        self.start_procedure(&ctx, channel, &component.user, in_progress).await?;
                        on_fail!(self.config.save_module_config::<Advertising, AdvertisingConfig>(&ad_config), "Failed to save ad_config")?;
                    }
                }
                if let Some((action, _)) = component.data.get_custom_id_action::<Advertising>()
                {
                    let mut ad_config = self.ad_config.write().await;
                    if let Some(in_progress) = ad_config.in_progress_ad.get_mut(&component.user.id) {
                        match action.as_str() {
                            "looking_job" => {
                                in_progress.description.is_searching = Some(true);
                            }
                            "looking_employee" => {
                                in_progress.description.is_searching = Some(false);
                            }
                            /***************************/
                            "job_freelance" => {
                                in_progress.description.kind = Some(Contract::Freelance(FreelanceInfos::default()));
                            }
                            "job_volunteering" => {
                                in_progress.description.kind = Some(Contract::Volunteering);
                            }
                            "job_internship" => {
                                in_progress.description.kind = Some(Contract::Internship(InternshipInfos::default()));
                            }
                            "job_workstudy" => {
                                in_progress.description.kind = Some(Contract::WorkStudy(WorkStudyInfos::default()));
                            }
                            "job_fixedterm" => {
                                in_progress.description.kind = Some(Contract::FixedTerm(FixedTermInfos::default()));
                            }
                            "job_openended" => {
                                in_progress.description.kind = Some(Contract::OpenEnded(OpenEndedInfos::default()));
                            }
                            /***************************/
                            "location_remote" => {
                                in_progress.description.location = Some(Location::Remote);
                            }
                            "location_flex" => {
                                in_progress.description.location = Some(Location::OnSiteFlex(None));
                            }
                            "location_onsite" => {
                                in_progress.description.location = Some(Location::OnSite(None));
                            }
                            /***************************/
                            "contact_discord" => {
                                in_progress.description.contact = Some(Contact::Discord);
                            }
                            "contact_other" => {
                                in_progress.description.contact = Some(Contact::Other(None));
                            }
                            /***************************/
                            "edit_title" => {
                                in_progress.description.title = None;
                            }
                            "edit_description" => {
                                in_progress.description.description = None;
                            }
                            "edit_responsibilities" => {
                                in_progress.description.responsibilities = None;
                            }
                            "edit_qualifications" => {
                                in_progress.description.qualifications = None;
                            }
                            "edit_urls" => {
                                in_progress.description.other_urls = None;
                            }
                            &_ => {
                                return Ok(());
                            }
                        }
                        // Update buttons
                        if !action.contains("edit") { self.update_buttons(&ctx, &mut component.message, action.as_str()).await?; }

                        on_fail!(component.defer(&ctx.http).await, "failed to defer interaction")?;

                        // Move to next step
                        let guild_channel = assert_some!(on_fail!(in_progress.edition_thread.to_channel(&ctx.http).await, "Failed to get channel data")?.guild(), "Invalid guild thread data")?;
                        self.advance(&ctx, guild_channel, in_progress, &component.user).await?;
                    }

                    // Save modifications
                    on_fail!(self.config.save_module_config::<Advertising, AdvertisingConfig>(&mut ad_config), "Failed to save ad_config")?;
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