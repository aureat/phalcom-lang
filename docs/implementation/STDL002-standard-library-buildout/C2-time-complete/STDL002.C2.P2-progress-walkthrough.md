---
id: STDL002.C2.P2-progress-walkthrough
program: STDL002
checkpoint: STDL002.C2
plan: STDL002.C2.P2
kind: implementation-progress-walkthrough
status: PARTIAL
completion: PARTIAL
verification: BASELINE_BLOCKED
prepared: 2026-09-20
---

# STDL002.C2.P2 Progress Walkthrough

This is an interim progress walkthrough, not a completed P2 plan walkthrough.
It records the live implementation state after the runtime-audit fixes and the
initial timezone backend/source work. It must not be used as evidence that P2
or C2 is complete.

## Executive Status

```text
P1 temporal foundation       substantially implemented through publication/T7
P1 final T8 closure          not completed
P1 final walkthrough         missing
P1 final handoff             missing
P2 backend spike             partially implemented and focused-tested
P2 public timezone surface   partially authored, not complete
P2 zoned semantics           only an incomplete source scaffold
P2 text parsing/formatting   absent
P2 time.testing              absent
P2 final certification       absent
C2 checkpoint                IN_PROGRESS / PARTIAL / UNVERIFIED
```

The current branch is `main` at the last pushed checkpoint:

```text
e5851099 fix(stdlib): remove redundant time type guards
```

The working tree also contains uncommitted timezone work. That work is not
part of the pushed checkpoint and is not currently build-green.

## Completed And Pushed

### Runtime audit checkpoint

Commit `8e4551b9` was committed and pushed to `origin/main` at the user's
request. It contains the runtime audit remediation tree that was present at
that point, including the associated runtime/compiler/resource changes and
implementation records.

### P1 temporal foundation

The live source contains the substantial P1 temporal substrate:

- `Duration` with signed exact nanosecond state and source arithmetic.
- `Measurement<T>` as ordinary generic data.
- `Clock`, `SystemClock`, `Instant`, and the canonical `clock` capability.
- `Timestamp`, `WallClock`, `SystemWallClock`, and the canonical `wallClock` capability.
- `Timestamp.now` composed through `wallClock` rather than a second host-now primitive.
- Gregorian `Month`, `Weekday`, `Date`, `Time`, and `DateTime` source values.
- `YearMonth`, `MonthDay`, `Period`, and `DateOverflow` source values.
- `UtcOffset` with signed second precision and the approved inclusive bound `-93_599..=93_599`.
- Ordinary package publication and a prelude containing only `Duration` and `Timestamp`.
- No temporal `Value` variants, temporal bytecodes, or temporal compiler specialization.

### P1 native-floor corrections

The pushed follow-up commit `e5851099` contains these audit fixes:

- Removed redundant static `x is T` checks from typed P1 time APIs.
- Preserved `Dynamic` equality checks and semantic range/validity checks.
- Added source `@primitive` metadata to the two irreducible host observations:
  `SystemClock.monotonicNanoseconds` and `SystemWallClock.unixNanoseconds`.
- Changed both observations from `Pure` to `Nondeterminism` in native metadata.
- Regenerated the native surface catalog.
- Updated the floor census to record the current `SystemClock` and
  `SystemWallClock` ownership instead of the retired `System._$` seam.

### Verified focused evidence

These commands passed at the pushed P1 checkpoint or during the corresponding
focused correction:

```text
cargo test -p phalcom-core --test core native_surface_contracts::time_ -- --nocapture
cargo test -p phalcom-core --test core modules_runtime::time_p1_ -- --nocapture
cargo test -p phalcom-core --test language-corpus corpus::time -- --nocapture
cargo test -p phalcom-core timezone --lib -- --nocapture
cargo test -p phalcom-modules --test builtin_provider builtin_universe_provider_has_virtual_identity_and_exports -- --nocapture
```

The timezone adapter unit lane passed four tests while the adapter was being
developed: lazy construction, alias identity, fold/gap facts, and copied
transition facts.

## P2 Work Completed So Far

