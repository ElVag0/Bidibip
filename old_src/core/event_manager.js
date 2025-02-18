const DI = require("../utils/discord_interface");
const {Message} = require("../utils/message");
const {InteractionBase, CommandInteraction, InteractionButton} = require("../utils/interactionBase");
const {User} = require("../utils/user");
const CONFIG = require("../config");
const {CommandDispatcher} = require("./command_dispatcher");
const {Channel} = require("../utils/channel");
const {Thread} = require("../utils/thread");
const {Reaction} = require("../utils/reaction");


class EventManager {
    constructor(client) {
        this._bound_modules = []
        this._command_modules = {}
        this._command_dispatcher = new CommandDispatcher(client)
        this._bound_buttons = {}

        DI.get().on_reaction_add = (reaction, user) => {
            this._reaction_added(new Reaction(reaction), new User(user))
        }
        DI.get().on_thread_create = thread => {
            this._thread_created(new Thread(thread))
        }
        DI.get().on_user_join = user => {
            this._on_user_join(new User(user))
        }
        DI.get().on_user_leave = user => {
            this._on_user_leave(new User(user))
        }
        DI.get().on_user_excluded = (targetId, executorId, reason, until) => {
            this._on_user_excluded(new User().set_id(targetId), new User().set_id(executorId), reason, until)
        }
        DI.get().on_user_kicked = (targetId, executorId, reason) => {
            this._on_user_kicked(new User().set_id(targetId), new User().set_id(executorId), reason)
        }
        DI.get().on_user_banned = (targetId, executorId, reason) => {
            this._on_user_banned(new User().set_id(targetId), new User().set_id(executorId), reason)
        }
        DI.get().on_user_unbanned = (targetId, executorId, reason) => {
            this._on_user_unbanned(new User().set_id(targetId), new User().set_id(executorId), reason)
        }
        DI.get().on_message_delete = (msg, from, by) => {
            const message = new Message(msg)
            if (!message.is_dm())
                this._server_message_delete(message, from, by)
        }
        DI.get().on_message_update = (old_message, new_message) => {
            const old = new Message(old_message)
            if (!old.is_dm())
                this._server_message_updated(old, new Message(new_message))
        }
        DI.get().on_message = msg => {
            const message = new Message(msg)
            if (message.is_dm())
                this._dm_message(message)
            else
                this._server_message(message)
        }
        DI.get().on_interaction = interaction => {
            switch (interaction.type) {
                case 2: // Chat command
                    const source_command = this._command_dispatcher.find(interaction.commandName)
                    const command_interaction = new CommandInteraction(this._command_dispatcher.find(interaction.commandName), interaction)

                    // Log
                    let option_string = ''
                    for (const [name, value] of Object.entries(command_interaction.options())) option_string += name + ': ' + value + ', '
                    console.validate(`User [${interaction.user.username}#${interaction.user.discriminator}] issued {'${interaction.commandName} ${option_string.substring(0, option_string.length - 2)}'}`)

                    // Ensure command exists
                    if (source_command === null) {
                        command_interaction.reply(new Message().set_text("Commande inconnue").set_client_only())
                            .catch(err => console.fatal(`failed to reply to interaction : ${err}`))
                        return
                    }

                    // Ensure user has permissions
                    if (!command_interaction.check_permissions()) {
                        command_interaction.author().full_name()
                            .then(full_name => {
                                new Message()
                                    .set_channel(new Channel().set_id(CONFIG.get().LOG_CHANNEL_ID))
                                    .set_text(`${full_name} (${'<@' + new User(interaction.user).id() + '>'}) a essayé d'exécuter la commande '${command.name}' sans en avoir la permission ${CONFIG.get().SERVICE_ROLE} !`)
                                    .send()
                                    .catch(err => console.fatal(`failed to notify usage error : ${err}`))
                                console.warning(`@${full_name} a essayé d\'exécuter la commande \'${command.name}\' sans en avoir la permission !`)
                            })
                            .catch(err => {
                                console.fatal(`failed to notify usage error : ${err}`)
                            })
                        command_interaction.skip()
                            .catch(err => console.fatal(`failed to skip interaction : ${err}`))
                    } else
                        source_command.execute(command_interaction)
                    break
                case 3: // Button clicked
                    (async () => {
                        const button_interaction = new InteractionButton(interaction)
                        let key = `${button_interaction.channel().id()}/${button_interaction.message().id()}`

                        let buttons = this._bound_buttons[key]
                        if (!buttons && interaction.message.interaction) {
                            key = `${button_interaction.channel().id()}/${interaction.message.interaction.id}`
                            buttons = this._bound_buttons[key]
                        }
                        console.validate(`User [${interaction.user.username}#${interaction.user.discriminator}] clicked on button #${button_interaction.button_id()} in ${button_interaction.channel().url()}`)

                        if (buttons) {
                            const removed = []
                            for (const button of buttons)
                                if (await button.callback.call(module, button_interaction).catch(err => console.fatal(`Button Interaction failed : ${err}`)) === false)
                                    removed.push(button)
                            for (const button of removed)
                                buttons.splice(buttons.indexOf(button), 1)

                            if (buttons.length === 0)
                                delete this._bound_buttons[key]
                        } else {
                            button_interaction.reply(new Message().set_client_only().set_text('Cette interaction n\'est plus valide')).catch(err => console.fatal(`failed to reply to interaction : ${err}`))
                            const message = button_interaction.message()
                            const disabled_button = await message.get_button_by_id(button_interaction.button_id())
                            disabled_button.set_enabled(false)
                            message.update(message)
                                .catch(err => console.warning(`udpate failed : ${err}`))
                        }
                    })().catch(err => console.error(`Button Interaction failed : ${err}`))
                    break
                default:
                    console.error('undefined interaction : ', interaction)
            }
        }
    }

