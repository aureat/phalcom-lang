---
id: STDL002.C2.P1.PLAN-HEADER
parent_plan: STDL002.C2.P1
category: STDL
program: STDL002
checkpoint: STDL002.C2
kind: implementation-plan-shared-header
planning_cycle: analysis-pass-2
file_pass: 1-of-2
status: PROPOSED
completion: HEADER_COMPLETE
verification: REPOSITORY_GROUNDED_AND_STRUCTURALLY_VERIFIED
prepared: 2026-09-17
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
planning_revision_subject: "feat(lang005): continue P5 iterator surface"
depends_on:
  - STDL002.C1
  - STDL002.C2-CHECKPOINT.md
  - STDL002.C2-GUIDANCE.md
follows: STDL002.C1.P1
supersedes: null
executor_profile: GPT-5.6-Luna-High-or-Extra-High
---

# STDL002.C2.P1 — Shared Plan Header: Temporal Foundation, Absolute Time, and Civil Calendar

> **For agentic workers:** this file is the mandatory shared header for `STDL002.C2.P1`. Read it first, then read the P1 body file and only the live repository files named by the active task. The body may specialize this header with task-local constraints, but it may not weaken or contradict any `FIXED` decision, `P1-INV-*` invariant, ownership boundary, or STOP/CONSULT trigger defined here.
>
> **Required execution style:** use the repository's Luna patch-grade workflow. Work in coherent reviewer-sized tasks; use sparse discriminating tests in BUILD MODE; broaden only at named gates; classify unrelated failures; bound debugging; maintain the C2 checkpoint only at durable events; and stop before improvising architecture.
>
> **Two-file plan-set rule:** this header owns plan-wide architecture and execution policy. `STDL002.C2.P1-plan-01-body.md` will own ordered tasks, exact files/symbols, task-local implementation shape, focused commands, gates, final acceptance, bookkeeping, walkthrough/handoff, and plan self-review.

**Goal:** Upgrade the completed C1 monotonic foundation into the stable P1 temporal substrate: modernize monotonic time around `Duration`/`Clock`/`SystemClock`/`Instant` and exported `clock`, add absolute wall time through `Timestamp`/`WallClock`/`SystemWallClock` and exported `wallClock`, implement the pure Gregorian civil-value layer through `UtcOffset`, and migrate package/prelude/concurrency/reflection contracts to the newly ratified visibility and `native`/`primitive` model without entering named-timezone/zoned/textual P2 scope.

**Architecture:** Ordinary Phalcom source owns temporal values, arithmetic, calendar algorithms, validation, capability composition, and public behavior. Identity-bearing or host-backed capabilities remain classes; transparent immutable semantic products become `data`; closed policies/categories become enums. Irreducible host observations live on the concrete system clock classes as `primitive` methods, while no P1 operation becomes `native` merely for convenience. The `time` package exports canonical clock singleton values, the prelude promotes only `Duration` and `Timestamp`, and the normal semantic/compiler/runtime pipeline remains authoritative for every other temporal operation.

**Tech stack:** Phalcom Universe source; first-class `data`; classes/inheritance/abstract members; enum/product semantics; canonical module/package/project identity; lexical visibility; tuple-shaped grouped imports/exports; Rust host-clock bridge through the evolving Phalcom `primitive` implementation-kind infrastructure; `phalcom-semantic`; `phalcom-core`; arbitrary-precision `Int`; ordinary VM values/objects/GC/reflection; existing concurrency reactor/timer consumers.

**Primary design / execution authorities:** `STDL002.C2-CHECKPOINT.md`; `STDL002.C2-GUIDANCE.md`; `STDL002.C1-CHECKPOINT.md`; accepted `data` decision `docs/decisions/accepted/0080-data-is-a-nominal-transparent-immutable-value-product.md`; current effective language specifications for modules/imports/exports, visibility, `data`, classes/abstract members, native/primitive declarations, enums, reflection, and errors; current C1 `docs/spec/current/stdlib/time.md` only for predecessor behavior not explicitly superseded by C2.

---

# 0. Plan-set contract and navigation

## 0.1 Intended repository location

Keep all C2 records together under the live repository's C2 checkpoint directory. If no C2 directory has been established when implementation begins, use:

```text
docs/implementation/STDL002-standard-library-buildout/C2-time-upgrade/
```

with:

```text
STDL002.C2-CHECKPOINT.md
STDL002.C2-GUIDANCE.md

STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-plan-01-body.md

STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md
```

If the repository has already established a mechanically different C2 directory before implementation, use that live directory and do not duplicate records.

## 0.2 File-set ownership

| File | Role | Owns |
|---|---|---|
| `STDL002.C2-CHECKPOINT.md` | checkpoint state authority | P1/P2 state, durable interfaces/invariants, consultations, deferred failures, next action |
| `STDL002.C2-GUIDANCE.md` | persistent C2 implementation guidance | cross-plan architectural/execution rules |
| `STDL002.C2.P1-plan-00-shared-header.md` | this file | P1 executor contract, repository grounding, authority, architecture, ownership, invariants, non-goals, coverage design, verification budget |
| `STDL002.C2.P1-plan-01-body.md` | next planning deliverable | ordered P1 tasks, exact implementation contracts, tests, gates, final acceptance and lifecycle deliverables |
| `STDL002.C2.P1-walkthrough.md` | implementation result | as-built P1 architecture and evidence |
| `STDL002.C2.P1-handoff.md` | continuation state | stable interfaces and prerequisites consumed by P2 |

Do not create a separate `requirements-analysis.md` dependency for P1. The second analysis pass is encoded into this header and the P1 body so an implementer can execute from the plan set plus its named authorities.

## 0.3 Header/body precedence

Within P1:

```text
explicit approved consultation/amendment
    ↓
this shared header for plan-wide architecture/policy
    ↓
P1 body for task-local mechanics
    ↓
older design/proposal documents
```

Normative language decisions and explicit user-ratified C2 decisions remain above both files.

If the body appears to require weakening a header invariant, do not select whichever wording is convenient. STOP AND CONSULT.

## 0.4 Where this header stops

This file intentionally ends after schema section `# 17. Baseline / unrelated failure policy`.

The P1 body must begin with:

```text
# 18. Tasks
```

and continue the canonical Luna plan schema through:

```text
# 19. Verification gates
# 20. Final focused acceptance
# 21. Performance/resource evidence
# 22. Checkpoint bookkeeping
# 23. Walkthrough deliverable
# 24. Handoff deliverable
# 25. Completion truth table
# 26. Plan self-review
```

Do not duplicate sections 0–17 in the body.

---

# 1. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High working under constrained architectural authority.

The principal-architect pass has already selected the temporal taxonomy, package exposure, ownership model, representation categories, host boundary, visibility model, and P1/P2 split. The implementer should spend most of its time implementing and verifying those decisions rather than reopening them.

The implementer:

- may adapt private helper names, file-local decomposition, imports, mechanical signature drift, exact test fixture names, and equivalent live repository APIs;
- must preserve every `FIXED` item in §12 and every `P1-INV-*` invariant in §9;
- must re-read live repository state before editing because visibility and declaration implementation kinds are actively migrating;
- must not reintroduce spelling-based implementation selectors, hidden-name conventions, or special visibility attributes as a substitute for the ratified lexical access model;
- must treat `@intrinsic` and `@primitive` as metadata only, never as the mechanism that installs implementations or grants access;
- must distinguish the `native` implementation kind from the `primitive` implementation kind;
- must not classify an operation as primitive merely because Rust can implement it faster;
- must keep public temporal semantics in Phalcom source when the result is derivable from available values;
- must not create temporal VM `Value` arms, temporal bytecodes, compiler name-recognition, or special temporal dispatch;
- must not introduce named IANA timezones, `TimeZone`, `ZonedDateTime`, local ambiguity resolution, final ISO/RFC parsing/formatting, or tzdb dependencies in P1;
- must preserve `System.sleep(Duration)` / `Future.timeout(Duration)` behavior while removing date/time discovery from `System`;
- must classify broad failures A/B/C/D and continue immediately on clearly unrelated baseline failures;
- must make at most one serious correction for the same underlying nontrivial semantic failure before consulting;
- must keep `STDL002.C2-CHECKPOINT.md` current only at durable interfaces, coherent verification gates, consultations/amendments, and deferred failures;
- must not broaden testing merely because a focused test passed;
- must produce the required P1 walkthrough and handoff after implementation.

Testing is evidence gathering, not ritual.

---

# 2. Goal and checkpoint acceptance objective

## 2.1 P1 deliverable

P1 establishes every temporal value/capability contract that P2 will consume.

P1 is successful when all of the following are simultaneously true:

