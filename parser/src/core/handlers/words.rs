use nom::IResult;

use crate::{
    config::{ Chars, WordConfig },
    core::{
        parsers::{
            word_infixed_with_unwanted_chars_parser,
            word_parser,
            word_prefixed_with_unwanted_chars_parser,
            word_suffixed_with_unwanted_chars_parser,
        },
        updates::{ ErrorPayload, TokenizerUpdate, WordPayload },
    },
};

pub fn create_add_word_update<'a>(
    input: &'a str,
    word_def: &'a WordConfig
) -> Option<Vec<TokenizerUpdate<'a>>> {
    let errored_infixed_word_update = create_errored_infixed_word_update(input, word_def);
    if errored_infixed_word_update.is_some() {
        return errored_infixed_word_update;
    }

    let errored_prefixed_word_update = create_errored_word_update(
        input,
        word_def,
        word_prefixed_with_unwanted_chars_parser,
        "Word prefixed with unwanted characters"
    );
    if errored_prefixed_word_update.is_some() {
        return errored_prefixed_word_update;
    }

    let errored_suffixed_word_update = create_errored_word_update(
        input,
        word_def,
        word_suffixed_with_unwanted_chars_parser,
        "Word suffixed with unwanted characters"
    );
    if errored_suffixed_word_update.is_some() {
        return errored_suffixed_word_update;
    }

    let word_update = word_parser(input, &word_def.chars)
        .ok()
        .map(|(rest, word)| {
            vec![
                TokenizerUpdate::AddWord(WordPayload {
                    word,
                    word_state: &word_def.default_state,
                    word_type: &word_def.label,
                    rest: Some(rest),
                    pre_len: 0,
                    post_len: 0,
                    len: word.chars().count(),
                })
            ]
        });

    word_update
}

fn create_errored_word_update<'a>(
    input: &'a str,
    word_def: &'a WordConfig,
    parser: fn(&'a str, &Chars) -> IResult<&'a str, (&'a str, &'a str)>,
    error_message: &'static str
) -> Option<Vec<TokenizerUpdate<'a>>> {
    parser(input, &word_def.chars)
        .ok()
        .map(|(rest, (one, two))| {
            let combined_word = format!("{}{}", one, two);
            let len = combined_word.chars().count();
            vec![
                TokenizerUpdate::AddError(ErrorPayload {
                    text: Box::leak(combined_word.into_boxed_str()),
                    message: error_message.into(),
                    len,
                    rest: Some(rest),
                })
            ]
        })
}

fn create_errored_infixed_word_update<'a>(
    input: &'a str,
    word_def: &'a WordConfig
) -> Option<Vec<TokenizerUpdate<'a>>> {
    word_infixed_with_unwanted_chars_parser(input, &word_def.chars)
        .ok()
        .map(|(rest, (one, two, three))| {
            let combined_word = format!("{}{}{}", one, two, three);
            let len = combined_word.chars().count();
            vec![
                TokenizerUpdate::AddError(ErrorPayload {
                    text: Box::leak(combined_word.into_boxed_str()),
                    message: "Word infixed with unwanted characters".into(),
                    len,
                    rest: Some(rest),
                })
            ]
        })
}
