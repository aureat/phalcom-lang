# Fuzzing Phalcom

Coverage-guided fuzzing of the language pipeline with
[`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) (libFuzzer).

The front-end targets require malformed input to return tokens or `Err(...)`
without panicking. The VM target additionally compiles and executes input,
forces a collection, and checks object-model invariants. Expected compile and
runtime errors are normal; panics, sanitizer failures, timeouts, and invariant
failures are bugs.

## Targets

| Target   | Entry point                     | What it checks                          |
| -------- | ------------------------------- | --------------------------------------- |
| `lexer`  | `phalcom_ast::lexer::Lexer`     | Tokenizing arbitrary UTF-8 never panics |
| `parser` | `phalcom_ast::parse_source`     | Parsing arbitrary UTF-8 never panics    |
| `vm`     | `phalcom_core::vm::VM`          | Compile, execute, GC, invariants        |

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

# Fuzz compiler, VM, heap, and GC. Use generated parser inputs plus valid
# language-test programs as seeds:
PHALCOM_GC_STRESS=1 cargo +nightly fuzz run vm \
  fuzz/corpus/parser phalcom-core/tests/lang -- \
  -dict=fuzz/phalcom.dict \
  -max_total_time=600 \
  -max_len=8192 \
  -timeout=2 \
  -print_final_stats=1

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
