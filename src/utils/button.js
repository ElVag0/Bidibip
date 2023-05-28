const {ButtonBuilder} = require("discord.js");

class Button {
    static Primary = 1
    static Secondary = 2
    static Success = 3
    static Danger = 4
    static Link = 5

    constructor(_api_handle) {
        this._type = Button.Primary
        this._enabled = true
        if (_api_handle) {
            this._type = _api_handle.data.style
            this._label = _api_handle.data.label
            this._id = _api_handle.data.custom_id
            this._url = _api_handle.url
            this._enabled = !_api_handle.disabled
        }
    }

    /**
     * Set button unique id
     * @param id {string}
     * @returns {Button}
     */
    set_id(id) {
        this._id = id
        return this
    }

    /**
     * Set button label
     * @param label {string}
     * @returns {Button}
     */
    set_label(label) {
        this._label = label
        return this
    }

    /**
     * Set button display type
     * @param type {number}
     * @returns {Button}
     */
    set_type(type) {
        this._type = type
        return this
    }

    /**
     * Set button link's url
     * @param url {string}
     * @returns {Button}
     */
    set_url(url) {
        this._url = url
        return this
    }

    /**
     * Set button enabled
     * @param enabled {boolean}
     * @returns {Button}
     */
    set_enabled(enabled) {
        this._enabled = enabled
        return this
    }

    _to_discord_item() {
        const button = new ButtonBuilder()
            .setLabel(this._label)
            .setStyle(this._type)
            .setDisabled(!this._enabled)

        if (this._type === Button.Link) {
            if (!this._url)
                console.fatal(`link button require an url :`, this)
            if (this._id)
                console.fatal(`link button doesn't allow custom ID:`, this)
            button.setURL(this._url)
        }
        else {
            if (!this._id)
                console.fatal(`button require a custom ID :`, this)
            button.setCustomId(this._id)
        }

        return button
    }
}

module.exports = {Button}