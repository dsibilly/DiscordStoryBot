use inkling::read_story_from_string;
use inkling::InklingError;
use inkling::LineBuffer;
use inkling::Prompt;
use inkling::Story;

/// Usage: Initialize with new() then use the fields, which well be updated whenever choose() is called.
/// while choices aren't Prompt::Done, there is still more story left.
pub struct Game {
    lines: LineBuffer,
    pub story: Story,
    choices: Prompt,
}

impl Game {
    pub fn new(content: &str) -> Result<Self, InklingError> {
        let mut me = Game {
            lines: Vec::new(),
            story: read_story_from_string(content).unwrap(),
            choices: Prompt::Done,
        };

        me.story.start()?;
        me.choices = me.story.resume(&mut me.lines)?;

        Ok(me)
    }

    pub fn choose_by_emoji(&mut self, emoji: &str) {
        let index = self
            .choices_as_strings()
            .iter()
            .position(|s| s == emoji)
            .expect("emoji choice was somehow not found...");
        self.choose_by_index(index)
            .expect("Choice was not possible");
    }

    pub fn choose_by_index(&mut self, i: usize) -> Result<(), InklingError> {
        self.lines.clear();
        self.story.make_choice(i)?;
        self.choices = self.story.resume(&mut self.lines)?;
        Ok(())
    }

    pub fn choose(&mut self, emoji: &str) -> Option<()> {
        let choices = self.choices_as_strings();
        let index = choices.iter().position(|x| x == emoji);

        if let Some(index) = index {
            self.choose_by_index(index).unwrap();
            Some(())
        } else {
            None
        }
    }

    pub fn lines_as_text(&self) -> String {
        self.lines
            .iter()
            .map(|s| &s.text)
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn choices_as_strings(&self) -> Vec<String> {
        if !self.is_over() {
            self.choices
                .get_choices()
                .unwrap()
                .iter()
                .map(|e| e.text.clone())
                .collect()
        } else {
            vec![]
        }
    }

    pub fn is_over(&self) -> bool {
        if let Prompt::Choice(_) = self.choices {
            false
        } else {
            true
        }
    }

    pub fn tags(&self) -> Vec<String> {
        let mut tags = vec![];

        for x in self.lines.clone() {
            tags.extend(x.tags.clone());
        }

        tags
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
        dbg!(&game.tags());
        dbg!(&game.choices_as_strings());
        dbg!(&game.is_over());

        dbg!(game.choose_by_index(0));
        dbg!(&game.lines_as_text());
        dbg!(&game.tags());
        dbg!(&game.choices_as_strings());
        dbg!(&game.is_over());

        assert_eq!(true, game.is_over());
    }
}
