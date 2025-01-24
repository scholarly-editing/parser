use crate::config::ParserConfig;
use crate::core::{
    orchestrator::Orchestrator,
    state::{Mode, Token},
};

type Word = (String, String, usize, usize);

pub struct PassageState {
    config: ParserConfig,
    words: Vec<Word>,
    pub valid: bool,
}

fn extract_word(token: Token) -> Option<Word> {
    if let Token::Word(word_token) = token {
        Some((
            word_token.word.to_string(),
            word_token.state.to_string(),
            word_token.token_order,
            word_token.span.0,
        ))
    } else {
        None
    }
}

impl PassageState {
    pub fn new(input: &str, config: ParserConfig) -> Self {
        let mode = Mode::Passage;
        let parser = Orchestrator::new(&config, &mode);
        let (tokens, errors) = parser.tokenize(&input);

        if errors.is_empty() {
            let words: Vec<Word> = tokens.into_iter().filter_map(extract_word).collect();
            Self {
                words,
                valid: true,
                config,
            }
        } else {
            Self {
                words: vec![],
                valid: false,
                config,
            }
        }
    }

    pub fn get_raw_passage(&self) -> String {
        self.words
            .iter()
            .map(|word| word.0.clone())
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn locate_tokens(&self, tokens: &[Word]) -> Option<(usize, usize)> {
        let first_token = tokens.first().unwrap();
        let last_token = tokens.last().unwrap();

        let mut first_token_index = None;
        let mut last_token_index = None;

        for token in self.words.iter() {
            if first_token_index.is_none() && token.0 == first_token.0 {
                first_token_index = Some(token.2);
            }

            if let Some(_) = first_token_index {
                if token.0 == last_token.0 {
                    last_token_index = Some(token.2);
                    break;
                } else if token.0 == first_token.0 {
                    first_token_index = Some(token.2);
                }
            }
        }

        match (first_token_index, last_token_index) {
            (Some(first_index), Some(last_index)) => Some((first_index, last_index)),
            _ => None,
        }
    }

    pub fn locate_fragment(&self, fragment: &str) -> Option<(usize, usize)> {
        let mode = Mode::Passage;
        let parser = Orchestrator::new(&self.config, &mode);
        let (tokens, errors) = parser.tokenize(fragment);

        if !errors.is_empty() {
            return None;
        }

        let words: Vec<Word> = tokens.into_iter().filter_map(extract_word).collect();
        self.locate_tokens(&words)
    }
}
