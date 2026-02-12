use crate::{
    config::TagConfig,
    core::{ parsers::parse_tag, updates::{ TagPayload, TokenizerUpdate } },
};

pub fn create_add_tag_update<'a>(
    input: &'a str,
    def: &'a TagConfig
) -> Option<Vec<TokenizerUpdate<'a>>> {
    parse_tag(&def.symbol)(input)
        .ok()
        .map(|(rest, _)| {
            vec![
                TokenizerUpdate::AddTag(TagPayload {
                    tag: &def.symbol,
                    label: &def.label,
                    rest: Some(rest),
                    pre_len: 0,
                    len: def.symbol.chars().count(),
                    post_len: 0,
                })
            ]
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TagConfig;
    use crate::core::updates::TokenizerUpdate;

    fn lacuna_tag() -> TagConfig {
        TagConfig {
            symbol: "***".to_string(),
            label: "lacuna".to_string(),
            precedence: usize::MAX,
        }
    }

    fn damage_tag() -> TagConfig {
        TagConfig {
            symbol: "...".to_string(),
            label: "damage".to_string(),
            precedence: usize::MAX,
        }
    }

    #[test]
    fn test_lacuna_tag_parsing() {
        let tag_config = lacuna_tag();

        let result = create_add_tag_update("***", &tag_config);

        assert!(result.is_some());
        let updates = result.unwrap();
        assert_eq!(updates.len(), 1);

        if let TokenizerUpdate::AddTag(payload) = &updates[0] {
            assert_eq!(payload.tag, "***");
            assert_eq!(payload.label, "lacuna");
            assert_eq!(payload.len, 3);
        } else {
            panic!("Expected AddTag update");
        }
    }

    #[test]
    fn test_damage_tag_parsing() {
        let tag_config = damage_tag();

        let result = create_add_tag_update("...", &tag_config);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddTag(payload) = &updates[0] {
            assert_eq!(payload.tag, "...");
            assert_eq!(payload.label, "damage");
        } else {
            panic!("Expected AddTag update");
        }
    }

    #[test]
    fn test_tag_followed_by_text() {
        let tag_config = lacuna_tag();

        let result = create_add_tag_update("*** text", &tag_config);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddTag(payload) = &updates[0] {
            assert_eq!(payload.tag, "***");
            assert_eq!(payload.rest, Some(" text"));
        } else {
            panic!("Expected AddTag update");
        }
    }

    #[test]
    fn test_tag_not_at_start_returns_none() {
        let tag_config = lacuna_tag();

        // Tag not at start of input
        let result = create_add_tag_update("text ***", &tag_config);

        assert!(result.is_none());
    }

    #[test]
    fn test_partial_tag_returns_none() {
        let tag_config = lacuna_tag();

        // Only two asterisks, not three
        let result = create_add_tag_update("**", &tag_config);

        assert!(result.is_none());
    }

    #[test]
    fn test_wrong_tag_returns_none() {
        let tag_config = lacuna_tag();

        // Different tag
        let result = create_add_tag_update("...", &tag_config);

        assert!(result.is_none());
    }

    #[test]
    fn test_tag_len_is_symbol_length() {
        let tag_config = lacuna_tag();

        let result = create_add_tag_update("***", &tag_config);

        let updates = result.unwrap();
        if let TokenizerUpdate::AddTag(payload) = &updates[0] {
            assert_eq!(payload.len, 3); // "***" is 3 chars
        } else {
            panic!("Expected AddTag update");
        }
    }

    #[test]
    fn test_single_char_tag() {
        let tag_config = TagConfig {
            symbol: "#".to_string(),
            label: "marker".to_string(),
            precedence: usize::MAX,
        };

        let result = create_add_tag_update("# text", &tag_config);

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddTag(payload) = &updates[0] {
            assert_eq!(payload.tag, "#");
            assert_eq!(payload.len, 1);
        } else {
            panic!("Expected AddTag update");
        }
    }

    #[test]
    fn test_empty_input_returns_none() {
        let tag_config = lacuna_tag();

        let result = create_add_tag_update("", &tag_config);

        assert!(result.is_none());
    }

    #[test]
    fn test_tag_pre_and_post_len_are_zero() {
        let tag_config = lacuna_tag();

        let result = create_add_tag_update("***", &tag_config);

        let updates = result.unwrap();
        if let TokenizerUpdate::AddTag(payload) = &updates[0] {
            assert_eq!(payload.pre_len, 0);
            assert_eq!(payload.post_len, 0);
        } else {
            panic!("Expected AddTag update");
        }
    }
}
