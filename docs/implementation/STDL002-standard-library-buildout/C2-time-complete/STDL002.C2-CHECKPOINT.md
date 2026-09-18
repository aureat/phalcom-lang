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
updated: 2026-09-18
repository: aureat/phalcom-lang
planning_baseline_revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
latest_planning_baseline_observed: 206e0ab91129e189782c4028de5a40379bcc033c
---

# STDL002.C2 — `time` Upgrade: Absolute, Civil, Zoned, and Textual Time

## 0. Checkpoint role

`STDL002.C2` upgrades the already-shipped monotonic `time` foundation into Phalcom's complete foundational temporal library.

`STDL002.C1` remains historically true and complete: it delivered the first monotonic `Clock`, `Instant`, and `Duration` surface and verified that slice. C2 does not rewrite C1's completion record. C2 is a successor checkpoint that intentionally modernizes parts of the C1 public and implementation surface while adding the remaining temporal domains.

C2 is implemented through exactly two numbered plans:

| Plan | Scope | Planning state |
|---|---|---|
| `STDL002.C2.P1` | modern temporal foundation: C1 migration, monotonic capabilities, absolute wall time, civil Gregorian values, fixed offsets, package/prelude exposure, and the minimum general-language prerequisite closure needed by those surfaces | plan set authored; implementation NOT_STARTED |
| `STDL002.C2.P2` | IANA timezone backend, stable timezone-database snapshot/lifetime, timezone identity/transitions, local resolution, zoned semantics, system-local conveniences, canonical text, deterministic testing clocks, ordinary-package/reflection/provenance/GC/performance closure, and final C2 checkpoint certification | plan authored; implementation NOT_STARTED; execution depends on completed P1 walkthrough/handoff |

C2 is complete only when both plans are complete **and** the checkpoint acceptance objective in §2 plus the final completion gate in §20 are satisfied.

The plan artifacts now exist. The installed P1 body filename is:

```text
STDL002.C2.P1-01-body.md
```

Earlier references to `STDL002.C2.P1-plan-01-body.md` are a naming-only planning artifact. They resolve to the installed body above and must not cause a duplicate body file to be created.

This checkpoint record is the durable lifecycle authority for C2. It owns plan state, established architecture, accepted amendments, verification evidence, deferred failures, consultations, and the current next action. Plan-local task mechanics belong in P1/P2; persistent implementation rules belong in `STDL002.C2-GUIDANCE.md`.

Two approved cross-plan amendments are already durable C2 authority:

1. LANG006 supersedes the older primitive-as-implementation-kind model. Declaration implementation kind is `Phalcom`, `Native`, or `Foreign`; `@primitive` and `@intrinsic` are independent, non-operative metadata.
2. Existing `time.md` material is stale and excluded. Authoritative time-specification work is `USER-DEFERRED` and is not a C2 completion requirement unless a later explicit user decision re-enables it.

## 1. Repository grounding

Initial checkpoint preparation used:

```text
repository: aureat/phalcom-lang
branch: main
revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
revision message: feat(lang005): continue P5 iterator surface
predecessor: STDL002.C1
```

The later P2 requirements/planning pass observed repository revision:

```text
206e0ab91129e189782c4028de5a40379bcc033c
```

This file keeps the original baseline for lifecycle truth but has been refreshed against the completed P1/P2 planning set and LANG006 authority.

Durable planning facts now established:

1. `STDL002.C1` is recorded `COMPLETE / COMPLETE / VERIFIED` and remains predecessor history.
2. C1 owns only the monotonic slice and explicitly deferred civil date/time and timezone work.
3. C2 has exactly two implementation plans; both plan artifacts now exist.
4. The P1 plan owns migration from C1's legacy time implementation conventions plus the absolute/civil/fixed-offset foundation.
5. The P2 plan owns timezone database lifetime/snapshot semantics, timezone/zoned behavior, canonical text, testing clocks, and final integration/certification.
6. P2 may not execute production work beyond its takeover task until a completed P1 walkthrough/handoff exists and is consumed.
7. LANG006 is the current authority for implementation kind and visibility mechanics: `Phalcom` / `Native` / `Foreign`, lexical identity-based visibility, and metadata-only `@primitive` / `@intrinsic`.
8. Older C2/P1 wording that described `primitive` as a declaration implementation kind is superseded by the approved C2 amendment in §23.
9. Existing `docs/spec/current/stdlib/time.md`, `docs/implementation/time.md`, and other prior `time.md` material are stale, excluded from semantic authority, and must not be read, edited, rehabilitated, or used as a completion gate during P1/P2 unless explicitly re-enabled later.
10. The public implementation must remain ordinary Phalcom first; Rust/provider code is restricted to explicit Native boundaries and host facts.

