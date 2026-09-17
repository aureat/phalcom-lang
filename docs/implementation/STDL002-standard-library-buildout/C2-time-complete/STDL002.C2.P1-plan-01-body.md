# 18. Tasks

This body continues `STDL002.C2.P1-plan-00-shared-header.md` and intentionally does not repeat sections 0–17. The shared header remains the plan-wide architecture and invariant authority; this file owns task order, task-local mechanics, verification gates, completion evidence, and lifecycle deliverables.

## 18.0 Body audit, approved deviations, and execution interpretation

The attached checkpoint, guidance, and shared header are mutually consistent on the P1 architecture. This body makes four explicit execution-time specializations after auditing them against the live planning baseline.

### P1-BODY-DEV-01 — existing `time.md` documents are excluded from P1

The user has explicitly ruled that the previously discovered `time.md` specification/design material is stale and must not be included in this work, and that authoritative language specification work for `time` is currently deferred.

This explicit ruling supersedes every P1 instruction in the checkpoint, guidance, or shared header that would otherwise require reading, updating, or certifying against an existing `time.md`, including references to:

```text
docs/spec/current/stdlib/time.md
docs/implementation/time.md
```

Consequences for this body:

1. no P1 task reads either file as semantic authority;
2. no P1 task edits, replaces, or attempts to rehabilitate either file;
3. no P1 acceptance gate requires a `time` specification update;
4. `P1-CV-50` remains a required agreement check, but its P1 evidence is **plan/API agreement**: the implemented public surface must agree with the user-ratified C2 checkpoint, shared header, this body, and executable source/semantic tests;
5. the stale specification leg of the original `P1-CV-50` wording is recorded as **USER-DEFERRED**, not as a P1 failure;
6. the C2 checkpoint must record this as an approved amendment so later sessions do not accidentally revive the stale files as authority;
7. P2 must not assume that specification work is automatically re-enabled merely because older planning text assigned final specification to P2. A later explicit instruction may reintroduce specification work.

This changes documentation evidence only. It does not weaken any `P1-INV-*` semantic invariant.

### P1-BODY-DEV-02 — future visibility and implementation-kind semantics are the target

The execution baseline currently contains older visibility/attribute/primitive mechanics. The user has explicitly stated that the ratified future semantics described by the checkpoint/header will be in place by the time P1 is implemented.

Therefore:

- T0 still verifies their live landing state because mechanics and paths may have changed;
- current legacy behavior is never used to downgrade the target design;
- P1 does not create a compatibility architecture for the old model;
- T1 does **not** become a project to implement the whole visibility, `native`, or `primitive` transition;
- if those general mechanisms are unexpectedly incomplete at execution time, P1 consumes already-ratified mechanical completion only when ownership is clear; otherwise it STOPs and CONSULTs under the shared-header triggers.

### P1-BODY-DEV-03 — body artifact filename

This delivered body uses the user-requested filename:

```text
STDL002.C2.P1-01-body.md
```

instead of the shared header's earlier mechanical filename `STDL002.C2.P1-plan-01-body.md`. This is a naming-only deviation. Repository cross-references should be updated to the installed filename rather than creating two competing body files.

### P1-BODY-DEV-04 — representative P1 integration versus final P2 certification

The checkpoint assigns final ordinary-package/reflection/integration closure to P2, while the shared header correctly requires P1-level evidence that the P1 substrate already works through ordinary package and reflection paths. This body preserves both statements:

- P1 must prove representative ordinary-package consumption, reflection categories, primitive ownership, and concurrency compatibility for the P1 surface;
- P2 still owns final C2 certification across timezone/zoned/textual additions.

P1 therefore proves that its substrate is real and ordinary; it does not certify future P2 surfaces.

## 18.1 Task graph

Execute in this order unless a STOP/CONSULT trigger prevents progress:

```text
T0  live takeover, baseline, and capability map
 ↓
T1  conditional general-language prerequisite closure
 ↓
G1  prerequisite readiness
 ↓
T2  Duration + Measurement foundation
 ↓
T3  monotonic capability migration
 ↓
G2  monotonic stabilization
 ↓
T4  absolute wall-time vertical
 ↓
G3  absolute-time stabilization
 ↓
T5  Gregorian arithmetic + Month/Weekday/Period foundation
 ↓
T6  validated civil values + UtcOffset
 ↓
G4  civil/calendar stabilization
 ↓
T7  package/prelude/ordinary-package integration
 ↓
T8  concurrency/reflection/provenance/runtime-architecture closure
 ↓
G5  P1 integration stabilization
 ↓
§20 final focused acceptance
```

BUILD mode remains sparse. Do not use a later gate to justify broad testing after every task.

---

## T0 — Re-ground the live repository and establish the P1 takeover record

### Purpose

Establish the exact execution HEAD, preserve unrelated work, verify that the ratified language mechanisms needed by P1 have landed, locate the live files/symbols that replaced planning-time paths, establish focused baseline behavior, and activate the checkpoint without changing production behavior.

This task is the implementation session's bounded takeover pass, not a new architecture analysis.

### Preconditions

- Read `AGENTS.md` and the workflow documents required by the shared header.
- Read `STDL002.C2-CHECKPOINT.md`, `STDL002.C2-GUIDANCE.md`, `STDL002.C2.P1-plan-00-shared-header.md`, and this body.
- Treat `P1-BODY-DEV-01` through `P1-BODY-DEV-04` as explicit user-authorized plan amendments.
- Do not read existing `time.md` documents as authority.

### Consumes

- all `P1-INV-*` invariants as the target architecture;
- all `P1-CV-*` coverage IDs as the eventual acceptance ledger;
- shared-header VERIFY-FIRST table in §12.3;
- current predecessor implementation as migration input only.

### Produces

A concise implementation-session takeover record, preferably in the P1 walkthrough as it is created, containing:

```text
execution branch
execution HEAD
planning baseline HEAD
working-tree state
unrelated local edits that must be preserved
active task = T0
live language capability matrix
live source/linkage/test path map
focused baseline test result classification
approved documentation-spec deferral amendment
```

The capability matrix must classify each required mechanism as one of:

```text
LANDED
LANDED_WITH_MECHANICAL_DRIFT
PARTIALLY_LANDED_BUT_OWNED_ELSEWHERE
MISSING_BUT_P1_AUTHORIZED_NARROW_CLOSURE
BLOCKING_ARCHITECTURAL_MISMATCH
```

### Files and symbols

Read-only unless performing checkpoint bookkeeping:

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/sol-principal-architect-project-doctrine.md

STDL002.C2-CHECKPOINT.md
STDL002.C2-GUIDANCE.md
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md

phalcom-core/core/universe/src/time/package.ph
phalcom-core/core/universe/src/time/clock.ph
phalcom-core/core/universe/src/package.ph
phalcom-core/core/universe/src/concurrency/fiber.ph

phalcom-core/src/primitive/system.rs
phalcom-core/src/universe/primitives.rs

phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/impl_decl.rs

