# Standard Library — Monotonic Time

> **Status:** Normative monotonic-time surface.
>
> **Implementation owner:** STDL002.C1.P1.

The time library provides elapsed-time measurement and monotonic scheduling
inputs. It deliberately has no wall-clock, calendar, timezone, or civil-time
semantics.

## 1. `Duration`

`Duration` is an ordinary value object storing a signed integer nanosecond
count.

| Signature | Result |
|---|---|
| `Duration.nanoseconds(_)` | `Duration` |
| `Duration.microseconds(_)` | `Duration` |
| `Duration.milliseconds(_)` | `Duration` |
| `Duration.seconds(_)` | `Duration` |
| `Duration.minutes(_)` | `Duration` |
| `Duration.hours(_)` | `Duration` |
| `nanoseconds` | `Int` |
| `microseconds` | `Number` |
| `milliseconds` | `Number` |
| `seconds` | `Number` |
| `+(_)`, `-(_)` | `Duration` |
| `*(_)` with an `Int` factor | `Duration` |
| `isZero`, `isNegative` | `Bool` |

Constructors accept `Int` only. Fractional constructor arguments are not
implicitly rounded or truncated.

Equality and hashing use the normalized nanosecond value. Negative durations
are valid.

## 2. `Clock`

`Clock` is an extensible monotonic time domain.

| Signature | Result |
|---|---|
| `System.clock` | the process-wide `Clock` singleton |
| `clock.now` | an `Instant` in that clock domain |
| `clock.since(_)` | a `Duration` |
| protected `clock.instant(_)` | an `Instant` in that clock domain |

`System.clock === System.clock` is stable for the lifetime of the VM. The
system clock reads the VM-owned monotonic source and never uses Unix epoch
time.

## 3. `Instant`

An `Instant` retains both its originating clock identity and its monotonic
tick value. It is not a globally comparable timestamp.

| Signature | Result |
|---|---|
| `instant.elapsed` | `Duration`, measured by its originating clock |
| `instant.since(_)` | `Duration` |
| `instant.plus(_)`, `instant.minus(_)` | `Instant` |
| `instant.compare(_)` | `Ordering` |
| `<`, `<=`, `>`, `>=` | `Bool` |

Difference and ordering operations require both instants to belong to the
same clock domain. A cross-domain operation raises `ArgumentError`; equality
across clock domains is `false`.

## 4. Representation and boundaries

`Clock`, `Instant`, and `Duration` are ordinary Phalcom classes. They do not
add VM `Value` variants or native object kinds. Only the irreducible
`System._$monotonicNanoseconds` host read is native.

Wall-clock and civil-time types remain deferred:

```text
Timestamp / Date / DateTime / TimeZone
```
