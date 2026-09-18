---
id: LANG005.C6.P5-AUDIT-2026-09-18
kind: implementation-audit
status: COMPLETE
audited_revision: b7a73454c742e64973f4729ab577d91a2166238a
repository: aureat/phalcom-lang
branch: main
audit_date: 2026-09-18
audit_disposition: REOPEN_REQUIRED
---

# LANG005.C6.P5 Full Implementation Audit

## Executive Summary

This audit reviewed the completed implementation of `LANG005.C6.P5` at repository revision
`b7a73454c742e64973f4729ab577d91a2166238a`
(`feat(lang005): complete P5 stateful iteration, cursor retirement, and stabilization`)
against the P5 requirements analysis, shared plan header, Part I–III patch-grade plans,
checkpoint, walkthrough, handoff, canonical iterator specification, implementation source,
tests, fixtures, current documentation, and retirement surfaces.

The implementation is **substantial and directionally correct**, but the current repository
state does **not** satisfy P5's own completion contract. The recorded
`COMPLETE / IMPLEMENTED / FOCUSED_TESTED` state is too strong.

The audit disposition is:

> **REOPEN P5. Do not treat G4, G5, or G6 as passed.**
>
> The implementation should be considered **PARTIAL with blocking correctness,
> semantic-authority, API-surface, and stabilization defects** until the findings in this
> report are resolved and the prescribed gates are rerun.

The most important positive result is that the central architecture has, in fact, moved in
the intended direction. `for` has a canonical semantic lane product; lowering carries
selected/requirement trait calls; the compiler no longer has the old cursor-based
`compile_for_lanes` compatibility lowering; stateful iterator objects exist for the core
collection families; the native runtime `Iterable` class is gone; and the new abstract
`Collection` hierarchy is real. These are meaningful, reusable advances.

However, the migration is internally inconsistent at several critical seams:

1. **Positional `*` expansion still falls back to ordinary dynamic `iterator` / `next`
   sends when semantic evidence is absent.** The semantic checker explicitly discards the
   failure reason, and tests intentionally rely on structural method coincidence. This is
   the exact architecture G5 required P5 to delete.
2. **Runtime trait-environment root collection handles `for_iteration_lanes` but not
   `positional_expansions`.** Lowering knows how to attach expansion environments, but the
   compiler does not collect the roots needed to populate them.
3. **The lazy iterator toolbox is behaviorally implemented but not semantically typed as
   planned.** Core adapter constraints were omitted or erased to `Dynamic`; most notably,
   `ChainIterator<I,J>` advertises `Item = I::Item` while returning `J.next` without
   constraining `J::Item == I::Item`.
4. **T10's required eager collection surface was not implemented.** Direct `map`,
   `map(indexed:)`, `filter`, `flatMap`, and `isEmpty` preservation is absent from the
   reusable collection classes. Active fixtures still expect it.
5. **Tuple/`Dynamic` closure is incomplete.** P5 added a global semantic representation for
   dynamic associated-type bindings, but the canonical implicit-iteration resolver requires
   associated projections to produce a `TypeId`. A `Dynamic` projection deliberately has no
   `TypeId`. Therefore `Tuple::Item = Dynamic` (and `RefRange::Item = Dynamic`) does not form
   a usable canonical iteration lane through the current resolver.
6. **Single-lane `for` leaks one `None` operand on natural exhaustion.** The compiler
   duplicates the `Option`, branches by popping one copy, and never removes the remaining
   `None`. `end_scope` explicitly does not clean arbitrary evaluation-stack values.
7. **The active language corpus was not migrated and was omitted from final stabilization.**
   Live PASS fixtures still call `iterate` / `iteratorValue`, assert repeatable lazy views,
   and expect direct eager collection transforms that no longer exist.
8. **Current normative docs, guides, and production comments still teach the retired cursor
   protocol.** The committed handoff's statement that current documentation was synchronized
   is factually incorrect.
9. **The final walkthrough/handoff do not provide the coverage/accounting required by the
   P5 plans.** They use `revision: local-head`, contain no `P5-CV-*` closure matrix, and claim
   zero residue despite current-facing repository hits.

These are not cosmetic deviations. Several directly violate fixed P5 invariants and gate
conditions.

---

# 1. Audit Scope and Method

## 1.1 Audited revision

Repository:

```text
aureat/phalcom-lang
branch: main
revision: b7a73454c742e64973f4729ab577d91a2166238a
subject: feat(lang005): complete P5 stateful iteration, cursor retirement, and stabilization
```

The final commit is unusually broad: GitHub reports **154 changed files**, of which roughly
32 are source/example files and 122 are documentation/agent files. The commit therefore
mixes P5 implementation with substantial documentation movement/churn. That does not make
the P5 implementation incorrect, but it makes causality and review substantially harder.

## 1.2 Authorities reviewed

The audit reconciled the implementation against the following planning/specification
authorities:

```text
docs/implementation/LANG005/LANG005.C6/
  LANG005.C6-CHECKPOINT.md
  LANG005.C6.P5-requirements-analysis.md
  LANG005.C6.P5-plan-00-shared-header.md
  LANG005.C6.P5-plan-01-semantic-and-for-cutover.md
  LANG005.C6.P5-plan-02-builtins-and-iterator-surface.md
  LANG005.C6.P5-plan-03-retirement-and-stabilization.md
  LANG005.C6.P5-walkthrough.md
  LANG005.C6.P5-handoff.md

docs/specs/iterator.md
docs/specs/callables/arguments.md
docs/specs/associated-types.md
docs/specs/generic-constraints.md
docs/specs/objects/traits.md
```

The implementation audit covered:

- semantic conformance/projection authority;
- semantic `for` lane publication;
- positional expansion semantic publication;
- semantic fingerprints/dependencies;
- semantic-to-compiler lowering;
- runtime trait environment transport;
- compiler `for` lowering;
- positional expansion compiler/runtime paths;
- VM iteration-adjacent bytecodes;
- core Universe collection declarations;
- stateful iterator implementations;
- adapter/default APIs;
- Map/Range/Tuple/String/Bytes specifics;
- `Collection` native/runtime hierarchy;
- package/public-name cutover;
- Sheetcalc `RefRange`;
- boundedness;
- focused tests;
- registered language corpus fixtures;
- LSP/current docs/source comments;
- checkpoint/walkthrough/handoff completion records.

## 1.3 Verification limitation

This audit had repository source and GitHub history/status access, but did **not** execute
the Rust test suite locally.

At the final revision, GitHub reports no combined commit statuses and no workflow runs.
Therefore the test counts in `LANG005.C6.P5-walkthrough.md`,
`LANG005.C6.P5-handoff.md`, and `LANG005.C6-CHECKPOINT.md` are treated as
**committed implementation records**, not independently reproduced CI evidence.

