// use crate::{
//     config::ParserConfig,
//     tokenizer::{
//         parsers::parse_line_break,
//         state::{ Mode, Token, TokenizerError, TokenizerState },
//     },
// };

// pub fn get_tokens(
//     input: &str,
//     config: ParserConfig,
//     mode: Mode
// ) -> Result<Vec<Token>, Vec<TokenizerError>> {
//     let mut state = TokenizerState::new(input);

//     while state.parsing_not_finished() {
//         if let Some(new_state) = parse_line_break(&mut state) {
//             state = new_state;
//             continue;
//         }
//     }

//     if state.errors.is_empty() {
//         Ok(state.tokens)
//     } else {
//         Err(state.errors)
//     }
// }
