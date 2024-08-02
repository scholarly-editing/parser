use std::{ error::Error, fs::File, io::Read };

use serde::Deserialize;
#[derive(Deserialize)]
pub struct ParserConfig {
    pub main: MainConfig,
    pub prefix: Vec<PrefixConfig>,
    pub brackets: Vec<BracketsConfig>,
    pub sequence: Vec<SequenceConfig>,
}

#[derive(Deserialize)]
pub struct MainConfig {
    pub _language: String,
    pub char_range: Vec<u32>,
    pub additional_chars: Vec<u32>,
    pub _page_marker_pattern: String,
}

#[derive(Deserialize)]
pub struct PrefixConfig {
    pub symbol: String,
    pub label: String,
}

#[derive(Deserialize)]
pub struct BracketsConfig {
    pub open: String,
    pub close: String,
    pub label: String,
    pub open_label: String,
    pub close_label: String,
}

#[derive(Deserialize)]
pub struct SequenceConfig {
    pub symbol: String,
    pub label: String,
}

impl ParserConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_str(&contents)
    }

    pub fn from_str(contents: &str) -> Result<Self, Box<dyn Error>> {
        let config: ParserConfig = toml::from_str(contents)?;
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
            [main]
            language = "Arabic"
            char_range = [600, 0x60FF]
            additional_chars = []
            page_marker_pattern = 'fol\.\d+[rv]'

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

            [[brackets]]
            open = "[["
            close = "]]"
            label = "cross-out"

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

        let config = ParserConfig::from_str(test_toml_content)?;

        assert_eq!(config.main._language, "Arabic");
        assert_eq!(config.main.char_range, vec![600, 0x60ff]);
        assert_eq!(config.main.additional_chars, Vec::<u32>::new());
        assert_eq!(config.main._page_marker_pattern, "fol\\.\\d+[rv]");
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
        assert_eq!(config.brackets[2].open, "[[");
        assert_eq!(config.brackets[2].close, "]]");
        assert_eq!(config.brackets[2].label, "cross-out");
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
