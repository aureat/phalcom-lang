---
id: STDL002.C1.P1
category: STDL
program: STDL002
checkpoint: STDL002.C1
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
depends_on: []
follows: null
supersedes: null
deferred_reason: null
---

# STDL002.C1.P1 — monotonic `Clock`, `Instant`, and `Duration`

The implementation should be deliberately small at the native floor. The current repository already has almost exactly the host-side primitive we need: `VM` owns a `start_time: std::time::Instant`, explicitly documented as being for future `System` timing primitives, and initializes it with `Instant::now()`.

The `time` Universe package also already exists but is empty apart from its documentation header.

So the correct architecture is:

```text
Rust/native floor
    System._$monotonicNanoseconds -> Int
                    │
                    ▼
Phalcom time library
    Clock
    Instant
    Duration
                    │
                    ▼
    System.clock -> Clock
                    │
                    ▼
C3 reactor later consumes Duration / monotonic deadlines
```

No new `Value` variant, heap object kind, runtime-specialized time representation, or native `Instant`/`Duration` class is necessary.

---

## 1. Ratify the implementation surface first

Before code, freeze these semantics.

```phalcom
System.clock -> Clock

class Clock {
  now -> Instant
  since(_ instant: Instant) -> Duration
}

class Instant {
  elapsed -> Duration
  since(_ earlier: Instant) -> Duration
  plus(_ duration: Duration) -> Instant
  minus(_ duration: Duration) -> Instant
  compare(_ other: Instant) -> Ordering
}

class Duration {
  @class nanoseconds(_ value: Int) -> Duration
  @class microseconds(_ value: Int) -> Duration
  @class milliseconds(_ value: Int) -> Duration
  @class seconds(_ value: Int) -> Duration
  @class minutes(_ value: Int) -> Duration
  @class hours(_ value: Int) -> Duration

  nanoseconds -> Int
  microseconds -> Number
  milliseconds -> Number
  seconds -> Number

  +(_ other: Duration) -> Duration
  -(_ other: Duration) -> Duration
  *(_ factor: Int) -> Duration

  compare(_ other: Duration) -> Ordering
  isZero -> Bool
  isNegative -> Bool
}
```

One correction from the conceptual design is important: do **not** try to define both:

```phalcom
Instant - Instant -> Duration
Instant - Duration -> Instant
```

Phalcom dispatch does not overload a selector by parameter type. Both are the same `-(_)` selector. Use the explicit `since`, `plus`, and `minus` family instead.

That actually fits Phalcom's preference for clear APIs better.

---

# 2. Representation decisions

### `Duration`

Use an ordinary Phalcom instance with one private `Int` field:

```text
_nanoseconds: Int
```

That gives signed, arbitrary-precision duration arithmetic for free. Phalcom's `Int` already has arbitrary-precision fallback rather than fixed 64-bit overflow semantics.

A duration may be negative. This is useful because:

```phalcom
a.since(b)
```

has a mathematically meaningful negative result when `a` precedes `b`.

The reactor will reject a negative duration where negative waiting is nonsensical; the value type itself should not prohibit signed duration arithmetic.

### `Instant`

Use an ordinary Phalcom instance containing:

```text
_clock: Clock
_ticks: Int
```

where `_ticks` is monotonic nanoseconds in that Clock's private timeline.

The `_clock` reference is essential. It gives Instants a clock domain.

These are legal:

```phalcom
const a = clock.now
const b = clock.now

b.since(a)
```

These are not:

```phalcom
clockA.now.since(clockB.now)
```

unless `clockA === clockB`.

Cross-domain comparison/difference should raise an argument/domain error.

### `Clock`

`Clock` itself is an ordinary source class.

`System.clock` returns one singleton `Clock` object. Clock object identity therefore *is* the monotonic domain identity for the system clock.

This lets a future test clock naturally get its own domain simply by being another Clock object.

---

# 3. The native floor should be exactly one new primitive

Add:

```phalcom
@class
@internal
@native
_$monotonicNanoseconds -> Int
```

