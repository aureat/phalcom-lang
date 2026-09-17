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
repository: aureat/phalcom-lang
planning_baseline_revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
---

# Implementer Guidance — STDL002.C2 `time` Upgrade

## 0. Purpose

This file is the persistent architectural and execution guidance for all `STDL002.C2` planning and implementation sessions.

C2 has two plans:

```text
C2.P1
    temporal foundation + C1 migration
    absolute wall time
    civil Gregorian calendar
    fixed offsets
    package/prelude exposure

C2.P2
    IANA timezone backend
    zoned semantics
    current local conveniences
    parsing/formatting
    deterministic test clocks
    integration/specification closure
```

The checkpoint's core architectural sentence is:

> The `time` package is ordinary Phalcom source first: transparent immutable temporal values use `data`, identity/resource capabilities use classes, Rust supplies only irreducible host facts or an explicitly native timezone backend, visibility is lexical and identity-based, and Universe status never justifies spelling-based access, implicit privilege, temporal VM special cases, or a `System` date/time façade.

Every C2 patch must preserve that sentence.

This file is guidance, not a competing status log. Lifecycle truth belongs in `STDL002.C2-CHECKPOINT.md`. Exact tasks, files, tests, and gates belong in the active P1/P2 patch-grade plan.

---

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

The patch-plan schema requires each P1/P2 plan to separate fixed architecture, mechanical flexibility, verify-first assumptions, bounded testing, and STOP/CONSULT triggers.

## 1.2 Program/checkpoint authority

```text
docs/implementation/STDL002-standard-library-buildout/STDL002-PROGRAM.md
docs/implementation/STDL002-standard-library-buildout/C1-time/STDL002.C1-CHECKPOINT.md
docs/implementation/STDL002-standard-library-buildout/C1-time/STDL002.C1.P1-monotonic-clock-instant-duration.md
STDL002.C2-CHECKPOINT.md
STDL002.C2-GUIDANCE.md
active C2 plan
latest completed C2 walkthrough/handoff, if any
```

C1 is predecessor evidence, not current C2 architecture authority where C2 explicitly changes the surface.

## 1.3 Normative language authority

Read the live authoritative equivalents for:

```text
modules/imports/exports
visibility/access control
classes/inheritance/abstract declarations
data declarations/construction/invariants
enums/payload enums/pattern matching
native and primitive method declaration kinds
@intrinsic and @primitive metadata
intrinsic/native/primitive reflection/provenance
generic data
inherent impls/extensions
errors
reflection
```

The visibility and native/primitive areas are actively changing. Verify live documents and code rather than trusting legacy visibility decorators, implementation-selector spellings, decorator-based native declarations, or old primitive-registry documentation.

## 1.4 Time implementation

Inspect the live equivalents of:

```text
phalcom-core/core/universe/src/time/package.ph
phalcom-core/core/universe/src/time/clock.ph
phalcom-core/core/universe/src/package.ph
docs/spec/current/stdlib/time.md

System.clock / current monotonic native bridge
System.sleep(Duration)
Future.timeout(Duration)
reactor deadline conversion
semantic time capability tests
runtime/language-corpus time tests
reflection/native-surface/floor invariants
```

Do not re-audit unrelated standard-library packages.

---

# 2. Authority and drift rules

Use this order when authorities disagree:

```text
1. explicit user-ratified C2 decisions / approved checkpoint amendments
2. current authoritative language specs
3. STDL002.C2-CHECKPOINT.md durable invariants
4. this guidance
5. completed C2 walkthroughs/handoffs
6. C1 as-built behavior that C2 has not superseded
7. live implementation mechanics
8. active patch-grade plan mechanics
9. historical plans/drafts/legacy tests
```

For mechanical details:

```text
live repository
    >
planning-time helper names / paths / private signatures
```

Mechanical drift is adaptable.

Architectural drift in visibility identity, declaration implementation kind, primitive/native semantics, clock identity, temporal representation, timezone lifetime, semantic authority, or public API requires consultation.

