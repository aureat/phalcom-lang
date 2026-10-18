---
id: COLL006.C1.P9
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - COLL006.C1.P8
follows: COLL006.C1.P8
supersedes: null
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 3f8cd71358b5c5155ac0a92c7ab232e5a86c4fb6
prepared_at: 2026-10-12
---

# COLL006.C1.P9 — Collection Convergence, Migration, Certification, and C1 Closure

## 0. Executor contract

This document is a **Luna patch-grade implementation contract** for `COLL006.C1.P9`.

P9 is the final convergence and certification plan for `COLL006.C1`. It is **not** permission to redesign storage, iteration, type-system, literal-selection, persistence, ordering, or collection semantics already established by P1–P8.

The implementer must:

- execute P9 only after P8 is `COMPLETE / IMPLEMENTED` and has produced a current walkthrough and handoff;
- consume P8's as-built public API, persistence route, RangeMap iteration shape, normalization semantics, callback/GC invariants, native-floor delta, and residual ledger before any P9 source edit;
- preserve P1–P8 architectural ownership and reopen an earlier owner only when evidence demonstrates that an earlier invariant is false or incomplete;
- prioritize **gate closure, evidence quality, cross-family consistency, migration completeness, residual classification, and release truth** over adding new mechanisms;
- avoid opportunistic runtime/compiler refactors merely because P9 touches many files;
- not introduce a new collection storage substrate;
- not replace P7's persistent hash structure;
- not replace P8's persistent ordered structure;
- not reinterpret generic `Interval<T>` through numeric/progression `Range`;
- not create a second `Iterator` / `Iterable` hierarchy;
- not introduce a dynamic/existential runtime Vector extent merely to simplify conversion APIs;
- not change established iterator item types without an explicit public compatibility ruling;
- not add a native primitive unless a separately accepted native-floor amendment explicitly authorizes it;
- not weaken, skip, rename away, or silently reclassify an acceptance test merely to make P9 pass;
- not claim `RELEASE_COMPLETE` while known current release/CI gates fail;
- classify every surviving failure using the fixed P9 residual vocabulary;
- produce durable evidence records as work proceeds, not only at the end;
- update `COLL006.C1-CHECKPOINT.md` at every durable gate transition;
- create the P9 walkthrough, residual ledger, verification ledger, migration ledger, and handoff required by this plan.

Testing is evidence gathering. During BUILD mode, run the narrowest discriminator that can falsify the active implementation hypothesis. Broad workspace/CI certification belongs to the late P9 gates.

---

# 1. Mission

Close `COLL006.C1` as one coherent collection system.

P1–P8 establish collection families and mechanisms. P9 must make their resulting public API, iteration model, conversion graph, diagnostics, documentation, property laws, GC behavior, callback behavior, performance contracts, lifecycle state, and repository-wide verification evidence agree.

P9 must therefore converge:

```text
P1–P8 as-built surfaces
        ↓
one coherent collection API
        ↓
one coherent specification/documentation set
        ↓
one coherent property / GC / callback / performance story
        ↓
one closed residual ledger
        ↓
one truthful checkpoint closure
```

P9 does not close successfully merely because focused collection tests pass.

---

# 2. Prepared-against repository state

This plan was prepared against:

```text
repository:      aureat/phalcom-lang
branch:          main
revision:        3f8cd71358b5c5155ac0a92c7ab232e5a86c4fb6
revision title:  feat(LANG007.C4.P6): implement static control flow,
                 structural iteration, and proof predicates
date:            2026-10-12
```

Current known state at preparation time:

```text
P1–P7:                  complete
P8:                     proposed / not started / unverified
P9:                     proposed / not started / unverified
generated native count: 488
canonical sequence rule: runtime bare [] -> Array
                         static bare []  -> Vector
                         contextual List remains available
```

Important current convergence findings:

```text
Map still contains legacy nongeneric mutable-field Entry
Bag lacks canonical Iterator/Iterable conformance
Bijection lacks canonical Iterator/Iterable conformance
List -> Array conversion has no complete canonical public surface
Array/List/Buffer -> Vector<T,N> conversion has no complete canonical public surface
several current collection specs still describe pre-P4 List-literal semantics
COLL006/STATUS.md is stale relative to C1 checkpoint
R-PARSE-ASSOCIATED-INVOKE remains an explicit inherited P9 obligation
R-P7-NUMERIC-HASH-LAW remains a real observed equality/hash mismatch
P6 families have strong deterministic tests but limited consolidated randomized model evidence
```

These are planning inputs, not immutable facts. T0 must re-census them against the actual P9 execution revision.

---

# 3. Hard predecessor: P8

P9 production work is blocked until P8 finishes.

P8 owns:

