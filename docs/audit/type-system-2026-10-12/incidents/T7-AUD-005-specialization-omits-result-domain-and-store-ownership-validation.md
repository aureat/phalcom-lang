# T7-AUD-005 — Specialization omits result domain and store ownership validation

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

Direct Parameter and MemberProjection specialization return typed terms without checking correspondence to the residual declared domain or active StaticValueStore lineage. Public constructors allow forged or malformed residual payloads.

**Discriminating evidence:** A Kind(Type) residual parameter supplied Value(Int(77)) returns Int(77). Forged Kind(Type) Product::Shape projection returns ordinary Variant value. Foreign StaticValueRef can be returned as Reduced although get() reports StoreMismatch.

## Live implementation anchors

- [`phalcom-semantic/src/static_eval/specialize.rs:73-96`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/specialize.rs#L73-L96)
- [`phalcom-semantic/src/static_eval/specialize.rs:199-243`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/specialize.rs#L199-L243)
- [`phalcom-semantic/src/static_eval/machine.rs:1066-1085`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/machine.rs#L1066-L1085)
- [`phalcom-semantic/src/static_eval/residual.rs:236-290`](https://github.com/aureat/phalcom-lang/blob/608c8c6cd3346556f3bbc98f0713a6df6e0a72ac/phalcom-semantic/src/static_eval/residual.rs#L236-L290)

## Architectural repair requirements

Centralize typed specialization result validation. Check exact Kind/ValueType domains, static value type, store lineage, projection declared domain, child/whole result domains, and checked signature. Reject malformed public-API nodes before publication.

## Mandatory focused acceptance

Test kind/value mismatch, foreign values, projection domain mismatch, wrong composite types, cross-residual-store references and legal exact ValueType(ProductShape) projection.

## Verification provenance and limits

The earlier attached agent-session export contains a successful standalone Rust probe run against exactly this reviewed revision where labeled `ARCHIVED_EXECUTION_AND_SOURCE`, and the companion uploaded `Pasted text.txt` includes the full probe source. This turn independently re-read the committed GitHub source at the stated SHA. This audit **did not recompile, re-run Cargo tests, or edit `main`**. For `SOURCE_CONFIRMED*` findings, a precise code-path defect is identified but a distinct new executable reproduction is still required. `SOURCE_CONFIRMED_RESOURCE_RISK_NO_STRESS_PROBE` describes a risk rather than a demonstrated panic.

## Relation to prior incidents

This is a follow-on audit finding, not a modification of `LANG007.C4.P7-INC-001`, `INC-002`, or `INC-003`. The accepted three-way environment design remains the baseline; incomplete or contradictory implementations should be corrected within that architecture except where an explicit architecture consultation is required.
