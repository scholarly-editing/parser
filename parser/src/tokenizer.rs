pub mod parsers;
pub mod state;
pub mod util;

use self::{ parsers::Parser, state::{ Token, TokenizerError, TokenizerState } };

pub fn get_tokens<'a, 'b, 'c>(
    input: &'a str,
    parser: &'a Parser<'b, 'c>
) -> Result<Vec<Token<'a>>, Vec<TokenizerError<'a>>> {
    let mut state = TokenizerState::new(input);

    while state.parsing_not_finished() {
        if let Some(action) = parser.run(&state) {
            action.apply(&mut state);
        } else {
            state.handle_error();
        }
    }

    if state.errors.is_empty() {
        Ok(state.tokens)
    } else {
        Err(state.errors)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_parse_on_valid_input() {
//         let input =
//             "  قد  تصدّرت أبوابه وأُتقنت عجائبه على أفواه البهائم والطيور والوحوش والأنام
// والهوام وسائر حشرات ممّا تحتاج إليه الملوك في سياسة *رعيّتها وتدبيرها الذي لا قوام
// لهم في أمورهم إلّا بحسن سياسة الملوك وحسن أخلاقها ورأفتها *ورحمتها ولذلك لم يدع
// أنوشروان اقتناء هذا الكتاب الذي بلغه عنه  *** أنّه ببلاد الهند وضمّه إلى نفسه وملكه والعمل [[بما]]
// بحسن تدبيره وعجائبه ممّا يقرّب إلى الله (عزّ وجلّ ويزجر  عن معاصي الله فلمّا عزم على ما أراد من
// أمره وهمّ باقتناء هذا الكتاب والبعثة)  في انتساخه قال في نفسه من لهذا الأمر العظيم والحال
// الجسيم والأدب النفيس الذي يتزيّنون به [[أهل]] {ملوك} الهند دون ملوك فارس وقد هممت أن لا
// أدع بعد المشقّة وصعوبة الأمر ومخاطرة أنفسنا في طلب هذا الكتاب حتّى نصل إليه وإلى
// ";

//         let test_toml_content =
//             r#"
//             [main]
//             _language = "Arabic"
//             char_range = [600, 0x60FF]
//             additional_chars = []
//             _page_marker_pattern = 'fol\.\d+[rv]'

//             [[prefix]]
//             symbol = "*"
//             label = "emendation"

//             [[prefix]]
//             symbol = "?"
//             label = "unintelligible"

//             [[prefix]]
//             symbol = "؟"
//             label = "unintelligible"

//             [[prefix]]
//             symbol = "!"
//             label = "error"

//             [[prefix]]
//             symbol = "†"
//             label = "corrupt"

//             [[brackets]]
//             open = "("
//             close = ")"
//             label = "title"

//             [[brackets]]
//             open = "["
//             close = "]"
//             label = "superfluous"

//             [[brackets]]
//             open = "[["
//             close = "]]"
//             label = "cross-out"

//             [[brackets]]
//             open = "{"
//             close = "}"
//             label = "suppletion"

//             [[brackets]]
//             open = "<"
//             close = ">"
//             label = "added"

//             [[sequence]]
//             symbol = "***"
//             label = "lacuna"

//             [[sequence]]
//             symbol = "..."
//             label = "damage"

//         "#;

//         let config = ParserConfig::from_str(test_toml_content).unwrap();

//         // Split the input string into lines, trim each line, and rejoin
//         let processed_input = input.lines().map(str::trim).collect::<Vec<&str>>().join("\n");

//         // Add a space at the end to match the format in the parse_with_errors call
//         let formatted_input = format!("{} ", processed_input);

//         let tokens = get_tokens(&formatted_input, config, Mode::default());
//         assert!(tokens.is_ok());
//     }

//     #[test]
//     fn test_parse_on_invalid_input() {
//         let input =
//             "  قدk  تصدّرت أبوابه وأُت!قنت عجائبه على أفواه البهائم والطيور والوحوش والأنام
// والهوام وسائر حشرات ممّا تحتاج إليه الملوك في سياسة *رعيّتها وتدبيرها الذي لا قوام
// لهم في أمورهم إلّا بحسن سياسة الملوك وحسن أخلاقها ورأفتها *ورحمتها ولذلك لم يدع
// أنوشروان اقتناء هذا الكتاب الذي بلغه عنه  *** أنّه ببلاد الهند وضمّه إلى نفسه وملكه والعمل [[بما]]
// بحسن تدبيره وعجائبه ممّا يقرّب إلى الله عزّ وجلّ ويزجر   c * عن معاصي الله فلمّا عزم على ما أراد من
// أمره وهمّ باقتناء هذا الكتاب والبعثة في انتساخه قال في نفسه من لهذا الأمر العظيم والحال
// الجسيم والأدب النفيس ال)ذي يتزيّنون به [[أهل]] {ملوك} الهند دون ملوك فارس وقد هممت أن لا
// أدع بعد المشقّة وصعوبة الأمر ومخاطرة أنفسنا في طلب هذا الكتاب حتّى نصل إليه وإلى
// ";

//         // Split the input string into lines, trim each line, and rejoin
//         let processed_input = input.lines().map(str::trim).collect::<Vec<&str>>().join("\n");

//         // Add a space at the end to match the format in the parse_with_errors call
//         let formatted_input = format!("{} ", processed_input);

//         let test_toml_content =
//             r#"
//             [main]
//             _language = "Arabic"
//             char_range = [600, 0x60FF]
//             additional_chars = []
//             _page_marker_pattern = 'fol\.\d+[rv]'

//             [[prefix]]
//             symbol = "*"
//             label = "emendation"

//             [[prefix]]
//             symbol = "?"
//             label = "unintelligible"

//             [[prefix]]
//             symbol = "؟"
//             label = "unintelligible"

//             [[prefix]]
//             symbol = "!"
//             label = "error"

//             [[prefix]]
//             symbol = "†"
//             label = "corrupt"

//             [[brackets]]
//             open = "("
//             close = ")"
//             label = "title"

//             [[brackets]]
//             open = "["
//             close = "]"
//             label = "superfluous"

//             [[brackets]]
//             open = "[["
//             close = "]]"
//             label = "cross-out"

//             [[brackets]]
//             open = "{"
//             close = "}"
//             label = "suppletion"

//             [[brackets]]
//             open = "<"
//             close = ">"
//             label = "added"

//             [[sequence]]
//             symbol = "***"
//             label = "lacuna"

//             [[sequence]]
//             symbol = "..."
//             label = "damage"

//         "#;

//         let config = ParserConfig::from_str(test_toml_content).unwrap();

//         let tokens = get_tokens(&formatted_input, config, Mode::default());
//         dbg!(&tokens);
//         assert!(tokens.is_err());
//     }
// }
