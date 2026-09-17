---
id: STDL002.C2
category: STDL
program: STDL002
kind: implementation-checkpoint
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - STDL002.C1
follows: STDL002.C1
supersedes: null
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_baseline_revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
---

# STDL002.C2 — `time` Upgrade: Absolute, Civil, Zoned, and Textual Time

## 0. Checkpoint role

`STDL002.C2` upgrades the already-shipped monotonic `time` foundation into Phalcom's complete foundational temporal library.

`STDL002.C1` remains historically true and complete: it delivered the first monotonic `Clock`, `Instant`, and `Duration` surface and verified that slice. C2 does not rewrite C1's completion record. C2 is a successor checkpoint that intentionally modernizes parts of the C1 public and implementation surface while adding the remaining temporal domains.

C2 is implemented through exactly two numbered plans:

| Plan | Scope | Initial state |
|---|---|---|
| `STDL002.C2.P1` | modern temporal foundation: C1 migration, monotonic capabilities, absolute wall time, civil Gregorian values, fixed offsets, package/prelude exposure, prerequisite language closure needed by those surfaces | NOT_STARTED |
| `STDL002.C2.P2` | IANA timezone backend, zoned semantics, system-local conveniences, parsing/formatting, testing clocks, ordinary-package certification, reflection/integration closure, final specification | NOT_STARTED |

C2 is complete only when both plans are complete and the checkpoint acceptance objective in §2 is satisfied.

This checkpoint record is the durable lifecycle authority for C2. It owns plan state, established architecture, accepted amendments, verification evidence, deferred failures, consultations, and the current next action. Plan-local task mechanics belong in P1/P2; persistent implementation rules belong in `STDL002.C2-GUIDANCE.md`.

---

## 1. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
revision message: feat(lang005): continue P5 iterator surface
predecessor: STDL002.C1
```

Verified predecessor facts at this baseline:

1. `STDL002.C1` is recorded `COMPLETE / COMPLETE / VERIFIED`.
2. C1 owns only the monotonic slice and explicitly deferred civil date/time and timezone work.
3. The current Universe `time` package contains `clock.ph` and `package.ph`.
4. The current C1 implementation represents `Duration`, `Instant`, and `Clock` as ordinary Phalcom classes.
5. The current C1 implementation predates the newly ratified lexical visibility and `native`/`primitive` implementation-kind model and therefore contains legacy implementation-access/linkage conventions that C2 must migrate.
6. The current `time/package.ph` re-exports `Clock`, `Instant`, and `Duration`.
7. The Universe root currently promotes `Clock`, `Instant`, and `Duration`.
8. The current normative `docs/spec/current/stdlib/time.md` describes the monotonic-only C1 surface.
9. The wider repository is actively changing visibility, declaration implementation kinds, and related language surface. C2 must verify those live mechanics before editing and must not reinstate superseded conventions.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 2. Checkpoint acceptance objective

C2 is accepted when the Universe `time` package provides one coherent temporal model covering:

```text
exact elapsed time
monotonic clock domains
absolute wall-clock timestamps
proleptic Gregorian civil date/time
calendar-relative periods
fixed UTC offsets
IANA/rule-based time zones
civil-local ambiguity and gaps
zoned date-times
canonical machine parsing/formatting
deterministic clock test support
```

while preserving all of the following architectural conditions:

```text
ordinary Phalcom source owns ordinary temporal semantics

primitive methods expose only genuinely irreducible host facts

native methods may implement substantial Rust-backed library behavior
without becoming part of the irreducible primitive floor

Universe membership grants no spelling-based access-control bypass

@intrinsic and @primitive are metadata only

System is not the public date/time namespace

the time package exports the canonical clock capabilities directly

Duration and Timestamp are the only temporal prelude promotions

timezone/database cost is lazy; importing time does not eagerly load tzdb

