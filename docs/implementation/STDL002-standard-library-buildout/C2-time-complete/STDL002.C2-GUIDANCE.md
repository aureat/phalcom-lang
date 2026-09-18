---
id: STDL002.C2.GUIDANCE
category: STDL
program: STDL002
checkpoint: STDL002.C2
kind: implementer-guidance
status: ACTIVE_PLANNING_GUIDANCE
requires:
  - STDL002.C2-CHECKPOINT.md
prepared: 2026-09-17
updated: 2026-09-18
repository: aureat/phalcom-lang
planning_baseline_revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
latest_planning_baseline_observed: 206e0ab91129e189782c4028de5a40379bcc033c
---

# Implementer Guidance — STDL002.C2 `time` Upgrade

## 0. Purpose

This file is the persistent architectural and execution guidance for all `STDL002.C2` implementation sessions.

C2 has exactly two implementation plans:

```text
C2.P1
    temporal foundation + C1 migration
    absolute wall time
    civil Gregorian calendar
    fixed offsets
    package/prelude exposure
    minimum prerequisite language closure where still missing

C2.P2
    stable timezone database/backend lifetime
    TimeZone / transitions / local ambiguity
    zoned semantics
    current local conveniences
    canonical parsing/formatting
    deterministic test clocks
    ordinary-package/reflection/provenance/GC/performance closure
    final C2 checkpoint certification
```

The checkpoint's core architectural sentence is:

> The `time` package is ordinary Phalcom source first: transparent immutable temporal values use `data`, identity/resource capabilities use classes, Rust/provider code enters only through explicit Native boundaries, genuinely irreducible host observations may be classified with non-operative `@primitive` metadata, visibility is lexical and identity-based, and Universe status never justifies spelling-based access, implicit privilege, temporal VM special cases, or a `System` date/time façade.

Every C2 patch must preserve that sentence.

Two amendments are already binding:

1. LANG006 defines implementation kind as `Phalcom`, `Native`, or `Foreign`; `@primitive` / `@intrinsic` are independent non-operative metadata. Older C2/P1 language describing a primitive implementation kind is superseded.
2. Existing `time.md` material is stale and excluded. Authoritative time specification work is `USER-DEFERRED` and is not a C2 completion requirement unless explicitly re-enabled.

This file is guidance, not a competing status log. Lifecycle truth belongs in `STDL002.C2-CHECKPOINT.md`. Exact tasks, files, tests, and gates belong in the authored P1/P2 patch-grade plans.

# 1. Required read order

Before production edits for C2, read in this order.

## 1.1 Repository/workflow authority

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/sol-principal-architect-project-doctrine.md
```

The project planning doctrine assigns architecture, ownership, invariants, test-surface design, and escalation criteria to the principal architect; the implementer should act narrowly under those decisions.

The patch-plan schema requires P1/P2 execution to separate fixed architecture, mechanical flexibility, verify-first assumptions, bounded testing, and STOP/CONSULT triggers.

## 1.2 Program/checkpoint authority

```text
docs/implementation/STDL002-standard-library-buildout/STDL002-PROGRAM.md
docs/implementation/STDL002-standard-library-buildout/C1-time/STDL002.C1-CHECKPOINT.md
docs/implementation/STDL002-standard-library-buildout/C1-time/STDL002.C1.P1-monotonic-clock-instant-duration.md
STDL002.C2-CHECKPOINT.md
STDL002.C2-GUIDANCE.md
```

For P1 execution also read:

```text
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md
latest P1 walkthrough/handoff if continuing a partially completed implementation
```

For P2 execution also read:

```text
STDL002.C2.P2-plan.md
completed STDL002.C2.P1-walkthrough.md
completed STDL002.C2.P1-handoff.md
LANG006.C1-CHECKPOINT.md
LANG006.C1.P1-HEADER.md
LANG006.C1.P1-BODY.md
```

C1 is predecessor evidence, not current C2 architecture authority where C2 explicitly changes the surface.

Earlier references to `STDL002.C2.P1-plan-01-body.md` resolve to the installed `STDL002.C2.P1-01-body.md`; do not create a duplicate body.

## 1.3 Normative language authority

Read the live authoritative equivalents for:

```text
modules/imports/exports
visibility/access control
classes/inheritance/abstract declarations
data declarations/construction/invariants
enums/payload enums/pattern matching
ImplementationKind::Phalcom / Native / Foreign
@intrinsic and @primitive metadata
native/foreign reflection and metadata provenance
generic data
inherent impls/extensions
errors
reflection
```

LANG006 is the current authority for implementation kind and metadata semantics. Do not trust older checkpoint/header wording that treated `primitive` as an implementation kind.

## 1.4 Time implementation

Inspect the live equivalents of:

```text
phalcom-core/core/universe/src/time/package.ph
phalcom-core/core/universe/src/time/clock.ph
phalcom-core/core/universe/src/package.ph

