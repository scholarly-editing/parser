use std::{ error::Error, fs::File, io::Read };

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ParserConfigDeserializer {
    pub page: PageConfigDeserializer,
    pub block: Vec<BlockConfigDeserializer>,
    pub word: Vec<WordConfigDeserializer>,
    pub prefix: Vec<PrefixConfigDeserializer>,
    pub brackets: Vec<BracketsConfigDeserializer>,
    pub sequence: Vec<SequenceConfigDeserializer>,
}

#[derive(Deserialize)]
pub struct PageConfigDeserializer {
    pub prefix: Vec<String>,
    pub suffix: Vec<String>,
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
    pub name: String,
    pub has_text: Option<bool>,
    #[serde(rename = "type")]
    pub block_type: BlockTypeDeserializer,
    pub end_marker: Option<String>,
}

#[derive(Deserialize)]
pub struct WordConfigDeserializer {
    pub label: String,
    pub char_range: Vec<u32>,
    pub additional_chars: Vec<u32>,
}

#[derive(Deserialize)]
pub struct PrefixConfigDeserializer {
    pub symbol: String,
    pub label: String,
}

#[derive(Deserialize)]
pub struct BracketsConfigDeserializer {
    pub open: String,
    pub close: String,
    pub label: String,
    pub skip: Option<bool>,
}

#[derive(Deserialize)]
pub struct SequenceConfigDeserializer {
    pub symbol: String,
    pub label: String,
}

impl ParserConfigDeserializer {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_str(&contents)
    }

    pub fn from_str(contents: &str) -> Result<Self, Box<dyn Error>> {
        let config: ParserConfigDeserializer = toml::from_str(contents)?;
        Ok(config)
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
            [page]
prefix = ["fol."]
suffix = ["r", "v"]

[[block]]
name = "illustration"
type = "STANDALONE"

[[block]]
name = "legend"
has_text = true
type = "WITH_TEXT"
end_marker = "---"

[[block]]
name = "margin"
has_text = true
type = "WITH_TEXT"
end_marker = "---"

[[word]]
label = "Arabic"
char_range = [600, 0x60FF]
additional_chars = []


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

[[sequence]]
symbol = "***"
label = "lacuna"


[[sequence]]
symbol = "..."
label = "damage"

        "#;

        let config = ParserConfigDeserializer::from_str(test_toml_content)?;

        assert_eq!(config.page.prefix, vec!["fol.".to_string()]);
        assert_eq!(config.page.suffix, vec!["r".to_string(), "v".to_string()]);
        assert_eq!(config.block.len(), 3);
        assert_eq!(config.block[0].name, "illustration");
        assert_eq!(config.block[0].block_type, BlockTypeDeserializer::Standalone);
        assert_eq!(config.block[1].name, "legend");
        assert_eq!(config.block[1].has_text, Some(true));
        assert_eq!(config.block[1].block_type, BlockTypeDeserializer::WithText);
        assert_eq!(config.block[1].end_marker, Some("---".to_string()));
        assert_eq!(config.block[2].name, "margin");
        assert_eq!(config.block[2].has_text, Some(true));
        assert_eq!(config.block[2].block_type, BlockTypeDeserializer::WithText);
        assert_eq!(config.block[2].end_marker, Some("---".to_string()));
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
        assert_eq!(config.sequence.len(), 2);
        assert_eq!(config.sequence[0].symbol, "***");
        assert_eq!(config.sequence[0].label, "lacuna");
        assert_eq!(config.sequence[1].symbol, "...");
        assert_eq!(config.sequence[1].label, "damage");

        Ok(())
    }
}
