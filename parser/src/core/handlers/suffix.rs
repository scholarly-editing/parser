use crate::{
    config::{ SuffixConfig, WordConfig },
    core::{
        errors::TextError,
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
                    error: TextError::InvalidSuffixPosition,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SuffixConfig, WordConfig};
    use crate::core::updates::TokenizerUpdate;

    fn arabic_word_config() -> WordConfig {
        WordConfig {
            label: "Arabic".to_string(),
            chars: ((0x600, 0x6FF), vec![]),
            default_state: "sound".to_string(),
            precedence: usize::MAX,
        }
    }

    fn middle_arabic_suffix() -> SuffixConfig {
        SuffixConfig {
            symbol: "~".to_string(),
            label: "middle-arabic".to_string(),
            precedence: usize::MAX,
        }
    }

    #[test]
    fn test_suffixed_word() {
        let word_config = arabic_word_config();
        let suffix_config = middle_arabic_suffix();
        
        let result = create_add_suffixed_word_update("كلمة~", &word_config, &suffix_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        assert_eq!(updates.len(), 1);
        
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "كلمة");
            assert_eq!(payload.word_state, "middle-arabic");
            assert_eq!(payload.post_len, 1); // "~" is 1 char
            assert_eq!(payload.pre_len, 0);
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_suffixed_word_followed_by_space() {
        let word_config = arabic_word_config();
        let suffix_config = middle_arabic_suffix();
        
        let result = create_add_suffixed_word_update("كلمة~ أخرى", &word_config, &suffix_config);
        
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
    fn test_wrong_suffix_use_at_start() {
        let word_config = arabic_word_config();
        let suffix_config = middle_arabic_suffix();
        
        // Suffix at start of input without preceding word should be error
        let result = create_add_suffixed_word_update("~", &word_config, &suffix_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddError(payload) = &updates[0] {
            assert!(payload.error.to_string().contains("suffix"));
        } else {
            panic!("Expected AddError update for wrong suffix use");
        }
    }

    #[test]
    fn test_suffix_not_present_returns_none() {
        let word_config = arabic_word_config();
        let suffix_config = middle_arabic_suffix();
        
        // Word without suffix
        let result = create_add_suffixed_word_update("كلمة", &word_config, &suffix_config);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_suffix_with_different_word_type() {
        let word_config = arabic_word_config();
        let suffix_config = middle_arabic_suffix();
        
        // Non-Arabic word with suffix
        let result = create_add_suffixed_word_update("hello~", &word_config, &suffix_config);
        
        // Should return None because "hello" doesn't match Arabic word config
        assert!(result.is_none());
    }

    #[test]
    fn test_suffix_post_len_matches_symbol_chars() {
        let word_config = arabic_word_config();
        let suffix_config = middle_arabic_suffix();
        
        let result = create_add_suffixed_word_update("كلمة~", &word_config, &suffix_config);
        
        let updates = result.unwrap();
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.post_len, 1); // "~" is 1 char
            assert_eq!(payload.pre_len, 0);
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_multi_char_suffix() {
        let word_config = arabic_word_config();
        let suffix_config = SuffixConfig {
            symbol: "~~".to_string(),
            label: "double-suffix".to_string(),
            precedence: usize::MAX,
        };
        
        let result = create_add_suffixed_word_update("كلمة~~", &word_config, &suffix_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "كلمة");
            assert_eq!(payload.post_len, 2); // "~~" is 2 chars
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_word_len_excludes_suffix() {
        let word_config = arabic_word_config();
        let suffix_config = middle_arabic_suffix();
        
        let result = create_add_suffixed_word_update("كلمة~", &word_config, &suffix_config);
        
        let updates = result.unwrap();
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            // "كلمة" has 4 characters
            assert_eq!(payload.len, 4);
        } else {
            panic!("Expected AddWord update");
        }
    }
}