> Re-read named live source paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition. Do not use stale time specifications to fill design gaps.

## 2. Checkpoint acceptance objective

C2 is accepted when the Universe `time` package provides one coherent foundational temporal model covering:

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

ImplementationKind is one of:
    Phalcom
    Native
    Foreign

irreducible host/runtime observations are Native declarations marked @primitive
where that metadata classification is appropriate

native methods may implement substantial Rust-backed library behavior
without becoming part of the irreducible primitive floor

@intrinsic and @primitive are metadata only and never install implementations

Universe membership grants no spelling-based access-control bypass

System is not the public date/time namespace

the time package exports the canonical clock capabilities directly

Duration and Timestamp are the only temporal prelude promotions

timezone/database cost is lazy; importing time does not eagerly load tzdb

one VM/process temporal backend observes one coherent stable timezone-database
snapshot/generation according to the selected P2 adapter design

no temporal Value variants, temporal bytecodes, or VM-special temporal dispatch

stale time.md material is excluded; authoritative specification work is USER-DEFERRED
```

The public API must be consumable by an ordinary package through normal import, name-resolution, typechecking, lowering, bytecode, dispatch, allocation, GC, and reflection paths.

C2 acceptance is a checkpoint gate, not merely the fact that P1/P2 source exists. Plan completion evidence, ordinary-package certification, provenance, GC/lifetime, timezone correctness, parse/format round trips, and checkpoint lifecycle records all remain required.

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

### 3.5 Implementation kind and primitive metadata

LANG006 is the current authority. C2 consumes the normalized declaration implementation-kind model:

```text
Phalcom
    implementation body is ordinary Phalcom source

Native
    implementation body is supplied by Phalcom's native implementation provider
    (typically Rust in the current implementation)

Foreign
    implementation is supplied through an external interoperability boundary
```

`primitive` is **not** a fourth implementation kind.

The irreducible-floor distinction remains architecturally important, but is represented by metadata classification:

```text
@primitive
    Universe-authored informational metadata identifying an irreducible
    host/runtime-floor declaration when appropriate
```

Therefore the expected C2 combinations are:

```text
ordinary derivable temporal behavior
    ImplementationKind::Phalcom

host monotonic observation
host wall-clock observation
host/system timezone configuration discovery
    ImplementationKind::Native
    @primitive metadata

IANA/TZif database lookup/rule operations
    ImplementationKind::Native
    no @primitive metadata merely because Rust supplies the body
