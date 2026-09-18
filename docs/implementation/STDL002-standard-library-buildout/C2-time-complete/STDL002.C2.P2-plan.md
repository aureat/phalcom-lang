---
id: STDL002.C2.P2
category: STDL
program: STDL002
checkpoint: STDL002.C2
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: REQUIREMENTS_ANALYZED
depends_on:
  - STDL002.C2-CHECKPOINT.md
  - STDL002.C2-GUIDANCE.md
  - STDL002.C2.P1-plan-00-shared-header.md
  - STDL002.C2.P1-plan-01-body.md
  - STDL002.C2.P1-walkthrough.md
  - STDL002.C2.P1-handoff.md
  - LANG006.C1-CHECKPOINT.md
  - LANG006.C1.P1-HEADER.md
  - LANG006.C1.P1-BODY.md
repository: aureat/phalcom-lang
planning_baseline_observed: 206e0ab91129e189782c4028de5a40379bcc033c
prepared: 2026-09-18
---

# STDL002.C2.P2 — Time Zones, Zoned Semantics, Textual Forms, Testing Support, and C2 Closure

> **Executor authority.** This plan is the ordered implementation authority for `STDL002.C2.P2`. It is designed for a constrained implementation agent working after `STDL002.C2.P1` has completed. The implementer must consume the actual P1 walkthrough/handoff before production edits. Where the live repository mechanically differs from paths or helper names named below, adapt locally. Where the live architecture or semantics materially contradict this plan, STOP AND CONSULT rather than silently redesigning C2.

> **P1 dependency.** This plan may be written before P1 implementation is complete, but it may not be executed past T0 until a completed `STDL002.C2.P1-walkthrough.md` and `STDL002.C2.P1-handoff.md` exist and their stable interfaces have been reconciled into the P2 takeover record.

> **Specification deferral.** Existing `time.md` documents are stale and are not semantic authority for this plan. Authoritative `time` specification work remains explicitly deferred. P2 does not read, update, rehabilitate, or certify against `docs/implementation/time.md` or any prior `time.md` specification unless a later explicit user instruction re-enables specification work.

---

# 0. Plan purpose

`STDL002.C2.P2` closes the second and final implementation plan of the C2 time checkpoint.

P1 establishes the value/capability substrate:

```text
Duration
Measurement<T>

Clock
SystemClock
clock
Instant

Timestamp
WallClock
SystemWallClock
wallClock

Month
Weekday
Date
Time
DateTime
YearMonth
MonthDay
Period
DateOverflow
UtcOffset
```

P2 builds only on top of that stable foundation.

P2 owns:

```text
timezone backend and lifetime
timezone database snapshot semantics
TimeZone
ZoneTransition
LocalTimeResolution
LocalTimeDisambiguation
ZonedDateTime

UTC/fixed/named/system timezone forms
offset lookup
transition lookup
local civil resolution
DST overlap/gap semantics
system-local conveniences

canonical machine formatting
canonical parsing
offset/zone consistency checking
round-trip properties

time.testing deterministic clocks

ordinary-package certification
reflection/type-observation certification
native/primitive provenance audit
GC/lifetime certification
performance/resource sanity
C2 checkpoint closure
```

P2 does **not** reopen P1's temporal taxonomy.

---

# 1. Approved plan amendments and cross-plan reconciliation

## 1.1 P2-DEV-01 — authoritative time specification remains deferred

Older C2 planning material assigned “final specification” to P2. That instruction is superseded by the explicit user ruling that the existing `time.md` material is stale and authoritative `time` specification work is deferred.

Consequences:

1. no task reads existing `time.md` files as authority;
2. no task edits or replaces them;
3. no completion gate requires a time specification;
4. executable source, public API, tests, checkpoint state, walkthrough, and handoff are the P2 semantic evidence;
5. the C2 checkpoint must record specification work as `USER-DEFERRED`;
6. P2 completion does not silently reactivate specification work.

## 1.2 P2-DEV-02 — LANG006 supersedes STDL002's older primitive-as-implementation-kind assumption

The older C2 design used the conceptual categories:

```text
source
native
primitive
```

The newer LANG006 authority normalizes declaration implementation kind to:

```text
Phalcom
Native
Foreign
```

while `@primitive` and `@intrinsic` are independent metadata.

P2 therefore uses the following reconciled model:

```text
ordinary Phalcom implementation
    ImplementationKind::Phalcom

Rust/provider implementation owned by Phalcom
    ImplementationKind::Native

external interoperability boundary
    ImplementationKind::Foreign

irreducible host/runtime-floor classification
    @primitive metadata on an appropriate Native declaration
```

This preserves the substantive C2 distinction:

```text
host observation/configuration fact
    Native + primitive metadata

timezone database/rule operation
    Native, not primitive

calendar/zoned/text policy
    Phalcom source
```

No task may make `@primitive` install an implementation, change dispatch, change visibility, or become a separate implementation kind.

## 1.3 P2-DEV-03 — P2 consumes future LANG006 visibility/native semantics

P2 targets the completed LANG006 declaration model:

```text
default public
private
protected
module
package
internal

native
foreign

@primitive
@intrinsic
```

No P2-owned surface may depend on:

```text
@native
@private
@protected
@internal
_$selector
general __implementationField
blanket native internal privilege
```

## 1.4 P2-DEV-04 — P1 ordinary-package proof is representative; P2 owns final C2 certification

P1 proves its substrate works through ordinary package/reflection paths.

P2 performs final C2 certification across the timezone/zoned/textual surface and therefore owns the final ordinary-package, reflection, provenance, GC/lifetime, and public-surface closure.

---

# 2. Executor contract

The implementer:

- must read `AGENTS.md` and current workflow authority before editing;
- must read the C2 checkpoint, C2 guidance, completed P1 walkthrough, completed P1 handoff, LANG006 checkpoint/header/body, and this plan;
- must not execute production P2 work until P1 is complete;
- may adapt file names, helper names, local decomposition, and equivalent private implementation details to live drift;
- must not reopen P1 semantics without checkpoint amendment;
- must not expose backend Rust types in the Phalcom API;
- must not make the timezone backend the canonical temporal model;
- must not move derivable calendar, ambiguity policy, parsing grammar, or canonical formatting into Rust merely for convenience;
- must not add temporal `Value` variants, temporal heap kinds, temporal bytecodes, compiler recognition of temporal class names, or special temporal dispatch;
- must not add a broad temporal trait hierarchy;
- must not add public `System.timeZone`, `System.now`, `System.wallClock`, or equivalent date/time discovery façade;
- must not widen the prelude beyond `Duration` and `Timestamp`;
- must not depend on ambient host tzdata for all deterministic behavioral tests;
- must not assume backend finite ranges define Phalcom's P1 value ranges;
- must preserve exact/module/package/internal lexical access semantics from LANG006;
- must use declaration-backed `native` methods and explicit native provider registration;
- must treat `@primitive` as non-operative metadata;
- must use BUILD / STABILIZE / CERTIFY testing discipline;
- must classify unrelated failures before spending P2 scope on them;
- must STOP AND CONSULT on every trigger in §18 or task-local triggers;
- must not commit or push unless separately authorized.

Testing is evidence gathering. Run the smallest command that distinguishes the invariant currently being changed.

---

# 3. P2 hard prerequisites

P2 cannot progress beyond T0 unless the completed P1 handoff proves all of the following.

## 3.1 Required P1 public values and capabilities

```text
Duration
Measurement<T>
Clock
SystemClock
clock
Instant
Timestamp
WallClock
SystemWallClock
wallClock
Month
Weekday
Date
Time
DateTime
YearMonth
MonthDay
Period
DateOverflow
UtcOffset
```

## 3.2 Required P1 laws

```text
Duration
    signed exact nanoseconds
    arbitrary-precision Int semantics

Timestamp
    signed nanoseconds from Unix epoch
    POSIX/non-leap timeline
    arbitrary-precision Int semantic representation

Date/DateTime
    proleptic Gregorian
    no leap second
    timezone-free civil meaning

Period
    months + days
    months -> overflow -> days application order
    no total order

UtcOffset
    signed seconds
    historical second precision supported within P1 bound

Clock
    object identity defines monotonic Instant domain

SystemClock.instance === clock
SystemWallClock.instance === wallClock

Timestamp.now
    delegates through canonical wallClock
```

## 3.3 Required P1 integration facts

The handoff must provide:

```text
actual source paths
actual export paths
actual prelude paths
actual P1 signatures/selectors
UtcOffset accepted bound
civil/year accepted range
Timestamp conversion helpers
native binding mechanism
primitive metadata/linkage mechanism after LANG006 reconciliation
visibility choices
reflection categories
ordinary-package fixture conventions
focused test targets
known baseline failures
known consultations/amendments
```

If any of these remain unresolved, T0 records the blocker and P2 does not guess.

---

# 4. Fixed P2 semantic model

## 4.1 Temporal categories remain distinct

```text
Instant
Timestamp
DateTime
ZonedDateTime
Duration
Period
UtcOffset
TimeZone
```

remain separate semantic categories.

No implicit conversions may collapse them.

## 4.2 ZonedDateTime canonical state

Canonical semantic state:

```text
Timestamp + TimeZone
```

Derived only:

```text
UtcOffset
DateTime
Date
Time
civil components
```

Offset and civil projections are not independent equality state.

## 4.3 Exact versus calendar movement

