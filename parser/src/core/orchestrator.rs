use std::collections::VecDeque;

use crate::config::{
    BracketsConfig, ParserConfig, PrefixConfig, SuffixConfig, TagConfig, WordConfig,
};

use super::{
    handlers::{Handler, HandlerSequence},
    state::{Mode, Token},
    update_manager::TokenizerUpdateManager,
};

pub struct Orchestrator<'a> {
    handlers: HandlerSequence<'a>,
}

impl<'a> Orchestrator<'a> {
    pub fn new(config: &'a ParserConfig, mode: &'a Mode) -> Self {
        let space_handler = Handler::Space;
        let line_break_handler = Handler::LineBreak;
        let word_handlers = build_word_handlers(&config.word);
        let prefixed_word_handlers = build_prefixed_word_handlers(&config.prefix, &config.word);
        let suffixed_word_handlers = build_suffixed_word_handlers(&config.suffix, &config.word);
        let tag_handlers = build_tag_handlers(&config.tags);

        let mut bracketed_text_handlers =
            VecDeque::from(vec![space_handler.clone(), line_break_handler.clone()]);

        bracketed_text_handlers.extend(word_handlers.iter().cloned());
        bracketed_text_handlers.extend(prefixed_word_handlers.iter().cloned());
        bracketed_text_handlers.extend(suffixed_word_handlers.iter().cloned());
        bracketed_text_handlers.extend(tag_handlers.iter().cloned());

        let mut handlers = vec![];

        match mode {
            Mode::MultiplePages => {
                let page_break_handler = Handler::PageBreak(&config.page);
                bracketed_text_handlers.push_front(page_break_handler.clone());

                let text_with_brackets_handlers = build_bracketed_handlers(
                    &config.brackets,
                    HandlerSequence(bracketed_text_handlers.into_iter().collect()),
                );
                handlers.push(page_break_handler);
                handlers.push(line_break_handler);
                handlers.push(space_handler);
                handlers.extend(text_with_brackets_handlers);
                handlers.extend(tag_handlers);
                handlers.extend(prefixed_word_handlers);
                handlers.extend(suffixed_word_handlers);
                handlers.extend(word_handlers);

                Self {
                    handlers: HandlerSequence(handlers),
                }
            }
            Mode::SinglePage => {
                let text_with_brackets_handlers = build_bracketed_handlers(
                    &config.brackets,
                    HandlerSequence(bracketed_text_handlers.into_iter().collect()),
                );
                handlers.push(line_break_handler);
                handlers.push(space_handler);
                handlers.extend(text_with_brackets_handlers);
                handlers.extend(tag_handlers);
                handlers.extend(prefixed_word_handlers);
                handlers.extend(suffixed_word_handlers);
                handlers.extend(word_handlers);

                Self {
                    handlers: HandlerSequence(handlers),
                }
            }
            Mode::Passage => {
                let text_with_brackets_handlers = build_bracketed_handlers(
                    &config.brackets,
                    HandlerSequence(bracketed_text_handlers.into_iter().collect()),
                );
                let passage_line_break_handler = Handler::PassageLineBreak;
                handlers.push(passage_line_break_handler);
                handlers.push(space_handler);
                handlers.extend(text_with_brackets_handlers);
                handlers.extend(tag_handlers);
                handlers.extend(prefixed_word_handlers);
                handlers.extend(suffixed_word_handlers);
                handlers.extend(word_handlers);

                Self {
                    handlers: HandlerSequence(handlers),
                }
            }
        }
    }

    pub fn tokenize(&'a self, input: &'a str) -> (Vec<Token<'a>>, Vec<usize>) {
        let mut state_manager = TokenizerUpdateManager::init(input);
        while state_manager.state.parsing_not_finished() {
            if let Some(actions) = self.handlers.process_first(&state_manager.state.remaining) {
                for action in actions {
                    state_manager.apply(action);
                }
            } else {
                state_manager.handle_unknown_error();
            }
        }
        (state_manager.state.tokens, state_manager.state.errors)
    }
}

fn build_word_handlers<'a>(config: &'a Vec<WordConfig>) -> Vec<Handler<'a>> {
    let mut expressions = vec![];
    let mut config: Vec<_> = config.iter().collect();
    config.sort_by_key(|w| w.precedence);

    for word in config {
        let word_expression = Handler::Word(word);
        expressions.push(word_expression);
    }
    expressions
}

fn build_prefixed_word_handlers<'a>(
    config: &'a Vec<PrefixConfig>,
    words: &'a Vec<WordConfig>,
) -> Vec<Handler<'a>> {
    let mut handlers = vec![];
    let mut config: Vec<_> = config.iter().collect();
    config.sort_by_key(|w| w.precedence);
    for prefix in config {
        for word in words {
            let prefixed_word_handler = Handler::PrefixedWord(prefix, word);
            handlers.push(prefixed_word_handler);
        }
    }
    handlers
}

fn build_suffixed_word_handlers<'a>(
    config: &'a Vec<SuffixConfig>,
    words: &'a Vec<WordConfig>,
) -> Vec<Handler<'a>> {
    let mut handlers = vec![];
    let mut config: Vec<_> = config.iter().collect();
    config.sort_by_key(|w| w.precedence);
    for suffix in config {
        for word in words {
            let suffixed_word_handler = Handler::SuffixedWord(suffix, word);
            handlers.push(suffixed_word_handler);
        }
    }
    handlers
}

fn build_tag_handlers<'a>(config: &'a Vec<TagConfig>) -> Vec<Handler<'a>> {
    let mut expressions = vec![];
    let mut config: Vec<_> = config.iter().collect();
    config.sort_by_key(|w| w.precedence);
    for sequence in config {
        let sequence_expression = Handler::Tag(sequence);
        expressions.push(sequence_expression);
    }
    expressions
}

fn build_bracketed_handlers<'a>(
    config: &'a Vec<BracketsConfig>,
    content_expression_set: HandlerSequence<'a>,
) -> Vec<Handler<'a>> {
    let mut expressions = vec![];
    let mut config: Vec<_> = config.iter().collect();
    config.sort_by_key(|w| w.precedence);
    for bracket in config.iter() {
        let bracket_expression =
            Handler::Bracketed(bracket, config.clone(), content_expression_set.clone());
        expressions.push(bracket_expression);
    }
    expressions
}