```

`@intrinsic` and `@primitive` remain non-operative: they do not select linkage, install a body, change visibility, alter identity, change representation, force lowering, or grant access.

C2 must keep the primitive metadata floor minimal. Performance alone is never sufficient reason to mark a Native operation `@primitive`.

`Foreign` is not expected to be needed by the ordinary C2 time implementation; if P1/P2 unexpectedly require it, use the general LANG006 semantics rather than inventing a time-specific foreign bridge.

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

## 8. Phalcom/Native/primitive-metadata boundary

### 8.1 Irreducible host observations

The expected irreducible C2 floor is narrowly limited to host facts such as:

```text
host monotonic-clock observation
host wall-clock observation
host/system timezone configuration discovery
```

Under LANG006 these are represented as:

```text
ImplementationKind::Native
+ @primitive metadata
```

P1 owns the first two.

P2 owns system timezone discovery.

The metadata does not install the Native provider and does not change visibility or dispatch.

### 8.2 Native timezone implementation

IANA/TZif database behavior is expected to use `Native` provider implementations without primitive metadata:

```text
named-zone open/lookup
offset-at-timestamp
civil-local resolution
transition-before
transition-after
canonical-name/database-adapter operations where required
```

These operations are substantial and external-data-backed but are conceptually library implementation. Using Rust/Jiff is an engineering/backend choice, not an irreducibility claim.

P2 must place them behind a Phalcom-owned adapter with lazy initialization, stable snapshot/generation semantics, cached named-zone handles, bounded backend conversion, and GC-safe lifetime ownership. Public APIs must not expose backend Rust/Jiff types.

### 8.3 Foreign boundary

C2 does not require a Foreign implementation path for ordinary time behavior. If live backend integration would require a Foreign declaration instead of the planned Native provider model, that is architectural drift and requires consultation.

### 8.4 Forbidden boundary inflation

Do not move the following into Rust merely because they are easier or faster there:

```text
Duration arithmetic
Timestamp arithmetic
Gregorian leap-year/day calculations
epoch-day conversion
Date/DateTime/Period arithmetic
fixed-offset conversion
disambiguation policy
ZonedDateTime equality/hash semantics
ZonedDateTime shift/advance semantics
ISO/RFC public grammar
canonical formatting
value equality/hash/comparison
Date.today / ZonedDateTime.now policy composition
```

If source-level implementation exposes a language defect, fix or escalate the owning language defect instead of expanding Native code reflexively.

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

The implementation authority is the P1 shared header plus the installed body:

```text
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md
```

Required scope:

```text
consume/verify the new visibility and LANG006 implementation-kind language surface

migrate C1 legacy time implementation conventions
    remove time-owned legacy implementation-selector naming
    remove legacy visibility-as-implementation-marker usage
    remove System.clock
    move host monotonic observation onto SystemClock

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
host wall-clock observation through Native + @primitive

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

representative ordinary-package/reflection/provenance certification
```

P1 also owns the **minimum prerequisite language closure** strictly necessary for this surface if it has not already landed, including:

```text
usable bodyless abstract member obligations for Clock.now / WallClock.now
validated data construction/invariants needed by Date/Time/UtcOffset/MonthDay
```

Visibility, Native/Foreign implementation kinds, metadata semantics, and grouped imports/exports are expected to come from the general language transition; P1 consumes them rather than creating a time-specific substitute.

P1 does **not** own IANA time zones, zoned date-times, system-local timezone semantics, full parsing/formatting, or final C2 certification.

# `STDL002.C2.P2` — Time Zones, Zoned Semantics, Textual Forms, Testing Support, and Closure

P2 depends on the complete P1 foundation and may not execute production work beyond takeover/re-grounding until the completed P1 walkthrough/handoff has been consumed.

Required scope:

```text
timezone database adapter
freshly verified mature Rust backend behind Phalcom-owned Native API
lazy database initialization
stable coherent VM/process database snapshot/generation
cached named-zone handles
backend range conversion without narrowing P1 temporal ranges

TimeZone
    utc
    fixed
    named
    system
    stable semantic identity

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

canonical machine formatting
canonical parsing
ISO/RFC temporal forms where defined by the P2 plan
offset/zone consistency checks
round-trip properties
expanded-year/fraction/second-offset correctness
lossless Duration/Period text

