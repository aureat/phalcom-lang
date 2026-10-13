---
id: LANG007.C5.P1
category: LANG
program: LANG007
checkpoint: LANG007.C5
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-10-13
prepared_against_revision: b9c8ddd9551c2e15a8a728085f2f259a3e92b6d3
depends_on:
  - canonical closed-sum match projection
  - canonical generic associated types and Universe source delivery
follows: null
supersedes: null
---

# LANG007.C5.P1 — Interval algebra, immutable Progression, and relational matching

## 0. Executor contract

Execute in an isolated worktree, as requested by the user on 2026-10-13 after
planning. Preserve the original checkout's unrelated work. Read the checkpoint
and both companion designs. The user selected typed relationship values and
then authorized starting implementation; record specific contract rulings at G1
before source edits. Do not ask again for general implementation permission.

Prefer small cohesive source modules. Implement algorithms in source where the
language can express them. Rust/native support is authorized when actually
needed, with an explicit owning-layer reason and the normal architecture rules;
authorization is not an instruction to duplicate algorithms across layers.

Adapt mechanics, preserve fixed semantics, use sparse discriminating feedback,
classify unrelated failures, and obey the shared escalation protocol. Completion
requires truthful shared state plus walkthrough and handoff records.

## 1. Goal

Implement validated interval algebra and immutable affine progressions, expose
their computed relationships through existing exhaustive matching, and prove
the hotel quotation / scheduling example with focused evidence.

## 2. Checkpoint acceptance objective

P1 aims to close the bounded C5 objective, including its newly requested
Progression scope. It does not certify all LANG007 or the workspace. The target
is `IMPLEMENTED / FOCUSED_TESTED` only after all required cases and gates pass.

## 3. Repository grounding

```text
repository: aureat/phalcom-lang
planning checkout: /Users/altunhasanli/dev/phalcom/phalcom
branch: main
revision: b9c8ddd9551c2e15a8a728085f2f259a3e92b6d3
relevant predecessor: canonical match / generic source data machinery on main
```

Verified by read-only inspection:

- `Pattern` has variants, nested patterns, or-patterns, wildcards, and Char;
  it has no interval relational node or general data-product node.
- `MatchResolution` / `PatternResolution` and executable projection already
  separate semantic authority from compiler execution.
- `compile_match_with_spec` stores the ordinary scrutinee once and stages
  binding payloads; match arms do not have Boolean guard fields.
- source Range exposes raw optional Number bounds and upper inclusion. Runtime
  `..` / `..=` construction exists although two specs describe inactive operators.
- Date has exact civil-day coordinates, ordering, and Period-based advancement;
  Timestamp has exact nanosecond coordinates and ordering.
- canonical Iterator/Iterable use Item/Iterator associated types and `next`
  returning Option; Progression must join those interfaces.
- builtin source registration uses explicit paths in `phalcom-modules/src/builtin.rs`.
- there is no interval-algebra or Progression implementation at the named owners.
  The graph's `source_index/interval.rs` concerns source-position lookup, not
  this mathematical feature.

The planning checkout had unrelated functional/Option/Result/provider changes.
They were not part of this plan's baseline revision. Do not copy them into the
new worktree merely because provider registration is also an expected C5 edit.
Re-read named paths before editing; architectural drift requires consultation.

## 4. Required reads

1. Root AGENTS guidance, implementation-record lifecycle convention, and shared
   consultation protocol in `docs/workflow/`.
2. [C5 checkpoint](LANG007.C5-CHECKPOINT.md),
   [interval design](interval-relational-matching-spec.md),
   [Progression design](progression-spec.md), and
   [hotel example](hotel-booking-example.md).
3. `docs/specs/README.md`, `objects/data-objects.md`, `objects/enums.md`,
   `objects/traits.md`, `associated-types.md`, `iterator.md`, `universe.md`,
   `collections/range.md`, and `collection-literals.md`.
4. `typing/type/type.reflection-pattern.md` T1063–T1078 and
   `structural-products.md` / `structural-sums.md` for identity boundaries.
5. The exact source owners in §6 and relevant core/semantic test READMEs.

There is no completed C5 predecessor walkthrough/handoff to inherit. Do not
re-audit all C4 plans; check only a seam that actual C5 execution depends on.

## 5. Normative authority

