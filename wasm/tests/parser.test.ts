import { describe, it, expect, beforeAll } from 'vitest';
let wasmModule: any;


const input = `
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
`

const config = `
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
`

describe('Parser Test', () => {


    beforeAll(async () => {
        wasmModule = await import('../parser-wasm/pkg'); // Adjust the path as needed
    });
    it('should load parse text', async () => {
        const result = wasmModule.tokenize(input, config);
        console.log(JSON.stringify(result, null, 2));
        expect(result).not.toBeNull();
    });
});