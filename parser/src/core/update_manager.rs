use super::errors::TextError;
use super::{
    updates::{
        BlockEndPayload,
        BlockPayload,
        ErrorPayload,
        LineBreakPayload,
        PageBreakPayload,
        PassageLineBreakPayload,
        SpacePayload,
        TagPayload,
        TokenizerUpdate,
        WordPayload,
    },
    state::{ BlockState, BlockToken, PageBreakToken, TagToken, Token, TokenizerError, TokenizerState, WordInBlockToken, WordToken },
};

pub struct TokenizerUpdateManager<'a> {
    pub state: TokenizerState<'a>,
    pub is_at_line_start: bool,
}

impl<'a> TokenizerUpdateManager<'a> {
    pub fn init(input: &'a str) -> Self {
        Self {
            state: TokenizerState::new(input),
            is_at_line_start: true, // Start of input is like start of line
        }
    }

    pub fn apply(&mut self, update: TokenizerUpdate<'a>) {
        match update {
            TokenizerUpdate::AddPageBreak(
                PageBreakPayload { rest, pre_len: position, len, post_len: position_after, repr },
            ) => {
                self.update_position(position);
                self.add_page_break(repr, len);
                self.update_position(len + position_after);
                self.update_remaining(rest);
                self.is_at_line_start = true;
            }
            TokenizerUpdate::AddLineBreak(LineBreakPayload { rest, len }) => {
                self.add_line_break();
                self.update_position(len);
                self.update_remaining(rest);
                self.is_at_line_start = true;
            }
            TokenizerUpdate::AddPassageLineBreak(PassageLineBreakPayload { rest, len }) => {
                self.add_space();
                self.update_position(len);
                self.update_remaining(rest);
                self.is_at_line_start = true;
            }
            TokenizerUpdate::AddSpace(SpacePayload { rest, len }) => {
                self.add_space();
                self.update_position(len);
                self.update_remaining(rest);
                // Space doesn't change line_start status
            }
            TokenizerUpdate::AddWord(
                WordPayload { word, word_state, word_type, rest, pre_len, len, post_len },
            ) => {
                self.update_position(pre_len);
                if self.state.can_add_token() {
                    self.add_word(word, word_state, word_type, len);
                } else {
                    self.handle_error(TextError::Custom("Can't add word here".to_string()), word, len);
                }
                self.update_position(len + post_len);
                self.update_remaining(rest);
                self.is_at_line_start = false;
            }
            TokenizerUpdate::AddTag(TagPayload { tag, label, rest, pre_len, len, post_len }) => {
                self.update_position(pre_len);
                if self.state.can_add_token() {
                    self.add_tag(tag, label, len);
                } else {
                    self.handle_error(TextError::Custom("Can't add tag here".to_string()), tag, len);
                }
                self.update_position(len + post_len);
                self.update_remaining(rest);
                self.is_at_line_start = false;
            }
            TokenizerUpdate::AddError(ErrorPayload { text, error, rest, len }) => {
                self.handle_error(error, text, len);
                self.update_position(len);
                self.update_remaining(rest);
                self.is_at_line_start = false;
            }
            TokenizerUpdate::AddBlockStart(BlockPayload {
                block_name, block_type, has_end_marker, end_marker, rest, len
            }) => {
                self.add_block_start(block_name, block_type, has_end_marker, end_marker, len);
                self.update_position(len);
                self.update_remaining(rest);
                self.is_at_line_start = false;
            }
            TokenizerUpdate::AddBlockEnd(BlockEndPayload { rest, len }) => {
                self.end_current_block();
                self.update_position(len);
                self.update_remaining(rest);
                // Block end marker consumes `---\n` (terminated with line_ending/eof),
                // so we need to increment line_number for the consumed newline
                self.state.line_number += 1;
                self.is_at_line_start = true;
            }
        }
    }

    pub fn handle_unknown_error(&mut self) {
        let error_token = self.state.remaining
            .char_indices()
            .find(|(_, c)| c.is_whitespace())
            .map_or(self.state.remaining, |(i, _)| &self.state.remaining[..i]);

        let remaining_after_error = self.state.remaining[error_token.len()..].trim_start_matches(
            |c: char| c.is_whitespace()
        );
        let err = TokenizerError {
            token: error_token,
            line: self.state.line_number,
            page: self.state.page_number,
            order_in_line: self.state.token_order_in_line,
            page_repr: self.state.current_page_repr,
            span: (self.state.parsed_len, self.state.parsed_len + error_token.chars().count()),
            error: TextError::Custom("Unwanted characters, can also be: 1) incomplete tag, 2) tokenizer bug.".to_string()),
        };
        self.update_position(error_token.chars().count() + 1); // split_whitespace() "eats" a space
        self.update_remaining(Some(remaining_after_error));
        self.state.tokens.push(Token::Error(err));
        self.state.errors.push(self.state.tokens.len() - 1);
    }