to `System`.

Implement it in:

```text
phalcom-core/src/primitive/system.rs
```

approximately as:

```rust
let nanos = vm.start_time.elapsed().as_nanos();
```

and normalize that `u128` into a Phalcom `Int`.

Do not expose the process-relative ticks directly as public API.

The public call path becomes:

```text
System.clock.now
       │
       ▼
Clock#now
       │
       ▼
System._$monotonicNanoseconds
       │
       ▼
Instant(clock: self, ticks: n)
```

The current `System` native infrastructure lives exactly in `primitive/system.rs`; the existing System surface is declared in the canonical Universe's concurrency source.

The primitive also needs installation in:

```text
phalcom-core/src/universe/primitives.rs
```

alongside the other System internal primitives. Current primitive installation is still explicitly wired there.

---

# 4. Checkpoint T0 — Source/model feasibility and baseline

Before modifying runtime behavior, verify:

```text
current HEAD/worktree
time package source topology
System source ownership
class-side field behavior
protected method access
internal selector calls between privileged Universe classes
native floor census
```

The class-side singleton pattern does not need invention: the Universe already uses a lazy class-side `_instance` in `Ellipsis`.

Class-side fields are part of the accepted language surface.

Required spike before the main patch:

```phalcom
class _ClockProbe {
  @class _instance

  @class
  instance {
    if (_instance == None) {
      _instance = _ClockProbe.create()
    }
    _instance
  }

  @private
  @constructor
  create() { self }
}
```

Prove:

```phalcom
_ClockProbe.instance === _ClockProbe.instance
```

Then prove a subclass can call an inherited `@protected` instance helper, because that is the intended extension seam for test clocks.

If either fails, adapt representation before adding runtime primitives.

---

# 5. Checkpoint T1 — Implement `Duration`

Create:

```text
phalcom-core/core/universe/src/time/clock.ph
```

or, if separation is preferred:

```text
time/duration.ph
time/instant.ph
time/clock.ph
```

I would start with one `clock.ph`; split only after the API stabilizes.

Update:

```text
phalcom-core/core/universe/src/time/package.ph
```

from its current empty package body to expose the time declarations. The package is already wired into the root Universe package.

Suggested implementation:

```phalcom
class Duration {
  @private
  @constructor
  create(_ nanoseconds: Int) {
    _nanoseconds = nanoseconds
  }

  @class
  nanoseconds(_ value: Int) -> Duration {
    Duration.create(value)
  }

  @class
  microseconds(_ value: Int) -> Duration {
    Duration.create(value * 1_000)
  }

  @class
  milliseconds(_ value: Int) -> Duration {
    Duration.create(value * 1_000_000)
  }

  @class
  seconds(_ value: Int) -> Duration {
    Duration.create(value * 1_000_000_000)
  }

  @class
  minutes(_ value: Int) -> Duration {
    Duration.seconds(value * 60)
  }

  @class
  hours(_ value: Int) -> Duration {
    Duration.minutes(value * 60)
  }

  nanoseconds -> Int {
    _nanoseconds
  }

  microseconds -> Number {
    _nanoseconds / 1_000
  }

  milliseconds -> Number {
    _nanoseconds / 1_000_000
  }

  seconds -> Number {
    _nanoseconds / 1_000_000_000
  }

  +(_ other: Duration) -> Duration {
    Duration.nanoseconds(_nanoseconds + other.nanoseconds)
  }

  -(_ other: Duration) -> Duration {
    Duration.nanoseconds(_nanoseconds - other.nanoseconds)
  }

  *(_ factor: Int) -> Duration {
    Duration.nanoseconds(_nanoseconds * factor)
  }

  isZero -> Bool {
    _nanoseconds == 0
  }

  isNegative -> Bool {
    _nanoseconds < 0
  }
}
```

Initially accept `Int`, not `Number`, in duration constructors.

That avoids inventing a fractional-Nanosecond rounding policy. Current `Int.new(_)` does not even accept a `Float`, which is useful evidence that silent truncation would be contrary to current numeric conversion rules.

