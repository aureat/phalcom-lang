# STDL002.C1 — Full `time` Library Design and Implementation Plan

## 1. Executive design

The Phalcom `time` library should be a substantial, source-owned Universe package implementing the complete foundational temporal model of the language. It should cover monotonic time, absolute wall time, civil Gregorian calendar values, fixed offsets, IANA time zones, zoned date-times, parsing, formatting, testing clocks, and the bridge between temporal values and concurrency.

The fundamental architectural principle is:

> Rust provides irreducible host observations and a practical native IANA timezone backend. Phalcom defines the temporal object model, arithmetic, policies, parsing, formatting, equality, calendar semantics, and public API.

The public model should not be organized around `System`. Temporal operations belong to the temporal abstractions themselves or to canonical values exported by the `time` package.

The intended style is therefore:

```phalcom
from time import clock

const started = clock.now

performWork()

System.print(started.elapsed)
```

rather than:

```phalcom
System.clock.now
```

For wall time:

```phalcom
const createdAt = Timestamp.now
```

rather than:

```phalcom
System.now
```

For civil and timezone-aware work:

```phalcom
from time import Date, Month, TimeZone, ZonedDateTime

const birthday = Date(
  year: 2002,
  month: Month::April,
  day: 14
)

const localNow = ZonedDateTime.now
const tokyo = localNow.withTimeZone(
  TimeZone.named("Asia/Tokyo")
)
```

The existing repository already has a real `time` Universe package and currently places `Clock`, `Instant`, and `Duration` in `time/clock.ph`; the package then re-exports them.  The Universe root currently promotes all three into its global export surface.  C1 should deliberately evolve that initial implementation into the architecture below.

---

# 2. Semantic taxonomy

The library should make the following distinctions fundamental.

| Concept | Meaning | Representation |
|---|---|---|
| `Duration` | exact elapsed amount measured in nanoseconds | `data` |
| `Clock` | abstract monotonic clock domain | abstract `class` |
| `SystemClock` | host monotonic clock implementation | `class` |
| `clock` | canonical process/VM monotonic clock | exported value |
| `Instant` | point in one specific monotonic clock domain | opaque `class` |
| `Timestamp` | absolute point on the POSIX/Unix wall-time timeline | `data` |
| `WallClock` | abstract source of absolute timestamps | abstract `class` |
| `SystemWallClock` | host wall-clock implementation | `class` |
| `wallClock` | canonical host wall clock | exported value |
| `Month` | Gregorian month | `enum` |
| `Weekday` | ISO weekday | `enum` |
| `Date` | Gregorian calendar date | `data` |
| `Time` | time of day without a date or zone | `data` |
| `DateTime` | timezone-free civil date and time | `data` |
| `YearMonth` | year + month partial civil value | `data` |
| `MonthDay` | month + day partial civil value | `data` |
| `Period` | calendar-relative months and days | `data` |
| `DateOverflow` | calendar overflow policy | `enum` |
| `UtcOffset` | fixed displacement from UTC | `data` |
| `TimeZone` | fixed or rule-based timezone | opaque `class` |
| `ZoneTransition` | timezone rule transition | `data` |
| `LocalTimeResolution` | result of mapping civil time into a timezone | payload `enum` |
| `LocalTimeDisambiguation` | ambiguity/gap resolution policy | `enum` |
| `ZonedDateTime` | `Timestamp` interpreted through a `TimeZone` | `data` |
| `Measurement<T>` | measured value plus elapsed duration | generic `data` |

The most important non-equivalences remain:

```text
Instant        != Timestamp

Duration       != Period

DateTime       != Timestamp

UtcOffset      != TimeZone

DateTime       != ZonedDateTime

24 exact hours != one calendar day

30 exact days  != one calendar month

same instant   != same ZonedDateTime value
```

These distinctions should be reflected in the type system instead of being encoded by flags on one enormous temporal type.

---

# 3. Package and module organization

The public package remains simply `time`. Its implementation should be divided into focused modules, but consumers normally import from the root package rather than depend on the implementation topology.

A good target layout is:

```text
phalcom-core/core/universe/src/time/
│
├── package.ph
│
├── duration.ph
│
├── clock.ph
│
├── timestamp.ph
│
├── current.ph
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

The submodules are implementation boundaries rather than separate conceptual libraries. The root `time/package.ph` should expose the complete supported public model.

The dependency direction should remain acyclic:

```text
Duration
   │
   ├───────────────┐
   ▼               ▼
Clock/Instant    Timestamp/WallClock
                   │
                   ▼
               Calendar
                   │
                   ▼
               Time zones
                   │
                   ▼
             Zoned semantics
                   │
                   ▼
             Current-time bridges
                   │
                   ▼
          Parsing / formatting
```

The apparent calendar/timezone cycle is avoided by putting timezone-dependent convenience behavior such as `Date.today`, `DateTime.resolve`, and `Timestamp.in` into higher-level modules using ordinary inherent implementations.

That is useful in its own right: C1 will exercise Phalcom's inherent-implementation machinery on real library types rather than only synthetic language tests.

---

# 4. Public root package

The final root package should export:

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

The backend implementation used for IANA zones is not exported.

The test clock types should live under:

```phalcom
from time.testing import ManualClock, ManualWallClock
```

rather than crowding the primary `time` namespace.

---

# 5. Prelude policy

The current Universe root exports `Clock`, `Instant`, and `Duration`. That should change.

The final temporal prelude should contain exactly:

```text
Duration
Timestamp
```

These two values are useful well outside code that thinks of itself as "date/time code."

`Duration` is consumed by concurrency, networking, retries, timeouts, caching, rate limiting, process APIs, and future scheduling facilities:

```phalcom
System.sleep(Duration.milliseconds(250))

future.timeout(Duration.seconds(5))
```

`Timestamp` is a natural general-purpose data-model type:

```phalcom
data LogEntry(
  timestamp: Timestamp,
  message: String
)

data User(
  createdAt: Timestamp,
  updatedAt: Timestamp
)
```

The canonical monotonic clock is deliberately **not** a prelude binding. A program that measures elapsed time explicitly opts into `time`:

```phalcom
from time import clock
```

Likewise, civil and timezone values require imports from `time`.

The root Universe still exposes the `time` package itself.

---

# 6. `Duration`

## 6.1 Representation

`Duration` should migrate from the current ordinary class representation to the new immutable transparent `data` model:

```phalcom
data Duration(
  nanoseconds: Int
) {
  ...
}
```

The current implementation already defines duration equality entirely by nanoseconds, derives hashing from the nanosecond count, and performs its arithmetic using `Int`.  `data` therefore expresses the existing semantics more directly.

`Duration` is signed.

Negative durations are valid mathematical values:

```phalcom
const difference = earlier.since(later)
```

APIs for which negative time is operationally meaningless—such as sleep—reject it at their own boundary rather than making `Duration` artificially unsigned.

## 6.2 Interface

```phalcom
data Duration(
  nanoseconds: Int
) {
  @class zero -> Duration

  @class nanoseconds(_ value: Int) -> Duration
  @class microseconds(_ value: Int) -> Duration
  @class milliseconds(_ value: Int) -> Duration
  @class seconds(_ value: Int) -> Duration
  @class minutes(_ value: Int) -> Duration
  @class hours(_ value: Int) -> Duration

  microseconds -> Number
  milliseconds -> Number
  seconds -> Number

  +(_ other: Duration) -> Duration
  -(_ other: Duration) -> Duration
  *(_ factor: Int) -> Duration

  negated -> Duration
  abs -> Duration

  compare(_ other: Duration) -> Ordering

  isZero -> Bool
  isNegative -> Bool

  @class parse(_ value: String) -> Duration
  toString -> String
}
```

The library should deliberately **not** initially expose:

```text
Duration.days(...)
Duration.weeks(...)
```

because those spellings are too easy to confuse with calendar units.

Exact twenty-four-hour time remains unambiguous:

```phalcom
Duration.hours(24)
```

Calendar days use:

```phalcom
Period.days(1)
```

## 6.3 No fractional construction policy yet

Factories taking temporal units should initially accept `Int`.

Do not silently decide how:

```phalcom
Duration.seconds(1.5)
```

rounds to nanoseconds.

Fractional construction can be added later with an explicit rounding model.

---

# 7. `Measurement<T>`

Timing an action should return a normal transparent product:

```phalcom
data Measurement<T>(
  value: T,
  duration: Duration
)
```

Usage:

```phalcom
from time import clock

