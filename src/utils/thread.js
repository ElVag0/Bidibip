const {Channel} = require("./channel");
const {User} = require("./user");

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

    _fill_from_api_handle(_api_handle) {
        super._fill_from_api_handle(_api_handle)
        this._owner = new User().set_id(_api_handle.ownerId)
    }
}

module.exports = {Thread}