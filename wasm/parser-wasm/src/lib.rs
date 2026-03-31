use std::panic;

use config_deserializer::ParserConfigDeserializer;
use parser::{
    config::ParserConfig,
    core::{
        orchestrator::Orchestrator,
        state::{Mode, Token},
    },
    document::Document,
    passage_state::PassageState,
};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Serialize)]
struct Result<'a> {
    #[serde(borrow)]
    tokens: Vec<Token<'a>>,
    errors: Vec<usize>,
}

#[derive(Serialize)]
struct DocumentResult<'a> {
    pages: Vec<Vec<Token<'a>>>,
    errors: Vec<usize>,
}

fn parse_mode(mode: Option<String>) -> std::result::Result<Mode, JsError> {
    match mode.as_deref() {
        None | Some("single_page") => Ok(Mode::SinglePage),
        Some("multiple_pages") => Ok(Mode::MultiplePages),
        Some("passage") => Ok(Mode::Passage),
        Some(other) => Err(JsError::new(&format!(
            "Invalid mode: \"{other}\". Expected \"single_page\", \"multiple_pages\", or \"passage\"."
        ))),
    }
}

fn parse_config(
    config: &str,
    config_format: Option<String>,
) -> std::result::Result<ParserConfig, JsError> {
    let deserializer: ParserConfigDeserializer = match config_format.as_deref() {
        None | Some("toml") => toml::from_str(config)
            .map_err(|e| JsError::new(&format!("TOML config error: {e}")))?,
        Some("json") => serde_json::from_str(config)
            .map_err(|e| JsError::new(&format!("JSON config error: {e}")))?,
        Some("yaml") => serde_yaml::from_str(config)
            .map_err(|e| JsError::new(&format!("YAML config error: {e}")))?,
        Some("xml") => serde_xml_rs::from_str(config)
            .map_err(|e| JsError::new(&format!("XML config error: {e}")))?,
        Some(other) => {
            return Err(JsError::new(&format!(
                "Invalid config_format: \"{other}\". Expected \"toml\", \"json\", \"yaml\", or \"xml\"."
            )));
        }
    };
    Ok(deserializer.into())
}

#[wasm_bindgen]
pub fn tokenize(
    input: &str,
    config: &str,
    mode: Option<String>,
    config_format: Option<String>,
) -> std::result::Result<JsValue, JsError> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let config = parse_config(config, config_format)?;
    let mode = parse_mode(mode)?;
    let input = input.nfc().collect::<String>();
    let parser = Orchestrator::new(&config, &mode);
    let (tokens, errors) = parser.tokenize(&input);
    let res = Result { tokens, errors };
    serde_wasm_bindgen::to_value(&res)
        .map_err(|e| JsError::new(&format!("Serialization error: {e}")))
}

/// Like `tokenize`, but groups tokens by page using `Document::new()`.
///
/// Returns `{ pages: Token[][], errors: number[] }` where each element of
/// `pages` is the token array for one page (split on `PageBreak` tokens).
///
/// **Note:** `errors` contains indices into the original *flat* token list,
/// not per-page indices.
#[wasm_bindgen]
pub fn tokenize_document(
    input: &str,
    config: &str,
    mode: Option<String>,
    config_format: Option<String>,
) -> std::result::Result<JsValue, JsError> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let config = parse_config(config, config_format)?;
    let mode = parse_mode(mode)?;
    let input = input.nfc().collect::<String>();
    let parser = Orchestrator::new(&config, &mode);
    let (tokens, errors) = parser.tokenize(&input);
    let document = Document::new(tokens);
    let res = DocumentResult {
        pages: document.pages,
        errors,
    };
    serde_wasm_bindgen::to_value(&res)
        .map_err(|e| JsError::new(&format!("Serialization error: {e}")))
}

#[wasm_bindgen]
pub struct Passage(PassageState);

#[wasm_bindgen]
pub struct Location(pub isize, pub isize);

/// Factory function to create a `Passage`. Returns a JS error on invalid config
/// instead of panicking.
///
/// `config_format` defaults to `"toml"` when omitted.
#[wasm_bindgen(js_name = "createPassage")]
pub fn create_passage(
    input: &str,
    config: &str,
    config_format: Option<String>,
) -> std::result::Result<Passage, JsError> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let config = parse_config(config, config_format)?;
    Ok(Passage(PassageState::new(input, config)))
}

#[wasm_bindgen]
impl Passage {
    pub fn locate_fragment(&self, fragment: &str) -> Location {
        self.0
            .locate_fragment(fragment)
            .map(|(start, end)| Location(start as isize, end as isize))
            .unwrap_or(Location(-1, -1))
    }

    pub fn get_raw_passage(&self) -> String {
        self.0.get_raw_passage()
    }

    pub fn is_valid(&self) -> bool {
        self.0.valid
    }
}
