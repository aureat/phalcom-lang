---
id: TEST002.C2
category: TEST
program: TEST002
checkpoint: TEST002.C2
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
requires:
  - TEST002.C1.P2
---

# TEST002.C2 - Runtime correctness remediation

## Objective and ownership boundary

This checkpoint owns the retained confirmed runtime-correctness findings from
the 2026-09-07 audit. It preserves compact bytecode, stable runtime identity,
GC ownership, native boundary atomicity, exact numeric domains, Resource-v2,
and Streams-v2 architecture established by TEST002.C2.P1. It does not own
workspace/release certification, the deferred executable-chunk verifier, or
unmeasured runtime/cache optimization.

## Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
| --- | --- | --- | --- | --- | --- |
| TEST002.C2.P1 | Runtime correctness remediation across retained AUD-RUNTIME findings | IN_PROGRESS | PARTIAL | UNVERIFIED | Active from `770cf561d04206b57f6b4b8b3bc96d86eaa77307`; first cluster is AUD-RUNTIME-006 numeric domains |

## Starting state

| Field | Value |
| --- | --- |
| Starting revision | `770cf561d04206b57f6b4b8b3bc96d86eaa77307` (`main`) |
| Starting tree | Clean except unrelated untracked `docs/implementation/TEST002/TEST002.C2/` and `docs/implementation/plan.py`; preserved |
| Inherited prerequisite | TEST002.C1.P2 is `IMPLEMENTED` + `FOCUSED_TESTED`; BL-19 resolved |
| Active plan | TEST002.C2.P1 |
| Active cluster | AUD-RUNTIME-006 numeric domain contracts |

## Established invariants

- `~/` computes exact represented-value floor division and returns exact `Int`.
- Float-domain `%` uses floor remainder, exact dyadic arithmetic, one final
  correctly-rounded binary64 result, and divisor-signed zero.
- Numeric conversion follows operation-specific domains; ordinary Float-result
  promotion is not reused for exact operations.
- `float_to_rational` is the canonical finite-binary64 decoder.
- No guest Rational value is introduced by this remediation.

## Stable interface / takeover map

| Fact | Owner | Consumer |
| --- | --- | --- |
| Exact finite Float decoding | `phalcom-core/src/primitive/float.rs::float_to_rational` | exact numeric primitives |
| Numeric primitive dispatch | `phalcom-core/src/primitive/number.rs` | native sends and language corpus |
| Retained numeric probe | `phalcom-core/tests/core/audit_probes/numeric_contracts.rs` | C2 focused evidence |

## Verification ledger

| Gate | Scope | Result |
| --- | --- | --- |
| T0 | AUD-RUNTIME-006 retained native probe before repair | FAIL reproduced: mixed `~/` rounded `9007199254740993` to `9007199254740992`; Float `%` used host remainder signs |
| T8/G4 | Exact `~/` and Float `%` numeric slice | PASS: retained and expanded numeric audit regressions, dyadic rounding unit, `numeric_values`, and `numeric_diagnostics` all pass |
| T4/G2 partial | AUD-RUNTIME-003 authority tracing | PASS: retained lexical-owner probe fails before trace repair and passes after closure, method, frame authority edges are traced |
| T2 | AUD-RUNTIME-001 constant-pool producer | In progress: retained probe reproduces index `65536 -> 0`; fallible producer migration is being completed across compiler call sites |
| T5 | AUD-RUNTIME-004 native admission/activation | Implemented and focused-tested: public admission is pre-mutation and Legacy/Shape native bodies use separate activation budget |
| T7 | AUD-RUNTIME-005 §6 native provenance | In progress: retained probe confirms nested native identity is lost/misattributed; canonical error-associated origin repair pending |
| T9/T10 | AUD-RUNTIME-008 Resource-v2 handles/attachment | In progress: retained handle/attachment contract requires exact bounded representation and hidden transactional object attachment |
| T11 | AUD-RUNTIME-007 BufferedWriter contract | In progress: short-write, zero-progress, layered flush, suffix preservation, and overlap contract repair pending |
| T12 | AUD-RUNTIME-009 Bytes UTF-8 declaration | Implemented and focused-tested: source declaration now publishes normative `Option<String>`; runtime decoder unchanged |
| T13 | AUD-RUNTIME-010 String raw byte result | Implemented and focused-tested: source/native/generated publication now matches bare `Int`/`None` behavior |
| T14 | AUD-RUNTIME-011 BufferedReader close | Confirmed and reproduced: wrapper closes while inner remains open; repair pending |
| T15 | AUD-RUNTIME-012 Resource stale close after reuse | Confirmed and reproduced: stale A close raises while B remains open; stale isolation preserved, no unsafe cross-generation idempotence repair |
| T16 | AUD-RUNTIME-013 raw-run capture reuse | Implemented and focused-tested: raw entry now unwinds before reuse; regression passes |
| T17 | AUD-RUNTIME-014 mixed relational rounding | Implemented and focused-tested: exact dyadic comparisons pass precision-boundary regression |
| T18 | AUD-RUNTIME-015 BufferedWriter Bytes iteration | Confirmed executed stream-corpus failure: buffered flush sends missing `Bytes.each(_)`; repair pending |

