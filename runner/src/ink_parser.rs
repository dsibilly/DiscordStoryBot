use crate::ink_lexer::InkToken;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

// TODO: Should all these &str be String instead?
//       - pros: deserializable, doesn't require lifetime stuff
//       - cons: maybe slower because more allocations
//       - alternatives: have it store the String too
//       - - SOLUTION: I just leaked the string, so that maks it 'static

// TODO: -> DONE and -> END

// TODO: for tags, it seems like tags always attach to the next line, even if that's after a divert. Should we do this too? Does it matter?

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum VariableValue {
    Int(i32),
    Float(f32),
    Content(String),
    Address(String),
}

impl Display for VariableValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableValue::Int(int) => write!(f, "{}", int),
            VariableValue::Float(float) => write!(f, "{}", float),
            VariableValue::Address(address) => write!(f, "{}", address.clone()),
            VariableValue::Content(content) => write!(f, "{}", content.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct InkStory<'a> {
    pub global_variables_and_constants: BTreeMap<&'a str, VariableValue>,
    pub knots: BTreeMap<String, Knot<'a>>,
    pub global_tags: Vec<&'a str>,
}

impl<'a> InkStory<'a> {
    pub fn get_author(&self) -> Option<String> {
        dbg!(&self.global_tags);

        let authors: Vec<String> = self
            .global_tags
            .iter()
            .filter_map(|&t| get_author_from_tag(t))
            .collect();

        dbg!(&authors);

        if authors.is_empty() {
            None
        } else {
            Some(authors[0].clone())
        }
    }

