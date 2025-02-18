const {Channel} = require("./channel");
const {User} = require("./user");
const {Message} = require("./message");

class Thread extends Channel {
    constructor(_api_handle) {
        super(_api_handle)
        if (_api_handle) {
            this._fill_from_api_handle(_api_handle)
        }
    }

    async owner() {
        if (!this._owner)
            await this._fetch_from_discord()
        return this._owner
    }

    /**
     * @return {Promise<boolean>}
     */
    async archived() {
        if (typeof this._archived !== 'boolean')
            await this._fetch_from_discord();
        return this._archived;
    }

    /**
     * @return {Promise<boolean>}
     */
    async archive() {
        const api_handle = await this._fetch_from_discord();
        if (!api_handle.archived)
            await api_handle.setArchived(true);
    }

    /**
     *
     * @param user {User}
     * @param visible {boolean}
     * @return {Promise<void>}
     */
    async make_visible_to_user(user, visible = true) {
        const api_handle = await this._fetch_from_discord();
        if (visible) {
            await api_handle.members.add(user.id())
        }
        else
            await api_handle.members.remove(user.id())
    }

    _fill_from_api_handle(_api_handle) {
        super._fill_from_api_handle(_api_handle)
        this._owner = new User().set_id(_api_handle.ownerId)
        this._archived = _api_handle.archived;
    }

    /**
     * Delete this thread definitively
     */
    async delete() {
        const api_handle = await this._fetch_from_discord();
        if (api_handle) {
            console.warning(`Deleted thread ${await this.name()}`)
            await api_handle.delete();
        }
    }

    first_message() {
        return new Message().set_id(this._id).set_channel(this)
    }
}

module.exports = {Thread}