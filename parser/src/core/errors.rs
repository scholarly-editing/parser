use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TextError {
    WordPrefixedWithUnwantedChars,
    WordSuffixedWithUnwantedChars,
    WordInfixedWithUnwantedChars,
    InvalidSuffixPosition,
    EmptyBrackets,
    // The content inside the brackets matters for these error types
    BlockStartInBrackets(String),
    BlockEndInBrackets,
    InvalidPageBreak(String),
    InvalidLineBreak,
    SpaceInBrackets,
    InvalidPrefixUse,
    // Generic error for extensibility
    Custom(String),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextError::WordPrefixedWithUnwantedChars => write!(f, "Word prefixed with unwanted characters"),
            TextError::WordSuffixedWithUnwantedChars => write!(f, "Word suffixed with unwanted characters"),
            TextError::WordInfixedWithUnwantedChars => write!(f, "Word infixed with unwanted characters"),
            TextError::InvalidSuffixPosition => write!(f, "cannot add suffix here"),
            TextError::EmptyBrackets => write!(f, "Empty brackets"),
            TextError::BlockStartInBrackets(_name) => write!(f, "Block marker not allowed inside brackets"),
            TextError::BlockEndInBrackets => write!(f, "Block end marker not allowed inside brackets"),
            TextError::InvalidPageBreak(_repr) => write!(f, "Invalid position for page break"),
            TextError::InvalidLineBreak => write!(f, "Invalid position for line break"),
            TextError::SpaceInBrackets => write!(f, "Space on the wrong side of a bracket"),
            TextError::InvalidPrefixUse => write!(f, "Wrong prefix use: prefix not attached to a word"),
            TextError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}
