use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc};
use std::time::Duration;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, ChannelId, ChannelType, CommandInteraction, CommandOptionType, ComponentInteractionDataKind, Context, CreateActionRow, CreateCommandOption, CreateEmbedAuthor, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EditChannel, EditInteractionResponse, EditMessage, EventHandler, GetMessages, GuildChannel, GuildId, Interaction, Member, Mentionable, Message, MessageId, PartialGuildChannel, ResolvedValue, UserId};
use serenity::all::colours::roles::GREEN;
use serenity::builder::{CreateButton, CreateEmbed};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{error};
use crate::core::config::Config;
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::interaction_utils::{make_custom_id, InteractionUtils};
use crate::core::message_reference::MessageReference;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::core::utilities::{CommandHelper, OptionHelper, TruncateText, Username};
use crate::modules::{BidibipModule, LoadModule};

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
    async fn update_vote_messages(&self, ctx: &Context, thread: GuildChannel, config: &RepostConfig) -> Result<(), Error> {
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
}

#[serenity::async_trait]
impl EventHandler for Repost {
    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        if thread.kind == ChannelType::PublicThread {
            if let Some(parent) = thread.parent_id {
                sleep(Duration::from_secs(1)).await;

                let mut config = self.repost_config.write().await;
                if config.votes.contains_key(&thread.id) {
                    return;
                }

                let repost_config = match config.forums.get(&parent) {
                    None => { return error!("Failed to get repost config"); }
                    Some(repost_config) => { repost_config.clone() }
                };

                let messages = match thread.messages(&ctx.http, GetMessages::new().limit(1)).await {
                    Ok(messages) => { messages }
                    Err(err) => { return error!("Failed to get first messages in thread : {}", err) }
                };

                let initial_message = match messages.first() {
                    None => { return error!("Failed to get first message in thread :") }
                    Some(initial_message) => { initial_message }
                };

                let thread_owner = match thread.owner_id {
                    None => { return error!("Failed to get owner id"); }
                    Some(user) => {
                        match GuildId::from(self.config.server_id).member(&ctx.http, user).await {
                            Ok(member) => { member }
                            Err(err) => { return error!("Failed to get owner member : {}", err); }
                        }
                    }
                };
                let forum_name = match parent.name(&ctx.http).await {
                    Ok(name) => { name }
                    Err(err) => { return error!("Failed to get forum name {}", err); }
                };


                if repost_config.vote_enabled {
                    let vote_message = match thread.send_message(&ctx.http, CreateMessage::new().content("Vote en réagissant au post !")).await {
                        Ok(message) => { message }
                        Err(err) => { return error!("Failed to send vote message, {}", err); }
                    };

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
                        match repost_channel.send_message(&ctx.http, message).await {
                            Ok(message) => { last_repost_message = Some(message) }
                            Err(err) => { return error!("Failed to repost message in {} : {}", repost_channel.mention(), err); }
                        }
                    }

                    if repost_config.vote_enabled {
                        let last_repost_message = match last_repost_message {
                            None => { return error!("Failed to get last repost message"); }
                            Some(last_repost_message) => { last_repost_message }
                        };
                        match config.votes.get_mut(&thread.id) {
                            None => { return error!("Failed to register last reposted message"); }
                            Some(thread) => { thread.reposted_message.insert(last_repost_message.into()); }
                        }
                    }
                }

                if repost_config.vote_enabled {
                    if let Err(err) = self.save_config(&config).await {
                        return error!("{}", err);
                    }
                    if let Err(err) = self.update_vote_messages(&ctx, thread, &config).await {
                        error!("Failed to update vote messages : {}", err)
                    }
                }
            }
        }
    }

    async fn channel_delete(&self, _: Context, channel: GuildChannel, _: Option<Vec<Message>>) {
        // On delete forum
        if self.repost_config.read().await.forums.contains_key(&channel.id) {
            let mut config = self.repost_config.write().await;
            config.forums.remove(&channel.id);
            if let Err(err) = self.save_config(&config).await {
                error!("{}", err)
            }
        }
    }

    async fn thread_delete(&self, _: Context, channel: PartialGuildChannel, _: Option<GuildChannel>) {
        // On delete thread
        if self.repost_config.read().await.votes.contains_key(&channel.id) {
            let mut config = self.repost_config.write().await;
            config.votes.remove(&channel.id);
            if let Err(err) = self.save_config(&config).await {
                error!("{}", err);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Component(component) => {
                match component.data.kind {
                    ComponentInteractionDataKind::Button => {
                        if let Some(data) = component.data.get_custom_id_data::<Repost>("vote-yes") {
                            let id = ChannelId::new(match u64::from_str(data.as_str()) {
                                Ok(id) => { id }
                                Err(err) => { return error!("Payload is not an id : {}", err) }
                            });

                            let mut config = self.repost_config.write().await;
                            if let Some(vote_config) = config.votes.get_mut(&id) {
                                vote_config.no.remove(&component.user.id);
                                if vote_config.yes.contains_key(&component.user.id) {
                                    vote_config.yes.remove(&component.user.id);
                                } else {
                                    vote_config.yes.insert(component.user.id, Username::from_user(&component.user));
                                }

                                match vote_config.source_thread.to_channel(&ctx.http).await {
                                    Ok(channel) => {
                                        match channel.guild() {
                                            None => { return error!("Failed to get guild channel") }
                                            Some(guild) => {
                                                if let Err(err) = self.update_vote_messages(&ctx, guild, &config).await {
                                                    return error!("Failed to update vote messages : {}", err);
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => { return error!("Failed to get source channel : {}", err) }
                                }
                            }
                            if let Err(err) = self.save_config(&config).await { return error!("{}", err); }
                            if let Err(err) = component.create_response(&ctx.http,
                                                                        CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new()
                                                                                .ephemeral(true)
                                                                                .content("Ton vote a bien été pris en compte !"))).await {
                                error!("Failed to send interaction response : {}", err)
                            }
                        } else if let Some(data) = component.data.get_custom_id_data::<Repost>("vote-no") {
                            let id = ChannelId::new(match u64::from_str(data.as_str()) {
                                Ok(id) => { id }
                                Err(err) => { return error!("Payload is not an id : {}", err) }
                            });

                            let mut config = self.repost_config.write().await;
                            if let Some(vote_config) = config.votes.get_mut(&id) {
                                vote_config.yes.remove(&component.user.id);
                                if vote_config.no.contains_key(&component.user.id) {
                                    vote_config.no.remove(&component.user.id);
                                } else {
                                    vote_config.no.insert(component.user.id, Username::from_user(&component.user));
                                }

                                match vote_config.source_thread.to_channel(&ctx.http).await {
                                    Ok(channel) => {
                                        match channel.guild() {
                                            None => { return error!("Failed to get guild channel") }
                                            Some(guild) => {
                                                if let Err(err) = self.update_vote_messages(&ctx, guild, &config).await {
                                                    return error!("Failed to update vote messages : {}", err);
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => { return error!("Failed to get source channel : {}", err) }
                                }
                            }
                            if let Err(err) = self.save_config(&config).await { return error!("{}", err); }
                            if let Err(err) = component.create_response(&ctx.http,
                                                                        CreateInteractionResponse::Message(
                                                                            CreateInteractionResponseMessage::new()
                                                                                .ephemeral(true)
                                                                                .content("Ton vote a bien été pris en compte !"))).await {
                                error!("Failed to send interaction response : {}", err)
                            }
                        } else if let Some(data) = component.data.get_custom_id_data::<Repost>("see-votes") {
                            let id = ChannelId::new(match u64::from_str(data.as_str()) {
                                Ok(id) => { id }
                                Err(err) => { return error!("Payload is not an id : {}", err) }
                            });

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

                                if let Err(err) = component
                                    .create_response(&ctx.http,
                                                     CreateInteractionResponse::Message(
                                                         CreateInteractionResponseMessage::new()
                                                             .ephemeral(true)
                                                             .embed(CreateEmbed::new()
                                                                 .title("Votes actuels")
                                                                 .description(format!("Nombre de votes : {}", vote_config.yes.len() + vote_config.no.len()))
                                                                 .field("Pour ✅", y_str, true)
                                                                 .field("Contre ❌", n_str, true)))).await {
                                    error!("Failed to send interaction response : {}", err)
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[serenity::async_trait]
impl BidibipModule for Repost {
    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) {
        match name {
            "set-forum-link" => {
                let forum = match command.data.options().find("forum") {
                    None => { return error!("missing forum parameter"); }
                    Some(forum) => {
                        if let ResolvedValue::Channel(channel) = forum {
                            if channel.kind != ChannelType::Forum {
                                return error!("Not a forum");
                            }
                            channel.id
                        } else {
                            return error!("forum parameter is not a channel");
                        }
                    }
                };

                let channel = match command.data.options().find("repost-channel") {
                    None => { return error!("missing repost-channel parameter"); }
                    Some(forum) => {
                        if let ResolvedValue::Channel(channel) = forum {
                            if channel.kind != ChannelType::Text {
                                return error!("Not a regular channel");
                            }
                            channel.id
                        } else {
                            return error!("repost-channel parameter is not a channel");
                        }
                    }
                };

                let vote = match command.data.options().find("vote") {
                    None => { false }
                    Some(vote) => {
                        if let ResolvedValue::Boolean(vote) = vote {
                            vote
                        } else {
                            return error!("vote option is not a boolean");
                        }
                    }
                };

                let enabled = match command.data.options().find("enabled") {
                    None => { true }
                    Some(enabled) => {
                        if let ResolvedValue::Boolean(enabled) = enabled {
                            enabled
                        } else {
                            return error!("Enable option is not a boolean");
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
                    if let Err(err) = self.save_config(&repost_config).await { return error!("{}", err); }
                    if let Err(err) = command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content(format!("Forum {} connecté du channel {} !", forum.mention(), channel.mention())))).await {
                        error!("Failed to send confirmation message {}", err)
                    }
                } else {
                    let mut repost_config = self.repost_config.write().await;
                    repost_config.forums.remove(&forum);
                    if let Err(err) = self.save_config(&repost_config).await { return error!("{}", err); }
                    if let Err(err) = command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content(format!("Forum {} déconnecté au channel {} !", forum.mention(), channel.mention())))).await {
                        error!("Failed to send confirmation message {}", err)
                    }
                }
            }
            "reposte" => {
                if let Err(err) = command.defer_ephemeral(&ctx.http).await {
                    return error!("Failed to defer command interaction : {}", err);
                }
                let thread = match command.channel_id.to_channel(&ctx.http).await {
                    Ok(thread) => {
                        match thread.guild() {
                            None => { return error!("Failed to get guild thread") }
                            Some(guild) => { guild }
                        }
                    }
                    Err(err) => { return error!("Failed to get thread : {}", err) }
                };

                let reposted_message = match &command.data.options().find("message") {
                    None => { return error!("Missing option 'message'") }
                    Some(message) => {
                        if let ResolvedValue::String(message) = message {
                            let id = match message.split("/").last() {
                                None => { message }
                                Some(last) => { last }
                            };
                            match u64::from_str(id) {
                                Ok(id) => {
                                    match thread.message(&ctx.http, MessageId::from(id)).await {
                                        Ok(message) => { message }
                                        Err(err) => { return command.respond_user_error(&ctx.http, format!("Le message fourni n'est pas valid : {}", err)).await; }
                                    }
                                }
                                Err(_) => {
                                    return command.respond_user_error(&ctx.http, "L'option message doit être un identifiant de message ou le lien vers le message").await;
                                }
                            }
                        } else { return error!("Not a stringValue"); }
                    }
                };

                let forum = match thread.parent_id {
                    None => {
                        return command.respond_user_error(&ctx.http, "La commande doit être exécutée depuis un fil qui t'appartient").await;
                    }
                    Some(forum) => {
                        match forum.to_channel(&ctx.http).await {
                            Ok(forum) => {
                                match forum.guild() {
                                    None => { return error!("Channel is not a guild channel") }
                                    Some(channel) => { channel }
                                }
                            }
                            Err(err) => { return error!("Failed to get forum data : {}", err) }
                        }
                    }
                };

                let mut config = self.repost_config.write().await;
                let repost_config = match config.forums.get(&forum.id) {
                    None => {
                        return command.respond_user_error(&ctx.http, "La fonctionnalité de reposte n'est pas activée ici").await;
                    }
                    Some(forum_config) => { forum_config.clone() }
                };

                let member = match &command.member {
                    None => { return error!("Invalid member") }
                    Some(member) => { member.clone() }
                };

                for repost_channel in &repost_config.repost_channel {
                    for message in make_repost_message(&reposted_message, &thread, &forum.name, member.as_ref()) {
                        match repost_channel.send_message(&ctx.http, message).await {
                            Ok(message) => {
                                if let Some(votes) = config.votes.get_mut(&thread.id) {
                                    votes.reposted_message.insert(message.into());
                                }
                            }
                            Err(err) => { return error!("Failed to repost message in {} : {}", repost_channel.mention(), err); }
                        }
                    }
                }
                if repost_config.vote_enabled {
                    if let Err(err) = self.save_config(&config).await { return error!("{}", err); }
                    if let Err(err) = self.update_vote_messages(&ctx, thread, &config).await {
                        error!("Failed to update vote messages : {}", err)
                    }
                }
                println!("au bout");

                if let Err(err) = command.edit_response(&ctx.http, EditInteractionResponse::new().content("Message reposté !")).await {
                    error!("Failed to send confirmation message : {}", err)
                }
            }
            &_ => {}
        }
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