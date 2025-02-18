const DI = require('./discord_interface')
const Discord = require('discord.js')
const CONFIG = require("../config");
const assert = require("assert");

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
        this._profile_picture = image_url
        return this
    }

    /**
     * Set id
     * @param user_id {number}
     * @return {User}
     */
    set_id(user_id) {
        if (typeof(user_id) !== 'number' && typeof(user_id) !== 'string')
            console.fatal("Id should be a number : ", user_id);
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
        return `${await this.name()}`
    }

    /**
     * Get a string mention to this user
     * @return {string}
     */
    mention() {
        return '<@' + this._id + '>'
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

    /**
     * Send message in DM
     * @param message {Message}
     */
    async send(message) {
        const user = await this._fill_internal()
        user.send(await message._output_to_discord())
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
        return user
    }

    async add_role(role_id) {
        const user = await this._fetch_guild_user()
        user.roles.add(await this._get_role_internal(role_id))
    }

    async remove_role(role_id) {
        const user = await this._fetch_guild_user()
        user.roles.remove(await this._get_role_internal(role_id))
    }

    async _get_role_internal(role_id) {
        const guild = DI.get()._client.guilds.cache.get(CONFIG.get().SERVER_ID)
        let role = guild.roles.cache.get(role_id)
        if (!role)
            role = await guild.roles.fetch(role_id)
                .catch(err => console.error(`Failed to find role ${err}`))
        return role
    }

    /**
     * Kick user
     * @param reason {string}
     * @return {Promise<void>}
     */
    async kick(reason) {
        const member = await this._fetch_guild_user()
        console.warning(`${await this.name()} a été expulsé (${reason})`)
        await member.kick(reason)
            .catch(err => console.error(`Failed to kick user : ${err}`))
    }

    async _fetch_guild_user() {
        const guild = DI.get()._client.guilds.cache.get(CONFIG.get().SERVER_ID)
        let member = guild.members.cache.get(this._id)
        if (!member)
            member = await guild.members.fetch(this._id)
                .catch(err => console.error(`Failed to find role ${err}`))
        return member
    }
}

module.exports = {User}