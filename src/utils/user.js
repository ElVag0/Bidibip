const DI = require('./discord_interface')
const Discord = require('discord.js')

class User {
    constructor(_api_handle = null) {
        if (_api_handle)
            this._from_discord_user(_api_handle)
    }

    /**
     * Set full name
     * @param name {string}
     * @param discriminator {number}
     * @return {User}
     */
    set_name(name, discriminator) {
        this._name = name
        this._discriminator = discriminator
        return this
    }

    /**
     * Set profile picture
     * @param image_url {string}
     * @return {User}
     */
    set_profile_picture(image_url) {
        this.profile_picture = image_url
        return this
    }

    /**
     * Set id
     * @param user_id {number}
     * @return {User}
     */
    set_id(user_id) {
        this._id = user_id
        return this
    }

    /**
     * Get user id
     * @return {number}
     */
    id() {
        if (!this._id)
            console.fatal('User id is null')
        return this._id
    }

    /**
     * Get profile picture
     * @return {Promise<string>}
     */
    async profile_picture() {
        if (!this._profile_picture)
            await this._fill_internal()
        return this._profile_picture
    }

    /**
     * Get username
     * @return {Promise<string>}
     */
    async name() {
        if (!this._name)
            await this._fill_internal()
        return this._name
    }

    /**
     * Get user full name
     * @return {Promise<number>}
     */
    async discriminator() {
        if (!this._discriminator)
            await this._fill_internal()
        return this._discriminator
    }

    /**
     * Get user full name
     * @return {Promise<string>}
     */
    async full_name() {
        return `${await this.name()}#${await this.discriminator()}`
    }

    /**
     *
     * @param _api_handle {Discord.ClientUser}
     * @returns {User}
     * @private
     */
    _from_discord_user(_api_handle) {
        try {
            this._id = _api_handle.id
            this._name = _api_handle.username
            this._discriminator = _api_handle.discriminator
            this._profile_picture = _api_handle.displayAvatarURL() || _api_handle.avatarURL() || _api_handle.defaultAvatarURL()
        } catch (err) {
            console.fatal(`failed to parse discord user ${_api_handle} : ${err}`)
        }
        return this
    }

    async _fill_internal() {
        if (!this._id)
            console.fatal('Cannot retrieve user infs : user.id is null')
        let user = DI.get()._client.users.cache.get(this._id)
        if (!user) {
            user = DI.get()._client.users.fetch(this._id)
                .catch(err => {
                    console.fatal(`failed to fetch discord user ${this._id} : ${err}`)
                })
        }

        this._from_discord_user(user)
    }
}

module.exports = {User}