## AUD-RUNTIME-006 root-cause record

Observed: mixed `Int`/`Float` `~/` converted the exact integer through the
shared `PromotedPair::Float` path, and Float `%` used Rust's truncating `%`.

Causal hypothesis: operation-independent promotion discarded represented-value
identity before exact quotient/remainder logic could run.

Evidence: the retained probe failed with `9007199254740993 ~/ 1.0 ==
9007199254740992` and `-5.0 % 3.0 == -2.0`; `number_floor_div` and
`number_mod` both matched on `PromotedPair::Float`.

Predicted effect: operation-specific dyadic arithmetic using
`float_to_rational`, with one final binary64 rounding for Float `%`, preserves
the exact quotient and divisor-signed floor remainder.

Discriminating test: `audit_probes::numeric_contracts` now covers the retained
cases, 2^53 neighbors, mixed signs, special values, and signed zero; the
private dyadic helper covers ties and subnormal boundaries.

## Durable numeric interface

- `number_floor_div` now extracts exact `BigInt`/finite-Float dyadics and uses
  `div_floor` without converting exact `Int` operands to Float.
- `number_mod` keeps Int-only remainder exact and computes Float-domain floor
  remainder from exact dyadics, rounding once with ties-to-even.
- `dyadic_to_float` is the private canonical exact dyadic-to-binary64 helper;
  `float_to_rational` remains the canonical binary64 decoder.

## AUD-RUNTIME-003 root-cause record

Observed: a rooted closure carrying `lexical_class` lost its class owner after
forced GC; the retained probe failed at `heap.try_get(owner).is_some()`.

Causal hypothesis: `heap::trace` enumerated Object variants but omitted newly
added authority fields inside existing `ClosureObject`, `MethodObject`, and
`CallFrame` payloads.

Evidence: the closure probe failed with production tracing unchanged. The
collector's `Object::Closure` arm traced module, upvalues, and constants only;
the Method arm traced holder but not `access_owner`; `trace_frame` traced the
closure and receiver context but not `foreign_receiver_guard.layout_owner`.

Predicted effect: tracing every strong authority/layout edge keeps the owner
alive through the actual runtime object or frame without globally rooting old
classes.

Discriminating test: `audit_probes::runtime_invariants::gc_lexical_owner_survives_collection`
fails with the trace edges removed and passes with the canonical edges restored.

## Durable GC interface

- `heap::trace::trace_object` traces `ClosureObject.lexical_class` and
  `foreign_receiver_guard.layout_owner`.
- `heap::trace::trace_object` traces `MethodObject.access_owner` separately
  from `holder`.
- `heap::trace::trace_frame` traces a frame's foreign layout owner.

## Deferred / baseline issue ledger

- AUD-RUNTIME-001, -002, -004, -005, -007, and -008 remain active C2
  clusters and are not represented as fixed by this numeric slice.
- AUD-RUNTIME-009 is implemented and focused-tested: source declaration now
  publishes normative `Option<String>` and does not alter the correct decoder.
- AUD-RUNTIME-010 is implemented and focused-tested: bare `Int`/`None` is now
  published consistently by source, native metadata, generated surface, and
  runtime regressions.
- AUD-RUNTIME-011 requires a `.ph` wrapper lifecycle repair; AUD-RUNTIME-012
  is classified as intentional stale-generation rejection under Resource-v2,
  with public evidence preserved and no source fix required unless the accepted
  stream contract is amended.
- AUD-RUNTIME-005.1 full executable-chunk verification, .2 rest/native
  metadata optimization, and .3 sparse per-instruction metadata remain
  deferred hardening/performance work as specified by P1.
- Unrelated pre-existing failures remain outside this checkpoint unless a
  focused result proves they are caused by a C2 change.

## Consultation / amendment ledger

| Incident | Plan/task | Trigger | Decision |
| --- | --- | --- | --- |
| Constant-pool fallibility fanout | TEST002.C2.P1 / T2 | Shared `Compiler::add_constant` change reaches approximately 100 compiler producers | Proceed with coordinated mechanical `Result` migration; preserve compact operands and reject before publication |
| Native boundary drift | TEST002.C2.P1 / T5 | Retained audit expected primitive panic, but live `invocation_layout_for_selector` now rejects before primitive entry while still mutating the public stack; native recursion has no activation counter | Repair the current admission ordering and add the separate activation budget without weakening existing re-entry/fiber semantics |

## Active plan and next action

- Active plan: TEST002.C2.P1.
- Latest completed plan: TEST002.C1.P2 (inherited).
- Next action: reproduce the retained AUD-RUNTIME-006 probe, then add its
  focused regression before changing numeric production code.
