extern crate discord;

use discord::model::ChannelId;
use discord::model::Event;
use discord::model::Message;
use discord::model::MessageId;
use discord::model::Reaction;
use discord::Discord;

use color_backtrace;

#[derive(Default)]
struct Game {
    channel_id: Option<ChannelId>,
    most_recent_message_id: Option<MessageId>,
}

fn main() {
    color_backtrace::install();

    let token = include_str!("../client_id.txt").trim();

    // Log in to Discord using a bot token from the environment
    let discord = Discord::from_bot_token(token).expect("login failed");

    // Establish and use a websocket connection
    let (mut connection, _) = discord.connect().expect("connect failed");

    println!("Ready.");

    let mut game = Game::default();

    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                handle_user_message(&message, &discord, &mut game);
            }
            Ok(Event::ReactionAdd(reaction)) => {
                handle_user_reaction(&reaction, &discord, &mut game);
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}", code, body);
                break;
            }
            Err(err) => println!("Receive error: {:?}", err),
        }
    }
}

fn handle_user_message(message: &Message, discord: &Discord, game: &mut Game) {
    println!("{} says: {}", message.author.name, message.content);

    match message.content.as_str() {
        "!play" => {
            dbg!("start play!");
            game.channel_id = Some(message.channel_id);
        }
        "!test" => {
            let sent_message = discord.send_message(
                message.channel_id,
                "This is a reply to the test.",
                "",
                false,
            );

            game.most_recent_message_id = sent_message.ok().map(|a| a.id);
        }
        _ => {}
    }
}

fn handle_user_reaction(reaction: &Reaction, discord: &Discord, game: &mut Game) {
    if let Some(most_recent_message_id) = game.most_recent_message_id {
        if reaction.message_id == most_recent_message_id {
            dbg!("MOST RECENT");
        }
    }
    dbg!(&reaction.user_id);
    dbg!(&reaction.channel_id);
    dbg!(&reaction.message_id);
    dbg!(&reaction.emoji);
}
