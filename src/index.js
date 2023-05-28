const CONFIG = require('./config')
const DI = require('./utils/discord_interface')
const MODULE_MANAGER = require('./core/module_manager')
const {Client, Partials, GatewayIntentBits} = require('discord.js')

require('./utils/logger').init_logger()

// Suppress "(node:3468) ExperimentalWarning: buffer.File is an experimental feature and might change at any time"
const initial_emit = process.emit;
process.emit = function (name, data, ...args) {
    if (name === `warning` && typeof data === `object` && data.name === `ExperimentalWarning`) return false
    return initial_emit.apply(process, arguments);
};

/*
GIT AUTO-UPDATER
 */
const AutoGitUpdate = require('auto-git-update')
const {Message} = require("./utils/message");
const {Channel} = require("./utils/channel");
const updater = new AutoGitUpdate({
    repository: 'https://github.com/Unreal-Engine-FR/Bidibip',
    branch: CONFIG.get().UPDATE_FOLLOW_BRANCH,
    tempLocation: CONFIG.get().CACHE_DIR + '/updater/',
    exitOnComplete: true
})

// Check for update before starting the app
updater.autoUpdate()
    .then(result => {
        if (result) {
            console.validate('Application up to date ! Starting client...')

            /*
            CREATE DISCORD CLIENT
             */
            const client = new Client(
                {
                    partials: [Partials.Channel],
                    intents: [
                        GatewayIntentBits.Guilds,
                        GatewayIntentBits.GuildMessages,
                        GatewayIntentBits.GuildMembers,
                        GatewayIntentBits.MessageContent,
                        GatewayIntentBits.GuildMessageReactions,
                        GatewayIntentBits.DirectMessages
                    ] // This is the action the bot will be able to do
                }
            )

            /*
            START DISCORD CLIENT
             */
            client.on('ready', async () => {
                await DI.init(client, updater) // Setup interface
                DI.get().module_manager = MODULE_MANAGER.get().init() // load modules
                new Message() // Send welcome message
                    .set_text('Coucou tout le monde ! :wave:')
                    .set_channel(new Channel().set_id(CONFIG.get().LOG_CHANNEL_ID))
                    .send()
                    .catch(err => console.fatal(`failed to send welcome message : ${err}`))
            })
            client.login(CONFIG.get().APP_TOKEN)
                .then(_token => {
                    console.validate(`Successfully logged in !`)
                })
                .catch(error => console.fatal(`Failed to login : ${error}`))

        } else {
            console.warning('Application outdated, waiting for update...')
        }
    })
    .catch(err => console.error(`Update failed : ${err}`))
