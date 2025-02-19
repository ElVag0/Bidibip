use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use serenity::all::{ChannelId, Command, CommandPermissions, Context, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Interaction, Ready};
use serenity::builder::Builder;
use serenity::prelude::EventHandler;
use tokio::sync::RwLock;
use tracing::{error, info};
use crate::core::config::Config;
use crate::core::logger::DiscordLogConnector;
use crate::modules::{load_modules, Module};

pub struct GlobalInterface {
    config: Arc<Config>,
    modules: Vec<Box<dyn Module>>,
    log_connector: Arc<DiscordLogConnector>
}

impl GlobalInterface {
    pub fn new(config: Arc<Config>, log_connector: Arc<DiscordLogConnector>) -> Self {
        Self { config:config.clone(), modules: load_modules(config), log_connector }
    }

    pub async fn update_commands(&self, ctx: &Context) {
        let mut commands = HashMap::new();

        for module in &self.modules {
            for (name, command) in module.fetch_command() {
                commands.insert(name.clone(), command.name(name));
            }
        }

        let guild_id = GuildId::new(self.config.server_id);

        for command in guild_id.get_commands(&ctx.http).await.unwrap() {
            if commands.contains_key(&command.name) {
                commands.remove(&command.name);
            } else {
                match guild_id.delete_command(&ctx.http, command.id).await {
                    Ok(_) => {}
                    Err(err) => { error!("Failed to remove outdated command {err}") }
                };
            }
        }

        for command in commands {
            match guild_id.create_command(&ctx.http, command.1).await {
                Ok(command) => { info!("Registered new command {}", command.name) }
                Err(err) => { error!("Failed to register new command {err}") }
            };
        }
    }
}

#[serenity::async_trait]
impl EventHandler for GlobalInterface {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.log_connector.init_for_channel(ChannelId::new(self.config.log_channel), ctx.http.clone());

        self.update_commands(&ctx).await;

        for module in &self.modules {
            module.ready(ctx.clone(), ready.clone()).await;
            info!("Initialized module {}", module.name());
        }

        info!("Je suis prêt à botter des culs ! >:)");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {

        for module in &self.modules {
            module.interaction_create(ctx.clone(), interaction.clone()).await;
        }
    }
}