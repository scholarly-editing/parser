use config_deserializer::ParserConfigDeserializer;
use parser::{ config::ParserConfig, core::{ orchestrator::Orchestrator, state::Mode } };
fn main() {
    let input =
        "
    Lo-2-1 باب الأسد والثور
    Lo-2-3 قال {الملك} ديشلم *لبيدبا الفيلسوف اضرب لي مثل الرجلين المتحابّين
    يقطع بينهما الخؤون الكذوب ويحملهما على العداوة Lo-2-4 *** تقاطعا وتدابرا
    Lo-2-7 ومن أمثال ذلك أنّه كان بأرض ديستايد تاجر مكثر
    وكانت له بنون Lo-2-8 سلّم إليهم أمواله فلم يحترفوا بها بل تساهلوا في ذلك *تساهلًا
    يضيّع ذلك المال عليه وعليهم فلامهم أبوهم ووعظهم Lo-2-9 وقال يا بنيّ إنّ
    صاحب الدنيا يطلب ثلاث أمور لن يدركها إلّا بأربعة أشياء Lo-2-10 السعة
    في المعيشة والمنزلة في الناس والزاد إلى الآخرة Lo-2-11 وأمّا التي يحتاج إليها
    وإلى إدراكها فاكتساب المال من معروف وجوهه وحسن القيام عليه
    والتثمير له وإنفاقه فيما يصلح المعيشة ويرضي الأهل والإخوان ليعود عليه
    في الآخرة نفعه Lo-2-14 فمن أضاع ذلك لم يدرك ما أراد وإن هو لم يكتسب لم يكن
    له مال يعيش به Lo-2-16 وإن هو أضاعه لم تمنعه قلّة الإنفاق من سرعة النفاد
    Lo-2-17 كالكحل الذي لا يُؤخذ منه إلّا ما هو شبيه بالغبار ثمّ هو مع ذلك سريع النفاد
    والفناء Lo-2-19 وإن هو اكتسب وأصلح وأمسك عن وضعه في مواضعه كان ممّن
    يُعدّ فقيرًا ثمّ إنّ عدم وضعه في مواضعه لا يمنعه من أن يضيع في غير مراده
    واختياره Lo-2-20 كالمياه التي تجتمع إلى مكان ولا تزال تصبّ في ذلك المكان حتّى
    إذا زادت عن الحدّ سالت من سائر النواحي وربّما انبثق المكان فذهب
    ";

    //     let input =
    //         "
    // الماء ضياعًا وفسادًا Lo-2-21 ثمّ إنّ بني التاجر اتّعظوا وأخذوا برأي أبيهم Lo-2-22 فانطلق أكبرهم
    // متوجّهًا بتجارة إلى مدينة يُقال لها منون فأتى في طريقه على مكان شديد
    // الوحل وكان معه عجلة يجرّها ثوران أحدهما يُقال <له> شتربة
    // والآخر نبذة Lo-2-23 فوحل شتربة في ذلك المكان فاستخرجه الرجل
    // وأعوانه بعد ما كاد يتلف ولم يبق فيه قوّة على السير Lo-2-26 فخلّف التاجر
    // عنده رجلًا من أصحابه يقيم عليه أيّامًا فإن رآه قد صلح أتبعه به Lo-2-27 فلمّا
    // كان من الغد من ذلك اليوم برم الرجل بمكانه واستوحش لتفرّده في ذلك
    // المكان فترك الثور ولحق التاجر فأخبره أنّ الثور قد مات Lo-2-28 وقال
    // له إنّ الإنسان إذا انقضت مدّته وحانت منيّته فهو وإن اجتهد
    // في التوقّي والحذر من الأمور التي يخاف منها الهلاك على نفسه لم
    // يغن ذلك عنه شيئًا ولم يصل إلّا إلى العناء وربّما عاد اجتهاده في
    // توقّفه وحذره عليه بعطبه فيهلك Lo-2-29 كالذي قيل إنّه سلك مفازة
    // مشهورة بسكنى السباع وهي مخوفة بهم فخرج إليها ولم يمنعه الخوف من
    // الخروج إليها Lo-2-35 فلم يلبث إلّا قليلًا حتّى عرض له ذئب هو أخبثها
    // وأضراها Lo-2-36 فلمّا نظر الرجل إلى الذئب قاصدًا نحوه خافه فنظر يمينًا وشمالًا
    // يطلب موضعًا يختبئ فيه من الذئب فلم ير قريبًا من ذلك المكان إلّا قرية
    // خلف واد فمضى متوجّهًا نحو القرية والوادي يعدو عدوًا متداركًا
    // ";

    let test_toml_content =
        r#"
       [[page]]
prefix = "fol."
suffix = "r"

[[page]]
prefix = "fol."
suffix = "v"

[[word]]
label = "Arabic"
char_range = [0x600, 0x60FF]
additional_chars = []
default_state = "sound"


[[prefix]]
symbol = "|"
label = "verse"


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

    let processed_input = input.lines().map(str::trim).collect::<Vec<&str>>().join("\n");

    let slice = &processed_input.chars().skip(1115).take(7).collect::<String>();
    println!("Slice from 1115 to 1122: {}", slice);

    let config: ParserConfig = ParserConfigDeserializer::from_toml_str(test_toml_content)
        .unwrap()
        .into();

    let mode = Mode::MultiplePages;
    let parser = Orchestrator::new(&config, &mode);

    test(&processed_input, &parser);
}

fn test<'a>(input: &'a str, parser: &'a Orchestrator<'a>) {
    let (_tokens, _errors) = parser.tokenize(&input);
    // dbg!(_errors);
    dbg!(_tokens);
}
