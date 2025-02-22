use std::collections::{HashMap, HashSet};
use std::sync::{Arc};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelType, CommandInteraction, CommandOptionType, Context, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, Mentionable, ResolvedValue, User};
use tokio::sync::RwLock;
use tracing::error;
use crate::core::config::Config;
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::core::utilities::{OptionHelper, Username};
use crate::modules::{BidibipModule, LoadModule};

pub struct Repost {
    config: Arc<Config>,
    repost_config: RwLock<RepostConfig>,
}

#[derive(Default, Serialize, Deserialize)]
struct VoteConfig {
    reposted_message: u64,
    vote_message: u64,
    yes: HashMap<u64, Username>,
    no: HashMap<u64, Username>,
}


#[derive(Default, Serialize, Deserialize)]
struct RepostChannelConfig {
    repost_channel: u64,
    vote_enabled: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct RepostConfig {
    // Forum - RepostChannel
    forums: HashMap<u64, RepostChannelConfig>,
    // Channel - config
    votes: HashMap<u64, VoteConfig>,
}

#[serenity::async_trait]
impl EventHandler for Repost {}

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
                    repost_config.forums.remove(&forum.get());
                    repost_config.forums.insert(forum.get(), RepostChannelConfig {
                        repost_channel: channel.get(),
                        vote_enabled: vote,
                    });
                    if let Err(err) = self.config.save_module_config::<Repost, RepostConfig>(&repost_config) {
                        return error!("Failed to save repost config {}", err);
                    }
                    if let Err(err) = command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content(format!("Forum {} connecté du channel {} !", forum.mention(), channel.mention())))).await {
                        error!("Failed to send confirmation message {}", err)
                    }
                } else {
                    let mut repost_config = self.repost_config.write().await;
                    repost_config.forums.remove(&forum.get());
                    if let Err(err) = self.config.save_module_config::<Repost, RepostConfig>(&repost_config) {
                        return error!("Failed to save repost config {}", err);
                    }
                    if let Err(err) = command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true).content(format!("Forum {} déconnecté au channel {} !", forum.mention(), channel.mention())))).await {
                        error!("Failed to send confirmation message {}", err)
                    }
                }
            }
            "reposte" => {}
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