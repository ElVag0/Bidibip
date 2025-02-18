// MODULE HISTORY
const CONFIG = require("../../config").get()
const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')
const {Channel} = require("../../utils/channel");
const {ModuleBase} = require("../../utils/module_base");
const humanizeDuration = require("humanize-duration");
const {CommandInfo} = require("../../utils/interactionBase");
const {User} = require("../../utils/user");

class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos)
        this.enabled = true // default value is true

        this.client = create_infos.client

        this.commands = [
            new CommandInfo('warn', 'Warn une personne', this.warn)
                .set_admin_only()
                .add_user_option("utilisateur", "Utilisateur concerné", true)
                .add_text_option("raison", "Raison du warn", [], true)
                .add_message_option("message", "Message permetant d'établir le contexte du warn", false)
                .add_file_option("attachment", "Screenshot ou fichier", false)
                .add_text_option("action", "Action à faire sur l'utilisateur", ["aucune", "mute-5mn", "mute-1h", "mute-1-jour", "kick", "ban", "ban-delete-messages"], false)
        ];

        if (!this.module_config['warns'])
            this.module_config.warns = {};
    }

    /**
     * @param command_interaction {CommandInteraction}
     * @return {Promise<void>}
     */
    async warn(command_interaction) {

        const user = new User().set_id(await command_interaction.read("utilisateur"))

        if (!user)
            console.fatal("Invalid user")
        const reason = await command_interaction.read("raison")

        const action = await command_interaction.read('action')
        if (action) {
            console.log(action)
            switch (action) {
                case 'aucune':
                    break;
                case 'mute-5mn':
                    await user.exclude(reason, 300)
                    break;
                case 'mute-1h':
                    await user.exclude(reason, 3600)
                    break;
                case 'mute-1-jour':
                    await user.exclude(reason, 86400)
                    break;
                case 'kick':
                    console.log("kickkk")
                    await user.kick(reason)
                    break;
                case 'ban':
                    await user.ban(reason)
                    break;
                case 'ban-delete-messages':
                    await user.ban(reason, 21600) // 6 hours
                    break;
            }
        }

        const embed = new Embed()
            .set_title((action && action !== "aucune") ? action : "Warn")
            .set_description(reason);

        const existing_warns = this.module_config.warns[user.id()];

        const message = await command_interaction.read('message')
        if (message)
            embed.add_field("Message", message.url(), true)

        const OutMessage =
            new Message()
                .set_channel(new Channel().set_id(this.app_config.WARN_CHANNEL))
                .set_text(`Warn de ${user.mention()} par ${command_interaction.author().mention()} <@&${this.app_config.ADMIN_ROLE_ID}>`)
                .add_embed(embed)


        const attachment = await command_interaction.read('attachment')
        if (attachment)
            OutMessage.add_attachment(attachment)

        const sent_message = await OutMessage.send()

        if (!this.module_config.warns[user.id()])
            this.module_config.warns[user.id()] = {warns: []}
        this.module_config.warns[user.id()].warns.push({
            date: Date.now(),
            link: sent_message.url(),
            message: reason,
        })

        if (existing_warns) {
            let string = ""
            for (const warn of existing_warns.warns) {
                string += `${new Date(warn.date).toLocaleDateString('fr-fr', {
                    weekday: "long",
                    year: "numeric",
                    month: "short",
                    day: "numeric"
                })}\t${warn.link} : ${warn.message}\n`
            }
            await new Message().set_channel(new Channel().set_id(this.app_config.WARN_CHANNEL))
                .add_embed(new Embed()
                    .set_title("Récidives")
                    .set_description(string))
                .send()
        }
        await command_interaction.skip();
        this.save_config();
    }
}

module.exports = {Module}