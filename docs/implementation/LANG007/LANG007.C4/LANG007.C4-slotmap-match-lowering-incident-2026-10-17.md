# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: SlotMap runtime package
- Checkpoint: LANG007.C4
- Plan: No numbered plan is active for this runtime smoke
- Task/Gate: Run `packages/slotmap/tests/runtime`
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `efe7067abb3dd7c17bea4a5a19f316e575dac30f` (clean)
- Current revision: `efe7067abb3dd7c17bea4a5a19f316e575dac30f` (working changes only)

## 2. Escalation trigger

RED trigger 8: the compiler requires a formal match-lowering product from
semantic analysis, but the runtime compiler cannot find one for a checked
source match. The owning layer is not yet established.

## 3. Required invariant

Every compiled `match` must use a declaration-backed, proven
`MatchLoweringSpec`. Keep `compile_match_expr` fail-closed. Do not synthesize
pattern identities from syntax or weaken exhaustiveness.

## 4. Expected architecture

The semantic checker records each match resolution in its callable analysis.
`build_module_lowering_semantics_with_runtime_types_calls_and_conditional_calls`
projects those resolutions into `LoweringSiteKind::Match` entries keyed by
source and expression range. `compile_match_expr` consumes the corresponding
entry to emit bytecode.

## 5. Observed behavior

