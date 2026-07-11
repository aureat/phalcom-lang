# Fuzzing Phalcom

Coverage-guided fuzzing of the language front end with
[`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) (libFuzzer).

The goal is simple and strict: **no input, however malformed, may make the
lexer or parser panic.** They must always return tokens / `Ok(Program)` /
`Err(...)`. A crash found here is a bug.

## Targets

| Target   | Entry point                     | What it checks                          |
| -------- | ------------------------------- | --------------------------------------- |
| `lexer`  | `phalcom_ast::lexer::Lexer`     | Tokenizing arbitrary UTF-8 never panics |
| `parser` | `phalcom_ast::parse_source`     | Parsing arbitrary UTF-8 never panics    |

## Prerequisites

`cargo-fuzz` requires a **nightly** toolchain (libFuzzer needs sanitizer flags
that are nightly-only):

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
```

This `fuzz/` crate is intentionally **not** part of the root workspace (note the
empty `[workspace]` table in `Cargo.toml`), so it never affects
`cargo build`/`cargo test` at the repo root.

## Running

From the repository root:

```sh
# Fuzz the parser, seeded with the token dictionary:
cargo +nightly fuzz run parser -- -dict=fuzz/phalcom.dict

# Fuzz the lexer:
cargo +nightly fuzz run lexer -- -dict=fuzz/phalcom.dict

# Time-boxed run (e.g. in CI or a smoke check): 60 seconds
cargo +nightly fuzz run parser -- -dict=fuzz/phalcom.dict -max_total_time=60
```

Seed the corpus with real programs for a faster start:

```sh
mkdir -p fuzz/corpus/parser
cp examples/*.ph fuzz/corpus/parser/
```

## When a crash is found

`cargo-fuzz` writes the offending input to `fuzz/artifacts/<target>/`. Reproduce
and minimize it:

```sh
cargo +nightly fuzz run parser fuzz/artifacts/parser/crash-<hash>
cargo +nightly fuzz tmin parser fuzz/artifacts/parser/crash-<hash>
```

Then add the minimized input as a regression test in `phalcom-ast/tests/`.

## Notes

- `phalcom.dict` is derived from `phalcom-ast/src/token.rs`; keep it in sync when
  keywords or operators change.
- The corpus (`fuzz/corpus/`) and artifacts (`fuzz/artifacts/`) are git-ignored.