This limitation does not weaken the principal findings below. Several are direct,
deterministic contradictions visible in the committed source and active fixtures.

---

# 2. Overall Disposition

## 2.1 Recommended checkpoint state

Current recorded state:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

Recommended state after this audit:

```text
IN_PROGRESS / PARTIAL
audit: REOPEN_REQUIRED
G4: NOT ACCEPTED
G5: NOT ACCEPTED
G6: NOT ACCEPTED
```

P6 certification should not treat P5 as a closed predecessor until the blocking findings
are repaired.

## 2.2 Severity summary

| ID | Severity | Finding | Gate impact |
|---|---|---|---|
| `P5-AUD-001` | **BLOCKER** | positional `*` still has dynamic structural iteration fallback | T12, G5, G6 |
| `P5-AUD-002` | **HIGH** | expansion runtime trait-environment roots are not collected | T12, G5 |
| `P5-AUD-003` | **HIGH / SOUNDNESS** | iterator adapter generic contracts are erased/unconstrained | T9, G4, G6 |
| `P5-AUD-004` | **HIGH / API REGRESSION** | required eager collection forwarders and `isEmpty` are missing | T10, G4, G6 |
| `P5-AUD-005` | **BLOCKER / ARCHITECTURAL** | `Dynamic` associated Items cannot be consumed by canonical iteration resolver | T6/T13, G3/G5/G6 |
| `P5-AUD-006` | **HIGH** | single-lane `for` leaks one stack operand at natural exhaustion | T4, G2/G6 |
| `P5-AUD-007` | **HIGH / VERIFICATION** | active cursor/repeatability fixtures remain and required corpus gates were omitted | G4, T15, G6 |
| `P5-AUD-008` | **HIGH / STABILIZATION** | current normative docs/guides/source comments still describe cursor iteration | T15, G6 |
| `P5-AUD-009` | **MEDIUM-HIGH** | Tuple zipped traversal reintroduces structural `iterator`/`next` behavior | T9/T13, G5 |
| `P5-AUD-010` | **MEDIUM** | inline semantic support is a hard-coded module-category shadow prelude | T12/T13 maintainability |
| `P5-AUD-011` | **MEDIUM / RISK** | missing runtime generic bindings silently degrade to `Dynamic` and can mask transport bugs | cross-cutting |
| `P5-AUD-012` | **MEDIUM / RECORDS** | final completion records do not meet P5 coverage/census requirements | T15, G6 |
| `P5-AUD-013` | **LOW-MEDIUM / HYGIENE** | final P5 commit is review-hostile in scope and has no external CI evidence | verification quality |

---

# 3. What P5 Implemented Correctly

The audit found significant implementation quality in the core migration. These pieces
should be preserved while remediating the failures.

## 3.1 Canonical `for` semantic ownership is real

The implementation correctly moved `for` meaning into semantic products rather than
letting the compiler discover iterability by selector shape.

The current pipeline is structurally:

```text
phalcom-semantic
    resolve canonical Iterable proof
    normalize Item / Iterator projections
    prove Iterator on projected iterator type
    publish ForIterationLaneResolution
        ↓
phalcom-core semantic_lowering
    project to ForIterationLoweringSpec
        ↓
compiler
    emit selected-trait or requirement-slot invocation
        ↓
VM
    execute frozen runtime recipes
```

This is exactly the right authority direction for C6.

## 3.2 `compile_for` now fails closed when lane metadata is missing

The legacy compatibility branch has been removed. Current compiler control flow gathers
all lane specs and rejects a missing canonical projection instead of silently compiling a
cursor protocol path.

That is a major improvement over the pre-Part-III state and satisfies the architectural
intent of T13-A.

Reference:

```text
phalcom-core/src/compiler/lib/loops.rs
```

The old `compile_for_lanes` function is no longer present.

## 3.3 Multi-lane lowering preserves the important control-flow invariants

`compile_canonical_for_lanes` evaluates all source expressions first, left-to-right,
stores them, and only then acquires iterators. It then advances every iterator exactly once
before examining lane exhaustion. This preserves the required two-phase preheader and
strict lockstep behavior.

The strict mismatch path raises `ZipLengthMismatchError` rather than truncating to the
shortest lane.

## 3.4 Core stateful iterators exist

The migration has real stateful iterator objects for:

- `List<T>`;
- `Set<T>`;
- `Map<K,V>`;
- Map keys/values/entries views;
- `Range`;
- `Bytes`;
- `StringByteSequence`;
- `StringCodePointSequence`;
- `Tuple`;
- Sheetcalc `RefRange`.

The ordinary source objects retain traversal state through ordinary traced fields rather
than introducing special VM heap arms.

## 3.5 Map canonical Item is the intended labeled tuple

The direct Map iterator and entries view use:

```text
(key: K, value: V)
```

rather than retaining the old key-only cursor payload. This aligns the production path
with the accepted P5 model.

## 3.6 Runtime native `Iterable` identity is actually retired

The implementation correctly moved the runtime taxonomy to:

```text
Object
└── Collection (abstract)
    ├── List
    ├── Map
    ├── Set
    ├── Tuple
    ├── Range
    └── Bytes
```

`Collection` itself carries no iteration semantics. `UniverseKey::Iterable` and the old
native/runtime `Iterable` class are removed.

This is a successful T14 outcome and should not be reverted.

## 3.7 VM cursor protocol probing was removed

The old VM `has_iteration_protocol` structural lookup helper is gone.

The remaining `PackTryExpandTuplePositionals` / `ListTryExpandTuplePositionals` runtime
fast path is now a pure Unit/Tuple product-lane probe. That is the correct VM boundary.

## 3.8 Boundedness remains compiler-owned and conservative

`phalcom-core/src/compiler/boundedness.rs` recognizes `.iter` / `.iterator`, lazy adapter
selectors, finite `take`, and conservative Unknown behavior without becoming a trait solver.
The overall ownership is correct.

---

# 4. Blocking Findings

## P5-AUD-001 — Positional expansion still establishes iterability structurally

**Severity:** BLOCKER  
**Owners:** T12, G5, G6  
**Directly violates:** Part III terminal condition, G5 requirements 3/5/6, no-runtime-
structural-proof invariant.

### 4.1.1 Compiler evidence

Current code:

[`phalcom-core/src/compiler/lib/expr.rs`, lines 93–150](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/src/compiler/lib/expr.rs#L93-L150)

```rust
let spec = self
    .lowering()
    .and_then(|l| l.positional_expansions.get(&expr_range))
    .cloned();

self.emit(Bytecode::GetLocal(source_slot), range);
if let Some(spec) = &spec {
    self.emit_for_iteration_requirement(&spec.iterator, range)?;
} else {
    self.emit_getter_send("iterator", range);
}

...

if let Some(spec) = &spec {
    self.emit_for_iteration_requirement(&spec.next, range)?;
} else {
    self.emit_getter_send("next", range);
}
```

The Part III plan explicitly required:

```text
else dedicated PositionalExpansionLoweringSpec required
...
No ordinary dynamic `iterator` or `next` send is a conformance fallback.
```

The implementation does the opposite whenever the lowering product is missing.

### 4.1.2 Semantic failure is silently discarded

Call argument expansion:

[`phalcom-semantic/src/checker/call.rs`, lines 301–312](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-semantic/src/checker/call.rs#L301-L312)

```rust
if let ApplicationArgument::Expansion { range, .. } = argument {
    if let TypeKnowledge::Known(evidence) = &typed.knowledge {
        let (resolution, _) =
            crate::checker::iteration::resolve_positional_expansion(...);

        if let Some(resolution) = resolution {
            ctx.positional_expansions.insert(...);
        }
    }
}
```

The resolver itself explicitly returns the reason:

[`phalcom-semantic/src/checker/iteration.rs`, lines 320–343](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-semantic/src/checker/iteration.rs#L320-L343)

```rust
match resolve_implicit_iteration(...) {
    Ok(resolved) => (Some(resolution), None),
    Err(reason) => (None, Some(reason)),
}
```

but the caller discards it with `_`.

List and Tuple literal spread analysis does the same thing:

[`phalcom-semantic/src/checker/expression.rs`, lines 1740–1755](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-semantic/src/checker/expression.rs#L1740-L1755)

```rust
let (resolution, _) = resolve_positional_expansion(...);
...
} else if let Some(form) = list_form {
    projected = project_applied_argument(...);
}
```

Therefore the compiler fallback is not dead defensive code. The semantic layer can
deliberately publish no canonical expansion product and still let compilation proceed.

### 4.1.3 Current tests prove the structural path is intentionally exercised

`phalcom-core/tests/core/collections/outgoing_packs.rs` defines:

```phalcom
class CountdownIterator {
  next { ... }
  iterator { self }
}

class Countdown {
  iterator { CountdownIterator.new(_n) }
}

receiver.collect(*Countdown.from(n: 2))
```

There is no `impl Iterator` and no `impl Iterable`. The test nevertheless expects success.

The same file tests:

```phalcom
class Broken {
  iterator { throw Error.new("iterate boom") }
}
Receiver.new().collect(*Broken.new())
```

and expects the runtime `iterator` body to execute.

`outgoing_packs_completion.rs` goes further and explicitly pins the wrong failure mode:

```rust
Receiver.new().collect(*42)
```

must produce:

```text
does not understand 'iterator'
```

That is runtime structural lookup, not semantic conformance rejection.

### 4.1.4 Consequence

P5 currently has two different meanings of “iterable”:

```text
for:
    canonical Iterable evidence required

* expansion:
    canonical evidence when available
    otherwise selector coincidence at runtime
```

That violates the central P5 migration objective.

### 4.1.5 Required repair

1. Make failed canonical expansion resolution produce a semantic diagnostic or an explicit
   invalid expansion product.
2. After the Unit/Tuple direct lane, make `PositionalExpansionLoweringSpec` mandatory.
3. Delete both `else { emit_getter_send(...) }` branches.
4. Rewrite structural-positive tests with real canonical `impl Iterator` / `impl Iterable`.
5. Add a negative lookalike test:
   a class with `iterator`/`next` methods but no conformance must be rejected before runtime.
6. Preserve runtime exceptions from a **canonically conforming** custom iterator as the
   positive error-propagation test.

---

## P5-AUD-002 — Positional expansion runtime trait environments are not rooted

**Severity:** HIGH  
**Owner:** T12 / G5  
**Risk:** exact/derived generic expansion can lose required impl bindings.

`semantic_lowering.rs` clearly intends expansion and `for` to share the same runtime
environment attachment:

[`phalcom-core/src/modules/semantic_lowering.rs`, lines 503–532](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/src/modules/semantic_lowering.rs#L503-L532)

```rust
for spec in lowering.for_iteration_lanes.values_mut() {
    attach_requirement(&mut spec.iterator, range, environments);
    attach_requirement(&mut spec.next, range, environments);
}

for spec in lowering.positional_expansions.values_mut() {
    attach_requirement(&mut spec.iterator, range, environments);
    attach_requirement(&mut spec.next, range, environments);
}
```

However, `ProgramCompiler::project_analyzed` gathers runtime trait roots only from ordinary
expression trait dispatches and `analysis.for_iteration_lanes`:

[`phalcom-core/src/modules/compile.rs`, lines 670–730](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/src/modules/compile.rs#L670-L730)

There is no corresponding loop over:

```rust
analysis.positional_expansions.values()
```

in the root-gathering phase.

### Consequence

The lowering spec may correctly contain a `Selected(TraitInvocationSpec)` for expansion,
but the runtime type bindings needed by that selected conformance may never be exported
into the runtime root table. The later attachment phase cannot attach data that was never
collected.

The current test suite does not discriminate this because structural fallback remains
available and the focused spread tests do not establish the required exact/derived generic
environment cases.

### Required repair

Factor the root-collection logic for implicit iteration requirements into a shared helper and
invoke it for both:

```text
analysis.for_iteration_lanes
analysis.positional_expansions
```

Then add runtime expansion tests for:

- exact generic conformance requiring owner/impl bindings;
- assumed `T |= Iterable`;
- derived `T |= Iterator => Iterable`;
- conditional/derived conformance with nested evidence if supported by the current P4/P5
  runtime recipe model.

---

## P5-AUD-003 — Iterator adapter type contracts are erased or underconstrained

**Severity:** HIGH / SOUNDNESS  
**Owner:** T9 / G4 / G6

The Part II plan was explicit:

> “do not erase constraints to Dynamic to avoid stating associated projections.”

It further fixed these semantic requirements:

```text
upstream type conforms to Iterator
mapper/predicate signatures are typed from Iterator::Item
filterMap mapper returns Option<U>
flatMap mapper returns S where S |= Iterable
flatten requires current Item |= Iterable
chain second iterator Item == Self::Item
zip output preserves both item types
mapWhile mapper returns Option<U>
```

The final source does not implement those contracts.

### 4.3.1 Iterator defaults are dynamically typed

[`iterator_laws.ph`, lines 3–22](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/core/universe/src/collections/iterator_laws.ph#L3-L22)

```phalcom
trait Iterator {
  type Item

  next -> Option<Self::Item>

  map(_ f) { MappedIterator.new(self, f) }
  filter(_ pred) { FilterIterator.new(self, pred) }
  filterMap(_ f) { FilterMapIterator.new(self, f) }
  flatMap(_ f) { FlatMapIterator.new(self, f) }
  flatten { FlattenIterator.new(self) }
  chain(_ other) { ChainIterator.new(self, other) }
  ...
  zip(_ other) { StrictZipIterator.new(self, other) }
}
```

No callback signatures, method generics, result types, or conformance/equality constraints
are present.

### 4.3.2 `flatMap`, `flatten`, zip and indexing explicitly erase Item to Dynamic

[`iterator_laws.ph`, lines 445–524](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/core/universe/src/collections/iterator_laws.ph#L445-L524)

```phalcom
impl<I, S> Iterator for FlatMapIterator<I, S> {
  type Item = Dynamic
}

impl<I> Iterator for FlattenIterator<I> {
  type Item = Dynamic
}
```

[`iterator_laws.ph`, lines 780–846](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/core/universe/src/collections/iterator_laws.ph#L780-L846)

```phalcom
impl<I, J> Iterator for StrictZipIterator<I, J> {
  type Item = Dynamic
}

impl<I> Iterator for IndexedIterator<I> {
  type Item = Dynamic
}
```

### 4.3.3 `ChainIterator` is materially unsound against its own published Item contract

[`iterator_laws.ph`, lines 526–559](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/core/universe/src/collections/iterator_laws.ph#L526-L559)

```phalcom
class ChainIterator<I, J> {
  ...
  next {
    ...
    if (_phase == 1) {
      let step = _second.next
      if (step.isSome) { return step }
      ...
    }
  }
}

impl<I, J> Iterator for ChainIterator<I, J> {
  type Item = <I as Iterator>::Item
}
```

There is no constraint:

```text
J |= Iterator
<J as Iterator>::Item == <I as Iterator>::Item
```

Yet the implementation returns `J.next` directly under a conformance that claims the
result is `Option<I::Item>`.

This is more than lost inference precision. It lets an adapter's declared semantic contract
diverge from the values it can actually yield.

### Required repair

Implement the Part II generic surface as designed. If current syntax requires explicit Item
parameters, use them; the plan explicitly allows that mechanical representation.

At minimum, add semantic tests that prove:

```text
Iterator<Int>.map(String mapper) -> Iterator<String>
filter preserves Item
filterMap changes to U
chain(Int, Int) accepted
chain(Int, String) rejected
flatten requires Item |= Iterable
flatMap mapper result |= Iterable
zip produces a pair of both Items
indexed produces (Int, Item)
```

A single difficult method may be isolated and escalated. The entire toolbox must not remain
Dynamic as a workaround.

---

## P5-AUD-004 — Required eager collection transforms and `isEmpty` were never restored

**Severity:** HIGH / USER-FACING API REGRESSION  
**Owner:** T10 / G4 / G6

Part II fixed the intended role split:

```text
List.map(f)       -> eager collection transform
List.iter.map(f)  -> lazy iterator adapter
```

and required the reusable collection families to keep inherent:

```text
map
map(indexed:)
filter
flatMap
isEmpty
```

The final implementation does not contain those methods in `List`, `Set`, `Map`, `Tuple`,
`Bytes`, or `Range`.

Repository search under:

```text
phalcom-core/core/universe/src/collections/
```

finds no collection-owned `isEmpty`, `map(indexed:)`, `filter(_)`, or `flatMap(_)`; the
transform selectors exist only as lazy defaults on `Iterator`.

`Iterable` intentionally does not own them, because every Iterator derives Iterable and
the selector overlap would be ambiguous.

Therefore a collection such as `List` now has no direct `map` implementation.

### Active fixture proving the intended API

`phalcom-core/tests/fixtures/language/iterator/iterator_direct_map_still_eager.ph` is still
marked PASS:

```phalcom
let calls = 0
let result = [1, 2, 3].map |x| { calls = calls + 1; x * 2 }
System.print(calls)
System.print(result.is(List))
System.print(result.at(2))
```

The final focused gate did not run the `language-corpus iterator` lane that contains this
fixture.

### Required repair

Implement the T10 forwarding wrappers, for example:

```phalcom
map(_ f) {
  self.iterator.map(f).toList
}

filter(_ p) {
  self.iterator.filter(p).toList
}

flatMap(_ f) {
  self.iterator.flatMap(f).toList
}

isEmpty { self.size == 0 }
```

with the exact accepted `map(indexed:)` behavior and the Range-specific legacy semantics
described by the plan.

Then run the exact T10/G4 language-corpus commands before claiming G4.

---

## P5-AUD-005 — Tuple/RefRange `Dynamic` Item cannot publish a usable canonical lane

**Severity:** BLOCKER / ARCHITECTURAL  
**Owner:** T6/T13 / G3/G5/G6

The final implementation broadened the semantic associated-type model to represent an
explicit Dynamic associated binding:

[`phalcom-semantic/src/impls.rs`, around lines 472–480](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-semantic/src/impls.rs#L472-L480)

```rust
pub enum AssociatedTypeBindingValue {
    Type(TypeId),
    /// An explicit `Dynamic` associated-type binding. This is a runtime
    /// boundary, not a canonical type or proof-producing semantic fact.
    Dynamic,
}
```

This is a meaningful cross-cutting semantic change, and it was the exact issue the Part III
plan identified as a mandatory consultation seam.

The problem is that canonical iteration still requires a concrete `TypeId`.

`resolve_implicit_iteration` does:

```rust
let Some(item_type) = normalize_projection(ctx, item_projection).type_id() else {
    return Err(UnresolvedApplicationReason::DispatchMissing);
};
```

`ProjectionNormalizationResult::type_id()` returns a type only for:

```rust
Normalized(TypeId)
Symbolic(TypeId)
```

and returns `None` for `Dynamic`.

Reference:

[`phalcom-semantic/src/types/projection.rs`, lines 109–115](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-semantic/src/types/projection.rs#L109-L115)

```rust
pub fn type_id(&self) -> Option<TypeId> {
    match self {
        Self::Normalized(ty) | Self::Symbolic(ty) => Some(*ty),
        _ => None,
    }
}
```

But final Tuple declares:

```phalcom
impl CanonicalIterable for Tuple {
  type Item = Dynamic
  type Iterator = TupleIterator
}
```

and Sheetcalc `RefRange` also uses `Item = Dynamic`.

### Consequence

The declaration can exist as a conformance, but the common canonical iteration resolver
cannot turn its Item projection into `ForIterationLaneResolution.item_type`.

Thus the implementation has not closed the Tuple/Dynamic vertical end-to-end.

No focused test found in the current repository directly proves:

```phalcom
for x in someTuple { ... }
```

through a published canonical lane.

### Architectural significance

The plan explicitly said not to “force `Dynamic` through static proof machinery” and to
consult if `Item = Dynamic` remained incompatible with canonical proof publication.

Instead, P5 expanded the associated-type representation globally but did not adapt the
iteration product itself to represent a dynamic Item.

That is an incomplete cross-layer change.

### Required decision

Choose and ratify one coherent model:

**Option A — iteration product can carry Dynamic knowledge**

Change the iteration semantic product so Item is represented by `TypeKnowledge` /
normalized term state rather than an unconditional `TypeId`, while keeping Iterator type
resolution/proof precise enough to select executable requirements.

**Option B — Tuple gets a real semantic Item type**

If the type system has an accepted proper type representing the heterogeneous tuple item
surface, use it rather than Dynamic.

**Option C — Tuple is not yet a canonical Iterable**

If neither model is ratified, remove the false completion claim and defer Tuple canonical
for-iteration explicitly.

For `RefRange`, the likely better solution is to publish its actual `Ref` element type
rather than `Dynamic`.

This should be treated as an architectural consultation/remediation item, not patched by a
compiler special case.

---

## P5-AUD-006 — Single-lane `for` leaks one operand on natural exhaustion

**Severity:** HIGH  
**Owner:** T4 / G2/G6  
**Type:** compiler stack-discipline bug.

Current single-lane lowering:

[`phalcom-core/src/compiler/lib/loops.rs`, lines 138–169](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/src/compiler/lib/loops.rs#L138-L169)

```rust
self.emit(Bytecode::GetLocal(iterator_slot as u16), range);
self.emit_for_iteration_requirement(&spec.next, range)?;
self.emit(Bytecode::Dup, range);
let exit_on_none = self.emit_forward_jump(Bytecode::JumpIfNone, range);
self.emit(Bytecode::GetVariantPayload(0), range);
...
self.emit_backward_loop(loop_start, range);

let exit_label = self.chunk_len();
self.patch_forward_jump_to(exit_on_none, exit_label);
...
self.end_scope(range);
```

`JumpIfNone` pops only the duplicate. At natural exhaustion:

```text
stack before Dup:        [..., None]
after Dup:               [..., None, None]
JumpIfNone pops one:     [..., None]
branch to exit:          [..., None]
```

The remaining `None` is never popped.

This is not implicitly cleaned by lexical scope teardown. `end_scope` explicitly says:

[`phalcom-core/src/compiler/lib/scope.rs`, lines 195–221](https://github.com/aureat/phalcom-lang/blob/b7a73454c742e64973f4729ab577d91a2166238a/phalcom-core/src/compiler/lib/scope.rs#L195-L221)

> “We therefore never `Pop` locals here…”

and only emits `CloseUpvalue` where needed.

The positional expansion loop uses the same duplicate/branch shape but correctly emits:

```rust
self.patch_forward_jump(exit);
self.emit(Bytecode::Pop, range);
```

### Impact

One naturally completed single-lane `for` leaves one operand in the active frame.

A loop that repeatedly executes an inner `for` can grow the VM stack linearly for the
lifetime of that method/fiber activation.

### Correct shape

Use distinct natural-exhaustion and common-exit labels:

```text
next
dup
JumpIfNone natural_exit
extract Some
...
loop

natural_exit:
    Pop        # remove retained original None

common_exit:
    ...
```

`break` jumps should target `common_exit`, not `natural_exit`, because a break path does
not own a retained exhaustion sentinel.

Add a regression that repeatedly runs a naturally exhausted single-lane `for` inside an
outer loop in one activation.

---

# 5. Major Stabilization and Coverage Findings

## P5-AUD-007 — Active corpus still encodes superseded cursor/repeatability semantics

**Severity:** HIGH / VERIFICATION  
**Owners:** T10/T13/T15/G4/G6

The Part II plan explicitly said:

> A legacy test that encodes cursor/stateless-pipeline semantics is **not** a reason to
> preserve those semantics. Migrate the fixture when P5's normative behavior changed.

It specifically required `language-corpus` iterator, sequence, list, and collections runs.

The final walkthrough instead records only the new focused suites.

### Registered corpus source is still cursor-era

`phalcom-core/tests/core/language/corpus.rs` still documents and executes:

```rust
support::check_pass("iteration");
support::check_pass("sequence");
support::check_pass("iterator");
```

with comments describing `iterate(_)` / `iteratorValue(_)`.

### Active PASS fixture: Range cursor protocol

`collections/range_cursor_protocol_direct.ph` still contains:

```phalcom
System.print(r.iterate(None))
System.print(r.iterate(1))
...
System.print(r.iteratorValue(1))
...
```

Those methods were removed from `Range`.

### Active PASS fixture: Map cursor protocol

`collections/map_wren_cursor_roundtrip.ph` directly calls:

```phalcom
let c = m.iterate(None)
System.print(m.iteratorValue(c))
...
cursor = m2.iterate(cursor)
```

Those methods were removed from `Map`.

### Active PASS fixture: repeatable lazy view

`sequence/sequence_takeview_repeatable.ph` states:

```text
Iterator take can be traversed repeatedly with identical results
```

and traverses the same stored `source.iter.take(3)` twice.

That is the opposite of P5's one-shot stored-pipeline invariant.

### Active PASS fixture: direct eager map

`iterator/iterator_direct_map_still_eager.ph` expects direct List `map`, which the final
source does not provide.

### Consequence

The repository still contains registered tests that describe the old architecture and/or
the API the final implementation failed to preserve.

The fact that these were not included in the final reported gate means G4/G6 did not
exercise a directly affected test surface required by the plan.

### Required repair

Migrate or replace the fixtures, then run at least:

```sh
cargo test -p phalcom-core --test core language::golden   # or actual current corpus registration/filter
```

using the repository's current test registration, plus the plan-prescribed iterator,
sequence, list, collections, bytes, and strings slices.

Do not delete coverage merely to make the gate green; convert old cursor tests into
canonical stateful equivalents.

---

## P5-AUD-008 — Current normative docs and production comments still teach cursor iteration

**Severity:** HIGH / STABILIZATION  
**Owner:** T15 / G6

The handoff claims:

```text
LSP hover blurbs, current documentation, and language design overlay synchronized.
```

That is not true repository-wide.

### Current normative iteration spec is still entirely cursor-based

`docs/spec/current/iteration.md` begins:

```text
Status: Normative
```

and states:

```text
Every form of iteration ... bottoms out in one two-selector cursor protocol.
A value is iterable iff it implements that protocol.
```

It defines:

```text
iterate(_)
iteratorValue(_)
```

says no iterator object is allocated, and presents the cursor `for` desugar.

This directly contradicts `docs/specs/iterator.md` and the P5 implementation target.

### Current user guides are stale

`docs/guide/collections.md` says:

```text
Define these two methods and every combinator ... works on your type for free
```

with `iterate` / `iteratorValue`.

`docs/guide/control-flow.md` says `for` lowers through the cursor protocol.

### Spread specification is stale

`docs/spec/collections/06-rest-spread-and-pack-operators.md` still says:

```text
*   Tuple/Unit direct positional lane; otherwise Iterable cursor protocol
    iterate(_) + iteratorValue(_)
```

and:

```text
Generic `*` is compiled as ordinary cursor sends
```

### Production source comments are stale

`phalcom-ast/src/ast.rs` describes the AST `For` node and `ForStatement` as an inlined
cursor loop using `iterate` / `iteratorValue`.

`phalcom-core/src/bytecode.rs` documents `JumpIfNone` as the cursor-protocol
end-of-iteration branch.

`phalcom-core/src/vm/dispatch.rs` still names the popped value `cursor` and repeats the
cursor-protocol comment.

### Consequence

P5 leaves two contradictory current authorities:

```text
docs/specs/iterator.md             canonical Iterator/Iterable stateful model
docs/spec/current/iteration.md     normative cursor model
```

That is unacceptable for a completed language migration.

### Required repair

Update all current-facing and production-maintainer surfaces. Preserve old ADRs and
historical implementation documents as history; do not rewrite them as if the old decision
never existed.

---

## P5-AUD-009 — `Tuple.zipped` reintroduces structural iterator behavior internally

**Severity:** MEDIUM-HIGH / ARCHITECTURAL  
**Owner:** T9/T13/G5

`TupleZipIterator` stores:

```phalcom
_iterators: List<Dynamic>
```

and constructs each lane by ordinary dispatch:

```phalcom
_iterators._$push(tuple.at(i).iterator)
```

Then it advances each lane with ordinary:

```phalcom
_iterators.at(i).next
```

There is no semantic evidence product proving that each heterogeneous tuple element
conforms to `Iterable` / `Iterator`.

Therefore zipped traversal has a structural/dynamic concept of iterability even though the
P5 terminal invariant says method coincidence does not establish iterability.

This is a difficult heterogeneous-product problem, but that is precisely why it needs an
explicit semantic policy rather than an untracked Dynamic escape.

### Required action

Clarify and ratify one of:

1. `Tuple.zipped` is deliberately dynamic and may fail at runtime on non-iterable lanes;
2. heterogeneous tuple shape analysis produces per-element iterable evidence;
3. zipped accepts a narrower statically representable tuple domain.

Whichever policy is chosen must be reflected in `docs/specs/iterator.md` and tests.

---

# 6. Architectural Risks and Medium Findings

## P5-AUD-010 — Inline implicit-Universe semantic support is a hard-coded shadow prelude

Current inline compilation loads every Universe node matching:

```rust
["collections", ..] | ["scalar", "string"]
```

Reference:

```text
phalcom-core/src/modules/compile.rs
EntrySelection::Inline
```

This repaired the immediate List/Bytes/Map/Set conformance visibility failure, but it is
fragile:

- a future canonical iterable source outside those module categories will not be visible;
- a new unrelated semantic error in any included collections module can affect every inline
  snippet;
- the set duplicates implicit-Universe knowledge in compiler code;
- standalone/project compilation and inline compilation can drift again.

The Part III plan permitted a curated support set, so this is not by itself a violation.
It is nevertheless a maintainability hazard.

A better follow-on is to define one canonical “implicit Universe semantic support closure”
owned by the module/semantic layer rather than path-pattern logic in the compiler entry
selection.

---

## P5-AUD-011 — Missing runtime generic bindings silently degrade to Dynamic

Current runtime type instantiation contains:

```rust
TypeNode::Parameter(param) => Some(env.get(param).unwrap_or_else(|| {
    context.overlay.type_ref(RuntimeOverlayTypeNode::Special("Dynamic".into()))
}))
```

The comment calls this the “correct erasure fallback.”

This behavior was inherited into Part III, so this audit does not classify it as an
unambiguous P5 task violation. It is still a significant architectural risk.

Why it matters here:

- P5 already has a concrete missing-root problem for expansion evidence (`P5-AUD-002`);
- silently turning an absent binding into Dynamic can convert a transport/invariant defect
  into apparently successful execution;
- earlier C6 records explicitly emphasized that missing type environments should not be
  repaired by runtime inference.

Recommended follow-on: distinguish a *ratified explicit erasure boundary* from an
*unexpected missing binding*. The latter should remain diagnosable as an invariant failure.

---

## P5-AUD-012 — Completion records are materially insufficient

**Severity:** MEDIUM / RECORD INTEGRITY  
**Owner:** T15/G6

The final walkthrough is only 68 lines and the handoff 61 lines.

Both use:

```text
revision: local-head
```

instead of the actual committed revision.

Neither contains the required `P5-CV-*` coverage closure matrix.

The Part II plan alone assigns specific rows such as:

```text
P5-CV-75  direct built-in map stays eager
P5-CV-102 List.map eager vs List.iter.map lazy
```

yet the final records do not state a disposition for them.

They also claim:

```text
Census: 0 matches
```

while current-facing docs, AST comments, language corpus comments, and active fixtures
contain many cursor references.

The records therefore fail as durable evidence even apart from code defects.

### Required repair

A corrected walkthrough should include:

- exact revision SHA;
- task-by-task final disposition;
- G1–G6 final disposition;
- every `P5-CV-*` row assigned PASS / FAIL / DEFERRED / SUPERSEDED with evidence;
- exact commands actually executed;
- classified remaining historical grep hits;
- explicit baseline/deferred tests;
- residual risks and consultation decisions;
- the Tuple/Dynamic decision;
- the expansion exact/assumed/derived evidence matrix.

---

## P5-AUD-013 — Final commit scope and CI posture reduce auditability

The final P5 commit contains 154 files and mixes P5 source changes with broad unrelated
documentation changes, Obsidian state, unstructured docs, callable spec movement, numeric
documents, and other material.

This is not a semantic bug, but it weakens reviewer confidence because:

- unrelated churn obscures the actual migration;
- bisectability is reduced;
- gate evidence is harder to associate with code changes;
- no GitHub commit status/workflow run independently corroborates the committed test
  summary.

Future closure/remediation should use focused commits by finding or task group.

---

# 7. Task-by-Task Audit

| Task / Gate | Audit disposition | Rationale |
|---|---|---|
| T0 | **Mostly satisfied** | repository was re-grounded and plans were detailed, though final record closure is weak |
| T1 | **PASS** | canonical semantic `for_iteration_lanes` product exists |
| T2 | **PASS** | lowering carries selected/requirement trait execution specs |
| T3 | **PASS** | List stateful vertical exists |
| G1 | **PASS with caveat** | semantic/lowering foundation is sound enough to retain |
| T4 | **PARTIAL** | single-lane canonical lowering works, but leaks one operand on natural exhaustion |
| T5 | **PASS** | generalized lanes preserve two-phase acquisition, patterns, ordinals and strict mismatch |
| G2 | **REOPEN** | T4 stack-discipline defect means canonical loop gate is not fully clean |
| T6 | **PARTIAL** | Set/Bytes/String stateful paths are good; Tuple Dynamic Item is not consumable by canonical resolver |
| T7 | **PASS** | Map + views use stateful iterators and labeled entry Item |
| T8 | **PASS** | Range stateful forward subset is implemented |
| G3 | **REOPEN** | Tuple vertical incomplete and directly affected corpus still encodes old behavior |
| T9 | **PARTIAL / FAIL semantic contract** | adapter state machines exist, but generic constraints/type propagation are not implemented as required |
| T10 | **FAIL** | eager wrappers, indexed eager map, `isEmpty`, and fixture migration are missing |
| T11 | **PASS** | boundedness ownership and canonical selector recognition are in place |
| G4 | **FAIL** | T9/T10 requirements unmet; mandated corpus slices not closed |
| T12 | **FAIL** | dedicated expansion product exists, but semantic failure is discarded and dynamic selector fallback survives; runtime roots missing |
| T13 | **PARTIAL** | `compile_for_lanes` and core cursor methods are removed; RefRange/Tuple declarations added; repository fixture/semantic closure incomplete |
| T14 | **PASS** | Collection hierarchy/public runtime identity retirement is substantially correct |
| G5 | **FAIL** | expansion structural fallback violates the gate; structural lookalikes are currently accepted |
| T15 | **FAIL** | docs/comments/fixtures/census/records not fully migrated |
| G6 | **FAIL** | multiple directly affected failures remain and final evidence matrix is absent |

---

# 8. Test-Surface Audit

## 8.1 What the final records claim

The committed walkthrough/handoff report:

```text
capabilities::iteration                 6 passed
capabilities::derived_conformance      38 passed
language::iteration                    13 passed
language::derived_conformance          11 passed
core_outgoing_packs & completion       28 passed
phalcom-lsp                             84 passed
affected crate checks                   PASS
git diff --check                        PASS
```

Those are useful focused results, assuming accurately recorded.

## 8.2 Why those results are insufficient

The selected suite is biased toward the new path and omits tests that the Part II/III plans
explicitly required for migration closure.

Most importantly, it omitted the active language-corpus slices where the old public API and
cursor semantics remain registered.

The outgoing-packs suite is not evidence of canonical-only expansion because its current
tests deliberately exercise non-conforming structural `iterator` objects.

`capabilities::iteration` is also not a comprehensive canonical-positive suite: several
tests are legacy cursor-shape negatives/unknown cases. It does not replace exact/assumed/
derived expansion evidence tests.

## 8.3 Required new discriminators

### Expansion authority

```text
EXP-CAN-01 concrete exact Iterable expansion
EXP-CAN-02 generic T |= Iterable expansion
EXP-CAN-03 T |= Iterator expansion through derived Iterable theorem
EXP-CAN-04 conditional/derived conformance with runtime environment bindings
EXP-CAN-05 structural lookalike with iterator/next but no conformance is rejected
EXP-CAN-06 invalid scalar *42 fails semantically / before target activation
EXP-CAN-07 iterator body exception from valid conformance propagates
```

### Adapter typing

```text
ITER-TYPE-01 map changes Item T -> U
ITER-TYPE-02 filter preserves T
ITER-TYPE-03 filterMap changes to U
ITER-TYPE-04 chain same Item accepted
ITER-TYPE-05 chain mismatched Item rejected
ITER-TYPE-06 flatten requires inner Iterable
ITER-TYPE-07 flatMap requires mapper result Iterable
ITER-TYPE-08 zip result preserves both lane Item types
ITER-TYPE-09 indexed produces (Int, T)
```

### Eager/lazy role split

```text
SURFACE-01 List.map is eager List
SURFACE-02 List.iter.map is lazy
SURFACE-03 direct filter/flatMap remain eager
SURFACE-04 map(indexed:) remains eager
SURFACE-05 reusable collection isEmpty remains non-consuming
SURFACE-06 same stored lazy pipeline is one-shot
SURFACE-07 new source.iter rebuild is fresh
```

### Compiler stack discipline

```text
FOR-STACK-01 repeated natural completion of single-lane for in one activation
FOR-STACK-02 break path does not execute sentinel cleanup
FOR-STACK-03 continue closes captures and does not grow stack
```

### Dynamic Item closure

```text
TUPLE-ITER-01 direct tuple iterator next behavior
TUPLE-ITER-02 for over Tuple publishes canonical lane or is explicitly diagnosed per ratified policy
REFRANGE-ITER-01 RefRange publishes its real Item type and works through generic Iterable bound
```

---

# 9. Remediation Plan

The remediation should be patch-sized and ordered by authority. Do not start by rewriting
docs while semantic execution remains split.

## Phase A — Restore one canonical expansion/iteration authority

### A1. Fail closed for positional expansion

Modify semantic call/list/tuple spread analysis so a missing canonical proof is not silently
discarded.

Delete the dynamic compiler fallback.

Acceptance:

```text
Unit/Tuple direct product fast path remains
all other * sources require PositionalExpansionLoweringSpec
structural iterator/next lookalike rejected
```

### A2. Root positional expansion trait environments

Refactor the runtime-root extraction used for `for_iteration_lanes` and apply it to
`positional_expansions`.

Add exact/assumed/derived runtime execution tests.

### A3. Resolve the Dynamic associated-Item architecture

This needs an explicit design decision, not another local patch.

Determine how `ForIterationLaneResolution` and related products represent explicit Dynamic
Item knowledge, or remove/defer Tuple's canonical conformance until that model is real.

Give `RefRange` its actual element type if possible.

### A4. Fix single-lane stack cleanup

Introduce separate natural-exhaustion and common exit labels.

Add a stress regression.

---

## Phase B — Complete Part II's public and typed surface

### B1. Implement adapter generic contracts

Bring `iterator_laws.ph` into line with T9.4.

Do not globally weaken the toolbox to Dynamic.

### B2. Restore inherent eager collection wrappers

Implement:

```text
map
map(indexed:)
filter
flatMap
isEmpty
```

on the accepted reusable collection families using thin forwarding only.

### B3. Ratify or repair Tuple.zipped semantics

Do not let an accidental Dynamic structural protocol become an undocumented second
iterability model.

---

## Phase C — Migrate the actual affected regression corpus

Rewrite current PASS fixtures that still assert:

- cursor selectors;
- repeatable stored pipelines;
- old Map key-only cursor behavior;
- removed Range cursor behavior.

Keep historical fixtures only if moved/marked as historical and no longer executed as
current acceptance tests.

Run the exact Part II G4 corpus commands.

---

## Phase D — Stabilization and records

Update:

```text
docs/spec/current/iteration.md
docs/guide/collections.md
docs/guide/control-flow.md
docs/spec/collections/06-rest-spread-and-pack-operators.md
other current stdlib collection/Bytes docs
phalcom-ast/src/ast.rs comments
phalcom-core/src/bytecode.rs JumpIfNone comment
phalcom-core/src/vm/dispatch.rs JumpIfNone comment/name
language corpus comments
.agents language overlay
```

Then perform the final census across:

```text
phalcom-core/src
phalcom-core/core/universe/src
phalcom-semantic/src
phalcom-native-meta/src
phalcom-modules/src
phalcom-ast/src
phalcom-lsp/src
examples
phalcom-core/tests/fixtures
docs/spec/current
docs/guide
docs/spec/collections
```

Classify historical ADR/implementation-plan hits separately.

Finally rewrite the P5 walkthrough/handoff with the real commit SHA and the complete
`P5-CV-*` disposition matrix.

---

# 10. Suggested Re-Certification Gates

## Gate R1 — semantic and lowering authority

```sh
cargo test -p phalcom-semantic --test semantic capabilities::iteration
cargo test -p phalcom-semantic --test semantic capabilities::derived_conformance
cargo test -p phalcom-semantic --test semantic incremental
```

Must include the new expansion exact/assumed/derived/lookalike discriminators.

## Gate R2 — core canonical iteration

```sh
cargo test -p phalcom-core --test core language::iteration
cargo test -p phalcom-core --test core language::derived_conformance
```

Add the stack-balance stress case and Tuple/RefRange disposition.

## Gate R3 — iterator surface and migration corpus

Run the current repository equivalents of:

```text
language corpus: iterator
iterator_negative
sequence
sequence_negative
list
collections
bytes
strings
iteration
iteration_disasm
iteration_negative
```

No current PASS fixture may reference the retired cursor protocol.

## Gate R4 — outgoing packs

```sh
cargo test -p phalcom-core --test core core_outgoing_packs
cargo test -p phalcom-core --test core core_outgoing_packs_completion
```

The tests must now prove canonical evidence, not structural fallback.

## Gate R5 — tooling/current docs/census

```text
phalcom-lsp tests
affected crate checks
git diff --check
final classified cursor/name census
P5-CV closure table
```

Only after all five remediation gates pass should P5 return to `COMPLETE`.

---

# 11. Final Assessment

P5 is not a failed implementation. It has already done the hardest structural work:

- it established semantic iteration authority;
- it projected that authority into executable lowering metadata;
- it migrated the core runtime model to real stateful iterators;
- it removed the runtime `Iterable` superclass;
- it installed the abstract `Collection` taxonomy;
- it changed Map to entry-tuple iteration;
- it preserved direct-jump loop control;
- it removed VM cursor-protocol discovery;
- it built a large source-authored iterator/default library.

Those are strong foundations.

The problem is that the final completion step closed the project record before the
migration was internally singular.

The repository currently contains **three incompatible iteration notions**:

```text
1. canonical trait evidence
   used by for

2. structural iterator/next selector coincidence
   still accepted by positional *

3. historical cursor iterate/iteratorValue
   still encoded by active corpus and current normative/user documentation
```

P5's explicit objective was to end with exactly one.

In addition, Part II's typed adapter contracts and eager collection surface were not
finished, and the Tuple/Dynamic solution is only half-landed: the type system can now store
a Dynamic associated binding, but the canonical iteration resolver cannot consume it.

Accordingly:

> **The correct next action is not P6 certification. Reopen P5 for a focused remediation
> cycle, repair the semantic/runtime seams above, run the omitted affected corpus, and then
> regenerate the completion records from actual evidence.**

The implementation is close enough that this should be a targeted closure effort rather
than a redesign. The central semantic → lowering → compiler → VM architecture should be
preserved.

---

# Appendix A — High-Value Code References

| Area | Current reference |
|---|---|
| positional expansion compiler fallback | `phalcom-core/src/compiler/lib/expr.rs:93–150` |
| expansion semantic failure discarded in calls | `phalcom-semantic/src/checker/call.rs:301–312` |
| expansion failure discarded in list literal | `phalcom-semantic/src/checker/expression.rs:1740–1755` |
| canonical expansion resolver | `phalcom-semantic/src/checker/iteration.rs:320–343` |
| runtime root collection for `for` only | `phalcom-core/src/modules/compile.rs:670–730` |
| environment attachment knows expansions | `phalcom-core/src/modules/semantic_lowering.rs:503–532` |
| iterator default surface | `phalcom-core/core/universe/src/collections/iterator_laws.ph:3–22` |
| FlatMap/Flatten Dynamic Item | `iterator_laws.ph:445–524` |
| Chain underconstraint | `iterator_laws.ph:526–559` |
| Zip/Indexed Dynamic Item | `iterator_laws.ph:780–846` |
| single-lane for stack leak | `phalcom-core/src/compiler/lib/loops.rs:138–169` |
| scope does not clean evaluation stack | `phalcom-core/src/compiler/lib/scope.rs:195–221` |
| Dynamic associated binding representation | `phalcom-semantic/src/impls.rs:472–480` |
| normalization has no TypeId for Dynamic | `phalcom-semantic/src/types/projection.rs:109–115` |
| active cursor corpus registration/comments | `phalcom-core/tests/core/language/corpus.rs` |
| stale normative cursor spec | `docs/spec/current/iteration.md` |
| stale collection guide | `docs/guide/collections.md` |
| stale spread spec | `docs/spec/collections/06-rest-spread-and-pack-operators.md` |
| final completion record | `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-walkthrough.md` |
| final handoff | `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P5-handoff.md` |

---

# Appendix B — Completion Claims That Must Be Corrected

The current walkthrough/handoff statements below are not supported by final source state:

| Recorded claim | Audit result |
|---|---|
| “non-Tuple/non-Unit `*` requires canonical Iterable evidence” | **False** — missing spec falls back to dynamic `iterator`/`next` |
| “VM executes PositionalExpansionLoweringSpec without checking selectors at runtime” | **False for missing-spec path** |
| “all checklists satisfied” | **False** — T9/T10/T12/T15 have unmet fixed requirements |
| “complete iterator_laws combinator toolbox” | behavior largely present, but **generic contract incomplete/unsound** |
| “current documentation synchronized” | **False** — normative/current docs still cursor-based |
| “census 0 legacy cursor hits” | **Too narrowly scoped / false for current-facing repository surfaces** |
| “P5 COMPLETE” | **Not supported; reopen required** |

---

# Appendix C — Audit Confidence

| Finding | Confidence |
|---|---|
| dynamic expansion fallback exists and is reachable | **Very high** |
| expansion trait-root omission | **Very high** |
| adapter constraints missing | **Very high** |
| eager wrappers / isEmpty missing | **Very high** |
| Dynamic Item cannot yield `TypeId` in iteration resolver | **Very high** |
| single-lane exhaustion leaves one stack value | **Very high** |
| active cursor/repeatability fixtures remain | **Very high** |
| current docs/comments stale | **Very high** |
| Tuple.zipped structural policy conflicts with single-authority intent | **High**, final policy needs ratification |
| inline support hard-coded fragility | **High** |
| runtime missing-binding Dynamic fallback masks defects | **High risk**, broader semantic policy needs separate review |
