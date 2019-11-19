const Discord = require('discord.js');
const client = new Discord.Client();


/// EVENTS

client.on('ready', () => {
	console.log(`Logged in as ${client.user.tag}!`);
});

client.on('message', message => {
    if (message.content === '!play') {
        message.channel.send('╔═════════════════════════════════════════╗');
        message.channel.send('║ You step into an [adjective] [location] ║');
        message.channel.send('╚═════════════════════════════════════════╝');
        message.channel.send('What would you like to do?')
            .then(message => {
                message.react('⬅');
                message.react('⬆');
                message.react('⬇');
                message.react('➡');
            });
    }

	if (message.content === '!ping') {
		message.reply('Pong!');
		message.channel.send('Pong to no one...');
	}

	if (message.content === '!react') {
	    message.react('😄');
	}
});


/// LOGIN

// Read in the client ID, and log in
var fs = require('fs');
var filename = "client_id.txt";
fs.readFile(filename, 'utf8', function(err, data) {
    if (err) throw err;
    client.login(data.trim());
});