time.testing deterministic clocks
ordinary-package consumption certification
reflection and type-observation certification
Native/@primitive provenance audit
GC/lifetime certification
performance/resource/lazy-init sanity
checkpoint walkthrough/handoff closure
```

Authoritative `time` specification work is **USER-DEFERRED**. P2 does not read, edit, rehabilitate, or certify against stale `time.md` documents and does not treat a specification artifact as a completion requirement unless a later explicit user instruction re-enables that work.

P2 does not reopen P1's type/category decisions unless live evidence proves them unsound and an approved checkpoint amendment is recorded.

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

### `INV-07` — implementation kind follows LANG006

C2 declarations use `ImplementationKind::Phalcom`, `ImplementationKind::Native`, or `ImplementationKind::Foreign`. `primitive` is not an implementation kind.

### `INV-08` — primitive metadata means irreducible classification, not linkage

`@primitive` is reserved for genuinely irreducible host/runtime-floor facts where appropriate. Timezone database operations may be Native without being marked `@primitive`.

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

Public `time` APIs compile and run through normal package semantics. Universe privilege is limited to explicitly authorized metadata and Native linkage; Native code does not receive blanket internal access.

### `INV-17` — no temporal VM specialization by default

No new temporal `Value` arm, object kind, temporal bytecode, or standard-library dispatch special case is introduced without a separately approved measured justification.

### `INV-18` — timezone database snapshot is stable and coherent

The selected P2 adapter provides one coherent VM/process timezone-database snapshot/generation. Existing `TimeZone` values do not silently change rule meaning mid-process.

### `INV-19` — backend limits do not redefine Phalcom ranges

Finite backend timestamp/civil/offset ranges are interop constraints only. They do not narrow P1's canonical temporal semantics; backend conversion failure is explicit.

### `INV-20` — parsing and formatting remain Phalcom-owned semantics

The Rust timezone backend may supply timezone facts but does not define the accepted public grammar or canonical formatting contract.

### `INV-21` — stale time specifications are excluded

Existing `time.md` material is not C2 semantic authority. Authoritative time-specification work remains `USER-DEFERRED` until explicitly re-enabled.

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

## 13. TimeZone identity and local-time resolution rules

P2 must give `TimeZone` stable semantic identity independent of raw allocation/backend pointer identity.

Required identity model:

```text
UTC
    one canonical UTC semantic zone

fixed
    identity = UtcOffset value
    equal fixed offsets compare equal

named
    identity = selected database snapshot/generation + stable canonical logical identifier
    raw backend pointer identity is not public equality

system
    discovery operation returning an ordinary stable TimeZone value
    not a mutable singleton "system zone"
```

Alias/canonical-name behavior is verify-first against the selected backend. Prefer canonical-name equality when the backend can expose it reliably. If stable alias identity cannot be defined without inventing a large normalization subsystem, STOP AND CONSULT.

`TimeZone.system` observes current host configuration each time it is invoked. A later host configuration change may affect a later call, but must not mutate previously returned `TimeZone` values. Discovery failure is explicit; it must not silently fall back to UTC.

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

Required policy:

| Case | Reject | Earlier | Later | Compatible |
|---|---|---|---|---|
| unique | value | value | value | value |
| overlap | error | earlier | later | earlier |
| gap | error | before | after | after |

`Gap.before` and `Gap.after` have fixed semantic meaning:

```text
Gap.before
    requested local DateTime interpreted with the pre-transition offset

Gap.after
    requested local DateTime interpreted with the post-transition offset
```

This meaning must be verified against the selected backend using an ordinary spring-forward gap, a non-one-hour transition, and a historical transition. Do not redefine the payload from backend convenience behavior.

Default conversion policy:

```text
DateTime.in(zone)
    Reject

ZonedDateTime.advance(Period)
    Compatible
```

The backend supplies factual mapping data. Phalcom source constructs `LocalTimeResolution` and applies disambiguation policy.

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
whole-repository implementation-kind/metadata migration
```

C2 consumes the active language surface; it does not become the project that migrates every other Universe package.

---

## 17. Expected source areas

Expected C2 ownership includes the live equivalents of:

```text
phalcom-core/core/universe/src/time/**
phalcom-core/core/universe/src/package.ph
phalcom-semantic/tests/semantic/capabilities/time.rs
phalcom-core time/language-corpus tests
Native provider/descriptor paths needed by time
LANG006 metadata/provenance paths used by @primitive classification
time-related reflection/core-surface contracts
concurrency call sites that consume the migrated Duration/clock API
P2 timezone backend/dependency and VM-owned adapter state
C2 checkpoint/walkthrough/handoff records
```

The following are explicitly excluded unless a later user instruction re-enables them:

```text
docs/spec/current/stdlib/time.md
docs/implementation/time.md
other stale time.md specification/design material
```

P1 may need narrow parser/AST/semantic/compiler/runtime changes for missing prerequisites such as abstract-member obligations or validated `data` construction.

P2 may add the Rust timezone adapter/dependency and VM-owned timezone database state. That state must have explicit non-root/root classification in the VM/GC ownership model and must not leak backend types into public reflection.

