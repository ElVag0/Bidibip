const DI = require("../utils/discord_interface");

class CommandDispatcher {
    constructor() {
        this.modules = {}
        this.commands = {}
    }

    add(module) {
        if (this.modules[module.name])
            return // already added

        if (!module.commands)
            return // doesn't implement old

        this.modules[module.name] = {module: module, command: module.commands}

        for (const command of module.commands) {
            command.module = module
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

    get_commands(permissions) {
        let commands = []
        for (const [_, command] of Object.entries(this.commands))
            if (command.command.has_permission(permissions))
                commands.push(command.command)
        return commands
    }
}

module.exports = {CommandDispatcher: CommandDispatcher}