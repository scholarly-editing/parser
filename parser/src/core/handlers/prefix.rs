use crate::{
    config::{ PrefixConfig, WordConfig },
    core::{
        parsers::{ prefixed_word_parser, wrong_prefix_use_parser },
        updates::{ ErrorPayload, TokenizerUpdate, WordPayload },
    },
};

pub fn create_add_prefixed_word_update<'a>(
    input: &'a str,
    word_def: &'a WordConfig,
    prefix_def: &'a PrefixConfig
) -> Option<Vec<TokenizerUpdate<'a>>> {
    let wrong_prefix_use_update = wrong_prefix_use_parser(
        input,
        &prefix_def.symbol,
        &word_def.chars
    )
        .ok()
        .map(|(rest, (prefix, space, word))| {
            let combined_word = format!("{}{}{}", prefix, space, word);
            let len = combined_word.chars().count();
            vec![
                TokenizerUpdate::AddError(ErrorPayload {
                    text: Box::leak(combined_word.into_boxed_str()),
                    message: "Wrong prefix use: prefix not attched to a word".into(),
                    len,
                    rest: Some(rest),
                })
            ]
        });

    if wrong_prefix_use_update.is_some() {
        return wrong_prefix_use_update;
    }

    let prefixed_word_update = prefixed_word_parser(input, &prefix_def.symbol, &word_def.chars)
        .ok()
        .map(|(rest, word)| {
            vec![
                TokenizerUpdate::AddWord(WordPayload {
                    word,
                    word_state: prefix_def.label.as_str(),
                    word_type: &word_def.label,
                    rest: Some(rest),
                    pre_len: prefix_def.symbol.chars().count(),
                    post_len: 0,
                    len: word.chars().count(),
                })
            ]
        });

    prefixed_word_update
}
