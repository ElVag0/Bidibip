const {ModuleBase} = require("../../utils/module_base");
const {Channel} = require("../../utils/channel");
const {Message} = require("../../utils/message");
const {Embed} = require("../../utils/embed");
const {json_to_message} = require("../../utils/json_to_message");


class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos);

        if (this.module_config.approval) {
            const approval_message = new Message()
                .set_channel(new Channel().set_id(this.app_config.REGLEMENT_CHANNEL_ID))
                .set_id(this.module_config.approval)

            approval_message.is_valid().then(async result => {
                    if (result)
                        await this.bind_approval(approval_message)
                            .catch(err => console.error(`Failed to bind approval button : ${err}`))
                    else
                        console.error('No approval message is configured')
                }
            )
        }
        else
            console.error('No approval message is configured')
    }


    /**
     * @param message {Message}
     * @return {Promise<void>}
     */
    async server_message(message) {
        if (message.channel().id() === this.app_config.REGLEMENT_CHANNEL_ID && message.attachments().length !== 0 && message.attachments()[0].name() === "reglement.json") {

            json_to_message(await message.attachments()[0].get_content())
                .then(async messages => {
                    for (const to_send of messages) {
                        await to_send.set_channel(new Channel().set_id(this.app_config.REGLEMENT_CHANNEL_ID)).send()
                            .then(async sent => {
                                if (await sent.get_button_by_id("approval")) {
                                    this.module_config.approval = sent.id()
                                    this.save_config()
                                    await this.bind_approval(sent)
                                    console.validate("Successfully updated approval message")
                                }
                            })
                            .catch(err => console.fatal(`Failed to send reglement message : ${err}`))
                    }
                })
                .catch(async error => {
                    await (await message.author()).send(new Message()
                        .set_text('Echec de la conversion du message')
                        .add_embed(new Embed().set_title("Raison").set_description(`${error}`)))
                })
        }
    }

    /**
     * @param message {Message}
     * @return {Promise<void>}
     */
    async bind_approval(message) {
        if (await message.get_button_by_id('approval')) {

            let approvalButton = await message.get_button_by_id('approval')
            if (!approvalButton.is_enabled())
                console.fatal("Approval button is disabled ! Please fix it now !");
            this.bind_button(message, async (interaction) => {
                if (interaction.button_id() === 'approval') {
                    await interaction.author().add_role(this.app_config.MEMBER_ROLE_ID)
                    await interaction.reply(new Message().set_text(`Bienvenue sur le serveur ${interaction.author().mention()} ! Tu as pris connaissance du réglement.`).set_client_only())
                } else
                    await interaction.skip()
            })
            console.validate("Successfully bound approval button");
        }
        else {
            console.fatal("Failed to find approval button id");
        }
    }
}

module
    .exports = {Module}