use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serenity::all::{CommandInteraction, CommandType, Context, CreateCommand, CreateCommandOption, EntryPointHandlerType, EventHandler, InstallationContext, InteractionContext, Permissions};
use tracing::error;
use crate::core::config::Config;
use crate::modules::warn::Warn;

mod say;
mod warn;
mod log;
mod history;
mod modo;
mod help;

pub struct CreateCommandDetailed {
    pub name: String,
    pub name_localizations: HashMap<String, String>,
    pub description: Option<String>,
    pub description_localizations: HashMap<String, String>,
    pub options: Vec<CreateCommandOption>,
    pub default_member_permissions: Option<String>,
    pub dm_permission: Option<bool>,
    pub kind: Option<CommandType>,
    pub integration_types: Option<Vec<InstallationContext>>,
    pub contexts: Option<Vec<InteractionContext>>,
    pub nsfw: bool,
    pub handler: Option<EntryPointHandlerType>,
}

impl CreateCommandDetailed {
    /// Creates a new builder with the given name, leaving all other fields empty.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            kind: None,

            name: name.into(),
            name_localizations: HashMap::new(),
            description: None,
            description_localizations: HashMap::new(),
            default_member_permissions: None,
            dm_permission: None,

            integration_types: None,
            contexts: None,

            options: Vec::new(),
            nsfw: false,
            handler: None,
        }
    }

    /// Specifies the name of the application command, replacing the current value as set in
    /// [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`. Two
    /// global commands of the same app cannot have the same name. Two guild-specific commands of
    /// the same app cannot have the same name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Specifies a localized name of the application command.
    ///
    /// ```rust
    /// # serenity::builder::CreateCommand::new("")
    /// .name("birthday")
    /// .name_localized("zh-CN", "生日")
    /// .name_localized("el", "γενέθλια")
    /// # ;
    /// ```
    pub fn name_localized(mut self, locale: impl Into<String>, name: impl Into<String>) -> Self {
        self.name_localizations.insert(locale.into(), name.into());
        self
    }

    /// Specifies the type of the application command.
    pub fn kind(mut self, kind: CommandType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Specifies the default permissions required to execute the command.
    pub fn default_member_permissions(mut self, permissions: Permissions) -> Self {
        self.default_member_permissions = Some(permissions.bits().to_string());
        self
    }

    /// Specifies if the command is available in DMs.
    #[cfg_attr(feature = "unstable_discord_api", deprecated = "Use contexts instead")]
    pub fn dm_permission(mut self, enabled: bool) -> Self {
        self.dm_permission = Some(enabled);
        self
    }

    /// Specifies the description of the application command.
    ///
    /// **Note**: Must be between 1 and 100 characters long.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Specifies a localized description of the application command.
    ///
    /// ```rust
    /// # serenity::builder::CreateCommand::new("")
    /// .description("Wish a friend a happy birthday")
    /// .description_localized("zh-CN", "祝你朋友生日快乐")
    /// # ;
    /// ```
    pub fn description_localized(
        mut self,
        locale: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.description_localizations.insert(locale.into(), description.into());
        self
    }

    /// Adds an application command option for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn add_option(mut self, option: CreateCommandOption) -> Self {
        self.options.push(option);
        self
    }

    /// Sets all the application command options for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn set_options(mut self, options: Vec<CreateCommandOption>) -> Self {
        self.options = options;
        self
    }

    /// Adds an installation context that this application command can be used in.
    pub fn add_integration_type(mut self, integration_type: InstallationContext) -> Self {
        self.integration_types.get_or_insert_with(Vec::default).push(integration_type);
        self
    }

    /// Sets the installation contexts that this application command can be used in.
    pub fn integration_types(mut self, integration_types: Vec<InstallationContext>) -> Self {
        self.integration_types = Some(integration_types);
        self
    }

    /// Adds an interaction context that this application command can be used in.
    pub fn add_context(mut self, context: InteractionContext) -> Self {
        self.contexts.get_or_insert_with(Vec::default).push(context);
        self
    }

    /// Sets the interaction contexts that this application command can be used in.
    pub fn contexts(mut self, contexts: Vec<InteractionContext>) -> Self {
        self.contexts = Some(contexts);
        self
    }

    /// Whether this command is marked NSFW (age-restricted)
    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.nsfw = nsfw;
        self
    }

    /// Sets the command's entry point handler type. Only valid for commands of type
    /// [`PrimaryEntryPoint`].
    ///
    /// [`PrimaryEntryPoint`]: CommandType::PrimaryEntryPoint
    pub fn handler(mut self, handler: EntryPointHandlerType) -> Self {
        self.handler = Some(handler);
        self
    }
}



#[serenity::async_trait]
pub trait BidibipModule: Sync + Send + EventHandler {
    // Module display name
    fn name(&self) -> &'static str;

    // Get a list of available commands for this module
    fn fetch_commands(&self) -> Vec<CreateCommandDetailed> { vec![] }

    // When one of the specified command is executed
    async fn execute_command(&self, _ctx: Context, _name: &str, _command: CommandInteraction) {}
}

pub async fn load_modules(config: Arc<Config>) -> Vec<Box<dyn BidibipModule>> {
    let mut modules: Vec<Box<dyn BidibipModule>> = vec![];

    // SAY
    modules.push(Box::new(say::Say {}));

    // WARN
    match Warn::new(config.clone()).await {
        Ok(module) => {
            modules.push(Box::new(module))
        }
        Err(err) => { error!("Failed to load warn module : {err}") }
    }

    // LOG
    modules.push(Box::new(log::Log {}));

    // HISTORY
    modules.push(Box::new(history::History::new(config.clone())));

    // HELP
    modules.push(Box::new(help::Help{}));

    // MODO
    match modo::Modo::new(config.clone()).await {
        Ok(module) => {
            modules.push(Box::new(module))
        }
        Err(err) => { error!("Failed to load modo module : {err}") }
    }

    modules
}