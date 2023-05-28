// MODULE REPOST
const {CommandInfo} = require("../../utils/interactionBase")
const {Message} = require("../../utils/message")
const {Channel} = require("../../utils/channel")
const fs = require("fs")
const CONFIG = require("../../config")
const {Embed} = require("../../utils/embed")
const {Button} = require("../../utils/button")
const {InteractionRow} = require("../../utils/interaction_row")
const MODULE_MANAGER = require("../../core/module_manager")
const {Thread} = require("../../utils/thread");
const {Attachment} = require("../../utils/attachment");

function make_key(message) {
    return `${message.channel().id()}/${message.id()}`
}

/**
 * @param key {string}
 * @returns {Message}
 */
function message_from_key(key) {
    const split = key.split('/')
    return new Message().set_id(split[1]).set_channel(new Channel().set_id(split[0]))
}

async function format_message(message) {
    const author = await message.author()

    const res = extract_images(await message.text())
    const res_message = new Message()
        .add_embed(new Embed()
            .set_title(`De ${await author.full_name()} : ${await message.channel().name()}`)
            .set_description(res.text)
            .set_thumbnail(await author.profile_picture()))

    for (const attachment of res.attachments)
        res_message.add_attachment(attachment)

    return res_message
}

function extract_images(initial_text) {
    const split = initial_text.split(/\r\n|\r|\n| /)
    const attachments = []
    for (let i = split.length - 1; i >= 0; --i)
    {
        if (split[i].includes('http') && /\.(jpg|jpeg|png|webp|avif|gif)$/.test(split[i].toLowerCase())) {
            attachments.push(new Attachment().set_file(split[i]))
            split.splice(i, 1)
        }
    }

    return {
        text: split.join(' '),
        attachments: attachments
    }
}

class Module {
    constructor(create_infos) {
        this.enabled = false

        this.client = create_infos.client

        this.waiting_channels = new Set()

        this.commands = [
            new CommandInfo('set-forum-link', 'Lie un forum a un channel de repost')
                .add_channel_option('forum', 'Forum où seront suivis les nouveaux posts')
                .add_channel_option('repost-channel', 'Canal où seront repostés les évenements du forum')
                .add_bool_option('vote', 'Avec ou sans fonctionnalités de vote')
                .add_bool_option('enabled', 'Active ou desactive le lien', false, true)
                .set_admin_only(),
            new CommandInfo('view-forum-links', 'Voir la liste des liens entre forums et salons')
                .set_admin_only(),
            new CommandInfo('promote', 'Promeut le message donné dans le salon de repost')
                .add_text_option('message', 'lien du message à promouvoir'),
        ]

        try {
            const data = fs.readFileSync(CONFIG.get().SAVE_DIR + '/repost/repost-links.json', 'utf8')
            this.repost_data = JSON.parse(data)
        } catch (_) {
        }

        if (!this.repost_data) this.repost_data = {}
        if (!this.repost_data.repost_links) this.repost_data.repost_links = {}
        if (!this.repost_data.repost_votes) this.repost_data.repost_votes = {}
        if (!this.repost_data.thread_owners) this.repost_data.thread_owners = {}

        this.bound_message = {}
        for (const [key, _] of Object.entries(this.repost_data.repost_votes)) {
            const split = key.split('-')
            this._bind_messages(message_from_key(split[0]), message_from_key(split[1]))
        }
    }

