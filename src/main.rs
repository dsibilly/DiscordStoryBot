#![deny(rust_2018_idioms)]
#![allow(clippy::too_many_arguments, clippy::expect_fun_call)]

// TODO: rename to Discord Story Bot
// TODO: set which hours the bot is allowed to run
// TODO: only one story active at a time
// TODO: allow starting a new story if no story is active
// TODO: choose story beat time in tags
// TODO: save the state whenever it changes, and be able to load it up again

use std::cmp::min;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use serenity::async_trait;
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

use structopt::StructOpt;

use discord_story_bot::Game;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, StructOpt)]
#[structopt(name = "Discord Story Bot", about = "about: TODO")]
struct Opt {
    /// client token file path
    #[structopt(parse(from_os_str))]
    token_file: PathBuf,

    /// .ink story file path
    #[structopt(parse(from_os_str))]
    story: PathBuf,

    /// Optional saved state file to load.
    #[structopt(parse(from_os_str))]
    state: Option<PathBuf>, // TODO: use this

    /// Optional knot to start with (can be used with state, but not required). Default is the beginning.
    #[structopt(short, long)]
    knot: Option<String>,

    /// whether to pin the story messages
    #[structopt(long)]
    do_not_pin: bool,
}

// TODO: verify the story at the start to make sure all choices in it use discord-valid emoji (https://emojipedia.org/emoji-13.1/)
// TODO: maybe if it's a single letter, we can find the emoji version of that letter?
// TODO: save state always, and look for state when starting with a flag, whatever makes it easy to restart from where you left off if the server crashes

// TODO: say what the previous choice was (as long as it's not in []s, of course)
// TODO: and support having the emoji within []'s

#[tokio::main]
async fn main() {
    let opt: Opt = Opt::from_args();
    println!("{:#?}", opt);

    let story = fs::read_to_string(opt.story).unwrap();
    let token = fs::read_to_string(opt.token_file).unwrap();

    let game = Game::new(&story, opt.knot).set_do_not_pin(opt.do_not_pin);

    let mut client = Client::builder(token)
        .event_handler(Handler {
            game: Mutex::new(game),
            prefix: Mutex::new("!".to_string()),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

struct Handler<'a> {
    game: Mutex<Game<'a>>,
    prefix: Mutex<String>,
}

#[async_trait]
impl<'a> EventHandler for Handler<'a> {
    async fn message(&self, ctx: Context, msg: Message) {
        let prefix = self.prefix.lock().unwrap().clone();

        if msg.content.starts_with(&(prefix.to_string() + "help")) {
            let channel = msg.channel_id;
            channel
                .say(&ctx.http, "To start a story type something like \"".to_string() + &prefix + "play 30\", where \"30\" is the number of seconds each voting round should last.\n\
                To change the prefix, use the \"" + &prefix + "prefix\" command")
                .await
                .expect("Could not send help text");
        }

        if msg.content.starts_with(&(prefix.to_string() + "prefix")) {
            let channel = msg.channel_id;
            if msg.content.contains(' ') {
                {
                    let mut prefix_locked = self.prefix.lock().unwrap();
                    *prefix_locked = msg.content.split_once(' ').unwrap().1.to_string();
                }

                channel
                    .say(
                        &ctx.http,
                        "prefix has been set to: ".to_string() + &prefix.clone(),
                    )
                    .await
                    .expect("Could not send prefix information text");
            } else {
                channel
                    .say(
                        &ctx.http,
                        "To set a different prefix, type something like \"".to_string()
                            + &prefix
                            + "prefix +\", where \"+\" is the new prefix to use, instead of \""
                            + &prefix
                            + "\".",
                    )
                    .await
                    .expect("Could not send prefix information text");
            }
        }

        if msg.content.starts_with(&(prefix.to_string() + "play")) {
            let mut countdown_time = 5;

            // Parse a number if we got one after "play "
            if msg.content.contains(' ') {
                let subs = msg.content.split(' ').collect::<Vec<&str>>();
                dbg!(subs[1]);
                if let Ok(num) = subs[1].parse::<u32>() {
                    countdown_time = num;
                }
            }

            let mut most_recent_message = msg.clone();

            while !self.game.lock().unwrap().is_over() {
                // Get list of choice options
                let mut text = (self.game.lock().unwrap().lines_as_text()).clone();
                let choices = self.game.lock().unwrap().choices_as_strings();

                if !self.game.lock().unwrap().should_hide_choices() {
                    text = text + "\n\n" + &choices.join("\n");
                }

                // only the first grapheme, so we get just the emoji at the start
                let approved_emoji = choices
                    .iter()
                    .map(|s| s.graphemes(true).next().unwrap().to_string())
                    .collect::<Vec<_>>();

                let images: Vec<String> = self.game.lock().unwrap().images();
                dbg!(&images);

                let (choice, story_message) = self
                    .do_story_beat(
                        &ctx,
                        &most_recent_message,
                        &text,
                        images,
                        &approved_emoji,
                        &choices,
                        countdown_time,
                    )
                    .await;
                dbg!(&choice);
                most_recent_message = story_message;

                self.game.lock().unwrap().choose(&choice);
            }

            let text = self.game.lock().unwrap().lines_as_text();
            let channel = msg.channel_id;
            channel
                .say(&ctx.http, text + &"\nEND.".to_string())
                .await
                .expect("Could not send next initial text");

            dbg!("STORY IS OVER NOW");
        }

        if msg.content == (prefix.to_string() + "continue") {
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
        previous_message: &Message,
        text: &str,
        paths: Vec<String>, // TODO: make this more generic
        approved_emoji: &[String],
        choices: &[String],
        countdown: u32,
    ) -> (String, Message) {
        let channel = previous_message.channel_id;
        let mut countdown = countdown as i32;
        let countdown_increment: i32 = 5;

        let paths: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

        // always use send_files, because we can send it no files, and that's fine for a normal message it seems
        let mut message = channel
            .send_files(&ctx, paths, |m| {
                m.content(text.to_string() + &format!("\n({}s remaining)", countdown))
            })
            .await
            .expect(&format!("Could not send message {}", &text));

        if !self.game.lock().unwrap().do_not_pin() {
            //dbg!(previous_message.unpin(ctx).await); // TODO: docs, saying that Manage Messages is required
            dbg!(message.pin(ctx).await); // TODO: docs, saying that Manage Messages is required
        }

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

        for r in &message.reactions {
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

        (
            choices
                .iter()
                .find(|s| s.starts_with(&winning_emoji))
                .unwrap()
                .to_string(),
            message,
        )
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn basic_story() {
        // TODO: split up the code above so the pieces of it are testable.
    }
}
