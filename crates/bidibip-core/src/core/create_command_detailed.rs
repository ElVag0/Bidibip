use std::collections::HashMap;
use std::str::FromStr;
use serenity::all::{CommandType, CreateCommand, CreateCommandOption, EntryPointHandlerType, InstallationContext, InteractionContext, Permissions};

#[derive(Clone)]
pub struct CreateCommandDetailed {
    pub name: String,
    pub name_localizations: HashMap<String, String>,
    pub description: Option<String>,
    pub description_localizations: HashMap<String, String>,
    pub options: Vec<CreateCommandOption>,
    pub default_member_permissions: Option<Permissions>,
    pub dm_permission: Option<bool>,
    pub kind: Option<CommandType>,
    pub integration_types: Option<Vec<InstallationContext>>,
    pub contexts: Option<Vec<InteractionContext>>,
    pub nsfw: bool,
    pub handler: Option<EntryPointHandlerType>,
}

impl From<CreateCommandDetailed> for CreateCommand {
    fn from(value: CreateCommandDetailed) -> Self {
        let mut cmd = CreateCommand::new(value.name.clone());

        for localization in value.name_localizations {
            cmd = cmd.name_localized(localization.0, localization.1);
        }
        if let Some(description) = value.description {
            cmd = cmd.description(description)
        }
        for localization in value.description_localizations {
            cmd = cmd.description_localized(localization.0, localization.1);
        }
        for option in value.options {
            cmd = cmd.add_option(option);
        }
        if let Some(perm) = value.default_member_permissions {
            cmd = cmd.default_member_permissions(perm);
        }
        if let Some(perm) = value.dm_permission {
            cmd = cmd.dm_permission(perm);
        }
        if let Some(kind) = value.kind {
            cmd = cmd.kind(kind)
        }
        if let Some(integration) = value.integration_types {
            cmd = cmd.integration_types(integration)
        }
        if let Some(contexts) = value.contexts {
            cmd = cmd.contexts(contexts)
        }
        cmd = cmd.nsfw(value.nsfw);
        if let Some(handler) = value.handler {
            cmd = cmd.handler(handler);
        }
        cmd
    }
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
        self.default_member_permissions = Some(permissions);
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


