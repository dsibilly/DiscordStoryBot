#![deny(rust_2018_idioms)]

// TODO: make this work WITHOUT Client, because I only want one of a thing to exist at a time ... if possible.

use std::sync::Arc;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

#[derive(Default)]
struct Handler {
    info: Arc<Mutex<i32>>, // everything must be thread-safe
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!play" {
            let mut info = self.info.lock().unwrap();
            *info += 1;

            let message_format = "╔═════════════════════════════════════════\n\
                                  ║ You step into an [adjective] [location]\n\
                                  ╚═════════════════════════════════════════\n\
                                  What would you like to do?";

            let mut countdown: i32 = 20;
            let countdown_increment: i32 = 5;

            match msg.channel_id.say(
                &ctx.http,
                message_format.to_string() + &format!(" - {}s remaining", countdown),
            ) {
                Err(why) => {
                    println!("Error sending message: {:?}", why);
                }
                Ok(mut message) => {
                    message
                        .react(&ctx, ReactionType::Unicode("🙂".into()))
                        .expect("could not react to message");

                    while countdown > 0 {
                        sleep(Duration::from_secs(countdown_increment as u64));
                        countdown -= countdown_increment;

                        message
                            .edit(&ctx, |m| {
                                m.content(
                                    message_format.to_string()
                                        + &format!(" - {}s remaining", countdown),
                                )
                            })
                            .expect("could not edit");
                    }

                    msg.channel_id
                        .say(&ctx.http, "Chosen!")
                        .expect("could not say");
                }
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let token = include_str!("../client_id.txt").trim();

    let mut client = Client::new(&token, Handler::default()).expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
