---
id: STDL002.C2
category: STDL
program: STDL002
kind: implementation-checkpoint
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - STDL002.C1
follows: STDL002.C1
supersedes: null
prepared: 2026-09-17
updated: 2026-09-19
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
| `STDL002.C2.P1` | modern temporal foundation, absolute time, civil calendar, fixed offset, package/prelude migration | shared header + installed body authored | IN_PROGRESS | PARTIAL | UNVERIFIED |
| `STDL002.C2.P2` | timezone/zoned/textual/testing layers and C2 integration closure | full patch-grade plan authored | PROPOSED | NOT_STARTED | REQUIREMENTS_ANALYZED |

Installed plan artifacts:

```text
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md
STDL002.C2.P2-plan.md
```

The P2 plan was authored before P1 implementation only because the user explicitly requested it. That does not waive its execution precondition: P2 must consume the completed P1 walkthrough/handoff before production work proceeds beyond takeover/re-grounding.

## 22. Deferred/baseline failures

### `STDL002.C2.P1-T0-BASELINE` — predecessor focused baseline

Execution takeover on `main` at `f7e2ca1585c755e16f7046e5c24d0858dc2c9574` was clean. The semantic capability filter passed 2/2 tests. The core `language-corpus time` filter selected three tests and failed in pre-existing C/D surfaces: the positive time fixture reaches the current identity-based `internal` access rule through `ManualClock.new()`/`allocate()`, the negative fractional-duration fixture reports the generic argument-mismatch diagnostic rather than its stale `Duration.seconds` substring, and the filter also includes two unrelated `runtime_errors` cases that fail at the same access boundary. The narrower `time_negative` filter selected one test and reproduced the same stale diagnostic expectation.

These failures occurred before P1 production edits and are migration/baseline evidence, not P1 implementation failures. They remain nonblocking until the migrated P1 fixtures replace the stale C1 expectations; no unrelated runtime-error repair is authorized here.

When implementation finds a failure, classify it:

```text
A — definitely caused by active C2 patch
B — probably caused by active C2 patch
C — unclear after one bounded classification pass
D — clearly unrelated/baseline
```

Record meaningful C/D failures here rather than consuming C2 scope repairing unrelated work.

### `STDL002.C2.P1.A1-G3-BASELINE` — builtin catalog unrelated failure

The focused `phalcom-modules --test builtin_catalog` run selected 9 tests and
passed 8. `bcat_08_root_alias_does_not_create_synthetic_declaration` failed
because the catalog fixture could not resolve the unrelated
`universe.collections.iterator_laws` import. This is classification D: the
failure is outside the time graph/publication changes and does not block the
A1 civil or standalone-closure acceptance evidence. It remains deferred to
the owning collections/catalog work.

### `STDL002.C2.P1-T2` — Duration and Measurement foundation

The T2 slice is implemented and focused-tested at the current execution head.
`Duration` now lives in the canonical `universe.time.duration` module as
restricted transparent data with exact signed `Int` nanoseconds; ordinary
class-side factories remain the validation/construction boundary.
`Measurement<T>` is generic transparent data in the same module. Runtime
prelude publication now uses the existing data registry descriptors, so
source-owned data behavior classes retain their defining-module identity on the
normal unqualified path. The time corpus exercises arithmetic/equality plus
`Measurement(7, Duration.seconds(1))` construction and component reads.

Focused evidence:

```text
PASS  cargo test -p phalcom-core --test language-corpus corpus::time -- --exact
PASS  cargo test -p phalcom-core --test language-corpus time_negative
PASS  cargo test -p phalcom-semantic --test semantic capabilities::time
PASS  cargo test -p phalcom-core --test core language::data_e2e::data_generic_applications_keep_exact_identity -- --exact
PASS  cargo test -p phalcom-core --test core compiler_product_optimization::private_data_factory_has_representation_independent_optimization -- --exact
PASS  cargo test -p phalcom-ast --test integration data_syntax::data_constructor_visibility
```

The direct child-module import used by the corpus is executable. The former
package-root import seam was closed by the generic standalone dependency
closure repair and is covered by the A1 package-root regression evidence.

### `STDL002.C2.P1-T3` — monotonic capability migration

