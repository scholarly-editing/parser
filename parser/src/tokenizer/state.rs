// make named tuples

#[derive(Debug, Clone)]
pub struct WordToken<'a> {
    pub word: &'a str,
    pub state: &'a str,
    pub word_type: &'a str,
    pub page_number: usize,
    pub line_number: usize,
    pub token_order: usize,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct TagToken<'a> {
    pub tag: &'a str,
    pub label: &'a str,
    pub page_number: usize,
    pub line_number: usize,
    pub token_order: usize,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct PageBreakToken<'a> {
    pub page_representation: &'a str,
    pub page_number: usize,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct BlockToken<'a> {
    pub block_name: &'a str,
    pub page_number: usize,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct WordInBlockToken<'a> {
    pub word: &'a str,
    pub state: &'a str,
    pub block_name: &'a str,
    pub page_number: usize,
    pub line_number: usize,
    pub token_order: usize,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Token<'a> {
    Word(WordToken<'a>),
    Tag(TagToken<'a>),
    PageBreak(PageBreakToken<'a>),
    Block(BlockToken<'a>),
    WordInBlock(WordInBlockToken<'a>),
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
pub struct PageBreakPayload<'a> {
    pub rest: &'a str,
}

#[derive(Debug)]
pub struct LineBreakPayload<'a> {
    pub rest: &'a str,
    pub position: usize,
}

#[derive(Debug)]
pub struct PassageLineBreakPayload<'a> {
    pub rest: &'a str,
    pub position: usize,
}

#[derive(Debug)]
pub struct SpacePayload<'a> {
    pub rest: &'a str,
    pub position: usize,
}

#[derive(Debug)]
pub struct WordPayload<'a> {
    pub word: &'a str,
    pub word_state: &'a str,
    pub word_type: &'a str,
    pub rest: &'a str,
    pub position: usize,
}

#[derive(Debug)]
pub struct TagPayload<'a> {
    pub tag: &'a str,
    pub label: &'a str,
    pub rest: &'a str,
    pub position: usize,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TokenizerAction<'a> {
    AddPageBreak(PageBreakPayload<'a>),
    AddLineBreak(LineBreakPayload<'a>),
    AddPassageLineBreak(PassageLineBreakPayload<'a>),
    AddSpace(SpacePayload<'a>),
    AddWord(WordPayload<'a>),
    AddTag(TagPayload<'a>),
}

impl<'a> TokenizerAction<'a> {
    pub fn apply(self, state: &mut TokenizerState<'a>) {
        match self {
            TokenizerAction::AddPageBreak(_) => unimplemented!(),
            TokenizerAction::AddLineBreak(LineBreakPayload { rest, position }) => {
                state.update_position(position);
                state.add_line_break();
                state.update_remaining(&rest);
            }
            TokenizerAction::AddPassageLineBreak(PassageLineBreakPayload { rest, position }) => {
                state.update_position(position);
                state.add_space();
                state.update_remaining(&rest);
            }
            TokenizerAction::AddSpace(SpacePayload { rest, position }) => {
                state.update_position(position);
                state.add_space();
                state.update_remaining(&rest);
            }
            TokenizerAction::AddWord(
                WordPayload { word, word_state, word_type, rest, position },
            ) => {
                state.update_position(position);
                state.add_word(word, word_state, word_type);
                state.update_remaining(&rest);
            }
            TokenizerAction::AddTag(TagPayload { tag, label, rest, position }) => {
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
        self.tokens.push(
            Token::PageBreak(PageBreakToken {
                page_representation: repr,
                page_number: self.page_number,
                position: self.curr_position,
            })
        );
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

    pub fn add_word(&mut self, word: &'a str, word_state: &'a str, word_type: &'a str) {
        self.tokens.push(
            Token::Word(WordToken {
                word,
                state: word_state,
                word_type,
                page_number: self.page_number,
                line_number: self.line_number,
                token_order: self.token_order_in_line,
                position: self.curr_position,
            })
        );
        self.last_is_space = false;
        self.token_order_in_line += 1;
    }

    pub fn add_tag(&mut self, tag: &'a str, label: &'a str) {
        self.tokens.push(
            Token::Tag(TagToken {
                tag,
                label,
                page_number: self.page_number,
                line_number: self.line_number,
                token_order: self.token_order_in_line,
                position: self.curr_position,
            })
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
