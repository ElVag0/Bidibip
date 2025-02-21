use std::sync::Arc;
use anyhow::Error;
use serenity::all::{Context, EventHandler, GuildId, Member, User};
use crate::core::config::Config;
use crate::core::module::BidibipSharedData;
use crate::modules::{BidibipModule, LoadModule};

struct Welcome {
    config: Arc<Config>,
}

struct WelcomeConfig {
    join_channel: u64,
    leave_channel: u64,
    welcome_messages: Vec<String>,
    leave_messages: Vec<String>
}

impl LoadModule<Welcome> for Welcome {
    fn name() -> &'static str {
        "welcome"
    }

    fn description() -> &'static str {
        "messages de bienvenue et de départ"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Welcome, Error> {
        Ok(Welcome { config: shared_data.config.clone() })
    }
}

#[serenity::async_trait]
impl EventHandler for Welcome {
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        todo!()
    }

    async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, member_data_if_available: Option<Member>) {
        todo!()
    }
}

impl BidibipModule for Welcome {}