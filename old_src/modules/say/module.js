// MODULE SAY
const {CommandInfo} = require("../../utils/interactionBase")
const {Message} = require('../../utils/message')
const {json_to_message} = require("../../utils/json_to_message");
const {Embed} = require("../../utils/embed");

class Module {
    constructor(create_infos) {
        this.client = create_infos.client

        // Command declaration
        this.commands = [
            new CommandInfo('say', 'Fait parler bidibip', this.say)
                .add_text_option('message', 'Ce que bidibip dira pour vous', [], false)
                .add_file_option('json', 'Message formaté au format JSON (documentation en cours)', false)
                .set_member_only()
        ]
    }

    /**
     * // When command is executed
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async say(command) {

        if (command.read('json')) {
            json_to_message(await command.read('json').get_content())
                .then(async messages => {
                    if (messages.length > 8) {
                        await command.reply(new Message().set_client_only().set_text("Tu peux envoyer un maximum de 8 messages à la fois"))
                        return
                    }
                    for (const message of messages) {
                        message.set_channel(command.channel())
                        await message.send()
                            .catch(err => console.fatal(`Failed to send message : ${err}`))
                    }
                    await command.skip()
                })
                .catch(async error => {
                    command.reply(
                        new Message()
                            .set_client_only()
                            .set_text('Echec de la conversion du message')
                            .add_embed(new Embed().set_title("Raison").set_description(`${error}`)))
                })
        } else if (command.read('message')) {
            await new Message()
                .set_channel(command.channel())
                .set_text(command.read('message'))
                .send()
            await command.skip()
        } else await command.skip()
    }
}

module.exports = {Module}