Fractional constructors can be a separate design:

```phalcom
Duration.seconds(1.5)
```

with an explicitly ratified rounding rule.

Do not smuggle that decision into the initial implementation.

---

# 6. `Duration` must have value semantics

Implement:

```phalcom
==(_ other: Dynamic) -> Bool
hash -> Int
compare(_ other: Duration) -> Ordering
<(_ other: Duration) -> Bool
<=(_ other: Duration) -> Bool
>(_ other: Duration) -> Bool
>=(_ other: Duration) -> Bool
```

If `==` compares nanoseconds, `hash` must also derive from nanoseconds. Do not leave inherited identity hashing with value equality.

`Ordering` already exists canonically with Less/Equal/Greater/Unordered.

For the first implementation, keep rendering simple and stable:

```phalcom
toString {
  "\(_nanoseconds)ns"
}
```

Adaptive pretty-printing such as:

```text
1ms
2.5s
3h
```

is a presentation improvement, not a semantic blocker.

---

# 7. Checkpoint T2 — Implement `Instant`

`Instant` should have no public raw constructor.

Suggested surface:

```phalcom
class Instant {
  @private
  @constructor
  create(_ clock: Clock, _ ticks: Int) {
    _clock = clock
    _ticks = ticks
  }

  @class
  @internal
  _$from(_ clock: Clock, _ ticks: Int) -> Instant {
    Instant.create(clock, ticks)
  }

  @internal
  _$clock -> Clock {
    _clock
  }

  @internal
  _$ticks -> Int {
    _ticks
  }

  elapsed -> Duration {
    _clock.since(self)
  }

  since(_ earlier: Instant) -> Duration {
    self.requireSameClock(earlier)
    Duration.nanoseconds(_ticks - earlier._$ticks)
  }

  plus(_ duration: Duration) -> Instant {
    Instant._$from(
      _clock,
      _ticks + duration.nanoseconds
    )
  }

  minus(_ duration: Duration) -> Instant {
    Instant._$from(
      _clock,
      _ticks - duration.nanoseconds
    )
  }
}
```

`_$from`, `_$ticks`, and `_$clock` belong in the implementation namespace and should carry `@internal` because canonical Universe source requires implementation-namespace declarations to state that intent explicitly.

Add one helper:

```phalcom
@private
requireSameClock(_ other: Instant) -> Unit
```

checking:

```phalcom
_clock === other._$clock
```

Cross-domain operations should fail before arithmetic.

---

# 8. `Instant` equality/order laws

Implement:

```text
same Clock identity + same ticks -> equal
different Clock identity         -> not equal
```

For ordering:

```text
same domain     -> compare ticks
different domain -> error
```

Do not return `Ordering.unordered` for distinct clocks.

Why? Because different monotonic clocks are not merely partially ordered values; there is no defined transformation between their origins. Treating them as unordered would make accidental cross-clock arithmetic look semantically valid.

Equality can safely return false across domains.

Ordering/difference should reject.

Like Duration, if Instant overrides `==`, it must override `hash` consistently:

```text
hash(clock identity, ticks)
```

A simple source-level mix of `_clock.hash` and `_ticks.hash` is enough.

---

# 9. Checkpoint T3 — Implement `Clock` and the singleton

Suggested class:

```phalcom
class Clock {
  @class _system

  @class
  @internal
  _$system -> Clock {
    if (_system == None) {
      _system = Clock.create()
    }

    _system
  }

  @private
  @constructor
  create() {
    self
  }

  now -> Instant {
    self.instant(System._$monotonicNanoseconds)
  }

  since(_ instant: Instant) -> Duration {
    self.now.since(instant)
  }

  @protected
  instant(_ ticks: Int) -> Instant {
    Instant._$from(self, ticks)
  }
}
```

The protected `instant(_)` helper is important.

It lets a future/manual test Clock implement:

