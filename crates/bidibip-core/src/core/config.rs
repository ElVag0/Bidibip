use std::fs;
use crate::modules::{BidibipModule, LoadModule};
use anyhow::Error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use serenity::all::{ApplicationId, ChannelId, GuildId, RoleId};
use tracing::warn;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Roles {
    pub support: RoleId,
    pub member: RoleId,
    pub helper: RoleId,
    pub administrator: RoleId,
    pub mute: RoleId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channels {
    pub log_channel: ChannelId, // Where everything is printed
    pub staff_channel: ChannelId, // The channel I should use to tell something important to the moderator team
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub server_id: GuildId,
    pub application_id: ApplicationId,
    pub log_directory: PathBuf,
    pub module_config_directory: PathBuf,
    pub disabled_modules: Vec<String>,
    pub channels: Channels,
    pub roles: Roles,
    pub cache_message_size: usize
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: "PLEASE FILL APP TOKEN FIRST".to_string(),
            server_id: GuildId::default(),
            application_id: ApplicationId::default(),
            log_directory: PathBuf::from("saved/logs"),
            module_config_directory: PathBuf::from("saved/config"),
            disabled_modules: vec![],
            channels: Channels {
                log_channel: ChannelId::default(),
                staff_channel: ChannelId::default(),
            },
            roles: Roles {
                support: RoleId::default(),
                member: RoleId::default(),
                helper: RoleId::default(),
                administrator: RoleId::default(),
                mute: RoleId::default(),
            },
            cache_message_size: 10000,
        }
    }
}

impl Config {
    pub fn from_file(path: PathBuf) -> Result<Self, Error> {
        if path.exists() {
            let config: Config = serde_json::from_str(&fs::read_to_string(path)?)?;

            assert_ne!(config.application_id, 0, "Invalid application id in config");
            assert_ne!(config.server_id, 0, "Invalid server id in config");
            assert_ne!(config.roles.support, 0, "Invalid helper role id in config");
            assert_ne!(config.roles.member, 0, "Invalid member role id in config");
            assert_ne!(config.roles.helper, 0, "Invalid helper role id in config");
            assert_ne!(config.roles.administrator, 0, "Invalid administrator role id in config");
            assert_ne!(config.roles.mute, 0, "Invalid helper role id in config");
            assert_ne!(config.channels.staff_channel, 0, "Invalid staff channel id in config");
            assert_ne!(config.channels.log_channel, 0, "Invalid staff channel id in config");

            Ok(config)
        } else {
            fs::write(path.clone(), serde_json::to_string_pretty(&Config::default())?)?;
            Err(Error::msg(format!("Created a new config file at {}. Please fill in information first", path.to_str().unwrap())))
        }
    }

    pub fn load_module_config<Module: LoadModule<Module> + BidibipModule, Config: Serialize + DeserializeOwned + Default>(&self) -> Result<Config, Error> {
        fs::create_dir_all(&self.module_config_directory)?;

        let config_file = self.module_config_directory.join(format!("{}_config.json", Module::name()));

        if !fs::exists(&config_file)? {
            // Create log files and channels
            fs::write(&config_file, serde_json::to_string_pretty(&Config::default())?)?;
            warn!("Initialized config file for module {} to {config_file:?}", Module::name());
        }

        Ok(serde_json::from_str(&fs::read_to_string(&config_file)?)?)
    }

    pub fn save_module_config<Module: LoadModule<Module> + BidibipModule, Config: Serialize>(&self, config: &Config) -> Result<(), Error> {
        fs::create_dir_all(&self.module_config_directory)?;

        let config_file = self.module_config_directory.join(format!("{}_config.json", Module::name()));
        fs::write(&config_file, serde_json::to_string_pretty(config)?)?;
        Ok(())
    }
}