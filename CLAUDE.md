# Phalcom

Phalcom is a class-based, object-oriented programming language implemented in Rust.
It compiles source to bytecode and executes it on a stack-based VM, with a Smalltalk-style
object model (classes, metaclasses, message-send method dispatch) and a bootstrap core
library written in Phalcom itself.

## Knowledge sources — read these BEFORE sweeping the codebase

This repo maintains a layered memory so sessions don't re-scan the tree from scratch:

1. **This file (`CLAUDE.md`)** — the always-loaded map: layout, build/run/test commands, conventions.
2. **Structural graph (graphify)** — code knowledge graph at `graphify-out/graph.json`.
   Use it for "where is X defined / what calls it / what breaks if I change it":
   - `graphify query "how does method dispatch work" --budget 2000`
   - `graphify explain "<node>"` — a node and its neighbors
   - `graphify affected "<symbol>"` — reverse impact of a change
   - `graphify path "A" "B"` — how two symbols connect
   - Rebuild after edits (no LLM): `graphify update . --no-cluster`
3. **Episodic memory (claude-mem / mem-search)** — for "*why* did we decide X / did we hit
   this before". Design rationale (object model, metaclass tower) lives here, not in code.

Rule of thumb: **structure → graphify, intent/decisions → mem-search, orientation → this file.**

## Workspace layout

Cargo workspace (edition 2024, resolver 2). Members:

| Crate | Role |
|---|---|
| `phalcom-ast` | Front end: lexer, tokens, AST, parse/lex errors. |
| `phalcom-common` | Shared primitives: source ranges, ref/pointer helpers. |
| `phalcom-core` | The language runtime: compiler (AST→bytecode), bytecode/VM, object model, core-class primitives, and the `phalcom` CLI binary. |
| `phalcom-repl` | Interactive REPL (line editing, completion, highlighting). |

### `phalcom-ast/src`
- `lexer.rs`, `token.rs` — lexing to a token stream.
- `ast.rs` — AST node definitions.
- `error.rs` — lex/parse diagnostics. `util.rs`, `build.rs` — support/codegen.

### `phalcom-common/src`
- `range.rs` — source spans/ranges. `refs.rs` — reference/handle helpers.

### `phalcom-core/src`
- **Compiler:** `compiler/` (`lib.rs`, `mod.rs`) — lowers AST to bytecode.
- **Bytecode & execution:** `bytecode.rs`, `chunk.rs`, `vm.rs`, `frame.rs`, `interpret.rs`.
- **Object model:** `value.rs`, `instance.rs`, `class.rs`, `method.rs`, `signature.rs`,
  `callable.rs`, `closure.rs`, and immediate types `boolean.rs`, `nil.rs`, `string.rs`.
- **Runtime state:** `universe.rs` (globals + bootstrap), `interner.rs` (symbol interning),
  `module.rs`, `diagnostics.rs`, `error.rs`.
- **Primitives (`primitive/`):** native Rust method implementations per core class —
  `object.rs`, `class.rs`, `method.rs`, `module.rs`, `number.rs`, `string.rs`, `symbol.rs`,
  `boolean.rs`, `nil.rs`, `system.rs`.
- **CLI (`bin/phalcom/`):** `main.rs` (entry), `cli.rs` (arg handling), `disasm.rs` (bytecode disassembler).

### `phalcom-core/core/core.ph`
Bootstrap core library written in Phalcom, loaded at startup to define base classes.

### `phalcom-repl/src`
REPL built on `reedline`. The whole crate is one binary rooted at `main.rs`, which declares
exactly four modules: `common.rs` (shared keyword table), `completer.rs`, `highlighter.rs`,
`repl.rs`. There is no `lib.rs` and no second editor stack — the former `rustyline/`
subdirectory and the orphaned `editor.rs`/`helper.rs` were deleted per U-REPL §S8 once they
were found to be declared by nothing and therefore never compiled.

## Build / run / test

```sh
cargo build                              # build the whole workspace
cargo run -p phalcom-core --bin phalcom  # run the phalcom CLI (interpreter)
cargo run -p phalcom-repl                # start the REPL
cargo test                               # run tests
cargo clippy --workspace                 # lints
```

Example programs live in `examples/*.ph` (e.g. `simple.ph`, `calculator.ph`, `person*.ph`).
The object-model design spec is `docs/spec/v0.2/object-model.md`.

## Conventions

- **Documentation is mandatory and is the default way code is written here.** All Rust code
  ships with professional rustdoc: a `//!` doc on every crate/module and a `///` doc on every
  public item (incl. fields, enum variants), following Rust's official conventions. The full
  rule + enforcement (`#![warn(missing_docs)]`, `cargo doc` clean, reviewers block on missing
  docs) is in [`docs/rust-documentation-guidelines.md`](docs/rust-documentation-guidelines.md).
  Undocumented public API is an incomplete change.
- Rust 2024 edition across all crates; shared deps are pinned in the root `[workspace.dependencies]`.
- Errors use `thiserror` + `miette` (fancy diagnostics); prefer surfacing spans via `phalcom-common` ranges.
- The object model follows Smalltalk-style semantics; method lookup keys on signature symbols
  (arity + kind encoded), so `foo` and `foo(_)` can coexist. See `docs/spec/v0.2/object-model.md` for the
  target design and current deviations before changing class/metaclass wiring.

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
- Community naming (`graphify label` / `cluster-only`) defaults to the `claude-cli` backend via a fish wrapper at `~/.config/fish/functions/graphify.fish`, billing to the Claude plan (no API key). A real LLM API key, if set, takes precedence; `graphify update` does no labeling and reuses existing names.