no temporal Value variants, temporal bytecodes, or VM-special temporal dispatch
```

The public API must be consumable by an ordinary package through normal import, name-resolution, typechecking, lowering, bytecode, dispatch, allocation, GC, and reflection paths.

---

## 3. Ratified language surface consumed by C2

### 3.1 Grouped imports and exports

Multiple imported/exported names use tuple-shaped grouping:

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

Exports use the corresponding grouped shape.

C2 final Universe source should use the current canonical grouped form when exporting/importing multiple bindings. Do not preserve an older comma-list syntax merely because C1 used it.

### 3.2 Visibility is identity-based and lexical

The intended access-control relations are:

| Visibility | Accessibility |
|---|---|
| default / `Public` | unrestricted after normal name/import resolution |
| `private` | declaring lexical class |
| `protected` | declaring class and eligible subclasses |
| `module` | same exact defining `Module` |
| `package` | same exact defining `Package` |
| `internal` | same internal `Project` / domain |

These relations are based on canonical declaration/module/package/project identity, not spelling, prefixes, directory names, or selector conventions.

`protected` is inheritance-oriented and is not to be treated as a point in a simple linear visibility lattice.

### 3.3 No implementation-name visibility convention

C2 must not introduce or retain time-library interfaces whose intended encapsulation depends on:

```text
special implementation selector spelling
special implementation field spelling
path- or prefix-based access conventions
visibility metadata used as an implementation-kind marker
```

Use ordinary names plus the correct lexical visibility.

Legacy C1 spellings may exist at takeover and are migration inputs, not target architecture.

### 3.4 `@intrinsic` and `@primitive`

Phalcom has built-in:

```text
@intrinsic
@primitive
```

These attributes carry information only.

Their presence does **not**, by itself:

- change visibility;
- change selector identity;
- change field identity;
- make a declaration `native`;
- make a declaration `foreign`;
- allocate a different runtime representation;
- install a primitive function;
- emit a dedicated bytecode;
- force intrinsic lowering;
- alter typechecking;
- grant access-control privilege.

Authored use is legal only in the canonical Universe Project; authored use outside Universe is a compile-time error.

C2 must not use either attribute as a substitute for a declaration implementation kind or for access control.

### 3.5 `native` and `primitive` implementation kinds

C2 relies on the active language distinction:

```text
native
    method implementation is supplied in Rust using Phalcom's native APIs;
    the behavior is not necessarily irreducible and could conceptually have
    a Phalcom implementation.

primitive
    implementation is irreducible at the language level because it supplies
    a host/runtime fact or capability unavailable from existing Phalcom values.
```

Both may ultimately execute Rust, but they express different architectural status.

C2 must keep the primitive floor minimal. Performance alone is never sufficient reason to mark an operation primitive.

The `@primitive` metadata attribute and the `primitive` implementation kind are distinct concepts and must not be conflated.

---

## 4. Public temporal model

The target semantic categories are:

| Concept | Meaning | Target representation |
|---|---|---|
| `Duration` | exact signed elapsed amount in nanoseconds | `data` |
| `Measurement<T>` | measured value + exact duration | generic `data` |
| `Clock` | abstract monotonic clock domain | abstract `class` |
| `SystemClock` | host monotonic-clock implementation | `class` |
| `clock` | exported canonical monotonic clock value | package value |
| `Instant` | coordinate in one exact `Clock` domain | opaque `class` |
| `Timestamp` | absolute POSIX/Unix-compatible timeline point | `data` |
| `WallClock` | abstract timestamp source | abstract `class` |
| `SystemWallClock` | host wall-clock implementation | `class` |
| `wallClock` | exported canonical wall-clock value | package value |
| `Month` | Gregorian month | `enum` |
| `Weekday` | ISO weekday | `enum` |
| `Date` | Gregorian date | validated `data` |
| `Time` | time-of-day | validated `data` |
| `DateTime` | timezone-free civil coordinate | `data` |
| `YearMonth` | year/month partial civil value | `data` |
| `MonthDay` | month/day partial civil value | validated `data` |
| `Period` | calendar-relative months + days | `data` |
| `DateOverflow` | calendar overflow policy | `enum` |
| `UtcOffset` | fixed displacement from UTC | validated `data` |
| `TimeZone` | fixed or rule-based timezone | opaque `class` |
| `ZoneTransition` | immutable transition description | `data` |
| `LocalTimeResolution` | unique/ambiguous/gap result | payload `enum` |
| `LocalTimeDisambiguation` | resolution policy | `enum` |
| `ZonedDateTime` | `Timestamp` interpreted through `TimeZone` | `data` |

The durable semantic non-equivalences are:

```text
Instant        != Timestamp
Duration       != Period
DateTime       != Timestamp
UtcOffset      != TimeZone
DateTime       != ZonedDateTime
24 exact hours != one calendar day
30 exact days  != one calendar month
same instant   != same ZonedDateTime structural value
```

C2 must not erase these distinctions through a generic "temporal amount" or "temporal point" abstraction.

---

## 5. Public package philosophy

Temporal discovery belongs to `time`, temporal classes, and exported capability values—not to `System`.

Target everyday forms:

```phalcom
from time import clock