Existing specs govern identity, data/trait construction, type checking,
iteration, evaluation, and matching. The two designs propose new library API
contracts, not already accepted language behavior. G1 records adoption under
the user's implementation authorization and resolves any conflicting existing
authority; do not promote historical implementation defects to language rules.

The Range inactive-operator conflict is explicit. C5 can use `Range.new` and
additive methods without changing literal semantics. If no accepted source
authority can resolve operator activation, leave activation outside C5, record
the disagreement, and avoid operator-dependent C5 fixtures. No silent parser
change is needed to implement the requested feature.

## 6. Takeover state

| Concept | Current symbol/path | Owner | Invariant |
| --- | --- | --- | --- |
| Pattern source | `phalcom-ast/src/ast.rs`, Pattern | AST | Existing grammar |
| Match meaning | `phalcom-semantic/src/match_semantics.rs` | Semantics | Canonical case/binding/coverage knowledge |
| Pattern resolution | `phalcom-semantic/src/checker/pattern.rs`, `coverage/` | Semantics | No library-name shortcuts |
| Pattern projection | `phalcom-core/src/modules/semantic_lowering.rs` | Compiler bridge | Declaration-backed executable facts |
| Pattern execution | `compiler/lib/match_expr.rs`, `patterns.rs` | Compiler | Evaluate once, stage then commit |
| Numeric Range | `core/universe/src/collections/range.ph`, `src/primitive/range.rs` | Source + existing native observations | Membership differs from iteration |
| Iteration | `core/universe/src/collections/iterator_laws.ph` | Source protocol | Fresh traversal and associated Item |
| Date | `core/universe/src/time/calendar/date.ph` | Source | Civil-day order/advance |
| Timestamp | `core/universe/src/time/timestamp.ph` | Source | Absolute nanosecond order |
| Builtin source | `phalcom-modules/src/builtin.rs` | Module provider | Canonical module identity |

Paths beginning `core/` or `src/` in this table are within `phalcom-core/`.

## 7. Architecture and file decomposition

Create an opt-in `universe.intervals` package with focused files:

```text
phalcom-core/core/universe/src/intervals/
    package.ph                 public exposure/import/export graph
    bounds.ph                  bound and structural-view data
    errors.ph                  IntervalError / ProgressionError
    domain.ph                  internal domain capabilities
    integer_domain.ph          exact integer cuts/grid arithmetic
    rational_domain.ph         dense exact cuts/grid arithmetic
    date_domain.ph             calendar-day adapter
    timestamp_domain.ph        exact nanosecond adapter
    interval.ph                immutable representation and factories
    interval_algebra.ph        intersection/subtract/union/hull
    interval_relation.ph       eight-case relation and payloads
    allen_relation.ph          thirteen-case geometry and converse
    progression.ph             immutable recipe and finite transforms
    progression_clip.ph        exact index restriction/subtraction
    progression_relation.ph    five-case point/region relation
    progression_iterator.ph    canonical iteration
```

Facade declarations may share the respective domain file or a small facade
file. Private decomposition is flexible; avoid circular source dependencies.
Keep views/errors/domains below algebra and iteration, and package exports last.
A traits/member association cycle is not a reason to invent a duplicate type
identity or a string-dispatched domain registry.

Source declarations own all library identities. Canonical semantic products
own calls, generic substitutions, associated Step/Item, payload binding types,
and coverage. Compiler lowering executes those products using ordinary sends
and variants. Runtime library bodies calculate algebra. No semantic interval
solver, new pattern node, or interval opcode is planned.

## 8. Ownership boundaries

| Fact | Canonical owner | Consumers | Forbidden duplicate |
| --- | --- | --- | --- |
| Domain/endpoint/grid semantics | Source domain/algebra methods | Runtime API | Second Rust mathematical classifier |
| Declared generic/Step/Item identity | Semantic declarations/conformance | Source checker, runtime reification, LSP | Bare-name or string key identity |
| Match case coverage/binding types | Semantic match products | Compiler and tooling | Native or compiler-specific coverage |
| Traversal position | Fresh ProgressionIterator | `next` / loops | Mutable state on Progression |
| Source files/module identity | Builtin provider | Analysis/bootstrap/imports | VM-only declarations |
| Actual runtime results | Source methods and ordinary VM data | Application code | Advisory shape promoted to proof |

