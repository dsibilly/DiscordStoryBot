#![deny(rust_2018_idioms)]
#![allow(
    clippy::too_many_arguments,
    clippy::expect_fun_call,
    clippy::or_fun_call
)]

// TODO: choose story beat time in tags (or at least a multiplier or something)
//       maybe the top tag is the default, but it can be overwritten by knot tags?
//       maybe there's a discord command `!play story1 30s` that overrides it?
//       Implement top tag first, then play command, then knot override if there's still a desire.
// TODO: 'pause' and 'resume' commands
// TODO: save the state whenever it changes, and be able to load it up again (per channel)
// TODO: set which hours the bot is allowed to run
// TODO: verify the story at the start to make sure all choices in it use discord-valid emoji (https://emojipedia.org/emoji-13.1/)
// TODO: save state always, and look for state when starting with a flag, whatever makes it easy to restart from where you left off if the server crashes
// TODO: say what the previous choice was (as long as it's not in []s, of course)
// TODO: and support having the emoji within []'s

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

use clap::Parser;

use discord_story_bot::{story_has_hidden_tag, Game};

use ink_runner::ink_parser::InkStory;
use ink_runner::ink_runner::import_story;
use unicode_segmentation::UnicodeSegmentation;
use walkdir::WalkDir;

#[derive(Debug, Parser)]
#[clap(
    name = "Discord Story Bot",
    about = "Run .ink files as interactable discord stories.",
    version
)]
struct Opt {
    /// client token file path
    #[clap(parse(from_os_str))]
    token_file: PathBuf,

    /// Optional saved state file to load.
    #[clap(short, long, parse(from_os_str))]
    state: Option<PathBuf>, // TODO: use this

    /// Optional knot to start with (can be used with state, but not required). Default is the beginning.
    #[clap(short, long)]
    knot: Option<String>,

    /// whether to pin the story messages
    #[clap(long)]
    do_not_pin: bool, // TODO: make this a config option, like prefix. Default to pinning.
}