The following work exists only in the current uncommitted WIP tree and is not
yet a stable implementation milestone.

### Backend and VM ownership

- Added direct Jiff `0.2.37` workspace dependency with bundled timezone data.
- Added a Rust-only `TimeZoneAdapter` owned by `VM`.
- Added exhaustive GC classification for the adapter as non-root Rust state.
- Kept guest heap handles out of the adapter design.
- Added lazy database construction so import, UTC, and fixed-offset construction
  do not initialize the named database.
- Added cached named-zone backend values and a VM generation identifier.
- Added owned local-resolution facts for unique, fold, and gap cases.
- Added owned transition facts with before/after offsets.

### Alias identity decision

Jiff preserves link spellings such as `US/Eastern` and
`America/New_York` in `iana_name()`. The plan's alias identity invariant
therefore required a decision. The approved direction is a pinned generated
IANA-link map, with semantic identity based on snapshot generation plus the
canonical identifier.

The current WIP contains only a small manually authored subset of that map.
It is not yet a complete generated tzdb link map and cannot be certified as
complete alias coverage.

### Initial source/native surface

The WIP adds or begins adding:

- `TimeZone` source declaration and package export.
- `ZoneTransition` source data declaration.
- `ZonedDateTime` source data scaffold with `Timestamp + TimeZone` state.
- `UniverseKey::TimeZone` and source-path/name mappings.
- Builtin Universe module graph/source embedding for the new time modules.
- Native factory declarations for UTC, fixed, named, and system forms.
- Native surface regeneration for the TimeZone declarations.

An intermediate ordinary-package run passed after the TimeZone module was
embedded and its generated native declarations were refreshed. Subsequent
zoned/handle edits have not been re-certified.

### Local-resolution facts

The current WIP also adds a factual local-resolution boundary:

- `LocalTimeDisambiguation` and `LocalTimeResolution` source enums are
  published through `universe.time.resolution`.
- The VM adapter exposes unique/fold/gap facts using Jiff's raw ambiguity
  result.
- `TimeZone.resolveNative(...)` returns an encoded list of timestamp facts,
  keeping backend enums and borrowed Jiff values private to Rust.
- `DateTime.resolve(zone)` constructs the public factual resolution values from
  that encoded result.

The parser-compatible labeled payload syntax is now known and the source
policy method compiles with the four requested policy branches. Runtime
coverage now includes a pinned `America/New_York` fixture for a unique local
time, a one-hour fall-back overlap, a one-hour spring-forward gap, and the
Earlier/Later/Compatible policies. The fixture asserts the resulting offsets.
The default Reject overlap fixture is also present. Adapter tests additionally
cover the 30-minute `Australia/Lord_Howe` transition and a historical
second-precision `Europe/Paris` transition.

## Remaining P1 Work

P2 cannot be considered fully unblocked until these P1 lifecycle obligations
are completed:

1. Finish P1 T8 stabilization and certification.
2. Resolve or explicitly classify the semantic argument-type gap exposed by
   `Duration.seconds(1.5)`. The semantic checker currently accepts a Float
   literal against a declared `Int` parameter in the affected path.
3. Reconcile the negative fixture expectation with semantic argument checking;
   do not restore redundant runtime type guards.
4. Run the final P1 reflection, native provenance, disassembly/no-specialization,
   concurrency compatibility, and cheap-import gates.
5. Produce the canonical `STDL002.C2.P1-walkthrough.md`.
6. Produce the canonical `STDL002.C2.P1-handoff.md` with actual signatures,
   ranges, paths, native linkage, visibility, fixture conventions, focused
   tests, and baseline failures.
7. Update the C2 checkpoint to truthful P1 completion/verification state.

## Remaining P2 Work

The following remain blockers to declaring all of C2 complete:

- Canonical parsing for civil, offset, duration, period, timestamp, and zoned
  forms is not implemented.
- `Timestamp` and `ZonedDateTime` canonical formatting still need completion.
- The IANA alias table is a focused audited subset, not a complete generated
  tzdb link map.
- Full P1 T8/P2 certification and canonical lifecycle walkthrough/handoff
  records are not complete.
