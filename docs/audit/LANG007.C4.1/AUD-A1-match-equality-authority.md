# AUD-A1 — Match coverage has a private equality-normalization authority

## Classification

- **Program:** LANG007
- **Checkpoint:** LANG007.C4.1 corrective closure
- **Severity:** Blocker
- **Status:** Confirmed architectural defect at `ee7e8ee7`
- **First recorded in:** `corrective-4.md`, against the preceding C4.1 state
- **Current owner:** `phalcom-semantic` checker and generic-evidence layers

This document describes a live defect. It is separate from the C4.1 `Self`
coordinate fixes, which remain substantially correct.

## Issue

Match synthesis still calls a private helper in
`phalcom-semantic/src/checker/expression.rs`:

```rust
fn apply_ambient_equalities(
    store: &mut TypeStore,
    ambient_constraints: &[GenericConstraint],
    ty: TypeId,
) -> TypeId
```

The helper scans only `Equivalent` constraints, recognizes only a canonical
`TypeData::Parameter` on one side, builds a local `TypeSubstitution`, and
repeatedly applies it to the scrutinee type. The result is then passed to the
coverage engine.

The checking context already owns a richer
`GenericEvidenceEnvironment`. It stores canonical constraints, equivalence
components, unique refinements, assumed conformance evidence, and the proof
machinery used by ordinary generic consumers. `checker/body.rs` still copies
the environment's constraints into `ambient_constraints`, and match coverage
then treats that copy as a second equality solver.

The current data flow is therefore:

```text
GenericEvidenceEnvironment
        ↓ copied constraints
CheckingContext::ambient_constraints
        ↓ private parameter-only substitution
match scrutinee root
        ↓
coverage decomposition
```

The intended ownership is:

```text
GenericEvidenceEnvironment
        ↓ shared evidence/refinement query
match scrutinee root
        ↓
coverage decomposition
```

## Evidence in the live tree

- `checker/expression.rs:5220-5255` defines `apply_ambient_equalities`.
- `checker/expression.rs:5260-5264` applies it before constructing the
  `CoverageSubject`.
- `checker/body.rs:234-235` assigns `ctx.generic_evidence` and then copies its
  constraints into `ctx.ambient_constraints`.
- `generic_evidence.rs:253-438` defines the environment, equivalence graph,
  `are_equivalent`, `equivalent_terms`, and `unique_refinement`.
- `generic_evidence.rs:604-640` uses the environment when proving equivalent
  generic obligations.
- `types/projection.rs:647-679` consumes the environment for generic
  projection normalization and lexical refinement.
- Existing match tests cover direct and chained equalities, but they do not
  prove that match coverage uses the same authority as those consumers.

## Why this is a semantic defect

The private helper is weaker than the canonical environment in several ways.

### 1. It loses transitive and non-parameter evidence

The environment represents equivalence as connected components. The helper
recognizes only a direct equality where one canonical operand is a parameter.
It does not directly consume an equivalent term set, a unique projection
refinement, or an equality whose useful endpoint is a projection or another
structured term.

For example, ordinary generic reasoning may establish:

```text
T == U
U == Option<V>
```

or may normalize an associated projection under assumed conformance. Match
coverage can still receive the unreduced `T` or `U` if the local substitution
does not happen to expose the expected parameter shape.

### 2. It creates consumer-dependent semantics

Call checking, projection normalization, conditional dispatch, and match
coverage may disagree about the same generic environment. A program can be
accepted or rejected depending on which consumer first asks about the type.

### 3. It makes future proof-state distinctions difficult to preserve

The generic evidence APIs already distinguish proven equivalence from blocked,
recursive, cancelled, budget-exceeded, dynamic, and internal outcomes in their
proof types. The helper returns only a `TypeId`. It cannot communicate why a
refinement was unavailable, so downstream coverage cannot distinguish a known
unrefined type from a blocked query.

### 4. It risks stale or partial normalization

`TypeSubstitution` performs structural rewriting. The helper adds a local
fixpoint loop with a recovery fallback that returns the original type. That is
an additional normalization policy with different termination and recovery
behavior from the canonical evidence and projection machinery.

## Consequences

If this remains open:

- GADT and generic match coverage can use a weaker root type than ordinary type
  checking.
- Exhaustiveness can report a missing case even when generic evidence proves
  that the case is reachable or required under a refined type.
- A future enhancement to generic evidence can silently fail to affect match
  analysis.
- Diagnostics and incremental dependencies may describe the same constraint
  differently in body checking and coverage.
- Any apparent fix in `apply_ambient_equalities` would deepen the second-source
  problem rather than close it.

This finding does **not** claim that every current equality fixture is wrong.
The existing direct and chained tests demonstrate useful behavior. The defect
is the architectural ownership boundary and the untested divergence surface.

## Correct architectural approach

Match coverage should consume a canonical operation owned by
`GenericEvidenceEnvironment` or a directly adjacent shared proof module. The
operation should:

1. accept a canonical `TypeTerm` or `TypeId` plus the current type store;
2. consult the environment's equivalence components and refinements;
3. preserve owner-aware `TypeParameterId` identity;
4. use the existing generic projection-normalization path where projections
   are involved;
5. return an explicit result that distinguishes at least:
   - unchanged/known term;
   - proven refined term;
   - unavailable or blocked refinement;
6. avoid rewriting inference variables or branch-local rigids into published
   canonical types;
7. preserve deterministic behavior independent of constraint insertion order.

`ambient_constraints` may remain as an input to other APIs that still require a
constraint slice during migration. It must not remain the equality authority
for match coverage. The implementation should remove or narrow
`apply_ambient_equalities`, not add more cases to it.

## Testing strategy

### Unit-level evidence tests

Add focused tests for the shared operation covering:

- direct `T == Int`;
- reversed `Int == T`;
- transitive `T == U`, `U == Option<V>`;
- two equivalent parameters with no concrete endpoint;
- an associated or type-member projection under lexical evidence;
- an unresolved/blocked projection, which must not become a false concrete
  type;
- repeated and reordered constraints, which must produce the same result.

### End-to-end match tests

Add source tests that distinguish ordinary generic checking from coverage:

- a callable with an equivalent constraint that makes a concrete enum case
  reachable and exhaustive;
- a reversed/chained constraint with the same expected result;
- a projection-based constraint where coverage must remain blocked or symbolic
  when no proof exists;
- a negative concrete mismatch that remains refuted.

Assert the formal `MatchResolution`, candidate variants, branch proof, and
`ExhaustivenessResult`, not only diagnostic presence.

### Adversarial checks

Compare the coverage root with the type used by ordinary expression checking.
A test helper should make it possible to assert that the root was refined by
the shared evidence operation without exposing private solver details.

## Investigation leads

- Reuse `GenericEvidenceEnvironment::equivalent_terms` and
  `unique_refinement` rather than recreating graph traversal.
- Review `ProjectionNormalizationMode::Generic` in `types/projection.rs` for
  the existing blocked/unknown outcomes.
- Check `checker/call.rs` around its generic-equivalence checks for established
  consumer behavior.
- Inspect the existing constrained match cases in
  `tests/semantic/adts/matching/exhaustiveness.rs`, especially direct,
  reverse, and chained option constraints.
- Keep `Self` specialization and local rigid opening outside this change;
  they are separate ownership concerns.

## Acceptance condition

The issue is closed only when match coverage no longer has a private equality
solver, the shared operation has explicit uncertainty behavior, focused tests
prove parity with ordinary generic evidence, and the semantic suite shows no
regression in existing GADT/refinement cases.
