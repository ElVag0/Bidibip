// MODULE ADVERTISING
const {CommandInfo} = require("../../utils/interaction")
const CONFIG = require('../../config').get()
MODULE_MANAGER = require("../../module_manager").get()

const {Message} = require('../../utils/message')
const {Embed} = require('../../utils/embed')
const {Button} = require('../../utils/button')

class Module {
    constructor(create_infos) {
        this.client = create_infos.client

        this.pending_request = {}

        this.commands = [
            new CommandInfo('paid', 'Ajouter une annonce payante')
                .add_text_option('remuneration', 'Comment le travail sera t-il compensé ?', ['Rémunération', 'Si le jeu fonctionne, partage des revenus', 'Pas de rémunération'])
                .add_text_option('contrat', 'Est-ce un contrat permanent ou contractuel ?', ['Permanent', 'Contractuel'])
                .add_text_option('role', 'Quel rôle recrutes-tu ? (Gameplay developer...)')
                .add_text_option('societe', 'Quel est le nom de l\'entreprise ?')
                .add_text_option('remote', 'Est-ce que le remote est possible ?', ['🌐 Distanciel accepté', '🏣 Presentiel seulement'])
                .add_text_option('responsabilites', 'Liste des responsabilites associes pour ce rôle ?')
                .add_text_option('qualifications', 'Lister les qualifications pour ce rôle.')
                .add_text_option('postuler', 'Comment peut-on postuler ?')
                .add_text_option('localisation', 'Ou est localisé l\'entreprise ?', [], false)
                .add_text_option('duree', 'Durée dans le cas d\'un contrat non Permanent', [], false)
                .set_member_only(),
            new CommandInfo('freelance', 'Ajouter une annonce de freelance')
                .add_text_option('nom', 'Quel est ton nom, ou le nom de ton studio ?')
                .add_text_option('portfolio', 'Entrez l\'url de votre site portfolio (URL requis)')
                .add_text_option('services', 'Quel est la liste des services que tu proposes ?')
                .add_text_option('contact', 'Comment les clients potentiels peuvent-ils vous contacter ?')
                .set_member_only(),
            new CommandInfo('unpaid', 'Ajouter une annonce bénévole')
                .add_text_option('titre', 'Ajoute un titre qui définit clairement ce que tu cherches')
                .add_text_option('description', 'Ajoute une description détaillée du projet et ce dont tu as besoin')
                .add_text_option('contact', 'Comment peut-on te contacter ?')
                .set_member_only(),
        ]
    }

    // When server command is executed
    async server_interaction(command) {

        if (command.match('paid')) {
            const result = this._build_paid(command)

            if (!result.valid) {
                command.reply(result.message)
                return
            }

            command.reply(result.message
                .set_client_only()
                .set_text('Prends le temps de vérifier ton message :')
                .add_interaction_row([
                    new Button()
                        .set_id('cancel')
                        .set_label('Annuler')
                        .set_type(Button.Danger),
                    new Button()
                        .set_id('send')
                        .set_label('Envoyer')
                        .set_type(Button.Success)
                ])).then(id => {
                this.pending_request[id] = {command: command, message: this._build_paid(command).message.set_channel(CONFIG.ADVERTISING_PAID_CHANNEL)}
                MODULE_MANAGER.event_manager().watch_interaction(this, id)
            })
        }
        if (command.match('unpaid')) {
            command.reply(this._build_unpaid(command)
                .set_client_only()
                .set_text('Prends le temps de vérifier ton message :')
                .add_interaction_row([
                    new Button()
                        .set_id('cancel')
                        .set_label('Annuler')
                        .set_type(Button.Danger),
                    new Button()
                        .set_id('send')
                        .set_label('Envoyer')
                        .set_type(Button.Success)
                ])).then(id => {
                this.pending_request[id] = {command: command, message: this._build_unpaid(command).set_channel(CONFIG.ADVERTISING_UNPAID_CHANNEL)}
                MODULE_MANAGER.event_manager().watch_interaction(this, id)
            })
        }
        if (command.match('freelance')) {
            const result = this._build_freelance(command)

            if (!result.valid) {
                command.reply(result.message)
                return
            }

            command.reply(result.message
                .set_client_only()
                .set_text('Prends le temps de vérifier ton message :')
                .add_interaction_row([
                    new Button()
                        .set_id('cancel')
                        .set_label('Annuler')
                        .set_type(Button.Danger),
                    new Button()
                        .set_id('send')
                        .set_label('Envoyer')
                        .set_type(Button.Success)
                ])).then(id => {
                this.pending_request[id] = {command: command, message: this._build_freelance(command).message.set_channel(CONFIG.ADVERTISING_FREELANCE_CHANNEL)}
                MODULE_MANAGER.event_manager().watch_interaction(this, id)
            })
        }
    }

