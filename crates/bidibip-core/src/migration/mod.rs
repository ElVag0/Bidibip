use serenity::all::Context;

mod warn_migration;
mod repost_migration;
mod modo_migration;

#[allow(unused)]
pub async fn migrate(ctx: &Context) {
    println!("migrate warn");
    warn_migration::migrate();
    println!("migrate repost");
    repost_migration::migrate(ctx).await;
    println!("migrate modo");
    modo_migration::migrate(ctx).await;
    println!("migration done");
}