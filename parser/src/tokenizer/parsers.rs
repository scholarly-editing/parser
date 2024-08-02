use nom::{
    bytes::complete::{ tag, take_while1 },
    character::complete::space1,
    sequence::terminated,
};
use std::rc::Rc;

use crate::config::ParserConfig;

use super::{
    state::{ Mode, Token, TokenizerAction, TokenizerState },
    util::{
        enclosed_word_parser,
        line_break,
        page_break,
        parse_sqeuence,
        prefixed_word_parser,
        spaces,
        suffixed_word_parser,
        word_parser,
    },
};

pub type ParserFn = Box<dyn Fn(&TokenizerState) -> Option<TokenizerAction>>;

fn create_word_parser(is_valid_char: Rc<dyn Fn(char) -> bool>) -> ParserFn {
    Box::new(move |state: &TokenizerState| {
        word_parser(&state.remaining, &is_valid_char)
            .ok()
            .map(|(rest, word)| {
                TokenizerAction::AddWord(
                    word.to_string(),
                    "sound".to_string(),
                    rest.to_string(),
                    word.len()
                )
            })
    })
}

fn create_prefixed_word_parser(
    is_valid_char: Rc<dyn Fn(char) -> bool>,
    prefix: String,
    label: String
) -> ParserFn {
    Box::new(move |state: &TokenizerState| {
        prefixed_word_parser(&state.remaining, &prefix, &is_valid_char)
            .ok()
            .and_then(|(rest, word)| {
                if state.can_add_token() {
                    Some(
                        TokenizerAction::AddWord(
                            word.to_string(),
                            label.clone(),
                            rest.to_string(),
                            word.len() + prefix.len()
                        )
                    )
                } else {
                    None
                }
            })
    })
}

