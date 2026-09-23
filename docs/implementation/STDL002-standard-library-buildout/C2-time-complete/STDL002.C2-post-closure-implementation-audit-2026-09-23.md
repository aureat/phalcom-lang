---
id: STDL002.C2.POST-CLOSURE-AUDIT-2026-09-23
program: STDL002
checkpoint: STDL002.C2
kind: post-closure-implementation-audit
status: FINAL
prepared: 2026-09-23
repository: aureat/phalcom-lang
audited_branch: main
audited_head: 1c9985b6d449d8680667a91651ea64b2808c44ca
integration_commit: 25a225e9d28317a8f6da90d7674ab30579920b09
administrative_checkpoint_status: CLOSED_FOR_NOW
specification_status: USER-DEFERRED
---

# STDL002.C2 Post-Closure Implementation Audit

## 0. Executive conclusion

This audit accepts the project decision that **STDL002.C2 is closed for now and the implementation is marked complete**. The purpose of this report is therefore not to reopen C2 as a prerequisite. It is to establish a truthful as-built defect/debt inventory for later standard-library, runtime, semantic, and maintenance work.

The current repository is materially different from the earlier post-P2 audit. The major P2 implementation did land in commit `25a225e9d28317a8f6da90d7674ab30579920b09` (`feat(stdlib): integrate complete time library`), and current `main` is `1c9985b6d449d8680667a91651ea64b2808c44ca`. The live tree now contains Jiff `0.2.37`, a VM-owned timezone adapter, `TimeZone`, `ZoneTransition`, `LocalTimeResolution`, `LocalTimeDisambiguation`, `ZonedDateTime`, current-time conveniences, `time.testing`, canonical formatting for most temporal values, and parsers for `Duration`, `Period`, `UtcOffset`, `Date`, `Time`, and `DateTime`.

Accordingly, the earlier audit's central conclusion that “P2 is absent” is obsolete. The earlier P1 native-floor findings have also been fixed: `SystemClock.monotonicNanoseconds` and `SystemWallClock.unixNanoseconds` now carry source `@primitive` metadata, their native effect metadata is `Nondeterminism`, and the core floor census records their new owners.

However, the current implementation still has substantial residual work. The most important remaining issues are not merely lifecycle paperwork. They include **real correctness bugs in timezone identity, backend-range handling, system timezone semantics, and textual serialization**, plus missing public APIs and major certification gaps.

The most consequential findings are:

1. **Timezone object identity is backed by a collision-prone 64-bit hash.** A hash collision can make different zones compare equal and can cause an existing `TimeZone` object to begin resolving through another zone's backend handle after the collision overwrites the adapter map entry.
2. **Named-zone canonical identity is incomplete.** The manually checked-in alias map covers only a small subset of IANA links, is case-sensitive, and is version-skewed with the bundled Jiff database. Equivalent accepted names can therefore compare unequal and serialize with noncanonical names.
3. **Unnamed system zones lose rule semantics.** When Jiff discovers a real TZif/POSIX-rule zone without an IANA identifier, the adapter collapses the full rule set to the offset at the current instant, turning a potentially DST-aware zone into a fixed offset.
4. **Native local civil resolution silently narrows/aliases large years.** `resolveNative` converts an arbitrary Phalcom `Int` year to `i64` and then uses unchecked `as i16`; values such as `65536` can wrap to year `0` and produce a valid but incorrect result instead of a backend-range error.
5. **UTC and fixed-offset operations are unnecessarily limited by Jiff's finite timestamp/civil range.** P1's `Timestamp` and civil model are wider, but all zoned projection/resolution paths currently pass through Jiff even when UTC/fixed arithmetic can be done exactly in source/general integer arithmetic.
6. **Canonical fractional formatting is wrong for nanoseconds with leading zeroes.** `Time.new(..., 1).toString` emits `.1`, which conventionally parses as 100,000,000 ns under the library's own parser rather than 1 ns. The same defect propagates to `DateTime`, `Timestamp`, and `ZonedDateTime` text.
7. **Expanded/negative year canonical text is not implemented correctly.** `Date`/`Timestamp` formatting does not implement the P2 four-digit/signed-expanded year grammar, and `Date.parse` cannot parse a negative year because it naïvely splits on `-`.
8. **`Timestamp.parse` and `ZonedDateTime.parse` are still missing.** Consequently numeric-offset timestamp normalization, zoned offset/zone consistency checking, explicit overlap selection by offset, and complete temporal round trips remain unsatisfied library goals.
9. **System timezone discovery is implemented but not classified as the required primitive host observation.** `TimeZone.systemNative` lacks source `@primitive`, its generated native effect contract is `Unknown`, and the core floor census omits it.
10. **Final C2 certification was never implemented in the repository records/tests.** There is no final P2 ordinary-package fixture, no P2 reflection census, no forced-GC/shared-zone-handle test, no system-discovery integration test, no complete parse/format property matrix, and no canonical final P1/P2 walkthrough/handoff. The committed checkpoint itself still says `IN_PROGRESS / PARTIAL / UNVERIFIED`.

The current time library is therefore best characterized as:

> **A broad, useful, architecture-consistent foundational time implementation with the major P1/P2 type surface landed, but with several high-severity correctness holes, incomplete canonical serialization/parsing, incomplete timezone identity semantics, and significant unperformed certification work.**

Those findings can be carried as post-C2 backlog without changing the administrative closure decision.

---

# 1. Audit basis and authority

## 1.1 Repository baseline

The live repository was inspected on `main` at:

```text
1c9985b6d449d8680667a91651ea64b2808c44ca
core(future): rename number package source
```

The time integration commit is:

```text
25a225e9d28317a8f6da90d7674ab30579920b09
feat(stdlib): integrate complete time library
```

The audit inspected the live equivalents of:

```text
Cargo.toml
Cargo.lock
phalcom-core/Cargo.toml

phalcom-core/core/universe/src/time/**
phalcom-core/src/timezone.rs
phalcom-core/src/iana_links.rs
phalcom-core/src/primitive/time.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/gc.rs
phalcom-core/src/vm/bootstrap.rs

phalcom-native-surface/src/generated.rs
phalcom-native-meta/src/universe.rs

phalcom-core/tests/fixtures/language/time/**
phalcom-core/tests/core/native/contracts.rs
phalcom-core/tests/core/modules/runtime.rs
phalcom-core/tests/core/reflection/type_metadata.rs
phalcom-semantic/tests/semantic/capabilities/time.rs

docs/spec/current/core/floor-census.md
docs/implementation/STDL002-standard-library-buildout/C2-time-complete/**
```

## 1.2 Historical records consumed

The attached implementation records were treated as historical evidence, not as proof of current state:

```text
STDL002.C2.P2-plan.md
STDL002.C2.P2-progress-walkthrough.md
STDL002.C2.P2-incident-walkthrough-handoff.md
STDL002.C2.P2-audit.md
```

The P2 plan remains the most useful statement of intended C2 semantics. The progress and incident records explain WIP decisions and previously open issues. The earlier post-P2 audit is useful as a baseline of what was missing before `25a225e9`, but many of its claims have been superseded by that commit.

## 1.3 Specification status

Authoritative `time.md` work remains `USER-DEFERRED`. This audit does **not** use the stale `docs/implementation/time.md` or stale `docs/spec/current/stdlib/time.md` as semantic authority.

The relevant authority hierarchy for this audit is:

```text
live source/runtime behavior
+ live tests
+ accepted P2 plan/checkpoint/guidance decisions
+ LANG006 implementation-kind/metadata rules
+ current backend behavior where the implementation depends on it
```

## 1.4 Execution limitation

The repository was inspected through the connected GitHub repository API. This audit did not execute the current checkout locally. Existing test files and previously recorded passing commands are evidence, but this report does not claim a fresh 2026-09-23 full test run.

---

# 2. Current as-built temporal architecture

The overall architecture is substantially aligned with C2's intended model.

## 2.1 Source-owned value/capability layer

The live package provides:

```text
Duration
Measurement<T>

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

`Duration`, `Timestamp`, civil values, `Period`, ambiguity policy, `ZonedDateTime` movement/equality, and most text logic remain source-owned. No temporal `Value` arm or temporal bytecode was found.

## 2.2 Backend layer

The workspace now depends on:

```toml
jiff = { version = "0.2.37", features = ["tzdb-bundle-always"] }
```

The lockfile contains:

```text
jiff         0.2.37
jiff-core    0.1.1
jiff-static  0.2.37
jiff-tzdb    0.1.8
```

`TimeZoneAdapter` is owned by each `VM` and is explicitly classified as non-root Rust state by GC. It contains:

```rust
database: Option<TimeZoneDatabase>
generation: u64
named: HashMap<String, TimeZone>
handles: HashMap<i64, ZoneHandle>
```

The bundled named-zone database is lazy. `utc()` and `fixed()` do not initialize it. Named lookup uses `TimeZoneDatabase::bundled()` on demand.

## 2.3 Public zone semantics

`TimeZone` currently exposes:

```phalcom
TimeZone.utc
TimeZone.fixed(offset)
TimeZone.named(identifier)
TimeZone.system

