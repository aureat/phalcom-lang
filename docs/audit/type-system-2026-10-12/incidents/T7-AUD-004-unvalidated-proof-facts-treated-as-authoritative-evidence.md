# T7-AUD-004 — Unvalidated proof facts treated as authoritative evidence

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

The public normalized specialization environment accepts canonical-shaped equality/subtype propositions without proving their validity or checking consistency. Evaluator evidence projection subsequently treats them as assumed truths.

**Discriminating evidence:** Directly supplying Equal(Int,String) while specializing Equivalent<Int,String> yields Bool(true). This proves an unsafe API boundary, not necessarily user-source reachability.

## Live implementation anchors

- [`phalcom-semantic/src/static_eval/application.rs:109-244`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/application.rs#L109-L244)
- [`phalcom-semantic/src/static_eval/application.rs:465-494`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/application.rs#L465-L494)
- [`phalcom-semantic/src/static_eval/machine.rs:3225-3285`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/machine.rs#L3225-L3285)

## Architectural repair requirements

Separate canonical proposition keys from proven/assumed evidence. Admit proof facts only through a trusted checker/proof publication channel or validate consistency and provenance before use. Fail closed on contradictory exact propositions.

## Mandatory focused acceptance

Supply Equal(Int,String) and both orientations of invalid Subtype; ensure invalid environment never proves closed false propositions. Preserve valid GADT-derived facts and check cold/specialized parity.

## Verification provenance and limits

The earlier attached agent-session export contains a successful standalone Rust probe run against exactly this reviewed revision where labeled `ARCHIVED_EXECUTION_AND_SOURCE`, and the companion uploaded `Pasted text.txt` includes the full probe source. This turn independently re-read the committed GitHub source at the stated SHA. This audit **did not recompile, re-run Cargo tests, or edit `main`**. For `SOURCE_CONFIRMED*` findings, a precise code-path defect is identified but a distinct new executable reproduction is still required. `SOURCE_CONFIRMED_RESOURCE_RISK_NO_STRESS_PROBE` describes a risk rather than a demonstrated panic.

## Relation to prior incidents

This is a follow-on audit finding, not a modification of `LANG007.C4.P7-INC-001`, `INC-002`, or `INC-003`. The accepted three-way environment design remains the baseline; incomplete or contradictory implementations should be corrected within that architecture except where an explicit architecture consultation is required.
