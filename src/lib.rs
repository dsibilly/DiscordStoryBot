#![deny(rust_2018_idioms)]

use ink_runner::ink_runner::StoryRunner;

/// Usage: Initialize with new() then use the fields, which well be updated whenever choose() is called.
/// while choices aren't Prompt::Done, there is still more story left.
pub struct Game<'a> {
    runner: StoryRunner<'a>,
    lines: Vec<String>,
    lines_with_tags: Vec<(String, Vec<String>)>,
    choices: Vec<String>,
    config: GameConfig,
}

#[derive(Default)]
struct GameConfig {
    hide_choices: bool,
    do_not_pin: bool,
}

impl<'a> Game<'a> {
    pub fn new(content: &str, knot: Option<String>) -> Self {
        let mut me = Game {
            runner: StoryRunner::build_from_str(content),
            lines: vec![],
            lines_with_tags: vec![],
            choices: vec![],
            config: Default::default(),
        };

        me.runner.set_knot(&match knot {
            Some(title) => title,
            None => "__INTRO__".to_string(),
        });

        me.lines_with_tags = me
            .runner
            .start()
            .into_iter()
            .map(|l| {
                (
                    l.text.to_string(),
                    l.tags.into_iter().map(|x| x.to_string()).collect(),
                )
            })
            .collect();
        me.lines = me
            .lines_and_tags()
            .iter()
            .map(|(line, _)| line.to_string())
            .collect();
        me.choices = me.runner.get_options();

        me.config.hide_choices = me
            .runner
            .story
            .global_tags
            .iter()
            .any(|&s| is_hide_choices_tag(s));

        // TODO: scan through all #img: tags to make sure those files exist, so it's caught early

        me
    }

    pub fn set_do_not_pin(mut self, do_not_pin: bool) -> Self {
        self.config.do_not_pin = do_not_pin;
        self
    }

    pub fn choose(&mut self, emoji: &str) {
        let lines = self.runner.step(emoji);
        self.lines = lines.into_iter().map(|l| l.text.to_string()).collect();
        self.choices = self.runner.get_options();
    }

    pub fn lines_as_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn choices_as_strings(&self) -> Vec<String> {
        self.choices.clone()
    }

    pub fn is_over(&self) -> bool {
        self.choices.is_empty()
    }

    pub fn lines_and_tags(&self) -> Vec<(String, Vec<String>)> {
        self.lines_with_tags.clone()
    }

    pub fn do_not_pin(&self) -> bool {
        self.config.do_not_pin
    }

    pub fn should_hide_choices(&self) -> bool {
        self.config.hide_choices
    }

    pub fn images(&self) -> Vec<String> {
        self.lines_and_tags()
            .into_iter()
            .map(|(_, tags)| tags)
            .flatten()
            .filter_map(|s| get_img_tag_image(&s))
            .collect()
    }

    pub fn set_knot(&mut self, knot: &str) {
        self.runner.set_knot(knot);
    }
}

pub fn get_img_tag_image(tag: &str) -> Option<String> {
    dbg!(tag);
    tag.strip_prefix("img:")
        .map(|path| "img/".to_string() + path.trim())
}

pub fn is_hide_choices_tag(tag: &str) -> bool {
    dbg!(tag);
    tag == "hide_choices"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_story() {
        let mut game = Game::new(include_str!("../stories/basic_story.ink"), None);
        dbg!(&game.lines_as_text());
        dbg!(&game.choices_as_strings());
        dbg!(&game.is_over());

        dbg!(game.choose("downtown?"));
        dbg!(&game.lines_as_text());
        dbg!(&game.choices_as_strings());

        assert_eq!(true, game.is_over());
    }

    #[test]
    fn hide_choices() {
        let game = Game::new(include_str!("../stories/basic_story.ink"), None);
        assert_eq!(game.config.hide_choices, false);
        let game = Game::new(include_str!("../stories/hide_choices.ink"), None);
        assert_eq!(game.config.hide_choices, true);
    }
}