System.clock / current monotonic native bridge
System.sleep(Duration)
Future.timeout(Duration)
reactor deadline conversion
semantic time capability tests
runtime/language-corpus time tests
reflection/native-surface/metadata-floor invariants
```

Do **not** read existing `time.md` documents as semantic authority. In particular, P1/P2 do not depend on:

```text
docs/spec/current/stdlib/time.md
docs/implementation/time.md
```

unless a later explicit user instruction re-enables specification work.

Do not re-audit unrelated standard-library packages.

# 2. Authority and drift rules

Use this order when authorities disagree:

```text
1. explicit user-ratified C2 decisions / approved checkpoint amendments
2. current authoritative language specs, including LANG006
3. STDL002.C2-CHECKPOINT.md durable invariants
4. this guidance
5. completed C2 walkthroughs/handoffs
6. active P1/P2 patch-grade plan architecture/mechanics
7. C1 as-built behavior that C2 has not superseded
8. live implementation mechanics
9. historical plans/drafts/legacy tests
```

For mechanical details:

```text
live repository
    >
planning-time helper names / paths / private signatures
```

Mechanical drift is adaptable.

Architectural drift in visibility identity, declaration implementation kind, metadata semantics, clock identity, temporal representation, timezone lifetime/snapshot/identity, semantic authority, or public API requires consultation.

Do not let a stale test re-ratify C1 surface that C2 intentionally supersedes.

Do not let stale `time.md` material become semantic authority merely because an older P1 header/checkpoint revision mentioned it.

The approved amendments are interpretation rules for older planning text:

```text
older "primitive implementation kind"
    -> Native implementation + @primitive metadata when irreducible

older "native/primitive distinction"
    -> Native implementation versus primitive-floor metadata classification

older "final time specification"
    -> USER-DEFERRED, not a C2 completion gate
```

These are architectural reconciliations, not optional executor choices.

# 3. Session bootstrap

At every C2 implementation session begin with:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then identify:

```text
active C2 plan/task/gate
predecessor walkthrough/handoff
current language-transition state
unrelated local edits
smallest discriminating test
applicable invariant IDs from C2-CHECKPOINT
applicable STOP/CONSULT trigger
```

Preserve unrelated work.

Do not reset, clean, broadly format, or broadly stage the repository.

Do not begin by running workspace tests.

---

# 4. Language-transition takeover rule

The live C1 time implementation predates the intended final visibility and LANG006 implementation-kind model.

C2 therefore treats these as **legacy takeover artifacts**:

```text
special Universe implementation-access attributes
implementation-only selector prefixes
the System-owned monotonic host-read seam
Clock's legacy hidden singleton accessor
Instant's legacy hidden construction/accessor selectors
legacy primitive registration assumptions
```

They are not the target pattern.

Before editing them, verify the current language support for:

```text
private
protected
module
package
internal

ImplementationKind::Phalcom
ImplementationKind::Native
ImplementationKind::Foreign

@intrinsic
@primitive

grouped tuple-shaped imports/exports
```

If the new language work is present, migrate time directly to it.

If it is partially landed:

- adapt mechanically where semantics are already fixed;
- do not create a time-specific compatibility layer;
- do not reintroduce spelling-based access;
- do not make metadata attributes operative;
- do not recreate a primitive implementation kind for time;
- STOP AND CONSULT if C2 would duplicate or contradict the active language transition.

C2 is a production consumer of the new model, not its alternative implementation.

# 5. Visibility rules to keep loaded

Visibility is not a naming convention.

| Visibility | Canonical rule |
|---|---|
| Public | ordinary resolution/import makes it accessible |
| `private` | same declaring lexical class |
| `protected` | declaring class + eligible subclasses |
| `module` | exact defining `Module` identity |
| `package` | exact defining `Package` identity |
| `internal` | exact internal `Project` / domain relation |

Important consequences for time:

```text
Clock.instant(...)
    protected
    because subclass implementations need it

SystemClock singleton storage/constructor helper
    private where possible

Gregorian helpers shared within one calendar module
    module where appropriate

helpers shared across time modules
    package where appropriate

timezone backend
    normally package/module scoped and non-exported

internal
    only if time must intentionally share a declaration across the
    internal Universe Project boundary
```

Do not use `internal` to mean "native", "primitive", "stdlib-only", or "not documented."

Do not infer accessibility from:

```text
leading underscores
special selector prefixes
special field prefixes
source paths
module spelling
package spelling
```

Protected access must be checked through inheritance eligibility, not by treating it as "wider private."

---

# 6. Metadata versus implementation kind

Keep these axes separate.

## 6.1 Implementation kind

LANG006 normalizes declaration implementation kind to:

```text
Phalcom
    body is implemented in ordinary Phalcom source