const start = clock.now
```

```phalcom
const createdAt = Timestamp.now
```

```phalcom
from time import Date, TimeZone

const today = Date.today
const tokyoToday = Date.today(
  in: TimeZone.named("Asia/Tokyo")
)
```

Do not add or preserve public date/time discovery aliases such as:

```text
System.clock
System.now
System.wallClock
System.timeZone
```

`System.sleep(Duration)` remains valid because sleeping is a scheduler/concurrency action that consumes a temporal value; it is not temporal discovery.

---

## 6. Package exports and prelude

### 6.1 `time` root exports

The final `time` package should export the complete supported temporal model, including the concrete system clock classes:

```text
Duration
Measurement

Clock
SystemClock
Instant
clock

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
TimeZone
ZoneTransition
LocalTimeResolution
LocalTimeDisambiguation
ZonedDateTime
```

`SystemClock` and `SystemWallClock` are not hidden merely because users usually consume the exported `clock` and `wallClock` values.

The package may contain non-exported backend declarations for timezone database integration.

### 6.2 Prelude promotion

Only:

```text
Duration
Timestamp
```

are promoted into the ordinary Universe/prelude root.

Do not promote:

```text
Clock
SystemClock
Instant
clock
WallClock
SystemWallClock
wallClock
Date
Time
DateTime
Period
TimeZone
ZonedDateTime
```

Users intentionally import those from `time`.

### 6.3 Stable singleton model

`SystemClock` owns its singleton:

```text
SystemClock.instance
```

and the package exports:

```text
clock = SystemClock.instance
```

The same model applies to `SystemWallClock` / `wallClock`.

The package binding and the class-side singleton must reference the same object. Clock object identity is monotonic-domain identity.

No laziness guarantee is required for `clock`/`wallClock`: constructing the capability object is cheap and must not read time or initialize timezone resources. Tzdb initialization remains lazy.

---

## 7. Visibility policy for the `time` implementation

Choose the narrowest semantic visibility based on who actually consumes a declaration:

```text
private
    helpers used only inside one lexical class

protected
    subclass capability hooks such as Clock's Instant-construction seam

module
    helpers shared by declarations in exactly one module

package
    helpers shared across modules of the exact time Package

internal
    only when a declaration genuinely must be shared across the internal
    Universe Project/domain beyond the time Package
