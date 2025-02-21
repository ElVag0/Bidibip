use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, ChannelId, ChannelType, CommandInteraction, Context, CreateButton, CreateEmbedAuthor, CreateMessage, CreateThread, EditThread, EventHandler, Interaction, Mentionable, RoleId, UserId};
use serenity::builder::{CreateActionRow, CreateEmbed};
use tokio::sync::RwLock;
use tracing::{error, warn};
use crate::core::config::Config;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::core::utilities::{CommandHelper, Username};
use crate::modules::{BidibipModule, CreateCommandDetailed, LoadModule};

pub struct Modo {
    config: Arc<Config>,
    modo_config: RwLock<ModoConfig>,
}


#[derive(Serialize, Deserialize, Default)]
struct UserTickets {
    thread: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct ModoConfig {
    modo_channel: u64,
    tickets: HashMap<u64, UserTickets>,
}

impl LoadModule<Modo> for Modo {
    fn name() -> &'static str {
        "modo"
    }

    fn description() -> &'static str {
        "Ouvre un canal directe avec la modération"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Modo, Error> {
        let module = Self { config: shared_data.config.clone(), modo_config: Default::default() };
        let modo_config = shared_data.config.load_module_config::<Modo, ModoConfig>()?;
        if modo_config.modo_channel == 0 {
            return Err(Error::msg("Invalid modo channel id"));
        }
        *module.modo_config.write().await = modo_config;
        Ok(module)
    }
}

#[serenity::async_trait]
impl BidibipModule for Modo {
    fn fetch_commands(&self, config: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("modo").description("ouvre un canal direct avec la modération")]
    }

    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) {
        if name == "modo" {
            let mut modo_config = self.modo_config.write().await;

            // Get or create thread
            let mut thread = None;
            if modo_config.tickets.contains_key(&command.user.id.get()) {
                if let Some(ticket) = modo_config.tickets.get(&command.user.id.get()) {
                    match ChannelId::from(ticket.thread).to_channel(&ctx.http).await {
                        Ok(channel) => {
                            if let Some(guild_channel) = channel.guild() {
                                thread = Some(guild_channel);
                            } else {
                                modo_config.tickets.remove(&command.user.id.get());
                                warn!("Failed to get guild_channel for modo command !");
                            }
                        }
                        Err(err) => {
                            modo_config.tickets.remove(&command.user.id.get());
                            warn!("Failed to find existing modo thread ! {}", err);
                        }
                    }
                } else {
                    modo_config.tickets.remove(&command.user.id.get());
                    return error!("This should never happen !!");
                }
            }
            if thread.is_none() {
                let new_thread = match ChannelId::from(modo_config.modo_channel).create_thread(&ctx.http, CreateThread::new(Username::from_user(&command.user).safe_full()).invitable(false).kind(ChannelType::PrivateThread)).await {
                    Ok(thread) => { thread }
                    Err(err) => { return error!("Failed to create modo thread : {}", err) }
                };
                modo_config.tickets.insert(command.user.id.get(), UserTickets { thread: new_thread.id.get() });
                thread = Some(new_thread);
            };

            // Send message
            if let Some(thread) = thread {
                if let Err(err) = thread.id.add_thread_member(&ctx.http, command.user.id).await {
                    return error!("Failed to add user to modo thread {}", err);
                }
                let mention_to_admins = RoleId::from(self.config.roles.administrator).mention();

                let mut embed = CreateEmbed::new().field("Canal de communication ouvert :robot:", format!("Tu es maintenant en communication directe avec les {}.\nA toi de nous dire ce qui ne va pas.", mention_to_admins), false);

                if let Some(thumbnail) = command.user.avatar_url() {
                    embed = embed.author(CreateEmbedAuthor::new(format!("{} < A l'aide ! 🖐", command.user.name)).icon_url(thumbnail));
                } else {
                    embed = embed.title(format!("{} < A l'aide ! 🖐", command.user.name));
                }

                if let Err(err) = thread.send_message(&ctx.http, CreateMessage::new()
                    .content(format!("{} {}", command.user.mention(), mention_to_admins))
                    .embed(embed)
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new("modo_close_thread").label("Fermer la discussion").style(ButtonStyle::Secondary)])])).await {
                    return error!("Failed to send modo welcome message : {}", err);
                }

                if let Err(err) = thread.id.edit_thread(&ctx.http, EditThread::new().archived(false).locked(false)).await {
                    return error!("Failed to unarchive thread {}", err);
                }
            } else {
                return error!("Failed to get thread for modo command");
            }
            command.skip(&ctx.http).await;

            self.config.save_module_config::<Modo, ModoConfig>(&*modo_config).unwrap();
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Modo {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Component(component) = interaction {
            if component.data.custom_id == "modo_close_thread" {
                let modo_config = self.modo_config.read().await;
                for (user, ticket_data) in &modo_config.tickets {
                    if ticket_data.thread == component.channel_id.get() {
                        if let Err(err) = component.channel_id.remove_thread_member(&ctx.http, UserId::from(*user)).await {
                            return error!("Failed to remove user from modo thread {}", err);
                        }

                        if let Err(err) = component.channel_id.edit_thread(&ctx.http, EditThread::new().archived(true).locked(true)).await {
                            return error!("Failed to archive thread {}", err);
                        }
                    }
                }
                component.skip(&ctx.http).await;
            }
        }
    }
}
