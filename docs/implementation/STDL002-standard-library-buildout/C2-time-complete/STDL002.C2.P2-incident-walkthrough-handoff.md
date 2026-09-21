---
id: STDL002.C2.P2.INCIDENT-2026-09-21
program: STDL002
checkpoint: STDL002.C2
plan: STDL002.C2.P2
kind: incident-report-walkthrough-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
prepared: 2026-09-21
repository: aureat/phalcom-lang
execution_branch: main
execution_head: e58510996fdceafba98fc407ab6fe20634623a84
---

# STDL002.C2.P2 — Integrated Incident Report, Walkthrough, and Handoff

This document deliberately combines the incident report, implementation
walkthrough, and takeover handoff for the current P2 execution state. It is a
durable handoff record, not evidence that P2 or C2 is complete.

> **Resolution update (2026-09-21).** The canonical data-inherent publication
> defect described below is resolved. The first semantic regression was not a
> valid caller-body discriminator, because its fixture shape did not schedule
> `Probe.run` for body analysis. The replacement snapshot-level regression
> proves that `Duration.seconds(_)` is published on the class-side dispatch
> surface with an `Int` parameter and an entry in the canonical callable
> signature table. The compiler-owned `duration_fractional` corpus fixture now
> expects and observes `type.call.argument_mismatch`. The aggregate negative
> lane passes after its stale `SystemClock` fixtures were corrected to import
> the non-prelude class through `universe.time` and stale expected substrings
> were aligned with current diagnostics.

## 1. Incident Report

### 1.1 Incident

The P2 time-library WIP exposes source-owned parser and timezone behavior, but
the canonical semantic bootstrap does not publish all source-owned inherent
implementations for Universe `data` declarations into the reusable dispatch
surface. As a result, calls such as:

```phalcom
Duration.seconds(1.5)
DateTime.parse("2024-01-05T03:04:05")
```

can fall through unresolved dispatch instead of consuming the declared
class-side signature. The first case therefore does not produce the expected
`ArgumentMismatch` diagnostic for a `Float` argument passed to an `Int`
parameter. The second case is unsafe to extend with cross-class parser calls
until the canonical source signatures are published correctly.

### 1.2 Impact

The incident blocks:

- trustworthy completion of P1 T8 argument/signature closure;
- safe addition of `Timestamp.parse`;
- safe addition of `ZonedDateTime.parse` and offset/zone consistency checks;
- aggregate negative time-corpus certification;
- final P2 and C2 lifecycle closure.

The incident does not establish that raw class-object dispatch is semantically
wrong. The live checker path still intentionally maps a raw class object to
`DispatchSide::Class`; the missing behavior is in canonical publication of
data-owned inherent members.

### 1.3 Reproduction

Focused semantic regression added in:

```text
phalcom-semantic/tests/semantic/capabilities/time.rs
```

The regression expects one `ArgumentMismatch` for `Duration.seconds(1.5)`.
Observed before and after the first attempted bootstrap merge:

```text
expected: 1 ArgumentMismatch
observed: 0 ArgumentMismatch
```

The exact language-corpus discriminator was:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus corpus::time_negative -- --nocapture
```

Observed result:

```text
FAIL
time/negative/duration_fractional.ph unexpectedly succeeded
```

The same command also confirmed that the existing DateTime invalid-separator
fixture raises as expected. An earlier filter against the `core` integration
test target selected zero tests and is not verification evidence.

### 1.4 Root-cause evidence

The relevant semantic path is:

```text
source class receiver
  -> Expr::Var class-object type
  -> synthesize_method_call
  -> resolve_dispatch_target_with_specialization
  -> class-side dispatch lookup
  -> apply_resolved_callable argument relation
