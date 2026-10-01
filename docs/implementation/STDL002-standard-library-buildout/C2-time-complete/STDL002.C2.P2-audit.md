# STDL002.C2 Post-Implementation Audit

## Executive conclusion

The repository state currently visible on GitHub does **not** contain a completed `STDL002.C2`, despite the premise that both plans were completed and pushed.

I audited the only visible branch, `main`, currently at:

```text
182559c4eea8bbba172558223d101bbcf4806a16
fix(core): trace runtime authority edges
```

Against that revision, the implementation is materially in this state:

| Area | Audit result |
|---|---|
| P1 temporal value/capability implementation | **Substantially implemented** |
| P1 package/prelude publication | **Implemented** |
| P1 ordinary-package integration | **Implemented** |
| P1 final T8 stabilization/certification | **Not closed** |
| P1 final walkthrough/handoff | **Missing** |
| P2 timezone implementation | **Absent** |
| P2 zoned semantics | **Absent** |
| P2 text parsing/formatting | **Absent** |
| `time.testing` | **Absent** |
| P2 walkthrough/handoff | **Absent** |
| timezone backend/Jiff dependency | **Absent** |
| VM timezone DB snapshot/cache | **Absent** |
| C2 checkpoint state | **`IN_PROGRESS / PARTIAL / UNVERIFIED`** |
| C2 native floor | **Incomplete** |
| C2 native surface | **Incomplete** |
| Existing P1 native floor ownership | **Architecturally good, but metadata has defects** |

There are also two concrete native-surface defects in the portion that did land:

1. both time-observation natives are incorrectly declared `effects = pure`, even though the native effect vocabulary explicitly supports `Nondeterminism`;
2. the LANG006/C2-required `@primitive` metadata classification is not present on the source declarations, and the current generated native surface does not preserve an equivalent primitive-classification field.

There is a third inventory defect: the canonical floor census still names the old `System._$monotonicNanoseconds` floor seam and does not reflect the landed `SystemClock`/`SystemWallClock` ownership.

So the concise answer to the second question is:

> **No. The complete C2 native floor has not landed, and the P2 native surface has not landed at all. The two P1 host observations are correctly narrow and correctly owned, but their metadata/inventory is not yet correct enough to certify the floor.**

---

# 1. Audit basis

I inspected the current GitHub repository rather than treating the checkpoint records as proof of completion.

The repository exposes only one branch:

```text
main
    182559c4eea8bbba172558223d101bbcf4806a16
```

The last commits visibly associated with the C2 time implementation are:

```text
e2030124  feat(stdlib): build out time foundation
f41ad082  fix(stdlib/time): close direct Period verification
3766f55  feat(stdlib/time): complete civil module blocker remediation
1fcb0bd  docs(stdlib/time): record UtcOffset bound incident
6d248cfe  feat(stdlib/time): implement bounded UtcOffset
b67fda93  feat(stdlib/time): close package and prelude publication
```

That final commit title is significant: **“close package and prelude publication” corresponds to P1 T7**, not P1 T8 or P2.

I also attempted to inspect GitHub combined status/Actions for the current HEAD. The connector lacks permission for commit-status APIs and returned HTTP 403, so I cannot independently claim current CI is green. Test evidence below therefore comes from the committed test suite and checkpoint records, not the GitHub Actions status.

---

# 2. The lifecycle records themselves say C2 is not complete

The current committed checkpoint is unambiguous.

At:

```text
docs/implementation/
  STDL002-standard-library-buildout/
    C2-time-complete/
      STDL002.C2-CHECKPOINT.md
```

the front matter is currently:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
```

Its final completion gate still correctly requires all of:

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

But immediately below that, its plan ledger says:

```text
P1
    Implementation status: IN_PROGRESS
    Completion: PARTIAL
    Verification: UNVERIFIED

P2
    Implementation status: PROPOSED
    Completion: NOT_STARTED
    Verification: REQUIREMENTS_ANALYZED
```

And the final lines of the committed checkpoint say:

> `T7 is complete; P1 remains active for its later T8 stabilization and final walkthrough/handoff. Do not mark P1 or C2 complete from this task alone.`

That is not merely stale metadata contradicted by a complete implementation elsewhere. The source tree agrees with it.

---

# 3. The live `time` package is a P1 package, not a P2 package

The current directory is:

```text
phalcom-core/core/universe/src/time/
├── calendar/
├── clock.ph
├── duration.ph
├── offset.ph
├── package.ph
└── timestamp.ph
```

The calendar subtree contains:

```text
date.ph
datetime.ph
month.ph
package.ph
partial.ph
period.ph
time.ph
weekday.ph
```

What is absent is more important:

```text
zone/
timezone.ph
zone_transition.ph
resolution.ph
zoned.ph

text/
iso8601.ph
parsing.ph

testing/
clocks.ph
```

There is no alternative implementation elsewhere either. Repository-wide searches for the actual P2 declarations found `TimeZone`, `ZonedDateTime`, `LocalTimeResolution`, `transitionBefore`, `TimeZone.named`, etc. only in plans, guidance, checkpoints, and stale historical documents—not executable `.ph` source.

---

# 4. The public package surface proves P2 did not land

Current `phalcom-core/core/universe/src/time/package.ph` is:

```phalcom
@!documentation("Time, duration, calendar, and monotonic clock utilities.")

