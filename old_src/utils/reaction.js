const {Message} = require("./message");

class Reaction {
    constructor(_api_handle) {
        if (_api_handle)
            this._fill_from_api_handle(_api_handle)
    }

    _fill_from_api_handle(_api_handle) {
        this._message = new Message(_api_handle.message)
        this._emoji = _api_handle._emoji.name
        this._api_handle = _api_handle
    }

    /**
     * Get message
     * @return {Message}
     */
    message() {
        return this._message
    }

    /**
     * Remove user from reactions
     * @param user {User}
     * @return {Promise<void>}
     */
    async remove_user(user) {
        await this._api_handle.users.remove(user.id())
    }

    /**
     * Add user from reactions
     * @param user {User}
     * @return {Promise<void>}
     */
    async add_user(user) {
        await this._api_handle.users.add(user.id())
    }

    /**
     * Get emoji
     * @return {string}
     */
    emoji() {
        return this._emoji
    }
}

module.exports = {Reaction}