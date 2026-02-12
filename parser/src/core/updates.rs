use super::errors::TextError;

#[derive(Debug)]
pub struct PageBreakPayload<'a> {
    pub rest: Option<&'a str>,
    pub pre_len: usize,
    pub len: usize,
    pub post_len: usize,
    pub repr: &'a str,
}

#[derive(Debug)]
pub struct LineBreakPayload<'a> {
    pub rest: Option<&'a str>,
    pub len: usize,
}

#[derive(Debug)]
pub struct PassageLineBreakPayload<'a> {
    pub rest: Option<&'a str>,
    pub len: usize,
}

#[derive(Debug)]
pub struct SpacePayload<'a> {
    pub rest: Option<&'a str>,
    pub len: usize,
}

#[derive(Debug)]
pub struct WordPayload<'a> {
    pub word: &'a str,
    pub word_state: &'a str,
    pub word_type: &'a str,
    pub rest: Option<&'a str>,
    pub pre_len: usize,
    pub len: usize,
    pub post_len: usize,
}

#[derive(Debug)]
pub struct TagPayload<'a> {
    pub tag: &'a str,
    pub label: &'a str,
    pub rest: Option<&'a str>,
    pub pre_len: usize,
    pub len: usize,
    pub post_len: usize,
}

#[derive(Debug)]
pub struct ErrorPayload<'a> {
    pub text: &'a str,
    pub error: TextError,
    pub rest: Option<&'a str>,
    pub len: usize,
    // add a `can_add_word_after` prop
}

#[derive(Debug)]
pub struct BlockPayload<'a> {
    pub block_name: &'a str,
    pub block_type: &'a str,
    pub has_end_marker: bool,
    pub end_marker: Option<&'a str>,
    pub rest: Option<&'a str>,
    pub len: usize,
}

#[derive(Debug)]
pub struct BlockEndPayload<'a> {
    pub rest: Option<&'a str>,
    pub len: usize,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TokenizerUpdate<'a> {
    AddPageBreak(PageBreakPayload<'a>),
    AddLineBreak(LineBreakPayload<'a>),
    AddPassageLineBreak(PassageLineBreakPayload<'a>),
    AddSpace(SpacePayload<'a>),
    AddWord(WordPayload<'a>),
    AddTag(TagPayload<'a>),
    AddError(ErrorPayload<'a>),
    AddBlockStart(BlockPayload<'a>),
    AddBlockEnd(BlockEndPayload<'a>),
}

impl<'a> TokenizerUpdate<'a> {
    pub fn get_rest(&self) -> Option<&'a str> {
        match self {
            TokenizerUpdate::AddPageBreak(payload) => payload.rest,
            TokenizerUpdate::AddLineBreak(payload) => payload.rest,
            TokenizerUpdate::AddPassageLineBreak(payload) => payload.rest,
            TokenizerUpdate::AddSpace(payload) => payload.rest,
            TokenizerUpdate::AddWord(payload) => payload.rest,
            TokenizerUpdate::AddTag(payload) => payload.rest,
            TokenizerUpdate::AddError(payload) => payload.rest,
            TokenizerUpdate::AddBlockStart(payload) => payload.rest,
            TokenizerUpdate::AddBlockEnd(payload) => payload.rest,
        }
    }
}