const measurement = clock.measure {
  compileProject()
}

System.print(
  "compiled in \(measurement.duration)"
)

const result = measurement.value
```

There is no reason for this to be an opaque class.

---

# 8. Monotonic time: `Clock`, `SystemClock`, `Instant`, and `clock`

## 8.1 `Clock` is an abstract domain

`Clock` should become a genuinely abstract source class:

```phalcom
@abstract
class Clock {
  @abstract
  now -> Instant

  since(_ instant: Instant) -> Duration {
    now.since(instant)
  }

  measure<T>(_ action: () -> T) -> Measurement<T> {
    const start = now
    const value = action()

    Measurement(
      value: value,
      duration: now.since(start)
    )
  }

  @protected
  instant(_ ticks: Int) -> Instant {
    ...
  }
}
```

This requires completion of the abstract-member obligation semantics we identified during repository inspection:

```text
bodyless @abstract member
        ↓
inherited abstract obligation
        ↓
concrete override satisfies obligation
        ↓
concrete subclass with unresolved obligation is rejected
```

Class-level abstract allocation refusal already exists in the runtime. The time library should become a production consumer that forces the member-obligation half to become correct as well.

This should be treated as an early C1 language prerequisite, not circumvented with a fake throwing implementation.

## 8.2 `SystemClock`

The host implementation is an ordinary exported class:

```phalcom
class SystemClock is Clock {
  @class
  instance -> SystemClock {
    ...
  }

  now -> Instant {
    self.instant(
      self.monotonicNanoseconds
    )
  }

  @private
  primitive monotonicNanoseconds -> Int
}
```

The exact parser placement of `primitive` should follow the language work currently landing; the semantic requirement is what matters here.

`SystemClock` itself is public. It is not a secret runtime class and does not need an implementation-looking name.

The constructor used to establish the singleton may remain private so that:

```phalcom
SystemClock.instance
```

is the canonical host-domain object.

## 8.3 Package-owned `clock`

The module exports:

```phalcom
const clock: Clock =
  SystemClock.instance
```

Consumers normally use:

```phalcom
from time import clock

const start = clock.now
```

rather than:

```phalcom
SystemClock.instance.now
```

The latter remains perfectly intelligible and supported; it is simply not the expected everyday spelling.

The `clock` binding does not need lazy initialization. Creating one `SystemClock` object performs no time read and owns no expensive external resource. Eager package initialization therefore has negligible cost and establishes one stable domain identity for the process/VM lifetime.

If Phalcom later gains lazy exported values, using it here is an optimization rather than part of the semantic contract.

## 8.4 `Instant`

`Instant` remains an opaque class because its raw clock-relative tick representation should not become part of the ordinary data model.

```phalcom
class Instant {
  clock -> Clock

  elapsed -> Duration

  since(_ earlier: Instant) -> Duration

  plus(_ duration: Duration) -> Instant
  minus(_ duration: Duration) -> Instant

  compare(_ other: Instant) -> Ordering

  ==(_ other: Dynamic) -> Bool
  hash -> Int
}
```

Its fundamental law is:

```text
same Clock object + same tick coordinate
    => same Instant value

different Clock object
    => different monotonic domain
```

Cross-domain equality is false.

Cross-domain subtraction or ordering is an error:

```phalcom
const a = clockA.now
const b = clockB.now

a == b
// false

a.since(b)
// error: different clock domains
```

`Instant` has no `now` class method.

This remains intentionally invalid as an API concept:

```phalcom
Instant.now
```

An instant always belongs to a clock, so the appropriate expression is:

```phalcom
clock.now
```

## 8.5 Raw Instant construction

The implementation should continue to protect the integrity of clock domains. The final mechanism should allow a `Clock` subclass to create an `Instant` belonging to itself without forcing a public raw-tick API.

This is one small language-visibility detail to resolve while eliminating the older implementation-only selector scheme. The desired semantic relationship is:

```text
Clock subclass
    │
    └── protected instant(ticks)
               │
               ▼
            Instant
```

The time design should not regress into making raw monotonic coordinates an ordinary user-facing interchange format merely to simplify implementation.

---

# 9. Absolute time: `Timestamp`

## 9.1 Timeline

`Timestamp` represents an absolute point on a Unix/POSIX-compatible timeline measured in signed nanoseconds from:

```text
1970-01-01T00:00:00Z
```

Leap seconds are not represented.

Civil seconds therefore have the range:

```text
0...59
```

and:

```text
23:59:60
```

is not a valid `Time`.

This keeps the host clock, timezone database, timestamp arithmetic, and common RFC/ISO interchange model aligned.

## 9.2 Representation

```phalcom
data Timestamp(
  unixNanoseconds: Int
)
```

The use of `Int` preserves exact arithmetic and handles pre-epoch timestamps naturally.

For example:

```text
Timestamp(-1 ns)
    =
1969-12-31T23:59:59.999999999Z
```

Correct conversion of negative timestamps therefore requires Euclidean/floor division rather than truncating division.

## 9.3 Interface

```phalcom
data Timestamp(
  unixNanoseconds: Int
) {
  @class epoch -> Timestamp

  @class now -> Timestamp

  @class unixNanoseconds(_ value: Int) -> Timestamp
  @class unixMicroseconds(_ value: Int) -> Timestamp
  @class unixMilliseconds(_ value: Int) -> Timestamp
  @class unixSeconds(_ value: Int) -> Timestamp

  unixMicroseconds -> Number
  unixMilliseconds -> Number
  unixSeconds -> Number

  plus(_ duration: Duration) -> Timestamp
  minus(_ duration: Duration) -> Timestamp
  since(_ earlier: Timestamp) -> Duration

  compare(_ other: Timestamp) -> Ordering

  @class parse(_ text: String) -> Timestamp
  toString -> String
}
```

Timestamp arithmetic accepts `Duration`, never `Period`.

This is valid:

```phalcom
const expiry =
  Timestamp.now.plus(
    Duration.minutes(30)
  )