struct Handler<'a> {
    game: Mutex<Game<'a>>,
    prefix: Mutex<String>,

    /// title (path, InkStory)
    stories: BTreeMap<String, (String, InkStory<'a>)>,
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
                    "To start a story type something like `".to_string()
                        + &prefix
                        + "play story1 duration`, where `story1` is the story you would like to run, and `duration` is the number of seconds that each story beat should last (defaults to 60).\n\nTo change the prefix, use the `"
                        + &prefix
                        + "prefix` command",
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
                        "To set a different prefix, type something like `".to_string()
                            + &prefix
                            + "prefix +`, where `+` is the new prefix to use, instead of `"
                            + &prefix
                            + "`.",
                    )
                    .await
                    .expect("Could not send prefix information text");
            }
        } else if msg.content.starts_with(&(prefix.to_string() + "play")) {
            // TODO: move this out into its own function, please.

            let stories = self.stories.keys().map(|k| k.as_str()).collect::<Vec<_>>();
            let stories_with_authors: Vec<String> = stories
                .iter()
                .filter(|&s| !story_has_hidden_tag(&self.stories[&s.to_string()].1))
                .map(|&s| {
                    format!(
                        "`{}`{}",
                        s,
                        self.stories[&s.to_string()]
                            .1
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
                        "To play a story, type something like `".to_string()
                            + &prefix
                            + "play story_name duration`, where `duration` is the number of seconds that each story beat should last (defaults to 60), and `story_name` is the one of the following:\n\n- " + &stories_with_authors.join("\n- "),
                    )
                    .await
                    .expect("Could not send play information text");
                return;
            }

            // Select a story, and set the duration
            let play_args = msg.content.split(' ').collect::<Vec<_>>();
            let story_name = play_args[1].to_string();
            let countdown_time = if play_args.len() >= 3 {
                play_args[2].parse::<u32>().unwrap_or(60)
            } else {
                60
            };

            let story = self.stories[&story_name].1.clone();
            let path: PathBuf = self.stories.get(&story_name).unwrap().0.clone().into();
            self.game.lock().unwrap().set_story(story, &path);

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

            let mut most_recent_message = msg.clone();

            let mut first = true;
            while !self.game.lock().unwrap().is_over() {
                let intro_text = if first {
                    first = false;
                    self.get_formatted_title_and_author()
                } else {
                    "".to_string()
                };

                let lines = (self.game.lock().unwrap().lines_as_text()).clone();
                let mut text = intro_text + &lines;
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
                    dbg!(most_recent_message.unpin(&ctx).await);
                    channel
                        .say(&ctx.http, "The story has been stopped.".to_string())
                        .await
                        .expect("Could not send \"story is stopped\" text");
                    dbg!("STORY IS STOPPED");
                    return;
                }
            }

            // TODO: will this _not_ send pictures?
            let text = self.game.lock().unwrap().lines_as_text();
            let channel = msg.channel_id;
            let images: Vec<String> = self.game.lock().unwrap().images();
            let images: Vec<&str> = images.iter().map(|s| s.as_str()).collect();

            let _final_message = channel
                .send_files(&ctx, images, |m| {
                    m.content(text.clone() + &"\nEND.".to_string())
                })
                .await
                .expect(&format!("Could not final message {}", &text));

            self.game.lock().unwrap().active = false;

            dbg!("STORY IS OVER NOW");
        } else if msg.content == (prefix.to_string() + "continue") {
            // TODO
        } else if msg.content == (prefix.to_string() + "pause") {
            // TODO
        } else if msg.content == (prefix.to_string() + "stop") {
            dbg!("STORY IS STOPPING");
            self.game.lock().unwrap().stopped = true;
            self.game.lock().unwrap().active = false;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl<'a> Handler<'a> {
    async fn do_story_beat(
        &self,
        ctx: &Context,
        previous_message: &Message,
        text: &str,
        images: Vec<String>, // TODO: make this more generic
        approved_emoji: &[String],
        choices: &[String],
        countdown: u32,
    ) -> (String, Message) {
        let channel = previous_message.channel_id;
        let mut countdown = countdown as u32;
        let countdown_increment: u32 = 5;

        let images: Vec<&str> = images.iter().map(|s| s.as_str()).collect();

        dbg!(&images);

        // Always use send_files, because we can send it no files, and that's fine for a normal message it seems
        let mut message = channel
            .send_files(&ctx, images, |m| {
                m.content(text.to_string() + "\n\n(" + &format_remaining_time(countdown) + ")")
            })
            .await
            .expect(&format!("Could not send message {}", &text));

        if !self.game.lock().unwrap().do_not_pin() {
            dbg!(previous_message.unpin(ctx).await); // TODO: docs, saying that Manage Messages is required
            dbg!(message.pin(ctx).await); // TODO: docs, saying that Manage Messages is required
        }

        // React to self with options
        for emoji in approved_emoji {
            message
                .react(ctx, ReactionType::Unicode(emoji.into()))
                .await
                .expect("could not react to message");
        }

        let mut old_message_content = text.to_string();

        // Count Down
        while countdown > 0 {
            let sleep_this_long = min(countdown, countdown_increment);
            sleep(Duration::from_secs(sleep_this_long as u64));
            countdown -= sleep_this_long;

            if self.game.lock().unwrap().stopped {
                dbg!("Stopping countdown");
                break;
            }

            let new_message_content =
                text.to_string() + "\n\n(" + &format_remaining_time(countdown) + ")";

            // Only send a message update if the message content is different than what we would have sent before
            if new_message_content != old_message_content {
                message
                    .edit(ctx, |m| m.content(&new_message_content))
                    .await
                    .expect("could not edit");
                old_message_content = new_message_content.to_string();
            }
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
            // TODO: should we do this filter outside, so we don't have to pass `choices` into this function?
            choices
                .iter()
                .find(|s| s.starts_with(&winning_emoji))
                .unwrap()
                .to_string(),
            message,
        )
    }

    fn get_formatted_title_and_author(&self) -> String {
        let title = self.game.lock().unwrap().get_title();
        let author = self.game.lock().unwrap().get_author();

        if let Some(title) = title {
            "__**".to_string()
                + &title
                + "**__"
                + &if let Some(author) = author {
                    " by _".to_string() + &author + "_\n\n"
                } else {
                    "\n\n".to_string()
                }
        } else {
            "".to_string()
        }
    }
}

#[tokio::main]
async fn main() {
    let opt: Opt = Opt::from_args();
    println!("{:#?}", opt);

    let stories: Vec<(PathBuf, String)> = get_ink_files_with_paths();

    let story_0 = stories
        .get(0)
        .expect("We could not find any stories. Put some in the local ./stories directory.");

    let story_text =
        fs::read_to_string(story_0.0.to_string_lossy().to_string() + "/" + &story_0.1 + ".ink")
            .unwrap();

    let token = fs::read_to_string(opt.token_file).unwrap();

    let game = Game::new(&story_text, opt.knot, &stories[0].0).set_do_not_pin(opt.do_not_pin);

    let stories = stories.iter().map(|(dir, file)| {
        let full_path = dir.to_string_lossy().to_string() + "/" + file + ".ink";

        (
            file.to_string(),
            (
                dir.to_string_lossy().to_string(),
                import_story(
                    &fs::read_to_string(&full_path)
                        .expect(&format!("could not read story {:?}", &full_path)),
                ),
            ),
        )
    });

    let mut client = Client::builder(token)
        .event_handler(Handler {
            game: Mutex::new(game),
            prefix: Mutex::new("!".to_string()),
            stories: stories.collect(),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn format_remaining_time(time_remaining: u32) -> String {
    match time_remaining {
        1 => "1 second remaining".to_string(),
        t @ 0 | t @ 2..=59 => format!("{} seconds remaining", t),
        60..=119 => "1 minute remaining".to_string(),
        t @ 120..=3599 => format!("{} minutes remaining", t / 60),
        3600..=7199 => "1 hour remaining".to_string(),
        t @ 7200..=86_399 => format!("{} hours remaining", t / (60 * 60)),
        86_400..=172_799 => "1 day remaining".to_string(),
        t => format!("{} days remaining", t / (60 * 60 * 24)),
    }
}

fn get_ink_files_with_paths() -> Vec<(PathBuf, String)> {
    let mut result = vec![];

    for entry in WalkDir::new("./stories/")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name().to_string_lossy().ends_with(".ink") {
            let path = entry.path();
            let directory = path.parent().unwrap().to_path_buf();
            result.push((
                directory,
                path.file_stem().unwrap().to_string_lossy().to_string(),
            ));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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

    #[test]
    fn get_ink_tests() {
        assert_eq!(
            get_ink_files_with_paths()[0],
            ("./stories".into(), "basic_story".to_string())
        );
    }
}
