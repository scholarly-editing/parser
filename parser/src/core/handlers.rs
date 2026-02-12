use spaces::create_add_passage_line_break_update;

use crate::config::{
    BlockConfig, BracketsConfig, PageConfig, PrefixConfig, SuffixConfig, TagConfig, WordConfig,
};

use super::updates::TokenizerUpdate;

use self::{
    blocks::{create_block_start_update, create_block_end_update},
    bracketed::create_bracketed_text_updates,
    page_breaks::{create_add_default_page_break_update, create_add_page_break_update_from_config},
    prefix::create_add_prefixed_word_update,
    spaces::{create_add_line_break_update, create_add_space_update},
    suffix::create_add_suffixed_word_update,
    tags::create_add_tag_update,
    words::create_add_word_update,
};

pub mod blocks;
pub mod bracketed;
pub mod page_breaks;
pub mod prefix;
pub mod spaces;
pub mod suffix;
pub mod tags;
pub mod words;

#[derive(Clone)]
pub enum Handler<'a> {
    Space,
    LineBreak,
    PassageLineBreak,
    PageBreak(&'a Option<Vec<PageConfig>>),
    PrefixedWord(&'a PrefixConfig, &'a WordConfig),
    SuffixedWord(&'a SuffixConfig, &'a WordConfig),
    Bracketed(
        &'a BracketsConfig,
        Vec<&'a BracketsConfig>,
        HandlerSequence<'a>,
    ),
    Tag(&'a TagConfig),
    Word(&'a WordConfig),
    Block(&'a BlockConfig),
    BlockEnd(&'a str), // end marker string
}

#[derive(Clone)]
pub struct HandlerSequence<'a>(pub Vec<Handler<'a>>);

/// Context passed to handlers that need state information
pub struct HandlerContext {
    pub is_at_line_start: bool,
    pub is_in_block: bool,
    pub current_block_end_marker: Option<String>,
}

impl<'a> Handler<'a> {
    /// Process without context (for backwards compatibility and handlers that don't need context)
    pub fn process(&self, input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
        self.process_with_context(input, &HandlerContext {
            is_at_line_start: false,
            is_in_block: false,
            current_block_end_marker: None,
        })
    }

    /// Process with context for handlers that need state information
    pub fn process_with_context(&self, input: &'a str, ctx: &HandlerContext) -> Option<Vec<TokenizerUpdate<'a>>> {
        match self {
            Handler::Space => create_add_space_update(input),
            Handler::LineBreak => create_add_line_break_update(input),
            Handler::PassageLineBreak => create_add_passage_line_break_update(input),
            Handler::PageBreak(page_config) => {
                if let Some(page_config) = page_config {
                    create_add_page_break_update_from_config(input, page_config)
                } else {
                    create_add_default_page_break_update(input)
                }
            }
            Handler::PrefixedWord(prefix_def, word_def) => {
                create_add_prefixed_word_update(input, word_def, prefix_def)
            }
            Handler::SuffixedWord(suffix_def, word_def) => {
                create_add_suffixed_word_update(input, word_def, suffix_def)
            }
            Handler::Tag(def) => create_add_tag_update(input, def),
            Handler::Word(word_def) => create_add_word_update(input, word_def),
            Handler::Bracketed(brackets_def, all_bracket_defs, content_def) => {
                create_bracketed_text_updates(input, brackets_def, all_bracket_defs, content_def)
            }
            Handler::Block(block_def) => {
                create_block_start_update(input, block_def, ctx.is_at_line_start)
            }
            Handler::BlockEnd(end_marker) => {
                create_block_end_update(input, end_marker, ctx.is_at_line_start)
            }
        }
    }
}

impl<'a> HandlerSequence<'a> {
    pub fn process_first(&self, input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
        let default_ctx = HandlerContext {
            is_at_line_start: false,
            is_in_block: false,
            current_block_end_marker: None,
        };
        self.process_first_with_context(input, &default_ctx)
    }

    pub fn process_first_with_context(&self, input: &'a str, ctx: &HandlerContext) -> Option<Vec<TokenizerUpdate<'a>>> {
        for expression in &self.0 {
            if let Some(update) = expression.process_with_context(input, ctx) {
                return Some(update);
            }
        }
        None
    }

    pub fn process_all(&self, input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
        let mut updates = vec![];
        let mut rest = input;
        while !rest.is_empty() {
            if let Some(update) = self.process_first(rest) {
                let last_update = update.last().unwrap();
                rest = last_update.get_rest().unwrap_or("");
                updates.extend(update);
            } else {
                return None;
            }
        }

        if updates.is_empty() {
            return None;
        }

        Some(updates)
    }
}
