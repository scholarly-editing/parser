use parser::{ config::ParserConfig, tokenizer::{ get_tokens, state::Mode } };
use std::time::Instant;
fn main() {
    let input =
        "  قد  تصدّرت أبوابه وأُتقنت عجائبه على أفواه البهائم والطيور والوحوش والأنام
والهوام وسائر حشرات ممّا تحتاج إليه الملوك في سياسة *رعيّتها وتدبيرها الذي لا قوام
لهم في أمورهم إلّا بحسن سياسة الملوك وحسن أخلاقها ورأفتها *ورحمتها ولذلك لم يدع
أنوشروان اقتناء هذا الكتاب الذي بلغه عنه  *** أنّه ببلاد الهند وضمّه إلى نفسه وملكه والعمل [[بما]]
بحسن تدبيره وعجائبه ممّا يقرّب إلى الله (عزّ وجلّ ويزجر  عن معاصي الله فلمّا عزم على ما أراد من
أمره وهمّ باقتناء هذا الكتاب والبعثة)  في انتساخه قال في نفسه من لهذا الأمر العظيم والحال
الجسيم والأدب النفيس الذي يتزيّنون به [[أهل]] {ملوك} الهند دون ملوك فارس وقد هممت أن لا
أدع بعد المشقّة وصعوبة الأمر ومخاطرة أنفسنا في طلب هذا الكتاب حتّى نصل إليه وإلى
";

    let test_toml_content =
        r#"
            [main]
            _language = "Arabic"
            char_range = [600, 0x60FF]
            additional_chars = []
            _page_marker_pattern = 'fol\.\d+[rv]'

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

    // Split the input string into lines, trim each line, and rejoin
    let processed_input = input.lines().map(str::trim).collect::<Vec<&str>>().join("\n");

    // Add a space at the end to match the format in the parse_with_errors call
    let formatted_input = format!("{} ", processed_input);

    fn stress_test(input: &str, multiplier: usize, toml_content: &str) {
        let repeated_input = input.repeat(multiplier);
        let start = Instant::now();
        let config = ParserConfig::from_str(toml_content).unwrap();
        let tokens = get_tokens(&repeated_input, config, Mode::default());
        let duration = start.elapsed();
        println!(
            "Multiplier: {}, Duration: {:.2?} seconds, Tokens: {}",
            multiplier,
            duration,
            tokens.unwrap().len()
        );
    }

    let multipliers = [10, 100];
    for &multiplier in &multipliers {
        stress_test(&formatted_input, multiplier, &test_toml_content);
    }
}

// Multiplier: 10, Duration: 251.23ms seconds, Tokens: 1220
//Multiplier: 100, Duration: 22.43s seconds, Tokens: 12200
// ---
// Multiplier: 10, Duration: 19.45ms seconds, Tokens: 1220
// Multiplier: 100, Duration: 728.38ms seconds, Tokens: 12200
// Multiplier: 1000, Duration: 64.62s seconds, Tokens: 122000
// Multiplier: 5000, Duration: 2127.61s seconds, Tokens: 610000
// Multiplier: 10000, Duration: 8202.65s seconds, Tokens: 1220000
