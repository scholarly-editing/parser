import { describe, it, expect, beforeAll } from 'vitest';

let wasmModule: any;

// ── Shared TOML config (matches config.example.toml) ────────────────────────

const tomlConfig = `
[[page]]
prefix = "fol."
suffix = "r"

[[page]]
prefix = "fol."
suffix = "v"

[[word]]
label = "Arabic"
char_range = [0x600, 0x6FF]
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

[[suffix]]
symbol = "~"
label = "middle-arabic"

[[brackets]]
precedence = 0
open = "[["
close = "]]"
label = "cross-out"
skip = true

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
`;

// ── Equivalent JSON config ──────────────────────────────────────────────────

const jsonConfig = JSON.stringify({
  page: [
    { prefix: "fol.", suffix: "r" },
    { prefix: "fol.", suffix: "v" },
  ],
  word: [
    {
      label: "Arabic",
      char_range: [0x600, 0x6ff],
      additional_chars: [],
      default_state: "sound",
    },
  ],
  prefix: [
    { symbol: "|", label: "verse" },
    { symbol: "*", label: "emendation" },
    { symbol: "?", label: "unintelligible" },
    { symbol: "؟", label: "unintelligible" },
    { symbol: "!", label: "error" },
    { symbol: "†", label: "corrupt" },
  ],
  suffix: [{ symbol: "~", label: "middle-arabic" }],
  brackets: [
    { precedence: 0, open: "[[", close: "]]", label: "cross-out", skip: true },
    { open: "(", close: ")", label: "title" },
    { open: "[", close: "]", label: "superfluous", skip: true },
    { open: "{", close: "}", label: "suppletion" },
    { open: "<", close: ">", label: "added" },
  ],
  tag: [
    { symbol: "***", label: "lacuna" },
    { symbol: "...", label: "damage" },
  ],
  block: [
    {
      start_marker: "[illustration]",
      type: "STANDALONE",
    },
    {
      start_marker: "[legend]",
      has_text: true,
      type: "WITH_TEXT",
      end_marker: "---",
    },
    {
      start_marker: "[margin]",
      has_text: true,
      type: "WITH_TEXT",
      end_marker: "---",
    },
  ],
});

// ── Helpers ─────────────────────────────────────────────────────────────────

type Token = Record<string, any>;

function tokenize(
  input: string,
  mode?: string,
  configFormat?: string,
  config?: string,
): { tokens: Token[]; errors: number[] } {
  return wasmModule.tokenize(
    input,
    config ?? tomlConfig,
    mode ?? null,
    configFormat ?? null,
  );
}

/** Extract Word tokens. */
function words(tokens: Token[]): any[] {
  return tokens.filter((t) => t.Word).map((t) => t.Word);
}

/** Extract Tag tokens. */
function tags(tokens: Token[]): any[] {
  return tokens.filter((t) => t.Tag).map((t) => t.Tag);
}

/** Extract PageBreak tokens. */
function pageBreaks(tokens: Token[]): any[] {
  return tokens.filter((t) => t.PageBreak).map((t) => t.PageBreak);
}

/** Extract Block tokens. */
function blocks(tokens: Token[]): any[] {
  return tokens.filter((t) => t.Block).map((t) => t.Block);
}

/** Extract WordInBlock tokens. */
function wordsInBlock(tokens: Token[]): any[] {
  return tokens.filter((t) => t.WordInBlock).map((t) => t.WordInBlock);
}

/** Extract Error tokens. */
function errors(tokens: Token[]): any[] {
  return tokens.filter((t) => t.Error).map((t) => t.Error);
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

beforeAll(async () => {
  wasmModule = await import('../parser-wasm/pkg');
});

// ── Basic word tokenization ─────────────────────────────────────────────────

describe('Basic word tokenization', () => {
  it('parses simple Arabic text into words', () => {
    const { tokens, errors: errs } = tokenize('باب الأسد والثور');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(3);
    expect(w.map((x: any) => x.word)).toEqual(['باب', 'الأسد', 'والثور']);
  });

  it('word tokens have correct type and state', () => {
    const { tokens } = tokenize('الفيلسوف');
    const w = words(tokens);

    expect(w).toHaveLength(1);
    expect(w[0].word_type).toBe('Arabic');
    expect(w[0].state).toBe('sound');
  });

  it('word tokens include position metadata', () => {
    const { tokens } = tokenize('كلمة');
    const w = words(tokens)[0];

    expect(w.page_number).toBe(0);
    expect(w.line_number).toBe(0);
    expect(w.token_order).toBe(0);
    expect(w.span).toBeDefined();
  });

  it('handles words with diacritics (shaddah, tanwin)', () => {
    const { tokens, errors: errs } = tokenize('المتحابّين تساهلًا');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(2);
  });

  it('handles line breaks between words', () => {
    const { tokens, errors: errs } = tokenize('باب\nالأسد');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(2);
  });
});

// ── Prefix handling ─────────────────────────────────────────────────────────

describe('Prefix handling', () => {
  it('* prefix → emendation state', () => {
    const { tokens, errors: errs } = tokenize('*لبيدبا');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(1);
    expect(w[0].word).toBe('لبيدبا');
    expect(w[0].state).toBe('emendation');
  });

  it('? prefix → unintelligible state', () => {
    const { tokens, errors: errs } = tokenize('?كلمة');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w[0].state).toBe('unintelligible');
  });

  it('! prefix → error state', () => {
    const { tokens, errors: errs } = tokenize('!خطأ');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w[0].state).toBe('error');
  });

  it('† prefix → corrupt state', () => {
    const { tokens, errors: errs } = tokenize('†فاسد');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w[0].state).toBe('corrupt');
  });

  it('| prefix → verse state', () => {
    const { tokens, errors: errs } = tokenize('|بيت');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w[0].state).toBe('verse');
  });

  it('~ suffix → middle-arabic state', () => {
    const { tokens, errors: errs } = tokenize('كلمة~');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(1);
    expect(w[0].word).toBe('كلمة');
    expect(w[0].state).toBe('middle-arabic');
  });
});

