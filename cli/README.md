# `sep` command-line tool

`sep` reads UTF-8 transcription files, tokenizes them with a parser configuration, and writes a result JSON and an error JSON for each input.

## Build and run

From the repository root:

~~~sh
cargo build -p sep --release
~~~

The example configuration recognizes Arabic words and `fol.1r` / `fol.1v` page markers. Put your text in a `.txt` file, then run:

~~~sh
cargo run -p sep -- \
  --config config-deserializer/config.example.toml \
  --input manuscript.txt \
  --mode multiple-pages
~~~

The CLI currently reads TOML configurations. Start from [`config.example.toml`](../config-deserializer/config.example.toml) and adjust the word ranges and editorial markers for your source.

## Options and defaults

| Option | Default | Description |
| --- | --- | --- |
| `--config`, `-c` | `./sep-config.toml` | TOML parser configuration |
| `--input`, `-i` | All `.txt` files in the current directory | One or more input files |
| `--output-dir`, `-o` | `./sep-output` | Directory for JSON output |
| `--mode`, `-m` | `multiple-pages` | `single-page`, `multiple-pages`, or `passage` |

If no input paths are given, the CLI processes `.txt` files in the current directory in sorted order. The output directory is created when needed. Use `cargo run -p sep -- --help` to see the command's help text.

## Output files

For `manuscript.txt`, the CLI writes:

- `sep-output/manuscript.result.json`: tokens, error indexes, token/error counts, and parsing position summary.
- `sep-output/manuscript.errors.json`: the error records referenced by those indexes.

The process prints a per-file summary and exits with a nonzero status if an input file cannot be processed. Parsing errors are included in the JSON output and summary.