```text
ZonedDateTime.shift(Duration)
    timestamp-coordinate movement
    preserve TimeZone

ZonedDateTime.advance(Period)
    derive local DateTime
    apply P1 civil Period semantics
    resolve in same TimeZone using Compatible policy
```

## 4.4 Zone conversion

```text
withTimeZone(other)
    preserve Timestamp
    change TimeZone interpretation
```

## 4.5 Equality

```text
ZonedDateTime equality
    Timestamp equality
    AND TimeZone equality
```

Instant-only comparison remains explicit:

```text
sameInstant
compareInstant
```

## 4.6 Local-time resolution

Public factual result:

```phalcom
enum LocalTimeResolution {
  Unique(value: ZonedDateTime)

  Ambiguous(
    earlier: ZonedDateTime,
    later: ZonedDateTime
  )

  Gap(
    before: ZonedDateTime,
    after: ZonedDateTime
  )
}
```

Public policy:

```phalcom
enum LocalTimeDisambiguation {
  Reject
  Earlier
  Later
  Compatible
}
```

Required policy table:

| Case | Reject | Earlier | Later | Compatible |
|---|---|---|---|---|
| Unique | value | value | value | value |
| overlap | error | earlier | later | earlier |
| gap | error | before | after | after |

The backend reports factual candidate mappings. Phalcom source applies this policy.

## 4.7 Gap payload meaning

P2 locks the intended semantic interpretation as:

```text
Gap.before
    requested local DateTime interpreted with the pre-transition offset

Gap.after
    requested local DateTime interpreted with the post-transition offset
```

This must be verified against the selected backend using:

```text
one ordinary one-hour spring-forward gap
one non-one-hour gap
one historical transition
```

If backend APIs cannot represent this meaning without policy distortion, STOP AND CONSULT before changing the public enum.

---

# 5. TimeZone identity model

P2 must not leave `TimeZone` equality to arbitrary allocation identity.

## 5.1 UTC

```text
TimeZone.utc
    canonical UTC zone
```

UTC should not initialize tzdb.

## 5.2 Fixed

```text
TimeZone.fixed(offset)
    semantic identity = UtcOffset value
```

Fixed zones should not initialize tzdb.

Two fixed zones with equal offsets must compare equal.

## 5.3 Named

```text
TimeZone.named(identifier)
```

uses the selected database snapshot.

Named-zone semantic identity is:

```text
database snapshot identity
+
canonical zone identifier
```

or an exactly equivalent stable logical identity.

Do not compare raw backend pointers.

Do not compare complete rule tables on each equality operation.

### Alias VERIFY-FIRST rule

During T1/T2, determine whether the selected backend exposes reliable canonical IANA identifiers for links/aliases.

Preferred endpoint:

```text
aliases that resolve to one canonical identifier
    compare equal
```

If the backend cannot provide stable canonical naming without inventing a large normalization subsystem, STOP AND CONSULT. Do not silently choose pointer identity or full-rule equality.

## 5.4 System

`TimeZone.system` is a discovery operation, not a mutable singleton zone.

Required semantics:

```text
call TimeZone.system
    observe current host timezone configuration
    resolve it to an ordinary TimeZone value

later host configuration change
    may affect a later TimeZone.system call
    does not mutate previously returned TimeZone values
```

A discovered named system zone should become an ordinary named zone under the current database snapshot.

A discovered fixed system offset, if the platform/backend can only provide that, becomes an ordinary fixed zone.

Failure to discover a system zone is explicit. Do not silently substitute UTC.

---

# 6. Backend architecture

## 6.1 Candidate

The first backend candidate is a current stable Jiff 0.2.x release, verified fresh at T1 execution time.

The backend must support:

```text
IANA/TZif named zones
UTC/fixed zones
offset at timestamp
civil local resolution with unique/fold/gap facts
transition traversal
system timezone discovery
second-resolution historical offsets where data contains them
```

The plan does not permanently mandate Jiff if live verification discovers a material incompatibility.

## 6.2 Phalcom-owned adapter

Public Phalcom APIs never expose backend Rust types.

Preferred ownership:

```text
VM
└── TimeZoneDatabaseAdapter
    ├── stable snapshot/generation state
    ├── named-zone cache
    └── backend zone handles

TimeZone Phalcom object
└── stable lightweight logical/native handle
```

The backend adapter owns backend state.

Phalcom GC owns Phalcom objects.

## 6.3 Database snapshot invariant

C2 requires:

```text
lazy initialization
stable VM/process snapshot semantics
existing zones do not silently change rules
```

A backend default cache that can transparently invalidate/re-read host tzdata is **not automatically sufficient**.

T1 must choose and prove one stable model.

Acceptable direction examples include:

```text
A. immutable bundled database generation

B. explicit owned database source pinned for VM lifetime

C. Phalcom-owned adapter that loads/pins rule data for the chosen snapshot

D. another backend-supported immutable snapshot mechanism
```

The weaker model:

```text
cache each zone only after first lookup,
while new zones may come from a later tzdb generation
```

does not satisfy the preferred “one coherent VM snapshot” invariant unless explicitly approved by checkpoint amendment.

## 6.4 Lazy initialization

These operations must not initialize named tzdb state:

```text
import time
TimeZone.utc
TimeZone.fixed(offset)
P1 Date/Time/Timestamp/Duration operations
```

Named database initialization is allowed on first:

```text
TimeZone.named(...)
TimeZone.system when discovery resolves a named zone
named-zone transition/offset operation
```

## 6.5 Range boundary

Backend finite ranges never redefine P1 types.

Required layering:

```text
Phalcom Timestamp / civil value
    P1 range

backend conversion
    bounded interop range

outside backend interop range
    explicit timezone-backend range error
```

UTC/fixed-zone operations that can remain source arithmetic should continue to work outside the named backend range when P1 values themselves are valid.

---

# 7. Native boundary contract

Native timezone code may own facts that require external timezone data:

```text
database initialization
named identifier resolution
system timezone discovery
offset lookup for named zone
local civil resolution for named zone
preceding/following transition
canonical backend zone identity if required
```

Phalcom source owns:

```text
TimeZone public constructors/wrappers
LocalTimeResolution enum construction
LocalTimeDisambiguation policy
ZonedDateTime equality
ZonedDateTime projections
shift
advance policy
withTimeZone
sameInstant / compareInstant
Date.today
ZonedDateTime.now
public parse grammar
public canonical formatting
Period/Duration textual semantics
error presentation where derivable
```

System timezone discovery is:

```text
ImplementationKind::Native
+ @primitive metadata
```

under LANG006.

Named timezone database operations are:

```text
ImplementationKind::Native
no primitive metadata
```

unless a later explicit design changes primitive classification.

---

# 8. Public P2 surface target

Exact punctuation/signature syntax may adapt to live Phalcom grammar, but P2 must provide these concepts.

## 8.1 TimeZone

Conceptually:

```phalcom
class TimeZone {
  class utc -> TimeZone
  class fixed(_ offset: UtcOffset) -> TimeZone
  class named(_ identifier: String) -> TimeZone
  class system -> TimeZone

  at(_ timestamp: Timestamp) -> ZonedDateTime

  transitionBefore(_ timestamp: Timestamp) -> Option<ZoneTransition>
  transitionAfter(_ timestamp: Timestamp) -> Option<ZoneTransition>
}
```

If the final surface exposes direct offset lookup, it should return `UtcOffset` and be derivable from the same backend fact path used by `ZonedDateTime`.

## 8.2 ZoneTransition

Default locked shape:

```phalcom
data ZoneTransition(
  timestamp: Timestamp,
  offsetBefore: UtcOffset,
  offsetAfter: UtcOffset
)
```

Do not add abbreviations, DST flags, country metadata, or backend-specific fields unless a real P2 consumer requires them.

## 8.3 LocalTimeResolution

As fixed in §4.6.

## 8.4 LocalTimeDisambiguation

As fixed in §4.6.

## 8.5 DateTime integration

Conceptually:

```phalcom
DateTime.resolve(_ zone: TimeZone) -> LocalTimeResolution
DateTime.in(_ zone: TimeZone) -> ZonedDateTime
DateTime.in(_ zone: TimeZone, disambiguation: LocalTimeDisambiguation) -> ZonedDateTime
```

Default one-argument `DateTime.in(zone)` uses:

```text
Reject
```

## 8.6 Timestamp integration

Conceptually:

```phalcom
Timestamp.in(_ zone: TimeZone) -> ZonedDateTime
```

This mapping is never ambiguous because Timestamp is an absolute coordinate.

## 8.7 ZonedDateTime

Conceptually:

```phalcom
data ZonedDateTime(timestamp: Timestamp, timeZone: TimeZone) {
  offset -> UtcOffset
  dateTime -> DateTime
  date -> Date
  time -> Time

  shift(_ duration: Duration) -> ZonedDateTime
  advance(_ period: Period) -> ZonedDateTime

  withTimeZone(_ zone: TimeZone) -> ZonedDateTime

  sameInstant(_ other: ZonedDateTime) -> Bool
  compareInstant(_ other: ZonedDateTime) -> Int-or-existing-comparison-type

  class now -> ZonedDateTime
  class now(in: TimeZone) -> ZonedDateTime
}
```

Use the live comparison return convention; do not invent a new temporal comparison result type.

## 8.8 Date conveniences

Conceptually:

```phalcom
Date.today -> Date
Date.today(in: TimeZone) -> Date
```

Default `Date.today` uses:

```text
Timestamp.now
+
TimeZone.system
```

No direct host “today” primitive is allowed.

---

# 9. Current-time dependency law

