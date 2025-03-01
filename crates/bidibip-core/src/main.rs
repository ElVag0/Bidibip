mod core;
mod modules;

use std::{env};
use serenity::all::token::validate;
use serenity::prelude::*;
use tracing::error;
use crate::core::config::Config;
use crate::core::module::GlobalInterface;

#[tokio::main]
async fn main() {
    // Open Config
    if let Err(error) = Config::init(env::current_exe().expect("Failed to find executable path").parent().unwrap().join("config.json")) {
        println!("Failed to load config : {}", error);
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