use crate::config::{
    BracketsConfig,
    PageConfig,
    PrefixConfig,
    SuffixConfig,
    TagConfig,
    WordConfig,
};

use super::updates::TokenizerUpdate;

use self::{
    bracketed::create_bracketed_text_updates,
    page_breaks::{ create_add_default_page_break_update, create_add_page_break_update_from_config },
    prefix::create_add_prefixed_word_update,
    spaces::{ create_add_line_break_update, create_add_space_update },
    suffix::create_add_suffixed_word_update,
    tags::create_add_tag_update,
    words::create_add_word_update,
};

pub mod words;
pub mod tags;
pub mod spaces;
pub mod page_breaks;
pub mod bracketed;
pub mod prefix;
pub mod suffix;

#[derive(Clone)]
pub enum Handler<'a> {
    Space,
    LineBreak,
    PageBreak(&'a Option<Vec<PageConfig>>),
    PrefixedWord(&'a PrefixConfig, &'a WordConfig),
    SuffixedWord(&'a SuffixConfig, &'a WordConfig),
    Bracketed(&'a BracketsConfig, Vec<&'a BracketsConfig>, HandlerSequence<'a>),
    Tag(&'a TagConfig),
    Word(&'a WordConfig),
}

#[derive(Clone)]
pub struct HandlerSequence<'a>(pub Vec<Handler<'a>>);

impl<'a> Handler<'a> {
    pub fn process(&'a self, input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
        match self {
            Handler::Space => create_add_space_update(input),
            Handler::LineBreak => create_add_line_break_update(input),
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
        }
    }
}

impl<'a> HandlerSequence<'a> {
    pub fn process_first(&'a self, input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
        for expression in &self.0 {
            if let Some(update) = expression.process(input) {
                return Some(update);
            }
        }
        None
    }

    pub fn process_all(&'a self, input: &'a str) -> Option<Vec<TokenizerUpdate<'a>>> {
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
