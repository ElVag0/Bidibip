// MODULE MODO
const {CommandInfo} = require("../../utils/interactionBase")
const {Thread} = require("../../utils/thread");
const {ModuleBase} = require("../../utils/module_base");
const {Channel} = require("../../utils/channel");
const {Message} = require("../../utils/message");
const {Embed} = require("../../utils/embed");
const {User} = require("../../utils/user");


class Module extends ModuleBase {
    constructor(create_infos) {
        super(create_infos)

        this.client = create_infos.client

        // Command declaration
        this.commands = [
            new CommandInfo('modo', 'Ouvre un canal directe avec la modération', this.modo)
                .set_member_only(),
            new CommandInfo('close', 'Ferme le canal de communication directe avec la modération', this.close)
                .set_admin_only()
        ]
        if (!this.module_config.opened_tickets) {
            this.module_config.opened_tickets = {};
        }
        if (!this.module_config.channel_users) {
            this.module_config.channel_users = {};
        }
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async modo(command) {
        const user = command.author();

        const message = new Message()
            .set_text(`${user.mention()}<@&${this.app_config.ADMIN_ROLE_ID}>`)
            .add_embed(
            new Embed()
                .set_embed_author_name(`${await user.name()} < A l'aide ! 🖐`)
                .set_author(user)
                .set_title(`Canal de communication ouvert 🤖`)
                .set_color('#b91212')
                .set_description(`Tu es maintenant en communication directe avec les <@&${this.app_config.ADMIN_ROLE_ID}>.\nA toi de nous dire ce qui ne va pas.`)
        )

        if (this.module_config.opened_tickets[user.id()]) {
            if (!await new Channel().set_id(this.module_config.opened_tickets[user.id()]).is_valid())
                delete this.module_config.opened_tickets[user.id()];
        }

        if (!this.module_config.opened_tickets[user.id()]) {
            await this.open_channel(user, message);
        }
        else {
            const channel = new Thread().set_id(this.module_config.opened_tickets[user.id()]);
            message.set_channel(channel)
            await channel.make_visible_to_user(user, true);
            await message.send();
        }

        const opened_channel = new Channel().set_id(this.module_config.opened_tickets[user.id()]);
        await command.reply(new Message().add_embed(new Embed().set_title('Canal de communication ouvert').set_description(`Parle avec la modération ici : ${opened_channel.url()}`)).set_client_only())
    }

    /**
     * @param command {CommandInteraction}
     * @return {Promise<void>}
     */
    async close(command) {
        const channel = command.channel();

        if (String((await channel.parent_channel()).id()) !== this.app_config.TICKET_CHANNEL_ID) {
            await command.reply(new Message().set_text('Pas un thread de ticket à la moderation').set_client_only());
            return;
        }

        const thread = new Thread().set_id(channel.id());

        if (this.module_config.channel_users[thread.id()]) {
            await thread.make_visible_to_user(new User().set_id(this.module_config.channel_users[thread.id()]), false);
        }

        if (!await thread.archived()) {
            await command.reply(new Message().set_text('Conversation archivée').set_client_only());
            await thread.archive()
                .catch(err => console.fatal('failed to archive thread : ', err));
        }
        else {
            await command.reply(new Message().set_text('Conversation déjà archivée').set_client_only());
        }
    }

    async open_channel(user, message) {
        const ticket_channel = new Channel().set_id(this.app_config.TICKET_CHANNEL_ID);

        const thread = new Thread().set_id(await ticket_channel.create_thread(await user.name(), true)
            .catch(err => console.fatal('Failed to create thread for ticket : ', err)));

        message.set_channel(thread);
        await message.send()

        this.module_config.opened_tickets[user.id()] = thread.id()
        this.module_config.channel_users[thread.id()] = user.id()
        await thread.make_visible_to_user(user, true);
        this.save_config()
    }
}

module.exports = {Module}