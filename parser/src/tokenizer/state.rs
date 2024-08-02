// make named tuples

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Token<'a> {
    Word(&'a str, &'a str, usize, usize, usize, usize), // word, state, page number, line number, token order, position
    Tag(&'a str, &'a str, usize, usize, usize, usize), // tag, label, page number, line number, token order, position
    PageBreak(&'a str, usize, usize), // page represntation, page number, position
    Block(&'a str, usize, usize), // block name, page number, position
    WordInBlock(&'a str, &'a str, &'a str, usize, usize, usize), // word, state, block name, page number, line number, token order
}

#[derive(Debug, Clone)]
pub struct TokenizerError<'a> {
    pub token: &'a str,
    pub line: usize,
    pub page: usize,
    pub order_in_line: usize,
    pub page_repr: &'a str,
    pub position: usize,
    pub message: &'a str,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TokenizerAction<'a> {
    AddPageBreak(&'a str),
    AddLineBreak(&'a str, usize),
    AddPassageLineBreak(&'a str, usize),
    AddSpace(&'a str, usize),
    AddWord(&'a str, &'a str, &'a str, usize),
    AddTag(&'a str, &'a str, &'a str, usize),
}

impl<'a> TokenizerAction<'a> {
    pub fn apply(self, state: &mut TokenizerState<'a>) {
        match self {
            TokenizerAction::AddPageBreak(_) => unimplemented!(),
            TokenizerAction::AddLineBreak(rest, position) => {
                state.update_position(position);
                state.add_line_break();
                state.update_remaining(&rest);
            }
            TokenizerAction::AddPassageLineBreak(rest, position) => {
                state.update_position(position);
                state.add_space();
                state.update_remaining(&rest);
            }
            TokenizerAction::AddSpace(rest, position) => {
                state.update_position(position);
                state.add_space();
                state.update_remaining(&rest);
            }
            TokenizerAction::AddWord(word, word_state, rest, position) => {
                state.update_position(position);
                state.add_word(word, word_state);
                state.update_remaining(&rest);
            }
            TokenizerAction::AddTag(tag, label, rest, position) => {
                state.update_position(position);
                state.add_tag(tag, label);
                state.update_remaining(&rest);
            }
        }
    }
}

#[derive(Debug)]
pub struct BlockState<'a> {
    name: &'a str,
    token_order_in_line: usize,
    line_number: usize,
}

#[derive(Debug)]
pub struct TokenizerState<'a> {
    pub tokens: Vec<Token<'a>>,
    pub errors: Vec<TokenizerError<'a>>,
    pub remaining: &'a str,
    token_order_in_line: usize,
    line_number: usize,
    page_number: usize,
    current_page_repr: &'a str,
    curr_position: usize,
    last_is_space: bool,
    current_block: Option<BlockState<'a>>,
}

impl<'a> TokenizerState<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: Vec::with_capacity(input.len() / 5),
            errors: Vec::with_capacity(input.len() / 100),
            remaining: input,
            token_order_in_line: 0,
            line_number: 0,
            page_number: 0,
            current_page_repr: "",
            curr_position: 0,
            last_is_space: true, // assume that an input starts with a psuedo space
            current_block: None,
        }
    }

    pub fn parsing_not_finished(&self) -> bool {
        !self.remaining.is_empty()
    }

    pub fn can_add_token(&self) -> bool {
        self.last_is_space
    }

    pub fn add_page_break(&mut self, repr: &'a str) {
        self.page_number += 1;
        self.token_order_in_line = 0;
        self.line_number = 0;
        self.current_page_repr = repr;
        self.tokens.push(Token::PageBreak(repr, self.page_number, self.curr_position));
        self.last_is_space = true;
    }

    pub fn add_line_break(&mut self) {
        self.line_number += 1;
        self.token_order_in_line = 0;
        self.last_is_space = true;
    }

    pub fn add_space(&mut self) {
        self.last_is_space = true;
    }

    pub fn add_word(&mut self, word: &'a str, word_state: &'a str) {
        self.tokens.push(
            Token::Word(
                word,
                word_state,
                self.page_number,
                self.line_number,
                self.token_order_in_line,
                self.curr_position
            )
        );
        self.last_is_space = false;
        self.token_order_in_line += 1;
    }

    pub fn add_tag(&mut self, tag: &'a str, label: &'a str) {
        self.tokens.push(
            Token::Tag(
                tag,
                label,
                self.page_number,
                self.line_number,
                self.token_order_in_line,
                self.curr_position
            )
        );
        self.last_is_space = false;
    }

    pub fn update_remaining(&mut self, rest: &'a str) {
        self.remaining = rest;
    }

    pub fn update_position(&mut self, position: usize) {
        self.curr_position += position;
    }

    pub fn handle_error(&mut self) {
        let error_token = self.remaining.split_whitespace().next().unwrap_or(self.remaining);
        let remaining_after_error = &self.remaining[error_token.len()..].trim_start();
        let err = TokenizerError {
            token: error_token,
            line: self.line_number,
            page: self.page_number,
            order_in_line: self.token_order_in_line,
            page_repr: self.current_page_repr,
            position: self.curr_position,
            message: "",
        };
        self.update_position(error_token.len());
        self.update_remaining(remaining_after_error);
        self.errors.push(err);
    }
}

pub enum Mode {
    SinglePage,
    MultiplePages,
    Passage,
}

impl Default for Mode {
    fn default() -> Self {
        Self::SinglePage
    }
}
