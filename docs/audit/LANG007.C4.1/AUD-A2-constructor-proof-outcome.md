# AUD-A2 — Constructor opening collapses refutation and blocked proof

## Classification

- **Program:** LANG007
- **Checkpoint:** LANG007.C4.1 corrective closure
- **Severity:** Blocker; exhaustiveness soundness boundary
- **Status:** Confirmed architectural defect at `ee7e8ee7`
- **Current owner:** `phalcom-semantic` GADT proof and coverage layers

## Issue

`phalcom-semantic/src/checker/coverage/domain.rs:59-126` exposes:

```rust
pub(crate) fn open_variant_case(...) -> Option<OpenedVariantCase>
```

The canonical GADT solver already distinguishes `Reachable` from `Refuted`
through `GadtProofResult`. The next local-proof step does not. In
`checker/gadt_proof.rs`, `LocalCaseProof` is an `Option`,
`unify_local_types` returns `bool`, and `open_variant_case` uses `?` when local
proof fails.

That produces the following collapse:

```text
constructor is mathematically refuted
                         ┐
local proof is unsupported, incomplete, or blocked ─┴→ None
```

Coverage interprets `None` differently depending on its caller, but the most
dangerous path is:

```text
open_variant_case -> None
        ↓
no constructor added
        ↓
cases.is_empty()
        ↓
DomainDecomposition::Empty
        ↓
ExhaustivenessResult::Proven
```

The existing `DomainDecomposition::Blocked(BlockReason)` and
`ExhaustivenessResult::Blocked(BlockReason)` products already provide the
correct downstream channel. The constructor-opening boundary does not use it.

## Evidence in the live tree

- `checker/gadt_proof.rs:39` defines `LocalCaseProof` as `Option`.
- `checker/gadt_proof.rs:51-75` returns `None` whenever the boolean local
  unifier returns false.
- `checker/gadt_proof.rs:828-900` contains a boolean canonical equality solver
  whose fallback arm returns `false` for every unsupported form.
- `checker/coverage/domain.rs:67-75` distinguishes only canonical
  `GadtProofResult::Refuted`; local proof failure is collapsed by `?`.
- `checker/coverage/domain.rs:162-170` maps failure for an exact case to
  `DomainDecomposition::Empty`.
- `checker/coverage/domain.rs:229-250` silently skips failed variants and
  returns `Empty` if none remain.
- `checker/coverage/usefulness.rs` already propagates `DomainDecomposition::Blocked`
  to `UsefulnessSearch::Blocked` and then to exhaustiveness.
- `checker/expression.rs:5393-5413` already publishes a
  `MatchAnalysisBlocked` diagnostic for blocked exhaustiveness.

The current GADT regression suite passes, including tests that distinguish
impossible concrete cases from generic cases. Those tests do not establish
that unsupported local representations are blocked rather than refuted.

## Why this is a semantic defect

The local proof layer handles query-local existential rigids. Its input can
contain composites, parent rigids, constructor-local rigids, projections,
open rows, and recovery products. A boolean structural unifier cannot say
whether a mismatch means:

1. the equations have no solution;
2. the equations are solvable only by a proof mechanism not implemented in this
   local path;
3. the input is incomplete or malformed;
4. the query was cancelled or exceeded a budget.

Those states have different semantic meanings. Only the first may remove a
constructor from a closed domain.

The canonical equality solver has the same shape in places: its documented
fallback says that remaining unequal forms are refuted. That is safe only when
the reachable form set is intentionally complete for the equality relation in
question. Local rigid opening has a broader and more recovery-sensitive input
surface, so this assumption must be made explicit and tested.

## Consequences

If this remains open:

- a producer or proof-support gap can make a domain appear uninhabited;
- exhaustiveness can be falsely reported as `Proven`;
- an invalid constructor can disappear from a general enum's candidate set;
- pattern usefulness can be reported as `Impossible` when the correct answer is
  `Useful` with blocked analysis;