expose .clock
expose .duration
expose .timestamp
expose .offset
expose .calendar

from universe.time.clock import (
  Clock,
  Instant,
  SystemClock
)

from universe.time.duration import (
  Duration,
  Measurement
)

from universe.time.timestamp import (
  Timestamp,
  WallClock,
  SystemWallClock,
  wallClock
)

from universe.time.offset import UtcOffset

from universe.time.calendar.month import Month
from universe.time.calendar.weekday import Weekday
from universe.time.calendar.period import Period

from universe.time.calendar.date import (
  Date,
  DateOverflow
)

from universe.time.calendar.time import Time
from universe.time.calendar.datetime import DateTime

from universe.time.calendar.partial import (
  YearMonth,
  MonthDay
)

let clock = SystemClock.instance

export (
  Clock,
  Instant,
  SystemClock,
  Duration,
  Measurement,
  Timestamp,
  WallClock,
  SystemWallClock,
  UtcOffset,
  Month,
  Weekday,
  Period,
  Date,
  DateOverflow,
  Time,
  DateTime,
  YearMonth,
  MonthDay,
  clock,
  wallClock
)
```

This is essentially the P1 target surface.

The following C2/P2-required exports do not exist:

```text
TimeZone
ZoneTransition
LocalTimeResolution
LocalTimeDisambiguation
ZonedDateTime
```

Nor are there public packages for:

```text
time.testing
time.zone
time.text
```

The prelude publication, however, is correct. The Universe root imports only:

```phalcom
from .time import (
  Duration,
  Timestamp
)
```

which matches the C2 invariant that only `Duration` and `Timestamp` are temporal prelude promotions.

So:

- **P1 publication architecture: good**
- **P2 publication: absent**

---

# 5. P2 itself is effectively not implemented

The gap is not a small missed selector. Almost the entire second plan is absent.

## 5.1 No timezone database backend

P2 required a mature backend, with Jiff as the first candidate.

Current root `Cargo.toml` has no Jiff dependency.

Current `phalcom-core/Cargo.toml` has no Jiff dependency.

There is no equivalent timezone crate dependency visible either.

There is no:

```text
TimeZoneDatabaseAdapter
ZoneHandle
tzdb state
named-zone cache
timezone snapshot/generation
```

in the runtime.

Therefore these P2 invariants cannot currently hold:

```text
lazy named tzdb initialization
stable VM/process tzdb snapshot
named-zone handle cache
existing TimeZone rule stability
backend interop range isolation
pinned timezone test data
```

They are simply not implemented yet.

---

## 5.2 No `TimeZone`

Required:

```phalcom
TimeZone.utc
TimeZone.fixed(offset)
TimeZone.named(identifier)
TimeZone.system
TimeZone.at(timestamp)
transitionBefore(timestamp)
transitionAfter(timestamp)
```

Live implementation: none.

---

## 5.3 No transitions

Required:

```phalcom
data ZoneTransition(
  timestamp: Timestamp,
  offsetBefore: UtcOffset,
  offsetAfter: UtcOffset
)
```

Live implementation: none.

---

## 5.4 No civil-local resolution

Required:

```phalcom
enum LocalTimeResolution {
  Unique(value: ZonedDateTime)
  Ambiguous(earlier: ZonedDateTime, later: ZonedDateTime)
  Gap(before: ZonedDateTime, after: ZonedDateTime)
}

enum LocalTimeDisambiguation {
  Reject
  Earlier
  Later
  Compatible
}
```

Live implementation: none.

Therefore none of the following exists:

```text
unique local mapping
DST fold
DST gap
Earlier
Later
Reject
Compatible
non-hour transition handling
Gap.before/Gap.after semantics
```

---

## 5.5 No `ZonedDateTime`

Required canonical state:

```text
Timestamp + TimeZone
```

Required operations:

```text
Timestamp.in(zone)

DateTime.resolve(zone)
DateTime.in(zone)
DateTime.in(zone, disambiguation)

ZonedDateTime.offset
ZonedDateTime.dateTime
ZonedDateTime.date
ZonedDateTime.time

shift(Duration)
advance(Period)
withTimeZone(zone)

sameInstant
compareInstant
```

Live implementation: none.

The particularly important law:

```text
Duration.hours(24)
    !=
Period.days(1)
```

across a DST boundary cannot currently be expressed because there is no zoned layer.

---

## 5.6 No current local-date conveniences

Required:

```text
Date.today
Date.today(in: zone)