Native
    body is supplied by Phalcom's native implementation provider
    (typically Rust in the current implementation)

Foreign
    body crosses an external interoperability boundary
```

There is no `primitive` implementation kind.

## 6.2 Metadata

```text
@intrinsic
@primitive
```

These are Universe-authored informational metadata.

They do not:

```text
supply a body
select native linkage
select foreign linkage
change visibility
change identity
change representation
grant Universe privilege
force bytecode/intrinsic lowering
alter typechecking
```

`@primitive` records an irreducible host/runtime-floor classification where appropriate. It is not what makes a declaration Native and it must never become an implementation registry key.

Expected C2 combination for irreducible host observations:

```text
Native declaration
+ @primitive metadata
```

Expected combination for timezone database/rule operations:

```text
Native declaration
without @primitive merely because the implementation is Rust-backed
```

`@intrinsic` should not appear in time merely to encourage optimization. Use it only if a separately ratified intrinsic fact exists.

# 7. Phalcom/Native/primitive-metadata decision test

Before assigning implementation kind or primitive metadata, ask two separate questions.

First:

```text
Can this behavior be correctly derived in ordinary Phalcom from values and
language facilities already available to the program?
```

- If yes, `ImplementationKind::Phalcom` is the default.
- If yes but the operation intentionally delegates a substantial implementation to Rust/provider code or external rule data for engineering reasons, `ImplementationKind::Native` may be appropriate.
- If it is an external interoperability boundary rather than a Phalcom-owned provider implementation, use `ImplementationKind::Foreign` according to LANG006. C2 does not expect this for ordinary time behavior.

Second, independently:

```text
Does the operation expose an irreducible host/runtime fact unavailable from
existing Phalcom values?
```

If yes, the Native declaration may be marked `@primitive`.

Expected C2 classification:

| Operation | Implementation kind | Metadata |
|---|---|---|
| observe monotonic host clock | Native | `@primitive` |
| observe wall host clock | Native | `@primitive` |
| discover host configured timezone | Native | `@primitive` |
| open/query IANA/TZif rule database | Native | none required |
| Gregorian arithmetic | Phalcom | none |
| Duration/Timestamp arithmetic | Phalcom | none |
| fixed-offset conversion | Phalcom | none |
| local ambiguity policy | Phalcom | none |
| ZonedDateTime equality/shift/advance | Phalcom | none |
| ISO parsing/formatting | Phalcom | none |

Performance does not convert Phalcom behavior into primitive-floor behavior.

Do not use `@primitive` to imply Native linkage; do not use Native linkage to imply `@primitive`.

# 8. Object-model taxonomy

Use the following test before choosing representation.

## `data`

Use when:

```text
value is immutable
complete semantic state is legitimately transparent
same structural state means same value
```

Time examples:

```text
Duration
Measurement<T>
Timestamp
Date
Time
DateTime
YearMonth
MonthDay
Period
UtcOffset
ZoneTransition
ZonedDateTime
```

Do not store derived/redundant fields in data.

Bad:

```text
Date(year, month, day, weekday, dayOfYear)
ZonedDateTime(timestamp, zone, offset, dateTime)
```

Good:

```text
Date(year, month, day)
ZonedDateTime(timestamp, timeZone)
```

## `enum`

Use for closed choices/results:

```text
Month
Weekday
DateOverflow
LocalTimeDisambiguation
LocalTimeResolution
```

## abstract `class`

Use for nominal source-capability families with shared concrete behavior:

```text
Clock
WallClock
```

## opaque `class`

Use when public structural state is not the whole operational meaning:

```text
Instant
SystemClock
SystemWallClock
TimeZone
timezone backend/database
```

Immutability alone does not imply `data`.

---

# 9. Abstract clock guidance

Target:

```phalcom
@abstract
class Clock {
  @abstract
  now -> Instant

  since(_ instant: Instant) -> Duration {
    now.since(instant)
  }

  measure<T>(_ action: () -> T) -> Measurement<T> {
    ...
  }