- The aggregate negative corpus remains blocked by the independent semantic
  argument-type defect exposed by `Duration.seconds(1.5)`.
- A focused semantic regression confirmed that `Duration.seconds(1.5)` reaches
  analysis with no diagnostic at all; the remaining defect is signature
  publication/resolution for the source-owned class-side method, not the
  relation engine or runtime guard removal.

## Latest Verification Snapshot

The latest inline verification produced:

```text
cargo check -p phalcom-core                              PASS
cargo test -p phalcom-core timezone --lib               PASS (6 tests)
cargo test -p phalcom-core core modules_runtime::time_p1_ PASS (3 tests)
cargo test -p phalcom-core language-corpus corpus::time PASS (positive time corpus)
cargo test -p phalcom-core language-corpus corpus::time_negative BLOCKED
```

The negative-lane failure is specifically:

```text
time/negative/duration_fractional.ph unexpectedly succeeded
```

The core crate emits only a dead-code warning for the adapter generation
accessor; no compile error remains in the current focused implementation.

### Backend and native boundary

- Complete and verify the full pinned/generated IANA alias map.
- Replace ad-hoc identity hashing with a stable guest-visible logical handle
  contract that cannot collide and survives all required operations.
- Finish timestamp conversion from P1 arbitrary-precision nanoseconds into the
  backend's bounded interop range, with explicit range errors.
- Implement native offset lookup against the stored logical zone handle.
- Add native named lookup, system discovery, local-resolution, and transition
  provider contracts with explicit provenance/effect metadata.
- Verify system discovery failure is explicit and never silently becomes UTC.
- Add native adapter tests for cache sharing, range failures, historical second
  offsets, system discovery, and snapshot stability.

### TimeZone and transition semantics

- Complete `TimeZone` equality/hash for UTC, fixed, named, and system values.
- Complete `TimeZone.at`, offset lookup, and transition-before/after source APIs.
- Materialize `ZoneTransition` values from copied adapter facts.
- Ensure UTC/fixed operations remain independent of named tzdb initialization.
- Add invalid-name and system-discovery error behavior.

### Local civil resolution

- Add `LocalTimeResolution` with `Unique`, `Ambiguous`, and `Gap` payloads.
- Add `LocalTimeDisambiguation` with `Reject`, `Earlier`, `Later`, and
  `Compatible` policies.
- Add focused runtime fixtures for `DateTime.resolve` and `DateTime.inZone`
  using one-hour, non-hour, and historical transitions.
- The Reject fixture is present; the combined negative lane remains blocked by
  the pre-existing `duration_fractional` semantic-type failure.
- Verify gap payload meaning for one-hour, non-hour, and historical transitions.
- Add pinned overlap/gap fixtures and all four policy tests.

### Zoned semantics

- `ZonedDateTime` offset, `Date`, and `Time` projections are now implemented
  through a private native local-component fact path.
- `shift(Duration)` is implemented as exact timestamp-coordinate movement and
  passes a DST fixture: 24 hours from New York 2024-03-09 12:00 becomes March
  10 at local hour 13.
- Timezone identity is now exposed through a stable adapter handle: canonical
  aliases and equal fixed offsets compare equal, while `sameInstant` and
  `compareInstant` remain explicit timestamp-only operations.
- A positive fixture now verifies alias equality, equal fixed-zone equality,
  exact 24-hour DST shifting, and timestamp-only instant comparison.
- `Date.today`, `Date.todayIn(zone)`, `ZonedDateTime.now`, and
  `ZonedDateTime.nowIn(zone)` now compose `Timestamp.now` with timezone
  interpretation; the positive corpus verifies their returned types.
- Added `time.testing.clocks` with deterministic `ManualClock` and
  `ManualWallClock`; the fixture verifies independent advancement, wall-clock
  setting, and unchanged production globals.
- Added source-owned canonical formatting for `Date`, `Time`, `DateTime`,
  `UtcOffset`, `Duration`, and `Period`; the formatting fixture passes.
- Added adapter-backed canonical timezone display for UTC, fixed offsets, and
  named zones; native catalog and positive time tests pass.
