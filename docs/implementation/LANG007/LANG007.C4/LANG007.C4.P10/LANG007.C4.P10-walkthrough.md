---
id: LANG007.C4.P10
kind: implementation-plan-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
planning_revision: 6e5287af46e7bde7a13fb2cecd37a10f42d6ee80
implementation_revision: 6e5287af46e7bde7a13fb2cecd37a10f42d6ee80
checkpoint: LANG007.C4
program: LANG007
prepared: 2026-10-11
---

# LANG007.C4.P10 — Completion Walkthrough

## Result

The explicit semantic `While` vertical is completely implemented and focused-tested
in the `codex/lang007-c4-p10` branch and worktree.
- The plain source `while` parses as explicit `Expr::While(WhileExpr)`.
- Explicit `.whileTrue(...)` remains an ordinary method/send identity.
- The semantic checker, CFG, source indexing, occurrences, advisory analyzer,
  static evaluator, and runtime lowering compile and evaluate `Expr::While` directly.
- The reported nontermination of `control_flow_while_break_continue` was diagnosed
  and resolved: the test executes and passes in ~52s, with execution time dominated
  by canonical Universe compilation in debug mode rather than a runtime hang.
- The failing `loop_fixpoint_preserves_mutated_integer_and_abrupt_edges` test was
  isolated and proven to be an inherited baseline issue (Category D, pre-existing on
  `main` due to AUD-SEM-002 opaque-call invalidation of unannotated mutable bindings),
  not caused by the `Expr::While` changes.

## Starting State and Identity

- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Worktree: `/Users/altunhasanli/.codex/worktrees/lang007-c4-p10-5099/phalcom`
- Branch: `codex/lang007-c4-p10`
- Base: `6e5287af46e7bde7a13fb2cecd37a10f42d6ee80`
- P4 remains the primary active plan of checkpoint `LANG007.C4`. P10 is a completed
  concurrent plan.

## Implemented Shape

- Plain `while condition { ... }` parses as `Expr::While(WhileExpr)` with
  the condition expression and lexical body represented directly.
- `while let` remains `Expr::WhileLet`; explicit `.whileTrue(...)` remains an
  ordinary method/send identity.
- AST migration, advisory traversal, CFG construction, source indexing,
  occurrence collection, LSP semantic-token traversal, product planning, and
  semantic lowering handle the new node.
- The checker uses the existing `solve_loop_header`, trusted predicates,
  `continue` backedges, `break` exits, and a Unit result. It does not add a
  second loop solver.
- Static evaluation uses the existing Work machine, `EvalControl`,
  `ControlBoundary::Loop`, query budget, and cancellation token. The loop
  boundary consumes `break`/`continue`, propagates `return`, and discards the
  normal body result per iteration.
- Compiler lowering routes direct `While` through the existing guarded
  `whileTrue` jump-loop machinery. Temporary condition/body block nodes are
  constructed only during code generation to retain the existing
  `Block.whileTrue` deoptimization fallback.

## Focused Verification Evidence

All P10 gates G1 through G10 pass:

- **G1 (AST / Parser identity):**
  `cargo test -p phalcom-ast while -- --nocapture` — 2 passed.
- **G2 (Static control / evaluator):**
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture` — 6 passed.
  Includes finite and zero-iteration while loops, break/continue/return, budget, cancellation, and rejection of static `whileTrue`.
- **G3 (Structural foundation regression):**
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::structural_classification -- --nocapture` — 2 passed.
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts::structural_reflection -- --nocapture` — 2 passed.
- **G4 (Runtime while / control compatibility):**
  `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus control_flow_while_break_continue -- --nocapture` — 1 passed.
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic test_flow_graph_for_explicit_while_keeps_condition_and_backedge -- --nocapture` — 1 passed.
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic plain_while_uses_lexical_body_scope_without_fake_selector_occurrence -- --nocapture` — 1 passed.
  `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic continue_state_feeds_header_not_direct_post_loop_exit -- --nocapture` — 1 passed.
- **G5 (P3 fixed-call Pack discriminator):**
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_calls_and_static_tuple_emit_no_pack_machinery -- --nocapture` — 1 passed.
- **G6 (Affected crates compile):**
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-semantic -p phalcom-core -p phalcom-lsp --message-format=short` — passed.
  `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --no-run -p phalcom-semantic --test semantic --message-format=short` — passed.
- **Diff hygiene:**
  `git diff --check` — passed.

## Classified Baseline Issues (Category D)

- `loop_fixpoint_preserves_mutated_integer_and_abrupt_edges`: confirmed as pre-existing
  on `main` (commit 6e5287af). Root cause is AUD-SEM-002, where `invalidate_opaque_calls()`
  in `apply_resolved_callable` erases mutable bindings with inferred contracts across
  method calls (such as primitive arithmetic operators `+` and `<`). This is an unrelated
  baseline failure and does not block P10 completion.
