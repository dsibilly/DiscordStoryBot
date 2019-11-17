const Discord = require('discord.js');
const client = new Discord.Client();


/// EVENTS

client.on('ready', () => {
	console.log(`Logged in as ${client.user.tag}!`);
});

client.on('message', msg => {
	if (msg.content === 'ping') {
		msg.reply('Pong!');
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

