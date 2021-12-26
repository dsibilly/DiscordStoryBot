#![allow(clippy::expect_fun_call)]

use crate::ink_lexer::{lex, strip_comments};
use crate::ink_parser::{
    lexed_to_parsed, DialogLine, Expression, InkStory, KnotEnd, Line, VariableValue,
};
use ron::ser::{to_string_pretty, PrettyConfig};
use ron::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub enum Output {
    Dialog(String),
    Tag(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct OutputLine {
    pub text: String,
    pub tags: Vec<String>,
}

impl From<&str> for OutputLine {
    fn from(s: &str) -> Self {
        OutputLine {
            text: s.to_string(),
            tags: vec![],
        }
    }
}

impl From<String> for OutputLine {
    fn from(s: String) -> Self {
        OutputLine {
            text: s,
            tags: vec![],
        }
    }
}

impl<'a> From<DialogLine<'a>> for OutputLine {
    fn from(s: DialogLine<'a>) -> Self {
        OutputLine {
            text: s.text.to_string(),
            tags: s.tags.iter().map(|s| s.to_string()).collect(),
        }
    }
}

pub struct StoryRunner<'a> {
    pub state: StoryState,
    pub story: InkStory<'a>,
}

// TODO: support starting a story part-way through
// TODO: ^-- would we need versions or something for that? Or just keep it simple?
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct StoryState {
    current_knot_title: String,
    variables: BTreeMap<String, VariableValue>, // TODO: set these
    visited_knots: BTreeMap<String, i32>,
    chosen_choices: HashSet<(String, String)>, // TODO: (Knot title, choices), but this should be a struct or something? BTreeMap?
}

impl Default for StoryState {
    fn default() -> Self {
        StoryState {
            current_knot_title: "__INTRO__".to_string(),
            variables: Default::default(),
            visited_knots: Default::default(),
            chosen_choices: Default::default(),
        }
    }
}

impl StoryState {
    /// convert the state into RON (rusty object notation) in a human-readable manner
    pub fn state_to_ron_pretty(&self) -> String {
        let pretty = PrettyConfig::new()
            .separate_tuple_members(true)
            .decimal_floats(true);
        to_string_pretty(self, pretty).expect("Serialization failed")
    }

    /// convert the state into RON (rusty object notation) in a dense manner
    pub fn state_to_ron(&self) -> String {
        to_string(self).expect("Serialization failed")
    }

    /// import state from a RON (rusty object notation) &str
    pub fn from_ron(ron: &str) -> Self {
        from_str(ron).expect("Failed to parse ron")
    }
}

#[allow(clippy::comparison_to_empty)]
impl<'a> StoryRunner<'a> {
    /// create a StoryRunner from a &str
    pub fn build_from_str(text: &str) -> Self {
        let mut runner = StoryRunner {
            state: Default::default(),
            story: Default::default(),
        };
        runner.import_story(text);
        runner
    }

    /// reload the story from a &str
    /// this leaks that story's memory, so don't do this a billion times if you like your RAM to stay small
    pub fn import_story(&mut self, text: &str) {
        self.story = import_story(text);
    }

    pub fn replace_story(&mut self, story: InkStory<'a>) {
        self.story = story;
        self.state = StoryState::default();
    }

    // TODO: take into account which non-sticky choices have already been visited too
    /// gives the options for the current choices that the player can choose
    pub fn get_options(&self) -> Vec<String> {
        if self.state.current_knot_title == "END" {
            return vec![];
        }

        let current_knot = self
            .story
            .knots
            .get(self.state.current_knot_title.as_str())
            .expect(&format!(
                "knot not found: \"{}\"",
                self.state.current_knot_title.as_str()
            )); // TODO: error

        match &current_knot.end {
            KnotEnd::Divert(_) => unreachable!(),
            KnotEnd::Choices(choices) => {
                let choices = choices
                    .iter()
                    .filter_map(|c| {
                        // TODO: make sure the conditionals all non-zero

                        if !c
                            .conditionals
                            .iter()
                            .all(|c| self.is_truthy(&self.evaluate_expression(c)))
                        {
                            return None;
                        }

                        if c.sticky
                            || !self.state.chosen_choices.contains(&(
                                self.state.current_knot_title.to_string(),
                                c.choice_text.to_string(),
                            ))
                        {
                            Some(c.choice_text.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                // If only fallback remains, emit it. Otherwise, remove it.
                if choices.len() == 1 && choices[0] == "" {
                    vec!["".to_string()]
                } else {
                    choices.into_iter().filter(|s| s != "").collect()
                }
            }
        }
    }

    pub fn evaluate_expression(&self, exp: &Expression) -> VariableValue {
        match exp {
            Expression::KnotVisited(s) => VariableValue::Address(s.clone()),
            Expression::Not(e) => {
                if self.is_truthy(&self.evaluate_expression(e)) {
                    VariableValue::Int(0)
                } else {
                    VariableValue::Int(1)
                }
            }
        }
    }

    pub fn is_truthy(&self, val: &VariableValue) -> bool {
        match val {
            VariableValue::Int(i) => *i != 0,
            VariableValue::Float(f) => *f != 0.0,
            VariableValue::Address(s) => {
                if let Some(x) = self.state.visited_knots.get(s) {
                    *x != 0
                } else {
                    false
                }
            }
            VariableValue::Content(s) => !s.is_empty(),
        }
    }

    /// start the story from the beginning; returns the text that should be shown
    pub fn start(&mut self) -> Vec<OutputLine> {
        // TODO: get all the global variables at once?

        let title = self.state.current_knot_title.clone();
        self.run_knot(&title)
    }

    /// steps the story, and gives the text that should be displayed
    pub fn step(&mut self, choice: &str) -> Vec<OutputLine> {
        self.run_choice(choice)
    }

    // TODO: maybe this should return a tuple: lines and choices
    fn run_knot(&mut self, knot_title: &str) -> Vec<OutputLine> {
        let previous_knot_title = self.state.current_knot_title.clone();
        self.state.current_knot_title = knot_title.to_string();

        // Increment the count of this knot being visited
        self.increment_visited(&knot_title);

        // If we entered a new knot's stitch, increment its visited count too
        // TODO: Test this !
        if let Some((knot, _)) = knot_title.split_once('.') {
            let previous_knot_title =
                if let Some((previous_knot, _)) = previous_knot_title.split_once('.') {
                    previous_knot.to_string()
                } else {
                    previous_knot_title
                };

            if knot != previous_knot_title {
                self.increment_visited(&previous_knot_title);
            }
        }

        if knot_title == "END" {
            return vec![];
        }

        let mut output = vec![];

        let knot = self
            .story
            .knots
            .get(knot_title)
            .expect(&format!("Couldn't find: {}", knot_title)) // TODO: error
            .clone();

        output.append(
            &mut knot
                .lines
                .clone()
                .into_iter()
                .map(|x| match x {
                    Line::Dialog(s) => s.into(),
                    Line::Operation(_) => todo!(), // TODO
                })
                .collect::<Vec<OutputLine>>(),
        );

        match &knot.end {
            KnotEnd::Divert(d) => {
                output.append(&mut self.run_knot(&d.knot_title));
            }
            KnotEnd::Choices(_) => {
                // If there is only one choice, and it's the fallback "" then run it.
                // Otherwise, stop running here.
                let choices = self.get_options();
                if choices.len() == 1 && choices[0] == "" {
                    output.append(&mut self.run_choice(""));
                }
            }
        }

        output
    }

    // TODO: this gives DialogLine. Should it? Maybe a String version instead?
    fn run_choice(&mut self, choice_str: &str) -> Vec<OutputLine> {
        self.state.chosen_choices.insert((
            self.state.current_knot_title.to_string(),
            choice_str.to_string(),
        ));

        let options = match &self
            .story
            .knots
            .get(self.state.current_knot_title.as_str())
            .unwrap()
            .end
        {
            KnotEnd::Choices(o) => o.clone(),
            KnotEnd::Divert(_) => unreachable!(),
        };

        let choice = match options.iter().find(|o| o.choice_text == choice_str) {
            Some(c) => c,
            None => {
                dbg!(choice_str);
                dbg!(options);
                todo!()
            } // TODO: return an error
        };

        let mut output = vec![];

        if choice.shown_text != "" {
            output.push(choice.shown_text.clone().into());
        }

        output.append(
            &mut choice
                .lines
                .clone()
                .into_iter()
                .map(|x| match x {
                    Line::Dialog(s) => s.into(),
                    _ => todo!(),
                })
                .collect::<Vec<OutputLine>>(),
        );

        output.append(&mut self.run_knot(&choice.divert.knot_title));

        output
    }

    pub fn increment_visited(&mut self, knot_title: &str) {
        if let Some(count) = self.state.visited_knots.get_mut(knot_title) {
            *count += 1;
        } else {
            self.state.visited_knots.insert(knot_title.to_string(), 1);
        }
    }

    pub fn set_knot(&mut self, knot: &str) {
        self.state.current_knot_title = knot.to_string();
    }
}

pub fn import_story<'a>(text: &str) -> InkStory<'a> {
    let stripped = strip_comments(text);
    let stripped = Box::leak(Box::new(stripped)); // TODO: this is a nasty way to make the string 'static, but it works...
    let tokens = lex(stripped);
    lexed_to_parsed(&tokens)
}
