use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Add;
use std::sync::{Arc};
use anyhow::Error;
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use crate::modules::{BidibipModule, CreateCommandDetailed, LoadModule};
use serenity::all::{ActionRowComponent, AuditLogEntry, ButtonStyle, ChannelId, CommandInteraction, CommandOptionType, CommandType, Context, CreateButton, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateModal, EventHandler, GuildId, Http, InputTextStyle, Interaction, Member, MemberAction, Mentionable, Message, ResolvedValue, RoleId, Timestamp, User, UserId};
use serenity::all::audit_log::Action;
use serenity::builder::{CreateActionRow, CreateEmbed, CreateInputText};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, warn};
use crate::core::config::Config;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::core::utilities::{OptionHelper, ResultDebug, TruncateText, Username};

pub struct Warn {
    config: Arc<Config>,
    warn_config: RwLock<WarnConfig>,
    // Key is modal id, value is (user id, action)
    pending_warn_actions: Mutex<HashMap<String, (User, ActionType)>>,
}

#[derive(Clone)]
enum ActionType {
    Warn,
    BanVocal,
    ExcludeOneHour,
    ExcludeOneDay,
    ExcludeOneWeek,
    Kick,
    Ban,
}

impl Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ActionType::Warn => { "warn" }
            ActionType::BanVocal => { "exclusion du vocal" }
            ActionType::ExcludeOneHour => { "exclusion du serveur (1h)" }
            ActionType::ExcludeOneDay => { "exclusion du serveur (1 jour)" }
            ActionType::ExcludeOneWeek => { "exclusion du serveur (une semaine)" }
            ActionType::Kick => { "kick" }
            ActionType::Ban => { "ban" }
        };
        write!(f, "{}", str)
    }
}

impl LoadModule<Warn> for Warn {
    fn name() -> &'static str {
        "warn"
    }

    fn description() -> &'static str {
        "Sanctions & historique des remarques"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Warn, Error> {
        let module = Self { config: shared_data.config.clone(), warn_config: Default::default(), pending_warn_actions: Default::default() };
        let warn_config = shared_data.config.load_module_config::<Warn, WarnConfig>()?;
        if warn_config.moderation_warn_channel == 0 {
            return Err(Error::msg("Invalid warn channel id"));
        }
        if warn_config.ban_vocal == 0 {
            return Err(Error::msg("Invalid ban-vocal role id"));
        }
        *module.warn_config.write().await = warn_config;
        Ok(module)
    }
}