zone.offsetSeconds(timestamp) -> Int
zone.transitionBefore(timestamp)
zone.transitionAfter(timestamp)
```

The class internally exposes native fact operations for identity, display, offset, local civil projection, local resolution, and transitions.

## 2.4 Local resolution

The public factual enum is implemented through `DateTime.resolve`:

```text
Unique(value)
Ambiguous(earlier, later)
Gap(before, after)
```

and the source policy supports:

```text
Reject
Earlier
Later
Compatible
```

The actual source policy table matches the intended behavior:

| Case | Reject | Earlier | Later | Compatible |
|---|---|---|---|---|
| Unique | value | value | value | value |
| overlap | error | earlier | later | earlier |
| gap | error | before | after | after |

## 2.5 Zoned semantics

`ZonedDateTime` canonical state is exactly:

```text
Timestamp + TimeZone
```

The live source implements:

```text
offset
date
time
dateTime
withTimeZone
shift(Duration)
advance(Period)
sameInstant
compareInstant
structural equality
now
nowIn(zone)
```

`advance` uses civil `DateTime.advance(..., Constrain)` followed by `Compatible` local resolution, while `shift` changes the underlying timestamp directly.

## 2.6 Text and testing support

Implemented parsers:

```text
Duration.parse
Period.parse
UtcOffset.parse
Date.parse
Time.parse
DateTime.parse
```

Missing parsers:

```text
Timestamp.parse
ZonedDateTime.parse
```

`ManualClock` and `ManualWallClock` are implemented in `universe.time.testing.clocks`.

---

# 3. Verification of the previous post-P2 audit

This section explicitly retires or updates the findings from the attached earlier audit.

## 3.1 Previous blockers

| Previous finding | Current classification | Current evidence |
|---|---|---|
| B1 — P2 public source absent | **RESOLVED** | `timezone.ph`, `zone_transition.ph`, `resolution.ph`, `zoned.ph`, `testing/` now exist |
| B2 — timezone backend/dependency absent | **RESOLVED** | Jiff `0.2.37` + bundled tzdb landed |
| B3 — VM/process tzdb snapshot/cache absent | **RESOLVED in architecture; verification incomplete** | VM owns lazy `TimeZoneAdapter`; named cache exists |
| B4 — system timezone primitive absent | **PARTIALLY RESOLVED** | runtime/native operation exists, but required `@primitive` metadata/effect/floor inventory is still missing |
| B5 — P2 native rule-query surface absent | **RESOLVED** | offset/local resolution/transition native fact paths exist |
| B6 — LocalTimeResolution/disambiguation absent | **RESOLVED** | source enums + policy implemented |
| B7 — ZonedDateTime absent | **RESOLVED** | source data + major operations implemented |
| B8 — final parser/formatter absent | **PARTIAL** | most formatters/parsers landed; `Timestamp.parse` and `ZonedDateTime.parse` remain absent; formatter correctness gaps remain |
| B9 — `time.testing` absent | **RESOLVED implementation; public-root/certification gap remains** | ManualClock/ManualWallClock exist under `.testing.clocks` |
| B10 — P1 T8/handoff incomplete | **Semantic incident resolved; lifecycle closure still missing** | canonical data class-side publication was fixed; final durable P1 lifecycle artifacts remain absent |
| B11 — P2 walkthrough/handoff absent | **STILL OPEN as lifecycle debt** | no canonical completed P2 walkthrough/handoff in C2 directory |
| B12 — checkpoint PARTIAL/UNVERIFIED | **STILL TRUE in repository records** | committed C2 checkpoint front matter remains stale |

## 3.2 Previous high-priority P1 defects

| Previous finding | Current classification | Current evidence |
|---|---|---|
| H1 — clock reads marked `Pure` | **RESOLVED** | both generated native records carry `Nondeterminism` |
| H2 — P1 clock source declarations lack `@primitive` | **RESOLVED** | both source declarations now carry `@primitive` |
| H3 — no native/primitive proof mechanism | **PARTIALLY RESOLVED** | P1 native tests assert owner/effects; equivalent P2 system-zone proof is missing |

## 3.3 Previous inventory/closure findings

| Previous finding | Current classification |
|---|---|
| M1 — old `System._$monotonicNanoseconds` census | **RESOLVED for monotonic clock** |
| M2 — wall-clock observation omitted from census | **RESOLVED** |
| M3 — final reflection/provenance/disassembly gate absent | **STILL OPEN as certification debt** |
| M4 — P1 body filename/checkpoint mismatch | **STILL PRESENT / low-priority record chore** |
| M5 — no CI status visible to audit | **AUDIT LIMITATION, not an implementation defect** |

## 3.4 Historical audit conclusion that must now be superseded

The old audit's conclusion that the source tree was “exactly at the P1→P2 boundary” is no longer true. The current tree is a **post-P2 implementation with incomplete closure/correctness**, not a P1-only implementation.

The old audit should therefore be retained only as historical evidence and explicitly superseded by this report for current-state decisions.

---

# 4. Classification scheme

This report uses the following classes.

| Class | Meaning |
|---|---|
| **BUG-HIGH** | User-visible semantic corruption, wrong identity, silent wrong time, or lossless-serialization failure that should be fixed before relying on the affected API for durable data |
| **BUG-MEDIUM** | Incorrect accepted/rejected input, inconsistent formatting/API behavior, or bounded semantics that are wrong in less common cases |
| **SURFACE** | Missing or accidentally exposed public standard-library API relative to C2's chosen public model |
| **ARCH** | Architectural debt/invariant mismatch that may not fail common tests today but makes correctness fragile |
| **VERIFY** | Implementation may be correct, but required evidence was never established |
| **PERF** | Avoidable cost/allocation/resource behavior without known semantic corruption |
| **DOC/LIFECYCLE** | Durable records disagree with the implementation or closure decision |
| **CHORE** | Cleanup, generation, stale warning/comment, naming, or low-risk maintenance work |
| **DEFERRED** | Explicitly deferred/non-goal work; not a defect in C2 |

---

# 5. High-severity correctness findings

## STDL002-AUD-001 — TimeZone handle identity is collision-prone

**Classification:** BUG-HIGH / ARCH  
**Status:** OPEN  
**Primary code:** `phalcom-core/src/timezone.rs::{identity_hash, register_handle}`  
**Affected public behavior:** `TimeZone.==`, `TimeZone.hash`, every native operation using the handle

Current code derives the guest-visible identity from a 64-bit hash:

```rust
pub(crate) fn identity_hash(&self, zone: &ZoneHandle) -> i64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    self.generation.hash(&mut hasher);
    zone.identity.hash(&mut hasher);
    hasher.finish() as i64
}