The VM constructs successfully. While compiling program-module closures before
initialization, the compiler rejects the match in `DenseSlotMap`'s subscript
getter. The diagnostic reporter now renders the source span. This is a compile
failure before the SlotMap test bodies run.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime
```

Observed output:

```text
error: internal: match expression has no formal lowering record
  packages/slotmap/src/dense-slot-map.ph:243:5
  match self.get(key) {
```

The AST span is byte range `6124..6270`, lines 243–246. Expected behavior: the
runtime package compiles, executes both test modules, and reaches
`System.print("slotmap runtime tests complete")`.

## 7. Relevant code path

```text
checker::declaration::check_class_bodies
  -> checks ClassMember::Index bodies
session callable analysis
  -> publishes callable analyses and match resolutions
semantic_lowering::build_module_lowering_semantics...
  -> projects analysis.match_resolutions to (source, range, Match)
compiler::lib::match_expr::compile_match_expr
  -> requires a matching range/kind entry
```

Key files are:

- `phalcom-semantic/src/checker/declaration.rs`: indexer bodies pass through
  `check_callable_body`.
- `phalcom-semantic/src/session.rs`: source callable extraction includes
  `ClassMember::Index`.
- `phalcom-core/src/modules/semantic_lowering.rs`: projection iterates
  `snapshot.callable_analyses` and each analysis's `match_resolutions`.
- `phalcom-core/src/compiler/lib/match_expr.rs`: compilation fails when the
  exact match site is absent.
- `packages/slotmap/src/dense-slot-map.ph`: failing subscript getter at lines
  242–247.

## 8. Scoped implementation diff

Current local changes unblock successive earlier failures in this same run:

- `phalcom-core/src/native/source.rs`: indexes non-facade intrinsic sums as
  enums and products as source-owned census entries.
- `phalcom-core/src/compiler/lib/data_decl.rs`: uses generic enum/product
  lowering when a matching semantic lowering exists; preserves Tuple, Record,
  and Pack facade ownership.
- `phalcom-core/tests/core/compiler/declaration_dispatch.rs`: covers the
  generic intrinsic lowering route.
- `phalcom-semantic/src/session.rs` and
  `phalcom-semantic/tests/semantic/impls/bodies.rs`: include indexers in
  callable-body publication and replay, with a proven-match regression.
- `packages/slotmap/src/dense-slot-map.ph`: routes the iterable trait
  implementation through an owning-class helper so private fields are read
  within the class.
- `packages/slotmap/src/slot-map.ph`: applies the same owner-class boundary to
  sparse-map iterator construction.
- `phalcom-core/src/vm/dispatch.rs`: renders a source span for missing match
  lowering diagnostics; it does not change match lowering behavior.

## 9. Evidence

- The source-index test module passed: 8 tests, including the bundled provider
  corpus check.
- The focused compiler dispatch regression passed: 1 test for generic intrinsic
  enum and product lowering.
- The initial runtime command passed Universe VM construction and then failed
  at the `DenseSlotMap` subscript getter match above (exit 65). After the fixes,
  both runtime modules pass and the program prints its completion marker.
- The semantic bootstrap trace lists nearby named `DenseSlotMap` callable
  bodies but no callable-body entry matching the subscript getter range. This
  suggests an analysis-publication gap, but the trace alone does not establish
  that the callable analysis is absent.
- `git diff --check` passes.

## 10. Implementer's causal hypothesis

The likely failure is that the indexer's callable analysis or its match
resolution does not reach the lowering bundle. Because the checker has an
explicit indexer path, inspect analysis publication, projection, and bundle
selection to find the missing edge.

## 11. Alternative hypotheses

1. The indexer analysis exists and contains the resolution, but its expression
   range differs from the AST match range.
2. Projection produces the site in a different source/module bundle than the
   compiler reads.
3. The compiler sees the correct bundle but matches on an incomplete key.

## 12. Attempts already made

- Earlier source indexing rejected `ProductKind` as an unsupported facade.
  Source-owned indexing fixed that guard; the focused source tests passed.
- VM bootstrap then rejected ordinary intrinsic data without a canonical
  facade. Generic lowering backed by existing semantic enum/product products
  advanced the run; its focused regression passed.
- The SlotMap trait implementation could not read private class fields. An
  owning-class helper removed that compile error; the runtime run advanced to
  the match-lowering failure.
- Added source-span rendering for `MissingMatchLoweringSemantics` solely to
  locate this failure. It identified the indexer match at bytes `6124..6270`.
- The consultation diagnosis and correction are recorded in §§15–16.

## 13. Why local authority was insufficient at escalation

The failure crosses the checker, callable-analysis publication, lowering
projection, and compiler-bundle boundary. The code has explicit indexer
checking and lowering paths, but the current run does not show which edge drops
the match product. Changing the contract or moving the match into a different
source method before checking those products could hide the actual defect.

## 14. Decision requested

The adviser answered this request; see §15. It is retained here as the original
consultation question.

Inspect the failing `DenseSlotMap` subscript getter in the program's semantic
snapshot and answer:

1. Is its callable analysis present, and does it contain a match resolution
whose expression range is `6124..6270`?
2. If so, does the module lowering bundle contain a matching
   `(source, 6124..6270, Match)` site, and is that bundle attached when the
   compiler compiles the module?
3. Which single owner-layer edge is missing, and what focused regression should
   prove the correction?

Recommend the smallest correction at that owning layer. Keep match coverage
and canonical pattern identity formal; do not add a syntax-derived fallback.

## 15. Consultation outcome

Adviser verdict: implementation bug only. The class-body query loop in
`phalcom-semantic/src/session.rs` skipped `ClassMember::Index`, so the
subscript getter never received a `CallableBody` analysis and its match
resolution could not reach lowering. The same omission existed in class-member
selector derivation and source-body replay. Use the existing
`callable_id_for_member` path for canonical index selectors and include the
indexer's source body in replay. No plan amendment is needed.

## 16. Correction and verification outcome

The semantic session now includes class indexers in body analysis, selector
derivation, and source-body replay. A focused regression confirms a class
subscript getter publishes its callable analysis and a `Proven` match
resolution, then changes the module source and verifies that incremental
source-body replay refreshes the indexer query and retains proven coverage. The
SlotMap package then exposed a separate source-level visibility error: the
`CanonicalIterable` implementation read private `_slots` outside `SlotMap`. An
owner-class `makeIterator` helper now performs that access, as the dense-map
implementation does for its private storage.

Verification completed:

- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic class_indexer_match_publishes_callable_body_and_proven_resolution -- --nocapture` — passed, 1 test.
- `RUSTFLAGS='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime` — both runtime modules passed and printed `slotmap runtime tests complete`.
- Earlier focused source-index coverage passed 8 tests; generic intrinsic enum/product dispatch passed 1 test.

Match compilation remains fail-closed and consumes only the formal proven
lowering product. Broader crate, workspace, lint, and release gates were not
run. This package smoke does not promote LANG007.C4 beyond its existing
checkpoint status.
