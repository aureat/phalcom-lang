# Current Handoff — Phalcom Updated Language Semantics

Continue this task. Original plan remains at `/Users/altunhasanli/.codex/attachments/4e37f9dd-c91d-4003-93a7-bd273d59985e/pasted-text.txt`; normative spec remains at `/Users/altunhasanli/dev/phalcom/phalcom/docs/work/pending/phalcom-updated-language-semantics-implementation-spec.md`.

## Mission and constraints

- Finish parser, compiler, VM, core-universe, pattern/loop, iterable, LSP, fixture, and verification work required by plan/spec.
- Preserve unrelated dirty worktree files. Do not reset, discard, commit, or change branch.
- Use targeted reads/writes. Run focused tests before workspace gates. Run `graphify update .` after code changes.

## Current state

Implementation is active on `main`. Worktree was already dirty before this task; task changes overlap AST/core/LSP paths. Do not classify all modified files as task-owned without reading their diff.

Completed:

- VM bytecodes/root plumbing for bilateral dispatch, exact sameness, ellipsis, Ordering validation, unsupported fallback, and runtime error kinds.
- Exact lookup-with-definer and strict-subclass helpers in `phalcom-core/src/heap/class.rs`.
- Bilateral compiler lowering with single evaluation/scratch locals; `<=>` uses `compare(_)` and reverses validated RHS Ordering.
- `===` lowers to intrinsic `Same`; user declarations of reserved `===`, `is`, and `is!` selectors are rejected.
- Number/int cooperative domain refusal returns canonical `unsupported`; numeric errors still raise.
- Object derived `< <= > >=`, Object `matches(_)`/`understands(_)`, Ellipsis, Ordering, and SelectorPattern native/runtime surfaces are present.
- Comparison-chain compiler, `if let`, `while let`, generalized pattern test/commit support, no-wrapper `for`, multi-lane strict lockstep lowering, and per-iteration pattern-local capture closing are present.
- `.indexed` and `.zipped` views are present. Indexed carries `(sourceCursor, ordinal)`; zipped carries one cursor per source and probes all lanes before strict mismatch.
- Parser trailing-closure guard prevents `if let`, `while let`, and `for` RHS expressions from consuming body `{}`.

## Evidence

- `cargo check -p phalcom-core` passed after loop/pattern changes.
- `cargo check -p phalcom-ast` passed after parser guard.
- `cargo build -p phalcom-core --bin phalcom` passed.
- CLI smoke passed for reflected `+(from)`, `<=>` Ordering and invalid compare return, `if let`, `while let`, tuple/variant patterns, indexed/zipped loops, strict lane mismatch, exact sameness, `matches`, and `understands`.

## Immediate next steps

1. Compile-check LSP. Inspect `phalcom-lsp/src/semantic/analyzer.rs` around `Expr::Binary` and `Expr::ComparisonChain`; the in-progress reflected selector construction uses `strip_suffix("(_")` and must remove full `"(_)"` to produce `name(from)`.
2. Add registered focused fixtures under `phalcom-core/tests/lang/control-flow/`, `iteration/`, and `arithmetic/`. Remove unregistered `phalcom-core/tests/lang/semantics_if_let_probe.ph` with `apply_patch`.
3. Latest unverified patch changed irrefutable record/map binding in `phalcom-core/src/compiler/lib/patterns.rs::compile_pattern_bind_from_slot` to use `get(_)`/Option checks instead of `at(_)`; compile-check and run direct `let #{...}` / `let {...}` smoke before trusting it. Then exercise variant/refutable patterns using `emit_pattern_match_tests`, `commit_pattern_bindings`, and `assign_pattern_from_slot`.
4. Check LSP spec §18: pattern definition scopes/narrowing, `matches` RHS-owned dispatch, `understands` LHS dispatch, `<=>`/bilateral candidate inference, operator occurrences/tokens. Existing dirty LSP changes already cover much AST visitor migration; preserve them.
5. Run serial gates: `cargo test -p phalcom-ast`, `cargo test -p phalcom-core`, `cargo test -p phalcom-lsp`, `cargo test --workspace`, `git diff --check`, `graphify update .`. Classify baseline/unrelated failures separately.

## Exact code map

- `phalcom-core/src/bytecode.rs`, `compiler/lib/jumps.rs`, `vm/dispatch.rs`, `vm/mod.rs`, `vm/bootstrap.rs`, `vm/gc.rs` — opcodes and semantic roots.
- `phalcom-core/src/compiler/lib/expr.rs` — bilateral lowering, comparison chains, `if let`/`while let`, scratch cleanup.
- `phalcom-core/src/compiler/lib/patterns.rs` — pattern declarations/tests/commit and required shape checks.
- `phalcom-core/src/compiler/lib/loops.rs` — loop locals, pattern lanes, strict zip control, capture closing.
- `phalcom-core/core/universe/src/collections/iterable.ph` and `collections/tuple.ph` — indexed/zipped views.
- `phalcom-core/src/primitive/number.rs`, `primitive/int.rs`, `primitive/object.rs`, `primitive/selector_pattern.rs`, `universe/primitives.rs`, `phalcom-native-surface/src/lib.rs` — native protocol.
- `phalcom-ast/src/parser.rs` — comparison grouping and trailing-closure guards.
- `phalcom-lsp/src/semantic/analyzer.rs`, `flow.rs`, `scope.rs`, `occurrence.rs`, `dispatch.rs`, `surface.rs`, `index.rs`, `selectors.rs` — semantic obligations.

## Do not re-explore

- Do not scan repository architecture broadly; graphify was already queried.
- Do not replace current Ordering source model or remove ordinary grouping parentheses.
- Do not treat focused builds/smokes as acceptance; final report must separate passing, baseline/unrelated, deferred, and unverified scope.
