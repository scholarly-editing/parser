pub const DEFAULT_PRECEDENCE: usize = usize::MAX;

pub struct ParserConfig {
    pub page: Option<Vec<PageConfig>>,
    pub block: Vec<BlockConfig>,
    pub word: Vec<WordConfig>,
    pub prefix: Vec<PrefixConfig>,
    pub suffix: Vec<SuffixConfig>,
    pub brackets: Vec<BracketsConfig>,
    pub tags: Vec<TagConfig>,
}

impl ParserConfig {
    pub fn builder() -> ParserConfigBuilder {
        ParserConfigBuilder::default()
    }
}

pub struct PageConfig {
    pub prefix: String,
    pub suffix: String,
    pub precedence: usize,
}

pub struct SuffixConfig {
    pub symbol: String,
    pub label: String,
    pub precedence: usize,
}

#[derive(Debug, PartialEq)]
pub enum BlockType {
    Standalone,
    WithText,
}

pub struct BlockConfig {
    pub start_marker: String,
    pub has_text: bool,
    pub block_type: BlockType,
    pub end_marker: Option<String>,
    pub is_inline: bool,
    pub precedence: usize,
}

pub type Chars = ((u32, u32), Vec<u32>);

pub struct WordConfig {
    pub label: String,
    pub chars: Chars,
    pub default_state: String,
    pub precedence: usize,
}

pub struct PrefixConfig {
    pub symbol: String,
    pub label: String,
    pub precedence: usize,
}

pub struct BracketsConfig {
    pub open: String,
    pub close: String,
    pub label: String,
    pub skip: Option<bool>,
    pub precedence: usize,
}

pub struct TagConfig {
    pub symbol: String,
    pub label: String,
    pub precedence: usize,
}

pub struct ParserConfigBuilder {
    page: Vec<PageConfig>,
    block: Vec<BlockConfig>,
    word: Vec<WordConfig>,
    prefix: Vec<PrefixConfig>,
    suffix: Vec<SuffixConfig>,
    brackets: Vec<BracketsConfig>,
    tag: Vec<TagConfig>,
}

impl ParserConfigBuilder {
    pub fn new() -> Self {
        Self {
            page: Vec::new(),
            block: Vec::new(),
            word: Vec::new(),
            suffix: Vec::new(),
            prefix: Vec::new(),
            brackets: Vec::new(),
            tag: Vec::new(),
        }
    }

    pub fn add_page(mut self, page: PageConfig) -> Self {
        self.page.push(page);
        self
    }

    pub fn add_block(mut self, block: BlockConfig) -> Self {
        self.block.push(block);
        self
    }

    pub fn add_word(mut self, word: WordConfig) -> Self {
        self.word.push(word);
        self
    }

    pub fn add_prefix(mut self, prefix: PrefixConfig) -> Self {
        self.prefix.push(prefix);
        self
    }

    pub fn add_suffix(mut self, suffix: SuffixConfig) -> Self {
        self.suffix.push(suffix);
        self
    }

    pub fn add_brackets(mut self, brackets: BracketsConfig) -> Self {
        self.brackets.push(brackets);
        self
    }

    pub fn add_tag(mut self, tag: TagConfig) -> Self {
        self.tag.push(tag);
        self
    }

    pub fn build(self) -> Result<ParserConfig, &'static str> {
        if self.word.is_empty() {
            return Err("Word vector cannot be empty");
        }

        Ok(ParserConfig {
            page: Some(self.page),
            block: self.block,
            word: self.word,
            prefix: self.prefix,
            brackets: self.brackets,
            tags: self.tag,
            suffix: self.suffix,
        })
    }
}

impl Default for ParserConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PageConfig {
    pub fn builder() -> PageConfigBuilder {
        PageConfigBuilder::default()
    }
}

pub struct PageConfigBuilder {
    prefix: String,
    suffix: String,
    precedence: usize,
}