```

The dispatch path is present and class-side aware. The canonical bootstrap in
`phalcom-semantic/src/session.rs` does the following in separate phases:

1. registers native surfaces;
2. builds data declaration products;
3. builds inherent implementation contributions;
4. publishes enum behavior;
5. retains the baseline dispatch/signature products.

Before this incident work, the canonical data contributions were not passed
through the same `build_effective_surface` merge used for structural workspace
modules. Consequently, data-owned source implementations such as
`Duration.seconds`, `Date.parse`, `Time.parse`, and `DateTime.parse` were not
reliably present in the baseline dispatch surface consumed by later analysis.

The first corrective attempt added a baseline merge in
`phalcom-semantic/src/session.rs` and a semantic regression, but the regression
still observed zero diagnostics. This means the merge attempt is incomplete or
ordered against the wrong canonical product. Do not treat it as a fix.

### 1.5 Classification

```text
Canonical data publication: RESOLVED.
Evidence: snapshot-level semantic regression and compiler corpus discriminator.

Aggregate negative corpus: RESOLVED.
Cause: `SystemClock` is intentionally non-prelude, but the affected negative
  fixtures referenced it unqualified. VM bootstrap made the runtime binding
  available, while semantic analysis correctly left the identifier unresolved.
  The fixtures now import `SystemClock` from `universe.time`, exercising the
  actual source signature and yielding `type.call.argument_mismatch`.
```

The resolved publication defect and corrected negative fixtures no longer block
parser work or aggregate negative-corpus certification.

## 2. Walkthrough

### 2.1 Execution baseline

```text
branch: main
HEAD: e58510996fdceafba98fc407ab6fe20634623a84
pushed checkpoint: e5851099 fix(stdlib): remove redundant time type guards
working tree: dirty with uncommitted P2 WIP and the incident regression/attempted merge
```

The working tree contains unrelated pre-existing edits and generated/untracked
P2 files. No commit or push was requested or performed.

### 2.2 Established P1/P2 substrate

The current WIP contains the following implemented or substantially authored
surfaces:

- source-owned `Duration`, `Timestamp`, civil calendar values, and `UtcOffset`;
- Jiff `0.2.37` with bundled timezone data;
- VM-owned lazy `TimeZoneAdapter` and cached named-zone handles;
- UTC, fixed, named, and system timezone factories;
- factual unique/fold/gap local resolution boundary;
- `TimeZone`, `ZoneTransition`, `LocalTimeResolution`,
  `LocalTimeDisambiguation`, and `ZonedDateTime` source declarations;
- timezone identity based on VM generation plus canonical logical identity;
- current-time conveniences composed through `Timestamp.now`;
- deterministic `ManualClock` and `ManualWallClock` source support;
- source-owned formatting for the current civil, offset, duration, period,
  timestamp, and zoned forms;
- source-owned parsing for `Duration`, `Period`, `UtcOffset`, `Date`, `Time`,
  and `DateTime`.

These facts describe the WIP tree only. They do not satisfy the final coverage
ledger or C2 closure requirements.

### 2.3 Backend and ownership facts

- Jiff backend types remain behind `phalcom-core/src/timezone.rs` and native
  providers.
- The adapter owns backend database/cache state; Phalcom heap objects carry a
  logical handle rather than a guest-visible backend type.
- UTC and fixed zones use non-database fast paths.
- Named database construction is lazy and uses bundled data.
- Alias handling currently relies on the generated/manual `iana_links` path;
  complete generated alias-map certification remains open.
- The current adapter still uses a hashed `i64` logical handle and needs a
  collision-proof stable handle contract before final certification.

### 2.4 Semantic and textual state

Current source formatting is covered by the positive formatting fixture. The
current parser fixture covers exact nanosecond duration syntax, signed-field
period syntax, signed offsets, civil date/time, and DateTime composition.

Still open:

- `Timestamp.parse`;
- `ZonedDateTime.parse`;
- numeric offset normalization for timestamp parsing;
- explicit offset/zone consistency validation;
- overlap selection by explicit offset;
- expanded/negative-year grammar certification;
- complete parse/format round-trip evidence.

The parser blocker is semantic, not a justification for moving parsing into
Jiff. Public grammar and policy remain Phalcom-owned.

### 2.5 Verification actually run

PASS:

```text
cargo check -p phalcom-core
cargo test -p phalcom-core --test language-corpus corpus::time -- --nocapture
cargo test -p phalcom-core timezone --lib -- --nocapture
git diff --check
```

The timezone adapter unit lane passed its focused lazy construction, alias,
fold/gap, non-hour, historical, and transition fact tests during the WIP
sequence. The positive time corpus passed its current cases.

PASS:

```text
cargo test -p phalcom-semantic --test semantic canonical_data_class_side_methods_publish_argument_signatures -- --nocapture
```

The regression proves `Duration.seconds(_)` is available through canonical
class-side dispatch with its `Int` parameter contract and callable signature.

```text
cargo test -p phalcom-core --test language-corpus corpus::time_negative -- --nocapture
```

The `duration_fractional` fixture now reaches the expected
`type.call.argument_mismatch` compiler diagnostic.

FAIL / INCOMPLETE:

```text
cargo test -p phalcom-core --test language-corpus corpus::time_negative -- --nocapture
```

The aggregate negative lane passes after correcting the non-prelude
`SystemClock` fixture imports and stale diagnostic expectations.

```text
cargo test -p phalcom-semantic --test semantic capabilities::time::canonical_data_class_side_methods_publish_argument_signatures -- --nocapture
```

Warnings include an unused adapter generation accessor; this is nonblocking,
but should be cleaned or justified during stabilization.

## 3. Handoff

### 3.1 Current state

```text
P1 temporal foundation: substantially implemented, lifecycle closure incomplete
P1 T8: canonical data class-side publication and aggregate negative-fixture
      certification resolved
