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
