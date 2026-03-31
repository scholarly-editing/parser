import { describe, it, expect, beforeAll } from 'vitest';

let wasmModule: any;

// ── Minimal TOML config for error tests ─────────────────────────────────────

const tomlConfig = `
[[page]]
prefix = "fol."
suffix = "r"

[[word]]
label = "Arabic"
char_range = [0x600, 0x6FF]
additional_chars = []
default_state = "sound"

[[prefix]]
symbol = "*"
label = "emendation"

[[suffix]]
symbol = "~"
label = "middle-arabic"

[[brackets]]
open = "{"
close = "}"
label = "suppletion"

[[tag]]
symbol = "***"
label = "lacuna"

[[block]]
start_marker = "[block]"
has_text = true
type = "WITH_TEXT"
end_marker = "---"
`;

// ── Helpers ─────────────────────────────────────────────────────────────────

type Token = Record<string, any>;

function tokenize(
  input: string,
  opts?: { mode?: string; configFormat?: string; config?: string },
): { tokens: Token[]; errors: number[] } {
  return wasmModule.tokenize(
    input,
    opts?.config ?? tomlConfig,
    opts?.mode ?? null,
    opts?.configFormat ?? null,
  );
}

function errorTokens(tokens: Token[]): any[] {
  return tokens.filter((t) => t.Error).map((t) => t.Error);
}

function errorTokensAt(tokens: Token[], indices: number[]): any[] {
  return indices.map((i) => tokens[i]).filter((t) => t?.Error).map((t) => t.Error);
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

beforeAll(async () => {
  wasmModule = await import('../parser-wasm/pkg');
});

// ── Parser error tokens ─────────────────────────────────────────────────────

describe('Parser error tokens', () => {
  it('invalid characters mid-word → WordInfixedWithUnwantedChars', () => {
    const { tokens, errors } = tokenize('كل@مة');

    expect(errors.length).toBeGreaterThan(0);
    const errs = errorTokensAt(tokens, errors);
    const infixed = errs.find(
      (e) => e.error === 'WordInfixedWithUnwantedChars',
    );
    expect(infixed).toBeDefined();
  });

  it('empty brackets {} → EmptyBrackets error', () => {
    const { tokens, errors } = tokenize('{}');

    expect(errors.length).toBeGreaterThan(0);
    const errs = errorTokensAt(tokens, errors);
    const empty = errs.find((e) => e.error === 'EmptyBrackets');
    expect(empty).toBeDefined();
  });

  it('space inside brackets { word } → SpaceInBrackets error', () => {
    const { tokens, errors } = tokenize('{ كلمة}');

    expect(errors.length).toBeGreaterThan(0);
    const errs = errorTokensAt(tokens, errors);
    const spaceErr = errs.find((e) => e.error === 'SpaceInBrackets');
    expect(spaceErr).toBeDefined();
  });

  it('invalid page break format → InvalidPageBreak error', () => {
    const { tokens, errors } = tokenize('{fol.1r}', { mode: 'multiple_pages' });

    expect(errors.length).toBeGreaterThan(0);
    const errs = errorTokensAt(tokens, errors);
    // serde serializes InvalidPageBreak(String) as { InvalidPageBreak: "..." }
    const pageErr = errs.find(
      (e) => typeof e.error === 'object' && e.error !== null && 'InvalidPageBreak' in e.error,
    );
    expect(pageErr).toBeDefined();
  });

  it('error tokens have required fields (token, line, page, span, error)', () => {
    const { tokens, errors } = tokenize('كل@مة');

    expect(errors.length).toBeGreaterThan(0);
    const errs = errorTokensAt(tokens, errors);
    expect(errs.length).toBeGreaterThan(0);

    for (const err of errs) {
      expect(err).toHaveProperty('token');
      expect(err).toHaveProperty('line');
      expect(err).toHaveProperty('page');
      expect(err).toHaveProperty('span');
      expect(err).toHaveProperty('error');
    }
  });

  it('errors array indices point to Error tokens', () => {
    const { tokens, errors } = tokenize('{}');

    expect(errors.length).toBeGreaterThan(0);
    for (const idx of errors) {
      expect(idx).toBeLessThan(tokens.length);
      expect(tokens[idx]).toHaveProperty('Error');
    }
  });

  it('word prefixed with unwanted chars → WordPrefixedWithUnwantedChars', () => {
    const { tokens, errors } = tokenize('@كلمة');

    expect(errors.length).toBeGreaterThan(0);
    const errs = errorTokensAt(tokens, errors);
    const prefixed = errs.find(
      (e) => e.error === 'WordPrefixedWithUnwantedChars',
    );
    expect(prefixed).toBeDefined();
  });

  it('word suffixed with unwanted chars → WordSuffixedWithUnwantedChars', () => {
    const { tokens, errors } = tokenize('كلمة@');

    expect(errors.length).toBeGreaterThan(0);
    const errs = errorTokensAt(tokens, errors);
    const suffixed = errs.find(
      (e) => e.error === 'WordSuffixedWithUnwantedChars',
    );
    expect(suffixed).toBeDefined();
  });
});

// ── API error handling ──────────────────────────────────────────────────────

describe('API error handling', () => {
  it('invalid TOML config string → throws JS Error', () => {
    expect(() =>
      wasmModule.tokenize('كلمة', 'not valid toml {{{{', null, null),
    ).toThrow();
  });

  it('invalid mode string → throws JS Error', () => {
    expect(() =>
      wasmModule.tokenize('كلمة', tomlConfig, 'invalid_mode', null),
    ).toThrow(/Invalid mode/);
  });

  it('config_format "json" with TOML content → throws JS Error', () => {
    expect(() =>
      wasmModule.tokenize('كلمة', tomlConfig, null, 'json'),
    ).toThrow(/JSON config error/);
  });

  it('empty input → returns empty tokens, no error', () => {
    const { tokens, errors } = tokenize('');

    expect(tokens).toHaveLength(0);
    expect(errors).toHaveLength(0);
  });

  it('empty config string → throws a meaningful error', () => {
    expect(() =>
      wasmModule.tokenize('كلمة', '', null, null),
    ).toThrow();
  });

  it('invalid config_format value → throws JS Error', () => {
    expect(() =>
      wasmModule.tokenize('كلمة', tomlConfig, null, 'csv'),
    ).toThrow(/Invalid config_format/);
  });
});

// ── Passage error handling ──────────────────────────────────────────────────

describe('Passage error handling', () => {
  it('createPassage with invalid config → throws JS Error', () => {
    expect(() =>
      wasmModule.createPassage('كلمة', 'not valid config {{', null),
    ).toThrow();
  });

  it('locate_fragment with text not in passage → returns (-1, -1)', () => {
    const passage = wasmModule.createPassage('باب الأسد', tomlConfig, null);
    const loc = passage.locate_fragment('غائب');

    expect(loc[0]).toBe(-1);
    expect(loc[1]).toBe(-1);
  });

  it('locate_fragment with text containing errors → returns (-1, -1)', () => {
    // Passage with errors is marked invalid; words list is empty
    const passage = wasmModule.createPassage('كل@مة', tomlConfig, null);

    expect(passage.is_valid()).toBe(false);
    const loc = passage.locate_fragment('كلمة');
    expect(loc[0]).toBe(-1);
    expect(loc[1]).toBe(-1);
  });
});