```text
Bound<T>
Interval<T>
canonical endpoint algebra
persistent ordered storage
RangeMap<K,V>
point lookup
strict insert
removal semantics
overlay/split semantics
normalization/coalescing
RangeMap equality/hash behavior
RangeMap iteration behavior
ordered-domain callback rules
P8 native route/floor decision
P8 GC and structural-sharing guarantees
P8 randomized model and complexity evidence
```

Before any P9 source edit, verify:

```text
P8 checkpoint status == COMPLETE
P8 completion == IMPLEMENTED
P8 verification >= FOCUSED_TESTED
P8 walkthrough exists
P8 handoff exists
P8 public Bound/Interval/RangeMap API is documented
P8 empty/invalid Interval semantics are fixed
P8 Ordering::Unordered behavior is fixed
P8 strict-insert overlap behavior is fixed
P8 removal semantics are fixed
P8 overlay semantics are fixed
P8 normalization semantics are fixed
P8 equality/hash behavior is fixed
P8 iterator item shape is fixed
P8 native-floor delta is known
P8 focused GC/callback/randomized/complexity gates passed
P8 residuals are explicitly handed off
```

If any item is missing, STOP.

P9 must inherit whichever P8 route landed:

```text
Route A:
    source-owned interval + persistent ordered implementation

Route B:
    source-owned interval semantics
    + GC-visible native persistent ordered substrate
    + accepted native-floor amendment
```

P9 must not replace P8's tree architecture, reinterpret interval algebra, change the `O(log n)` point-lookup contract, change the `O(log n + k)` splice/overlay contract, or weaken failure-atomic callback behavior.

---

# 4. Source-of-truth hierarchy

P9 uses this authority order:

1. `docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md`
2. C1 master/detailed implementation records
3. current normative `docs/specs/`
4. accepted TDRs
5. historical implementation records

Historical plans explain provenance but do not override current specifications. `COLL006/STATUS.md` is a summary and must not override the checkpoint.

---

# 5. Effective collection census

P9 must certify the system spanning:

```text
Sequence/storage:
  List<T>
  Array<T>
  Vector<T,N>
  Slice<T>
  Buffer<T>

Mutable keyed/ordered:
  Map<K,V>
  Set<T>
  MutableBag<T>
  IndexedDict<K,V>
  OrderedSet<T>

Persistent hash:
  Bag<T>
  Bijection<K,V>

P8 interval family:
  Bound<T>
  Interval<T>
  RangeMap<K,V>

Adjacent compatibility:
  Collection
  Iterator
  Iterable
  Tuple
  Record
  Range
  Bytes
```

---

# 6. Required P9 artifacts

Create under `docs/implementation/COLL006/COLL006.C1/P9/`:

```text
COLL006.C1.P9-luna-patch-grade-implementation-plan.md
COLL006.C1.P9-api-convergence-ledger.md
COLL006.C1.P9-migration-ledger.md
COLL006.C1.P9-property-certification-ledger.md
COLL006.C1.P9-residual-ledger.md
COLL006.C1.P9-verification-ledger.md
COLL006.C1.P9-walkthrough.md
COLL006.C1.P9-handoff.md
```

Exact names may follow live repository convention, but the evidence categories must remain distinct.

Also update:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
docs/implementation/COLL006/PROGRAM.md
docs/implementation/COLL006/STATUS.md
```

---

# 7. Fixed residual vocabulary

Every residual ends as exactly one of:

```text
RESOLVED
EXTERNAL_BASELINE
C1_OWNED
AMBIGUOUS
```

Rules:

```text
RESOLVED
    exact discriminator passes

EXTERNAL_BASELINE
    reproduces on contemporaneous main and ownership is proven outside C1

C1_OWNED
    comparative evidence identifies a C1 regression or inherited C1 obligation

AMBIGUOUS
    ownership remains unproven