pub(crate) fn register_handle(&mut self, zone: ZoneHandle) -> i64 {
    let id = self.identity_hash(&zone);
    self.handles.insert(id, zone);
    id
}
```

The Phalcom object stores only that `i64`; source equality is:

```phalcom
self.identityNative == other.identityNative
```

This is not merely a theoretical hash-quality concern. A collision has two semantic effects:

1. two distinct zones compare equal and have the same hash;
2. `register_handle` overwrites the previous `handles[id]`, so an already-created `TimeZone` object can subsequently resolve offsets/local time/transitions through the newly inserted *different* zone.

That violates the strongest C2 invariant: an existing `TimeZone` must not silently change rule meaning during its lifetime.

### Required follow-up

Replace hash-as-identity with a collision-proof interned handle, for example:

```text
ZoneIdentity -> stable monotonically allocated HandleId
HandleId      -> ZoneHandle
```

or another injective VM-local registry. A hash may remain an indexing optimization, but it must not be the semantic identity key without collision checking.

Add an adapter-level test that forces an artificial collision strategy or uses a test-only identity allocator to prove distinct identities can never alias.

---

## STDL002-AUD-002 — Named-zone canonical identity is not canonical for case variants

**Classification:** BUG-HIGH  
**Status:** OPEN  
**Primary code:** `phalcom-core/src/timezone.rs::named`

Jiff's `TimeZoneDatabase::get` is case-insensitive. The backend can accept a spelling such as:

```text
america/new_york
```

and resolve it to the canonical database name `America/New_York`.

The adapter, however, chooses identity **before** backend normalization:

```rust
let canonical = iana_links::canonical_name(identifier).unwrap_or(identifier);
...
Ok(ZoneHandle {
    identity: ZoneIdentity::Named(canonical.to_owned()),
    zone,
})
```

If `canonical_name` has no exact-case map entry, the raw caller spelling becomes semantic identity. Therefore conceptually equivalent successful lookups can produce different `TimeZone` identities:

```text
TimeZone.named("America/New_York")
TimeZone.named("america/new_york")
```

This also makes `toString` caller-spelling dependent rather than database-canonical.

### Required follow-up

Canonicalize from the **resolved backend zone name**, then apply link normalization, instead of using raw input as identity. At minimum test:

```text
canonical case
lowercase canonical name
mixed-case canonical name
canonical alias
lowercase alias
```

All accepted spellings that denote the chosen canonical zone should converge to one identity.

---

## STDL002-AUD-003 — IANA alias canonicalization is incomplete and version-skewed

**Classification:** BUG-HIGH / CHORE  
**Status:** OPEN  
**Primary code:** `phalcom-core/src/iana_links.rs`

The checked-in file claims:

```rust
//! Generated from the IANA tzdb 2026a `backward` link file.
```

but it is only 36 lines long and contains 32 alias spellings across 23 match arms. The actual IANA `2026a` `backward` file contains a much larger compatibility-link set, including many names not represented here such as:

```text
Australia/ACT
Brazil/East
CET
GB
Japan
Turkey
America/Buenos_Aires
Africa/Accra
Europe/Amsterdam
Europe/Belfast
... many more ...
```

The historical P2 progress record explicitly said the map was a small manually authored subset; no generator is present in the live repository.

There is also a data-version mismatch. The lockfile uses Jiff `0.2.37` / `jiff-tzdb 0.1.8`. Jiff's 0.2.x release history updated the bundled tzdb to `2026c` in 0.2.32, while this alias file claims `2026a`.

Consequences:

- aliases outside the small map resolve successfully in Jiff but retain alias identity instead of canonical identity;
- two equivalent names can compare unequal;
- canonical `ZonedDateTime.toString` can depend on input spelling;
- the alias normalization layer can drift from the rules database it is supposedly identifying.

### Required follow-up

Generate the complete alias mapping from the exact tzdb version bundled by the locked `jiff-tzdb` dependency, or derive canonical identity through a backend API plus a complete link map. Generation must be reproducible and version-coupled.

Tests should sample aliases from every major `backward` section, not only `US/Eastern`.

---

## STDL002-AUD-004 — Unnamed system zones lose DST/rule semantics

**Classification:** BUG-HIGH / PORTABILITY  
**Status:** OPEN  
**Primary code:** `phalcom-core/src/timezone.rs::system`

Current fallback:

```rust
let system = TimeZone::try_system()?;
if let Some(identifier) = system.iana_name() {
    return self.named(identifier);
}
let _ = self.database.get_or_insert_with(TimeZoneDatabase::bundled);
let offset = system.to_offset(jiff::Timestamp::now()).seconds();
Ok(self.fixed(offset))
```

Jiff can discover a fully rule-bearing system timezone without an IANA identifier. A documented Unix case is `/etc/localtime` being a regular TZif file rather than a symlink. Jiff reads the TZif rules and can perform DST-safe arithmetic, but cannot name the zone.

Phalcom currently discards that rule-bearing `TimeZone` and replaces it with one fixed offset sampled at the current instant.

That means a system configured with an unnamed New-York-like TZif or POSIX DST rule can behave as:

```text
TimeZone.system == fixed(-04:00)  // if discovered in summer
```

and will then report `-04:00` for winter timestamps as well.

This violates the goal that `TimeZone.system` return an ordinary stable zone value representing the host's configured timezone rather than merely its current offset.

### Required follow-up

Add an adapter identity variant capable of retaining an unnamed but stable rule-bearing system zone, e.g. a VM-local opaque rule identity. Preserve the owned Jiff `TimeZone` rules for the lifetime of that Phalcom zone.

A fixed-offset fallback is appropriate only when the platform/backend genuinely provides only a fixed offset, not when a rule set is available but unnamed.

Also remove the unnecessary bundled-database initialization on the current fixed fallback path.

---

## STDL002-AUD-005 — Local civil resolution can silently wrap large years

**Classification:** BUG-HIGH  
**Status:** OPEN  
**Primary code:** `phalcom-core/src/primitive/time.rs::timezone_resolve_native`

The adapter first converts Phalcom integers to `i64`, then performs unchecked narrowing:

```rust
let datetime = jiff::civil::DateTime::new(
    *year as i16,
    *month as i8,
    *day as i8,
    *hour as i8,
    *minute as i8,
    *second as i8,
    *nanosecond as i32,
)
```

Phalcom's P1 civil year is not represented as `i16`. An in-range `i64` value can wrap through `as i16` *before* Jiff checks its own `-9999..=9999` civil range. For example, a year congruent to `0` modulo 65536 can become year `0` and produce a valid but semantically unrelated timestamp.

This is worse than a range rejection: it can silently return the wrong instant.

### Required follow-up

Use checked conversions (`i16::try_from`, etc.) and then explicitly validate the Jiff-supported civil range before constructing the backend datetime. The error must state that the value is outside the timezone-backend interop range; it must never wrap.

Add boundary tests at:

```text
-10000
-9999
9999
10000
32767
32768
65535
65536
```

and representative huge Phalcom `Int` values.

---

## STDL002-AUD-006 — UTC/fixed operations incorrectly inherit Jiff's finite range

**Classification:** BUG-HIGH / UNSATISFIED C2 INVARIANT  
**Status:** OPEN  
**Affected code:** all `TimeZone` native fact paths

P2 explicitly separated P1's arbitrary-precision `Timestamp`/civil domain from the finite backend interop domain. Named IANA rules may legitimately reject values outside Jiff's range. UTC and fixed offsets do not require IANA transition data and should continue to work wherever P1 values can be converted by integer/Gregorian arithmetic.

Current implementation does not preserve that layering.

Every offset/local/transition path begins by converting the Phalcom timestamp to `jiff::Timestamp`:

```rust
let timestamp = timestamp_from_value(vm, &args[0])?;
```

This is used even for:

```text
TimeZone.utc
a fixed TimeZone
```

Similarly, `DateTime.resolve` always constructs a Jiff `civil::DateTime`, even for UTC/fixed zones.

Consequences:

- `ZonedDateTime.from(hugeTimestamp, TimeZone.utc).date` fails at Jiff's range instead of using P1 Gregorian arithmetic;
- fixed-offset projections fail outside Jiff's range;
- `transitionBefore/After` on UTC/fixed zones can report a backend-range error instead of simply `None`;
- UTC/fixed `DateTime.resolve` is artificially narrowed even though it is arithmetic and unambiguous.

### Required follow-up

Introduce explicit UTC/fixed fast paths before backend timestamp/civil conversion. Named/rule zones may remain bounded by Jiff; UTC/fixed must use exact P1 arithmetic.

This is a substantive unsatisfied library goal, not only a missing test.

---

## STDL002-AUD-007 — Fractional formatting is not nanosecond-lossless

**Classification:** BUG-HIGH / SERIALIZATION  
**Status:** OPEN  
**Primary code:** `time/calendar/time.ph`, `time/timestamp.ph`

The P2 grammar requires exactly nine fractional digits whenever the nanosecond field is nonzero. Current formatting directly interpolates the integer:

```phalcom
const fraction = if self.nanosecond == 0 { "" } else { ".\(self.nanosecond)" }
```

and Timestamp does the same for `nanos`.

This creates a semantic round-trip failure. For example:

```text
actual value: 00:00:00.000000001   (1 ns)
current text: 00:00:00.1
```

But `Time.parse("00:00:00.1")` right-pads one fractional digit to nine digits, interpreting it as:

```text
00:00:00.100000000   (100 ms)
```

The formatter therefore does not merely violate a cosmetic canonical-width rule; it changes the represented value when parsed according to the same library's parser.

The same defect propagates to:

```text
DateTime.toString
Timestamp.toString
ZonedDateTime.toString
```

Existing tests use `123_000_000`, which happens to already render as nine digits and therefore miss the defect.

### Required follow-up

Add a shared nine-digit nanosecond formatting helper and test:

```text
1
10
1000
1_000_000
12_345_678
100_000_000
999_999_999
```

for `Time`, `DateTime`, `Timestamp`, and `ZonedDateTime`.

---

## STDL002-AUD-008 — Expanded and negative year text contract is not implemented

**Classification:** BUG-HIGH / SERIALIZATION  
**Status:** OPEN

The intended machine grammar is:

```text
0000..9999  -> exactly four digits
negative    -> '-' + absolute year padded to at least four digits
>9999       -> '+' + at least five digits
```

Current `Date.toString` does:

```phalcom
const yearText = "\(self.year)"
```

and `Timestamp.toString` repeats the same behavior.

Examples therefore become:

```text
year 0      -> 0-01-01       (expected 0000-01-01)
year 1      -> 1-01-01       (expected 0001-01-01)
year -1     -> -1-01-01      (expected -0001-01-01)
year 10000  -> 10000-01-01   (expected +10000-01-01)
```

`Date.parse` is also incompatible with the planned grammar:

```phalcom
const parts = text.split("-")
```

A leading negative sign creates an extra split element, so `-0001-01-01` is rejected before integer parsing. Positive expanded years with `+` are rejected because `parseInteger` only recognizes `-`.

This breaks the library's claimed proleptic/expanded-year textual closure even before considering Timestamp/Zoned parsers.

### Required follow-up

Implement a sign-aware date lexer/parser rather than delimiter splitting, plus one canonical year formatter shared by Date/Timestamp/Zoned forms.

---

# 6. Missing standard-library surface and unsatisfied library goals

## STDL002-AUD-009 — `Timestamp.parse` is missing

**Classification:** SURFACE / HIGH  
**Status:** OPEN

The incident that originally blocked parser work was later resolved: canonical data class-side signatures now publish correctly and the negative corpus reaches the expected argument mismatch. Nevertheless, the final integrated source contains no `Timestamp.parse`.

Missing behavior includes:

```text
canonical ...Z parsing
RFC3339-style explicit numeric-offset parsing
normalization to absolute Timestamp
fraction 1..9 digit handling
pre-epoch parsing
expanded-year handling within supported semantics
```

This leaves `Timestamp.toString` without its required inverse and prevents full temporal serialization closure.

---

## STDL002-AUD-010 — `ZonedDateTime.parse` and offset/zone consistency are missing

**Classification:** SURFACE / HIGH  
**Status:** OPEN

No `ZonedDateTime.parse` exists.

Therefore the library does not implement the most important zoned serialization safety law:

```text
<DateTime><numeric offset>[<zone-id>]
```

must verify that the explicit offset is one of the offsets valid for that named zone at the parsed civil time/instant.

The missing behavior includes:

```text
named-zone bracket parsing
UTC/fixed bracket parsing
explicit offset/zone mismatch rejection
overlap disambiguation by explicit offset
gap rejection/consistency rules
canonical alias normalization during parse
parse(format(zoned)) == zoned
```

The current formatter can produce bracketed zoned text, but that text is not a supported self-round-tripping serialization format.

---

## STDL002-AUD-011 — `Timestamp.in(zone)` and `TimeZone.at(timestamp)` are missing

**Classification:** SURFACE / MEDIUM  
**Status:** OPEN

The live library can construct the same value manually:

```phalcom
ZonedDateTime.from(timestamp, zone)
```

but the planned symmetric convenience surface is absent:

```text
Timestamp.in(zone)
TimeZone.at(timestamp)
```

These are not necessary for semantic correctness, but their absence leaves the standard library surface incomplete relative to the chosen C2 model and removes the natural inverse-oriented APIs alongside `DateTime.resolve/inZone`.

---

## STDL002-AUD-012 — `time.testing` does not export its classes from the package root

**Classification:** SURFACE / MEDIUM  
**Status:** OPEN

Current package:

```phalcom
// time/testing/package.ph
expose .clocks
```

Current tests import:

```phalcom
from universe.time.testing.clocks import ManualClock, ManualWallClock
```

The intended public testing package was conceptually:

```phalcom
from time.testing import ManualClock, ManualWallClock
```

The classes exist, but the package root does not import/export them. This turns an implementation module name (`clocks`) into part of the public import path and means the intended `time.testing` surface is not fully published.

### Follow-up

Make `time.testing` re-export both classes while retaining `.clocks` as an internal/public child only if desired.

---

## STDL002-AUD-013 — Public offset lookup leaks `Int` instead of `UtcOffset`

**Classification:** SURFACE / API INCONSISTENCY  
**Status:** OPEN

`TimeZone` exposes:

```phalcom
offsetSeconds(_ timestamp: Timestamp) -> Int
```

This is used internally by:

```phalcom
ZonedDateTime.offset -> UtcOffset
```

but it is itself public. The C2 model said that if direct public offset lookup is exposed, it should be a temporal value:

```text
TimeZone.offset(timestamp) -> UtcOffset
```

The current public `Int` method exposes representation detail and duplicates the conversion responsibility already encapsulated by `UtcOffset`.

### Follow-up

Either:

1. make the seconds-returning method package/internal and expose `offset(timestamp) -> UtcOffset`; or
2. remove direct public offset lookup if `ZonedDateTime.offset` is the only intended public path.

---

# 7. Additional semantic/API inconsistencies

## STDL002-AUD-014 — Fixed-zone display is not the canonical UtcOffset spelling

**Classification:** BUG-MEDIUM / TEXT

`TimeZoneAdapter::display_name` renders fixed zones as:

```rust
+HH:MM:SS
```

unconditionally.

`UtcOffset.toString` omits `:SS` when seconds are zero. Therefore a zoned value can produce:

```text
...+01:00[+01:00:00]
```

instead of the preferred self-consistent:

```text
...+01:00[+01:00]
```

There is no fixed-zone zoned-format fixture, so this inconsistency is currently untested.

---

## STDL002-AUD-015 — `Period.parse` accepts trailing garbage

**Classification:** BUG-MEDIUM / PARSER

Current parser finds the first `D` but never verifies that it is the final character:

```phalcom
const dayEnd = text.indexOf("D")
...
const dayText = text.slice(monthEnd + 1, dayEnd)
```

Thus a string conceptually like:

```text
P1M2Dgarbage
```

can parse as `P1M2D` instead of being rejected.

Add an exact-end check and negative fixtures for trailing bytes, duplicate designators, and extra whitespace if the grammar is intended to be strict.

---

## STDL002-AUD-016 — System timezone discovery lacks required primitive/effect metadata

**Classification:** ARCH / PROVENANCE / HIGH  
**Status:** OPEN

The runtime host observation exists:

```phalcom
@class
internal native systemNative -> TimeZone
```

but it lacks:

```phalcom
@primitive
```

and the generated native surface records its effects as:

```text
EffectSpec::Unknown
```

The C2/LANG006 boundary explicitly classified system timezone discovery as an irreducible host/configuration observation:

```text
ImplementationKind::Native
+ @primitive metadata
```

The core floor census currently lists only:

```text
SystemClock.monotonicNanoseconds
SystemWallClock.unixNanoseconds
```

and omits the system timezone observation.

### Follow-up

Add source `@primitive`, give the provider an honest nondeterministic host-observation effect contract (and any additional current effect atom justified by the effect model), add a native surface/provenance test, and update the floor census.

---

## STDL002-AUD-017 — Gap payload wording in the plan is inverted relative to the implemented/Jiff behavior

**Classification:** DOC/SEMANTIC INCONSISTENCY  
**Status:** OPEN RECORD DECISION

The live implementation maps Jiff:

```text
Gap.before = AmbiguousZoned.earlier()
Gap.after  = AmbiguousZoned.later()
```

For New York `2024-03-10 02:30` this produces:

```text
before -> 01:30 -05:00
 after -> 03:30 -04:00