phalcom-semantic/tests/semantic/capabilities/time.rs
phalcom-core/tests/fixtures/language/time/**
phalcom-core/tests/fixtures/language/time_negative/**
phalcom-core/tests/core/mod.rs
phalcom-core/tests/core/native/contracts.rs
phalcom-core/tests/core/reflection/type_metadata.rs
phalcom-core/tests/core/modules/runtime.rs
```

Additionally locate the live execution-HEAD equivalents for:

```text
grouped import/export parser + AST + module tests
private/protected/module/package/internal visibility semantics
protected subclass eligibility
native implementation kind
primitive implementation kind
@intrinsic / @primitive metadata
bodyless abstract class members
inherited abstract obligations
concrete-subclass completeness
data validation / constructor access
generic data identity
class-side singleton storage
source-declared primitive linkage to ordinary Universe classes
arbitrary-precision negative division/modulo semantics
```

Planning-baseline facts that must be rechecked rather than blindly assumed include:

- declaration-only class members existed syntactically but compiler lowering still rejected `MemberBody::Declaration` in `class_decl.rs` / `impl_decl.rs`;
- the host monotonic read was still registered on `System` under the legacy `_$monotonicNanoseconds` selector;
- the old time tests still used `System.clock` and legacy access conventions.

### Required implementation shape

1. Run exactly:

   ```sh
   git status --short
   git branch --show-current
   git rev-parse HEAD
   git log -5 --oneline
   ```

2. Compare execution HEAD with planning revision `e0a77646c56f8b1a7e7692993d378d43bf3fc424`.
3. If the branch or HEAD has moved, inspect only commits/files affecting the P1 premises; do not re-audit unrelated repository areas.
4. Verify the language mechanisms listed above against effective current language authority and code/tests.
5. Record the exact live file/symbol names that T1–T8 will use.
6. Verify the existing focused test filters actually select tests before relying on them later.
7. Update the C2 checkpoint to mark P1 active and record the execution start revision.
8. Record `P1-BODY-DEV-01` in the checkpoint's consultation/amendment area: existing `time.md` documentation is excluded and authoritative time-spec work is deferred.
9. Do not change production time behavior in T0.

### Forbidden approaches

- no broad repository audit;
- no production source edits;
- no `git reset`, `git clean`, mass formatting, or broad staging;
- no attempt to “fix” current old visibility semantics inside time;
- no reading stale `time.md` files to fill design gaps;
- no new compatibility layer for legacy selector spellings;
- no full-workspace test run.

### Test changes required

None. T0 establishes baseline evidence only.

### Tests to run now

First confirm that the focused filters still select tests, then establish a narrow predecessor baseline:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
```

If an execution-HEAD test target was renamed, adapt the mechanical target only after locating the replacement; record the exact command used.

### Tests explicitly deferred

```text
full phalcom-semantic
full phalcom-core
all language corpus
workspace tests
clippy
LSP/extension suites
P2 timezone/zoned tests
```

### Acceptance

T0 is accepted when:

- execution HEAD and working-tree ownership are known;
- every VERIFY-FIRST language premise has a concrete status;
- exact live time/primitive/test paths are recorded;
- focused baseline behavior is classified;
- no production source has changed;
- the checkpoint says P1 is active at the correct revision;
- the stale-spec deferral is durably recorded.

### Local STOP / CONSULT triggers

STOP and CONSULT if:

- a ratified visibility/implementation-kind semantic has materially changed, not merely moved syntactically;
- the execution tree requires time-specific compatibility architecture to use future language semantics;
- the data model or abstract-member model contradicts the shared header rather than merely lacking final plumbing;
- unrelated local edits overlap the exact P1 source in a way that makes ownership ambiguous;
- the execution HEAD contains a competing C2/P1 implementation or plan with incompatible decisions.

### Checkpoint update

Record:

```text
P1 active
start revision
P1-BODY-DEV-01 documentation/spec deferral amendment
baseline focused commands/results
any C/D predecessor failures that matter to later gates
```

Do not record transient file-path discoveries as durable architecture unless they become stable public/ownership facts.

---

## T1 — Close only the minimal general-language prerequisites that are actually missing

### Purpose

Make the general language capable of expressing the already-ratified P1 design where execution HEAD still lacks a small, clearly owned prerequisite. T1 is conditional and may complete with **no production code changes**.

The expected candidate closures are:

1. bodyless abstract class-member obligations and concrete-subclass completeness;
2. a general validated-`data` construction/access mechanism sufficient for `Date`, `Time`, `MonthDay`, and `UtcOffset` without sacrificing data transparency.

Visibility, `native`, `primitive`, grouped imports/exports, and metadata semantics are expected to be supplied by the general language transition before P1. P1 consumes them; it does not independently redesign or reimplement them.

### Preconditions

- T0 capability matrix complete.
- Any missing mechanism is classified as `MISSING_BUT_P1_AUTHORIZED_NARROW_CLOSURE`.
- The relevant accepted language design is already clear enough that implementation is mechanical rather than architectural.

### Consumes

Primarily:

```text
P1-INV-08
P1-INV-15
P1-INV-21
P1-INV-22
P1-INV-24
P1-INV-25
P1-INV-27
P1-INV-28
```

Coverage dependencies:

```text
P1-CV-06
P1-CV-12
P1-CV-23
P1-CV-24
P1-CV-32
P1-CV-33
P1-CV-40
P1-CV-41
```

### Produces

Either:

```text
A. verification-only result: all prerequisites already landed
```

or the smallest general patch that provides the missing prerequisite, with owning-layer tests.

T1 must never produce a `time`-specific abstractness checker, `Date`-specific constructor rule, or visibility escape hatch.

### Files and symbols

Use the exact execution-HEAD files recorded by T0. At the planning baseline, likely areas include:

```text
phalcom-ast/src/**                         # MemberBody / declaration syntax, if still relevant
phalcom-semantic/src/**                    # class completeness / visibility / data construction
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/impl_decl.rs
phalcom-core/src/native/source.rs           # only if execution HEAD still routes declaration bodies here

phalcom-semantic/tests/semantic/capabilities/**
phalcom-core/tests/core/compiler/declaration_dispatch.rs
phalcom-core/tests/core/language/data*.rs or live equivalent
```

Relevant symbols include the live equivalents of:

```text
MemberBody::Declaration
abstract-class marker/state
abstract-member obligation set
concrete class completeness/finalization
data declaration constructor synthesis
constructor visibility / validation hook
```

### Required implementation shape

#### A. Abstract-member closure, only if missing

Implement the general rule set:

```text
bodyless abstract member
    is a requirement, not an executable throwing stub

abstract class
    may carry unresolved abstract member obligations

subclass
    inherits unresolved obligations

concrete implementation
    discharges a compatible obligation

concrete subclass
    with unresolved obligations is rejected before/at canonical class finalization

runtime allocation guard
    remains a separate defense for abstract classes; do not use it instead of semantic completeness
```

Compiler lowering must represent/skip a declaration-only abstract requirement through the normal class metadata/semantic product path rather than returning `DeclarationBodyRequiresImplementation` for a valid abstract requirement.

Do not generalize beyond what the accepted class model requires.

#### B. Validated `data` closure, only if missing

Use the accepted general data-construction model. Required semantics:

```text
data remains transparent immutable value state
validation occurs on every ordinary public construction lane
invalid state cannot be manufactured through a second synthesized bypass constructor
validation is not a compiler check for Date/Time/UtcOffset names
behavior remains ordinary source behavior
```

If the accepted language model does not yet define how validated transparent data construction works, STOP AND CONSULT. Do not invent a parallel constructor model under P1.

#### C. Future visibility/native/primitive transition

If these mechanisms are not fully landed:

- perform only clearly-owned mechanical follow-through already dictated by current accepted language work;
- do not reproduce the transition inside P1;
- if ownership or semantics are uncertain, STOP AND CONSULT.

### Forbidden approaches

- throwing placeholder bodies for `Clock.now` / `WallClock.now`;
- a checker rule that recognizes `Clock` or `WallClock` by name;
- converting validated data types into opaque classes;
- hidden semantic fields inside `data`;
- time-only “friend” access;
- selector-prefix privilege;
- operative `@primitive`/`@intrinsic` metadata;
- a second primitive registry invented for time;
- broad language cleanup unrelated to the exact prerequisite.

### Test changes required

Add owning-layer tests with stable discriminating prefixes so later commands select nonzero tests:

```text
abstract_member_*      # if abstract closure changed
validated_data_*       # if data validation closure changed
```

At minimum, abstract tests must prove:

```text
abstract class + declaration-only abstract member accepted
abstract subclass may remain incomplete
concrete subclass without implementation rejected
compatible concrete override accepted
valid abstract requirement does not lower to a throwing body
```

Validated-data tests must prove the general mechanism, not temporal names:

```text
valid construction succeeds
invalid construction rejects
all public construction lanes apply the invariant
value transparency/equality remains canonical
```

### Tests to run now

Only the branches actually changed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic abstract_member_
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_declaration_dispatch::abstract_member_
```

and/or:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic validated_data_
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core validated_data_
```

If execution HEAD already contains equivalent established tests under different names, run those exact live filters instead and record them. Do not add duplicate tests merely to preserve these planning-time prefixes.

### Tests explicitly deferred

Do not run time integration tests if T1 only changes prerequisite infrastructure and the temporal source has not yet been migrated. Do not run full semantic/core suites in BUILD mode.

### Acceptance

T1 is accepted when either:

- T0 proved all required general mechanisms already exist and T1 records `NO CODE REQUIRED`; or
- the smallest missing general prerequisite is implemented and its owning focused tests pass.

In either case, the executor can now write `Clock`/`WallClock` abstract requirements and validated civil data without a temporal workaround.

### Local STOP / CONSULT triggers

Use shared-header triggers 1–12, 23–25, and 28 in particular. Also STOP if the required change would become a competing implementation of active visibility/native/primitive work.

### Checkpoint update

Update only if T1 establishes a durable general capability that P1 now depends on, or if a consultation/amendment occurred. A verification-only T1 does not require checkpoint noise.

---

## T2 — Migrate `Duration` to data and add `Measurement<T>`

### Purpose

Create the exact elapsed-time value foundation consumed by clocks, timestamps, civil integration, and concurrency before changing the clock capability architecture.

### Preconditions

- G1 has passed.
- First-class `data`, generic `data`, value equality/hash, inherent behavior, and the chosen validation/construction mechanism are available.

### Consumes

```text
P1-INV-02
P1-INV-03
P1-INV-18
P1-INV-27
P1-INV-28
P1-INV-35
```

Primary coverage:

```text
P1-CV-01
P1-CV-02
P1-CV-03
P1-CV-04
```

### Produces

```text
Duration as data Duration(nanoseconds: Int)
Measurement<T> as data Measurement<T>(value: T, duration: Duration)
canonical source-level Duration behavior
no hidden/native Duration state
no fractional-unit semantics
```

### Files and symbols

Preferred target decomposition:

```text
CREATE  phalcom-core/core/universe/src/time/duration.ph
MODIFY  phalcom-core/core/universe/src/time/clock.ph
MODIFY  phalcom-core/core/universe/src/time/package.ph       # only enough to keep canonical publication coherent
MODIFY  phalcom-core/core/universe/src/package.ph            # only if the current direct Duration import path must move
MODIFY  phalcom-semantic/tests/semantic/capabilities/time.rs
MODIFY/ADD phalcom-core/tests/fixtures/language/time/**
MODIFY/ADD phalcom-core/tests/fixtures/language/time_negative/**
```

If execution-HEAD package conventions make retaining `Duration` in `clock.ph` temporarily materially simpler without creating mixed ownership, that is mechanically allowed, but final T7 topology must not leave clock/calendar/absolute semantics accidentally co-owned in one file.

### Required implementation shape

Implement `Duration` with canonical component:

```text
nanoseconds: Int
```

Preserve or provide the shared-header behavior:

```text
zero
nanoseconds(...)
microseconds(...)
milliseconds(...)
seconds(...)
minutes(...)
hours(...)

nanoseconds projection
microseconds projection
milliseconds projection
seconds projection

+ Duration
- Duration
* Int
negated
abs
comparison / relational behavior
isZero
isNegative
canonical data equality/hash
```

Rules:

- integer factories only;
- negative durations remain valid values;
- no `days`/`weeks` factories;
- no Float canonical state or implicit fractional rounding;
- use canonical data equality/hash rather than preserving manual opaque-class identity machinery;
- behavior belongs in ordinary Phalcom source/inherent implementation;
- `Measurement<T>` has exactly `value: T` and `duration: Duration` semantic state and no custom representation.

Do not yet redesign `Clock` beyond the minimal imports/references needed to consume the migrated `Duration`.

### Forbidden approaches

- native/Rust Duration arithmetic;
- hidden native nanoseconds parallel to the data component;
- temporal VM value/tag/opcode;
- `Duration.days` / `Duration.weeks`;
- fractional factory acceptance;
- changing negative Duration validity because sleep rejects negative waits;
- redesigning sleep/timeout in this task.

### Test changes required

Migrate/add focused time fixtures proving:

```text
same nanoseconds => equality + same hash
zero / positive / negative
all integer unit factories
unit projections
arithmetic and negation
abs
comparison
fractional factory input rejected by normal typing
wrong operand types rejected normally
Measurement<Int> preserves Int value and Duration
Measurement<String> has the corresponding applied generic identity
```

Semantic tests should assert `Duration` and `Measurement<T>` are source-owned canonical declarations/data applications, not runtime aliases.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
```

If the old clock vertical is intentionally temporarily broken by an atomic source split that T3 immediately completes, do **not** run unrelated integration just to confirm the known intermediate break. Keep T2 source/test changes coherent enough that the Duration-specific tests themselves are discriminating.

### Tests explicitly deferred

```text
clock-domain behavior
wall time
calendar values
ordinary-package full P1 surface
reflection/provenance closure
full concurrency integration
```

### Acceptance

- `P1-CV-01` through `P1-CV-04` have executable evidence;
- Duration is transparent `data` with only `Int` nanoseconds as semantic state;
- Measurement generic identity is canonical;
- no temporal runtime specialization was introduced.

### Local STOP / CONSULT triggers

STOP if generic data cannot preserve applied identity, if canonical data equality/hash is semantically incompatible with Duration, or if preserving Duration consumers appears to require a hidden non-data representation.

### Checkpoint update

Normally none until G2 unless T2 establishes a durable interface that differs from the shared header or discovers a blocking baseline failure.

---

## T3 — Replace the C1 concrete clock/System façade with the abstract `Clock` / `SystemClock` / `Instant` model

### Purpose

Complete the monotonic migration as one vertical slice: abstract clock domain, concrete exported system clock, canonical `clock` value, protected Instant minting, strict domain laws, primitive ownership migration, and removal of public `System.clock`.

### Preconditions

- T2 complete.
- Bodyless abstract requirements work.
- Ratified lexical visibility and `primitive` implementation kinds are available at execution HEAD.
- Stable class-side singleton state/package binding identity is verified.

### Consumes

```text
P1-INV-04 through P1-INV-09
P1-INV-21 through P1-INV-28
P1-INV-31
P1-INV-32
P1-INV-34
```

Primary coverage:

```text
P1-CV-05 through P1-CV-13
P1-CV-39
P1-CV-40
P1-CV-41
```

### Produces

```text
abstract Clock
public SystemClock is Clock
SystemClock.instance
exported clock bound to that exact singleton
opaque Instant bound to exact Clock object identity
protected Clock.instant(ticks) minting seam
package-controlled raw Instant construction
Clock.since
Clock.measure<T>
SystemClock monotonic host primitive
no public System.clock
no legacy time-owned hidden-selector access seam
```

### Files and symbols

Expected primary source:

```text
phalcom-core/core/universe/src/time/clock.ph
phalcom-core/core/universe/src/time/package.ph
phalcom-core/core/universe/src/concurrency/fiber.ph
```

Current primitive migration inputs:

```text
phalcom-core/src/primitive/system.rs
phalcom-core/src/universe/primitives.rs
```

Target primitive/linkage files are mechanically flexible and must use the canonical execution-HEAD source-declared primitive infrastructure. A new time-specific Rust module such as `phalcom-core/src/primitive/time.rs` is acceptable if that matches current organization; a second linkage system is not.

Tests:

```text
phalcom-semantic/tests/semantic/capabilities/time.rs
phalcom-core/tests/fixtures/language/time/**
phalcom-core/tests/fixtures/language/time_negative/**
phalcom-core/tests/core/native/contracts.rs
phalcom-core/tests/core/modules/runtime.rs          # singleton import identity if useful at this stage
```

### Required implementation shape

#### Clock

Express the conceptual contract from the header using execution-HEAD syntax:

```text
abstract now -> Instant
source since(Instant) -> Duration
source measure<T>(() -> T) -> Measurement<T>
protected instant(Int) -> Instant
```

`measure` must evaluate the action exactly once and use the same `Clock` object before/after.

#### Instant

Canonical opaque state remains logically:

```text
clock: Clock
ticks: Int
```

but ordinary callers cannot construct raw coordinates.

Required laws:

```text
same Clock object + same ticks => equal
same numeric ticks + different Clock objects => unequal
ordering across different clocks => reject
difference/since across different clocks => reject
plus/minus Duration => same clock domain, exact tick arithmetic
elapsed => consult originating clock object
```

Use lexical private/package/protected relations. Remove C1 time-owned selectors such as the live equivalents of:

```text
_$from
_$clock
_$ticks
_$system
```

Do not retain them as compatibility APIs.

#### SystemClock

`SystemClock` is public/exported and cannot expose a public construction path that manufactures arbitrary additional host domains.

Required identity:

```text
clock === SystemClock.instance
```

Creating the singleton must not read the host clock.

`SystemClock.now` should:

```text
read host monotonic nanoseconds through its primitive
mint an Instant through Clock.instant
```

The irreducible Rust host read remains based on VM/process monotonic elapsed time or the execution-HEAD equivalent and returns arbitrary-precision `Int` nanoseconds.

#### Primitive ownership

Move the host monotonic primitive from `System` holder identity to `SystemClock` holder identity using the canonical implementation-kind/linkage system.

Delete the temporal System bridge rather than aliasing it.

Metadata such as `@primitive`, if present, may describe the declaration but must not be what binds it.

#### System façade

Remove public `System.clock` in this task. Preserve scheduler/process operations on `System` such as `sleep`.

### Forbidden approaches

- public raw Instant constructor;
- public tick/clock getters added solely for implementation convenience;
- global numeric tick equality across clocks;
- `System.clock` compatibility alias surviving the task;
- `System._$monotonicNanoseconds` or equivalent public/internal temporal bridge surviving as the canonical host read;
- special selector spelling for access;
- concrete throwing body for abstract `Clock.now`;
- marking all of `SystemClock` native;
- Rust implementation of `Clock.since` or `measure`;
- new VM temporal opcode/value arm.

### Test changes required

Build a deterministic custom `Clock` subclass fixture using the protected seam. It must be capable of proving domain behavior without depending on wall time or scheduler sleeps.

Add/migrate tests for:

```text
SystemClock.instance stable identity
clock === SystemClock.instance
repeated package imports observe same clock object
Clock direct construction rejected
concrete Clock subclass must implement now
custom Clock can mint Instant through protected seam
non-subclass cannot call protected mint seam
ordinary package cannot raw-construct Instant
same ticks/different custom clocks => false
cross-domain compare/since reject
Instant +/- Duration preserves clock
Instant.elapsed calls originating custom Clock
Clock.measure evaluates action once and reports same-domain duration
System.clock no longer resolves
primitive provenance/owner is SystemClock
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts::time_monotonic_
```

If the primitive contract test is added under a different existing naming convention, use the exact live test path/filter recorded in T0 and ensure it selects nonzero tests.

### Tests explicitly deferred

Wall-clock, calendar, final prelude, broad ordinary-package, and full concurrency stabilization are deferred to later tasks/gates.

### Acceptance

`P1-CV-05` through `P1-CV-13` pass; public `System.clock` is gone; deterministic custom clock tests prove domain laws; the host read is owned by `SystemClock`; no legacy naming/access mechanism is required.

### Local STOP / CONSULT triggers

Use shared-header triggers 1–8, 17, 22–25, and 28. In particular, stop if stable package singleton identity cannot be preserved or primitive linkage still fundamentally requires System ownership.

### Checkpoint update

At G2, record the durable monotonic interface migration and removal of `System.clock`; do not update the checkpoint after each small edit inside T3.

---

## T4 — Add `Timestamp`, `WallClock`, `SystemWallClock`, and canonical wall-clock observation

### Purpose

Establish absolute time as a separate global-value timeline and capability vertical without introducing civil timezone semantics.

### Preconditions

- G2 passed.
- Primitive holder/linkage mechanism proven by SystemClock.
- Generic/source/data runtime remains stable after monotonic migration.

### Consumes

```text
P1-INV-10
P1-INV-11
P1-INV-12
P1-INV-14
P1-INV-24 through P1-INV-28
P1-INV-32
```

Primary coverage:

```text
P1-CV-14 through P1-CV-19
```

### Produces

```text
Timestamp data(unixNanoseconds: Int)
abstract WallClock
public SystemWallClock is WallClock
SystemWallClock.instance
exported wallClock bound to exact singleton
Timestamp.now delegating to wallClock
SystemWallClock wall-time primitive
exact signed Unix/POSIX nanosecond arithmetic
```

### Files and symbols

Preferred source:

```text
CREATE phalcom-core/core/universe/src/time/timestamp.ph
MODIFY phalcom-core/core/universe/src/time/package.ph
```

Primitive/linkage:

```text
execution-HEAD time primitive Rust module / registration
current system primitive files only as migration inputs if wall logic is initially placed there
```

Tests:

```text
phalcom-semantic/tests/semantic/capabilities/time.rs
phalcom-core/tests/fixtures/language/time/**
phalcom-core/tests/fixtures/language/time_negative/**
phalcom-core/tests/core/native/contracts.rs
```

### Required implementation shape

#### Timestamp

Canonical state:

```text
unixNanoseconds: Int
```

Implement in Phalcom source:

```text
epoch
nanoseconds/microseconds/milliseconds/seconds construction or canonical factories as ratified
matching projections
plus(Duration)
minus(Duration)
since(Timestamp) -> Duration
comparison / relational behavior
canonical data equality/hash
now
```

Signed values before and after epoch are valid. Use no floating canonical representation.

Where conversion arithmetic splits a signed nanosecond count into seconds/subseconds, use floor/Euclidean semantics so `-1ns` corresponds to:

```text
second = -1
nanosecond-within-second = 999_999_999
```

#### WallClock

Abstract capability with bodyless:

```text
now -> Timestamp
```

#### SystemWallClock

Public/exported concrete implementation with private construction/singleton storage and public class-side `instance`.

Required identity:

```text
wallClock === SystemWallClock.instance
```

Creation of the capability object performs no time read.

#### Wall primitive

The primitive observes host Unix/POSIX wall time and returns the exact signed `Int` nanosecond coordinate expected by Timestamp.

On a Rust `SystemTime`-style host API, handle both sides of the Unix epoch exactly:

```text
Ok(duration_since_epoch)  -> +(seconds * 1_000_000_000 + nanos)
Err(pre_epoch_duration)   -> -(seconds * 1_000_000_000 + nanos)
```

using arbitrary-precision integer construction without narrowing the semantic range to `i64`/Float.

`Timestamp.now` must call the canonical `wallClock` capability. It is not a second primitive.

### Forbidden approaches

- tagging Timestamp with WallClock identity;
- `Timestamp.now` calling Rust directly;
- floating seconds canonical state;
- host date/calendar library as canonical Timestamp representation;
- `System.now` or `System.wallClock`;
- named timezone/system-local conversion;
- native implementation of Timestamp arithmetic.

### Test changes required

Add tests for:

```text
Timestamp.epoch == 0ns
positive and negative scalar construction/projection
Timestamp +/- Duration inverse law
Timestamp.since signed result
ordering/equality/hash
-1ns split helper behavior where exposed by P1 conversion core
SystemWallClock.instance stable identity
wallClock === SystemWallClock.instance
repeated import identity
Timestamp.now returns a Timestamp through the canonical capability
wall primitive provenance/owner = SystemWallClock
no second Timestamp.now primitive
```

Do not write flaky tests that assert wall-clock passage by sleeping for arbitrary real time. Structural provenance/identity plus bounded monotonic inequalities, if needed, are preferable.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts::time_wall_
```

### Tests explicitly deferred

Civil conversion, fixed offset, final package/prelude, ordinary-package complete fixture, reflection closure, broad concurrency stabilization.

### Acceptance

`P1-CV-14` through `P1-CV-19` pass; absolute timeline is independent of monotonic domain identity; the wall primitive is owned by SystemWallClock; `Timestamp.now` has one canonical capability path.

### Local STOP / CONSULT triggers

Stop if exact signed host wall-time observation would require Float/narrow canonical state, if WallClock cannot express a bodyless abstract requirement, or if `Timestamp.now` cannot reference the canonical package/module binding without creating a second singleton/path.

### Checkpoint update

At G3, record the stable absolute-time capability interfaces and wall primitive ownership.

---

## T5 — Implement the pure Gregorian arithmetic core, `Month`, `Weekday`, and `Period`

### Purpose

Create one source-level arithmetic authority for proleptic Gregorian rules and the calendar-relative amount types needed by validated civil values.

### Preconditions

- G3 passed.
- Arbitrary-precision `Int` behavior needed for negative floor/Euclidean arithmetic is verified.
- No public civil year restriction is required merely for host interoperability.

### Consumes

```text
P1-INV-13
P1-INV-14
P1-INV-18
P1-INV-19
P1-INV-27
P1-INV-28
```

Primary coverage:

```text
P1-CV-20
P1-CV-21 (foundation; completed in T6)
P1-CV-22 (foundation; completed in T6)
P1-CV-26
P1-CV-27
```

### Produces

```text
calendar package/module foundation
Month enum
Weekday enum
Period data(months: Int, days: Int)
DateOverflow enum may be introduced here or T6, but its semantics remain fixed
one canonical source-level Gregorian helper authority
constant-time civil <-> epoch-day conversion helpers
```

### Files and symbols

Preferred decomposition:

```text
CREATE phalcom-core/core/universe/src/time/calendar/package.ph
CREATE phalcom-core/core/universe/src/time/calendar/month.ph
CREATE phalcom-core/core/universe/src/time/calendar/period.ph
CREATE phalcom-core/core/universe/src/time/calendar/date.ph      # helper authority may begin here
MODIFY phalcom-core/core/universe/src/time/package.ph            # expose/import wiring as needed

MODIFY/ADD phalcom-core/tests/fixtures/language/time/**
MODIFY/ADD phalcom-core/tests/fixtures/language/time_negative/**
```

The exact helper module may differ, but there must be one source-level authority for:

```text
isLeapYear(year)
daysInMonth(year, month)
daysFromCivil(year, month, day)
civilFromDays(epochDay)
```

### Required implementation shape

#### Month

Enum January..December, ISO/Gregorian number 1..12, with normal source behavior such as `number`, `fromNumber`, and `days(inYear:)`. Invalid numeric conversion rejects; it does not wrap.

#### Weekday

Enum Monday..Sunday with ISO number 1..7. Date-derived weekday uses epoch-day arithmetic; no redundant Date storage.

#### Period

Transparent data:

```text
months: Int
days: Int
```

Factories/normalization:

```text
years(n) -> months = n * 12
months(n)
weeks(n) -> days = n * 7
days(n)
zero
```

Arithmetic:

```text
+
-
* Int
negated
isZero
```

No total ordering API or derived `Comparable` relation.

#### Gregorian helpers

Use a mathematically proven constant-time proleptic Gregorian conversion suitable for arbitrary positive/negative years. The implementation must explicitly account for the language's division/modulo semantics. If `/` or `%` truncates toward zero, implement a local source-level floor/Euclidean helper rather than assuming positive-only formulas.

Required anchor evidence eventually includes:

```text
1970-01-01 <-> epoch day 0
1969-12-31 <-> epoch day -1
2000 leap
1900 not leap
2024 leap
representative year 0 / negative year behavior under the chosen proleptic numbering
400-year cycle representatives
civil -> day -> civil round trips
```

No host calendar library.

### Forbidden approaches

- Rust/Jiff/chrono calendar arithmetic;
- duplicated leap-year rules in Month and Date;
- cached weekday/dayOfYear semantic state;
- limiting years to four digits without a separate accepted reason;
- total ordering for Period;
- interpreting `Period.days(1)` as exact 24 hours;
- Float division to sidestep negative integer semantics.

### Test changes required

Add focused source/runtime tests for:

```text
Month numbering / fromNumber rejection
Month.days(inYear:) for leap/non-leap February
Weekday numbering
Period normalization and arithmetic
Period no-ordering negative compile/semantic case
leap-year anchors 2000/1900/2024
calendar helper anchors that can be observed through current public/internal package test lanes
```

If direct helper testing would require exporting implementation details, do not widen the API. Complete the conversion property evidence through Date in T6.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
```

Run semantic time tests only if public declaration/type shapes changed in a way the semantic capability suite already observes.

### Tests explicitly deferred

Full civil round-trip, Date validity, DateOverflow behavior, UtcOffset, package/prelude and integration gates.

### Acceptance

- source-level Gregorian helper authority exists once;
- Period is months+days only and cannot be totally ordered;
- leap-year/month-length foundations are correct;
- no host/native/runtime specialization entered the calendar model.

### Local STOP / CONSULT triggers

Stop if correct negative arithmetic requires changing global numeric semantics outside a small accepted helper path, if a public year bound becomes necessary without accepted evidence, or if the only way to test helpers would be to expose non-public calendar machinery as API.

### Checkpoint update

Normally defer durable recording to G4 unless an accepted year/range decision or consultation occurs.

---

## T6 — Implement validated civil values and fixed `UtcOffset`

### Purpose

Complete the P1 civil value layer using the Gregorian core: valid Date/Time/DateTime/partial values, calendar Period application, overflow policy, and fixed second-resolution UTC offsets.

### Preconditions

- T5 complete.
- Validated transparent `data` construction is available.
- UtcOffset bound can be grounded against intended P2 interoperability before the type is finalized.

### Consumes

```text
P1-INV-13 through P1-INV-20
P1-INV-27
P1-INV-28
P1-INV-36
```

Primary coverage:

```text
P1-CV-20 through P1-CV-34
```

### Produces

```text
Date data(year, month, day)
Time data(hour, minute, second, nanosecond)
DateTime data(date, time)
YearMonth data(year, month)
MonthDay data(month, day)
DateOverflow enum { Constrain, Reject }
completed Period application
UtcOffset data(seconds)
validated construction for invalid-state exclusion
```

### Files and symbols

Preferred decomposition:

```text
phalcom-core/core/universe/src/time/calendar/date.ph
CREATE phalcom-core/core/universe/src/time/calendar/time.ph
CREATE phalcom-core/core/universe/src/time/calendar/datetime.ph
CREATE phalcom-core/core/universe/src/time/calendar/partial.ph
CREATE phalcom-core/core/universe/src/time/offset.ph
MODIFY phalcom-core/core/universe/src/time/calendar/package.ph
MODIFY phalcom-core/core/universe/src/time/package.ph

MODIFY phalcom-semantic/tests/semantic/capabilities/time.rs
MODIFY/ADD phalcom-core/tests/fixtures/language/time/**
MODIFY/ADD phalcom-core/tests/fixtures/language/time_negative/**
```

If execution-HEAD topology prefers `time/zone/offset.ph` to prepare P2 dependency direction, that is mechanically acceptable so long as the module contains only fixed-offset semantics and introduces no named-zone behavior.

### Required implementation shape

#### Date

Semantic state only:

```text
year: Int
month: Month
day: Int
```

Validation:

```text
1 <= day <= daysInMonth(year, month)
```

Derived behavior must use the one Gregorian authority:

```text
weekday
dayOfYear
daysInMonth
isLeapYear
daysSince / epoch-day relation as ratified
compare
at(Time) -> DateTime
advance(Period, overflow: DateOverflow)
```

No cached weekday/dayOfYear/month-length state.

#### DateOverflow and Date.advance

Fixed application order:

```text
1. add Period.months to year/month coordinate
2. resolve the original day against target month using Constrain or Reject
3. add Period.days through epoch-day arithmetic
```

Required discriminator beyond simple Jan-31:

Choose at least one period containing both nonzero months and nonzero days where applying days first would yield a different final date. The test must prove the specified order, not just each operation independently.

#### Time

Semantic state:

```text
hour:       0..23
minute:     0..59
second:     0..59
nanosecond: 0..999_999_999
```

Provide component access, `midnight`, `noon`, comparison. No leap second. No `Time.now`.

#### DateTime

Semantic state:

```text
date: Date
time: Time
```

Provide projections, comparison, and calendar `advance(Period, overflow:)`. No timezone/offset/instant state, no `DateTime.now`, no general exact `Duration` arithmetic.

#### YearMonth

Semantic state `year + month`; source month arithmetic with correct negative month movement; day count/isLeapYear; `atDay` validates through Date.

#### MonthDay

Semantic state `month + day`; reject impossible combinations such as April 31; allow February 29; `inYear(nonLeap)` rejects when appropriate.

#### UtcOffset

Semantic state:

```text
seconds: Int
```

Provide:

```text
utc
seconds(...)
hours(...)
negated
comparison
canonical equality/hash
```

Second-level historical offsets must be representable within the selected range.

##### Mandatory VERIFY-FIRST bound decision

Before finalizing UtcOffset validation:

1. inspect any already-ratified P2/backend interoperability constraint that exists at execution time;
2. inspect the intended backend's supported fixed-offset range only if that backend choice has already been approved elsewhere;
3. prefer a stable semantic range that can represent relevant IANA historical second-resolution offsets and does not unnecessarily mirror a narrow convenience API;
4. record the selected bound and evidence in the P1 walkthrough/handoff.

If no accepted constraint exists and two plausible bounds have materially different public consequences, trigger shared-header STOP/CONSULT 16. Do not guess.

#### Civil range

Use arbitrary-precision year arithmetic unless a separately accepted interoperability contract requires a bound. Do not narrow Date/YearMonth simply because P2's native backend may have a finite optimized range; P2 may instead reject unsupported conversions at its own boundary if that is the accepted design.

### Forbidden approaches

- opaque-class substitution for Date/Time/MonthDay/UtcOffset;
- duplicate validation logic per parser/API lane;
- hidden timezone in DateTime;
- DateTime + Duration;
- Time.now;
- named timezone or DST behavior in UtcOffset;
- host date library for civil arithmetic;
- public derived-state fields;
- guessing an offset bound.

### Test changes required

Add deterministic tests covering at least:

```text
Date valid month edges
Date day 0 rejects
April 31 rejects
Feb 29 on 2024 succeeds
Feb 29 on 1900 rejects
Feb 29 on 2000 succeeds
Date weekday known anchors
1970-01-01 epoch-day 0
1969-12-31 epoch-day -1
civil <-> epoch-day round trip across pre/post epoch and leap boundaries
representative negative/year-zero round trips under chosen numbering
Time exact lower/upper boundaries
hour 24 rejects
minute 60 rejects
second 60 rejects
nanosecond 1_000_000_000 rejects
DateTime exact Date+Time projections
DateTime has no timezone state/API
Period years/weeks normalization
Period ordering unavailable
Jan 31 + one month Constrain
Jan 31 + one month Reject
months -> overflow -> days discriminator
YearMonth positive/negative month movement and atDay validation
MonthDay April31 rejects
MonthDay Feb29 exists and non-leap inYear rejects
UtcOffset zero/positive/negative/second precision
UtcOffset negation/order/equality/hash
UtcOffset values outside selected bound reject
no TimeZone/DST/named-zone API
```

Use deterministic loops for property-style civil round trips rather than adding a new property-testing dependency.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
```

### Tests explicitly deferred

```text
named timezone conversion
DST gaps/overlaps
ZonedDateTime
system-local Date.today
final parse/format grammar
P2 backend range behavior
```

### Acceptance

`P1-CV-20` through `P1-CV-34` have evidence; validation prevents invalid public states without losing data transparency; Period ordering is absent; month application order is proven; UtcOffset has a documented, evidence-based bound and no timezone semantics.

### Local STOP / CONSULT triggers

Use shared-header triggers 11–16, 19–21, 24, 28–29. The UtcOffset bound ambiguity is explicitly non-self-waivable.

### Checkpoint update

At G4, record only durable civil interfaces/ranges that P2 must consume, including the final UtcOffset bound if one was selected/approved.

---

## T7 — Finalize package topology, grouped publication, prelude, and representative ordinary-package use

### Purpose

Make the completed P1 source surface discoverable exactly through the intended package/prelude model, migrate remaining C1 publication assumptions, and prove that ordinary packages—not only Universe bootstrap code—can consume P1 normally.

### Preconditions

- G4 passed.
- All P1 public declarations exist with stable categories/signatures.
- Grouped import/export and lexical visibility semantics are live.

### Consumes

```text
P1-INV-21 through P1-INV-25
P1-INV-29 through P1-INV-34
P1-INV-36
```

Primary coverage:

```text
P1-CV-35 through P1-CV-42
P1-CV-48
P1-CV-49
```

### Produces

Final P1 `time` root publication:

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
```

Final temporal prelude:

```text
Duration
Timestamp
```

and a representative ordinary-package fixture exercising the public surface through normal module/project identity.

### Files and symbols

```text
phalcom-core/core/universe/src/time/package.ph
phalcom-core/core/universe/src/package.ph
phalcom-core/core/universe/src/time/**/package.ph
phalcom-semantic/tests/semantic/capabilities/time.rs

