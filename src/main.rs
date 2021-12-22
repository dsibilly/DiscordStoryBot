#![deny(rust_2018_idioms)]
#![allow(
    clippy::too_many_arguments,
    clippy::expect_fun_call,
    clippy::or_fun_call
)]

// TODO: make each story a directory, with images in its own img, so the story only has to worry about relative paths
// TODO: make the client id default to client_ids/client_id.txt if it's not included (on a flag)
// TODO: import all stories that are within the stories directory, where each one is its own directory
// TODO: if no command line args are used, use a config file, or maybe just _always_ use a config file
//       or maybe I can just put all that in systemctl file? (this works for now)
// TODO: 'pause' and 'resume' commands
// TODO: choose story beat time in tags (or at least a multiplier or something)
// TODO: save the state whenever it changes, and be able to load it up again (per channel)
// TODO: point to a directory, or auto-import all stories nested within "stories" (https://rust-lang-nursery.github.io/rust-cookbook/file/dir.html#recursively-find-all-files-with-given-predicate)
// TODO: set which hours the bot is allowed to run

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

use ink_runner::ink_parser::InkStory;
use ink_runner::ink_runner::import_story;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, StructOpt)]
#[structopt(name = "Discord Story Bot", about = "about: TODO")]
struct Opt {
    /// client token file path
    #[structopt(parse(from_os_str))]
    token_file: PathBuf,

    /// .ink story file paths
    #[structopt(parse(from_os_str))]
    stories: Vec<PathBuf>,

    /// Optional saved state file to load.
    #[structopt(short, long, parse(from_os_str))]
    state: Option<PathBuf>, // TODO: use this

    /// Optional knot to start with (can be used with state, but not required). Default is the beginning.
    #[structopt(short, long)]
    knot: Option<String>,

    /// whether to pin the story messages
    #[structopt(long)]
    do_not_pin: bool,
}

struct Handler<'a> {
    game: Mutex<Game<'a>>,
    prefix: Mutex<String>,
    stories: BTreeMap<String, InkStory<'a>>,
}

