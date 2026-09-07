# Handoff — Phalcom Updated Language Semantics

Continue this task from handoff below. Original/current spec is attached and remains authoritative:
`/Users/altunhasanli/.codex/attachments/09089d77-2270-4964-9e40-27bf2d0bba79/pasted-text.txt`.
Local normative specification:
`/Users/altunhasanli/dev/phalcom/phalcom/docs/work/pending/phalcom-updated-language-semantics-implementation-spec.md`.

## Mission and scope

- Deliver coordinated parser, AST, compiler, VM, core-universe, Ordering, pattern, iterable, LSP, migration, and verification work from attached plan.
- Preserve user constraints: strict zip default; exactly no-wrapper control headers; existing parentheses remain grouped expressions during migration; adapt existing Ordering; write focused tests during implementation; perform migration and final gates last.
- Preserve unrelated dirty files. Do not reset, discard, commit, or change branch unless explicitly requested.

## Current state

- Status: implementation in progress. AST/parser foundation and partial compiler/runtime/LSP surface are present; bilateral dispatch, runtime fixtures, full migration, and final verification remain.
- Branch: `main`.
- Initial unrelated dirty paths existed before this work, including `docs/.obsidian/workspace.json`, `examples/symbolic/src/torture.ph`, selected LSP/native/type-syntax files, and VS Code manual assets. Treat ownership as mixed where this task also touched a path; inspect diff before editing.
- Current new files: `phalcom-core/core/universe/src/object/ellipsis.ph`, `phalcom-core/core/universe/src/object/ordering.ph`, `phalcom-core/src/primitive/selector_pattern.rs`, plus the local implementation spec and this handoff.
- Run Cargo commands serially. Shared target artifacts previously caused stale/race symptoms; targeted clean/rebuild was used.

## Completed and verified

- Lexer/AST/parser recognize `===`, `<=>`, `...`, comparison chains, `if let`, `while let`, no-wrapper `for` lanes, tuple/list/record/map/variant patterns, and contextual `matches`/`understands`. Grouped conditions still parse through parentheses.
- Core compiler has initial comparison-chain lowering, `if let`/`while let` lowering, multi-lane strict-zip lowering, pattern test/bind helpers, `Value::same_as`, Object `===`/`matches`/`understands`, Number `<=>`, and SelectorPattern registration.
- Core universe bootstrap includes Ellipsis and adapted existing Ordering implementation. `record.get` was added for safe record-pattern lookup. Simple legacy `for (name in expr)` headers in `phalcom-core/core/**/*.ph` were mechanically migrated.
- LSP/module/native-surface arms were updated enough for `cargo check -p phalcom-lsp` to pass.
- Verified: `cargo check -p phalcom-core` passed after current pattern/if-let changes.
- Verified: `cargo check -p phalcom-lsp` passed.
- Verified: `cargo test -p phalcom-core universe::tests::kernel_number_leaf_tostring_override_flips_the_fast_path` passed after bootstrap/source changes.
- Parser regression tests were added at `phalcom-ast/src/parser.rs:5231-5284`; they still need focused execution.

## Known unfinished or risky areas

- No bilateral VM/compiler dispatch exists yet. `phalcom-core/src/bytecode.rs`, `phalcom-core/src/vm/dispatch.rs`, `phalcom-core/src/vm/send.rs`, `phalcom-core/src/heap/class.rs`, and runtime error definitions still need exact direct/reflected dispatch, strict-subtype reflected preference, canonical unsupported decline, error propagation, and `<=>` Ordering reversal/validation.
- `phalcom-core/src/compiler/lib/patterns.rs:56` and `:363`: refutable pattern infrastructure exists, but irrefutable Record/Map binding still follows old `at`-style access in places, and Variant binding does not yet validate constructor/class shape. Exercise with runtime fixtures before claiming completion.
- `phalcom-core/src/compiler/lib/loops.rs:218`: strict multi-lane lowering is implemented but stack/close/error behavior is unverified. Explicit `.indexed` and `.zipped` iterable view classes are not yet implemented.
- `phalcom-core/src/compiler/lib/expr.rs:1427` and `:1465`: `if let`/`while let` lowering compiles but needs runtime fixtures for scope visibility, failure values, break/continue, and captured locals.
- Ellipsis currently lowers through global `ellipsis`; semantic-root handling and user-facing runtime behavior need confirmation.
- `===` native behavior exists, but non-overridability/reserved-selector enforcement is not yet established.
- Legacy syntax remains outside migrated core sources. Do not run blanket replacement without inspecting nested parentheses, tuple patterns, and method-call syntax.
- Existing LSP lib tests previously showed three failures in analyzer/core syntax/loop fixpoint areas; classification against baseline/user dirty changes is pending. Do not call full LSP acceptance from prior focused checks.

## Targeted code map

- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-ast/src/token.rs`, `lexer.rs`, `ast.rs`, `parser.rs` — syntax/token and AST implementation.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/compiler/lib/expr.rs` — binary/comparison-chain/control-expression lowering.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/compiler/lib/patterns.rs` — pattern declaration, tests, extraction, and binding.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/compiler/lib/loops.rs` — plain and multi-lane `for` lowering.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/bytecode.rs`, `chunk.rs`, `compiler/lib/jumps.rs`, `vm/dispatch.rs` — add and route bilateral bytecode if using bytecode design.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/vm/send.rs`, `heap/class.rs`, `value/mod.rs`, `error.rs` — exact lookup, hierarchy/subtype checks, nested sends, and final errors.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/primitive/object.rs`, `primitive/number.rs`, `primitive/selector_pattern.rs`, `universe/primitives.rs` — native protocol.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/core/universe/src/object/ordering.ph`, `ellipsis.ph`, `package.ph` — core source definitions/bootstrap exports.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-lsp/src/semantic/*`, `completion.rs`, `index.rs`, `selectors.rs`, `semantic_tokens.rs` — semantic surface updates.

## Resume plan

1. Read attached spec sections on bilateral dispatch, exact sameness, Ordering, and iterable/pattern semantics. Then inspect only `bytecode.rs`, `chunk.rs`, `compiler/lib/jumps.rs`, `vm/dispatch.rs`, `vm/send.rs`, `heap/class.rs`, and `error.rs` at symbols named above.
2. Implement bilateral dispatch with operands evaluated once left-to-right. Direct exact selector first; reflected selector only on canonical unsupported decline; strict RHS subtype may prefer reflected. Preserve raised errors. For `<=>`, reverse only a validated Ordering result.
3. Add focused language/runtime fixtures for `===`, arithmetic/reflected dispatch, unsupported operands, numeric domain refusal versus numeric errors, `<=>`, Ellipsis, `if let`, `while let`, structured patterns, and strict zip. Register tests before broad migration.
4. Repair pattern extraction/shape checks and implement explicit indexed/zipped iterable views using existing collection conventions. Re-run focused fixtures after each change.
5. Finish LSP semantic/surface handling and classify the three known LSP failures against clean baseline.
6. Migrate remaining legacy Phalcom sources/tests/examples carefully. Run parser/core/LSP focused tests, then required workspace gates, `git diff --check`, and finally `graphify update .`.

## Do not re-explore

- Do not restart from repository-wide architecture scans; graphify was already queried for this task.
- Do not replace the existing Ordering implementation with a new model; adapt `/phalcom-core/core/universe/src/object/ordering.ph`.
- Do not remove grouping parentheses from ordinary expressions or reintroduce wrapper control headers. `for (x in xs)` must remain rejected while `(condition)` remains a group expression.
- Do not treat focused compile/test passes as full acceptance. Report passing, baseline/unrelated, deferred, and unverified scope separately.