```phalcom
class ManualClock is Clock {
  @constructor
  new() {
    _ticks = 0
  }

  now -> Instant {
    self.instant(_ticks)
  }

  advance(_ duration: Duration) -> ManualClock {
    _ticks = _ticks + duration.nanoseconds
    self
  }
}
```

without exposing raw Instant construction to ordinary callers.

That gives Phalcom dependency injection naturally:

```phalcom
class Cache {
  @constructor
  new(clock: Clock) {
    _clock = clock
  }
}
```

Production:

```phalcom
Cache.new(clock: System.clock)
```

Test:

```phalcom
Cache.new(clock: ManualClock.new())
```

No runtime mocking mechanism is necessary.

---

# 10. Add `System.clock`

Modify the canonical System declaration in:

```text
phalcom-core/core/universe/src/concurrency/fiber.ph
```

with:

```phalcom
@class
clock -> Clock {
  Clock._$system
}

@class
@internal
@native
_$monotonicNanoseconds -> Int
```

The public getter is pure `.ph`.

Only the irreducible OS read is native.

That is preferable to:

```phalcom
@class @native clock -> Clock
```

because singleton policy and object construction remain normal Phalcom behavior instead of becoming hidden Rust policy.

The class currently has only the scheduler/GC/output native seams, so this adds one precise floor operation rather than a new time subsystem inside Rust.

---

# 11. Checkpoint T4 — Native monotonic primitive

In:

```text
phalcom-core/src/primitive/system.rs
```

add:

```rust
#[phalcom_native_macros::primitive(
    System,
    "_$monotonicNanoseconds",
    params = [],
    returns = Int,
    types = "() -> Int",
    side = class,
    visibility = internal,
    effects = pure
)]
```

or the exact metadata syntax accepted by the current native descriptor system.

Implementation:

```rust
let nanos = vm.start_time.elapsed().as_nanos();
```

then convert exactly to Phalcom `Int`.

Do not cast blindly to `i64`.

Use the existing BigInt normalization path so the primitive is mathematically exact even though a real process is very unlikely to live long enough to exceed signed 64-bit nanoseconds.

Then update:

```text
phalcom-core/src/universe/primitives.rs
```

to import and install:

```text
_$monotonicNanoseconds
```

as a class-side internal primitive on System.

Also update:

```text
docs/spec/current/core/floor-census.md
```

because this is a real new native floor selector.

---

# 12. Do not create native `Clock`, `Instant`, or `Duration` classes

This is a hard implementation constraint for the first version.

Do not add:

```rust
Object::Instant
Object::Duration
Value::Instant
Value::Duration
```

Do not add Clock/Instant/Duration to primordial kernel class creation.

They should be normal Universe classes.

Reasons:

- `Duration` arithmetic already maps cleanly to `Int`;
- `Instant` has only two ordinary fields;
- clock identity naturally uses normal object identity;
- GC works automatically;
- reflection works automatically;
- typing works automatically;
- no new tagging/layout burden;
- C3 can use host-side monotonic values internally without requiring the public object representation to become reactor representation.

Optimize later only if profiling proves time-object allocation significant.

---

# 13. Checkpoint T5 — Typing and semantic verification

Add semantic tests proving:

```text
System.clock        : Clock
System.clock.now    : Instant
instant.elapsed     : Duration
instant.since(_)    : Duration
instant.plus(_)     : Instant
duration.seconds    : Number
Duration.seconds(1) : Duration
```

Also prove wrong arguments diagnose:

```phalcom
Duration.seconds(1.5)       // rejected initially
instant.plus(42)            // rejected
instant.since(duration)     // rejected
```

The numeric surface is already split into `Int` and `Float`, so annotations should use those actual types rather than the older `Number`-only design.

Run the full semantic suite because these classes become canonical Universe bindings.

---

# 14. Checkpoint T6 — Runtime behavior tests

Create a new language corpus lane if appropriate:

```text
phalcom-core/tests/fixtures/language/time/
```

Minimum permanent cases:

| Test | Law |
|---|---|
| `system_clock_singleton` | `System.clock === System.clock` |
| `clock_now_monotonic` | later tick is `>=` earlier tick |
| `duration_units` | exact unit conversions |
| `duration_arithmetic` | signed add/subtract/multiply |
| `duration_value_equality` | equality/hash law |
| `instant_since` | same-domain difference |
| `instant_plus_minus` | reversible tick arithmetic |
| `instant_domain_mismatch` | cross-clock difference/order rejects |
| `instant_equality_cross_domain` | false |
| `manual_clock` | subclass can call protected `instant(_)` |
| `elapsed_uses_origin_clock` | ManualClock `Instant.elapsed` consults that clock, not `System.clock` |
| `internal_tick_visibility` | application source cannot call `System._$monotonicNanoseconds` |

For monotonic clock testing, do **not** assert:

```text
sleep 5 ms
elapsed >= 5 ms
```

at this stage.

Avoid wall-clock-sensitive tests.

For the real system clock, only require non-decreasing observations:

```phalcom
const a = System.clock.now
const b = System.clock.now

Assert.truth(b.compare(a) != Ordering.less)
```

Use `ManualClock` for exact arithmetic.

---

# 15. The `elapsed` implementation must use the originating clock

This deserves its own regression.

Wrong:

```phalcom
elapsed {
  System.clock.now.since(self)
}
```

Correct:

```phalcom
elapsed {
  _clock.since(self)
}
```

Otherwise an Instant created by a ManualClock cannot be measured.

This is why storing the Clock object in every Instant is not redundant metadata.

---

# 16. Checkpoint T7 — `Clock.measure` convenience

Only after the foundational types are green, add:

```phalcom
class Measurement<T> {
  @constructor
  new(value: T, duration: Duration) {
    _value = value
    _duration = duration
  }

  value -> T { _value }
  duration -> Duration { _duration }
}
```

Then:

```phalcom
measure<T>(_ action: () -> T) -> Measurement<T> {
  const start = now
  const value = action()
  Measurement.new(
    value: value,
    duration: now.since(start)
  )
}
```

This is pure library code.

Do not make measurement native.

Do not make this checkpoint block the basic Clock landing if generic-method inference creates unrelated trouble.

Benchmarks already contain multiple comments showing `System.clock` as a missing facility, so this surface immediately unlocks in-language benchmarking even before `measure` lands.

---

# 17. Checkpoint T8 — Update the time/System specification

Create a dedicated normative document, preferably:

```text
docs/spec/current/stdlib/time.md
```

It should specify:

```text
Clock
Instant
Duration
clock domains
monotonicity
signed durations
cross-domain rejection
no calendar semantics
```

Then update:

```text
docs/spec/current/system.md
```

Current spec still says:

```text
clock -> monotonic seconds as Float
now   -> wall-clock epoch seconds as Float
```



Replace the first with:

```text
clock -> Clock
```

Do **not** implement `System.now` in this plan.

Mark wall/civil time as separately deferred:

```text
Timestamp / Date / DateTime / TimeZone
```

The clock project is monotonic time only.

---

# 18. Change C3 before reactor implementation starts

Once `Duration` exists, C3 should no longer expose:

```phalcom
System.sleep(_ milliseconds: Int)
```

Change the planned API to:

```phalcom
System.sleep(_ duration: Duration) -> Future<Unit>
```

This is the right moment to do it because C3 has not landed yet.

Then its implementation boundary becomes:

```text
Phalcom:
    Duration
       ↓
System.sleep(duration)
       ↓
native extracts duration.nanoseconds
       ↓
reactor absolute monotonic deadline
```

Do not make the reactor traffic in Phalcom `Instant` heap objects internally.

The reactor can keep efficient host/internal deadline representation while the public API remains typed.

Also change:

```text
Future.timeout(milliseconds: Int)
```

in C4 planning toward:

```phalcom
future.timeout(_ duration: Duration) -> Future<T>
```

and Backoff should eventually produce a Duration rather than an ambiguous numeric delay.

---

# 19. Reactor-time integration rule

C3 should consume the same monotonic source as `System.clock`.

