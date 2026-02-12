use crate::{
    config::{ PrefixConfig, WordConfig },
    core::{
        errors::TextError,
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
                    error: TextError::InvalidPrefixUse,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PrefixConfig, WordConfig};
    use crate::core::updates::TokenizerUpdate;

    fn arabic_word_config() -> WordConfig {
        WordConfig {
            label: "Arabic".to_string(),
            chars: ((0x600, 0x6FF), vec![]),
            default_state: "sound".to_string(),
            precedence: usize::MAX,
        }
    }

    fn emendation_prefix() -> PrefixConfig {
        PrefixConfig {
            symbol: "*".to_string(),
            label: "emendation".to_string(),
            precedence: usize::MAX,
        }
    }

    fn corrupt_prefix() -> PrefixConfig {
        PrefixConfig {
            symbol: "†".to_string(),
            label: "corrupt".to_string(),
            precedence: usize::MAX,
        }
    }

    fn unintelligible_arabic_prefix() -> PrefixConfig {
        PrefixConfig {
            symbol: "؟".to_string(),
            label: "unintelligible".to_string(),
            precedence: usize::MAX,
        }
    }

    #[test]
    fn test_emendation_prefix_attached_to_word() {
        let word_config = arabic_word_config();
        let prefix_config = emendation_prefix();
        
        let result = create_add_prefixed_word_update("*لبيدبا", &word_config, &prefix_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        assert_eq!(updates.len(), 1);
        
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "لبيدبا");
            assert_eq!(payload.word_state, "emendation");
            assert_eq!(payload.pre_len, 1); // "*" is 1 char
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_corrupt_prefix_dagger() {
        let word_config = arabic_word_config();
        let prefix_config = corrupt_prefix();
        
        let result = create_add_prefixed_word_update("†فاسد", &word_config, &prefix_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "فاسد");
            assert_eq!(payload.word_state, "corrupt");
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_arabic_question_mark_prefix() {
        let word_config = arabic_word_config();
        let prefix_config = unintelligible_arabic_prefix();
        
        let result = create_add_prefixed_word_update("؟كلمة", &word_config, &prefix_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "كلمة");
            assert_eq!(payload.word_state, "unintelligible");
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_wrong_prefix_use_space_after_prefix() {
        let word_config = arabic_word_config();
        let prefix_config = emendation_prefix();
        
        // Prefix followed by space and then word should be error
        let result = create_add_prefixed_word_update("* كلمة", &word_config, &prefix_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddError(payload) = &updates[0] {
            assert!(payload.error.to_string().contains("prefix"));
        } else {
            panic!("Expected AddError update for wrong prefix use");
        }
    }

    #[test]
    fn test_prefix_not_at_start_returns_none() {
        let word_config = arabic_word_config();
        let prefix_config = emendation_prefix();
        
        // Input doesn't start with prefix
        let result = create_add_prefixed_word_update("كلمة*", &word_config, &prefix_config);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_prefix_followed_by_non_matching_word_returns_none() {
        let word_config = arabic_word_config();
        let prefix_config = emendation_prefix();
        
        // Prefix followed by non-Arabic (doesn't match word config)
        let result = create_add_prefixed_word_update("*hello", &word_config, &prefix_config);
        
        // Should return None because "hello" doesn't match Arabic word config
        assert!(result.is_none());
    }

    #[test]
    fn test_prefix_only_returns_none() {
        let word_config = arabic_word_config();
        let prefix_config = emendation_prefix();
        
        let result = create_add_prefixed_word_update("*", &word_config, &prefix_config);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_prefixed_word_followed_by_space() {
        let word_config = arabic_word_config();
        let prefix_config = emendation_prefix();
        
        let result = create_add_prefixed_word_update("*كلمة أخرى", &word_config, &prefix_config);
        
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
    fn test_prefix_pre_len_matches_symbol_chars() {
        let word_config = arabic_word_config();
        let prefix_config = emendation_prefix();
        
        let result = create_add_prefixed_word_update("*كلمة", &word_config, &prefix_config);
        
        let updates = result.unwrap();
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.pre_len, 1); // "*" is 1 char
            assert_eq!(payload.post_len, 0);
        } else {
            panic!("Expected AddWord update");
        }
    }

    #[test]
    fn test_multi_char_prefix() {
        let word_config = arabic_word_config();
        let prefix_config = PrefixConfig {
            symbol: "**".to_string(),
            label: "double-emendation".to_string(),
            precedence: usize::MAX,
        };
        
        let result = create_add_prefixed_word_update("**كلمة", &word_config, &prefix_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddWord(payload) = &updates[0] {
            assert_eq!(payload.word, "كلمة");
            assert_eq!(payload.pre_len, 2); // "**" is 2 chars
        } else {
            panic!("Expected AddWord update");
        }
    }
}