    /**
     * // When command is executed
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async server_interaction(command) {
        if (command.match('set-forum-link')) {
            let source_forum = null
            try {
                source_forum = new Channel().set_id(command.read('forum'))
                if ((await source_forum.type()) !== Channel.TypeForum) {
                    throw new Error('Channel is not a forum')
                }
            } catch (err) {
                console.warning(`'forum' option does not correspond to a forum : ${err}`)
                await command.reply(new Message().set_text(`Le salon que vous avez fourni n'est pas un forum`).set_client_only())
                return
            }
            const repost_channels = new Channel().set_id(command.read('repost-channel'))
            const enable = command.read('enabled')

            if ((await repost_channels.type()) !== Channel.TypeChannel) {
                console.warning(`'repost-channel' option does not correspond to a standard channel`)
                await command.reply(new Message().set_text(`Le salon de repost que vous avez fourni n'est pas un salon standard`).set_client_only())
                return
            }

            if (enable) {
                console.info('created link between', source_forum, 'and', repost_channels)

                if (this.repost_data.repost_links[source_forum.id()]) {
                    if (this.repost_data.repost_links[source_forum.id()].repost_channels.indexOf(repost_channels.id()) === -1)
                        this.repost_data.repost_links[source_forum.id()].repost_channels.push(repost_channels.id())
                } else this.repost_data.repost_links[source_forum.id()] = {
                    repost_channels: [repost_channels.id()]
                }

                this.repost_data.repost_links[source_forum.id()].vote = command.read('vote')

                this._save_config()
                await command.reply(new Message().set_text(`Lien créé`).set_client_only())
            } else {
                if (this.repost_data.repost_links[source_forum.id()]) {
                    if (this.repost_data.repost_links[source_forum.id()].repost_channels.indexOf(repost_channels.id()) !== -1) {
                        this.repost_data.repost_links[source_forum.id()].repost_channels.splice(this.repost_data.repost_links[source_forum.id()].repost_channels.indexOf(repost_channels.id()), 1)
                        await command.reply(new Message().set_text(`Lien supprimé`).set_client_only())
                    }
                    if (this.repost_data.repost_links[source_forum.id()].repost_channels.length === 0)
                        delete this.repost_data.repost_links[source_forum.id()]
                    this._save_config()
                } else
                    await command.reply(new Message().set_text(`Ce lien n'existe pas`).set_client_only())
            }
        }
        if (command.match('view-forum-links')) {
            const embed = new Embed()
                .set_title('Liens de repost')
                .set_description('Liste des liens de repost entre forums et salons de repost')
            for (const [key, value] of Object.entries(this.repost_data.repost_links)) {
                let forums = ''
                for (const forum of value.repost_channels)
                    forums += `Vers https://discord.com/channels/${CONFIG.get().SERVER_ID}/${forum}\n`
                embed.add_field(`De https://discord.com/channels/${CONFIG.get().SERVER_ID}/${key}`, forums.length === 0 ? 'Pas de lien' : forums)
            }

            await command.reply(
                new Message()
                    .add_embed(embed)
                    .set_client_only())
        }
        if (command.match('promote')) {

            const message = new Message().set_id(command.read('message')).set_channel(command.channel())

            if (command.read('message').includes('/')) {
                const split = command.read('message').split('/')
                message.set_id(split[split.length - 1]).set_channel(new Channel().set_id(split[split.length - 2]))
            }

            const forum = await message.channel().parent_channel()
            if (!forum || await forum.type() !== Channel.TypeForum) {
                command.reply(new Message().set_client_only().set_text('Le message doit provenir d\'un forum'))
                return
            }

            if (!await message.exists()) {
                command.reply(new Message().set_client_only().set_text('Ce message n\'existe pas'))
                return
            }

            if (this.repost_data.repost_links[(await message.channel().parent_channel()).id()]) {

                if (!this.repost_data.thread_owners[message.channel().id()]) {
                    const thread_author = await new Thread().set_id(message.channel().id()).owner()
                        .catch(err => console.fatal(`Failed to get thread owner : ${err}`))
                    this.repost_data.thread_owners[message.channel().id()] = thread_author.id()
                }

                if (this.repost_data.thread_owners[message.channel().id()] !== command.author().id()) {
                    command.reply(new Message().set_client_only().set_text('Vous devez être l\'auteur du salon d\'où le message sera promu'))
                        .catch(err => console.fatal(`Failed to reply ${err}`))
                    return
                }

                const reposted_message = (await format_message(message))
                    .set_text(`Mise à jour dans https://discord.com/channels/${CONFIG.get().SERVER_ID}/${message.channel().id()}/${message.id()}`)
                    .add_interaction_row(new InteractionRow()
                        .add_button(new Button()
                            .set_label('Viens donc voir !')
                            .set_type(Button.Link)
                            .set_url(`https://discord.com/channels/${CONFIG.get().SERVER_ID}/${message.channel().id()}/${message.id()}`)))

                for (const attachment of message.attachments())
                    reposted_message.add_attachment(attachment)

                for (const channel of this.repost_data.repost_links[(await message.channel().parent_channel()).id()].repost_channels) {
                    await reposted_message
                        .set_channel(new Channel().set_id(channel))
                        .send()
                }

                command.reply(new Message().set_client_only().set_text('Ton post a été promu !'))
                    .catch(err => console.fatal(`Failed to reply ${err}`))
            } else
                command.reply(new Message().set_client_only().set_text('Ce forum a désactivé le repost'))
                    .catch(err => console.fatal(`Failed to reply ${err}`))
        }
    }

    /**
     * @param thread {Thread}
     * @returns {Promise<void>}
     */
    async thread_created(thread) {
        const forum = await thread.parent_channel()
        if (this.repost_data.repost_links[forum.id()]) {
            this.waiting_channels.add(thread.id())
        }
    }