ZonedDateTime.now
ZonedDateTime.now(in: zone)
```

Live implementation: absent.

---

# 6. P2 textual forms did not land

This is easy to verify from the actual P1 source.

The final P2 contract wanted:

```text
Date             2026-09-17
Time             13:47:12.345678901
DateTime         2026-09-17T13:47:12.345678901
UtcOffset        +03:00
Timestamp        2026-09-17T10:47:12.345678901Z
ZonedDateTime    2026-09-17T13:47:12.345678901+03:00[Europe/Istanbul]
```

Current `Timestamp.toString` is still:

```phalcom
toString -> String {
  "\(self.unixNanoseconds)ns"
}
```

Current `Date.toString` is:

```phalcom
toString -> String {
  "#{self.year}-#{self.month.number}-#{self.day}"
}
```

which does not zero-pad the month or day.

`Time` has no final machine formatter.

`DateTime` has no final machine formatter.

`UtcOffset` has no final machine formatter.

There are no corresponding `parse` entry points.

So the P2 parse/format requirement is unequivocally open.

That means the following are also absent:

```text
expanded-year grammar
1–9 digit fraction parsing
canonical nine-digit fractional output
offset seconds formatting
Timestamp numeric-offset parsing
zoned offset/zone consistency validation
Duration canonical machine parsing
mixed-sign Period roundtrip
parse(format(x)) properties
invalid grammar tests
```

---

# 7. `time.testing` did not land

P2 required deterministic testing capabilities conceptually equivalent to:

```phalcom
from time.testing import ManualClock, ManualWallClock
```

The live tree has no `time/testing/` package.

No live `ManualClock` implementation exists under the current time source.

No `ManualWallClock` exists.

This matters because P2's deterministic current-time/testing closure is not merely a convenience feature; it is one of the explicit final C2 acceptance dimensions.

---

# 8. P1 itself: what is good

The landed P1 work is much stronger than the lifecycle state might suggest. I found no indication that the implementation abandoned the architecture or shoved the library into Rust.

That part deserves credit technically.

## 8.1 `Duration` is source-owned transparent data

Current:

```phalcom
data Duration private(_ nanoseconds: Int)
```

with source factories for:

```text
nanoseconds
microseconds
milliseconds
seconds
minutes
hours
```

and source arithmetic.

There is no duration-specific VM representation or native arithmetic.

That matches the plan.

---

## 8.2 `Measurement<T>` is ordinary generic data

```phalcom
data Measurement<T>(_ value: T, _ duration: Duration)
```

No runtime special case.

Good.

---

## 8.3 Clock capability architecture is correct

Current shape:

```phalcom
class Clock {
  now -> Instant

  since(_ instant: Instant) -> Duration {
    self.now.since(instant)
  }

  measure<T>(_ action: () -> T) -> Measurement<T> {
    ...
  }

  protected instant(_ ticks: Int) -> Instant {
    Instant.from(self, ticks)
  }
}
```

Declaration-only `now` makes `Clock` abstract through the general abstract-obligation mechanism.

`Instant` stores:

```text
clock
ticks
```

and uses clock object identity as its monotonic domain.

Cross-domain comparison/difference rejects.

Cross-domain equality is false.

This is the intended architecture.

---

## 8.4 System clock singleton ownership is correct

```phalcom
class SystemClock is Clock {
  @class _instance

  @class
  instance -> SystemClock {
    if (_instance == None) {
      _instance = SystemClock.create()
    }
    _instance
  }

  ...
}
```

and package publication:

```phalcom
let clock = SystemClock.instance
```

The old `System.clock` façade does not remain the canonical path.

Good.

---

## 8.5 Absolute wall time is separate

`Timestamp` is:

```phalcom
data Timestamp private(_ unixNanoseconds: Int)
```

with signed arbitrary-precision integer canonical state.

`WallClock` is its own capability:

```phalcom
class WallClock {
  now -> Timestamp
}
```

and `SystemWallClock` owns the host read.

`Timestamp.now` correctly composes through the source-level canonical capability:

```phalcom
@class
now -> Timestamp { wallClock.now }
```

No second direct `Timestamp.now` host primitive exists.

This is exactly the intended ownership split.

---

## 8.6 Civil values remain source code

The current calendar code implements Gregorian rules in Phalcom source:

```text
Gregorian.isLeapYear
Gregorian.daysInMonth
Gregorian.daysFromCivil
Gregorian.civilFromDays
Gregorian.addMonthsYear
Gregorian.addMonthsNumber
```

`Date`, `Time`, `DateTime`, `Period`, `YearMonth`, `MonthDay`, and `UtcOffset` are all ordinary source declarations.

There is no Rust Gregorian implementation and no temporal opcode.

That is architecturally correct.

---

## 8.7 `UtcOffset` preserves historical second precision

Current bound:

```phalcom
if value < -93_599 or value > 93_599 {
    ...
}
```

giving:

```text
-25:59:59 .. +25:59:59
```

with whole-second state.

This matches the accepted interoperability consultation and preserves second resolution.

---

# 9. P1 still has an unfinished T8 closure

Although most P1 implementation exists, the plan itself has one remaining gate.

P1 T8 was specifically intended to certify things that can be missed even when the API works:

```text
System.sleep(Duration) compatibility
Future.timeout(Duration) compatibility
reactor deadline boundary

reflection categories
native ownership/provenance

no temporal opcode/value specialization
cheap import
no P2 dependency

