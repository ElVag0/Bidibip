// MODULE QUOTE
const {CommandInfo} = require("../../utils/interactionBase")
const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')
const {User} = require("../../utils/user");
const {Channel} = require("../../utils/channel");
const {ModuleBase} = require("../../utils/module_base");

class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos);

        this.commands = [
            new CommandInfo('quote', 'Envoyer une citation de la personne choisie', this.quote)
                .add_user_option('utilisateur', 'personne choisie')
                .set_member_only(),
            new CommandInfo('add_quote', 'Ajoute une citation à la base de donnée', this.add_quote)
                .add_text_option('message', 'id du message à citer')
                .set_member_only()
        ]
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async quote(command){
        const quote = this.module_config[command.read('utilisateur')]
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

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async add_quote(command){
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
            if (!this.module_config[author.id()])
                this.module_config[author.id()] = [{id: message.id(), text: await message.text()}]
            else {
                for (const quote of this.module_config[author.id()]) {
                    if (quote.id === message.id()) {
                        command.reply(new Message().set_text('Message déjà en base de donnée !').set_client_only())
                            .catch(err => console.fatal(`Failed to reply : ${err}`))
                        return
                    }
                }
                this.module_config[author.id()].push({id: message.id(), text: await message.text()})
            }

            this.save_config()
            await command.reply(new Message().set_text('Citation ajoutée à la base de donnée !'))
        } catch (err) {
            console.warning(`failed to get message : ${err}`)
            await command.reply(new Message().set_text(`Je n'ai pas trouvé le message :(`).set_client_only())
                .catch(err => console.fatal(`Failed to reply : ${err}`))
        }
    }
}

module.exports = {Module}