All public “now/today” convenience APIs compose the canonical P1 wall clock.

Required flow:

```text
Timestamp.now
    ↓
wallClock
    ↓
SystemWallClock or injected WallClock behavior
```

Then P2 adds zone interpretation:

```text
Date.today
    Timestamp.now
    + TimeZone.system
    -> ZonedDateTime
    -> Date

Date.today(in: zone)
    Timestamp.now
    + zone
    -> Date

ZonedDateTime.now
    Timestamp.now
    + TimeZone.system

ZonedDateTime.now(in: zone)
    Timestamp.now
    + zone
```

Forbidden:

```text
native Date.today
native ZonedDateTime.now
backend::now as public source
new wall observation primitive
System.timeZone public façade
```

---

# 10. Canonical text contract

P2 owns one stable machine grammar, not a locale framework.

No CLDR.

No natural-language parsing.

No localized names.

## 10.1 Date

Canonical family:

```text
YYYY-MM-DD
```

Expanded-year support must preserve P1's accepted year range.

Locked machine rule:

```text
0000 through 9999
    exactly four digits

negative year
    leading '-'
    absolute year padded to at least four digits

year > 9999
    leading '+'
    absolute year written with at least five digits
```

Examples:

```text
0000-01-01
0001-01-01
-0001-01-01
+10000-01-01
```

If P1 ultimately restricts the civil year range so expanded years are impossible, retain the grammar design but tests follow the P1 range.

## 10.2 Time

Canonical family:

```text
HH:MM:SS
HH:MM:SS.fraction
```

Canonical formatting:

```text
hour/minute/second
    exactly two digits

nanosecond == 0
    omit fractional part

nanosecond != 0
    emit exactly nine fractional digits
```

P2 chooses fixed nine-digit nonzero fractions to make canonical formatting simple and lossless.

Parsing may accept 1–9 fractional digits and right-pad to nanoseconds.

More than 9 digits is rejected; no silent rounding.

## 10.3 DateTime

Canonical:

```text
<Date>T<Time>
```

No zone or offset.

## 10.4 UtcOffset

Canonical:

```text
Z
```

is permitted only where the enclosing grammar explicitly allows UTC shorthand.

The standalone canonical `UtcOffset` textual form is:

```text
+HH:MM
-HH:MM
+HH:MM:SS
-HH:MM:SS
```

Canonical formatting:

```text
seconds remainder == 0
    ±HH:MM

seconds remainder != 0
    ±HH:MM:SS
```

Zero standalone offset formats:

```text
+00:00
```

rather than `Z`, unless the public API separately exposes an RFC/UTC-oriented formatter.

## 10.5 Timestamp

Canonical formatting:

```text
<Date>T<Time>Z
```

Always UTC.

Parsing should accept:

```text
canonical ...Z
RFC3339-style explicit numeric offset where otherwise valid
```

and normalize the result to absolute `Timestamp`.

Named zone suffixes do not belong to Timestamp parsing.

## 10.6 ZonedDateTime

Canonical formatting:

```text
<Date>T<Time><UtcOffset>[<zone-id>]
```

For UTC/fixed zones, P2 must choose a stable bracket identity.

Preferred rule:

```text
named zone
    [canonical-name]

UTC
    [UTC]

fixed nonzero offset
    [<same canonical numeric offset>]
```

Example:

```text
2026-09-17T13:47:12.345678901+03:00[Europe/Istanbul]
```

Parsing must verify that the explicit numeric offset is valid for the named zone at the parsed local datetime/instant.

Mismatch is an explicit error.

The explicit offset disambiguates overlap text and must not be ignored.

## 10.7 Duration

Canonical Duration machine format uses ISO-style exact-time units only.

No calendar years or months.

Required semantic coverage:

```text
zero
positive
negative
subsecond
seconds/minutes/hours
arbitrary precision
```

The P2 header/implementation may choose the exact ISO-inspired spelling after inspecting the existing string/number APIs, but the grammar must be fully lossless for signed integer nanoseconds.

A bare backend/Jiff duration formatter is not authority.

## 10.8 Period

Period has canonical fields:

```text
months
days
```

and may contain values whose fields have different signs unless P1 explicitly forbids that.

Therefore strict single-sign ISO duration syntax is insufficient as the only lossless format for all possible `Period` values.

Locked requirement:

```text
formatter + parser must round-trip every valid Period value,
including mixed-sign months/days if P1 permits them.
```

Preferred machine extension:

```text
P<signed-months>M<signed-days>D
```

with explicit signs allowed independently after `P`, while ordinary same-sign values may use familiar ISO-style forms.

Examples conceptually:

```text
P1M2D
-P1M2D
P1M-2D
P-1M2D
```

The exact accepted canonical form must be made single-valued in T7. Do not silently normalize mixed-sign Periods into approximate days.

---

# 11. Parsing/error contract

P2 must reuse the live general error model rather than inventing a broad temporal exception hierarchy.

T7/T8 VERIFY-FIRST:

```text
general parse/conversion error type
argument/value error type
range error type
lookup/not-found error type
```

Required error distinctions in tests:

```text
invalid temporal grammar
invalid civil component
invalid offset
unknown zone identifier
offset/zone mismatch
ambiguous local DateTime under Reject
gap local DateTime under Reject
backend interop range exceeded
system timezone discovery failure
```

If no existing general error category can express these without losing important semantics, STOP AND CONSULT before introducing new public error types.

---

# 12. time.testing contract

P2 adds deterministic testing support without global clock mutation.

Preferred public package:

```text
time.testing
```

Preferred source classes:

```text
ManualClock
ManualWallClock
```

Exact names may be adjusted during T9 only if repository conventions strongly favor another term.

## 12.1 ManualClock

Requirements:

```text
subclass/implementation of Clock
own monotonic domain
deterministic initial tick
now
advance(Duration)
```

Negative `advance` is rejected.

Do not provide arbitrary rewind by default.

Do not replace global `clock`.

## 12.2 ManualWallClock

Requirements:

```text
subclass/implementation of WallClock
deterministic current Timestamp
now
advance(Duration)
set(Timestamp)
```

Wall clocks may move backward, so explicit `set` is semantically valid.

Do not replace global `wallClock`.

## 12.3 Testing model

Production APIs that accept clocks can be dependency-injected.

Global conveniences remain global:

```text
Timestamp.now
Date.today
ZonedDateTime.now
```

P2 does not introduce thread-local/global test overrides merely for convenience.

---

# 13. Source-of-truth table

| Fact | Canonical owner | Forbidden duplicate |
|---|---|---|
| exact elapsed time | P1 `Duration` | backend duration type |
| absolute timeline | P1 `Timestamp` | backend timestamp as public state |
| civil date/time | P1 source values | backend civil type as public state |
| fixed offset | P1 `UtcOffset` | backend offset type as public state |
| timezone database rules | native adapter/backend | Phalcom reimplementation of TZif |
| timezone semantic identity | Phalcom `TimeZone` + adapter logical identity | raw Rust pointer equality |
| named-zone cache | VM/native adapter | per-object database parse |
| local mapping facts | native adapter | public Rust ambiguity type |
| ambiguity policy | Phalcom source | backend default policy |
| zoned state | `Timestamp + TimeZone` | redundant offset/date/time state |
| current absolute time | P1 `wallClock` | backend `now()` |
| system timezone observation | native host boundary + primitive metadata | public `System` façade |
| canonical parser grammar | Phalcom source | backend parser as public authority |
| canonical formatter | Phalcom source | backend display as public authority |
| testing clocks | `time.testing` source | global clock replacement |
| time spec | USER-DEFERRED | stale `time.md` |

---

# 14. Global P2 invariants

`P2-INV-01` — P1 temporal categories are not reopened.

`P2-INV-02` — `TimeZone` is not `UtcOffset`.

`P2-INV-03` — `ZonedDateTime` canonical state is exactly `Timestamp + TimeZone`.

`P2-INV-04` — Zoned equality is timestamp plus timezone identity; instant-only equality is explicit.

`P2-INV-05` — `shift(Duration)` is exact timeline movement.

`P2-INV-06` — `advance(Period)` is civil movement followed by timezone resolution.

`P2-INV-07` — `DateTime` remains timezone-free.

`P2-INV-08` — local resolution exposes Unique/Ambiguous/Gap facts explicitly.

`P2-INV-09` — ambiguity policy lives in Phalcom source.

`P2-INV-10` — Compatible means earlier for overlaps and after/later for gaps.

`P2-INV-11` — named-zone rules come from a mature native backend behind a Phalcom-owned adapter.

`P2-INV-12` — importing `time` does not eagerly initialize tzdb.

`P2-INV-13` — UTC/fixed zones do not require named tzdb initialization.

`P2-INV-14` — existing TimeZone values do not silently change rule meaning during their lifetime.

`P2-INV-15` — one VM observes one coherent timezone database snapshot/generation unless amended explicitly.

`P2-INV-16` — backend finite ranges do not narrow P1 `Timestamp`/civil semantics.

`P2-INV-17` — historical second-resolution offsets remain representable where backend data exposes them and P1 bounds permit them.

`P2-INV-18` — system timezone discovery is explicit host observation and does not create a public `System` time API.

`P2-INV-19` — system timezone discovery failure does not silently become UTC.

`P2-INV-20` — `Timestamp.now` remains the sole canonical public absolute-current-time source used by P2 conveniences.

`P2-INV-21` — parser grammar is Phalcom-owned.