```

This is intuitive and matches the library's policy:

```text
Earlier -> before
Later/Compatible -> after
```

However, the P2 plan text described `Gap.before` as “the requested local datetime interpreted with the pre-transition offset.” In a forward gap, applying the pre-transition offset to the nonexistent local time actually yields the **later/after** resolved instant. Conversely, the post-transition offset yields the **earlier/before** instant.

So the plan's prose and the implemented/Jiff semantics use opposite offset descriptions.

### Recommendation

Keep the intuitive implemented payload behavior, but correct the durable semantic wording in a future authoritative spec/decision record:

```text
Gap.before = resolved zoned value immediately on the earlier side of the gap
            (obtained by applying the post-transition offset to the requested nonexistent civil time)

Gap.after  = resolved zoned value on the later side of the gap
            (obtained by applying the pre-transition offset)
```

This is primarily a documentation/design inconsistency, not a reason to swap the live policy.

---

## STDL002-AUD-018 — Zero fixed offset is silently collapsed into UTC identity

**Classification:** OPEN DECISION / LOW-MEDIUM

`TimeZoneAdapter::fixed(0)` assigns:

```rust
ZoneIdentity::Utc
```

so:

```text
TimeZone.fixed(UtcOffset.utc) == TimeZone.utc
```

and it serializes as `UTC`, losing the fact that the caller requested a fixed numeric zone.

This may be a sensible canonicalization, but it is not explicitly recorded in the current C2 identity model, which described UTC and fixed zones separately.

The project should explicitly decide whether:

```text
UTC == fixed(+00:00)
```

as `TimeZone` structural identity, rather than leaving it as an adapter implementation accident.

---

## STDL002-AUD-019 — Native TimeZone payload uses an undeclared slot-0 convention

**Classification:** ARCH / MEDIUM

Native construction currently does:

```rust
let mut instance = InstanceObject::new(class, vm.heap.class(class).field_count.max(1));
instance.slots[0] = Value::int(identity);
```

but `class TimeZone` declares no source field corresponding to slot 0.

This works today and does not require a custom temporal heap object, but it is brittle:

- reflection/source layout does not document the stored native handle;
- if `TimeZone` later gains a real first source field, slot 0 can collide with it;
- the native provider is relying on implicit layout knowledge rather than a declared/native-payload contract.

A future runtime/native-surface cleanup should provide an explicit opaque native payload mechanism or a source-declared private handle field with supported native initialization semantics.

---

# 8. Range and backend-boundary issues

## 8.1 Backend Timestamp conversion is otherwise correctly signed

`timestamp_from_value` correctly handles negative nanosecond remainder by converting to Euclidean second+nanos form before constructing Jiff `Timestamp`. This is a positive implementation point and should be retained.

## 8.2 Named zones correctly report backend-range failure instead of narrowing P1 state

For arbitrary `Timestamp` values that cannot fit Jiff, named-zone operations return an `ArgumentError` rather than changing `Timestamp`'s canonical representation. That is the correct layering for named rule zones.

The defect is specifically that **UTC/fixed** routes currently share this bounded conversion when they need not.

## 8.3 Civil narrowing needs an explicit boundary helper

The year-cast defect is a symptom of the lack of one canonical conversion function:

```text
Phalcom DateTime -> Result<jiff::civil::DateTime, BackendRangeError>
```

All named-zone local-resolution entry points should go through a checked helper rather than direct casts inside the native provider.

---

# 9. Textual-form completeness audit

## 9.1 Date

| Requirement | Current state |
|---|---|
| `YYYY-MM-DD` ordinary year | mostly works for four-digit years |
| zero-pad years 0000..9999 | **missing** |
| negative expanded year | **formatter wrong; parser broken** |
| `+10000` expanded year | **formatter/parser missing** |
| component validation | implemented via `Date.new` |
| exact grammar/no junk | mostly strict after split, but sign grammar inadequate |

## 9.2 Time

| Requirement | Current state |
|---|---|
| HH/MM/SS two digits | implemented |
| zero fraction omitted | implemented |
| nonzero exactly 9 digits | **broken for leading-zero nanoseconds** |
| parser accepts 1..9 digits | implemented |
| >9 rejected | implemented |
| formatter/parser roundtrip | **not true for many valid nanoseconds** |

## 9.3 DateTime

Composition exists and parser is source-owned. It inherits all Date and Time defects.

## 9.4 UtcOffset

This is one of the stronger textual surfaces:

```text
+HH:MM
-HH:MM
+HH:MM:SS
-HH:MM:SS
```

with component and final bound validation. No major correctness defect was found in this slice.

## 9.5 Timestamp

Formatter exists but has:

```text
expanded-year defect
fraction-padding defect
```

and parser is absent.

## 9.6 ZonedDateTime

Formatter exists but inherits:

```text
Date year defect
Time fraction defect
alias/canonical-name defect
fixed bracket spelling inconsistency
```

Parser is absent, so offset/zone consistency and overlap-offset selection do not exist.

## 9.7 Duration

The chosen machine spelling:

```text
PT<signed nanoseconds>NS
```

is unusual but lossless and internally symmetric with the current parser. The P2 plan explicitly allowed implementation to choose the exact machine spelling after inspecting source capabilities, so this is not classified as a defect.

## 9.8 Period

The chosen canonical shape:

```text
P<signed months>M<signed days>D
```

preserves mixed signs and round-trips values emitted by `toString`. The parser's trailing-garbage acceptance remains a bug.

---

# 10. Native floor and native-surface audit

## 10.1 P1 floor is now correct

The two P1 host observations are correctly narrow:

```text
SystemClock.monotonicNanoseconds
SystemWallClock.unixNanoseconds
```

Both are:

```text
Native
source @primitive
native effect = Nondeterminism
internal
return Int
```

and native tests assert ownership/effects.

The earlier audit's P1 metadata findings are fully retired.

## 10.2 System timezone floor is execution-complete but metadata-incomplete

The actual host-discovery provider exists and uses `TimeZone::try_system()`, so there is no missing runtime capability.

The remaining problem is classification:

```text
source @primitive      missing
native effect contract Unknown
floor census row       missing
focused provenance test missing
```

Thus the **runtime native floor is functionally present**, but the **declared/provenance floor is incomplete**.

## 10.3 Ordinary timezone database operations correctly remain non-primitive

The following are native provider operations without source `@primitive`, which is the intended distinction:

```text
utcNative
fixedNative
namedNative
offsetSecondsNative
localNative
resolveNative
transitionBeforeNative
transitionAfterNative
displayNative
identityNative
```

This is architecturally correct: Rust/Jiff implementation does not make an operation part of the irreducible primitive floor.

## 10.4 Public backend type leakage

No Jiff type appears in the public Phalcom source surface. Native fact transport uses `Int`, `String`, and `List<Int>` and source materializes Phalcom values/enums.

That core C2 ownership invariant is satisfied.

---

# 11. Timezone identity audit

## 11.1 UTC

UTC has its own `ZoneIdentity::Utc` and does not load named tzdb. Good.

The unresolved question is whether fixed zero and named UTC aliases should structurally converge to this identity. Fixed zero currently does; named `UTC`/`Etc/UTC` does not necessarily.

## 11.2 Fixed zones

Equal nonzero fixed offsets derive the same hash identity and therefore compare equal in normal non-collision operation. Good in ordinary cases.

The handle hash must still be replaced for collision safety.

## 11.3 Named zones

The intended semantic model was:

```text
one DB snapshot + canonical zone identifier
```

The current implementation effectively uses:

```text
one VM generation (constant 1)
+ manually normalized or raw caller spelling
+ hash thereof
```

That is not equivalent because normalization is incomplete and case-sensitive.

## 11.4 Generation field

`generation` is initialized to `1` and never changes. Since handles are VM-local and backend data is bundled/immutable, it currently adds no meaningful discrimination inside a VM. It is harmless but effectively decorative.

If future code supports database generations/reload, generation semantics need a real lifecycle. If not, the field can be removed from identity construction once collision-proof interning is implemented.

---

# 12. System timezone audit

## 12.1 Correct properties

Current `TimeZone.system`:

- performs discovery each call rather than returning a mutable singleton;
- uses fallible Jiff system discovery rather than silently choosing UTC;
- when an IANA name is available, resolves it through the VM's bundled named database, yielding stable rules for that returned object;
- returns an ordinary Phalcom `TimeZone` object.

Those are strong properties.

## 12.2 Rule-loss bug on unnamed systems

As described in STDL002-AUD-004, unnamed but rule-bearing system zones are collapsed to fixed offset. This is the main system-zone correctness defect.

## 12.3 Extra host-now read

The fallback calls:

```rust
system.to_offset(jiff::Timestamp::now())
```

rather than using the P1 `wallClock` observation. This is not a second public “now” API, but it is an additional host time read inside system-zone discovery.

Once unnamed rule zones are preserved directly, this extra observation should disappear entirely.

## 12.4 Portability verification gap

There is no committed integration test covering at least:

```text
named system zone
unnamed /etc/localtime TZif
POSIX TZ rule
explicit discovery failure or unsupported platform
```

The implementation therefore has not established its intended cross-platform behavior.

---

# 13. Local-resolution and ZonedDateTime semantic audit

## 13.1 Unique/fold/gap factual boundary

The Rust adapter preserves all three factual states. Jiff borrowed/backend types do not leak into public values. Good.

## 13.2 Disambiguation policy

The source policy matches the expected table and is centralized in `LocalTimeResolution.resolve`. Good.

## 13.3 Non-hour transitions

Adapter tests include a Lord Howe 30-minute gap. This is good evidence that the implementation is not accidentally assuming every transition is one hour.

## 13.4 Historical second precision

The adapter includes a historical Europe/Paris transition test, but it merely asserts that offsets differ. It does not assert the exact second-resolution offsets and therefore does not fully prove `P2-CV-13`'s intended historical-second behavior.

## 13.5 `shift(Duration)`

Implemented correctly as timestamp-coordinate movement. A New York DST fixture verifies the civil hour changes after 24 exact hours.

## 13.6 `advance(Period)`

Implemented using civil DateTime + `Constrain` + `Compatible`, which is consistent with the intended semantic model.

However, the committed positive corpus does **not** test `advance` at a DST boundary and does not establish the required law:

```text
start.shift(Duration.hours(24)) != start.advance(Period.days(1))
```

This is a verification gap around a core semantic distinction.

## 13.7 Equality and sameInstant

Structural equality uses timestamp + timezone equality. `sameInstant` and `compareInstant` use timestamp only. Good, subject to the TimeZone identity defects described earlier.

## 13.8 Hash consistency

There is no explicit `ZonedDateTime.hash` method in the source. If ordinary data hashing derives from fields, it should match structural equality because the equality fields are exactly the data fields. This should nevertheless be explicitly tested, especially after TimeZone identity changes.

---

# 14. Certification and verification gaps

These are not all runtime bugs, but they were explicit C2 completion goals and remain valuable post-closure backlog.

## STDL002-AUD-020 — No final ordinary-package P2 certification

**Classification:** VERIFY / HIGH

`phalcom-core/tests/core/modules/runtime.rs` contains P1 ordinary-package tests only. No final ordinary project fixture was found that:

```text
imports TimeZone/ZonedDateTime normally
uses grouped imports
pattern-matches LocalTimeResolution
uses time.testing
parses/formats P2 values
```

The language corpus primarily exercises Universe fixtures and therefore does not replace the ordinary-package authority proof.

---

## STDL002-AUD-021 — No P2 reflection/category certification

**Classification:** VERIFY

No focused reflection test was found asserting the public categories of:

```text
TimeZone                  class
ZoneTransition            data
LocalTimeResolution       payload enum
LocalTimeDisambiguation   enum
ZonedDateTime             data
ManualClock               class
ManualWallClock           class
```

The general reflection machinery may work, but C2 never recorded the intended proof.

---

## STDL002-AUD-022 — No forced-GC zone/handle survival certification

**Classification:** VERIFY / ARCH

The adapter's GC ownership is structurally sensible: it stores Rust-owned Jiff values and no guest `ObjRef`, and VM GC explicitly classifies it as non-root.

But no test was found that forces collection around:

```text
named TimeZone
multiple objects sharing one handle
ZonedDateTime
LocalTimeResolution payloads
ZoneTransition
```

and then reuses those values.

Given the handle registry's role in every zone operation, a forced-GC test remains worthwhile even though no immediate root bug is evident.

---

## STDL002-AUD-023 — No final P2 native provenance test

**Classification:** VERIFY / PROVENANCE

P1 native tests assert clock ownership and nondeterminism. Equivalent P2 assertions are absent.

A final test should assert:

```text
TimeZone.systemNative
    Native + source @primitive + host-observation effect

