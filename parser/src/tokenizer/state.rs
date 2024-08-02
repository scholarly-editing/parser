// make named tuples

#[derive(Debug)]
#[non_exhaustive]
pub enum Token {
    Word(String, String, usize, usize, usize, usize), // word, state, page number, line number, token order, position
    Tag(String, String, usize, usize, usize, usize), // tag, label, page number, line number, token order, position
    PageBreak(String, usize, usize), // page represntation, page number, position
    Block(String, usize, usize), // block name, page number, position
    WordInBlock(String, String, usize, usize, usize, usize), // word, state, block name, page number, line number, token order
}

#[derive(Debug)]
pub struct TokenizerError {
    pub token: String,
    pub line: usize,
    pub page: usize,
    pub order_in_line: usize,
    pub page_repr: String,
    pub position: usize,
    pub message: String,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TokenizerAction {
    AddPageBreak(String),
    AddLineBreak(String, usize),
    AddPassageLineBreak(String, usize),
    AddSpace(String, usize),
    AddWord(String, String, String, usize),
    AddTag(String, String, String, usize),
}

impl TokenizerAction {
    pub fn apply(self, state: &mut TokenizerState) {
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
pub struct BlockState {
    name: String,
    token_order_in_line: usize,
    line_number: usize,
}

#[derive(Debug)]
pub struct TokenizerState {
    pub tokens: Vec<Token>,
    pub errors: Vec<TokenizerError>,
    pub remaining: String,
    token_order_in_line: usize,
    line_number: usize,
    page_number: usize,
    current_page_repr: String,
    curr_position: usize,
    last_is_space: bool,
    current_block: Option<BlockState>,
}

impl TokenizerState {
    pub fn new(input: &str) -> Self {
        Self {
            tokens: Vec::new(),
            errors: Vec::new(),
            remaining: input.to_string(),
            token_order_in_line: 0,
            line_number: 0,
            page_number: 0,
            current_page_repr: String::new(),
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

    pub fn add_page_break(&mut self, repr: String) {
        self.page_number += 1;
        self.token_order_in_line = 0;
        self.line_number = 0;
        self.current_page_repr = repr.clone();
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

    pub fn add_word(&mut self, word: String, word_state: String) {
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

    pub fn add_tag(&mut self, tag: String, label: String) {
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

    pub fn update_remaining(&mut self, rest: &str) {
        self.remaining = rest.to_string();
    }

    pub fn update_position(&mut self, position: usize) {
        self.curr_position += position;
    }

    pub fn handle_error(&mut self) {
        let error_token = self.remaining.split_whitespace().next().unwrap_or(&self.remaining);
        let remaining_after_error = self.remaining
            .replacen(error_token, "", 1)
            .trim_start()
            .to_string();
        let err = TokenizerError {
            token: error_token.to_string(),
            line: self.line_number,
            page: self.page_number,
            order_in_line: self.token_order_in_line,
            page_repr: self.current_page_repr.clone(),
            position: self.curr_position,
            message: "".to_string(),
        };
        self.update_position(error_token.len());
        self.update_remaining(&remaining_after_error);
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