`P2-INV-22` — canonical formatting is Phalcom-owned.

`P2-INV-23` — zoned parse with offset + zone checks consistency.

`P2-INV-24` — parse/format round trips are lossless for every supported P2 value.

`P2-INV-25` — `Period` formatting never approximates calendar months as exact days.

`P2-INV-26` — no temporal VM specialization is introduced.

`P2-INV-27` — no broad temporal trait hierarchy is introduced.

`P2-INV-28` — timezone backend operations are Native, not automatically primitive.

`P2-INV-29` — host-only timezone discovery may carry primitive metadata under LANG006, but metadata is non-operative.

`P2-INV-30` — public time source contains no legacy `_$`/general implementation-field access convention.

`P2-INV-31` — final P2 public behavior works from an ordinary user package.

`P2-INV-32` — reflection exposes ordinary Phalcom categories, not backend Rust types.

`P2-INV-33` — zone handle lifetime is safe across Phalcom GC.

`P2-INV-34` — repeated named-zone lookup uses a cache/handle path rather than reparsing database data per object.

`P2-INV-35` — deterministic semantic tests do not depend solely on ambient host tzdata.

`P2-INV-36` — existing stale `time.md` documents remain excluded and specification work remains USER-DEFERRED.

---

# 15. P2 coverage ledger

| ID | Dimension | Required evidence |
|---|---|---|
| P2-CV-01 | P1 handoff consumption | actual P1 signatures/ranges/path map recorded |
| P2-CV-02 | LANG006 primitive reconciliation | Native/Foreign/Phalcom + primitive metadata model recorded |
| P2-CV-03 | backend dependency | exact version/features verified |
| P2-CV-04 | snapshot lifetime | one stable VM database generation proven |
| P2-CV-05 | lazy initialization | plain import/UTC/fixed do not load named tzdb |
| P2-CV-06 | named cache | repeated named lookup reuses stable handle |
| P2-CV-07 | UTC TimeZone | canonical UTC behavior |
| P2-CV-08 | fixed TimeZone | identity/value behavior from UtcOffset |
| P2-CV-09 | named TimeZone | valid/invalid identifier behavior |
| P2-CV-10 | alias identity | chosen canonical alias rule proven |
| P2-CV-11 | system timezone | discovery success/failure and stability |
| P2-CV-12 | backend range | explicit interop range errors without P1 narrowing |
| P2-CV-13 | historical seconds | second-resolution historical offset fixture |
| P2-CV-14 | offset at timestamp | named-zone projection correctness |
| P2-CV-15 | transition before | known transition |
| P2-CV-16 | transition after | known transition |
| P2-CV-17 | ZoneTransition state | timestamp/before/after offsets |
| P2-CV-18 | unique local resolution | Unique payload |
| P2-CV-19 | overlap resolution | Ambiguous earlier/later |
| P2-CV-20 | gap resolution | Gap before/after |
| P2-CV-21 | non-hour transition | e.g. 30-minute or historical non-hour case |
| P2-CV-22 | Reject policy | overlap/gap reject |
| P2-CV-23 | Earlier policy | overlap/gap table |
| P2-CV-24 | Later policy | overlap/gap table |
| P2-CV-25 | Compatible policy | earlier overlap / after gap |
| P2-CV-26 | Timestamp.in | unambiguous conversion |
| P2-CV-27 | DateTime.resolve | factual resolution |
| P2-CV-28 | DateTime.in | default Reject |
| P2-CV-29 | Zoned canonical state | timestamp + timezone only |
| P2-CV-30 | derived offset | zone rule projection |
| P2-CV-31 | derived civil | DateTime/Date/Time projection |
| P2-CV-32 | equality | same timestamp + same zone |
| P2-CV-33 | sameInstant | same timestamp/different zone |
| P2-CV-34 | compareInstant | timestamp-only ordering |
| P2-CV-35 | shift | exact duration across DST |
| P2-CV-36 | advance | civil period across DST |
| P2-CV-37 | shift vs advance | demonstrate divergent result at DST |
| P2-CV-38 | withTimeZone | timestamp preserved |
| P2-CV-39 | Date.today zone | date depends on supplied/system zone |
| P2-CV-40 | ZonedDateTime.now | delegates through Timestamp.now |
| P2-CV-41 | Date format/parse | round trip incl expanded/negative year where valid |
| P2-CV-42 | Time format/parse | zero/nonzero fractions |
| P2-CV-43 | DateTime format/parse | round trip |
| P2-CV-44 | UtcOffset format/parse | minute + second offsets |
| P2-CV-45 | Timestamp format/parse | UTC + accepted numeric offset |
| P2-CV-46 | Zoned format/parse | offset + zone |
| P2-CV-47 | offset-zone mismatch | explicit rejection |
| P2-CV-48 | Duration format/parse | exact nanosecond round trip |
| P2-CV-49 | Period format/parse | months/days incl mixed signs if valid |
| P2-CV-50 | invalid parse | grammar/component failures |
| P2-CV-51 | ManualClock | independent domain + deterministic advance |
| P2-CV-52 | ManualWallClock | set/advance deterministic behavior |
| P2-CV-53 | no global test override | canonical clock/wallClock unchanged |
| P2-CV-54 | ordinary package | complete public P2 use outside Universe |
| P2-CV-55 | grouped imports | P2 public names through normal imports |
| P2-CV-56 | prelude stability | only Duration/Timestamp temporal prelude |
| P2-CV-57 | LocalTimeResolution matching | ordinary package pattern match |
| P2-CV-58 | reflection | TimeZone/ZoneTransition/enums/ZonedDateTime categories |
| P2-CV-59 | native provenance | named backend Native, host discovery primitive metadata |
| P2-CV-60 | GC survival | zone/zoned values survive forced GC |
| P2-CV-61 | handle sharing | one object's collection does not invalidate another |
| P2-CV-62 | no VM specialization | static/diff/disassembly evidence |
| P2-CV-63 | cheap import | no database/system probe during import |
| P2-CV-64 | resource sanity | cache/database/handle allocation sanity |
| P2-CV-65 | pinned fixture determinism | semantic tests independent of host tzdata |
| P2-CV-66 | system integration | platform discovery smoke where supported |
| P2-CV-67 | legacy absence | no old selector/attribute authority in P2 source |
| P2-CV-68 | C2 plan/API agreement | checkpoint + source/tests agree |
| P2-CV-69 | spec deferral | stale specs untouched; USER-DEFERRED recorded |
| P2-CV-70 | final C2 closure | checkpoint/walkthrough/handoff truthful |

---

# 16. Task graph

```text
T0  live takeover + P1 handoff + checkpoint amendment reconciliation
 ↓
T1  backend dependency spike + snapshot/lifetime proof
 ↓
G1  backend architecture locked
 ↓
T2  native TimeZoneDatabase adapter + named-zone cache
 ↓
T3  TimeZone UTC/fixed/named/system + ZoneTransition
 ↓
G2  timezone foundation stable
 ↓
T4  local resolution facts + LocalTimeResolution/Disambiguation
 ↓
T5  ZonedDateTime conversions/projections/equality/movement
 ↓
G3  zoned semantic stabilization
 ↓
T6  Date.today / ZonedDateTime.now conveniences
 ↓
T7  canonical machine formatting
 ↓
T8  parsing + offset-zone consistency + round trips
 ↓
G4  textual stabilization
 ↓
T9  time.testing deterministic clocks
 ↓
T10 ordinary-package/reflection/provenance/GC/performance closure
 ↓
G5  final C2 integration stabilization
 ↓
T11 certification + checkpoint/walkthrough/handoff closure
```

---

# 17. Tasks

## T0 — Re-ground live repository, consume P1 handoff, and amend cross-plan authority

### Purpose

Establish the exact execution baseline, consume P1's as-built interfaces instead of speculative ones, reconcile LANG006 primitive semantics, record specification deferral, classify live repository drift, and prove P2 is ready to start.

### Preconditions

- P1 walkthrough exists.
- P1 handoff exists.
- P1 is recorded complete in the C2 checkpoint.
- LANG006 implementation state is either complete or sufficiently landed for P2 source.
- no unresolved P1 STOP/CONSULT item blocks P2.

### Consumes

```text
STDL002.C2-CHECKPOINT.md
STDL002.C2-GUIDANCE.md
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-plan-01-body.md
STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md

LANG006.C1-CHECKPOINT.md
LANG006.C1.P1-HEADER.md
LANG006.C1.P1-BODY.md

AGENTS.md
workflow authority
```

Do not read stale `time.md` documents as authority.

### Produces

A P2 takeover record in `STDL002.C2.P2-walkthrough.md` containing:

```text
execution branch
execution HEAD
working tree status
P1 completion revision
P1 public surface map
P1 UtcOffset bound
P1 civil range
P1 native linkage mechanism
P1 primitive metadata/linkage mechanism
P1 baseline failures
LANG006 status
P2 source/test path map
approved primitive reconciliation amendment
approved spec-deferral amendment
```

### Files and symbols

At minimum re-read live equivalents of:

```text
phalcom-core/core/universe/src/time/**
phalcom-core/core/universe/src/package.ph
phalcom-core/core/universe/src/concurrency/**
phalcom-core/src/universe/primitives.rs
phalcom-core/src/native/**
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/gc.rs
phalcom-core/src/modules/**
phalcom-core/tests/core/**
phalcom-core/tests/fixtures/language/time/**
phalcom-semantic/tests/semantic/capabilities/time.rs
Cargo.toml
phalcom-core/Cargo.toml
```