Do not let a stale test re-ratify C1 surface that C2 intentionally supersedes.

---

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

The live C1 time implementation predates the intended final visibility and implementation-kind model.

C2 therefore treats these as **legacy takeover artifacts**:

```text
special Universe implementation-access attributes
implementation-only selector prefixes
the System-owned monotonic host-read seam
Clock's legacy hidden singleton accessor
Instant's legacy hidden construction/accessor selectors
```

They are not the target pattern.

Before editing them, verify the current language support for:

```text
private
protected
module
package
internal

native keyword
primitive keyword

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
- STOP AND CONSULT if C2 would duplicate or contradict the active language transition.

C2 is a production consumer of the new model, not its alternative implementation.

---

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

```text
source
    body implemented in Phalcom

native
    body supplied in Rust through Phalcom native APIs;
    behavior remains conceptually a library implementation

primitive
    irreducible host/runtime operation not expressible from available
    Phalcom-language values
```

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
select primitive linkage
change visibility
change identity
change representation
grant Universe privilege
force bytecode/intrinsic lowering
alter typechecking
```

If an operation uses the `primitive` implementation kind, the active plan may also require `@primitive` metadata for descriptive/reflection purposes if that is the current canonical Universe convention. The attribute is still not what makes the method primitive.

`@intrinsic` should not appear in time merely to encourage optimization. Use it only if a separately ratified intrinsic fact exists.

---

# 7. Primitive/native decision test

Before assigning either implementation kind, ask:

```text
Can this result be correctly derived in ordinary Phalcom from values and
language facilities already available to the program?
```

If **no** because the operation observes an external host/runtime fact, `primitive` may be appropriate.

If **yes**, source is the default.

If **yes**, but the operation intentionally delegates a substantial implementation to Rust/external data for engineering reasons, `native` may be appropriate.

Expected C2 classification:

| Operation | Classification |
|---|---|
| observe monotonic host clock | `primitive` |
| observe wall host clock | `primitive` |
| discover host configured timezone | `primitive` |
| open/query IANA/TZif rule database | `native` |
| Gregorian arithmetic | source |
| Duration/Timestamp arithmetic | source |
| fixed-offset conversion | source |
| local ambiguity policy | source |
| ZonedDateTime shift/advance | source |
| ISO parsing/formatting | source |

Performance does not convert source behavior into primitive behavior.

---

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
primitive host-clock ownership
visibility policy for time internals
```

P1 must not pre-implement named timezone semantics in ad hoc form.

In particular, do not make `UtcOffset` pretend to be `TimeZone`.

---

# 13. P2 architectural boundary

P2 builds only after the P1 value/capability model is stable.

P2 owns:

```text
TimeZone database lifetime/cache
TimeZone named/fixed/system/UTC forms
ZoneTransition
LocalTimeResolution
LocalTimeDisambiguation
ZonedDateTime
system-local current date/time conveniences
zone conversions
DST gap/overlap behavior
ISO/RFC parsing/formatting
testing clocks package
ordinary-package equivalence certification
reflection/provenance certification
final time specification
```

P2 must not reopen:

```text
Duration vs Period distinction
Clock identity law
Timestamp timeline representation
DateTime timezone-free meaning
UtcOffset fixed-value meaning
prelude policy
System façade removal
```

without checkpoint amendment.

---

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

Derived:

```text
offset
DateTime
Date
Time
civil components
```

Exact movement:

```text
shift(Duration)
    mutate timestamp coordinate logically
    preserve zone
```

Calendar movement:

```text
advance(Period)
    derive local DateTime
    advance in civil space
    resolve through zone
```

Zone conversion:

```text
withTimeZone(other)
    same Timestamp
    new TimeZone interpretation
