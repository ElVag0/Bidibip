use std::sync::{Arc};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Context, CreateMessage, GuildId, Member, Mentionable, User};
use crate::core::module::BidibipSharedData;
use crate::modules::{BidibipModule, LoadModule};
use rand::seq::{IndexedRandom};
use crate::core::config::Config;
use crate::core::error::BidibipError;
use crate::core::utilities::TruncateText;
use crate::on_fail;

pub struct Welcome {
    welcome_config: WelcomeConfig,
}

#[derive(Serialize, Deserialize, Default)]
struct WelcomeConfig {
    join_channel: ChannelId,
    leave_channel: ChannelId,
    reglement_channel: ChannelId,
    welcome_messages: Vec<String>,
    leave_messages: Vec<String>,
}

impl LoadModule<Welcome> for Welcome {
    fn name() -> &'static str {
        "welcome"
    }

    fn description() -> &'static str {
        "messages de bienvenue et de départ"
    }

    async fn load(_: &Arc<BidibipSharedData>) -> Result<Welcome, Error> {
        let welcome_config = Config::get().load_module_config::<Welcome, WelcomeConfig>()?;
        Ok(Welcome { welcome_config })
    }
}

#[serenity::async_trait]
impl BidibipModule for Welcome {
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) -> Result<(), BidibipError> {
        let mut sentence = match self.welcome_config.welcome_messages.choose(&mut rand::rng()) {
            None => { String::from("Bienvenue parmi nous {} :wave: !") }
            Some(sentence) => { sentence.clone() }
        };
        sentence += "\n> N'oublies pas de lire le {reglement} pour accéder au serveur.";
        let sentence = sentence.replace("{user}", new_member.user.mention().to_string().as_str()).replace("{reglement}", self.welcome_config.reglement_channel.mention().to_string().as_str());
        on_fail!(self.welcome_config.join_channel.send_message(&ctx.http, CreateMessage::new().content(sentence.truncate_text(2000))).await, "Failed to send welcome message")?;
        Ok(())
    }

    async fn guild_member_removal(&self, ctx: Context, _: GuildId, user: User, _: Option<Member>) -> Result<(), BidibipError> {
        let sentence = match self.welcome_config.leave_messages.choose(&mut rand::rng()) {
            None => { String::from("{} nous a quitté !") }
            Some(sentence) => { sentence.clone() }
        };
        let sentence = sentence.replace("{user}", user.mention().to_string().as_str());
        on_fail!(self.welcome_config.leave_channel.send_message(&ctx.http, CreateMessage::new().content(sentence.truncate_text(2000))).await, "Failed to send leave message")?;
        Ok(())
    }
}