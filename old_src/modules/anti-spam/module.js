const {ModuleBase} = require("../../utils/module_base");
const {Collection} = require("discord.js");
const {Channel} = require("../../utils/channel");
const {Message} = require("../../utils/message");
const {Embed} = require("../../utils/embed");
const {InteractionRow} = require("../../utils/interaction_row");
const {Button} = require("../../utils/button");
const {User} = require("../../utils/user");

class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos)
        this.messages = new Collection()

        for (const key of Object.values(this.module_config)) {
            this.bind_user(key)
        }
    }

    /**
     * @param message {Message}
     * @return {Promise<void>}
     */
    async server_message(message) {
        await this.flush_old_messages()
        const author = await message.author()
        const current_message = this.messages[author.id()]
        if (current_message && await current_message.content[0].is_valid()) {
            if (await current_message.content[0].text() === await message.text()) {
                for (const old of current_message.content) // skip if message have been posted in the same channel
                    if (old.channel().id() === message.channel().id())
                        return
                const elapsed = new Date() - current_message.time // ms
                current_message.content.push(message)
                if (elapsed < 60000 && current_message.content.length > 3) // 1mn cooldown
                {
                    await author.add_role(this.app_config.MUTE_ROLE_ID)
                    this.module_config[author.id()] = {author: author.id()}
                    this.save_config()
                    await new Message()
                        .set_channel(new Channel().set_id(this.app_config.ADMIN_CHANNEL))
                        .set_text(` Spam potentiel de ${author.mention()} (${message.url()}) <@&${this.app_config.ADMIN_ROLE_ID}> !!!`)
                        .add_embed(new Embed()
                            .set_title("Contenu")
                            .set_color('#FF0000')
                            .set_description(await current_message.content[0].text()))
                        .add_interaction_row(new InteractionRow()
                            .add_button(
                                new Button()
                                    .set_id('kick')
                                    .set_label('Kick')
                                    .set_type(Button.Danger))
                            .add_button(
                                new Button()
                                    .set_id('pardon')
                                    .set_label('Pardonner')
                                    .set_type(Button.Success)))
                        .send()
                        .then(result => {
                            this.module_config[author.id()].admin_message = result.channel().id() + "/" + result.id()
                        })
                        .catch(err => console.error(`Failed to send message to admin channel : ${err}`))

                    await new Message()
                        .set_channel(message.channel())
                        .set_text(`${author.mention()} J'ai l'impression que tu nous spam !`)
                        .add_embed(new Embed()
                            .set_color('#FF0000')
                            .set_title("Confirme que tu es un humain pour continuer à parler sur le serveur."))
                        .add_interaction_row(new InteractionRow()
                            .add_button(new Button()
                                .set_id('human')
                                .set_label('Je suis un humain')
                                .set_type(Button.Success)))
                        .send()
                        .then(async result => {
                            this.module_config[author.id()].message = result.channel().id() + "/" + result.id()
                        })
                        .catch(err => console.error(`Failed to send message to user : ${err}`))

                    this.save_config()
                    await this.bind_user(this.module_config[author.id()])

                    for (const message of current_message.content)
                        if (await message.is_valid())
                            message.delete()

                    delete this.messages[author.id()]
                }

                return
            }
        }

        this.messages[author.id()] = {
            time: new Date(),
            content: [message]
        }
    }

    async bind_user(key) {
        const author = new User().set_id(key.author)
        const message = new Message().set_channel(new Channel().set_id(key.message.split("/")[0])).set_id(key.message.split("/")[1])
        if (await message.is_valid()) {
            this.bind_button(message, async (button_interaction) => {
                const button_author = await button_interaction.author()
                if (button_author.id() === author.id()) {
                    await author.remove_role(this.app_config.MUTE_ROLE_ID)
                    delete this.module_config[author.id()]
                    await button_interaction.reply(new Message().set_text("Tu es de nouveau autorisé à parler !").set_client_only())
                    await message.delete()
                } else
                    await button_interaction.reply(new Message().set_text("Ce n'est pas à toi de répondre").set_client_only())
            })
        }
        if (!key.admin_message)
            return
        const admin_message = new Message().set_channel(new Channel().set_id(key.admin_message.split("/")[0])).set_id(key.admin_message.split("/")[1])
        if (await admin_message.is_valid()) {
            this.bind_button(admin_message, async (admin_button) => {
                if (admin_button.button_id() === 'kick') {
                    await author.kick(`Anti-spam confirmé`)
                    if (await message.is_valid())
                        await message.delete()
                    admin_message.clear_interactions();
                    admin_message.add_interaction_row(new InteractionRow().add_button(new Button("").set_enabled(false).set_id("kicked").set_label(`Dégagé par par ${await admin_button.author().name()} !`).set_type(Button.Danger)))
                    await admin_message.update(admin_message);
                } else {
                    await author.remove_role(this.app_config.MUTE_ROLE_ID)
                    if (await message.is_valid())
                        await message.delete()
                    admin_message.clear_interactions();
                    admin_message.add_interaction_row(new InteractionRow().add_button(new Button("").set_enabled(false).set_id("pardon").set_label(`Pardonné par ${await admin_button.author().name()} !`).set_type(Button.Success)))
                    await admin_message.update(admin_message);
                }
                delete this.module_config[author.id()]
                this.save_config()
                await admin_button.skip();
            })
        }
    }

    async flush_old_messages() {
        for (const [user_id, last_message] of this.messages) {
            const elapsed = new Date() - last_message.time // ms
            if (elapsed > 60000) // 1mn cooldown
                this.messages.delete(user_id)
        }
    }
}

module.exports = {Module}