```

This is intentionally not:

```phalcom
Timestamp.now.plus(
  Period.months(1)
)
```

A month is not an absolute amount of elapsed time.

---

# 10. Wall clocks

## 10.1 `WallClock`

Absolute time acquisition should have the same testability discipline as monotonic time:

```phalcom
@abstract
class WallClock {
  @abstract
  now -> Timestamp
}
```

## 10.2 `SystemWallClock`

```phalcom
class SystemWallClock is WallClock {
  @class
  instance -> SystemWallClock {
    ...
  }

  now -> Timestamp {
    Timestamp.unixNanoseconds(
      self.unixNanoseconds
    )
  }

  @private
  primitive unixNanoseconds -> Int
}
```

The primitive is irreducible because Phalcom source cannot derive the host's current wall clock from existing language values.

## 10.3 Exported `wallClock`

The package may export:

```phalcom
const wallClock: WallClock =
  SystemWallClock.instance
```

`Timestamp.now` is simply the convenient public surface over this canonical capability:

```phalcom
Timestamp.now
```

Production code needing dependency injection can instead receive `WallClock`.

```phalcom
class AuditLog {
  @constructor
  new(clock: WallClock) {
    _clock = clock
  }

  record(_ message: String) {
    LogEntry(
      timestamp: _clock.now,
      message: message
    )
  }
}
```

Composition:

```phalcom
from time import wallClock

const log =
  AuditLog.new(clock: wallClock)
```

---

# 11. Civil calendar foundation

The civil layer should use the proleptic Gregorian calendar.

It must be implemented in ordinary Phalcom rather than by calling a Rust date library.

The core algorithms should be constant-time epoch-day conversions:

```text
daysFromCivil(year, month, day) -> Int

civilFromDays(epochDay)
    -> (year, month, day)
```

Correct handling of years before the epoch must use floor/Euclidean division.

This code becomes an excellent stress test of Phalcom's negative integer arithmetic.

---

# 12. `Month`

```phalcom
enum Month {
  January
  February
  March
  April
  May
  June
  July
  August
  September
  October
  November
  December
}
```

Useful behavior:

```phalcom
Month::January.number
// 1

Month::February.days(inYear: 2028)
// 29
```

Interface:

```phalcom
number -> Int

days(
  inYear: Int
) -> Int

@class
fromNumber(_ number: Int) -> Month
```

Invalid month numbers are rejected rather than wrapped.

---

# 13. `Weekday`

Use ISO weekday numbering:

```text
Monday    = 1
Tuesday   = 2
...
Sunday    = 7
```

Surface:

```phalcom
enum Weekday {
  Monday
  Tuesday
  Wednesday
  Thursday
  Friday
  Saturday
  Sunday
}
```

with:

```phalcom
number -> Int
```

Calendar computation derives `Weekday`; it is never redundantly stored in `Date`.

---

# 14. `Date`

## 14.1 Representation

```phalcom
data Date(
  year: Int,
  month: Month,
  day: Int
)
```

`Date` is exactly its three public civil components.

Derived values such as weekday and day-of-year are behavior, not stored fields.

## 14.2 Validation requirement

Transparent data must not imply invalid semantic products are accepted.

These must fail:

```phalcom
Date(
  year: 2026,
  month: Month::February,
  day: 31
)

Date(
  year: 2026,
  month: Month::April,
  day: 0
)
```

C1 must therefore certify that the current `data` construction model can express validated immutable values. If the language does not yet provide an adequate validation/construction hook, that is a language prerequisite exposed by C1—not a reason to make `Date` an opaque class.

## 14.3 Interface

```phalcom
data Date(
  year: Int,
  month: Month,
  day: Int
) {
  weekday -> Weekday
  dayOfYear -> Int
  daysInMonth -> Int
  isLeapYear -> Bool

  advance(
    _ period: Period,
    overflow: DateOverflow = DateOverflow::Constrain
  ) -> Date

  daysSince(_ earlier: Date) -> Int

  at(_ time: Time) -> DateTime

  compare(_ other: Date) -> Ordering

  @class parse(_ text: String) -> Date
  toString -> String
}
```

Higher-level timezone-aware modules add:

```phalcom
Date.today

Date.today(
  in: TimeZone
)
```

rather than introducing a dependency from the core calendar module back into the timezone layer.

---

# 15. Leap years and date validity

The Gregorian leap-year rule is standard:

```text
divisible by 4
AND
(not divisible by 100 OR divisible by 400)
```

Examples:

```text
2000 leap
1900 not leap
2024 leap
2025 not leap
```

`Date` arithmetic and parsing must share the same validation path so that separate parts of the library cannot disagree about legal dates.

---

# 16. `Time`

`Time` represents only a civil time-of-day.

```phalcom
data Time(
  hour: Int,
  minute: Int,
  second: Int,
  nanosecond: Int
)
```

Valid ranges:

```text
hour        0...23
minute      0...59
second      0...59
nanosecond  0...999_999_999
```

Interface:

```phalcom
@class midnight -> Time
@class noon -> Time

compare(_ other: Time) -> Ordering

@class parse(_ text: String) -> Time
toString -> String
```

Convenient construction should permit omitted lower components:

```phalcom
Time(
  hour: 9,
  minute: 30,
  second: 0,
  nanosecond: 0
)
```

There should be no:

```phalcom
Time.now
```

because a bare `Time` does not encode which timezone supplied that local clock reading.

The explicit spelling is:

```phalcom
ZonedDateTime.now.time
```

---

# 17. `DateTime`

`DateTime` is a timezone-free civil coordinate.

```phalcom
data DateTime(
  date: Date,
  time: Time
)
```

It should expose useful projections:

```phalcom
year
month
day
weekday

hour
minute
second
nanosecond
```

Core interface:

```phalcom
advance(
  _ period: Period,
  overflow: DateOverflow = DateOverflow::Constrain
) -> DateTime

compare(_ other: DateTime) -> Ordering

@class parse(_ text: String) -> DateTime
toString -> String
```

It deliberately does **not** support exact `Duration` arithmetic.

This is intentionally absent:

```phalcom
dateTime.plus(Duration.hours(5))
```

Without a timezone, there is no physical timeline on which that operation has an unambiguous temporal meaning.

The timezone layer adds:

```phalcom
dateTime.resolve(zone)

dateTime.in(
  zone,
  disambiguation: ...
)
```

There should likewise be no:

```phalcom
DateTime.now
```

The explicit projection is:

```phalcom
ZonedDateTime.now.dateTime
```

---

# 18. `Period`

`Period` represents calendar-relative movement.

Its canonical representation is:

```phalcom
data Period(
  months: Int,
  days: Int
)
```

Years normalize into months.

Weeks normalize into days.

Therefore:

```phalcom
Period.years(1)
==
Period.months(12)
```

and:

```phalcom
Period.weeks(1)
==
Period.days(7)
```

Interface:

```phalcom
data Period(
  months: Int,
  days: Int
) {
  @class zero -> Period

  @class years(_ value: Int) -> Period
  @class months(_ value: Int) -> Period
  @class weeks(_ value: Int) -> Period
  @class days(_ value: Int) -> Period

  +(_ other: Period) -> Period
  -(_ other: Period) -> Period
  *(_ factor: Int) -> Period

  negated -> Period
  isZero -> Bool

  @class parse(_ text: String) -> Period
  toString -> String
}
```

`Period` has no total ordering.

There is no generally meaningful answer to:

```text
1 month < 31 days ?
```

without choosing a particular calendar anchor.

---

# 19. Calendar application order

Applying:

```phalcom
date.advance(period)
```

uses a fixed order:

```text
1. apply the month component
2. resolve target day according to DateOverflow
3. apply the day component by epoch-day arithmetic
```

For example:

```phalcom
const date =
  Date(
    year: 2025,
    month: Month::January,
    day: 31
  )

