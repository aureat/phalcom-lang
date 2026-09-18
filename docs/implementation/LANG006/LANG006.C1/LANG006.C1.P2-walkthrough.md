---
id: LANG006.C1.P2
kind: implementation-plan-walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
base_revision: 9f1d2c9fa98e94edc6088aee1834520361e08d3e
working_tree: uncommitted scoped implementation changes
---

# LANG006.C1.P2 — Implementation Walkthrough

## Result

P2 corrected the post-P1 access/provenance and implementation-kind defects in
the shared worktree. The focused semantic, native, bootstrap, AST, and LSP
evidence passes. The plan remains `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`
because the dedicated runtime invocation-parity fixtures, standalone ownership
fixtures, and full G5 suites remain unrun.

## Corrective delta

- Canonical module topology now supplies declaration and caller package/project
  identity to semantic access consumers. Package access is exact identity
  equality; no correctness path uses a parent-path fallback.
- Formal dispatch authorizes the callable selected by normal lookup. An
  inaccessible selected member remains distinguishable from a missing member
  and does not fall through to a superclass implementation.
- Source compiler method constructors carry exact module-derived runtime origin.
  Native anchors retain their declaring `ModuleId`; native installation starts
  with bootstrap provenance and is reconciled to the exact source module after
  source module materialization.
- Native/source conformance is checked through canonical semantic signatures,
  including owner, side, selector, generic contract, parameter shape/types, and
  return type. Erased/dynamic facts are deliberately non-conflicting, while
  receiver-flow and setter return behavior are handled as explicit contract
  rules rather than string normalization.
- `phalcom-common::ImplementationKind` is the sole language category. The
  native-meta execution axis is named `ImplementationProvenance`; reflection
  and presentation expose only `phalcom`, `native`, or `foreign`.
- Bodyless `Foreign` declarations survive semantic and presentation products and
  fail at `foreign.binding_unimplemented` before runtime without introducing an
  ABI or loader.
- Keyword hover/token coverage and the declaration specification now agree on
  all seven keywords and visibility-before-implementation ordering.

## Stable authority and invariants

The semantic access helper in `phalcom-semantic/src/access.rs` is shared by
editor/checker projections. `AccessContext` carries canonical caller package
identity. Runtime authorization consumes `RuntimeAccessOrigin` projected from
the declaring module; `is_universe` remains bootstrap provenance, not ordinary
`internal` authority. `ImplementationKind` and `ImplementationProvenance` must
not be collapsed again. `Foreign` must remain ABI-free and must not be
installed as a native or stub method.

## Focused verification

| Area | Evidence | Result |
| --- | --- | --- |
| Semantic access | `cargo test -p phalcom-semantic --test semantic integration::access_control` | PASS, 8 tests |
| Native semantic products | `cargo test -p phalcom-semantic --test semantic integration::native_conformance` | PASS, 3 tests |
| Native signature owner | `signature::tests::native_contract_rejects_parameter_and_return_type_mismatches` | PASS |
| Presentation | `cargo test -p phalcom-semantic --test semantic integration::presentation` | PASS, 6 tests |
| Core native metadata | `cargo test -p phalcom-core --lib native` | PASS, 10 tests |
| Reflection and Foreign | `cargo test -p phalcom-core --test core reflection_conformance` | PASS, 3 tests; explicit Foreign diagnostic observed |
| Universe bootstrap | exact `boot_01_bootstrap_measurement_separates_catalog_closure_and_execution` | PASS |
| AST migration | `cargo test -p phalcom-ast --lib lang006_` | PASS, 2 tests |
| Native packages | native-meta, native-surface, and native-decl focused suites | PASS |
| LSP | `cargo test -p phalcom-lsp hover` | PASS, selected hover/integration coverage |
| Compile ownership | `cargo check -p phalcom-semantic`; `cargo check -p phalcom-core` | PASS |

`git diff --check` passed after the record edits.

The full affected suites were also run: `phalcom-semantic --test semantic`
ended with 1,420 passed, 22 failed, and 50 ignored; `phalcom-core --test core`
ended with 537 passed, 74 failed, and 28 ignored. The failures are dominated
by pre-existing ADT, iterator, incremental, conformance, module, and runtime
behavior. Core constructor failures share the `allocate()` internal-authority
error already reproduced on the P1 baseline, and the semantic visibility
surface failure was reproduced on a clean P1 worktree.

## Deferred and baseline classification

The exact compiler declaration-dispatch failures involving iterator syntax,
retired private syntax, and bootstrap `Class.allocate()` access reproduce on a
temporary checkout of the pushed P1 revision. They are therefore carried
baseline evidence, not P2 regressions. The known semantic incremental caller
identity failure, VM walk/fold failure, and repository-wide formatting drift
remain outside this corrective surface. Full semantic/core/LSP certification,
runtime cache/dynamic/bound parity, and standalone module/package/cross-project
ownership fixtures were deliberately deferred.

## Consultations and deviations

No architectural consultation was required. The live bootstrap required two
additional expected semantic-diagnostic baseline entries after correcting
reflection source contracts for native descriptor reality; the canonical
reflection conformance suite passes with those entries. No ABI, loader,
ownership, or package-topology redesign was introduced.

## Current completion truth

P2 is implemented across the corrected vertical and focused-tested, but broad
certification is baseline-blocked and the checkpoint is not closed. Do not mark
`LANG006.C1` complete until the remaining P2 coverage is run and classified
under the plan's G5 rules.
