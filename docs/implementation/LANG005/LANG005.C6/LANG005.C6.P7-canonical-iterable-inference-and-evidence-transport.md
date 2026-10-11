---
id: LANG005.C6.P7
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG005.C6.P2
  - LANG005.C6.P3
  - LANG005.C6.P4
follows: null
supersedes: null
prepared_against_revision: c02e74757e5f7ebcd6a58debcd157936a475b009
---

# LANG005.C6.P7 — Canonical Iterable Inference and Evidence Transport

## Goal

Close the compiler-owned source/native and generic-call boundaries needed by
the numeric library's generic `Iterable<Item = Float>` APIs. Authored generic
contracts for native-backed methods must survive source/native binding and
canonical signature publication before ordinary call inference runs. Float
list literals, typed Lists, iterators, generic helper forwarding, and exported
generic Families must retain one canonical specialization and its selected
executable evidence. Empty list input to a typed reduction must reach the
reduction's documented `EmptyInputError` path.

This is a corrective plan inside the existing C6 generic evidence checkpoint.
It is independent of the outstanding P5 iterator-migration cleanup and does
not certify the whole checkpoint.

## Fixed invariants

- Generic public math signatures continue to use `Iterable` with associated
  `Item = Float`; no List-only downgrade is permitted.
- `InferVarId` remains inference-local. Generic constraints, trait identities,
  associated projections, and executable evidence remain owned by their
  existing canonical semantic products.
- Generic bodies forward rigid bindings and assumed evidence through ordinary
  call specialization. No numerics-specific checker path, Dynamic fallback,
  duplicated inference state, runtime conformance search, or fabricated exact
  evidence is allowed.
- Call lowering and module/Family execution transport only selected evidence
  through the established runtime type/conformance environments.
- Source/native binding publishes one canonical callable signature. For
  `Option.match`, owner-owned `Option.T` and method-owned `match.R` remain
  distinct binders; the source contract owns semantic types while the
  primitive retains its existing two-Object callback ABI and Object result.
- Iterables are consumed once. `stats.mean([])` reports the established
  `EmptyInputError`.
- Universe source placement is outside this plan; do not relocate `universe`.

## Verify-first repository assumptions

- Inspect the canonical merged signature for `Option.match` and its native
  identity before tracing call specialization. Verify source/native signature
  parity and the unchanged primitive ABI.
- Reconfirm the current behavior and source route for a typed nonempty List, an
  iterator from that List, `[1.0]`, a generic helper calling another helper
  under identical constraints, an exported/re-exported Family, and `[]`.
- Trace the published call specialization and runtime evidence environment
  through direct invocation, Family invocation, and module export before
  selecting any further call-path fix.
- Treat `MATH-GENERIC-ITERABLE-INCIDENT.md` as prior evidence, not proof that
  every failure has one cause; rerun the discriminators on the live tree.

## Tasks

### T1 — Reproduce and classify signature and inference paths

Add or update focused fixtures for native-backed generic callables, typed List,
iterator, Float list literal, same-bound generic helper forwarding, and
exported/re-exported Family calls. Record the canonical signature, native
identity, failing products, and diagnostics before changing semantic
publication or inference.

### T2 — Preserve source/native generic signatures

First verify publication order and binding mechanics around
`impls::build_inherent_impl_contribution`. The source declaration in the
`impl<T> Option<T>` block must provide the canonical `Option.match` signature
with declaration-owned `T` and callable-owned `R`; bind the matching native
identity to that same target-owned callable and validate identity, signature,
domain, visibility, and owner provenance. Permit replacement of an imported
primary entry only through a unique, exact native-anchor match; the normal
same-selector conflict rule remains strict. The catalog must explicitly mark
when callable types describe an ABI-erased contract. For that record, validate
slot count, labels, and runtime representation (including callback slots and
result), while keeping source binders and constraints in the canonical source
signature. Keep `native_contract_mismatch` strict and do not treat Object as a
universal wildcard. Project the single canonical signature to dispatch. Keep
the native shape primitive's Object callback/result ABI unchanged. Do not add a
second callable-owned `T`, retain competing semantic signatures, or extend the
class/enum presentation merger to scan impl syntax.

