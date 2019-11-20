extern crate discord;

use discord::model::Event;
use discord::model::MessageId;
use discord::Discord;

use color_backtrace;

#[derive(Default)]
struct Game {
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
                println!("{} says: {}", message.author.name, message.content);
                if message.content == "!test" {
                    let sent_message = discord.send_message(
                        message.channel_id,
                        "This is a reply to the test.",
                        "",
                        false,
                    );

                    game.most_recent_message_id = sent_message.ok().map(|a| a.id);
                } else if message.content == "!quit" {
                    println!("Quitting.");
                    break;
                }
            }
            Ok(Event::ReactionAdd(event)) => {
                if let Some(most_recent_message_id) = game.most_recent_message_id {
                    if event.message_id == most_recent_message_id {
                        dbg!("MOST RECENT");
                    }
                }
                dbg!(event.user_id);
                dbg!(event.channel_id);
                dbg!(event.message_id);
                dbg!(event.emoji);
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
