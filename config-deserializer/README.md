# `config-deserializer`

This crate reads parser configuration from TOML and converts it to the Rust `ParserConfig` used by the core library. The same schema is used by the CLI. The WebAssembly binding can also parse JSON, YAML, and XML configurations.

## Configuration sections

The TOML schema supports these top-level collections:

| Section | Purpose |
| --- | --- |
| `word` | Word label, Unicode code-point range, additional code points, and default state |
| `prefix`, `suffix` | Editorial symbols and the state labels they assign |
| `brackets` | Opening/closing markers, label, and optional `skip` behavior |
| `tag` | Standalone tag symbols and labels |
| `page` | Page-marker prefix and suffix |
| `block` | Block start marker, `STANDALONE` or `WITH_TEXT` type, and optional end marker |

The current deserializer requires `word`, `prefix`, `brackets`, and `tag` collections, even when some are empty. `page`, `block`, and `suffix` are optional. `precedence` is optional; smaller values are tried first when parser rules overlap. A word's `char_range` contains its first and last Unicode code points, and `additional_chars` lists other accepted code points.

For example:

~~~toml
prefix = []
suffix = []
brackets = []
tag = []

[[word]]
label = "Arabic"
char_range = [0x0600, 0x06FF]
additional_chars = []
default_state = "sound"
~~~

## Included examples

- [TOML](config.example.toml)
- [JSON](config.example.json)
- [YAML](config.example.yaml)
- [XML](config.example.xml)

All four files show the shared configuration shape. `ParserConfigDeserializer` currently loads TOML through `from_toml_file` and `from_toml_str`; JSON, YAML, and XML parsing is implemented in the WASM wrapper.

## Tests

Run this crate's tests from the workspace root:

~~~sh
cargo test -p config-deserializer
~~~