named/rule operations
    Native + no source @primitive
```

and verify no public `System.timeZone` seam exists.

---

## STDL002-AUD-024 — System discovery is untested

**Classification:** VERIFY / PORTABILITY

No focused system timezone success/failure test was found. `timezone_now.ph` exercises `ZonedDateTime.now`, but only checks the returned type and is inherently environment dependent; it does not certify discovery semantics.

Add a platform-controlled seam or process-level `TZ` fixtures where supported.

---

## STDL002-AUD-025 — Backend-range behavior is untested

**Classification:** VERIFY

There are no focused tests for:

```text
named Timestamp below/above Jiff range
named DateTime year below/above Jiff range
no wraparound
UTC/fixed outside Jiff range
```

This absence helped the unchecked `i16` year cast and UTC/fixed narrowing escape detection.

---

## STDL002-AUD-026 — Transition public surface is weakly tested

**Classification:** VERIFY

The public fixture verifies only that `transitionAfter(Timestamp.epoch)` returns `Some`. It does not assert:

```text
exact transition timestamp
offsetBefore
offsetAfter
transitionBefore
historical second offsets
fixed/UTC -> None
```

---

## STDL002-AUD-027 — Parse/format property matrix is incomplete

**Classification:** VERIFY / HIGH

There is no bounded property-style matrix establishing:

```text
parse(format(x)) == x
```

across temporal values and boundary cases.

The fractional formatting defect is precisely the sort of bug this gate would have caught.

---

## STDL002-AUD-028 — Lazy-init/cache/resource certification is partial

**Classification:** VERIFY / PERF

Adapter unit tests establish that `utc()` and `fixed(0)` do not initialize the bundled DB and that one alias pair shares the named cache.

Still missing as final integration evidence:

```text
actual `import time` init count
first named lookup vs second named lookup
system named vs unnamed behavior
shared handles across guest-object collection
adapter/handle memory sanity
no database reopen during repeated projections
```

---

## STDL002-AUD-029 — No current full affected-suite certification record

**Classification:** VERIFY / LIFECYCLE

The historical progress/incident records contain focused PASS commands, but there is no final C2 completion record mapping `P2-CV-01..70` to tests/results and no final affected-suite run record after the integrated commit.

This does not prove current tests fail. It means the requested final certification was never durably recorded.

---

# 15. Performance and efficiency audit

## STDL002-AUD-030 — Zoned projections repeat native lookup and allocate Lists

**Classification:** PERF / MEDIUM

`ZonedDateTime` obtains local civil fields through:

```phalcom
internal localParts -> List<Int> {
  self.timeZone.localNative(self.timestamp.unixNanoseconds)
}
```

Each call allocates a guest `List` with seven integers.

Then:

```phalcom
dateTime -> DateTime { DateTime.new(self.date, self.time) }
```

calls `localParts` twice—once for `date`, once for `time`. `toString` also derives `date`, `time`, and `offset` separately. `advance` calls `dateTime`, which repeats the local conversion.

This does not reopen the database, but it creates avoidable native dispatches and guest allocations on common projection/formatting paths.

### Follow-up

At source level, fetch `localParts` once per compound operation:

```phalcom
const parts = self.localParts
```

and derive Date+Time from one fact set. If the language later gains an efficient private tuple/record transport, replace the mutable-looking `List<Int>` bridge with a fixed-shape value.

Do not solve this by adding a temporal VM object.

---

## STDL002-AUD-031 — System fixed fallback initializes bundled DB unnecessarily

**Classification:** PERF / CHORE

In the no-IANA-name path, the implementation does:

```rust
let _ = self.database.get_or_insert_with(TimeZoneDatabase::bundled);
```

then discards the database result and returns a fixed zone.

This violates the spirit of “do not initialize named tzdb for fixed-zone behavior” and incurs a potentially material one-time resource cost exactly when the returned value does not use that database.

Fixing the larger unnamed-rule-zone bug should remove this line naturally.

---

## STDL002-AUD-032 — Manual alias layer adds maintenance cost on every tzdb update

**Classification:** PERF/MAINTENANCE CHORE

The alias map creates a second piece of timezone-database metadata that must stay synchronized with `jiff-tzdb`. Because it is currently manual and version-skewed, every dependency update risks semantic identity drift.

The long-term solution should make alias generation part of dependency/update tooling rather than a hand-maintained runtime match.

---

# 16. Lifecycle and repository-record inconsistencies

## STDL002-AUD-033 — C2 checkpoint state is stale

**Classification:** DOC/LIFECYCLE / HIGH RECORD INCONSISTENCY

The committed `STDL002.C2-CHECKPOINT.md` still says:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
```