The T3 monotonic slice is implemented and focused-tested. `Clock` now carries
the declaration-only `now` contract plus ordinary source `since` and generic
`measure` behavior. `SystemClock` is a concrete source-owned subclass with a
stable class-side singleton, and the time package binds `clock` to that exact
singleton. `System.clock` and the legacy System monotonic bridge are removed.

Native ownership now supports canonical source-declared Universe classes through
the existing descriptor/catalog, source-anchor verification, and provenance
path: primordial descriptors install before source bootstrap; source-owned
descriptors install after their declaring module materializes. The host read is
therefore owned by `SystemClock`, without a second time-specific registry.

The data lowering path also carries generic construction templates through the
active runtime type environment and avoids caching one open-generic descriptor
across different applications. The time fixture exercises this through
`Clock.measure<T>` and `Measurement<T>`.

Focused evidence:

```text
PASS  cargo check -p phalcom-core
PASS  cargo test -p phalcom-core --test language-corpus corpus::time -- --exact
PASS  cargo test -p phalcom-core --test language-corpus corpus::time_negative
PASS  cargo test -p phalcom-semantic --test semantic capabilities::time
PASS  cargo test -p phalcom-core --test core native_surface_contracts::time_monotonic_ -- --nocapture
PASS  cargo fmt --all -- --check
PASS  git diff --check
```

The package-root `from universe.time import clock` probe was the historical
module-linking reproducer for A1 and now passes through the generic closure
path; no time-specific alias or linker exception was retained.

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

### `C2-AMEND-05` — P1 takeover and documentation deferral recorded

P1 became active at execution revision `f7e2ca1585c755e16f7046e5c24d0858dc2c9574` after the required narrow baseline. The body-level amendment `P1-BODY-DEV-01` is reaffirmed: existing `time.md` documents are excluded as stale, and authoritative time-specification work remains `USER-DEFERRED`. The live prerequisite matrix is mechanically landed for grouped imports/exports, lexical visibility, LANG006 implementation kinds, non-operative metadata, abstract-member syntax, generic data identity, and singleton/package identity; validated transparent data construction and concrete abstract-member completeness remain T1 verification/closure points before temporal source migration.

### `C2-AMEND-06` — standalone Universe closure and T6 resume

Architect guidance is recorded by
`STDL002.C2.P1.A1-standalone-universe-period-t6-blocker-remediation-plan.md`.
The standalone compiler now reuses the canonical resolver-backed reachable
dependency worklist while retaining standalone entry admission and entry-only
runtime ownership. Strict linking remains strict, and no time-specific
resolution branch or duplicate temporal identity authority is permitted.

The amendment also authorizes publication and focused verification of the
already-authored non-`UtcOffset` T6 civil modules, then hands execution back to
the existing P1 body. The separate UtcOffset VERIFY-FIRST decision was later
resolved by `C2-CONSULT-03` below.

### `C2-CONSULT-01` — T1 validated-data construction contract is not defined in the live tree

P1 is paused before production time edits under the shared consultation trigger for a false/missing core plan premise and an ownership/semantic-authority boundary that cannot be chosen mechanically.