#[async_trait]
impl<'a> EventHandler for Handler<'a> {
    async fn message(&self, ctx: Context, msg: Message) {
        let prefix = self.prefix.lock().unwrap().clone();

        if msg.content.starts_with(&(prefix.to_string() + "help")) {
            // TODO: move this out into its own function, please.

            let channel = msg.channel_id;
            channel
                .say(
                    &ctx.http,
                    "To start a story type something like \"".to_string()
                        + &prefix
                        + "play story1\", where \"story1\" is the story you would like to run.\n\
                To change the prefix, use the \""
                        + &prefix
                        + "prefix\" command",
                )
                .await
                .expect("Could not send help text");
        } else if msg.content.starts_with(&(prefix.to_string() + "prefix")) {
            // TODO: move this out into its own function, please.

            let channel = msg.channel_id;
            if msg.content.contains(' ') {
                let new_prefix;
                {
                    let mut prefix_locked = self.prefix.lock().unwrap();
                    new_prefix = msg.content.split_once(' ').unwrap().1.to_string();
                    *prefix_locked = new_prefix.clone();
                }

                channel
                    .say(
                        &ctx.http,
                        "prefix has been set to: ".to_string() + &new_prefix,
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
        } else if msg.content.starts_with(&(prefix.to_string() + "play")) {
            // TODO: move this out into its own function, please.

            let stories = self.stories.keys().map(|k| k.as_str()).collect::<Vec<_>>();
            let stories_with_authors: Vec<String> = stories
                .iter()
                .map(|&s| {
                    format!(
                        "\"{}\"{}",
                        s,
                        self.stories[&s.to_string()]
                            .get_author()
                            .map(|a| format!(" by {}", a))
                            .unwrap_or("".to_string())
                    )
                })
                .collect();

            if !msg.content.contains(' ') {
                let channel = msg.channel_id;
                channel
                    .say(
                        &ctx.http,
                        "To play a story, type something like \"".to_string()
                            + &prefix
                            + "play story_name\", where \"story_name\" is the one of the following:\n- " + &stories_with_authors.join("\n- "),
                    )
                    .await
                    .expect("Could not send prefix information text");
                return;
            }

            // Select a story
            let story_name = msg.content.split_once(' ').unwrap().1.to_string();
            let story = self.stories[&story_name].clone();
            self.game.lock().unwrap().set_story(story);

            // See if there is already a game running
            if self.game.lock().unwrap().active {
                let channel = msg.channel_id;
                channel
                    .say(&ctx.http, "A story is already in progress".to_string())
                    .await
                    .expect("Could not send \"story in progress\" text");
                return;
            }

            self.game.lock().unwrap().active = true;

            let countdown_time = 65; // TODO: get this from the knot, or config, or something...

            //// Parse a number if we got one after "play "
            //if msg.content.contains(' ') {
            //    let subs = msg.content.split(' ').collect::<Vec<&str>>();
            //    dbg!(subs[1]);
            //    if let Ok(num) = subs[1].parse::<u32>() {
            //        countdown_time = num;
            //    }
            //}

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
                dbg!(self.game.lock().unwrap().lines_and_tags());

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

                if self.game.lock().unwrap().stopped {
                    self.game.lock().unwrap().stopped = false;
                    self.game.lock().unwrap().active = false;
                    let channel = msg.channel_id;
                    channel
                        .say(&ctx.http, "The story has been stopped.".to_string())
                        .await
                        .expect("Could not send \"story is stopped\" text");
                    dbg!("STORY IS STOPPED");
                    return;
                }
            }

            let text = self.game.lock().unwrap().lines_as_text();
            let channel = msg.channel_id;
            channel
                .say(&ctx.http, text + &"\nEND.".to_string())
                .await
                .expect("Could not send next initial text");

            self.game.lock().unwrap().active = false;

            dbg!("STORY IS OVER NOW");
        } else if msg.content == (prefix.to_string() + "continue") {
            // TODO
        } else if msg.content == (prefix.to_string() + "pause") {
            // TODO
        } else if msg.content == (prefix.to_string() + "stop") {
            dbg!("STORY IS STOPPING");
            self.game.lock().unwrap().stopped = true;
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
        let mut countdown = countdown as u32;
        let countdown_increment: u32 = 5;

        let paths: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();

        // always use send_files, because we can send it no files, and that's fine for a normal message it seems
        let mut message = channel
            .send_files(&ctx, paths, |m| {
                m.content(text.to_string() + "\n(" + &format_remaining_time(countdown) + ")")
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

            if self.game.lock().unwrap().stopped {
                dbg!("Stopping countdown");
                break;
            }

            message
                .edit(ctx, |m| {
                    m.content(text.to_string() + "\n(" + &format_remaining_time(countdown) + ")")
                })
                .await
                .expect("could not edit");
        }

        // Return the winning emoji
        let winning_emoji = if self.game.lock().unwrap().stopped {
            approved_emoji[0].to_string()
        } else {
            // Get the highest-rated emoji (from the approved list for this text)
            let mut counts = BTreeMap::new();

            for r in &message.reactions {
                if approved_emoji.contains(&r.reaction_type.to_string()) {
                    counts.insert(r.reaction_type.to_string(), r.count);
                }
            }

            counts
                .iter()
                .max_by_key(|a| a.1)
                .expect("No emoji was chosen, not even by the bot")
                .0
                .to_owned()
        };

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

// TODO: verify the story at the start to make sure all choices in it use discord-valid emoji (https://emojipedia.org/emoji-13.1/)
// TODO: maybe if it's a single letter, we can find the emoji version of that letter?
// TODO: save state always, and look for state when starting with a flag, whatever makes it easy to restart from where you left off if the server crashes

// TODO: say what the previous choice was (as long as it's not in []s, of course)
// TODO: and support having the emoji within []'s

#[tokio::main]
async fn main() {
    let opt: Opt = Opt::from_args();
    println!("{:#?}", opt);

    let story = fs::read_to_string(opt.stories[0].clone()).unwrap(); // TODO: handle multiple
                                                                     //let story = import_story(&fs::read_to_string(opt.stories[0].clone()).unwrap()); // TODO: handle multiple
    let token = fs::read_to_string(opt.token_file).unwrap();

    let game = Game::new(&story, opt.knot).set_do_not_pin(opt.do_not_pin);

    let mut client = Client::builder(token)
        .event_handler(Handler {
            game: Mutex::new(game),
            prefix: Mutex::new("!".to_string()),

            // TODO: This should be in a config file, or CLI args or something...
            stories: opt
                .stories
                .iter()
                .map(|s| {
                    (
                        s.file_stem()
                            .expect(&format!("invalid file: {}", s.as_path().to_str().unwrap()))
                            .to_string_lossy()
                            .to_string(),
                        import_story(
                            &fs::read_to_string(&s)
                                .expect(&format!("could not read story {:?}", &s)),
                        ),
                    )
                })
                .collect(),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn format_remaining_time(time_remaining: u32) -> String {
    match time_remaining {
        1 => format!("1 second remaining"),
        t @ 0 | t @ 2..=59 => format!("{} seconds remaining", t),
        60..=119 => format!("1 minute remaining"),
        t @ 120..=3599 => format!("{} minutes remaining", t / 60),
        3600..=7199 => format!("1 hour remaining"),
        t @ 7200..=86_399 => format!("{} hours remaining", t / (60 * 60)),
        86_400..=172_799 => format!("1 day remaining"),
        t => format!("{} days remaining", t / (60 * 60 * 24)),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    //use super::*;

    use crate::format_remaining_time;

    #[test]
    fn basic_story() {
        // TODO: split up the code above so the pieces of it are testable.
    }

    #[test]
    fn format_remaining_time_tests() {
        let minute = 60;
        let hour = minute * 60;
        let day = hour * 24;
        assert_eq!(format_remaining_time(0), "0 seconds remaining".to_string());
        assert_eq!(format_remaining_time(1), "1 second remaining".to_string());
        assert_eq!(format_remaining_time(2), "2 seconds remaining".to_string());
        assert_eq!(
            format_remaining_time(59),
            "59 seconds remaining".to_string()
        );
        assert_eq!(
            format_remaining_time(minute),
            "1 minute remaining".to_string()
        );
        assert_eq!(
            format_remaining_time(minute + 5),
            "1 minute remaining".to_string()
        );
        assert_eq!(
            format_remaining_time(minute + 59),
            "1 minute remaining".to_string()
        );
        assert_eq!(
            format_remaining_time(2 * minute),
            "2 minutes remaining".to_string()
        );
        assert_eq!(
            format_remaining_time(2 * minute + 59),
            "2 minutes remaining".to_string()
        );
        assert_eq!(format_remaining_time(hour), "1 hour remaining".to_string());
        assert_eq!(
            format_remaining_time(hour + 1),
            "1 hour remaining".to_string()
        );
        assert_eq!(
            format_remaining_time(2 * hour + 1),
            "2 hours remaining".to_string()
        );
        assert_eq!(
            format_remaining_time(23 * hour + 59),
            "23 hours remaining".to_string()
        );
        assert_eq!(format_remaining_time(day), "1 day remaining".to_string());
        assert_eq!(
            format_remaining_time(day + hour),
            "1 day remaining".to_string()
        );
        assert_eq!(
            format_remaining_time(2 * day + hour),
            "2 days remaining".to_string()
        );
    }
}
