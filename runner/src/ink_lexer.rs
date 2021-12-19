#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InkToken<'a> {
    Dialog(&'a str),
    KnotTitle(&'a str),
    StitchTitle(&'a str),
    Choice(&'a str),
    StickyChoice(&'a str),
    Divert(&'a str),
    VariableDeclaration(&'a str),
    Operation(&'a str),
    Tag(&'a str),
}

pub fn strip_comments(text: &str) -> String {
    strip_multi_line_comment(&strip_single_line_comment(&strip_backslash_r(text)))
}

fn strip_backslash_r(text: &str) -> String {
    text.chars().filter(|&c| c != '\r').collect()
}

fn strip_single_line_comment(text: &str) -> String {
    let mut remaining_text = text;
    let mut result = "".to_string();
    while let Some(comment_index) = remaining_text.find("//") {
        result += remaining_text[..comment_index].trim_end();
        remaining_text = &remaining_text[comment_index + 2..];

        match remaining_text.find('\n') {
            Some(end_of_line_index) => {
                remaining_text = &remaining_text[end_of_line_index..];
            }
            None => {
                remaining_text = "";
            }
        }
    }
    result += remaining_text;
    result
}

fn strip_multi_line_comment(text: &str) -> String {
    let mut remaining_text = text;
    let mut result = "".to_string();
    while let Some(comment_index) = remaining_text.find("/*") {
        result += remaining_text[..comment_index].trim_end();
        remaining_text = &remaining_text[comment_index + 3..]; // 3 because we need at least one character between "/*" and "*/"
        let end_index = remaining_text.find("*/").unwrap(); // TODO: make fallible
        remaining_text = remaining_text[end_index + 2..].trim_start();
    }
    result += remaining_text;
    result
}

pub fn lex(text: &str) -> Vec<InkToken<'_>> {
    use InkToken::*;

    text.lines()
        .filter_map(|line| {
            let line = line.trim();

            if line.is_empty() {
                None
            } else if line.starts_with('=') {
                let index = line.find(|c| c != '=').unwrap(); // TODO: escalate this error
                let right_index = line.rfind(|c| c != '=').unwrap(); // will always be valid if the previous one is valid
                let title = line[index..right_index + 1].trim();
                match index {
                    1 => Some(vec![StitchTitle(title)]),
                    _ => Some(vec![KnotTitle(title)]),
                }
            } else if let Some(stripped) = line.strip_prefix("->") {
                Some(vec![Divert(stripped.trim_start())])
            } else if line.starts_with('*') {
                lex_choice(line, false)
            } else if line.starts_with('+') {
                lex_choice(line, true)
            } else if line.starts_with('#') {
                let sections = line.split('#');
                Some(
                    sections
                        .filter_map(|s| {
                            if s.is_empty() {
                                None
                            } else {
                                Some(Tag(s.trim()))
                            }
                        })
                        .collect(),
                )
            } else if let Some(stripped) = line.strip_prefix("VAR ") {
                Some(vec![VariableDeclaration(stripped.trim_start())])
            } else if let Some(stripped) = line.strip_prefix('~') {
                Some(vec![Operation(stripped.trim_start())])
            } else {
                let mut tags = vec![];
                let mut line = line;
                let mut divert = None;

                if let Some(index) = line.find('#') {
                    tags = line[index + 1..]
                        .split('#')
                        .map(|s| Tag(s.trim()))
                        .collect();
                    line = line[0..index].trim();
                }

                if let Some(index) = line.find("->") {
                    divert = Some(line[index + 2..].trim());
                    line = line[0..index].trim();
                }

                let mut result = tags;
                result.push(Dialog(line.trim()));
                if let Some(divert) = divert {
                    result.push(Divert(divert));
                }

                Some(result)
            }
        })
        .flatten()
        .collect()
}

fn lex_choice(line: &str, sticky: bool) -> Option<Vec<InkToken<'_>>> {
    use InkToken::*;

    let mut tags = vec![];
    let mut line = line;
    let mut divert = None;

    if let Some(index) = line.find('#') {
        tags = line[index + 1..]
            .split('#')
            .map(|s| Tag(s.trim()))
            .collect();
        line = line[0..index].trim();
    }

    if let Some(index) = line.find("->") {
        divert = Some(line[index + 2..].trim());
        line = line[0..index].trim();
    }

    let choice_text = line[1..].trim();

    let mut result = vec![if sticky || choice_text.is_empty() {
        StickyChoice(choice_text)
    } else {
        Choice(choice_text)
    }];

    result.append(&mut tags);

    if let Some(divert) = divert {
        if !divert.is_empty() {
            result.push(Divert(divert));
        }
    }

    Some(result)
}

// TODO: after lexing, unescape things that need it
