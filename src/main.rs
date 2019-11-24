#![deny(rust_2018_idioms)]

use std::sync::Arc;
use std::sync::Mutex;
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
    play_story(include_str!("../stories/story1.ink")).expect("story error");

    let token = include_str!("../client_id.txt").trim();

    let mut client = Client::new(&token, Handler::default()).expect("Err creating client");

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

    fn choose(&mut self, i: usize) -> Result<(), InklingError> {
        self.lines.clear();
        self.story.make_choice(i)?;
        self.choices = self.story.resume(&mut self.lines)?;
        Ok(())
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
