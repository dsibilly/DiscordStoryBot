#![deny(rust_2018_idioms)]

// TODO: instructions when the bot starts up?
// TODO: rename to Discord Story Bot
// TODO: the .exe should take in the token (client id), and story file.
// TODO: the .exe should also take in the knot to start, or a saved state
// TODO: set which hours the bot is allowed to run
// TODO: only one story active at a time
// TODO: allow starting a new story if no story is active
// TODO: choose story beat time in tags
// TODO: set the prefix to whatever they want /help +help =help, etc.

use std::cmp::min;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use serenity::async_trait;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

use discord_bot::Game;

use unicode_segmentation::UnicodeSegmentation;

// TODO: verify the story at the start to make sure all choices in it use discord-valid emoji (https://emojipedia.org/emoji-13.1/)
// TODO: maybe if it's a single letter, we can find the emoji version of that letter?
// TODO: save state always, and look for state when starting with a flag, whatever makes it easy to restart from where you left off if the server crashes

// TODO: say what the previous choice was (as long as it's not in []s, of course)
// TODO: and support having the emoji within []'s

#[tokio::main]
async fn main() {
    let token = include_str!("../client_id.txt").trim();

    let game = Game::new(include_str!("../stories/story1.ink"));

    let mut client = Client::builder(token)
        .event_handler(Handler {
            game: Mutex::new(game),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

struct Handler<'a> {
    game: Mutex<Game<'a>>,
}

#[async_trait]
impl<'a> EventHandler for Handler<'a> {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!help") {
            let channel = msg.channel_id;
            channel
                .say(&ctx.http, "To start a story type something like \"!play 30\", where \"30\" is the number of seconds each voting round should last.".to_string())
                .await
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

            while !self.game.lock().unwrap().is_over() {
                // Get list of choice options
                let text = (self.game.lock().unwrap().lines_as_text()).clone();
                let choices = self.game.lock().unwrap().choices_as_strings();

                let text = text + "\n\n" + &choices.join("\n");

                // only the first grapheme, so we get just the emoji at the start
                let approved_emoji = choices
                    .iter()
                    .map(|s| s.graphemes(true).nth(0).unwrap().to_string())
                    .collect::<Vec<_>>();

                let images: Vec<String> = self.game.lock().unwrap().images();
                dbg!(&images);

                let choice = self
                    .do_story_beat(
                        &ctx,
                        &msg,
                        &text,
                        images,
                        &approved_emoji,
                        &choices,
                        countdown_time,
                    )
                    .await;
                dbg!(&choice);

                self.game.lock().unwrap().choose_by_emoji(&choice);
            }

            let text = self.game.lock().unwrap().lines_as_text();
            let channel = msg.channel_id;
            channel
                .say(&ctx.http, text + &"\nEND.".to_string())
                .await
                .expect("Could not send next initial text");

            dbg!("STORY IS OVER NOW");
        } else if msg.content == "!continue" {
            println!("huh?!");
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl<'a> Handler<'a> {
    // TODO: should this take in the whole story instead of just bits? Or get it from self or something?
    async fn do_story_beat(
        &self,
        ctx: &Context,
        msg: &Message,
        text: &str,
        paths: Vec<String>, // TODO: make this more generic
        approved_emoji: &[String],
        choices: &[String],
        countdown: u32,
    ) -> String {
        let channel = msg.channel_id;
        let mut countdown = countdown as i32;
        let countdown_increment: i32 = 5;

        //dbg!(images);

        //let paths = vec!["img/castle_lowres.jpg"];
        //let paths: Vec<&str> = vec![];

        let paths: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

        // always use send_files, because we can send it no files, and that's fine for a normal message it seems
        let mut message = channel
            .send_files(&ctx, paths, |m| {
                m.content(text.to_string() + &format!("\n({}s remaining)", countdown))
            })
            .await
            .expect(&format!("Could not send message {}", &text));

        dbg!(msg.unpin(ctx).await); // TODO: docs, saying that Manage Messages is required
        dbg!(message.pin(ctx).await);

        // React to self with options
        for emoji in approved_emoji {
            message
                .react(ctx, ReactionType::Unicode(emoji.into()))
                .await
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
                .await
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
        let winning_emoji = counts
            .iter()
            .max_by_key(|a| a.1)
            .expect("No emoji was chosen, not even by the bot")
            .0
            .to_owned();

        choices
            .iter()
            .find(|s| s.starts_with(&winning_emoji))
            .unwrap()
            .to_string()
    }
}