  // visibility: protected
  instant(_ ticks: Int) -> Instant {
    ...
  }
}
```

Exact visibility syntax must match the live language.

Do not replace the abstract requirement with a throwing concrete placeholder simply because abstract-member obligations are incomplete.

If bodyless abstract members are not yet implemented, P1 may close the minimal general language feature only if doing so does not conflict with active language architecture.

The important capability rule is:

> A Clock implementation may mint Instants in its own domain; arbitrary callers may not manufacture raw clock coordinates.

The exact private/protected construction mechanism may adapt to live constructor/data syntax while preserving that rule.

---

# 10. System clocks and package-owned singleton values

`SystemClock` is an ordinary public/exported class.

It stores/owns the canonical singleton:

```text
SystemClock.instance
```

The package exports:

```text
clock
```

bound to that exact instance.

Expected relation:

```text
clock === SystemClock.instance
```

Do not hide `SystemClock` merely because direct use is uncommon.

Do not expose a public `Clock.new` that manufactures additional host clock domains.

Custom clock domains arise from subclasses such as testing/simulation clocks.

Apply the analogous model to:

```text
SystemWallClock.instance
wallClock
```

Clock capabilities may initialize eagerly; initialization must not itself read time or initialize tzdb.

---

# 11. Prelude and import guidance

The time package is an explicit library namespace.

Typical explicit usage:

```phalcom
from time import (
  Date,
  DateTime,
  Month,
  Period,
  Time,
  TimeZone,
  ZonedDateTime,
  clock
)
```

The root/prelude promotes only:

```text
Duration
Timestamp
```

Therefore ordinary code may write:

```phalcom
Duration.seconds(5)
Timestamp.now
```

without importing `time`, while:

```phalcom
Date.today
clock.now
TimeZone.system
```

require the appropriate `time` import.

Grouped export forms should mirror grouped import organization where multiple names are published.

Do not create aliases or duplicate classes to implement prelude promotion. The prelude binding must refer to the canonical `time` declaration identity.

---

# 12. P1 architectural boundary

P1 is not "some easy time types." It establishes every source/model contract that P2 depends on.

P1 must leave these stable:

```text
Duration representation and exact-unit semantics
Measurement<T>
Clock abstract API
SystemClock singleton/export model
Instant domain/equality/order laws
Timestamp representation/timeline
WallClock abstraction
SystemWallClock singleton/export model
Month / Weekday
Date / Time / DateTime
YearMonth / MonthDay
Period
DateOverflow
UtcOffset
package module topology
prelude exposure
Native host-clock ownership + primitive metadata classification
visibility policy for time internals
```

P1 must not pre-implement named timezone semantics in ad hoc form.

In particular, do not make `UtcOffset` pretend to be `TimeZone`.

---

# 13. P2 architectural boundary

P2 builds only after the P1 value/capability model is stable and its completed walkthrough/handoff has been consumed.

P2 owns:

```text
TimeZone database adapter and stable snapshot/generation lifetime
cached named-zone handles
TimeZone UTC/fixed/named/system forms and semantic identity
ZoneTransition
LocalTimeResolution
LocalTimeDisambiguation
ZonedDateTime
system-local current date/time conveniences
zone conversions
DST gap/overlap behavior
backend range conversion without narrowing P1 values
canonical machine formatting and parsing
offset/zone consistency
lossless round trips
time.testing deterministic clocks
ordinary-package equivalence certification
reflection/type-observation certification
Native/@primitive provenance certification
GC/lifetime certification
lazy-init/performance/resource sanity
final C2 checkpoint walkthrough/handoff closure
```

Authoritative time specification work is `USER-DEFERRED` and is not part of P2 acceptance unless explicitly re-enabled.

P2 must not reopen:

```text
Duration vs Period distinction
Clock identity law
Timestamp timeline representation
DateTime timezone-free meaning
UtcOffset fixed-value meaning
prelude policy
System façade removal
LANG006 implementation-kind/metadata semantics
specification deferral
```

without checkpoint amendment.

# 14. Temporal semantic laws

Keep these laws explicit in code/tests.

## 14.1 `Duration`

```text
signed exact nanoseconds
arbitrary-precision Int backing semantics
negative values valid
sleep/wait consumers reject invalid negative waits at their own boundary
```

Initially avoid `Duration.days/weeks`; exact 24 hours is `Duration.hours(24)`.

## 14.2 `Timestamp`

```text
signed nanoseconds from Unix epoch
POSIX/non-leap timeline
no leap-second civil value
Duration arithmetic only
```

Negative split must use floor/Euclidean semantics.

## 14.3 Civil values

```text
proleptic Gregorian
Time second 0..59
Date validation shared across construction/parser/arithmetic
DateTime = Date + Time
DateTime has no inherent timezone
DateTime does not gain general exact Duration arithmetic
```

## 14.4 `Period`

Canonical fields:

```text
months
days
```

Normalization:

```text
years -> months
weeks -> days
```

Application:

```text
months
overflow resolution
days
```

No total ordering.

## 14.5 `UtcOffset`

Canonical state is signed seconds.

Do not store redundant hours/minutes/seconds.

Second-level historical offsets must remain representable if the selected timezone backend exposes them.

---

# 15. Zoned semantics

Canonical `ZonedDateTime` state:

```text
Timestamp + TimeZone
```

Derived only:

```text
offset
DateTime
Date
Time
civil components
```

Do not store derived offset/civil fields as independent canonical equality state.

Exact movement:

```text
shift(Duration)
    change timestamp coordinate
    preserve TimeZone
```

Calendar movement:

```text
advance(Period)
    derive local DateTime
    apply P1 civil Period semantics
    resolve through same TimeZone
    use Compatible disambiguation