Touching unrelated standard-library domains is not expected.

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
module/package/private/protected/internal access boundaries
ordinary-package consumption
reflection/type observation
Native/@primitive metadata provenance
timezone database lazy initialization and stable snapshot lifetime
timezone transition gaps/overlaps
historical second-resolution offsets where backend data supplies them
named/fixed/UTC/system zone identity and non-equivalence
exact-vs-calendar arithmetic
parse/format round trip
timezone offset/identifier consistency
backend range error behavior without P1 narrowing
GC/lifetime of native zone handles
cache/handle sharing and resource sanity
time.testing deterministic clocks
concurrency integration with Duration
baseline-failure classification
```

P1 owns representative ordinary-package/reflection/provenance evidence for its substrate. P2 owns the final C2 ordinary-package/reflection/provenance/GC/performance certification across the completed timezone/zoned/textual surface.

Tests should prove semantics at the owning layer rather than mirror private implementation details. Deterministic timezone-rule assertions use controlled/pinned fixtures rather than relying solely on ambient host tzdata.

Execution breadth remains narrow during BUILD and broadens only at named plan/checkpoint gates.

## 19. Global STOP / CONSULT triggers

In addition to repository-wide workflow triggers, C2 requires immediate consultation if:

1. the new visibility model cannot express the required `Clock`/`Instant` construction relationship without a spelling convention or privilege bypass;
2. `@intrinsic` or `@primitive` would need operative semantics to make C2 work;
3. the live LANG006 `Phalcom` / `Native` / `Foreign` implementation-kind model materially differs from the accepted C2 reconciliation;
4. C2 would need to recreate a legacy spelling-based or special-attribute implementation-access mechanism to bridge an incomplete language transition;
5. abstract-member obligations cannot support `Clock.now` / `WallClock.now` without a broader redesign;
6. validated `data` values cannot enforce `Date`, `Time`, `UtcOffset`, or `MonthDay` invariants without changing the accepted `data` model;
7. `SystemClock.instance` / exported `clock` cannot preserve one stable domain identity through normal package initialization;
8. timezone state would require uncertain per-object Rust ownership/GC semantics rather than a proven stable adapter/handle model;
9. the timezone backend forces public semantics to mirror backend quirks or accepted grammar;
10. the selected backend cannot provide a coherent stable VM/process timezone-database snapshot/generation under a reasonable ownership model;
11. backend local resolution cannot preserve unique/ambiguous/gap factual mappings cleanly, including the required Gap.before/after semantics;
12. backend finite ranges would force narrowing P1 temporal ranges;
13. historical second-resolution offsets would be lost where the backend data exposes them and P1 permits them;
14. `TimeZone` equality would need raw backend pointer identity or repeated full-rule comparison;
15. `ZonedDateTime` requires redundant public canonical fields for correctness;
16. a public C2 operation appears to require a temporal VM bytecode or `Value` specialization for correctness rather than optimization;
17. a source-level time bug can only be bypassed by moving large derivable semantics to Rust;
18. C2 conflicts materially with an active language checkpoint;
19. expected semantic test behavior would have to change rather than fixing an implementation defect;
20. deterministic timezone tests can only be expressed against mutable ambient host tzdata;
21. parsing/formatting cannot round-trip all valid C2 values, including second-resolution offsets or valid mixed-sign `Period` values if P1 permits them;
22. checkpoint scope expands into timers/reactor redesign, locale/CLDR, recurrence, general FFI redesign, or whole-Universe migration;
23. a stale time specification would need to be revived merely because older planning text referenced it.

Triggered consultation is not self-waivable.

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
Native/@primitive metadata boundary inventory
stable timezone database snapshot/lifetime behavior
TimeZone identity/cache correctness
timezone transition and local-resolution correctness
ZonedDateTime equality/conversion/shift/advance correctness
parse/format round trips and offset-zone consistency
time.testing deterministic clock behavior
GC/lifetime safety of zone-backed values
lazy-init/performance/resource sanity
required focused owning suites
P1 and P2 walkthrough/handoff lifecycle records
checkpoint/source/test agreement
```

