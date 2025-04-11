use std::fs;
use crate::modules::{BidibipModule, LoadModule};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use anyhow::Error;
use serenity::all::{ApplicationId, ChannelId, GuildId, RoleId};
use tracing::warn;
use crate::assert_some;
use crate::core::interaction_utils::make_custom_id;

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

#[derive(Serialize, Deserialize, Debug, Default)]
struct ButtonIds {
    max: u64,
    free: Vec<u64>,
}

fn is_id_zero(v: &u64) -> bool {
    *v == 0
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ButtonId {
    #[serde(skip_serializing_if = "is_id_zero")]
    #[serde(default)]
    id: u64,
}

impl From<u64> for ButtonId {
    fn from(value: u64) -> Self {
        Self { id: value }
    }
}

impl ButtonId {
    pub fn new() -> Result<Self, Error> {
        let mut buttons = Config::get().buttons.1.write().unwrap();

        let id = if let Some(old) = buttons.free.pop() {
            old
        } else {
            buttons.max += 1;
            buttons.max
        };

        fs::write(Config::get().buttons.0.clone(), serde_json::to_string(&*buttons)?)?;

        Ok(Self {
            id
        })
    }

    pub fn custom_id<T: BidibipModule + LoadModule<T>>(&self) -> String {
        make_custom_id::<T>(self.id.to_string().as_str(), "")
    }

    pub fn raw(&self) -> u64 {
        self.id
    }

    pub fn free(&mut self) -> Result<(), Error> {
        let mut buttons = Config::get().buttons.1.write().unwrap();

        buttons.free.push(self.id);
        self.id = 0;

        fs::write(Config::get().buttons.0.clone(), serde_json::to_string(&*buttons)?)?;
        Ok(())
    }
}

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub server_id: GuildId,
    pub application_id: ApplicationId,
    pub log_directory: PathBuf,
    pub button_id_config: PathBuf,
    pub module_config_directory: PathBuf,
    pub disabled_modules: Vec<String>,
    pub channels: Channels,
    pub roles: Roles,
    pub cache_message_size: usize,
    #[serde(skip_serializing, skip_deserializing)]
    buttons: (PathBuf, RwLock<ButtonIds>),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: "PLEASE FILL APP TOKEN FIRST".to_string(),
            server_id: GuildId::default(),
            application_id: ApplicationId::default(),
            log_directory: PathBuf::from("saved/logs"),
            button_id_config: PathBuf::from("button/buttons.json"),
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
            buttons: Default::default(),
        }
    }
}

impl Config {
    pub fn init(path: PathBuf) -> Result<(), Error> {
        if path.exists() {
            let mut config: Config = serde_json::from_str(&fs::read_to_string(&path)?)?;

            assert_ne!(config.application_id, 0, "Invalid application id in config");
            assert_ne!(config.server_id, 0, "Invalid server id in config");
            assert_ne!(config.roles.support, 0, "Invalid helper role id in config");
            assert_ne!(config.roles.member, 0, "Invalid member role id in config");
            assert_ne!(config.roles.helper, 0, "Invalid helper role id in config");
            assert_ne!(config.roles.administrator, 0, "Invalid administrator role id in config");
            assert_ne!(config.roles.mute, 0, "Invalid helper role id in config");
            assert_ne!(config.channels.staff_channel, 0, "Invalid staff channel id in config");
            assert_ne!(config.channels.log_channel, 0, "Invalid staff channel id in config");

            let parent = assert_some!(path.parent(), "Failed to get parent path")?;
            let buttons_path = Path::join(parent, "buttons.json");
            if !buttons_path.exists() {
                fs::write(buttons_path.clone(), serde_json::to_string_pretty(&ButtonIds::default())?)?;
            };
            config.buttons = (buttons_path.to_path_buf(), serde_json::from_str(&fs::read_to_string(&buttons_path)?)?);

            #[allow(unused)]
            GLOBAL_CONFIG.set(config);

            Ok(())
        } else {
            fs::write(path.clone(), serde_json::to_string_pretty(&Config::default())?)?;
            Err(Error::msg(format!("Created a new config file at {}. Please fill in information first", path.to_str().unwrap())))
        }
    }

    pub fn get() -> &'static Self {
        match GLOBAL_CONFIG.get() {
            None => { panic!("Global config have not been initialized using GLOBAL_CONFIG::init(path)") }
            Some(elem) => { elem }
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