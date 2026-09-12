# Cargo-fuzz targets

The five targets exercise the public regex API with inputs mutated by libFuzzer.
They use no checked-in seeds. A small shared generator turns arbitrary bytes into
valid, bounded patterns and matching text, so mutations can reach nested groups,
classes, escapes, Unicode, and quantifiers without first producing valid syntax.

| Target | Checks |
| --- | --- |
| `parse` | Parses raw UTF-8 and generated valid patterns, including native escapes. |
| `differential` | Compares whole-string matching with the `regex` crate on both legacy inputs and generated patterns with known matches and near misses. |
| `algebra` | Checks parsed patterns and compares directly constructed, unsimplified trees against an independent bounded language matcher. |
| `display` | Formats parsed and directly constructed trees and checks parseable output against the original expression. |
| `ranges-counts` | Checks exact, bounded, and lower-bound repetitions at count boundaries, plus character ranges at their endpoints. |

`parse` still consumes raw bytes when they are valid UTF-8. `differential`,
`algebra`, and `display` still decode raw bytes with the same `Arbitrary` string
types used by their previous versions, preserving the meaning of existing
corpus entries. They also feed the unmodified bytes to the generator. The
`ranges-counts` input format is unchanged. Inputs and generated structures are
bounded to keep each iteration useful.

The legacy `differential` check still accepts only ASCII alphanumeric
characters and `()|*+?`. Its generated path uses a larger shared subset,
including character classes, escaped metacharacters, counted repetitions, and
Unicode literals. It excludes `\d`, `\w`, and `\s`, whose definitions differ
between the two crates. The reference uses `\A(?:pattern)\z` to match the
entire string.

The `algebra` target's reference matcher splits text at UTF-8 character
boundaries and tracks reachable positions for repetition. Unlike checking a
derivative against `Regex::matches`, this oracle does not call the code being
tested. `display` only reparses output without `Empty` or `Epsilon` nodes,
because those intentionally display as mathematical symbols.

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
cargo test --manifest-path fuzz/Cargo.toml
```
