# T7-AUD-002 — Static formal bindings not reconciled with TypeParameterId bindings

---
repository: aureat/phalcom-lang
branch: main
reviewed_revision: 608c8c6cd3346556f3bbc98f0713a6df6e0a72ac
date: 2026-10-12
checkpoint: LANG007.C4.P7
task: T7
review_mode: read-only
---

## Classification

- **Severity:** CRITICAL
- **Evidence:** ARCHIVED_EXECUTION_AND_SOURCE
- **Status:** OPEN — not remedied on reviewed `main` revision.

## Required invariant

Static residual specialization must preserve a checked, typed, canonical, deterministic result semantically equivalent to a cold exact evaluation; relevant environments must be retained across escaped residual boundaries, and semantic failures must remain distinct from execution terminals.

## Observed defect / causal evidence

Exact kinded formal bindings and canonical type parameter bindings represent the same semantic assignment in different keys. Proof substitution primarily uses type bindings; captured argument bindings are not lowered into that substitution. The independent keys can disagree.

**Discriminating evidence:** Open Predicate<Int,U> specialized U=Int stays Symbolic while cold Predicate<Int,Int> returns Bool(true). Captured T=Int contradicted by type T=String can be accepted.

## Live implementation anchors

- [`phalcom-semantic/src/static_eval/application.rs:367-494`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/application.rs#L367-L494)
- [`phalcom-semantic/src/static_eval/machine.rs:1214-1260`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/machine.rs#L1214-L1260)
- [`phalcom-semantic/src/static_eval/specialize.rs:162-197`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/specialize.rs#L162-L197)

## Architectural repair requirements

Create a signature-owned bidirectional formal↔type-parameter mapping. Reconcile exact kinded formal assignments and canonical type bindings before proof/Apply/projection evaluation; fail on contradictions after substitution.

## Mandatory focused acceptance

For Predicate<T,U>, retain captured T=Int and specialize U=Int -> Bool(true). Substitute composite Boxed<U>. Require conflict for captured T=Int plus supplied T=String through either key representation.

## Verification provenance and limits

The earlier attached agent-session export contains a successful standalone Rust probe run against exactly this reviewed revision where labeled `ARCHIVED_EXECUTION_AND_SOURCE`, and the companion uploaded `Pasted text.txt` includes the full probe source. This turn independently re-read the committed GitHub source at the stated SHA. This audit **did not recompile, re-run Cargo tests, or edit `main`**. For `SOURCE_CONFIRMED*` findings, a precise code-path defect is identified but a distinct new executable reproduction is still required. `SOURCE_CONFIRMED_RESOURCE_RISK_NO_STRESS_PROBE` describes a risk rather than a demonstrated panic.

## Relation to prior incidents

This is a follow-on audit finding, not a modification of `LANG007.C4.P7-INC-001`, `INC-002`, or `INC-003`. The accepted three-way environment design remains the baseline; incomplete or contradictory implementations should be corrected within that architecture except where an explicit architecture consultation is required.
