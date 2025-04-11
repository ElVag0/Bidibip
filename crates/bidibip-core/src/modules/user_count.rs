use crate::core::error::BidibipError;
use crate::core::global_interface::BidibipSharedData;
use crate::modules::{BidibipModule, LoadModule};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{ActivityData, Context, GuildId, Member, Ready, User};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::core::config::Config;
use crate::on_fail;

#[derive(Serialize, Deserialize)]
pub struct UserCount {
    user_count: AtomicUsize,
}

impl UserCount {
    fn update(&self, ctx: Context) {
        ctx.set_activity(Some(ActivityData::custom(format!("Nous sommes {} membres", self.user_count.load(Ordering::SeqCst)))));
    }
}

impl LoadModule<UserCount> for UserCount {
    fn name() -> &'static str {
        "Compteur de membres"
    }

    fn description() -> &'static str {
        "Compte le nombre de membres et l'affiche dans l'activité de Bidibip"
    }

    async fn load(_: &Arc<BidibipSharedData>) -> Result<UserCount, Error> {
        Ok(Self {
            user_count: AtomicUsize::default(),
        })
    }
}
#[serenity::async_trait]
impl BidibipModule for UserCount {
    async fn guild_member_addition(&self, ctx: Context, _: Member) -> Result<(), BidibipError> {
        self.user_count.fetch_add(1, Ordering::SeqCst);
        self.update(ctx);
        Ok(())
    }
    async fn guild_member_removal(&self, ctx: Context, _: GuildId, _: User, _: Option<Member>) -> Result<(), BidibipError> {
        self.user_count.fetch_sub(1, Ordering::SeqCst);
        self.update(ctx);
        Ok(())
    }

    async fn ready(&self, ctx: Context, _: Ready) -> Result<(), BidibipError> {
        let count = on_fail!(Config::get().server_id.members(&ctx.http, None, None).await, "Failed to get all members")?.len();
        self.user_count.store(count, Ordering::SeqCst);
        self.update(ctx);
        Ok(())
    }
}