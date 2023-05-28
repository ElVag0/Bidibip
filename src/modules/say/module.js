// MODULE SAY
const {CommandInfo} = require("../../utils/interaction")
const {Message} = require('../../utils/message')

class Module {
    constructor(create_infos) {
        this.client = create_infos.client

        // Command declaration
        this.commands = [
            new CommandInfo('say', 'Fait parler bidibip')
                .add_text_option('message', 'Ce que bidibip dira pour vous', [], true)
                .set_member_only()
        ]
    }

    /**
     * // When command is executed
     * @param command {Interaction}
     * @return {Promise<void>}
     */
    async server_interaction(command) {
        if (command.match('say')) {
            await new Message()
                .set_channel(command.channel())
                .set_text(command.read('message'))
                .send()
            await command.skip()
        }
    }
}

module.exports = {Module}