use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc};
use std::time::Duration;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, ChannelId, ChannelType, CommandInteraction, CommandOptionType, ComponentInteraction, ComponentInteractionDataKind, Context, CreateActionRow, CreateCommandOption, CreateEmbedAuthor, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EditChannel, EditInteractionResponse, EditMessage, GetMessages, GuildChannel, GuildId, Interaction, Member, Mentionable, Message, MessageId, PartialGuildChannel, ResolvedValue, UserId};
use serenity::all::colours::roles::GREEN;
use serenity::builder::{CreateButton, CreateEmbed};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{error};
use crate::core::config::Config;
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::error::BidibipError;
use crate::core::interaction_utils::{make_custom_id, InteractionUtils};
use crate::core::message_reference::MessageReference;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::core::utilities::{CommandHelper, OptionHelper, TruncateText, Username};
use crate::modules::{BidibipModule, LoadModule};
use crate::{assert_condition, assert_some, on_fail};

pub struct Repost {
    config: Arc<Config>,
    repost_config: RwLock<RepostConfig>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
struct VoteConfig {
    thread_name: String,
    source_message_url: String,
    source_thread: ChannelId,
    reposted_message: HashSet<MessageReference>,
    vote_message: MessageReference,
    yes: HashMap<UserId, Username>,
    no: HashMap<UserId, Username>,
}


#[derive(Default, Serialize, Deserialize, Clone)]
struct RepostChannelConfig {
    repost_channel: HashSet<ChannelId>,
    vote_enabled: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct RepostConfig {
    // Forum - RepostChannel
    forums: HashMap<ChannelId, RepostChannelConfig>,
    // Channel - config
    votes: HashMap<ChannelId, VoteConfig>,
}

fn find_urls(initial_text: &String) -> Vec<String> {
    let split = initial_text.split_whitespace();
    let mut attachments = vec![];
    for i in split {
        if i.contains("http") {
            attachments.push(i.to_string());
        }
    }
    attachments.reverse();
    attachments
}

fn make_repost_message(source_message: &Message, thread: &GuildChannel, forum_name: &String, author_member: &Member) -> Vec<CreateMessage> {
    let mut messages = vec![];
    let mut embeds = vec![];

    let mut urls = find_urls(&source_message.content);

    for attachment in &source_message.attachments {
        urls.push(attachment.url.clone());
    }

    let mut author = CreateEmbedAuthor::new(author_member.display_name());
    if let Some(thumbnail) = author_member.user.avatar_url() {
        author = author.icon_url(thumbnail);
    }

    let mut first_embed = CreateEmbed::new()
        .color(GREEN)
        .title(thread.name.clone())
        .description(source_message.content.truncate_text(4096));
    for (i, url) in urls.iter().enumerate() { // only for first image
        if url.matches(r#".(jpg|jpeg|png|webp|avif|gif)$"#).count() > 0 {
            first_embed = first_embed.image(url);
            urls.remove(i);
            break;
        }
    }

    for url in &urls { // only for first image
        if !url.matches(r#".(mp4|mov|avi|mkv|flv|jpg|jpeg|png|webp|avif|gif)$"#).count() > 0 {
            author = author.url(url);
            break;
        }
    }

    first_embed = first_embed.author(author);

    embeds.push(first_embed);

    messages.push(CreateMessage::new()
        .content(format!("Nouveau post dans {} : {}", forum_name, source_message.link()))
        .embeds(embeds));

    // Add remaining hyperlinks as message (one message per link)
    for url in urls {
        messages.push(CreateMessage::new().content(url));
    }

    // Add link button to last message
    if let Some(last) = messages.pop().clone() {
        messages.push(last.components(vec![CreateActionRow::Buttons(vec![CreateButton::new_link(source_message.link()).label("Viens donc voir !")])]))
    }

    messages
}


impl Repost {
    async fn update_vote_messages(&self, ctx: &Context, thread: GuildChannel, config: &RepostConfig) -> Result<(), BidibipError> {
        if let Some(config) = config.votes.get(&thread.id) {
            let cfg = config.clone();
            let http = ctx.http.clone();
            let mut thr = thread.clone();
            tokio::spawn(async move {
                let no = cfg.no.len();
                let yes = cfg.yes.len();
                let status = if yes > no { "✅" } else { "❌" };
                if let Err(err) = thr.edit(&http, EditChannel::new().name(format!("[{}{}-{}] {}", status, yes, no, cfg.thread_name))).await {
                    return error!("Failed to update thread name : {}", err);
                }
            });
            let no = config.no.len();
            let yes = config.yes.len();

            let mut vote_buttons = vec![
                CreateButton::new(make_custom_id::<Repost>("vote-yes", Some(thread.id))).style(ButtonStyle::Success).label(format!("Pour ✅ {yes}")),
                CreateButton::new(make_custom_id::<Repost>("vote-no", Some(thread.id))).style(ButtonStyle::Danger).label(format!("Contre ❌ {no}")),
                CreateButton::new(make_custom_id::<Repost>("see-votes", Some(thread.id))).style(ButtonStyle::Secondary).label("Voir les votes".to_string()),
            ];

            let mut message = config.vote_message.message(&ctx.http).await?;
            message.edit(&ctx.http, EditMessage::new().components(vec![CreateActionRow::Buttons(vote_buttons.clone())])).await?;

            vote_buttons.insert(0, CreateButton::new_link(config.source_message_url.clone()).label("Viens donc voir !"));

            for reposted in &config.reposted_message {
                let mut message = reposted.message(&ctx.http).await?;
                message.edit(&ctx.http, EditMessage::new().components(vec![CreateActionRow::Buttons(vote_buttons.clone())])).await?;
            }
        }

        Ok(())
    }