fn create_bracketed_words_parser(
    is_valid_char: Rc<dyn Fn(char) -> bool>,
    open: String,
    close: String,
    label: String
) -> ParserFn {
    Box::new(move |state: &TokenizerState| {
        prefixed_word_parser(&state.remaining, &open, &is_valid_char)
            .ok()
            .and_then(|(rest, word)| {
                if bracket_is_closed(rest, &close, &is_valid_char) {
                    create_bracketed_token(
                        state,
                        word,
                        format!("{}_open", &label),
                        rest,
                        open.len()
                    )
                } else {
                    None
                }
            })
            .or_else(|| {
                suffixed_word_parser(&state.remaining, &close, &is_valid_char)
                    .ok()
                    .and_then(|(rest, word)| {
                        if bracket_was_open(state, &label) {
                            create_bracketed_token(
                                state,
                                word,
                                format!("{}_close", &label),
                                rest,
                                close.len()
                            )
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| {
                enclosed_word_parser(&state.remaining, &open, &close, &is_valid_char)
                    .ok()
                    .and_then(|(rest, word)|
                        create_bracketed_token(
                            state,
                            word,
                            label.clone(),
                            rest,
                            open.len() + close.len()
                        )
                    )
            })
    })
}

fn create_sequence_parser(seq: String, seq_type: String) -> ParserFn {
    Box::new(move |state: &TokenizerState| {
        parse_sqeuence(&seq)(&state.remaining)
            .ok()
            .and_then(|(rest, _)| {
                if state.can_add_token() {
                    Some(
                        TokenizerAction::AddTag(
                            seq.to_string(),
                            seq_type.to_string(),
                            rest.to_string(),
                            seq.len()
                        )
                    )
                } else {
                    None
                }
            })
    })
}

fn is_char_in_range(code: u32, range: &(u32, u32)) -> bool {
    code >= range.0 && code <= range.1
}

pub fn create_parsers(config: Rc<ParserConfig>, mode: &Mode) -> Vec<ParserFn> {
    let config_ref = config.clone();
    let is_valid_char = Rc::new(move |c: char| {
        let range = (
            *config_ref.main.char_range.first().unwrap(),
            *config_ref.main.char_range.last().unwrap(),
        );
        let code = c as u32;
        is_char_in_range(code, &range) || config_ref.main.additional_chars.contains(&code)
    });
    let word_parser = create_word_parser(is_valid_char.clone());

    let mut prefix_parsers: Vec<ParserFn> = vec![];

    for prefix_def in &config.prefix {
        prefix_parsers.push(
            create_prefixed_word_parser(
                is_valid_char.clone(),
                prefix_def.symbol.clone(),
                prefix_def.label.clone()
            )
        );
    }

    let mut bracketed_parsers: Vec<ParserFn> = vec![];

    for bracket_def in &config.brackets {
        bracketed_parsers.push(
            create_bracketed_words_parser(
                is_valid_char.clone(),
                bracket_def.open.clone(),
                bracket_def.close.clone(),
                bracket_def.label.clone()
            )
        );
    }

    let mut sequence_parsers: Vec<ParserFn> = vec![];

    for seq_def in &config.sequence {
        sequence_parsers.push(
            create_sequence_parser(seq_def.symbol.clone(), seq_def.label.clone())
        );
    }

    let mut parsers: Vec<ParserFn> = vec![Box::new(parse_spaces)];

    match mode {
        Mode::SinglePage => {
            parsers.push(Box::new(parse_line_break));
            parsers.extend(prefix_parsers);
            parsers.extend(bracketed_parsers);
            parsers.extend(sequence_parsers);
            parsers.push(word_parser);
            parsers
        }
        Mode::Passage => {
            parsers.push(Box::new(parse_passage_line_break));
            parsers.extend(prefix_parsers);
            parsers.extend(bracketed_parsers);
            parsers.extend(sequence_parsers);
            parsers.push(word_parser);
            parsers
        }
        Mode::MultiplePages => unimplemented!(),
    }
}

pub fn parse_line_break(state: &TokenizerState) -> Option<TokenizerAction> {
    line_break(&state.remaining)
        .ok()
        .map(|(rest, chars)| { TokenizerAction::AddLineBreak(rest.to_string(), chars.len()) })
}

pub fn parse_passage_line_break(state: &TokenizerState) -> Option<TokenizerAction> {
    line_break(&state.remaining)
        .ok()
        .map(|(rest, chars)| {
            TokenizerAction::AddPassageLineBreak(rest.to_string(), chars.len())
        })
}

pub fn parse_page_break(state: &mut TokenizerState) -> Option<TokenizerState> {
    page_break(&state.remaining)
        .ok()
        .map(|(rest, repr)| {
            let mut new_state = state.clone();
            new_state.update_remaining(rest);
            new_state.update_position(repr.len());
            new_state.add_page_break(repr);
            new_state
        })
}

pub fn parse_spaces(state: &TokenizerState) -> Option<TokenizerAction> {
    spaces(&state.remaining)
        .ok()
        .map(|(rest, repr)| { TokenizerAction::AddSpace(rest.to_string(), repr.len()) })
}

fn bracket_is_closed<'a>(
    input: &'a str,
    close: &'a str,
    is_valid_char: &Rc<dyn Fn(char) -> bool>
) -> bool {
    let result = terminated::<&'a str, &'a str, &'a str, nom::error::Error<&'a str>, _, _>(
        take_while1(
            |c: char| c != close.chars().next().unwrap() && (c.is_whitespace() || is_valid_char(c))
        ),
        terminated::<&'a str, &'a str, &'a str, nom::error::Error<&'a str>, _, _>(
            tag(close),
            space1
        )
    )(input);
    result.is_ok()
}

fn bracket_was_open<'a>(state: &TokenizerState, bracket_type: &'a str) -> bool {
    let mut open_found = false;
    let openning_bracket_type = format!("{}_open", bracket_type);
    for token in state.tokens.iter().rev() {
        if let Token::Word(_, token_type, _, _, _, _) = token {
            if token_type.as_str() == &openning_bracket_type {
                open_found = true;
                break;
            } else if token_type.as_str() != "sound" {
                break;
            }
        }
    }

    open_found
}

pub fn create_bracketed_token(
    state: &TokenizerState,
    word: &str,
    token_type: String,
    rest: &str,
    additional_len: usize
) -> Option<TokenizerAction> {
    if state.can_add_token() {
        let mut new_state = state.clone();
        new_state.add_word(word.to_string(), token_type.to_string());
        new_state.update_remaining(rest);
        new_state.update_position(word.len() + additional_len);
        Some(
            TokenizerAction::AddWord(
                word.to_string(),
                token_type.to_string(),
                rest.to_string(),
                word.len() + additional_len
            )
        )
    } else {
        None
    }
}
