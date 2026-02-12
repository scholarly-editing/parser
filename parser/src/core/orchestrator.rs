use std::collections::VecDeque;

use crate::config::{
    BlockConfig, BracketsConfig, ParserConfig, PrefixConfig, SuffixConfig, TagConfig, WordConfig,
};

use super::{
    handlers::{Handler, HandlerContext, HandlerSequence},
    state::{Mode, Token},
    update_manager::TokenizerUpdateManager,
};

pub struct Orchestrator<'a> {
    handlers: HandlerSequence<'a>,
    block_handlers: Vec<Handler<'a>>,
    block_end_markers: Vec<&'a str>,
}

impl<'a> Orchestrator<'a> {
    pub fn new(config: &'a ParserConfig, mode: &'a Mode) -> Self {
        let space_handler = Handler::Space;
        let line_break_handler = Handler::LineBreak;
        let word_handlers = build_word_handlers(&config.word);
        let prefixed_word_handlers = build_prefixed_word_handlers(&config.prefix, &config.word);
        let suffixed_word_handlers = build_suffixed_word_handlers(&config.suffix, &config.word);
        let tag_handlers = build_tag_handlers(&config.tags);
        let block_handlers = build_block_handlers(&config.block);
        let block_end_markers = collect_block_end_markers(&config.block);

        let mut bracketed_text_handlers =
            VecDeque::from(vec![space_handler.clone(), line_break_handler.clone()]);

        bracketed_text_handlers.extend(word_handlers.iter().cloned());
        bracketed_text_handlers.extend(prefixed_word_handlers.iter().cloned());
        bracketed_text_handlers.extend(suffixed_word_handlers.iter().cloned());
        bracketed_text_handlers.extend(tag_handlers.iter().cloned());
        bracketed_text_handlers.extend(block_handlers.iter().cloned());

        for marker in &block_end_markers {
            bracketed_text_handlers.push_back(Handler::BlockEnd(marker));
        }

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
                    block_handlers,
                    block_end_markers,
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
                    block_handlers,
                    block_end_markers,
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
                    block_handlers,
                    block_end_markers,
                }
            }
        }
    }

    pub fn tokenize(&'a self, input: &'a str) -> (Vec<Token<'a>>, Vec<usize>) {
        let mut state_manager = TokenizerUpdateManager::init(input);

        while state_manager.state.parsing_not_finished() {
            let ctx = HandlerContext {
                is_at_line_start: state_manager.is_at_line_start,
                is_in_block: state_manager.is_in_block(),
                current_block_end_marker: state_manager.get_current_block_end_marker().map(|s| s.to_string()),
            };

            // First check for block end marker if we're in a block
            if state_manager.is_at_line_start {
                if let Some(end_marker) = &ctx.current_block_end_marker {
                    // Find if the end marker exists in our stored markers
                    let mut matched_actions = None;

                    for &stored_marker in &self.block_end_markers {
                        if stored_marker == end_marker {
                            let end_handler = Handler::BlockEnd(stored_marker);
                            if let Some(actions) = end_handler.process_with_context(&state_manager.state.remaining, &ctx) {
                                matched_actions = Some(actions);
                                break;
                            }
                        }
                    }

                    if let Some(actions) = matched_actions {
                        for action in actions {
                            state_manager.apply(action);
                        }
                        // If we applied block end, continue to next iteration
                        if !state_manager.is_in_block() {
                            continue;
                        }
                    }
                }
            }

            // Then check for block start markers (only at line start and not in block)
            if state_manager.is_at_line_start && !state_manager.is_in_block() {
                let mut found_block = false;
                for block_handler in &self.block_handlers {
                    if let Some(actions) = block_handler.process_with_context(&state_manager.state.remaining, &ctx) {
                        for action in actions {
                            state_manager.apply(action);
                        }
                        found_block = true;
                        break;
                    }
                }
                if found_block {
                    continue;
                }
            }

            // Regular handler processing
            if let Some(actions) = self.handlers.process_first_with_context(&state_manager.state.remaining, &ctx) {
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

fn build_block_handlers<'a>(config: &'a Vec<BlockConfig>) -> Vec<Handler<'a>> {
    let mut handlers = vec![];
    let mut config: Vec<_> = config.iter().collect();
    config.sort_by_key(|b| b.precedence);
    for block in config {
        handlers.push(Handler::Block(block));
    }
    handlers
}

fn collect_block_end_markers<'a>(config: &'a Vec<BlockConfig>) -> Vec<&'a str> {
    config
        .iter()
        .filter_map(|b| b.end_marker.as_deref())
        .collect()
}
