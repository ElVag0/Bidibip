use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use serenity::all::{ChannelId, Context, GuildId, Interaction, Ready};
use serenity::prelude::EventHandler;
use tracing::{error, info};
use crate::core::config::Config;
use crate::core::logger::DiscordLogConnector;
use crate::modules::{load_modules, BidibipModule};

pub struct GlobalInterface {
    config: Arc<Config>,
    modules: Vec<ModuleData>,
    log_connector: Arc<DiscordLogConnector>
}

struct ModuleData {
    module: Box<dyn BidibipModule>,
    commands: HashSet<String>
}

impl GlobalInterface {
    pub fn new(config: Arc<Config>, log_connector: Arc<DiscordLogConnector>) -> Self {
        let mut modules = vec![];
        for module in load_modules(config.clone()) {
            let mut commands = HashSet::new();
            for command in module.fetch_commands() {
                commands.insert(command.0);
            }

            modules.push(ModuleData{module, commands})
        }


        Self { config:config.clone(), modules, log_connector }
    }

    pub async fn update_commands(&self, ctx: &Context) {
        let mut commands = HashMap::new();

        for module in &self.modules {
            for (name, command) in module.module.fetch_commands() {
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
            module.module.ready(ctx.clone(), ready.clone()).await;
            info!("Initialized module {}", module.module.name());
        }

        info!("Je suis prêt à botter des culs ! >:)");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        for module in &self.modules {
            module.module.interaction_create(ctx.clone(), interaction.clone()).await;
        }

        if let Interaction::Command(command) = interaction {
            for module in &self.modules {
                if module.commands.contains(&command.data.name) {
                    module.module.execute_command(ctx.clone(), command.data.name.as_str(), command.clone()).await;
                }
            }
        }
    }
}