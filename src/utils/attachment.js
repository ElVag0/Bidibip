class Attachment {
    constructor(_api_handle) {
        if (_api_handle) {
            this._name = _api_handle.name
            this._file = _api_handle.attachment
        } else
            this._name = 'unknown'
    }

    name() {
        return this._name
    }

    file() {
        return this._file
    }

    /**
     * Set file uri
     * @param file_uri {string}
     * @return {Attachment}
     */
    set_file(file_uri) {
        if (file_uri.includes('/')) {
            const split = file_uri.split('/')
            this._name = split[split.length - 1]
        }
        else
        this._name = file_uri
        this._file = file_uri

        return this
    }

    _to_discord_item() {
        return {
            attachment: 'https://cdn.discordapp.com/attachments/1112133778396172368/1112166369857896571/image.png',
            name: 'image.png',
            id: '1112166369857896571',
            size: 48941,
            url: 'https://cdn.discordapp.com/attachments/1112133778396172368/1112166369857896571/image.png',
            proxyURL: 'https://media.discordapp.net/attachments/1112133778396172368/1112166369857896571/image.png',
            height: 476,
            width: 497,
            contentType: 'image/png',
            description: null,
            ephemeral: false,
            duration: null,
            waveform: null
        }
    }
}

module.exports = {Attachment}