```

Do not use `internal` simply to mean "stdlib implementation detail."

Do not rely on filenames, prefixes, selector spelling, or Universe source paths to simulate access control.

The timezone backend can normally remain non-exported and package-visible/module-visible without becoming `internal`.

---

## 8. Primitive/native boundary

### 8.1 Primitive candidates

The expected irreducible C2 primitive surface is narrowly limited to host facts such as:

```text
host monotonic-clock observation
host wall-clock observation
host/system timezone configuration discovery
```

P1 owns the first two.

P2 owns system timezone discovery.

Exact declaration names and Rust linkage APIs are verify-first because the implementation-kind transition is active.

### 8.2 Native timezone implementation

IANA/TZif database behavior is expected to be a `native` backend rather than a `primitive` floor:

```text
named-zone open/lookup
offset-at-timestamp
civil-local resolution
transition-before
transition-after
possibly canonical-name/database adapter operations
```

These operations are substantial and external-data-backed but are conceptually implementable in Phalcom; using Rust/Jiff is an implementation choice, not an irreducibility claim.

### 8.3 Forbidden boundary inflation

Do not move the following into Rust merely because they are easier or faster there:

```text
Duration arithmetic
Timestamp arithmetic
Gregorian leap-year/day calculations
epoch-day conversion
Date/DateTime/Period arithmetic
fixed-offset conversion
disambiguation policy
ZonedDateTime shift/advance semantics
ISO/RFC grammar
canonical formatting
value equality/hash/comparison
```

If source-level implementation exposes a language defect, fix or escalate the language defect instead of expanding native code reflexively.

---

## 9. Module architecture

The intended package decomposition is:

```text
phalcom-core/core/universe/src/time/
│
├── package.ph
├── duration.ph
├── clock.ph
├── timestamp.ph
│
├── calendar/
│   ├── package.ph
│   ├── month.ph
│   ├── period.ph
│   ├── date.ph
│   ├── time.ph
│   ├── datetime.ph
│   └── partial.ph
│
├── zone/
│   ├── package.ph
│   ├── offset.ph
│   ├── backend.ph
│   ├── timezone.ph
│   ├── resolution.ph
│   └── zoned.ph
│
├── text/
│   ├── package.ph
│   ├── iso8601.ph
│   └── parsing.ph
│
└── testing/
    ├── package.ph
    └── clocks.ph
```

Mechanical decomposition may adapt to the live tree, but ownership/dependency direction must remain equivalent.

Target dependency direction:

```text
Duration
   ↓
Clock / Instant
   ↓
Timestamp / WallClock
   ↓
civil Gregorian core
   ↓
UtcOffset
   ↓
TimeZone backend + TimeZone
   ↓
local resolution + ZonedDateTime
   ↓
current-time conveniences
   ↓
parsing / formatting / completion
```

Avoid source cycles by placing timezone-dependent conveniences in higher-level modules or inherent implementations rather than making core civil modules depend downward on zone implementation.

---

## 10. Plan decomposition

# `STDL002.C2.P1` — Temporal Foundation, Absolute Time, and Civil Calendar

P1 owns the coherent source/runtime foundation required before timezone rules can be layered on top.

Required scope:

```text
consume/verify the new visibility and implementation-kind language surface

migrate C1 legacy time implementation conventions
    remove time-owned legacy implementation-selector naming
    remove legacy visibility-as-implementation-marker usage
    remove System.clock
    move the legacy System-owned monotonic host read onto SystemClock

Duration -> data
Measurement<T>

Clock -> abstract class
SystemClock -> concrete exported class
SystemClock.instance
time::clock
Clock-domain Instant integrity
protected Clock construction seam

Timestamp -> data
WallClock -> abstract class
SystemWallClock -> concrete exported class
SystemWallClock.instance
time::wallClock
Timestamp.now
host wall-clock primitive

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

Gregorian epoch-day algorithms
calendar validation/arithmetic
negative/pre-epoch arithmetic correctness

time package root restructuring
grouped imports/exports
prelude migration to Duration + Timestamp only

concurrency compatibility
    System.sleep(Duration)
    Future.timeout(Duration)
    existing reactor semantics
```

P1 also owns the **minimum prerequisite language closure** strictly necessary for this surface if it has not already landed, including:

```text
usable bodyless abstract member obligations for Clock.now / WallClock.now
validated data construction/invariants needed by Date/Time/UtcOffset/MonthDay
```

However, P1 must not independently redesign active language-transition work. If satisfying these prerequisites would conflict with or duplicate an in-flight language architecture, STOP AND CONSULT.

P1 does **not** own IANA time zones, zoned date-times, system-local timezone semantics, full parsing/formatting, or final C2 certification.

# `STDL002.C2.P2` — Time Zones, Zoned Semantics, Textual Forms, and Closure

P2 depends on the complete P1 foundation.

Required scope:

```text
timezone database adapter
preferred mature Rust backend behind Phalcom-owned API
lazy database initialization
stable VM/process database snapshot

TimeZone
    utc
    fixed
    named
    system

