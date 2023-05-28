// MODULE HISTORY
const CONFIG = require("../../config").get()
const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')
const {Channel} = require("../../utils/channel");

class Module {
    constructor(create_infos) {
        this.enabled = true // default value is true

        this.client = create_infos.client
    }

    /**
     * @param old_message {Message}
     * @param new_message {Message}
     * @return {Promise<void>}
     */
    async server_message_updated(old_message, new_message) {
        const author = await old_message.author()
        console.info(`Message updated [${await author.full_name()}] :\n${await old_message.text()}\nto\n${await new_message.text()}`)

        const old_text = await old_message.text()
        const new_text = await old_message.text()

        await new Message().set_channel(new Channel().set_id(CONFIG.LOG_CHANNEL_ID))
            .add_embed(
                new Embed()
                    .set_title(`@${await author.full_name()} (${author.id()})`)
                    .set_description('Message modifié :')
                    .add_field('ancien', old_text.length <= 0 ? '[Message vide]' : old_text.substring(0, 1024))
                    .add_field('nouveau', new_text.length <= 0 ? '[Message vide]' : new_text.substring(0, 1024))
            ).send().catch(err => console.fatal(`failed to send log message : ${err}`, err))
    }

    async server_message_delete(message) {

        const author = await message.author()
        console.info(`Message deleted [${await author.full_name()}] :\n${message._text}`)
        await new Message().set_channel(new Channel().set_id(CONFIG.LOG_CHANNEL_ID))
            .add_embed(
                new Embed()
                    .set_title(`@${await author.full_name()} (${author.id()})`)
                    .set_description('Message supprimé :')
                    .add_field(message._text ? 'Contenu' : '[Message vide]', message._text ? message._text.substring(0, 1024) : '[Message sans texte]')
            ).send()
    }
}

module.exports = {Module}