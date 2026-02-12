use std::panic;

use config_deserializer::ParserConfigDeserializer;
use parser::{
    config::ParserConfig,
    core::{
        orchestrator::Orchestrator,
        state::{Mode, Token},
    },
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

#[wasm_bindgen]
pub fn tokenize(input: &str, config: &str) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let config: ParserConfig = ParserConfigDeserializer::from_toml_str(config)
        .unwrap()
        .into();
    let mode = Mode::SinglePage;
    let input = input.nfc().collect::<String>();
    let parser = Orchestrator::new(&config, &mode);
    let (tokens, errors) = parser.tokenize(&input);
    let res = Result { tokens, errors };
    serde_wasm_bindgen::to_value(&res).unwrap()
}

#[wasm_bindgen]
pub struct Passage(PassageState);

#[wasm_bindgen]
pub struct Location(pub isize, pub isize);

#[wasm_bindgen]
impl Passage {
    pub fn new(input: &str, config: &str) -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let config: ParserConfig = ParserConfigDeserializer::from_toml_str(config)
            .unwrap()
            .into();
        Self(PassageState::new(input, config))
    }

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