and its top plan table still describes P1/P2 implementation as not started.

That directly contradicts both the live source and the current administrative decision that STDL002 is closed and implementation complete.

Since the user has chosen to close STDL002 for now, the appropriate follow-up is a **record correction**, not a prerequisite to more implementation work. The checkpoint should distinguish:

```text
implementation_status: COMPLETE/CLOSED_FOR_NOW
verification_status: PARTIAL / POST-CLOSURE-DEBT
known_followups: link to this audit
```

or whatever lifecycle vocabulary the project currently uses.

---

## STDL002-AUD-034 — Canonical final P1/P2 walkthrough/handoff artifacts are absent

**Classification:** DOC/LIFECYCLE

The C2 directory contains progress/incident records but no final canonical:

```text
STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md
STDL002.C2.P2-walkthrough.md
STDL002.C2.P2-handoff.md
```

The integrated incident record explicitly said it was not a completion handoff.

If the program remains administratively closed, the missing final records can be represented by a compact post-closure handoff that points to this audit instead of reconstructing the entire original T11 process.

---

## STDL002-AUD-035 — Existing committed post-P2 audit is now stale

**Classification:** DOC/LIFECYCLE

The previous `STDL002.C2.P2-audit.md` says P2, Jiff, TimeZone, ZonedDateTime, and `time.testing` are absent. Those statements were true for the older audited revision but are false at current `main`.

It should be marked historical/superseded rather than left as an apparently current implementation audit.

---

## STDL002-AUD-036 — Alias-map “generated” comment is misleading

**Classification:** CHORE / DOC

No generator was found, and the table is visibly only a small subset of the claimed IANA `backward` input. The comment should not say “Generated from ...” unless generation is reproducible and complete.

---

## STDL002-AUD-037 — P1 plan body filename bookkeeping remains inconsistent

**Classification:** CHORE

The checkpoint historically refers to an installed `STDL002.C2.P1-01-body.md`, while the live file naming uses `STDL002.C2.P1-plan-01-body.md`. This has no runtime impact but should be normalized when lifecycle records are touched.

---

# 17. Open design decisions that should not be mistaken for bugs

## 17.1 UTC versus fixed `+00:00` identity

Decide explicitly whether these are one `TimeZone` identity or distinct structural values with equivalent rules.

Current implementation collapses them.

