use std::fs::File;
use std::{io};
use std::env::current_exe;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use anyhow::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::all::{CommandInteraction, CommandType, Context, CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage};
use tokio::sync::RwLock;
use crate::core::config::Config;
use crate::core::create_command_detailed::CreateCommandDetailed;
use crate::core::error::BidibipError;
use crate::core::module::{BidibipSharedData, PermissionData};
use crate::modules::{BidibipModule, LoadModule};
use crate::core::utilities::CommandHelper;
use crate::{assert_some, on_fail};

pub struct Utilities {
    shared_data: Arc<BidibipSharedData>,
    utilities_config: RwLock<UtilitiesConfig>,
}

#[derive(Serialize, Deserialize, Default)]
struct UtilitiesConfig {
    current_version_release_date: Option<String>,
}

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
    async fn execute_command(&self, ctx: Context, cmd: &str, command: CommandInteraction) -> Result<(), BidibipError> {
        if cmd == "modules" {
            let modules = self.shared_data.modules.read().await;

            let mut actions = vec![];
            for module in modules.deref() {
                actions.push(CreateActionRow::Buttons(vec![CreateButton::new("test").label(module.name.clone())]))
            }


            on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("{} modules disponibles", modules.len()))
                    .ephemeral(true)
                    .components(actions)
            )).await, "Failed to create response")?;

            command.skip(&ctx.http).await;
        } else if cmd == "update" {
            let client = reqwest::Client::new();
            let response = on_fail!(client.get("https://api.github.com/repos/Unreal-Engine-FR/Bidibip/releases")
                .header("User-Agent", "Bidibip-updater")
                .send().await, "Failed to get release list")?;
            let data: Vec<ReleaseData> = on_fail!(response.json().await, "Faield to deserialize releases")?;

            if let Some(latest) = data.first() {
                let mut config = self.utilities_config.write().await;

                let remote_date: DateTime<Utc> = on_fail!(latest.published_at.parse(), "Failed to read remote date")?;

                let should_update = match &config.current_version_release_date {
                    None => { true }
                    Some(date) => {
                        let locale_date: DateTime<Utc> = on_fail!(date.parse(), "Failed to read remote date")?;
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
                    }
                } else {
                    on_fail!(command.create_response(&ctx.http, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().ephemeral(true)
                        .content(format!("La version actuelle de Bidibip est au moins aussi récente que la dernière disponible ({})", latest.name)))).await, "Failed to send response")?;
                }
            }
        }
        Ok(())
    }

    fn fetch_commands(&self, config: &PermissionData) -> Vec<CreateCommandDetailed> {
        vec![CreateCommandDetailed::new("modules")
                 .description("Informations sur les modules")
                 .kind(CommandType::ChatInput)
                 .default_member_permissions(config.at_least_admin()),
             CreateCommandDetailed::new("update")
                 .description("Mets à jour Bidibip")
                 .kind(CommandType::ChatInput)
                 .default_member_permissions(config.at_least_admin()),
             /*
             CreateCommandDetailed::new("settings")
                 .description("Panneau de configuration")
                 .kind(CommandType::ChatInput)
                 .default_member_permissions(config.at_least_admin())*/
        ]
    }
}