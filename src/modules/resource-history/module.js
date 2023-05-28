// MODULE SAY
const {CommandInfo} = require("../../utils/interaction")
const simpleGit = require("simple-git")
const CONFIG = require("../../config")
const fs = require("fs");


class Module {
    constructor(create_infos) {
        this.enabled = false

        this.client = create_infos.client

        // Command declaration
        this.commands = [
            new CommandInfo('save-resource', 'Sauvegarde une nouvelle resource')
                .set_member_only()
        ]
    }

    /**
     * When module is started
     * @return {Promise<void>}
     */
    async start() {

        if (!fs.existsSync(CONFIG.get().CACHE_DIR + '/history-repos/'))
            fs.mkdirSync(CONFIG.get().CACHE_DIR + '/history-repos/', {recursive: true})

        const git = simpleGit({baseDir:CONFIG.get().CACHE_DIR + '/history-repos/'});

        const configs = await git.listConfig()

        await git.addConfig('user.name', 'Anonymous')
        await git.addConfig('user.email', '<>')

        if (!await git.checkIsRepo())
            await git.clone('git@github.com:Unreal-Engine-FR/resources-history.git', CONFIG.get().CACHE_DIR + '/history-repos/')
                .catch(err => console.fatal(`Failed to clone repos : ${err}`))

        fs.appendFileSync(CONFIG.get().CACHE_DIR + '/history-repos/test.txt', 'toto')

        await git.add('*')
            .catch(err => console.fatal(`failed to add updated file : ${err}`))
        await git.commit('Updated database', {
            '--author': '"Anonymous <>"',
        })
            .catch(err => console.fatal(`Failed to commit : ${err}`))
        await git.push()
            .catch(err => console.fatal(`failed to push update : ${err}`))
    }

    /**
     * // When command is executed
     * @param command {Interaction}
     * @return {Promise<void>}
     */
    async server_interaction(command) {
        if (command.match('say')) {
        }
    }
}

module.exports = {Module}