```

Zone conversion:

```text
withTimeZone(other)
    preserve Timestamp
    change TimeZone interpretation
```

Equality:

```text
same Timestamp + same TimeZone semantic identity
```

Instant comparison is explicit:

```text
sameInstant
compareInstant
```

Do not make structural equality silently instant-only.

Current-time conveniences compose the canonical P1 wall-clock path rather than observing time independently:

```text
Date.today
    Timestamp.now + TimeZone.system -> Date

Date.today(in: zone)
    Timestamp.now + zone -> Date

ZonedDateTime.now
    Timestamp.now + TimeZone.system

ZonedDateTime.now(in: zone)
    Timestamp.now + zone
```

Do not add Native `Date.today`, Native `ZonedDateTime.now`, backend-specific `now()`, or a public `System` temporal discovery façade.

# 16. Local-time ambiguity

P2 must never assume every `DateTime + TimeZone` maps to one instant.

Resolution states:

```text
Unique(value)
Ambiguous(earlier, later)
Gap(before, after)
```

Required payload ordering:

```text
Ambiguous.earlier
    earlier absolute instant

Ambiguous.later
    later absolute instant
```

Required gap semantics:

```text
Gap.before
    requested local DateTime interpreted with the pre-transition offset

Gap.after
    requested local DateTime interpreted with the post-transition offset
```

The backend supplies factual rule results. Phalcom source owns public enum construction and disambiguation policy.

A Native timezone operation must return enough factual information for Phalcom to preserve unique/overlap/gap distinctions. Do not bury `Earlier` / `Later` / `Compatible` policy inside Rust.

Policy table:

| Case | Reject | Earlier | Later | Compatible |
|---|---|---|---|---|
| Unique | value | value | value | value |
| overlap | error | earlier | later | earlier |
| gap | error | before | after | after |

Default guidance:

```text
DateTime.in(zone)
    Reject by default

ZonedDateTime.advance(Period)
    Compatible by default
```

The exact gap payload semantics must be verified against at least:

```text
one ordinary one-hour gap
one non-one-hour transition
one historical transition
```

If the backend cannot expose these facts cleanly without applying its own public policy, STOP AND CONSULT.

# 17. Timezone backend/lifetime and TimeZone identity guidance

Prefer a mature Rust backend such as Jiff behind a Phalcom-owned Native adapter, subject to P2's fresh dependency/API verification at execution time.

Public API must not expose backend/Jiff types.

Preferred ownership:

```text
VM/process lifetime timezone database snapshot/generation
    canonical database adapter
    cached named-zone handles
    TimeZone source objects store stable lightweight logical handle/identity
```

Requirements:

```text
database init lazy
no tzdb load at plain time import
UTC/fixed zones do not require named tzdb initialization
one coherent database snapshot/generation per intended VM/process lifetime
existing TimeZone objects do not silently change rules mid-process
named lookup reuses cached handles
backend finite ranges are explicit interop errors, not P1 range definitions
tests use controlled/pinned data or explicit fixtures where host tzdata drift matters
system timezone discovery is an integration capability test, not the sole rule oracle
no per-TimeZone custom GC ownership if a stable adapter/handle model works
```

The default backend global database/cache is not automatically acceptable merely because it is convenient. P2 must prove that its invalidation/reload behavior is compatible with the stable-snapshot invariant or wrap/select a database source that is.

`TimeZone` semantic identity is:

```text
UTC
    canonical UTC identity

fixed
    UtcOffset value identity

named
    database snapshot/generation identity + stable canonical logical identifier

system
    no distinct mutable identity category after discovery;
    TimeZone.system returns an ordinary resolved TimeZone value