    fn add_page_break(&mut self, repr: &'a str, len: usize) {
        self.state.page_number += 1;
        self.state.token_order_in_line = 0;
        self.state.line_number = 0;
        self.state.current_page_repr = repr;
        self.state.tokens.push(
            Token::PageBreak(PageBreakToken {
                page_representation: repr,
                page_number: self.state.page_number,
                span: (self.state.parsed_len, self.state.parsed_len + len),
            })
        );
        self.state.last_is_space = true;
    }

    fn add_line_break(&mut self) {
        self.state.line_number += 1;
        self.state.token_order_in_line = 0;
        self.state.last_is_space = true;
    }

    fn add_space(&mut self) {
        self.state.last_is_space = true;
    }

    fn add_word(&mut self, word: &'a str, word_state: &'a str, word_type: &'a str, len: usize) {
        // Check if we're inside a block
        if let Some(ref block) = self.state.current_block {
            self.state.tokens.push(
                Token::WordInBlock(WordInBlockToken {
                    word,
                    state: word_state,
                    block_name: block.name,
                    line_number: self.state.line_number - block.line_number,
                    token_order: self.state.token_order_in_line,
                    span: (self.state.parsed_len, self.state.parsed_len + len),
                })
            );
        } else {
            self.state.tokens.push(
                Token::Word(WordToken {
                    word,
                    state: word_state,
                    word_type,
                    page_number: self.state.page_number,
                    line_number: self.state.line_number,
                    token_order: self.state.token_order_in_line,
                    span: (self.state.parsed_len, self.state.parsed_len + len),
                })
            );
        }
        self.state.last_is_space = false;
        self.state.token_order_in_line += 1;
    }

    fn add_tag(&mut self, tag: &'a str, label: &'a str, len: usize) {
        self.state.tokens.push(
            Token::Tag(TagToken {
                tag,
                label,
                page_number: self.state.page_number,
                line_number: self.state.line_number,
                token_order: self.state.token_order_in_line,
                span: (self.state.parsed_len, self.state.parsed_len + len),
            })
        );
        self.state.last_is_space = false;
    }

    fn add_block_start(&mut self, block_name: &'a str, block_type: &'a str, _has_end_marker: bool, end_marker: Option<&'a str>, len: usize) {
        // Add block token
        self.state.tokens.push(
            Token::Block(BlockToken {
                block_name,
                page_number: self.state.page_number,
                span: (self.state.parsed_len, self.state.parsed_len + len),
            })
        );

        // Only set current_block for WITH_TEXT blocks that need content parsing
        if block_type == "with_text" {
            self.state.current_block = Some(BlockState {
                name: block_name,
                end_marker,
                token_order_in_line: self.state.token_order_in_line,
                // Store the next line number so block-internal lines start at 0
                line_number: self.state.line_number + 1,
            });
        }
        // For standalone blocks, no state change needed

        self.state.last_is_space = true;
        self.state.token_order_in_line = 0;
    }

    fn end_current_block(&mut self) {
        self.state.current_block = None;
        self.state.last_is_space = true;
        self.state.token_order_in_line = 0;
    }

    /// Check if we're currently inside a block and return the end marker if any
    pub fn get_current_block_end_marker(&self) -> Option<&'a str> {
        self.state.current_block.as_ref().and_then(|b| b.end_marker)
    }

    /// Check if we're inside a block
    pub fn is_in_block(&self) -> bool {
        self.state.current_block.is_some()
    }

    fn update_remaining(&mut self, rest: Option<&'a str>) {
        if let Some(rest) = rest {
            self.state.remaining = rest;
        }
    }

    fn update_position(&mut self, position: usize) {
        self.state.parsed_len += position;
    }

    fn handle_error(&mut self, error: TextError, text: &'a str, len: usize) {
        let err = TokenizerError {
            token: text,
            line: self.state.line_number,
            page: self.state.page_number,
            order_in_line: self.state.token_order_in_line,
            page_repr: self.state.current_page_repr,
            span: (self.state.parsed_len, self.state.parsed_len + len),
            error,
        };
        self.state.tokens.push(Token::Error(err));
        self.state.errors.push(self.state.tokens.len() - 1);
        self.state.last_is_space = false;
    }
}