date.advance(
  Period.months(1),
  overflow: DateOverflow::Constrain
)
// 2025-02-28
```

With:

```phalcom
overflow: DateOverflow::Reject
```

the same operation fails because February 31 is not representable.

This order must be normative.

---

# 20. `DateOverflow`

```phalcom
enum DateOverflow {
  Constrain
  Reject
}
```

`Constrain` maps an invalid target day to the final valid day of the target month.

`Reject` fails instead.

This policy appears only where month arithmetic can create invalid civil coordinates.

---

# 21. `YearMonth`

Some applications model a month without inventing a fake day.

```phalcom
data YearMonth(
  year: Int,
  month: Month
) {
  days -> Int
  isLeapYear -> Bool

  plusMonths(_ count: Int) -> YearMonth
  minusMonths(_ count: Int) -> YearMonth

  atDay(_ day: Int) -> Date

  @class parse(_ text: String) -> YearMonth
  toString -> String
}
```

Do not accept an arbitrary `Period` here because a day component cannot be meaningfully applied to a value that has no day.

---

# 22. `MonthDay`

Useful for birthdays, anniversaries, recurring dates, and partial data:

```phalcom
data MonthDay(
  month: Month,
  day: Int
) {
  inYear(_ year: Int) -> Date

  @class parse(_ text: String) -> MonthDay
  toString -> String
}
```

Validation must reject structurally impossible combinations such as April 31.

February 29 remains representable because it is valid in some years; `inYear` may fail for non-leap years.

---

# 23. `UtcOffset`

A fixed offset is not a timezone.

```phalcom
data UtcOffset(
  seconds: Int
)
```

The canonical state is one signed number of seconds.

Do not store independent hour/minute/second fields.

Interface:

```phalcom
@class utc -> UtcOffset

@class seconds(_ value: Int) -> UtcOffset

@class hours(_ value: Int) -> UtcOffset

negated -> UtcOffset

compare(_ other: UtcOffset) -> Ordering

@class parse(_ text: String) -> UtcOffset
toString -> String
```

Second precision should be supported because historical IANA offsets are not universally minute-aligned.

Typical rendering:

```text
Z
+03:00
-05:00
+00:19:32
```

---

# 24. Timezone backend architecture

Named IANA zones require a real rule database. This is the largest legitimate native portion of C1.

The implementation should use a Rust timezone backend—preferably Jiff behind a narrow Phalcom-owned adapter—but Jiff is never part of the public API.

Conceptually:

```text
Phalcom TimeZone
      │
      ▼
TimeZoneDatabase
      │
      ▼
native adapter
      │
      ▼
Jiff / IANA TZif data
```

The database object should be initialized lazily on first named-zone use.

Unlike `clock`, timezone initialization may involve filesystem discovery, database parsing, caches, and considerably more allocation, so laziness materially matters here.

The database snapshot should remain stable for the VM/process lifetime. Restarting Phalcom may observe a newer system timezone database, but existing `TimeZone` objects should never silently change rules while a program is running.

Production lookup policy:

```text
system timezone database where appropriate
        +
bundled/platform fallback where necessary
```

Tests should use a controlled/pinned rule set or explicit fixtures so that CI does not change behavior when a host updates tzdata.

---

# 25. `native` versus `primitive`

C1 should actively exercise the new distinction.

## 25.1 `primitive`

A `primitive` is an operation that cannot be derived from existing Phalcom-language information.

The C1 primitive floor should remain very small:

| Operation | Why primitive |
|---|---|
| host monotonic nanosecond read | no Phalcom value can derive the OS monotonic clock |
| host wall-clock nanosecond read | no Phalcom value can derive current wall time |
| host timezone identity/configuration discovery | platform environment state must be queried |

Conceptually:

```phalcom
class SystemClock is Clock {
  @private
  primitive monotonicNanoseconds -> Int
}
```

and:

```phalcom
class SystemWallClock is WallClock {
  @private
  primitive unixNanoseconds -> Int
}
```

## 25.2 `native`

Timezone rule execution is different.

Parsing TZif files, finding transitions, resolving local civil times, and mapping timestamps through IANA rules **could** theoretically be implemented in Phalcom. We choose a Rust/Jiff implementation because it is a mature, platform-sensitive database subsystem.

That makes it `native`, not part of the irreducible primitive floor.

Conceptually:

```phalcom
class TimeZoneDatabase {
  native open(_ identifier: String) -> Int

  native offsetAt(
    _ handle: Int,
    _ timestamp: Timestamp
  ) -> UtcOffset

  native resolve(
    _ handle: Int,
    _ dateTime: DateTime
  ) -> ZoneLookup

  native transitionBefore(
    _ handle: Int,
    _ timestamp: Timestamp
  ) -> Option<ZoneTransition>

  native transitionAfter(
    _ handle: Int,
    _ timestamp: Timestamp
  ) -> Option<ZoneTransition>

  primitive systemIdentifier -> String
}
```

`TimeZoneDatabase` does not need special implementation-looking selector names or a special visibility attribute. It can simply remain unexported from the public package.

This distinction gives C1 a useful architectural test:

```text
primitive = irreducible host fact

native = Rust implementation of semantics that remain conceptually
         part of the library and could be replaced later

Phalcom = everything else
```

---

# 26. `TimeZone`

`TimeZone` is an immutable but opaque class.

It should not be `data`, because its complete behavioral state includes database rule identity and native backend handles that are not part of the public structural value.

Surface:

```phalcom
class TimeZone {
  @class utc -> TimeZone

  @class fixed(
    _ offset: UtcOffset
  ) -> TimeZone

  @class named(
    _ identifier: String
  ) -> TimeZone

  @class system -> TimeZone

  identifier -> Option<String>

  isFixed -> Bool
  isNamed -> Bool

  offsetAt(
    _ timestamp: Timestamp
  ) -> UtcOffset

  transitionBefore(
    _ timestamp: Timestamp
  ) -> Option<ZoneTransition>

  transitionAfter(
    _ timestamp: Timestamp
  ) -> Option<ZoneTransition>

  ==(_ other: Dynamic) -> Bool
  hash -> Int

  toString -> String
}
```

Higher-level zoned modules add:

```phalcom
zone.at(timestamp)

zone.resolve(dateTime)
```

---

# 27. `TimeZone.system`

`TimeZone.system` queries the host's configured timezone and resolves it through the current VM's timezone database snapshot.

Usage:

```phalcom
from time import TimeZone

