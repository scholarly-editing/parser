use std::marker::PhantomData;

use nom::{
    bytes::complete::{ tag, take_while1 },
    character::complete::space1,
    sequence::terminated,
};

use crate::config::ParserConfig;

use super::{
    state::{ Mode, Token, TokenizerAction, TokenizerState },
    util::{
        enclosed_word_parser,
        is_valid_char,
        line_break,
        parse_sqeuence,
        prefixed_word_parser,
        spaces,
        suffixed_word_parser,
        word_parser,
        Chars,
    },
};

pub type ParserFn<'a> = Box<dyn (Fn(&TokenizerState<'a>) -> Option<TokenizerAction<'a>>) + 'a>;

fn word_parser_fn<'a>(state: &TokenizerState<'a>, chars: &Chars) -> Option<TokenizerAction<'a>> {
    word_parser(state.remaining, chars)
        .ok()
        .and_then(|(rest, word)| {
            if state.can_add_token() {
                Some(TokenizerAction::AddWord(word, "sound", rest, word.len()))
            } else {
                None
            }
        })
}

fn prefixed_word_parser_fn<'a>(
    state: &TokenizerState<'a>,
    chars: &Chars,
    prefix: &'a str,
    label: &'a str
) -> Option<TokenizerAction<'a>> {
    prefixed_word_parser(state.remaining, prefix, chars)
        .ok()
        .and_then(|(rest, word)| {
            if state.can_add_token() {
                Some(TokenizerAction::AddWord(word, label, rest, word.len() + prefix.len()))
            } else {
                None
            }
        })
}

fn run_bracketed_words_parser<'a>(
    state: &TokenizerState<'a>,
    chars: &Chars,
    open: &'a str,
    close: &'a str,
    label: &'a str,
    open_label: &'a str,
    close_label: &'a str
) -> Option<TokenizerAction<'a>> {
    prefixed_word_parser(state.remaining, open, chars)
        .ok()
        .and_then(|(rest, word)| {
            if bracket_is_closed(rest, close, chars) {
                create_bracketed_token(state, word, open_label, rest, open.len())
            } else {
                None
            }
        })
        .or_else(|| {
            suffixed_word_parser(state.remaining, close, chars)
                .ok()
                .and_then(|(rest, word)| {
                    if bracket_was_open(state, label) {
                        create_bracketed_token(state, word, close_label, rest, close.len())
                    } else {
                        None
                    }
                })
        })
        .or_else(|| {
            enclosed_word_parser(state.remaining, open, close, chars)
                .ok()
                .and_then(|(rest, word)|
                    create_bracketed_token(state, word, label, rest, open.len() + close.len())
                )
        })
}

fn run_sequence_parser<'a>(
    state: &TokenizerState<'a>,
    seq: &'a str,
    seq_type: &'a str
) -> Option<TokenizerAction<'a>> {
    parse_sqeuence(seq)(state.remaining)
        .ok()
        .and_then(|(rest, _)| {
            if state.can_add_token() {
                Some(TokenizerAction::AddTag(seq, seq_type, rest, seq.len()))
            } else {
                None
            }
        })
}
pub struct Parser<'b, 'c> {
    config: &'b ParserConfig,
    mode: &'c Mode,
    chars: Chars,
}

impl<'b, 'c> Parser<'b, 'c> {
    pub fn new(config: &'b ParserConfig, mode: &'c Mode) -> Self {
        let range = &config.main.char_range;
        let first = *range.first().unwrap();
        let last = *range.last().unwrap();
        let chars = ((first, last), config.main.additional_chars.clone());

        Parser { config, mode, chars }
    }

    pub fn run<'a>(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        match self.mode {
            Mode::SinglePage => self.run_single_page(state),
            Mode::Passage => None,
            Mode::MultiplePages => unimplemented!(),
        }
    }

    fn run_single_page<'a>(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        parse_spaces(state)
            .or_else(|| parse_line_break(state))
            .or_else(|| self.parse_prefixed_word(state))
            .or_else(|| self.parse_bracketed_words(state))
            .or_else(|| self.parse_sequence(state))
            .or_else(|| word_parser_fn(state, &self.chars))
    }

    fn parse_prefixed_word<'a>(
        &'a self,
        state: &TokenizerState<'a>
    ) -> Option<TokenizerAction<'a>> {
        self.config.prefix
            .iter()
            .find_map(|prefix_def| {
                prefixed_word_parser_fn(state, &self.chars, &prefix_def.symbol, &prefix_def.label)
            })
    }

    fn parse_bracketed_words<'a>(
        &'a self,
        state: &TokenizerState<'a>
    ) -> Option<TokenizerAction<'a>> {
        self.config.brackets
            .iter()
            .find_map(|bracket_def| {
                run_bracketed_words_parser(
                    state,
                    &self.chars,
                    &bracket_def.open,
                    &bracket_def.close,
                    &bracket_def.label,
                    &bracket_def.open_label,
                    &bracket_def.close_label
                )
            })
    }

    fn parse_sequence<'a>(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        self.config.sequence
            .iter()
            .find_map(|seq_def| { run_sequence_parser(state, &seq_def.symbol, &seq_def.label) })
    }
}

pub fn parse_line_break<'a>(state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
    line_break(state.remaining)
        .ok()
        .map(|(rest, chars)| TokenizerAction::AddLineBreak(rest, chars.len()))
}

pub fn parse_passage_line_break<'a>(state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
    line_break(state.remaining)
        .ok()
        .map(|(rest, chars)| TokenizerAction::AddPassageLineBreak(rest, chars.len()))
}

pub fn parse_spaces<'a>(state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
    spaces(state.remaining)
        .ok()
        .map(|(rest, repr)| TokenizerAction::AddSpace(rest, repr.len()))
}

fn bracket_is_closed<'a>(input: &'a str, close: &str, chars: &Chars) -> bool {
    let result = terminated::<&'a str, &'a str, &'a str, nom::error::Error<&'a str>, _, _>(
        take_while1(
            |c: char|
                c != close.chars().next().unwrap() && (c.is_whitespace() || is_valid_char(c, chars))
        ),
        terminated::<&'a str, &'a str, &'a str, nom::error::Error<&'a str>, _, _>(
            tag(close),
            space1
        )
    )(input);
    result.is_ok()
}

fn bracket_was_open<'a>(state: &TokenizerState<'a>, bracket_type: &str) -> bool {
    let mut open_found = false;
    let openning_bracket_type = format!("{}_open", bracket_type);
    for token in state.tokens.iter().rev() {
        if let Token::Word(_, token_type, _, _, _, _) = token {
            if *token_type == openning_bracket_type {
                open_found = true;
                break;
            } else if *token_type != "sound" {
                break;
            }
        }
    }

    open_found
}

pub fn create_bracketed_token<'a>(
    state: &TokenizerState<'a>,
    word: &'a str,
    token_type: &'a str,
    rest: &'a str,
    additional_len: usize
) -> Option<TokenizerAction<'a>> {
    if state.can_add_token() {
        Some(TokenizerAction::AddWord(word, token_type, rest, word.len() + additional_len))
    } else {
        None
    }
}
