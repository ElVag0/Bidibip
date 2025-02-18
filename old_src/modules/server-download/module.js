const {ModuleBase} = require("../../utils/module_base");
const DI = require("../../utils/discord_interface");
const {CommandInfo} = require("../../utils/interactionBase");
const {Channel} = require("../../utils/channel");
const fs = require("fs")
const {Message} = require("../../utils/message");
const {InteractionRow} = require("../../utils/interaction_row");
const {Button} = require("../../utils/button");
const {Embed} = require("../../utils/embed");

class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos)
        this.enabled = false

        this.fetching_channels = {}

        this.commands = [

            new CommandInfo("download-all", "récupère l'historique du serveur", async (command_interaction) => {
                for (const channel of DI.get()._client.channels.cache.values())
                    if (channel.type !== 4 && channel.type !== 15)
                        this.download_channel(new Channel().set_id(channel.id), command_interaction.author(), command_interaction.channel())
                            .catch(err => console.error(`Failed to download channel ${channel.id} : ${err}`))

                await command_interaction.skip()
            })
                .set_admin_only(),
            new CommandInfo("download-channel", "récupère l'historique d'un channel", async (command_interaction) => {

                const channel_id = command_interaction.read('channel')

                if (this.fetching_channels[channel_id])
                    return command_interaction.reply(new Message().set_client_only().set_text('Already fetching this channel'))

                await this.download_channel(new Channel().set_id(channel_id), command_interaction.author(), command_interaction.channel())

                await command_interaction.skip()
            })
                .add_channel_option("channel", "download target")
                .set_admin_only()
        ]
    }

    get_download_status(channel_id) {
        let names = []
        fs.readdirSync(this.app_config.SAVE_DIR + `/message_history/${channel_id}/`).forEach(name => names.push(name.split('.')[0]))
        names = names.sort((a, b) => a - b)

        // Never downloaded
        if (names.length === 0)
            return null

        const last_file = names[0]
        const last_file_content = JSON.parse(fs.readFileSync(this.app_config.SAVE_DIR + `/message_history/${channel_id}/${last_file}.json`, 'utf-8'))
        const last_file_ids = Object.keys(last_file_content).sort((a, b) => b - a)
        const last_message = last_file_content[last_file_ids[0]]

        const first_file = names[names.length - 1]
        const first_file_content = JSON.parse(fs.readFileSync(this.app_config.SAVE_DIR + `/message_history/${channel_id}/${first_file}.json`, 'utf-8'))
        const first_file_ids = Object.keys(first_file_content).sort((a, b) => a - b)
        const first_message = first_file_content[first_file_ids[0]]

        return {
            first_message: first_message.i,
            first_file: Number(first_file),
            last_message: last_message.i,
            last_file: Number(last_file)
        }
    }

    async download_channel(channel, author, log_channel) {

        if (this.fetching_channels[channel.id()])
            return

        this.fetching_channels[channel.id()] = true

        if (!fs.existsSync(this.app_config.SAVE_DIR + `/message_history/${channel.id()}/`))
            fs.mkdirSync(this.app_config.SAVE_DIR + `/message_history/${channel.id()}/`, {recursive: true})

        const message = new Message()
            .set_channel(log_channel)
            .add_embed(new Embed().set_title('Progression...').set_description('starting...'))
        const config = {
            stop_process: false,
            base_message: message
        }

        const bind_reply = (message_reply) => {
            config.message_to_update = message_reply
            this.bind_button(message_reply, async (interaction) => {
                if (interaction.button_id() === 'stop' && interaction.author().id() === author.id())
                    config.stop_process = true
                await interaction.skip()
            })
        }

        const res = this.get_download_status(channel.id())
        if (!res)
            message.set_text(`Downloading messages of ${new Channel().set_id(channel.id()).url()}`)
        else
            message.set_text(`Downloading messages of ${new Channel().set_id(channel.id()).url()}\nfrom ${new Message().set_id(res.first_message).set_channel(new Channel().set_id(channel.id())).url()}\nafter ${new Message().set_id(res.last_message).set_channel(new Channel().set_id(channel.id())).url()}`)

        message.add_interaction_row(new InteractionRow()
            .add_button(new Button()
                .set_id('stop').set_label('Stop').set_type(Button.Danger)))
        bind_reply(await message.send())

        this.fetch_channel(channel, res, config)
            .catch(err => console.fatal(`Failed to fetch channel : ${err}`))
    }

    async fetch_messages(discord_handle, before_id = null, after_id = null) {
        const options = {}
        if (before_id)
            options.before = before_id
        if (after_id)
            options.after = after_id

        return await discord_handle.messages.fetch(options)
            .catch(console.fatal)
    }

    message_to_json(msg) {
        const data = {
            i: msg.id,
            t: msg.content,
            u: msg.author.username + "#" + msg.author.discriminator
        }
        const attachments = []
        for (const [_, v] of msg.attachments)
            attachments.push({n: v.name, f: v.attachment})
        if (attachments.length !== 0)
            data.a = attachments
        return data
    }

    async fetch_channel(channel, bounds, config) {
        const _handle = await channel._fetch_from_discord()
            .catch(console.fatal)

        let first_file = bounds ? bounds.first_file + 1 : 0
        let last_file = bounds ? bounds.last_file - 1 : null

        let first_message = bounds ? bounds.first_message : null
        let last_message = bounds ? bounds.last_message : null

        let message_count = 0

        let finished = false
        while (!config.stop_process && !finished) {

            for (let i = 0; i < 10 && !config.stop_process && !finished; ++i) {
                finished = true

                let older_count = 0
                // Older messages
                if (first_message !== false) {
                    let older_count = 0
                    const json = {}
                    let older_messages = await this.fetch_messages(_handle, first_message, null)
                        .catch(async err => console.fatal(`Failed to fetch messages (${await channel.name()}) : ${err}`))

                    older_messages.forEach(msg => {
                        finished = false
                        json[msg.createdTimestamp] = this.message_to_json(msg)

                        message_count += 1
                        older_count += 1
                    })
                    if (older_messages.last())
                        first_message = older_messages.last().id

                    if (older_count !== 0) {
                        console.info(`Saved ${older_count} messages of ${await channel.name()} to ${first_file}.json`)
                        fs.writeFileSync(`${this.app_config.SAVE_DIR}/message_history/${channel.id()}/${first_file}.json`, JSON.stringify(json))
                        first_file += 1
                    }
                }

                let newer_count = 0
                // Newer messages
                if (last_message !== false && last_file !== null) {
                    const json = {}
                    let older_messages = await this.fetch_messages(_handle, null, last_message)
                        .catch(err => console.fatal(`Failed to fetch messages : ${err}`))

                    older_messages.forEach(msg => {

                        finished = false
                        json[msg.createdTimestamp] = this.message_to_json(msg)

                        message_count += 1
                        newer_count += 1
                    })
                    if (older_messages.last())
                        last_message = older_messages.first().id

                    if (newer_count !== 0) {
                        console.info(`Saved ${newer_count} messages of ${await channel.name()} to ${last_file}.json`)
                        fs.writeFileSync(`${this.app_config.SAVE_DIR}/message_history/${channel.id()}/${last_file}.json`, JSON.stringify(json))
                        last_file -= 1
                    }
                }

                let update_text = `Saved ${older_count === 0 ? '0 older messages' : `${older_count} older messages to ${first_file}.json`} and ${newer_count === 0 ? '0 newer messages' : `${newer_count} newer messages to ${last_file}.json`} out of ${message_count} total messages !`
                update_text += `\nFirst message : ${new Message().set_id(first_message).set_channel(channel).url()}\nLast message : ${new Message().set_id(last_message).set_channel(channel).url()}`
                config.base_message.embeds()[0].set_description(update_text)
                config.message_to_update.update(config.base_message)
                    .catch(err => console.fatal(`Failed to edit message : ${err}`))
            }
        }

        await config.message_to_update.update(config.base_message.add_embed(
            new Embed().set_title('Message download operation finished !')
                .set_description(`${message_count} messages downloaded`)
                .add_field("Status", finished ? "finished" : config.stop_process ? "stopped by user" : "undefined"))
            .clear_interactions())
            .catch(err => console.fatal(`Failed to edit message : ${err}`))
        delete this.fetching_channels[channel.id()]
    }
}

module.exports = {Module}