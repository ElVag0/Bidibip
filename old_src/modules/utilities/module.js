// MODULE UTILITIES
const {CommandInfo} = require("../../utils/interactionBase")
const MODULE_MANAGER = require('../../core/module_manager').get()
const CONFIG = require('../../config').get()
const LOGGER = require('../../utils/logger').get()
const {Embed} = require('../../utils/embed')
const {Message} = require('../../utils/message')
const DI = require('../../utils/discord_interface')
const {Channel} = require("../../utils/channel");
const {ModuleBase} = require("../../utils/module_base");
const fs = require("fs");

class Module extends ModuleBase {
    /**
     * @param create_infos contains module infos => {
     *      client: discord_client
     * }
     */
    constructor(create_infos) {
        super(create_infos);
        this.enabled = true // default value is true

        // Command declaration
        this.commands = [
            new CommandInfo('modules_infos', 'Informations sur les modules', this.modules_infos)
                .set_admin_only(),
            new CommandInfo('set_module_enabled', 'Active ou desactive un module', this.set_module_enabled)
                .add_text_option('nom', 'nom du module')
                .add_bool_option('enabled', 'Nouvel etat du module')
                .set_admin_only(),
            new CommandInfo('load_module', 'Charge un module', this.load_module)
                .add_text_option('nom', 'nom du module')
                .set_admin_only(),
            new CommandInfo('unload_module', 'Decharge un module', this.unload_module)
                .add_text_option('nom', 'nom du module')
                .set_admin_only(),
            new CommandInfo('update', 'Vérifie les mises à jour et les effectue le cas échéant', this.update)
                .set_admin_only(),
            new CommandInfo('restart', 'Fait redémarrer le bot', this.restart)
                .set_admin_only()
        ]

        const update_activity = () => {
            DI.get().get_user_count().then(members => {
                DI.get().set_activity(`${members} Utilisateurs`)
            })
                .catch(err => console.fatal(`failed to set activity : ${err}`))
        }
        update_activity()
        setInterval(update_activity, 3600000)

        LOGGER.bind((level, message) => {
            if (level === 'E' || level === 'F' || level === 'W' || level === 'V') {
                const log =
                    new Message()
                        .set_text(level === 'W' || level === 'V' ? null : 'A BOBO ' + CONFIG.SERVICE_ROLE + ' !!! :(')
                        .set_channel(new Channel().set_id(CONFIG.LOG_CHANNEL_ID))

                if (level === 'E' || level === 'F') {
                    log.add_embed(new Embed()
                        .set_title(level === 'E' ? 'Error' : 'Fatal')
                        .set_color(level === 'E' ? "#FF0000" : "#FF00FF")
                        .set_description('```log\n ' + message.substring(0, 4080) + '\n```'))
                }
                else
                    log.set_text((level === 'W' ? '__**Warning**__\n' : '__**Info**__\n') + message.substring(0, 4080))
                log.send()
                    .catch(err => {
                        LOGGER._delegates = []
                        console.fatal(`failed to send error message ${err}`)
                    })
            }
        })
        DI.get().check_permissions_validity().catch(err => console.fatal(`Failed to check permissions validity : ${err}`))

        try {
            this.messages = JSON.parse(fs.readFileSync(`${__dirname}/messages.json`, 'utf8'))
        } catch (err) {
            console.error(`Failed to load welcome and leave messages : ${err}`)
        }
    }

    async modules_infos(command_interaction) {
        const embed = new Embed()
            .set_title('modules')
            .set_color('#0000FF')
            .set_description('liste des modules disponibles')

        for (const module of MODULE_MANAGER.all_modules_info())
            embed.add_field(module.name, `chargé : ${module.loaded ? ':white_check_mark:' : ':x:'}\tactivé : ${module.enabled ? ':white_check_mark:' : ':x:'}`)

        command_interaction.reply(
            new Message()
                .add_embed(embed)
                .set_client_only()
        )
    }

