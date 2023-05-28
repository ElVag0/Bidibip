// MODULE HELP
const {CommandInfo} = require("../../utils/interactionBase")
const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')
const fs = require("fs");
const CONFIG = require('../../config')
const {InteractionRow} = require("../../utils/interaction_row");
const {Button} = require("../../utils/button");
const {Channel} = require("../../utils/channel");
const MODULE_MANAGER = require("../../core/module_manager")

const CURRENT_APPLICATIONS = {}
let APPLICATIONS = {}


function save_applications() {

    if (!fs.existsSync(CONFIG.get().SAVE_DIR + '/admin-application/'))
        fs.mkdirSync(CONFIG.get().SAVE_DIR + '/admin-application/', {recursive: true})
    fs.writeFile(CONFIG.get().SAVE_DIR + '/admin-application/applications.json', JSON.stringify(APPLICATIONS, (key, value) => typeof value === 'bigint' ? value.toString() : value), 'utf8', err => {
        if (err) {
            console.fatal(`failed to save applications : ${err}`)
        } else {
            console.info(`Saved applications to file ${CONFIG.get().SAVE_DIR + '/admin-application/applications.json'}`)
        }
    })
}

class Module {
    constructor(create_infos) {
        this.enabled = false // disabled by default

        this.client = create_infos.client

        this.commands = [
            new CommandInfo('candidature', 'Faire une proposition de candidature à la modération')
                .add_text_option('que-fais-tu-ici', 'A quoi postules-tu ?')
                .add_text_option('ambitions', 'Quelles sont tes ambitions en tant que modérateur')
                .add_text_option('pourquoi-toi', 'Pourquoi devrait-on te choisir toi plutôt que quelqu\'un d\'autre')
                .add_text_option('autre', 'si tu as d\'autres remarques à faire en particulier', [], false)
                .set_member_only(),
            new CommandInfo('candidatures', 'Voir la liste des candidatures')
                .set_member_only(),
            new CommandInfo('candidature-de', 'Voir la candidature d\'une personne')
                .add_user_option('utilisateur', 'Nom de l\'utilisateur')
                .set_member_only()
        ]

        try {
            const data = fs.readFileSync(CONFIG.get().SAVE_DIR + '/admin-application/applications.json', 'utf8')
            APPLICATIONS = JSON.parse(data)
        } catch (err) {
            APPLICATIONS = {}
        }
    }

    /**
     * // When command is executed
     * @param command {InteractionBase}
     * @return {Promise<void>}
     */
    async server_interaction(command) {
        if (command.match('candidature')) {
            const author = await command.author()
            const application = {
                id: author.id(),
                author: await author.full_name(),
                what: command.read('que-fais-tu-ici'),
                ambition: command.read('ambitions'),
                why: command.read('pourquoi-toi'),
                other: command.read('autre'),
                thumbnail: await author.profile_picture(),
            }

            const last = APPLICATIONS[author.id()]

            const message = this._format_candidature(application)
                .set_client_only()
                .set_text(last ? ':warning:ATTENTION:warning: Ta nouvelle candidature effacera la précédente' : 'Voici ta candidature')
                .add_interaction_row(
                    new InteractionRow()
                        .add_button(
                            new Button()
                                .set_id('cancel')
                                .set_label('Annuler')
                                .set_type(Button.Danger))
                        .add_button(
                            new Button()
                                .set_id('confirm')
                                .set_label('Envoyer')
                                .set_type(Button.Success)))

            if (last)
                message.add_embed(this._format_candidature(last)._embeds[0].set_title('Ancienne candidature'))

            await command.reply(message)
                .then(interaction => {
                    MODULE_MANAGER.get().bind_button(this, interaction, this.candidature_response)
                    CURRENT_APPLICATIONS[interaction.id()] = {command: command, application: application}
                })
        }
        if (command.match('candidatures')) {
            const embed = new Embed()
                .set_title('Candidatures')
                .set_description('liste des candidatures')
            for (const [_, value] of Object.entries(APPLICATIONS)) {
                embed.add_field(value.author, value.id)
            }

            await command.reply(new Message()
                .add_embed(embed))
        }
        if (command.match('candidature-de')) {
            const author = await command.author()
            const application = APPLICATIONS[author.id()]
            if (application) {
                await command.reply(this
                    ._format_candidature(application)
                    .set_client_only())
            } else
                await command.reply(new Message().set_text('Cet utilisateur n\'a pas posté de candidature').set_client_only())
        }
    }

    /**
     * @param button {ButtonInteraction}
     */
    async candidature_response(button) {

        button.message()._embeds = [button.message()._embeds[0]]

        if (!CURRENT_APPLICATIONS[button.base_id()])
            return
        if (button.button_id() === 'confirm') {
            button.message().set_text('Nouvelle candidature !')
                .clear_interactions()
                .set_channel(new Channel().set_id(CONFIG.get().ADMIN_APPLICATIONS_CHANNEL))
                .send()
                .then(message => {
                    if (!CURRENT_APPLICATIONS[button.base_id()])
                        return

                    console.info('Saved new application : ', CURRENT_APPLICATIONS[button.base_id()].application)
                    const application = JSON.parse(JSON.stringify(CURRENT_APPLICATIONS[button.base_id()].application, (key, value) => typeof value === 'bigint' ? value.toString() : value))
                    APPLICATIONS[application.id] = application
                    save_applications()

                    CURRENT_APPLICATIONS[button.base_id()].command.edit_reply(new Message()
                        .set_text(`Ta candidature a bien été publiée : https://discord.com/channels/${CONFIG.get().SERVER_ID}/${message.channel().id()}/${message.id()}`))
                        .catch(err => console.fatal(`failed to edit reply : ${err}`))
                    delete CURRENT_APPLICATIONS[button.base_id()]
                })
        } else {
            await CURRENT_APPLICATIONS[button.base_id()].command.delete_reply()
            delete CURRENT_APPLICATIONS[button.base_id()]
        }
    }

    _format_candidature(application) {
        const embed = new Embed()
            .set_title(`Candidature de @${application.author}`)
            .set_description(application.what)
            .set_thumbnail(application.thumbnail)
            .add_field('Ambitions', application.ambition)
            .add_field('Pourquoi ?', application.why)

        if (application.other)
            embed.add_field('Autre', application.other)

        return new Message().add_embed(embed)
    }
}

module
    .exports = {Module}