    pub fn get_title(&self) -> Option<String> {
        dbg!(&self.global_tags);

        let titles: Vec<String> = self
            .global_tags
            .iter()
            .filter_map(|&t| get_title_from_tag(t))
            .collect();

        dbg!(&titles);

        if titles.is_empty() {
            None
        } else {
            Some(titles[0].clone())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DialogLine<'a> {
    pub text: &'a str,
    pub has_newline: bool,
    pub tags: Vec<&'a str>,
}

impl<'a> From<&'a str> for DialogLine<'a> {
    fn from(s: &'a str) -> Self {
        DialogLine {
            text: s,
            has_newline: true,
            tags: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Knot<'a> {
    pub title: String,
    pub lines: Vec<Line<'a>>,
    pub end: KnotEnd<'a>,
    pub knot_tags: Vec<&'a str>,
}

impl<'a> Default for Knot<'a> {
    fn default() -> Self {
        Knot {
            title: "__INTRO__".to_string(),
            lines: vec![],
            end: "END".into(),
            knot_tags: vec![],
        }
    }
}

impl<'a> Knot<'a> {
    fn new(s: &str) -> Self {
        Knot {
            title: s.to_string(),
            lines: vec![],
            end: KnotEnd::Divert("END".into()),
            knot_tags: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KnotEnd<'a> {
    Choices(Vec<Choice<'a>>),
    Divert(Divert),
}

impl<'a> From<&'a str> for KnotEnd<'a> {
    fn from(s: &'a str) -> Self {
        KnotEnd::Divert(s.into())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Choice<'a> {
    pub conditionals: Vec<Expression>,
    pub choice_text: String,
    pub shown_text: String,
    pub has_newline: bool,
    pub lines: Vec<Line<'a>>,
    pub divert: Divert,
    pub sticky: bool,
}

impl<'a> Default for Choice<'a> {
    fn default() -> Self {
        Choice {
            conditionals: vec![],
            choice_text: "".to_string(),
            shown_text: "".to_string(),
            has_newline: true,
            lines: vec![],
            divert: Default::default(),
            sticky: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Not(Box<Expression>),
    Identifier(String),
}

impl From<&str> for Expression {
    fn from(knot_title: &str) -> Self {
        Expression::Identifier(knot_title.to_string())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Divert {
    pub knot_title: String,
}

impl Default for Divert {
    fn default() -> Self {
        "END".into()
    }
}

impl From<&str> for Divert {
    fn from(s: &str) -> Self {
        Divert {
            knot_title: s.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Line<'a> {
    Dialog(DialogLine<'a>),
    Expression(Expression),
    Operation(Operation),
}

impl<'a> From<&'a str> for Line<'a> {
    fn from(s: &'a str) -> Self {
        Line::Dialog(s.into())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct VariableDeclaration<'a> {
    name: &'a str,
    statement: Statement,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Statement {}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Operation {}

pub fn lexed_to_parsed<'a>(tokens: &[InkToken<'a>]) -> InkStory<'a> {
    let mut story = InkStory::default();
    let mut index = 0;

    let mut starting_lines: Vec<Line<'_>> = vec![];
    let mut starting_divert = None;
    let mut starting_choices: Vec<Choice<'_>> = vec![];
    let mut starting_tags: Vec<&str> = vec![];

    while index < tokens.len() {
        match tokens[index] {
            InkToken::VariableDeclaration(s) => {
                let (segment0, segment1) = s.split_once('=').unwrap();

                // TODO: error if it's already there
                story
                    .global_variables_and_constants
                    .insert(segment0.trim(), parse_variable_value(segment1));
                index += 1;
            }
            InkToken::Dialog((s, has_newline)) => {
                let tags = if starting_lines.is_empty() {
                    starting_tags.clone()
                } else {
                    vec![]
                };

                // If it's got {}s in it, split them out.
                if s.contains('{') && s.contains('}') {
                    starting_lines.append(&mut parse_internal_expressions(s, has_newline, tags));
                } else {
                    starting_lines.push(Line::Dialog(DialogLine {
                        text: s,
                        has_newline,
                        tags,
                    }));
                }
                index += 1;
            }
            InkToken::KnotTitle(s) => {
                index += 1;
                let (knots, index_out) = parse_knot(s, &tokens[index..], false);
                for knot in knots {
                    story.knots.insert(knot.title.clone(), knot);
                }
                index += index_out;

                // if __INTRO__ has no end, it's now a divert to this knot
                if starting_choices.is_empty() && starting_divert.is_none() {
                    starting_divert = Some(KnotEnd::Divert(s.into()));
                }
            }
            InkToken::Divert(s) => {
                starting_divert = Some(KnotEnd::Divert(s.into()));
                index += 1;
            }
            InkToken::Choice((s, has_newline)) => {
                let (choice, index_out) = parse_choice(s, &tokens[index..], false, has_newline);
                starting_choices.push(choice);
                index += index_out;
            }
            InkToken::StickyChoice((s, has_newline)) => {
                let (choice, index_out) = parse_choice(s, &tokens[index..], true, has_newline);
                starting_choices.push(choice);
                index += index_out;
            }
            InkToken::Tag(s) => {
                if starting_lines.is_empty() {
                    // If there are no lines yet, attach tags to the knot
                    starting_tags.push(s);
                    story.global_tags.push(s);
                } else {
                    // If there was a line, attach the tag to the most recent line

                    // TODO: refactor this. It's a mess.
                    let l = starting_lines.len() - 1;
                    if let Line::Dialog(last_line) = &mut starting_lines[l].clone() {
                        last_line.tags.push(s);
                        starting_lines.pop();
                        starting_lines.push(Line::Dialog(DialogLine {
                            text: last_line.text,
                            has_newline: last_line.has_newline,
                            tags: last_line.tags.clone(),
                        }))
                    } else {
                        unimplemented!();
                    }
                }
                index += 1;
            }
            _ => {
                //index += 1;
                dbg!(&tokens[index]);
                unimplemented!();
            }
        }
    }

    story.knots.insert(
        "__INTRO__".to_string(),
        Knot {
            title: "__INTRO__".to_string(),
            lines: starting_lines,
            end: starting_divert.unwrap_or(if starting_choices.is_empty() {
                KnotEnd::Divert("END".into())
            } else {
                KnotEnd::Choices(starting_choices)
            }),
            knot_tags: starting_tags.clone(),
        },
    );

    story
}

fn parse_variable_value(s: &str) -> VariableValue {
    if let Ok(i) = s.trim().parse::<i32>() {
        VariableValue::Int(i)
    } else if let Ok(f) = s.trim().parse::<f32>() {
        VariableValue::Float(f)
    } else if s.trim().starts_with("->") {
        VariableValue::Address(s.trim()[2..].trim().to_string())
    } else if s.trim().starts_with('"') && s.trim().ends_with('"') {
        let quoted_text = s.trim();
        VariableValue::Content(s.trim()[1..quoted_text.len() - 1].to_string())
    } else {
        // TODO: error
        dbg!(s);
        unimplemented!()
    }
}

fn parse_knot<'a>(title: &str, tokens: &[InkToken<'a>], is_stitch: bool) -> (Vec<Knot<'a>>, usize) {
    // TODO: operations
    let mut knot = Knot::new(title);
    let mut index = 0;
    let mut tag_buildup = vec![];
    let mut first_dialog = true;
    let mut stitches = vec![];
    let mut stitch_knots = vec![];
    let mut first_stitch_title = None;

    let mut choices = vec![];

    while index < tokens.len() {
        match tokens[index] {
            InkToken::Dialog((s, has_newline)) => {
                if first_dialog {
                    knot.knot_tags = tag_buildup.clone();
                    first_dialog = false;
                }
                knot.lines.push(Line::Dialog(DialogLine {
                    text: s,
                    has_newline,
                    tags: tag_buildup,
                }));
                // !!! TODO: there's no way to know if a following tag was on the same line as a dialog line. Do we need to make sure tags always go first? Or is there another way to do this?
                //           ^-- currently only tags that come before a line will be included
                //           - Maybe the solution is to have two kinds of tags (lonely tag, and end-of-line tag), and have the end-of-line one come before dialog, so it's easy to parse.

                // TODO: document that our tags don't follow past diverts, so don't have trailing tags, please.

                // TODO: it seems like remaining tags at the end just get appended to whichever line was last. Weird.
                //       ... but not really. They make their own empty line that isn't a line. Bah. Weird.
                tag_buildup = vec![];
                index += 1;
            }
            InkToken::Tag(s) => {
                tag_buildup.push(s);
                index += 1;
            }
            InkToken::Divert(s) => {
                knot.end = KnotEnd::Divert(s.into());
                index += 1;

                // Only break if the next thing isn't a stitch title
                if tokens.len() > index {
                    if let InkToken::StitchTitle(_) = tokens[index] {
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            InkToken::Choice((s, has_newline)) => {
                let (choice, index_out) = parse_choice(s, &tokens[index..], false, has_newline);
                choices.push(choice);
                index += index_out;
            }
            InkToken::StickyChoice((s, has_newline)) => {
                let (choice, index_out) = parse_choice(s, &tokens[index..], true, has_newline);
                choices.push(choice);
                index += index_out;
            }
            InkToken::Operation(_) => {
                // TODO
                index += 1;
                //todo!();
            }
            InkToken::KnotTitle(_) => {
                break;
            }
            InkToken::StitchTitle(s) => {
                if is_stitch {
                    break;
                }

                index += 1;
                let (mut stitch, index_out) =
                    parse_knot(&(title.to_string() + "." + s), &tokens[index..], true);
                stitches.push(s);

                stitch_knots.append(&mut stitch);
                if first_stitch_title.is_none() {
                    first_stitch_title = Some(s);
                }
                index += index_out;
            }
            _ => {
                //index += 1;
                dbg!(&tokens[index]);
                unimplemented!();
            }
        }
    }

    if !choices.is_empty() {
        knot.end = KnotEnd::Choices(choices);
    } else if !stitches.is_empty() {
        knot.end = KnotEnd::Divert(((title.to_string() + "." + stitches[0]).as_str()).into());
    }

    // Once the knot is parsed, find diverts and justify them:
    //       first_class --> train.first_class, for example
    for stitch in stitch_knots.iter_mut() {
        // If the divert has a dot, don't do these checks
        match &mut stitch.end {
            KnotEnd::Divert(d) => {
                let divert_str = &d.knot_title;
                if !divert_str.contains('.') && stitches.contains(&divert_str.as_str()) {
                    d.knot_title = knot.title.to_string() + "." + divert_str;
                    dbg!(&d.knot_title);
                }
            }
            KnotEnd::Choices(choices) => {
                for c in choices {
                    let divert_str = &c.divert.knot_title;
                    if !divert_str.contains('.') && stitches.contains(&divert_str.as_str()) {
                        c.divert.knot_title = knot.title.to_string() + "." + divert_str;
                        dbg!(&c.divert.knot_title);
                    }
                }
            }
        }
    }

    let mut x = vec![knot];
    x.append(&mut stitch_knots);
    (x, index)
}

fn parse_choice<'a>(
    title: &'a str,
    tokens: &[InkToken<'a>],
    sticky: bool,
    has_newline: bool,
) -> (Choice<'a>, usize) {
    let mut title = title.trim();

    let mut conditionals: Vec<Expression> = vec![];

    if title.starts_with('{') && title.contains('}') {
        let close_index = title.find('}').unwrap();
        let conditional_text = &title[1..close_index];
        conditionals.push(parse_expression(conditional_text.trim()));
        title = title[close_index + 1..].trim();
    }

    let mut choice_text = title.to_string();
    let mut shown_text = title.to_string();

    if title.contains('[') && title.contains(']') {
        let open_index = title.find('[').unwrap();
        let closed_index = title.find(']').unwrap();

        choice_text = title[0..open_index].to_string() + &title[(open_index + 1)..closed_index];
        shown_text = title[0..open_index].to_string() + &title[(closed_index + 1)..];

        choice_text = choice_text.trim().to_string();
        shown_text = shown_text.trim().to_string();
    }

    let mut choice = Choice {
        conditionals,
        choice_text,
        shown_text,
        has_newline,
        lines: vec![],
        divert: Default::default(),
        sticky,
    };

    let mut index = 1; // so we skip our own self

    while index < tokens.len() {
        match tokens[index] {
            InkToken::Dialog((s, has_newline)) => {
                choice.lines.push(Line::Dialog(DialogLine {
                    text: s,
                    has_newline,
                    tags: vec![],
                })); // TODO: tags on this line too
                index += 1;
            }
            InkToken::Tag(_) => {
                // TODO
                index += 1;
            }
            InkToken::Divert(s) => {
                choice.divert = s.into();
                index += 1;
                break;
            }
            InkToken::Choice(_) => {
                break;
            }
            InkToken::StickyChoice(_) => {
                break;
            }
            _ => todo!(), // TODO: error, as it's not allowed in a Choice
        }
    }

    (choice, index)
}

pub fn parse_expression(text: &str) -> Expression {
    if let Some(text) = text.to_lowercase().strip_prefix("not ") {
        Expression::Not(Box::from(parse_expression(text)))
    } else {
        Expression::Identifier(text.to_string())
    }
}

/// When a dialog line has {}'s in it, this will parse them out.
pub fn parse_internal_expressions<'a>(
    text: &'a str,
    has_newline: bool,
    tags: Vec<&'a str>,
) -> Vec<Line<'a>> {
    let open_brace = text.find('{').unwrap();
    let close_brace = text.find('}').unwrap();

    let mut output = vec![Line::Dialog(DialogLine {
        text: text[0..open_brace].trim_start(),
        has_newline: false,
        tags,
    })];

    output.push(Line::Expression(parse_expression(
        text[open_brace + 1..close_brace].trim(),
    )));

    let new_text = &text[close_brace + 1..]; //.trim();
    if new_text.contains('{') && new_text.contains('}') {
        output.append(&mut parse_internal_expressions(
            new_text,
            has_newline,
            vec![],
        ));
    } else {
        dbg!(&text);
        dbg!(&open_brace);
        output.push(Line::Dialog(DialogLine {
            text: text[close_brace + 1..].trim_end(),
            has_newline,
            tags: vec![],
        }));
    }

    output
    // TODO: strip out empty dialog lines?
}

pub fn get_author_from_tag(tag: &str) -> Option<String> {
    tag.strip_prefix("author:").map(|s| s.trim().to_string())
}

pub fn get_title_from_tag(tag: &str) -> Option<String> {
    tag.strip_prefix("title:").map(|s| s.trim().to_string())
}