P1 plan/API agreement
final walkthrough
final handoff
```

The current checkpoint explicitly stops after T7.

I found live ordinary-package tests:

```rust
fn time_p1_ordinary_package_uses_grouped_public_surface_and_temporal_prelude()
fn time_p1_ordinary_package_keeps_non_prelude_temporal_names_explicit()
fn time_p1_ordinary_package_cannot_cross_instant_or_clock_visibility_boundaries()
```

Those are good.

But the P1 plan called for final focused lanes such as:

```text
type_metadata_invariants::time_p1_*
native_surface_contracts::time_p1_*
observability_disassembly::time_p1_*
```

Repository search currently finds those names only in the plan, not implemented tests.

There are some generic/native time tests, so this is partly a **certification gap**, not necessarily a behavior gap. But it matters because T8 is the gate that proves the architecture really stayed ordinary.

---

# 10. The final P1 lifecycle artifacts are missing

The C2 directory contains an A1 blocker-remediation walkthrough and handoff.

It does **not** contain the final:

```text
STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md
```

And there is no:

```text
STDL002.C2.P2-walkthrough.md
STDL002.C2.P2-handoff.md
```

Therefore the P2 execution precondition itself was never satisfied in the committed lifecycle state.

This also means there is no durable as-built P1 contract for P2 to consume.

---

# 11. Native floor audit

This is where the implementation has both a strong design and important incompleteness.

## 11.1 What the complete C2 floor should contain

Under the revised LANG006/C2 model, the irreducible temporal host observations are conceptually:

| Host fact | Implementation kind | Primitive metadata |
|---|---|---|
| monotonic clock observation | Native | `@primitive` |
| wall-clock observation | Native | `@primitive` |
| host/system timezone discovery | Native | `@primitive` |

Everything else should remain source or ordinary Native library behavior.

That is an excellent floor design because it asks Rust only for facts Phalcom cannot derive.

---

## 11.2 What actually exists

Current:

```text
SystemClock.class::monotonicNanoseconds
SystemWallClock.class::unixNanoseconds
```

in:

```text
phalcom-core/src/primitive/time.rs
```

There is no system timezone discovery operation.

Therefore the full C2 floor is:

```text
required: 3 conceptual host observations
landed:   2
missing:  1
```

So the C2 native floor is **not complete**.

---

# 12. The two landed floor operations are otherwise narrow

The monotonic operation is:

```rust
pub fn system_clock_monotonic_nanoseconds(
    vm: &mut VM,
    _receiver: &Value,
    _args: &[Value],
) -> PhResult<Value> {
    let nanos = BigInt::from(vm.start_time.elapsed().as_nanos());
    Ok(normalize_bigint(nanos, &mut vm.heap))
}
```

This is exactly the right amount of host behavior.

It does not:

- build `Duration`;
- build `Instant`;
- choose a clock;
- perform arithmetic;
- impose policy.

Those remain source semantics.

Likewise wall time only returns the host fact:

```rust
let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
    Ok(duration) => ...,
    Err(error) => ...,
};
```

and source constructs:

```phalcom
Timestamp.nanoseconds(SystemWallClock.unixNanoseconds)
```

Again, good.

The native floor has not inflated into the library.

---

# 13. Native ownership is correct

The generated native surface correctly identifies:

```text
owner: SystemClock
side: Class
selector: monotonicNanoseconds
visibility: Internal
return: Int
```

and:

```text
owner: SystemWallClock
side: Class
selector: unixNanoseconds
visibility: Internal
return: Int
```

The tests explicitly assert that neither operation still belongs to `System`.

From:

```text
phalcom-core/tests/core/native/contracts.rs
```

the implementation checks that the old owner is absent.

This is an important successful migration from C1.

---

# 14. Native-surface defect: clock reads are marked `Pure`

This is the most important correctness defect I found in the landed P1 native metadata.

Both providers are declared:

```rust
effects = pure
```

The generated catalog consequently contains:

```rust
// SystemClock.monotonicNanoseconds
effects: ::phalcom_native_meta::EffectSpec::Pure,
```

and:

```rust
// SystemWallClock.unixNanoseconds
effects: ::phalcom_native_meta::EffectSpec::Pure,
```

at approximately:

```text
phalcom-native-surface/src/generated.rs
    11711
    11758
```

That is semantically wrong.

The same native metadata system explicitly has:

```rust
pub enum NativeEffect {
    Mutation,
    Io,
    Scheduling,
    Reflection,
    Nondeterminism,
    Blocking,
}
```

and the native macro recognizes:

```text
nondeterminism
```

A time observation is precisely nondeterministic in this sense:

```text
read clock
read clock again
result may differ
```

A pure call is ordinarily eligible to be treated as stable under stronger future analyses and optimizations.

For example, this transformation must **not** become valid:

```phalcom
const a = SystemWallClock.unixNanoseconds
const b = SystemWallClock.unixNanoseconds
```

into conceptually:

```phalcom
const a = SystemWallClock.unixNanoseconds
const b = a
```

The metadata must prevent that assumption.

The correct native metadata should conceptually be:

```rust
effects = [nondeterminism]
```

for both observations, assuming the current macro's canonical list syntax.

### Severity

**High.**

Today this may only affect effect reporting. But the semantic analyzer already imports native effect atoms, and future optimization/proving is exactly where incorrect `Pure` metadata becomes dangerous.

This should be corrected before the native surface is certified.

---

# 15. Native-surface defect: the required `@primitive` classification is absent

LANG006 deliberately separated:

```text
ImplementationKind
    Phalcom
    Native
    Foreign
