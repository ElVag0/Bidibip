use std::collections::HashMap;
use std::sync::Arc;
use std::thread::ThreadId;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, ChannelType, CommandInteraction, Context, CreateChannel, CreateCommand, CreateMessage, CreateThread, EditThread, EventHandler, GuildChannel, GuildId, Member, Mentionable, ThreadMember, UserId};
use serenity::builder::CreateEmbed;
use tokio::sync::RwLock;
use tracing::error;
use crate::core::config::Config;
use crate::core::utilities::{ResultDebug, Username};
use crate::modules::BidibipModule;

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

impl Modo {
    pub async fn new(config: Arc<Config>) -> Result<Self, Error> {
        let module = Self { config: config.clone(), modo_config: Default::default() };
        let modo_config: ModoConfig = config.load_module_config(&module)?;
        if modo_config.modo_channel == 0 {
            return Err(Error::msg("Invalid modo channel id"));
        }
        *module.modo_config.write().await = modo_config;
        Ok(module)
    }
}

#[serenity::async_trait]
impl BidibipModule for Modo {
    fn name(&self) -> &'static str {
        "Modo"
    }

    fn fetch_commands(&self) -> Vec<(String, CreateCommand)> {
        vec![("modo".to_string(), CreateCommand::new("modo").description("ouvre un canal direct avec la modération"))]
    }

    async fn execute_command(&self, ctx: Context, name: &str, command: CommandInteraction) {
        if name == "modo" {
            let mut modo_config = self.modo_config.write().await;

            if modo_config.tickets.contains_key(&command.user.id.get()) {} else {
                let mut thread = match ChannelId::from(modo_config.modo_channel).create_thread(&ctx.http, CreateThread::new(Username::from_user(&command.user).safe_full()).invitable(false).kind(ChannelType::PrivateThread)).await {
                    Ok(thread) => { thread }
                    Err(err) => { return error!("Failed to create modo thread : {}", err) }
                };

                if let Err(err) = thread.id.add_thread_member(&ctx.http, command.user.id).await {
                    return error!("Failed to add user to modo thread {}", err);
                }
                let mention_to_admins = UserId::from(self.config.roles.administrator).mention();

                let mut embed = CreateEmbed::new()
                    .title(format!("{} < A l'aide ! 🖐", ))
                    .field("Canal de communication ouvert :robot:", format!("Tu es maintenant en communication directe avec les {}.\nA toi de nous dire ce qui ne va pas.", mention_to_admins), false);

                if let Some(thumbnail) = command.user.avatar_url() {
                    embed = embed.thumbnail(thumbnail);
                }

                thread.send_message(&ctx.http, CreateMessage::new()
                    .content(format!("{} {}", command.user.mention(), mention_to_admins))
                    .embed())


                thread.edit_thread(&ctx.http, EditThread::new().archived(false).locked(false)).await;
            }
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Modo {}