```

Equality:

```text
same Timestamp + same TimeZone
```

Instant comparison is explicit:

```text
sameInstant
compareInstant
```

Do not make structural equality silently instant-only.

---

# 16. Local-time ambiguity

P2 must never assume every `DateTime + TimeZone` maps to one instant.

Resolution states:

```text
Unique
Ambiguous(earlier, later)
Gap(before, after)
```

The backend supplies factual rule results.

Phalcom source owns public policy.

A native timezone operation should return enough factual information for Phalcom to build the public enum. Do not bury `Earlier`/`Later`/`Compatible` policy inside Rust unless the source-level wrapper is purely forwarding an already-ratified policy contract.

Default guidance:

```text
DateTime.in(zone)
    Reject by default

ZonedDateTime.advance(Period)
    Compatible by default
```

The former is explicit conversion from an underspecified civil coordinate; the latter is calendar scheduling within an already-zoned value.

---

# 17. Timezone backend/lifetime guidance

Prefer a mature Rust backend such as Jiff behind a Phalcom-owned adapter, subject to P2's fresh dependency/API verification.

Public API must not expose Jiff types.

Preferred ownership:

```text
VM/process lifetime timezone database snapshot
    canonical database adapter
    cached named-zone handles
    TimeZone source objects store stable lightweight handle/identity
```

Requirements:

```text
database init lazy
no tzdb load at plain time import
existing TimeZone objects do not silently change rules mid-process
tests use controlled/pinned data or explicit fixtures where host tzdata drift matters
no per-TimeZone custom GC ownership if a stable VM-lifetime handle works
```

STOP AND CONSULT if the chosen backend forces materially different lifetime/identity semantics.

---

# 18. Textual forms

P2 should own one stable machine grammar, not a locale presentation framework.

Expected canonical families:

```text
Date             2026-09-17
Time             13:47:12.345678901
DateTime         2026-09-17T13:47:12.345678901
UtcOffset        +03:00
Timestamp        2026-09-17T10:47:12.345678901Z
ZonedDateTime    2026-09-17T13:47:12.345678901+03:00[Europe/Istanbul]
Period           ISO-style calendar form
Duration         ISO-style exact-time form without calendar-month/year semantics
```

A zoned string containing both explicit offset and zone identifier must be checked for consistency at that timestamp.

Parsing stays Phalcom-owned where expressible. The native timezone backend may validate rules but does not own the entire public parser grammar.

---

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

Runtime should see normal classes/data/enums and ordinary method dispatch except at explicit native/primitive call boundaries.

---

# 21. Language-feature certification intent

C2 intentionally serves as a production acceptance test.

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
module/package visibility
class-side singleton state
exported module values
grouped imports/exports
prelude canonical re-export
inherent implementations
arbitrary Int arithmetic
negative division/modulo
strings/parsing
errors
native methods
primitive methods
Universe-only metadata
reflection
GC
concurrency integration
```

When time exposes a defect in one of these general features:

```text
prefer fixing the owning language implementation
over hiding the problem in Rust or Universe privilege
```

If the required language fix collides with active architecture in another checkpoint, consult rather than patching around it.

---

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

Each plan should map tests to checkpoint invariants and select the dimensions it actually owns.

P1 should include at minimum:

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
primitive provenance/ownership
ordinary package consumption
concurrency Duration compatibility
reflection of migrated declarations
```

P2 should include at minimum:

```text
UTC/fixed/named zones
system timezone
historical offset with seconds if backend data provides one
transitionBefore/After
known DST gap
known DST overlap
Unique/Ambiguous/Gap payload semantics
disambiguation policies
same instant/different zones
shift(Duration) vs advance(Period)
Date.today zone dependence
parse/format round trips
invalid parse
offset-zone mismatch
lazy database initialization
zone handle lifetime/GC
ordinary package consumption
native provenance
final reflection surface
```

Property testing is particularly valuable for civil epoch-day round trips and parser/formatter round trips.

---

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
3. package/module visibility is being inferred from path/string rather than canonical identity;
4. `@primitive` or `@intrinsic` must become operative to complete a task;
5. primitive/native implementation-kind semantics conflict with this checkpoint;
6. a legacy spelling-based or special-attribute compatibility shim starts becoming permanent architecture;
7. abstract-member obligations require a competing class-completeness model;
8. `data` validation would require abandoning transparency or introducing a parallel constructor model;
9. Timestamp needs floating-point canonical storage;
10. DateTime would need an implicit timezone to make an API work;
11. Period would need exact-duration semantics;
12. UtcOffset is being used as a substitute for named timezone rules;
13. backend local resolution cannot preserve 0/1/2 factual mappings cleanly;
14. timezone handles create uncertain GC/root/lifetime behavior;
15. ZonedDateTime needs redundant public state for correctness;
16. a new temporal VM value/bytecode seems necessary;
17. a source-level time bug can only be bypassed by moving large semantics to Rust;
18. C2 conflicts materially with an active language checkpoint;
19. expected semantic test behavior would have to change;
20. checkpoint scope expands into timers/reactor redesign, general FFI redesign, or whole-Universe migration.

A triggered condition cannot be self-waived.

---

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
record start revision if useful
consume predecessor handoff
```

