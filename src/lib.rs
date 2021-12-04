#![deny(rust_2018_idioms)]

use ink_runner::ink_runner::StoryRunner;

/// Usage: Initialize with new() then use the fields, which well be updated whenever choose() is called.
/// while choices aren't Prompt::Done, there is still more story left.
pub struct Game<'a> {
    runner: StoryRunner<'a>,
    lines_2: Vec<String>,
    choices_2: Vec<String>,
}

impl<'a> Game<'a> {
    pub fn new(content: &str) -> Self {
        let mut me = Game {
            runner: StoryRunner::build_from_str(content),
            lines_2: vec![],
            choices_2: vec![],
        };

        me.lines_2 = me
            .runner
            .start()
            .into_iter()
            .map(|l| l.text.to_string())
            .collect();
        me.choices_2 = me.runner.get_options();

        me
    }

    pub fn choose_by_emoji(&mut self, emoji: &str) {
        let lines = self.runner.step(emoji);
        self.lines_2 = lines.into_iter().map(|l| l.text.to_string()).collect();
        self.choices_2 = self.runner.get_options();
    }

    pub fn lines_as_text(&self) -> String {
        return self.lines_2.join("\n");
    }

    pub fn choices_as_strings(&self) -> Vec<String> {
        self.choices_2.clone()
    }

    pub fn is_over(&self) -> bool {
        self.choices_2.is_empty()
    }

    pub fn tags(&self) -> Vec<String> {
        unimplemented!()
    }
}

#[cfg(test)]
#[allow(unused)] // TODO: Delete This
mod tests {
    use super::*;

    #[test]
    fn basic_story() {
        let mut game = Game::new(include_str!("../stories/basic_story.ink")).expect("wut");
        dbg!(&game.lines_as_text());
        dbg!(&game.choices_as_strings());
        dbg!(&game.is_over());

        dbg!(game.choose_by_index(0));
        dbg!(&game.lines_as_text());
        dbg!(&game.choices_as_strings());
        dbg!(&game.is_over());

        assert_eq!(true, game.is_over());
    }
}