ZoneTransition
LocalTimeResolution
LocalTimeDisambiguation
ZonedDateTime

Timestamp.in(zone)
TimeZone.at(timestamp)
DateTime.resolve(zone)
DateTime.in(zone, disambiguation)
ZonedDateTime.shift(Duration)
ZonedDateTime.advance(Period)
ZonedDateTime.withTimeZone(zone)
sameInstant / compareInstant

Date.today
Date.today(in: zone)
ZonedDateTime.now
ZonedDateTime.now(in: zone)

canonical machine parsing/formatting
ISO/RFC temporal forms
offset/zone consistency checks
round-trip properties

time.testing clocks
ordinary-package consumption certification
reflection and type-observation certification
native/primitive inventory audit
performance/allocation sanity
comprehensive stdlib time specification
checkpoint closure
```

P2 does not reopen P1's type/category decisions unless live evidence proves them unsound.

---

## 11. Durable global invariants

### `INV-01` — C1 history remains truthful

C1 stays a completed monotonic predecessor. C2 records migrations as C2 work rather than rewriting C1's historical state.

### `INV-02` — temporal categories remain distinct

`Instant`, `Timestamp`, `DateTime`, `ZonedDateTime`, `Duration`, `Period`, `UtcOffset`, and `TimeZone` do not collapse into interchangeable wrappers.

### `INV-03` — monotonic domain identity is object identity

Each distinct `Clock` object defines a distinct `Instant` domain. Cross-domain equality is false; cross-domain difference/order rejects.

### `INV-04` — one canonical system monotonic domain

`SystemClock.instance` and exported `clock` are the same object for the relevant VM/package lifetime.

### `INV-05` — one canonical system wall-clock capability

`SystemWallClock.instance` and exported `wallClock` are the same capability object. Timestamp values themselves have global value semantics and do not inherit wall-clock identity.

### `INV-06` — ordinary source semantics dominate

All derivable calendar/arithmetic/policy/text semantics remain Phalcom source unless a later measured and approved optimization changes implementation without changing semantics.

### `INV-07` — primitive means irreducible

No operation is classified `primitive` merely for performance, convenience, or because Rust already has a library implementation.

### `INV-08` — native does not mean primitive

Timezone database operations may be `native` while remaining outside the irreducible floor.

### `INV-09` — metadata attributes are non-operative by themselves

`@intrinsic` and `@primitive` never grant visibility, install implementations, change identity, change representation, or force lowering.

### `INV-10` — access control uses canonical lexical identity

No time implementation depends on selector spelling or source-path naming for visibility.

### `INV-11` — no legacy implementation-selector convention in final C2 time surface

No C2-owned target API uses special selector spelling as an access-control or implementation mechanism.

### `INV-12` — prelude stays small

Only `Duration` and `Timestamp` are temporally promoted; all other temporal declarations are accessed through `time`.

### `INV-13` — importing time is cheap

Importing `time` may allocate canonical lightweight clock capability objects but must not eagerly load/parse the timezone database or probe expensive timezone resources.

### `INV-14` — ZonedDateTime canonical state is non-redundant

Its semantic state is `Timestamp + TimeZone`; offset/civil projections are derived implementation results, not independent equality state.

### `INV-15` — exact and calendar movement remain distinct

`Duration` drives exact timeline movement; `Period` drives civil calendar movement.

### `INV-16` — ordinary-package equivalence

Public `time` APIs compile and run through normal package semantics. Universe privilege is limited to explicitly authorized Universe-only declarations and host/native linkage.

### `INV-17` — no temporal VM specialization by default

No new temporal `Value` arm, object kind, temporal bytecode, or standard-library dispatch special case is introduced without a separately approved measured justification.

---

## 12. Calendar and timeline rules

C2 fixes these semantics unless amended explicitly:

```text
Timestamp:
    signed Int nanoseconds from Unix epoch
    POSIX/Unix-compatible non-leap timeline

civil calendar:
    proleptic Gregorian

Time.second:
    0..59
    no 23:59:60 value

Period canonical storage:
    months + days
    years normalize to months
    weeks normalize to days

