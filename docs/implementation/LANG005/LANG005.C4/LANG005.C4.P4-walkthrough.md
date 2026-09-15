---
id: LANG005.C4.P4-WALKTHROUGH
category: LANG
program: LANG005
checkpoint: LANG005.C4
plan: LANG005.C4.P4
kind: implementation-walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
revision: dd802182
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

## T4 ambiguity-diagnostic continuation

The live semantic boundary now emits `DiagnosticCode::TraitDispatchAmbiguous`
when exact trait-evidenced candidates remain ambiguous. The diagnostic is
driven by the existing per-expression candidate product and records, in
deterministic resolver order, each candidate's exact target, exact `TraitRef`,
selector, `TraitRequirementId`, source `ImplId`, and witness/default
selection. Canonical conformance source spans are presented as supporting
labels. The expression retains all candidates and publishes no selected
`TraitDispatchSite`.

The regression
`ordinary_body_dispatch_reports_ambiguous_trait_evidence_at_expression_boundary`
proves one dedicated diagnostic, two candidate identity notes, two source
labels, causal ownership, retained candidates, and no selected target. The
existing 51-test `impls::queries` suite also remains green.

## T5 dispatch interaction matrix

The focused semantic matrix now covers seven ordinary lookup boundaries:
inherent-only behavior, trait-default behavior, inherent precedence over a
default, inherent precedence over a conformance witness, shared concrete
witness convergence, competing defaults, and incomplete conformance proof
states. It asserts the diagnostic family, retained ambiguity candidates,
selected-site presence, and exact target/trait identity for proven selection.
All 52 `impls::queries` tests pass.

## T15 runtime ambiguity and convergence verticals

The core closure adds two vertical regressions. Competing independent defaults
fail compilation with the dedicated `trait.dispatch.ambiguous` diagnostic;
two traits sharing the target's concrete `render` member compile and execute
the inherent result. These tests preserve the semantic/compiler boundary and
confirm that convergence does not require runtime trait discovery.

## Verification outcome

Focused semantic, core, LSP, AST, and workspace-check gates passed. The
workspace test, workspace Clippy, format check, and existing Iterable stress
surface remain baseline-blocked; exact results and classifications are owned
by the checkpoint record and handoff. No release-complete claim is made.

The remaining stress, invalid-corpus, anti-authority, and broad certification
gates are still open, so this plan remains partial.
