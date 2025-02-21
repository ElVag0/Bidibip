use std::sync::{Arc};
use anyhow::Error;
use serenity::all::{CommandInteraction, Context, EventHandler};
use tracing::error;
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::module::{BidibipSharedData, ModuleData, PermissionData};

mod say;
mod warn;
mod log;
mod history;
mod modo;
mod help;
mod utilities;
mod welcome;
mod reglement;

#[serenity::async_trait]
pub trait BidibipModule: Sync + Send + EventHandler {
    // When one of the specified command is executed
    async fn execute_command(&self, _ctx: Context, _name: &str, _command: CommandInteraction) {}
    // Get a list of available commands for this module
    fn fetch_commands(&self, _config: &PermissionData) -> Vec<CreateCommandDetailed> { vec![] }
}

pub trait LoadModule<T: BidibipModule> {
    // Module display name
    fn name() -> &'static str;
    // Module display name
    fn description() -> &'static str;
    // Module constructor
    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<T, Error>;
}

async fn load_module<T: 'static + LoadModule<T> + BidibipModule>(shared_data: &Arc<BidibipSharedData>) {
    match T::load(shared_data).await {
        Ok(module) => {
            shared_data.modules.write().await.push(ModuleData {
                module: Box::new(module),
                command_names: Default::default(),
                name: T::name().to_string(),
                description: T::description().to_string(),
            });
        }
        Err(err) => { error!("Failed to load module {} : {}", T::name(), err) }
    }
}


pub async fn load_modules(shared_data: &Arc<BidibipSharedData>) {
    load_module::<say::Say>(shared_data).await;
    load_module::<warn::Warn>(shared_data).await;
    load_module::<log::Log>(shared_data).await;
    load_module::<history::History>(shared_data).await;
    load_module::<help::Help>(shared_data).await;
    load_module::<modo::Modo>(shared_data).await;
    load_module::<utilities::Utilities>(shared_data).await;
    load_module::<welcome::Welcome>(shared_data).await;
    load_module::<reglement::Reglement>(shared_data).await;
}