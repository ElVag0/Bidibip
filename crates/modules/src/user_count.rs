use anyhow::Error;
use utils::config::Config;
use utils::error::BidibipError;
use utils::global_interface::BidibipSharedData;
use utils::module::{BidibipModule, LoadModule};
use serde::{Deserialize, Serialize};
use serenity::all::{ActivityData, Context, GuildId, Http, Member, MembersIter, Ready, User};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use serenity::futures::StreamExt;
use tracing::info;

#[derive(Serialize, Deserialize)]
pub struct UserCount {
    user_count: AtomicUsize,
}

impl UserCount {
    fn update(&self, ctx: Context) {
        ctx.set_activity(Some(ActivityData::custom(format!("Nous sommes {} membres", self.user_count.load(Ordering::SeqCst)))));
    }
}

#[serenity::async_trait]
impl LoadModule<UserCount> for UserCount {
    fn name() -> &'static str {
        "member-count"
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
        let mut all_members: Vec<Member> = vec![];

        {
            let mut members = MembersIter::<Http>::stream(&ctx, Config::get().server_id).boxed();
            while let Some(member_result) = members.next().await {
                if let Ok(member) = member_result {
                    all_members.push(member);
                }
            }
        }

        let count = all_members.len();
        info!("There is {} users", all_members.len());
        self.user_count.store(count, Ordering::SeqCst);
        self.update(ctx);
        Ok(())
    }
}