use std::panic;

use config_deserializer::ParserConfigDeserializer;
use parser::{ config::ParserConfig, core::{ orchestrator::Orchestrator, state::{ Mode, Token } } };
use serde::{ Deserialize, Serialize };
use wasm_bindgen::prelude::*;
use unicode_normalization::UnicodeNormalization;

#[derive(Deserialize, Serialize)]
struct Result<'a> {
    #[serde(borrow)] tokens: Vec<Token<'a>>,
    errors: Vec<usize>,
}

#[wasm_bindgen]
pub fn tokenize(input: &str, config: &str) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let config: ParserConfig = ParserConfigDeserializer::from_toml_str(config).unwrap().into();
    let mode = Mode::SinglePage;
    let input = input.nfc().collect::<String>();
    let parser = Orchestrator::new(&config, &mode);
    let (tokens, errors) = parser.tokenize(&input);
    let res = Result { tokens, errors };
    serde_wasm_bindgen::to_value(&res).unwrap()
}