```

from:

```text
@primitive
@intrinsic
```

metadata.

The current source declarations are:

```phalcom
@class
internal native monotonicNanoseconds -> Int
```

and:

```phalcom
@class
internal native unixNanoseconds -> Int
```

There is no `@primitive`.

Repository-wide search did not find authored `.ph` usages of `@primitive` at all.

That means the C2 rule:

```text
irreducible host observation
    Native
    + @primitive metadata
```

has not actually been represented on these declarations.

This distinction matters. The Rust attribute:

```rust
#[phalcom_native_macros::primitive(...)]
```

is **not** the Phalcom `@primitive` metadata.

Conflating those would reproduce exactly the LANG006 distinction we went to considerable effort to remove.

---

# 16. The generated native surface cannot currently certify that metadata distinction

`PrimitiveSurfaceSpec` currently contains:

```rust
pub struct PrimitiveSurfaceSpec {
    pub key: PrimitiveKey,
    pub visibility: NativeVisibility,
    pub stability: NativeStability,
    pub anchor: NativeAnchorPolicy,

    pub params: &'static ParameterTupleSpec,
    pub returns: &'static TypeExprSpec,
    pub callable: &'static CallableTypeSpec,

    pub raises: RaisesSpec,
    pub effects: EffectSpec,
    pub flow: ReturnFlowSpec,
    pub termination: TerminationSpec,

    ...

    pub intrinsic: Option<NativeIntrinsicId>,
    pub trust: NativeTrust,
    pub docs: Option<&'static str>,
    pub conceptual: Option<&'static str>,
}
```

There is no equivalent:

```rust
primitive: bool
```

or primitive-metadata category.

This does not necessarily mean native surface metadata must itself own source attributes—the source semantic product could remain the authority—but the final C2 provenance audit needs a way to establish:

```text
declaration implementation kind = Native
declaration has @primitive metadata = true
```

for the host observations.

Right now the source declarations do not have the metadata either, so the proof cannot be made from either side.

### Severity

**High architectural/provenance gap.**

It does not break clock execution, but it means the final LANG006/C2 native-boundary contract is not actually represented.

---

# 17. The floor census is stale

`../../../__unstructured__/spec/current/core/floor-census.md` still has, around line 553:

```text
| `_$monotonicNanoseconds` | static | `system_monotonic_nanoseconds` |
  internal VM-owned monotonic nanosecond source;
  the only STDL002.C1.P1 floor addition
```

This is stale in at least three ways.

First, canonical ownership is now:

```text
SystemClock.monotonicNanoseconds
```

not `System._$monotonicNanoseconds`.

Second, the current P1 has another irreducible time observation:

```text
SystemWallClock.unixNanoseconds
```

which is absent from this census.

Third, complete C2 is intended to add system timezone discovery.

So the native floor census does not describe the live temporal floor.

If this census is still the canonical floor inventory, that is a real closure defect—not merely documentation polish.

---

# 18. Native surface beyond the floor: completely missing for P2

The native floor and native surface must not be confused.

The P2 **non-floor Native surface** was supposed to provide external timezone-rule facts:

```text
named-zone lookup/open
offset at Timestamp
local DateTime resolution
transition before
transition after
canonical backend identity if required
```

Those are Native because a real TZif/IANA backend is appropriate, but they are not primitive host observations.

Live repository: **none exist**.

A useful expected-vs-live table is:

| Native temporal capability | Expected category | Live |
|---|---|---|
| monotonic host read | Native + primitive metadata | Native implementation exists; primitive metadata missing |
| wall-clock host read | Native + primitive metadata | Native implementation exists; primitive metadata missing |
| system timezone discovery | Native + primitive metadata | **Missing** |
| named timezone lookup | Native | **Missing** |
| named offset lookup | Native | **Missing** |
| civil-local mapping | Native | **Missing** |
| transition before | Native | **Missing** |
| transition after | Native | **Missing** |
| canonical zone identity, if backend needs it | Native | **Missing** |
| calendar arithmetic | Phalcom | correctly source-owned |
| fixed-offset arithmetic | Phalcom | correctly source-owned |
| ambiguity policy | Phalcom | not implemented because P2 absent |
| ZonedDateTime semantics | Phalcom | not implemented |
| text parser | Phalcom | not implemented |
| formatter | Phalcom | not implemented |

This is why I would not characterize the native surface as “nearly complete.” The P2 half of it is absent.

---

# 19. No timezone VM ownership means the critical lifetime problem was never solved

One of the most important architectural requirements in P2 was not merely “use Jiff.”

It was:

```text
VM/process
└── TimeZoneDatabaseAdapter
    ├── stable database snapshot/generation
    ├── cached named-zone handles
    └── backend zone state
