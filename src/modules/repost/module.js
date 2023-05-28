// MODULE REPOST
const {CommandInfo} = require("../../utils/interaction")
const {Forum} = require("../../utils/forum")
const {Message} = require("../../utils/message")
const {Channel} = require("../../utils/channel")
const fs = require("fs")
const CONFIG = require("../../config")
const {resolve} = require("path")
const {Embed} = require("../../utils/embed");
const {Button} = require("../../utils/button");
const {InteractionRow} = require("../../utils/interaction_row");

class Module {
    constructor(create_infos) {
        this.enabled = false

        this.client = create_infos.client

        this.commands = [
            new CommandInfo('set-forum-link', 'Lie un forum a un channel de repost')
                .add_channel_option('forum', 'Forum où seront suivis les nouveaux posts')
                .add_channel_option('repost-channel', 'Canal où seront repostés les évenements du forum')
                .add_bool_option('enabled', 'Active ou desactive le lien', false, true)
                .set_admin_only(),
            new CommandInfo('view-forum-links', 'Voir la liste des liens entre forums et salons')
                .set_admin_only()
        ]

        try {
            const data = fs.readFileSync(CONFIG.get().SAVE_DIR + '/repost/repost-links.json', 'utf8')
            this.repost_links = JSON.parse(data)
        } catch (err) {
            this.repost_links = {}
        }
    }

    /**
     * // When command is executed
     * @param command {Interaction}
     * @return {Promise<void>}
     */
    async server_interaction(command) {
        if (command.match('set-forum-link')) {
            let source_forum = null
            try {
                source_forum = new Forum().set_id(command.read('forum'))
                if ((await source_forum.channel().type()) !== Channel.TypeForum) {
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

                if (this.repost_links[source_forum.channel().id()]) {
                    if (this.repost_links[source_forum.channel().id()].repost_channels.indexOf(repost_channels.id()) !== -1) {
                        await command.reply(new Message().set_text(`Le lien entre ce forum et ce salon est déjà créé`).set_client_only())
                        return
                    }

                    this.repost_links[source_forum.channel().id()].repost_channels.push(repost_channels.id())
                } else this.repost_links[source_forum.channel().id()] = {repost_channels: [repost_channels.id()]}
                this._save_config()
                await command.reply(new Message().set_text(`Lien créé`).set_client_only())
            } else {
                if (this.repost_links[source_forum.channel().id()]) {
                    if (this.repost_links[source_forum.channel().id()].repost_channels.indexOf(repost_channels.id()) !== -1) {
                        this.repost_links[source_forum.channel().id()].repost_channels.splice(this.repost_links[source_forum.channel().id()].repost_channels.indexOf(repost_channels.id()), 1)
                        await command.reply(new Message().set_text(`Lien supprimé`).set_client_only())
                    }
                    if (this.repost_links[source_forum.channel().id()].repost_channels.length === 0)
                        delete this.repost_links[source_forum.channel().id()]
                    this._save_config()
                } else
                    await command.reply(new Message().set_text(`Ce lien n'existe pas`).set_client_only())
            }
        }
        if (command.match('view-forum-links')) {

            const embed = new Embed()
                .set_title('Liens de repost')
                .set_description('Liste des liens de repost entre forums et salons de repost')
            for (const [key, value] of Object.entries(this.repost_links)) {
                let forums = ''
                for (const forum of value.repost_channels)
                    forums += `Vers https://discord.com/channels/${CONFIG.get().SERVER_ID}/${forum}\n`
                embed.add_field(`De https://discord.com/channels/${CONFIG.get().SERVER_ID}/${key}`, forums.length === 0 ? 'Pas de lien' : forums)
            }

            command.reply(
                new Message()
                    .add_embed(embed)
                    .set_client_only())
        }
    }

    /**
     * @param thread {Thread}
     * @returns {Promise<void>}
     */
    async thread_created(thread) {
        const parent_id = (await thread.channel().parent_channel()).id()

        if (this.repost_links[parent_id]) {
            for (const channel of this.repost_links[parent_id].repost_channels) {
                await new Message()
                    .set_text(`Forum created : https://discord.com/channels/${CONFIG.get().SERVER_ID}/${thread.channel().id()}`)
                    .set_channel(new Channel().set_id(channel))
                    .add_interaction_row(new InteractionRow()
                        .add_button(new Button().set_id('test').set_label('toto').set_type(Button.Danger)))
                    .send()
            }
        }
    }

    _save_config() {
        if (!fs.existsSync(CONFIG.get().SAVE_DIR + '/repost/'))
            fs.mkdirSync(CONFIG.get().SAVE_DIR + '/repost/', {recursive: true})
        fs.writeFile(CONFIG.get().SAVE_DIR + '/repost/repost-links.json', JSON.stringify(this.repost_links, (key, value) => typeof value === 'bigint' ? value.toString() : value), 'utf8', err => {
            if (err) {
                console.fatal(`failed to save reposts : ${err}`)
            } else {
                console.info(`Saved reposts to file ${CONFIG.get().SAVE_DIR + '/repost/repost-links.json'}`)
            }
        })
    }
}

module.exports = {Module}