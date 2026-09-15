---
id: LANG005.C4.P4-WALKTHROUGH
category: LANG
program: LANG005
checkpoint: LANG005.C4
plan: LANG005.C4.P4
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
revision: df2272d179e04eb18d50fd4e4bbb1ed7c80630c6
date: 2026-09-15
---

# LANG005.C4.P4 Walkthrough — Completion and Certification

## Scope and naming

Execution followed the user-supplied file
`LANG005.C4.P4-completion-and-certification-plan.md`. Its internal metadata
still names the historical plan `LANG005.C4.P3-COMPLETION`; the execution
records use the requested P4 filename and identifier without rewriting the
authoritative supplied plan.

The first actionable step was T0 live takeover and baseline lock. The branch
was fetched and checked out locally at `df2272d1`, unrelated VM-spec changes
were preserved, and the generic semantic/runtime, editor, terminal-state, and
temporary-probe seams were re-grounded before editing.

## Implemented authority chain

- `phalcom-semantic/src/impls.rs::resolve_conformance_head` now preserves the
  complete applied exact-case target using `TypeStore::exact_case_type`, so an
  `impl` for `Result<Int>::Ok(_)` does not publish a generic `Result<T>` head.
- `phalcom-semantic/src/trait_dispatch.rs` and the conformance index retain the
  canonical source `ImplId`, exact target, exact `TraitRef`, evidence, and
  selection. Generic source conformances therefore specialize to multiple
  exact applications without minting per-application source identity.
- `phalcom-semantic/src/dispatch.rs` and checker context/call/expression/
  statement paths preserve non-proven trait terminals through ordinary
  analysis. `TraitDispatchTerminal` distinguishes incomplete, unknown,
  blocked, dynamic, cancelled, budget-exceeded, and internal-failure states;
  only proven selections publish a `TraitDispatchSite` or reach lowering.
- `phalcom-semantic/src/session.rs` attaches the canonical conformance
  semantic view to formal top-level checking, closing the exact-case source
  expression path.
- `phalcom-semantic/src/editor.rs` consumes the existing trait-dispatch index
  and exact resolver for editor member projection. It preserves conformance
  callable identity and represents data-component witnesses as
  `EditorMemberTarget::DataComponent`; it does not introduce a second solver.
- `phalcom-lsp/src/completion.rs` passes the complete `ReceiverAlternative`
  through completion projection, retaining exact receiver type identity and
  exposing data-component fields.
- `phalcom-core/src/modules/semantic_lowering.rs` resolves conformance-owned
  callable owners through canonical `ConformanceIndex` metadata instead of
  applying declaration-owner assumptions.
- `phalcom-core/src/vm/associated.rs` retrieves detached conformance methods
  from `detached_method_objects`; inherent class/metaclass lookup remains on
  the existing class dictionary path.

## Durable regression coverage

The normal tests now cover exact generic source execution for both `Value<Int>`
and `Value<String>`, exact enum-case dispatch, generic trait-bound references
across forced GC, editor projection of generic trait members and data
components, exact-case expression selection, and incomplete proof states that
must not become runtime dynamic. The conformance witness runtime test asserts
that the selected witness is executable while its selector remains absent from
the target class dictionary.

Temporary C4 P3 probe scripts and branch-specific workflows were removed after
their useful coverage was represented by these durable tests.

## Verification outcome

Focused semantic, core, LSP, AST, and workspace-check gates passed. The
workspace test, workspace Clippy, format check, and existing Iterable stress
surface remain baseline-blocked; exact results and classifications are owned
by the checkpoint record and handoff. No release-complete claim is made.
