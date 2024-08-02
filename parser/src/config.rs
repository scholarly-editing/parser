pub mod deserializer;

pub struct ParserConfig {
    pub page: PageConfig,
    pub block: Vec<BlockConfig>,
    pub word: Vec<WordConfig>,
    pub prefix: Vec<PrefixConfig>,
    pub brackets: Vec<BracketsConfig>,
    pub sequence: Vec<SequenceConfig>,
}

pub struct PageConfig {
    pub prefix: Vec<String>,
    pub suffix: Vec<String>,
}

pub enum BlockType {
    Standalone,
    WithText,
}

pub struct BlockConfig {
    pub name: String,
    pub has_text: Option<bool>,
    pub block_type: BlockType,
    pub end_marker: Option<String>,
}

pub type Chars = ((u32, u32), Vec<u32>);

pub struct WordConfig {
    pub label: String,
    pub chars: Chars,
}

pub struct PrefixConfig {
    pub symbol: String,
    pub label: String,
}

pub struct BracketsConfig {
    pub open: String,
    pub close: String,
    pub label: String,
    pub open_label: String,
    pub close_label: String,
    pub skip: Option<bool>,
}

pub struct SequenceConfig {
    pub symbol: String,
    pub label: String,
}

impl From<deserializer::ParserConfigDeserializer> for ParserConfig {
    fn from(deserializer: deserializer::ParserConfigDeserializer) -> Self {
        Self {
            page: PageConfig {
                prefix: deserializer.page.prefix,
                suffix: deserializer.page.suffix,
            },
            block: deserializer.block
                .into_iter()
                .map(|b| b.into())
                .collect(),
            word: deserializer.word
                .into_iter()
                .map(|w| w.into())
                .collect(),
            prefix: deserializer.prefix
                .into_iter()
                .map(|p| p.into())
                .collect(),
            brackets: deserializer.brackets
                .into_iter()
                .map(|b| b.into())
                .collect(),
            sequence: deserializer.sequence
                .into_iter()
                .map(|s| s.into())
                .collect(),
        }
    }
}

impl From<deserializer::PageConfigDeserializer> for PageConfig {
    fn from(deserializer: deserializer::PageConfigDeserializer) -> Self {
        Self {
            prefix: deserializer.prefix,
            suffix: deserializer.suffix,
        }
    }
}

impl From<deserializer::BlockConfigDeserializer> for BlockConfig {
    fn from(deserializer: deserializer::BlockConfigDeserializer) -> Self {
        Self {
            name: deserializer.name,
            has_text: deserializer.has_text,
            block_type: match deserializer.block_type {
                deserializer::BlockTypeDeserializer::Standalone => BlockType::Standalone,
                deserializer::BlockTypeDeserializer::WithText => BlockType::WithText,
            },
            end_marker: deserializer.end_marker,
        }
    }
}

impl From<deserializer::WordConfigDeserializer> for WordConfig {
    fn from(deserializer: deserializer::WordConfigDeserializer) -> Self {
        let range = &deserializer.char_range;
        let first = *range.first().unwrap();
        let last = *range.last().unwrap();
        Self {
            label: deserializer.label,
            chars: ((first, last), deserializer.additional_chars),
        }
    }
}

impl From<deserializer::PrefixConfigDeserializer> for PrefixConfig {
    fn from(deserializer: deserializer::PrefixConfigDeserializer) -> Self {
        Self {
            symbol: deserializer.symbol,
            label: deserializer.label,
        }
    }
}

impl From<deserializer::BracketsConfigDeserializer> for BracketsConfig {
    fn from(deserializer: deserializer::BracketsConfigDeserializer) -> Self {
        Self {
            open: deserializer.open,
            close: deserializer.close,
            open_label: format!("{}_open", &deserializer.label),
            close_label: format!("{}_close", &deserializer.label),
            label: deserializer.label,
            skip: deserializer.skip,
        }
    }
}

impl From<deserializer::SequenceConfigDeserializer> for SequenceConfig {
    fn from(deserializer: deserializer::SequenceConfigDeserializer) -> Self {
        Self {
            symbol: deserializer.symbol,
            label: deserializer.label,
        }
    }
}