1. C1's `Duration` behavior has migrated to the canonical first-class `data` model without losing exact signed nanosecond semantics.
2. `Measurement<T>` exists as generic immutable transparent data.
3. `Clock` is the abstract monotonic-domain abstraction rather than the concrete system implementation.
4. `SystemClock` is a public concrete `Clock` implementation and owns the canonical system monotonic singleton.
5. `time::clock` is an exported value bound to that exact singleton.
6. `Instant` remains opaque and domain-bound; arbitrary user code cannot mint raw clock coordinates.
7. the irreducible monotonic host read is owned by `SystemClock`, not by the public `System` date/time façade.
8. `Timestamp` exists as signed `Int` Unix/POSIX nanoseconds and is promoted into the prelude.
9. `WallClock` is the abstract wall-time capability.
10. `SystemWallClock` is the public host implementation and owns its canonical singleton.
11. `time::wallClock` is an exported value bound to that singleton.
12. `Timestamp.now` obtains the host timestamp through the canonical wall-clock capability.
13. the irreducible wall-clock observation is a `primitive` method on the appropriate concrete system clock implementation.
14. `Month`, `Weekday`, `Date`, `Time`, `DateTime`, `YearMonth`, `MonthDay`, `Period`, `DateOverflow`, and `UtcOffset` exist with the P1 semantics in this header.
15. Gregorian conversion/arithmetic is ordinary Phalcom source and handles negative/pre-epoch values correctly.
16. invalid civil/fixed-offset values cannot enter the public semantic model through ordinary construction.
17. the `time` package is decomposed into coherent modules and exposes its root surface through grouped import/export forms.
18. the temporal prelude consists of `Duration` and `Timestamp`, not `Clock`/`Instant`.
19. public `System.clock` is removed; no public `System.now`, `System.wallClock`, or `System.timeZone` is introduced.
20. existing concurrency APIs consuming `Duration` continue to work.
21. semantic/reflection/runtime tests observe ordinary source-owned temporal declarations through the canonical package identity.
22. public P1 APIs are usable from ordinary user packages through normal imports and normal access control.
23. no P2 named-zone, zoned, timezone-database, or final textual grammar semantics have been pulled forward.

## 2.2 Relationship to C2

P1 is one of exactly two plans in C2.

P1 does **not** close C2. It creates the stable substrate for P2:

```text
P1
    exact elapsed time
    monotonic domains
    absolute timestamp
    civil Gregorian values
    fixed offset value
    package/prelude migration
        ↓
P2
    named/fixed TimeZone objects
    IANA/TZif backend
    local-time resolution
    ZonedDateTime
    system-local conveniences
    canonical parsing/formatting
    testing-clock package closure
    final certification/specification
```

P1 may update the current time specification to describe the P1 surface accurately, but P2 owns completion of the comprehensive timezone/zoned/textual specification.

---

# 3. Repository grounding

Prepared against remote `main`:

```text
repository: aureat/phalcom-lang
branch: main
revision: e0a77646c56f8b1a7e7692993d378d43bf3fc424
subject: feat(lang005): continue P5 iterator surface
predecessor checkpoint: STDL002.C1
predecessor plan: STDL002.C1.P1
```

A local checkout was not mutated during this planning pass. The implementation body must begin with a live local re-grounding task.

## 3.1 Verified predecessor state

At the planning baseline:

1. `STDL002.C1-CHECKPOINT.md` is `COMPLETE / COMPLETE / VERIFIED`.
2. C1 intentionally owns monotonic time only.
3. `phalcom-core/core/universe/src/time/clock.ph` currently contains `Duration`, `Instant`, and `Clock` as ordinary classes.
4. current `Duration` stores private nanoseconds and manually implements value equality/hash/arithmetic.
5. current `Instant` stores a `Clock` reference plus raw ticks and rejects cross-clock order/difference.
6. current `Clock` owns a source-level singleton accessor and obtains the host tick from `System`.
7. current `time/package.ph` re-exports `Clock`, `Instant`, and `Duration`.
8. the Universe root currently promotes `Clock`, `Instant`, and `Duration`.
9. semantic time tests currently type `System.clock`, `Clock`, `Instant`, and `Duration` as prelude-visible source-owned Universe declarations.
10. language fixtures currently encode `System.clock` and a legacy hidden host-read visibility test.
11. `System` is currently declared in `concurrency/fiber.ph`; it owns public `clock`, sleep, scheduler operations, and the legacy monotonic host-read bridge.
12. the Rust host read currently lives in the System primitive area and is registered as a System-side primitive.
13. `docs/spec/current/stdlib/time.md` currently describes only the C1 monotonic surface.
14. `docs/implementation/time.md` contains broader design input for the full time library but is not the P1 execution authority.
15. the STDL002 program still describes civil/timezone work as the later deep half of time, so C2 is a legitimate successor checkpoint rather than a retroactive redefinition of C1.

## 3.2 Verified language-model facts relevant to P1

At this baseline:

1. TDR-0080 is accepted and defines `data` as a distinct nominal transparent immutable value-product category.
2. `data` components are the complete semantic instance state; behavior is layered without hidden semantic storage.
3. `data` has value semantics and no observable allocation identity.
4. current repository implementation work for `data`, enum/product representation, reflection, and generic applied identity has already progressed substantially under LANG005.
5. current main still contains older decorator-era and presentation-era documents/code for `@native`, `@internal`, and `@intrinsic`.
6. current main also contains presentation infrastructure that can render intrinsic/native provenance.
7. those older repository forms are not authority over the newer user-ratified C2 rules:
   - visibility is lexical and identity-based;
   - `native` and `primitive` are method implementation kinds;
   - `@intrinsic` and `@primitive` are Universe-only metadata with no operative effect by presence alone;
   - grouped imports/exports use tuple-shaped organization.
8. the language transition is active, so P1 must verify what has landed at execution HEAD before deciding mechanics.

## 3.3 Planning interpretation of drift

The expected drift is known and intentional:

```text
C1 implementation reality
    old access/linkage conventions
        ↓ migrate
current ratified language model
    lexical identity-based visibility
    native/primitive implementation kinds
    metadata-only @intrinsic/@primitive
        ↓
P1 target time implementation
```

Do not treat the presence of old C1 conventions as evidence that P1 should preserve them.

Do not, however, implement an independent time-specific version of language work that is actively being completed elsewhere.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

# 4. Required reads before implementation

## 4.1 Workflow authority

Read:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/sol-principal-architect-project-doctrine.md
```

Use `docs/workflow/luna-implementer-prompt.md` when the execution environment expects it.

## 4.2 C2 state

Read the live C2 records:

```text
STDL002.C2-CHECKPOINT.md
STDL002.C2-GUIDANCE.md
STDL002.C2.P1-plan-00-shared-header.md
STDL002.C2.P1-plan-01-body.md
```

If continuing partially completed P1, additionally read:

```text
STDL002.C2.P1-walkthrough.md   # if already created/partial
STDL002.C2.P1-handoff.md       # if already created
```

## 4.3 Predecessor state

Read:

```text
docs/implementation/STDL002-standard-library-buildout/C1-time/STDL002.C1-CHECKPOINT.md
docs/implementation/STDL002-standard-library-buildout/C1-time/STDL002.C1.P1-monotonic-clock-instant-duration.md

docs/spec/current/stdlib/time.md
```

Use C1 as predecessor/as-built evidence, not as authority to veto C2 migrations explicitly defined here.

## 4.4 Normative language decisions/specs

Begin with:

```text
docs/specs/README.md
docs/decisions/accepted/0080-data-is-a-nominal-transparent-immutable-value-product.md
```

Then locate and read the current effective authorities for:

```text
modules / import / export
lexical visibility / access control
class inheritance and abstract declarations
abstract member obligations
data construction and behavior
enum declarations
generic data/applications
native method implementation kind
primitive method implementation kind
@intrinsic metadata
@primitive metadata
reflection/provenance
errors
```

Because those surfaces are moving, do not assume a historical filename remains authoritative.

## 4.5 Live source

Read the live equivalents of:

```text
phalcom-core/core/universe/src/time/package.ph
phalcom-core/core/universe/src/time/clock.ph
phalcom-core/core/universe/src/package.ph

phalcom-core/core/universe/src/concurrency/fiber.ph
phalcom-core/src/primitive/system.rs
phalcom-core/src/universe/primitives.rs
```

Then inspect only the current implementation-kind/linkage files required to bind source-declared `primitive` methods on ordinary Universe classes.

## 4.6 Tests and conventions

Read:

```text
phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md