### T3 — Repair canonical contextual inference

Correct the owning inference path so a Float list literal under
`Iterable`/associated-item equality obtains the same canonical specialization
as equivalent typed input. Ensure empty list literals can use the declared
element contract without inventing value evidence or silently selecting an
unrelated generic type.

### T4 — Preserve generic assumptions through ordinary forwarding

Ensure a generic body can call a generic helper with the same rigid generic
bindings and assumed conformance/equality evidence. Publish the resulting
specialization through existing canonical call products.

### T5 — Audit executable evidence transport

Verify selected conformance recipes survive direct calls, exported/re-exported
Family invocation, and module-export activation. Repair the owning compiler or
runtime seam if any path loses the established conformance environment. Keep
semantic proof and runtime evidence selection separate.

### T6 — Verify the public numeric client

Exercise the existing numeric package as one source client, including
`stats.mean` on nonempty and empty Float literals, a source-defined generic
helper, an exported probability Family, and numerical endpoints. Confirm
single-pass behavior for one-shot inputs.

## Verification gates

- **G1:** Focused semantic inference and generic-evidence discriminators pass
  for native-backed generic signature parity, typed List, iterator, literal,
  generic forwarding, and Family routes. The canonical `Option.match` product
  has distinct owner/callable binders and retains native identity.
- **G2:** Focused compiler/runtime evidence tests pass for direct and exported
  generic calls; the Option primitive retains its two-callback behavior and no
  runtime proof search is introduced.
- **G3:** The numeric package end-to-end client compiles and runs. Empty input
  reaches `EmptyInputError`; representative nonempty numerical outputs match
  the proposal's tolerances.
- **G4:** Scoped diff review, `git diff --check`, and implementation records
  capture exact evidence and remaining unrelated baselines. No workspace or
  release certification is implied.

## Escalation boundary

Stop and consult before changing generic identity, evidence ownership,
canonicalization, runtime environment representation/lifetime, or the accepted
empty-literal inference contract. Record any material amendment in the C6
checkpoint before resuming.

## Amendment — source/native generic signature boundary (2026-10-11)

The first callback discriminator disproved the initial inference diagnosis.
`Option.match` reached call application with no declaration or callable generic
signature; its native signature exposed two `Object` callbacks and an `Object`
return. The canonical source declaration instead defines
`match<R>(some: (T) -> R, none: () -> R) -> R`. The checker therefore correctly
rejected the enclosing `Bool` return. This is a source/native publication
mismatch, not a generic callback inference defect.

The owning correction is the existing source/native binding and canonical
signature publication path, but the live source tree narrows the owner further:
`build_inherent_impl_contribution` currently skips bodyless native methods on
intrinsic runtime owners, while native-catalog import publishes their erased
Object signature. Verify how those products are ordered and keyed before
editing. The source impl declaration must provide the canonical signature;
bind the native identity to the same target-owned callable and validate
identity/signature/domain/visibility/owner provenance. Preserve `Option.T` as
declaration-owned and `match.R` as callable-owned, and leave the shape
primitive ABI unchanged. `core_surface::merge` is presentation-only and
class/enum-only; do not extend it to scan impl syntax. Do not add an
Option-specific inference route. After the binding is correct, resume the
original Iterable, helper-forwarding, Family transport, and numeric-client
acceptance gates as distinct discriminators.

### Amendment — inherent-impl signature ownership (2026-10-11)

The second consultation inspected the live publication owner. The bodyless
native `Option.match` source declaration is skipped by
`impls::build_inherent_impl_contribution` for intrinsic runtime owners, with a
comment that the native catalog owns the checked signature. That behavior
cannot preserve source-level generic binders. The canonical signature must
instead be formed from the source impl declaration and associated with the
matching native surface identity, while the primitive descriptor retains its
Object ABI. Confirm exact publication order and binding keys before editing.
The class/enum `core_surface::merge` remains presentation-only and must not be
expanded to scan impl syntax. This is a verify-first refinement of T2, not a
new acceptance objective.

The current catalog records `PrimitiveAbi::Shape` but does not say whether its
callable types are the semantic contract or an erased ABI shape. Add an
explicit contract-mode marker for the Option record before using its Object
slots as ABI erasure. Normal records retain strict source/native comparison;
the marked record gets a separate ABI slot/representation check. If its source
anchor is absent, ambiguous, or incompatible, fail closed rather than accept
the erased Object signature as the source contract.

