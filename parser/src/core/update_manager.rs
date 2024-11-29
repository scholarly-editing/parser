use super::{
    updates::{
        ErrorPayload,
        LineBreakPayload,
        PageBreakPayload,
        PassageLineBreakPayload,
        SpacePayload,
        TagPayload,
        TokenizerUpdate,
        WordPayload,
    },
    state::{ PageBreakToken, TagToken, Token, TokenizerError, TokenizerState, WordToken },
};

pub struct TokenizerUpdateManager<'a> {
    pub state: TokenizerState<'a>,
}

impl<'a> TokenizerUpdateManager<'a> {
    pub fn init(input: &'a str) -> Self {
        Self { state: TokenizerState::new(input) }
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
            }
            TokenizerUpdate::AddLineBreak(LineBreakPayload { rest, len }) => {
                self.add_line_break();
                self.update_position(len);
                self.update_remaining(rest);
            }
            TokenizerUpdate::AddPassageLineBreak(PassageLineBreakPayload { rest, len }) => {
                self.add_space();
                self.update_position(len);
                self.update_remaining(rest);
            }
            TokenizerUpdate::AddSpace(SpacePayload { rest, len }) => {
                self.add_space();
                self.update_position(len);
                self.update_remaining(rest);
            }
            TokenizerUpdate::AddWord(
                WordPayload { word, word_state, word_type, rest, pre_len, len, post_len },
            ) => {
                self.update_position(pre_len);
                if self.state.can_add_token() {
                    self.add_word(word, word_state, word_type, len);
                } else {
                    self.handle_error("Can't add word here", word, len);
                }
                self.update_position(len + post_len);
                self.update_remaining(rest)
            }
            TokenizerUpdate::AddTag(TagPayload { tag, label, rest, pre_len, len, post_len }) => {
                self.update_position(pre_len);
                if self.state.can_add_token() {
                    self.add_tag(tag, label, len);
                } else {
                    self.handle_error("Can't add tag here", tag, len);
                }
                self.update_position(len + post_len);
                self.update_remaining(rest);
            }
            TokenizerUpdate::AddError(ErrorPayload { text, message, rest, len }) => {
                self.handle_error(message, text, len);
                self.update_position(len);
                self.update_remaining(rest);
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
            message: "Unwanted characters, can also be: 1) incomplete tag, 2) tokenizer bug.",
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

    fn update_remaining(&mut self, rest: Option<&'a str>) {
        if let Some(rest) = rest {
            self.state.remaining = rest;
        }
    }

    fn update_position(&mut self, position: usize) {
        self.state.parsed_len += position;
    }

    fn handle_error(&mut self, message: &'a str, text: &'a str, len: usize) {
        let err = TokenizerError {
            token: text,
            line: self.state.line_number,
            page: self.state.page_number,
            order_in_line: self.state.token_order_in_line,
            page_repr: self.state.current_page_repr,
            span: (self.state.parsed_len, self.state.parsed_len + len),
            message,
        };
        self.state.tokens.push(Token::Error(err));
        self.state.errors.push(self.state.tokens.len() - 1);
        self.state.last_is_space = false;
    }
}
