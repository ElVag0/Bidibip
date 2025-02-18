const Discord = require("discord.js");

class Embed {
    constructor(_api_handle) {
        this.fields = []

        if (_api_handle)
            this._from_discord_api(_api_handle)
    }

    /**
     * Set embed title text
     * @param title {string}
     * @returns {Embed}
     */
    set_title(title) {
        this.title = title
        return this
    }

    /**
     * Set embed author name override
     * @param name {string}
     * @returns {Embed}
     */
    set_embed_author_name(name) {
        this.embed_author_name = name
        return this
    }

    /**
     * Set author url
     * @param url {string}
     * @returns {Embed}
     */*
    set_author_url(url) {
        this._author_url = url
        return this
    }

    /**
     * Set embed title author
     * @param user {User}
     * @returns {Embed}
     */
    set_author(user) {
        this._author = user
        return this
    }

    /**
     * Set embed description text
     * @param description {string}
     * @returns {Embed}
     */
    set_description(description) {
        this.description = description
        return this
    }

    /**
     * Set embed thumbnail image
     * @param thumbnail {string} url
     * @returns {Embed}
     */
    set_thumbnail(thumbnail) {
        this.thumbnail = thumbnail
        return this
    }

    /**
     * Set embed image
     * @param image {string} url
     * @returns {Embed}
     */
    set_image(image) {
        this._image = image
        return this
    }

    /**
     * @param color {string}
     * @returns {Embed}
     */
    set_color(color) {
        this._color = color
        return this
    }

    /**
     * Add field to embed (a field contains a name and a description)
     * @param name {string} field name
     * @param value {string} field text
     * @param inline {boolean} should be displayed as inline bloc
     * @returns {Embed}
     */
    add_field(name, value, inline = false) {
        this.fields.push({name: name, value: value, inline: inline})
        return this
    }

    _from_discord_api(_api_handle) {
        this.title = _api_handle.title
        this.description = _api_handle.description
        this.thumbnail = _api_handle.thumbnail ? _api_handle.thumbnail.url : null
        this._image = _api_handle.image
        this._color = _api_handle.color
        if (_api_handle.author) {
            this._author = {
                name: async () => _api_handle.author.name,
                profile_picture: async () => _api_handle.author.iconURL
            }
            this._author_url = _api_handle.author.url
        }
        if (_api_handle.fields)
            for (const field of _api_handle.fields) {
                this.add_field(field.name, field.value, field.inline)
            }
    }

    is_valid() {
        return (this.title || this._author)
    }

    async _to_discord_api() {
        try {
            if (!this.is_valid())
                console.fatal(`embed is not valid : `, this)

            const embed = new Discord.EmbedBuilder()
                .setDescription(!this.description || this.description.length === 0 ? null : this.description)
                .setThumbnail(this.thumbnail)
                .setImage(this._image)

            if (this._color)
                embed.setColor(this._color)

            if (this.title)
                embed.setTitle(this.title)

            if (this._author) {
                embed.setAuthor({
                    name: this.embed_author_name ? this.embed_author_name : this.title ? this.title : await this._author.name(),
                    url: this._author_url,
                    iconURL: await this._author.profile_picture() ? await this._author.profile_picture() : null,
                })
            }

            for (const field of this.fields) {
                if (field.value.length > 1024) {
                    console.fatal('Embed fields cannot have more than 1024 characters :', field)
                }
                embed.addFields(field)
            }
            return embed
        } catch (err) {
            console.fatal(`Embed is not valid : ${err}\nEmbed :`, this, '\nError : ', err)
        }
    }
}

module.exports = {Embed}