#[derive(Serialize, Deserialize, Default)]
struct WarnConfig {
    public_warn_channel: ChannelId,
    moderation_warn_channel: ChannelId,
    #[serde(rename = "ban-vocal")]
    ban_vocal: RoleId,
    // Key is user id
    warns: HashMap<UserId, WarnedUserList>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct WarnedUserList {
    warns: Vec<UserWarn>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserWarn {
    // The warn date
    date: u64,
    // The person who warned
    from: Username,
    // The warned person
    to: Username,
    // Optional contextual link
    link: Option<String>,
    // Warn reason
    reason: String,
    // Warn details
    details: Option<String>,
    // Action
    action: String,
    // Link to the message in the warn channel history
    full_message_link: String,
}

#[serenity::async_trait]
impl BidibipModule for Warn {
    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) {
        let (action, target) =
            if name == "sanction" {
                match command.data.options().find("action") {
                    None => {
                        error!("Missing action value");
                        return;
                    }
                    Some(action) => {
                        if let ResolvedValue::String(action) = action {
                            match command.data.options().find("cible") {
                                None => {
                                    error!("Missing target value");
                                    return;
                                }
                                Some(target) => {
                                    if let ResolvedValue::User(user, _) = target {
                                        let action = match action {
                                            "warn" => ActionType::Warn,
                                            "banvoc" => ActionType::BanVocal,
                                            "exclusion 1h" => ActionType::ExcludeOneHour,
                                            "exclusion 1 jour" => ActionType::ExcludeOneDay,
                                            "exclusion une semaine" => ActionType::ExcludeOneWeek,
                                            "kick" => ActionType::Kick,
                                            "ban" => ActionType::BanVocal,
                                            &_ => { return error!("Unhandled sanction command action") }
                                        };

                                        (action, user.id)
                                    } else {
                                        error!("Invalid user");
                                        return;
                                    }
                                }
                            }
                        } else {
                            error!("Wrong action value");
                            return;
                        }
                    }
                }
            } else if let Some(target) = command.data.target_id {
                println!("bah ? '{}'", name);

                let action = match name {
                    "warn" => ActionType::Warn,
                    "ban du vocal" => ActionType::BanVocal,
                    "exclusion 1h" => ActionType::ExcludeOneHour,
                    "exclusion 1 jour" => ActionType::ExcludeOneDay,
                    "exclusion une semaine" => ActionType::ExcludeOneWeek,
                    "kick" => ActionType::Kick,
                    "ban" => ActionType::BanVocal,
                    &_ => { return error!("Unhandled sanction command") }
                };
                (action, target.to_user_id())
            } else {
                error!("Invalid target");
                return;
            };


        match target.to_user(&ctx.http).await {
            Ok(user) => {
                self.open_warn_modal(ctx, user, action, command).await;
            }
            Err(err) => { error!("Failed to fetch user data : {err}") }
        }
    }

    fn fetch_commands(&self, config: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![
            CreateCommandDetailed::new("warn").kind(CommandType::User).default_member_permissions(config.at_least_admin()),
            CreateCommandDetailed::new("ban du vocal").kind(CommandType::User).default_member_permissions(config.at_least_helper()),
            CreateCommandDetailed::new("kick").kind(CommandType::User).default_member_permissions(config.at_least_admin()),
            CreateCommandDetailed::new("exclusion 1h").kind(CommandType::User).default_member_permissions(config.at_least_helper()),
            CreateCommandDetailed::new("ban").kind(CommandType::User).default_member_permissions(config.at_least_admin()),
            CreateCommandDetailed::new("sanction")
                .default_member_permissions(config.at_least_admin())
                .description("Sanctionne un utilisateur")
                .add_option(CreateCommandOption::new(CommandOptionType::User, "cible", "utilisateur à sanctionner").required(true))
                .add_option(CreateCommandOption::new(CommandOptionType::String, "action", "sanction à appliquer")
                    .required(true)
                    .add_string_choice("warn", "warn")
                    .add_string_choice("ban du vocal", "ban du vocal")
                    .add_string_choice("exclusion une heure", "exclusion 1h")
                    .add_string_choice("exclusion un jour", "exclusion 1 jour")
                    .add_string_choice("exclusion une semaine", "exclusion une semaine")
                    .add_string_choice("kick", "kick")
                    .add_string_choice("ban", "ban")
                ),
        ]
    }
}

impl Warn {
    /// Open the warn modal to the person who wants to warn a person
    /// user : warned user
    /// action : warn, kick, ban...
    async fn open_warn_modal(&self, ctx: Context, user: User, action: ActionType, command: CommandInteraction) {
        let mut pending_warn_actions = self.pending_warn_actions.lock().await;

        // Generate action id
        let mut id = 0;
        loop {
            let key = format!("WarnModalId{}", id);
            if pending_warn_actions.contains_key(&key) {
                id += 1;
                continue;
            }
            pending_warn_actions.insert(key, (user.clone(), action.clone()));
            break;
        };

        // Send modal widget
        command.create_response(&ctx.http, CreateInteractionResponse::Modal(
            CreateModal::new(format!("WarnModalId{}", id), format!("{} de {}", action, user.name).truncate_text(45)).components(vec![
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Short, "Raison", "reason")
                        .required(true)
                        .placeholder("Ce message sera transmis à la personne concernée")),
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Paragraph, "Autres informations", "other")
                        .required(false)
                        .placeholder("Autres informations (ne sera pas transmis)")),
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Short, "Url", "url")
                        .required(false)
                        .placeholder("Lien vers le message contextuel")),
            ]))).await.on_fail("Failed to create interaction modal");
    }

    async fn store_new_warn(&self, warn_data: UserWarn) {
        let mut warn_config = self.warn_config.write().await;
        let warn_list = &mut warn_config.warns.entry(warn_data.to.id()).or_default().warns;
        warn_list.push(warn_data.clone());
        // Update database
        self.config.save_module_config::<Self, WarnConfig>(&*warn_config).unwrap();
    }

    async fn send_moderation_warn_message(&self, http: &Http, warn_data: &UserWarn) -> Result<Message, Error> {
        let mut warn_config = self.warn_config.write().await;

        let mut embed = CreateEmbed::new()
            .title(warn_data.action.clone())
            .description(warn_data.reason.clone());
        {
            let warn_list = warn_config.warns.entry(warn_data.to.id()).or_default().warns.len();
            if let Some(details) = &warn_data.details {
                embed = embed.field("Details", details, false);
            }
            if let Some(url_data) = &warn_data.link {
                embed = embed.field("Url", url_data, true);
            }
            if warn_list > 0 {
                embed = embed.field("Encore lui !", format!("Déjà {} warn(s)", warn_list), true);
            }
        }

        Ok(warn_config.moderation_warn_channel.send_message(http,
                                                            CreateMessage::new()
                                                                .content(format!("Sanction de {} par {} {}", warn_data.to.full(), warn_data.from.full(), self.config.roles.administrator.mention()))
                                                                .embed(embed)
                                                                .components(vec![
                                                                    CreateActionRow::Buttons(vec![
                                                                        CreateButton::new("warn_update_message")
                                                                            .label("Historique")
                                                                            .style(ButtonStyle::Secondary)
                                                                    ])
                                                                ])).await?)
    }

    async fn send_warn_public_message(&self, http: &Http, warn_data: &UserWarn, action: &ActionType) -> Result<(), Error> {
        let warn_config = self.warn_config.read().await;
        match action {
            ActionType::Ban => {
                warn_config.public_warn_channel.send_message(http, CreateMessage::new().embed(CreateEmbed::new().title(format!("{} a été banni par {}", warn_data.to.safe_full(), warn_data.from.safe_full())).description(&warn_data.reason))).await?;
            }
            ActionType::Kick => {
                warn_config.public_warn_channel.send_message(http, CreateMessage::new().embed(CreateEmbed::new().title(format!("{} a été kick par {}", warn_data.to.safe_full(), warn_data.from.safe_full())).description(&warn_data.reason))).await?;
            }
            ActionType::Warn => {}
            ActionType::BanVocal => {
                warn_config.public_warn_channel.send_message(http, CreateMessage::new().embed(CreateEmbed::new().title(format!("{} a été exclu du vocal par {}", warn_data.to.safe_full(), warn_data.from.safe_full())).description(&warn_data.reason))).await?;
            }
            ActionType::ExcludeOneHour => {
                warn_config.public_warn_channel.send_message(http, CreateMessage::new().embed(CreateEmbed::new()
                    .title(format!("{} a été exclu par {}", warn_data.to.safe_full(), warn_data.from.safe_full()))
                    .description(&warn_data.reason)
                    .field("durée", "une heure", true))).await?;
            }
            ActionType::ExcludeOneDay => {
                warn_config.public_warn_channel.send_message(http, CreateMessage::new().embed(CreateEmbed::new()
                    .title(format!("{} a été exclu par {}", warn_data.to.safe_full(), warn_data.from.safe_full()))
                    .description(&warn_data.reason)
                    .field("durée", "une journée", true))).await?;
            }
            ActionType::ExcludeOneWeek => {
                warn_config.public_warn_channel.send_message(http, CreateMessage::new().embed(CreateEmbed::new()
                    .title(format!("{} a été exclu par {}", warn_data.to.safe_full(), warn_data.from.safe_full()))
                    .description(&warn_data.reason)
                    .field("durée", "une semaine", true))).await?;
            }
        }
        Ok(())
    }

    async fn send_warn_private_message(&self, http: &Http, warn_data: &UserWarn, action: &ActionType) -> Result<(), Error> {
        let server_name = match self.config.server_id.to_partial_guild(http).await {
            Ok(guild) => { guild.name }
            Err(err) => {
                error!("Failed to get server data : {}", err);
                "Unreal Engine FR".to_string()
            }
        };

        match GuildId::from(self.config.server_id).member(http, warn_data.to.id()).await {
            Ok(member) => {
                match action {
                    ActionType::Ban => {
                        member.user.direct_message(http, CreateMessage::new().content(format!("## Hello :wave:\nTu as été banni de **{server_name}** pour raison :\n\n> `{}`\n\nBonne continuation à toi ! :wave:", warn_data.reason))).await?;
                    }
                    ActionType::Kick => {
                        member.user.direct_message(http, CreateMessage::new().content(format!("## Hello :wave:\nTu as été exclu de **{server_name}** pour raison :\n\n> `{}`\n\nNous tolérerons ton retour à la seule condition que tu sois en mesure de respecter notre communauté. :point_up:\nBien à toi.", warn_data.reason))).await?;
                    }
                    ActionType::Warn => {
                        member.user.direct_message(http, CreateMessage::new().content(format!("## Hello :wave:\nJe suis le robot de **{server_name}**.\nJe tiens à te rappeler que certains comportements ne sont pas tolérés sur notre communauté, à savoir :\n\n> `{}`\n\nMerci de prendre cet avertissement en considération. :point_up:", warn_data.reason))).await?;
                    }
                    ActionType::BanVocal => {
                        member.user.direct_message(http, CreateMessage::new().content(format!("## Hello :wave:\nTu as été banni des salons vocaux de **{server_name}**.\nJe tiens à te rappeler que certains comportements ne sont pas tolérés sur notre communauté, à savoir :\n\n> `{}`\n\nMerci de prendre cet avertissement en considération. :point_up:", warn_data.reason))).await?;
                    }
                    ActionType::ExcludeOneHour => {
                        member.user.direct_message(http, CreateMessage::new().content(format!("## Hello :wave:\nTu as été exclu de **{server_name}** pour une heure.\nJe tiens à te rappeler que certains comportements ne sont pas tolérés sur notre communauté, à savoir :\n\n> `{}`\n\nMerci de prendre cet avertissement en considération. :point_up:", warn_data.reason))).await?;
                    }
                    ActionType::ExcludeOneDay => {
                        member.user.direct_message(http, CreateMessage::new().content(format!("## Hello :wave:\nTu as été exclu de **{server_name}** pour un jour.\nJe tiens à te rappeler que certains comportements ne sont pas tolérés sur notre communauté, à savoir :\n\n> `{}`\n\nMerci de prendre cet avertissement en considération. :point_up:", warn_data.reason))).await?;
                    }
                    ActionType::ExcludeOneWeek => {
                        member.user.direct_message(http, CreateMessage::new().content(format!("## Hello :wave:\nTu as été exclu de **{server_name}** pour une semaine.\nJe tiens à te rappeler que certains comportements ne sont pas tolérés sur notre communauté, à savoir :\n\n> `{}`\n\nMerci de prendre cet avertissement en considération. :point_up:", warn_data.reason))).await?;
                    }
                }
            }
            Err(err) => {
                error!("Failed to get user data : {err}")
            }
        }
        Ok(())
    }

    /// Actually kick or ban the person
    async fn apply_warn(&self, http: &Http, warn_data: &UserWarn, action: &ActionType) -> Result<(), Error> {
        let mut member = GuildId::from(self.config.server_id).member(http, warn_data.to.id()).await?;
        match action {
            ActionType::Ban => {
                member.ban_with_reason(http, 0, warn_data.reason.as_str()).await.on_fail("Failed to ban member");
            }
            ActionType::Kick => {
                if let Err(error) = member.disconnect_from_voice(http).await {
                    warn!("Failed to disconnect user from voice : {}", error);
                }
                member.kick_with_reason(http, warn_data.reason.as_str()).await.on_fail("Failed to kick member");
            }
            ActionType::BanVocal => {
                member.add_role(http, self.warn_config.read().await.ban_vocal).await?
            }
            ActionType::ExcludeOneHour => {
                member.disable_communication_until_datetime(http, Timestamp::from(Timestamp::now().add(TimeDelta::hours(1)))).await?
            }
            ActionType::ExcludeOneDay => {
                member.disable_communication_until_datetime(http, Timestamp::from(Timestamp::now().add(TimeDelta::days(1)))).await?
            }
            ActionType::ExcludeOneWeek => {
                member.disable_communication_until_datetime(http, Timestamp::from(Timestamp::now().add(TimeDelta::weeks(1)))).await?
            }
            ActionType::Warn => {}
        }
        Ok(())
    }

    /// Apply warn sanction (store / send messages / kick-ban if required)
    async fn handle_warn_action(&self, http: &Http, warn_data: UserWarn, affect_user: bool, action: ActionType) {
        // Send requests
        let mod_message = self.send_moderation_warn_message(http, &warn_data);
        let pub_message = self.send_warn_public_message(http, &warn_data, &action);
        let priv_message = self.send_warn_private_message(http, &warn_data, &action);
        let apply_warn = if affect_user { Some(self.apply_warn(http, &warn_data, &action)) } else { None };

        match mod_message.await
        {
            Ok(message) => {
                let mut data = warn_data.clone();
                data.full_message_link = message.link();
                self.store_new_warn(data).await;
            }
            Err(err) => { return error!("Failed to send warn moderation message : {}", err) }
        }

        if let Err(err) = pub_message.await
        {
            return error!("Failed to send warn public message : {}", err);
        }

        if let Err(err) = priv_message.await
        {
            return error!("Failed to send warn private message : {}", err);
        }

        if let Some(apply_warn) = apply_warn {
            if let Err(err) = apply_warn.await
            {
                return error!("Failed apply warn sentence : {}", err);
            }
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Warn {
    async fn guild_audit_log_entry_create(&self, ctx: Context, entry: AuditLogEntry, _: GuildId) {
        if let Action::Member(member_action) = entry.action {
            if entry.user_id.get() != self.config.application_id.get() {
                if let Some(target) = entry.target_id {
                    let to = match UserId::from(target.get()).to_user(&ctx.http).await {
                        Ok(user) => { user }
                        Err(err) => { return error!("Failed to get user : {err}") }
                    };

                    let from = match entry.user_id.to_user(&ctx.http).await {
                        Ok(user) => { user }
                        Err(err) => { return error!("Failed to get user : {err}") }
                    };

                    match &member_action {
                        MemberAction::Kick => {
                            let warn_data = UserWarn {
                                date: Utc::now().timestamp() as u64,
                                from: Username::from_user(&from),
                                to: Username::from_user(&to),
                                link: None,
                                reason: entry.reason.unwrap_or_default(),
                                details: Some(String::from("Kick manuel")),
                                action: ActionType::Kick.to_string(),
                                full_message_link: "".to_string(),
                            };
                            self.handle_warn_action(&ctx.http, warn_data, false, ActionType::Kick).await;
                        }
                        MemberAction::Update => {
                            let member = match GuildId::from(self.config.server_id).member(&ctx.http, to.id).await {
                                Ok(data) => { data }
                                Err(err) => { return error!("Failed to get member data : {}", err); }
                            };

                            if member.communication_disabled_until.is_some() {
                                let warn_data = UserWarn {
                                    date: Utc::now().timestamp() as u64,
                                    from: Username::from_user(&from),
                                    to: Username::from_user(&to),
                                    link: None,
                                    reason: entry.reason.unwrap_or_default(),
                                    details: Some(String::from("Exclusion manuelle")),
                                    action: ActionType::ExcludeOneHour.to_string(),
                                    full_message_link: "".to_string(),
                                };
                                self.handle_warn_action(&ctx.http, warn_data, false, ActionType::ExcludeOneHour).await;
                            }
                        }
                        MemberAction::BanAdd => {
                            let warn_data = UserWarn {
                                date: Utc::now().timestamp() as u64,
                                from: Username::from_user(&from),
                                to: Username::from_user(&to),
                                link: None,
                                reason: entry.reason.unwrap_or_default(),
                                details: Some(String::from("Ban manuel")),
                                action: ActionType::Ban.to_string(),
                                full_message_link: "".to_string(),
                            };
                            self.handle_warn_action(&ctx.http, warn_data, false, ActionType::Ban).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Detect when a warned user join the server and tell the moderation to stay vigilant
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        let warns = self.warn_config.read().await;

        if let Some(data) = warns.warns.get(&new_member.user.id) {
            if !data.warns.is_empty() {
                let mut last = String::new();
                let mut last_date = 0;
                for warn in &data.warns {
                    if warn.date > last_date {
                        last_date = warn.date;
                        last = warn.full_message_link.clone();
                    }
                }

                if let Err(err) = self.config.channels.staff_channel.send_message(&ctx.http, CreateMessage::new().content(format!("{} vient de rejoindre le serveur avec {} warn(s) à son actif ! {}", Username::from_user(&new_member.user).full(), data.warns.len(), last))).await {
                    error!("Failed to send message : {}", err)
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // When user sent a modal response
        if let Interaction::Modal(modal) = interaction {
            if let Some((target, action)) = self.pending_warn_actions.lock().await.get(&modal.data.custom_id) {
                if let Err(err) = modal.defer(&ctx.http).await {
                    return error!("Failed to close modal interaction : {}", err);
                }

                let mut reason = String::new();
                let mut details = None;
                let mut url = None;

                for component in &modal.data.components {
                    for component in &component.components {
                        if let ActionRowComponent::InputText(text) = component {
                            match text.custom_id.as_str() {
                                "reason" => {
                                    reason = text.value.clone().unwrap_or(String::new());
                                }
                                "other" => {
                                    if let Some(text) = &text.value {
                                        if !text.is_empty() { details = Some(text.clone()) }
                                    }
                                }
                                "url" => {
                                    if let Some(text) = &text.value {
                                        if !text.is_empty() { url = Some(text.clone()) }
                                    }
                                }
                                &_ => {
                                    error!("Unhandled input {}", text.custom_id);
                                }
                            }
                        } else {
                            error!("Unsupported component type");
                        }
                    }
                }

                let warn_data = UserWarn {
                    date: Utc::now().timestamp() as u64,
                    from: Username::from_user(&modal.user),
                    to: Username::from_user(target),
                    link: url,
                    reason,
                    details,
                    action: action.to_string(),
                    full_message_link: "".to_string(),
                };

                self.handle_warn_action(&ctx.http, warn_data, true, action.clone()).await;

                self.pending_warn_actions.lock().await.remove(&modal.data.custom_id);
            }
        }
        // When the user clicked on the "history" button
        else if let Interaction::Component(component) = interaction {
            if component.data.custom_id == "warn_update_message" {
                let config = self.warn_config.write().await;

                for (_, user) in &config.warns {
                    for warn in &user.warns {
                        if warn.full_message_link == component.message.link() {
                            let mut embed = CreateEmbed::new().title(format!("{} warns", user.warns.len()));
                            for warn in &user.warns {
                                let date = DateTime::from_timestamp(warn.date as i64, 0);
                                if let Some(date) = date {
                                    embed = embed.field(format!("{} ({})", warn.action.clone(), date.format("%d %B %Y")),
                                                        format!("{}\n{}", warn.reason.clone(), warn.full_message_link.clone()),
                                                        false);
                                } else {
                                    return error!("Failed to parse warn date time");
                                }
                            }
                            match component.create_response(&ctx.http, CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .ephemeral(true)
                                    .embed(embed)
                            )).await {
                                Ok(_) => {}
                                Err(err) => { return error!("Failed to send warn history : {}", err) }
                            };
                            break;
                        }
                    }
                }
            }
        }
    }
}