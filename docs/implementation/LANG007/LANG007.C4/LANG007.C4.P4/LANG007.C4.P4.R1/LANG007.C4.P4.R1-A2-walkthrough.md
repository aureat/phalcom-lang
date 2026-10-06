---
id: LANG007.C4.P4.R1-A2
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
recorded: 2026-10-06
starting_revision: 4a1cd6c7
implementation_revision: 1002ab70
consolidated_branch: main
---

# R1 A2 — Constructor proof outcome walkthrough

A2 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`, committed on main as
`1002ab70`. It started from clean `4a1cd6c7` with upstream parity 0/0. The user
confirmed the preceding consolidation was pushed and its local branch deleted.
Implementation was performed inline; the subsequent user request authorizes
committing and pushing all pending A2 work. No branch change or delegation occurred.
This delivery updates records only and does not rerun behavioral verification.

## Correction and owning interfaces

`LocalCaseProof` and `VariantOpenOutcome` now distinguish Reachable, Refuted,
and Blocked. Local equality refutes supported structural contradictions and
occurs-check failures, preserves constructor/parent rigid identity, and blocks
unsupported forms, opaque parent-rigid equalities, and unresolved projections.
The existing shared `CheckerControl` supplies cancellation, step, and depth
limits; these failures remain typed blocked results. A bounded representation
check prevents open rows/tuple tails and heterogeneous applications from being
silently localized through a representation that cannot preserve them.

The canonical equality helper and `ProofMerge` likewise preserve unsupported
uncertainty. Failed positional union pairings do not prove set inequality;
unequal opaque local bindings do not prove contradiction. Exact-case domains
become Empty only after refutation; general enum domains skip refuted variants
but block on any uncertain applicable variant. Union decomposition retains local
member terms and checks all members before returning Open.

`ResolvedVariantPattern.blocked` carries uncertainty alongside the existing
family, owner, and selector identities. A blocked contextual singleton remains
a declaration-backed variant rather than becoming a variable binding. Nested
and Or patterns forward this state to the owning coverage engine. Branch-proof
extraction and usefulness propagate blocked proof merges without fabricating a
candidate or equality. Inhabitation and pattern summaries pass the existing
shared control into opening. The compiler consumes the semantic diagnostic and
rejects the blocked match before lowering; no compiler production patch was
needed.

All four production `open_variant_case` callers were inspected. The paired
issue plan records the canonical-helper, proof-merge, pattern-product, and
consumer amendment. Semantic ownership, canonical identity, rigid scopes, and
materialization remain unchanged. Inline review found and corrected additional
opaque-merge/union-order collapse paths. No external consultation or architectural
redesign was required.

## Focused evidence

All commands cleared `RUSTFLAGS` and `RUSTC_WRAPPER` and used the pinned toolchain.

| Discriminator | Completed result |
|---|---|
| New nested parent-rigid source regression before production changes | Failed: exhaustiveness was Proven instead of Blocked |
| Same source regression after correction | 1 passed |
| Final semantic integration `adts::matching` | 155 passed, 0 failed, 16 existing ignored |
| Library `local_outcome_tests` | 4 passed, 0 failed |
| Library `checker::coverage` | 3 passed, 0 failed |
| Core `a2_blocked_nested_constructor_match_is_rejected_before_lowering` | 1 passed; compiler error contains MatchAnalysisBlocked |
| `cargo check -p phalcom-semantic` | Passed; existing deprecation warnings retained |
| Scoped diff/whitespace review | Passed |

The two source regressions assert Blocked exhaustiveness, Useful rather than
Impossible, MatchAnalysisBlocked, and absence of a false contradictory-pattern
diagnostic. The singleton regression also checks retained owner identity with
no fabricated proven candidate. Unit tests cover supported nested parent
rigids, concrete refutation, occurs checks, projection uncertainty, local/canonical
proof merges, unordered unions, cancellation, step/depth budgets, and mixed
reachable/blocked enum, exact-case, and union domains. Existing matching tests
retain concrete incompatibility and supported nested-constructor evidence.

The mixed-domain fixture initially paired a generic declaration form/local
origin incorrectly; its canonical/local views were corrected. The completed
fixture then found an early Open return concealing a later blocked union member.
The focused correction examines the remaining members and its discriminator
passed. Final matching evidence was reacquired after these production changes.

## Residual evidence and limits

An earlier, broader `--lib checker::` probe reported 80 passed and 2 failed.
One failure was the new domain fixture and is resolved by the final focused
coverage run. The other is
`checker::context::tests::tracking_resolver_demands_the_exact_unready_type_member_owner`:
its unchanged resolver fixture panics when the stale resolver product is fetched.
A bounded classification pass inspected the unchanged context code/test; this
is outside A2 acceptance and remains deferred classification C. No predecessor
execution establishes that it is baseline. It was not patched, disabled, or
rerun without a relevant correction.

A source probe using `U::Item` in a constructor result was rejected at declaration
formation with AnnotationUnresolved and EnumVariantResultInvalid; the invalid
variant was absent. Its apparent exhaustiveness result was not evidence of a
valid constructor proof collapse. The probe was removed, while direct typed
proof tests cover unresolved projections. This work does not certify or repair
that source producer.

Full semantic/core suites, workspace build/tests, broad formatting/lint/release
certification, and runtime execution were deliberately not run. Existing TEL-1
expanded inference and Hypothesis blockers remain recorded in their own files.
R1 and P4 remain `IN_PROGRESS / PARTIAL / UNVERIFIED`. I1 absence-invalidation
certification is the next R1 item; P4's corrective completion plan retains its
separate primary implementation authority.
