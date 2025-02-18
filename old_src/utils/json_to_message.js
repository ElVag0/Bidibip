const {Message} = require("./message");
const {Embed} = require("./embed");
const {InteractionRow} = require("./interaction_row");
const {Button} = require("./button");
const {Channel} = require("./channel");


async function json_to_message(json) {
    return new Promise(async (success, error) => {
        let data = null
        try {
            data = JSON.parse(json)
        } catch (err) {
            return error(`Le fichier JSON fourni n'est pas valide : ${err}`)
        }

        if (!data['messages'])
            return error(`Le fichier JSon doit contenir un champ 'messages' contenant une liste de messages.`)

        const messages = []
        for (const m of data.messages) {

            const new_message = new Message()

            if (!(m['textes'] && m.textes.length !== 0) && !(m['embeds'] && m.embeds.length !== 0))
                return error(`Chaque message doit contenir au moins un champ 'textes' ou 'embeds'`)

            if (m['textes']) {
                let text = ''
                for (const t of m.textes) {
                    text += `${String(t)}\n`
                }
                new_message.set_text(text)
            }
            if (m['embeds']) {
                for (const e of m.embeds) {
                    if (!e['titre'])
                        return error(`Chaque embed doit contenir un champ 'titre'`)

                    const embed = new Embed()
                        .set_title(String(e.titre))
                    if (e.description)
                        embed.set_description(String(e.description))
                    new_message.add_embed(embed)
                }
            }

            if (m['interactions']) {
                for (const interaction of m.interactions) {
                    const row = new InteractionRow()
                    const b = interaction['bouton']
                    if (b) {
                        if (!b['type'] || !b['identifiant'] || !b['texte'])
                            return error(`Chaque bouton doit contenir un champ 'identifiant', 'type' et 'texte`)
                        let type = Button.Primary
                        switch (b.type) {
                            case 'Primary':
                                type = Button.Primary;
                                break
                            case 'Secondary':
                                type = Button.Secondary;
                                break
                            case 'Success':
                                type = Button.Success;
                                break
                            case 'Danger':
                                type = Button.Danger;
                                break
                            case 'Link':
                                type = Button.Link;
                                break
                        }
                        const button = new Button()
                            .set_label(String(b.texte))
                            .set_id(String(b.identifiant))
                            .set_url(String(b.url))
                            .set_type(String(b.type))
                        row.add_button(button)
                    }
                    new_message.add_interaction_row(row)
                }
            }

            messages.push(new_message)
        }
        success(messages)
    })
}

module.exports = {json_to_message}