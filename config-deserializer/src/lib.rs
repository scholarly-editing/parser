use std::{ error::Error, fs::File, io::Read };

use parser::config::{
    BlockConfig,
    BlockType,
    BracketsConfig,
    PageConfig,
    ParserConfig,
    PrefixConfig,
    SuffixConfig,
    TagConfig,
    WordConfig,
    DEFAULT_PRECEDENCE,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ParserConfigDeserializer {
    pub page: Option<Vec<PageConfigDeserializer>>,
    pub block: Option<Vec<BlockConfigDeserializer>>,
    pub word: Vec<WordConfigDeserializer>,
    pub prefix: Vec<PrefixConfigDeserializer>,
    pub suffix: Option<Vec<SuffixConfigDeserializer>>,
    pub brackets: Vec<BracketsConfigDeserializer>,
    pub tag: Vec<TagConfigDeserializer>,
}

#[derive(Deserialize)]
pub struct PageConfigDeserializer {
    pub prefix: String,
    pub suffix: String,
    pub precedence: Option<usize>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum BlockTypeDeserializer {
    #[serde(rename = "STANDALONE")]
    Standalone,
    #[serde(rename = "WITH_TEXT")]
    WithText,
}

#[derive(Deserialize)]
pub struct BlockConfigDeserializer {
    pub start_marker: String,
    pub has_text: Option<bool>,
    pub is_inline: Option<bool>,
    #[serde(rename = "type")]
    pub block_type: BlockTypeDeserializer,
    pub end_marker: Option<String>,
    pub precedence: Option<usize>,
}

#[derive(Deserialize)]
pub struct WordConfigDeserializer {
    pub label: String,
    pub char_range: Vec<u32>,
    pub additional_chars: Vec<u32>,
    pub default_state: String,
    pub precedence: Option<usize>,
}

#[derive(Deserialize)]
pub struct PrefixConfigDeserializer {
    pub symbol: String,
    pub label: String,
    pub precedence: Option<usize>,
}

#[derive(Deserialize)]
pub struct SuffixConfigDeserializer {
    pub symbol: String,
    pub label: String,
    pub precedence: Option<usize>,
}

#[derive(Deserialize)]
pub struct BracketsConfigDeserializer {
    pub open: String,
    pub close: String,
    pub label: String,
    pub skip: Option<bool>,
    pub precedence: Option<usize>,
}

#[derive(Deserialize)]
pub struct TagConfigDeserializer {
    pub symbol: String,
    pub label: String,
    pub precedence: Option<usize>,
}

impl ParserConfigDeserializer {
    pub fn from_toml_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_toml_str(&contents)
    }

    pub fn from_toml_str(contents: &str) -> Result<Self, Box<dyn Error>> {
        let config: ParserConfigDeserializer = toml::from_str(contents)?;
        Ok(config)
    }
}

impl From<ParserConfigDeserializer> for ParserConfig {
    fn from(deserializer: ParserConfigDeserializer) -> Self {
        Self {
            page: deserializer.page.map(|p| {
                let mut pages: Vec<PageConfig> = p
                    .into_iter()
                    .map(|p| p.into())
                    .collect();
                pages.sort_by_key(|p| p.precedence);
                pages
            }),
            block: {
                deserializer.block
                    .map(|blocks| {
                        let mut blocks: Vec<BlockConfig> = blocks
                            .into_iter()
                            .map(|b| b.into())
                            .collect();
                        blocks.sort_by_key(|b| b.precedence);
                        blocks
                    })
                    .unwrap_or_default()
            },
            word: {
                let mut words: Vec<WordConfig> = deserializer.word
                    .into_iter()
                    .map(|w| w.into())
                    .collect();
                words.sort_by_key(|w| w.precedence);
                words
            },
            prefix: {
                let mut prefixes: Vec<PrefixConfig> = deserializer.prefix
                    .into_iter()
                    .map(|p| p.into())
                    .collect();
                prefixes.sort_by_key(|p| p.precedence);
                prefixes
            },
            suffix: {
                deserializer.suffix
                    .map(|suffixes| {
                        let mut suffixes: Vec<SuffixConfig> = suffixes
                            .into_iter()
                            .map(|s| s.into())
                            .collect();
                        suffixes.sort_by_key(|s| s.precedence);
                        suffixes
                    })
                    .unwrap_or_default()
            },
            brackets: {
                let mut brackets: Vec<BracketsConfig> = deserializer.brackets
                    .into_iter()
                    .map(|b| b.into())
                    .collect();
                brackets.sort_by_key(|b| b.precedence);
                brackets
            },
            tags: {
                let mut tags: Vec<TagConfig> = deserializer.tag
                    .into_iter()
                    .map(|s| s.into())
                    .collect();
                tags.sort_by_key(|s| s.precedence);
                tags
            },
        }
    }
}

impl From<PageConfigDeserializer> for PageConfig {
    fn from(deserializer: PageConfigDeserializer) -> Self {
        Self {
            prefix: deserializer.prefix,
            suffix: deserializer.suffix,
            precedence: deserializer.precedence.unwrap_or(DEFAULT_PRECEDENCE),
        }
    }
}

impl From<BlockConfigDeserializer> for BlockConfig {
    fn from(deserializer: BlockConfigDeserializer) -> Self {
        Self {
            start_marker: deserializer.start_marker,
            has_text: deserializer.has_text.unwrap_or(false),
            block_type: match deserializer.block_type {
                BlockTypeStandalone => BlockType::Standalone,
                BlockTypeWithText => BlockType::WithText,
            },
            is_inline: deserializer.is_inline.unwrap_or(false),
            end_marker: deserializer.end_marker,
            precedence: deserializer.precedence.unwrap_or(DEFAULT_PRECEDENCE),
        }
    }
}

impl From<WordConfigDeserializer> for WordConfig {
    fn from(deserializer: WordConfigDeserializer) -> Self {
        let range = &deserializer.char_range;
        let first = *range.first().unwrap();
        let last = *range.last().unwrap();
        Self {
            label: deserializer.label,
            chars: ((first, last), deserializer.additional_chars),
            default_state: deserializer.default_state,
            precedence: deserializer.precedence.unwrap_or(DEFAULT_PRECEDENCE),
        }
    }
}

impl From<PrefixConfigDeserializer> for PrefixConfig {
    fn from(deserializer: PrefixConfigDeserializer) -> Self {
        Self {
            symbol: deserializer.symbol,
            label: deserializer.label,
            precedence: deserializer.precedence.unwrap_or(DEFAULT_PRECEDENCE),
        }
    }
}

impl From<SuffixConfigDeserializer> for SuffixConfig {
    fn from(deserializer: SuffixConfigDeserializer) -> Self {
        Self {
            symbol: deserializer.symbol,
            label: deserializer.label,
            precedence: deserializer.precedence.unwrap_or(DEFAULT_PRECEDENCE),
        }
    }
}

impl From<BracketsConfigDeserializer> for BracketsConfig {
    fn from(deserializer: BracketsConfigDeserializer) -> Self {
        Self {
            open: deserializer.open,
            close: deserializer.close,
            label: deserializer.label,
            skip: deserializer.skip,
            precedence: deserializer.precedence.unwrap_or(DEFAULT_PRECEDENCE),
        }
    }
}

impl From<TagConfigDeserializer> for TagConfig {
    fn from(deserializer: TagConfigDeserializer) -> Self {
        Self {
            symbol: deserializer.symbol,
            label: deserializer.label,
            precedence: deserializer.precedence.unwrap_or(DEFAULT_PRECEDENCE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_from_file() -> Result<(), Box<dyn Error>> {
        let test_toml_content =
            r#"
            [[page]]
prefix = "fol."
suffix = "r"

[[page]]
prefix = "fol."
suffix = "v"

[[block]]
start_marker = "[illustration]"
type = "STANDALONE"

[[block]]
start_marker = "[legend]"
has_text = true
type = "WITH_TEXT"
end_marker = "---"

[[block]]
start_marker = "margin"
has_text = true
type = "WITH_TEXT"
end_marker = "---"

[[word]]
label = "Arabic"
char_range = [600, 0x60FF]
additional_chars = []
default_state = "sound"


[[suffix]]
symbol = "~"
label = "middle-arabic"

[[prefix]]
symbol = "*"
label = "emendation"

[[prefix]]
symbol = "?"
label = "unintelligible"

[[prefix]]
symbol = "؟"
label = "unintelligible"

[[prefix]]
symbol = "!"
label = "error"

[[prefix]]
symbol = "†"
label = "corrupt"

[[brackets]]
open = "("
close = ")"
label = "title"


[[brackets]]
open = "["
close = "]"
label = "superfluous"
skip = true

[[brackets]]
precedence = 0
open = "[["
close = "]]"
label = "cross-out"
skip = true


[[brackets]]
open = "{"
close = "}"
label = "suppletion"


[[brackets]]
open = "<"
close = ">"
label = "added"


[[tag]]
symbol = "***"
label = "lacuna"


[[tag]]
symbol = "..."
label = "damage"


        "#;

        let config = ParserConfigDeserializer::from_toml_str(test_toml_content).unwrap();

        assert_eq!(config.page.as_ref().unwrap()[0].prefix, "fol.".to_string());
        assert_eq!(config.page.as_ref().unwrap()[0].suffix, "r".to_string());
        assert_eq!(config.page.as_ref().unwrap()[1].prefix, "fol.".to_string());
        assert_eq!(config.page.as_ref().unwrap()[1].suffix, "v".to_string());
        // assert_eq!(config.block.len(), 3);
        // assert_eq!(config.block[0].start_marker, "[illustration]");
        // assert_eq!(config.block[0].block_type, BlockTypeDeserializer::Standalone);
        // assert_eq!(config.block[1].start_marker, "[legend]");
        // assert_eq!(config.block[1].has_text, Some(true));
        // assert_eq!(config.block[1].block_type, BlockTypeDeserializer::WithText);
        // assert_eq!(config.block[1].end_marker, Some("---".to_string()));
        // assert_eq!(config.block[2].start_marker, "margin");
        // assert_eq!(config.block[2].has_text, Some(true));
        // assert_eq!(config.block[2].block_type, BlockTypeDeserializer::WithText);
        // assert_eq!(config.block[2].end_marker, Some("---".to_string()));
        assert_eq!(config.word.len(), 1);
        assert_eq!(config.word[0].label, "Arabic");
        assert_eq!(config.word[0].char_range, vec![600, 0x60ff]);
        assert_eq!(config.word[0].additional_chars, Vec::<u32>::new());
        assert_eq!(config.prefix.len(), 5);
        assert_eq!(config.prefix[0].symbol, "*");
        assert_eq!(config.prefix[0].label, "emendation");
        assert_eq!(config.prefix[1].symbol, "?");
        assert_eq!(config.prefix[1].label, "unintelligible");
        assert_eq!(config.prefix[2].symbol, "؟");
        assert_eq!(config.prefix[2].label, "unintelligible");
        assert_eq!(config.prefix[3].symbol, "!");
        assert_eq!(config.prefix[3].label, "error");
        assert_eq!(config.prefix[4].symbol, "†");
        assert_eq!(config.prefix[4].label, "corrupt");
        // assert_eq!(config.suffix.len(), 1);
        // assert_eq!(config.suffix[0].symbol, "~");
        // assert_eq!(config.suffix[0].label, "middle-arabic");
        assert_eq!(config.brackets.len(), 5);
        assert_eq!(config.brackets[0].open, "(");
        assert_eq!(config.brackets[0].close, ")");
        assert_eq!(config.brackets[0].label, "title");
        assert_eq!(config.brackets[1].open, "[");
        assert_eq!(config.brackets[1].close, "]");
        assert_eq!(config.brackets[1].label, "superfluous");
        assert_eq!(config.brackets[1].skip, Some(true));
        assert_eq!(config.brackets[2].open, "[[");
        assert_eq!(config.brackets[2].close, "]]");
        assert_eq!(config.brackets[2].label, "cross-out");
        assert_eq!(config.brackets[2].skip, Some(true));
        assert_eq!(config.brackets[3].open, "{");
        assert_eq!(config.brackets[3].close, "}");
        assert_eq!(config.brackets[3].label, "suppletion");
        assert_eq!(config.brackets[4].open, "<");
        assert_eq!(config.brackets[4].close, ">");
        assert_eq!(config.brackets[4].label, "added");
        assert_eq!(config.tag.len(), 2);
        assert_eq!(config.tag[0].symbol, "***");
        assert_eq!(config.tag[0].label, "lacuna");
        assert_eq!(config.tag[1].symbol, "...");
        assert_eq!(config.tag[1].label, "damage");

        Ok(())
    }
}
