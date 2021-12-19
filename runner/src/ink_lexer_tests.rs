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
            Dialog("LONDON, 1872"),
            Dialog("Residence of Monsieur Phileas Fogg."),
            Divert("paris_downtown"),
            KnotTitle("paris_downtown"),
            Tag("downtown tag"),
            Tag("tag ya"),
            Dialog("It was cool downtown."),
            Divert("paris_suburbs"),
            KnotTitle("paris_suburbs"),
            Tag("suburbs tag"),
            Tag("tag too"),
            Dialog("Suburbs were cool too."),
            Operation("health -= 2"),
            Divert("london"),
            KnotTitle("london"),
            Tag("tag1"),
            Tag("tag2"),
            Dialog("Monsieur Phileas Fogg returned home early from the Reform Club, and in a new-fangled steam-carriage, besides!"),
            Dialog("health: \"{health}\""),
            Tag("tag 4"),
            Tag("tag 5"),
            Tag("tag 3"),
            Dialog("\"Passepartout,\" said he. \"We are going around the world!\""),
            StickyChoice("❤"),
            Dialog("I was utterly astonished."),
            Divert("astonished"),
            StickyChoice("🙂"),
            Tag("tag_nod"),
            Tag("tag_nod2"),
            Divert("nod"),
            KnotTitle("astonished"),
            Dialog("\"You are in jest!\" I told him in dignified affront. \"You make mock of me, Monsieur.\""),
            Dialog("\"I am quite serious.\""),
            StickyChoice("🙁"),
            Divert("ending"),
            KnotTitle("nod"),
            Choice("one"),
            Tag("tagend1"),
            Tag("tagend2"),
            Divert("ending"),
            Choice("two"),
            Dialog("I nodded curtly, not believing a word of it."),
            Divert("ending"),
            KnotTitle("ending"),
            Dialog("\"We shall circumnavigate the globe within eighty days.\" He was quite calm as he proposed this wild scheme. \"We leave for Paris on the 8:25. In an hour.\""),
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
            Dialog("LONDON, 1872"),
            Dialog("Residence of Monsieur Phileas Fogg."),
            Divert("downtown"),
            KnotTitle("downtown"),
            Dialog("It was cool downtown."),
            Divert("suburbs"),
            KnotTitle("suburbs"),
            Dialog("Suburbs were cool too."),
            Operation("health -= 2"),
            Divert("london"),
            KnotTitle("london"),
            Dialog("Monsieur Phileas Fogg returned home early from the Reform Club, and in a new-fangled steam-carriage, besides!"),
            Dialog("health: \"{health}\""),
            Dialog("\"Passepartout,\" said he. \"We are going around the world!\""),
            Choice("❤"),
            Dialog("I was utterly astonished."),
            Divert("astonished"),
            Choice("🙂"),
            Divert("nod"),
            KnotTitle("astonished"),
            Dialog("whoa!"),
            Choice("🙁"),
            Divert("ending"),
            KnotTitle("nod"),
            Dialog("I nodded curtly, not believing a word of it."),
            Divert("ending"),
            KnotTitle("ending"),
            Dialog("It's the ending!"),
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
            Dialog("hey"),
            Divert("fallback"),
            KnotTitle("fallback"),
            Dialog("sup"),
            Choice("wut"),
            Divert("fallback"),
            Choice("wutwut"),
            Dialog("text"),
            Divert("fallback"),
            StickyChoice(""),
            Dialog("can I put things here?"),
            Divert("fallback2"),
            KnotTitle("fallback2"),
            Dialog("sup2"),
            Choice("wut2"),
            Divert("fallback2"),
            StickyChoice(""),
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
            Dialog("first class"),
            Divert("missed_train"),
            StitchTitle("second_class"),
            Dialog("second class"),
            Divert("train.missed_train"),
            StitchTitle("missed_train"),
            Dialog("you missed the train"),
            Divert("END"),
        ]
    );
}

#[test]
fn test_lex_choices() {
    assert_eq!(
        lex(&strip_comments(include_str!("../samples/choices.ink"))),
        vec![
            Dialog("What do you want to say?"),
            Choice("Hey"),
            Dialog("I'm Bruce."),
            Divert("END"),
            Choice("[sup]"),
            Dialog("What is up?"),
            Divert("END"),
        ]
    );
}
