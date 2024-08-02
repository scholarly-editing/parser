use parser::{
    config::{ deserializer::ParserConfigDeserializer, ParserConfig },
    tokenizer::{ get_tokens, parsers::Parser, state::Mode },
};
use std::time::Instant;
fn main() {
    let input =
        "  قد  تصدّرت أبوابه وأُتقنت عجائبه على أفواه البهائم والطيور والوحوش والأنام
والهوام وسائر حشرات ممّا تحتاج إليه الملوك في سياسة *رعيّتها وتدبيرها الذي لا قوام
لهم في أمورهم إلّا بحسن سياسة الملوك وحسن أخلاقها ورأفتها *ورحمتها ولذلك لم يدع
أنوشروان اقتناء هذا الكتاب الذي بلغه عنه  *** أنّه ببلاد الهند وضمّه إلى نفسه وملكه والعمل [[بما]]
بحسن تدبيره وعجائبه ممّا يقرّب إلى الله (عزّ وجلّ ويزجر  عن معاصي الله فلمّا عزم على ما أراد من
أمره وهمّ باقتناء هذا الكتاب والبعثة  في انتساخه قال في نفسه من لهذا الأمر العظيم والحال
الجسيم والأدب النفيس الذي يتزيّنون) به [[أهل]] {ملوك} الهند دون ملوك فارس وقد هممت أن لا
أدع بعد المشقّة وصعوبة الأمر ومخاطرة أنفسنا في طلب هذا الكتاب حتّى نصل إليه وإلى
";

    let test_toml_content =
        r#"
            [page]
prefix = ["fol."]
suffix = ["r", "v"]

[[block]]
name = "illustration"
type = "STANDALONE"

[[block]]
name = "legend"
has_text = true
type = "WITH_TEXT"
end_marker = "---"

[[block]]
name = "margin"
has_text = true
type = "WITH_TEXT"
end_marker = "---"

[[word]]
label = "Arabic"
char_range = [600, 0x60FF]
additional_chars = []


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
open_label = "title_open"
close_label = "title_close"


[[brackets]]
open = "["
close = "]"
label = "superfluous"
skip = true
open_label = "superfluous_open"
close_label = "superfluous_close"

[[brackets]]
open = "[["
close = "]]"
label = "cross-out"
skip = true
open_label = "cross-out_open"
close_label = "cross-out_close"

[[brackets]]
open = "{"
close = "}"
label = "suppletion"
open_label = "suppletion_open"
close_label = "suppletion_close"

[[brackets]]
open = "<"
close = ">"
label = "added"
open_label = "added_open"
close_label = "added_close"

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

    let multipliers = [10, 100, 1000, 5000, 10000, 50000, 100000];
    let config: ParserConfig = ParserConfigDeserializer::from_str(test_toml_content)
        .unwrap()
        .into();
    let mode = Mode::default();
    let parser = Parser::new(&config, &mode);

    for &multiplier in &multipliers {
        stress_test(&formatted_input, multiplier, &parser);
    }
}

fn stress_test<'a>(input: &'a str, multiplier: usize, parser: &'a Parser<'a>) {
    let repeated_input = input.repeat(multiplier);
    let start = Instant::now();
    let tokens = get_tokens(&repeated_input, &parser);
    let duration = start.elapsed();
    println!(
        "Multiplier: {}, Duration: {:.2?} seconds, Tokens: {}",
        multiplier,
        duration,
        tokens.unwrap().len()
    );
}

// Multiplier: 10, Duration: 251.23ms seconds, Tokens: 1220
//Multiplier: 100, Duration: 22.43s seconds, Tokens: 12200
// ---
// Multiplier: 10, Duration: 19.45ms seconds, Tokens: 1220
// Multiplier: 100, Duration: 728.38ms seconds, Tokens: 12200
// Multiplier: 1000, Duration: 64.62s seconds, Tokens: 122000
// Multiplier: 5000, Duration: 2127.61s seconds, Tokens: 610000
// Multiplier: 10000, Duration: 8202.65s seconds, Tokens: 1220000
// ---
// Multiplier: 10, Duration: 15.92ms seconds, Tokens: 1220
// Multiplier: 100, Duration: 85.13ms seconds, Tokens: 12200
// Multiplier: 1000, Duration: 686.70ms seconds, Tokens: 122000
// Multiplier: 5000, Duration: 3.43s seconds, Tokens: 610000
// Multiplier: 10000, Duration: 6.87s seconds, Tokens: 1220000
// Multiplier: 50000, Duration: 34.35s seconds, Tokens: 6100000
// Multiplier: 100000, Duration: 69.12s seconds, Tokens: 12200000
