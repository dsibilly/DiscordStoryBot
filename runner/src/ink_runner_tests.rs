#![cfg(test)]

use crate::ink_parser::{Expression, VariableValue};
use crate::ink_runner::{OutputLine, StoryRunner, StoryState};
use pretty_assertions::assert_eq;

#[test]
fn test_strip_comments() {
    assert_eq!(true, true);
}

#[test]
fn serialize_to_ron() {
    let data = StoryState::default();

    let ron = data.state_to_ron();
    let data_from_ron = StoryState::from_ron(&ron);

    println!("Ron: {}", ron);

    assert_eq!(data_from_ron, data);
    assert_eq!(
        "(current_knot_title:\"__INTRO__\",variables:{},visited_knots:{},chosen_choices:[])",
        ron
    );
}

#[test]
fn run_story() {
    let mut runner = StoryRunner::build_from_str(
        r"VAR cool = 100
        -> town
        == town
        you are in town
        -> END",
    );

    assert_eq!(runner.start(), vec!["you are in town".into()]);
}

#[test]
fn run_story_with_choices() {
    let story = r"Which way to go?
        * to town
          let's go to town
        -> END
        * to the moon
          let's go do the moon
        -> END";
    let mut runner = StoryRunner::build_from_str(story);

    let start_lines = runner.start();
    assert_eq!(start_lines, vec!["Which way to go?".into()]);

    let options = runner.get_options();
    assert_eq!(options, vec!["to town", "to the moon"]);

    let stepped = runner.step("to town");
    assert_eq!(stepped, vec!["to town".into(), "let's go to town".into()]);
}

#[test]
fn run_story_with_choices_longer() {
    let story = r"Which way to go?
        * to town
          let's go to town
        -> town
        * to the moon
          let's go to the moon
        -> moon
        == town
        town is cool
        -> END
        == moon
        moon is great
        -> END";
    let mut runner = StoryRunner::build_from_str(story);

    let start_lines = runner.start();
    assert_eq!(start_lines, vec!["Which way to go?".into()]);

    let options = runner.get_options();
    assert_eq!(options, vec!["to town", "to the moon"]);

    let stepped = runner.step("to town");
    assert_eq!(
        stepped,
        vec![
            "to town".into(),
            "let's go to town".into(),
            "town is cool".into()
        ]
    );

    let mut runner = StoryRunner::build_from_str(story);

    runner.start();
    runner.get_options();
    let stepped = runner.step("to the moon");
    assert_eq!(
        stepped,
        vec![
            "to the moon".into(),
            "let's go to the moon".into(),
            "moon is great".into()
        ]
    );
}

#[test]
fn run_story1() {
    let story = include_str!("../samples/story1.ink");
    let mut runner = StoryRunner::build_from_str(story);

    dbg!(&runner.story);

    let start_lines = runner.start();
    assert_eq!(
        start_lines,
        vec![
            "LONDON, 1872".into(),
            "Residence of Monsieur Phileas Fogg.".into(),
            "It was cool downtown.".into(),
            "Suburbs were cool too.".into(),
            "Monsieur Phileas Fogg returned home early from the Reform Club, and in a new-fangled steam-carriage, besides!".into(),
            "health: \"{health}\"".into(),
            "\"Passepartout,\" said he. \"We are going around the world!\"".into(),
        ]
    );

    let options = runner.get_options();
    assert_eq!(options, vec!["❤", "🙂"]);

    let stepped = runner.step("❤");
    assert_eq!(
        stepped,
        vec![
            "❤".into(),
            "I was utterly astonished.".into(),
            "whoa!".into()
        ]
    );

    let options = runner.get_options();
    assert_eq!(options, vec!["🙁"]);

    let mut runner = StoryRunner::build_from_str(story);

    runner.start();
    runner.get_options();
    let stepped = runner.step("🙂");
    assert_eq!(
        stepped,
        vec![
            "🙂".into(),
            "I nodded curtly, not believing a word of it.".into(),
            "It's the ending!".into()
        ]
    );
}