```

Named `TimeZone` equality must not use raw backend pointer equality. Alias/canonical-name behavior is verify-first and must be locked before final equality/hash behavior.

`TimeZone.system` performs host discovery on invocation. Previously returned zones remain stable if host configuration later changes. Discovery failure is explicit and must not silently substitute UTC.

Backend ranges never narrow P1's `Timestamp` or civil semantics. Named-zone operations outside the backend's supported conversion range fail explicitly; UTC/fixed operations should continue through Phalcom arithmetic wherever P1 values remain valid.

Second-resolution historical offsets must survive backend conversion and `UtcOffset` representation where data and P1 bounds permit them.

STOP AND CONSULT if the chosen backend forces materially different lifetime, identity, ambiguity, range, offset precision, or GC semantics.

# 18. Textual forms

P2 owns one stable machine grammar, not a locale presentation framework.

Canonical families include:

```text
Date             YYYY-MM-DD with expanded signed years where P1 range permits
Time             HH:MM:SS[.fffffffff]
DateTime         <Date>T<Time>
UtcOffset        ±HH:MM or ±HH:MM:SS when second precision is required
Timestamp        <Date>T<Time>Z
ZonedDateTime    <Date>T<Time><offset>[<zone-identity>]
Period           lossless calendar months+days form
Duration         lossless exact-time form without calendar-month/year semantics
```

P2's concrete plan further locks these requirements:

```text
nanosecond precision is lossless
fractions parse at 1..9 digits with no silent >9-digit rounding
second-resolution historical offsets survive text round trips
expanded/negative years follow the P1 civil range
Timestamp text normalizes accepted numeric offsets to an absolute value
ZonedDateTime text containing both offset and zone validates their consistency
explicit overlap offsets must not be ignored
Period formatting/parsing must round-trip every valid months+days value,
including mixed-sign fields if P1 permits them
```

Parsing stays Phalcom-owned where expressible. The Native timezone backend may validate timezone facts but does not own the public accepted grammar or canonical formatter.

C2 does not add locale/CLDR presentation, natural-language parsing, or a general `strftime`-style formatting language.

# 19. No time-specific trait hierarchy

Do not respond to repeated method names by inventing broad temporal traits.

Rejected by default:

```text
Temporal
TemporalAmount
TemporalPoint
DateLike
TimeLike
ClockLike
ZoneLike
```

The concrete distinctions are semantically important.

If a later generic API demonstrates a real shared law, propose it separately with consumers and laws, not preemptively during C2.

---

# 20. Compiler/runtime constraints

C2 should exercise the normal language pipeline.

Do not add by default:

```text
Value::Duration
Value::Timestamp
Value::Date
Value::Instant
Value::TimeZone

temporal heap object variants

MAKE_DATE
ADD_TIMESTAMP
DATE_ADVANCE
RESOLVE_TIMEZONE
other temporal bytecodes