```

with guarantees that:

```text
existing TimeZone objects do not silently change rules
plain import does not load tzdb
UTC/fixed do not load tzdb
named zones share cached state
GC cannot invalidate native handles
```

There is no corresponding state in the live VM.

Therefore no implementation evidence exists for:

- snapshot stability;
- native-handle lifetime;
- GC classification;
- cache ownership;
- lazy named-db load;
- backend generation identity.

These were some of the most subtle P2 requirements.

---

# 20. Reflection and provenance closure remains unfinished

C2 intentionally uses `time` as a production test of the ordinary language pipeline.

The final reflection surface should prove categories such as:

```text
Duration                  data
Measurement<T>            generic data
Clock                     abstract class
SystemClock               class
Instant                   class
Timestamp                 data
WallClock                 abstract class
SystemWallClock           class
Month                     enum
Weekday                   enum
Date                      data
Time                      data
DateTime                  data
YearMonth                 data
MonthDay                  data
Period                    data
DateOverflow              enum
UtcOffset                 data

TimeZone                  class
ZoneTransition            data
LocalTimeResolution       enum
LocalTimeDisambiguation   enum
ZonedDateTime             data
```

The current P1 code has generic reflection/census integration, but I did not find the P1 T8 time-specific category/provenance closure planned under:

```text
type_metadata_invariants::time_p1_*
native_surface_contracts::time_p1_*
observability_disassembly::time_p1_*
```

And P2's reflection types don't exist at all.

So final C2 reflection certification is necessarily incomplete.

---

# 21. Ordinary-package P1 integration is one of the strongest landed areas

This part is good.

The current runtime tests contain:

```rust
#[test]
fn time_p1_ordinary_package_uses_grouped_public_surface_and_temporal_prelude()
```

and:

```rust
#[test]
fn time_p1_ordinary_package_keeps_non_prelude_temporal_names_explicit()
```

plus access-control tests.

Those prove that P1 is not just accidentally working inside privileged Universe compilation.

The prelude test also explicitly excludes:

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
UtcOffset
```

from the bare prelude.

That matches the intended architecture exactly.

P2 needs an equivalent final fixture containing `TimeZone`, zoned values, resolution pattern matching, text parsing/formatting, etc. It does not exist.

---

# 22. Concurrency integration appears compatible, but P1 never certified it through T8

There is current runtime support for:

```text
System.sleep(Duration)
future.timeout(Duration)
```

`System.sleep`'s Rust seam explicitly documents that the Phalcom wrapper supplies the signed nanoseconds after public validation.

A current concurrency fixture uses:

```phalcom
const t2 = cs.future.timeout(Duration.milliseconds(500))
```

So the migrated Duration is in actual use.

This is evidence that the implementation probably satisfies the functional compatibility aspect of T8.

But the P1 checkpoint never executed/recorded its final T8 closure, so this remains:

> **implemented elsewhere and plausibly compatible, but not certified as P1 closure evidence.**

That distinction matters for lifecycle truth.

---

# 23. No temporal VM specialization found

This is another positive result.

I found no evidence of:

```text
Value::Duration
Value::Timestamp
Value::Date
Value::Instant
Value::TimeZone

MAKE_DATE
ADD_TIMESTAMP
DATE_ADVANCE
RESOLVE_TIMEZONE
```

or temporal runtime object variants.

The current P1 values compile through ordinary source/data/class/enum machinery.

That is exactly what C2 wanted.

P2, when it is actually implemented, should preserve this property.

---

# 24. Plan/source path bookkeeping also contains a small inconsistency

The current checkpoint says the installed P1 body is:

```text
STDL002.C2.P1-01-body.md
```

and earlier `...P1-plan-01-body.md` references are naming-only aliases.

But the live GitHub C2 directory actually contains:

```text
STDL002.C2.P1-plan-01-body.md
```

and not the stated installed `STDL002.C2.P1-01-body.md`.

This is not a semantic implementation defect, but it is another reason the lifecycle record is not yet trustworthy enough to call closed.

The final cleanup should choose one canonical filename and update the checkpoint/handoff references accordingly.

---

# 25. Findings by severity

## Blockers — prevent C2 completion

| ID | Finding | Consequence |
|---|---|---|
| B1 | P2 public source is absent | Core C2 feature layer missing |
| B2 | timezone backend/dependency absent | No named timezone semantics |
| B3 | VM/process tzdb snapshot/cache absent | Lifetime/snapshot invariants unimplemented |
| B4 | system timezone discovery primitive absent | C2 native floor incomplete |
| B5 | P2 native rule-query surface absent | TimeZone cannot be implemented as planned |
| B6 | LocalTimeResolution/disambiguation absent | DST gaps/folds unsupported |
| B7 | ZonedDateTime absent | zoned semantics unsupported |
| B8 | final parser/formatter absent | textual C2 scope unsupported |
| B9 | `time.testing` absent | deterministic testing surface missing |
| B10 | P1 T8/final handoff incomplete | P2 execution prerequisite not satisfied |
| B11 | P2 walkthrough/handoff absent | final C2 certification impossible |
| B12 | checkpoint remains PARTIAL/UNVERIFIED | lifecycle correctly says not complete |

