use std::sync::Arc;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateCommand, EventHandler};
use tokio::sync::RwLock;
use crate::core::config::Config;
use crate::modules::BidibipModule;

pub struct Modo {
    config: Arc<Config>,
    modo_config: RwLock<ModoConfig>
}

#[derive(Serialize, Deserialize, Default)]
struct ModoConfig {
    modo_channel: u64,
}

impl Modo {
    pub async fn new(config: Arc<Config>) -> Result<Self, Error> {
        let module = Self { config: config.clone(), modo_config: Default::default() };
        let modo_config: ModoConfig = config.load_module_config(&module)?;
        if modo_config.modo_channel == 0 {
            return Err(Error::msg("Invalid warn channel id"));
        }
        *module.modo_config.write().await = modo_config;
        Ok(module)
    }
}

#[serenity::async_trait]
impl BidibipModule for Modo {
    fn name(&self) -> &'static str {
        "Modo"
    }

    fn fetch_commands(&self) -> Vec<(String, CreateCommand)> {
        vec![("modo".to_string(), CreateCommand::new("modo").description("ouvre un canal direct avec la modération"))]
    }
}

#[serenity::async_trait]
impl EventHandler for Modo {}
