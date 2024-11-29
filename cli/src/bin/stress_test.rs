use std::time::Instant;

use config_deserializer::ParserConfigDeserializer;
use parser::{
    config::ParserConfig,
    core::{ orchestrator::Orchestrator, state::Mode },
    document::Document,
};
fn main() {
    let input =
        "
        fol.1r
        قد  تصدّرت أبوابه وأُتقنت عجائبه على أفواه البهائم والطيور والوحوش والأنام
والهوام وسائر حشرات ممّا تحتاج إليه الملوك في سياسة *رعيّتها وتدبيرها الذي لا قوام
لهم في أمورهم إلّا بحسن سياسة الملوك وحسن أخلاقها ورأفتها *ورحمتها ولذلك لم يدع
أنوشروان اقتناء هذا الكتاب الذي بلغه عنه  *** أنّه ببلاد  الهند وضمّه~ إلى نفسه وملكه والعمل بما
بحسن تدبيره وعجائبه ممّا يقرّب إلى الله (عزّ وجلّ ويزجر  عن معاصي الله فلمّا عزم على ما أراد من
أمره وهمّ باقتناء هذا الكتاب والبعثة  في انتساخه قال في نفسه من لهذا الأمر العظيم والحال
الجسيم والأدب النفيس الذي يتزيّنون) به [[أهل]] {ملوك} الهند دون ملوك فارس وقد هممت أن لا
أدع بعد المشقّة وصعوبة الأمر ومخاطرة (أنفسنا في طلب هذا الكتاب حتّى نصل إليه وإلى
        fol.2v
        قد  تصدّرت أبوابه وأُتقنت عجائبه على أفواه) البهائم والطيور والوحوش والأنام
والهوام وسائر حشرات ممّا تحتاج إليه الملوك في سياسة *رعيّتها وتدبيرها الذي لا قوام
لهم في أمورهم إلّا بحسن سياسة الملوك وحسن أخلاقها ورأفتها *ورحمتها ولذلك لم يدع
أنوشروان اقتناء هذا الكتاب الذي بلغه عنه  *** أنّه ببلاد الهند وضمّه إلى نفسه وملكه والعمل بما
بحسن تدبيره وعجائبه ممّا يقرّب إلى الله (عزّ وجلّ ويزجر  عن معاصي الله فلمّا عزم على ما أراد من
أمره وهمّ باقتناء هذا الكتاب والبعثة  في انتساخه قال في نفسه من لهذا الأمر العظيم والحال
الجسيم والأدب النفيس الذي يتزيّنون) به [[أهل]] {ملوك} الهند دون ملوك فارس وقد هممت أن لا
أدع بعد المشقّة وصعوبة الأمر ومخاطرة أنفسنا في طلب هذا الكتاب حتّى نصل إليه وإلى
";

    let test_toml_content =
        r#"
          [[page]]
prefix = "fol."
suffix = "r"

[[page]]
prefix = "fol."
suffix = "v"

[[block]]
start_marker = "[illustration]"
type = "STANDALONE"

[[block]]
start_marker = "[legend]"
has_text = true
type = "WITH_TEXT"
end_marker = "---"

[[block]]
start_marker = "[margin]"
has_text = true
type = "WITH_TEXT"
end_marker = "---"

[[block]]
start_marker = "|"
has_text = true
type = "WITH_TEXT"
end_marker = "/"

[[word]]
label = "Arabic"
char_range = [600, 0x60FF]
additional_chars = []
default_state = "sound"


[[suffix]]
symbol = "~"
label = "middle-arabic"

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
skip = true

[[brackets]]
precedence = 0
open = "[["
close = "]]"
label = "cross-out"
skip = true


[[brackets]]
open = "{"
close = "}"
label = "suppletion"


[[brackets]]
open = "<"
close = ">"
label = "added"


[[tag]]
symbol = "***"
label = "lacuna"


[[tag]]
symbol = "..."
label = "damage"


        "#;

    // Split the input string into lines, trim each line, and rejoin
    let processed_input = input.lines().map(str::trim).collect::<Vec<&str>>().join("\n");

    let multipliers = [10, 100, 1000, 5000, 10000, 50000, 100000];
    // let multipliers = [1];
    let config: ParserConfig = ParserConfigDeserializer::from_toml_str(test_toml_content)
        .unwrap()
        .into();

    let mode = Mode::MultiplePages;
    let parser = Orchestrator::new(&config, &mode);

    for &multiplier in &multipliers {
        stress_test(&processed_input, multiplier, &parser);
    }
}

fn stress_test<'a>(input: &'a str, multiplier: usize, parser: &'a Orchestrator<'a>) {
    let repeated_input = input.repeat(multiplier);
    let start = Instant::now();
    let (tokens, _) = parser.tokenize(&repeated_input);
    let document = Document::new(tokens);
    let duration = start.elapsed();
    println!("Pages: {}, Duration: {:.2?} seconds", document.pages.len(), duration);
}

// Pages: 20, Duration: 25.94ms seconds
// Pages: 200, Duration: 138.29ms seconds
// Pages: 2000, Duration: 1.29s seconds
// Pages: 10000, Duration: 6.44s seconds
// Pages: 20000, Duration: 13.02s seconds
// Pages: 100000, Duration: 64.90s seconds
// Pages: 200000, Duration: 131.04s seconds

// after release:
// Pages: 20, Duration: 1.87ms seconds
// Pages: 200, Duration: 18.35ms seconds
// Pages: 2000, Duration: 101.41ms seconds
// Pages: 10000, Duration: 437.59ms seconds
// Pages: 20000, Duration: 876.37ms seconds
// Pages: 100000, Duration: 4.38s seconds
// Pages: 200000, Duration: 8.84s seconds