    async fn save_config(&self, config: &RepostConfig) -> Result<(), Error> {
        if let Err(err) = self.config.save_module_config::<Repost, RepostConfig>(&config) {
            Err(Error::msg(format!("Failed to save repost config : {}", err)))
        } else {
            Ok(())
        }
    }

    async fn user_vote(&self, ctx: Context, channel: ChannelId, component: ComponentInteraction, is_yes: bool) -> Result<(), BidibipError> {
        let mut config = self.repost_config.write().await;
        if let Some(vote_config) = config.votes.get_mut(&channel) {
            if is_yes {
                vote_config.no.remove(&component.user.id);
                if vote_config.yes.contains_key(&component.user.id) {
                    vote_config.yes.remove(&component.user.id);
                } else {
                    vote_config.yes.insert(component.user.id, Username::from_user(&component.user));
                }

                let channel = on_fail!(vote_config.source_thread.to_channel(&ctx.http).await, "Failed to get source channel")?;
                let guild = assert_some!(channel.guild(), "Failed to get guild channel")?;
                on_fail!(self.update_vote_messages(&ctx, guild, &config).await, "Failed to update vote messages")?;
            } else {
                vote_config.yes.remove(&component.user.id);
                if vote_config.no.contains_key(&component.user.id) {
                    vote_config.no.remove(&component.user.id);
                } else {
                    vote_config.no.insert(component.user.id, Username::from_user(&component.user));
                }
                let channel = on_fail!(vote_config.source_thread.to_channel(&ctx.http).await, "Failed to get source channel")?;
                let guild = assert_some!(channel.guild(), "Failed to get guild channel")?;
                on_fail!(self.update_vote_messages(&ctx, guild, &config).await, "Failed to update vote messages")?;
            }
        }
        on_fail!(self.save_config(&config).await, "failed to save config")?;
        on_fail!(component.create_response(&ctx.http,CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new()
                                                                                .ephemeral(true)
                                                                                .content("Ton vote a bien été pris en compte !"))).await, "Failed to send interaction response")?;
        Ok(())
    }
}

#[serenity::async_trait]
impl BidibipModule for Repost {
    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        match name {
            "set-forum-link" => {
                let forum = assert_some!(command.data.options().find("forum"), "missing forum parameter")?;
                let forum = if let ResolvedValue::Channel(channel) = forum {
                    assert_condition!(channel.kind == ChannelType::Forum, "Not a forum")?;
                    channel.id
                } else {
                    error!("forum parameter is not a channel");
                    return Err(BidibipError::msg("forum parameter is not a channel"));
                };

                let forum_param = assert_some!(command.data.options().find("repost-channel"), "missing repost-channel parameter")?;
                let channel = if let ResolvedValue::Channel(channel) = forum_param {
                    assert_condition!(channel.kind == ChannelType::Text, "Not a regular channel")?;
                    channel.id
                } else {
                    error!("repost-channel parameter is not a channel");
                    return Err(BidibipError::msg("repost-channel parameter is not a channel"));
                };

                let vote = match command.data.options().find("vote") {
                    None => { false }
                    Some(vote) => {
                        if let ResolvedValue::Boolean(vote) = vote {
                            vote
                        } else {
                            error!("vote option is not a boolean");
                            return Err(BidibipError::msg("vote option is not a boolean"));
                        }
                    }
                };

                let enabled = match command.data.options().find("enabled") {
                    None => { true }
                    Some(enabled) => {
                        if let ResolvedValue::Boolean(enabled) = enabled {
                            enabled
                        } else {
                            error!("Enable option is not a boolean");
                            return Err(BidibipError::msg("Enable option is not a boolean"));
                        }
                    }
                };

                if enabled {
                    let mut repost_config = self.repost_config.write().await;
                    let data = repost_config.forums.entry(forum).or_insert(RepostChannelConfig {
                        repost_channel: HashSet::new(),
                        vote_enabled: false,
                    });
                    data.repost_channel.insert(channel);
                    data.vote_enabled = vote;
                    on_fail!(self.save_config(&repost_config).await, "Failed to save config")?;
                    on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content(format!("Forum {} connecté du channel {} !", forum.mention(), channel.mention())))).await, "Failed to send confirmation message")?;
                } else {
                    let mut repost_config = self.repost_config.write().await;
                    repost_config.forums.remove(&forum);
                    on_fail!(self.save_config(&repost_config).await, "Failed to save config")?;
                    on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content(format!("Forum {} déconnecté au channel {} !", forum.mention(), channel.mention())))).await, "Failed to send confirmation message")?;
                }
            }
            "reposte" => {
                on_fail!(command.defer_ephemeral(&ctx.http).await, "Failed to defer command interaction")?;
                let thread = assert_some!(on_fail!(command.channel_id.to_channel(&ctx.http).await, "Failed to get thread")?.guild(), "Failed to get guild thread")?;

                let message = &assert_some!(command.data.options().find("message"), "Missing option 'message'")?;

                let reposted_message = if let ResolvedValue::String(message) = message {
                    let id = match message.split("/").last() {
                        None => { message }
                        Some(last) => { last }
                    };
                    match u64::from_str(id) {
                        Ok(id) => {
                            match thread.message(&ctx.http, MessageId::from(id)).await {
                                Ok(message) => { message }
                                Err(err) => {
                                    command.respond_user_error(&ctx.http, format!("Le message fourni n'est pas valid : {}", err)).await;
                                    return Ok(());
                                }
                            }
                        }
                        Err(_) => {
                            command.respond_user_error(&ctx.http, "L'option message doit être un identifiant de message ou le lien vers le message").await;
                            return Ok(());
                        }
                    }
                } else {
                    error!("Not a stringValue");
                    return Err(BidibipError::msg("Not a stringValue"));
                };

                let forum = match thread.parent_id {
                    None => {
                        command.respond_user_error(&ctx.http, "La commande doit être exécutée depuis un fil qui t'appartient").await;
                        return Ok(());
                    }
                    Some(forum) => {
                        assert_some!(on_fail!(forum.to_channel(&ctx.http).await, "Failed to get forum data")?.guild(), "Channel is not a guild channel")?
                    }
                };

                let mut config = self.repost_config.write().await;
                let repost_config = match config.forums.get(&forum.id) {
                    None => {
                        command.respond_user_error(&ctx.http, "La fonctionnalité de reposte n'est pas activée ici").await;
                        return Ok(());
                    }
                    Some(forum_config) => { forum_config.clone() }
                };

                let member = assert_some!(&command.member, "Invalid member")?;

                for repost_channel in &repost_config.repost_channel {
                    for message in make_repost_message(&reposted_message, &thread, &forum.name, member.as_ref()) {
                        let message = on_fail!(repost_channel.send_message(&ctx.http, message).await, format!("Failed to repost message in {}", repost_channel.mention()))?;
                        if let Some(votes) = config.votes.get_mut(&thread.id) {
                            votes.reposted_message.insert(message.into());
                        }
                    }
                }
                if repost_config.vote_enabled {
                    on_fail!(self.save_config(&config).await, "Failed to save config")?;
                    on_fail!(self.update_vote_messages(&ctx, thread, &config).await, "Failed to update vote messages")?;
                }
                on_fail!(command.edit_response(&ctx.http, EditInteractionResponse::new().content("Message reposté !")).await, "Failed to send confirmation message")?;
            }
            &_ => {}
        }
        Ok(())
    }

    fn fetch_commands(&self, config: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("set-forum-link")
                 .description("Lie un forum à un channel de repost")
                 .add_option(CreateCommandOption::new(CommandOptionType::Channel, "forum", "Forum où seront suivis les nouveaux posts").required(true))
                 .add_option(CreateCommandOption::new(CommandOptionType::Channel, "repost-channel", "Canal où seront repostés les évenements du forum").required(true))
                 .add_option(CreateCommandOption::new(CommandOptionType::Boolean, "vote", "Active les fonctionnalités de vote").required(true))
                 .add_option(CreateCommandOption::new(CommandOptionType::Boolean, "enabled", "Active ou désactive le lien").required(true))
                 .default_member_permissions(config.at_least_admin()),
             CreateCommandDetailed::new("reposte")
                 .description("Promeut le message donné dans le salon de repost")
                 .add_option(CreateCommandOption::new(CommandOptionType::String, "message", "lien du message à promouvoir").required(true))
                 .default_member_permissions(config.at_least_member())
        ]
    }

    async fn channel_delete(&self, _: Context, channel: GuildChannel, _: Option<Vec<Message>>) -> Result<(), BidibipError> {
        // On delete forum
        if self.repost_config.read().await.forums.contains_key(&channel.id) {
            let mut config = self.repost_config.write().await;
            config.forums.remove(&channel.id);
            on_fail!(self.save_config(&config).await, "Failed to delete channel")?;
        }
        Ok(())
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> Result<(), BidibipError> {
        match interaction {
            Interaction::Component(component) => {
                match component.data.kind {
                    ComponentInteractionDataKind::Button => {
                        if let Some(data) = component.data.get_custom_id_data::<Repost>("vote-yes") {
                            self.user_vote(ctx, ChannelId::new(on_fail!(u64::from_str(data.as_str()), "Payload is not an id")?), component, true).await?;
                        } else if let Some(data) = component.data.get_custom_id_data::<Repost>("vote-no") {
                            self.user_vote(ctx, ChannelId::new(on_fail!(u64::from_str(data.as_str()), "Payload is not an id")?), component, false).await?;
                        } else if let Some(data) = component.data.get_custom_id_data::<Repost>("see-votes") {
                            let id = ChannelId::new(on_fail!(u64::from_str(data.as_str()), "Payload is not an id")?);

                            let mut config = self.repost_config.write().await;
                            if let Some(vote_config) = config.votes.get_mut(&id) {
                                let mut y_str = String::new();
                                let mut n_str = String::new();

                                for y in &vote_config.yes {
                                    y_str += format!("{}\n", y.1.full()).as_str();
                                }
                                for n in &vote_config.no {
                                    n_str += format!("{}\n", n.1.full()).as_str();
                                }

                                on_fail!(component.create_response(&ctx.http,
                                                         CreateInteractionResponse::Message(
                                                             CreateInteractionResponseMessage::new()
                                                                 .ephemeral(true)
                                                                 .embed(CreateEmbed::new()
                                                                     .title("Votes actuels")
                                                                     .description(format!("Nombre de votes : {}", vote_config.yes.len() + vote_config.no.len()))
                                                                     .field("Pour ✅", y_str, true)
                                                                     .field("Contre ❌", n_str, true)))).await, "Failed to send interaction response")?;
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) -> Result<(), BidibipError> {
        if thread.kind == ChannelType::PublicThread {
            if let Some(parent) = thread.parent_id {
                sleep(Duration::from_secs(1)).await;

                let mut config = self.repost_config.write().await;
                if config.votes.contains_key(&thread.id) {
                    return Ok(());
                }

                let repost_config = assert_some!(config.forums.get(&parent), "Failed to get repost config")?.clone();
                let messages = on_fail!(thread.messages(&ctx.http, GetMessages::new().limit(1)).await,"Failed to get first messages in thread")?;
                let initial_message = assert_some!(messages.first(), "Failed to get first message in thread")?;
                let thread_owner = on_fail!(GuildId::from(self.config.server_id).member(&ctx.http,assert_some!(thread.owner_id, "Failed to get owner id")?).await, "Failed to get owner member")?;
                let forum_name = on_fail!(parent.name(&ctx.http).await, "Failed to get forum name")?;

                if repost_config.vote_enabled {
                    let vote_message = on_fail!(thread.send_message(&ctx.http, CreateMessage::new().content("Vote en réagissant au post !")).await, "Failed to send vote message")?;
                    config.votes.insert(thread.id, VoteConfig {
                        thread_name: thread.name.clone(),
                        source_message_url: initial_message.link(),
                        source_thread: thread.id,
                        reposted_message: HashSet::new(),
                        vote_message: vote_message.into(),
                        yes: Default::default(),
                        no: Default::default(),
                    });
                }

                for repost_channel in repost_config.repost_channel {
                    let mut last_repost_message = None;
                    for message in make_repost_message(&initial_message, &thread, &forum_name, &thread_owner) {
                        last_repost_message = Some(on_fail!(repost_channel.send_message(&ctx.http, message).await, format!("Failed to repost message in {}", repost_channel.mention()))?);
                    }

                    if repost_config.vote_enabled {
                        let last_repost_message = assert_some!(last_repost_message, "Failed to get last repost message")?;
                        assert_some!(config.votes.get_mut(&thread.id), "Failed to register last reposted message")?.reposted_message.insert(last_repost_message.into());
                    }
                }

                if repost_config.vote_enabled {
                    on_fail!(self.save_config(&config).await, "Failed to save config")?;
                    on_fail!(self.update_vote_messages(&ctx, thread, &config).await,"Failed to update vote messages")?;
                }
            }
        }
        Ok(())
    }

    async fn thread_delete(&self, _: Context, channel: PartialGuildChannel, _: Option<GuildChannel>) -> Result<(), BidibipError> {
        // On delete thread
        if self.repost_config.read().await.votes.contains_key(&channel.id) {
            let mut config = self.repost_config.write().await;
            config.votes.remove(&channel.id);
            if let Err(err) = self.save_config(&config).await {
                error!("{}", err);
            }
        }
        Ok(())
    }
}

impl LoadModule<Repost> for Repost {
    fn name() -> &'static str {
        "repost"
    }

    fn description() -> &'static str {
        "Permet de lier un salon à un forum"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Repost, Error> {
        let welcome_config = shared_data.config.load_module_config::<Repost, RepostConfig>()?;
        Ok(Repost { config: shared_data.config.clone(), repost_config: RwLock::new(welcome_config) })
    }
}