const {Button} = require("../../utils/button");
const {Message} = require("../../utils/message");
const {InteractionRow} = require("../../utils/interaction_row");
const {hire} = require("./hire");
const {search} = require("./search");
const {AdvertisingKinds} = require("./types");

/**
 * @param baseText {string}
 * @return string
 */
function parseUrls(baseText) {
    let elements = baseText.split(' ');
    for (let j in elements) {
        let sub_elements = elements[j].split('\n');
        for (const i in sub_elements) {
            let is_url = false;
            let display_url = sub_elements[i];
            let initial = 'https://';
            if (display_url.startsWith('http')) {
                if (sub_elements[i].startsWith('https')) {
                    display_url = display_url.substring(8);
                } else {
                    initial = 'http://';
                    display_url = display_url.substring(7);
                }
                is_url = true;
            }
            /* Mailto are not supported in discord
            else if (display_url.indexOf('@') > -1) {
                is_url = true;
                initial = 'mailto:'
            }
             */
            else {
                const split = display_url.split('/')[0].split('.');
                if (split.length > 1) {
                    switch (split[split.length - 1]) {
                        case 'fr':
                        case 'eu':
                        case 'com':
                        case 'uk':
                        case 'io':
                        case 'org':
                        case 'net':
                        case 'ru':
                        case 'cn':
                        case 'lol':
                        case 'de':
                        case 'jp':
                        case 'ca':
                            is_url = true;
                            break;
                        case 'gg':
                            is_url = true;
                            display_url = display_url.split('/')[0];
                            break;
                        default:
                            break;
                    }
                }
            }
            if (is_url)
                sub_elements[i] = `[${display_url}](${initial}${display_url})`;
        }
        elements[j] = sub_elements.join('\n');
    }
    return elements.join(' ')
}

class AdvertisingSetup {
    /**
     * @param ctx {ModuleBase}
     * @param thread {Thread}
     * @param author {User}
     */
    constructor(ctx, thread, author) {
        this.thread = thread;
        this.author = author;
        this.ctx = ctx;

        this.start()
            .catch((error) => {
                console.fatal(`Announcement failed : ${error}`)
            })
        if (ctx['advertising_setups'][thread.id()])
            delete ctx['advertising_setups'][thread.id()];
        ctx['advertising_setups'][thread.id()] = this;
    }

    async start() {
        switch (await this.askChoice('Que cherches tu ?', ['👩‍🔧 Je cherche du travail', '🕵️‍♀️ Je recrute'])) {
            case 0:
                await search(this, await this.askChoice('Quel type de contrat recherches-tu ?', AdvertisingKinds.allTexts()));
                break;
            case 1:
                await hire(this, await this.askChoice('Quel type de contrat proposes-tu ?', AdvertisingKinds.allTexts()));
                break;
            case null:
                console.fatal('Invalid response');
                break;
            default:
                console.fatal(`Unhandled response`);
        }
    }

    async askChoice(text, questions) {
        const baseMessage = new Message().set_text(`## ▶  ${text}`);

        let interaction = null;
        for (const i in questions) {
            if (i % 5 === 0) {
                if (interaction)
                    baseMessage.add_interaction_row(interaction);
                interaction = new InteractionRow();
            }
            interaction.add_button(new Button().set_id(questions[i]).set_label(questions[i]));
        }
        const message = await this.thread.send(baseMessage.add_interaction_row(interaction));

        return await new Promise((resolve) => {
            this.ctx.bind_button(message, async (interaction) => {
                let selected = null;
                for (const i in questions) {
                    const id = questions[i];
                    if (interaction.button_id() === id) {
                        (await message.get_button_by_id(id)).set_type(Button.Success).set_enabled(false);
                        selected = Number(i);
                    } else
                        (await message.get_button_by_id(id)).set_type(Button.Secondary).set_enabled(false);
                }
                await message.update(message)
                await interaction.skip();
                resolve(selected);
            })
        });
    }

    async askUser(question, max_length = null, required = true) {
        const message = new Message().set_text(`## ▶  ${question}`);
        if (!required) {
            message.add_interaction_row(new InteractionRow().add_button(new Button().set_id("no").set_label("✖ Je ne suis pas concerné").set_type(Button.Secondary)));
        }
        const sent = await this.thread.send(message);

        return await new Promise((resolve) => {
            if (!required) {
                this.ctx.bind_button(sent, async (interaction) => {
                    if (interaction.button_id() === 'no') {
                        await interaction.skip();
                        await sent.delete()
                        resolve(null);
                    }
                })
            }
            this.waiting_message = async (text) => {
                if (text.length > max_length) {
                    this.thread.send(new Message().set_text(`❌ Ton message est trop long et doit faire moins de ${max_length} caractères`));
                    return;
                }
                resolve(parseUrls(text));
                delete this.waiting_message;
            }
        });
    }

    /**
     * @param message {Message}
     */
    async received_message(message) {
        if (this.waiting_message)
            await this.waiting_message(await message.text())
    }
}

module.exports = {AdvertisingSetup}