- Added canonical UTC `Timestamp` formatting and bracketed
  `ZonedDateTime` formatting; deterministic epoch/New York output passes.
- Added `ZoneTransition.new` materialization and positive transition lookup
  coverage through `TimeZone.transitionAfter`.
- Added source-owned canonical parsing for exact nanosecond `Duration` and
  signed-field `Period` forms, with positive round-trip and invalid-input
  fixtures.
- Added source-owned `UtcOffset.parse` for signed `HH:MM` and `HH:MM:SS`
  forms, including invalid-field coverage.
- Added source-owned `Date.parse` and `Time.parse`, including fractional
  nanosecond right-padding and invalid component coverage.
- Added source-owned `DateTime.parse`, composing the canonical Date and Time
  parsers with strict `T` separation.
- `Timestamp.parse` was evaluated but intentionally deferred: the current
  source class-resolution rule resolves cross-class parser calls against the
  receiver class, so adding it would require a semantic call-resolution fix
  rather than a local timestamp patch.
- `DateTime`, `ZonedDateTime`, and local-resolution declarations are now
  co-located in the canonical calendar datetime module, with thin public
  re-export modules. This removes the source initialization cycle without
  foreign inherent impls.
- `ZonedDateTime.dateTime` and `advance(Period)` are now implemented using the
  existing Compatible resolution policy.
- Implement `withTimeZone`, `sameInstant`, and timestamp-only comparison.
- Add explicit structural equality based on timestamp plus timezone identity.
- Add DST tests demonstrating `Duration.hours(24)` differs from `Period.days(1)`.

### Current-time conveniences

- Implement `Date.today` and `Date.today(in:)` through `Timestamp.now`.
- Implement `ZonedDateTime.now` and `ZonedDateTime.now(in:)` through the
  canonical wall clock.
- Do not add a public `System` timezone façade or a second host-now primitive.

### Canonical text

- Canonical formatting for all listed source-owned temporal values is now
  implemented and covered by the positive formatting fixture. Parsing remains
  open.
- Implement parsing with expanded years, fractional nanoseconds, numeric
  offsets, zoned bracket identifiers, and mixed-sign Period round trips.
- Validate explicit offset/zone consistency during zoned parsing.
- Reuse the live general error model or consult before adding public error types.
- Add lossless parse/format and invalid-input fixtures.

### Deterministic testing support

- Add `time.testing` package.
- Implement independent `ManualClock` with deterministic monotonic ticks.
- Implement `ManualWallClock` with deterministic `Timestamp`, advance, and set.
- Do not replace global `clock` or `wallClock` and do not add global overrides.

### Final C2 certification

- Ordinary-package use of the complete P2 surface.
- Grouped import/export and prelude stability.
- Reflection categories for TimeZone, ZoneTransition, resolution enums, and
  ZonedDateTime.
- Native provenance proving primitive versus ordinary Native boundaries.
- GC survival and shared-handle lifetime tests.
- No temporal VM specialization evidence.
- Lazy-init, cache/resource, and performance sanity checks.
- Pinned deterministic timezone fixtures independent of ambient host tzdata.
- Canonical P2 walkthrough and handoff.
- C2 checkpoint update only after all gates pass.

## Current Blocking State

The current WIP tree now compiles through the core crate and the ordinary
package publication path after the timestamp/handle repair. The source layer
is still intentionally incomplete: local resolution, complete transition
materialization, text, testing clocks, and final P2 certification have not
landed. Therefore:

```text
P2 implementation: PARTIAL
P2 verification:   FOCUSED_TESTED (backend/publication slice only)
C2 completion:     NOT COMPLETE
```

The pushed tree remains safe to use at `e5851099`. The uncommitted WIP must be
completed or discarded by the next implementation pass; it must not be
described as a finished P2 milestone.

## Next Ordered Action

Complete the P1 T8 lifecycle evidence and handoff, then continue P2 from the
green backend/publication slice. The next implementation slice is local
resolution and transition materialization, followed by complete zoned
semantics, text, and `time.testing`.
