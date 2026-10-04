# WebAssembly and JavaScript

The `parser-wasm` crate exposes the Rust parser to JavaScript through `wasm-bindgen`. It includes tokenization, page-grouped document results, and passage lookup.

## Build

Requirements: Rust, `wasm-pack`, Node.js, and pnpm 9.11.0. From this directory, build a Node package for tests:

~~~sh
pnpm install
pnpm run build:parser-node
pnpm test
~~~

For a browser bundler target, run `pnpm run build:parser-web`. The build commands generate the package under `parser-wasm/pkg`.

## API

`tokenize(input, config, mode?, config_format?)` returns `{ tokens, errors }`. The optional mode is `single_page`, `multiple_pages`, or `passage`; it defaults to `single_page`. The optional config format is `toml`, `json`, `yaml`, or `xml`; it defaults to `toml`.

~~~javascript
import * as wasm from "./parser-wasm/pkg";

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

const result = wasm.tokenize(
  "باب الأسد والثور",
  tomlConfig,
  "single_page",
  "toml",
);
console.log(result.tokens);
console.log(result.errors); // indexes into result.tokens
~~~

The configuration must use the schema in [`config-deserializer`](../config-deserializer/README.md). Ready-to-adapt examples are available as [TOML](../config-deserializer/config.example.toml), [JSON](../config-deserializer/config.example.json), [YAML](../config-deserializer/config.example.yaml), and [XML](../config-deserializer/config.example.xml).

`tokenize_document(input, config, mode?, config_format?)` returns `{ pages, errors }`, grouping tokens by page. Its error indexes refer to the original flat token list. `createPassage(input, config, config_format?)` returns a passage object with `is_valid()`, `get_raw_passage()`, and `locate_fragment(fragment)`.

The wrapper normalizes text to Unicode NFC before `tokenize` and `tokenize_document`. Invalid modes, config formats, and configuration text are returned to JavaScript as errors.
