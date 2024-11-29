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