Own only C5 source, adapters, registrations, focused tests, and records. Do not
repair arbitrary generic, solver, bootstrap, C4, or collection issues. A blocking
owning-layer defect requires bounded diagnosis and the consultation protocol.

## 9. Global invariants

Preserve design `INV-01`–`INV-12` and Progression `PR-01`–`PR-08` by reference;
their exact contracts and owners are in the companion documents. Additionally:

- source/root identity survives imports and generic reification;
- Interval/Progression are ordinary immutable data with private raw construction;
- relation payloads are API evidence, not unforgeable arithmetic proofs;
- Range iteration/equality/hash/slicing retain their existing behavior;
- no full collection materialization or scanning in binary interval algebra,
  Progression clipping, `at`, `take`, or `drop`;
- supported source generic conformance may not be replaced with Dynamic merely
  to make the implementation pass.

## 10. Non-goals

No new pattern syntax, guard support, extractor protocol, Float domain,
user-domain admission, arbitrary recurrence, monthly affine stepping,
progression/progression CRT intersection, generalized multiranges, interval
indexes, uncertain-interval constraint solving, booking transactions, scheduler
changes, compile-time interval execution, or workspace certification.

## 11. Expected impact map

Source: new `intervals/` files above; `collections/range.ph` for explicit
adapters; root `core/universe/src/package.ph` for package exposure; named
builtin provider paths and provider tests. Root exposure makes modules
discoverable, not automatically prelude-visible.

Tests, to create and register in existing binaries:

- `phalcom-semantic/tests/semantic/capabilities/intervals.rs` and
  `progressions.rs`: imported source types, cases, and associated types;
- `phalcom-semantic/tests/semantic/incremental/interval_matching.rs`:
  declaration edits, evidence retraction, and cold/incremental agreement;
- `phalcom-core/tests/core/language/intervals.rs` and `progressions.rs`:
  library algorithms, exact differential oracles, execution, GC;
- `phalcom-core/tests/fixtures/language/intervals/` and `progressions/`:
  user-visible examples and negative cases, including hotel fixtures;
- provider tests for opt-in package/source ownership.

Docs: adopted normative owner pages, specs index, Range additions, C5 records,
and LANG007 program/status entries. Adjacent registration/import mechanics are
allowed. Changes to semantic authority, generic identity, native representation,
or runtime contracts require consultation before edits outside this map.

## 12. Implementer decision authority

### FIXED

Typed relationship values, explicit `.view`, domains/case precedence, private
validated representation, exact normalized set/grid semantics, phase-preserving
clip, fresh canonical iterators, ordinary match coverage, and no duplicated
semantic/runtime algebra. All errors and non-goals in the designs are fixed
after G1 adoption.

### MECHANICALLY FLEXIBLE

File splitting, imports, private helper names, equivalent source method shapes,
type aliases, internal facade placement, and existing test helper selection.
Do not change public selector labels without updating the adopted contract and
all examples; source selectors are canonical identity, not cosmetic spelling.

### VERIFY-FIRST

| Assumption | Verify at | If false |
| --- | --- | --- |
| Source-owned generic sums and private data constructors work through imports | T1 exact reproducer | Consult if canonical ownership/reification needs changes |
| Generic domain associated Step can be used without query-local type leakage | T4 / semantic conformance | Mechanical spelling adaptation allowed; authority changes require consultation |
| Date/Timestamp arithmetic supports exact successor/grid coordinates | T2/T4 domain methods | Report actual limits; do not wrap/clamp |
| Rational exposes exact arithmetic needed for floor/ceiling | T2/T4 live number APIs | Use source exact numerator/denominator methods if present; representation-only missing observations may justify native support |
| Universe package files can be opt-in and avoid eager bootstrap | T1/T2 provider/source graph | Consult before expanding bootstrap policy |
| Current broader baseline is not needed for focused acceptance | First gate | Classify actual observed failure, never infer green baseline |

## 13. STOP / CONSULT triggers

Inherit the shared protocol's mandatory conditions. Especially stop if:

- a source-only domain requires new type identity, rooting, or dispatch rules;
- a compiler/library special case for the spelling `Interval` is proposed;
- generic associated types or variant metadata cannot express the required API;
- ordinary source arithmetic cannot express the algorithm and the proposed
  native move has architectural consequences;
- a test oracle requires hull membership to count as progression membership;
- a domain boundary/error must be weakened, or the same semantic failure survives
  one serious causal correction;
- unexpected cross-layer work materially expands P1.