```text
Observed:
  DataDef has transparent immutable components and a canonical DataConstructorId;
  compiler/lib/data_decl.rs emits direct data construction metadata and
  ConstructData lowering has no validation hook; DataDef attributes are not
  consumed by the data compiler path. Existing @invariant weaving is limited
  to ClassDef members, and the current accepted data decision does not define
  validated-data construction.

Required P1 invariant:
  Date, Time, MonthDay, and UtcOffset remain transparent data while every
  ordinary public construction lane rejects invalid state; no bypass lane may
  manufacture invalid values.

Why local authority is insufficient:
  Adding a constructor hook, data-level invariant attribute, hidden validated
  class wrapper, or compiler-recognized temporal constructor would select a new
  general data representation/validation contract and create a second source
  of construction authority without an accepted language decision.

Requested decision:
  Ratify the general validated-transparent-data construction model and its
  owning layer/observable error semantics, or amend P1 to use an already
  accepted alternative. After that decision, resume T1 and record the exact
  focused owning-layer tests before temporal source migration.

Resolution: superseded by the current LANG006 implementation at `7a9ba64987890821fba0e79d96d7457dce5e3adf` and the user's explicit continuation authority. Data declarations now normalize primary-constructor visibility, semantic signatures carry that visibility, and access is checked before construction lowering. Validated data therefore remains transparent immutable data: an ordinary class-side factory performs validation and then calls the restricted primary constructor, as in `Date.new(...) -> Date`. No compiler temporal hook, hidden class wrapper, or second construction authority is introduced. Invalid values remain rejected at the public factory boundary, while same-owner construction retains the normal lexical access rule. The consultation is closed for P1 execution; remaining abstract-member completeness is verified as part of T1/G1.

### `C2-AMEND-06` — validated transparent data construction is now an available general-language prerequisite

The current execution head supplies the previously missing general mechanism. P1 consumes the following fixed model:

```text
data primary constructors may be restricted by ordinary visibility
private/module/package/internal follow the canonical lexical-identity rules
protected is rejected for non-inheritable data declarations
ordinary impl/class-side factories validate inputs in Phalcom source
the factory constructs the transparent data value through the restricted
primary constructor after validation
construction access is authorized before normal lowering on every lane
```

The model is general and is not time-specific. P1 may use it for `Duration`,
`Timestamp`, `Date`, `Time`, `DateTime`, `YearMonth`, `MonthDay`, and
`UtcOffset` without adding compiler-recognized temporal validation or changing
data representation. The implementation authority is the existing AST,
semantic constructor signature/access product, and normal source-level `impl`
factory path; the runtime remains unaware of temporal validity rules.
```

### `C2-AMEND-07` — enum cases remain bare source cases

Explicit user direction for the remaining civil-value work: new enum
declarations must use the current bare-case syntax and must not put `@variant`
on enum cases. This applies to `Month`, `Weekday`, `DateOverflow`, and any
other enums added by P1; it does not authorize changing unrelated legacy enum
fixtures in this slice.

### `C2-CONSULT-03` — UtcOffset bound approved and incident resolved

The UtcOffset range incident is recorded in
`STDL002.C2.P1-T6-utc-offset-bound-incident.md`. The advisor approved the
following decision:

```text
Authority:
  approved P1/P2 backend-interoperability constraint

Canonical public domain:
  min_seconds = -93_599
  max_seconds =  93_599
  inclusive

Equivalent:
  -25:59:59 ..= +25:59:59

Rationale:
  signed whole-second state;
  historical second precision remains representable;
  the approved Jiff-compatible POSIX/TZif-oriented envelope is exact;
  symmetry makes negation total at both extrema.

Non-claims and downstream obligation:
  this is not a maximum-historically-observed-offset claim;
  P2 may not narrow the P1 domain and must preserve exact second precision.
```

This decision resolves shared-header STOP/CONSULT trigger 16 for T6. The
implementation remains ordinary validated transparent data: no temporal
compiler hook, VM primitive, native representation, or timezone semantics is
introduced.

### `C2-CONSULT-02` — T3 needs general abstract-class and source-native ownership support

T2 is complete, but P1 T3 cannot be implemented faithfully from the current
live language/runtime contracts without an architectural decision.

```text
Observed:
  phalcom-ast class members always call parse_member_body(); bodyless
  declaration members are admitted only through parse_behavior_member(...,
  allow_index_declaration = true) for trait contexts. ClassDef has no abstract
  marker or abstract-obligation surface.

  The runtime ClassObject has is_abstract, but current abstract roots are
  manually marked in core_classes.rs. There is no source-class path that turns
  an ordinary class with declaration-only members into an abstract class and
  no semantic completeness product for that obligation.

  The existing monotonic host read is registered as the generated/native
  Universe System class member System.class::monotonicNanoseconds. NativeSource
  indexing and the native metadata catalog anchor source native classes through
  UniverseKey; there is no SystemClock UniverseKey or general source-owned
  native-class attachment path. The current primitive installer hardcodes the
  System class owner.

Required P1 invariant:
  Clock is an abstract source capability with declaration-only now, source
  implementations remain ordinary Phalcom, SystemClock owns the host primitive,
  and System no longer owns temporal discovery or the monotonic read.

Why local authority is insufficient:
  Writing a throwing Clock body, treating Clock as a trait, retaining System as
  the canonical primitive owner, or inventing a one-off compiler/runtime hook
  would violate the fixed P1 ownership/abstractness contract and create a
  second language model.

Requested decision:
  Ratify the general abstract-class syntax/obligation model and the general
  source-class native/primitive anchoring mechanism needed by SystemClock, or
  amend P1 to an explicitly transitional concrete Clock/System owner. T3/T4
  production edits remain paused until that decision is recorded.

Resolution: user clarified that classes may be abstract and may contain
bodyless abstract members. The live AST already preserves declaration-only
class members, so the earlier abstractness blocker was overstated: T3 may
complete the existing general compiler/runtime path rather than redesign the
language around traits or a throwing body. The remaining independent question
is only how the existing native/primitive linkage attaches the host read to
source-owned `SystemClock` identity; do not conflate that linkage audit with
abstract-class support.
```

