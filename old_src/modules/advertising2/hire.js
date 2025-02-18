const {AdvertisingKinds} = require("./types");
const {Message} = require("../../utils/message");
const {Embed} = require("../../utils/embed");
const {InteractionRow} = require("../../utils/interaction_row");
const {Button} = require("../../utils/button");
const {Channel} = require("../../utils/channel");


function format_remote(remote_id) {
    switch (remote_id) {
        case 0:
            return "🌍 Distanciel uniquement";
        case 1:
            return "🏙 Travail en Présentiel avec Flexibilité Télétravail";
        case 2:
            return "🤷‍♀️ Distanciel ou présentiel au choix";
        case 3:
            return "🏣 Présentiel uniquement";
    }
}

async function format_message(data) {
    const message = new Message();
    const embed = new Embed()
            .set_author(data.author)
            .set_color(data.color)
            .set_title(data.title + " Chez " + (data.studio || await data.author.name()))
            .set_description(format_remote(data.allow_remote))

    if (data.duration)
        embed.add_field('Durée du contrat :', data.duration, data.allow_remote !== 0);

    if (data.pay)
        embed.add_field('Salaire :', data.pay, true);

    if (data.where)
        embed.add_field('Localisation', data.where);
    message.add_embed(embed);

    message.add_embed(new Embed()
        .set_title('Responsabilités')
        .set_color(data.color)
        .set_description(data.responsibility));

    message.add_embed(new Embed()
        .set_title('Qualifications')
        .set_color(data.color)
        .set_description(data.qualification));

    const last = new Embed()
        .set_title('Autres informations')
        .set_color(data.color)
        .add_field("Comment postuler :", data.how, true);

    if (data.links)
        last.add_field("Liens utils :", data.links);

    message.add_embed(last);

    return message;
}

/**
 * @param ctx {AdvertisingSetup}
 * @param kind {number}
 */
async function hire(ctx, kind) {
    const data = {kind, author: ctx.author, color: AdvertisingKinds.get(kind).color};
    data.title = await ctx.askUser("Donnes un titre à ton annonce :", 128);
    data.studio = await ctx.askUser("Quel est le nom de ton studio ou de ton entreprise ?", 100, false);
    data.allow_remote = await ctx.askChoice("Peut-on travailler en distanciel ?", ["🌍 Distanciel possible", "🏙 Présentiel flexible", "🤷‍♀️ Au choix", "🏣 Présentiel uniquement"]);

    if (data.allow_remote !== 0)
        data.where = await ctx.askUser("Où se situent les bureaux ? (ville / pays)", 1024);

    if (kind !== AdvertisingKinds.PAID_UNLIMITED.type && kind !== AdvertisingKinds.UNPAID.type)
        data.duration = await ctx.askUser("Quelle est la durée du contrat ?", 1024);

    if (kind !== AdvertisingKinds.INTERN_FREE.type && kind !== AdvertisingKinds.UNPAID.type)
        data.pay = await ctx.askUser("Quelle est le salaire (taux horraire) ?", 1024);

    data.responsibility = await ctx.askUser("Quelles seront les responsabilités et le travail à réaliser ?", 1024);
    data.qualification = await ctx.askUser("Quelles sont les qualifications recherchées ?", 1024);
    if (kind === AdvertisingKinds.UNPAID.type)
        data.counterpart = await ctx.askUser("Personne ne veut travailler gratuitement. Précises ce que gagne la personne gagne à travailler avec toi !", 1024);
    data.how = await ctx.askUser("Comment peut-on postuler ? discord, mail (préciser le mail)...", 1024);
    data.links = await ctx.askUser("Tu peux ajouter quelques liens utils si tu le souhaites (portfolio, site web etc..)",1024, false);

    const formatted_message = (await format_message(data)).set_text("Vérifie ton annonce avant qu'elle soit postée");

    formatted_message.add_interaction_row(new InteractionRow()
        .add_button(new Button().set_id('yes').set_label('Poster l\'annonce').set_type(Button.Success))
        .add_button(new Button().set_id('no').set_label('Recommencer').set_type(Button.Danger)))

    const result = await ctx.thread.send(formatted_message);
    ctx.ctx.bind_button(result, async (interaction) => {

        if (interaction.button_id() === 'yes') {

            const authorTitle = ` (par ${await ctx.author.name()})`;
            let baseTitle = `${AdvertisingKinds.get(kind).emote} Offre ${AdvertisingKinds.get(kind).text_small} : ${data.title}${data.studio ? " chez " + data.studio : ""}`;

            if (baseTitle.length > 100 - authorTitle.length)
                baseTitle = `${baseTitle.substring(0, 100 - 3 - authorTitle.length)}...`;

            const tags = AdvertisingKinds.getTags(kind);
            tags.push("Offre d'emploi");
            switch (data.allow_remote) {
                case 0:
                    tags.push('Distanciel')
                    break;
                case 2:
                    tags.push('Présentiel flexible')
                    break;
                case 2:
                    tags.push('Distanciel')
                    tags.push('Présentiel')
                    break;
                case 3:
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


module.exports = {hire};