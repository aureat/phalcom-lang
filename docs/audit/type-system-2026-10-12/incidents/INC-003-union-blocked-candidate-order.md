# INC-003 — Union-target subtype checking short-circuits on blocked candidates

Finding ID: `INC-003`  
Severity: High / P1  
Confidence: High  
Status: Confirmed by source-level control-flow analysis; direct behavioral fixture deferred because the audit is read-only  

Subsystem: Canonical subtype relation / union candidate evaluation  
Owning file: `phalcom-semantic/src/types/relation.rs`  
Owning function: `check_subtype_impl`  
Affected consumers: Generic constraints, expected-type checking, union assignability, projection-heavy inference, pattern/type reasoning, and any consumer that treats `RelationOutcome::Blocked` as a non-proof state  

## Specification

The normative union law in `docs/specs/typing/type/type.union.md` states that a
union is a least upper bound. In particular, `A <: A | B` and `B <: A | B`.
The union operator is commutative and its semantic result must not depend on
storage or candidate enumeration order.

## Expected behavior

When checking `sub <: U1 | U2`, the relation may be proven if any union member
proves `sub <: Ui`. A blocked candidate means that candidate could not be
decided; it must not prevent a later candidate from establishing the union
relation. If no candidate proves the relation, the result may conservatively be
blocked when at least one candidate is unresolved, otherwise it may be refuted.

## Actual behavior

In the target-union branch, the implementation returns immediately on any
non-refuted terminal outcome:

```rust
(_, TypeData::Union(members)) => {
    for &m in members.iter() {
        let outcome = check_subtype_impl(store, hierarchy, sub, m, ...);
        match outcome {
            RelationOutcome::Proven { .. } => return RelationOutcome::proven(()),
            RelationOutcome::Refuted(_) => {}
            terminal => return terminal,
        }
    }
    RelationOutcome::Refuted(...)
}
```

Therefore a blocked projection/parameter candidate that appears before an
equal or otherwise provable union member causes the whole relation to return
`Blocked`, even though the union law is already satisfied by the later member.

## Minimal reproducer

Construct a proper type-level projection `P` and a distinct proper nominal type
`N`, then construct a canonical union whose member order is `N, P` and query:

```text
P <: N | P
```

The `P <: N` branch reaches the explicit projection-vs-nominal `Blocked` case;
the `P <: P` branch is proven. The current loop returns at the first branch and
never reaches the proof.

Equivalent variants use a rigid or unresolved parameter as the blocked first
candidate and an exact matching type as a later candidate.

## Reproduction command

No repository test was added or modified because the audit is read-only. The
owning focused target is `phalcom-semantic --test semantic`, with a direct
relation fixture under `types::relation` or `tests/audit_probes` after an
authorized regression change.

## Observed output

Source inspection establishes that `RelationOutcome::Blocked` exits the target
union loop before all candidates are evaluated. The existing candidate-order
probe covers successful/refuted union alternatives, but does not cover a
blocked-first/proven-later case.

## Expected output

The result for `P <: N | P` must be `RelationOutcome::Proven`, independent of
the union member allocation order. If every candidate is refuted, the result
can be refuted; if no candidate proves and at least one remains blocked, the
result should remain blocked after the full candidate set has been examined.

## Root cause

The union branch conflates “this candidate did not prove the disjunction” with
“the disjunction cannot be decided.” For existential target-union checking,
`Blocked` is an aggregate result only after all candidates have been explored.
The current implementation applies a universal-style short circuit to an
existential relation.

## Violated invariant

- **T5 — Sound uncertainty:** blocked information is allowed to suppress a
  proof that another union branch establishes.
- **T9 — Determinism:** equivalent union semantics can vary with member order
  when unresolved candidates are present.

## Failure classification

Type-system completeness failure (`B`) and semantic consistency risk (`E`). A
valid subtype judgment can be rejected as blocked. If downstream recovery
incorrectly treats the blocked state as an ordinary failure or uses stale
fallback facts, this can also affect diagnostics and incremental outputs.

## Affected feature combinations

- Unions × associated/type-member projections.
- Unions × generic parameters and rigid variables.
- Unions × expected-type propagation.
- Unions × bounded inference and cancellation.
- Union relation × candidate-order determinism.

## Related historical incidents

Existing audit probes include `union_candidate_allocation_order_is_irrelevant`,
but that probe does not exercise blocked candidates. No historical record was
treated as proof of the current defect.

## Recommended correction layer

Correct the union aggregation in `types/relation.rs`. Evaluate every member
until a proof is found; retain whether any candidate was blocked/cancelled/
budget-exhausted; return that aggregate uncertainty only if no candidate proves
the relation. Preserve cancellation and budget policy explicitly rather than
using the first candidate's terminal state as the union result.

## Required regression tests

1. Blocked-first/proven-later target union proves.
2. Proven-first/blocked-later target union proves.
3. All-refuted target union refutes.
4. All-blocked target union remains blocked.
5. A cancelled or budget-exhausted branch cannot be converted to proof when no
   candidate proves.
6. Permuting canonical union members preserves the result.
7. Source-union (`A | B <: C`) aggregation still requires every member and
   retains unresolved states correctly.

## Potential collateral effects

Correct aggregation may expose previously hidden unresolved projections or
budget states in diagnostics. That is preferable to silently treating the first
candidate as authoritative; downstream callers should continue to distinguish
blocked from refuted.

## Verification status

Control-flow/specification contradiction confirmed on the audit reference
revision. Direct runtime/test reproduction is pending an authorized test-only
probe.
