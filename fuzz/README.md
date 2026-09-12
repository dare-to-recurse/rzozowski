# Cargo-fuzz targets

The five targets exercise the public regex API with inputs mutated by libFuzzer.
They use no checked-in seeds, smoke tests, or shared fuzz library.

| Target | Checks |
| --- | --- |
| `parse` | Parses arbitrary UTF-8 patterns to find lexer and parser panics. |
| `differential` | Compares whole-string matching with the `regex` crate for patterns in their shared syntax. |
| `algebra` | Checks simplification, nullability, and the derivative law on parsed patterns. |
| `display` | Formats successfully parsed patterns. |
| `ranges-counts` | Checks count ranges and character ranges against scalar comparisons. |

`parse` consumes raw bytes when they are valid UTF-8. The other targets use
`libfuzzer-sys` to decode mutated bytes into strings or scalar values. Invalid
patterns are normal inputs and are skipped by targets that need a parsed regex.
Pattern and text lengths are bounded to keep each iteration useful.

The `differential` target only compares patterns made from ASCII alphanumeric
characters and `()|*+?`, where the two regex dialects agree. Its reference uses
`\A(?:pattern)\z` to match the entire string. Patterns rejected by either parser
are skipped.

## Running

Install a nightly Rust toolchain and cargo-fuzz. A C++ compiler is also required;
see the [cargo-fuzz setup guide](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html)
for supported platforms. Install `rust-src` for cargo-fuzz versions that build
the standard library with instrumentation:

```sh
rustup toolchain install nightly --component rust-src
cargo install cargo-fuzz --locked
```

From the repository root, with a nightly toolchain active:

```sh
cargo fuzz run <target> --jobs=<n>
```

For example, `cargo fuzz run differential --jobs=4` runs four workers. Use
`cargo fuzz list` to list the targets. Runs are unbounded by default; pass
libFuzzer options after `--` to set limits:

```sh
cargo fuzz run parse --jobs=4 -- -max_total_time=60 -timeout=5 -max_len=512
```

Cargo-fuzz keeps working inputs in `fuzz/corpus/<target>` and failures in
`fuzz/artifacts/<target>`. Both directories are ignored by Git. When a target
finds a failure, minimize the artifact and add a focused regression test under
the main crate's `tests/` directory. The separate fuzz workspace keeps
libFuzzer's native dependencies out of normal library builds.

To format and lint the fuzz package:

```sh
cargo fmt --manifest-path fuzz/Cargo.toml -- --check
cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings
```