## Lifecycle

The C6 checkpoint remains `IN_PROGRESS / PARTIAL`. On completion, create the
normal P7 walkthrough and handoff, update the checkpoint ledger and residual
failures, and leave broader P5/C6 certification to its own gates.

### Amendment — contextual inference for empty list literals (2026-10-11)

The principal architect accepted a syntax-directed rule: an empty list literal
fixes its outer constructor as `List`; when the canonical `List<T> |= Iterable`
conformance and a uniquely determining associated equality provide `T`, infer
`List<T>` and transport its ordinary canonical evidence. Do not search for an
arbitrary iterable subject or invent evidence. Without enough context to
determine `T`, the expression remains underconstrained. This rule is now
recorded in `docs/specs/generic-constraints.md`, §13.

T3 must verify constrained `[]`, a bare Iterable bound with no item evidence,
and an uncontextualized `[]`. Continue to separate semantic specialization
from executable evidence transport; runtime probes use a typed nonempty input
until the canonical inference discriminator passes.

### Amendment — Family call-recipe transport (2026-10-11)

The public numeric client now compiles, but `stats.mean` fails at runtime
because `InvokeSpecialized` validates the `Descriptive.mean(_)` call recipe
against the ordinary `Function.call(***)` gateway. Family activation then
performs the correct live dispatch without receiving the call-site recipe.

Keep the semantic recipe attached to the Family application. Extend Family
activation so it derives the selector and performs live lookup on the captured
receiver first, then validates canonical callable identity and materializes
the call's type/conformance environments for that selected method. Preserve
shape checks, Family error behavior, caller authority, and argument order.
Do not direct-dispatch the semantic method or change the recipe identity to
the Function gateway. Ordinary closure calls continue through the gateway.

Add focused coverage for lowering association, `stats.mean([1.0, 3.0])`, a
selector-compatible but canonical-identity-incompatible recipe, ordinary
closure calls, and Family shape errors. This is a material amendment to T4's
Family/runtime transport boundary; the P7 acceptance objective is unchanged.

### Execution note — module-export Family gateway (2026-10-11)

The first runtime discriminator showed the public package call crosses the
module-export gateway before Family activation. That gateway was validating
the generic recipe against `Function.call(***)`. The runtime now carries the
original call shape and recipe through that module-export seam and reaches the
captured Family member. `RUSTFLAGS='' cargo check -p phalcom-core` passes.

The focused test
`audit_probes::math::public_statistics_family_applies_its_recipe_after_live_member_selection`
now gets through `stats.mean(values.iterator)` and computes the valid result,
but fails its negative control: a cloned program with the callable dispatch
side changed still executes successfully. Canonical identity validation is
therefore not proven and remains open. Verify that the altered recipe reaches
runtime and that selected source methods have canonical runtime identity
metadata; do not mark T4 or G2 complete from the positive path alone.

Separately, Float list-literal calls were rechecked after the package closure
correction and the focused generic Iterable reproducer passed for typed Lists,
iterators, Float literals, and literal iterators.

### Completion amendment — package publication and callable metadata closure (2026-10-11)

Package discovery now retains seeded Universe source modules through complete
linking. Package semantic publication uses the fallible session API, so failed
updates become `ProgramCompileError::SemanticUpdate` rather than silently
publishing a baseline snapshot with callable signatures but no callable body
analyses.

Runtime metadata export publishes canonical callable records and declaration
callable membership. Non-Universe callable references are bound at class
finalization and again after module initialization. Specialized validation
fails closed for missing non-Universe metadata and rejects selector-compatible
canonical identity mismatches; bootstrapped Universe methods retain their
native identity floor.

Focused verification passed:

```text
RUSTFLAGS='' cargo check -p phalcom-core
Issue A generic Iterable reproducer
Issue B positive/negative Family reproducer
Issue A + Issue B combined serial run
```

The Issue B negative control reports the expected canonical callable identity
mismatch. Workspace/release certification remains deferred, and P5/P6
stateful-iteration work remains independent.