special temporal dispatch
compiler recognition of Date/Duration names
```

If profiling later justifies transparent specialization, that is separate optimization work.

Runtime should see normal classes/data/enums and ordinary method dispatch except at explicit Native/Foreign provider boundaries.

---

# 21. Language-feature certification intent

C2 intentionally serves as a production acceptance test for the ordinary language/runtime pipeline.

Expected exercised language features include:

```text
data
validated data
generic data
nested data
enums
payload enums
pattern matching
abstract classes
abstract methods
inheritance
protected access
module/package/internal visibility
class-side singleton state
exported module values
grouped imports/exports
prelude canonical re-export
inherent implementations
arbitrary Int arithmetic
negative division/modulo
strings/parsing
errors
Native methods
LANG006 @primitive/@intrinsic metadata semantics
reflection
GC
concurrency integration
```

`Foreign` is part of the general LANG006 implementation-kind model but is not expected to be required by ordinary C2 time behavior.

When time exposes a defect in one of these general features:

```text
prefer fixing the owning language implementation
over hiding the problem in Rust, metadata, or Universe privilege
```

If the required language fix collides with active architecture in another checkpoint, consult rather than patching around it.

# 22. Ordinary-package certification

Universe compilation is not sufficient evidence.

C2 must include ordinary-package fixtures that:

```text
import time through normal package syntax
consume grouped imports
observe prelude Duration/Timestamp
construct/use civil values
use exported clock
use timezone/zoned values
pattern-match LocalTimeResolution
exercise reflection where applicable
```

Public semantics must not depend on accidental all-Universe visibility.

Where practical, pure calendar/source algorithms may additionally be exercised under ordinary package context or equivalent differential compilation.

Do not physically fork duplicate source copies unless the test infrastructure cannot reuse the same source semantics safely.

---

# 23. Testing doctrine for C2

Design broad correctness coverage; execute narrowly.

## BUILD

Use:

```text
exact parser/semantic regression
exact runtime behavior regression
small owning module tests
```

Do not run:

```text
full workspace
full clippy
all language corpus
all semantic tests
```

after routine edits.

## STABILIZE

At coherent P1/P2 gates run:

```text
time semantic capability tests
time language/runtime corpus
directly affected Universe/core-surface contracts
affected concurrency tests
affected reflection/provenance tests
timezone-specific focused suite in P2
```

## CERTIFY

Only at the explicit final plan/checkpoint gate, broaden to the exact crate/workspace commands required by P2.

No unchanged failing test should be rerun unless relevant state changed.

---

# 24. Coverage dimensions to plan explicitly

Each plan maps tests to checkpoint invariants and selects the dimensions it actually owns.

P1 includes at minimum:

```text
singleton/export identity
Clock domain mismatch
Manual/custom Clock subclass behavior
Duration data equality/hash
Timestamp pre/post epoch
-1ns boundary
leap-year rules
month length
Jan-31 month advance Constrain/Reject
negative years if supported
invalid Date/Time/Offset construction
Period normalization/order absence
grouped import/export
prelude identity
visibility negatives
Native + @primitive host-observation provenance
ordinary package consumption
concurrency Duration compatibility
reflection of migrated declarations
```

P2 includes at minimum:

```text
UTC/fixed/named zones
system timezone
named-zone identity/cache
stable timezone database snapshot/lifetime
historical offset with seconds if backend data provides one
transitionBefore/After
known DST gap
known DST overlap
non-one-hour transition fixture
Unique/Ambiguous/Gap payload semantics
disambiguation policies
same instant/different zones
shift(Duration) vs advance(Period)
Date.today zone dependence
parse/format round trips
invalid parse
offset-zone mismatch
backend interop range failures without P1 narrowing
lazy database initialization
zone handle lifetime/GC
ordinary package consumption
Native/@primitive provenance
final reflection surface
time.testing clocks
resource/cache sanity
```

Property testing is particularly valuable for civil epoch-day round trips and parser/formatter round trips.

Deterministic timezone tests must use controlled/pinned fixtures where rule drift matters; ambient host tzdata is reserved for capability/integration smoke tests.

# 25. Failure classification

Unexpected failures:

```text
A — definitely caused by C2
B — probably caused by C2
C — unclear after one bounded classification pass
D — unrelated/baseline
```

A/B are active responsibility.

C gets one bounded classification pass; if nonblocking and still unclear, record it.

D is recorded and deferred immediately.

Do not weaken temporal invariants or expected semantics to make broad gates green.

---

# 26. Debugging budget

Mechanical failures:

```text
up to roughly three coherent inspect -> cause -> correction -> focused rerun cycles
while evidence shows real progress
```

Semantic failures require:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure.

If it persists, STOP AND CONSULT.

Architectural failure gets zero speculative fixes.

---

# 27. C2-specific STOP / CONSULT triggers

Stop immediately if any of these appears:

1. visibility requires spelling-based exceptions;
2. protected construction cannot express the Clock/Instant capability;
3. package/module/internal visibility is being inferred from path/string rather than canonical identity;
4. `@primitive` or `@intrinsic` must become operative to complete a task;
5. live LANG006 `Phalcom` / `Native` / `Foreign` semantics conflict with this checkpoint;
6. a legacy spelling-based, primitive-registry, or special-attribute compatibility shim starts becoming permanent architecture;
7. abstract-member obligations require a competing class-completeness model;
8. `data` validation would require abandoning transparency or introducing a parallel constructor model;
9. Timestamp needs floating-point canonical storage;
10. DateTime would need an implicit timezone to make an API work;
11. Period would need exact-duration semantics;
12. UtcOffset is being used as a substitute for named timezone rules;
13. backend local resolution cannot preserve unique/ambiguous/gap factual mappings cleanly;
14. timezone handles create uncertain GC/root/lifetime behavior;
15. no coherent stable VM/process timezone-database snapshot/generation can be provided;
16. backend ranges would force narrowing P1 temporal semantics;
17. historical second-resolution offsets would be lost;
18. named TimeZone identity would require raw pointer equality or repeated full-rule comparison;
19. ZonedDateTime needs redundant public state for correctness;
20. a new temporal VM value/bytecode seems necessary;
21. a source-level time bug can only be bypassed by moving large semantics to Rust;
22. C2 conflicts materially with an active language checkpoint;
23. expected semantic test behavior would have to change to accommodate an implementation defect;
24. deterministic timezone semantics can only be tested against mutable ambient host tzdata;
25. canonical text cannot round-trip all valid values required by P2;
26. checkpoint scope expands into timers/reactor redesign, locale/CLDR, recurrence, general FFI redesign, or whole-Universe migration;
27. an older `time.md` would need to be revived merely to satisfy superseded planning text.

A triggered condition cannot be self-waived.

# 28. Consultation packet for C2

Before escalating, collect only focused evidence:

```text
C2 plan/task/gate
relevant INV-* identifiers
desired semantic behavior
live language/repository fact that conflicts
minimal Phalcom reproducer
relevant declarations and ownership
scoped diff
tests run and exact output
implementer causal hypothesis
one serious attempt, if semantic
specific architectural decision requested
```

Do not ask the adviser to rediscover the entire repository.

---

# 29. Checkpoint bookkeeping

At P1/P2 start:

```text
update C2 checkpoint active plan
record execution revision
consume predecessor handoff where applicable
record the standing C2 amendments if not already present:
    LANG006 implementation-kind reconciliation
    time specification USER-DEFERRED
```

During a plan update the checkpoint only for durable events:

```text
stable new interface/invariant
backend/version/snapshot decision
TimeZone identity decision
text grammar decision
consultation/amendment
meaningful deferred failure
coherent verification gate
```

At plan completion record:

```text
status/completion/verification
established interfaces
tests actually run
tests deferred
baseline failures
consultations/amendments
next action
```

Create:

```text
STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md