## 17.2 Anonymous system-zone identity

To fix rule-loss for unnamed TZif/POSIX system zones, the library needs a stable identity model that does not depend on an IANA name. Options include:

```text
opaque per-discovery rule identity
content-derived collision-proof rule identity
VM-interned anonymous system zone handle
```

The public `toString`/serialization story for such zones also needs a deliberate policy because there is no lossless IANA name.

## 17.3 Public offset lookup

Choose whether `TimeZone` publicly exposes:

```text
offset(timestamp) -> UtcOffset
```

or leaves offset projection exclusively on `ZonedDateTime`. Do not keep `offsetSeconds -> Int` public accidentally.

## 17.4 Naming style for labeled conveniences

The plan described forms such as:

```text
Date.today(in: zone)
ZonedDateTime.now(in: zone)
DateTime.in(zone, disambiguation)
```

The implementation uses:

```text
todayIn
nowIn
inZone
```

The P2 plan allowed adaptation to live syntax, so this is not automatically a defect. It is nevertheless worth stabilizing once Phalcom's final labeled-selector conventions are settled.

## 17.5 Numeric projections from arbitrary-precision values

`Duration.seconds`, `Timestamp.seconds`, etc. return `Float`. Canonical state remains arbitrary-precision `Int`, so this does not corrupt stored values, but very large projections lose precision.

The project should eventually decide whether the stdlib should also expose exact quotient/remainder or rational/integer projections for serialization/numeric work. This was not a C2 blocker and is not classified as a current bug.

## 17.6 Error taxonomy

Parsing, unknown zones, backend-range failures, ambiguity rejection, and system discovery mostly use `ArgumentError`. The P2 plan intentionally deferred dedicated temporal error types unless the general model proved inadequate.

This remains an ergonomics decision, not a violation by itself.

---

# 18. Missing/unsatisfied C2 goal matrix

This table distinguishes **implemented**, **implemented with defect**, **missing surface**, and **uncertified**.

| Goal | Current state | Audit classification |
|---|---|---|
| Duration exact ns | implemented | satisfied |
| Clock/Instant domains | implemented | satisfied |
| Timestamp arbitrary Int state | implemented | satisfied |
| Wall clock capability | implemented | satisfied |
| Gregorian civil values | implemented | satisfied |
| Period months+days | implemented | satisfied |
| UtcOffset seconds | implemented | satisfied |
| Jiff backend | implemented | satisfied |
| bundled stable named DB | implemented | substantially satisfied |
| lazy named DB | implemented | satisfied except unnecessary system fallback init |
| named cache | implemented | satisfied structurally |
| collision-proof handle | **not implemented** | BUG-HIGH |
| complete canonical alias identity | **not implemented** | BUG-HIGH |
| UTC zone | implemented | satisfied |
| fixed zone | implemented | range layering defect |
| named zone | implemented | identity defects |
| system zone | implemented | unnamed-rule loss + provenance gap |
| transition before | implemented | weakly tested |
| transition after | implemented | weakly tested |
| Unique/fold/gap facts | implemented | satisfied |
| all disambiguation policies | implemented | satisfied |
| Gap semantics | implemented sensibly | plan wording inconsistent |
| Timestamp.in(zone) | **missing** | SURFACE |
| TimeZone.at(timestamp) | **missing** | SURFACE |
| DateTime.resolve | implemented | satisfied |
| default Reject | implemented | satisfied |
| Zoned canonical state | implemented | satisfied |
| offset/date/time/dateTime projections | implemented | perf + range issues |
| shift(Duration) | implemented | satisfied |
| advance(Period) | implemented | uncertified at DST boundary |
| shift vs advance law | implementation likely correct | required test missing |
| withTimeZone | implemented | satisfied |
| sameInstant/compareInstant | implemented | satisfied subject to zone identity |
| Date.today | implemented | satisfied subject to system zone |
| ZonedDateTime.now | implemented | satisfied subject to system zone |
| Date format | implemented | expanded-year bug |
| Time format | implemented | fractional bug |
| DateTime format | implemented | inherits bugs |
| UtcOffset format | implemented | satisfied |
| Timestamp format | implemented | year/fraction bugs |
| Zoned format | implemented | year/fraction/zone-id bugs |
| Duration format/parse | implemented | satisfied for chosen grammar |
| Period format/parse | implemented | trailing-junk parser bug |
| Date parse | implemented | expanded/negative-year bug |
| Time parse | implemented | satisfied |
| DateTime parse | implemented | inherits Date bug |
| UtcOffset parse | implemented | satisfied |
| Timestamp.parse | **missing** | SURFACE/HIGH |
| ZonedDateTime.parse | **missing** | SURFACE/HIGH |
| offset-zone consistency | **missing** | unsatisfied goal |
| overlap parse by explicit offset | **missing** | unsatisfied goal |
| full parse/format roundtrip | **not satisfied** | BUG/VERIFY |
| ManualClock | implemented | satisfied |
| ManualWallClock | implemented | satisfied |
| `time.testing` root exports | **missing** | SURFACE |
| ordinary-package P2 certification | **missing** | VERIFY |
| reflection certification | **missing** | VERIFY |
| system-zone primitive provenance | **incomplete** | ARCH/VERIFY |
| forced GC certification | **missing** | VERIFY |
| no temporal VM value/opcode specialization | static audit passes | satisfied |
| resource/perf sanity | partial | VERIFY/PERF |
| pinned deterministic timezone tests | partly via bundled DB | partial |
| system integration smoke | missing | VERIFY |
| final P2 coverage ledger | missing | DOC/VERIFY |
| final lifecycle records | missing/stale | DOC/LIFECYCLE |
| authoritative time spec | intentionally deferred | DEFERRED, not defect |

---

# 19. Test gaps to add in a future maintenance checkpoint

The following test set is deliberately ordered by bug-finding value.

## 19.1 Identity tests

```text
TimeZone.named("America/New_York") == TimeZone.named("america/new_york")
TimeZone.named("US/Eastern") == TimeZone.named("America/New_York")
TimeZone.named("Australia/ACT") == TimeZone.named("Australia/Sydney")
TimeZone.named("Turkey") == TimeZone.named("Europe/Istanbul")
TimeZone.named("GB") == TimeZone.named("Europe/London")
```

Add a test-only collision allocator or collision-injection seam to prove handle identity cannot overwrite another zone.

## 19.2 Backend range tests

```text
DateTime year 9999 named zone -> supported if Jiff permits
DateTime year 10000 named zone -> explicit backend range error
DateTime year 65536 named zone -> error, never year 0
huge Timestamp + TimeZone.utc -> correct source arithmetic
huge Timestamp + fixed offset -> correct source arithmetic
huge Timestamp + named zone -> explicit backend range error
```

## 19.3 Text round-trip tests

For each of Time/DateTime/Timestamp/ZonedDateTime:

```text
nanosecond = 1
nanosecond = 10
nanosecond = 100
nanosecond = 1_000
nanosecond = 1_000_000
nanosecond = 12_345_678
nanosecond = 123_000_000
nanosecond = 999_999_999
```

For Date/Timestamp year formatting:

```text
-10000
-1
0
1
9999
10000
```

## 19.4 Parser strictness

```text
Period.parse("P1M2Dgarbage") -> reject
Date.parse("-0001-01-01") -> correct negative date
Date.parse("+10000-01-01") -> correct expanded date
fraction > 9 -> reject
numeric timestamp offset -> normalize
zoned invalid offset -> reject
zoned overlap each valid offset -> select exact occurrence
```

## 19.5 System timezone tests

Where platform control permits:

```text
TZ=America/New_York -> named stable zone
TZ=US/Eastern -> canonical identity
TZ=EST5EDT,... -> preserve rules, not fixed current offset
regular-file /etc/localtime fixture -> preserve TZif rules
invalid/undiscoverable config -> explicit failure
```

## 19.6 Zoned laws

```text
shift(24h) != advance(1 day) at spring-forward
shift(24h) != advance(1 day) at fall-back where appropriate
withTimeZone preserves Timestamp
sameInstant true across different zones
ordinary equality false across different zones
hash agrees with equality
```

## 19.7 Transitions

Assert exact known:

```text
transitionBefore
transitionAfter
timestamp
offsetBefore
offsetAfter
non-hour transition
historical second-resolution transition
UTC/fixed no transition
```

## 19.8 GC

Force GC while retaining:

```text
one named TimeZone
one second TimeZone sharing identity/handle
one ZonedDateTime
one LocalTimeResolution payload
one ZoneTransition
```

Then perform offset/local/transition operations again.

---

# 20. Chores and low-risk cleanup

These should not distract from correctness work, but they are worth recording.