```

Closure:

```text
C1_OWNED  -> must repair
AMBIGUOUS -> closure prohibited
EXTERNAL_BASELINE -> may permit BASELINE_BLOCKED, never silently RELEASE_COMPLETE
```

---

# 8. P9 global invariants

- **P9-INV-01:** earlier P1–P8 architecture is preserved.
- **P9-INV-02:** every C1 iterable has actual canonical `Iterator`/`Iterable` evidence.
- **P9-INV-03:** `Iterable::Item == Iterator::Item` under canonical evidence.
- **P9-INV-04:** `Entry<K,V>` has one canonical immutable definition.
- **P9-INV-05:** Entry modernization does not silently change established mapping iterator item shapes.
- **P9-INV-06:** every conversion declares allocation/view, mutability, failure behavior, and element/reference semantics.
- **P9-INV-07:** no conversion invents runtime/existential Vector extent `N`.
- **P9-INV-08:** zero-copy conversions retain owner storage.
- **P9-INV-09:** failed conversion publishes no partial target.
- **P9-INV-10:** bare runtime brackets remain Array; bare static brackets remain Vector; contextual List remains available.
- **P9-INV-11:** List remains mutable and dynamically resizable.
- **P9-INV-12:** `Range` remains numeric progression/slicing; `Interval<T>` remains generic region abstraction.
- **P9-INV-13:** no keyed collection holds live heap/object borrows across user `hash`, `==`, or `<=>`.
- **P9-INV-14:** persistent old roots never mutate.
- **P9-INV-15:** persistent nodes/views/iterators remain GC-correct.
- **P9-INV-16:** hash-law certification cannot ignore a known equality/hash mismatch.
- **P9-INV-17:** current public/normative docs agree with implementation.
- **P9-INV-18:** historical records remain historical.
- **P9-INV-19:** no active public `SetList` terminology remains.
- **P9-INV-20:** no unresolved `C1_OWNED` or `AMBIGUOUS` residual remains at closure.
- **P9-INV-21:** `RELEASE_COMPLETE` is claimed only from current broad evidence.

---

# 9. T0 — Takeover, rebaseline, and architecture freeze

## Goal

Establish execution truth before edits.

## Actions

Record:

```text
branch
HEAD
origin/main
git status
worktree identity
native primitive count
Object/Value size invariants
```

Read current:

```text
C1 checkpoint
P8 walkthrough/handoff
P8 final plan
P8 TDR/consultation records
docs/specs/README.md
docs/specs/iterator.md
docs/specs/sequence-literals.md
collection specs
scripts/verify.sh
.github/workflows/ci.yml
```

Re-run exact discriminators for:

```text
R-PARSE-ASSOCIATED-INVOKE
R-P7-NUMERIC-HASH-LAW
P4 D1
P4 D2
P4 D4
all P8 residuals
```

Mandatory parser-package command:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

Re-census constructor authority:
- current normative class specification;
- whether `init Self(...)` architecture landed;
- whether `@constructor call(...)` remains relevant;
- any accepted superseding construction model.

Create initial residual ledger containing:

```text
ID
discriminator
observed result
origin/main result where needed
owner
classification
next action
```

## Acceptance

```text
P8 takeover complete
current SHA recorded
native baseline recorded
constructor authority known
known residuals have current discriminators
no production source modified
```

---

# 10. G0 — Takeover gate

Pass only when:

```text
P8 complete
P8 handoff consumed
P8 architecture frozen
current main recorded
native baseline recorded
constructor authority identified
known residuals re-discovered
no unresolved P8 semantic decision
```

No P9 production implementation before G0.

---

# 11. T1 — API convergence decision ledger

Create `COLL006.C1.P9-api-convergence-ledger.md`.

Freeze:

### D1 — `Entry<K,V>` ownership
Preferred: reusable immutable source/data type under `universe.collections.entry`.

### D2 — mapping iterator compatibility
Default recommendation: preserve `(key: K, value: V)` iteration for C1 compatibility; use `Entry` as explicit association value.

### D3 — association APIs
Define exactly where APIs accept/produce `Entry`, labeled tuples, or both.

### D4 — List → Array
Fix exact public selector/factory shape.

### D5 — checked conversion to `Vector<T,N>`
Prefer target-owned construction using statically known `N`, subject to live language capability.

### D6 — Vector conversion failure form
Fix exact `Result`/raised-error/error-class contract.

### D7 — constructor convention
Use the construction model that is actually normative at execution time.

### D8 — RangeMap association presentation
Consume P8 item shape; decide only the P9 presentation/conversion boundary.

### D9 — normative spec ownership
Fix which document owns family, iteration, literal, conversion, and interval semantics.

### D10 — numeric hash-law ownership
Decide repair ownership or external baseline path before final hash certification.

---

# 12. G1 — API convergence gate

Pass only when D1–D10 are written and unambiguous.

---

# 13. T2 — Canonical `Entry<K,V>` implementation

## Goal

Replace the obsolete Map-local association value without destabilizing iteration.

Preferred shape:

```telos
data Entry<K, V>(
  _ key: K,
  _ value: V,
)
```

Use exact current data-object syntax.

Likely files:

```text
phalcom-core/core/universe/src/collections/entry.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/core/universe/src/collections/map.ph
```

Potential module registration only if live module topology requires it.

## Requirements

1. one generic immutable Entry;
2. legacy Map-local Entry removed/retired;
3. Map iteration preserved unless G1 explicitly changes it;
4. approved association constructors/conversions updated;
5. `Iterable.toMap` adjusted only as required by D3;
6. IndexedDict/Bijection/RangeMap boundaries updated only as approved;
7. no native heap arm unless ordinary data is proven insufficient — such a finding triggers consultation.

## Tests

```text
generic typing
immutability
key/value projections
data equality/hash semantics
Entry mapping conversion
labeled-tuple compatibility if retained
Map iterator compatibility
IndexedDict iterator compatibility
Bijection iterator compatibility
RangeMap presentation compatibility
```

Negative:

```text
mutation rejected
wrong association item diagnosed
duplicate conversion failure atomic
```

---

# 14. T3 — Iterator / Iterable convergence

Audit canonical conformance for:

```text
List
Array
Vector
Slice
Buffer
Map
Set
MutableBag
IndexedDict
OrderedSet
Bag
Bijection
RangeMap
```

Known likely fixes to reverify:

```text
BagIterator<T> / Bag<T>
BijectionIterator<K,V> / Bijection<K,V>
```

Required Item matrix:

| Family | Item |
|---|---|
| `List<T>` | `T` |
| `Array<T>` | `T` |
| `Vector<T,N>` | `T` |
| `Slice<T>` | `T` |
| `Buffer<T>` | `T` |
| `Set<T>` | `T` |
| `OrderedSet<T>` | `T` |
| `MutableBag<T>` | `T` |
| `Bag<T>` | `T` |
| `Map<K,V>` | `(key: K, value: V)` |
| `IndexedDict<K,V>` | `(key: K, value: V)` |
| `Bijection<K,V>` | approved mapping item |
| `RangeMap<K,V>` | approved P8/P9 item |
| `Range` | `Number` |
| `Bytes` | `Int` |

Behavior requirements:

```text
reusable source -> fresh independent iterators
iterator.iterator -> same traversal
after first None, later next remains None
```

Add static/generic probes proving trait evidence rather than only dynamic method success.

STOP if conformance exposes a semantic associated-type defect; do not spoof it dynamically.

---

# 15. G2 — Entry and iteration gate

Pass criteria:

```text
one canonical Entry
legacy Entry retired
no silent iterator migration
all C1 iterables canonically conform
Item matrix verified
fresh traversal verified
iterator self-iterability verified
generic/for-loop probes pass
```

---

# 16. T4 — Conversion graph

Required graph:

```text
List<T>    <-> Array<T>      allocating

