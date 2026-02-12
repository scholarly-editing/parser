use nom::IResult;

use crate::{
    config::{ Chars, WordConfig },
    core::{
        errors::TextError,
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
        TextError::WordPrefixedWithUnwantedChars
    );
    if errored_prefixed_word_update.is_some() {
        return errored_prefixed_word_update;
    }

    let errored_suffixed_word_update = create_errored_word_update(
        input,
        word_def,
        word_suffixed_with_unwanted_chars_parser,
        TextError::WordSuffixedWithUnwantedChars
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
    error: TextError
) -> Option<Vec<TokenizerUpdate<'a>>> {
    parser(input, &word_def.chars)
        .ok()
        .map(|(rest, (one, two))| {
            let combined_word = format!("{}{}", one, two);
            let len = combined_word.chars().count();
            vec![
                TokenizerUpdate::AddError(ErrorPayload {
                    text: Box::leak(combined_word.into_boxed_str()),
                    error: error.clone(),
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
                    error: TextError::WordInfixedWithUnwantedChars,
                    len,
                    rest: Some(rest),
                })
            ]
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WordConfig;
    use crate::core::updates::TokenizerUpdate;

    fn arabic_word_config() -> WordConfig {
        WordConfig {
            label: "Arabic".to_string(),
            chars: ((0x600, 0x6FF), vec![]),
            default_state: "sound".to_string(),
            precedence: usize::MAX,
        }
    }

    #[test]
    fn test_simple_arabic_word() {
        let config = arabic_word_config();
        let result = create_add_word_update("كلمة", &config);

        assert!(result.is_some());
        let updates = result.unwrap();
        assert_eq!(updates.len(), 1);

        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "كلمة");
            assert_eq!(payload.word_state, "sound");
            assert_eq!(payload.word_type, "Arabic");
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_arabic_word_followed_by_space() {
        let config = arabic_word_config();
        let result = create_add_word_update("كلمة أخرى", &config);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "كلمة");
            assert_eq!(payload.rest, Some(" أخرى"));
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_word_with_diacritics() {
        let config = arabic_word_config();
        // Word with shaddah and tanwin
        let result = create_add_word_update("المتحابّين", &config);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "المتحابّين");
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_word_with_tanwin_fatha() {
        let config = arabic_word_config();
        let result = create_add_word_update("تساهلًا", &config);

        assert!(result.is_some());
    }

    #[test]
    fn test_non_arabic_returns_none() {
        let config = arabic_word_config();
        let result = create_add_word_update("hello", &config);

        // Latin text should not match Arabic word config
        // The word parser returns None when no valid chars are found at start
        assert!(result.is_none());
    }

    #[test]
    fn test_word_infixed_with_numbers_produces_error() {
        let config = arabic_word_config();
        let result = create_add_word_update("كل123مة", &config);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddError(payload) = &updates[0] {
            assert!(payload.error.to_string().contains("infixed"));
        } else {
            panic!("Expected AddError update for infixed unwanted chars");
        }
    }

    #[test]
    fn test_word_suffixed_with_unwanted_chars_produces_error() {
        let config = arabic_word_config();
        let result = create_add_word_update("كلمة@#", &config);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddError(payload) = &updates[0] {
            assert!(payload.error.to_string().contains("suffixed"));
        } else {
            panic!("Expected AddError update for suffixed unwanted chars");
        }
    }

    #[test]
    fn test_word_prefixed_with_unwanted_chars_produces_error() {
        let config = arabic_word_config();
        let result = create_add_word_update("@كلمة", &config);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddError(payload) = &updates[0] {
            assert!(payload.error.to_string().contains("prefixed"));
        } else {
            panic!("Expected AddError update for prefixed unwanted chars");
        }
    }

    #[test]
    fn test_empty_input_returns_none() {
        let config = arabic_word_config();
        let result = create_add_word_update("", &config);

        assert!(result.is_none());
    }

    #[test]
    fn test_space_only_input_returns_none() {
        let config = arabic_word_config();
        let result = create_add_word_update("   ", &config);

        // Space-only input should not match word parser
        assert!(result.is_none());
    }

    #[test]
    fn test_word_len_is_char_count() {
        let config = arabic_word_config();
        let result = create_add_word_update("كلمة", &config);

        let updates = result.unwrap();
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            // "كلمة" has 4 characters
            assert_eq!(payload.len, 4);
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_additional_chars_in_word() {
        // Test with additional characters allowed
        let config = WordConfig {
            label: "ArabicExtended".to_string(),
            chars: ((0x600, 0x6FF), vec![0x200C, 0x200D]), // Zero-width non-joiner/joiner
            default_state: "sound".to_string(),
            precedence: usize::MAX,
        };

        let result = create_add_word_update("كلمة", &config);
        assert!(result.is_some());
    }
}
