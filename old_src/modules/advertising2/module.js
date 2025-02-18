// MODULE ADVERTISING2
const {CommandInfo} = require("../../utils/interactionBase")
const {Message} = require('../../utils/message')
const {Channel} = require("../../utils/channel");
const {ModuleBase} = require("../../utils/module_base");
const {Thread} = require("../../utils/thread.js");
const {InteractionRow} = require("../../utils/interaction_row");
const {Button} = require("../../utils/button");
const {AdvertisingSetup} = require("./setup");

class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos)
        this.client = create_infos.client

        this.commands = [
            new CommandInfo('annonce', 'Créer une annonce d\'offre ou de recherche d\'emploi', this.annonce)
                .set_member_only(),
        ];

        (async () => {
            if (this.module_config.registered_threads)
                for (const thread of this.module_config.registered_threads) {
                    const channel = new Thread().set_id(thread)
                    if (await channel.is_valid()) {
                        await channel.delete();
                    }
                }
        })().catch((error)=> {console.error(`Failed to cleanup old thread : ${error}`)})

        this.module_config.registered_threads = []
        this.save_config();
        this.advertising_setups = {};
    }

    /**
     * @param command_interaction {CommandInteraction}
     * @return {Promise<void>}
     */
    async annonce(command_interaction) {
        let thread_channel = new Channel().set_id(command_interaction.channel().id());
        if (await thread_channel.type() === Channel.TypeThread || await thread_channel.type() === Channel.TypePrivateThread) {
            thread_channel = new Channel().set_id(this.app_config.TICKET_CHANNEL_ID);
        }
        const user = await command_interaction.author();
        const thread = await new Thread().set_id(await thread_channel.create_thread(`Annonce de ${await user.name()}`, true)
            .catch(err => console.fatal(`Failed to create thread for announcement : ${err}`)));

        await command_interaction.reply(new Message().set_client_only().set_text(`Bien reçu, la suite se passe ici :arrow_right: ${thread.url()}`));
        const message = await thread.send(new Message()
            .set_text(`# Bienvenue dans le formulaire de création d'annonce ${(user.mention())} !`)
            .add_interaction_row(new InteractionRow()
                .add_button(new Button().set_id("restart").set_label("♻ Recommencer").set_type(Button.Secondary))
                .add_button(new Button().set_id("cancel").set_label("❌ Annuler").set_type(Button.Danger))));

        new AdvertisingSetup(this, thread, user);
        this.module_config.registered_threads.push(thread.id())
        this.save_config();

        this.bind_button(message, async (interaction) => {
            if (interaction.button_id() === 'restart') {
                await interaction.skip();
                new AdvertisingSetup(this, thread, user);
            }
            else if (interaction.button_id() === 'cancel') {
                await thread.delete();
            }
        });
    }

    /**
     * @param message {Message}
     * @return {Promise<void>}
     */
    async server_message(message) {
        const channelId = (await message.channel()).id();
        if (this.advertising_setups[channelId])
            await this.advertising_setups[channelId].received_message(message);
    }
}

module.exports = {Module}