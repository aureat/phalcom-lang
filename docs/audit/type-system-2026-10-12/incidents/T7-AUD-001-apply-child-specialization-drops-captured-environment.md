# T7-AUD-001 — Apply child specialization drops captured environment

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

An Apply residual composes captured and supplied specialization environments but recursively visits child residuals using only `supplied`. A captured exact binding needed by a nested Parameter is thereby lost.

**Discriminating evidence:** `captured T=Int` and `Apply(Identity, Residual(Parameter(T)))`, specialized with an empty supplied environment, remains symbolic; expected `ReducedKinded(Int)`.

## Live implementation anchors

- [`phalcom-semantic/src/static_eval/specialize.rs:99-160`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/specialize.rs#L99-L160)

## Architectural repair requirements

Specialize each child against the correct composed environment. Preserve parent capture, reject same-layer contradictions, and prevent host ambient state from silently changing the effective environment.

## Mandatory focused acceptance

Construct a detached Apply with captured T=Int and child Parameter(T); specialize with empty facts and compare to cold Identity<Int>. Repeat with nested Apply/Proof/MemberProjection and conflicting supplied T.

## Verification provenance and limits

The earlier attached agent-session export contains a successful standalone Rust probe run against exactly this reviewed revision where labeled `ARCHIVED_EXECUTION_AND_SOURCE`, and the companion uploaded `Pasted text.txt` includes the full probe source. This turn independently re-read the committed GitHub source at the stated SHA. This audit **did not recompile, re-run Cargo tests, or edit `main`**. For `SOURCE_CONFIRMED*` findings, a precise code-path defect is identified but a distinct new executable reproduction is still required. `SOURCE_CONFIRMED_RESOURCE_RISK_NO_STRESS_PROBE` describes a risk rather than a demonstrated panic.

## Relation to prior incidents

This is a follow-on audit finding, not a modification of `LANG007.C4.P7-INC-001`, `INC-002`, or `INC-003`. The accepted three-way environment design remains the baseline; incomplete or contradictory implementations should be corrected within that architecture except where an explicit architecture consultation is required.
