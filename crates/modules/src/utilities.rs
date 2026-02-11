use std::fs::File;
use std::{io};
use std::collections::HashSet;
use std::env::current_exe;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use anyhow::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::all::{CommandInteraction, CommandOptionType, CommandType, Context, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, Ready, ResolvedValue};
use tokio::sync::RwLock;
use tracing::info;
use utils::module::{LoadModule, BidibipModule};
use utils::global_interface::BidibipSharedData;
use utils::error::BidibipError;
use utils::config::Config;
use utils::global_interface::PermissionData;
use utils::create_command_detailed::CreateCommandDetailed;
use utils::{on_fail, assert_some};
use utils::utilities::{OptionHelper, TruncateText};

pub struct Utilities {
    shared_data: Arc<BidibipSharedData>,
    utilities_config: RwLock<UtilitiesConfig>,
}

#[derive(Serialize, Deserialize, Default)]
struct UtilitiesConfig {
    disabled_modules: HashSet<String>,
    current_version_release_date: Option<String>,
}

#[serenity::async_trait]
impl LoadModule<Utilities> for Utilities {
    fn name() -> &'static str {
        "utilities"
    }

    fn description() -> &'static str {
        "Utilitaires de modération"
    }

    async fn load(shared_data: &Arc<BidibipSharedData>) -> Result<Utilities, Error> {
        Ok(Utilities { shared_data: shared_data.clone(), utilities_config: RwLock::new(Config::get().load_module_config::<Utilities, UtilitiesConfig>()?) })
    }
}

#[derive(Deserialize)]
struct AssetData {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct ReleaseData {
    name: String,
    published_at: String,
    assets: Vec<AssetData>,
}

#[serenity::async_trait]
impl BidibipModule for Utilities {
    async fn ready(&self, ctx: Context, _: Ready) -> Result<(), BidibipError> {
        let config = self.utilities_config.read().await;

        if let Some(version) = &config.current_version_release_date {
            info!("Version de bidibip : {}", version)
        }

        for module in &config.disabled_modules {
            self.shared_data.set_module_enabled(&ctx, module.as_str(), false, false).await;
        }
        Ok(())
    }