| Chore | Rationale |
|---|---|
| replace misleading “Generated from 2026a” alias comment or add actual generator | current claim is not reproducible |
| remove/justify `generation` if no DB reload model exists | currently constant `1` |
| centralize canonical year/fraction helpers | avoid Date/Time/Timestamp divergence |
| reuse Gregorian conversion instead of duplicate `Timestamp.civilFromDays` algorithm | reduce semantic drift |
| fetch zoned local parts once in compound operations | reduce native sends/List allocations |
| expose `ManualClock`/`ManualWallClock` from testing package root | intended public ergonomics |
| normalize P1 plan-body filename references | lifecycle hygiene |
| mark old P2 audit superseded | prevents stale diagnosis |
| add documentation to testing package | discoverability |
| record exact Jiff/tzdb data version in durable backend handoff | reproducibility |

---

# 21. Explicitly deferred/non-goal work — do not misclassify as missing

The following were outside the chosen C2 contract or explicitly deferred and should **not** be treated as implementation bugs in this audit:

```text
authoritative time specification rewrite (USER-DEFERRED)
locale/CLDR formatting
natural-language date parsing
non-Gregorian calendars
leap-second time scale support
recurrence/cron/scheduling rules
timer/reactor redesign
broad Temporal trait hierarchy
new temporal VM Value variants or bytecodes
global test-clock replacement
public System.now/System.timeZone facade
```

Likewise, Jiff's finite named-zone/civil range is acceptable as an **interop boundary**. The defect is only where that finite range silently corrupts or unnecessarily narrows Phalcom's wider source domains.

---

# 22. Recommended post-C2 backlog order

Because STDL002 is closed, this section is phrased as future maintenance/next-checkpoint priority rather than a demand to reopen it.

## Priority A — correctness and durable-data safety

1. Replace hash-derived TimeZone handles with collision-proof interning.
2. Fix checked civil-year/backend conversion; eliminate `as i16` wrapping.
3. Fix nine-digit fractional formatting in Time/Timestamp and all composed forms.
4. Fix canonical/expanded Date/Timestamp year formatting and Date parsing.
5. Complete canonical named-zone identity: case normalization + complete version-coupled IANA links.
6. Preserve unnamed system-zone rules rather than collapsing to current offset.
7. Restore UTC/fixed arithmetic outside Jiff's finite range.

These are the issues most likely to create silently wrong values, incorrect equality, or non-round-trippable serialized data.

## Priority B — missing C2 public surface

8. Implement `Timestamp.parse`.
9. Implement `ZonedDateTime.parse` with offset-zone conflict rejection and overlap selection.
10. Add `Timestamp.in(zone)` and `TimeZone.at(timestamp)` conveniences.
11. Publish `ManualClock`/`ManualWallClock` from `time.testing` root.
12. Hide `offsetSeconds` representation detail or replace with `UtcOffset` public API.

## Priority C — native boundary/provenance

13. Mark `TimeZone.systemNative` with source `@primitive`.
14. Give it explicit nondeterministic host-observation effect metadata.
15. Add it to the core floor census.
16. Add focused TimeZone native provenance tests.

## Priority D — certification

17. Add final ordinary-package P2 fixture.
18. Add reflection category tests.
19. Add forced-GC/shared-handle tests.
20. Add system discovery integration tests.
21. Add complete range tests.
22. Add parse/format property tests.
23. Add transition-before/exact transition tests.
24. Add shift-vs-advance DST tests.
25. Record resource/lazy/cache sanity evidence.

## Priority E — lifecycle/chore

26. Correct the stale C2 checkpoint to reflect the chosen administrative closure and post-closure debt.
27. Add a compact final post-closure handoff linking this audit.
28. Mark the previous P2 audit historical/superseded.
29. Normalize plan filenames/comments/generation tooling.

---

# 23. Architectural strengths to preserve

The remaining issues do **not** justify replacing the current architecture wholesale. The following choices remain strong and should be preserved while fixing the defects:

```text
Duration = exact source-owned nanosecond data
Timestamp = source-owned signed Int POSIX nanoseconds
Instant is distinct from Timestamp
Period is distinct from Duration
DateTime is timezone-free
UtcOffset is distinct from TimeZone
ZonedDateTime canonical state = Timestamp + TimeZone
ambiguity is explicit
policy is source-owned
backend facts are Native, not public Jiff types
host clock observations are narrow primitives
system timezone discovery is not a public System facade
only Duration/Timestamp are temporal prelude promotions
bundled named database is lazy and VM-owned
no temporal Value variants/opcodes were introduced
ManualClock/ManualWallClock do not mutate production globals
```

The implementation's main problems are boundary correctness and incomplete closure, not a failed core model.

---

# 24. Final classification summary

## 24.1 Real correctness bugs

```text
STDL002-AUD-001  hash-derived TimeZone identity collision
STDL002-AUD-002  case/canonical-name identity divergence
STDL002-AUD-003  incomplete/version-skew IANA alias normalization
STDL002-AUD-004  unnamed system zone loses DST/rules
STDL002-AUD-005  unchecked civil-year narrowing/wrap
STDL002-AUD-006  UTC/fixed wrongly narrowed to backend range
STDL002-AUD-007  nanosecond formatter changes meaning on round trip
STDL002-AUD-008  expanded/negative year grammar broken
STDL002-AUD-014  fixed zone canonical bracket spelling mismatch
STDL002-AUD-015  Period parser trailing-junk acceptance
```

## 24.2 Missing standard-library surface

```text
STDL002-AUD-009  Timestamp.parse
STDL002-AUD-010  ZonedDateTime.parse + offset/zone consistency
STDL002-AUD-011  Timestamp.in / TimeZone.at
STDL002-AUD-012  time.testing root exports
STDL002-AUD-013  proper public UtcOffset lookup (or hide Int representation)
```

## 24.3 Architecture/provenance issues

```text
STDL002-AUD-016  TimeZone.systemNative missing primitive/effect/floor classification
STDL002-AUD-017  Gap payload wording inconsistent with implementation/Jiff semantics
STDL002-AUD-018  fixed +00:00 vs UTC identity not explicitly decided
STDL002-AUD-019  undeclared slot-0 native handle representation
```

## 24.4 Verification/certification debt

```text
STDL002-AUD-020  no final ordinary-package P2 proof
STDL002-AUD-021  no P2 reflection-category proof
STDL002-AUD-022  no forced-GC/handle survival proof
STDL002-AUD-023  no final P2 native provenance proof
STDL002-AUD-024  no system timezone integration proof
STDL002-AUD-025  no backend-range boundary proof
STDL002-AUD-026  weak transition public-surface proof
STDL002-AUD-027  no complete parse/format property matrix
STDL002-AUD-028  partial lazy/cache/resource proof
STDL002-AUD-029  no final coverage-ledger/affected-suite record
```

## 24.5 Performance/maintenance debt

```text
STDL002-AUD-030  repeated zoned local conversion/List allocation
STDL002-AUD-031  needless bundled DB init in system fixed fallback
STDL002-AUD-032  manual alias layer update burden
```

## 24.6 Lifecycle/record debt

```text
STDL002-AUD-033  checkpoint still IN_PROGRESS/PARTIAL/UNVERIFIED
STDL002-AUD-034  final canonical walkthrough/handoff artifacts absent
STDL002-AUD-035  prior committed audit now stale
STDL002-AUD-036  alias file inaccurately claims complete generation
STDL002-AUD-037  P1 plan-body filename bookkeeping inconsistency
```

---

# 25. Final assessment

STDL002's closed implementation has achieved the most important architectural objective: Phalcom now has a coherent source-first temporal model spanning exact duration, monotonic clock domains, wall-clock timestamps, Gregorian civil values, fixed offsets, IANA named zones, ambiguity, zoned datetimes, current-time conveniences, deterministic test clocks, and a narrow Rust/Jiff rule backend without introducing temporal VM value kinds or public backend types.

That is materially stronger than the earlier post-P2 audit reported, because that audit predated the integration commit.

The remaining debt is nevertheless substantial enough that consumers should not yet treat all textual and timezone behavior as a finished serialization/identity contract. In particular:

- do not rely on canonical string round trips for subsecond values with leading-zero nanoseconds;
- do not rely on expanded/negative-year text;
- there is no Timestamp/Zoned parser contract yet;
- named-zone equality is not fully canonical across all aliases/case spellings;
- the current 64-bit hashed handle is not a sound semantic identity mechanism;
- unnamed system zones can lose their transition rules;
- very large civil years can be silently mis-resolved through native narrowing;
- UTC/fixed zones do not yet preserve the wider P1 range that C2 intended.

The best follow-up strategy is therefore **not a timezone redesign**. It is a focused post-C2 hardening sequence that preserves the existing model while making identity collision-proof, range conversions checked and layered, textual forms genuinely lossless, the missing parsers/public conveniences complete, and the unperformed ordinary-package/reflection/GC/provenance certifications explicit.

STDL002 can remain administratively closed while those items are owned by a future stdlib hardening checkpoint or consolidated defect program.

