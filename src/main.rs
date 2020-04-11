#![deny(rust_2018_idioms)]

use std::cmp::min;
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use inkling::read_story_from_string;
use inkling::InklingError;
use inkling::LineBuffer;
use inkling::Prompt;
use inkling::Story;

use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

fn main() {
    let token = include_str!("../client_id.txt").trim();

    let game = Game::new(include_str!("../stories/story1.ink")).expect("wut");

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

/// Usage: Initialize with new() then use the fields, which well be updated whenever choose() is called.
/// while choices aren't Prompt::Done, there is still more story left.
struct Game {
    lines: LineBuffer,
    story: Story,
    choices: Prompt,
}

impl Game {
    fn new(content: &str) -> Result<Self, InklingError> {
        let mut me = Game {
            lines: Vec::new(),
            story: read_story_from_string(content).unwrap(),
            choices: Prompt::Done,
        };

        me.story.start()?;
        me.choices = me.story.resume(&mut me.lines)?;

        Ok(me)
    }

    fn choose_by_emoji(&mut self, emoji: &str) {
        let index = self
            .choices_as_strings()
            .iter()
            .position(|s| s == emoji)
            .expect("emoji choice was somehow not found...");
        self.choose(index).expect("Choice was not possible");
    }

    fn choose(&mut self, i: usize) -> Result<(), InklingError> {
        self.lines.clear();
        self.story.make_choice(i)?;
        self.choices = self.story.resume(&mut self.lines)?;
        Ok(())
    }

    fn lines_as_text(&self) -> String {
        self.lines
            .iter()
            .map(|s| &s.text)
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn choices_as_strings(&self) -> Vec<String> {
        self.choices
            .get_choices()
            .unwrap()
            .iter()
            .map(|e| e.text.clone())
            .collect()
    }
}

struct Handler {
    game: Mutex<Game>,
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        let mut has_choices = false;
        {
            let game = self.game.lock().unwrap();
            if let Prompt::Choice(_) = &game.choices {
                has_choices = true;
            }
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

            while has_choices {
                let mut text = "".into();
                let mut approved_emoji = vec![];

                if let Ok(game) = self.game.lock() {
                    text = (game.lines_as_text()).clone();
                    approved_emoji = game.choices_as_strings();
                }

                let choice = self.do_story_beat(&ctx, &msg, &text, &approved_emoji, countdown_time);

                if let Ok(mut game) = self.game.lock() {
                    game.choose_by_emoji(&choice);

                    has_choices = false;
                    if let Prompt::Choice(_) = &game.choices {
                        has_choices = true;
                    }
                }
            }

            if let Ok(game) = self.game.lock() {
                let text = game.lines_as_text();
                let channel = msg.channel_id;
                channel
                    .say(&ctx.http, text + &"\nEND.".to_string())
                    .expect("Could not send next initial text");
            }

            dbg!();
            dbg!("STORY IS OVER NOW");
        } else if msg.content == "!continue" {
            println!("huh?!");
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl Handler {
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
            .say(
                &ctx.http,
                text.to_string() + &format!("\n({}s remaining)", countdown),
            )
            .expect("Could not send next initial text");

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
        let mut counts = HashMap::new();

        for r in message.reactions {
            if approved_emoji.contains(&r.reaction_type.to_string()) {
                counts.insert(r.reaction_type.to_string(), r.count);
            }
        }

        // Return the winning emoji
        (&counts)
            .iter()
            .max_by_key(|a| a.1)
            .expect("No emoji was chosen, not even by the bot")
            .0
            .to_owned()
    }
}
