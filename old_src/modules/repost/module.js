// MODULE REPOST
const {CommandInfo} = require("../../utils/interactionBase")
const DI = require("../../utils/discord_interface")
const {Message} = require("../../utils/message")
const {Channel} = require("../../utils/channel")
const {Embed} = require("../../utils/embed")
const {Button} = require("../../utils/button")
const {InteractionRow} = require("../../utils/interaction_row")
const {Thread} = require("../../utils/thread")
const {ModuleBase} = require("../../utils/module_base")
const {User} = require("../../utils/user");

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

async function format_message(message, header) {
    const message_group = []
    const author = await message.author()
    const channel = message.channel()
    const first_message_embed =
        new Embed()
            .set_author(author)
            .set_color('#96ff96')
            .set_description(await message.text())

    message_group.push(
        new Message()
            .set_text(`${header} ${channel.url()} !`)
            .add_embed(first_message_embed))

    const linked_files = find_urls(await message.text()).concat()
    for (const attachment of message.attachments())
        linked_files.push(attachment.file())

    for (const i in linked_files) // only for first image
        if (/\.(jpg|jpeg|png|webp|avif|gif)$/.test(linked_files[i])) {
            first_message_embed.set_image(linked_files[i])
            linked_files.splice(i, 1)
            break
        }

    for (const i in linked_files) // only for first file not image or video
        if (!/\.(mp4|mov|avi|mkv|flv|jpg|jpeg|png|webp|avif|gif)$/.test(linked_files[i])) {
            first_message_embed.set_author_url(linked_files[0])
            break
        }

    // Add remaining hyperlinks as message (one message per link)
    for (const hyperlink of linked_files)
        message_group.push(new Message()
            .set_text(hyperlink))

    // Add url to last message
    message_group[message_group.length - 1]
        .add_interaction_row(
            new InteractionRow()
                .add_button(new Button()
                    .set_label('Viens donc voir !')
                    .set_type(Button.Link)
                    .set_url(message.url())))

    return message_group
}

function find_urls(initial_text) {
    const split = initial_text.split(/\r\n|\r|\n| /)
    const attachments = []
    for (let i = split.length - 1; i >= 0; --i)
        if (split[i].includes('http'))
            attachments.push(split[i])

    return attachments
}

