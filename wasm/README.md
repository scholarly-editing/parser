# @scholarly-editing/parser

WebAssembly parser for scholarly editing texts and manuscripts.

The `parser-wasm` crate exposes the Rust parser to JavaScript/TypeScript through `wasm-bindgen`. It includes tokenization, page-grouped document results, and passage lookup.

## Installation

~~~sh
npm install @scholarly-editing/parser
# or
pnpm add @scholarly-editing/parser
# or
yarn add @scholarly-editing/parser
~~~

## Build & Test

Requirements: Rust, Node.js, and pnpm 9.11.0.

~~~sh
pnpm install
pnpm run build
pnpm test
~~~

## Usage

~~~javascript
import { tokenize, tokenize_document, createPassage } from "@scholarly-editing/parser";

const tomlConfig = [
  "prefix = []",
  "brackets = []",
  "tag = []",
  "",
  "[[word]]",
  "label = \"Arabic\"",
  "char_range = [0x0600, 0x06FF]",
  "additional_chars = []",
  "default_state = \"sound\"",
].join("\n");

const result = tokenize(
  "باب الأسد والثور",
  tomlConfig,
  "single_page",
  "toml",
);

console.log(result.tokens);
console.log(result.errors); // indexes into result.tokens
~~~

## API

- `tokenize(input, config, mode?, config_format?)` returns `{ tokens, errors }`.
  - `mode`: `"single_page"` | `"multiple_pages"` | `"passage"` (default: `"single_page"`)
  - `config_format`: `"toml"` | `"json"` | `"yaml"` | `"xml"` (default: `"toml"`)
- `tokenize_document(input, config, mode?, config_format?)` returns `{ pages, errors }`, grouping tokens by page. Error indexes refer to the original flat token list.
- `createPassage(input, config, config_format?)` returns a passage object with `is_valid()`, `get_raw_passage()`, and `locate_fragment(fragment)`.

The configuration must use the schema in [`config-deserializer`](../config-deserializer/README.md). Ready-to-adapt examples are available as [TOML](../config-deserializer/config.example.toml), [JSON](../config-deserializer/config.example.json), [YAML](../config-deserializer/config.example.yaml), and [XML](../config-deserializer/config.example.xml).

The wrapper normalizes text to Unicode NFC before `tokenize` and `tokenize_document`. Invalid modes, config formats, and configuration text are returned to JavaScript as errors.
