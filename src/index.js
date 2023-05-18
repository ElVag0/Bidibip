const MODULE_MANAGER = require('./module_manager').get()
const CONFIG = require('./config').get()
require('./logger').get()

const {patch_client} = require('./discord_interface')

/*
GIT AUTO-UPDATER
 */
const AutoGitUpdate = require('auto-git-update')
const updater = new AutoGitUpdate({
    repository: 'https://github.com/Unreal-Engine-FR/Bidibip',
    branch: 'feature/bidibip-v3',
    tempLocation: '../tmp/',
    exitOnComplete: true
});
updater.autoUpdate()
    .then(result => console.log(result ? `Update complete !` : `Update failed`))
    .catch(err => console.log(`Update failed : ${err}`))

/*
CREATE DISCORD CLIENT
 */
const Discord = require('discord.js');
const client = new Discord.Client(
    {
        partials: [Discord.Partials.Channel],
        intents: [
            Discord.GatewayIntentBits.Guilds,
            Discord.GatewayIntentBits.GuildMessages,
            Discord.GatewayIntentBits.GuildMembers,
            Discord.GatewayIntentBits.MessageContent,
            Discord.GatewayIntentBits.DirectMessages
        ]
    }
)

/*
START DISCORD CLIENT
 */
client.on('ready', () => {
    patch_client(client)
    MODULE_MANAGER.init(client)
})
client.login(CONFIG.TOKEN)
    .then(_token => {
        console.log(`Successfully logged in !`)
    })
    .catch(error => console.log(`Failed to login : ${error}`))