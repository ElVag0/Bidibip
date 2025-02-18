const fs = require("fs");
const CONFIG = require("../config");
const MODULE_MANAGER = require("../core/module_manager");
const {InteractionButton} = require("../utils/interactionBase")
const {InteractionRow} = require("./interaction_row");
const {Button} = require("./button");
const {Message} = require("./message");

class ModuleBase {
    constructor(create_infos) {
        this.enabled = true
        this.name = create_infos.name
        this.module_config = {}
        this.commands = []

        this.load_config()
        this.app_config = CONFIG.get()
    }

    new_command(command) {
        this.commands.push(command)
    }

    load_config(default_object = {}) {
        const path = `${CONFIG.get().SAVE_DIR}/config/${this.name}.json`
        if (!fs.existsSync(path)) {
            this.module_config = default_object
            return this.module_config
        }
        try {
            this.module_config = JSON.parse(fs.readFileSync(path, 'utf8'))
            return this.module_config
        } catch (err) {
            console.error(`Failed to load config for module '${this.name}' : ${err}`)
            this.module_config = default_object
            return this.module_config
        }
    }

    save_config(config_overwrite = null) {
        const path = `${CONFIG.get().SAVE_DIR}/config/${this.name}.json`
        const object = config_overwrite ? config_overwrite : this.module_config
        try {
            if (!fs.existsSync(`${CONFIG.get().SAVE_DIR}/config/`))
                fs.mkdirSync(`${CONFIG.get().SAVE_DIR}/config/`, {recursive: true})

            fs.writeFileSync(
                path,
                JSON.stringify(object,
                    (key, value) => typeof value === 'bigint' ? value.toString() : value),
                'utf8')
            console.info(`Saved config of module '${this.name}'`)
        } catch (err) {
            console.fatal(`Failed to load config for module '${this.name}' : ${err}`)
        }
    }

    /**
     * @callback ButtonCallback
     * @param {InteractionButton} button_interaction
     * @return {Promise<void>}
     */

    /**
     * Bind a callback to given button
     * @param button {Message|InteractionBase} Message or interaction that contains the button
     * @param callback {ButtonCallback} Method called when button is clicked
     */
    bind_button(button, callback) {
        MODULE_MANAGER.get().bind_button(this, button, callback)
    }

    /**
     * @param interaction {CommandInteraction}
     * @param message {Message}
     * @return {Promise<boolean>}
     */
    async ask_user_confirmation(interaction, message) {
        const message_copy = Object.assign(new Message(), message)

        message_copy.set_client_only()
            .add_interaction_row(
                new InteractionRow()
                    .add_button(
                        new Button()
                            .set_id('ask-user-cancel')
                            .set_label('Annuler')
                            .set_type(Button.Danger))
                    .add_button(
                        new Button()
                            .set_id('ask-user-confirm')
                            .set_label('Envoyer')
                            .set_type(Button.Success)))

        const out_interaction = await interaction.reply(message_copy)

        return new Promise((resolve, reject) => {
            this.bind_button(out_interaction, async (button_interaction) => {
                if (button_interaction.button_id() === 'ask-user-confirm')
                    resolve(true)
                else if (button_interaction.button_id() === 'ask-user-cancel')
                    resolve(false)
                else reject(button_interaction)
                return false
            })
        })
    }
}

module.exports = {ModuleBase}