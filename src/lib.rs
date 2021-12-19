#![deny(rust_2018_idioms)]

use ink_runner::ink_runner::StoryRunner;

/// Usage: Initialize with new() then use the fields, which well be updated whenever choose() is called.
/// while choices aren't Prompt::Done, there is still more story left.
pub struct Game<'a> {
    runner: StoryRunner<'a>,
    lines_with_tags: Vec<(String, Vec<String>)>,
    choices: Vec<String>,
}

impl<'a> Game<'a> {
    pub fn new(content: &str, knot: Option<String>) -> Self {
        let mut me = Game {
            runner: StoryRunner::build_from_str(content),
            lines_with_tags: vec![],
            choices: vec![],
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
        me.choices = me.runner.get_options();

        // TODO: scan through all #img: tags to make sure those files exist, so it's caught early

        me
    }

    pub fn choose(&mut self, emoji: &str) {
        let lines = self.runner.step(emoji);
        self.lines_with_tags = lines.into_iter().map(|l| (
                                l.text.to_string(),
                                l.tags.into_iter().map(|x| x.to_string()).collect()
                            )).collect();
        self.choices = self.runner.get_options();
    }

    pub fn lines_as_text(&self) -> String {
        self.lines_with_tags.clone()
            .into_iter()
            .map(|( s,_)| s)
            .collect::<Vec<String>>()
            .join("\n")
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

#[cfg(test)]
#[allow(unused)] // TODO: Delete This
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
}
