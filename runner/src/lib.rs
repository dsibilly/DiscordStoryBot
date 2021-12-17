#![deny(rust_2018_idioms)]

pub mod ink_lexer;
mod ink_lexer_tests;
pub mod ink_parser;
mod ink_parser_tests;
pub mod ink_runner;
mod ink_runner_tests;

// TODO: put ron and serde behind a feature flag
