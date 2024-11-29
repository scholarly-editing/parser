use crate::core::{
    parsers::{ parse_line_break, parse_spaces },
    updates::{ LineBreakPayload, SpacePayload, TokenizerUpdate },
};

pub fn create_add_space_update<'a>(input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
    parse_spaces(input)
        .ok()
        .map(|(rest, repr)|
            vec![
                TokenizerUpdate::AddSpace(SpacePayload {
                    rest: Some(rest),
                    len: repr.chars().count(),
                })
            ]
        )
}

pub fn create_add_line_break_update<'a>(input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
    parse_line_break(input)
        .ok()
        .map(|(rest, chars)|
            vec![
                TokenizerUpdate::AddLineBreak(LineBreakPayload {
                    rest: Some(rest),
                    len: chars.chars().count(),
                })
            ]
        )
}
