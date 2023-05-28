const DI = require("./discord_interface");

class Channel {

    static TypeChannel = 0
    static TypeThread = 11
    static TypeForum = 15

    constructor(_api_handle) {
        if (_api_handle) {
            this._fill_from_api_handle(_api_handle)
        }
    }

    set_id(id) {
        this._id = id
        return this
    }

    set_name(name) {
        this._name = name
        return this
    }

    set_parent_channel(channel) {
        this._parent_channel = channel
        return this
    }

    async type() {
        if (!this._type)
            await this._fetch_from_discord()
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
        return this._name
    }

    async parent_channel() {
        if (!this._parent_channel)
            await this._fetch_from_discord()
        return this._parent_channel
    }

    async _fetch_from_discord() {
        let _api_handle = await DI.get()._client.channels.cache.get(this.id())
        if (!_api_handle)
            _api_handle = await DI.get()._client.channels.fetch(this.id())
                .catch(err => console.fatal(`failed to get channel ${this.id()}:`, err))

        this._fill_from_api_handle(_api_handle)
    }

    _fill_from_api_handle(_api_handle) {
        this._id = _api_handle.id
        this._parent_channel = new Channel().set_id(_api_handle.parentId)
        this._type = _api_handle.type
        this._name = _api_handle.name
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
}

module.exports = {Channel}