    async receive_interaction(value, id, _message) {
        MODULE_MANAGER.event_manager().release_interaction(this, id)
        const output_message = this.pending_request[id]
        if (value === 'send') {
            output_message.message.send()
            output_message.message.set_channel(CONFIG.SHARED_SHARED_CHANNEL)
                .send()
                .then(message => {
                    output_message.command.edit_reply(new Message()
                        .set_text(`Ton annonce a bien été publiée : https://discord.com/channels/${CONFIG.SERVER_ID}/${message.channel()}/${message.id()}`))
                        .catch(err => console.fatal(`failed to edit reply : ${err}`))
                })
        } else
            await output_message.command.delete_reply()
    }

    _build_paid(command) {

        if (command.read('remuneration') !== 'Rémunération')
            return {
                message: new Message()
                    .set_text('Pour les projets de loisirs ou pour tout autre type de payement, veuillez utiliser la commande /unpaid.')
                    .set_client_only(),
                valid: false
            }

        if (command.read('contrat') === 'Contractuel' && !command.read('duree'))
            return {
                message: new Message()
                    .set_text('Veuillez spécifier l\'option \'duree\' dans le cas d\'un contrat temporaire')
                    .set_client_only(),
                valid: false
            }

        const duree = command.read('contrat') === 'Contractuel' ? command.read('duree') : 'permanent'

        const embed = new Embed()
            .set_title((command.read('role') || 'option manquante') + " Chez " + (command.read('societe') || 'option manquante'))
            .set_description(command.read('remote') || 'option manquante')
            .add_field('Durée du contrat', duree, true)

        if (command.read('localisation'))
            embed.add_field('Localisation', command.read('localisation'), true)

        embed.add_field('Responsabilités', command.read('responsabilites') || 'valeur manquante')
            .add_field('Qualifications\n', command.read('qualifications') || 'valeur manquante')
            .add_field('Comment postuler\n', command.read('postuler') || 'valeur manquante')

        return {
            message: new Message()
                .add_embed(embed),
            valid: true
        }
    }

    _build_unpaid(command) {
        return new Message()
            .add_embed(new Embed()
                .set_title(command.read('titre') || 'Option manquante')
                .set_description(command.read('description') || 'Option manquante')
                .add_field('contact', command.read('contact') || 'Option manquante')
            )
    }

    _build_freelance(command) {

        const url = command.read('portfolio') || 'option manquante'
        const url_regex = /https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\\+.~#?&/=]*)/g
        if (!url_regex.test(url)) {
            return {message: new Message().set_client_only().set_text('Le portfolio doit être une URL'), valid: false}
        }

        return {
            message: new Message()
                .add_embed(new Embed()
                    .set_title(command.read('nom') || 'Option manquante')
                    .set_description(url)
                    .add_field('Services', command.read('services') || 'Option manquante')
                    .add_field('Contacts', command.read('contact') || 'Option manquante')),
            valid: true
        }
    }
}

module.exports = {Module}