// ── Bracket handling ────────────────────────────────────────────────────────

describe('Bracket handling', () => {
  it('{word} → suppletion_single state', () => {
    const { tokens, errors: errs } = tokenize('{الملك}');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(1);
    expect(w[0].word).toBe('الملك');
    expect(w[0].state).toBe('suppletion_single');
  });

  it('{word word} → suppletion_open / suppletion_close', () => {
    const { tokens, errors: errs } = tokenize('{كلمة أخرى}');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(2);
    expect(w[0].state).toBe('suppletion_open');
    expect(w[1].state).toBe('suppletion_close');
  });

  it('[word] → superfluous_single (skip bracket)', () => {
    const { tokens, errors: errs } = tokenize('[زائد]');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(1);
    expect(w[0].state).toBe('superfluous_single');
  });

  it('[[word]] → cross-out_single (skip=true, precedence=0)', () => {
    const { tokens, errors: errs } = tokenize('[[مشطوب]]');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(1);
    expect(w[0].state).toBe('cross-out_single');
  });

  it('(word) → title_single', () => {
    const { tokens, errors: errs } = tokenize('(العنوان)');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(1);
    expect(w[0].state).toBe('title_single');
  });

  it('<word> → added_single', () => {
    const { tokens, errors: errs } = tokenize('<له>');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(1);
    expect(w[0].state).toBe('added_single');
  });

  it('brackets in context: text {word} text', () => {
    const { tokens, errors: errs } = tokenize('قال {الملك} ديشلم');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(3);
    expect(w[0].state).toBe('sound');
    expect(w[1].state).toBe('suppletion_single');
    expect(w[2].state).toBe('sound');
  });
});

// ── Tag handling ────────────────────────────────────────────────────────────

describe('Tag handling', () => {
  it('*** → Tag with label "lacuna"', () => {
    const { tokens, errors: errs } = tokenize('***');
    const t = tags(tokens);

    expect(errs).toHaveLength(0);
    expect(t).toHaveLength(1);
    expect(t[0].tag).toBe('***');
    expect(t[0].label).toBe('lacuna');
  });

  it('... → Tag with label "damage"', () => {
    const { tokens, errors: errs } = tokenize('...');
    const t = tags(tokens);

    expect(errs).toHaveLength(0);
    expect(t).toHaveLength(1);
    expect(t[0].tag).toBe('...');
    expect(t[0].label).toBe('damage');
  });

  it('tag between words', () => {
    const { tokens, errors: errs } = tokenize('كلمة *** أخرى');
    const w = words(tokens);
    const t = tags(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(2);
    expect(t).toHaveLength(1);
    expect(t[0].label).toBe('lacuna');
  });

  it('tag has position metadata', () => {
    const { tokens } = tokenize('***');
    const t = tags(tokens)[0];

    expect(t.page_number).toBe(0);
    expect(t.line_number).toBe(0);
    expect(t.span).toBeDefined();
  });
});

// ── Page break handling ─────────────────────────────────────────────────────

describe('Page break handling', () => {
  it('fol. page marker produces PageBreak token', () => {
    const { tokens, errors: errs } = tokenize(
      'fol.1r\nكلمة',
      'multiple_pages',
    );
    const pb = pageBreaks(tokens);

    expect(errs).toHaveLength(0);
    expect(pb).toHaveLength(1);
    expect(pb[0].page_representation).toBe('fol.1r');
    expect(pb[0].page_number).toBe(1);
  });

  it('page_number increments across pages', () => {
    const { tokens } = tokenize(
      'fol.1r\nكلمة\nfol.1v\nأخرى',
      'multiple_pages',
    );
    const pb = pageBreaks(tokens);

    expect(pb).toHaveLength(2);
    expect(pb[0].page_number).toBe(1);
    expect(pb[1].page_number).toBe(2);
  });

  it('words after page break have updated page_number', () => {
    const { tokens, errors: errs } = tokenize(
      'fol.1r\nكلمة\nfol.1v\nأخرى',
      'multiple_pages',
    );
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w[0].page_number).toBe(1);
    expect(w[1].page_number).toBe(2);
  });
});