Today `VM.start_time` already exists. For the first Clock implementation, use it directly.

When C3 implements deterministic timer tests, refactor the host side behind one VM-owned source:

```rust
trait MonotonicSource {
    fn now_nanos(&self) -> u128;
}
```

or an equally small concrete test seam.

Do **not** create separate clocks:

```text
System.clock host source A
reactor host source B
```

They must measure the same monotonic timeline.

This host-source injection can land with C3 rather than blocking the Clock foundation.

---

# 20. Performance requirements

The first implementation should have these costs:

```text
System.clock
    first call: one Clock allocation
    later calls: zero Clock allocation

Clock.now
    one native monotonic read
    one Int value
    one ordinary Instant allocation

Instant.since
    one Duration allocation

Duration arithmetic
    one ordinary Duration allocation
```

No hash maps, locking, atomics, OS calls beyond monotonic time read, or custom GC roots.

If timing-heavy profiling later shows object allocation distortion, optimization can specialize representation transparently.

Do not pre-optimize the public value model into new VM tags.

---

# 21. Explicitly forbidden shortcuts

Do not implement:

```phalcom
System.clock -> Float
```

as the final surface.

Do not expose:

```phalcom
Instant.ticks
```

publicly.

Do not let different Clock domains compare by raw tick value.

Do not use wall-clock/Unix epoch time for `Clock.now`.

Do not use `Float` internally for elapsed time.

Do not let `System.sleep` continue using anonymous millisecond integers once Duration exists.

Do not add a wall-time `System.now` in the same patch.

Do not introduce OS timezone/calendar semantics.

Do not add special VM object variants for Duration/Instant without measurements.

---

# 22. Suggested implementation sequence

I would implement this in four commits:

```text
1. feat(time): add Duration and Instant value model
2. feat(time): add monotonic Clock and System.clock
3. test(time): certify clock domains, typing, and manual clocks
4. docs(time): ratify monotonic time surface and update reactor plans
```

The key implementation order is:

```text
Duration
   ↓
Instant
   ↓
Clock
   ↓
native monotonic ticks
   ↓
System.clock
   ↓
semantic/runtime certification
   ↓
C3/C4 API migration
```

That keeps failures local.

---

# 23. Delivery gates

Run focused first:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib <system_clock_primitive_tests>

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test language-corpus <time_lane> -- --exact

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic <time_type_tests>
```

Then:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core native_surface_contracts

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core reflection_conformance

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus

cargo fmt --all -- --check

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo clippy --workspace --all-targets -- -D warnings
```

Because the change adds an internal native selector, also run the native floor census and ensure the only irreducible floor growth is:

```text
System._$monotonicNanoseconds
```

Everything above that should be ordinary Phalcom.

---

# 24. Implementation record — 2026-09-12

The monotonic time foundation is implemented through the source-owned `time.clock`
module. `Duration`, `Instant`, and `Clock` are ordinary Phalcom classes; the only
new native floor is `System._$monotonicNanoseconds -> Int`. The implementation is
registered in the built-in module graph and prelude, and the generated native
surface and floor census include the new internal primitive.

Implemented verification coverage:

- positive `time` language corpus: 1 passed;
- negative `time` language corpus: 1 passed;
- semantic `capabilities::time`: 2 passed;
- native reflection census: passed;
- native surface generator freshness: passed;
- monotonic floor census: passed.

This plan remains partial because the already-landed CONC002 C3/C4 lane still
uses `System.sleep(Int) -> Future<Unit>`. Migrating that public API and its
timeout consumers to `Duration` is a follow-up at the concurrency ownership
boundary; the current implementation preserves that active work. Full semantic
verification also remains baseline-blocked by three unrelated generic/Future
tests, and the broad core verifier was not completed in this shared checkout.

---

## Final target

The important end state is:

```phalcom
const clock = System.clock

const start = clock.now

performWork()

const elapsed = start.elapsed

System.print(elapsed.seconds)
```

and testable code can instead receive:

```phalcom
const clock = ManualClock.new()
```

without changing the code under test.
