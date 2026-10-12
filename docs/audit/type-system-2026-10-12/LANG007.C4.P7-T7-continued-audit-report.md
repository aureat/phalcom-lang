# LANG007.C4.P7.T7 — Continued Audit Report (2026-10-12)

---
repository: aureat/phalcom-lang
branch: main
reviewed_revision: 608c8c6cd3346556f3bbc98f0713a6df6e0a72ac
date: 2026-10-12
checkpoint: LANG007.C4.P7
task: T7
review_mode: read-only
---


## Verdict

**T7 is implemented on main; G7 acceptance should be reopened.** Recorded focused success does not establish cold/open-specialized parity, domain integrity, proof soundness, or deterministic normalization. Several failures were executed in the prior review against the identical SHA. No code changed in this audit.

## Summary of findings

| ID | Severity | Finding | Evidence |
|---|---|---|---|
| [T7-AUD-001](T7-AUD-001-apply-child-specialization-drops-captured-environment.md) | CRITICAL | Apply child specialization drops captured environment | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-002](T7-AUD-002-static-formal-bindings-not-reconciled-with-typeparameterid-bindings.md) | CRITICAL | Static formal bindings not reconciled with TypeParameterId bindings | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-003](T7-AUD-003-non-tail-symbolic-child-incorrectly-becomes-entire-result.md) | CRITICAL | Non-tail symbolic child incorrectly becomes entire result | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-004](T7-AUD-004-unvalidated-proof-facts-treated-as-authoritative-evidence.md) | CRITICAL | Unvalidated proof facts treated as authoritative evidence | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-005](T7-AUD-005-specialization-omits-result-domain-and-store-ownership-validation.md) | CRITICAL | Specialization omits result domain and store ownership validation | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-006](T7-AUD-006-budget-cancellation-rewritten-into-malformed-semantic-environment.md) | HIGH | Budget/cancellation rewritten into malformed semantic environment | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-007](T7-AUD-007-cyclic-substitution-and-fixed-round-closure-break-normalization.md) | HIGH | Cyclic substitution and fixed-round closure break normalization | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-008](T7-AUD-008-composite-openness-and-closed-exact-proof-specialization-inconsistent.md) | HIGH | Composite openness and closed exact proof specialization inconsistent | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-009](T7-AUD-009-mixed-kinded-value-formal-positions-use-different-index-domains.md) | HIGH | Mixed kinded/value formal positions use different index domains | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-010](T7-AUD-010-detached-specialization-host-contaminates-cycle-identity.md) | HIGH | Detached specialization host contaminates cycle identity | ARCHIVED_EXECUTION_AND_SOURCE |
| [T7-AUD-011](T7-AUD-011-callee-inherited-environment-incorrectly-reconstructed-from-identity-projection.md) | HIGH | Callee inherited environment incorrectly reconstructed from identity projection | SOURCE_CONFIRMED_ARCHIVED_CAPTURE_SUPPORT |
| [T7-AUD-012](T7-AUD-012-memberprojection-stuck-branch-drops-composed-specialization-environment.md) | HIGH | MemberProjection Stuck branch drops composed specialization environment | SOURCE_CONFIRMED_NOT_BEHAVIORALLY_REPRODUCED |
| [T7-AUD-013](T7-AUD-013-conformance-facts-are-represented-but-not-substituted-consumed-consistently.md) | HIGH | Conformance facts are represented but not substituted/consumed consistently | SOURCE_CONFIRMED_NOT_BEHAVIORALLY_REPRODUCED |
| [T7-AUD-014](T7-AUD-014-recursive-specialization-execution-uses-host-call-stack.md) | MEDIUM | Recursive specialization execution uses host call stack | SOURCE_CONFIRMED_RESOURCE_RISK_NO_STRESS_PROBE |

## Provenance and discrimination

- **Reviewed commit:** [608c8c6cd3346556f3bbc98f0713a6df6e0a72ac](https://github.com/aureat/phalcom-lang/commit/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac).
- **Prior execution evidence:** full uploaded `t7-residual-specialization-environment-review.json` logs; last run of outside-repository Rust probe and the source in `Pasted text.txt`. Prior Rust probe had explanatory expected-outcome comments, not assertions: failures are the observed output contrasted with specification and cold results.
- **This turn:** GitHub current main branch and source inspected; no working-copy checkout or Cargo test was run. Do not classify this turn as a fresh behavioral execution.
- **Baseline tests:** previous log records 11 static-recursive integration and 12 residual-foundation tests passed. Those are not a substitute for expanded acceptance.

## Dependency-aware repair sequence

1. **R1 — Protect proof and typing invariants (blocking):** T7-AUD-003, 004, 005. Correct non-tail escape and refuse invalid environments/results before further memoization.
2. **R2 — Reconcile exact/static type environment (blocking):** T7-AUD-001, 002, 009, 011. Preserve complete activation environment; derive canonical formal-to-type mapping.
3. **R3 — Normalize and interpret semantic terms (blocking):** T7-AUD-007, 008, 012, 013. Fully substitute proof goals, projections, conformance keys; preserve unknown vs refuted.
4. **R4 — Context isolation and resource correctness (blocking for certification):** T7-AUD-006, 010; audit T7-AUD-014 for stack safety before declaring broad compatibility.
5. **R5 — Certification:** turn all archived probes into invariant-bearing regressions, not stdout snapshots. Run isolated gates, whole static suite, incremental and cold equivalence, and impacted checker/compiler integrations.

## Acceptance matrix

| Gate | Required proof |
|---|---|
| G7.1 | Parameter, Apply, Proof, MemberProjection cold exact vs detached and live open-then-specialize parity |
| G7.2 | Runtime kind, domain, value typing and cross-store ownership fail closed |
| G7.3 | Nested capture, independent formal owners, mixed kinded/value formal indices |
| G7.4 | Trusted proof assumptions only; contradictory Equal and wrong-target Conforms cannot prove |
| G7.5 | Recursive substitution closure idempotent, order-independent, cycle-safe |
| G7.6 | BudgetExceeded/Cancelled propagate inside every specialization phase |
| G7.7 | Open composite proofs remain open; exact composite proofs reduce |
| G7.8 | Detached specialization is host-independent; real recursive cycle detection preserved |
| G7.9 | Non-tail residual suspensions re-evaluate enclosing definition; no skipped statements |
| G7.10 | Repeat specialization with no new facts and partial then full facts is semantically idempotent |

## Architectural ruling recommendation

Maintain three separate concepts: **activation capture**, **application identity**, and **supplied specialization**. Strengthen them with a trustworthy proof-evidence boundary and a canonical mapping from checked formal parameters to generic type parameters. Preserve source-evaluator semantics by retaining an Apply residual whenever a symbolic intermediate prevents completing the checked body. A typed residual representation that does not preserve its continuation cannot satisfy P7-I10.

## Status terminology

- Previous records mark `T7 / G7` `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; this is an audit finding, not an on-repository lifecycle edit.
- Recommended updated acceptance status: `IMPLEMENTED / AUDIT_BLOCKED` or project-equivalent `IN_PROGRESS / PARTIAL / FAILED_FOCUSED_AUDIT` pending verification and remediation.
- `INC-001` and `INC-002` architectural decisions remain valid; `INC-003` intent is correct but implementation is incomplete.