#[test]
fn run_bot_story() {
    let story = include_str!("../samples/bot.ink");
    let mut runner = StoryRunner::build_from_str(story);

    dbg!(&runner.story);

    let start_lines = runner.start();
    assert_eq!(
        start_lines,
        vec![
            "LONDON, 1872".into(),
            "Residence of Monsieur Phileas Fogg.".into(),
            OutputLine {
                text: "It was cool downtown.".into(),
                tags: vec!["downtown tag".into(), "tag ya".into()],
            },
            OutputLine {
                text: "Suburbs were cool too.".into(),
                tags: vec!["suburbs tag".into(), "tag too".into()],
            },
            OutputLine {
                text: "Monsieur Phileas Fogg returned home early from the Reform Club, and in a new-fangled steam-carriage, besides!".into(),
                tags: vec!["health +1".into(), "tag1".into(), "tag2".into()],
            },
            OutputLine {
                text: "health: \"{health}\"".into(),
                tags: vec![]
            },
            OutputLine {
                text: "\"Passepartout,\" said he. \"We are going around the world!\"".into(),
                tags: vec!["tag 4".into(), "tag 3".into()],
            },
        ]
    );

    let options = runner.get_options();
    assert_eq!(options, vec!["❤", "🙂"]);

    let stepped = runner.step("❤");
    assert_eq!(
        stepped,
        vec!["❤".into(), "I was utterly astonished.".into(), "\"You are in jest!\" I told him in dignified affront. \"You make mock of me, Monsieur.\"".into(), "\"I am quite serious.\"".into()]
    );

    let options = runner.get_options();
    assert_eq!(options, vec!["🙁"]);

    let mut runner = StoryRunner::build_from_str(story);

    runner.start();
    runner.get_options();
    let stepped = runner.step("🙂");
    assert_eq!(
        stepped,
        vec![
            "🙂".into(),
            "I nodded curtly, not believing a word of it.".into(),
            "\"We shall circumnavigate the globe within eighty days.\" He was quite calm as he proposed this wild scheme. \"We leave for Paris on the 8:25. In an hour.\"".into()
        ]
    );

    assert_eq!(runner.get_options(), Vec::<String>::new());
}

#[test]
fn run_recursive() {
    let story = include_str!("../samples/recursive.ink");
    let mut runner = StoryRunner::build_from_str(story);
    //dbg!(&runner.story);

    let start_lines = runner.start();
    assert_eq!(start_lines, vec!["start".into(), "At place A.".into()]);

    let options = runner.get_options();
    assert_eq!(
        options,
        vec!["stick.", "also stick.", "once only.", "also once only."]
    );

    let stepped = runner.step("stick.");
    assert_eq!(
        stepped,
        vec!["stick.".into(), "stuck".into(), "At place A.".into()]
    );
    let options = runner.get_options();
    assert_eq!(
        options,
        vec!["stick.", "also stick.", "once only.", "also once only."]
    );

    let stepped = runner.step("once only.");
    assert_eq!(
        stepped,
        vec!["once only.".into(), "uno".into(), "At place A.".into()]
    );
    let options = runner.get_options();
    assert_eq!(options, vec!["stick.", "also stick.", "also once only."]);

    let stepped = runner.step("also once only.");
    assert_eq!(
        stepped,
        vec![
            "also once only.".into(),
            "uno two".into(),
            "At place A.".into()
        ]
    );
    let options = runner.get_options();
    assert_eq!(options, vec!["stick.", "also stick."]);

    let stepped = runner.step("also stick.");
    assert_eq!(stepped, vec!["also stick.".into(), "so stuck".into()]);
    let options = runner.get_options();
    let empty: Vec<String> = vec![];
    assert_eq!(options, empty);
}

#[test]
fn run_fallbacks() {
    let story = include_str!("../samples/fallbacks.ink");
    let mut runner = StoryRunner::build_from_str(story);
    dbg!(&runner.story);

    let start_lines = runner.start();
    assert_eq!(start_lines, vec!["hey".into(), "sup".into()]);

    let options = runner.get_options();
    assert_eq!(options, vec!["wut", "wutwut"]);

    assert_eq!(runner.step("wut"), vec!["wut".into(), "sup".into()]);
    assert_eq!(runner.get_options(), vec!["wutwut"]);

    assert_eq!(
        runner.step("wutwut"),
        vec![
            "wutwut".into(),
            "text".into(),
            "sup".into(),
            "can I put things here?".into(),
            "sup2".into()
        ]
    );
    assert_eq!(runner.get_options(), vec!["wut2"]);

    assert_eq!(runner.step("wut2"), vec!["wut2".into(), "sup2".into(),]);
    let empty: Vec<String> = vec![];
    assert_eq!(runner.get_options(), empty);
}

