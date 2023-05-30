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
}

module.exports = {Attachment}