Collect the exact reproducer, call path, invariant, scoped diff, and decision
question. Record the adviser ruling and amend the checkpoint before resuming.

## 14. Debugging budget

Mechanical: at most roughly three coherent cause/correction cycles while making
progress. Semantic: write observed/hypothesis/evidence/prediction/discriminator,
then one serious correction for that cause. Architectural: zero speculative
corrections. Do not rerun unchanged failures or repair unrelated baselines.

## 15. Testing surface / obligation ledger

| ID | Dimension | Contracts | Required evidence |
| --- | --- | --- | --- |
| CV-01 | Endpoint validation/normalization | INV-01–03 | Empty, singleton, reversed, unbounded, all four inclusion combinations, invalid inputs |
| CV-02 | Independent set algebra | INV-04–07 | Small finite Int oracle and dense Rational boundary probes |
| CV-03 | Total relation partition | INV-04–06 | All eight cases, contact gap, payload reconstruction, symmetry |
| CV-04 | Allen partition/converse | INV-08 | All thirteen cases and None preconditions |
| CV-05 | Canonical type/case ownership | INV-09 | Imported generic variants, payload types, omission/redundancy diagnostics |
| CV-06 | Evaluation/binding order | INV-10 | Receiver/argument counters, one relation call, failed alternative no binding leak |
| CV-07 | Range compatibility | INV-12, PR-07 | Dense exact versus integer restriction, unsupported Float, unchanged old behavior |
| CV-08 | Progression recipe validation | PR-01–02 | Zero/negative count, zero stride, step/domain mismatch, finite/unbounded/direction |
| CV-09 | Iterator lifetime and independence | PR-03 | Nested loops, two cursors, repeated None, GC after producer scope |
| CV-10 | Exact clipping | PR-04 | Positive/negative strides, negative coordinates, Rational cuts, included/excluded endpoint, phase preservation |
| CV-11 | Finite transforms | PR-05 | Take/drop, indexing, singleton, empty, double reverse, unbounded errors |
| CV-12 | Sparse membership/relate | PR-06 | Hull intersects but no grid point; all five relation cases; duration distinction |
| CV-13 | Generic/incremental parity | INV-09, PR-08 | Domain/Step/Item imported identity, edit/remove/re-add, cold == incremental |
| CV-14 | Representation/rooting | INV-11, PR-03 | Existing data GC and runtime generic metadata, no native backing pointer |
| CV-15 | Hotel vertical | INV-04–08, PR-04/06 | Exact free pieces, recommendation/cost, policy failures, arrival/service grids |
| CV-16 | Resource behavior | INV-11, PR-04–05 | Huge finite/unbounded recipes, bounded arithmetic, no eager materialization |

The slow finite oracle enumerates actual point/night sets and actual progression
values; it must not call the production normalizer or reproduce its branch tree.
Dense Rational probes include exact endpoints and interior witnesses; finite
enumeration alone does not prove dense topology.

## 16. Verification execution budget

BUILD: one exact imported generic-sum baseline at T1; exact regression per
coherent implemented slice where useful; compiler checks only if informative.
Do not test intentionally incomplete cyclic package registration.

STABILIZE: G2 interval-only modules, G3 Progression/Range modules, G4 semantic /
incremental / projection cases, G5 hotel fixtures. Run Cargo validations serially.
Confirm all filters selected actual tests. Proposed selectors below become
mandatory only after their modules/fixtures are registered; use `-- --list` once
if selection is uncertain.

CERTIFY: not entered by P1. DO NOT RUN workspace tests/clippy, broad corpora,
all semantic tests, or all core tests during BUILD. Broaden only for a concrete
unresolved concern or explicit user/checkpoint gate.

## 17. Baseline / unrelated failure policy

A/B patch-caused failures are active responsibility. C gets one bounded causal
classification pass and can be deferred if outside acceptance and nonblocking.
D is recorded immediately and left alone. A blocker in the exact source-owned
generic/data seam cannot be disguised as a green C5 gate. Never weaken assertions,
mark source tests ignored, or replace required behavior with Dynamic to pass.

## 18. Tasks

### T1 — Adopt contracts and establish the isolated seam

**Purpose / preconditions:** finish the planning records, then create the
user-requested worktree from the recorded live revision; carry only C5 planning
files and scoped program index edits. Read status/diff and applicable guidance.