During a plan update the checkpoint only for durable events:

```text
stable new interface/invariant
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
consultations
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

Do not create a competing plan-specific checkpoint-state document.

---

# 30. P1 planning instructions

The very next planning artifact should be P1.

Before writing it, verify:

```text
current grouped import/export syntax
current visibility syntax/semantics
current native keyword syntax/registration
current primitive keyword syntax/registration
current @intrinsic/@primitive handling
current abstract-member status
current data constructor/validation model
current static/class-side singleton field syntax
current time consumers
current C1 tests/floor/reflection contracts
```

Then produce a Luna patch-grade plan using the canonical schema.

P1 tasks should be coherent vertical slices rather than one task per type.

A likely task dependency shape is:

```text
T0 takeover + live language-surface verification

T1 minimal language prerequisite closure
    only if live tree still lacks required general capability

T2 monotonic model migration
    Duration / Measurement / Clock / SystemClock / Instant / clock

T3 absolute time
    Timestamp / WallClock / SystemWallClock / wallClock

T4 civil primitives and algorithms
    Month / Weekday / Period / epoch-day foundation

T5 validated civil value layer
    Date / Time / DateTime / partial values / UtcOffset

T6 package/prelude/concurrency/ordinary-package integration

Gates
    focused semantics
    runtime/calendar properties
    integration/reflection
```

The final P1 plan may adapt the task grouping after live verification but must preserve the P1 ownership boundary.

---

# 31. P2 planning instructions

Do not write P2 until P1 has a completed walkthrough/handoff unless explicitly instructed otherwise.

P2 planning must consume the exact stable P1 interfaces instead of restating speculative versions.

A likely dependency shape is:

```text
timezone dependency/backend spike and ownership verification
    ↓
native TimeZoneDatabase adapter
    ↓
TimeZone + transitions
    ↓
LocalTimeResolution/Disambiguation
    ↓
ZonedDateTime conversions/arithmetic/current-time conveniences
    ↓
parsing/formatting
    ↓
testing clocks + ordinary-package/reflection/provenance closure
    ↓
final specification and checkpoint certification
```

The P2 plan must include exact timezone fixture strategy and backend-version verification rather than depending on ambient host tzdata for all behavioral assertions.

---

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
timezone backend is native, not the primitive floor
host observations are primitive
@intrinsic/@primitive metadata is non-operative
visibility is identity-based and lexical
no special implementation-selector naming in final C2 time source
no broad temporal trait hierarchy
no temporal VM specialization by default
```

These are architect decisions, not implementer convenience choices.

---

# 33. Current next action

Create `STDL002.C2.P1` only.

Do not implement C2 and do not draft P2 in the same planning turn unless explicitly requested.

P1 must be sufficiently patch-grade that the implementer can spend most of the session coding rather than rediscovering:

```text
the new visibility model
the native/primitive distinction
the C1 migration boundary
the data/class/enum representation choices
the module/prelude policy
the calendar laws
the intended testing budget
the exact conditions that require architectural consultation
```