class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos);

        // These are the channels waiting for the first message
        this.threads_waiting_first_message = new Set()

        this.commands = [
            new CommandInfo('set-forum-link', 'Lie un forum a un channel de repost', this.set_forum_link)
                .add_channel_option('forum', 'Forum où seront suivis les nouveaux posts')
                .add_channel_option('repost-channel', 'Canal où seront repostés les évenements du forum')
                .add_bool_option('vote', 'Avec ou sans fonctionnalités de vote')
                .add_bool_option('enabled', 'Active ou desactive le lien', false, true)
                .set_admin_only(),
            new CommandInfo('view-forum-links', 'Voir la liste des liens entre forums et salons', this.view_forum_link)
                .set_admin_only(),
            new CommandInfo('reposte', 'Promeut le message donné dans le salon de repost', this.promote)
                .set_member_only()
                .add_message_option('message', 'lien du message à promouvoir'),
        ]

        this.load_config({
            reposted_forums: {},
            repost_votes: {},
            vote_messages: {}
        })

        // Keep track for vote button
        for (const [key, value] of Object.entries(this.module_config.vote_messages)) {
            (async () => {
                const thread = new Thread().set_id(value)
                const message = message_from_key(key)
                if (!await message.is_valid() || !await thread.is_valid())
                    delete this.module_config.vote_messages[key]

                if (!await thread.is_valid() && this.module_config.repost_votes[value])
                    delete this.module_config.repost_votes[value]

                if (await thread.is_valid() && await message.is_valid())
                    this.bind_vote_button(message, thread)
                        .catch(err => console.error(`Failed to update vote button : ${err}`))
                else {
                    this.save_config()
                    console.warning('Cleaned up outdated repost message or thread for ', thread)
                }
            })()
                .catch(err => console.error(`Failed to bind repost messages : ${err}`))
        }
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async set_forum_link(command) {

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
        const bound_channels = new Channel().set_id(command.read('repost-channel'))
        const enable = command.read('enabled')

        if ((await bound_channels.type()) !== Channel.TypeChannel) {
            console.warning(`'repost-channel' option does not correspond to a standard channel`)
            await command.reply(new Message().set_text(`Le salon de repost que vous avez fourni n'est pas un salon standard`).set_client_only())
            return
        }

        if (enable) {
            console.info('created link between', source_forum, 'and', bound_channels)

            if (this.module_config.reposted_forums[source_forum.id()]) {
                if (this.module_config.reposted_forums[source_forum.id()].bound_channels.indexOf(bound_channels.id()) === -1)
                    this.module_config.reposted_forums[source_forum.id()].bound_channels.push(bound_channels.id())
            } else this.module_config.reposted_forums[source_forum.id()] = {
                bound_channels: [bound_channels.id()]
            }

            this.module_config.reposted_forums[source_forum.id()].vote = command.read('vote')

            this.save_config()
            await command.reply(new Message().set_text(`Lien créé`).set_client_only())
        } else {
            if (this.module_config.reposted_forums[source_forum.id()]) {
                if (this.module_config.reposted_forums[source_forum.id()].bound_channels.indexOf(bound_channels.id()) !== -1) {
                    this.module_config.reposted_forums[source_forum.id()].bound_channels.splice(this.module_config.reposted_forums[source_forum.id()].bound_channels.indexOf(bound_channels.id()), 1)
                    await command.reply(new Message().set_text(`Lien supprimé`).set_client_only())
                }
                if (this.module_config.reposted_forums[source_forum.id()].bound_channels.length === 0)
                    delete this.module_config.reposted_forums[source_forum.id()]
                this.save_config()
            } else
                await command.reply(new Message().set_text(`Ce lien n'existe pas`).set_client_only())
        }
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async view_forum_link(command) {
        const embed = new Embed()
            .set_title('Liens de repost')
            .set_color('#96ff96')
            .set_description('Liste des liens de repost entre forums et salons de repost')
        for (const [key, value] of Object.entries(this.module_config.reposted_forums)) {
            let forums = ''
            for (const forum of value.bound_channels) {
                forums += `Vers ${new Channel().set_id(forum).url()}\n`
            }
            embed.add_field(`De ${new Channel().set_id(key).url()} (vote : ${value.vote ? ':white_check_mark:' : ':x:'})`, forums.length === 0 ? 'Pas de lien' : forums)
        }

        await command.reply(
            new Message()
                .add_embed(embed)
                .set_client_only())
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async promote(command) {
        // The message we should promote
        const message_to_promote = command.read('message')

        // Ensure message exists
        if (!await message_to_promote.exists())
            return command.reply(new Message().set_client_only().set_text('Ce message n\'existe pas'))

        // Ensure message was posted inside a forum
        const thread = new Thread().set_id(message_to_promote.channel().id())
        const forum = await thread.parent_channel()
        if (!forum || await forum.type() !== Channel.TypeForum)
            return command.reply(new Message().set_client_only().set_text('Le message doit provenir d\'un forum'))

        // Ensure command author is allowed to promote this message
        if ((await thread.owner()).id() !== command.author().id())
            return command.reply(new Message().set_client_only().set_text('Vous devez être l\'auteur du salon d\'où le message sera promu'))

        // Ensure repost is enabled
        if (!this.module_config.reposted_forums[forum.id()])
            return command.reply(new Message().set_client_only().set_text(`Le repost n'est pas activé dans ce forum`))

        const message_to_send = await format_message(message_to_promote, 'Mise à jour dans ')
            .catch(err => console.fatal(`Failed to create update messages : ${err}`))
        const last_message = message_to_send.pop()

        // Repost in every channels
        for (const channel of this.module_config.reposted_forums[forum.id()].bound_channels) {
            for (const repost_message of message_to_send)
                await repost_message.set_channel(new Channel().set_id(channel)).send()

            last_message.set_channel(new Channel().set_id(channel)).send().then(async sent_last => {
                if (this.module_config.reposted_forums[forum.id()].vote)
                    await this.bind_vote_button(sent_last, thread)
            })
        }

        await
            command
                .reply(
                    new

                    Message()

                        .set_client_only()

                        .set_text(
                            'Ton post a été promu !'
                        ))
    }

    /**
     * @param thread {Thread}
     * @returns {Promise<void>}
     */
    async thread_created(thread) {
        // We detected a thread have been created. Now we wait it's first message
        if (this.module_config.reposted_forums[(await thread.parent_channel()).id()])
            this.threads_waiting_first_message.add(thread.id())
    }

    /**
     * On received server messages
     * @param message {Message}
     * @return {Promise<void>}
     */
    async server_message(message) {
        const thread = new Thread().set_id((await message.channel()).id())

        // Ensure the thread that contains the message is waiting for it's first message
        if (!this.threads_waiting_first_message.has(thread.id()))
            return

        // Don't track anymore
        this.threads_waiting_first_message.delete(thread.id())

        const forum = await thread.parent_channel()

        // Repost is not enabled for this forum
        if (!this.module_config.reposted_forums[forum.id()])
            return

        // Create message
        await message.pin()
        message.set_text(`# ${await thread.name()}\n${await message.text()}`)
        const messages = (await format_message(message, `Nouveau post dans **#${await forum.name()}** : `))
        const last_message = messages.pop()

        if (this.module_config.reposted_forums[forum.id()].vote) {
            await message.react('📩')
            await new Message()
                .set_text(`Vote en réagissant au post !`)
                .set_channel(message.channel())
                .add_interaction_row(new InteractionRow()
                    .add_button(new Button()
                        .set_type(Button.Secondary)
                        .set_label('✉️ Voir les votes')
                        .set_id('see-votes')))
                .send()
                .then(reposted_message => this.bind_vote_button(reposted_message, thread))
        }

        for (const channel of this.module_config.reposted_forums[forum.id()].bound_channels) {
            for (const repost_message of messages)
                await repost_message
                    .set_channel(new Channel().set_id(channel))
                    .send()

            await last_message
                .set_channel(new Channel().set_id(channel))
                .send()
                .then(reposted_message => this.bind_vote_button(reposted_message, thread))
        }

        await this.update_all_messages(thread)
    }

    async create_or_update_vote_buttons(message, vote_thread, ephemeral = false) {

        if (await vote_thread.archived()) {
            if ((await message.channel()).id() !== vote_thread.id())
                await message.delete();
            return;
        }

        let yes_button = await message.get_button_by_id('button-vote-yes')
        let no_button = await message.get_button_by_id('button-vote-no')
        let see_votes_button = await message.get_button_by_id('see-votes')
        if (message._interactions.length === 0) message.add_interaction_row(new InteractionRow())
        const interaction_row = message._interactions[0]
        if (!yes_button) {
            yes_button = new Button()
                .set_id('button-vote-yes')
                .set_type(Button.Primary)
            interaction_row.add_button(yes_button)
        }
        if (!no_button) {
            no_button = new Button()
                .set_id('button-vote-no')
                .set_type(Button.Primary)
            interaction_row.add_button(no_button)
        }
        if (!see_votes_button) {
            see_votes_button = new Button()
                .set_type(Button.Secondary)
                .set_label('✉️ Voir les votes')
                .set_id('see-votes')
            interaction_row.add_button(see_votes_button)
        }

        const yes_text = `Pour ✅ ${Object.entries(this.module_config.repost_votes[vote_thread.id()].vote_yes).length}`
        const no_text = `Contre ❌ ${Object.entries(this.module_config.repost_votes[vote_thread.id()].vote_no).length}`
        if (yes_text !== yes_button._label || no_text !== no_button._label) {
            yes_button.set_label(yes_text)
            no_button.set_label(no_text)

            // Update button
            await message.update(message)
                .catch(err => console.fatal(`Failed to update message :`, err))
        }
        if (!ephemeral && !this.module_config.vote_messages[make_key(message)]) {
            this.module_config.vote_messages[make_key(message)] = vote_thread.id()
            this.save_config()
        }
    }

    async bind_vote_button(message, thread) {
        // Not a thread
        if (!await thread.parent_channel())
            return

        // Check if vote is enabled or not
        if (!this.module_config.reposted_forums[(await thread.parent_channel()).id()].vote)
            return

        // Init if not exists
        if (!this.module_config.repost_votes[thread.id()]) {
            this.module_config.repost_votes[thread.id()] = {
                vote_yes: {},
                vote_no: {},
                bound_messages: [],
                channel_title: await thread.name()
            }
            this.save_config()
        }

        if (!this.module_config.repost_votes[thread.id()].bound_messages) {
            console.warning("Failed to find bound message : the original thread was probably deleted");
            return;
        }

        if (this.module_config.repost_votes[thread.id()].bound_messages.indexOf(make_key(message)) === -1)
            this.module_config.repost_votes[thread.id()].bound_messages.push(make_key(message))

        this.bind_button(message, async (button_interaction) => {
            await this.click_vote_button(button_interaction, thread)
        })

        await this.create_or_update_vote_buttons(message, thread)
            .catch(err => console.fatal(`Failed to create or update vote button : ${err}`))
    }

    async click_vote_button(button_interaction, thread) {

        const vote_infos = this.module_config.repost_votes[thread.id()]
        if (!vote_infos)
            return

        if (button_interaction.button_id() === 'button-vote-yes') {
            if (vote_infos.vote_yes[button_interaction.author().id()])
                delete vote_infos.vote_yes[button_interaction.author().id()]
            else {
                vote_infos.vote_yes[button_interaction.author().id()] = true
                if (vote_infos.vote_no[button_interaction.author().id()])
                    delete vote_infos.vote_no[button_interaction.author().id()]
            }
            this.save_config()
            await button_interaction.skip()
        } else if (button_interaction.button_id() === 'button-vote-no') {
            if (vote_infos.vote_no[button_interaction.author().id()])
                delete vote_infos.vote_no[button_interaction.author().id()]
            else {
                vote_infos.vote_no[button_interaction.author().id()] = true
                if (vote_infos.vote_yes[button_interaction.author().id()])
                    delete vote_infos.vote_yes[button_interaction.author().id()]
            }
            this.save_config()
            await button_interaction.skip()
        } else if (button_interaction.button_id() === 'see-votes') {
            let vote_yes_str = ''
            for (const [yes_id, _] of Object.entries(vote_infos.vote_yes))
                vote_yes_str += `${new User().set_id(yes_id).mention()}\n`
            let vote_no_str = ''
            for (const [no_id, _] of Object.entries(vote_infos.vote_no))
                vote_no_str += `${new User().set_id(no_id).mention()}\n`

            button_interaction.reply(new Message()
                .set_client_only()
                .add_embed(new Embed()
                    .set_title('Votes actuels')
                    .set_color('#96ff96')
                    .set_description('Nombre de votes : ' + (Object.entries(vote_infos.vote_yes).length + Object.entries(vote_infos.vote_no).length))
                    .add_field('Pour ✅', vote_yes_str === '' ? '-' : vote_yes_str, true)
                    .add_field('Contre ❌', vote_no_str === '' ? '-' : vote_no_str, true)))
        }

        await this.update_all_messages(thread)
    }

    async update_all_messages(thread) {
        const vote_infos = this.module_config.repost_votes[thread.id()]
        if (!vote_infos)
            return;
        for (const message_to_update of vote_infos.bound_messages) {
            const vote_message = message_from_key(message_to_update)
            if (!await vote_message.is_valid() || !await thread.is_valid())
                delete this.module_config.vote_messages[message_to_update]

            if (!await thread.is_valid() && this.module_config.repost_votes[thread.id()])
                delete this.module_config.repost_votes[thread.id()]

            if (await vote_message.is_valid() && await thread.is_valid())
                await this.create_or_update_vote_buttons(vote_message, thread)
                    .catch(err => console.error(`Failed to update vote button : ${err}`))
            else {
                this.save_config()
                console.warning('Cleaned up outdated repost message or thread for ', thread)
            }
        }
        if (!vote_infos.channel_title) {
            vote_infos.channel_title = await thread.name()
            this.save_config()
        }
        const num_yes = Object.entries(vote_infos.vote_yes).length
        const num_no = Object.entries(vote_infos.vote_no).length
        const prefix = num_yes === num_no ? '📩' : (num_yes > num_no ? '✅' : '❌')
        if (await thread.is_valid())
            thread.set_name(`[${prefix} ${num_yes}-${num_no}] ${vote_infos.channel_title.substring(0, 80)}`)
    }

    /**
     * On message reaction
     * @param reaction {Reaction}
     * @param user {User}
     * @return {Promise<void>}
     */
    async add_reaction(reaction, user) {
        const thread = new Thread().set_id(reaction.message().channel().id())
        const vote_data = this.module_config.repost_votes[thread.id()]
        const first_message = thread.first_message()
        if (reaction.message().id() !== first_message.id())
            return
        if (!vote_data)
            return

        const reactions = await first_message.reactions()
        if (reactions.length !== 0 && reactions[0] === reaction.emoji()) {
            await reaction.remove_user(user)

            const text = vote_data.vote_yes[user.id()] ? `${user.mention()} tu as déjà voté pour ✅ !` : (vote_data.vote_no[user.id()] ? `${user.mention()} tu as déjà voté contre ❌ !` : `J'attends ton vote ${user.mention()}`)

            await new Message().set_text(text).set_client_only().set_channel(thread)
                .send().then(message => {
                    this.create_or_update_vote_buttons(message, thread, true)
                    this.bind_button(message, async (button_interaction) => {
                        await this.click_vote_button(button_interaction, thread)
                        if (button_interaction.author().id() === user.id())
                            await message.delete()
                    })
                })
        }
    }
}

module
    .exports = {Module}