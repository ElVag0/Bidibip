const DI = require("../utils/discord_interface");

class CommandDispatcher {
    constructor() {
        this.modules = {}
        this.commands = {}
        this.interaction_listener = {}
    }

    add(module) {
        if (this.modules[module.name])
            return // already added

        if (!module.commands)
            return // doesn't implement old

        this.modules[module.name] = {module: module, command: module.commands}

        for (const command of module.commands) {
            if (this.commands[command.name])
                this.commands[command.name].modules.push(module)
            else
                this.commands[command.name] = {
                    command: command,
                    modules: [module]
                }
        }

        this._refresh_commands()
    }

    remove(module) {
        if (!this.modules[module.name])
            return // already removed

        delete this.modules[module.name]

        for (const [key, command] of Object.entries(this.commands)) {
            if (command.modules.indexOf(module) !== -1)
                command.modules.splice(command.modules.indexOf(module), 1)
            if (command.modules.length === 0)
                delete this.commands[key]
        }

        this._refresh_commands()
    }

    _refresh_commands() {
        if (this.refresh_timer)
            clearInterval(this.refresh_timer)

        const cm = this

        this.refresh_timer = setTimeout(() => {
            (async () => {
                await DI.get().set_slash_commands(cm.all_commands()).catch(err => console.fatal(`Failed to update discord commands : ${err}`))
            })().catch(err => console.error(`Failed to update discord commands : ${err}`))
        }, 100)
    }

    all_commands() {
        let commands = []
        for (const [_, value] of Object.entries(this.commands))
            commands.push(value.command)
        return commands
    }

    find(command_name) {
        const command = this.commands[command_name]
        return command ? command.command : null
    }

    execute_command(command) {
        const found_command = this.commands[command.source_command().name]
        if (found_command)
            for (const module of found_command.modules)
                if (module.server_interaction)
                    (async () => {
                        module.server_interaction(command)
                            .catch(err => console.error(`Failed to call 'server_interaction()' on module ${module.name} :\n${err}`))
                    })()

    }

    button_pressed(interaction) {
        for (const module of found_command.modules)
            if (module.server_interaction)
                (async () => {
                    module.server_interaction(command)
                        .catch(err => console.error(`Failed to call 'server_interaction()' on module ${module.name} :\n${err}`))
                })()

    }

    get_commands(permissions) {
        let commands = []
        for (const [_, command] of Object.entries(this.commands))
            if (command.command.has_permission(permissions))
                commands.push(command.command)
        return commands
    }
}

module.exports = {CommandDispatcher: CommandDispatcher}