use nom::{
    bytes::complete::{ tag, take_while1 },
    character::complete::space1,
    sequence::terminated,
};

use crate::config::{ Chars, ParserConfig, WordConfig };

use super::{
    state::{
        LineBreakPayload,
        Mode,
        PassageLineBreakPayload,
        SpacePayload,
        TagPayload,
        Token,
        TokenizerAction,
        TokenizerState,
        WordPayload,
        WordToken,
    },
    util::{
        enclosed_word_parser,
        is_valid_char,
        line_break,
        parse_sqeuence,
        prefixed_word_parser,
        spaces,
        suffixed_word_parser,
        word_parser,
    },
};

pub type ParserFn<'a> = Box<dyn (Fn(&TokenizerState<'a>) -> Option<TokenizerAction<'a>>) + 'a>;

fn word_parser_fn<'a>(
    state: &TokenizerState<'a>,
    word_def: &'a WordConfig
) -> Option<TokenizerAction<'a>> {
    word_parser(state.remaining, &word_def.chars)
        .ok()
        .and_then(|(rest, word)| {
            if state.can_add_token() {
                Some(
                    TokenizerAction::AddWord(WordPayload {
                        word,
                        word_state: "sound",
                        word_type: &word_def.label,
                        rest,
                        position: word.len(),
                    })
                )
            } else {
                None
            }
        })
}

fn prefixed_word_parser_fn<'a>(
    state: &TokenizerState<'a>,
    word_def: &'a WordConfig,
    prefix: &'a str,
    label: &'a str
) -> Option<TokenizerAction<'a>> {
    prefixed_word_parser(state.remaining, prefix, &word_def.chars)
        .ok()
        .and_then(|(rest, word)| {
            if state.can_add_token() {
                Some(
                    TokenizerAction::AddWord(WordPayload {
                        word,
                        word_state: label,
                        word_type: &word_def.label,
                        rest,
                        position: word.len() + prefix.len(),
                    })
                )
            } else {
                None
            }
        })
}

fn run_bracketed_words_parser<'a>(
    state: &TokenizerState<'a>,
    word_def: &'a WordConfig,
    open: &'a str,
    close: &'a str,
    label: &'a str,
    open_label: &'a str,
    close_label: &'a str
) -> Option<TokenizerAction<'a>> {
    prefixed_word_parser(state.remaining, open, &word_def.chars)
        .ok()
        .and_then(|(rest, word)| {
            if bracket_is_closed(rest, close, &word_def.chars) {
                create_bracketed_token(state, word, &word_def.label, open_label, rest, open.len())
            } else {
                None
            }
        })
        .or_else(|| {
            suffixed_word_parser(state.remaining, close, &word_def.chars)
                .ok()
                .and_then(|(rest, word)| {
                    if bracket_was_open(state, label) {
                        create_bracketed_token(
                            state,
                            word,
                            &word_def.label,
                            close_label,
                            rest,
                            close.len()
                        )
                    } else {
                        None
                    }
                })
        })
        .or_else(|| {
            enclosed_word_parser(state.remaining, open, close, &word_def.chars)
                .ok()
                .and_then(|(rest, word)|
                    create_bracketed_token(
                        state,
                        word,
                        &word_def.label,
                        label,
                        rest,
                        open.len() + close.len()
                    )
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
                Some(
                    TokenizerAction::AddTag(TagPayload {
                        tag: seq,
                        label: seq_type,
                        rest,
                        position: seq.len(),
                    })
                )
            } else {
                None
            }
        })
}
pub struct Parser<'a> {
    config: &'a ParserConfig,
    mode: &'a Mode,
}

impl<'a> Parser<'a> {
    pub fn new(config: &'a ParserConfig, mode: &'a Mode) -> Self {
        Parser { config, mode }
    }

    pub fn run(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        match self.mode {
            Mode::SinglePage => self.run_single_page(state),
            Mode::Passage => None,
            Mode::MultiplePages => unimplemented!(),
        }
    }

    fn run_single_page(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        parse_spaces(state)
            .or_else(|| parse_line_break(state))
            .or_else(|| self.parse_prefixed_word(state))
            .or_else(|| self.parse_bracketed_words(state))
            .or_else(|| self.parse_sequence(state))
            .or_else(|| self.parse_words(state))
    }

    fn parse_words(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        self.config.word.iter().find_map(|word_def| { word_parser_fn(state, &word_def) })
    }

    fn parse_prefixed_word(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        self.config.prefix
            .iter()
            .find_map(|prefix_def| {
                self.config.word
                    .iter()
                    .find_map(|word_def| {
                        prefixed_word_parser_fn(
                            state,
                            &word_def,
                            &prefix_def.symbol,
                            &prefix_def.label
                        )
                    })
            })
    }

    fn parse_bracketed_words(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        self.config.brackets
            .iter()
            .find_map(|bracket_def| {
                self.config.word
                    .iter()
                    .find_map(|word_def| {
                        run_bracketed_words_parser(
                            state,
                            &word_def,
                            &bracket_def.open,
                            &bracket_def.close,
                            &bracket_def.label,
                            &bracket_def.open_label,
                            &bracket_def.close_label
                        )
                    })
            })
    }

    fn parse_sequence(&'a self, state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
        self.config.sequence
            .iter()
            .find_map(|seq_def| { run_sequence_parser(state, &seq_def.symbol, &seq_def.label) })
    }
}

pub fn parse_line_break<'a>(state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
    line_break(state.remaining)
        .ok()
        .map(|(rest, chars)|
            TokenizerAction::AddLineBreak(LineBreakPayload { rest, position: chars.len() })
        )
}

pub fn parse_passage_line_break<'a>(state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
    line_break(state.remaining)
        .ok()
        .map(|(rest, chars)|
            TokenizerAction::AddPassageLineBreak(PassageLineBreakPayload {
                rest,
                position: chars.len(),
            })
        )
}

pub fn parse_spaces<'a>(state: &TokenizerState<'a>) -> Option<TokenizerAction<'a>> {
    spaces(state.remaining)
        .ok()
        .map(|(rest, repr)| TokenizerAction::AddSpace(SpacePayload { rest, position: repr.len() }))
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
        if let Token::Word(WordToken { state: token_type, .. }) = token {
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
    word_type: &'a str,
    word_state: &'a str,
    rest: &'a str,
    additional_len: usize
) -> Option<TokenizerAction<'a>> {
    if state.can_add_token() {
        Some(
            TokenizerAction::AddWord(WordPayload {
                word,
                word_state,
                word_type,
                rest,
                position: word.len() + additional_len,
            })
        )
    } else {
        None
    }
}