    async fn execute_command(&self, ctx: Context, cmd: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        match cmd {
            "modules" => {
                let enabled_modules = self.shared_data.get_enabled_modules().await;
                let disabled_modules = self.shared_data.get_disabled_modules().await;
                let mut available_modules = String::new();
                for module in enabled_modules.deref() {
                    available_modules += format!(":white_check_mark: `{}` : {}\n", module.name, module.description).as_str();
                }

                for module in disabled_modules.deref() {
                    available_modules += format!(":x: `{}` : {}\n", module.name, module.description).as_str();
                }

                on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("# {} / {} modules disponibles:\n{}", enabled_modules.len(), disabled_modules.len() + enabled_modules.len(), available_modules.truncate_text(1900)))
                    .ephemeral(true)
            )).await, "Failed to create response")?;
            }
            "update" => {
                let client = reqwest::Client::new();
                let response = on_fail!(client.get("https://api.github.com/repos/Unreal-Engine-FR/Bidibip/releases")
                .header("User-Agent", "Bidibip-updater")
                .send().await, "Failed to get release list")?;
                let data: Vec<ReleaseData> = on_fail!(response.json().await, "Failed to deserialize releases")?;

                if let Some(latest) = data.first() {
                    let mut config = self.utilities_config.write().await;

                    let remote_date: DateTime<Utc> = on_fail!(latest.published_at.parse(), "Failed to read remote date")?;

                    let should_update = match &config.current_version_release_date {
                        None => { true }
                        Some(date) => {
                            let locale_date: DateTime<Utc> = on_fail!(date.parse(), "Failed to read local date")?;
                            locale_date > remote_date
                        }
                    };

                    if should_update {
                        let mut response = None;

                        for asset in &latest.assets {
                            #[cfg(target_os = "windows")]
                            if asset.name == "bidibip_windows.zip" {
                                response = Some(on_fail!(client.get(&asset.browser_download_url)
                                .header("User-Agent", "Bidibip-updater")
                                .send().await, "Failed to get release list")?);
                            }
                            #[cfg(target_os = "linux")]
                            if asset.name == "bidibip_linux.zip" {
                                response = Some(on_fail!(client.get(&asset.browser_download_url)
                                .header("User-Agent", "Bidibip-updater")
                                .send().await, "Failed to get release list")?);
                            }
                        }

                        if let Some(response) = response {
                            let current_exe = on_fail!(current_exe(), "Failed to get exe path")?;
                            let exe = assert_some!(current_exe.parent(), "invalid parent path")?;

                            #[cfg(target_os = "windows")]
                            let new_binary = Path::join(exe, "bidibip-updated.exe");
                            #[cfg(target_os = "linux")]
                            let new_binary = Path::join(exe, "bidibip-updated");
                            let mut out = on_fail!(File::create(new_binary), "Failed to create update file")?;
                            let body = on_fail!(response.text().await, "Failed to get body")?;
                            on_fail!(io::copy(&mut body.as_bytes(), &mut out), "failed to copy update content")?;

                            on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true)
                            .content(format!("Mise à jour de bidibip disponible ({}). Redémarrage en cours...", latest.name)))).await, "Failed to send response")?;

                            config.current_version_release_date = Some(remote_date.to_string());
                            on_fail!(Config::get().save_module_config::<Utilities, UtilitiesConfig>(&config), "Failed to save config")?;
                            exit(0);
                        } else {
                            return Ok(());
                        }
                    } else {
                        on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true)
                        .content(format!("La version actuelle de Bidibip est au moins aussi récente que la dernière disponible ({})", latest.name)))).await, "Failed to send response")?;
                    }
                }
            }
            "set-module-enabled" => {
                let module = assert_some!(command.data.options().find("module"), "missing module option")?;
                let enabled = assert_some!(command.data.options().find("activer"), "missing activer option")?;

                if let ResolvedValue::String(name) = module {
                    if let ResolvedValue::Boolean(enabled) = enabled {
                        if !self.shared_data.available_modules().await.contains(&name.to_string()) {
                            on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Ce module n'existe pas").ephemeral(true))).await, "Failed to respond")?;
                            return Ok(());
                        }
                        self.shared_data.set_module_enabled(&ctx, name, enabled, true).await;
                        on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(
                            if enabled {
                                format!("Module {} activé", name)
                            } else {
                                format!("Module {} désactivé", name)
                            }
                        ).ephemeral(true))).await, "Failed to respond")?;

                        let mut config = self.utilities_config.write().await;
                        if enabled {
                            config.disabled_modules.remove(&name.to_string());
                        } else {
                            config.disabled_modules.insert(name.to_string());
                        }
                        on_fail!(Config::get().save_module_config::<Utilities, UtilitiesConfig>(&config), "Failed to save module config")?;
                        return Ok(());
                    }
                }

                on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("Invalid parameter").ephemeral(true))).await, "Failed to respond")?;
            }
            &_ => {}
        }
        Ok(())
    }

    fn fetch_commands(&self, config: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("modules")
                 .description("Informations sur les modules")
                 .kind(CommandType::ChatInput)
                 .default_member_permissions(config.at_least_admin()),
             CreateCommandDetailed::new("update")
                 .description("Redémarre et mets à jour Bidibip")
                 .kind(CommandType::ChatInput)
                 .default_member_permissions(config.at_least_admin()),
             CreateCommandDetailed::new("set-module-enabled")
                 .description("Active ou désactive un module")
                 .kind(CommandType::ChatInput)
                 .add_option(CreateCommandOption::new(CommandOptionType::String, "module", "nom du module concerné").required(true))
                 .add_option(CreateCommandOption::new(CommandOptionType::Boolean, "activer", "active ou désactive le module").required(true))
                 .default_member_permissions(config.at_least_admin())
        ]
    }
}