Also locate the live native provider path created/normalized by LANG006.

### Required implementation shape

1. Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

2. Read the completed P1 walkthrough and handoff in full.
3. Record exact P1 signatures and ranges.
4. Verify no P1 surface already implemented a conflicting timezone concept.
5. Verify current LANG006 implementation-kind state.
6. Amend C2 checkpoint terminology so “primitive” is a metadata classification on Native declarations, not an independent implementation kind.
7. Record stale-spec deferral.
8. Discover test targets with `cargo test ... -- --list`.
9. Run only the smallest existing P1 time integration tests needed to prove the starting substrate is green or classify baseline failures.

### Forbidden approaches

- no production timezone code in T0;
- no guessing missing P1 signatures;
- no resurrecting stale specs;
- no compatibility layer for old primitive keyword semantics;
- no weakening P2 because LANG006 is unfinished.

### Test changes required

None unless a P1 handoff claim is missing an obvious executable proof; in that case update only the P1 completion evidence before advancing.

### Tests to run now

Focused P1 acceptance filters named in the P1 handoff.

### Tests explicitly deferred

All P2 timezone tests.

### Acceptance

`P2-CV-01`, `P2-CV-02`, and `P2-CV-69` have durable evidence.

### Local STOP / CONSULT triggers

- P1 handoff absent or materially incomplete;
- P1 semantics differ from checkpoint without recorded amendment;
- primitive/native reconciliation conflicts with live LANG006;
- `UtcOffset` bound is still undecided;
- P1 civil range is still undecided where P2 backend conversion needs it;
- current time source still depends on legacy access/linkage semantics.

### Checkpoint update

Mark P2 active and record:

```text
P1 consumed
LANG006 primitive reconciliation
specification USER-DEFERRED
P2 start revision
```

---

## T1 — Verify backend version, feature set, snapshot strategy, and interop bounds

### Purpose

Select the concrete timezone backend and prove that its APIs, dependency features, database ownership, range, and ambiguity/transition behavior can satisfy P2 without leaking backend semantics into the public API.

### Preconditions

T0 accepted.

### Consumes

P1 ranges and all backend-related invariants.

### Produces

A backend decision record inside the P2 walkthrough:

```text
crate/version
features
platform behavior
database source
snapshot ownership strategy
named-zone cache strategy
alias/canonical-name behavior
timestamp/civil interop range
offset range
system-zone discovery behavior
local ambiguity mapping
transition API
test fixture strategy
```

### Files and symbols

```text
Cargo.toml
phalcom-core/Cargo.toml
new native timezone adapter module location
VM ownership/root classification
test support/fixture directories
```

### Required implementation shape

1. Verify current stable Jiff 0.2.x APIs and features against the exact lockfile-compatible version.
2. Confirm named zone lookup, transition iteration, local ambiguity, and system discovery.
3. Verify default database invalidation behavior.
4. Choose a stable VM snapshot strategy.
5. Verify backend timestamp/civil and offset bounds.
6. Verify alias/canonical-name capability.
7. Verify a pinned-fixture strategy.
8. Prototype only enough Rust code/tests to prove lifetime and API facts.
9. Do not publish public TimeZone source yet.

### Forbidden approaches

- using the default global database without proving snapshot semantics;
- exposing Jiff types publicly;
- narrowing P1 types to backend ranges;
- using host tzdata as the sole deterministic test oracle;
- accepting backend default ambiguity policy as Phalcom policy.

### Test changes required

Add adapter-level/native-unit tests only where necessary to prove:

```text
snapshot stability
known named lookup
known transition
known gap/fold facts
range conversion
```

### Tests to run now

Only the new backend/adapter spike tests plus `cargo check -p phalcom-core`.

### Tests explicitly deferred

Public TimeZone and source-language tests.

### Acceptance

`P2-CV-03`, `P2-CV-04`, `P2-CV-12`, and fixture strategy for `P2-CV-65` are proven.

### Local STOP / CONSULT triggers

- no coherent immutable/stable snapshot model;
- backend cannot expose both sides of gap/fold cleanly;
- backend loses second-resolution historical offsets required by P1 range;
- backend forces public range narrowing;
- canonical alias behavior cannot be defined reasonably;
- adding backend state creates uncertain GC/root ownership.

### Checkpoint update

Record selected backend, version, snapshot model, and interop limits as durable P2 decisions.

---

## G1 — Backend architecture locked

Pass only if:

```text
[ ] exact backend/version/features known
[ ] stable VM snapshot semantics proven
[ ] named cache design known
[ ] backend range known
[ ] alias identity rule known
[ ] local resolution maps cleanly to 0/1/2 factual candidates
[ ] transition API sufficient
[ ] pinned deterministic fixture strategy exists
[ ] GC/root ownership is understood
```

---

## T2 — Implement native TimeZoneDatabase adapter and named-zone cache

### Purpose

Build the narrow Rust/provider boundary that owns stable timezone database state and cached named-zone handles.

### Preconditions

G1 passed.

### Consumes

Backend decision record.

### Produces

A native adapter equivalent to:

```text
TimeZoneDatabaseAdapter
ZoneHandle / logical zone key
named lookup
offset lookup
local resolve
transition lookup
system discovery helper
```

### Files and symbols

Expected areas:

```text
phalcom-core/src/time/ or phalcom-core/src/native/time/
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/gc.rs
phalcom-core/src/native/**
phalcom-native-* if descriptor metadata is required
Cargo.toml / Cargo.lock
```

Exact placement follows live native architecture.

### Required implementation shape

- store database adapter at VM/process lifetime according to G1;
- classify new VM state explicitly in exhaustive GC root destructuring;
- cache named zones by canonical identifier;
- provide stable handles/keys rather than raw pointers;
- provide conversion helpers from P1 Timestamp/DateTime/UtcOffset to backend bounded types;
- return explicit range errors from conversion;
- return factual local mapping data without applying public disambiguation policy;
- keep UTC/fixed fast paths outside named DB where practical.

### Forbidden approaches

- one database/parser per TimeZone object;
- raw Rust pointers stored unsafely in Phalcom values;
- policy decisions inside native local resolution;
- backend parser/formatter as public text contract;
- per-send path/string resource lookup.

### Test changes required

Native adapter tests for:

```text
lazy state construction
cache hit identity
known lookup
unknown zone
range error
offset seconds
local unique/fold/gap facts
transition before/after
```

### Tests to run now

Focused native adapter unit/integration filters and `cargo check -p phalcom-core`.

### Tests explicitly deferred

Full source API.

### Acceptance

`P2-CV-04`, `05`, `06`, `12`, `13`, `14`, `15`, `16`.

### Local STOP / CONSULT triggers

- adapter requires temporal VM value specialization;
- cache handle cannot be made GC-safe;
- snapshot coherence breaks under lazy lookup;
- backend facts cannot be represented without lossy offset/time conversion.

### Checkpoint update

Record stable adapter ownership and snapshot semantics.

---

## T3 — Implement TimeZone and ZoneTransition source/runtime surface

### Purpose

Expose UTC, fixed, named, and system timezone values as ordinary Phalcom objects backed by the narrow adapter, plus transition values.

### Preconditions

T2 complete.

### Consumes

P1 `Timestamp`, `DateTime`, `UtcOffset`; T2 adapter.

### Produces

```text
TimeZone
ZoneTransition
TimeZone.utc
TimeZone.fixed
TimeZone.named
TimeZone.system
TimeZone.at
transitionBefore
transitionAfter
```

### Files and symbols

Expected:

```text
phalcom-core/core/universe/src/time/package.ph
new time/timezone.ph or equivalent
new time/zone_transition.ph or equivalent
native source declarations/providers
package exports
semantic/runtime fixtures
```

### Required implementation shape

- `TimeZone` remains opaque ordinary class;
- UTC canonical;
- fixed equality/hash from offset;
- named equality/hash from snapshot + canonical logical identifier;
- `TimeZone.system` observes host configuration through a Native declaration carrying primitive metadata;
- named database methods are Native without primitive metadata;
- `TimeZone.at(timestamp)` builds `ZonedDateTime` through source-level construction;
- `ZoneTransition` uses only timestamp/before/after offsets;
- no eager DB initialization on import/UTC/fixed;
- invalid named identifiers fail explicitly.

### Forbidden approaches

- arbitrary object identity equality for TimeZone;
- public backend handles;
- silent UTC system fallback;
- storing redundant current offset as TimeZone identity;
- public System façade.

### Test changes required

Add source/runtime tests for:

```text
UTC identity
fixed equality/hash
named equality/hash
alias identity rule
invalid name
system success/failure test seam
historical seconds
transition values
lazy import
```

### Tests to run now

Focused time language/runtime filters and native provenance filter.

### Tests explicitly deferred

Gap/fold public policy, ZonedDateTime movement, text.

### Acceptance

`P2-CV-07` through `P2-CV-17`, plus lazy-import evidence.

### Local STOP / CONSULT triggers

- backend alias semantics contradict locked identity model;
- TimeZone equality would require live backend rule comparison;
- system discovery needs a public System API;
- fixed/UTC construction unexpectedly triggers database work.

### Checkpoint update

Record TimeZone semantic identity and public constructors.

---

## G2 — Timezone foundation stable

Pass only if:

```text
[ ] UTC/fixed/named/system forms work
[ ] named identity is stable
[ ] system discovery is explicit
[ ] unknown names fail explicitly
[ ] historical second offset case passes
[ ] transition before/after pass
[ ] no named DB load on import/UTC/fixed
[ ] backend range errors do not narrow P1 types
```

