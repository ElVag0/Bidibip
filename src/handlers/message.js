const commands = require('../commands');
const quotes = require('../../data/quotes.json');
const {tryToSend} = require('../utils');
const {errorMessage} = require('../../data/strings');
const {Guild} = require('discord.js');

// this variable is for the bot to not spam
// the helperMessage message 
// let canSendHelp = true;
// const helperTimeout = 30 * 1000;
// const errorMessage = 

module.exports = (client, msg) => {
    const words = msg.content.split(' ');
    const command = words.shift().substr(1).toLowerCase();
    const message = words.join(' ');

    if (msg.author.bot) return;
    if (command.length === 0) return;
    if (/^!{2,}/.test(msg.content)) return;

    // ----------------------------------
    // MESSAGES IN DISCORD SERVER
    // ----------------------------------
    if (msg.guildId && msg.content[0] == '!') {
        const mapping = {
            link   : (text) => tryToSend(msg.channel, commands.link(text)),
            bidibip: () => tryToSend(msg.channel, commands.bidibip),
            quote  : (userName) => tryToSend(msg.channel, commands.quote(userName, msg)),
            quotes : () => tryToSend(msg.channel, commands.quotes()),
            say    : (text) => {
                if(msg.member.roles.cache.has(process.env.ROLE_ONLY)) {
                    tryToSend(msg.channel, commands.say(text));   
                }
            },
            add: (messageId) => {
                if(msg.member.roles.cache.has(process.env.ROLE_ONLY)) {
                    commands.add(client, messageId, msg)
                        .then(text => tryToSend(msg.channel, text))
                        .catch(err => {
                            console.log(err);
                            tryToSend(msg.channel, `\`${messageId}\` pas trouvé :/`);
                        });  
                }
             
            }
            
        };

        const quotedPersons = Object.keys(quotes).map(pseudo => pseudo.toLowerCase());

        msg.delete();

        // Unknown command, send help message
        // if (canSendHelp === true || command === 'bidibip') {
        //     canSendHelp = false;
        //     setTimeout(() => canSendHelp = true, helperTimeout);
        //     msg.author.send(helperMessage);
        // }

        // in this context, the command is a pseudo
        if (quotedPersons.includes(command)) return mapping.quote(command);

        // Handle any bug in commands
        try {
            if (command in mapping) return mapping[command](message);
        } catch (error) {
            console.log(error);
            return tryToSend(msg.channel, errorMessage);
        }
    }

    // ----------------------------------
    // MESSAGE IN DM
    // ----------------------------------
    
    const server = client.guilds.cache.get(process.env.SERVER_ID);
    if (!msg.guildId && server.member(msg.author.id) && server.members.cache.get(msg.author.id).roles.cache.has(process.env.ROLE_MEMBER)){
        const mapping = {
            paid     : () => commands.paid(client, msg, msg.author),
            unpaid   : () => commands.unpaid(client, msg, msg.author),
            freelance: () => commands.freelance(client, msg, msg.author),
        };

        // Handle any bug in commands
        try {
            if (command in mapping) return mapping[command](message);
        } catch (error) {
            console.log(error);
            return tryToSend(msg.channel, errorMessage);
        }
    }
};