P2 backend/publication: partially implemented and focused-tested
P2 zoned semantics: partially implemented
P2 text: partial formatting and partial parsing only
P2 time.testing: authored and needs final certification
P2 final certification: not started
C2 checkpoint: IN_PROGRESS / PARTIAL / UNVERIFIED
```

Do not create a canonical completed `STDL002.C2.P2-walkthrough.md` or
`STDL002.C2.P2-handoff.md` from this record. Those completion artifacts require
G5/T11 evidence and must remain distinct if and when the plan completes.

### 3.2 Immediate next action

Resume `Timestamp.parse` and `ZonedDateTime.parse` under the existing P2
grammar and ownership constraints.

### 3.3 Remaining ordered work after unblock

- complete remaining P1/P2 argument and negative-fixture certification only as
  new behavior is added;
- finish Timestamp and ZonedDateTime parsing with consistency checks;
- complete generated IANA alias-map and collision-proof handle certification;
- certify ordinary-package imports and grouped exports;
- certify reflection categories and Native/primitive provenance;
- certify GC survival, shared handles, lazy initialization, and resource sanity;
- complete the P2 coverage ledger and final focused/affected suites;
- update `STDL002.C2-CHECKPOINT.md` truthfully;
- produce separate canonical P2 walkthrough and handoff records only at plan
  completion.

### 3.4 Specification status

Authoritative time specification work remains `USER-DEFERRED`. Existing stale
`time.md` documents are excluded from authority and must not be edited or used
as a completion gate.

### 3.5 Ownership and non-goals

- Do not add temporal `Value` variants, temporal heap kinds, bytecodes, or
  compiler recognition of temporal class names.
- Do not make `@primitive` operative; it remains metadata on appropriate Native
  declarations.
- Do not expose Jiff types in the Phalcom API or reflection products.
- Do not replace the canonical `wallClock` with a second host-now primitive.
- Do not use ambient host tzdata as the sole deterministic test oracle.
- Do not weaken the P2 invariant that `ZonedDateTime` state is exactly
  `Timestamp + TimeZone`.

## 4. Record Index

```text
Incident: this document, §1
Walkthrough: this document, §2
Handoff: this document, §3
Checkpoint authority: STDL002.C2-CHECKPOINT.md
Active plan: STDL002.C2.P2-plan.md
Interim prior progress: STDL002.C2.P2-progress-walkthrough.md
```