#[test]
fn run_stitches() {
    let story = include_str!("../samples/stitches.ink");
    let mut runner = StoryRunner::build_from_str(story);
    dbg!(&runner.story);

    let start_lines = runner.start();
    assert_eq!(
        start_lines,
        vec!["first class".into(), "you missed the train".into()]
    );

    let empty: Vec<String> = vec![];
    assert_eq!(runner.get_options(), empty);
}

#[test]
fn run_stitches_advanced() {
    let story = include_str!("../samples/stitches_advanced.ink");
    let mut runner = StoryRunner::build_from_str(story);
    dbg!(&runner.story);

    let start_lines = runner.start();
    assert_eq!(
        start_lines,
        vec![
            "Train time!".into(),
            "first class".into(),
            "bus time".into(),
            "train was better".into(),
            "second class".into(),
            "you missed the train".into(),
        ]
    );

    let empty: Vec<String> = vec![];
    assert_eq!(runner.get_options(), empty);
}

#[test]
fn run_stitches_with_choices() {
    let story = include_str!("../samples/stitches_with_choices.ink");
    let mut runner = StoryRunner::build_from_str(story);
    dbg!(&runner.story);

    let start_lines = runner.start();
    assert_eq!(start_lines, vec!["first class".into(),]);

    assert_eq!(runner.get_options(), vec!["be late", "be early"]);

    assert_eq!(
        runner.step("be early"),
        vec![
            "be early".into(),
            "second class".into(),
            "you missed the train".into()
        ]
    );

    assert_eq!(runner.get_options(), vec!["cool", "uncool"]);

    assert_eq!(
        runner.step("uncool"),
        vec!["uncool".into(), "you missed the train".into()]
    );

    assert_eq!(runner.get_options(), vec!["cool"]);

    assert_eq!(runner.step("cool"), vec!["cool".into()]);

    let empty: Vec<String> = vec![];
    assert_eq!(runner.get_options(), empty);
}

#[test]
fn test_evaluate_expression() {
    let mut runner = StoryRunner::build_from_str(
        "VAR nine = 9\n->space\n== space\n* done? -> END\n* {zoo} saw zoo -> END\n* {not zoo} did not see zoo -> END\n* {space} saw space -> END\n* {not space} did not see space -> END\n== zoo\n-> END",
    );
    runner.start();

    assert_eq!(runner.is_truthy(&VariableValue::Int(0)), false);
    assert_eq!(runner.is_truthy(&VariableValue::Int(1)), true);
    assert_eq!(runner.is_truthy(&VariableValue::Int(-1)), true);

    assert_eq!(runner.is_truthy(&VariableValue::Float(0.0)), false);
    assert_eq!(runner.is_truthy(&VariableValue::Float(0.1)), true);
    assert_eq!(runner.is_truthy(&VariableValue::Float(1.0)), true);
    assert_eq!(runner.is_truthy(&VariableValue::Float(-1.0)), true);

    assert_eq!(
        runner.is_truthy(&VariableValue::Content("".to_string())),
        false
    );
    assert_eq!(
        runner.is_truthy(&VariableValue::Content("0".to_string())),
        true
    );
    assert_eq!(
        runner.is_truthy(&VariableValue::Content("1".to_string())),
        true
    );

    assert_eq!(
        runner.is_truthy(&VariableValue::Address("not a knot name".to_string())),
        false
    );
    assert_eq!(
        runner.is_truthy(&VariableValue::Address("zoo".to_string())),
        false
    );
    assert_eq!(
        runner.is_truthy(&VariableValue::Address("space".to_string())),
        true
    );

    assert_eq!(
        runner.evaluate_expression(&Expression::KnotVisited("space".to_string())),
        VariableValue::Address("space".to_string())
    );
    assert_eq!(
        runner.evaluate_expression(&Expression::Not(Box::new(Expression::KnotVisited(
            "zoo".to_string()
        )))),
        VariableValue::Int(1)
    );
    assert_eq!(
        runner.evaluate_expression(&Expression::Not(Box::new(Expression::KnotVisited(
            "space".to_string()
        )))),
        VariableValue::Int(0)
    );

    let choices = runner.get_options();
    assert_eq!(
        choices,
        vec![
            "done?".to_string(),
            "did not see zoo".to_string(),
            "saw space".to_string()
        ]
    );
}