- compiler lowering may accept a match because the semantic layer already
  published a false proof, bypassing later fail-closed safeguards;
- diagnostics lose the distinction between unreachable code and analysis
  uncertainty.

This is a soundness issue even if no current fixture triggers the unsupported
form. The semantic product explicitly promises a blocked state, so the
producer must preserve it.

## Correct architectural approach

Introduce typed results at both relevant boundaries.

### Local proof result

Replace the `Option` alias with a result that can represent:

```text
Reachable(bindings, equalities)
Refuted
Blocked(BlockReason)
```

The exact Rust name is flexible. The result must preserve local bindings and
equalities on success.

Refactor the local structural unifier to return a typed outcome. Explicit
structural contradictions should be `Refuted`: different closed nominal forms,
different exact variants, incompatible arity, and an occurs-check failure.
Unsupported representation, unresolved projection, open-row uncertainty,
budget exhaustion, cancellation, or malformed recovery state should be
`Blocked` with an appropriate `BlockReason`.

### Variant-opening result

Make `open_variant_case` return a typed outcome such as:

```text
Reachable(OpenedVariantCase)
Refuted
Blocked(BlockReason)
```

Map canonical `GadtProofResult::Refuted` to `Refuted`. Map local proof outcomes
without losing their category. Do not use `Option` as the transport type.

### Domain decomposition

Update every caller:

- exact-case decomposition must return `Empty` only for `Refuted`;
- a blocked exact-case opening must return `Blocked`;
- general enum decomposition must retain reachable cases, skip refuted cases,
  and immediately return `Blocked` if any potentially applicable case cannot
  be classified safely;
- contextual singleton and variant-pattern resolution must retain blocked
  analysis instead of silently dropping the candidate.

The conservative rule for a closed enum is important: if one variant cannot be
classified, do not prove the whole domain empty or exhaustive.

## Testing strategy

### Direct proof tests

Add unit tests for local proof outcomes:

- concrete incompatible GADT result: `Refuted`;
- constructor-local rigid that cannot satisfy a concrete index: `Refuted`;
- supported nested parent-rigid shape: `Reachable`;
- unsupported projection or heterogeneous static argument: `Blocked`;
- cancellation and budget exhaustion: `Blocked`;
- cyclic or malformed recovery input: `Blocked`.

### Coverage tests

Add source-level tests that assert:

- a refuted constructor is omitted and does not create a missing witness;
- a blocked constructor yields `ExhaustivenessResult::Blocked` and
  `MatchAnalysisBlocked`;
- a general enum with one reachable and one blocked candidate does not become
  `Empty` or `Proven`;
- exact-case and union decomposition preserve the same distinction.

Assert the formal match product and diagnostics. A test that only checks that a
match is not rejected is insufficient because `Missing` and `Blocked` have
different meanings.

### Lowering guard

Add or extend a compiler test proving that a blocked semantic match cannot
lower as an exhaustiveness proof. The semantic test remains the authority for
the proof state; the compiler test checks the projection boundary.

## Investigation leads

- `types/outcome.rs` already defines `BlockReason` values for recursion,
  unknown types, cancellation, budgets, and suppressed dependencies.
- `types/rigid.rs` defines local-type shape and materialization errors. Reuse
  its distinction between canonical and rigid terms.
- `types/case_instantiation.rs` is the owner of constructor-local rigid
  opening and local constraints.
- `checker/coverage/usefulness.rs` already has the desired blocked propagation;
  the first implementation should minimize changes there.
- Existing tests `match_gadt_13_variant_local_rigid_is_not_guessed_to_fit_concrete_index`
  and `match_gadt_16_nested_local_expected_subject_preserves_parent_rigid_without_materialization`
  are key compatibility tests.

## Acceptance condition

The issue is closed only when every constructor-opening caller handles all
typed outcomes, explicit contradictions remain refuted, inability to prove
remains blocked, and no coverage path maps a blocked opening to `Empty` or
`ExhaustivenessResult::Proven`.
