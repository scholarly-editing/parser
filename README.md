# Scholarly Editing Parser

A configurable parser for structured manuscript transcriptions and scholarly editions. It turns text and editorial markup into typed tokens, preserves page and block structure, and reports malformed or unrecognized text with locations.

The core is written in Rust and is available as a command-line program, a Python extension, and a WebAssembly package. The included examples focus on Arabic manuscripts; word character ranges and editorial conventions are configurable for other projects.

## What it can parse

- Words selected by Unicode character ranges, with configurable labels and default states
- Editorial prefixes and suffixes, tags, and bracketed text
- Page breaks and named blocks, including blocks with text
- Single-page, multiple-page, and passage parsing modes
- Structured tokens and error locations that can be serialized as JSON

## Workspace

| Component | Purpose |
| --- | --- |
| [`parser`](parser/README.md) | Rust parsing library and token model |
| [`config-deserializer`](config-deserializer/README.md) | TOML configuration deserialization and sample configurations |
| [`cli`](cli/README.md) | `sep` command-line program |
| [`python/py-parser`](python/py-parser/README.md) | Python `Passage` binding |
| [`wasm`](wasm/README.md) | JavaScript/WebAssembly API and Vitest suite |

## Quick start

Install the Rust toolchain, then save UTF-8 text like this as `manuscript.txt`. The example configuration recognizes Arabic words and `fol.1r`-style page markers:

~~~text
fol.1r
باب الأسد والثور
~~~

From the repository root, run:

~~~sh
cargo run -p sep -- \
  --config config-deserializer/config.example.toml \
  --input manuscript.txt \
  --mode multiple-pages
~~~

The CLI writes `sep-output/manuscript.result.json` with the tokens and summary, and `sep-output/manuscript.errors.json` with any parsing errors. See the [CLI guide](cli/README.md) for options and defaults.

## Development

Run the Rust workspace tests with:

~~~sh
cargo test --workspace
~~~

The WebAssembly tests use the Node target:

~~~sh
cd wasm
pnpm install
pnpm run build:parser-node
pnpm test
~~~

See the component guides for setup and API examples. The project is licensed under [GPL-3.0-or-later](LICENSE).