    bind(module) {
        const index = this._bound_modules.indexOf(module)
        if (index !== -1)
            return // already bound

        // Register old
        if (module.commands)
            for (const command of module.commands) {
                if (!this._command_modules[command.name])
                    this._command_modules[command.name] = [module]
                else
                    this._command_modules[command.name].push(module)
            }

        this._command_dispatcher.add(module)
        this._bound_modules.push(module)
    }

    unbind(module) {
        const index = this._bound_modules.indexOf(module)
        if (index === -1)
            return // already unbound

        this._command_dispatcher.remove(module)
        this._bound_modules.splice(this._bound_modules.indexOf(module), 1)

        // Unregister old
        for (const [, value] of Object.entries(this._command_modules))
            if (value.indexOf(module) !== 0)
                value.splice(value.indexOf(module), 1)
    }

    _server_message(message) {
        for (const module of this._bound_modules)
            if (module.server_message)
                (async () => {
                    module.server_message(message)
                        .catch(err => console.error(`Failed to call 'server_message()' on module ${module.name} :\n${err}`))
                })()
    }

    _dm_message(message) {
        for (const module of this._bound_modules)
            if (module.dm_message)
                (async () => {
                    module.dm_message(message)
                        .catch(err => console.error(`Failed to call 'dm_message()' on module ${module.name} :\n${err}`))
                })()
    }

    _server_message_updated(old_message, new_message) {
        for (const module of this._bound_modules)
            if (module.server_message_updated)
                (async () => {
                    module.server_message_updated(old_message, new_message)
                        .catch(err => console.error(`Failed to call 'server_message_updated()' on module ${module.name} :\n${err}`))
                })()
    }

    _server_message_delete(message, from, by) {
        for (const module of this._bound_modules)
            if (module.server_message_delete)
                (async () => {
                    module.server_message_delete(message, from, by)
                        .catch(err => console.error(`Failed to call 'server_message_delete()' on module ${module.name} :\n${err}`))
                })()
    }

    _thread_created(thread) {
        for (const module of this._bound_modules)
            if (module.thread_created)
                (async () => {
                    module.thread_created(thread)
                        .catch(err => console.error(`Failed to call 'thread_created()' on module ${module.name} :\n${err}`))
                })()
    }

    _reaction_added(reaction, user) {
        for (const module of this._bound_modules)
            if (module.add_reaction)
                (async () => {
                    module.add_reaction(reaction, user)
                        .catch(err => console.error(`Failed to call 'add_reaction()' on module ${module.name} :\n${err}`))
                })()
    }

    _on_user_join(user) {
        for (const module of this._bound_modules)
            if (module.user_joined)
                (async () => {
                    module.user_joined(user)
                        .catch(err => console.error(`Failed to call 'user_joined()' on module ${module.name} :\n${err}`))
                })()
    }

    _on_user_leave(user) {
        for (const module of this._bound_modules)
            if (module.user_leaved)
                (async () => {
                    module.user_leaved(user)
                        .catch(err => console.error(`Failed to call 'user_leaved()' on module ${module.name} :\n${err}`))
                })()
    }
    _on_user_excluded(target, executor, reason, until) {
        for (const module of this._bound_modules)
            if (module.user_excluded)
                (async () => {
                    module.user_excluded(target, executor, reason, until)
                        .catch(err => console.error(`Failed to call 'user_excluded()' on module ${module.name} :\n${err}`))
                })()
    }
    _on_user_kicked(target, executor, reason) {
        for (const module of this._bound_modules)
            if (module.user_kicked)
                (async () => {
                    module.user_kicked(target, executor, reason)
                        .catch(err => console.error(`Failed to call 'user_kicked()' on module ${module.name} :\n${err}`))
                })()
    }
    _on_user_banned(target, executor, reason) {
        for (const module of this._bound_modules)
            if (module.user_banned)
                (async () => {
                    module.user_banned(target, executor, reason)
                        .catch(err => console.error(`Failed to call 'user_banned()' on module ${module.name} :\n${err}`))
                })()
    }
    _on_user_unbanned(target, executor, reason) {
        for (const module of this._bound_modules)
            if (module.user_unbanned)
                (async () => {
                    module.user_unbanned(target, executor, reason)
                        .catch(err => console.error(`Failed to call 'user_unbanned()' on module ${module.name} :\n${err}`))
                })()
    }
    get_commands(permissions) {
        return this._command_dispatcher ? this._command_dispatcher.get_commands(permissions) : null
    }

    bind_button(module, button_message, callback) {
        const key = `${button_message.channel().id()}/${button_message.id()}`
        if (!this._bound_buttons[key])
            this._bound_buttons[key] = []

        for (const v of this._bound_buttons[key])
            if (v.callback === callback && module === v.module)
                return

        this._bound_buttons[key].push({
            module: module,
            callback: callback
        })
    }
}

module.exports = {EventManager}