### `C2-PREREQ-01` — source abstract-member closure is implemented

The general compiler/runtime path now treats declaration-only Phalcom class
members as abstract obligations. Class layouts retain side-qualified pending
selectors, inherit obligations from the superclass, remove them when a concrete
member overrides the same dispatch identity, and mark the runtime class
abstract while any obligation remains. Declaration-only members do not lower to
throwing bodies or executable method objects.

Focused evidence:

```text
PASS cargo test -p phalcom-core --test core language::abstract_classes::declaration_only_member_makes_class_abstract_and_concrete_override_executes -- --exact
PASS cargo test -p phalcom-core --test core language::abstract_classes::abstract_class_construction_is_rejected -- --exact
PASS cargo test -p phalcom-core --test core language::abstract_classes::subclass_with_unresolved_abstract_member_remains_abstract -- --exact
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
PASS cargo fmt --all -- --check
PASS git diff --check
```

This closes the abstractness portion of the former consultation. It does not
resolve the separate source-owned Native descriptor/linkage gap for
`SystemClock`; T3 remains the active plan task for that seam.

## 24. `STDL002.C2.P1-T4` — absolute wall-time vertical focused closure

T4 is implemented and focused-tested. `Timestamp` is transparent signed Unix /
POSIX nanosecond data; `WallClock` is an abstract source capability with a
bodyless `now -> Timestamp`; `SystemWallClock` is the public singleton
implementation; and `Timestamp.now` delegates through the canonical source
`wallClock` binding to `SystemWallClock.now`.

The host observation is the only wall-time primitive and is owned by
`SystemWallClock`. It constructs the signed result with arbitrary-precision
integers on both sides of the epoch and does not narrow through `i64` or
`Float`. `Timestamp` arithmetic, comparison, equality, hashing, and factories
remain ordinary Phalcom behavior. During the focused run, the shared data
runtime also exposed that every transparent data declaration had been sharing
the primordial `Class` metaclass; T4 corrected that owning-layer defect by
giving each data behavior its own metaclass, preserving independent class-side
factory tables for `Duration`, `Measurement`, and `Timestamp`.

Focused evidence:

```text
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time -- --exact
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time_negative
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts::time_wall_ -- --nocapture
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
PASS cargo fmt --all
PASS git diff --check
```

The corpus covers epoch and signed scalar state, duration arithmetic and
`since`, ordering/equality/hash, stable `SystemWallClock` identity, and the
`Timestamp.now` capability path. The native contract covers exact owner,
return type, runtime `Int`, and absence of a second `now` primitive.

The direct explicit-import probe for `wallClock` still reaches the known
canonical Universe module-linking seam (the same family as the deferred T7
package-root import issue); no time-specific alias or second singleton was
introduced to mask it. Repeated singleton identity is covered through the
canonical source binding and runtime fixture; explicit-import closure remains
deferred to T7's module-linking work.

### `C2-G3` — T4 acceptance state

```text
P1-CV-14  PASS  Timestamp epoch and signed scalar factories
P1-CV-15  PASS  Timestamp plus/minus/since Duration laws
P1-CV-16  PARTIAL  signed state is covered; exposed Euclidean split remains T6
P1-CV-17  PARTIAL  singleton identity passes; explicit-import identity awaits T7 linking
P1-CV-18  PASS  Timestamp.now uses source wallClock path; no second native now
P1-CV-19  PASS  wall primitive provenance/owner is SystemWallClock
```