impl PageConfigBuilder {
    pub fn new() -> Self {
        Self {
            prefix: String::new(),
            suffix: String::new(),
            precedence: DEFAULT_PRECEDENCE,
        }
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn suffix(mut self, suffix: &str) -> Self {
        self.suffix = suffix.to_string();
        self
    }

    pub fn precedence(mut self, precedence: usize) -> Self {
        self.precedence = precedence;
        self
    }

    pub fn build(self) -> PageConfig {
        PageConfig {
            prefix: self.prefix,
            suffix: self.suffix,
            precedence: self.precedence,
        }
    }
}

impl Default for PageConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockConfig {
    pub fn builder() -> BlockConfigBuilder {
        BlockConfigBuilder::default()
    }
}

pub struct BlockConfigBuilder {
    start_marker: String,
    has_text: bool,
    is_inline: bool,
    block_type: BlockType,
    end_marker: Option<String>,
    precedence: usize,
}

impl BlockConfigBuilder {
    pub fn new() -> Self {
        Self {
            start_marker: String::new(),
            has_text: false,
            is_inline: false,
            block_type: BlockType::Standalone,
            end_marker: None,
            precedence: DEFAULT_PRECEDENCE,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.start_marker = name.to_string();
        self
    }

    pub fn has_text(mut self, has_text: bool) -> Self {
        self.has_text = has_text;
        self
    }

    pub fn is_inline(mut self, is_inline: bool) -> Self {
        self.is_inline = is_inline;
        self
    }

    pub fn block_type(mut self, block_type: BlockType) -> Self {
        self.block_type = block_type;
        self
    }

    pub fn end_marker(mut self, end_marker: &str) -> Self {
        self.end_marker = Some(end_marker.to_string());
        self
    }

    pub fn precedence(mut self, precedence: usize) -> Self {
        self.precedence = precedence;
        self
    }

    pub fn build(self) -> BlockConfig {
        BlockConfig {
            start_marker: self.start_marker,
            has_text: self.has_text,
            block_type: self.block_type,
            end_marker: self.end_marker,
            precedence: self.precedence,
            is_inline: self.is_inline,
        }
    }
}

impl Default for BlockConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WordConfig {
    pub fn builder() -> WordConfigBuilder {
        WordConfigBuilder::default()
    }
}

pub struct WordConfigBuilder {
    label: String,
    char_range: Vec<u32>,
    additional_chars: Vec<u32>,
    default_state: String,
    precedence: usize,
}

impl WordConfigBuilder {
    pub fn new() -> Self {
        Self {
            label: String::new(),
            char_range: Vec::new(),
            additional_chars: Vec::new(),
            default_state: String::new(),
            precedence: DEFAULT_PRECEDENCE,
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn char_range(mut self, char_range: Vec<u32>) -> Self {
        self.char_range = char_range;
        self
    }

    pub fn additional_chars(mut self, additional_chars: Vec<u32>) -> Self {
        self.additional_chars = additional_chars;
        self
    }

    pub fn default_state(mut self, default_state: &str) -> Self {
        self.default_state = default_state.to_string();
        self
    }

    pub fn precedence(mut self, precedence: usize) -> Self {
        self.precedence = precedence;
        self
    }

    pub fn build(self) -> WordConfig {
        WordConfig {
            label: self.label,
            chars: (
                (self.char_range[0], self.char_range[1]),
                self.additional_chars,
            ),
            default_state: self.default_state,
            precedence: self.precedence,
        }
    }
}

impl Default for WordConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PrefixConfig {
    pub fn builder() -> PrefixConfigBuilder {
        PrefixConfigBuilder::default()
    }
}

pub struct PrefixConfigBuilder {
    symbol: String,
    label: String,
    precedence: usize,
}

impl PrefixConfigBuilder {
    pub fn new() -> Self {
        Self {
            symbol: String::new(),
            label: String::new(),
            precedence: DEFAULT_PRECEDENCE,
        }
    }

    pub fn symbol(mut self, symbol: &str) -> Self {
        self.symbol = symbol.to_string();
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn precedence(mut self, precedence: usize) -> Self {
        self.precedence = precedence;
        self
    }

    pub fn build(self) -> PrefixConfig {
        PrefixConfig {
            symbol: self.symbol,
            label: self.label,
            precedence: self.precedence,
        }
    }
}

impl Default for PrefixConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SuffixConfigBuilder {
    symbol: String,
    label: String,
    precedence: usize,
}

impl SuffixConfigBuilder {
    pub fn new() -> Self {
        Self {
            symbol: String::new(),
            label: String::new(),
            precedence: DEFAULT_PRECEDENCE,
        }
    }

    pub fn symbol(mut self, symbol: &str) -> Self {
        self.symbol = symbol.to_string();
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn precedence(mut self, precedence: usize) -> Self {
        self.precedence = precedence;
        self
    }

    pub fn build(self) -> SuffixConfig {
        SuffixConfig {
            symbol: self.symbol,
            label: self.label,
            precedence: self.precedence,
        }
    }
}

impl Default for SuffixConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BracketsConfig {
    pub fn builder() -> BracketsConfigBuilder {
        BracketsConfigBuilder::default()
    }
}

pub struct BracketsConfigBuilder {
    open: String,
    close: String,
    label: String,
    skip: Option<bool>,
    precedence: usize,
}

impl BracketsConfigBuilder {
    pub fn new() -> Self {
        Self {
            open: String::new(),
            close: String::new(),
            label: String::new(),
            skip: None,
            precedence: DEFAULT_PRECEDENCE,
        }
    }

    pub fn open(mut self, open: &str) -> Self {
        self.open = open.to_string();
        self
    }

    pub fn close(mut self, close: &str) -> Self {
        self.close = close.to_string();
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn skip(mut self, skip: bool) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn precedence(mut self, precedence: usize) -> Self {
        self.precedence = precedence;
        self
    }

    pub fn build(self) -> BracketsConfig {
        BracketsConfig {
            open: self.open,
            close: self.close,
            label: self.label,
            skip: self.skip,
            precedence: self.precedence,
        }
    }
}

impl Default for BracketsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TagConfig {
    pub fn builder() -> TagConfigBuilder {
        TagConfigBuilder::default()
    }
}

pub struct TagConfigBuilder {
    symbol: String,
    label: String,
    precedence: usize,
}

impl TagConfigBuilder {
    pub fn new() -> Self {
        Self {
            symbol: String::new(),
            label: String::new(),
            precedence: DEFAULT_PRECEDENCE,
        }
    }

    pub fn symbol(mut self, symbol: &str) -> Self {
        self.symbol = symbol.to_string();
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn precedence(mut self, precedence: usize) -> Self {
        self.precedence = precedence;
        self
    }

    pub fn build(self) -> TagConfig {
        TagConfig {
            symbol: self.symbol,
            label: self.label,
            precedence: self.precedence,
        }
    }
}

impl Default for TagConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
