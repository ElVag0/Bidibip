const DI = require("./discord_interface");
const CONFIG = require("../config");
const {Embed} = require("./embed");
const {ChannelType} = require('discord.js');
const assert = require("assert");

class Channel {

    static TypeChannel = 0
    static TypeThread = 11
    static TypePrivateThread = 12
    static TypeForum = 15

    constructor(_api_handle) {
        if (_api_handle) {
            this._fill_from_api_handle(_api_handle)
        }
    }

    /**
     * @param id {string}
     * @returns {Channel | Thread}
     */
    set_id(id) {
        if (typeof(id) !== 'number' && typeof(id) !== 'string')
            console.fatal("Id should be a number : ", id);
        this._id = id
        return this
    }

    set_parent_channel(channel) {
        this._parent_channel = channel
        return this
    }

    async type() {
        if (!this._type)
            await this._fetch_from_discord()
                .catch(console.error)
        return this._type
    }

    id() {
        if (!this._id)
            console.fatal('Channel id is not valid')
        return this._id
    }

    async name() {
        if (!this._name)
            await this._fetch_from_discord()
                .catch(console.error)
        return this._name
    }

    async parent_channel() {
        if (!this._parent_channel)
            await this._fetch_from_discord()
                .catch(console.error)
        return this._parent_channel
    }

    /**
     * Get url to this channel
     * @return {string}
     */
    url() {
        return `https://discord.com/channels/${CONFIG.get().SERVER_ID}/${this._id}`
    }

    async _fetch_from_discord() {
        let _api_handle = DI.get()._client.channels.cache.get(this.id())
        if (!_api_handle)
            _api_handle = await DI.get()._client.channels.fetch(this.id())
                .catch(err => console.fatal(`failed to get channel ${this.id()}:`, err))

        this._fill_from_api_handle(_api_handle)
        return _api_handle
    }

    /**
     * Set channel name
     * @param new_name {string}
     */
    async set_name(new_name) {
        const handle = await this._fetch_from_discord()
            .catch(console.fatal)
        await handle.setName(new_name)
        this._name = new_name
    }

    async is_valid() {
        let _api_handle = await DI.get()._client.channels.cache.get(this.id())
        if (!_api_handle)
            _api_handle = await DI.get()._client.channels.fetch(this.id())
                .catch(() => {
                    return false
                })
        if (_api_handle)
            this._fill_from_api_handle(_api_handle)
        return !!_api_handle
    }

    _fill_from_api_handle(_api_handle) {
        this._id = _api_handle.id
        this._parent_channel = new Channel().set_id(_api_handle.parentId)
        this._type = _api_handle.type
        this._name = _api_handle.name
    }

    /**
     * @return {Promise<number[]|string[]>}
     */
    async get_messages() {
        const api_handle = await this._fetch_from_discord()
        const messages = await api_handle.messages.fetch()
            .catch(err => console.fatal(`Failed to fetch messages in channel ${this.id()} : ${err}`))

        const result_messages = []
        messages.forEach(msg => {
            result_messages.push(msg.id)
        })
        return result_messages
    }

    /**
     * Send message to this channel
     * @param message {Message}
     * @returns {Message} sent message
     */
    async send(message) {
        message.set_channel(this)
        return message.send();
    }

    /**
     * Create a thread in this channel
     * @param title {string}
     * @param private_thread {boolean}
     * @param message {Message|undefined}
     * @param tags {String[]|undefined}
     * @return {Promise<string>} channel id
     */
    async create_thread(title, private_thread = false, message = undefined, tags = undefined) {
        const api_handle = await this._fetch_from_discord()

        if (!api_handle)
            console.fatal(`Invalid thread : ${this.id()}`);

        if (title.length > 100)
            console.fatal(`Thread title is too long : ${title.length} | ${title}`)

        const selectedTags = []
        if (tags) {
            for (const tagName of tags) {
                let found = false;
                for (const tag of api_handle.availableTags) {
                    if (tag.name === tagName) {
                        selectedTags.push(tag.id);
                        found = true;
                        break;
                    }
                }
                if (!found) {

                    const availableTags = []
                    for (const tag of api_handle.availableTags)
                        availableTags.push(tag.name)
                    console.fatal(`There is no tag ${tagName} available in the forum ${await this.name()}. Available tags :\n${availableTags}`);
                }
            }
        }
        const thread = await api_handle.threads.create({
            name: title,
            type: private_thread ? ChannelType.PrivateThread : ChannelType.Thread,
            reason: 'create modo ticket',
            message: message ? await message._output_to_discord() : undefined,
            appliedTags: tags ? selectedTags : undefined
        }).catch(err => console.fatal('Failed to create thread : ', err));

        return String(thread.id);
    }
}

module.exports = {Channel}