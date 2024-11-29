use serde::{ Deserialize, Serialize };

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WordToken<'a> {
    pub word: &'a str,
    pub state: &'a str,
    pub word_type: &'a str,
    pub page_number: usize,
    pub line_number: usize,
    pub token_order: usize,
    pub span: (usize, usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagToken<'a> {
    pub tag: &'a str,
    pub label: &'a str,
    pub page_number: usize,
    pub line_number: usize,
    pub token_order: usize,
    pub span: (usize, usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageBreakToken<'a> {
    pub page_representation: &'a str,
    pub page_number: usize,
    pub span: (usize, usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockToken<'a> {
    pub block_name: &'a str,
    pub page_number: usize,
    pub span: (usize, usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WordInBlockToken<'a> {
    pub word: &'a str,
    pub state: &'a str,
    pub block_name: &'a str,
    pub line_number: usize,
    pub token_order: usize,
    pub span: (usize, usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenizerError<'a> {
    pub token: &'a str,
    pub line: usize,
    pub page: usize,
    pub order_in_line: usize,
    pub page_repr: &'a str,
    pub span: (usize, usize),
    pub message: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub enum Token<'a> {
    #[serde(borrow)] PageBreak(PageBreakToken<'a>),
    Word(WordToken<'a>),
    Tag(TagToken<'a>),
    Block(BlockToken<'a>),
    WordInBlock(WordInBlockToken<'a>),
    Error(TokenizerError<'a>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockState<'a> {
    name: &'a str,
    token_order_in_line: usize,
    line_number: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenizerState<'a> {
    pub tokens: Vec<Token<'a>>,
    pub errors: Vec<usize>,
    pub remaining: &'a str,
    pub token_order_in_line: usize,
    pub line_number: usize,
    pub page_number: usize,
    pub current_page_repr: &'a str,
    pub parsed_len: usize,
    pub last_is_space: bool,
    pub current_block: Option<BlockState<'a>>,
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
            parsed_len: 0,
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
}