Authoritative `time` specification work is `USER-DEFERRED` and is **not** a C2 completion requirement. Existing stale `time.md` files must remain untouched by C2 unless a later explicit user decision re-enables specification work.

Workspace-wide release certification is separate. Broad workspace test failures may remain classified and recorded when they are demonstrably unrelated; C2 does not require an unrelated repository baseline to become globally green.

## 21. Plans ledger

| Plan | Objective | Plan artifact state | Implementation status | Completion | Verification |
|---|---|---|---|---|---|
| `STDL002.C2.P1` | modern temporal foundation, absolute time, civil calendar, fixed offset, package/prelude migration | shared header + installed body authored | PROPOSED | NOT_STARTED | UNVERIFIED |
| `STDL002.C2.P2` | timezone/zoned/textual/testing layers and C2 integration closure | full patch-grade plan authored | PROPOSED | NOT_STARTED | REQUIREMENTS_ANALYZED |

Installed plan artifacts:

```text
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md
STDL002.C2.P2-plan.md
```

The P2 plan was authored before P1 implementation only because the user explicitly requested it. That does not waive its execution precondition: P2 must consume the completed P1 walkthrough/handoff before production work proceeds beyond takeover/re-grounding.

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

### `C2-AMEND-01` — stale time specification excluded and specification work deferred

Approved by explicit user direction during P1/P2 planning.

```text
existing time.md material
    stale
    not semantic authority
    not a required read
    not edited by P1/P2

authoritative time specification work
    USER-DEFERRED
    not a P1/P2/C2 completion gate
    re-enabled only by later explicit user instruction
```

This supersedes older checkpoint/guidance/P1-header wording that required reading or producing a final `time` specification.

### `C2-AMEND-02` — LANG006 implementation-kind reconciliation

Approved by the later LANG006 authority consumed during P2 planning.

```text
ImplementationKind
    Phalcom
    Native
    Foreign

@primitive / @intrinsic
    independent non-operative metadata
```

The older C2/P1 conceptual wording that treated `primitive` as a declaration implementation kind is superseded.

C2 preserves the primitive-floor **classification** as follows:

```text
host monotonic observation
host wall-clock observation
host/system timezone discovery
    Native + @primitive metadata

timezone database/rule operations
    Native without @primitive merely for being Rust-backed
```

Metadata does not install implementations or change visibility, identity, representation, lowering, or dispatch.

### `C2-AMEND-03` — P2 plan authored early by explicit instruction, execution dependency retained

P2 planning was completed before P1 implementation in response to explicit user instruction. This is a planning-order exception only.

P2 production implementation remains blocked on:

```text
completed P1 walkthrough
completed P1 handoff
P2 T0 consumption/reconciliation of as-built P1 interfaces
```

### `C2-AMEND-04` — P1 body installed filename

The installed P1 body is:

```text
STDL002.C2.P1-01-body.md
```

References to `STDL002.C2.P1-plan-01-body.md` in earlier planning text are naming-only aliases and must not create a competing file.

Any future material change to:

```text
temporal semantic categories
LANG006 implementation-kind/metadata distinction
visibility/access model
prelude policy
clock identity/lifetime
timezone ownership/lifetime
timezone identity/snapshot semantics
two-plan decomposition
specification deferral status
```

must be recorded here after approval.

## 24. Current next action

Execute `STDL002.C2.P1` from the authored P1 plan set:

```text
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md
```

P1 takeover must first verify that the required general language surface has actually landed, especially:

```text
lexical identity-based visibility
grouped imports/exports
LANG006 Phalcom/Native/Foreign implementation kinds
non-operative @intrinsic/@primitive metadata
bodyless abstract-member obligations
validated transparent data construction
generic data identity
class-side singleton/package-value identity
```

Do not implement a time-specific substitute for missing general-language architecture.

After P1 reaches its final gate, produce and finalize:

```text
STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md
```

Then begin `STDL002.C2.P2` by consuming those as-built records in P2 T0. Do not treat the already-authored P2 plan as permission to skip P1 reconciliation.

P2 closes C2 only after its final certification gate, walkthrough/handoff, and checkpoint update. Authoritative time-specification work remains `USER-DEFERRED`.