**Consumes:** designs, live closed-sum/type/Universe authorities. **Produces:**
G1 adoption ledger, exact baseline outcome, active C5 state, and authoritative
interval/progression API pages after adoption.

**Read:** named owners in §6, source Rational, Date/Period, Timestamp/Duration,
provider registration, and test conventions. **Modify/create:** C5 records,
`docs/specs/intervals.md`, `docs/specs/progressions.md`, specs index; add only
necessary Range adapter contract prose. Do not activate range operators by stealth.

**Shape:** record each conflict/ruling and its scope. Use one exact regression
proving imported generic closed-sum payload types and execution with today's
machinery before attributing later failures to C5. Existing semantic selector:
`adts::matching::patterns::resolve_exact_and_labeled_variant_patterns`.

**Tests now:** that semantic case, then the exact imported generic-sum core
baseline reproducer; no workspace test. **Deferred:** new algebra and iteration.
**Acceptance:** isolation exists, source identities/case execution evidence is
known, and G1 contract decisions are durable. **STOP:** owning authority or
generic reification conflicts. **Checkpoint update:** active plan, adopted
decisions, baseline result, worktree path, and revision.

### T2 — Validated interval domains and immutable representation

**Purpose:** establish bounds, views, errors, four domain adapters, and checked
Interval factories. **Preconditions:** G1. **Consumes:** canonical source data /
generic APIs. **Produces:** validated `Interval<T,D>` and facade constructors.

**Files:** bounds/errors/domain files, four domain files, interval.ph, provider
path registrations and opt-in package graph, focused core interval tests.

**Shape:** split discrete normalization from dense inclusion; distinguish
rejecting reversed user inputs from empty computed intersections. Use private
raw constructors and exact successor; do not expose unchecked Interval creation.
Prefer existing source arithmetic/observations; any native necessity is a named
architecture decision, not a speed optimization.

**Tests added / now:** CV-01 exact factory cases, one Date leap-day successor,
one Timestamp nanosecond case, and Rational singleton/empty. Run only the new
construction test group once the source graph is coherent. **Deferred:** algebra,
Allen, Progression, broad bootstrap. **Acceptance:** factories and `.view` produce
correct normalized immutable values. **STOP:** representation/identity changes
or unsupported exact arithmetic. **Checkpoint update:** stable factory/domain
interfaces, with actual evidence strength.

### T3 — Algebra, eight set cases, and thirteen Allen cases

**Purpose:** implement binary interval computations. **Preconditions:** T2.
**Consumes:** validated bounds/domains. **Produces:** IntervalPieces,
IntervalRelation, AllenRelation and operations defined by the design.

**Files:** interval_algebra.ph, interval_relation.ph, allen_relation.ph and
interval core tests/fixtures. **Shape:** one shared intersection/cut implementation;
discard empty residuals; preserve holes; exact eight-case precedence; finite
proper-span Allen preconditions. No enumeration or hidden Rust classifier.

**Tests:** CV-02/03/04 including dense both-open touching hole, closed/closed
singleton intersection, all/unbounded subtraction, every Allen converse,
normalization-equivalent discrete inputs, and independent small-set oracle.
**Run now:** G2 at this coherent algebra boundary. **Deferred:** Progression,
cross-layer type/tooling matrix, hotel. **Acceptance:** all algebra identities
and payloads agree with the independent oracle. **STOP:** oracle change or
special-case pattern coverage. **Checkpoint update:** implemented algebra
interfaces and G2 evidence.

### T4 — Immutable Progression and canonical independent iteration

**Purpose:** implement affine recipes, transforms, grid membership, interval
clipping/subtraction, point/region relations, and fresh iterators.
**Preconditions:** T2/T3. **Consumes:** domains, interval cuts, associated types,
canonical Iterator/Iterable. **Produces:** Progression API and relation/piece sums.

**Files:** progression*.ph, domain Step/grid methods, package exposure, core
progression tests, semantic Progression type cases. **Shape:** keep mutable
cursor only in iterator; validate step before empty normalization; use exact
floor/ceil index calculations; retain phase/sign; no scan in transforms.
Daily Date stepping excludes months. Duration applies only to Timestamp.

