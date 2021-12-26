#![deny(rust_2018_idioms)]

use ink_runner::ink_parser::InkStory;
use ink_runner::ink_runner::StoryRunner;
use std::path::{Path, PathBuf};

/// Usage: Initialize with new() then use the fields, which well be updated whenever choose() is called.
/// while choices aren't Prompt::Done, there is still more story left.
pub struct Game<'a> {
    runner: StoryRunner<'a>,
    lines_with_tags: Vec<(String, Vec<String>)>,
    choices: Vec<String>,
    config: GameConfig,
    story_path: PathBuf,
    pub active: bool, // TODO: should this be private? Or on Handler, maybe? (and should these be an enum?)
    pub stopped: bool, // TODO: should this be private? Or on Handler, maybe?
    pub paused: bool, // TODO: should this be private? Or on Handler, maybe?
}

#[derive(Default)]
struct GameConfig {
    hide_choices: bool,
    do_not_pin: bool,
}

impl<'a> Game<'a> {
    pub fn new(content: &str, knot: Option<String>, path: &Path) -> Self {
        let mut me = Game {
            runner: StoryRunner::build_from_str(content),
            lines_with_tags: vec![],
            choices: vec![],
            config: Default::default(),
            story_path: path.to_path_buf(),
            active: false,
            stopped: false,
            paused: false,
        };
        me.reset(knot);
        me
    }

    pub fn set_do_not_pin(mut self, do_not_pin: bool) -> Self {
        self.config.do_not_pin = do_not_pin;
        self
    }

    pub fn choose(&mut self, emoji: &str) {
        let lines = self.runner.step(emoji);
        self.lines_with_tags = lines
            .into_iter()
            .map(|l| {
                (
                    l.text.to_string(),
                    l.tags.iter().map(|s| s.to_string()).collect(),
                )
            })
            .collect();
        self.choices = self.runner.get_options();
    }

    pub fn lines_as_text(&self) -> String {
        self.lines_with_tags
            .iter()
            .map(|(line, _)| line.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
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
            .map(|s| self.story_path.to_string_lossy().to_string() + "/" + &s)
            .collect()
    }

    pub fn set_knot(&mut self, knot: &str) {
        self.runner.set_knot(knot);
    }

    pub fn set_story(&mut self, story: InkStory<'a>, path: &Path) {
        self.runner.replace_story(story);
        self.story_path = path.to_path_buf();
        self.reset(None);
    }

    fn reset(&mut self, knot: Option<String>) {
        self.runner.set_knot(&match knot {
            Some(title) => title,
            None => "__INTRO__".to_string(),
        });

        self.lines_with_tags = self
            .runner
            .start()
            .into_iter()
            .map(|l| (l.text.to_string(), l.tags.into_iter().collect()))
            .collect();

        self.choices = self.runner.get_options();

        self.config.hide_choices = self
            .runner
            .story
            .global_tags
            .iter()
            .any(|&s| s == "hide_choices");

        // TODO: scan through all #img: tags to make sure those files exist, so it's caught early
    }

    pub fn get_author(&self) -> Option<String> {
        self.runner.story.get_author()
    }
}

pub fn get_img_tag_image(tag: &str) -> Option<String> {
    tag.strip_prefix("img:").map(|path| path.trim().to_string())
}

pub fn story_has_hidden_tag(story: &InkStory<'_>) -> bool {
    story.global_tags.iter().any(|&s| s == "hidden")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn basic_story() {
        let mut game = Game::new(
            include_str!("../stories/basic_story.ink"),
            None,
            &"".to_string().into(),
        );
        dbg!(&game.lines_as_text());
        dbg!(&game.choices_as_strings());
        dbg!(&game.is_over());

        dbg!(game.choose("🏢 downtown?"));
        dbg!(&game.lines_as_text());
        dbg!(&game.choices_as_strings());

        assert_eq!(true, game.is_over());
    }

    #[test]
    fn hide_choices() {
        let game = Game::new(
            include_str!("../stories/basic_story.ink"),
            None,
            &"".to_string().into(),
        );
        assert_eq!(game.config.hide_choices, false);
        let game = Game::new(
            include_str!("../stories/hide_choices/hide_choices.ink"),
            None,
            &"".to_string().into(),
        );
        assert_eq!(game.config.hide_choices, true);
    }

    #[test]
    fn parse_images() {
        assert_eq!(get_img_tag_image("img:A.png"), Some("A.png".to_string()));
    }

    #[test]
    fn hidden_choice_text() {
        let mut game = Game::new(
            include_str!("../stories/hidden_choice_text.ink"),
            None,
            &"".to_string().into(),
        );
        assert_eq!(
            game.choices,
            vec![
                "😊".to_string(),
                "😀 - time to smile".to_string(),
                "😎 - be cool".to_string(),
            ]
        );
        game.choose("😊");
        assert_eq!(game.lines_as_text(), "You smile, a grin as big as the sun.");

        let mut game = Game::new(
            include_str!("../stories/hidden_choice_text.ink"),
            None,
            &"".to_string().into(),
        );
        game.choose("😀 - time to smile");
        assert_eq!(
            game.lines_as_text(),
            "😀 - you fight the need to frown, eyes watering."
        );

        let mut game = Game::new(
            include_str!("../stories/hidden_choice_text.ink"),
            None,
            &"".to_string().into(),
        );
        game.choose("😎 - be cool");
        assert_eq!(game.lines_as_text(), "You are being very cool.");
    }
}
