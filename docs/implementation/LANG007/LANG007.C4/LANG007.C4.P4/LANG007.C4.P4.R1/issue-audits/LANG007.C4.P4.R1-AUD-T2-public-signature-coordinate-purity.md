# AUD-T2 — Public covering signatures need recursive coordinate-purity coverage

## Classification

- **Program:** LANG007
- **Checkpoint:** LANG007.C4.1 corrective closure
- **Severity:** Medium; verification gap around a high-value invariant
- **Status:** No current implementation defect demonstrated
- **Current owner:** covering inherent-impl signature materialization and tests

## Issue

C4.1 depends on a strict identity boundary:

```text
published callable signature = declaration-owned coordinates
source-body checking view    = impl-owned coordinates when required
```

The implementation has recursive specialization machinery and debug assertions,
and `impls/signatures.rs::test_permutation_signature_canonicalization` checks a
top-level return parameter. The audit did not find one reusable regression that
walks every relevant type-bearing position and proves that an accepted public
signature contains no parameter owned by the current `ImplId`.

This is a certification gap. The live code may already be correct for the
missing forms.

## Invariant to lock down

For every accepted unconditional covering-inherent callable definition:

```text
no TypeData::Parameter in the published signature has
TypeParameterOwner::Impl(current_impl)
```

The check must recurse through:

- parameter types and return type;
- `Applied` origins and static type arguments;
- associated projections and trait-reference arguments;
- type-member projections and type arguments;
- record rows, fields, tails, and row arguments;
- tuples, packs, pack remainders, and pack concatenations;
- callable parameter and result types;
- exact cases and unions;
- generic constraints, including trait-reference arguments.

The body-local signature is allowed to retain impl-owned coordinates. The test
must inspect the published `EffectiveCallableDefinition.signature`, not only
the source-body signature.

## Evidence in the live tree

- `impls.rs:5537` contains `apply_covering_instantiation_to_signature`.
- `impls.rs:5881-5888` has debug assertions around published parameter/result
  coordinates.
- `types/store.rs:2122-2210` already recursively detects a particular
  `TypeParameterId` through many structural forms, including rows and
  projections.
- `checker/gadt_proof.rs:698-821` contains another structural parameter walk,
  but it is private to GADT proof code and is not a public-signature test
  helper.
- `tests/semantic/impls/signatures.rs` currently has shallow permutation
  assertions.

## Risks covered by the missing test

An impl-owned parameter escaping through a less common carrier can:

- make the public callable contract depend on an implementation declaration
  that is not part of the declaration's identity;
- cause downstream calls to use a parameter that cannot be resolved in the
  declaration coordinate space;
- break incremental reuse when an impl binder changes but the public contract
  should remain stable;
- make source/body specialization appear correct while publication is wrong;
- reintroduce the original C4.1 coordinate-leak defect through a new type form.

## Correct architectural approach

Keep the existing production materialization ownership. Add a test-side or
shared semantic traversal that checks ownership recursively without rewriting
the signature. The helper should accept:

```text
TypeStore
published signature
current ImplId
```

and return the first offending parameter path, if any. A path such as
`return.associated_projection.trait_ref[0]` makes failures actionable.

Do not make the public signature test pass by erasing unknown or unsupported
positions. If a carrier cannot be traversed, expand the canonical traversal or
mark the test case staged with a specific missing representation.

## Testing strategy

### Recursive helper

Implement a deterministic walker in `tests/semantic/impls/signatures.rs` or a
focused test support module. It should recurse through every `TypeData` form
that can carry a proper type and inspect row tails and static arguments.

### Matrix fixture

Construct a covering impl whose source types exercise as many carriers as the
parser and signature builder support. Assert:

- the published signature has no current-impl parameter;
- declaration-owned parameters remain in the expected positions;
- the source-body signature still contains the impl-owned coordinates where
  inverse covering materialization requires them;
- callable-local parameters remain distinct from both declaration and impl
  parameters.

### Negative protection

Retain existing shadowing and binder-confusion tests. Add a test-only mutation
or direct constructed signature containing an impl-owned parameter in each
carrier to prove the helper catches the escape rather than vacuously passing.

## Investigation leads

- Compare `apply_covering_instantiation_to_signature` with
  `types/store.rs::contains_type_parameter` for missed row-tail and pack paths.
- Inspect `CallableSemanticSignature` and generic-constraint representation to
  ensure the helper checks constraints, not only parameter/result types.
- Use `TypeData::Applied` static arguments carefully: `RecordRow` arguments
  live outside `TypeId` and must be walked through the row store.
- Keep this test independent of body checking so a body failure cannot hide a
  publication failure.

## Acceptance condition

The issue is closed when one reusable recursive assertion covers the published
signature and constraints, the covering permutation/shadowing cases pass, and
the helper reports an actionable path for a deliberately malformed
impl-coordinate escape.