    /**
     * On received server messages
     * @param message {Message}
     * @return {Promise<void>}
     */
    async server_message(message) {
        const thread = await message.channel()

        if (this.waiting_channels.has(thread.id())) {
            const source_parent = await thread.parent_channel()
            if (this.repost_data.repost_links[source_parent.id()]) {
                const author = await message.author()

                for (const channel of this.repost_data.repost_links[source_parent.id()].repost_channels) {
                    const url = `https://discord.com/channels/${CONFIG.get().SERVER_ID}/${thread.id()}`

                    const interaction = new InteractionRow()

                    if (this.repost_data.repost_links[source_parent.id()].vote)
                        interaction
                            .add_button(
                                new Button()
                                    .set_id('yes')
                                    .set_label('Pour ✅ (0)')
                                    .set_type(Button.Success))
                            .add_button(
                                new Button()
                                    .set_id('no')
                                    .set_label('Contre ❌ (0)')
                                    .set_type(Button.Danger))

                    interaction.add_button(
                        new Button()
                            .set_label('J\'y cause !')
                            .set_type(Button.Link)
                            .set_url(`${url}`))

                    await (await format_message(message))
                        .set_text(`Nouveau post dans #${await source_parent.name()} : ${url}`)
                        .set_channel(new Channel().set_id(channel))
                        .add_interaction_row(interaction)
                        .send()
                        .then(reposted_message => {
                            const message = new Message()
                                .set_text(`Message original : https://discord.com/channels/${CONFIG.get().SERVER_ID}/${reposted_message.channel().id()}/${reposted_message.id()}`)
                                .set_channel(thread)


                            if (this.repost_data.repost_links[source_parent.id()].vote)
                                message.add_interaction_row(new InteractionRow()
                                    .add_button(new Button()
                                        .set_id('yes')
                                        .set_label(`Pour ✅ (0)`)
                                        .set_type(Button.Success))
                                    .add_button(new Button()
                                        .set_id('no')
                                        .set_label('Contre ❌ (0)')
                                        .set_type(Button.Danger)))

                            message.send()
                                .then(forum_message => {
                                    forum_message.pin()

                                    this.repost_data.repost_votes[`${make_key(reposted_message)}-${make_key(forum_message)}`] = {
                                        vote_yes: [],
                                        vote_no: []
                                    }

                                    this.repost_data.thread_owners[thread.id()] = author.id()

                                    this._bind_messages(reposted_message, forum_message)

                                    this._save_config()
                                })
                        })
                }
            }

            this.waiting_channels.delete(thread.id())
        }
    }

    _bind_messages(reposted_message, forum_message) {
        /**
         * @param button_interaction {ButtonInteraction}
         */
        const callback = async button_interaction => {
            const message = button_interaction.message()

            const big_key = this.bound_message[make_key(message)]

            const vote_group = this.repost_data.repost_votes[big_key]
            const author = button_interaction.author().id()
            if (vote_group) {
                // Update data
                if (button_interaction.button_id() === 'yes') {
                    if (vote_group.vote_yes.indexOf(author) !== -1) {
                        button_interaction.reply(new Message().set_text('Tu a déjà voté !').set_client_only())
                        return
                    }
                    if (vote_group.vote_no.indexOf(author) !== -1)
                        vote_group.vote_no.splice(vote_group.vote_no.indexOf(author), 1)
                    vote_group.vote_yes.push(button_interaction.author().id())
                } else if (button_interaction.button_id() === 'no') {
                    if (vote_group.vote_no.indexOf(author) !== -1) {
                        button_interaction.reply(new Message().set_text('Tu a déjà voté !').set_client_only())
                        return
                    }
                    if (vote_group.vote_yes.indexOf(author) !== -1)
                        vote_group.vote_yes.splice(vote_group.vote_no.indexOf(author), 1)
                    vote_group.vote_no.push(button_interaction.author().id())
                } else {
                    console.fatal('unknown interaction')
                }

                // Update vote count

                const message_0 = message_from_key(big_key.split('-')[0])
                if (await message_0.exists()) {
                    ;(await message_0.get_button_by_id('yes')).set_label(`Pour ✅ (${vote_group.vote_yes.length})`)
                    ;(await message_0.get_button_by_id('no')).set_label(`Contre ❌ (${vote_group.vote_no.length})`)
                    await message_0.update(message_0)
                }

                const message_1 = message_from_key(big_key.split('-')[1])
                if (await message_1.exists()) {
                    ;(await message_1.get_button_by_id('yes')).set_label(`Pour ✅ (${vote_group.vote_yes.length})`)
                    ;(await message_1.get_button_by_id('no')).set_label(`Contre ❌ (${vote_group.vote_no.length})`)
                    await message_1.update(message_1)
                }
                this._save_config()

                button_interaction.reply(new Message().set_text('Merci pour ton vote !').set_client_only())
            }
        }

        const big_key = `${make_key(reposted_message)}-${make_key(forum_message)}`
        this.bound_message[make_key(reposted_message)] = big_key
        this.bound_message[make_key(forum_message)] = big_key

        MODULE_MANAGER.get().bind_button(this, reposted_message, callback)
        MODULE_MANAGER.get().bind_button(this, forum_message, callback)
    }

    _save_config() {
        if (!fs.existsSync(CONFIG.get().SAVE_DIR + '/repost/'))
            fs.mkdirSync(CONFIG.get().SAVE_DIR + '/repost/', {recursive: true})
        fs.writeFile(CONFIG.get().SAVE_DIR + '/repost/repost-links.json', JSON.stringify(this.repost_data, (key, value) => typeof value === 'bigint' ? value.toString() : value), 'utf8', err => {
            if (err) {
                console.fatal(`failed to save reposts : ${err}`)
            } else {
                console.info(`Saved reposts to file ${CONFIG.get().SAVE_DIR + '/repost/repost-links.json'}`)
            }
        })
    }
}

module.exports = {Module}