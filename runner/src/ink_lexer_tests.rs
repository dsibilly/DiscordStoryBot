#![cfg(test)]

use crate::ink_lexer::{lex, strip_comments, InkToken::*};
use pretty_assertions::assert_eq;

#[test]
fn test_strip_comments() {
    assert_eq!(strip_comments("dialog"), "dialog".to_string());
    assert_eq!(strip_comments("// comment"), "".to_string());
    assert_eq!(strip_comments("dialog // comment"), "dialog".to_string());
    assert_eq!(
        strip_comments("dialog // comment\ndialog 2"),
        "dialog\ndialog 2".to_string()
    );
    assert_eq!(
        strip_comments("dialog\ndialog 2 // comment"),
        "dialog\ndialog 2".to_string()
    );
    assert_eq!(
        strip_comments("// comment\ndialog\ndialog 2 // comment"),
        "\ndialog\ndialog 2".to_string()
    );
    assert_eq!(
        strip_comments("/* comment*/\ndialog\ndialog 2 // comment"),
        "dialog\ndialog 2".to_string()
    );
    assert_eq!(
        strip_comments("/* comment\n*/dialog\ndialog 2 // comment"),
        "dialog\ndialog 2".to_string()
    );
    assert_eq!(
        strip_comments("/* comment\ndialog*/\ndialog 2 // comment"),
        "dialog 2".to_string()
    );
    assert_eq!(
        strip_comments("/* comment\n*/ dialog\ndialog 2 /* comment */"),
        "dialog\ndialog 2".to_string()
    );
    assert_eq!(
        strip_comments(include_str!("../samples/comments.ink")),
        "\"What do you make of this?\" she asked.\n\n\"I couldn't possibly comment,\" I replied."
    );
}