// ── Block handling ──────────────────────────────────────────────────────────

describe('Block handling', () => {
  it('standalone block produces Block token', () => {
    const { tokens } = tokenize('[illustration]\nكلمة');
    const b = blocks(tokens);

    expect(b).toHaveLength(1);
    expect(b[0].block_name).toBe('illustration');
  });

  it('with-text block produces Block + WordInBlock tokens', () => {
    const { tokens } = tokenize('[legend]\nنص الأسطورة\n---\nكلمة');
    const b = blocks(tokens);
    const wib = wordsInBlock(tokens);
    const w = words(tokens);

    expect(b).toHaveLength(1);
    expect(b[0].block_name).toBe('legend');
    expect(wib.length).toBeGreaterThanOrEqual(1);
    // Word after end marker is a regular word
    expect(w.some((x: any) => x.word === 'كلمة')).toBe(true);
  });

  it('WordInBlock has block_name set', () => {
    const { tokens } = tokenize('[margin]\nحاشية\n---');
    const wib = wordsInBlock(tokens);

    expect(wib).toHaveLength(1);
    expect(wib[0].block_name).toBe('margin');
    expect(wib[0].word).toBe('حاشية');
  });
});

// ── Error token tests ───────────────────────────────────────────────────────

describe('Error tokens', () => {
  it('unknown character produces Error token', () => {
    const { tokens, errors: errs } = tokenize('كلمة @ أخرى');

    expect(errs.length).toBeGreaterThan(0);
    const e = errors(tokens);
    expect(e.length).toBeGreaterThan(0);
  });

  it('Error token includes span and metadata', () => {
    const { tokens } = tokenize('كلمة @ أخرى');
    const e = errors(tokens);

    expect(e[0].span).toBeDefined();
    expect(e[0].error).toBeDefined();
  });

  it('empty brackets produce error', () => {
    const { errors: errs } = tokenize('{}');
    expect(errs.length).toBeGreaterThan(0);
  });
});

// ── Mode tests ──────────────────────────────────────────────────────────────

describe('Mode selection', () => {
  it('single_page mode (default) parses words', () => {
    const { tokens, errors: errs } = tokenize('باب الأسد', 'single_page');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(2);
  });

  it('multiple_pages mode recognises page breaks', () => {
    const { tokens } = tokenize(
      'fol.1r\nباب\nfol.1v\nالأسد',
      'multiple_pages',
    );
    const pb = pageBreaks(tokens);
    expect(pb.length).toBe(2);
  });

  it('passage mode treats line breaks as spaces', () => {
    const { tokens, errors: errs } = tokenize('كلمة\nأخرى', 'passage');
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w).toHaveLength(2);
    // line_number stays 0 in passage mode since line breaks are spaces
    expect(w[0].line_number).toBe(0);
    expect(w[1].line_number).toBe(0);
  });

  it('invalid mode throws a JS error', () => {
    expect(() => tokenize('كلمة', 'invalid_mode')).toThrow(/Invalid mode/);
  });
});

// ── Config format tests ─────────────────────────────────────────────────────

describe('Config format selection', () => {
  it('TOML config produces correct tokens', () => {
    const { tokens, errors: errs } = tokenize(
      '*لبيدبا',
      'single_page',
      'toml',
      tomlConfig,
    );
    const w = words(tokens);

    expect(errs).toHaveLength(0);
    expect(w[0].state).toBe('emendation');
  });

  it('JSON config produces identical results to TOML', () => {
    const input = '{الملك} *لبيدبا *** كلمة';

    const tomlResult = tokenize(input, 'single_page', 'toml', tomlConfig);
    const jsonResult = tokenize(input, 'single_page', 'json', jsonConfig);

    const tomlWords = words(tomlResult.tokens).map((w: any) => ({
      word: w.word,
      state: w.state,
    }));
    const jsonWords = words(jsonResult.tokens).map((w: any) => ({
      word: w.word,
      state: w.state,
    }));

    expect(jsonWords).toEqual(tomlWords);
    expect(tags(jsonResult.tokens).length).toBe(tags(tomlResult.tokens).length);
  });

  it('invalid config_format throws a JS error', () => {
    expect(() => tokenize('كلمة', 'single_page', 'csv')).toThrow(
      /Invalid config_format/,
    );
  });
});
