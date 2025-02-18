const fs = require('fs')
const path = require('path')
const {EventManager} = require("./event_manager");
const CONFIG = require("../config");

class ModuleManager {
    constructor() {
        this._module_list = {}
        this._config_data = {}
    }

    /**
     * Initialize module manager with given discord client (can be used to reload everything)
     * @param discord_client
     */
    init(discord_client) {
        // Unload existing modules
        this.unload_modules()

        // Load configuration
        if (!fs.existsSync(CONFIG.get().SAVE_DIR + '/config/modules.json'))
            this._config_data = {}
        else {
            try {
                this._config_data = JSON.parse(fs.readFileSync(CONFIG.get().SAVE_DIR + '/config/modules.json', 'utf8'))
            }
            catch (err) {
                console.error(`Failed to read config json : ${err}`)
                this._config_data = {}
            }
        }

        // Init modules with new client
        this._client = discord_client
        this._event_manager = new EventManager(this._client)

        // Load all modules
        try {
            const names = fs.readdirSync(path.join(__dirname, '../modules'), {withFileTypes: true})
                .filter(dirent => dirent.isDirectory())
                .map(dirent => dirent.name)

            for (const module of names) {
                this.load_module(module)
                    .catch(err => console.error(`Failed to load module : ${err}`))
            }
        } catch (err) {
            console.fatal("failed to load modules : " + err)
        }
        return this
    }

    /**
     * Try to load a module by name (module will be automatically started if enabled is not set to false by default)
     * @param module_name {string}
     */
    async load_module(module_name) {
        await import('../modules/' + module_name + '/module.js')
            .then(module => {
                if (this._module_list[module_name])
                    console.warning(`Module '${module_name}' is already loaded`)
                else {
                    let instance = new module.Module({
                        name: module_name
                    })
                    instance.name = module_name
                    this._module_list[module_name] = instance

                    // Find if module was enabled last time
                    if (this._config_data[module_name]) {
                        if (this._config_data[module_name].enabled) {
                            instance.enabled = false
                            this.start(module_name)
                        }
                    }
                    // Start module if not specified to be stopped by default
                    else {
                        this._config_data[module_name] = {}
                        if (instance.enabled !== false) {
                            instance.enabled = false
                            this.start(module_name)
                        }
                    }
                    console.validate(`Successfully loaded module '${module_name}'`)
                }
            })
            .catch(error => {
                console.error(`Failed to load module '${module_name}' : ${error}`)
            })
    }

    /**
     * Try to unload module by name (module will be stopped first if it is enabled)
     * @param module_name {string}
     */
    unload_module(module_name) {
        if (module_name === 'utilities')
            console.error('Module utilities cannot be unloaded')

        if (!this._module_list[module_name]) {
            return console.error(`Module '${module_name}' does not exists`)
        }

        // Disable module
        if (this._module_list[module_name].enabled === true)
            this.stop(module_name)

        delete this._module_list[module_name]

        console.validate(`Successfully unloaded module '${module_name}'`)
    }

    /**
     * Unload all loaded modules
     */
    unload_modules() {
        for (const module of this.loaded_modules()) {
            this.unload_module(module)
        }
    }

    /**
     * Get a list of loaded module names
     * @returns {string[]}
     */
    loaded_modules() {
        let modules = []
        for (const [key, _] of Object.entries(this._module_list))
            modules.push(key)
        return modules
    }

    /**
     * Start a module by name (return if already started)
     * @param module_name {string}
     */
    start(module_name) {

        if (!this._module_list[module_name]) {
            console.error(`There is no module called '${module_name}'`)
            return
        }

        if (!this._config_data[module_name].enabled) {
            this._config_data[module_name].enabled = true
            this._save_config()
        }

        const module = this._module_list[module_name]
        if (module.enabled !== true) {
            module.enabled = true
            this._event_manager.bind(module)
            if (module.start)
                module.start()

            console.validate(`Successfully enabled module '${module_name}'`)
        } else
            console.warning(`Module '${module_name}' is already enabled`)
    }

    /**
     * Stop a module by name (return if already stopped)
     * @param module_name {string}
     */
    stop(module_name) {
        if (module_name === 'utilities')
            console.error('Module utilities cannot be disabled')

        if (!this._module_list[module_name]) {
            console.error(`There is no module called '${module_name}'`)
            return
        }

        if (this._config_data[module_name].enabled || this._config_data[module_name].enabled === null) {
            this._config_data[module_name].enabled = false
            this._save_config()
        }

        const module = this._module_list[module_name]
        if (module.enabled === true) {
            if (module.stop)
                module.stop()
            this._event_manager.unbind(module)
            module.enabled = false

            console.validate(`Successfully disabled module '${module_name}'`)
        } else
            console.warning(`Module '${module_name}' is already disabled`)
    }

    /**
     * Get event manager
     * @return {EventManager}
     */
    event_manager() {
        return this._event_manager
    }

    /**
     * Get state about all modules
     * @return {*[]}
     */
    all_modules_info() {
        const names = fs.readdirSync(path.join(__dirname, '../modules'), {withFileTypes: true})
            .filter(dirent => dirent.isDirectory())
            .map(dirent => dirent.name)

        let modules = []
        for (const module_name of names) {
            modules.push({
                name: module_name,
                loaded: !!this._module_list[module_name],
                enabled: this._module_list[module_name] && this._module_list[module_name].enabled
            })
        }
        return modules
    }

    _save_config() {
        if (!fs.existsSync(CONFIG.get().SAVE_DIR + '/config/'))
            fs.mkdirSync(CONFIG.get().SAVE_DIR + '/config/', {recursive: true})
        fs.writeFile(
            CONFIG.get().SAVE_DIR + '/config/modules.json',
            JSON.stringify(this._config_data),
            'utf8',
            err => {
                if (err) {
                    console.fatal(`failed to save module config : ${err}`)
                    command.skip()
                }
            })
        console.info(`saved module config to ${CONFIG.get().SAVE_DIR}/config/modules.json`)
    }

    /**
     * Bind a callback to given button
     * @param module
     * @param button {Message|InteractionBase} Message or interaction that contains the button
     * @param callback
     */
    bind_button(module, button, callback) {
        this._event_manager.bind_button(module, button, callback)
    }
}

/**
 * Module manager global singleton
 * @type {ModuleManager}
 */
let
    MODULE_MANAGER = null

function

get() {
    if (MODULE_MANAGER === null)
        MODULE_MANAGER = new ModuleManager()
    return MODULE_MANAGER
}

module.exports = {get}