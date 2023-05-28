// MODULE UTILITIES
const {CommandInfo} = require("../../utils/interactionBase")
const MODULE_MANAGER = require('../../core/module_manager').get()
const CONFIG = require('../../config').get()
const LOGGER = require('../../utils/logger').get()
const {Embed} = require('../../utils/embed')
const {Message} = require('../../utils/message')
const DI = require('../../utils/discord_interface')
const {Channel} = require("../../utils/channel");
class Module {
    /**
     * @param create_infos contains module infos => {
     *      client: discord_client
     * }
     */
    constructor(create_infos) {
        this.enabled = true // default value is true

        this.client = create_infos.client

        // Command declaration
        this.commands = [
            new CommandInfo('modules_infos', 'Informations sur les modules')
                .set_admin_only(),
            new CommandInfo('set_module_enabled', 'Active ou desactive un module')
                .add_text_option('nom', 'nom du module')
                .add_bool_option('enabled', 'Nouvel etat du module')
                .set_admin_only(),
            new CommandInfo('load_module', 'Charge un module')
                .add_text_option('nom', 'nom du module')
                .set_admin_only(),
            new CommandInfo('unload_module', 'Decharge un module')
                .add_text_option('nom', 'nom du module')
                .set_admin_only(),
            new CommandInfo('update', 'Vérifie les mises à jour et les effectue le cas échéant')
                .set_admin_only(),
            new CommandInfo('restart', 'Fait redémarrer le bot')
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
            if (level === 'E' || level === 'F') {
                new Message()
                    .set_text('A BOBO ' + CONFIG.SERVICE_ROLE + ' !!! :(')
                    .set_channel(new Channel().set_id(CONFIG.LOG_CHANNEL_ID))
                    .add_embed(new Embed()
                        .set_title(level === 'E' ? 'Error' : 'Fatal')
                        .set_description('```log\n ' + message + '\n```'))
                    .send()
                    .catch(err => console.fatal(`failed to send error message ${err}`))
            }
        })
        DI.get().check_permissions_validity().catch(err => console.fatal(`Failed to check permissions validity : ${err}`))
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
     * // When command is executed
     * @param command {InteractionBase}
     * @return {Promise<void>}
     */
    async server_interaction(command) {
        if (command.match('modules_infos')) {
            const embed = new Embed()
                .set_title('modules')
                .set_description('liste des modules disponibles')

            for (const module of MODULE_MANAGER.all_modules_info()) {
                embed.add_field(module.name, `chargé : ${module.loaded ? ':white_check_mark:' : ':x:'}\tactivé : ${module.enabled ? ':white_check_mark:' : ':x:'}`)
            }

            command.reply(
                new Message()
                    .add_embed(embed)
                    .set_client_only()
            )
        }
        if (command.match('set_module_enabled')) {
            if (command.read('enabled') === true)
                MODULE_MANAGER.start(command.read('nom'))
            else
                MODULE_MANAGER.stop(command.read('nom'))
            command.skip()
        }
        if (command.match('load_module')) {
            MODULE_MANAGER.load_module(command.read('nom'))
            command.skip()
        }
        if (command.match('unload_module')) {
            MODULE_MANAGER.unload_module(command.read('nom'))
            command.skip()
        }
        if (command.match('update')) {
            DI.get().check_updates()
                .then(res => {
                    if (res.upToDate) {
                        command.reply(new Message()
                            .set_text(`Je suis à jour ! (version ${res.currentVersion})`)
                            .set_client_only())
                    } else {
                        command.reply(new Message()
                            .set_text(`Mise à jour disponible, Redémarage en cours ! (${res.currentVersion} => ${res.remoteVersion})`)
                            .set_client_only())
                            .then(_ => {
                                console.warning("restarting for update...")
                                process.exit(0)
                            })
                    }
                })
                .catch(err => {
                    command.reply(new Message().set_text(`Impossible de vérifier les mises à jour : ${err}`).set_client_only())
                })
        }
        if (command.match('restart')) {
            command.reply(new Message()
                .set_text(`Redémarrage en cours...`)
                .set_client_only())
                .then(_ => {
                    console.warning("restarting...")
                    process.exit(0)
                })
        }
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
}

module.exports = {Module}