mod core;
mod modules;

use std::{env};
use std::path::{Path};
use std::sync::Arc;
use serenity::all::token::validate;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::error;
use crate::core::config::Config;
use crate::core::module::GlobalInterface;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let log_connector = core::logger::init_logger(Path::new("saved/log"));

    // Open Config
    let config = match Config::from_file(env::current_exe().expect("Failed to find executable path").parent().unwrap().join("config.json")) {
        Ok(config) => { Arc::new(config) }
        Err(error) => {
            error!("Failed to load config : {}", error);
            return;
        }
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents =
        GatewayIntents::GUILDS |
            GatewayIntents::GUILD_MESSAGES |
            GatewayIntents::GUILD_MEMBERS |
            GatewayIntents::MESSAGE_CONTENT |
            GatewayIntents::GUILD_MESSAGE_REACTIONS |
            GatewayIntents::DIRECT_MESSAGES |
            GatewayIntents::GUILD_MODERATION;

    if validate(&config.token).is_err() {
        error!("Invalid token. Please check config file first");
        return;
    }

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&config.token, intents).event_handler(GlobalInterface::new(config, log_connector)).await.expect("Failed to create client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}