Array<T>   -> Slice<T>       zero-copy
Vector<T,N>-> Slice<T>       zero-copy
Buffer<T>  -> Slice<T>       zero-copy

Array<T>   -> Vector<T,N>    checked allocating
List<T>    -> Vector<T,N>    checked allocating
Buffer<T>  -> Vector<T,N>    checked allocating
```

Existing useful conversions remain.

## List ↔ Array

```text
shallow element/reference copy
independent storage
order preserved
Array fixed-size
List resizable
source unchanged
```

## Slice views

Certify:

```text
Array.slice(..)
Vector.slice(..)
Buffer.slice(..)
```

as zero-copy owner-retaining views with existing nested-slice normalization.

## Vector conversion

```text
target Vector<T,N> statically known
exact cardinality == N
no truncation
no padding
no dynamic N
no partial publication
immutable result
```

Test:

```text
too few
too many
N=0
N=1
larger N
iteration/callback failure
resource failure where injectable
```

Heap-backed element identity/reference must be preserved; do not deep-clone user objects.

Round-trip properties:

```text
List -> Array -> List preserves sequence
Vector -> Array preserves contents and yields distinct mutable target
Array -> Vector<T,N> -> Array preserves contents
Slice -> List snapshots independently
mutating allocated target does not mutate source
```

---

# 17. G3 — Conversion gate

Pass criteria:

```text
all required edges exist
allocation/view semantics match ledger
Vector cardinality exact
no dynamic N
zero-copy proven structurally
failure atomicity proven
round-trip properties pass
```

---

# 18. T5 — Constructor convergence

Re-read constructor authority immediately before edits:

```text
docs/specs/objects/classes.md
docs/specs/source-declarations.md
accepted construction TDRs
live parser/semantic constructor implementation
TELOS-Class-Initialization-DataClass plan status
```

### Case A
Current `@constructor` model remains normative: normalize only where needed for API coherence.

### Case B
`init Self(...)` model has landed and is normative: migrate collections directly to it.

### Case C
New model remains proposed but imminent: avoid a broad temporary migration unless C1 semantics require it.

Do not rename raw/native mechanism selectors merely for cosmetic consistency.

Test construction type/result, generic application, empty/from construction, error paths, and representation guards.

---

# 19. T6 — Diagnostics convergence

Build a diagnostics matrix covering:

### Sequence/literal
```text
underconstrained empty literal
incompatible heterogeneous literal
ambiguous family
invalid explicit head
Vector extent mismatch
invalid static extent
```

### Mutable sequence
```text
Array bounds
const Array mutation
Buffer invalid capacity
Buffer exhaustion
Slice mutation
```

### Keyed collections
```text
mutable collection key rejection
duplicate key/value
invalid position
duplicate positional insertion
invalid move/replace
re-entrant mutation rejection
```

### P8
```text
invalid interval
prohibited empty interval
RangeMap overlap
unordered comparison
invalid remove/overlay
```

### Conversion
```text
wrong association shape
duplicate key during map conversion
Vector cardinality mismatch
unsupported conversion
```

Diagnostics must be stable, user-facing, and owned by the correct layer.

---

# 20. T7 — Normative specs, docs, and maintained-source migration

Mandatory current audits/updates:

```text
docs/specs/collections/README.md
docs/specs/collections/list.md
docs/specs/collections/array.md
docs/specs/collections/vector.md
docs/specs/collection-families.md
docs/specs/iterator.md
docs/specs/sequence-literals.md   # audit; edit only if necessary
```

Remove stale active claims such as:

```text
bare [a,b] always constructs List
[] is always List
Array is only proposed/unimplemented
Vector is only proposed/unimplemented
```

Public docs must distinguish:

```text
Range       -> numeric progression / slicing
Interval<T> -> generic ordered-domain region
```

Migration census maintained source/spec/examples for:

```text
SetList
legacy bare-List wording
obsolete Array/Vector status
legacy cursor terminology
stale Entry examples
stale constructor examples
Range used where Interval is intended
```

Classify each hit:

```text
active/public -> fix
intentional historical fixture -> preserve/annotate
historical plan/decision -> preserve
```

Audit link integrity across C1/P6/P7/P8/P9/TDR/spec references.

Update `COLL006/STATUS.md` and `PROGRAM.md` to summarize checkpoint truth rather than compete with it.

---

# 21. G4 — Diagnostics and migration gate

Pass criteria:

```text
diagnostic matrix complete
current docs agree with live semantics
Array/Vector no longer documented as unimplemented
active bare-List wording removed
Range/Interval distinction explicit
no active SetList terminology
current lifecycle/spec links valid
STATUS/PROGRAM aligned
historical records preserved
```

---

# 22. T8 — Property certification consolidation

Create/update property-certification ledger.

All randomized tests must use deterministic seeds and report the seed on failure.

## MutableBag

Linear multiplicity model; generate:

```text
add
remove
clear
count
```

Compare counts, total, distinct count, iteration multiplicity after every operation.

## IndexedDict

Reference `Vec<(key,value)>`; generate:

```text
insert/overwrite
strict positional insert
remove key
removeAt
move
replace/value update
clear
```

Compare size, lookup, positions, `indexOfKey`, iteration order, and state preservation on errors.

## OrderedSet

Reference unique vector; generate:

```text
add
strict positional insert
remove
removeAt
replaceAt
move
clear
```

Compare order/membership after every step.

## Bag

Reference multiplicity map; test:

```text
adding
removing
union
intersection
difference
+
old-root persistence
```

## Bijection

Reference forward/reverse maps. After every successful update:

```text
forward[k] == v iff reverse[v] == k
```

Test conflicts, inverse, double inverse, and old roots.

## RangeMap

Reuse P8's canonical reference model and seeds. Do not create a second interval semantics model.

## Conversion properties

Property-check List/Array, Vector/Array, full Slice, exact Vector cardinality, and association conversions.

## Equality/hash

For immutable hashable families, require:

```text
a == b => hash(a) == hash(b)
```

subject to truthful resolution/classification of `R-P7-NUMERIC-HASH-LAW`.

---

# 23. T9 — GC and callback/re-entry certification

Verify iterator roots for all C1 iterators, view roots for Slice ownership, and multi-version persistent GC for:

```text
Bag
Bijection
RangeMap
```

Create versions, root old/new, force GC, drop selected intermediates, force GC again, verify retained versions and collectability of unreachable unique nodes.

Mutable removals/replacements must stop tracing removed children.

Callback matrix across:

```text
Map
Set
MutableBag
IndexedDict
OrderedSet
Bag
Bijection
RangeMap
```

Callbacks:

```text
hash
==
<=>
P8 normalization value equality if applicable
```

User callbacks should exercise:

```text
read collection
allocate
force GC
throw
mutate unrelated state
allowed same-receiver read
forbidden same-receiver mutation
```

After failure, mutable state must match its contract and persistent operations must publish no partial root.

---

# 24. G5 — Property / GC / callback gate

Pass criteria:

```text
all reference models pass
all recorded seeds pass
conversion properties pass
iterator/view GC tests pass
persistent-version GC tests pass
callback allocation/GC/error tests pass
failure atomicity passes
no heap borrow spans user callback
```

---

# 25. T10 — Performance contract certification

Prefer structural counters over wall-clock thresholds.

Required contracts:

| Operation | Contract |
|---|---:|
| Array index | O(1) |
| Vector index | O(1) |
| Slice creation | O(1) |
| Slice index | O(1) |
| Buffer index/set/push | O(1) |
| MutableBag lookup/update | expected O(1) |
| IndexedDict key lookup | expected O(1) |
| IndexedDict positional access | O(1) |
| OrderedSet membership | expected O(1) |
| Bag/Bijection lookup | persistent hash depth |
| Bag/Bijection update | path-copy hash depth |
| Bijection.inverse | O(1) |
| RangeMap lookup | O(log n) |
| RangeMap overlay | O(log n + k) |

Detect regressions such as:

```text
Array/Vector literal stages through List
Slice copies values
IndexedDict keyed lookup scans positional storage
OrderedSet membership scans order vector
persistent update copies whole collection
Bijection.inverse rebuilds
RangeMap lookup scans all intervals
RangeMap overlay does k independent tree searches
conversion deep-clones user objects
```

For RangeMap comparator-count scaling, use deterministic sizes such as:

```text
32
1024
32768
```

and validate logarithmic qualitative growth. Overlay evidence must be compatible with `log n + k`.

---

# 26. G6 — Performance gate

Pass only when all required asymptotic architectures have structural evidence and no hidden linear/full-copy fallback remains.

---

# 27. T11 — Residual resolution and ownership closure

## `R-PARSE-ASSOCIATED-INVOKE`

Mandatory discriminator:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

If failing, consume existing consultation and identify current owner. Because P5 explicitly handed this to P9, do not classify it external merely because main also fails.

**Current result (2026-10-18): RESOLVED for the P5 package path.** The
mandatory gate passes 4/4 Rust integration checks and 13/13 runtime fixtures.
The failure was missing contextual owner evidence for unqualified
`Some(...)` inside equality; explicit `Option<Token>::Some(...)` uses the
existing static variant-construction route. See the P5 closeout in the
checkpoint, P5 walkthrough, and P9 residual/verification ledgers. Dynamic
associated-family invocation remains a separate compiler follow-up outside
this package acceptance.

## `R-P7-NUMERIC-HASH-LAW`

Re-run exact observed-law cases:

```text
Int / equal integral Float > 2^53
LargeInt / equal integral Float
NaN payload equivalence
zero/-zero
ordinary equal Int/Float
```

End as `RESOLVED`, `EXTERNAL_BASELINE`, or `C1_OWNED`; never silent ignored state.

If externally blocked, final verification cannot truthfully be `RELEASE_COMPLETE` unless current normative policy explicitly permits the law failure.

## P4 D1 / D2 / D4

Re-run exact discriminators after current LANG007.C4 work. Do not preserve stale blockers by prose alone.

## P8 residuals

Consume every P8 handoff residual.

## New P9 residuals

For every new failure record:

```text
ID
command
minimal discriminator
main comparison
owner
classification
resolution
```

---

# 28. G7 — Residual closure gate

Pass criteria:

```text
R-PARSE-ASSOCIATED-INVOKE classified/acted on
R-P7-NUMERIC-HASH-LAW classified/acted on
D1/D2/D4 re-discovered
P8 residuals classified
all new residuals classified
no unresolved C1_OWNED
no AMBIGUOUS
```

---

# 29. T12 — Focused C1 cross-layer certification

Refresh actual test filters against the live repository and prove nonzero selection.

Mandatory categories:

```text
Array
Vector
Slice
Buffer
MutableBag
IndexedDict
OrderedSet
persistent hash
Bag
Bijection
P8 Interval
P8 persistent ordered
P8 RangeMap
Entry
conversions
collection contract
```

Also certify:

```text
runtime bare [] -> Array
static bare [] -> Vector
contextual List/Array/Vector
explicit heads
spreads
empty literals
const materialization
frozen const Array
Vector extent identity
cold/incremental parity
canonical Iterable conformance
for-loop consumption
associated Item equality
iterator exhaustion/freshness
strict zip
toList/toSet/toMap
parser package
native surface
GC/memory
negative diagnostics
```

Mechanically verify active collection docs contain no known stale assertions.

---

# 30. G8 — Focused C1 certification gate

Pass criteria:

```text
all P1–P8 focused acceptance suites remain green
P9 Entry tests pass
iteration convergence passes
conversion graph passes
diagnostics pass
property/GC/callback/performance evidence passes
parser-package status agrees with residual ledger
native-surface check passes
cold/incremental parity passes where applicable
no active normative-doc contradictions
```

---

# 31. T13 — Broad repository / CI parity certification

Immediately before running broad gates, re-read:

```text
scripts/verify.sh
.github/workflows/ci.yml
toolchain configuration
```

Current prepared-against broad gate:

```sh
scripts/verify.sh --full
```

At preparation time it covers:

```text
native-surface generator check
native sentinel drift check
workspace build
workspace nextest
workspace doctests
workspace clippy
```

Current CI additionally includes:

```text
nightly all-target build/test
VS Code extension E2E
rustfmt
warnings-denied clippy
Miri on phalcom-ast
```

Run the current equivalents, not blindly this frozen list.

Every broad failure enters the residual ledger. Do not delete tests, weaken warnings, or patch unrelated systems without ownership evidence.

---

# 32. G9 — Repository / CI gate

### Clean
All current required release/CI lanes green → may proceed toward `RELEASE_COMPLETE`.

### Baseline blocked
Allowed only when all C1-owned work is complete, focused C1 is green, every surviving broad failure is proven `EXTERNAL_BASELINE`, and no `AMBIGUOUS` residual exists.

---

# 33. T14 — Lifecycle closure

Finalize:

```text
API convergence ledger
migration ledger
property certification ledger
residual ledger
verification ledger
walkthrough
handoff
C1 checkpoint
PROGRAM
STATUS
```

The walkthrough must include:

```text
starting/ending revision
P8 takeover
API decisions
files changed
tests added
exact commands/results
native count before/after
property seeds
performance evidence
GC/callback evidence
spec/doc migrations
residual outcomes
deviations
known risks
final verification status
```

The handoff must state final stable C1 architecture, public APIs, iteration item contracts, conversion graph, constructor model, native floor, remaining external residuals, release status, and future work outside C1.

---

# 34. G10 — C1 closure gate

Close only if:

```text
all P9 artifacts exist
all API decisions implemented
all C1-owned focused gates pass
no contradictory public collection APIs
no active stale bare-List docs
no active SetList terminology
Entry modernized
iteration coherent
conversion graph coherent
diagnostics audited
property/GC/callback/performance evidence recorded
all residuals classified
no unresolved C1_OWNED
no AMBIGUOUS
broad status truthfully recorded
checkpoint/program/status aligned
```

---

# 35. Final status rules

## Release complete

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: RELEASE_COMPLETE
```

