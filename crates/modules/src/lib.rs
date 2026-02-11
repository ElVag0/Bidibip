use std::sync::{Arc};
use tracing::{error};
use utils::global_interface::BidibipSharedData;
use utils::module::{LoadModule, BidibipModule};
mod say;
mod warn;
mod log;
mod history;
mod modo;
mod help;
mod utilities;
mod welcome;
mod reglement;
mod repost;
mod advertising;
mod user_count;
mod anti_spam;
async fn load_module_helper<T: 'static + LoadModule<T> + BidibipModule>(shared_data: &Arc<BidibipSharedData>) {
    match T::load(shared_data).await {
        Ok(module) => {
            shared_data.register_module(module).await;
        }
        Err(err) => { error!("Failed to load module {} : {}", T::name(), err) }
    }
}

pub async fn load_modules(shared_data: &Arc<BidibipSharedData>) {
    load_module_helper::<say::Say>(shared_data).await;
    load_module_helper::<warn::Warn>(shared_data).await;
    load_module_helper::<log::Log>(shared_data).await;
    load_module_helper::<history::History>(shared_data).await;
    load_module_helper::<help::Help>(shared_data).await;
    load_module_helper::<modo::Modo>(shared_data).await;
    load_module_helper::<utilities::Utilities>(shared_data).await;
    load_module_helper::<welcome::Welcome>(shared_data).await;
    load_module_helper::<reglement::Reglement>(shared_data).await;
    load_module_helper::<repost::Repost>(shared_data).await;
    load_module_helper::<advertising::Advertising>(shared_data).await;
    load_module_helper::<user_count::UserCount>(shared_data).await;
    load_module_helper::<anti_spam::AntiSpam>(shared_data).await;
}