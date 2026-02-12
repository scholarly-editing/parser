use std::error::Error;
use std::fs;
use std::path::Path;

use parser::core::state::{Token, TokenizerError, TokenizerState};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Serializable output types
// ---------------------------------------------------------------------------

/// Full parsing result written to `<stem>.result.json`.
#[derive(Serialize)]
pub struct ResultOutput<'a> {
    pub tokens: &'a [Token<'a>],
    pub error_indices: &'a [usize],
    pub token_count: usize,
    pub error_count: usize,
    pub line_number: usize,
    pub page_number: usize,
    pub parsed_len: usize,
}

impl<'a> ResultOutput<'a> {
    /// Build a [`ResultOutput`] from a completed [`TokenizerState`].
    pub fn from_state(state: &'a TokenizerState<'a>) -> Self {
        Self {
            tokens: &state.tokens,
            error_indices: &state.errors,
            token_count: state.tokens.len(),
            error_count: state.errors.len(),
            line_number: state.line_number,
            page_number: state.page_number,
            parsed_len: state.parsed_len,
        }
    }
}

// ---------------------------------------------------------------------------
// Error extraction
// ---------------------------------------------------------------------------

/// Extract the inner [`TokenizerError`] structs from the token list using
/// the error indices.
pub fn extract_errors<'a>(state: &'a TokenizerState<'a>) -> Vec<&'a TokenizerError<'a>> {
    state
        .errors
        .iter()
        .filter_map(|&idx| match state.tokens.get(idx) {
            Some(Token::Error(err)) => Some(err),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// File writing
// ---------------------------------------------------------------------------

/// Write a serializable value as pretty-printed JSON to `path`.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json)?;
    Ok(())
}