phalcom-semantic/tests/semantic/capabilities/time.rs
phalcom-core/tests/fixtures/language/time/**
```

Locate, rather than assume stale paths for:

```text
reflection/core-surface native-provenance tests
module/import/export tests
visibility tests
data declaration tests
abstract class/member tests
concurrency Duration/sleep/timeout tests
primitive inventory/floor tests
```

Avoid broad repository exploration unless a named premise fails.

---

# 5. Normative authority

## 5.1 Authority order

When sources disagree:

```text
1. explicit user-ratified C2 decisions / approved checkpoint amendments
2. accepted current language decisions and effective specs
3. STDL002.C2-CHECKPOINT.md durable invariants
4. STDL002.C2-GUIDANCE.md persistent guidance
5. this P1 shared header
6. completed C2 walkthrough/handoff facts
7. active P1 body for task mechanics
8. C1 predecessor records for unsuperseded behavior
9. current implementation/tests as repository reality
10. older drafts/proposals/historical conventions
```

Current code can prove that a feature is not yet implemented. It cannot, by itself, override accepted target semantics.

A stale test can be migrated when it encodes C1 behavior that C2 explicitly supersedes.

## 5.2 Ratified P1 behavior

The following are plan inputs, not implementer choices:

```text
time is the public temporal namespace

System is not the public date/time discovery namespace

Duration and Timestamp are the only temporal prelude promotions

SystemClock and SystemWallClock are public/exported concrete classes

clock and wallClock are package-exported canonical singleton values

Clock and WallClock are abstract capability classes

Instant is opaque and clock-domain-bound

Duration, Timestamp and civil value products use data where their complete
semantic state is legitimately transparent

visibility is canonical-identity + lexical-relation based

protected is inheritance-oriented, not a linear visibility level

grouped imports/exports use tuple-shaped organization

@intrinsic and @primitive are metadata only

native and primitive are implementation kinds, not visibility

primitive means irreducible

ordinary derivable temporal semantics remain source-level

no temporal VM specialization is part of P1
```

## 5.3 C1 behavior retained unless superseded

P1 preserves:

```text
signed Duration
Int nanosecond basis
Clock object identity as Instant domain identity
cross-domain Instant equality false
cross-domain Instant order/difference rejected
Instant.elapsed consults its originating Clock
System.sleep(Duration)
Future.timeout(Duration)
reactor host monotonic deadline efficiency
no public raw Instant tick constructor
```

P1 explicitly supersedes:

```text
System.clock as public discovery API
Clock itself as concrete host-clock implementation
Clock / Instant prelude promotion
System ownership of the monotonic host-read primitive
legacy implementation-selector/access conventions inside time
Duration's ordinary opaque class representation
```

## 5.4 Current C1 specification status

`docs/spec/current/stdlib/time.md` is authoritative for the landed C1 surface until C2 changes are approved/implemented, but it is deliberately incomplete for C2.

The P1 body must include the appropriate specification update so that P1 completion does not leave the effective time specification asserting superseded public APIs.

P2 will later complete the comprehensive timezone/zoned/textual sections.

---

# 6. Takeover state

## 6.1 Stable predecessor interface map

| Concept | Current owner | P1 treatment | Invariant |
|---|---|---|---|
| `Duration` | `time/clock.ph` class | migrate to `data`; preserve exact semantics | signed exact nanoseconds |
| `Clock` | `time/clock.ph` concrete class | migrate to abstract capability | object identity defines Instant domain |
| system clock singleton | current `Clock` class-side state | move to `SystemClock.instance` | exactly one canonical host domain |
| public clock access | current `System.clock` | replace with exported `time::clock` | package value === singleton |
| `Instant` | `time/clock.ph` opaque class | preserve opaque domain-bound model; replace legacy access seam | arbitrary user raw construction forbidden |
| monotonic host read | current `System` primitive | move to `SystemClock` primitive method | irreducible host observation |
| `System.sleep(Duration)` | concurrency/System | preserve | concurrency owns sleep semantics |
| `Future.timeout(Duration)` | concurrency | preserve | time owns Duration; concurrency owns timeout behavior |
| reactor deadline | Rust host monotonic internals | preserve | public Instant is not timer-queue representation |
| time package | Universe `time` | expand/restructure | one public package namespace |
| temporal prelude | Clock/Instant/Duration currently promoted | migrate to Duration/Timestamp | canonical declaration identity, no wrappers |

## 6.2 New P1 interfaces that P2 will inherit

P1 must leave stable:

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

P2 must not need to replace these types to implement timezones.

## 6.3 Migration, not parallel compatibility architecture

P1 should change the canonical surface rather than add permanent dual APIs.

Do not finish P1 with both:

```text
System.clock
and
time::clock
```

as co-equal public discovery surfaces merely to avoid fixture migration.

Short-lived intermediate compatibility during a coherent edit may exist if mechanically necessary, but no compatibility alias may survive P1 acceptance unless explicitly approved.

---

# 7. Architecture

## 7.1 Package topology

Target conceptual source organization:

```text
phalcom-core/core/universe/src/time/
│
├── package.ph
├── duration.ph
├── clock.ph
├── timestamp.ph
│
└── calendar/
    ├── package.ph
    ├── month.ph
    ├── period.ph
    ├── date.ph
    ├── time.ph
    ├── datetime.ph
    └── partial.ph
```

`UtcOffset` may live in a P1 `offset.ph` or under a future `zone/` subtree if that gives P2 the cleaner dependency direction.

The file split is mechanically flexible.

The dependency direction is fixed:

```text
Duration
   ↓
Clock / Instant
   ↓
Timestamp / WallClock
   ↓
Gregorian arithmetic primitives
   ↓
Month / Weekday / Period
   ↓
Date / Time / DateTime / YearMonth / MonthDay
   ↓
UtcOffset
```

No P1 source may depend on named timezone rules.

## 7.2 Representation taxonomy

### `data`

P1 uses `data` when:

```text
the value is immutable
the complete semantic state is transparent
structurally equal state means the same value
allocation identity is not semantic
```

Target:

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
```

Do not add hidden semantic storage to these values.

Do not store redundant derived components.

Examples:

```text
Date:
    store year, month, day
    derive weekday/dayOfYear/daysInMonth

DateTime:
    store Date + Time
    do not duplicate year/month/day/hour/... state

UtcOffset:
    store signed seconds
    do not store hours/minutes/seconds separately
```

### `enum`

Target:

```text
Month
Weekday
DateOverflow
```

### abstract `class`

Target:

```text
Clock
WallClock
```

### concrete/opaque `class`

Target:

```text
SystemClock
SystemWallClock
Instant
```

`Instant` stays opaque because its raw clock + tick coordinate is capability-controlled state, not a public interchange product.

## 7.3 `Duration`

Canonical semantic state:

```text
nanoseconds: Int
```

Required behavior:

```text
zero
nanoseconds / microseconds / milliseconds / seconds / minutes / hours factories
nanoseconds exact projection
microseconds / milliseconds / seconds numeric projection
+ Duration
- Duration
* Int
negated
abs
compare / relational behavior
isZero
isNegative
value equality/hash
```

P1 retains integer unit construction. Do not invent fractional rounding semantics.

Do not add `Duration.days` or `Duration.weeks`; exact twenty-four-hour time is:

```phalcom
Duration.hours(24)
```

Calendar-relative days belong to `Period`.

Final machine parse/format grammar is P2. P1 may retain a minimal stable `toString` necessary for current contracts but must not accidentally ratify a conflicting final text grammar.

## 7.4 `Measurement<T>`

Canonical transparent product:

```text
Measurement<T>
    value: T
    duration: Duration
```

`Clock.measure` evaluates the action once, preserves the returned value, and measures elapsed time using the same originating clock domain.

No custom runtime representation.

## 7.5 `Clock`

Semantic model:

```text
Clock object identity
    ==
monotonic coordinate domain identity
```

Target conceptual interface:

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

  // intended visibility: protected
  instant(_ ticks: Int) -> Instant {
    ...
  }
}
```

Exact visibility/abstract syntax must match live language authority.

`Clock` has no public constructor that creates another host monotonic domain.

Subclass-defined clocks remain legitimate.

## 7.6 `Instant` construction authority

P1 must replace the old hidden-selector bridge with ordinary lexical access.

Preferred semantic relationship:

```text
Instant raw constructor:
    package-visible to time package implementation

Clock.instant(ticks):
    protected
    creates Instant(clock: self, ticks: ticks)

ordinary user:
    cannot call raw Instant constructor

Clock subclass:
    can call protected Clock.instant
```

This uses the intended visibility model directly:

- package visibility authorizes the cooperating `time` declarations;
- protected visibility authorizes subclasses;
- no spelling-based privilege is required.

If the live constructor model cannot express this relationship, that is a P1 STOP/CONSULT condition rather than justification for a special selector convention.

`Instant` methods may directly inspect private state of another `Instant` through the lexical declaring-class rule; do not expose raw tick/clock getters solely for implementation convenience.

## 7.7 `SystemClock`

`SystemClock` is public and exported.

Semantic shape:

```text
SystemClock is Clock

private singleton storage
private construction path
public class-side instance -> SystemClock

now:
    read host monotonic nanoseconds
    mint Instant through protected Clock seam
```

The host observation is a `primitive` implementation because it cannot be derived from Phalcom values.

Conceptually:

```phalcom
class SystemClock is Clock {
  @class
  instance -> SystemClock {
    ...
  }

  now -> Instant {
    self.instant(self.monotonicNanoseconds)
  }

  // intended visibility: private
  primitive monotonicNanoseconds -> Int
}
```

The exact modifier ordering/syntax is verify-first.

Do not mark the entire class native.

Do not move ordinary singleton behavior into Rust.

## 7.8 Exported `clock`

The `time` package exports:

```text
clock: Clock
```

bound to:

```text
SystemClock.instance
```

Required identity:

```text
clock === SystemClock.instance
```

Repeated imports in distinct user modules must observe the same exported object.

The package binding may be initialized eagerly. Creating the singleton object must not itself read the host clock.

## 7.9 `Timestamp`

Canonical state:

```text
unixNanoseconds: Int
```

Timeline:

```text
Unix/POSIX-compatible
epoch = 1970-01-01T00:00:00Z
signed nanoseconds
no leap-second encoding
```

Required P1 behavior:

```text
epoch
now

unixNanoseconds
unixMicroseconds
unixMilliseconds
unixSeconds factories/projections

plus(Duration)
minus(Duration)
since(Timestamp) -> Duration
compare / relational behavior
value equality/hash
```

`Timestamp` uses arbitrary-precision `Int` semantics. Do not store floating-point seconds as canonical state.

A timestamp one nanosecond before epoch must split as:

```text
1969-12-31T23:59:59.999999999Z
```

wherever P1 conversion helpers exercise that boundary.

Final public timestamp text parsing/formatting belongs to P2.

## 7.10 `WallClock`

Target:

```phalcom
@abstract
class WallClock {
  @abstract
  now -> Timestamp
}
```

It is a replaceable capability rather than part of `Timestamp`'s structural value.

## 7.11 `SystemWallClock`

Public/exported concrete host implementation:

```text
SystemWallClock is WallClock

private singleton storage
private construction path
public class-side instance -> SystemWallClock

now:
    obtain irreducible host Unix/POSIX time
    construct Timestamp
```

Host wall-clock observation is `primitive`.

Ordinary timestamp construction/arithmetic remains source-level.

## 7.12 Exported `wallClock` and `Timestamp.now`

The package exports:

```text
wallClock: WallClock
```

bound to the exact `SystemWallClock.instance`.

Required identity:

```text
wallClock === SystemWallClock.instance
```

`Timestamp.now` is the zero-import convenience exposed by prelude `Timestamp`.

Its semantic source is the canonical wall-clock capability.

Do not make `Timestamp.now` itself an unrelated second host-read path.

## 7.13 `Month`

Closed enum:

```text
January ... December
```

ISO/Gregorian numbering:

```text
January = 1
...
December = 12
```

Expected behavior includes:

```text
number
fromNumber
days(inYear:)
```

Invalid month numbers reject; they do not wrap.

## 7.14 `Weekday`

Closed enum:

```text
Monday ... Sunday
```

ISO numbering:

```text
Monday = 1
...
Sunday = 7
```

`Date.weekday` derives this value; it is not stored redundantly in Date.

## 7.15 Gregorian arithmetic core

Use pure source-level constant-time civil/epoch-day algorithms.

Canonical conceptual helpers:

```text
isLeapYear(year)
daysInMonth(year, month)
daysFromCivil(year, month, day) -> Int
civilFromDays(epochDay) -> (year, month, day)
```

The exact private/package helper decomposition is flexible.

Requirements:

```text
proleptic Gregorian
correct negative/pre-epoch arithmetic
Euclidean/floor division where mathematically required
no host date library for ordinary calendar arithmetic
round-trip properties
```

Do not arbitrarily restrict civil arithmetic to four-digit years merely because common ISO examples do.

If a public supported year range must be restricted, the P1 body must ground that decision in accepted language/backend requirements. A material arbitrary range restriction is a STOP/CONSULT condition.

## 7.16 `Date`

Canonical transparent state:

```text
year: Int
month: Month
day: Int
```

Validity:

```text
day >= 1
day <= daysInMonth(year, month)
```

Derived behavior:

```text
weekday
dayOfYear
daysInMonth
isLeapYear
daysSince
compare
at(Time) -> DateTime
advance(Period, overflow:)
```

Invalid states must not be publicly constructible.

The mechanism for validated data construction is VERIFY-FIRST because `data` construction/visibility work is evolving.

Do not solve validation by changing `Date` to an opaque class merely to avoid a language gap.

Do not solve validation with a time-specific compiler special case.

## 7.17 `Time`

Canonical transparent state:

```text
hour: Int
minute: Int
second: Int
nanosecond: Int
```

Valid ranges:

```text
hour        0...23
minute      0...59
second      0...59
nanosecond  0...999_999_999
```

No leap-second value.

Expected P1 behavior:

```text
midnight
noon
compare
component access
```

Final parsing/formatting is P2.

There is no `Time.now`.

## 7.18 `DateTime`

Canonical state:

```text
date: Date
time: Time
```

`DateTime` is a timezone-free civil coordinate.

Expected behavior:

```text
component projections
compare
advance(Period, overflow:)
```

No implicit system timezone.

No general `Duration` arithmetic.

No `DateTime.now`.

Timezone resolution is P2.

## 7.19 `Period`

Canonical transparent state:

```text
months: Int
days: Int
```

Normalization:

```text
years -> months
weeks -> days
```

Factories:

```text
zero
years
months
weeks
days
```

Arithmetic:

```text
+ Period
- Period
* Int
negated
isZero
```

No total ordering.

The question:

```text
1 month < 31 days
```

has no anchor-independent answer and must not be encoded as a general comparison.

## 7.20 `DateOverflow`

Closed policy enum:

```text
Constrain
Reject
```

Application order for `Date.advance(Period)`:

```text
1. apply month component
2. resolve target day under DateOverflow
3. apply day component via epoch-day arithmetic
```

Example:

```text
2025-01-31 + 1 month, Constrain
    -> 2025-02-28

2025-01-31 + 1 month, Reject
    -> error
```

This ordering is fixed.

## 7.21 `YearMonth`

Canonical state:

```text
year: Int
month: Month
```

Expected behavior:

```text
days
isLeapYear
plusMonths
minusMonths
atDay -> Date
```

Do not accept a day-bearing `Period` directly as if the value had a day coordinate.

## 7.22 `MonthDay`

Canonical state:

```text
month: Month
day: Int
```

Must reject structurally impossible month/day combinations such as April 31.

February 29 is representable because it is valid in some years.

Expected:

```text
inYear(year) -> Date
```

which rejects a non-leap target year when appropriate.

## 7.23 `UtcOffset`

Canonical state:

```text
seconds: Int
```

It is a fixed displacement value, not a timezone rule set.

Expected behavior:

```text
utc
seconds(...)
hours(...)
negated
compare
value equality/hash
```

Historical second-resolution offsets must remain representable.

The precise accepted numeric bound must be verified against intended P2 interoperability rather than guessed from current host APIs. P1 must document the selected stable bound.

`UtcOffset` must never silently acquire named-zone/DST semantics.

## 7.24 Package/prelude architecture

Final P1 `time` root exports:

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

Use grouped tuple-shaped imports/exports when publishing multiple names.

Universe/prelude promotion:

```text
Duration
Timestamp
```

Only.

Prelude exposure is a canonical re-export/reference to the same declarations, not a duplicate type or wrapper.

## 7.25 `System` boundary

P1 removes temporal discovery:

```text
System.clock
```

and does not add:

```text
System.now
System.wallClock
System.timeZone
```

P1 preserves active runtime/concurrency operations such as:

```text
System.sleep(Duration)
```

because sleep is a scheduler operation that consumes time, not temporal discovery.

P1 must not opportunistically migrate every other legacy `System` native/internal operation. Only time-owned dependencies are in scope.

## 7.26 Source/native/primitive ownership

Decision test:

```text
Can the result be derived correctly in Phalcom from already-available values?
```

If yes:

```text
source by default
```

If no because it observes an external host/runtime fact:

```text
primitive may be appropriate
```

P1 expected primitives:

```text
SystemClock host monotonic observation
SystemWallClock host wall-clock observation
```

P1 expected native methods:

```text
none required by temporal semantics
```

If the live runtime requires a native administrative adapter around primitive linkage, keep that machinery below the language-level ownership and do not relabel source semantics as native.

## 7.27 Metadata-only attributes

`@intrinsic` and `@primitive` may carry Universe-authored metadata if the live canonical language surface requires or permits them.

Their presence must not be relied on to:

```text
bind Rust code
select primitive dispatch
change visibility
grant package/internal access
change selector identity
change representation
force intrinsic lowering
alter typechecking
```

If P1 would only work by making these attributes operative, STOP AND CONSULT.

## 7.28 No temporal runtime specialization

Forbidden P1 architecture:

```text
Value::Duration
Value::Timestamp
Value::Date
Value::Instant
Value::UtcOffset

temporal heap arms

MAKE_DATE
ADD_TIMESTAMP
ADVANCE_PERIOD
READ_CLOCK
other temporal bytecodes

compiler checks for declaration names such as "Date" or "Duration"

VM date/calendar arithmetic
```

`data` may of course use the already-ratified general data representation/optimizer.

---

# 8. Ownership boundaries

## 8.1 P1 owns

P1 owns:

- the C1-to-C2 monotonic API migration;
- `Duration` data migration;
- `Measurement<T>`;
- `Clock` abstract behavior;
- `SystemClock` concrete implementation and singleton;
- exported `clock`;
- `Instant` access/construction migration while preserving domain laws;
- monotonic primitive holder migration;
- `Timestamp`;
- `WallClock`;
- `SystemWallClock`;
- exported `wallClock`;
- wall-clock primitive;
- Gregorian arithmetic;
- `Month`;
- `Weekday`;
- `Date`;
- `Time`;
- `DateTime`;
- `YearMonth`;
- `MonthDay`;
- `Period`;
- `DateOverflow`;
- `UtcOffset`;
- `time` root module restructuring;
- `Duration` / `Timestamp` prelude mapping;
- removal of `System.clock`;
- compatibility of existing sleep/timeout/reactor consumers;
- P1-level semantic/runtime/reflection/ordinary-package tests;
- P1 time-spec update;
- minimal general abstract-member closure only if required and architecturally compatible;
- consumption of the new visibility and implementation-kind model.

## 8.2 P1 does not own

P1 does not own:

- IANA/TZif database integration;
- Rust timezone library selection/installation;
- `TimeZone`;
- `ZoneTransition`;
- `LocalTimeResolution`;
- `LocalTimeDisambiguation`;
- `ZonedDateTime`;
- `TimeZone.system`;
- `Date.today`;
- `ZonedDateTime.now`;
- local civil ambiguity/gap resolution;
- named-zone transition search;
- final ISO/RFC parsing/formatting grammar;
- localized formatting;
- `time.testing` final package unless a minimal test-local clock fixture is needed for P1 tests;
- timers/reactor redesign;
- whole-System migration;
- whole-Universe visibility migration;
- whole-repository native/primitive migration;
- general FFI redesign;
- temporal optimizer specialization;
- P2 certification.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| C2 lifecycle state | `STDL002.C2-CHECKPOINT.md` | plans/guidance/handoffs | plan-local competing state |
| temporal P1 public semantics | accepted C2 decisions + current time spec after P1 update | semantic/compiler/runtime/tests | Rust-only shadow contract |
| declaration/module/package/project identity | module/semantic infrastructure | visibility checks | path/string-name inference |
| member visibility relation | canonical semantic access-control product | compiler/runtime/reflection | selector-prefix checks |
| method implementation kind | canonical declaration/native infrastructure | linker/runtime/reflection | attribute-name inference |
| `@intrinsic` / `@primitive` metadata | declaration metadata | tooling/reflection/optimizer where separately authorized | implementation selection |
| `Duration` semantic state | `Duration.nanoseconds` data component | all time/concurrency code | hidden parallel native state |
| `Clock` domain identity | `Clock` object identity | `Instant` equality/order/difference | global tick-only identity |
| canonical system monotonic instance | `SystemClock` class-side singleton state | `time::clock`, production elapsed-time code | second package singleton |
| monotonic host observation | `SystemClock` primitive | `SystemClock.now` | `System` temporal host getter |
| `Instant` raw state authority | package-controlled constructor + private instance state | `Clock`, `Instant` methods | public raw tick API |
| `Timestamp` state | signed Unix nanoseconds | wall clock, civil conversion, P2 | host datetime object |
| canonical wall-clock instance | `SystemWallClock` singleton | `wallClock`, `Timestamp.now` | second host read singleton |
| wall host observation | `SystemWallClock` primitive | `SystemWallClock.now` | separate `Timestamp.now` primitive |
| Gregorian validity | one source-level calendar helper authority | constructors/arithmetic/tests | parser-specific or Date-specific duplicate rules |
| civil/epoch conversion | one source-level calendar algorithm authority | Date/Timestamp/future P2 | host date-library semantics |
| `Period` application order | Date/DateTime source behavior | P1/P2 calendar movement | timezone backend policy |
| prelude temporal mapping | Universe root package | semantic resolution | wrapper/alias types |
| sleep/timeout behavior | concurrency subsystem | users of Duration | time package scheduler behavior |
| reactor deadline representation | concurrency/reactor runtime | timers/futures | public Instant storage requirement |

---

# 9. Global invariants

## P1-INV-01 — C1 history remains truthful

C1 remains the completed monotonic predecessor. P1 records migrations as C2 work; it does not rewrite C1's historical completion record.

## P1-INV-02 — `Duration` remains exact signed elapsed time

Canonical state is signed `Int` nanoseconds. P1 does not conflate exact duration with calendar period.

## P1-INV-03 — `Duration` is transparent value data

Its complete semantic state is the nanosecond component. No hidden allocation identity or native backing becomes observable.

## P1-INV-04 — clock identity defines monotonic domain identity

Two different `Clock` objects define different `Instant` domains even if their numeric ticks happen to match.

## P1-INV-05 — system monotonic clock identity is singular

`time::clock` and `SystemClock.instance` are the exact same object for the relevant VM/package lifetime.

## P1-INV-06 — `SystemClock` is ordinary public library surface

It is exported and reflectable as an ordinary class. Ordinary users usually consume `clock`, but direct reference to `SystemClock` is not forbidden.

## P1-INV-07 — monotonic host observation is irreducible and owned by `SystemClock`

The host monotonic read is a `primitive` implementation on the concrete clock abstraction, not a public `System` time getter.

## P1-INV-08 — arbitrary user code cannot forge raw `Instant`s

Only cooperating time-package implementation code plus eligible Clock subclasses may mint domain-bound raw coordinates through the intended lexical access relationship.

## P1-INV-09 — cross-domain Instant laws remain strict

Cross-domain equality is false. Cross-domain ordering/difference rejects rather than comparing raw ticks.

## P1-INV-10 — `Timestamp` is signed Unix/POSIX nanoseconds

Canonical state is arbitrary-precision `Int` nanoseconds from Unix epoch. No floating-point or host datetime object is canonical state.

## P1-INV-11 — wall-clock capability identity is distinct from timestamp value identity

`SystemWallClock.instance`/`wallClock` is a capability singleton; `Timestamp` values are globally meaningful values and do not inherit wall-clock-domain identity.

## P1-INV-12 — `Timestamp.now` has one canonical host source

It observes the canonical `wallClock`; it does not create a second independent host-read pathway.

## P1-INV-13 — civil calendar is proleptic Gregorian

Leap-year, month-length, epoch-day, weekday, and calendar arithmetic agree on one source-level Gregorian model.

## P1-INV-14 — leap seconds are not representable in P1 civil time

`Time.second` is `0...59`; Timestamp uses the selected POSIX/Unix-compatible timeline.

## P1-INV-15 — civil values cannot publish invalid states

`Date`, `Time`, `MonthDay`, and `UtcOffset` must enforce their declared invariants through a general language-supported construction mechanism.

## P1-INV-16 — `Date` stores no derived calendar state

`weekday`, `dayOfYear`, and month length are derived.

## P1-INV-17 — `DateTime` is timezone-free

It contains only civil Date + Time meaning. It has no hidden offset, zone, or instant.

## P1-INV-18 — `Period` remains calendar-relative

Canonical state is months + days. Years normalize to months; weeks normalize to days. Period has no total ordering.

## P1-INV-19 — month arithmetic order is fixed

Apply months, resolve invalid target day using `DateOverflow`, then apply days.

## P1-INV-20 — `UtcOffset` is not `TimeZone`

It is a fixed signed-seconds displacement only; no DST/history/rule-set behavior is smuggled into P1.

## P1-INV-21 — access control is lexical and identity-based

Visibility checks depend on canonical declaring class/module/package/project identities and inheritance eligibility, never spelling or path conventions.

## P1-INV-22 — protected access is inheritance-oriented

Do not implement protected as merely "more visible than private" in a linear lattice.

## P1-INV-23 — no special implementation-selector naming survives P1

P1 time source uses ordinary names and the actual visibility system; naming conventions do not create privilege.

## P1-INV-24 — metadata attributes are non-operative by presence

`@intrinsic` and `@primitive` do not install methods, grant visibility, select representations, or force lowering.

## P1-INV-25 — implementation kind and metadata are separate axes

`native` / `primitive` implementation semantics are not inferred from metadata attributes or selector spelling.

## P1-INV-26 — primitive means irreducible

Only host facts unavailable from existing Phalcom values qualify. Performance or implementation convenience alone is insufficient.

## P1-INV-27 — derivable temporal meaning remains source-level

Calendar arithmetic, Duration/Timestamp arithmetic, fixed-offset arithmetic, validation, equality, hashing, and Period policy are ordinary Phalcom behavior.

## P1-INV-28 — no P1 temporal VM specialization

No temporal Value arm, temporal bytecode, temporal object kind, or compiler-name special case is introduced.

## P1-INV-29 — temporal prelude is exactly `Duration` + `Timestamp`

Clock/Instant/system-clock/civil/fixed-offset declarations require normal `time` imports.

## P1-INV-30 — prelude identity is canonical identity

Prelude promotion does not create wrapper or alias types with different declaration identity.

## P1-INV-31 — `System` is not temporal discovery

P1 removes public `System.clock` and does not add other date/time discovery methods. Scheduler/process actions may still consume temporal values.

## P1-INV-32 — time import remains cheap

Constructing canonical clock capability objects is allowed. P1 introduces no expensive external database initialization.

## P1-INV-33 — grouped import/export is the canonical multi-name form

New/migrated P1 package source and user-facing fixtures use tuple-shaped grouping when importing/exporting multiple names.

## P1-INV-34 — ordinary package use must work

Public time behavior must not depend on accidental all-Universe visibility or bootstrap-only name access.

## P1-INV-35 — concurrency remains behaviorally compatible

Sleep, timeout, and reactor deadline behavior continue to consume the canonical Duration semantics without requiring public Clock/Instant prelude exposure.

## P1-INV-36 — P1 stops before named timezone semantics

No named-zone database, DST transition, ambiguous-local-time, or ZonedDateTime contract becomes P1 architecture.

---

# 10. Non-goals

P1 deliberately excludes:

```text
IANA / TZif rule database
Jiff or another timezone dependency
TimeZone
ZoneTransition
LocalTimeResolution
LocalTimeDisambiguation
ZonedDateTime
TimeZone.system
Date.today
ZonedDateTime.now
named-zone offset lookup
DST gap/overlap behavior
transitionBefore / transitionAfter
final ISO/RFC parse grammar
final canonical temporal toString grammar
locale/CLDR formatting
natural-language time
recurrence / cron / RRULE
business-day / holiday calendars
alarms / reminders
timer APIs or reactor redesign
thread model changes
generic temporal traits
whole-System cleanup
whole-Universe visibility migration
whole-repository native/primitive migration
general native/FFI redesign
performance-specific temporal bytecodes
specialized temporal heap representations
P2 certification
```

Do not pull P2 work into P1 just because a future API is easy to sketch.

P1 may create package-private helpers deliberately designed for P2 consumption only when they belong naturally to the P1 value/calendar foundation and do not force P2 semantic decisions early.

---

# 11. Expected impact map

This section describes likely ownership areas, not a frozen edit list. The P1 body will name exact task-level files after a final live re-grounding.

## 11.1 Expected Universe source

Likely modify:

```text
phalcom-core/core/universe/src/time/package.ph
phalcom-core/core/universe/src/time/clock.ph
phalcom-core/core/universe/src/package.ph
phalcom-core/core/universe/src/concurrency/fiber.ph
```

Likely create/split under:

```text
phalcom-core/core/universe/src/time/duration.ph
phalcom-core/core/universe/src/time/timestamp.ph
phalcom-core/core/universe/src/time/calendar/**
```

The body may retain fewer files if the live package conventions strongly favor a flatter layout, but no single file should become the accidental owner of unrelated clock + calendar + absolute-time behavior.

## 11.2 Expected primitive/linkage source

Current baseline likely requires migration away from the System-owned monotonic primitive in:

```text
phalcom-core/src/primitive/system.rs
phalcom-core/src/universe/primitives.rs
```

The target may require a new time-specific primitive registration/source area or the new declaration-driven implementation-kind infrastructure that exists at execution HEAD.

**Fixed:** runtime holder identity becomes `SystemClock`/`SystemWallClock` as designed.

**Flexible:** exact Rust file/module/registry location.

Do not preserve System holder identity simply because the old registry is convenient.

## 11.3 Expected semantic/module/prelude source

Likely affected:

```text
Universe root/package source ingestion
prelude export publication
source-contract/capability tests
module import/export tests
visibility resolution tests
reflection/core-surface publication
```

Do not add time-specific semantic resolution branches.

## 11.4 Expected tests

Likely extend/migrate:

```text
phalcom-semantic/tests/semantic/capabilities/time.rs
phalcom-core/tests/fixtures/language/time/**
```

Likely add focused tests for:

```text
grouped time imports
prelude Duration/Timestamp identity
Clock/SystemClock/clock surface
Instant domain behavior
wall clock/Timestamp
Gregorian arithmetic
validated civil values
Period overflow
UtcOffset
ordinary-package access
reflection/provenance
concurrency compatibility
```

Use existing test-placement conventions.

## 11.5 Expected documentation/state

P1 must update/create:

```text
STDL002.C2-CHECKPOINT.md
STDL002.C2.P1-walkthrough.md
STDL002.C2.P1-handoff.md
docs/spec/current/stdlib/time.md
```

Update other live docs that state `System.clock` or Clock/Instant prelude behavior when those statements are part of the effective current surface, but do not launch a general documentation cleanup.

## 11.6 Potential prerequisite language areas

Only if live HEAD still lacks required general semantics:

```text
abstract member obligation/completeness
validated data construction or constructor access
current visibility syntax/plumbing
current primitive declaration/linkage
```

The body must distinguish:

```text
P1-authorized prerequisite closure
vs
active external language-transition dependency
```

and must not duplicate an in-flight general mechanism.

## 11.7 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem when doing so changes architecture, semantic authority, identity, representation, access-control design, or runtime ownership.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

1. the P1/P2 boundary;
2. the public temporal taxonomy in §7;
3. `Duration` as signed nanosecond `data`;
4. `Measurement<T>` as generic data;
5. `Clock` and `WallClock` as abstract classes;
6. `SystemClock` and `SystemWallClock` as public/exported concrete classes;
7. package-exported canonical `clock` / `wallClock` values;
8. Clock object identity as Instant domain identity;
9. raw Instant construction being unavailable to ordinary users;
10. `Timestamp` signed Unix/POSIX nanosecond state;
11. proleptic Gregorian civil semantics;
12. no leap-second civil value;
13. `Period` canonical months + days model;
14. month-then-overflow-then-day Period application order;
15. `UtcOffset` fixed signed-seconds meaning;
16. prelude policy: Duration + Timestamp only;
17. no temporal discovery façade on `System`;
18. lexical/canonical identity-based visibility;
19. non-linear inheritance meaning of protected visibility;
20. metadata-only semantics of `@intrinsic` / `@primitive`;
21. `native` versus `primitive` implementation-kind distinction;
22. irreducibility test for primitive;
23. source ownership of derivable temporal semantics;
24. absence of temporal VM special cases;
25. ordinary-package accessibility requirement;
26. grouped multi-import/export form;
27. existing concurrency behavior as an integration contract.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact private/helper names;
- exact file split under `time`;
- whether related tiny civil helpers share one module or several;
- local iterator/loop style in pure Phalcom calendar algorithms;
- exact Rust source-file location for host-clock functions;
- exact registration helper names under the new implementation-kind infrastructure;
- test fixture filenames;
- private cache/singleton field names;
- exact error-message wording where no normative diagnostic wording exists;
- exact non-public helper signatures;
- use of an existing generic validation helper versus a new reusable helper when semantics are identical;
- mechanical import changes caused by live repository drift;
- local documentation cross-linking.

Mechanical flexibility does **not** permit changing public signatures/semantics merely to avoid implementing a prerequisite.

## 12.3 VERIFY-FIRST

The body must verify these before task execution relies on them:

| Assumption | Where to verify | If false |
|---|---|---|
| grouped tuple-shaped import/export syntax has landed | parser/AST/module tests + effective spec | if only mechanical syntax drift, adapt; if semantic model absent, STOP AND CONSULT |
| `private/protected/module/package/internal` lexical visibility has landed | effective spec + semantic access-control implementation/tests | do not emulate with naming; STOP if required relation absent |
| protected eligibility is inheritance-based | semantic visibility authority/tests | STOP if implementation models it as simple linear visibility |
| `native` method implementation kind has landed | parser/AST/native declaration infrastructure | do not fall back to old decorator semantics as final P1 architecture |
| `primitive` method implementation kind has landed | parser/AST/linkage/runtime infrastructure | STOP if host-clock migration would require inventing a parallel binding system |
| `@intrinsic` / `@primitive` metadata surface has landed | AST/semantic metadata/spec | P1 need not use absent metadata; never make absence block core behavior unless current authority requires annotation |
| Universe-only authored metadata legality is enforced by project identity | semantic/project checks | if absent but P1 uses metadata, STOP rather than hand-rolling time-specific validation |
| bodyless abstract member obligations are implemented | parser/semantic/class finalization tests | if absent, use body plan's explicitly authorized general closure path; consult if architecture differs materially |
| concrete subclasses with unresolved abstract members are rejected | semantic/class finalization | same as above |
| runtime abstract construction is guarded | class/runtime allocation | preserve existing guard; do not duplicate |
| `data` behavior via current impl/inherent mechanism is available | current LANG005 source/spec/tests | adapt mechanics |
| generic `data` supports `Measurement<T>` with exact type identity | LANG005 data/generic tests | STOP if generic data contract is incomplete |
| data construction can enforce invariants without abandoning transparency | data constructor/access/validation authority | STOP AND CONSULT if no general mechanism exists |
| data equality/hash semantics are available or derivable canonically | data/product specs/runtime | use canonical mechanism; do not invent time-specific identity |
| ordinary exported values have stable shared package binding identity | module runtime/init tests | STOP if each import would re-evaluate singleton construction |
| class-side state supports one singleton per VM/package lifetime | current class static field semantics | adapt mechanics; STOP if identity cannot be stable |
| arbitrary-precision `Int` arithmetic used by time is correct for negative floor/Euclidean needs | numeric spec + current operations | if required operation absent/wrong, plan focused numeric prerequisite or consult; do not silently use Float |
| current error taxonomy has suitable argument/value/range failure forms | errors source/spec | adapt to existing errors; dedicated temporal errors are not P1 default |
| `System.sleep` / `Future.timeout` consume Duration through stable public projections | concurrency source/tests | migrate call sites mechanically; do not redesign reactor |
| semantic reflection can expose data/class/enum declarations normally | SEMA/LANG reflection tests | fix general observation path, not time-specific reflection |
| current time spec may be updated incrementally in P1 | docs/spec authority convention | if spec update requires a separate decision record, follow that process rather than silently editing normative text |

---

# 13. Global STOP / CONSULT triggers

The implementer must stop editing the affected issue and prepare a consultation packet if any of the following occurs.

1. The ratified visibility relations cannot express the `Clock` → `Instant` construction capability without a spelling-based exception.
2. A package-visible raw `Instant` construction seam plus protected `Clock.instant` cannot be represented by the live language.
3. Access checks would need source-path/string comparison instead of canonical class/module/package/project identity.
4. Protected access would need to be treated as a linear visibility level rather than an inheritance relation.
5. P1 would need to revive a special implementation-selector naming convention.
6. `@intrinsic` or `@primitive` would need operative semantics to bind code, grant access, alter representation, or force lowering.
7. The live `native` / `primitive` implementation-kind architecture materially contradicts the C2 distinction.
8. The host clock can only be moved off `System` by introducing a second ad hoc primitive registry or binding mechanism.
9. Abstract member obligations require a materially different class-completeness architecture than the accepted model.
10. Concrete abstractness can only be enforced through a time-specific checker rule.
11. `data` construction cannot enforce Date/Time/MonthDay/UtcOffset invariants through a general language mechanism.
12. Preserving data transparency would require hidden semantic state or conversion to opaque classes.
13. Exact Timestamp/Duration arithmetic would require Float canonical state.
14. Correct Gregorian pre-epoch conversion would require a numeric semantic change whose ownership is nonlocal/uncertain.
15. Civil year range would need an arbitrary public restriction not justified by accepted semantics.
16. `UtcOffset` range must be chosen in a way that materially constrains P2 named-zone interoperability and two plausible choices have different public consequences.
17. `SystemClock.instance` and `time::clock` cannot maintain one stable object identity through normal package initialization.
18. `SystemWallClock.instance` and `time::wallClock` cannot maintain one stable capability identity.
19. P1 would need an implicit timezone to implement `Date`, `Time`, `DateTime`, or Timestamp arithmetic.
20. `DateTime` would need exact Duration arithmetic to satisfy an existing test rather than the accepted model.
21. `Period` would need a total order or exact-duration interpretation.
22. Existing concurrency tests can only pass by restoring public `System.clock`.
23. A temporal VM `Value` variant, temporal bytecode, or compiler type-name special case appears necessary for correctness.
24. A large source-level temporal operation is being moved to Rust only to bypass a compiler/semantic/runtime bug.
25. A required language prerequisite materially overlaps or conflicts with an active LANG checkpoint.
26. A current normative spec, accepted decision, user ruling, and test oracle materially conflict.
27. The same nontrivial semantic failure persists after one serious corrective attempt.
28. Two plausible fixes have meaningfully different ownership/identity/representation implications.
29. P1 implementation begins to require IANA/timezone/zoned semantics from P2.
30. An invariant in §9 would need to be weakened merely to pass existing tests.
31. Scope expands into timers/reactor redesign, generic FFI redesign, whole-System migration, or whole-Universe access-control migration.

A triggered consultation cannot be self-waived.

## 13.1 Consultation packet

Collect cheaply:

```text
P1 task/gate
relevant P1-INV-* IDs
expected architecture
live conflicting fact
minimal Phalcom reproducer
relevant source symbols/paths
scoped diff
tests/commands actually run
observed failure
causal hypothesis
one serious attempt already made, if applicable
exact trigger number
specific decision requested
```

Do not ask the adviser to rediscover the repository.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Examples:

```text
renamed helper
import path drift
new visibility syntax ordering
primitive-registration API rename
Rust type/borrow cleanup
test filter mismatch
fixture path drift
```

Allow up to three coherent correction cycles while evidence shows progress.

A cycle is:

```text
inspect
identify one concrete cause
make one coherent correction
rerun the smallest discriminating command
```

If the issue stops looking mechanical, reclassify it.

## 14.2 Semantic failures

Before editing:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Permit one serious correction for the same underlying semantic failure.

If substantially the same failure persists: STOP AND CONSULT.

Do not repeatedly patch around:

```text
wrong clock identity
invalid civil state leakage
negative-date arithmetic error
visibility bypass
incorrect data equality
prelude duplicate identity
primitive/native provenance mismatch
```

## 14.3 Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

## 14.4 No unchanged reruns

Never rerun an unchanged failing test unless something relevant to its failure changed.

A broad failing suite is not itself a causal hypothesis.

---

# 15. Testing surface analysis

The P1 body must map every task and gate to the applicable coverage IDs below.

## 15.1 Coverage ledger

| Coverage ID | Dimension | Invariant(s) | Required evidence |
|---|---|---|---|
| `P1-CV-01` | Duration canonical value | 02, 03 | equal nanoseconds => value equality/hash; signed/zero/negative behavior |
| `P1-CV-02` | Duration units | 02 | ns/us/ms/s/min/h integer factories and projections preserve exact intended values |
| `P1-CV-03` | Duration type rejection | 02 | fractional factory input and wrong arithmetic operands reject through normal typing/validation |
| `P1-CV-04` | Measurement generic data | 03, 27 | `Measurement<T>` preserves exact value type + measured Duration |
| `P1-CV-05` | system clock singleton | 04, 05, 32 | repeated `SystemClock.instance`, repeated `clock`, cross-module import identity |
| `P1-CV-06` | Clock abstractness | 04 | Clock cannot be concretely instantiated; concrete subclass must implement `now` |
| `P1-CV-07` | custom clock domain | 04, 08, 09 | deterministic subclass creates its own Instant domain through protected seam |
| `P1-CV-08` | cross-domain Instant equality | 04, 09 | same ticks/different clocks => false |
| `P1-CV-09` | cross-domain Instant order/difference | 04, 09 | compare/since reject different domains |
| `P1-CV-10` | Instant arithmetic | 02, 08 | plus/minus Duration retain origin Clock and exact tick delta |
| `P1-CV-11` | Instant elapsed | 04 | elapsed consults originating custom clock, not global clock |
| `P1-CV-12` | raw Instant access control | 08, 21, 23 | ordinary user cannot call raw constructor; eligible time/package/subclass code can through intended lanes |
| `P1-CV-13` | primitive monotonic ownership | 07, 24, 25, 26 | host observation belongs to SystemClock primitive; no public System clock bridge |
| `P1-CV-14` | Timestamp epoch/scalars | 10 | epoch, signed ns/us/ms/s factories/projections |
| `P1-CV-15` | Timestamp arithmetic | 10 | plus/minus/since exact Duration behavior |
| `P1-CV-16` | pre-epoch boundary | 10, 13 | -1ns conversion math yields preceding second + 999,999,999ns |
| `P1-CV-17` | wall clock singleton | 11, 12 | `wallClock === SystemWallClock.instance` and stable repeated imports |
| `P1-CV-18` | Timestamp.now source | 11, 12 | convenience delegates to canonical wall capability; no second host pathway |
| `P1-CV-19` | primitive wall ownership | 12, 24, 25, 26 | host wall observation is primitive on SystemWallClock |
| `P1-CV-20` | leap-year rules | 13 | 2000 leap; 1900 not; 2024 leap; representative negative/zero-year semantics per chosen proleptic model |
| `P1-CV-21` | Gregorian round trip | 13 | civil -> epoch day -> civil property across pre/post epoch and leap boundaries |
| `P1-CV-22` | weekday | 13, 16 | known dates and epoch-adjacent dates map correctly |
| `P1-CV-23` | Date validity | 15 | invalid day 0, month-day overflow, Feb 29 non-leap reject |
| `P1-CV-24` | Time validity | 14, 15 | hour/minute/second/ns boundaries; second 60 rejects |
| `P1-CV-25` | DateTime composition | 17 | exact Date + Time projections; no timezone state |
| `P1-CV-26` | Period normalization | 18 | years==12 months; weeks==7 days; arithmetic/negation |
| `P1-CV-27` | Period no ordering | 18 | no accidental total-order API/proof |
| `P1-CV-28` | DateOverflow Constrain | 19 | Jan 31 + 1 month => Feb last day |
| `P1-CV-29` | DateOverflow Reject | 19 | invalid target day rejects rather than clamps |
| `P1-CV-30` | Period application order | 19 | months/overflow/days ordering distinguished by a case where reordering changes result |
| `P1-CV-31` | YearMonth | 13, 15 | month arithmetic, day count, atDay validation |
| `P1-CV-32` | MonthDay | 15 | impossible combination rejects; Feb 29 exists; non-leap `inYear` rejects |
| `P1-CV-33` | UtcOffset state/precision | 20 | fixed signed seconds, negation/order, second precision, invalid bound rejection |
| `P1-CV-34` | UtcOffset non-zone semantics | 20, 36 | no named/DST/timezone-rule API in P1 |
| `P1-CV-35` | grouped imports | 33, 34 | ordinary package tuple-shaped import of multiple time names resolves correct identities |
| `P1-CV-36` | grouped exports | 33 | `time/package.ph` grouped publication resolves canonical declarations/values |
| `P1-CV-37` | prelude surface | 29, 30 | Duration/Timestamp work unqualified; Clock/Instant/Date/clock do not resolve unqualified |
| `P1-CV-38` | prelude identity | 30 | explicitly imported Duration/Timestamp and prelude names are same declaration identity |
| `P1-CV-39` | System façade removal | 31 | public `System.clock` no longer resolves; no new wall/date/time getters |
| `P1-CV-40` | visibility semantics | 21, 22, 23 | private/protected/module/package boundaries use canonical identity; negative ordinary-user access |
| `P1-CV-41` | metadata non-operation | 24, 25 | metadata presence/absence does not create implementation/access semantics |
| `P1-CV-42` | ordinary package integration | 34 | representative user package imports/constructs/uses P1 values normally |
| `P1-CV-43` | reflection | 03, 06, 10, 13, 30 | data/class/enum categories and canonical types visible through ordinary reflection |
| `P1-CV-44` | concurrency sleep | 02, 35 | `System.sleep(Duration)` still accepts Duration and rejects invalid negative wait per existing contract |
| `P1-CV-45` | Future timeout | 02, 35 | timeout still consumes migrated Duration without semantic drift |
| `P1-CV-46` | reactor boundary | 04, 35 | public clock migration does not force reactor timer storage to public Instant |
| `P1-CV-47` | no temporal VM special case | 28 | disassembly/runtime architecture evidence shows ordinary source/data dispatch; no new temporal opcode/value arm |
| `P1-CV-48` | cheap import | 32, 36 | importing time does not trigger timezone/native database initialization or other P2 resource work |
| `P1-CV-49` | P2 exclusion | 36 | no named TimeZone/ZonedDateTime declarations or timezone dependency introduced |
| `P1-CV-50` | spec/API agreement | 01, 29, 31 | updated time spec and source tests agree on P1 public surface |

## 15.2 Property-style coverage

Where current test infrastructure supports it economically, prefer property coverage for:

```text
civil <-> epoch-day round trip
Date day arithmetic around epoch
leap-year cycle boundaries
Duration arithmetic identities
Timestamp +/- Duration inverse
Period normalization identities
UtcOffset negation
```

Do not add a new property-testing framework solely for P1 if existing deterministic loops/fixtures can cover the same risk cheaply.

## 15.3 Negative-access tests

Access-control tests must prove the target relations, not merely that a specially named selector fails.

At minimum, design negative cases for:

```text
ordinary package -> package-visible Instant raw construction
non-subclass -> protected Clock minting seam
unrelated module/package -> module/package helper
ordinary project -> any Universe-only metadata authoring rule when applicable
unqualified use of non-prelude temporal names
```

## 15.4 Ordinary-package tests are mandatory

Universe self-compilation is insufficient.

At least one representative ordinary package must:

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

and use:

```text
Duration from prelude
Timestamp from prelude
clock from explicit import
civil value construction
Period arithmetic
UtcOffset
```

without relying on bootstrap-only visibility.

---

# 16. Verification execution budget

## 16.1 Modes

### BUILD MODE

Default mode.

Use only the smallest test/compiler command that answers the current question.

Continue through intentionally incomplete intermediate states when a broader test would only confirm known incompleteness.

### STABILIZE MODE

Enter at named body gates after coherent vertical slices.

Run focused feature suites and directly affected integration.

### CERTIFY MODE

P1 certification is still narrower than full C2/workspace certification.

Use broad crate/workspace commands only if the body names them at a final P1 gate or evidence requires them.

P2 remains responsible for final C2 timezone/textual closure and any broader checkpoint certification not explicitly consumed by P1.

## 16.2 Verification ladder

```text
T0 — exact regression / smallest new fixture
T1 — directly affected time or language-feature tests
T2 — owning semantic/core module suite
T3 — adjacent cross-layer integration (Universe/module/reflection/concurrency)
T4 — broad affected crate/package verification
T5 — workspace/release verification
```

Passing one level does not automatically authorize or require the next.

## 16.3 Mandatory during BUILD

The body should normally require only:

```text
targeted parser/semantic test when consuming new syntax/access rules
targeted data/abstract-member test when closing a prerequisite
targeted time source/runtime fixture for the active vertical
targeted Rust unit test only when host primitive/linkage is being edited
```

Do not run all semantic or all core tests after each task.

## 16.4 Mandatory at named P1 gates

Expected coherent gates will include focused evidence for:

```text
language prerequisite readiness
monotonic migration
absolute-time vertical
civil/calendar vertical
package/prelude/ordinary-package integration
concurrency/reflection stabilization
```

The body will supply exact commands.

## 16.5 Mandatory during final P1 stabilization

At minimum, run the live equivalents of:

```text
time semantic capability tests
time language fixture suite
directly affected data/abstract/visibility/import tests
directly affected reflection/core-surface tests
directly affected concurrency sleep/timeout tests
primitive/native provenance or inventory tests affected by moving clock host reads
```

Confirm filters select actual tests.

## 16.6 Only if evidence demands

Broaden to:

```text
full phalcom-semantic
full phalcom-core
larger language corpus
incremental semantic suites
optimizer/product suites
```

only when changed shared infrastructure or failures create a real unresolved concern.

## 16.7 Explicitly deferred / DO NOT RUN reflexively during BUILD

Do not repeatedly run:

```text
cargo test --workspace
workspace clippy
all repository language fixtures
all semantic tests
all VM/core tests
all LSP/extension tests
timezone/zoned suites that do not yet exist in P1
```

Reasons:

- P1 touches several shared language mechanisms but most edits are locally discriminable;
- broad failures may be unrelated in an actively changing repository;
- workspace testing after every coherent edit wastes execution time without increasing causal information;
- P2 is not present yet, so its future surface is not a P1 BUILD oracle.

## 16.8 Test serialism

Run expensive Cargo validation commands serially when concurrent builds would compete for resources or obscure failure attribution.

---

# 17. Baseline / unrelated failure policy

Unexpected failures must be classified:

```text
A — definitely caused by P1
B — probably caused by P1
C — unclear
D — clearly unrelated / baseline
```

Policy:

- **A:** active responsibility; fix within P1.
- **B:** investigate and fix unless evidence reclassifies it.
- **C:** perform one bounded classification pass. If nonblocking and still unclear, record in the C2 checkpoint and continue.
- **D:** record meaningful failures and continue immediately.

Do not consume adviser compute on D unless it blocks a required P1 acceptance gate.

Do not:

```text
weaken temporal assertions
restore superseded System.clock behavior
skip negative visibility checks
change expected calendar semantics
hide failing cases behind native code
```

merely to make a broad suite green.

When baseline confirmation is required, use a clean predecessor or other non-destructive comparison. Preserve unrelated user/agent work in the active checkout.

---

> **Body continuation point:** the next planning file must begin with `# 18. Tasks` and must reference `P1-INV-*` and `P1-CV-*` identifiers from this header rather than redefining them.
