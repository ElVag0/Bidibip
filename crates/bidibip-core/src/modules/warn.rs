use std::collections::HashMap;
use std::sync::{Arc};
use anyhow::Error;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use crate::modules::{BidibipModule};
use serenity::all::{ActionRowComponent, AuditLogEntry, ButtonStyle, ChannelId, CommandInteraction, CommandOptionType, CommandType, ComponentInteractionDataKind, Context, CreateButton, CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateModal, CreateSelectMenu, EditInteractionResponse, EventHandler, GuildId, Http, InputTextStyle, Interaction, Member, MemberAction, Mentionable, MessageInteractionMetadata, ResolvedValue, RoleId, User, UserId};
use serenity::all::audit_log::Action;
use serenity::builder::{CreateActionRow, CreateEmbed, CreateInputText, CreateSelectMenuKind};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};
use crate::core::config::Config;
use crate::core::utilities::{ModalHelper, OptionHelper, ResultDebug, Username};

pub struct Warn {
    config: Arc<Config>,
    warn_config: RwLock<WarnConfig>,
    // Key is modal id, value is (user id, action)
    pending_warns: Mutex<HashMap<String, (User, String)>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct WarnConfig {
    warn_channel: u64,
    // Key is user id
    warns: HashMap<u64, WarnedUserList>,
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
    fn name(&self) -> &'static str {
        "warn"
    }

