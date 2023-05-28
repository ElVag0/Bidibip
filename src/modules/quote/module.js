// MODULE QUOTE
const {CommandInfo} = require("../../utils/interactionBase")
const {resolve} = require('path')
const CONFIG = require('../../config').get()
const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')

const fs = require('fs')
const {User} = require("../../utils/user");
const {Channel} = require("../../utils/channel");

class Module {
    constructor(create_infos) {
        this.client = create_infos.client

        this.commands = [
            new CommandInfo('quote', 'Envoyer une citation de la personne choisie')
                .add_user_option('utilisateur', 'personne choisie')
                .set_member_only(),
            new CommandInfo('add_quote', 'Ajoute une citation à la base de donnée')
                .add_text_option('message', 'id du message à citer')
                .set_member_only()
        ]

        try {
            const data = fs.readFileSync(CONFIG.SAVE_DIR + '/quotes/quotes.json', 'utf8')
            this.quotes = JSON.parse(data)
        } catch (err) {
            this.quotes = {}
        }
    }

    /**
     * // When command is executed
     * @param command {InteractionBase}
     * @return {Promise<void>}
     */
    async server_interaction(command) {
        if (command.match('quote')) {
            const quote = this.quotes[command.read('utilisateur')]
            if (!quote) {
                await command.reply(new Message().set_text(`Je n'ai pas de citation pour cet utilisateur... Mais je suis sûr qu'il est très cool !`).set_client_only())
                return
            }

            const selected_quote = quote[Math.floor(Math.random() * quote.length)]
            if (selected_quote.text.length === 0) {
                await command.reply(new Message().set_text(`La citation '${selected_quote.id}' n\'est pas valide... Un administrateur va passer pour la supprimer !`))
                console.warning(`there is an empty quote in the database : ${selected_quote.id}`)
                return
            }
            const author = await command.author()
            await new Message()
                .set_channel(command.channel())
                .add_embed(new Embed()
                    .set_title(selected_quote.text)
                    .set_description(`*${await new User().set_id(command.read('utilisateur')).name()}* - Demandé par ${await author.name()}`))
                .send()
            await command.skip()
        }

        if (command.match('add_quote')) {
            try {
                let channel = await command.channel()
                let message_id = command.read('message')

                // We sent a full link
                if (message_id.includes('/'))
                {
                    const url = message_id.split('/')
                    channel = new Channel().set_id(url[url.length - 2])
                    message_id = url[url.length - 1]
                }



                const message = new Message().set_channel(channel).set_id(message_id)
                let text = await message.text()
                if (!text || text.length === 0) {
                    command.reply(new Message().set_text('Ce message ne peut pas être ajouté comme citation').set_client_only())
                        .catch(err => console.fatal(`Failed to reply : ${err}`))
                    return
                }

                const author = await message.author()
                if (!this.quotes[author.id()])
                    this.quotes[author.id()] = [{id: message.id(), text: await message.text()}]
                else {
                    for (const quote of this.quotes[author.id()]) {
                        if (quote.id === message.id()) {
                            command.reply(new Message().set_text('Message déjà en base de donnée !').set_client_only())
                                .catch(err => console.fatal(`Failed to reply : ${err}`))
                            return
                        }
                    }
                    this.quotes[author.id()].push({id: message.id(), text: await message.text()})
                }

                if (!fs.existsSync(CONFIG.SAVE_DIR + '/quotes'))
                    fs.mkdirSync(CONFIG.SAVE_DIR + '/quotes', {recursive: true})
                fs.writeFile(CONFIG.SAVE_DIR + '/quotes/quotes.json', JSON.stringify(this.quotes), 'utf8', err => {
                    if (err) {
                        console.fatal(`failed to save quotes : ${err}`)
                        command.skip()
                    } else {
                        command.reply(new Message().set_text('Citation ajoutée à la base de donnée !'))
                        console.info(`Saved quotes to file ${CONFIG.SAVE_DIR + '/quotes/quotes.json'}`)
                    }
                })
            } catch (err) {
                console.warning(`failed to get message : ${err}`)
                await command.reply(new Message().set_text(`Je n'ai pas trouvé le message :(`).set_client_only())
                    .catch(err => console.fatal(`Failed to reply : ${err}`))
            }
        }
    }
}

module.exports = {Module}