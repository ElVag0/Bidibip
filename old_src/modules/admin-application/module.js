// MODULE HELP
const {CommandInfo} = require("../../utils/interactionBase")
const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')
const {Channel} = require("../../utils/channel")
const {User} = require("../../utils/user")
const {ModuleBase} = require("../../utils/module_base")

class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos)
        this.enabled = false // disabled by default

        this.commands = [
            new CommandInfo('candidature', 'Faire une proposition de candidature à la modération', this.candidature)
                .add_text_option('que-fais-tu-ici', 'A quoi postules-tu ?')
                .add_text_option('ambitions', 'Quelles sont tes ambitions en tant que modérateur')
                .add_text_option('pourquoi-toi', 'Pourquoi devrait-on te choisir toi plutôt que quelqu\'un d\'autre')
                .add_text_option('autre', 'si tu as d\'autres remarques à faire en particulier', [], false)
                .set_member_only(),
            new CommandInfo('candidatures', 'Voir la liste des candidatures', this.candidatures)
                .set_member_only(),
            new CommandInfo('candidature-de', 'Voir la candidature d\'une personne', this.candidature_de)
                .add_user_option('utilisateur', 'Nom de l\'utilisateur')
                .set_member_only()
        ]
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async candidature(command) {
        const author = await command.author()

        const application = {
            id: author.id(),
            what: command.read('que-fais-tu-ici'),
            ambition: command.read('ambitions'),
            why: command.read('pourquoi-toi'),
            other: command.read('autre'),
        }

        const last = this.module_config[author.id()]

        const message = (await this.format_application(application))
            .set_client_only()
            .set_text(last ? ':warning:ATTENTION:warning: Ta nouvelle candidature effacera la précédente' : 'Voici ta candidature')
        if (last)
            message.add_embed((await this.format_application(last)).first_embed().set_title('Ancienne candidature'))

        if (await this.ask_user_confirmation(command, message) === true) {
            (await this.format_application(application))
                .set_channel(new Channel().set_id(this.app_config.ADMIN_APPLICATIONS_CHANNEL))
                .send()
                .then(message => {
                    this.module_config[application.id] = application
                    this.save_config()

                    command.edit_reply(new Message()
                        .set_text(`Ta candidature a bien été publiée : ${message.url()}`))
                        .catch(err => console.fatal(`failed to edit reply : ${err}`))
                })
        } else {
            await command.delete_reply()
        }
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async candidatures(command) {
        const embed = new Embed()
            .set_title('Candidatures')
            .set_color('#34abeb')
            .set_description('liste des candidatures')

        for (const value of Object.values(this.module_config))
            embed.add_field(await new User().set_id(value.id).full_name(), new User().set_id(value.id).mention())

        await command.reply(new Message()
            .set_client_only()
            .add_embed(embed))
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async candidature_de(command) {
        const application = this.module_config[command.read('utilisateur')]
        if (application) {
            await command.reply((await this.format_application(application))
                .set_client_only())
        } else
            await command.reply(new Message().set_text('Cet utilisateur n\'a pas posté de candidature').set_client_only())
    }

    async format_application(application) {
        const user = new User().set_id(application.id)
        const embed = new Embed()
            .set_author(user)
            .set_color('#34abeb')
            .set_description(application.what)
            .set_thumbnail(await user.profile_picture())
            .add_field('Ambitions', application.ambition)
            .add_field('Pourquoi ?', application.why)

        if (application.other)
            embed.add_field('Autre', application.other)

        return new Message().add_embed(embed)
    }
}

module.exports = {Module}