Date.advance(period):
    months first
    overflow policy
    days second

DateOverflow:
    Constrain
    Reject

ZonedDateTime.shift(Duration):
    exact timestamp movement

ZonedDateTime.advance(Period):
    civil movement then timezone resolution

DateTime:
    timezone-free civil coordinate
    no general exact Duration arithmetic

ZonedDateTime equality:
    Timestamp + TimeZone structural equality

sameInstant:
    explicit timestamp-only equivalence
```

Precise supported civil year range may be finalized by P1 after verifying backend/interoperability constraints. Choosing a materially restrictive range without evidence is a STOP/CONSULT condition.

---

## 13. Local-time resolution rules

P2 must expose local civil ambiguity explicitly:

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

and policy:

```phalcom
enum LocalTimeDisambiguation {
  Reject
  Earlier
  Later
  Compatible
}
```

Expected policy:

| Case | Reject | Earlier | Later | Compatible |
|---|---|---|---|---|
| unique | value | value | value | value |
| overlap | error | earlier | later | earlier |
| gap | error | before | after | after |

The exact `Gap(before, after)` boundary construction must be specified and tested against the selected timezone backend rather than inferred casually during implementation.

---

## 14. Text scope

P2 includes stable machine forms and round-trip parsing for the core temporal types.

It does not include:

```text
locale-sensitive formatting
CLDR
localized month/weekday names
natural-language parsing
relative human text
custom strftime-style formatting language
non-Gregorian calendars
```

Parsing grammar is owned by Phalcom even if native timezone data is consulted for zoned validation. Do not delegate the public accepted grammar wholesale to the Rust backend.

---

## 15. Traits and abstraction policy

C2 introduces no broad temporal trait family.

Do not create:

```text
Temporal
TemporalAmount
TemporalPoint
DateLike
TimeLike
ZoneLike
ClockLike
```

Use:

```text
data
    transparent immutable semantic values

enum
    closed alternatives/policies/results

abstract class
    nominal capability families with inherited behavior

class
    opaque/resource/rule-backed abstractions