    /**
     * // When module is started
     * @return {Promise<void>}
     */
    async start() {
    }

    /**
     * When module is stopped
     * @return {Promise<void>}
     */
    async stop() {
    }

    /**
     * @param command_interaction {CommandInteraction}
     * @return {Promise<void>}
     */
    async set_module_enabled(command_interaction) {
        if (command_interaction.read('enabled') === true)
            MODULE_MANAGER.start(command_interaction.read('nom'))
        else
            MODULE_MANAGER.stop(command_interaction.read('nom'))
        await this.modules_infos(command_interaction)
    }

    /**
     * @param command_interaction {CommandInteraction}
     * @return {Promise<void>}
     */
    async load_module(command_interaction) {
        await MODULE_MANAGER.load_module(command_interaction.read('nom'))
        await this.modules_infos(command_interaction)
    }

    /**
     * @param command_interaction {CommandInteraction}
     * @return {Promise<void>}
     */
    async unload_module(command_interaction) {
        MODULE_MANAGER.unload_module(command_interaction.read('nom'))
        await this.modules_infos(command_interaction)
    }

    /**
     * @param command_interaction {CommandInteraction}
     * @return {Promise<void>}
     */
    async update(command_interaction) {
        DI.get().check_updates()
            .then(res => {
                if (res.upToDate) {
                    command_interaction.reply(new Message()
                        .set_text(`Je suis à jour ! (version ${res.currentVersion})`)
                        .set_client_only())
                } else {
                    command_interaction.reply(new Message()
                        .set_text(`Mise à jour disponible, Redémarrage en cours ! (${res.currentVersion} => ${res.remoteVersion})`)
                        .set_client_only())
                        .then(_ => {
                            console.warning("restarting for update...")
                            process.exit(0)
                        })
                }
            })
            .catch(err => {
                command_interaction.reply(new Message().set_text(`Impossible de vérifier les mises à jour : ${err}`).set_client_only())
            })
    }

    /**
     * @param command_interaction {CommandInteraction}
     * @return {Promise<void>}
     */
    async restart(command_interaction) {
        command_interaction.reply(new Message()
            .set_text(`Redémarrage en cours...`)
            .set_client_only())
            .then(_ => {
                console.warning("restarting...")
                process.exit(0)
            })
    }

    /**
     * On received server messages
     * @param message {Message}
     * @return {Promise<void>}
     */
    async server_message(message) {
    }

    /**
     * On update message on server
     * @param old_message {Message}
     * @param new_message {Message}
     * @return {Promise<void>}
     */
    async server_message_updated(old_message, new_message) {
    }

    /**
     * On delete message on server
     * @param message {Message}
     * @return {Promise<void>}
     */
    async server_message_delete(message) {
    }

    /**
     * When user joined the server
     * @param user {User}
     * @return {Promise<void>}
     */
    async user_joined(user) {
        let randomMessage = this.messages.join[Math.floor(Math.random() * this.messages.join.length)];
        randomMessage += "\n> N'oublies pas de lire le {reglement} pour accéder au serveur."
        new Message()
            .set_text(randomMessage.replace(/{user}/g, user.mention())
                .replace(/{reglement}/g, `<#${this.app_config.REGLEMENT_CHANNEL_ID}>`))
            .set_channel(new Channel().set_id(this.app_config.WELCOME_CHANNEL)).send()
            .catch(err => console.fatal(`Failed to send welcome message : ${err}`))
    }

    /**
     * When user leaved the server
     * @param user {User}
     * @return {Promise<void>}
     */
    async user_leaved(user) {
        let randomMessage = this.messages.leave[Math.floor(Math.random() * this.messages.leave.length)];
        new Message()
            .set_text(randomMessage.replace(/{user}/g, user.mention()))
            .set_channel(new Channel().set_id(this.app_config.LOG_CHANNEL_ID)).send()
            .catch(err => console.fatal(`Failed to send leave message : ${err}`))
    }
}

module.exports = {Module}