phalcom-core/tests/core/modules/runtime.rs
phalcom-core/tests/fixtures/modules_v1/** or execution-HEAD ordinary-package fixture root
phalcom-core/tests/fixtures/language/time/**
phalcom-core/tests/fixtures/language/time_negative/**
```

### Required implementation shape

#### Time root

Use canonical tuple-shaped grouped imports/exports for multiple names. Do not preserve legacy comma-list publication solely for C1 compatibility.

Publication must resolve to the canonical declarations/values created by T2–T6. No wrappers, duplicate types, or second singleton values.

#### Prelude

Universe root promotes exactly `Duration` and `Timestamp`.

Required positive evidence:

```text
Duration.seconds(5) works unqualified
Timestamp.now works unqualified
```

Required negative evidence:

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

do not become temporal prelude names merely because they are exported by `time`.

Explicitly importing Duration/Timestamp from `time` must resolve to the same canonical declaration identity as the unqualified prelude names.

#### System boundary

Confirm `System.clock` remains absent and no `System.now`, `System.wallClock`, or `System.timeZone` has appeared.

#### Ordinary-package fixture

Create a representative project/package under the existing modules-v1 or successor fixture infrastructure. It must compile/run through the ordinary project/module linker, not a special Universe test path.

Its source must use canonical grouped import form equivalent to:

```phalcom
from time import (
  Date,
  DateTime,
  Month,
  Period,
  Time,
  UtcOffset,
  clock
)
```

and must also use unqualified prelude:

```text
Duration
Timestamp
```

The fixture must prove at least:

```text
grouped resolution
civil construction and observation
Period arithmetic
UtcOffset construction
clock.now
Duration/Timestamp prelude access
canonical identities where reflection/semantic fixture support exists
ordinary package cannot call package-visible Instant raw construction
ordinary non-subclass cannot call protected Clock.instant
```

It must not use bootstrap-only project identity.

#### Cheap import

Package/module initialization may allocate the canonical `SystemClock` and `SystemWallClock` capability objects. It must not:

```text
read either host clock merely by import
load a timezone database
probe system timezone configuration
create a P2 native backend
```

No timezone dependency may be added in P1.

### Forbidden approaches

- prelude wrapper/alias types;
- retaining Clock/Instant in prelude for compatibility;
- ordinary-package test compiled as Universe/internal project;
- broadening visibility to make the fixture pass;
- reintroducing System.clock;
- adding P2 TimeZone stubs merely to reserve names;
- eager resource/database initialization;
- non-grouped migrated multi-name imports/exports where canonical grouped form applies.

### Test changes required

Semantic capability tests must now assert the final P1 publication/identity model rather than C1 assumptions.

Add ordinary-package integration tests named with a stable `time_p1_` prefix in the existing `modules_runtime` integration module or its execution-HEAD equivalent.

Add explicit negative-resolution/access fixtures for non-prelude and non-visible names.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_runtime::time_p1_
```

### Tests explicitly deferred

Final reflection/provenance/concurrency stabilization is T8/G5. P2 ordinary-package tests for TimeZone/ZonedDateTime remain deferred.

### Acceptance

`P1-CV-35` through `P1-CV-42` pass; `P1-CV-48` and `P1-CV-49` have structural evidence; the representative ordinary package behaves like normal user code; prelude identity is canonical; no System temporal façade survives.

### Local STOP / CONSULT triggers

Stop if ordinary-package success requires Universe-only access, if exported values are re-evaluated per import and lose singleton identity, if prelude publication creates duplicate declaration identity, or if grouped import/export semantics materially differ from the ratified model.

### Checkpoint update

Record the final P1 package/prelude surface after this task only if G5 later confirms integration; do not prematurely mark P1 complete.

---

## T8 — Stabilize concurrency consumers, reflection/provenance, and ordinary runtime architecture

### Purpose

Close every P1 integration edge that could otherwise leave the source surface correct but the repository inconsistent: Duration consumers, reactor boundary, reflection categories, primitive ownership/provenance, absence of temporal VM specialization, absence of P2 resource work, and plan/API agreement.

### Preconditions

- T7 complete.
- Public P1 package/prelude surface is stable.
- Representative ordinary-package fixture passes.

### Consumes

```text
P1-INV-03
P1-INV-06
P1-INV-10
P1-INV-21 through P1-INV-35
P1-INV-36
```

Primary coverage:

```text
P1-CV-43 through P1-CV-50
```

with `P1-CV-50` interpreted under `P1-BODY-DEV-01`.

### Produces

```text
System.sleep(Duration) compatibility
Future.timeout(Duration) compatibility
reactor deadline boundary preserved
reflection categories for all P1 declarations
primitive ownership/provenance evidence
no temporal opcode/value/name-specialization evidence
cheap import / no P2 dependency evidence
P1 plan/API agreement evidence
no stale time specification work
```

### Files and symbols

Concurrency:

```text
phalcom-core/core/universe/src/concurrency/fiber.ph
execution-HEAD reactor/timer conversion source reached by System.sleep/Future.timeout
```

Reflection/provenance/tests:

```text
phalcom-core/tests/core/reflection/type_metadata.rs
phalcom-core/tests/core/native/contracts.rs
phalcom-core/tests/core/observability/disassembly.rs
phalcom-core/tests/core/modules/runtime.rs
phalcom-core/tests/fixtures/language/time/**
```

Runtime/compiler inspection areas:

```text
phalcom-core/src/value.rs or execution-HEAD Value representation
phalcom-core/src/opcode.rs / bytecode definitions or execution-HEAD equivalents
phalcom-core/src/compiler/**
phalcom-core/src/vm/**
```

State records:

```text
STDL002.C2-CHECKPOINT.md
STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md
```

Explicitly excluded:

```text
docs/spec/current/stdlib/time.md
docs/implementation/time.md
```

### Required implementation shape

#### Concurrency compatibility

Audit only affected Duration consumption lanes.

`System.sleep(Duration)` must continue to:

- accept the migrated canonical Duration;
- reject negative wait according to its existing scheduler contract;
- project nanoseconds through the public/source canonical Duration component/API;
- convert to host deadline representation at the concurrency boundary;
- avoid treating public `Instant` as reactor timer storage merely because Instant exists.

`Future.timeout(Duration)` must preserve its existing suspension/timeout semantics while consuming the migrated Duration.

Do not redesign the reactor, scheduler, timer queue, Fiber, or Future architecture.

#### Reflection

Add focused assertions for the normal declaration categories/identity:

```text
Duration             data
Measurement<T>       generic data
Clock                abstract class
SystemClock          class
Instant              class
Timestamp            data
WallClock            abstract class
SystemWallClock      class
Month                enum
Weekday              enum
Date                 data
Time                 data
DateTime             data
YearMonth            data
MonthDay             data
Period               data
DateOverflow         enum
UtcOffset             data
```

Reflection must observe canonical package/prelude identity rather than time-specific metadata.

If implementation-kind provenance is reflectable in the current language, assert that only the two host observations are primitive and that no ordinary temporal source method became native/primitive accidentally.

#### Primitive inventory

The final P1 temporal primitive inventory is expected to contain exactly the irreducible host observations needed by:

```text
SystemClock
SystemWallClock
```

within the time domain. Existing scheduler/runtime primitives outside time remain outside this inventory and are not migrated merely for aesthetic consistency.

#### No temporal VM specialization

Inspect the diff plus runtime/compiler registries. Prove no new temporal-specific:

```text
Value arm
heap object kind
bytecode/opcode
compiler recognition by names such as Duration/Date/Timestamp/Instant/UtcOffset
VM calendar arithmetic
special dispatch path
```

was introduced.

A focused disassembly/runtime test should show representative P1 source using ordinary construction/method dispatch/data behavior if the existing observability test infrastructure makes this cheap. Static diff/search evidence is acceptable for the negative “no new opcode/value arm” claim and should be captured in the walkthrough.

#### P2 exclusion / cheap import

Confirm dependency and initialization diffs contain no timezone database crate, named-zone backend, system timezone probe, TimeZone/ZonedDateTime declarations, or tzdb state.

#### P1-CV-50 under approved documentation deferral

Do **not** compare against or edit stale `time.md` files.

Instead compare the final public declaration/export/prelude surface against:

```text
STDL002.C2-CHECKPOINT.md
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-01-body.md
semantic/publication tests
ordinary-package fixture
```

Record the old specification leg as `USER-DEFERRED` in the walkthrough/checkpoint amendment record.

### Forbidden approaches

- changing scheduler semantics to accommodate Duration migration;
- storing public Instant objects in reactor queues without a separate need;
- time-specific reflection code;
- adding native implementations for source arithmetic;
- temporal VM opcodes/value arms;
- adding timezone dependencies or stub API;
- editing stale `time.md` files;
- broad documentation cleanup.

### Test changes required

Add focused tests with stable `time_p1_` prefixes where practical:

```text
modules_runtime::time_p1_*
type_metadata_invariants::time_p1_*
native_surface_contracts::time_p1_*
observability_disassembly::time_p1_*   # only if added
```

Language fixtures should cover sleep/timeout with migrated Duration if existing concurrency integration is best exercised there.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_runtime::time_p1_
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core type_metadata_invariants::time_p1_
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts::time_p1_
```

If a focused disassembly test is added:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core observability_disassembly::time_p1_
```

Run the exact existing concurrency-focused filter located in T0 for `System.sleep` / `Future.timeout` if those behaviors are not already exercised by the time corpus. Record the exact command rather than inventing a new broad suite.

### Tests explicitly deferred

```text
full workspace
full core unless G5 evidence requires it
full semantic unless G5 evidence requires it
P2 timezone/zoned/text suites
release certification
stale time specification validation
```

### Acceptance

`P1-CV-43` through `P1-CV-49` have direct evidence. `P1-CV-50` has plan/API agreement evidence with its stale-spec leg explicitly `USER-DEFERRED`. Concurrency behavior remains intact, reflection/provenance is ordinary, and the diff contains no temporal runtime specialization or P2 dependency.

### Local STOP / CONSULT triggers

Stop if Duration migration appears to require reactor redesign, reflection requires time-specific recognition, provenance cannot distinguish primitive implementation kind from metadata, or a runtime special case appears necessary for correctness.

### Checkpoint update

Defer completion state until G5 and §20. Record any meaningful C/D failure discovered here immediately if it would otherwise be forgotten.

---

# 19. Verification gates

Gates are coherent stabilization points. They are not invitations to run every test in the repository. Confirm every named filter selects actual tests on execution HEAD.

## G1 — Language prerequisite readiness

### Entry condition

T0 complete and T1 either:

- verified all prerequisite mechanisms already landed; or
- completed the smallest authorized general closure.

### Required evidence

```text
grouped import/export available
lexical private/protected/module/package/internal available
protected eligibility is inheritance-based
native/primitive implementation kinds available
@intrinsic/@primitive non-operative metadata model available if used
bodyless abstract class requirements usable
concrete unresolved abstract subclass rejected
validated transparent data construction available
generic data identity available
stable class-side singleton/package exported value identity available
negative arithmetic primitives sufficient for Gregorian source implementation
```

### Commands

Run only the owning tests corresponding to mechanisms T1 actually changed. If T1 was verification-only, use the existing exact tests discovered in T0 and do not manufacture duplicate suites.

Typical changed-path filters:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic abstract_member_
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic validated_data_
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core compiler_declaration_dispatch::abstract_member_
```

### Pass condition

Every prerequisite needed by T2–T6 is either proven live or deliberately resolved through an accepted narrow patch. No time-specific workaround exists.

### Gate failure policy

- A/B failures: fix within T1 ownership.
- C: one bounded classification pass.
- D: record and continue if nonblocking.
- architectural mismatch: consult immediately.

### Checkpoint event

Record G1 only if it establishes a new durable language prerequisite relied on by P1 or a consultation/amendment occurred.

---

## G2 — Monotonic migration stabilization

### Entry condition

T2 and T3 complete.

### Coverage emphasized

```text
P1-CV-01 through P1-CV-13
P1-CV-39 through the System.clock removal portion
P1-CV-40/41 where needed for Instant/primitive access
```

### Commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts::time_monotonic_
```

Also run any exact ordinary-module singleton identity test already added in T3 if it is not selected by those commands.

### Required manual/static evidence

```text
no public System.clock declaration
no legacy time-owned raw Instant selector convention
monotonic primitive owner resolves to SystemClock
Clock.now is an abstract obligation, not throwing source
clock singleton construction itself performs no host read
```

### Pass condition

The C1 monotonic semantics have been migrated without preserving a parallel old façade and custom clock tests prove domain identity.

### Checkpoint event

Record the stable monotonic P1 interface and primitive ownership as a durable C2 fact.

---

## G3 — Absolute-time stabilization

### Entry condition

T4 complete and G2 still green after the T4 changes.

### Coverage emphasized

```text
P1-CV-14 through P1-CV-19
```

### Commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts::time_wall_
```

### Required manual/static evidence

```text
Timestamp canonical state is signed Int nanoseconds
wallClock === SystemWallClock.instance
Timestamp.now calls canonical wallClock path
only SystemWallClock owns host wall observation
no System.now/System.wallClock
no Float/host-datetime canonical state
```

### Pass condition

Absolute time is a global value timeline with one canonical wall capability and primitive host source.

### Checkpoint event

Record Timestamp/WallClock/SystemWallClock/wallClock stable interface and primitive ownership.

---

## G4 — Civil/calendar stabilization

### Entry condition

T5 and T6 complete.

### Coverage emphasized

```text
P1-CV-20 through P1-CV-34
```

### Commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
```

If property/round-trip cases are housed in a more specific existing core integration module, run its exact focused filter as recorded by T0/T6; do not broaden to all core tests just to run the calendar loop.

### Required evidence

```text
leap rules 2000/1900/2024
civil <-> epoch-day round trip over representative negative/positive range
1970-01-01 and 1969-12-31 anchors
Time rejects second 60
Date invalid-state rejection
DateTime has no timezone/Duration arithmetic
Period has no total order
Jan31 Constrain and Reject
months -> overflow -> days discriminator
YearMonth/MonthDay validation
UtcOffset exact selected bound + second precision
no named-zone semantics
```

### Pass condition

All P1 civil values have canonical transparent state, one Gregorian arithmetic authority, and validated public construction.

### Checkpoint event

Record the stable civil value contracts and UtcOffset bound. If the bound required consultation, record the approved decision.

---

## G5 — P1 integration stabilization

### Entry condition

T7 and T8 complete; G2–G4 evidence still valid.

### Coverage emphasized

```text
P1-CV-35 through P1-CV-50
plus regression confirmation for P1-CV-01 through P1-CV-34
```

### Required focused commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus time_negative
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_runtime::time_p1_
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core type_metadata_invariants::time_p1_
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core native_surface_contracts::time_p1_
```

If present:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core observability_disassembly::time_p1_
```

Also run the exact focused concurrency filter discovered in T0 if sleep/timeout coverage is not already selected by the time corpus.

### Conditional broader verification

Broaden beyond these commands only when one of the following is true:

1. T1 changed shared semantic/compiler infrastructure with plausible non-time blast radius;
2. a focused failure indicates a shared regression outside the time filters;
3. the lifecycle convention explicitly requires a broader affected-crate gate for the actual diff.

When warranted, choose the smallest next ladder rung, for example:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
```

or:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
```

Do not automatically run both, and do not run `cargo test --workspace` merely because P1 reached G5.

### Required static/diff evidence

Use targeted searches/diff inspection to confirm:

```text
no System.clock
no special time implementation-selector convention
no temporal Value arm
no temporal opcode
no compiler name check for Date/Duration/Timestamp/Instant/UtcOffset
no timezone dependency
no TimeZone/ZonedDateTime declarations
no stale time.md edits
```

### Pass condition

Every P1 coverage row is either PASS with named evidence or, only where explicitly authorized, USER-DEFERRED. No A/B failures remain. Any C/D failure is recorded according to §17 of the header. The P1 source/API is coherent enough for P2 to consume without redesign.

### Checkpoint event

Record G5 result, actual commands, failure classifications, and verification level. Do not yet mark P1 complete until §20 acceptance and lifecycle deliverables are complete.

---

# 20. Final focused acceptance

P1 final acceptance is coverage-ledger driven. Each row below must be checked against actual implementation evidence; do not mark a row complete merely because a neighboring test passed.

`PASS` below means “required before P1 completion.” `USER-DEFERRED` is permitted only for the stale specification leg of `P1-CV-50` under `P1-BODY-DEV-01`.

| Coverage ID | Owning task/gate | Required final evidence | Acceptance state |
|---|---|---|---|
| `P1-CV-01` | T2 / G2 | Duration equal-nanosecond value equality/hash plus signed zero/negative cases | PASS |
| `P1-CV-02` | T2 / G2 | integer ns/us/ms/s/min/h factories and exact projections | PASS |
| `P1-CV-03` | T2 / G2 | fractional factory/wrong operand rejected by normal typing/validation | PASS |
| `P1-CV-04` | T2 / G2 | `Measurement<T>` preserves value type and measured Duration with canonical applied identity | PASS |
| `P1-CV-05` | T3 / G2 | repeated `SystemClock.instance`, `clock`, and cross-module imports are identical object | PASS |
| `P1-CV-06` | T1+T3 / G1+G2 | Clock direct construction unavailable; concrete subclass must implement `now` | PASS |
| `P1-CV-07` | T3 / G2 | deterministic custom Clock mints its own Instants through protected seam | PASS |
| `P1-CV-08` | T3 / G2 | same ticks on distinct Clock objects compare unequal by equality | PASS |
| `P1-CV-09` | T3 / G2 | cross-domain Instant compare/since rejects | PASS |
| `P1-CV-10` | T3 / G2 | Instant +/- Duration preserves originating clock and exact delta | PASS |
| `P1-CV-11` | T3 / G2 | Instant.elapsed consults originating custom Clock | PASS |
| `P1-CV-12` | T3+T7 / G2+G5 | ordinary user raw Instant construction rejected; package/subclass intended lanes succeed | PASS |
| `P1-CV-13` | T3+T8 / G2+G5 | primitive provenance says SystemClock owns monotonic host read; System temporal bridge absent | PASS |
| `P1-CV-14` | T4 / G3 | Timestamp epoch and signed scalar factories/projections | PASS |
| `P1-CV-15` | T4 / G3 | Timestamp plus/minus/since exact Duration laws | PASS |
| `P1-CV-16` | T4+T6 / G3+G4 | -1ns split/conversion proves floor/Euclidean pre-epoch behavior | PASS |
| `P1-CV-17` | T4 / G3 | `wallClock === SystemWallClock.instance`; repeated import identity stable | PASS |
| `P1-CV-18` | T4+T8 / G3+G5 | source/provenance inspection proves `Timestamp.now` delegates canonical wallClock and has no second primitive | PASS |
| `P1-CV-19` | T4+T8 / G3+G5 | primitive provenance says SystemWallClock owns wall observation | PASS |
| `P1-CV-20` | T5+T6 / G4 | 2000 leap, 1900 non-leap, 2024 leap, representative year-zero/negative model cases | PASS |
| `P1-CV-21` | T5+T6 / G4 | civil→epoch-day→civil deterministic property loop across pre/post epoch/leap boundaries | PASS |
| `P1-CV-22` | T6 / G4 | known weekdays including epoch-adjacent dates | PASS |
| `P1-CV-23` | T1+T6 / G4 | invalid Date states reject through general validated-data construction | PASS |
| `P1-CV-24` | T1+T6 / G4 | Time boundary validation; second 60 rejects | PASS |
| `P1-CV-25` | T6 / G4 | DateTime exact Date+Time composition; reflection/state has no zone/offset | PASS |
| `P1-CV-26` | T5+T6 / G4 | years→months, weeks→days, arithmetic/negation | PASS |
| `P1-CV-27` | T5+T6 / G4 | semantic negative case proves Period has no total-order API/conformance | PASS |
| `P1-CV-28` | T6 / G4 | Jan31 +1 month Constrain -> target-month last day | PASS |
| `P1-CV-29` | T6 / G4 | Jan31 +1 month Reject produces normal error rather than clamping | PASS |
| `P1-CV-30` | T6 / G4 | mixed months+days test distinguishes months→overflow→days from reordered evaluation | PASS |
| `P1-CV-31` | T6 / G4 | YearMonth month arithmetic/day count/atDay validation | PASS |
| `P1-CV-32` | T1+T6 / G4 | impossible MonthDay rejects; Feb29 exists; non-leap inYear rejects | PASS |
| `P1-CV-33` | T6 / G4 | UtcOffset signed-seconds state, second precision, negation/order and evidence-based bound rejection | PASS |
| `P1-CV-34` | T6+T8 / G4+G5 | API/reflection/diff contains no named/DST/timezone-rule semantics in UtcOffset | PASS |
| `P1-CV-35` | T7 / G5 | ordinary package grouped import resolves multiple time declarations correctly | PASS |
| `P1-CV-36` | T7 / G5 | grouped time root publication resolves canonical declarations/values | PASS |
| `P1-CV-37` | T7 / G5 | Duration/Timestamp unqualified; Clock/Instant/Date/clock and other non-prelude names fail unqualified | PASS |
| `P1-CV-38` | T7 / G5 | explicitly imported Duration/Timestamp share canonical declaration identity with prelude | PASS |
| `P1-CV-39` | T3+T7 / G2+G5 | `System.clock` no longer resolves and no System wall/date/time discovery getters exist | PASS |
| `P1-CV-40` | T1+T3+T7 / G1+G5 | private/protected/module/package access tests use canonical identity; ordinary-user negatives pass | PASS |
| `P1-CV-41` | T1+T8 / G1+G5 | metadata presence/absence demonstrably does not create linkage/access semantics | PASS |
| `P1-CV-42` | T7 / G5 | representative ordinary project imports/constructs/uses P1 values and `clock` normally | PASS |
| `P1-CV-43` | T8 / G5 | reflection observes correct data/class/enum categories and canonical type identities | PASS |
| `P1-CV-44` | T8 / G5 | `System.sleep(Duration)` consumes migrated Duration and keeps existing negative-wait boundary | PASS |
| `P1-CV-45` | T8 / G5 | `Future.timeout(Duration)` preserves timeout/suspension behavior with migrated Duration | PASS |
| `P1-CV-46` | T8 / G5 | reactor inspection/tests show host deadline representation remains internal and public Instant is not forced into timer storage | PASS |
| `P1-CV-47` | T8 / G5 | diff/search/disassembly evidence: ordinary source/data dispatch, no temporal opcode/value arm/name-special case | PASS |
| `P1-CV-48` | T7+T8 / G5 | import/init inspection proves no tzdb/system-zone/P2 resource initialization; singleton creation is cheap | PASS |
| `P1-CV-49` | T7+T8 / G5 | dependency/API diff has no TimeZone/ZonedDateTime/named-zone backend or P2 declarations | PASS |
| `P1-CV-50` | T8 / G5 | implemented API agrees with checkpoint + shared header + this body + executable publication tests; existing `time.md` specification evidence is explicitly deferred | PASS for plan/API agreement; stale-spec leg USER-DEFERRED |

## 20.1 Final acceptance procedure

After G5 passes:

1. inspect `git diff --stat` and scoped diff for all P1-owned areas;
2. verify each `P1-CV-*` row above has a named test, inspection, or provenance record;
3. verify every affected `P1-INV-*` remains true;
4. verify no A/B failure remains;
5. verify all C/D failures are recorded with classification and nonblocking rationale;
6. verify T1 did not leave a temporary general-language workaround;
7. verify `System.clock` and time-owned legacy selector conventions are absent;
8. verify package/prelude identity and ordinary package use;
9. verify no P2 dependency/surface was pulled in;
10. verify no stale `time.md` file was edited or used as acceptance authority;
11. complete §21 evidence, §22 checkpoint bookkeeping, §23 walkthrough, and §24 handoff;
12. only then mark the P1 plan complete under the lifecycle convention.

## 20.2 Final focused command record

The walkthrough must record the exact commands actually executed and their results, not merely copy this plan's proposed commands. If an execution-HEAD rename required a different filter, record both the reason and the actual nonzero filter.

Do not claim “all tests pass” unless a genuinely all-inclusive suite was run. The intended P1 claim is narrower: all required P1 focused acceptance gates passed, with broader suites only where evidence justified them.

---

# 21. Performance/resource evidence

P1 does not require benchmark ceremony. It does require enough evidence that the architecture has not accidentally introduced the costs the checkpoint forbids.

## 21.1 Required resource claims

### Clock import/initialization

Prove by source/init inspection that:

```text
importing time
    may construct SystemClock.instance
    may construct SystemWallClock.instance
    does not read either clock
    does not initialize timezone state
    does not probe system timezone configuration
    does not load external data
```

### Monotonic `now`

Expected hot-path shape:

```text
ordinary method dispatch to SystemClock.now
    -> one irreducible monotonic host observation
    -> ordinary Instant construction in same Clock domain
```

No repeated package lookup, timezone work, or Rust calendar conversion should be present.

### Wall `now`

Expected hot-path shape:

```text
Timestamp.now
    -> canonical wallClock capability
    -> SystemWallClock.now
    -> one irreducible wall host observation
    -> ordinary Timestamp data construction
```

There must not be two host observations for one call.

### Gregorian/civil operations

Core Gregorian conversion and Date/Period operations must be bounded constant-time integer arithmetic for one operation. No database, filesystem, system timezone, locale, or native date library is involved.

### Reactor boundary

Existing reactor/timer code may continue using efficient host `Instant`/`Duration`-like deadline types internally. P1 must not impose public Phalcom `Instant` allocation/storage on each reactor deadline merely to unify names.

### Runtime representation

General data/product optimizations already authorized by LANG005 may apply. P1 itself adds no time-specific representation specialization.

## 21.2 Evidence to capture

Record in the walkthrough:

```text
package initialization path inspection
primitive call path for SystemClock.now
primitive call path for SystemWallClock.now
calendar helper algorithm complexity
reactor deadline representation inspection
Cargo/dependency diff showing no timezone backend
Value/opcode/compiler diff showing no temporal specialization
```

Useful targeted searches should be adapted to execution-HEAD paths, for example:

```sh
rg -n 'System\.clock|System\.now|System\.wallClock|System\.timeZone' phalcom-core/core/universe/src
rg -n 'TimeZone|ZonedDateTime|tzdb|TZif|jiff' Cargo.toml phalcom-core Cargo.lock
rg -n 'Value::(Duration|Timestamp|Date|Instant|UtcOffset)|MAKE_DATE|ADD_TIMESTAMP|ADVANCE_PERIOD|READ_CLOCK' phalcom-core
```

Interpret matches carefully; historical comments/tests may not be production special cases. Use scoped diff/ownership evidence rather than raw match count.

## 21.3 When to benchmark

Run a benchmark only if one of these occurs:

- a focused regression test or profiler shows a material performance regression;
- the migration adds an unexpected allocation/lookup on a hot clock path;
- Gregorian source arithmetic is unexpectedly superlinear or allocates in a loop;
- the user explicitly requests measured performance evidence.

If no such signal exists, source-path/resource evidence is sufficient for P1.

---

# 22. Checkpoint bookkeeping

`STDL002.C2-CHECKPOINT.md` remains the only durable C2 state ledger. Do not create a P1 checkpoint shadow file.

## 22.1 At P1 start — T0

Update the existing P1 ledger entry from proposed/not-started to the lifecycle convention's active/in-progress state and record:

```text
execution start revision
active plan = STDL002.C2.P1
active task = T0 if the checkpoint format records it
P1-BODY-DEV-01: stale existing time.md documents excluded; authoritative time specification deferred
```

Preserve C1 historical truth.

## 22.2 During P1

Update the checkpoint only for durable events:

```text
approved consultation/amendment
new stable interface P2 must consume
selected UtcOffset bound
meaningful C/D failure that survives the active task
G2 monotonic stabilization result
G3 absolute-time stabilization result
G4 civil stabilization result
G5 integration stabilization result
```

Do not log every helper rename, local test rerun, or temporary compilation error.

## 22.3 Required stable interface record by P1 completion

The checkpoint should summarize, without duplicating the plan, that P1 established:

```text
Duration data exact signed nanoseconds
Measurement<T>
abstract Clock
SystemClock + singleton + exported clock
opaque clock-domain Instant
SystemClock primitive monotonic observation
Timestamp signed Unix/POSIX nanoseconds
abstract WallClock
SystemWallClock + singleton + exported wallClock
Timestamp.now via canonical wallClock
SystemWallClock primitive wall observation
Month / Weekday
Date / Time / DateTime
YearMonth / MonthDay
Period / DateOverflow
UtcOffset + selected bound
prelude = Duration + Timestamp
System.clock removed
concurrency Duration consumers preserved
no temporal VM specialization
no P2 timezone surface
specification work deferred by explicit user ruling
```

## 22.4 P1 completion state

After §20, §23, and §24 are complete, update the P1 ledger using the exact lifecycle tokens current at execution time. The semantic state must mean:

```text
plan status: complete
implementation completion: complete
verification: focused P1 acceptance complete
```

Do **not** mark C2 complete/accepted. P2 remains outstanding.

If the lifecycle convention distinguishes `FOCUSED_TESTED` from `VERIFIED`, use the narrower truthful token corresponding to the commands actually run. Do not claim workspace/release certification unless it was separately performed.

## 22.5 Failure bookkeeping

For each meaningful C/D failure record:

```text
command
failure/test name
classification C or D
bounded evidence for non-P1 causality or uncertainty
whether it blocks a required P1 gate
next owner if known
```

Never weaken a P1 acceptance row to hide it.

---

# 23. Walkthrough deliverable

Create:

```text
STDL002.C2.P1-walkthrough.md
```

The walkthrough is an as-built implementation record, not a restatement of this plan.

## 23.1 Required structure

At minimum include these sections:

```text
1. Result / lifecycle state
2. Execution baseline and final revision
3. Approved deviations/amendments
4. Language prerequisites actually consumed/changed
5. As-built module/file topology
6. Duration and Measurement implementation
7. Monotonic model as built
8. Absolute wall-time model as built
9. Gregorian algorithm and civil values
10. UtcOffset selected bound and evidence
11. Package exports and prelude
12. Visibility/access model used
13. Primitive/native inventory
14. System boundary and removed C1 seams
15. Concurrency compatibility
16. Ordinary-package integration
17. Reflection/provenance evidence
18. No-temporal-specialization evidence
19. Performance/resource evidence
20. Coverage ledger P1-CV-01..50
21. Tests actually run and exact results
22. Deferred/baseline failures
23. Consultations
24. Remaining risks / P2 constraints
```

## 23.2 Required as-built details

The walkthrough must show actual code-level facts, including:

- actual live syntax used for visibility, abstract members, `native`, `primitive`, and metadata;
- actual primitive declaration/linkage mechanism and Rust holder paths;
- actual module/file locations after mechanical drift;
- actual validated-data construction mechanism;
- actual singleton storage/package binding mechanism;
- actual Gregorian floor/Euclidean helper strategy;
- actual UtcOffset bound and why it was selected;
- actual ordinary-package fixture location;
- actual reflection/provenance test locations;
- actual reactor boundary that remained unchanged.

Do not claim future semantics based only on the plan if execution code differs mechanically; document the final mechanics.

## 23.3 Coverage ledger in walkthrough

For every `P1-CV-01` through `P1-CV-50`, record:

```text
coverage ID
evidence type: test / inspection / provenance / diff
exact test name or file/symbol inspected
result
related P1-INV IDs
```

For `P1-CV-50`, explicitly state:

```text
plan/API agreement: PASS
existing time.md specification evidence: USER-DEFERRED by explicit instruction
```

No stale `time.md` content should be quoted or summarized.

## 23.4 Walkthrough quality bar

A later P2 planner should be able to answer “what exactly exists now?” from the walkthrough without reopening P1 design questions or reverse-engineering the diff.

---

# 24. Handoff deliverable

Create:

```text
STDL002.C2.P1-handoff.md
```

The handoff is specifically for the next C2 phase. It must be concise enough to load quickly but complete enough that P2 does not redesign P1.

## 24.1 Required contents

### Repository state

```text
P1 final revision
branch
working-tree caveats
P1 checkpoint state
verification level
```

### Stable interfaces P2 may consume directly

List actual signatures/locations for:

```text
Duration
Measurement<T>
Clock
SystemClock
clock
Instant
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
```

### Stable semantic laws P2 must not reopen

At minimum:

```text
Duration != Period
Instant != Timestamp
DateTime is timezone-free
UtcOffset != TimeZone
Clock object identity defines Instant domain
SystemClock.instance === clock
SystemWallClock.instance === wallClock
Timestamp is signed Unix/POSIX nanoseconds
Date/Time use proleptic Gregorian/no leap second
Date.advance order = months -> overflow -> days
prelude = Duration + Timestamp only
System is not temporal discovery
```

### Internal seams P2 may rely on

Document actual P1 package-visible/helper interfaces deliberately suitable for P2, if any:

```text
civil <-> epoch-day conversion
Date/Time component validation helpers
Timestamp signed split/combine helpers
UtcOffset constructor/range helpers
canonical package/module identities
```

Do not promote helpers to public solely for the handoff.

### Host/native/primitive boundary

State exactly:

```text
monotonic host primitive owner/path
wall host primitive owner/path
native temporal methods in P1, expected none unless administrative runtime plumbing is below language surface
metadata annotations actually used
```

### UtcOffset interoperability

Record:

```text
selected numeric bound
second-resolution support
reason/evidence
any backend constraint P2 must respect
```

### Timestamp/civil range

Record whether P1 civil/Timestamp arithmetic is unbounded `Int` semantically and identify any explicit accepted boundary that P2 must handle when crossing into a finite native backend.

### Deferred failures/consultations

Carry forward only unresolved items that P2 needs to know.

### Specification status

State explicitly:

```text
existing time.md files were excluded from P1 as stale
authoritative time specification work remains deferred until explicitly reauthorized
P2 must not treat old time.md content as P1 contract
```

### P2 first re-grounding commands

Provide the same minimal session bootstrap:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Then point P2 to the completed checkpoint, P1 walkthrough, and P1 handoff. Do not draft P2 tasks in this handoff.

## 24.2 Explicit “do not redesign” list for P2

P2 handoff must repeat, briefly, that it must not reopen P1's:

```text
representation categories
clock identity law
Timestamp timeline
DateTime timezone-free meaning
Duration/Period distinction
UtcOffset fixed-value meaning
prelude policy
System façade removal
Gregorian arithmetic authority
primitive ownership of host clock observations
```

unless a new consultation/checkpoint amendment authorizes it.

---

# 25. Completion truth table

P1 completion statements must remain semantically precise.

| Claim | Required evidence | Does it imply the next row? |
|---|---|---|
| Source written | P1 production source edits exist | No |
| Tests added/migrated | Required focused tests/fixtures exist and select nonzero tests | No |
| Task accepted | Task-local acceptance criteria met and its focused BUILD tests pass | No |
| Gate passed | Named gate commands/evidence pass with failures classified | No |
| All P1 coverage mapped | Every `P1-CV-01..50` has evidence; only CV-50 stale-spec leg may be USER-DEFERRED | No |
| P1 focused acceptance passed | G1–G5 applicable evidence + §20 complete; no A/B failures | No |
| Walkthrough complete | As-built architecture/evidence recorded | No |
| Handoff complete | P2-consumable stable interfaces and constraints recorded | No |
| P1 complete | implementation + focused acceptance + checkpoint + walkthrough + handoff complete | No |
| C2 complete | P1 **and P2** complete plus checkpoint acceptance objective satisfied | No |
| Workspace/release certified | explicitly requested broad release/workspace verification completed | N/A |

## 25.1 Allowed final P1 statement

If only the planned focused verification was run, the truthful completion form is equivalent to:

```text
STDL002.C2.P1 implementation is complete and focused P1 acceptance is complete.
Required P1 semantic/runtime/module/reflection/concurrency/provenance gates passed.
C2 remains incomplete pending P2.
Workspace/release certification was not implied or performed unless separately recorded.
Existing time specification work remains deferred by explicit instruction.
```

Do not compress those distinctions into “C2 time is done.”

---

# 26. Plan self-review

Before handing this body to an implementer, and again before P1 completion, review it against the following checklist.

## 26.1 Architecture consistency

- [ ] Every task preserves all applicable `P1-INV-*` invariants.
- [ ] No task changes the P1/P2 boundary.
- [ ] `Duration` and `Timestamp` are the only temporal prelude names.
- [ ] `SystemClock` / `SystemWallClock` remain public/exported concrete classes.
- [ ] `clock` / `wallClock` remain exact singleton bindings.
- [ ] Clock object identity remains Instant domain identity.
- [ ] `Timestamp` remains signed Unix/POSIX nanoseconds.
- [ ] `DateTime` remains timezone-free.
- [ ] `Period` remains months+days with no total order.
- [ ] `UtcOffset` remains fixed signed seconds, not TimeZone.
- [ ] no temporal VM specialization is planned.
- [ ] no derivable temporal behavior was moved to Rust for convenience.

## 26.2 Language-transition consistency

- [ ] Future ratified visibility semantics are treated as target authority even if planning baseline code is old.
- [ ] protected access is inheritance-oriented, not a linear lattice.
- [ ] no selector/field spelling creates visibility.
- [ ] `native` and `primitive` are implementation kinds.
- [ ] `@intrinsic` and `@primitive` remain metadata-only.
- [ ] T1 does not duplicate the wider visibility/native/primitive transition.
- [ ] abstract-member closure, if needed, is general and non-throwing.
- [ ] validated data construction, if needed, is general and preserves transparency.

## 26.3 Task dependency quality

- [ ] T0 is read-only except checkpoint activation.
- [ ] T1 is conditional and can cleanly be `NO CODE REQUIRED`.
- [ ] T2 establishes values before clocks depend on them.
- [ ] T3 completes monotonic migration before wall time.
- [ ] T4 completes absolute time before civil integration.
- [ ] T5 owns one Gregorian arithmetic authority.
- [ ] T6 consumes that authority rather than duplicating formulas.
- [ ] T7 waits for stable public declarations before final package/prelude publication.
- [ ] T8 stabilizes integration without redesigning subsystems.

## 26.4 Testing discipline

- [ ] Every task has explicit tests-to-run-now and explicit deferrals.
- [ ] BUILD commands are discriminating and narrow.
- [ ] filters are confirmed to select actual tests.
- [ ] broader suites occur only at named gates or when evidence justifies them.
- [ ] every `P1-CV-01..50` maps to concrete final evidence.
- [ ] property-style civil coverage uses deterministic loops unless an existing framework already exists.
- [ ] access tests prove identity-based visibility rather than special-name rejection.
- [ ] ordinary-package use is tested outside Universe privilege.
- [ ] reflection/provenance tests assert ordinary categories and implementation kind.
- [ ] concurrency tests verify behavior without turning P1 into a reactor redesign.

## 26.5 Escalation discipline

- [ ] every task has local STOP/CONSULT triggers.
- [ ] shared-header global triggers remain in force.
- [ ] UtcOffset bound ambiguity is not guessed.
- [ ] year-range restrictions are not invented casually.
- [ ] one serious semantic correction limit is preserved.
- [ ] architectural failures receive zero speculative redesign attempts.
- [ ] unrelated C/D failures are recorded rather than absorbed into P1 scope.

## 26.6 Lifecycle discipline

- [ ] checkpoint remains the only durable state ledger.
- [ ] C1 history remains untouched/truthful.
- [ ] checkpoint is updated only at durable events.
- [ ] walkthrough records as-built facts rather than plan intentions.
- [ ] handoff gives P2 stable interfaces without drafting P2.
- [ ] P1 completion does not imply C2 completion.
- [ ] focused P1 verification does not imply workspace/release certification.

## 26.7 Approved documentation/specification deviation

- [ ] no existing `time.md` file is treated as authority.
- [ ] no P1 task edits `docs/spec/current/stdlib/time.md`.
- [ ] no P1 task edits `docs/implementation/time.md`.
- [ ] checkpoint records the explicit user deferral.
- [ ] `P1-CV-50` is satisfied through plan/API/source-test agreement, with the stale-spec leg marked USER-DEFERRED.
- [ ] P1 handoff tells P2 that specification work remains deferred until explicitly reauthorized.

## 26.8 Final executor readiness test

The body is implementation-grade only if a Luna implementer can answer all of the following without rediscovering architecture:

```text
What do I edit first?
What is fixed versus mechanically flexible?
Which current legacy seams must disappear?
What language prerequisites may I close, and which must I not own?
How is each temporal category represented?
Who owns each host observation?
How are Instant domains protected?
How do negative/pre-epoch calendar operations work?
What validates civil data?
What is the Period application order?
When must I consult about UtcOffset?
What exactly is in the prelude?
How do I prove ordinary-package equivalence?
Which concurrency/reflection edges must remain compatible?
Which tests run after each slice?
When may testing broaden?
What does P1 completion truthfully mean?
What does P2 inherit without redesign?
Why is existing time.md material absent from this work?
```

If any answer still requires an architectural choice rather than a mechanical live-repository lookup, the implementer must resolve it through the applicable STOP/CONSULT path before coding past that point.