---

## High-priority defects in landed P1

| ID | Finding | Impact |
|---|---|---|
| H1 | time host observations declared `effects = pure` | Incorrect semantic/effect contract |
| H2 | source host observations lack `@primitive` metadata | LANG006/C2 provenance contract incomplete |
| H3 | no current mechanism demonstrated for final Native + primitive-metadata inventory | Cannot certify final floor boundary |

---

## Medium closure/inventory issues

| ID | Finding |
|---|---|
| M1 | floor census still describes legacy `System._$monotonicNanoseconds` |
| M2 | floor census omits current wall-clock observation |
| M3 | final P1 reflection/provenance/disassembly gate not recorded |
| M4 | P1 body filename in checkpoint disagrees with live file |
| M5 | no current CI status available to this audit because GitHub status endpoint is forbidden to the connector |

---

# 26. Is the native floor minimal?

There are two different answers.

### P1 runtime behavior: largely yes

The two landed host operations are appropriately irreducible:

```text
monotonic observation
wall-clock observation
```

Everything derivable from them remains Phalcom source.

That is excellent boundary discipline.

There is no sign of:

```text
native Duration arithmetic
native Timestamp arithmetic
native Date arithmetic
native Gregorian arithmetic
native Period application
```

So **the floor has not become too large**.

### C2 overall: no

C2 requires one additional irreducible host fact:

```text
host/system timezone configuration discovery
```

It has not landed.

Therefore the floor is minimal but incomplete.

That distinction is important:

> The problem is not excessive native code. The problem is that the planned P2 native boundary was never built, and the two existing host reads have incorrect provenance/effect metadata.

---

# 27. Is the native surface complete?

No.

Not close enough to certify C2.

The current time-related generated native surface contains only the two P1 host-read records.

A complete C2 surface needs two layers:

```text
primitive-classified Native floor:
    monotonic clock observation
    wall-clock observation
    system timezone discovery

ordinary Native timezone backend:
    named zone resolve
    offset at timestamp
    local civil resolution
    transition before
    transition after
    stable backend identity/lifetime support
```

Only the first two rows exist.

---

# 28. What I would keep unchanged

A remediation pass should **not** redesign the landed P1 architecture.

The following choices look correct and should be retained:

```text
Duration = transparent exact nanosecond data
Measurement<T> = generic data

Clock = abstract source capability
Instant = opaque clock-domain value
SystemClock owns monotonic host observation
clock = canonical singleton

Timestamp = signed Int Unix/POSIX nanoseconds
WallClock = abstract source capability
SystemWallClock owns wall observation
wallClock = canonical singleton
Timestamp.now composes through wallClock

Gregorian arithmetic = Phalcom source
Date/Time/DateTime = source values
Period = months + days
UtcOffset = signed seconds

only Duration/Timestamp in prelude

no System date/time discovery façade

no temporal Value variants
no temporal opcodes
no broad temporal trait hierarchy
```

Those are sound.

---

# 29. Required remediation sequence

I would not patch this as random missing API methods. The repository is exactly at the P1→P2 boundary originally planned.

The proper sequence is:

```text
1. Finish P1 T8
   ├── concurrency compatibility audit
   ├── full reflection categories
   ├── Native/@primitive provenance
   ├── no temporal VM-specialization evidence
   ├── cheap-import evidence
   └── P1-CV-43..50 closure

2. Correct P1 native metadata
   ├── monotonic read: nondeterministic, not pure
   ├── wall read: nondeterministic, not pure
   └── attach/verify @primitive source metadata

3. Repair native-floor inventory
   ├── SystemClock ownership
   ├── SystemWallClock ownership
   └── remove legacy System._$ census row

4. Produce final P1 walkthrough/handoff

5. Execute P2 T0 against that handoff

6. Land backend architecture
   ├── Jiff/current backend dependency
   ├── stable VM snapshot
   ├── cache
   ├── bounded conversion
   └── pinned deterministic fixture strategy

7. Land native timezone boundary
   ├── system-zone host observation
   ├── named-zone lookup
   ├── offset lookup
   ├── local resolution
   └── transition queries

8. Land source timezone/zoned layer
   ├── TimeZone
   ├── ZoneTransition
   ├── LocalTimeResolution
   ├── LocalTimeDisambiguation
   └── ZonedDateTime

9. Land current-time conveniences

10. Land canonical formatter/parser

11. Land time.testing

12. Final ordinary-package/reflection/GC/provenance/perf audit

13. P2 walkthrough/handoff

14. Update checkpoint:
    COMPLETE / COMPLETE / VERIFIED
```

This is essentially the already-authored P2 plan. The live repository has not yet reached the point where a new architecture is necessary.

---

# 30. Suggested immediate native fixes

Before doing timezone work, I would fix the host-read contracts.

Conceptually:

```rust
#[phalcom_native_macros::primitive(
    SystemClock,
    "monotonicNanoseconds",
    params = [],
    returns = Int,
    types = "() -> Int",
    side = class,
    visibility = internal,
    effects = [nondeterminism]
)]
```

and equivalently:

```rust
#[phalcom_native_macros::primitive(
    SystemWallClock,
    "unixNanoseconds",
    params = [],
    returns = Int,
    types = "() -> Int",
    side = class,
    visibility = internal,
    effects = [nondeterminism]
)]
```

Then the corresponding source declarations should carry the ratified language metadata, using the canonical LANG006 authored form, conceptually:

```phalcom
@class
@primitive
internal native monotonicNanoseconds -> Int
```

and:

```phalcom
@class
@primitive
internal native unixNanoseconds -> Int
```

The important part is not decorator order in that illustration; the important invariant is:

```text
implementation kind = Native
metadata          = primitive
effects           = nondeterministic
```

Those are three separate axes.

Tests should then assert all three separately.

---

# 31. What the final native-boundary test should assert

I would make the final temporal native inventory test explicit rather than relying on ad-hoc searches.

Conceptually:

```text
SystemClock.monotonicNanoseconds
    implementation = Native
    primitive metadata = true
    effects contains Nondeterminism
    owner = SystemClock
    side = class
    visibility = Internal
    return = Int

SystemWallClock.unixNanoseconds
    implementation = Native
    primitive metadata = true
    effects contains Nondeterminism
    owner = SystemWallClock
    side = class
    visibility = Internal
    return = Int

<system timezone observation>
    implementation = Native
    primitive metadata = true
    effects contains appropriate host observation effect(s)

named-zone lookup
offset lookup
local-resolution lookup
transition-before
transition-after
    implementation = Native
    primitive metadata = false
```

And then negatively assert that these are **not** native:

```text
Duration arithmetic
Timestamp arithmetic
Gregorian arithmetic
Date.advance
Period arithmetic
UtcOffset negation
ambiguity policy
ZonedDateTime shift
ZonedDateTime advance
text parsing
canonical formatting
```

That would make the native floor auditable instead of inferential.

---

# 32. Final closure matrix

| C2 contract | Current live state | Result |
|---|---|---|
| Duration | implemented | PASS |
| Measurement | implemented | PASS |
| Clock/SystemClock | implemented | PASS |
| Instant domains | implemented | PASS |
| Timestamp | implemented | PASS |
| WallClock/SystemWallClock | implemented | PASS |
| Gregorian civil model | implemented | PASS |
| Period | implemented | PASS |
| UtcOffset | implemented | PASS |
| package/prelude policy | implemented | PASS |
| ordinary-package P1 | implemented | PASS |
| P1 T8 | not closed | FAIL |
| final P1 handoff | absent | FAIL |
| timezone DB | absent | FAIL |
| stable DB snapshot | absent | FAIL |
| TimeZone | absent | FAIL |
| transitions | absent | FAIL |
| local resolution | absent | FAIL |
| ZonedDateTime | absent | FAIL |
| system timezone | absent | FAIL |
| Date.today | absent | FAIL |
| zoned now | absent | FAIL |
| canonical text | absent | FAIL |
| parsing | absent | FAIL |
| offset/zone consistency | absent | FAIL |
| time.testing | absent | FAIL |
| zone-handle GC | absent | FAIL |
| P2 ordinary package | absent | FAIL |
| P2 reflection | absent | FAIL |
| P2 native provenance | absent | FAIL |
| P2 performance/lazy init | absent | FAIL |
| P2 walkthrough/handoff | absent | FAIL |
| checkpoint complete | no | FAIL |
| host floor minimality | good for landed P1 | PASS/PARTIAL |
| host floor completeness | missing system timezone | FAIL |
| native effects metadata | incorrectly Pure | FAIL |
| primitive classification metadata | absent | FAIL |
| floor census | stale | FAIL |

---

# 33. Final assessment

The landed code should be described as:

> **A substantial and mostly well-architected P1 implementation through T7, plus several general-language prerequisite fixes.**

It should **not** currently be described as:

> **Completed STDL002.C2 / completed P1+P2.**

The source tree, Cargo dependency graph, native catalog, lifecycle records, public package exports, and test artifacts all independently converge on the same conclusion.

The strongest aspects are P1's source-first architecture, correct capability ownership, canonical prelude design, ordinary-package integration, and refusal to specialize temporal values in the VM.

The critical gaps are not edge cases. **The complete timezone/zoned/text/testing half of C2—P2—is absent.**

And on the native question specifically:

```text
P1 native floor shape:
    architecturally narrow and correctly owned
    BUT effect/provenance metadata needs correction

C2 native floor:
    incomplete
    missing host timezone discovery

C2 native library surface:
    incomplete
    timezone backend operations entirely absent

native floor inventory:
    stale
    still documents legacy System ownership
```

So I would reopen C2 rather than mark it complete. The good news is that I do **not** see evidence that the architecture needs to be redesigned: the existing P2 plan remains the appropriate route from the current live state to genuine closure.