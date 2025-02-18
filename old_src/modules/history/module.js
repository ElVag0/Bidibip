// MODULE HISTORY
const CONFIG = require("../../config").get()
const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')
const {Channel} = require("../../utils/channel");
const {ModuleBase} = require("../../utils/module_base");
const humanizeDuration = require("humanize-duration");

class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos)
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
        const new_text = await new_message.text()

        await new Message().set_channel(new Channel().set_id(CONFIG.LOG_CHANNEL_ID))
            .add_embed(
                new Embed()
                    .set_title(`@${await author.full_name()} (${author.id()})`)
                    .set_description('Message modifié :')
                    .set_color('#FFFF00')
                    .add_field('ancien', old_text.length <= 0 ? '[Message vide]' : old_text.substring(0, 1024))
                    .add_field('nouveau', new_text.length <= 0 ? '[Message vide]' : new_text.substring(0, 1024))
            ).send().catch(err => console.fatal(`failed to send log message : ${err}`, err))
    }

    async server_message_delete(message, from, by) {

        message.author().then(async author => {
            console.info(`Message from ${from} deleted [${by}] :\n${message._text}`)
            await new Message().set_channel(new Channel().set_id(CONFIG.LOG_CHANNEL_ID))
                .add_embed(
                    new Embed()
                        .set_title(`@${await author.full_name()} (${author.id()})`)
                        .set_description(`Message de ${from} supprimé par ${by} :`)
                        .set_color('#FF0000')
                        .add_field(message._text ? 'Contenu' : '[Message vide]', message._text ? message._text.substring(0, 1024) : '[Message sans texte]')
                ).send()
        }).catch(async _ => {
            console.info(`Unknown message from ${from} deleted by ${by} : ${message.url()}`)
            await new Message().set_channel(new Channel().set_id(CONFIG.LOG_CHANNEL_ID))
                .add_embed(
                    new Embed()
                        .set_title(`Message de ${from} supprimé par ${by}`)
                        .set_description(`Ancien message supprimé : ${message.url()}`)
                        .set_color('#FF0000')
                ).send()
        })
    }

    /**
     *
     * @param target {User}
     * @param executor {User}
     * @param reason {String}
     * @param until {String}
     * @return {Promise<void>}
     */
    async user_excluded(target, executor, reason, until) {
        if (until) {
            const date = Date.parse(until)
            const delay = new Date(date - Date.now())

            new Message()
                .set_text(`<@&${this.app_config.ADMIN_ROLE_ID}>`)
                .add_embed(
                    new Embed()
                        .set_title(`${await target.full_name()} a été exclu par ${await executor.full_name()}`)
                        .set_description(reason ? reason : "Aucune raison fournie")
                        .add_field("Durée", `${humanizeDuration(delay.getTime(), {language: "fr"})}`, true))
                .set_channel(new Channel().set_id(this.app_config.WELCOME_CHANNEL)).send()
                .catch(err => console.fatal(`Failed to send exclusion message : ${err}`))
        } else {
            new Message()
                .set_text(`<@&${this.app_config.ADMIN_ROLE_ID}>`)
                .add_embed(
                    new Embed()
                        .set_title(`L'exclusion de ${await target.full_name()} a été révoquée par ${await executor.full_name()}`))
                .set_channel(new Channel().set_id(this.app_config.WELCOME_CHANNEL)).send()
                .catch(err => console.fatal(`Failed to send exclusion message : ${err}`))
        }
    }

    /**
     *
     * @param target {User}
     * @param executor {User}
     * @param reason {String}
     * @return {Promise<void>}
     */
    async user_kicked(target, executor, reason) {
        new Message()
            .set_text(`<@&${this.app_config.ADMIN_ROLE_ID}>`)
            .add_embed(
                new Embed()
                    .set_title(`${await target.full_name()} a été kick par ${await executor.full_name()}`)
                    .set_description(reason ? reason : "Aucune raison fournie"))
            .set_channel(new Channel().set_id(this.app_config.WELCOME_CHANNEL)).send()
            .catch(err => console.fatal(`Failed to send kick message : ${err}`))
    }

    /**
     *
     * @param target {User}
     * @param executor {User}
     * @param reason {String}
     * @return {Promise<void>}
     */
    async user_banned(target, executor, reason) {
        new Message()
            .set_text(`<@&${this.app_config.ADMIN_ROLE_ID}>`)
            .add_embed(
                new Embed()
                    .set_title(`${await target.full_name()} a été banni par ${await executor.full_name()}`)
                    .set_description(reason ? reason : "Aucune raison fournie"))
            .set_channel(new Channel().set_id(this.app_config.WELCOME_CHANNEL)).send()
            .catch(err => console.fatal(`Failed to send ban message : ${err}`))
    }

    /**
     *
     * @param target {User}
     * @param executor {User}
     * @param reason {String}
     * @return {Promise<void>}
     */
    async user_unbanned(target, executor, reason) {
        new Message()
            .set_text(`<@&${this.app_config.ADMIN_ROLE_ID}>`)
            .add_embed(
                new Embed()
                    .set_title(`Le bannissement de ${await target.full_name()} a été révoqué par ${await executor.full_name()}`))
            .set_channel(new Channel().set_id(this.app_config.WELCOME_CHANNEL)).send()
            .catch(err => console.fatal(`Failed to send unban message : ${err}`))
    }
}

module.exports = {Module}