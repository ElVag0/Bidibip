const {Message} = require("../../utils/message");
const {Embed} = require("../../utils/embed");
const {AdvertisingKinds} = require("./types");
const {InteractionRow} = require("../../utils/interaction_row");
const {Button} = require("../../utils/button");
const {Channel} = require("../../utils/channel");


function format_remote(remote_id) {
    switch (remote_id) {
        case 0:
            return "🌍 En distanciel uniquement";
        case 1:
            return "‍🤷‍ Distanciel ou présentiel possible";
        case 2:
            return "🏣 Présentiel uniquement";
    }
}

async function format_message(data) {
    const message = new Message();
    const embed = new Embed()
        .set_author(data.author)
        .set_color(data.color)
        .set_title(`${await data.author.name()} : ${data.title}`)
        .set_description(format_remote(data.allow_remote))

    if (data.duration)
        embed.add_field('Durée du contrat :', data.duration, data.allow_remote !== 0);

    if (data.where)
        embed.add_field('Je cherche du travail dans cette région :', data.where);
    message.add_embed(embed);

    message.add_embed(new Embed()
        .set_title('Compétences')
        .set_color(data.color)
        .set_description(data.skills));

    message.add_embed(new Embed()
        .set_title('Experiences passées')
        .set_color(data.color)
        .set_description(data.xp));

    const last = new Embed()
        .set_title('Autres informations')
        .set_color(data.color)
        .add_field("Me contacter :", data.how, true);

    if (data.cv)
        last.add_field("Curiculum Vitae :", data.cv, true);
    if (data.lm)
        last.add_field("Lettre de motivation :", data.lm, true);
    if (data.folio)
        last.add_field("Portfolio :", data.lm, true);

    if (data.links)
        last.add_field("Liens utils :", data.links);

    message.add_embed(last);

    return message;
}

/**
 * @param ctx {AdvertisingSetup}
 * @param kind {number}
 */
async function search(ctx, kind) {
    const data = {kind, author: ctx.author, color: AdvertisingKinds.get(kind).color};
    data.title = await ctx.askUser("Donne un titre à ton annonce ou précise ta spécialité :", 128);
    data.allow_remote = await ctx.askChoice("Souhaites-tu travailler à distance ou en présentiel ?", ["🌍 En distanciel", "🤷‍ Télétravail flexible", "🏣 Présentiel uniquement"]);

    if (data.allow_remote !== 0)
        data.where = await ctx.askUser("Dans quelle région cherches-tu du travail ?", 1024);

    if (kind !== AdvertisingKinds.PAID_UNLIMITED.type && kind !== AdvertisingKinds.UNPAID.type)
        data.duration = await ctx.askUser("Quelle est la durée idéale du contrat ?", 1024);

    data.skills = await ctx.askUser("Quelles sont tes compétences ?", 1024);
    data.xp = await ctx.askUser("Quelles sont tes experiences passées ?", 1024);
    if (kind === AdvertisingKinds.UNPAID.type)
        data.counterpart = await ctx.askUser("Que souhaites tu en contrepartie ?", 1024);
    data.how = await ctx.askUser("Comment peut-on te contacter ? (précises ton mail / discord...)", 1024);
    data.cv = await ctx.askUser("Ajoutes un lien vers ton CV :",1024, false);
    data.lm = await ctx.askUser("Ajoutes un lien vers une lettre de motivation :",1024, false);
    data.folio = await ctx.askUser("Ajoutes les url de tes portfolios :",1024, false);
    data.links = await ctx.askUser("Tu peux ajouter quelques liens utiles si tu le souhaites (portfolio, book, site web etc..)",1024, false);

    const formatted_message = (await format_message(data)).set_text("Vérifie ton annonce avant qu'elle soit postée");

    formatted_message.add_interaction_row(new InteractionRow()
        .add_button(new Button().set_id('yes').set_label('Poster l\'annonce').set_type(Button.Success))
        .add_button(new Button().set_id('no').set_label('Recommencer').set_type(Button.Danger)))

    const result = await ctx.thread.send(formatted_message);
    ctx.ctx.bind_button(result, async (interaction) => {

        if (interaction.button_id() === 'yes') {
            const authorTitle = ` (par ${await ctx.author.name()})`;
            let baseTitle = `${AdvertisingKinds.get(kind).emote} Cherche ${AdvertisingKinds.get(kind).text_small} : ${data.title}`;

            if (baseTitle.length > 100 - authorTitle.length)
                baseTitle = `${baseTitle.substring(0, 100 - 3 - authorTitle.length)}...`;

            const tags = AdvertisingKinds.getTags(kind);
            tags.push("Recherche d'emploi");
            switch (data.allow_remote) {
                case 0:
                    tags.push('Distanciel')
                    break;
                case 1:
                    tags.push('Distanciel')
                    tags.push('Présentiel')
                    break;
                case 2:
                    tags.push('Présentiel')
                    break;
            }
            await new Channel().set_id(ctx.ctx.app_config.ADVERTISING_FORUM).create_thread(
                `${baseTitle}${authorTitle}`,
                false,
                await format_message(data),
                tags)
                .then(async _thread => {
                    await interaction.skip();
                    await ctx.thread.delete();
                })
        }
        else if (interaction.button_id() === 'no') {
            await interaction.skip();
            await ctx.start();
        }
    });
}


module.exports = {search};