Only when focused C1 and all current required broad/CI lanes are green and no residual remains.

## Baseline blocked

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
```

Only when all C1 work is complete, focused C1 is green, surviving failures are proven external baseline, and no ambiguity remains.

Otherwise P9 stays open.

---

# 36. STOP / CONSULT triggers

STOP AND CONSULT if:

1. P8 handoff is missing or contradicts implementation.
2. P8 RangeMap public semantics remain unresolved.
3. Entry migration would silently change mapping iterator items.
4. association convergence requires a new generalized association trait.
5. conversion requires a new type-system/static-domain capability.
6. Vector conversion appears to require runtime/existential `N`.
7. constructor convergence collides with a newly landed construction architecture.
8. canonical Iterable conformance exposes an architectural semantic defect.
9. a new native primitive appears necessary merely for convenience.
10. property testing proves an earlier storage invariant false.
11. callback safety reveals a live heap borrow across user code.
12. P8 persistence cannot meet its promised complexity contract.
13. numeric hash-law repair requires unowned Number semantic changes.
14. parser-package repair requires unowned compiler architecture.
15. two current normative specs contradict one another.
16. one evidence-based attempted correction leaves a serious defect with unclear ownership.
17. release success would require weakening a test/warning/invariant.

A triggered consultation cannot be self-waived.

---

# 37. Expected source touch map

Likely:

```text
phalcom-core/core/universe/src/collections/entry.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/core/universe/src/collections/map.ph
phalcom-core/core/universe/src/collections/list.ph
phalcom-core/core/universe/src/collections/array.ph
phalcom-core/core/universe/src/collections/vector.ph
phalcom-core/core/universe/src/collections/slice.ph
phalcom-core/core/universe/src/collections/buffer.ph
phalcom-core/core/universe/src/collections/mutable_bag.ph
phalcom-core/core/universe/src/collections/indexed_dict.ph
phalcom-core/core/universe/src/collections/ordered_set.ph
phalcom-core/core/universe/src/collections/bag.ph
phalcom-core/core/universe/src/collections/bijection.ph
phalcom-core/core/universe/src/collections/iterator_laws.ph
P8 interval/range_map source files only for P9-owned convergence edits
```

Potential module registration:

```text
phalcom-modules/src/builtin.rs
```

Semantic/compiler/runtime/native files are touched only when a proven C1-owned defect requires them.

---

# 38. Expected test touch map

Likely additions/extensions:

```text
phalcom-core/tests/core/collections/entry.rs
phalcom-core/tests/core/collections/conversions.rs
phalcom-core/tests/core/collections/convergence.rs
family-specific seeded property suites
collections/contract.rs
mutable_bag.rs
indexed_dict.rs
ordered_set.rs
persistent_hash.rs
bag.rs
bijection.rs
P8 interval/range_map suites
memory/gc.rs
modules/parse_package.rs
```

Every new filter must prove nonzero test selection.

---

# 39. Expected spec/document touch map

```text
docs/specs/collection-families.md
docs/specs/collections/README.md
docs/specs/collections/list.md
docs/specs/collections/array.md
docs/specs/collections/vector.md
docs/specs/iterator.md
docs/specs/sequence-literals.md      # audit; edit only if needed
P8 interval/rangemap spec owners
docs/implementation/COLL006/PROGRAM.md
docs/implementation/COLL006/STATUS.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
```

Update `docs/specs/README.md` only if ownership/index entries change.

---

# 40. Evidence discipline

Every gate transition records:

```text
revision
working-tree status
exact command
number of selected tests
pass/fail count
ignored tests
failure IDs
residual classifications
```

Randomized evidence additionally records:

```text
seed
iteration count
model
operation distribution if relevant
```

Performance evidence records:

```text
input sizes
counter values
expected complexity relationship
observed relationship
```

GC evidence records:

```text
roots
expected-live objects
expected-collectible objects
forced-GC points
observed reachability
```

Do not substitute narrative confidence for executable evidence.

---

# 41. Recommended durable patch slices

```text
P9-A  T0/T1 rebaseline + decision freeze
P9-B  Entry + iterator convergence
P9-C  conversion graph + constructor convergence
P9-D  diagnostics + specs/docs migration
P9-E  property/GC/callback/performance certification
P9-F  residual repair/classification
P9-G  broad verification + lifecycle closure
```

Avoid mixing semantic compiler repairs, large docs migration, property framework work, and unrelated cleanup in one commit when separation is possible.

---

# 42. Slice review checklist

Before marking any task complete:

```text
Does this belong to P9?
Does it preserve the earlier owner?
Did it change public API?
Is the API ledger updated?
Did it change normative semantics?
Is the owning spec updated?
Did it add native surface?
Where is floor authority?
Did it alter Iterable::Item?
Where is migration approval?
Can failure publish partial state?
Does user callback run under a heap borrow?
Does the test prove semantics rather than implementation coincidence?
Did the command select nonzero tests?
Was current-main comparison required?
Was residual ownership recorded?
```

---

# 43. Completion checklist

## API
- [ ] canonical generic immutable Entry exists
- [ ] legacy Entry retired
- [ ] mapping iterator compatibility explicitly decided
- [ ] all C1 Iterable conformances complete
- [ ] conversion graph complete
- [ ] constructor convention converged
- [ ] RangeMap presentation integrated without domain redesign

## Semantics
- [ ] sequence-family semantics unchanged/current
- [ ] Range/Interval separation explicit
- [ ] conversion allocation/view semantics explicit
- [ ] failure atomicity proven
- [ ] equality/hash claims truthful

## Testing
- [ ] seeded reference models complete
- [ ] conversion properties complete
- [ ] GC convergence complete
- [ ] callback/re-entry convergence complete
- [ ] performance evidence complete
- [x] parser package rerun — 4/4 Rust integration checks; 13/13 runtime fixtures (2026-10-18)
- [ ] native surface rerun
- [ ] focused C1 gate green

## Residuals
- [x] `R-PARSE-ASSOCIATED-INVOKE` closed/classified for the P5 package path
- [ ] `R-P7-NUMERIC-HASH-LAW` closed/classified
- [ ] D1/D2/D4 re-discovered
- [ ] P8 residuals closed/classified
- [ ] no unresolved `C1_OWNED`
- [ ] no `AMBIGUOUS`

## Documentation/lifecycle
- [ ] active List/Array/Vector specs current
- [ ] no active SetList
- [ ] current lifecycle links valid
- [ ] STATUS/PROGRAM aligned
- [ ] API ledger complete
- [ ] migration ledger complete
- [ ] property ledger complete
- [ ] residual ledger complete
- [ ] verification ledger complete
- [ ] walkthrough complete
- [ ] handoff complete
- [ ] checkpoint truthful

## Release
- [ ] live `verify.sh --full` equivalent run
- [ ] current CI parity refreshed
- [ ] fmt checked
- [ ] warnings-denied clippy checked
- [ ] all-target build/test checked
- [ ] current E2E lane checked
- [ ] current Miri lane checked
- [ ] final verification status chosen from evidence

---

# 44. Executor summary

P9 should feel unlike P6–P8.

P6–P8 are primarily feature/storage plans. P9 is primarily a **convergence-and-proof plan**.

At every step ask:

```text
What cross-family contract remains inconsistent?
What evidence proves the intended contract?
What is the narrowest correct patch?
What gate closes when the evidence exists?
```

The target end state is:

```text
one public collection vocabulary
one iterator model
one association boundary
one conversion graph
one current constructor model
one coherent specification set
one reproducible property corpus
one GC/callback safety story
one performance story
one residual ledger
one truthful release state
```

Only then should `COLL006.C1` be closed.
