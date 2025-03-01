use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ButtonStyle, ChannelId, ChannelType, CommandInteraction, Context, CreateButton, CreateEmbedAuthor, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateThread, EditThread, Interaction, Mentionable, UserId};
use serenity::builder::{CreateActionRow, CreateEmbed};
use tokio::sync::RwLock;
use tracing::{warn};
use crate::core::config::Config;
use crate::core::error::BidibipError;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::core::utilities::{CommandHelper, Username};
use crate::modules::{BidibipModule, CreateCommandDetailed, LoadModule};
use crate::{assert_some, on_fail};

pub struct Modo {
    modo_config: RwLock<ModoConfig>,
}


#[derive(Serialize, Deserialize, Default)]
struct UserTickets {
    thread: ChannelId,
}

#[derive(Serialize, Deserialize, Default)]
struct ModoConfig {
    modo_channel: ChannelId,
    tickets: HashMap<UserId, UserTickets>,
}

impl LoadModule<Modo> for Modo {
    fn name() -> &'static str {
        "modo"
    }

    fn description() -> &'static str {
        "Ouvre un canal directe avec la modération"
    }

    async fn load(_: &Arc<BidibipSharedData>) -> Result<Modo, Error> {
        let module = Self { modo_config: Default::default() };
        let modo_config = Config::get().load_module_config::<Modo, ModoConfig>()?;
        if modo_config.modo_channel == 0 {
            return Err(Error::msg("Invalid modo channel id"));
        }
        *module.modo_config.write().await = modo_config;
        Ok(module)
    }
}

#[serenity::async_trait]
impl BidibipModule for Modo {
    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        if name == "modo" {
            let mut modo_config = self.modo_config.write().await;

            // Get or create thread
            let mut thread = None;
            if modo_config.tickets.contains_key(&command.user.id) {
                let ticket = assert_some!(modo_config.tickets.get(&command.user.id), "This should never happen !!")?;
                match ticket.thread.to_channel(&ctx.http).await {
                    Ok(channel) => {
                        if let Some(guild_channel) = channel.guild() {
                            thread = Some(guild_channel);
                        } else {
                            modo_config.tickets.remove(&command.user.id);
                            warn!("Failed to get guild_channel for modo command !");
                        }
                    }
                    Err(err) => {
                        modo_config.tickets.remove(&command.user.id);
                        warn!("Failed to find existing modo thread ! {}", err);
                    }
                }
            }
            if thread.is_none() {
                let new_thread = on_fail!(modo_config.modo_channel.create_thread(&ctx.http, CreateThread::new(Username::from_user(&command.user).safe_full()).invitable(false).kind(ChannelType::PrivateThread)).await, "Failed to create modo thread")?;
                modo_config.tickets.insert(command.user.id, UserTickets { thread: new_thread.id });
                thread = Some(new_thread);
            };

            // Send message
            let thread = assert_some!(thread, "Failed to get thread for modo command")?;

            on_fail!(thread.id.add_thread_member(&ctx.http, command.user.id).await, "Failed to add user to modo thread")?;
            let mention_to_admins = Config::get().roles.administrator.mention();

            let mut embed = CreateEmbed::new().field("Canal de communication ouvert :robot:", format!("Tu es maintenant en communication directe avec les {}.\nA toi de nous dire ce qui ne va pas.", mention_to_admins), false);

            if let Some(thumbnail) = command.user.avatar_url() {
                embed = embed.author(CreateEmbedAuthor::new(format!("{} < A l'aide ! 🖐", command.user.name)).icon_url(thumbnail));
            } else {
                embed = embed.title(format!("{} < A l'aide ! 🖐", command.user.name));
            }

            on_fail!(thread.send_message(&ctx.http, CreateMessage::new()
                    .content(format!("{} {}", command.user.mention(), mention_to_admins))
                    .embed(embed)
                    .components(vec![CreateActionRow::Buttons(vec![CreateButton::new("modo_close_thread").label("Fermer la discussion").style(ButtonStyle::Secondary)])])).await, "Failed to send modo welcome message")?;

            on_fail!(thread.id.edit_thread(&ctx.http, EditThread::new().archived(false).locked(false)).await, "Failed to unarchive thread")?;

            on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .ephemeral(true)
                        .embed(CreateEmbed::new().title("Canal de communication ouvert").description(format!("Parle avec la modération ici : {}", thread.mention()))))).await, "Failed to send redirection message")?;

            on_fail!(Config::get().save_module_config::<Modo, ModoConfig>(&*modo_config), "Failed to save module config")?;
        }
        Ok(())
    }

    fn fetch_commands(&self, _: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("modo").description("ouvre un canal direct avec la modération")]
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> Result<(), BidibipError> {
        if let Interaction::Component(component) = interaction {
            if component.data.custom_id == "modo_close_thread" {
                let modo_config = self.modo_config.read().await;
                for (user, ticket_data) in &modo_config.tickets {
                    if ticket_data.thread == component.channel_id.get() {
                        on_fail!(component.channel_id.remove_thread_member(&ctx.http, *user).await, "Failed to remove user from modo thread")?;
                        on_fail!(component.channel_id.edit_thread(&ctx.http, EditThread::new().archived(true).locked(true)).await,"Failed to archive thread")?;
                    }
                }
                component.skip(&ctx.http).await;
            }
        }
        Ok(())
    }
}
