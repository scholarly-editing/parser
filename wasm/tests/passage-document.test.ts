import { describe, it, expect, beforeAll } from 'vitest';

let wasmModule: any;

// ── Shared TOML config ──────────────────────────────────────────────────────

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
symbol = "*"
label = "emendation"

[[prefix]]
symbol = "?"
label = "unintelligible"

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
open = "["
close = "]"
label = "superfluous"
skip = true

[[tag]]
symbol = "***"
label = "lacuna"
`;

// ── Equivalent JSON config ──────────────────────────────────────────────────

const jsonConfig = JSON.stringify({
  page: [
    { prefix: 'fol.', suffix: 'r' },
    { prefix: 'fol.', suffix: 'v' },
  ],
  word: [
    {
      label: 'Arabic',
      char_range: [0x600, 0x6ff],
      additional_chars: [],
      default_state: 'sound',
    },
  ],
  prefix: [
    { symbol: '*', label: 'emendation' },
    { symbol: '?', label: 'unintelligible' },
  ],
  brackets: [
    { precedence: 0, open: '[[', close: ']]', label: 'cross-out', skip: true },
    { open: '{', close: '}', label: 'suppletion' },
    { open: '[', close: ']', label: 'superfluous', skip: true },
  ],
  tag: [{ symbol: '***', label: 'lacuna' }],
});

// ── Helpers ─────────────────────────────────────────────────────────────────

type Token = Record<string, any>;

function flatTokenCount(pages: Token[][]): number {
  return pages.reduce((sum, page) => sum + page.length, 0);
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

beforeAll(async () => {
  wasmModule = await import('../parser-wasm/pkg');
});

// ── Passage tests ───────────────────────────────────────────────────────────

describe('Passage', () => {
  it('is_valid() returns true for valid Arabic text', () => {
    const passage = wasmModule.createPassage('باب الأسد والثور', tomlConfig);

    expect(passage.is_valid()).toBe(true);
  });

  it('get_raw_passage() returns space-separated words', () => {
    const passage = wasmModule.createPassage('باب الأسد والثور', tomlConfig);

    expect(passage.get_raw_passage()).toBe('باب الأسد والثور');
  });

  it('locate_fragment() returns valid indices for a known substring', () => {
    const passage = wasmModule.createPassage('باب الأسد والثور', tomlConfig);
    const location = passage.locate_fragment('الأسد والثور');

    expect(location[0]).toBeGreaterThanOrEqual(0);
    expect(location[1]).toBeGreaterThanOrEqual(0);
    expect(location[1]).toBeGreaterThanOrEqual(location[0]);
  });

  it('locate_fragment() returns (-1, -1) for a fragment not in the passage', () => {
    const passage = wasmModule.createPassage('باب الأسد والثور', tomlConfig);
    const location = passage.locate_fragment('كلمة مفقودة');

    expect(location[0]).toBe(-1);
    expect(location[1]).toBe(-1);
  });

  it('words with prefixes (emendations) are included in raw passage', () => {
    const passage = wasmModule.createPassage(
      'قال *لبيدبا الفيلسوف',
      tomlConfig,
    );

    expect(passage.is_valid()).toBe(true);
    const raw = passage.get_raw_passage();
    expect(raw).toContain('لبيدبا');
    expect(raw).toContain('قال');
    expect(raw).toContain('الفيلسوف');
  });

  it('words in brackets are included in raw passage', () => {
    const passage = wasmModule.createPassage(
      'قال {الملك} ديشلم',
      tomlConfig,
    );

    expect(passage.is_valid()).toBe(true);
    expect(passage.get_raw_passage()).toContain('الملك');
  });

  it('is_valid() returns false when input contains errors', () => {
    const passage = wasmModule.createPassage('كل@مة', tomlConfig);

    expect(passage.is_valid()).toBe(false);
  });

  it('get_raw_passage() returns empty string for invalid passage', () => {
    const passage = wasmModule.createPassage('كل@مة', tomlConfig);

    expect(passage.get_raw_passage()).toBe('');
  });

  it('locate_fragment() finds a single word', () => {
    const passage = wasmModule.createPassage('باب الأسد والثور', tomlConfig);
    const location = passage.locate_fragment('الأسد');

    expect(location[0]).toBeGreaterThanOrEqual(0);
    expect(location[1]).toBe(location[0]); // single word: start == end
  });
});

// ── Passage with different config formats ───────────────────────────────────

describe('Passage with JSON config', () => {
  it('createPassage with JSON config produces same raw passage as TOML', () => {
    const input = 'قال *لبيدبا {الملك} ديشلم';

    const tomlPassage = wasmModule.createPassage(input, tomlConfig, 'toml');
    const jsonPassage = wasmModule.createPassage(input, jsonConfig, 'json');

    expect(jsonPassage.is_valid()).toBe(tomlPassage.is_valid());
    expect(jsonPassage.get_raw_passage()).toBe(tomlPassage.get_raw_passage());
  });

  it('locate_fragment returns same indices with JSON and TOML configs', () => {
    const input = 'باب الأسد والثور';

    const tomlPassage = wasmModule.createPassage(input, tomlConfig, 'toml');
    const jsonPassage = wasmModule.createPassage(input, jsonConfig, 'json');

    const tomlLoc = tomlPassage.locate_fragment('الأسد');
    const jsonLoc = jsonPassage.locate_fragment('الأسد');

    expect(jsonLoc[0]).toBe(tomlLoc[0]);
    expect(jsonLoc[1]).toBe(tomlLoc[1]);
  });
});

// ── Document tests (tokenize_document) ──────────────────────────────────────

describe('tokenize_document', () => {
  it('multi-page input produces multiple page arrays', () => {
    const input = 'fol.1r\nكلمة\nfol.1v\nأخرى';
    const result = wasmModule.tokenize_document(
      input,
      tomlConfig,
      'multiple_pages',
    );

    expect(result.pages).toBeDefined();
    expect(Array.isArray(result.pages)).toBe(true);
    expect(result.pages.length).toBe(2);
  });

  it('each page starts with its PageBreak token', () => {
    const input = 'fol.1r\nكلمة\nfol.1v\nأخرى';
    const result = wasmModule.tokenize_document(
      input,
      tomlConfig,
      'multiple_pages',
    );

    for (const page of result.pages) {
      expect(page[0]).toHaveProperty('PageBreak');
    }
  });

  it('word tokens within each page have the correct page_number', () => {
    const input = 'fol.1r\nكلمة\nfol.1v\nأخرى';
    const result = wasmModule.tokenize_document(
      input,
      tomlConfig,
      'multiple_pages',
    );

    const page1Words = result.pages[0]
      .filter((t: Token) => t.Word)
      .map((t: Token) => t.Word);
    const page2Words = result.pages[1]
      .filter((t: Token) => t.Word)
      .map((t: Token) => t.Word);

    expect(page1Words.length).toBeGreaterThan(0);
    expect(page2Words.length).toBeGreaterThan(0);

    for (const w of page1Words) {
      expect(w.page_number).toBe(1);
    }
    for (const w of page2Words) {
      expect(w.page_number).toBe(2);
    }
  });

  it('single-page input (no page breaks) produces one page', () => {
    const input = 'باب الأسد والثور';
    const result = wasmModule.tokenize_document(input, tomlConfig);

    expect(result.pages).toHaveLength(1);
    const words = result.pages[0]
      .filter((t: Token) => t.Word)
      .map((t: Token) => t.Word);
    expect(words).toHaveLength(3);
  });

  it('errors array is populated for input with known errors', () => {
    const input = 'fol.1r\nكلمة {} أخرى';
    const result = wasmModule.tokenize_document(
      input,
      tomlConfig,
      'multiple_pages',
    );

    expect(result.errors.length).toBeGreaterThan(0);
  });

  it('three-page input groups correctly', () => {
    const input = 'fol.1r\nباب\nfol.1v\nالأسد\nfol.2r\nوالثور';
    const result = wasmModule.tokenize_document(
      input,
      tomlConfig,
      'multiple_pages',
    );

    expect(result.pages).toHaveLength(3);

    const pageWords = result.pages.map((page: Token[]) =>
      page
        .filter((t: Token) => t.Word)
        .map((t: Token) => t.Word.word),
    );
    expect(pageWords[0]).toEqual(['باب']);
    expect(pageWords[1]).toEqual(['الأسد']);
    expect(pageWords[2]).toEqual(['والثور']);
  });
});

// ── Integration: tokenize vs tokenize_document ──────────────────────────────

describe('Integration: tokenize vs tokenize_document', () => {
  const multiPageInput = [
    'fol.1r',
    'باب الأسد والثور',
    'قال {الملك} ديشلم *لبيدبا الفيلسوف',
    'fol.1v',
    'ومن أمثال ذلك أنّه كان بأرض',
    'fol.2r',
    '*** تقاطعا وتدابرا',
  ].join('\n');

  it('total token count matches between flat and grouped', () => {
    const flat = wasmModule.tokenize(
      multiPageInput,
      tomlConfig,
      'multiple_pages',
    );
    const doc = wasmModule.tokenize_document(
      multiPageInput,
      tomlConfig,
      'multiple_pages',
    );

    const docTotalTokens = flatTokenCount(doc.pages);
    expect(docTotalTokens).toBe(flat.tokens.length);
  });

  it('error counts match between flat and document', () => {
    const flat = wasmModule.tokenize(
      multiPageInput,
      tomlConfig,
      'multiple_pages',
    );
    const doc = wasmModule.tokenize_document(
      multiPageInput,
      tomlConfig,
      'multiple_pages',
    );

    expect(doc.errors.length).toBe(flat.errors.length);
  });

  it('document has the expected number of pages', () => {
    const doc = wasmModule.tokenize_document(
      multiPageInput,
      tomlConfig,
      'multiple_pages',
    );

    // Input has fol.1r, fol.1v, fol.2r → 3 pages
    expect(doc.pages).toHaveLength(3);
  });
});