STDL002.C2.P2-walkthrough.md
STDL002.C2.P2-handoff.md
```

as applicable.

At final P2 closure, explicitly record:

```text
C2 completion revision
backend/version/features
timezone database snapshot/lifetime model
TimeZone identity rule
backend interop ranges
canonical text grammar
ordinary-package/reflection/provenance evidence
GC/resource evidence
specification = USER-DEFERRED
```

Do not create a competing plan-specific checkpoint-state document.

# 30. P1 execution instructions

The P1 plan set already exists. Do not create another P1 plan.

Execute from:

```text
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md
```

At takeover, verify:

```text
current grouped import/export syntax
current visibility syntax/semantics
current Phalcom/Native/Foreign registration
current @intrinsic/@primitive metadata handling
current abstract-member status
current data constructor/validation model
current static/class-side singleton field syntax
current time consumers
current C1 tests/floor/reflection contracts
```

Interpret older P1-header language through the standing C2 amendments:

```text
primitive implementation kind
    -> Native + @primitive metadata for irreducible host facts

existing time.md authority/spec requirement
    -> excluded / USER-DEFERRED
```

P1 T1 remains conditional: it may close only the minimum general-language prerequisite that is missing and already architecturally ratified. It must not implement the wider LANG006 transition itself.

P1 must finish with its walkthrough/handoff before P2 production execution proceeds.

# 31. P2 execution instructions

The P2 patch-grade plan already exists:

```text
STDL002.C2.P2-plan.md
```

It was authored before P1 implementation only by explicit user instruction. This does not waive its hard execution dependency.

Before P2 production work:

```text
read completed P1 walkthrough/handoff
reconcile exact P1 signatures, ranges, paths, visibility, reflection categories,
and Native/@primitive linkage into P2 T0
verify exact timezone backend version/features fresh
verify stable database snapshot strategy
verify alias/identity semantics
verify backend interop ranges
verify controlled/pinned timezone fixture strategy
```

The intended dependency shape is:

```text
T0  P1 handoff/LANG006/backend re-ground
 ↓
T1  backend dependency + snapshot/lifetime proof
 ↓
T2  Native TimeZoneDatabase adapter/cache
 ↓
T3  TimeZone + transitions
 ↓
T4  LocalTimeResolution/Disambiguation
 ↓
T5  ZonedDateTime semantics
 ↓
T6  current-time conveniences
 ↓
T7  canonical formatting
 ↓
T8  parsing/consistency/round trips
 ↓
T9  time.testing clocks
 ↓
T10 ordinary-package/reflection/provenance/GC/performance closure
 ↓
T11 checkpoint/walkthrough/handoff certification
```

P2 must not depend on ambient host tzdata for all behavioral assertions.

Authoritative time specification work remains `USER-DEFERRED`; do not add it back to T11 unless explicitly re-enabled.

# 32. Things not to redesign during implementation

Unless a STOP/CONSULT trigger fires, do not reopen:

```text
C2 is two plans
time is the public temporal namespace
System has no date/time discovery façade
Duration/Timestamp are the temporal prelude names
SystemClock and SystemWallClock are public/exported
clock/wallClock are package-exported canonical instances
Clock domain identity is Clock object identity
Timestamp uses signed Int nanoseconds
civil calendar is proleptic Gregorian
Duration != Period
UtcOffset != TimeZone
DateTime != ZonedDateTime
ZonedDateTime canonical state = Timestamp + TimeZone
timezone backend/rule operations are Native, not primitive-floor metadata by default
host observations are Native + @primitive metadata
ImplementationKind is Phalcom / Native / Foreign
@intrinsic/@primitive metadata is non-operative
visibility is identity-based and lexical
no special implementation-selector naming in final C2 time source
no broad temporal trait hierarchy
no temporal VM specialization by default
stable timezone database snapshot/lifetime is required
backend finite ranges do not narrow P1 temporal semantics
canonical parser/formatter semantics are Phalcom-owned
existing stale time.md files are excluded
specification work is USER-DEFERRED
```

These are architect decisions, not implementer convenience choices.

# 33. Current next action

Execute `STDL002.C2.P1` from the existing plan set.

Do not create another requirements analysis or another P1 plan unless a material architectural consultation changes the checkpoint.

The immediate implementation sequence is:

```text
1. P1 T0 live takeover
2. verify required LANG006/general-language prerequisites
3. execute P1 through final focused acceptance
4. produce P1 walkthrough + handoff
5. begin P2 T0 and consume the as-built P1 contract
6. execute P2 through final C2 certification
7. update checkpoint to truthful COMPLETE state only when its final gate passes
```

The P2 plan is already authored, but production P2 work remains dependent on the completed P1 walkthrough/handoff.

Do not revive stale `time.md` work during this sequence. Authoritative time specification remains `USER-DEFERRED`.