```

General-purpose existing equality/hash/order/string/reflection contracts may be used where canonical.

---

## 16. Non-goals

C2 does not own:

```text
cron
RRULE / iCalendar recurrence
business calendars
holiday calendars
market calendars
appointments
alarms/reminders
job schedulers
natural-language time
localized presentation
non-Gregorian calendars
TAI/GPS/NTP synchronization
astronomical calendars
leap-second tables
timer queue/reactor redesign
thread model redesign
a generic FFI redesign
whole-repository visibility migration
whole-repository native/primitive migration
```

C2 consumes the active language surface; it does not become the project that migrates every other Universe package.

---

## 17. Expected source areas

Expected C2 ownership includes the live equivalents of:

```text
phalcom-core/core/universe/src/time/**
phalcom-core/core/universe/src/package.ph
docs/spec/current/stdlib/time.md
phalcom-semantic/tests/semantic/capabilities/time.rs
phalcom-core time/language-corpus tests
native/primitive registry/descriptor paths needed by time
time-related reflection/core-surface contracts
concurrency call sites that consume the migrated Duration/clock API
```

P1 may need narrow parser/AST/semantic/compiler/runtime changes for missing prerequisites such as abstract-member obligations or validated `data` construction.

P2 may add the Rust timezone adapter/dependency and VM-owned timezone database state.

Touching unrelated standard-library domains is not expected.

---

## 18. Testing surface owned by the checkpoint

C2 plans must design coverage across the applicable dimensions:

```text
canonical construction and use
invalid component rejection
boundary values
negative/pre-epoch arithmetic
leap years and month edges
clock-domain identity
singleton/export identity
package/prelude visibility
module/package/private/protected access boundaries
ordinary-package consumption
reflection/type observation
native/primitive provenance
timezone transition gaps/overlaps
named/fixed zone non-equivalence
exact-vs-calendar arithmetic
parse/format round trip
timezone offset/identifier consistency
lazy timezone initialization
GC/lifetime of native zone handles
concurrency integration with Duration
baseline-failure classification
```

Tests should prove semantics at the owning layer rather than mirror private implementation details.

Execution breadth remains narrow during BUILD and broadens only at named plan/checkpoint gates.

---

## 19. Global STOP / CONSULT triggers

In addition to repository-wide workflow triggers, C2 requires immediate consultation if:

1. the new visibility model cannot express the required `Clock`/`Instant` construction relationship without a spelling convention or privilege bypass;
2. `@intrinsic` or `@primitive` would need operative semantics to make C2 work;
3. the live `native`/`primitive` implementation-kind architecture materially differs from the distinction in §3.5;
4. C2 would need to recreate a legacy spelling-based or special-attribute implementation-access mechanism to bridge an incomplete language transition;
5. abstract-member obligations cannot support `Clock.now` / `WallClock.now` without a broader redesign;
6. validated `data` values cannot enforce `Date`, `Time`, `UtcOffset`, or `MonthDay` invariants without changing the accepted `data` model;
7. `SystemClock.instance` / exported `clock` cannot preserve one stable domain identity through normal package initialization;
8. timezone state would require per-object Rust ownership/GC semantics rather than VM-lifetime handles/canonical backend state;
9. the timezone backend forces public semantics to mirror backend quirks or accepted grammar;
10. ZonedDateTime requires redundant public canonical fields for performance;
11. a public C2 operation appears to require a temporal VM bytecode or `Value` specialization for correctness rather than optimization;
12. fixing a time-library language defect would materially conflict with an active LANG checkpoint;
13. an accepted invariant in this file must be weakened to satisfy existing tests.

Triggered consultation is not self-waivable.

---

## 20. Completion and verification truth

C2 lifecycle states must remain truthful.

```text
source written
!= tests added
!= focused tests passed
!= plan complete
!= checkpoint accepted
!= workspace/release certified
```

A plan may complete `FOCUSED_TESTED` while unrelated baseline failures remain recorded.

C2 may be marked `COMPLETE` only after P1 and P2 are complete and the final checkpoint gate has established:

```text
full public temporal surface
ordinary package integration
prelude/export correctness
semantic/runtime/reflection behavior
native/primitive boundary inventory
timezone transition correctness
parse/format round trips
required focused owning suites
final authoritative time specification
```

Workspace-wide release certification is separate unless explicitly included by the final P2 plan or requested by the user.

---

## 21. Plans ledger

| Plan | Objective | Status | Completion | Verification |
|---|---|---|---|---|
| `STDL002.C2.P1` | modern temporal foundation, absolute time, civil calendar, fixed offset, package/prelude migration | PROPOSED | NOT_STARTED | UNVERIFIED |
| `STDL002.C2.P2` | timezone/zoned/textual layers and C2 integration closure | PROPOSED | NOT_STARTED | UNVERIFIED |

No P1/P2 plan file exists yet at checkpoint creation time.

---

## 22. Deferred/baseline failures

None recorded at checkpoint creation.

When implementation finds a failure, classify it:

```text
A — definitely caused by active C2 patch
B — probably caused by active C2 patch
C — unclear after one bounded classification pass
D — clearly unrelated/baseline
```

Record meaningful C/D failures here rather than consuming C2 scope repairing unrelated work.

---

## 23. Consultations and amendments

None at checkpoint creation.

Any material change to:

```text
temporal semantic categories
primitive/native distinction
visibility/access model
prelude policy
clock identity/lifetime
timezone ownership/lifetime
two-plan decomposition
```

must be recorded here after approval.

---

## 24. Current next action

Prepare `STDL002.C2.P1` as a Luna-oriented patch-grade implementation plan.

The P1 planning pass must:

1. re-read this checkpoint and `STDL002.C2-GUIDANCE.md`;
2. verify the live status of grouped import/export syntax, lexical visibility, abstract members, validated `data` construction, `native`/`primitive` keywords, and `@intrinsic`/`@primitive`;
3. inspect the exact live C1 time implementation and all current consumers;
4. turn P1 scope from §10 into concrete ordered tasks and focused gates;
5. avoid planning P2 timezone/zoned/text work beyond interfaces P1 must establish for it.
