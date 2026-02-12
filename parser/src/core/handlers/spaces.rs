use crate::core::{
    parsers::{parse_line_break, parse_spaces},
    updates::{LineBreakPayload, SpacePayload, TokenizerUpdate},
};

pub fn create_add_space_update<'a>(input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
    parse_spaces(input).ok().map(|(rest, repr)| {
        vec![TokenizerUpdate::AddSpace(SpacePayload {
            rest: Some(rest),
            len: repr.chars().count(),
        })]
    })
}

pub fn create_add_line_break_update<'a>(input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
    parse_line_break(input).ok().map(|(rest, chars)| {
        vec![TokenizerUpdate::AddLineBreak(LineBreakPayload {
            rest: Some(rest),
            len: chars.chars().count(),
        })]
    })
}

pub fn create_add_passage_line_break_update<'a>(
    input: &'a str,
) -> Option<Vec<TokenizerUpdate<'a>>> {
    parse_line_break(input).ok().map(|(rest, chars)| {
        vec![TokenizerUpdate::AddSpace(SpacePayload {
            rest: Some(rest),
            len: chars.chars().count(),
        })]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::updates::TokenizerUpdate;

    #[test]
    fn test_single_space() {
        let result = create_add_space_update(" text");

        assert!(result.is_some());
        let updates = result.unwrap();
        assert_eq!(updates.len(), 1);

        if let TokenizerUpdate::AddSpace(payload) = &updates[0] {
            assert_eq!(payload.len, 1);
            assert_eq!(payload.rest, Some("text"));
        } else {
            panic!("Expected AddSpace update");
        }
    }

    #[test]
    fn test_multiple_spaces() {
        let result = create_add_space_update("    text");

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddSpace(payload) = &updates[0] {
            assert_eq!(payload.len, 4);
            assert_eq!(payload.rest, Some("text"));
        } else {
            panic!("Expected AddSpace update");
        }
    }

    #[test]
    fn test_tab_as_space() {
        let result = create_add_space_update("\ttext");

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddSpace(payload) = &updates[0] {
            assert_eq!(payload.len, 1); // Tab counts as 1 char
        } else {
            panic!("Expected AddSpace update");
        }
    }

    #[test]
    fn test_no_space_at_start_returns_none() {
        let result = create_add_space_update("text");

        assert!(result.is_none());
    }

    #[test]
    fn test_empty_input_returns_none() {
        let result = create_add_space_update("");

        assert!(result.is_none());
    }

    #[test]
    fn test_line_break_not_space() {
        // Line breaks should not be parsed as spaces
        let result = create_add_space_update("\ntext");

        assert!(result.is_none());
    }

    #[test]
    fn test_line_break_parsing() {
        let result = create_add_line_break_update("\ntext");

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddLineBreak(payload) = &updates[0] {
            assert_eq!(payload.len, 1);
            assert_eq!(payload.rest, Some("text"));
        } else {
            panic!("Expected AddLineBreak update");
        }
    }

    #[test]
    fn test_crlf_line_break() {
        let result = create_add_line_break_update("\r\ntext");

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddLineBreak(payload) = &updates[0] {
            assert_eq!(payload.len, 2); // \r\n is 2 chars
        } else {
            panic!("Expected AddLineBreak update");
        }
    }

    #[test]
    fn test_no_line_break_returns_none() {
        let result = create_add_line_break_update("text");

        assert!(result.is_none());
    }

    #[test]
    fn test_passage_line_break() {
        let result = create_add_passage_line_break_update("\ntext");

        assert!(result.is_some());
        let updates = result.unwrap();

        // In passage mode, line breaks become spaces
        if let TokenizerUpdate::AddSpace(payload) = &updates[0] {
            assert_eq!(payload.len, 1);
            assert_eq!(payload.rest, Some("text"));
        } else {
            panic!("Expected AddSpace update for passage line break");
        }
    }

    #[test]
    fn test_space_only_input() {
        let result = create_add_space_update("   ");

        assert!(result.is_some());
        let updates = result.unwrap();

        if let TokenizerUpdate::AddSpace(payload) = &updates[0] {
            assert_eq!(payload.len, 3);
            assert_eq!(payload.rest, Some(""));
        } else {
            panic!("Expected AddSpace update");
        }
    }
}