#[test]
fn test_lex() {
    assert_eq!(
        lex(include_str!("../samples/story_with_variables.ink")),
        vec![
            VariableDeclaration("health = 100"),
            Dialog(("LONDON, 1872", true)),
            Dialog(("Residence of Monsieur Phileas Fogg.", false)),
            Divert("paris_downtown"),
            KnotTitle("paris_downtown"),
            Tag("downtown tag"),
            Tag("tag ya"),
            Dialog(("It was cool downtown.", true)),
            Divert("paris_suburbs"),
            KnotTitle("paris_suburbs"),
            Tag("suburbs tag"),
            Tag("tag too"),
            Dialog(("Suburbs were cool too.", true)),
            Operation("health -= 2"),
            Divert("london"),
            KnotTitle("london"),
            Tag("tag1"),
            Tag("tag2"),
            Dialog(("Monsieur Phileas Fogg returned home early from the Reform Club, and in a new-fangled steam-carriage, besides!", true)),
            Dialog(("health: \"{health}\"", true)),
            Tag("tag 4"),
            Tag("tag 5"),
            Tag("tag 3"),
            Dialog(("\"Passepartout,\" said he. \"We are going around the world!\"", true)),
            StickyChoice(("❤", true)),
            Dialog(("I was utterly astonished.", true)),
            Divert("astonished"),
            StickyChoice(("🙂", false)),
            Tag("tag_nod"),
            Tag("tag_nod2"),
            Divert("nod"),
            KnotTitle("astonished"),
            Dialog(("\"You are in jest!\" I told him in dignified affront. \"You make mock of me, Monsieur.\"", true)),
            Dialog(("\"I am quite serious.\"", true)),
            StickyChoice(("🙁", true)),
            Divert("ending"),
            KnotTitle("nod"),
            Choice(("one", false)),
            Tag("tagend1"),
            Tag("tagend2"),
            Divert("ending"),
            Choice(("two", true)),
            Dialog(("I nodded curtly, not believing a word of it.", true)),
            Divert("ending"),
            KnotTitle("ending"),
            Dialog(("\"We shall circumnavigate the globe within eighty days.\" He was quite calm as he proposed this wild scheme. \"We leave for Paris on the 8:25. In an hour.\"", true)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_lex_story1() {
    assert_eq!(
        lex(include_str!("../samples/story1.ink")),
        vec![
            VariableDeclaration("health = 100"),
            Dialog(("LONDON, 1872", true)),
            Dialog(("Residence of Monsieur Phileas Fogg.", true)),
            Divert("downtown"),
            KnotTitle("downtown"),
            Dialog(("It was cool downtown.", true)),
            Divert("suburbs"),
            KnotTitle("suburbs"),
            Dialog(("Suburbs were cool too.", true)),
            Operation("health -= 2"),
            Divert("london"),
            KnotTitle("london"),
            Dialog(("Monsieur Phileas Fogg returned home early from the Reform Club, and in a new-fangled steam-carriage, besides!", true)),
            Dialog(("health: \"{health}\"", true)),
            Dialog(("\"Passepartout,\" said he. \"We are going around the world!\"", true)),
            Choice(("❤", true)),
            Dialog(("I was utterly astonished.", true)),
            Divert("astonished"),
            Choice(("🙂", false)),
            Divert("nod"),
            KnotTitle("astonished"),
            Dialog(("whoa!", true)),
            Choice(("🙁", true)),
            Divert("ending"),
            KnotTitle("nod"),
            Dialog(("I nodded curtly, not believing a word of it.", true)),
            Divert("ending"),
            KnotTitle("ending"),
            Dialog(("It's the ending!", true)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_lex_trailind_equals() {
    assert_eq!(
        lex("=== knot title ===\n-> END"),
        vec![KnotTitle("knot title"), Divert("END"),]
    );
    assert_eq!(
        lex("=== knot title\n-> END"),
        vec![KnotTitle("knot title"), Divert("END"),]
    );
}

#[test]
fn test_lex_fallbacks() {
    assert_eq!(
        lex(&strip_comments(include_str!("../samples/fallbacks.ink"))),
        vec![
            Dialog(("hey", false)),
            Divert("fallback"),
            KnotTitle("fallback"),
            Dialog(("sup", true)),
            Choice(("wut", false)),
            Divert("fallback"),
            Choice(("wutwut", true)),
            Dialog(("text", true)),
            Divert("fallback"),
            StickyChoice(("", true)),
            Dialog(("can I put things here?", true)),
            Divert("fallback2"),
            KnotTitle("fallback2"),
            Dialog(("sup2", true)),
            Choice(("wut2", false)),
            Divert("fallback2"),
            StickyChoice(("", false)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_lex_stitches() {
    assert_eq!(
        lex(&strip_comments(include_str!("../samples/stitches.ink"))),
        vec![
            KnotTitle("train"),
            StitchTitle("first_class"),
            Dialog(("first class", true)),
            Divert("missed_train"),
            StitchTitle("second_class"),
            Dialog(("second class", true)),
            Divert("train.missed_train"),
            StitchTitle("missed_train"),
            Dialog(("you missed the train", true)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_lex_choices() {
    assert_eq!(
        lex(&strip_comments(include_str!("../samples/choices.ink"))),
        vec![
            Dialog(("What do you want to say?", true)),
            Choice(("Hey", true)),
            Dialog(("I'm Bruce.", true)),
            Divert("END"),
            Choice(("[sup]", true)),
            Dialog(("What is up?", true)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_lex_choices_with_hidden_choice_text() {
    assert_eq!(
        lex(&strip_comments(include_str!(
            "../samples/choices_with_hidden_text.ink"
        ))),
        vec![
            Dialog(("What do you want to say?", true)),
            Choice(("[\"Hey\"] \"Sup, my dude?\"", true)),
            Dialog(("He stared at me in disbelief.", true)),
            Divert("END"),
            Choice(("\"Why[?\"] not!\"", true)),
            Dialog(("So we left, right there.", true)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_lex_const() {
    assert_eq!(
        lex(&strip_comments(include_str!("../samples/const.ink"))),
        vec![
            VariableDeclaration("GRAVITY = 9.81"),
            Dialog(("Gravity is an acceleration of {GRAVITY} m/s^2.", true)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_newlines() {
    assert_eq!(
        lex(r"
        this has a newline
        -> next
        == next
        but this does not -> next2
        == next2
        , right?
        -> END"),
        vec![
            Dialog(("this has a newline", true)),
            Divert("next"),
            KnotTitle("next"),
            Dialog(("but this does not", false)),
            Divert("next2"),
            KnotTitle("next2"),
            Dialog((", right?", true)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_lex_newlines() {
    assert_eq!(
        lex(&strip_comments(include_str!(
            "../samples/hidden_choice_text.ink"
        ))),
        vec![
            Tag("hidden"),
            Dialog(("what to do?", true)),
            Choice(("[😊] You smile, a grin as big as the sun.", true)),
            Divert("END"),
            Choice((
                "😀 [- time to smile]- you fight the need to frown, eyes watering.",
                true
            )),
            Divert("END"),
            Choice(("[😎 - be cool] You are being very cool.", false)),
            Divert("END"),
        ]
    );
}

#[test]
fn test_glue() {
    assert_eq!(
        lex(&strip_comments(include_str!("../samples/glue.ink"))),
        vec![
            Dialog("What do you want to say?"),
            Choice("[\"Hey\"] \"Sup, my dude?\""),
            Dialog("He stared at me in disbelief."),
            Divert("END"),
            Choice("\"Why[?\"] not!\""),
            Dialog("So we left, right there."),
            Divert("END"),
        ]
    );
}
