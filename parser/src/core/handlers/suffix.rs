use crate::{
    config::{ SuffixConfig, WordConfig },
    core::{
        parsers::{ suffixed_word_parser, wrong_suffix_use_parser },
        updates::{ ErrorPayload, TokenizerUpdate, WordPayload },
    },
};

pub fn create_add_suffixed_word_update<'a>(
    input: &'a str,
    word_def: &'a WordConfig,
    suffix_def: &'a SuffixConfig
) -> Option<Vec<TokenizerUpdate<'a>>> {
    let wrong_suffix_use_update = wrong_suffix_use_parser(input, &suffix_def.symbol)
        .ok()
        .map(|(rest, suffix)| {
            let len = suffix.chars().count();
            vec![
                TokenizerUpdate::AddError(ErrorPayload {
                    text: suffix,
                    message: "cannot add suffix here".into(),
                    len,
                    rest: Some(rest),
                })
            ]
        });

    if wrong_suffix_use_update.is_some() {
        return wrong_suffix_use_update;
    }

    let suffixed_word_update = suffixed_word_parser(input, &suffix_def.symbol, &word_def.chars)
        .ok()
        .map(|(rest, word)| {
            vec![
                TokenizerUpdate::AddWord(WordPayload {
                    word,
                    word_state: suffix_def.label.as_str(),
                    word_type: &word_def.label,
                    rest: Some(rest),
                    pre_len: 0,
                    post_len: suffix_def.symbol.chars().count(),
                    len: word.chars().count(),
                })
            ]
        });

    suffixed_word_update
}