---

## T4 — Implement LocalTimeResolution and LocalTimeDisambiguation

### Purpose

Expose local-time factual ambiguity explicitly and implement public policy in Phalcom source.

### Preconditions

G2 passed.

### Consumes

Native local resolution facts from T2/T3.

### Produces

```text
LocalTimeResolution
LocalTimeDisambiguation
DateTime.resolve(zone)
DateTime.in(zone)
DateTime.in(zone, disambiguation)
```

### Files and symbols

```text
time source modules
enum declarations
inherent DateTime implementation
native fact adapter declarations
semantic/pattern fixtures
```

### Required implementation shape

1. backend returns factual candidates only;
2. source builds `Unique`, `Ambiguous`, `Gap`;
3. `Ambiguous.earlier` has earlier instant;
4. `Ambiguous.later` has later instant;
5. `Gap.before` uses pre-transition offset interpretation;
6. `Gap.after` uses post-transition offset interpretation;
7. disambiguation policy table implemented once in source;
8. one-argument `DateTime.in(zone)` uses Reject.

### Forbidden approaches

- backend enum exposed directly;
- backend `compatible()` result used without preserving raw resolution;
- hiding ambiguity by default;
- treating gap as one ordinary mapping.

### Test changes required

Pinned fixtures for:

```text
Unique
1-hour overlap
1-hour gap
non-hour transition
historical transition
all 4 disambiguation policies
payload ordering
```

### Tests to run now

Focused semantic pattern tests and runtime time-resolution filters.

### Tests explicitly deferred

Zoned advance/shift and text.

### Acceptance

`P2-CV-18` through `P2-CV-28`.

### Local STOP / CONSULT triggers

- backend does not expose enough factual information for Gap.before/after semantics;
- public policy must move into Rust;
- non-hour gap invalidates the chosen payload meaning.

### Checkpoint update

Record finalized gap payload semantics if not already recorded at G1.

---

## T5 — Implement ZonedDateTime semantics

### Purpose

Build the core zoned value as ordinary source semantics over `Timestamp + TimeZone`.

### Preconditions

T4 complete.

### Consumes

P1 arithmetic/calendar semantics, TimeZone, local resolution.

### Produces

```text
ZonedDateTime
Timestamp.in(zone)
derived offset/dateTime/date/time
equality/hash
sameInstant
compareInstant
shift(Duration)
advance(Period)
withTimeZone
```

### Files and symbols

```text
new zoned_datetime.ph or live equivalent
inherent Timestamp implementation
time package exports
runtime/semantic fixtures
reflection expectations
```

### Required implementation shape

- canonical storage exactly Timestamp + TimeZone;
- projections derive through timezone rule lookup;
- equality/hash use timestamp plus timezone equality;
- `Timestamp.in(zone)` is direct absolute conversion;
- `shift` changes timestamp only;
- `advance`:
  1. derive local DateTime;
  2. apply P1 Period civil semantics;
  3. resolve same zone;
  4. apply Compatible;
- `withTimeZone` preserves timestamp;
- `sameInstant`/`compareInstant` ignore zone identity.

### Forbidden approaches

- storing canonical offset/date/time redundantly;
- exact Duration arithmetic on DateTime;
- converting Period to Duration;
- instant-only ordinary equality;
- backend-owned ZonedDateTime as canonical state.

### Test changes required

Required cases:

```text
same instant / same zone equality
same instant / different zone inequality
sameInstant true across zones
withTimeZone timestamp preservation
shift across DST
advance across DST
shift(Duration.hours(24)) differs from advance(Period.days(1))
hash consistency
derived civil projections
```

### Tests to run now

Focused P2 zoned tests only.

### Tests explicitly deferred

Current-time conveniences and text.

### Acceptance

`P2-CV-29` through `P2-CV-38`.

### Local STOP / CONSULT triggers

- correct equality appears to require redundant offset state;
- source implementation requires large Rust policy movement;
- Period advance cannot reuse P1 DateTime/Date calendar semantics.

### Checkpoint update

Record stable ZonedDateTime canonical state and movement laws.

---

## G3 — Zoned semantic stabilization

Pass only if:

```text
[ ] all local resolution states/policies pass
[ ] ZonedDateTime state is non-redundant
[ ] equality/sameInstant distinction passes
[ ] shift/advance distinction passes at known DST boundary
[ ] withTimeZone preserves instant
[ ] no temporal VM specialization added
```

---

## T6 — Add Date.today and ZonedDateTime.now conveniences

### Purpose

Compose P1 wall-clock capabilities with P2 timezone values without creating new host-time sources.

### Preconditions

G3 passed.

### Consumes

`Timestamp.now`, `TimeZone.system`, supplied TimeZone arguments.

### Produces

```text
Date.today
Date.today(in: zone)
ZonedDateTime.now
ZonedDateTime.now(in: zone)
```

### Files and symbols

Inherent `impl` modules for Date/ZonedDateTime and package exports if needed.

### Required implementation shape

Use only:

```text
Timestamp.now
+
zone interpretation
```

Default zone comes from `TimeZone.system`.

### Forbidden approaches

- direct backend now call;
- direct OS date call;
- new wall primitive;
- public System temporal API.

### Test changes required

Deterministic supplied-zone tests and minimal system-zone smoke through an injectable/native test seam where needed.

### Tests to run now

Focused current-time composition tests.

### Tests explicitly deferred

Full `time.testing` package until T9.

### Acceptance

`P2-CV-39`, `P2-CV-40`.

### Local STOP / CONSULT triggers

- convenience API cannot be expressed through Timestamp.now;
- system timezone lookup forces eager import-time work.

### Checkpoint update

Only if a durable convenience signature differs from checkpoint expectation.

---

## T7 — Implement canonical machine formatting

### Purpose

Implement stable, lossless, Phalcom-owned canonical text for P1 and P2 temporal values.

### Preconditions

G3 passed.

### Consumes

§10 text contract.

### Produces

Canonical formatting for:

```text
Date
Time
DateTime
UtcOffset
Timestamp
ZonedDateTime
Duration
Period
```

using the existing Phalcom string representation/toString conventions where appropriate.

### Files and symbols

Time source files, string helpers, tests.

### Required implementation shape

- source implementation where expressible;
- no locale dependencies;
- no backend formatter as public authority;
- fixed canonical output;
- nine fractional digits when Time/Timestamp fraction nonzero;
- second-resolution offsets preserved;
- expanded years supported where P1 permits;
- ZonedDateTime includes numeric offset + bracket zone identity;
- Period representation must preserve mixed-sign values if valid.

### Forbidden approaches

- locale formatting;
- backend display passthrough;
- rounding > nanosecond;
- reducing offsets to minutes;
- Period months-to-days approximation.

### Test changes required

Golden canonical forms covering:

```text
zero/negative/expanded year
fraction zero/nonzero
offset seconds
pre-epoch Timestamp
named/fixed/UTC ZonedDateTime
negative Duration
large Duration
Period positive/negative/mixed-sign
```

### Tests to run now

Focused formatter tests.

### Tests explicitly deferred

Parser round trips until T8.

### Acceptance

Formatting halves of `P2-CV-41` through `P2-CV-49`.

### Local STOP / CONSULT triggers

- existing string protocol cannot produce needed output without a broad unrelated string redesign;
- Period mixed-sign values cannot be represented losslessly under the proposed syntax.

### Checkpoint update

Record final canonical text grammar if a bounded syntax choice was finalized.

---

## T8 — Implement parsing, validation, and offset-zone consistency

### Purpose

Parse canonical/accepted machine temporal text in Phalcom source and establish lossless round trips.

### Preconditions

T7 complete.

### Consumes

P1 constructors/validators, TimeZone lookup, §10 grammar.

### Produces

Parse entry points for the textual types and explicit errors.

### Files and symbols

Time source parser helpers, inherent/static parse methods according to live naming convention, tests.

### Required implementation shape

- parse with ordinary strings/Int;
- use P1 constructors for semantic validation;
- 1–9 fractional digits accepted and right-padded;
- no >9 digit silent rounding;
- Timestamp numeric offset normalized to UTC instant;
- ZonedDateTime parses explicit offset + zone;
- offset-zone consistency verified against zone rules;
- overlap can be selected by explicit offset;
- mismatch rejected;
- parser errors use live general error model.

### Forbidden approaches

- routing all public parsing through Jiff;
- accepting malformed values because backend parser is permissive;
- ignoring explicit zoned offset;
- returning method-missing on parse failure.

### Test changes required

Property/round-trip and negatives:

```text
parse(format(x)) == x
invalid Date
invalid Time
invalid offset
invalid expanded year syntax
fraction > 9
unknown zone
offset-zone mismatch
overlap with each valid offset
gap invalid/mismatch
Duration malformed
Period malformed/mixed-sign
```

### Tests to run now

Focused parse/format tests and property-style bounded loops.

### Tests explicitly deferred

Final full C2 integration.

### Acceptance

`P2-CV-41` through `P2-CV-50`.

### Local STOP / CONSULT triggers

- current error model cannot express necessary failures;
- parser needs locale/regex subsystem redesign;
- zoned offset consistency cannot be checked without backend policy leakage.

### Checkpoint update

Record canonical grammar and parse behavior.

---

## G4 — Textual stabilization

Pass only if:

```text
[ ] every textual type has one canonical formatter
[ ] every canonical form parses
[ ] parse(format(x)) round trips
[ ] offset seconds survive
[ ] expanded/negative year cases follow P1 range
[ ] zoned offset/zone mismatch rejects
[ ] Duration and Period remain semantically distinct
[ ] no locale framework added
```

---

## T9 — Implement time.testing deterministic clocks

### Purpose

Provide ordinary controllable Clock/WallClock implementations for user and stdlib tests without mutating global clock singletons.

### Preconditions

P1 clock abstractions stable; G4 passed.

### Consumes

P1 `Clock`, `WallClock`, `Instant`, `Timestamp`, `Duration`.

### Produces

Preferred:

```text
time.testing.ManualClock
time.testing.ManualWallClock
```

### Files and symbols

```text
phalcom-core/core/universe/src/time/testing/**
package exports
semantic/runtime tests
```

### Required implementation shape

`ManualClock`:

```text
own Clock identity/domain
initial tick
now
advance(nonnegative Duration)
```

`ManualWallClock`:

```text
current Timestamp
now
advance(Duration)
set(Timestamp)
```

No global singleton replacement.

### Forbidden approaches

- reassigning `clock` or `wallClock`;
- replacing `SystemClock.instance`;
- thread-local hidden global time override;
- making testing clocks native when source can express them.

### Test changes required

```text
independent ManualClock domains
advance changes Instant ticks
cross-domain Instant rules still apply
negative ManualClock advance rejected
ManualWallClock set/advance
canonical global clock identity unaffected
```

### Tests to run now

Focused `time.testing` semantic/runtime tests.

### Tests explicitly deferred

Final package suite.

### Acceptance

`P2-CV-51`, `P2-CV-52`, `P2-CV-53`.

### Local STOP / CONSULT triggers

- abstract Clock/WallClock contracts from P1 cannot support source test clocks;
- implementing testing clocks would require global mutation.

### Checkpoint update

Record final testing package API.

---

## T10 — Final ordinary-package, reflection, provenance, GC, and performance closure

### Purpose

Prove the complete C2 time surface behaves as ordinary Phalcom, has correct semantic/reflection categories, uses the intended native boundary, survives GC safely, imports cheaply, and does not create VM specialization.

### Preconditions

T9 complete.

### Consumes

All prior P2 tasks.

### Produces

Final integration evidence.

### Files and symbols

Expected:

```text
ordinary package integration fixtures
phalcom-core/tests/core/modules/runtime.rs
phalcom-core/tests/core/reflection/**
phalcom-core/tests/core/native/**
phalcom-core/tests/core/observability_disassembly* or live equivalent
VM GC tests
performance/resource probes if existing harness supports them
```

### Required implementation shape

#### Ordinary package

Create/update a normal package fixture that:

```text
imports time normally
uses grouped imports
observes prelude Duration/Timestamp
constructs civil values
uses clock/wallClock
uses UTC/fixed/named zones
pattern-matches LocalTimeResolution
uses ZonedDateTime
parses/formats
uses time.testing where appropriate
```

#### Reflection

Prove categories:

```text
TimeZone                  class
ZoneTransition            data
LocalTimeResolution       enum with payload variants
LocalTimeDisambiguation   enum
ZonedDateTime             data
ManualClock               class/subclass
ManualWallClock           class/subclass
```

No backend Rust types appear as public semantic types.

#### Provenance

Audit:

```text
named timezone backend methods
    Native

system timezone discovery
    Native + primitive metadata

calendar/zoned policy
    Phalcom

parser/formatter
    Phalcom
```

#### GC

Force GC around:

```text
named TimeZone
multiple TimeZone values sharing cached handle
ZonedDateTime
LocalTimeResolution payloads
transition values
```

and prove continued correctness.

#### Static/runtime architecture audit

Search for forbidden temporal specialization:

```text
Value::TimeZone
Value::ZonedDateTime
temporal heap variants
RESOLVE_TIMEZONE
MAKE_ZONED
DATE_ADVANCE
compiler recognition of TimeZone/ZonedDateTime names
```

#### Performance/resource sanity

Record evidence for:

```text
plain import
first named lookup
cached named lookup
database lifetime
zone handle sharing
```

Do not create a benchmark framework solely for C2.

### Forbidden approaches

- Universe-only fixture as final proof;
- reflection-specific native shadow types;
- GC roots for backend objects unless truly required;
- optimization-driven semantic changes;
- broad workspace testing before focused gates are green.

### Test changes required

All missing final integration tests.

### Tests to run now

Focused ordinary-package, reflection, native provenance, GC, and disassembly/static filters.

### Tests explicitly deferred

Only broad affected-package/workspace certification to T11.

### Acceptance

`P2-CV-54` through `P2-CV-67`.

### Local STOP / CONSULT triggers

- ordinary package needs visibility exceptions;
- reflection requires backend-specific public representation;
- GC reveals unsafe handle lifetime;
- import triggers eager database loading;
- a temporal opcode/value specialization appears necessary.

### Checkpoint update

Record G5 evidence when coherent.

---

## G5 — Final C2 integration stabilization

Pass only if:

```text
[ ] public P2 works from ordinary package
[ ] reflection categories are correct
[ ] native/primitive provenance is correct under LANG006
[ ] forced GC does not invalidate zone values
[ ] time import remains cheap
[ ] named cache works
[ ] no temporal VM specialization exists
[ ] prelude still contains only Duration/Timestamp
[ ] no legacy implementation spelling is semantic authority
[ ] stale time.md files remain untouched
```

---

## T11 — Certify P2, close C2, and produce durable lifecycle records

### Purpose

Run final focused/affected certification, classify failures, audit the final diff and static invariants, update checkpoint state truthfully, and produce the P2 walkthrough/handoff.

### Preconditions

G5 passed.

### Consumes

All `P2-INV-*` and `P2-CV-*`.

### Produces

```text
STDL002.C2.P2-walkthrough.md
STDL002.C2.P2-handoff.md
updated STDL002.C2-CHECKPOINT.md
```

C2 becomes complete only after these records match the repository.

### Required implementation shape

#### T11.1 Compile affected crates

Run live equivalents:

```sh
cargo check -p phalcom-common
cargo check -p phalcom-ast
cargo check -p phalcom-modules
cargo check -p phalcom-semantic
cargo check -p phalcom-native-meta
cargo check -p phalcom-native-surface
cargo check -p phalcom-core
cargo check -p phalcom-lsp
```

Only include crates actually affected after diff review.

#### T11.2 Focused coverage ledger

Map every `P2-CV-01` through `P2-CV-70` to:

```text
test/command
result
evidence file
deferred/NA reason if legitimately not applicable
```

No coverage ID may disappear silently.

#### T11.3 Affected package suites

Once focused evidence is green, run relevant package suites once.

Likely:

```sh
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-core --test language-corpus
cargo test -p phalcom-core --test core
```

Add native/LSP suites only if touched by P2.

#### T11.4 Static census

Search canonical source/code for:

```text
System.timeZone
System.now
System.wallClock

_$...
@native
@internal
@private
@protected

Value::TimeZone
Value::ZonedDateTime
RESOLVE_TIMEZONE
temporal bytecodes

Jiff public type names in Phalcom-facing reflection/type metadata

stale time.md edits
```

Every remaining hit must be justified.

#### T11.5 Dependency/init review

Confirm:

```text
plain import does not initialize tzdb
UTC/fixed do not initialize named tzdb
only chosen backend dependency added
no locale/CLDR dependency
no general FFI dependency
```

#### T11.6 Diff review

Run:

```sh
git diff --stat
git diff
```

Audit for:

```text
backend policy leaked into public API
duplicate canonical TimeZone state
path/string access control
primitive metadata made operative
specification files edited
P1 semantics reopened
global testing clock override
ambient tzdata-only deterministic tests
```

#### T11.7 Workspace structural certification

Run:

```sh
cargo check --workspace
```

Run `cargo test --workspace` only if repository workflow requires it or focused/affected suites cannot discriminate shared-surface regressions.

Classify unrelated failures A/B/C/D.

### Tests to run now

Focused + affected package suites + workspace check according to the above budget.

### Tests explicitly deferred

Only unrelated baseline failures and release-level certification outside C2.

### Acceptance

All applicable `P2-CV-*` have evidence; `P2-CV-68`, `69`, and `70` pass.

### Local STOP / CONSULT triggers

- final spec/API/source disagree;
- one semantic failure persists after one serious correction;
- a coverage hole requires weakening an invariant;
- broad failure indicates architecture rather than a local defect.

### Checkpoint update

Record:

```text
P2 status = COMPLETE
P2 verification truth
C2 status = COMPLETE
established P2 interfaces
backend/version/snapshot
tests actually run
tests deferred
baseline failures
consultations/amendments
specification = USER-DEFERRED
next action
```

---

# 18. Global STOP / CONSULT triggers

Stop immediately if any of the following occurs:

1. P1 walkthrough/handoff is missing or materially incomplete.
2. P1 public semantics conflict with the C2 checkpoint without a recorded amendment.
3. LANG006 implementation-kind/metadata semantics conflict with the primitive reconciliation in §1.2.
4. correct timezone behavior would require making `@primitive` operative.
5. correct visibility would require spelling/path exceptions.
6. named-zone backend cannot provide clean factual unique/fold/gap information.
7. the selected backend cannot support a stable coherent VM/process database snapshot without a materially different ownership architecture.
8. timezone handles introduce uncertain GC/root/lifetime behavior.
9. public TimeZone equality would need raw backend pointer identity.
10. backend range pressure would narrow P1 Timestamp/civil types.
11. second-resolution historical offsets would be lost.
12. system-zone discovery would need silent UTC fallback.
13. ZonedDateTime appears to require redundant canonical state.
14. `shift`/`advance` laws cannot be implemented without reopening P1 Duration/Period semantics.
15. canonical text cannot round-trip all valid Period values.
16. parse error requirements force a broad new exception hierarchy.
17. deterministic timezone semantics can only be tested against ambient mutable host tzdata.
18. a temporal `Value` variant, heap variant, opcode, or compiler-recognized temporal name appears necessary.
19. implementation pressure expands into timers/reactor redesign.
20. implementation pressure expands into locale/CLDR.
21. implementation pressure expands into general FFI/native-extension redesign.
22. implementation pressure expands into dunder redesign.
23. ordinary-package behavior differs from Universe behavior because of accidental privilege.
24. native timezone code receives blanket internal privilege.
25. one semantic failure persists after one serious evidence-driven correction.
26. two plausible fixes differ materially in timezone identity, database lifetime, GC ownership, or public semantics.
27. an invariant must be weakened merely to make existing tests pass.
28. specification work appears necessary merely because older planning text mentioned it; this remains USER-DEFERRED.

A triggered consultation cannot be self-waived.

---

# 19. Debugging budget

## 19.1 Mechanical failures

Allow up to three coherent cycles:

```text
inspect
identify concrete cause
make one coherent correction
run smallest discriminating test
```

Examples:

```text
renamed P1 selector
moved source file
backend API minor-version drift
missing native descriptor
exhaustive enum match
test filter drift
```

## 19.2 Semantic failures

Before editing, record:

```text
Observed:
Expected:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same semantic cause.

If it persists, STOP AND CONSULT.

## 19.3 Architectural failures

Zero speculative architecture fixes.

Consult immediately.

## 19.4 Test rerun rule

Do not rerun an unchanged failing test unless relevant code/configuration/evidence changed.

---

# 20. Verification gates

## G1 — Backend architecture

Required:

```text
P2-CV-01..06
P2-CV-12
fixture strategy
```

## G2 — Timezone foundation

Required:

```text
P2-CV-07..17
lazy import proof
range proof
```

## G3 — Zoned semantics

Required:

```text
P2-CV-18..40
```

## G4 — Textual forms

Required:

```text
P2-CV-41..50
```

## G5 — Integration closure

Required:

```text
P2-CV-51..67
```

## Final certification

Required:

```text
P2-CV-68..70
all invariant review
diff/static census
checkpoint lifecycle records
```

No gate may be passed solely because a broader suite is green.

---

# 21. Final focused acceptance

At final acceptance, all applicable coverage rows must have concrete evidence.

Minimum semantic fixture matrix:

```text
UTC
fixed +03:00
fixed second-resolution offset

America/New_York
    known overlap
    known gap

one non-one-hour transition zone
    e.g. Lord Howe or equivalent pinned fixture

one historical second-resolution zone/transition

Europe/Istanbul or another stable named-zone fixture

one system timezone discovery smoke test
```

Pinned test data should be used for deterministic rule assertions.

Host/system database tests should verify integration capability, not volatile historical expectations.

---

# 22. Performance/resource evidence

P2 does not require a broad performance optimization effort.

It does require architectural sanity evidence.

Record, where cheaply observable:

```text
plain time import
    database init count = 0

TimeZone.utc
    named DB init count unchanged

TimeZone.fixed
    named DB init count unchanged

first TimeZone.named("X")
    lazy init/load occurs as designed

second TimeZone.named("X")
    cache hit

multiple ZonedDateTime projections
    no database reopen

GC
    cached backend zone remains valid

database lifetime
    one adapter/snapshot per intended VM/process domain
```

If an existing allocation/perf harness can cheaply measure:

```text
first named lookup allocations
cached named lookup allocations
adapter memory
zone-handle memory
```

record them as sanity evidence.

Do not add semantic specialization merely to improve these numbers.

---

# 23. Checkpoint bookkeeping

At P2 start:

```text
active_plan = STDL002.C2.P2
record execution revision
record P1 completion consumption
record LANG006 primitive reconciliation
record time specification USER-DEFERRED
```

During P2, update checkpoint only for durable events:

```text
backend/version/snapshot selected
TimeZone identity finalized
Gap payload semantics finalized
canonical text grammar finalized
consultation/amendment
meaningful baseline/deferred failure
coherent gate completion
```

At P2 completion record:

```text
status/completion/verification
public P2 interfaces
backend/version/features
database lifetime model
tests run
tests deferred
baseline failures
consultations
specification deferral
C2 completion
next action
```

Do not create a competing P2 checkpoint-state file.

---

# 24. Walkthrough deliverable

Create:

```text
STDL002.C2.P2-walkthrough.md
```

It must contain at least:

1. execution baseline;
2. P1 handoff consumed;
3. LANG006 reconciliation;
4. backend version/features;
5. database snapshot architecture;
6. adapter ownership/lifetime;
7. TimeZone semantic identity;
8. alias/canonical-name rule;
9. backend interop ranges;
10. ZoneTransition shape;
11. local resolution factual mapping;
12. Gap.before/after evidence;
13. disambiguation policy;
14. ZonedDateTime state/equality;
15. shift/advance examples;
16. current-time composition;
17. text grammar;
18. parser/error behavior;
19. `time.testing` API;
20. ordinary-package evidence;
21. reflection evidence;
22. native/primitive provenance inventory;
23. GC/lifetime evidence;
24. lazy-init/cache/resource evidence;
25. no-VM-specialization audit;
26. full `P2-CV-*` evidence ledger;
27. failures/classifications;
28. consultations/amendments;
29. specification USER-DEFERRED statement;
30. final checkpoint state.

---

# 25. Handoff deliverable

Create:

```text
STDL002.C2.P2-handoff.md
```

Because P2 closes C2, the handoff is not another time implementation plan. It records the stable completed temporal architecture for later stdlib/runtime work.

Include:

```text
C2 completion revision
public time API summary
prelude summary
P1/P2 semantic laws
backend/version/features
database snapshot/lifetime model
TimeZone identity rule
backend interop ranges
canonical text grammar
time.testing surface
native/primitive provenance
ordinary-package proof
reflection categories
GC/resource facts
known baseline failures
deferred specification status
explicit non-goals still deferred
recommended next STDL002 checkpoint
```

Do not draft a new timezone redesign from the handoff.

---

# 26. Completion truth table

| Condition | Required for P2 complete | Required for C2 complete |
|---|---:|---:|
| P1 complete and consumed | yes | yes |
| backend selected/verified | yes | yes |
| stable DB snapshot | yes | yes |
| TimeZone complete | yes | yes |
| transitions complete | yes | yes |
| local resolution complete | yes | yes |
| ZonedDateTime complete | yes | yes |
| current-time conveniences complete | yes | yes |
| formatting complete | yes | yes |
| parsing complete | yes | yes |
| time.testing complete | yes | yes |
| ordinary-package certification | yes | yes |
| reflection/provenance certification | yes | yes |
| GC/lifetime certification | yes | yes |
| performance/resource sanity | yes | yes |
| focused coverage ledger | yes | yes |
| P2 walkthrough | yes | yes |
| P2 handoff | yes | yes |
| checkpoint updated | yes | yes |
| stale time spec edited | **must be no** | **must be no** |
| authoritative time spec | USER-DEFERRED | USER-DEFERRED |
| workspace tests fully green | not necessarily; classified evidence required | not necessarily; classified evidence required |

---

# 27. Plan self-review

Before declaring this plan ready for implementation, verify:

## Architecture

```text
[ ] backend is a rule engine, not Phalcom's semantic model
[ ] TimeZone has explicit semantic identity
[ ] ZonedDateTime state is Timestamp + TimeZone only
[ ] P1 ranges are not narrowed by backend
[ ] database snapshot ownership is explicit
[ ] GC ownership is explicit
```

## Language transition

```text
[ ] LANG006 native/primitive model is used
[ ] @primitive remains metadata-only
[ ] future visibility rules are assumed
[ ] no old implementation selector/field convention is used
```

## Time semantics

```text
[ ] local ambiguity is explicit
[ ] Gap.before/after meaning is specified
[ ] disambiguation table is specified
[ ] shift and advance are distinct
[ ] current-time conveniences reuse Timestamp.now
[ ] System façade remains absent
```

## Text

```text
[ ] canonical machine forms are stable
[ ] offset seconds survive
[ ] expanded years are addressed
[ ] fractions are nanosecond-lossless
[ ] zoned offset/zone consistency is required
[ ] Period mixed-sign round trip is addressed
[ ] no locale framework is added
```

## Testing

```text
[ ] pinned timezone fixtures are required
[ ] ordinary package is final authority proof
[ ] reflection/provenance are covered
[ ] GC/lifetime are covered
[ ] lazy initialization is covered
[ ] system integration is separate from deterministic rule tests
[ ] BUILD/STABILIZE/CERTIFY budget is explicit
```

## Lifecycle

```text
[ ] P1 walkthrough/handoff is a hard precondition
[ ] checkpoint amendments are explicit
[ ] P2 walkthrough/handoff are required
[ ] C2 closure criteria are explicit
[ ] stale time.md remains excluded
[ ] spec status is USER-DEFERRED
```

If all of these remain true at execution takeover, this plan is ready to be followed as the patch-grade implementation authority for `STDL002.C2.P2`.
