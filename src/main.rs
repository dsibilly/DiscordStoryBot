#![deny(rust_2018_idioms)]

use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use inkling::read_story_from_string;
use inkling::Choice;
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
    //play_story(include_str!("../stories/story1.ink")).expect("story error");

    let token = include_str!("../client_id.txt").trim();

    //let game = Game::new(include_str!("../stories/story1.ink")).expect("wut");
    let game_string = include_str!("../stories/story1.ink");

    let mut client = Client::new(
        &token,
        Handler {
            game_string: game_string.into(),
        },
    )
    .expect("huh?");

    //}).expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

//fn play_game(content: &str) -> Result<(), InklingError> {}

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
        self.choose(index);
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

fn play_story(story_content: &str) -> Result<(), InklingError> {
    let mut game = Game::new(story_content)?;

    print_lines(&game.lines);

    while let Prompt::Choice(choices) = &game.choices {
        dbg!(&choices
            .iter()
            .map(|x| x.text.clone())
            .collect::<Vec<String>>());

        game.choose(0)?;
        print_lines(&game.lines);
    }

    Ok(())
}

fn print_lines(lines: &LineBuffer) {
    for line in lines {
        print!("{}", line.text);

        if line.text.ends_with('\n') {
            print!("\n");
        }
    }
}

struct Handler {
    game_string: String,
    //game: Game,
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        // TODO: give this inner mutability instead of making it a local variable?
        let mut game: Game = Game::new(&self.game_string).expect("Could not parse story");

        if msg.content == "!play" {
            let intro_lines = &(game.lines_as_text());

            let approved_emoji = game.choices_as_strings();

            let choice = self.do_story_beat(&ctx, &msg, intro_lines, &approved_emoji);

            game.choose_by_emoji(&choice);

            dbg!(game.lines_as_text());

        // TODO: progress in the story.
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
    ) -> String {
        let channel = msg.channel_id;
        let mut countdown: i32 = 5;
        let countdown_increment: i32 = 5;

        let mut message = channel
            .say(
                &ctx.http,
                text.to_string() + &format!("Choose - {}s remaining", countdown),
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
            sleep(Duration::from_secs(countdown_increment as u64));
            countdown -= countdown_increment;

            message
                .edit(ctx, |m| {
                    m.content(text.to_string() + &format!(" - {}s remaining", countdown))
                })
                .expect("could not edit");
        }

        // Get the highest-rated emoji (from the approved list for this text)
        let mut counts = HashMap::new();

        for r in message.reactions {
            if approved_emoji.contains(&r.reaction_type.to_string()) {
                counts.insert((&r).reaction_type.to_string(), r.count);
            }
        }

        let winning_emoji = (&counts)
            .iter()
            .max_by_key(|a| a.1)
            .expect("No emoji was chosen, not even by the bot")
            .0
            .to_owned();

        // Declare the winner?
        // TODO: do this as the start of the next message instead, to reduce bot message count
        channel
            .say(&ctx.http, "Chosen: ".to_string() + &winning_emoji)
            .expect("could not say who won. Could not send that message.");

        // Return the winning emoji
        winning_emoji.to_string()
    }
}