const zone = TimeZone.system
```

It should not silently replace an unknown system timezone with a fixed snapshot offset. Doing so loses future transitions and produces semantically false timezone behavior.

If the platform cannot identify the configured timezone reliably, the operation should report that inability through the canonical error model.

---

# 28. Fixed zones versus named zones

These are different:

```phalcom
TimeZone.fixed(
  UtcOffset.hours(3)
)
```

and:

```phalcom
TimeZone.named(
  "Europe/Istanbul"
)
```

They may currently produce the same offset but do not have the same semantics.

The named zone owns historical and future transition rules.

The fixed zone always remains `+03:00`.

---

# 29. `ZoneTransition`

Timezone transitions are useful for advanced scheduling, debugging, and inspection.

```phalcom
data ZoneTransition(
  timestamp: Timestamp,
  offsetBefore: UtcOffset,
  offsetAfter: UtcOffset
) {
  shift -> Duration
  isGap -> Bool
  isOverlap -> Bool
}
```

Derived values must not be stored redundantly.

For a spring-forward transition:

```text
offsetAfter > offsetBefore
    => gap
```

For a fall-back transition:

```text
offsetAfter < offsetBefore
    => overlap
```

---

# 30. Mapping local civil time into a timezone

Civil local times do not always identify exactly one instant.

A local `DateTime` can be:

```text
unique
ambiguous
nonexistent
```

The API should expose that reality instead of hiding it behind an arbitrary default.

```phalcom
enum LocalTimeResolution {
  Unique(
    value: ZonedDateTime
  )

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

`Ambiguous` values are ordered by absolute timestamp.

For a gap:

- `before` is the adjusted valid interpretation on the earlier side of the transition;
- `after` is the adjusted valid interpretation on the later side.

Those semantics must be precisely tested around real DST transitions.

Usage:

```phalcom
const requested =
  DateTime(
    date: date,
    time: time
  )

match requested.resolve(zone) {
  LocalTimeResolution::Unique(value) {
    schedule(value)
  }

  LocalTimeResolution::Ambiguous(
    earlier: earlier,
    later: later
  ) {
    chooseBetween(earlier, later)
  }

  LocalTimeResolution::Gap(
    before: before,
    after: after
  ) {
    reportInvalidLocalTime(
      before,
      after
    )
  }
}
```

This is an excellent real-world test of payload enums and pattern matching.

---

# 31. Disambiguation policy

For callers that want policy rather than inspection:

```phalcom
enum LocalTimeDisambiguation {
  Reject
  Earlier
  Later
  Compatible
}
```

Semantics:

| Situation | `Reject` | `Earlier` | `Later` | `Compatible` |
|---|---|---|---|---|
| unique | value | value | value | value |
| overlap | error | earlier | later | earlier |
| gap | error | before | after | after |

This matches the practical meaning of "compatible": preserve the conventional earlier interpretation for repeated times and move forward through skipped times.

---

# 32. `ZonedDateTime`

## 32.1 Canonical state

A zoned date-time should have exactly two canonical fields:

```phalcom
data ZonedDateTime(
  timestamp: Timestamp,
  timeZone: TimeZone
)
```

That expresses its semantics cleanly:

> An absolute instant interpreted through a timezone.

Do not store redundant canonical fields such as:

```text
timestamp
timezone
offset
date
time
```

because inconsistent combinations would become structurally representable.

## 32.2 Interface

```phalcom
data ZonedDateTime(
  timestamp: Timestamp,
  timeZone: TimeZone
) {
  offset -> UtcOffset

  dateTime -> DateTime
  date -> Date
  time -> Time

  year -> Int
  month -> Month
  day -> Int
  weekday -> Weekday

  hour -> Int
  minute -> Int
  second -> Int
  nanosecond -> Int

  shift(
    _ duration: Duration
  ) -> ZonedDateTime

  advance(
    _ period: Period,
    overflow:
      DateOverflow = DateOverflow::Constrain,
    disambiguation:
      LocalTimeDisambiguation =
        LocalTimeDisambiguation::Compatible
  ) -> ZonedDateTime

  withTimeZone(
    _ zone: TimeZone
  ) -> ZonedDateTime

  sameInstant(
    _ other: ZonedDateTime
  ) -> Bool

  compareInstant(
    _ other: ZonedDateTime
  ) -> Ordering

  @class now -> ZonedDateTime

  @class now(
    in: TimeZone
  ) -> ZonedDateTime

  @class parse(
    _ text: String
  ) -> ZonedDateTime

  toString -> String
}
```

---

# 33. Exact movement versus calendar movement

This distinction is one of the most important APIs in the library.

## `shift(Duration)`

Moves on the absolute timeline:

```phalcom
meeting.shift(
  Duration.hours(24)
)
```

means exactly:

```text
timestamp + 86,400 seconds
```

The resulting local wall clock may change across DST.

## `advance(Period)`

Moves in civil calendar space:

```phalcom
meeting.advance(
  Period.days(1)
)
```

means:

> Same local calendar rule one day later, then resolve that civil value through the timezone.

A daily meeting should therefore normally use:

```phalcom
nextMeeting =
  meeting.advance(
    Period.days(1)
  )
```

not:

```phalcom
meeting.shift(
  Duration.hours(24)
)
```

This distinction is encoded in the types rather than documentation alone.

---

# 34. Zoned equality

`ZonedDateTime` structural equality should mean:

```text
same Timestamp
AND
same TimeZone
```

Therefore:

```phalcom
istanbul == newYork
```

can be false even when:

```phalcom
istanbul.sameInstant(newYork)
```

is true.

Example:

```phalcom
const instant = Timestamp.now

const istanbul =
  instant.in(
    TimeZone.named("Europe/Istanbul")
  )

const newYork =
  instant.in(
    TimeZone.named("America/New_York")
  )

istanbul.sameInstant(newYork)
// true

istanbul == newYork
// false
```

Ordinary total comparison should not be exposed on `ZonedDateTime` if equality remains structural. Use the explicit:

```phalcom
compareInstant
```

when absolute-time ordering is intended.

---

# 35. Conversion convenience APIs

Once the zone layer is loaded, the following should be available as ordinary inherent behavior:

```phalcom
Timestamp.in(
  _ zone: TimeZone
) -> ZonedDateTime
```

```phalcom
TimeZone.at(
  _ timestamp: Timestamp
) -> ZonedDateTime
```

```phalcom
DateTime.resolve(
  _ zone: TimeZone
) -> LocalTimeResolution
```

```phalcom
DateTime.in(
  _ zone: TimeZone,
  disambiguation:
    LocalTimeDisambiguation =
      LocalTimeDisambiguation::Reject
) -> ZonedDateTime
```

The default for converting a naked `DateTime` should be `Reject`, because the caller has explicitly supplied an ambiguous civil coordinate and should not silently acquire policy.

By contrast, `ZonedDateTime.advance(Period)` defaults to `Compatible`, because recurring civil schedules need useful calendar behavior and already have a timezone context.

That asymmetry is deliberate.

---

# 36. Current time convenience

The complete convenience surface should be:

```text
Timestamp.now

ZonedDateTime.now
ZonedDateTime.now(in: zone)

Date.today
Date.today(in: zone)

clock.now
wallClock.now
```

It should deliberately **not** include:

```text
Instant.now
Time.now
DateTime.now
```

because those expressions erase context that is semantically required to define "now."

## `Date.today`

Conceptually:

```phalcom
Date.today
```

means:

```text
Timestamp.now
    ↓
TimeZone.system
    ↓
local ZonedDateTime
    ↓
date
```

Explicit zone:

```phalcom
Date.today(
  in: TimeZone.named("Asia/Tokyo")
)
```

---

# 37. Parsing and canonical formatting

C1 should include machine-oriented parsing and formatting.

It should **not** include locale-aware presentation.

Canonical forms:

| Type | Example |
|---|---|
| `Date` | `2026-09-17` |
| `Time` | `13:47:12.345678901` |
| `DateTime` | `2026-09-17T13:47:12.345678901` |
| `UtcOffset` | `+03:00` |
| `Timestamp` | `2026-09-17T10:47:12.345678901Z` |
| `ZonedDateTime` | `2026-09-17T13:47:12.345678901+03:00[Europe/Istanbul]` |
| `Period` | `P1Y2M3D` |
| `Duration` | `PT1H30M` |

For `Duration`, calendar day/month/year syntax should not be accepted. Exact 24-hour duration can render as:

```text
PT24H
```

while:

```text
P1D
```

belongs to `Period`.

Parsing should be implemented in Phalcom rather than delegated wholesale to Jiff or another Rust library.

That ensures the accepted grammar is Phalcom's stable contract rather than whatever an external dependency happens to accept.

---

# 38. Zoned parsing consistency

A zoned serialized value can contain both an offset and a named zone:

```text
2026-09-17T13:47:12+03:00[Europe/Istanbul]
```

The parser must validate that the supplied offset is actually valid for that timezone at the parsed timestamp.

It must not silently discard one half and trust the other.

This catches stale, corrupted, or inconsistent serialized temporal values.

---

# 39. Formatting scope boundary

C1 includes:

```text
ISO/ISO-like canonical machine strings
RFC-compatible Timestamp interchange
round-trip parsing
stable toString
```

C1 does not include:

```text
locale-sensitive month names
localized weekday names
CLDR
"September 17, 2026"
"17 Eylül 2026"
relative presentation like "3 hours ago"
natural-language parsing
custom strftime-style formatting DSL
```

Those belong to a later internationalization/presentation library.

---

# 40. Traits

C1 should introduce **no new time-specific public trait hierarchy**.

Do not create:

```text
Temporal
TemporalPoint
TemporalAmount
DateLike
TimeLike
ZoneLike
ClockLike
```

The absence of those abstractions is intentional.

`Duration` and `Period` both support arithmetic but are semantically incompatible.

`Instant`, `Timestamp`, `Date`, `DateTime`, and `ZonedDateTime` are all "points" in colloquial language but inhabit different coordinate systems.

The concrete types preserve useful information that an umbrella trait would erase.

Use ordinary general-purpose equality, hashing, comparison, reflection, and standard-library contracts where applicable.

Use abstract classes specifically where nominal capability inheritance is meaningful:

```text
Clock
WallClock
```

---

# 41. No special VM temporal values

Do not add:

```text
Value::Duration
Value::Timestamp
Value::Date
Value::Instant
Value::TimeZone
```

or equivalent specialized tags merely because these are standard-library types.

The current monotonic implementation already deliberately uses ordinary Phalcom objects rather than new VM value variants.

That philosophy should continue.

C1 is meant partly to test normal Phalcom execution:

```text
ordinary data allocation
ordinary class allocation
ordinary dispatch
ordinary fields
ordinary inheritance
ordinary enums
ordinary generic data
ordinary GC
ordinary reflection
ordinary bytecode
```

Optimization should come later from profiling.

---

# 42. No temporal bytecodes

Do not introduce bytecodes such as:

```text
MAKE_DATE
ADD_TIMESTAMP
ADVANCE_PERIOD
RESOLVE_TIMEZONE
MAKE_DURATION
```

All of these are ordinary messages and arithmetic.

The VM should not ask whether a receiver is a "standard-library temporal value" before dispatch.

---

# 43. Calendar algorithms stay in Phalcom

The following should be source-level implementations:

```text
leap-year calculation
days-in-month
epoch-day conversion
weekday
day-of-year
Date + Period
DateTime + Period
Duration arithmetic
Timestamp + Duration
UtcOffset arithmetic
fixed-offset timestamp conversion
ZonedDateTime shift/advance policy
disambiguation policy
ISO parsing
ISO formatting
```

If one of these reveals a compiler, semantic-analysis, integer, `data`, enum, or runtime defect, fix Phalcom rather than immediately moving the operation to Rust.

---

# 44. Testing clocks

A dedicated `time.testing` package should provide deterministic clocks.

## `ManualClock`

```phalcom
class ManualClock is Clock {
  @constructor
  new(
    ticks: Int = 0
  ) {
    ...
  }

  now -> Instant {
    ...
  }

  advance(
    _ duration: Duration
  ) -> ManualClock {
    ...
  }
}
```

Usage:

```phalcom
from time.testing import ManualClock

const clock =
  ManualClock.new()

const start = clock.now

clock.advance(
  Duration.seconds(10)
)

start.elapsed
// 10 seconds
```

Because `Instant.elapsed` consults the clock that produced it, deterministic testing remains natural.

## `ManualWallClock`

```phalcom
class ManualWallClock is WallClock {
  @constructor
  new(
    timestamp: Timestamp
  ) {
    ...
  }

  now -> Timestamp

  set(
    _ timestamp: Timestamp
  ) -> ManualWallClock

  advance(
    _ duration: Duration
  ) -> ManualWallClock
}
```

This enables deterministic audit timestamps, token expiry tests, cache tests, and other wall-time behavior.

---

# 45. Practical usage patterns

## 45.1 Measuring elapsed work

```phalcom
from time import clock

const start = clock.now

compileProject()

System.print(
  start.elapsed
)
```

Or:

```phalcom
const measured =
  clock.measure {
    compileProject()
  }

System.print(
  measured.duration
)
```

## 45.2 Exact expiry

```phalcom
const issuedAt =
  Timestamp.now

const expiresAt =
  issuedAt.plus(
    Duration.minutes(30)
  )
```

No timezone is involved.

## 45.3 Calendar due date

```phalcom
from time import Date, Period

const due =
  invoiceDate.advance(
    Period.days(30)
  )
```

This is intentionally different from thirty exact 24-hour intervals.

## 45.4 Civil date construction

```phalcom
from time import Date, Month

const release =
  Date(
    year: 2027,
    month: Month::March,
    day: 15
  )
```

## 45.5 Today's date

```phalcom
from time import Date

const today =
  Date.today
```

## 45.6 Today's date somewhere else

```phalcom
from time import Date, TimeZone

const todayInTokyo =
  Date.today(
    in:
      TimeZone.named("Asia/Tokyo")
  )
```

## 45.7 Same event in two zones

```phalcom
from time import TimeZone

const event =
  Timestamp.now

const istanbul =
  event.in(
    TimeZone.named("Europe/Istanbul")
  )

const newYork =
  event.in(
    TimeZone.named("America/New_York")
  )

istanbul.sameInstant(newYork)
// true
```

## 45.8 Daily 09:00 recurrence

```phalcom
const next =
  meeting.advance(
    Period.days(1)
  )
```

This preserves civil scheduling semantics across a DST transition.

## 45.9 Exact 24-hour delay

```phalcom
const next =
  meeting.shift(
    Duration.hours(24)
  )
```

This preserves elapsed time instead.

## 45.10 Explicit ambiguity handling

```phalcom
match requested.resolve(zone) {
  LocalTimeResolution::Unique(value) {
    submit(value)
  }

  LocalTimeResolution::Ambiguous(
    earlier: earlier,
    later: later
  ) {
    askUser(earlier, later)
  }

  LocalTimeResolution::Gap(
    before: before,
    after: after
  ) {
    explainGap(before, after)
  }
}
```

---

# 46. Error policy

C1 should reuse the canonical Phalcom error taxonomy where possible rather than invent a large temporal hierarchy.

The library must nevertheless distinguish the following semantic failure classes in diagnostics:

```text
invalid Date components
invalid Time components
invalid UtcOffset
different Clock domains
unknown timezone identifier
system timezone unavailable
ambiguous local time rejected
nonexistent local time rejected
out-of-range timezone backend conversion
invalid temporal text
inconsistent zoned string offset/zone pair
```

The exact existing `Error` subclasses should be audited during implementation and reused where they accurately describe the failure.

Only introduce a dedicated temporal error type where the existing hierarchy cannot express a stable public distinction.

---

# 47. Immutability

All temporal value types are immutable.

That includes:

```text
Duration
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
Measurement
```

`TimeZone` is also behaviorally immutable even though it remains an opaque class.

Clock implementations may be mutable where that is part of their semantics; `ManualClock` is the obvious example.

---

# 48. Timezone caching and object identity

Named `TimeZone` lookup should cache/intern backend rule handles.

A practical architecture is:

```text
TimeZoneDatabase
    database snapshot

    canonical name -> native handle

native handle
    -> immutable zone rules
```

A `TimeZone` object can hold:

```text
kind
canonical identifier
native handle
```

without exposing those fields publicly.

The handle indexes VM-owned immutable backend state and can live for the VM lifetime. This avoids introducing custom GC-managed Rust timezone objects.

Fixed-offset zones need no IANA handle.

---

# 49. Derived `ZonedDateTime` projection caching

The semantic identity of `ZonedDateTime` remains:

```text
Timestamp + TimeZone
```

If profiling later shows repeated calls such as:

```phalcom
value.year
value.month
value.day
value.hour
```

perform too many rule lookups, an immutable implementation may cache the derived `UtcOffset` and/or `DateTime`.

Those caches are implementation details.

They do not become equality fields.

Do not complicate the initial representation until measurements justify it.

---

# 50. C1 implementation sequence

The checkpoint should remain one `time` checkpoint but land through six coherent plan slices.

## STDL002.C1.P1 — monotonic foundation modernization

The existing P1 implementation becomes the basis rather than being discarded.

This slice should:

```text
migrate Duration to data
add Measurement<T>
complete abstract-member obligation support
make Clock abstract
add SystemClock
move host monotonic read onto SystemClock as primitive
add package-exported clock
remove public temporal discovery through System
remove Clock and Instant from prelude/root promotion
retain Duration in prelude
preserve System.sleep(Duration)
preserve Future.timeout(Duration)
certify ManualClock inheritance
```

Existing P1 already uses ordinary Phalcom classes and one tiny host floor rather than specialized runtime values.  The modernization should preserve that architecture while adopting the newly ratified language surface.

## STDL002.C1.P2 — absolute wall time

Implement:

```text
Timestamp
WallClock
SystemWallClock
wallClock
Timestamp.now
host wall-time primitive
negative/pre-epoch conversion properties
Timestamp arithmetic
```

At this point add `Timestamp` to the prelude.

No timezone dependency is required.

## STDL002.C1.P3 — Gregorian civil calendar

Implement:

```text
Month
Weekday
Date
Time
DateTime
YearMonth
MonthDay
Period
DateOverflow

epoch-day algorithms
calendar validation
calendar arithmetic
negative-year/pre-epoch property testing
validated data construction
```

This slice should remain pure Phalcom.

## STDL002.C1.P4 — timezone infrastructure

Implement:

```text
UtcOffset
TimeZone
TimeZoneDatabase
ZoneTransition

Jiff-backed native adapter
system timezone primitive
named/fixed/UTC zones
timezone handle caching
offset lookup
transition lookup
database snapshot policy
```

No full `ZonedDateTime` arithmetic yet.

## STDL002.C1.P5 — zoned semantics

Implement:

```text
ZonedDateTime
LocalTimeResolution
LocalTimeDisambiguation

Timestamp.in
TimeZone.at
TimeZone.resolve
DateTime.resolve
DateTime.in

shift(Duration)
advance(Period)
Date.today
ZonedDateTime.now
DST gap/overlap behavior
```

This is the central semantic integration slice.

## STDL002.C1.P6 — textual/completion layer

Implement:

```text
ISO/RFC parsing
canonical toString forms
Duration textual form
Period textual form
Timestamp serialization
ZonedDateTime serialization
round-trip properties

full reflection coverage
ordinary-package consumer tests
prelude/export tests
native/primitive floor audit
final comprehensive time specification
```

P6 closes C1.

---

# 51. Language-feature certification role

C1 should explicitly function as a production acceptance test of Phalcom's current language.

Each slice should certify language functionality as well as temporal functionality.

| Feature | C1 exercise |
|---|---|
| `data` | Duration, Timestamp, Date, Time, DateTime, Period, ZonedDateTime |
| validated `data` | Date, Time, UtcOffset, MonthDay |
| data behavior | arithmetic and derived properties |
| nested data | DateTime, ZonedDateTime |
| generic data | Measurement<T> |
| enums | Month, Weekday, DateOverflow |
| payload enums | LocalTimeResolution |
| abstract classes | Clock, WallClock |
| abstract methods | `Clock.now`, `WallClock.now` |
| inheritance | SystemClock, testing clocks |
| protected behavior | Clock Instant factory |
| class-side state | SystemClock.instance |
| exported values | `clock`, `wallClock` |
| module/package exports | complete `time` root re-export |
| prelude re-export | Duration, Timestamp |
| inherent impls | timezone-dependent extensions |
| generics | Measurement<T> |
| pattern matching | LocalTimeResolution |
| native methods | timezone database |
| primitive methods | host clocks/system timezone |
| arbitrary `Int` arithmetic | timestamp/calendar math |
| negative division/modulo | pre-epoch Gregorian conversion |
| strings | ISO parsing/formatting |
| reflection | every declaration category |
| GC | large populations of immutable temporal values |
| concurrency integration | Duration with sleep/timeouts |

The governing policy should be:

> If implementing ordinary temporal semantics exposes a Phalcom compiler, semantic, runtime, reflection, module, `data`, enum, generic, or arithmetic defect, fix the language rather than hiding the defect by moving the operation into Rust.

---

# 52. Ordinary-package equivalence tests

Because the implementation lives in the Universe, C1 should guard against Universe-only shortcuts.

At minimum, an ordinary application fixture must consume the complete public API through normal imports.

For example:

```phalcom
from time import \
  Date,
  DateTime,
  Month,
  Period,
  Time,
  TimeZone,
  ZonedDateTime,
  clock

const started =
  clock.now

const local =
  DateTime(
    date:
      Date(
        year: 2026,
        month: Month::November,
        day: 1
      ),
    time:
      Time(
        hour: 1,
        minute: 30,
        second: 0,
        nanosecond: 0
      )
  )

const zone =
  TimeZone.named(
    "America/New_York"
  )

match local.resolve(zone) {
  ...
}

System.print(
  started.elapsed
)
```

This must compile through the normal user package path.

Where feasible, pure source modules such as Gregorian arithmetic should additionally be compiled under ordinary-package conditions as a differential certification test.

---

# 53. Reflection requirements

Temporal declarations should look like ordinary declarations because they are ordinary declarations.

Reflection should identify:

```text
Duration as data
Timestamp as data
Date as data
Clock as abstract class
SystemClock as class
Month as enum
LocalTimeResolution as enum
TimeZone as class
ZonedDateTime as data
Measurement<T> as generic data
```

No temporal type should require a special reflection branch simply because it lives in the Universe.

`clock.class` should naturally be:

```text
SystemClock
```

even if its exported static type is `Clock`.

---

# 54. Performance expectations

The initial implementation should favor semantic simplicity.

Typical operations should have roughly these costs:

```text
clock
    zero allocation after package initialization

clock.now
    one primitive host clock read
    one Int
    one Instant allocation

Timestamp.now
    one primitive wall-clock read
    one Int
    one Timestamp value allocation

Duration arithmetic
    ordinary Int arithmetic
    one Duration value

Date arithmetic
    ordinary integer arithmetic
    one Date value

fixed-offset ZonedDateTime projection
    pure Phalcom arithmetic

named-zone projection
    one native timezone lookup
    ordinary Phalcom result values
```

No hash maps, locks, or tzdb work should occur merely because `time` was imported, except the small ordinary objects such as the canonical clock values.

Timezone database initialization is lazy.

---

# 55. Interaction with concurrency

`Duration` is shared with concurrency but belongs to `time`.

Concurrency owns:

```text
sleep
timer queues
reactor deadlines
Future.timeout
scheduler suspension
```

The time library owns:

```text
Duration
Instant
Clock
Timestamp
calendar values
timezone rules
zoned semantics
```

`System.sleep(Duration)` therefore remains legitimate even after date/time discovery is removed from `System`.

Sleep is an active scheduler/runtime operation, not a temporal query.

The reactor should continue using an efficient internal monotonic deadline representation rather than allocating public `Instant` objects for every timer.

Its monotonic source must nevertheless represent the same physical host timeline as `SystemClock`.

---

# 56. Deliberate exclusions from C1

The following are outside the foundational `time` library:

```text
cron
RRULE / iCalendar recurrence
holiday calendars
business-day calendars
market calendars
appointments
alarms
reminders
job schedulers
natural-language time parsing
"three hours ago" presentation
localized date/time formatting
CLDR
non-Gregorian calendars
TAI
GPS time
astronomical Julian dates
NTP synchronization
leap-second tables
```

Those can build on C1 later without reshaping its core abstractions.

---

# 57. Final package philosophy

The final public surface should feel like an object-oriented temporal library rather than a bag of system calls.

The common expressions are:

```phalcom
Duration.seconds(5)

Timestamp.now

Date.today

TimeZone.system

ZonedDateTime.now
```

and:

```phalcom
from time import clock

clock.now
```

Not:

```text
System.clock
System.now
System.date
System.timeZone
```

`System` remains the runtime/process/concurrency capability surface.

`time` owns temporal concepts.

---

# 58. Final public surface summary

The intended root API is approximately:

```phalcom
// Exact amounts

data Duration(
  nanoseconds: Int
)

data Measurement<T>(
  value: T,
  duration: Duration
)


// Monotonic time

@abstract
class Clock {
  @abstract
  now -> Instant

  since(_ instant: Instant) -> Duration

  measure<T>(
    _ action: () -> T
  ) -> Measurement<T>
}

class SystemClock is Clock {
  @class
  instance -> SystemClock
}

const clock: Clock =
  SystemClock.instance

class Instant {
  clock -> Clock
  elapsed -> Duration

  since(_ earlier: Instant) -> Duration
  plus(_ duration: Duration) -> Instant
  minus(_ duration: Duration) -> Instant

  compare(_ other: Instant) -> Ordering
}


// Absolute time

data Timestamp(
  unixNanoseconds: Int
) {
  @class epoch -> Timestamp
  @class now -> Timestamp

  plus(_ duration: Duration) -> Timestamp
  minus(_ duration: Duration) -> Timestamp
  since(_ earlier: Timestamp) -> Duration

  compare(_ other: Timestamp) -> Ordering
}

@abstract
class WallClock {
  @abstract
  now -> Timestamp
}

class SystemWallClock is WallClock {
  @class
  instance -> SystemWallClock
}

const wallClock: WallClock =
  SystemWallClock.instance


// Civil calendar

enum Month {
  January
  February
  March
  April
  May
  June
  July
  August
  September
  October
  November
  December
}

enum Weekday {
  Monday
  Tuesday
  Wednesday
  Thursday
  Friday
  Saturday
  Sunday
}

enum DateOverflow {
  Constrain
  Reject
}

data Period(
  months: Int,
  days: Int
)

data Date(
  year: Int,
  month: Month,
  day: Int
)

data Time(
  hour: Int,
  minute: Int,
  second: Int,
  nanosecond: Int
)

data DateTime(
  date: Date,
  time: Time
)

data YearMonth(
  year: Int,
  month: Month
)

data MonthDay(
  month: Month,
  day: Int
)


// Zones

data UtcOffset(
  seconds: Int
)

class TimeZone {
  @class utc -> TimeZone
  @class fixed(_ offset: UtcOffset) -> TimeZone
  @class named(_ identifier: String) -> TimeZone
  @class system -> TimeZone

  offsetAt(_ timestamp: Timestamp) -> UtcOffset

  transitionBefore(
    _ timestamp: Timestamp
  ) -> Option<ZoneTransition>

  transitionAfter(
    _ timestamp: Timestamp
  ) -> Option<ZoneTransition>
}

data ZoneTransition(
  timestamp: Timestamp,
  offsetBefore: UtcOffset,
  offsetAfter: UtcOffset
)

enum LocalTimeDisambiguation {
  Reject
  Earlier
  Later
  Compatible
}

enum LocalTimeResolution {
  Unique(
    value: ZonedDateTime
  )

  Ambiguous(
    earlier: ZonedDateTime,
    later: ZonedDateTime
  )

  Gap(
    before: ZonedDateTime,
    after: ZonedDateTime
  )
}

data ZonedDateTime(
  timestamp: Timestamp,
  timeZone: TimeZone
)
```

The package then layers convenience behavior over those canonical abstractions:

```phalcom
Timestamp.in(zone)

DateTime.resolve(zone)
DateTime.in(
  zone,
  disambiguation: ...
)

TimeZone.at(timestamp)
TimeZone.resolve(dateTime)

Date.today
Date.today(in: zone)

ZonedDateTime.now
ZonedDateTime.now(in: zone)

ZonedDateTime.shift(duration)
ZonedDateTime.advance(period)
ZonedDateTime.withTimeZone(zone)
ZonedDateTime.sameInstant(other)
```

And the Universe root/prelude exports only:

```text
Duration
Timestamp
```

while the full package owns:

```text
time::clock
time::wallClock
time::Clock
time::SystemClock
time::Instant
time::WallClock
time::SystemWallClock
time::Date
time::Time
time::DateTime
time::Period
time::TimeZone
time::ZonedDateTime
...
```

That is the complete target architecture for `STDL002.C1`.