const quotes = require('../modules/quote/data/quotes.json');

module.exports = () => {
    const pseudos = Object.keys(quotes);
    let pseudoList = '';
    pseudos.sort().forEach(pseudo => pseudoList += pseudo+'\n');

    return {
        embed: {
            title      : 'Liste des pseudos qui ont des quotes',
            description: pseudoList,
            color      : process.env.COLOR_QUOTE,
        }
    };

};