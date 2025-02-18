const fs = require("fs")
const CONFIG = require('../../config')
const logger = require('../../utils/logger')
logger.init_logger()

function string_to_unicode(input) {
    let str = ''
    for (let i = 0; i < input.length; ++i) {
        if ((input.charCodeAt(i) >= 32 && input.charCodeAt(i) <= 127)) {
            str += input.charAt(i)
        }
        else if (input.charAt(i) === '\n') // replace line break with space
            str += ' '
    }

    str = str.replaceAll(/<:.*:[0-9]*>/g, '') // remove emotes
    str = str.replaceAll(/<[@#]!?[0-9]*>/g, '') // remove mentions
    str = str.replaceAll(/https?:\/\/\S*/g, '')
    return str
}

function get_files(channel = null) {

    let discussion = []

    const channels = channel ? [channel] : fs.readdirSync(`${CONFIG.get().SAVE_DIR}/message_history`)

    for (const channel of channels) {
        console.info(`Start process of channel #${channel}`)
        const channel_conversations = []
        for (const chunk of fs.readdirSync(`${CONFIG.get().SAVE_DIR}/message_history/${channel}`).sort((a, b) => Number(a.split('.')[0]) - Number(b.split('.')[0]))) {
            const data = JSON.parse(fs.readFileSync(`${CONFIG.get().SAVE_DIR}/message_history/${channel}/${chunk}`, "utf-8"))
            for (const [k, v] of Object.entries(data)) {
                const text = string_to_unicode(v.t.replaceAll(/[éèê]/g, 'e').replaceAll('ç', 'c'))
                if (text.length !== 0)
                    channel_conversations.push(/*string_to_unicode(v.u.split('#')[0]) + " : " + */text + '\n')
            }
        }
        discussion = discussion.concat(channel_conversations.reverse())
    }

    let string = ''
    for (const text of discussion)
        string += `${text}`

    fs.writeFileSync(`${CONFIG.get().SAVE_DIR}/generated_save.txt`, string, "utf-8")

    console.validate('Saved', discussion.length, 'messages')
}
get_files('543932597173223425')

//get_files('447767547597684738')