const fs = require('fs')
const CONFIG = require('../config').get()

function arg_to_string(arg, depth = '  ', object_map = new Set()) {
    if (arg instanceof Function) {
        return `\x1b[36m[Function ${arg.name || '(anonymous)'}]\x1b[0m`
    } else if (Array.isArray(arg)) {
        let res = ''
        for (const item of arg)
            res += `${arg_to_string(item, depth, object_map)}, `
        return res.length > 2 ? '[' + res.substring(0, res.length - 2) + ']' : '';
    } else if (arg === Object(arg)) {
        if (object_map.has(arg))
            return `\x1b[36m[Circular]\x1b[0m`;
        object_map.add(arg)
        let string = (arg.constructor ? `\x1b[36m${arg.constructor.name}\x1b[0m` : '') + ' {'
        let started = false
        for (const [key, value] of Object.entries(arg)) {
            string += started ? ',\n' : '\n'
            started = true
            string += `${depth}${key}: ` + arg_to_string(value, depth + '  ', object_map)
        }
        return started ? string + '\n' + depth.substring(2) + '}' : string + '}'
    } else if (typeof arg === 'number' || typeof arg === 'boolean') {
        return `\x1b[33m${arg.toString()}\x1b[0m`
    } else if (typeof arg === 'string' && depth !== '  ') {
        return `\x1b[32m'${arg.replaceAll('\n', '\n' + depth)}'\x1b[0m`
    } else if (arg === null && depth !== '  ') {
        return `\x1b[1m${arg}\x1b[0m`
    }

    return String(arg)
}

function format_string(format, ...args) {
    const text_format = arg_to_string(format)
    let index = 0
    let text = text_format.replace(/%s/g, match => {
        const arg = arg_to_string(args[index++])
        return typeof arg !== 'undefined' ? arg : match
    })
    while (index < args.length) {
        text += ` ${arg_to_string(args[index++])}`
    }
    return text
}

/**
 * @param stack {string}
 */
function filter_call_stack(stack) {
    result = stack.split('\n').filter(function (line) {
        return line.indexOf("logger.js:") === -1 && line.indexOf("process.processTicksAndRejections") === -1;
    }).join('\n')

    return result
}

/**
 * When used, allow to use this class members on console (console.error, console.info etc...)
 */
class Logger {
    constructor() {
        console.log = (message, ...args) => {
            this._internal_print('D', format_string(message, args))
        }

        console.info = this.info
        console.validate = this.validate
        console.warning = this.warning
        console.error = this.error
        console.fatal = this.fatal


        const log_dir = CONFIG.SAVE_DIR + "/log/"
        let date_str = new Date().toLocaleString().replaceAll('/', '-').replaceAll(':', '.').replaceAll(', ', '_')
        this.log_file = `${log_dir}/${date_str}.log`

        process.stdout.write('created log file : ' + this.log_file + '\n')

        if (!fs.existsSync(log_dir))
            fs.mkdirSync(log_dir, {recursive: true})

        fs.readdir(log_dir, (err, files) => {
            files.forEach(file => {
                const file_full = log_dir + '/' + file
                const days = (new Date() - fs.statSync(file_full).mtime) / 1000 / 60 / 60 / 24
                if (days >= CONFIG.LOG_KEEP_DAYS) {
                    this.info(`removed old log file : ${file_full}`)
                    fs.rmSync(file_full)
                }
            })
        })

        this.logStream = fs.createWriteStream(this.log_file);

        this._delegates = []
    }

    /**
     * Bind a delegate that will receive log events
     * @param func {function}
     */
    bind(func) {
        this._delegates.push(func)
    }

    /**
     * Send information message
     * @param message {*}
     * @param args {*}
     */
    info(message, ...args) {
        LOGGER._internal_print('I', format_string(message, args))
    }

    /**
     * Send validation message
     * @param message {*}
     * @param args {*}
     */
    validate(message, ...args) {
        LOGGER._internal_print('V', format_string(message, args))
    }

    /**
     * Send warning message
     * @param message {*}
     * @param args {*}
     */
    warning(message, ...args) {
        LOGGER._internal_print('W', format_string(message, args))
    }

    /**
     * Send error message (will include a stack trace)
     * @param message {*}
     * @param args {*}
     */
    error(message, ...args) {
        LOGGER._internal_print('E', format_string(message, args) + '\n' + filter_call_stack(Error().stack))
    }

    /**
     * Send assertion message and throw error
     * @param message {*}
     * @param args {*}
     */
    fatal(message, ...args) {
        LOGGER._internal_print('F', format_string(message, args))
    }

    _internal_print(level, message) {
        try {
            this.logStream.write(`[${new Date().toLocaleString()}] [${level}] ${message}\n`);
        } catch (err) {
            process.stdout.write(`failed to write log to file ${this.log_file} : ${err}\n`)
        }

        let level_var = `[${level}]`
        switch (level) {
            case 'V':
                level_var = '\x1b[32m[V]\x1b[0m'
                break
            case 'I':
                level_var = '\x1b[36m[I]\x1b[0m'
                break
            case 'W':
                level_var = '\x1b[35m[W]\x1b[0m'
                break
            case 'D':
                level_var = '\x1b[1m\x1b[90m[D]\x1b[0m'
                break
            case 'E':
                level_var = '\x1b[1m\x1b[31m[E]\x1b[0m'
                break
            case 'F':
                level_var = '\x1b[1m\x1b[35m[F]\x1b[0m'
                break
        }

        process.stdout.write(`[${new Date().toLocaleString()}] ${level_var} ${message}\n`)

        for (const delegate of this._delegates)
            delegate(level, message)

        if (level === 'F') {
            try {
                fs.appendFileSync(this.log_file, filter_call_stack(Error().stack) + '\n')
            } catch (err) {
                process.stdout.write(`failed to write error to file ${this.log_file} : ${err}\n`)
            }
            throw new Error(message + '\n' + filter_call_stack(Error().stack))
        }

        if (level === 'E' || level === 'F')
            this._crash_dump()
    }

    _crash_dump() {
        const crash_dir = CONFIG.SAVE_DIR + "/crash/"

        if (!fs.existsSync(crash_dir))
            fs.mkdirSync(crash_dir, {recursive: true})

        let date_str = new Date().toLocaleString().replaceAll('/', '-').replaceAll(':', '.').replaceAll(', ', '_')
        const crash_file = `${crash_dir}/${date_str}.crash`

        fs.copyFile(this.log_file, crash_file, (err) => {
            if (err) throw new Error(`failed to copy crash dump : ${err}`)
        });
    }
}


let LOGGER = null

function init() {
    LOGGER = new Logger()
}

function get() {
    return LOGGER
}

module.exports = {init_logger: init, get}