T4 is therefore `IMPLEMENTED` + `FOCUSED_TESTED`, with the explicit-import
portion of the acceptance surface deferred as a known integration seam.

## 25. `STDL002.C2.P1-T5/T6` — Gregorian/calendar foundation and blocker remediation

The T5 source slice is now present. `Month` and `Weekday` use bare enum cases
with no `@variant` annotations, ISO numbering, invalid-number rejection, and
source-level enum behavior. `Period` is transparent months-plus-days data with
year/month/week/day factories, arithmetic, negation, and zero detection; it
does not define ordering. The single Gregorian helper authority is colocated
with `Month` so the public month module does not require the currently-broken
nested sibling-module import path. It contains leap-year, month-length,
constant-time civil/epoch-day conversion, and explicit floor-division logic
for negative years.

Focused evidence so far:

```text
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time -- --exact
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time_negative
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
PASS cargo fmt --all -- --check
PASS git diff --check
```

The time corpus now observes leap/non-leap February, Month and Weekday
numbering, alongside the already-covered T2-T4 behavior. The direct
`Period` owner import now passes through the standalone compiler path. Its
initial diagnostic was an authored `Period.isZero` Boolean-body mismatch, not
a duplicate declaration/type identity: the canonical data owner, declaration
form, constructor result, and factory returns agree. The body now uses an
equivalent typed conditional. The Period runtime and ordering-negative lanes
are now covered below.

### T5/T6 module-import limitation and remediation

A focused probe adding `from universe.time import Period` to the existing time
corpus fails during package linking before the imported name is typechecked:

```text
program compile failure: unresolved import 'universe.time.clock' from module universe:time
```

The failure is distinct from the direct child-module path: imports such as
`from universe.time.calendar.month import Month` and
`from universe.time.calendar.weekday import Weekday` remain the currently
working path. The live builtin graph and source provider register the `time`
package and its child modules, but root-package publication does not currently
make the package's own absolute child imports link successfully. The same
limitation explains the previously recorded explicit `wallClock` import and
the direct synthetic `Period` identity seam; it must be repaired in the
general module/package linker or publication path, not by adding time-specific
aliases or duplicate declarations.

The live blocker was reproduced at execution HEAD: a standalone import of
`universe.time.timestamp` failed on the target-owned `universe.time.duration`
edge, and a standalone import of `universe.time` failed on
`universe.time.clock`. The standalone sibling-import rejection remained green.
The compiler repair now discovers those transitive modules through the same
resolver-backed worklist used by project/package analysis; the exact graph
regressions pass. Loading the newly reachable temporal sources exposed
existing semantic diagnostics in their bodies, which remains separate from the
module-closure gate; the source-level diagnostics were repaired at their
owning Phalcom definitions.

The module-layer limitation is closed. Direct standalone `Period`, `Date`,
`Time`, `DateTime`, `YearMonth`, and `MonthDay` imports now compile and execute
through the normal language-corpus path. The standalone `universe.time` and
`universe.time.calendar` package regressions prove child discovery and
canonical package exports.

The T6 civil source is now published through the canonical builtin graph and
source provider: `Date`, `DateOverflow`, `Time`,
`DateTime`, `YearMonth`, and `MonthDay` are registered as normal Universe
modules, and both `universe.time.calendar` and `universe.time` expose the
existing civil declarations. No time-specific compiler/module-resolution
branch was added.

The source graph was completed mechanically where live source required it:
`Date` now imports `Weekday`; the existing `Date`/`Time`/`DateTime` comparison
tails use typed conditionals where the checker does not infer Boolean results
through overloaded conjunction; `DateTime.==` uses a typed conditional; and
the live `Date.advance` overflow label/variable is internally consistent.
Unsupported source spellings (`|>` and `!==`) were removed from the published
source in favor of equivalent live-language forms. These changes retain
ordinary Phalcom ownership and do not add a second temporal construction or
identity authority.

When the new `UtcOffset` export caused the complete `universe.time` package
to be semantically checked, the existing `Instant` and `Timestamp` `<=`/`>=`
tails exposed the same conjunction return-contract diagnostic. Those four
tails were changed to equivalent typed conditionals at their owning source
definitions so package publication is semantic-clean; no temporal runtime or
compiler behavior changed.