**Tests added:** CV-08–12/14/16, including two cursors, nested loops, GC, sparse
hull counterexample, negative coordinates, descending clip, exact Rational
boundary, huge count indexing, finite reverse, and unbounded errors.
**Run now:** discriminating recipe/clip and iterator tests after each coherent
group, not after each file. **Deferred:** G3 until Range bridge exists; broad
iteration/generic suites. **Acceptance:** exact recipe laws and fresh iterators
with canonical Item and Step. **STOP:** generic associated-type/reification or
GC authority defect. **Checkpoint update:** stable recipe and iterator interfaces.

### T5 — Explicit Range/Interval/Progression bridges

**Purpose:** connect existing numeric Range without altering old behavior.
**Preconditions:** T3/T4. **Consumes:** raw Range observations and checked APIs.
**Produces:** `toExactInterval`, `toIntegerInterval`, Range `relate`, Range
`progression(step:)`, discrete Interval `progression(step:)`.

**Files:** collections/range.ph, source facade/bridge helpers, focused Range /
Progression fixtures and provider tests. **Shape:** exact numeric membership
bridge versus explicit integer restriction; reject unsupported endpoints;
positive/negative stride anchoring as specified; no `first`/`last` inference,
materialization, reanchoring of an existing grid, or RangeIterator rewrite.

**Tests:** CV-07/10/11 plus two distinct negative-stride versus reversal grids,
omitted required anchor, reversed Range conversion, fractional Rational bound,
unsupported Float, and old membership/iteration/hash/slicing regressions.
**Run now:** G3. **Deferred:** broad collections corpus. **Acceptance:** all
bridges agree with selected domain semantics and old behavior is preserved.
**STOP:** implicit coercion or legacy contract alteration. **Checkpoint update:**
bridge interfaces and completed G3 evidence.

### T6 — Canonical matching and incremental integration

**Purpose:** prove ordinary match/type/metadata machinery supports all new values.
**Preconditions:** T3–T5. **Consumes:** exported source declarations.
**Produces:** semantic/projection/runtime evidence, not a new interval resolver.

**Files:** semantic capabilities/intervals.rs and progressions.rs, incremental/
interval_matching.rs, core interval/progression projection tests. Production
semantic/compiler edits are only for a demonstrably C5-blocking owning-layer
defect under the normal consultation rules; do not add builtin-name branches.

**Tests:** CV-05/06/09/13/14, all-eight/all-thirteen/all-five coverage, one missing
case, redundant case, nested `.view`, generic payload binding types, or-pattern
binding compatibility, operand counters, snapshots across declaration edits,
reification through imports, and GC-preserved result payloads.
**Run now:** G4. **Deferred:** unrelated full semantic/LSP/optimizer suites.
**Acceptance:** compiler and consumers use canonical published identities and
agree with cold analysis; no arithmetic facts are inferred from forged values.
**STOP:** a second solver, weakened coverage, or representation special case.
**Checkpoint update:** G4 evidence and any narrow consultation/amendment.

### T7 — Executable hotel and scheduling example

**Purpose:** deliver the nontrivial end-user vertical. **Preconditions:** T6.
**Consumes:** all public APIs. **Produces:** complete executable hotel fixture,
expected output, negative cases, and updated example matching the source.

**Files:** new interval/progression language fixture directories and owning core
tests. **Shape:** implement every critical availability/quote algorithm in the
example, not undefined placeholder helpers; capture `now` once; preserve half-open
nights and separate Timestamp services. Use integer cents and finite application
inventory; Progression remains unbounded/lazy where specified.

**Tests:** CV-15 plus independent enumerated-night oracle, active/expired/own-ID
policy, 1,020.00 recommendation, sparse grids, turnover and 90-minute service
duration. **Run now:** G5. **Deferred:** production database, time-zone redesign,
broad time corpus. **Acceptance:** documented outputs are compiled/runtime
results and every produced itinerary satisfies the independent night oracle.
**STOP:** changing expected business policy or using a hull as availability.
**Checkpoint update:** end-to-end evidence and residual limitations.

### T8 — Focused stabilization and delivery records

**Purpose:** close P1 honestly. **Preconditions:** required gates have results.
**Consumes:** scoped diff and coverage ledger. **Produces:** final C5/P1 state,
walkthrough/handoff, reviewed public specs/examples and scoped source diff.

