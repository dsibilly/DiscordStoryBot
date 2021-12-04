#![deny(rust_2018_idioms)]

use std::cmp::min;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

use discord_bot::Game;

// TODO: verify the story at the start to make sure all choices in it use discord-valid emoji (https://emojipedia.org/emoji-13.1/)
// TODO: maybe if it's a single letter, we can find the emoji version of that letter?
// TODO: save state always, and look for state when starting with a flag, whatever makes it easy to restart from where you left off if the server crashes
// TODO: an option to attach photos
// TODO: an option to attach audio
// TODO: an option to attach video

fn main() {
    let token = include_str!("../client_id.txt").trim();

    let game = Game::new(include_str!("../stories/story1.ink"));

    let mut client = Client::new(
        &token,
        Handler {
            game: Mutex::new(game),
        },
    )
    .expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

struct Handler<'a> {
    game: Mutex<Game<'a>>,
}

impl<'a> EventHandler for Handler<'a> {
    fn message(&self, ctx: Context, msg: Message) {
        let mut is_over = true;

        if let Ok(game) = self.game.lock() {
            is_over = game.is_over();
        }

        if msg.content.starts_with("!help") {
            let channel = msg.channel_id;
            channel
                .say(&ctx.http, "To start a story type something like \"!play 30\", where \"30\" is the number of seconds each voting round should last.".to_string())
                .expect("Could not send help text");
        }

        if msg.content.starts_with("!play") {
            let mut countdown_time = 5;

            // Parse a number if we got one after "play "
            if msg.content.contains(' ') {
                let subs = msg.content.split(' ').collect::<Vec<&str>>();
                dbg!(subs[1]);
                if let Ok(num) = subs[1].parse::<u32>() {
                    countdown_time = num;
                }
            }

            while !is_over {
                let mut text = "".into();
                let mut approved_emoji = vec![];

                // Get list of choice options
                if let Ok(game) = self.game.lock() {
                    text = (game.lines_as_text()).clone();

                    approved_emoji = game.choices_as_strings();
                    dbg!(&approved_emoji); // TODO: this is wrong. Why?
                }

                let choice = self.do_story_beat(&ctx, &msg, &text, &approved_emoji, countdown_time);
                dbg!(&choice);

                is_over = true;
                if let Ok(mut game) = self.game.lock() {
                    game.choose_by_emoji(&choice);

                    is_over = game.is_over();
                }
            }

            if let Ok(game) = self.game.lock() {
                let text = game.lines_as_text();
                let channel = msg.channel_id;
                channel
                    .say(&ctx.http, text + &"\nEND.".to_string())
                    .expect("Could not send next initial text");
            }

            dbg!("STORY IS OVER NOW");
        } else if msg.content == "!continue" {
            println!("huh?!");
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl<'a> Handler<'a> {
    fn do_story_beat(
        &self,
        ctx: &Context,
        msg: &Message,
        text: &str,
        approved_emoji: &[String],
        countdown: u32,
    ) -> String {
        let channel = msg.channel_id;
        let mut countdown = countdown as i32;
        let countdown_increment: i32 = 5;

        let mut message = channel
            //.send_files()
            .say(
                &ctx.http,
                text.to_string() + &format!("\n({}s remaining)", countdown),
            )
            .expect("Could not send next initial text");

        dbg!(msg.unpin(ctx)); // TODO: docs, saying that Manage Messages is required
        dbg!(message.pin(ctx));

        // React to self with options
        for emoji in approved_emoji {
            message
                .react(ctx, ReactionType::Unicode(emoji.into()))
                .expect("could not react to message");
        }

        // Count Down
        while countdown > 0 {
            let sleep_this_long = min(countdown, countdown_increment);
            sleep(Duration::from_secs(sleep_this_long as u64));
            countdown -= sleep_this_long;

            message
                .edit(ctx, |m| {
                    m.content(text.to_string() + &format!("\n({}s remaining)", countdown))
                })
                .expect("could not edit");
        }

        // Get the highest-rated emoji (from the approved list for this text)
        let mut counts = BTreeMap::new();

        for r in message.reactions {
            if approved_emoji.contains(&r.reaction_type.to_string()) {
                counts.insert(r.reaction_type.to_string(), r.count);
            }
        }

        // Return the winning emoji
        counts
            .iter()
            .max_by_key(|a| a.1)
            .expect("No emoji was chosen, not even by the bot")
            .0
            .to_owned()
    }
}
