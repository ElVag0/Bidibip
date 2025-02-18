const {Button} = require("./button");
const {ActionRowBuilder} = require("discord.js");

class InteractionRow {
    constructor(_api_handle) {

        this._items = []

        if (_api_handle) {
            for (const component of _api_handle.components) {
                if (component.type === 2) {
                    this.add_button(new Button(component))
                }
                else {
                    console.warning('New interaction row item :', _api_handle)
                    console.fatal('NOT IMPLEMENTED YET')
                }
            }
        }
    }

    /**
     * Added button
     * @param button {Button}
     */
    add_button(button) {
        this._items.push(button)
        return this
    }

    _to_discord_row() {
        const discord_row = new ActionRowBuilder()
        for (const item of this._items)
            discord_row.addComponents(item._to_discord_item())
        return discord_row
    }
}

module.exports = {InteractionRow}