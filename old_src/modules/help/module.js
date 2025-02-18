// MODULE HELP
const {CommandInfo} = require("../../utils/interactionBase")
const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')
MODULE_MANAGER = require("../../core/module_manager").get()

class Module {
    constructor(create_infos) {
        this.client = create_infos.client

        this.commands = [
            new CommandInfo('help', 'Voir la liste des commandes disponibles',
                async (command) => {
                    const commands = MODULE_MANAGER.event_manager().get_commands(command.context_permissions())
                    if (!commands)
                        return

                    const embed = new Embed()
                        .set_title('Aide de Bidibip')
                        .set_color('#98eb34')
                        .set_description('liste des commandes disponibles :')
                    for (const command of commands)
                        embed.add_field(command.name, command.description)

                    const message = new Message()
                        .add_embed(embed)
                        .set_client_only()

                    await command.reply(message)
                })]
    }
}

module.exports = {Module}