    fn fetch_commands(&self) -> Vec<(String, CreateCommand)> {
        vec![
            ("warn".to_string(), CreateCommand::new("warn").kind(CommandType::User)),
            ("ban du vocal".to_string(), CreateCommand::new("ban du vocal").kind(CommandType::User)),
            ("kick".to_string(), CreateCommand::new("kick").kind(CommandType::User)),
            ("ban".to_string(), CreateCommand::new("ban").kind(CommandType::User)),
            ("sanction".to_string(), CreateCommand::new("sanction")
                .description("Sanctionne un utilisateur")
                .add_option(CreateCommandOption::new(CommandOptionType::User, "cible", "utilisateur à sanctionner").required(true))
                .add_option(CreateCommandOption::new(CommandOptionType::String, "action", "sanction à appliquer")
                    .required(true)
                    .add_string_choice("warn", "warn")
                    .add_string_choice("ban du vocal", "ban du vocal")
                    .add_string_choice("kick", "kick")
                    .add_string_choice("ban", "ban")
                )
            ),
        ]
    }

    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) {
        let (action, target) =
            if name == "sanction" {
                match command.data.options().find("action") {
                    None => {
                        error!("Missing action value");
                        return;
                    }
                    Some(option) => {
                        if let ResolvedValue::String(val) = option {
                            match command.data.options().find("cible") {
                                None => {
                                    error!("Missing target value");
                                    return;
                                }
                                Some(target) => {
                                    if let ResolvedValue::User(user, _) = target {
                                        (val.to_string(), user.id)
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
                (name.to_string(), target.to_user_id())
            } else {
                error!("Invalid target");
                return;
            };


        match target.to_user(&ctx.http).await {
            Ok(user) => {
                self.open_warn_modal(ctx, user, action.as_str(), command).await;
            }
            Err(err) => { error!("Failed to fetch user data : {err}") }
        }
    }
}

impl Warn {
    pub async fn new(config: Arc<Config>) -> Result<Self, Error> {
        let module = Self { config: config.clone(), warn_config: Default::default(), pending_warns: Default::default() };
        let warn_config: WarnConfig = config.load_module_config(&module)?;
        if warn_config.warn_channel == 0 {
            return Err(Error::msg("Invalid warn channel id"));
        }
        *module.warn_config.write().await = warn_config;
        Ok(module)
    }

    // Open warn modal
    async fn open_warn_modal(&self, ctx: Context, user: User, name: &str, command: CommandInteraction) {
        let title = match name {
            "warn" => { format!("Warn de {}", user.name) }
            "ban du vocal" => { format!("Exclusion du vocal de {}", user.name) }
            "kick" => { format!("Kick de {}", user.name) }
            "ban" => { format!("Ban de {}", user.name) }
            val => { panic!("Unhandled command {}", val) }
        };

        let mut warns = self.pending_warns.lock().await;
        let mut id = 0;
        loop {
            let key = format!("WarnModalId{}", id);
            if warns.contains_key(&key) {
                id += 1;
                continue;
            }
            warns.insert(key, (user.clone(), name.to_string()));
            break;
        };

        command.create_response(&ctx.http, CreateInteractionResponse::Modal(
            CreateModal::new(format!("WarnModalId{}", id), title).components(vec![
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Short, "Raison", "reason")
                        .required(true)
                        .placeholder("Ce message sera transmis à la personne concernée")),
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Paragraph, "Autres informations", "other")
                        .required(false)
                        .placeholder("Informations complémentaires pour l'historique")),
                CreateActionRow::InputText(
                    CreateInputText::new(InputTextStyle::Short, "Url", "url")
                        .required(false)
                        .placeholder("Lien vers le message incriminant")),
            ]))).await.on_fail("Failed to create interaction modal");
    }

    async fn apply_warn(&self, http: &Http, mut warn_data: UserWarn, affect_user: bool) {
        let mut write_config = self.warn_config.write().await;

        let warn_channel = write_config.warn_channel;
        let mut warn_list = &mut write_config.warns.entry(warn_data.to.id()).or_default().warns;
        let mut embed = CreateEmbed::new()
            .title(warn_data.action.clone())
            .description(warn_data.reason.clone());

        if let Some(details) = &warn_data.details {
            embed = embed.field("Details", details, false);
        }
        if let Some(url_data) = &warn_data.link {
            embed = embed.field("Url", url_data, true);
        }
        if !warn_list.is_empty() {
            embed = embed.field("Récidive !", format!("Déjà {} warn(s)", warn_list.len()), true);
        }

        let msg = match ChannelId::new(warn_channel)
            .send_message(http,
                          CreateMessage::new()
                              .content(format!("Sanction de {} par {} {}", warn_data.to.full(), warn_data.from.full(), RoleId::from(self.config.roles.administrator).mention()))
                              .embed(embed)
                              .components(vec![
                                  CreateActionRow::Buttons(vec![
                                      CreateButton::new("warn_update_message")
                                          .label("Historique")
                                          .style(ButtonStyle::Secondary)
                                  ])
                              ]),
            ).await {
            Ok(msg) => { msg }
            Err(err) => { return error!("Failed to print warn message : {}", err) }
        };
        warn_data.full_message_link = msg.link();

        // Update database
        warn_list.push(warn_data.clone());
        self.config.save_module_config(self, &*write_config).unwrap();


        if affect_user {
            let id = UserId::from(warn_data.to.id());

            match UserId::from(warn_data.to.id()).to_user(http).await {
                Ok(user) => {
                    if let Some(member) = user.member {
                        let member = Member::from(member.as_ref().clone());


                        match warn_data.action.as_str() {
                            "ban" => { member.ban_with_reason(http, 0, warn_data.reason.as_str()).await.on_fail("Failed to ban member"); }
                            "kick" => { member.kick_with_reason(http, warn_data.reason.as_str()).await.on_fail("Failed to kick member"); }
                            "warn" => {}
                            &_ => { error!("Unhandled warn action") }
                        }
                    } else {
                        error!("Failed to get member data");
                    }
                }
                Err(err) => { error!("Failed to get user data : {err}") }
            }
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Warn {
    async fn guild_audit_log_entry_create(&self, ctx: Context, entry: AuditLogEntry, _: GuildId) {
        if let Action::Member(member) = entry.action {
            if entry.user_id != self.config.application_id {
                if let Some(target) = entry.target_id {
                    let to = match UserId::from(target.get()).to_user(&ctx.http).await {
                        Ok(user) => { user }
                        Err(err) => { return error!("Failed to get user : {err}") }
                    };

                    let from = match entry.user_id.to_user(&ctx.http).await {
                        Ok(user) => { user }
                        Err(err) => { return error!("Failed to get user : {err}") }
                    };

                    match &member {
                        MemberAction::Kick => {
                            let warn_data = UserWarn {
                                date: Utc::now().timestamp() as u64,
                                from: Username::from_user(&from),
                                to: Username::from_user(&to),
                                link: None,
                                reason: entry.reason.unwrap_or_default(),
                                details: Some(String::from("Kick manuellement")),
                                action: String::from("kick"),
                                full_message_link: "".to_string(),
                            };
                            self.apply_warn(&ctx.http, warn_data, false).await;
                        }
                        MemberAction::Prune => {
                            let warn_data = UserWarn {
                                date: Utc::now().timestamp() as u64,
                                from: Username::from_user(&from),
                                to: Username::from_user(&to),
                                link: None,
                                reason: entry.reason.unwrap_or_default(),
                                details: Some(String::from("Exclusion manuelle")),
                                action: String::from("mute"),
                                full_message_link: "".to_string(),
                            };
                            self.apply_warn(&ctx.http, warn_data, false).await;
                        }
                        MemberAction::BanAdd => {
                            let warn_data = UserWarn {
                                date: Utc::now().timestamp() as u64,
                                from: Username::from_user(&from),
                                to: Username::from_user(&to),
                                link: None,
                                reason: entry.reason.unwrap_or_default(),
                                details: Some(String::from("Ban manuel")),
                                action: String::from("ban"),
                                full_message_link: "".to_string(),
                            };
                            self.apply_warn(&ctx.http, warn_data, false).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        let warns = self.warn_config.read().await;

        if let Some(data) = warns.warns.get(&new_member.user.id.get()) {
            if !data.warns.is_empty() {
                let mut last = String::new();
                let mut last_date = 0;
                for warn in &data.warns {
                    if warn.date > last_date {
                        last_date = warn.date;
                        last = warn.full_message_link.clone();
                    }
                }

                if let Err(err) = ChannelId::new(self.config.channels.staff_channel).send_message(&ctx.http, CreateMessage::new().content(format!("{} vient de rejoindre le serveur avec {} warn(s) à son actif !", last, warns.warns.len()))).await {
                    error!("Failed to send message : {}", err)
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Modal(modal) = interaction {
            if let Some((target, action)) = self.pending_warns.lock().await.get(&modal.data.custom_id) {
                let mut reason = String::new();
                let mut details = None;
                let mut url = None;

                for component in &modal.data.components {
                    for component in &component.components {
                        if let ActionRowComponent::InputText(text) = component {
                            match text.custom_id.as_str() {
                                "reason" => { reason = text.value.clone().unwrap_or(String::new()); }
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
                    action: action.clone(),
                    full_message_link: "".to_string(),
                };

                self.apply_warn(&ctx.http, warn_data, true).await;

                modal.close(&ctx.http).await;
            }
        } else if let Interaction::Component(component) = interaction {
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