**Files:** task-owned source/tests/docs only. **Shape:** map every CV obligation
to evidence; resolve A/B failures, classify C/D, verify no accidental unrelated
or whitespace-only edits. Do not rerun already-green unchanged gates.
**Tests now:** only missing required focused evidence or checks justified by
new stabilization corrections. **Deferred:** workspace/release suite. **Acceptance:**
all obligations satisfied or explicitly truthful incomplete state; no fabricated
COMPLETE. **STOP:** unresolved mandatory acceptance. **Checkpoint update:** exact
completion/verification, consultations, residuals, and next action.

## 19. Verification gates

### G1 — Decision and exact baseline

Record detailed adoption and the Range authority resolution/limited deferral.
Run the exact baseline cases in T1 before attributing failures to new APIs.
No broad baseline certification is required. A blocking owning-layer failure
stops only the affected implementation path and enters consultation.

### G2 — Interval algebra

After registration, use:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::intervals
```

Expected: factory, topology, algebra, payload, Allen, and independent-oracle
cases pass. The module is newly planned, not claimed to exist at planning time.

### G3 — Progression and Range

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::progressions
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus intervals
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language-corpus progressions
```

Register the focused fixtures before selection. Existing Range regression cases
are selected by exact inspected names, not by inventing a nonexistent target.
Expected: exact grids, independent iterators, bridges and compatibility pass.

### G4 — Canonical semantic and incremental products

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::intervals
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::progressions
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::interval_matching
```

Expected: actual tests selected; canonical declaration/case/associated types,
diagnostics, and edit/retraction agreement pass. Do not count unknown/blocked
facts as proof. Existing compiler/runtime tests in G2/G3 provide projection
execution evidence; T6 adds the exact order/GC cases if missing.

### G5 — Hotel vertical

Select the newly registered `hotel_booking` and `hotel_service_progressions`
fixtures by actual listed names. Expected: worked pieces, costs, policies,
phase-aligned grids, and physical service boundaries match §15 CV-15.
No automatic broadening after PASS. Canceled/time-out gates remain incomplete.

## 20. Final focused acceptance

The walkthrough maps CV-01–16 and INV/PR contracts to named tests and completed
results, including selected test counts. “Files written” and “tests added” do
not discharge evidence obligations. Checkpoint closure requires all acceptance
items in the shared record, not only the first Integer-domain slice.

## 21. Performance/resource evidence

Check huge finite recipes and clipped unbounded recipes without member scanning;
binary interval algebra emits at most two pieces. Count dependence must not
appear in clipping/indexing implementation loops. Exact big-number arithmetic
cost is permitted and depends on operand bit length. No wall-clock benchmark,
zero-allocation claim, or optimizer/native migration is required for P1.

## 22. Checkpoint bookkeeping

At start record worktree, baseline, adoption, active plan, and revision. During
execution update only stable interfaces, coherent gate evidence, consultations,
and significant deferred failures. At completion update plan ledger, exact
completion/verification, adopted invariants, residuals, and next action. The
checkpoint owns durable state; do not create a competing implementation-state.

## 23. Walkthrough deliverable

Create `LANG007.C5.P1-walkthrough.md` with implemented architecture, source and
native changes with reasons, actual tests/counts/results, deviations/adviser
rulings, deferred tests, resource evidence, residual risks, and exact state.
If only a bounded first slice exists, mark the record interim and partial.

## 24. Handoff deliverable

Create `LANG007.C5.P1-handoff.md` with worktree/revision, stable interfaces,
invariants, current task, first discriminator, blockers/deferred failures, must
reads, and what not to redesign. Do not require the next implementer to repeat
already-established focused evidence.

## 25. Completion truth table

```text
planning complete                     != source implemented
one supported domain implemented      != four-domain C5 complete
source + tests written                != focused-tested
interval tests pass                   != Progression/bridge/hotel verified
all required focused gates pass       => FOCUSED_TESTED
clean / committed / pushed worktree   != release certification
```

Commit/push only if requested. Retain the user's new isolated worktree for
continuation; do not archive it when work is still active.

## 26. Plan self-review

Every algebra/grid fact has a source owner and every formal match/type fact a
semantic owner. The design separates runtime evidence from proof, sparse grids
from hulls, civil dates from absolute time, Range membership from iteration, and
traversal direction from set topology. Test obligations include both dense
boundary probes and independent discrete oracles. Tasks follow a valid
dependency order with bounded gates and escalation rules; broad certification
is deferred. Public APIs, examples, and final compiled fixtures must be kept
consistent during mechanical adaptation.
