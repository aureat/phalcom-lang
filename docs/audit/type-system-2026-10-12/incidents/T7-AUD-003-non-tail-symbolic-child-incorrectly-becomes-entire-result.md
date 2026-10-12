# T7-AUD-003 — Non-tail symbolic child incorrectly becomes entire result

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

An unresolved proof inside a block can escape as the complete residual even though the checked definition has subsequent statements and a different result domain. Specialization reduces the child Bool instead of replaying the enclosing program.

**Discriminating evidence:** `type Ignored<T: Type>: Int { let x = Subtype<T,Int>; 42 }` open -> Proof residual; specializing T=Int -> Bool(true) instead of cold Int(42).

## Live implementation anchors

- [`phalcom-semantic/src/static_eval/machine.rs:2567-2614`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/machine.rs#L2567-L2614)
- [`phalcom-semantic/src/static_eval/machine.rs:2659-2684`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/machine.rs#L2659-L2684)

## Architectural repair requirements

Distinguish whole-result residuals from suspended intermediate operations. For unresolved non-tail statements, preserve normalized Apply of the checked definition and re-execute once exact; use child escape only when semantically result-equivalent. Validate returned domain.

## Mandatory focused acceptance

Open/exact parity for non-tail let initializer, arithmetic left operand, condition, pattern scrutinee and ignored result. All return exact checked result; no Bool may escape an Int signature.

## Verification provenance and limits

The earlier attached agent-session export contains a successful standalone Rust probe run against exactly this reviewed revision where labeled `ARCHIVED_EXECUTION_AND_SOURCE`, and the companion uploaded `Pasted text.txt` includes the full probe source. This turn independently re-read the committed GitHub source at the stated SHA. This audit **did not recompile, re-run Cargo tests, or edit `main`**. For `SOURCE_CONFIRMED*` findings, a precise code-path defect is identified but a distinct new executable reproduction is still required. `SOURCE_CONFIRMED_RESOURCE_RISK_NO_STRESS_PROBE` describes a risk rather than a demonstrated panic.

## Relation to prior incidents

This is a follow-on audit finding, not a modification of `LANG007.C4.P7-INC-001`, `INC-002`, or `INC-003`. The accepted three-way environment design remains the baseline; incomplete or contradictory implementations should be corrected within that architecture except where an explicit architecture consultation is required.
