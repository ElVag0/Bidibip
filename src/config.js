const path = require('path')

class Configuration {
    constructor() {

        /* TOKENS (should always be configured in .env) */
        this.APP_TOKEN = null
        this.APP_ID = null
        this.SERVER_ID = null
        this.LOG_CHANNEL_ID = null

        // ID should always be strings
        this.ADMIN_ROLE_ID = '1108800112081244190'
        this.MEMBER_ROLE_ID = '1108799832371515443'

        /* UTILITIES */
        this.CACHE_DIR = '../Bidibip_Cache/' // Where temps data is stored
        this.SAVE_DIR = './saved/' // Where permanent data is stored
        this.LOG_KEEP_DAYS = 14 // This is the max keep duration for logs (in drays)
        this.SERVICE_ROLE = '<@&329553341275308033>' // This is the mentioned role when something goes wrong
        this.UPDATE_FOLLOW_BRANCH = 'dev' // Which branch should we follow (main for production)

        /* ADMIN-APPLICATIONS */
        this.ADMIN_APPLICATIONS_CHANNEL=null

        /* ADVERTISING MODULE */
        this.ADVERTISING_UNPAID_CHANNEL = null
        this.ADVERTISING_PAID_CHANNEL = null
        this.ADVERTISING_FREELANCE_CHANNEL = null
        this.SHARED_SHARED_CHANNEL = null

        require('dotenv').config()

        for (const [key, value] of Object.entries(process.env)) {
            this[key] = value
        }

        if (this.APP_TOKEN === null)
            console.fatal(`APP_TOKEN is null : Maybe you forget to setup .env`)

        if (!path.isAbsolute(this.SAVE_DIR)) this.SAVE_DIR = path.resolve(this.SAVE_DIR)
        if (!path.isAbsolute(this.CACHE_DIR)) this.CACHE_DIR = path.resolve(this.CACHE_DIR)

    }
}

let CONFIG = null

/**
 * Get configuration
 * @return {Configuration}
 */
function get() {
    if (CONFIG === null)
        CONFIG = new Configuration()
    return CONFIG
}

module.exports = {get}