mod core;
mod modules;

use std::{env};
use std::path::PathBuf;
use serenity::all::token::validate;
use serenity::prelude::*;
use tracing::error;
use crate::core::config::Config;
use crate::core::global_interface::GlobalInterface;

#[tokio::main]
async fn main() {
    let config_path = match env::var("BIDIBIP_CONFIG") {
        Ok(config) => { PathBuf::from(config.as_str()) }
        Err(_) => {
            let args: Vec<String> = env::args().collect();
            if args.len() >= 2 {
                PathBuf::from(args[1].clone())
            } else {
                env::current_exe().expect("Failed to find executable path").parent().unwrap().join("config.json")
            }
        }
    };

    // Open Config
    if let Err(error) = Config::init(config_path.clone()) {
        println!("Failed to load config from {} : {}", config_path.display(), error);
        return;
    };
    let log_connector = core::logger::init_logger();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents =
        GatewayIntents::GUILDS |
            GatewayIntents::GUILD_MESSAGES |
            GatewayIntents::GUILD_MEMBERS |
            GatewayIntents::MESSAGE_CONTENT |
            GatewayIntents::GUILD_MESSAGE_REACTIONS |
            GatewayIntents::DIRECT_MESSAGES |
            GatewayIntents::GUILD_MODERATION;

    if validate(&Config::get().token).is_err() {
        error!("Invalid token. Please check config file first");
        return;
    }

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&Config::get().token, intents).event_handler(GlobalInterface::new(log_connector).await).await.expect("Failed to create client");
    client.cache.set_max_messages(Config::get().cache_message_size);
    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}