`UtcOffset` is now implemented against the approved P1/P2
backend-interoperability decision recorded in `C2-CONSULT-03`: the public
validated domain is inclusive `-93_599..=93_599` signed seconds. The range is
an interoperability envelope, not a historical-maximum claim; P2 may not
narrow it and must preserve exact second precision.

### Independent direct-calendar regression closure

The direct child-module lane was strengthened with invalid-number fixtures for `Month.fromNumber(13)`
and `Weekday.fromNumber(0)`. The Gregorian month-movement helper was also
adjusted from a record-returning shape to typed year/month integer projections;
this avoids a canonical semantic return-mismatch diagnostic while preserving
one source-level arithmetic authority.

Focused evidence:

```text
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time -- --exact
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time_negative -- --exact
```

The direct Gregorian round-trip fixture also caught and corrected an inverse
algorithm defect: `civilFromDays` now restores the `+719468` offset that
`daysFromCivil` removes. Coverage includes 1970-01-01, 1969-12-31, year zero,
leap-year anchors, and negative month movement.

The direct calendar negative lane now also covers Gregorian month-range
rejection and non-leap-year day rejection. The unregistered T6 `Date` source
was mechanically prepared for later canonical publication: it imports the
`Weekday` dependency it uses and reads `Gregorian.civilFromDays`'s record
through `Record.get(...).unwrapOr(...)`, matching the live record API rather
than unsupported field syntax. No module/linker behavior was changed.

Additional focused evidence:

```text
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time_negative -- --exact
```

The amendment's G3 evidence is now complete:

```text
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time -- --exact --nocapture
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus corpus::time_negative -- --exact --nocapture
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time -- --nocapture
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_universe::standalone_universe_time_package_import_closure -- --exact --nocapture
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_universe::standalone_universe_calendar_package_exports_civil_types -- --exact --nocapture
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core reflection_census::physical_universe_corpus_matches_catalog -- --exact --nocapture
PASS RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core reflection_census::package_exposures_match_catalog_children -- --exact --nocapture
```

The positive corpus now covers civil boundaries, epoch anchors, leap-year
round trips, `DateTime` projections, Period month/day application, overflow
constrain/reject, `YearMonth` movement, `MonthDay`, and `Time` boundaries. The
negative corpus covers invalid civil constructors and the intentionally
unavailable `Period` ordering operation.

## 26. Current next action

P1 remains active after the A1 blocker-remediation amendment. T1's
validated-data and abstract-member prerequisites, T2 Duration/Measurement, T3
monotonic time, T4 absolute wall-time, T5 Period identity, and the non-offset
T6 civil layer are implemented and focused-tested. The standalone transitive
Universe closure and canonical package-publication blocker is removed; no
time-specific workaround remains.

The approved UtcOffset decision has been consumed and the T6 civil contract
is focused-tested. The next action returns to the existing P1 body: continue
the remaining package/ordinary-package closure and T7 publication work. Do
not replace that remaining work with a time-specific workaround:

```text
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md
```

P1 takeover verified the required general language surface and the preceding
focused gates. Remaining work is the later ordered implementation gates and
ordinary-package closure:

```text
lexical identity-based visibility
grouped imports/exports
LANG006 Phalcom/Native/Foreign implementation kinds
non-operative @intrinsic/@primitive metadata
bodyless abstract-member obligations
validated transparent data construction (landed and focused-tested in T2)
generic data identity
class-side singleton/package-value identity
```

The T6 UtcOffset continuation is recorded by
`STDL002.C2.P1-T6-utc-offset-bound-incident.md` and the updated A1
walkthrough/handoff. Its focused evidence includes the inclusive extrema,
second-resolution historical-style value, total negation at both extrema,
ordering/equality/hash behavior, invalid boundary rejection, canonical
package publication, and source ownership in `time.offset`.

Do not implement a time-specific substitute for missing general-language architecture.

After P1 reaches its final gate, produce and finalize:

```text
STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md
```

Then begin `STDL002.C2.P2` by consuming those as-built records in P2 T0. Do not treat the already-authored P2 plan as permission to skip P1 reconciliation.

P2 closes C2 only after its final certification gate, walkthrough/handoff, and checkpoint update. Authoritative time-specification work remains `USER-DEFERRED`.
