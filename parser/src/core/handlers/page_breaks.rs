use crate::{
    config::PageConfig,
    core::{
        parsers::{ parse_default_page_break, parse_page_break_from_config },
        updates::{ PageBreakPayload, TokenizerUpdate },
    },
};

pub fn create_add_page_break_update_from_config<'a>(
    input: &'a str,
    page_config: &'a Vec<PageConfig>
) -> Option<Vec<TokenizerUpdate<'a>>> {
    page_config.iter().find_map(|p| {
        parse_page_break_from_config(input, &p.prefix, &p.suffix)
            .ok()
            .map(|(rest, (pre_padding, ((prefix, page_number, suffix), line_ending), post_padding))|
                vec![
                    TokenizerUpdate::AddPageBreak(PageBreakPayload {
                        rest: Some(rest),
                        pre_len: pre_padding.chars().count(),
                        len: prefix.chars().count() +
                        page_number.chars().count() +
                        suffix.chars().count(),
                        post_len: line_ending.chars().count() + post_padding.chars().count(),
                        repr: Box::leak(
                            format!("{}{}{}", prefix, page_number, suffix).into_boxed_str()
                        ),
                    })
                ]
            )
    })
}

pub fn create_add_default_page_break_update<'a>(
    input: &'a str
) -> Option<Vec<TokenizerUpdate<'a>>> {
    parse_default_page_break(input)
        .ok()
        .map(|(rest, (pre_padding, (digits, line_ending), post_padding))|
            vec![
                TokenizerUpdate::AddPageBreak(PageBreakPayload {
                    rest: Some(rest),
                    pre_len: pre_padding.len(),
                    len: digits.len(),
                    post_len: line_ending.len() + post_padding.len(),
                    repr: digits,
                })
            ]
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PageConfig;
    use crate::core::updates::TokenizerUpdate;

    fn folio_recto_config() -> PageConfig {
        PageConfig {
            prefix: "fol.".to_string(),
            suffix: "r".to_string(),
            precedence: usize::MAX,
        }
    }

    fn folio_verso_config() -> PageConfig {
        PageConfig {
            prefix: "fol.".to_string(),
            suffix: "v".to_string(),
            precedence: usize::MAX,
        }
    }

    #[test]
    fn test_default_page_break() {
        let result = create_add_default_page_break_update("1\ntext");
        
        assert!(result.is_some());
        let updates = result.unwrap();
        assert_eq!(updates.len(), 1);
        
        if let TokenizerUpdate::AddPageBreak(payload) = &updates[0] {
            assert_eq!(payload.repr, "1");
        } else {
            panic!("Expected AddPageBreak update");
        }
    }

    #[test]
    fn test_default_page_break_multi_digit() {
        let result = create_add_default_page_break_update("123\ntext");
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddPageBreak(payload) = &updates[0] {
            assert_eq!(payload.repr, "123");
        } else {
            panic!("Expected AddPageBreak update");
        }
    }

    #[test]
    fn test_folio_page_break_recto() {
        let page_config = vec![folio_recto_config(), folio_verso_config()];
        
        let result = create_add_page_break_update_from_config("fol.1r\ntext", &page_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddPageBreak(payload) = &updates[0] {
            assert_eq!(payload.repr, "fol.1r");
        } else {
            panic!("Expected AddPageBreak update");
        }
    }

    #[test]
    fn test_folio_page_break_verso() {
        let page_config = vec![folio_recto_config(), folio_verso_config()];
        
        let result = create_add_page_break_update_from_config("fol.1v\ntext", &page_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddPageBreak(payload) = &updates[0] {
            assert_eq!(payload.repr, "fol.1v");
        } else {
            panic!("Expected AddPageBreak update");
        }
    }

    #[test]
    fn test_page_break_without_line_ending_returns_none() {
        let result = create_add_default_page_break_update("1 text");
        
        assert!(result.is_none());
    }

    #[test]
    fn test_folio_page_break_with_spaces() {
        let page_config = vec![folio_recto_config()];
        
        // With leading and trailing spaces
        let result = create_add_page_break_update_from_config("  fol.1r\n  text", &page_config);
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddPageBreak(payload) = &updates[0] {
            assert_eq!(payload.pre_len, 2); // 2 leading spaces
        } else {
            panic!("Expected AddPageBreak update");
        }
    }

    #[test]
    fn test_wrong_folio_suffix_returns_none() {
        let page_config = vec![folio_recto_config()]; // Only recto
        
        let result = create_add_page_break_update_from_config("fol.1v\ntext", &page_config);
        
        // Verso suffix not in config, should fail
        assert!(result.is_none());
    }

    #[test]
    fn test_no_page_number_returns_none() {
        let page_config = vec![folio_recto_config()];
        
        let result = create_add_page_break_update_from_config("fol.r\ntext", &page_config);
        
        // No number between prefix and suffix
        assert!(result.is_none());
    }

    #[test]
    fn test_page_break_crlf() {
        let result = create_add_default_page_break_update("1\r\ntext");
        
        assert!(result.is_some());
        let updates = result.unwrap();
        
        if let TokenizerUpdate::AddPageBreak(payload) = &updates[0] {
            // post_len should include \r\n
            assert!(payload.post_len >= 2);
        } else {
            panic!("Expected AddPageBreak update");
        }
    }

    #[test]
    fn test_empty_page_config() {
        let page_config: Vec<PageConfig> = vec![];
        
        let result = create_add_page_break_update_from_config("fol.1r\ntext", &page_config);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_non_page_text_returns_none() {
        let result = create_add_default_page_break_update("text\nmore");
        
        assert!(result.is_none());
    }

    #[test]
    fn test_page_break_len_calculation() {
        let page_config = vec![folio_recto_config()];
        
        let result = create_add_page_break_update_from_config("fol.12r\ntext", &page_config);
        
        let updates = result.unwrap();
        if let TokenizerUpdate::AddPageBreak(payload) = &updates[0] {
            // "fol." (4) + "12" (2) + "r" (1) = 7
            assert_eq!(payload.len, 7);
        } else {
            panic!("Expected AddPageBreak update");
        }
    }
}
