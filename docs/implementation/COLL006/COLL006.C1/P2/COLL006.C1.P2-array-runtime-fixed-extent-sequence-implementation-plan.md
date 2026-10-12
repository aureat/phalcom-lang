---
id: COLL006.C1.P2
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: advanced-implementer-plan
title: "Array<T> Runtime Fixed-Extent Sequence"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
branch: main
audited_revision: 61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1
audited_at: 2026-10-08
predecessor: COLL006.C1.P1
successors:
  - COLL006.C1.P3
  - COLL006.C1.P4
analysis_source: COLL006.C1.P2-array-requirements-and-architecture-analysis.md
---

# COLL006.C1.P2 — `Array<T>` Runtime Fixed-Extent Sequence

## Advanced Implementer Execution Plan

## 0. Execution thesis

P2 introduces one new canonical Telos collection family:

```text
Array<T>
```

Its meaning is:

> **An owned, homogeneous, ordered sequence whose extent is runtime state fixed permanently after construction, while existing elements remain mutable.**

P2 is complete only when that meaning is represented coherently at every layer that owns part of the proposition:

```text
canonical Universe source declaration
    ↓
canonical semantic nominal generic identity
    ↓
runtime behavior class
    ↓
fixed native payload
    ↓
minimal irreducible storage floor
    ↓
Telos-authored sequence protocol
    ↓
canonical Iterator / Iterable conformance
    ↓
P1 sequence-literal semantic resolution
    ↓
checked lowering product
    ↓
direct fixed construction / private spread builder
    ↓
ArrayObject
    ↓
GC, equality, key policy, diagnostics, invariants
```

P2 must **not** merely make programs that mention `Array` execute. It must establish one authority chain in which syntax does not decide the family; the compiler does not re-infer the family; List is not used as hidden staging storage; Array cannot resize after publication; Array's runtime extent is not part of its type; generic element type `T` remains semantic type information rather than per-object runtime metadata; mutable structural equality does not leak into hash-key unsoundness; and P4 can later add const/static Array and flip the runtime bare-literal default without redesigning Array.

The implementation should therefore be reasoned about as a **semantic/runtime integration**, not as a collection-class feature.

---

# 1. Starting repository state

## 1.1 Audited revision

The plan was frozen against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1
commit: fix: repair random resource and bootstrap blockers and record continuation
```

`main` was rechecked immediately before this plan was generated and remains at that revision.

## 1.2 P1 has not landed at this revision

Current production code still contains the pre-P1 chain:

```text
Expr::ListLiteral
    ↓
synthesize_list_literal
    ↓
List<T>
    ↓
BuildList / BeginListLiteral / ...
    ↓
ListObject
```

There is no live `BracketLiteral`, `SequenceLiteralResolution`, `UniverseKey::Array`, `ArrayObject`, `BuildArray`, or Array source module on this audited `main`.

### P2 implication

**P2 is not executable from the semantic/literal integration tranches onward until the P1 takeover interfaces exist.**

Do not absorb P1 into P2. If the implementer begins P2 before P1 lands, only repository reconnaissance, decision preparation, and isolated runtime experiments may proceed. No duplicate neutral-literal or inference architecture may be committed under P2.

---

# 2. Authority hierarchy

When implementation facts disagree, resolve them in this order:

1. `docs/implementation/COLL006/PROGRAM.md`
2. `docs/implementation/COLL006/STATUS.md`
3. `docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md`
4. `docs/implementation/COLL006/COLL006.C1/COLL006.C1-master-implementation-record.md`
5. the landed P1 implementation and its closure/walkthrough records
6. accepted Telos decision records governing the affected subsystem
7. current executable repository behavior
8. this plan

This plan organizes execution. It does not supersede a more authoritative semantic ruling.

## 2.1 Explicitly stale design input

The older analysis:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/collections/
    telos-collections-requirements-analysis.md
```

contains the older taxonomy:

```text
Array<T,N>
Vector<T>
```

That is **not** the current COLL006 design. Current authority is:

```text
Array<T>       runtime extent, fixed after construction, mutable elements
Vector<T,N>    static extent N, immutable
List<T>        runtime extent, dynamically resizable
```

An implementation that introduces `Array<T,N>` is a P2 failure, even if motivated by that historical document.

---

# 3. Telos semantic philosophy that governs this plan

## 3.1 Semantic identity is source-owned

`Array` is a canonical Universe declaration. `UniverseKey`, native metadata, runtime `ClassId`, generated native surfaces, and compiler lowering are projections of that declaration. They must not become competing identity authorities.

## 3.2 Syntax records evidence; semantic analysis decides meaning

After P1, `<Int>[1,2,3]` is syntactically a headed bracket literal. The parser does not own the proposition that one head argument means Array. P2 adds that semantic rule to checked sequence resolution.

Likewise, `let a: Array<Int> = [1,2,3]` selects Array through contextual semantic evidence, not compiler pattern matching.

## 3.3 Runtime representation enforces runtime invariants

If a published Array must never resize, the representation should make accidental resizing difficult. Therefore `Box<[Value]>` is preferred over `Vec<Value>` with a convention not to push.

## 3.4 Generic typing is erased where representation does not need it

The runtime Array payload stores `Value`s. It does not store `TypeId T`, a runtime generic descriptor, or an element classifier tag merely to restate what semantic typing already proved.

## 3.5 Public simplicity may rest on precise internal machinery

The public language should permit `let a: Array<_> = [1,2,3]` and `let b = <Int>[1,2,3]` without exposing the inference session, sequence-resolution product, compiler builder, or native storage representation.

## 3.6 No hidden semantic conversion

A literal checked as Array must become Array directly. This is forbidden:

```text
Array literal
  -> construct List
  -> convert List to Array
```

The construction mechanism is part of the implementation contract because it protects allocation behavior, identity, future mutability semantics, and compiler authority.

## 3.7 Failure should be local and explicit

If checked Array literal resolution is missing, fail closed as an internal compiler error. Do not silently fall back to List. If an inference hole cannot be solved, report underconstraint. Do not make it `Dynamic`.

---

# 4. P2 semantic contract

## 4.1 Type identity

`Array<T>` has exactly one generic parameter:

```text
T : Type
```

The extent is runtime state. Therefore `Array<Int>` length 3 and `Array<Int>` length 7 have the same semantic type.

## 4.2 Extent

For any published `ArrayObject A`:

```text
len(A) is fixed from publication until collection
```

No public or internal Array operation may change it.

## 4.3 Element mutation

For ordinary runtime Array, indexed replacement is allowed and preserves type, size, and object identity.

## 4.4 Ownership

Array owns its storage. P2 Array is not a view. P5 owns Slice.

## 4.5 Ordering

Array iteration and structural equality use increasing index order.

## 4.6 Variance

`Array<T>` is invariant in `T`. Do not add variance machinery.

## 4.7 Hash/key policy

Array uses structural equality but remains mutable. It is not a valid `Map`/`Set` key under the current mutable-collection key law. Do not add structural hash to make it a key.

---

# 5. P2 acceptance objective

P2 is acceptable only when all of the following are true:

1. Canonical `Array` exists as a Universe source declaration.
2. `Array<T>` forms a canonical one-parameter nominal generic type.
3. `Array` is a direct nominal subclass of `Collection`.
4. `Array` has a dedicated runtime behavior class.
5. `Array` has a dedicated native object representation.
6. Published Array storage cannot resize.
7. Ordinary Array element replacement works in O(1).
8. `size` works in O(1).
9. indexed read works in O(1).
10. Array is precisely traced by GC.
11. replaced Array elements cease to be retained solely by old storage.
12. source-level Array uses canonical `Iterator` / `Iterable`.
13. Array equality is ordered structural equality.
14. cross-family same-content equality is false.
15. Array is rejected as mutable `Map`/`Set` key.
16. no resize selector is exposed.
17. no hidden resize primitive exists on Array storage.
18. contextual `Array<T>` literals construct Array.
19. contextual `Array<_>` literals infer T through P1.
20. `<T>[...]` constructs Array.
21. `<_>[...]` infers T through P1.
22. empty explicit/contextual typed Array literals work.
23. unresolved empty holes remain underconstrained.
24. runtime-length positional spreads into Array work.
25. spread item typing uses canonical P1 evidence.
26. fixed literals construct Array directly.
27. spread literals finalize directly into Array.
28. no Array literal path constructs List internally.
29. bare runtime `[...]` remains List until P4.
30. compiler lowering consumes the checked P1 sequence-resolution product.
31. compiler does not infer Array family from AST/head/context independently.
32. object-model/bootstrap/native-surface invariants include Array.
33. focused Array tests pass.
34. P1 regression tests remain green.
35. no P3 static-cardinality machinery is introduced.
36. no P4 static sequence evaluator/materializer is introduced.
37. lifecycle records truthfully state any unresolved broader baseline failures.
38. P2 walkthrough records actual landed architecture and deviations from this plan.

---

# 6. Hard scope boundaries

## 6.1 P1 owns

P2 must consume, not rebuild:

```text
neutral bracket-literal AST
explicit bracket head syntax
`_` generic/static inference-hole syntax
inference-bearing annotations
EvaluationPhase
checked SequenceLiteralResolution
sequence semantic fingerprinting
ModuleLoweringSemantics projection
compiler checked-product consumption
canonical positional-expansion semantic evidence
```

## 6.2 P2 owns

```text
Array canonical identity
Array<T> type form
Array runtime class
ArrayObject
fixed mutable storage
minimal Array storage primitive floor
source Array protocol
Array iterator/conformance
Array structural equality
Array mutable-key rejection
contextual Array literal activation
one-head explicit Array literal activation
fixed Array literal direct lowering
runtime-spread private builder/finalization
Array GC/object-model/native integration
Array-specific diagnostics
```

## 6.3 P3 owns

P2 must not implement:

```text
Vector<T,N>
static Int generic extent
N >= 0 type-formation checks
exact positional cardinality proof
runtime length == N invariant
static-cardinality spread rejection
Vector structural hash
```

## 6.4 P4 owns

P2 must not implement:

```text
runtime bare [...] -> Array default switch
static bare [...] -> Vector default switch
StaticSequence
static Array evaluation
static Array materialization
const Array source semantics
repository-wide bare-List migration
TDR-0025 / TDR-0028 semantic default amendment
```

P2 may establish a **private frozen runtime state** only as a representation seam for P4, subject to Gate G3 below.

---

# 7. Fixed / mechanically flexible / verify-first decisions

## 7.1 FIXED

Do not change these without consultation:

```text
Array<T>, not Array<T,N>
runtime extent is not part of type
fixed extent after publication
element mutation allowed
Array invariant in T
Array and List are sibling families
Array has dedicated runtime identity
published storage is not growable
no hidden List staging for Array literals
semantic checker owns family selection
compiler consumes semantic resolution
runtime bare [...] remains List until P4
<T>[...] is Array once P2 activates it
Array<_> / <_> use inference holes, never Dynamic
Array structural equality is same-family/order-sensitive
Array is not a mutable hash key
canonical Iterator / Iterable are reused
```

## 7.2 MECHANICALLY FLEXIBLE

```text
exact Rust field names
whether Object::Array stores ArrayObject or Box<ArrayObject>
exact private builder object name
exact Array builder opcode names
whether builder storage helper is reusable internally by future Vector
exact internal construction helper names
source placement of tiny Array helper methods
exact diagnostic wording
exact private frozen-state encoding
```

## 7.3 VERIFY-FIRST

```text
P1 SequenceLiteralResolution exact shape
P1 lowering projection exact APIs
P1 PositionalExpansionTarget shape
native source identity registration path on landing revision
whether generated native signatures can expose owner parameter T safely
constructor/rest-method support for @constructor call(...)
current floor-census mechanism
current Object slot-size ceiling test
current generic native-representation allocation guard
```

---

# 8. Durable P2 invariants

## 8.1 Type invariants

- **P2-INV-TYPE-01 — One type parameter.** `Array<T>` has one type-domain parameter and no extent parameter.
- **P2-INV-TYPE-02 — Runtime extent erased from type identity.** Same T, different lengths, same canonical type.
- **P2-INV-TYPE-03 — Invariance.** No Array covariance/contravariance.
- **P2-INV-TYPE-04 — Canonical source identity.** Only the canonical Universe declaration owns built-in Array identity.

## 8.2 Representation invariants

- **P2-INV-REP-01 — Fixed payload.** A published Array payload has no operation capable of changing its slot count.
- **P2-INV-REP-02 — Dedicated object identity.** Array is not represented as List.
- **P2-INV-REP-03 — Fully initialized publication.** Every Array slot contains a valid `Value` before guest visibility.
- **P2-INV-REP-04 — No runtime T tag.** ArrayObject does not duplicate semantic generic T as per-object metadata.
- **P2-INV-REP-05 — Object slot discipline.** Array does not silently inflate the common heap slot beyond the accepted budget.

## 8.3 Mutation invariants

- **P2-INV-MUT-01 — Replacement only.** Array mutation changes values, not extent.
- **P2-INV-MUT-02 — Set preserves size.** Every successful indexed write preserves `size`.
- **P2-INV-MUT-03 — No resize selector.** Guest code cannot invoke Array append/insert/remove/resize behavior.
- **P2-INV-MUT-04 — Frozen seam.** If private frozen state lands, every native write path checks it.

## 8.4 Literal invariants

- **P2-INV-LIT-01 — Semantic family authority.** Array family selection is published by semantic analysis.
- **P2-INV-LIT-02 — Direct fixed construction.** A fixed Array literal does not construct a List or call conversion.
- **P2-INV-LIT-03 — Private dynamic construction.** A spread-containing Array literal accumulates only in compiler/VM-private builder state before final Array publication.
- **P2-INV-LIT-04 — No early default switch.** Unconstrained runtime bare brackets preserve the pre-P4 compatibility result.
- **P2-INV-LIT-05 — Hole semantics.** `_` remains a solver variable and must resolve before canonical result publication.
- **P2-INV-LIT-06 — Runtime spread permitted.** Array does not require statically known spread cardinality.

## 8.5 Compiler invariants

- **P2-INV-COMP-01 — No family inference in compiler.** Compiler code does not inspect annotation/head/phase to decide Array versus List.
- **P2-INV-COMP-02 — Missing resolution fails closed.** An executable checked bracket literal without semantic product is an internal error.
- **P2-INV-COMP-03 — Canonical expansion evidence.** Spread lowering consumes P1's checked expansion resolution.

## 8.6 GC invariants

- **P2-INV-GC-01 — Every element traced.** All contained `Value`s are traversed by GC.
- **P2-INV-GC-02 — Replaced value not retained.** An overwritten element has no hidden stale Array edge.
- **P2-INV-GC-03 — Builder values traced.** Private dynamic builder storage traces accumulated values while user iteration can re-enter the VM.

## 8.7 Equality/key invariants

- **P2-INV-EQ-01 — Same-family structural equality.** Array equality is ordered element equality.
- **P2-INV-EQ-02 — Cross-family false.** Array is not equal to List/Tuple solely from equal elements.
- **P2-INV-KEY-01 — Mutable-key rejection.** Array cannot be admitted into Map/Set under identity hash plus structural equality.

---

# 9. Implementer reasoning protocol

Use the following loop rather than patching from symptoms.

## 9.1 OBSERVE

Before each tranche, read the exact current implementation, identify the existing authority for the proposition being changed, identify exhaustive matches and generated projections, identify tests that encode current behavior, and distinguish stale documentation from current authority.

## 9.2 MODEL

Write the smallest semantic model of the change:

```text
semantic type: Array<T>
runtime payload: fixed slots
public mutation: slot replacement
family selection: semantic product
compiler action: construct selected family
```

Use this model to reject implementation shortcuts that introduce a second truth.

## 9.3 LOCATE AUTHORITY

For every decision, ask who is allowed to know it:

```text
"this literal is Array"      -> semantic checker
"Array has one type arg"     -> canonical declaration/type form
"this object is Array"       -> runtime class mapping
"slot count is fixed"        -> ArrayObject representation
"spread is iterable"         -> P1 semantic expansion resolution
```

## 9.4 PREDICT BEFORE EDIT

Before modifying a seam, predict which tests should fail, which exhaustive matches should fail compilation, which output should change, and which behavior must remain unchanged.

## 9.5 IMPLEMENT A COHERENT SLICE

Prefer one complete vertical slice over speculative placeholders. For example, `UniverseKey::Array + source declaration + array_class + Object::Array + Value::class + GC + runtime construction test` is coherent.

## 9.6 DISCRIMINATE FAILURES

Classify each failure as expected red, implementation defect, stale test, P1 contract mismatch, baseline unrelated failure, or architectural contradiction.

## 9.7 RECORD DURABLE FACTS

When implementation discovers a fact that successor plans need, write it into the P2 walkthrough/checkpoint. Do not bury architectural discoveries only in commit messages.

---

# 10. Problem-solving state machine

## GREEN — expected implementation work

Continue autonomously for missing exhaustive Array arms, straightforward registration omissions, local test expectations, ordinary compiler/runtime wiring bugs, and source API implementation under fixed semantics.

## AMBER — bounded diagnosis

Stop editing and diagnose when P1's actual interfaces differ from the expected P2 seam, runtime class identity mismatches cross source/native/semantic layers, native generic signatures lose T unexpectedly, dynamic spread requires a new compiler abstraction, GC/rooting is ambiguous, native representation allocation routes through `InstanceObject`, or Array causes broad unrelated baseline movement.

Use one serious causal correction. If the same semantic failure survives, escalate.

## RED — mandatory consultation

Do not redesign locally if P2 seems to require `Array<T,N>`, a new type-store category, compiler-side family inference, List staging, growable published storage, unauthorized primitive-floor widening, runtime T metadata, a public const Array type, changed Iterator/Iterable laws, the bare-literal default switch, or P3 static-cardinality machinery.

---

# 11. Entry gates

## G0 — Takeover and lifecycle gate

Required:

1. verify current `main` SHA;
2. verify no unrelated work is being overwritten;
3. read COLL006 program/status/checkpoint/master, P1 plan, and P1 walkthrough/closure if landed;
4. mark P2 `ACTIVE` only when implementation actually begins;
5. record implementation start revision and verification baseline.

Exit: durable P2 record identifies start SHA, P1 status, blockers, branch/worktree, and baseline.

## G1 — P1 predecessor gate

Required proof: landed P1 provides working equivalents of neutral bracket literal AST, preserved explicit head syntax, inference-hole syntax, inference-bearing contextual annotations, `EvaluationPhase`, checked sequence-literal resolution, semantic product publication, fingerprint participation, lowering projection, compiler product consumption, and canonical positional-expansion resolution.

Required probes include `let x: List<_> = [1,2,3]` and the legacy bare List compatibility case.

**STOP:** if P1 is incomplete, do not recreate missing pieces under P2.

## G2 — Native-floor authorization gate

Accepted TDR-0017 freezes the VM-blessed primitive floor. TDR-0018 explicitly admits List's native array storage seam. Before landing new Array storage primitives, one of the following must be recorded:

```text
A. accepted Array/container floor amendment exists
B. authoritative project ruling explicitly records COLL006 as sufficient amendment authority
```

Do not infer permission merely because List has similar storage.

## G3 — Representation gate

Before adding `Object::Array`, decide whether P2 lands private frozen state now.

Recommended ruling:

```rust
struct ArrayObject {
    elements: Box<[Value]>,
    frozen: bool,
}
```

with ordinary P2 Arrays unfrozen, a future P4 private materializer allowed to construct frozen Arrays, and no public freeze API in P2.

If the project defers the field to P4, record that explicitly before implementation.

## G4 — Dynamic builder architecture gate

Choose a private construction mechanism with growable private storage, no guest-visible class, no List staging, precise GC tracing, no heap borrow across user iteration, and target-specific Array finalization.

Internal naming is flexible. Do not generalize a public SequenceBuilder API.

## G5 — Public constructor probe

P2 does not need a rich public constructor to prove literals. Before publishing one, verify current `@constructor`, `call(...)`, rest-parameter typing, and representation. Default: if no clean constructor exists without scope expansion, omit it in P2 and defer constructor/conversion convergence to P9. Do not copy legacy `new` merely for convenience.

---

# 12. Target architecture

## 12.1 Source/type layer

```text
universe.collections.array

native class Array<T> is Collection
```

Canonical type is the nominal Array declaration applied to `[T]`. No dedicated Array TypeStore variant.

## 12.2 Runtime layer

```text
Value::Obj
   ↓
ObjRef
   ↓
Object::Array(ArrayObject)
   ↓
Box<[Value]>
```

## 12.3 Fixed literal path

```text
BracketLiteral
  ↓
P1 semantic resolution
family = Array
construction = fixed(count)
  ↓
ModuleLoweringSemantics
  ↓
compiler
  ↓
BuildArray(count)        # representative name
  ↓
ArrayObject::from_vec
```

## 12.4 Dynamic spread path

```text
BracketLiteral with spread
  ↓
P1 semantic resolution
family = Array
expansion evidence = checked
  ↓
compiler
  ↓
private growable builder
  ↓
checked positional expansion
  ↓
finish
  ↓
Vec<Value> -> Box<[Value]>
  ↓
Object::Array
```

There is no List node in this pipeline.

---

# 13. Ordered implementation program

# T0 — Activate P2 and freeze predecessor facts

## Purpose

Establish exact starting state before code changes.

## Actions

1. record `HEAD`;
2. read current P1 closure/walkthrough;
3. verify P1 tests;
4. reconcile this plan with actual P1 symbol names;
5. record native-floor authorization status;
6. record G3 frozen-state ruling;
7. record G4 builder ruling;
8. mark P2 lifecycle state appropriately.

## Exit criteria

Start SHA, P1 completion, floor gate, and representation gate are all explicit. No architecture ambiguity is hidden as implementation detail.

---

# T1 — Establish canonical `Array<T>` Universe identity

## Goal

Make Array a canonical source-owned generic family before literal/runtime machinery depends on it.

## Primary files/symbols to reverify

```text
phalcom-core/core/universe/src/collections/array.ph               # new
phalcom-core/core/universe/src/collections/package.ph
phalcom-modules/src/builtin.rs
phalcom-native-meta/src/universe.rs
phalcom-core/src/native/source.rs                                # if still current
phalcom-semantic/src/core_surface/identity.rs
phalcom-semantic/src/types/store.rs                              # only helper additions if needed
phalcom-semantic/tests/semantic/adts/native_core.rs
phalcom-semantic/tests/semantic/foundations/declarations.rs
phalcom-semantic/tests/semantic/foundations/type_members.rs
phalcom-semantic/tests/semantic/integration/type_member_closure.rs
```

## Required implementation

### T1.1 Universe key

Add `UniverseKey::Array` and all exhaustive mappings.

### T1.2 Canonical source module

Add `universe.collections.array` to the built-in Universe graph.

### T1.3 Source declaration

Start with the minimal canonical declaration:

```telos
native class Array<T> is Collection {
  // protocol added later
}
```

Do not inherit List.

### T1.4 Class relation

Add canonical `Array -> Collection` to the single authoritative relation table.

### T1.5 Type form

Add `Array<T>` with `T : Type`. No N.

### T1.6 Core semantic declaration identity

Add canonical Array declaration to `CoreDeclarationIds`.

### T1.7 Prelude/exposure

Follow current collection exposure policy so unqualified `Array<Int>` works if List/Map/Set do. Do not invent a new prelude policy only for Array.

## Tests

Prove `UniverseKey::from_name("Array")`, canonical owner/path, generic arity 1, `Array<Int>` formation, `Array<String>` distinctness, wrong arity rejection, `Array` superclass `Collection`, cross-module import/re-export identity, and that a user-defined `Array` elsewhere is not canonical Universe Array.

## Exit gate T1

The semantic layer can refer to one canonical `Array<T>` family without runtime storage existing yet.

## STOP conditions

Stop if type formation starts including length, native metadata becomes semantic declaration authority, Array requires a new kind/type-store node, or source identity needs a second competing built-in identity path.

---

# T2 — Add runtime Array class and fixed storage

## Goal

Make the VM represent an Array with fixed slot count and correct class identity.

## Primary files

```text
phalcom-core/src/heap/array.rs                                  # new
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs
phalcom-core/tests/core/object_model/invariants.rs
docs/specs/vm/values-and-objects.md
```

## T2.1 `ArrayObject`

Preferred baseline:

```rust
pub struct ArrayObject {
    elements: Box<[Value]>,
    frozen: bool,
}
```

Keep methods intentionally small: construction from initialized values, `len`, `is_empty`, `get`, narrow `set`, `elements`, and frozen-state inspection if G3 chose it.

Do not expose `push`, `insert`, `remove`, `replace_slice`, or `as_mut_vec`.

## T2.2 Fully initialized construction

Construction consumes initialized `Vec<Value>` and converts it to boxed slice. Never publish uninitialized `Value` slots.

## T2.3 `Object::Array`

Add the object arm. If object-size tests show inline payload inflates the common slot beyond the accepted budget, store `Box<ArrayObject>` rather than enlarging every heap slot.

## T2.4 Heap allocation/access

Add `alloc_array`, `array`, `array_mut`, and `as_array`. The mutable accessor must not expose resize capability.

## T2.5 Runtime class

Add `CoreClasses::array_class` under `collection_class`. Update exhaustive `CoreClasses::resolve`, root enumeration, class census, and parallel-metaclass verification.

## T2.6 `Value::class`

Map `Object::Array(_)` to `array_class`.

## T2.7 GC trace

Trace every element. Keep `trace_object` wildcard-free.

## T2.8 Internal rendering/type name

Update exhaustive object-kind rendering and diagnostic type names without requiring runtime reflection of T.

## Tests

Prove zero/one/many lengths, fixed size after set, correct class, sane rendering, object-slot budget, GC retention of contained values, and collectability of an overwritten old value.

## Exit gate T2

A VM-level Array exists with correct fixed representation, class identity, and GC behavior independent of literals.

## STOP conditions

Stop if published representation requires `Vec<Value>`, Array must be represented as List, object-slot budget has no bounded boxing solution, or frozen state starts introducing public const semantics.

---

# T3 — Add the minimal Array native storage floor

## Goal

Expose only irreducible native operations needed by the Telos source protocol.

## Preconditions

G2 floor authorization is resolved.

## Primary files

```text
phalcom-core/src/primitive/array.rs                             # new
phalcom-core/src/primitive/mod.rs
phalcom-core/core/universe/src/collections/array.ph
native surface generated/verification outputs
native source tests
floor census tests
```

## Required floor

Conceptually:

```text
rawLength
rawAt(_)
rawSet(_,_)
```

plus a strictly internal construction hook only if required by non-bytecode paths. Direct literal bytecode allocation does not require a public source constructor primitive.

## T3.1 Receiver extraction

Add `expect_array` parallel to existing collection receiver extractors.

## T3.2 Index normalization

Reuse the canonical shared index normalizer. Do not create Array-specific negative-index rules.

## T3.3 Read convention

Follow current sequence convention: valid index returns the value; out-of-range raw read returns immediate `None` if that remains canonical.

## T3.4 Write convention

Valid write replaces one slot and preserves size. Out-of-range write is a hard runtime error. Frozen write, if the seam landed, fails before storage mutation.

## T3.5 Generic semantic signature

Use the strongest source-visible generic contract current native metadata can express. If the raw read convention's value-or-None shape cannot be represented precisely, retain the established Dynamic raw boundary and put typed guarantees on strict/source wrappers. Do not redesign the type system for raw API aesthetics.

## T3.6 Floor census

Update native floor census and decision references exactly as required by G2 authorization.

## Negative requirement

No native Array primitive for append, insert, remove, resize, reserve, clear, or arbitrary range replacement.

## Tests

Cover raw length/read/write, negative index normalization, out-of-range behavior, size preservation, frozen write if applicable, and wrong receiver diagnostics.

## Exit gate T3

Native layer exposes enough storage power for source Array behavior and no structural resize power.

---

# T4 — Build the Telos-authored Array protocol

## Goal

Make Array a usable typed collection through source-defined behavior while keeping control logic out of Rust.

## Primary file

```text
phalcom-core/core/universe/src/collections/array.ph
```

Related: iterator laws and collection contract tests.

## T4.1 Basic sequence surface

Implement the appropriate current sequence API: `size`, `first`, `last`, safe/strict indexed reads, `get`, subscript, indexed replacement, and subscript assignment as supported by current conventions.

Use List behavior as a convention reference, not an implementation dependency.

## T4.2 Replacement semantics

Every successful write preserves `size`.

## T4.3 No structural mutation API

Do not define append, prepend, insert, remove, pop, clear-as-resize, or arbitrary cardinality-changing range replacement.

## T4.4 Iterator

Implement `ArrayIterator<T>` as a small Telos object holding the Array and an index. Conform to canonical `Iterator` and `Iterable`. Do not add a new iteration root.

## T4.5 Live-view iteration rule

Iterator reads current element values at each index. Add a test where an element is mutated after iterator creation but before that index is consumed; iterator should observe the new value. Extent remains stable.

## T4.6 Equality

Implement same-family ordered structural equality in Telos source. No native equality primitive.

## T4.7 `!=`

Route through equality.

## T4.8 Minimal conversions

Only add conversions needed for P2 usability and existing collection-law consistency. P9 owns full conversion convergence.

## T4.9 Constructor policy

Apply G5. Default: no public `new`, no uninitialized length constructor, and no native variadic constructor merely for convenience. If a clean `@constructor call(...)` is proven using existing rest semantics without new architecture, it may be included and must construct directly—not through List. Otherwise defer.

## Tests

Extend collection contract coverage for size, indexed read, equality laws, cross-kind false, absence of growth selector, indexed set and size preservation, associated Item typing, iteration order, and live-view mutation behavior.

## Exit gate T4

Array is a coherent runtime/source collection before literal integration begins.

---

# T5 — Integrate Array with mutable-key policy

## Goal

Preserve the existing collection equality/hash law.

## Primary file

```text
phalcom-core/src/primitive/mod.rs
```

and Map/Set tests.

## T5.1 Guard

Extend the canonical mutable-collection key predicate to include `Object::Array(_)`.

## T5.2 Diagnostic cleanup

Current helper/comment/error text already lags implementation because Bytes is rejected while wording names only List/Map/Set. Replace family enumeration with durable generic wording where possible, or accurately enumerate all mutable families.

## T5.3 Tests

Prove both Map and Set reject Array as a key/value-key with catchable user-visible errors, while existing List/Map/Set/Bytes behavior remains unchanged.

## Exit gate T5

Array structural equality cannot enter a hash table under incompatible identity hash.

---

# T6 — Activate Array in P1 semantic sequence resolution

## Goal

Teach the semantic checker—not the compiler—that a checked bracket literal may denote Array.

## Preconditions

P1 landed and T1 canonical Array identity exists.

## Primary files

Expected, subject to P1 landing shape:

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-core/src/modules/semantic_lowering.rs
P1 sequence literal modules/types if factored separately
semantic tests
```

## T6.1 Contextual Array

Support `let a: Array<Int> = [1,2,3]`. Expected canonical applied type selects family Array and element type Int.

## T6.2 Contextual hole

Support `let a: Array<_> = [1,2,3]` using P1's inference-bearing annotation. Element expressions constrain the hole. Canonical publication occurs only after successful inference finish.

## T6.3 Explicit one-head form

Activate `<Int>[1,2,3]` as Array. P1 parser preserves the generic argument; P2 semantic analysis resolves it under Array's type parameter domain.

## T6.4 Explicit hole head

Support `<_>[1,2,3]` with the same hole semantics.

## T6.5 Empty literals

Required:

```telos
let a: Array<Int> = []   // success
let b = <Int>[]          // success
let c: Array<_> = []     // underconstrained
let d = <_>[]            // underconstrained
```

No Dynamic fallback.

## T6.6 Element constraints

Every ordinary element checks against selected T. Every spread lane's associated Item checks against T through canonical P1 expansion evidence.

## T6.7 Runtime spread cardinality

No count proof is required. This is a key P2/P3 boundary: Array's result type does not mention length.

## T6.8 Compatibility default

Bare unconstrained runtime literal remains the P1 compatibility result—List. Add an explicit regression. Do not use `EvaluationPhase::Runtime` to select Array yet.

## T6.9 Semantic product

The existing P1 product should identify family Array, result type `Array<T>`, element type T, and construction plan fixed versus dynamic-spread. Extend P1's product; do not create a second Array-only resolution table.

## T6.10 Fingerprint

Ensure family/result/construction meaning participates in the existing sequence-resolution fingerprint path.

## Tests

Positive: contextual typed/hole, explicit typed/hole, typed empty, mixed legal inference, tuple spread, Iterable spread, multiple spreads, zero-item spread.

Negative: unresolved empty hole, wrong head arity/kind, incompatible element, non-Iterable spread, incompatible Item, unavailable two-head Vector form if P3 is not yet landed.

## Exit gate T6

Semantic layer proves and publishes Array literal meaning while compiler remains family-agnostic.

## STOP conditions

Stop if compiler context is needed to solve T, Array selection requires parser changes beyond P1 contract, length enters type identity, or runtime spread cardinality starts entering type inference.

---

# T7 — Implement direct fixed Array literal lowering

## Goal

Compile a checked fixed Array literal directly to final Array storage.

## Primary files

Expected:

```text
phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/modules/semantic_lowering.rs
docs/specs/vm/bytecodes.md
docs/work/analyses/bytecodes.md
disassembly tests
```

## T7.1 Consume lowering product

Compiler identifies Array only from the projected checked sequence-literal resolution. No AST-head inspection.

## T7.2 Add direct instruction

Representative design:

```rust
BuildArray(u16)
```

Exact name is flexible. Inherit the existing literal count width unless a real architectural reason requires broader bytecode redesign.

## T7.3 Evaluation order

Preserve source left-to-right evaluation. If stack collection reverses due to pop order, reverse exactly once as the current List path does.

## T7.4 Final allocation

VM constructs final Array directly from evaluated values, converting temporary host vector storage into the fixed payload.

## T7.5 No public constructor send

Do not lower literal to `Array.call(...)` or another overrideable send merely to avoid bytecode work.

## T7.6 No List path

Disassembly must prove absence of `BuildList`, `BeginListLiteral`, `ListLiteralAppend`, `FinishListLiteral`, and List conversion for fixed Array literals.

## Tests

Disassemble contextual typed, explicit typed, and empty fixed Array literals. Assert direct Array instruction and runtime class/size/content.

## Exit gate T7

Every fixed checked Array literal constructs one final Array collection payload without List staging.

---

# T8 — Implement dynamic-spread Array literal construction

## Goal

Support runtime-sized spreads while preserving fixed published Array extent.

## Primary files

Expected:

```text
phalcom-core/src/heap/<private_builder>.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
P1 positional expansion lowering helpers
disassembly/runtime/GC tests
```

## T8.1 Private builder

Use VM-private growable storage such as `Vec<Value>`. The builder is not an Array, has no guest-visible class, and must not escape as a receiver.

## T8.2 Bytecode shape

Representative names:

```text
BeginArrayLiteral
ArrayLiteralAppend
ArrayTryExpandTuplePositionals
FinishArrayLiteral
```

If P1 has generalized builder opcodes, integrate rather than duplicate mechanics.

## T8.3 Scratch-local rooting

Follow existing compiler scratch-local discipline. Builder remains in a rooted VM value location while expansion executes arbitrary guest code.

## T8.4 Expansion

Reuse P1's canonical expansion resolution: Unit/Tuple fast path if canonical, Iterable/Iterator otherwise. Do not create Array-specific trait solving.

## T8.5 Re-entry discipline

Never hold a mutable builder or heap borrow across guest sends. Snapshot/release, execute guest code, reacquire builder, append.

## T8.6 Finalization

Finish validates builder kind, moves accumulated values, allocates final `ArrayObject`, places the final value according to stack discipline, and leaves no guest-visible builder.

## T8.7 GC tracing

Private builder traces every accumulated element. Add forced-GC tests during spread iteration.

## T8.8 Failure cleanup

If iterator execution raises, no partial Array is published, scratch state unwinds normally, builder becomes collectible, and accumulated values remain rooted until unwind completes.

## Tests

Cover empty spread, one spread, multiple spreads, ordinary elements around spreads, tuple spread, stateful iterator, iterator raise, forced GC between yields, and disassembly proving Array builder opcodes with no List builder.

## Exit gate T8

Runtime-length positional spread constructs Array safely, directly, and without weakening published fixed extent.

---

# T9 — Harden diagnostics, object model, reflection, and API negative space

## Goal

Ensure Array behaves as a complete canonical runtime family rather than a literal-only special case.

## T9.1 Closed/reserved core name behavior

If canonical core names are reserved, add Array to the appropriate surface so user `class Array` follows the same rule as List/Map/Set.

## T9.2 Reflection

Verify Array class name, superclass, generic arity/type form, and normal method reflection. Do not add special Array reflection architecture.

## T9.3 Generic allocation guard

Verify generic instance allocator does not create `InstanceObject` for Array's native representation.

## T9.4 No resize selectors

Add negative sends for likely accidental APIs: `append`, `prepend`, `insert`, `remove`, and internal List resize primitives.

## T9.5 Diagnostics

Ensure deterministic diagnostics for underconstrained holes, wrong explicit head shape, incompatible elements, bad spread source/item, out-of-range mutation, frozen mutation seam if present, and mutable-key rejection.

## T9.6 Debug rendering

Ensure diagnostics never claim Array is List.

## Exit gate T9

The negative semantic surface is as deliberate as the positive API.

---

# T10 — Stabilization and certification

## Goal

Prove P2 as a cross-layer feature and record actual implementation truth.

## T10.1 Build phase

Run narrow package checks after each tranche using actual package names on the landing revision, at least around native metadata, semantics, modules, and core runtime.

## T10.2 Focused semantic tests

Run Universe identity, generic type form, contextual/explicit Array literals, hole inference, empty underconstraint, spread typing, P1 compatibility defaults, and incremental/fingerprint tests.

## T10.3 Focused runtime tests

Run ArrayObject, Array primitives, collection contract, Map/Set key rejection, GC, object-model invariants, bytecode/disassembly, and literal execution.

## T10.4 P1 regression gate

Run all P1-focused tests. P2 must not regress List hole inference, neutral syntax, bare List compatibility, or checked-product compiler discipline.

## T10.5 Broader collection gate

Run existing List, Map, Set, Tuple, Bytes, positional expansion, and collection corpus coverage because Array touches shared collection/runtime seams.

## T10.6 Workspace gate

Run the broadest repository gate expected by current lifecycle doctrine. If unrelated baseline failures exist, record exact failures and prove they reproduce without P2 where possible. Do not falsely mark broad verification green.

## T10.7 Performance assertions

Do not claim benchmark-level speed without measurement. Do prove architecture: fixed path has no collection staging allocation; index read/write use direct fixed storage; dynamic path uses one private builder plus final Array payload.

## T10.8 Diff audit

Before closure:

```text
git diff --check
search for accidental Array<T,N>
search for BuildList in Array lowering
search for Object::Array exhaustive omissions
search for stale mutable-key wording
search for public append/resize on Array
search for compiler head/context family inference
```

---

# 14. Test matrix

| Area | Positive proof | Negative proof |
|---|---|---|
| Identity | canonical `Array<Int>` | local `Array` not canonical / reserved rule |
| Generic arity | one T | two args reject |
| Extent type | lengths 0/1/N same type | no N in type identity |
| Runtime class | Object::Array → Array | not List |
| Storage | set works | size cannot change |
| Empty | typed empty succeeds | hole-only empty underconstrained |
| Context | `Array<Int> = [...]` | incompatible element |
| Hole | `Array<_> = [1]` | unresolved hole |
| Head | `<Int>[...]` | wrong head arity |
| Fixed lowering | direct BuildArray | no BuildList |
| Spread | runtime Iterable works | non-Iterable rejected |
| Builder | final Array | builder cannot escape |
| GC | elements retained | replaced old edge collectible |
| Equality | ordered structural | cross-kind false |
| Key law | scalar key unaffected | Array key rejected |
| Iteration | index order/live values | no structural invalidation path |
| Compatibility | bare runtime still List | no early P4 switch |
| Floor | authorized primitives counted | no unrecorded floor expansion |

---

# 15. Expected repository impact map

This is an audit guide, not a mandatory exact write set. Reverify after P1 lands.

## Canonical identity / semantic

```text
phalcom-native-meta/src/universe.rs
phalcom-semantic/src/core_surface/identity.rs
phalcom-semantic/src/types/store.rs                       # only if helper required
phalcom-semantic/src/checker/expected.rs                  # P1 extension seam
phalcom-semantic/src/checker/expression.rs                # P1 sequence resolver
phalcom-semantic/src/checker/analysis.rs                  # product family if required
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/tests/...
```

## Universe source/modules

```text
phalcom-core/core/universe/src/collections/array.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-modules/src/builtin.rs
phalcom-core/src/native/source.rs
```

## Runtime class/storage

```text
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/heap/array.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs
```

## Native storage floor

```text
phalcom-core/src/primitive/array.rs
phalcom-core/src/primitive/mod.rs
generated native surface / descriptor files
floor census tests
```

## Compiler/lowering

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/dispatch.rs
private builder object files
```

## Tests/docs

```text
phalcom-core/tests/core/collections/contract.rs
phalcom-core/tests/core/object_model/invariants.rs
phalcom-core/tests/core/collections/...
semantic sequence-literal tests
GC tests
disassembly tests
docs/specs/vm/bytecodes.md
docs/specs/vm/values-and-objects.md
COLL006 lifecycle records
```

---

# 16. Failure policy

## Class A — expected local implementation defect

Examples: forgotten exhaustive Array arm, registration omission, stack reversal bug, or source method typo. Fix directly.

## Class B — stale test/documentation

Update only when authoritative semantics changed; record why.

## Class C — predecessor contract mismatch

If P1 sequence product lacks enough information for Array, do not patch around it in compiler syntax. Diagnose whether P1 intentionally left an extension point, has a defect, or requires an agreed amendment. Escalate if semantic ownership would split.

## Class D — architecture contradiction

Examples: Array cannot be fixed without public growable storage, native floor cannot admit storage, or P1 forces compiler family inference. Stop and consult.

---

# 17. Detailed design rulings for likely implementation questions

- **Should published Array use `Vec<Value>`?** No. Private builder may.
- **Should Array reuse `ListObject`?** No. Share conventions/helpers, not representation identity.
- **Should Array have N?** No. That is Vector.
- **Should Array be immutable because `let` is immutable?** No. Binding immutability is not object immutability.
- **Should a frozen Array become hashable?** Not as a type-level P2 policy. Frozen state protects const aliases, not key admissibility.
- **Should `<T>[...]` be parsed directly as Array?** No. Semantic checker resolves family.
- **Should fixed literals call a public constructor?** No. Use direct, non-overrideable construction.
- **Should dynamic spread use List temporarily?** No.
- **Should dynamic spread know final length statically?** No.
- **Should `Array<_> = []` become `Array<Dynamic>`?** No. Underconstrained.
- **Should P2 change `[1,2,3]` default to Array?** No. P4 owns that cutover.
- **Should P2 implement const Array?** No. P4 owns static evaluation/materialization.

---

# 18. Native-floor decision content required before landing

If no existing accepted record already covers Array, the required decision should be narrow. It should establish approximately:

> `Array` is a dedicated native representation with fixed initialized `Value` storage. The irreducible native seam consists only of allocation/finalization support plus size/indexed read/indexed replacement. Higher sequence behavior remains Telos source. No structural resize primitive is admitted.

The rationale is that fixed dense `Value` storage is not expressible in Telos source without another backing collection; using List would violate semantic identity and fixed-extent representation; and the native seam is storage, not protocol.

The decision must not use P2 as an excuse to move equality, iteration, higher-order combinators, display, or conversion graph into Rust.

---

# 19. P2/P3 handoff obligations

P2 must leave P3 with:

1. a canonical sequence-family semantic product that already supports Array;
2. compiler family-controlled construction without AST inference;
3. an example of fixed dense `Value` storage;
4. a private dynamic builder pattern whose storage mechanics may be reusable;
5. no assumption that every sequence extent is runtime-only;
6. no generic cardinality solver polluted with Array concerns;
7. stable `Collection`/Iterator/Iterable integration.

P3 must not have to undo `Array<T,N>`, compiler head-arity dispatch, List staging, or a public generic sequence builder.

---

# 20. P2/P4 handoff obligations

P2 must leave P4 with:

1. stable Array runtime identity;
2. stable fixed storage;
3. stable direct fixed literal construction;
4. stable dynamic spread construction;
5. semantic family selection capable of Array context/head;
6. bare runtime List compatibility still active;
7. optional private frozen state ready for materialization;
8. no public const/freeze type split;
9. tests clearly separating explicit/contextual Array from compatibility bare List.

P4 then owns static sequence evaluation, static Array value, frozen materialization, alias mutation defense end-to-end, default switch, and migration.

---

# 21. Verification budget

Use three passes.

## BUILD

Make each coherent slice compile and prove its local proposition. Run frequently and narrowly.

## STABILIZE

Integrate adjacent seams and eliminate regressions across semantic Array + P1 tests, runtime Array + collection tests, GC/object model, compiler/disassembly, and native source/floor.

## CERTIFY

Run broad repository gates once architecture is stable. Do not spend most of implementation rerunning the full workspace after every local edit.

---

# 22. Required evidence for P2 completion

The walkthrough must contain concrete evidence:

- **E1 — Canonical type:** `Array<Int>` is canonical and one-parameter.
- **E2 — Representation:** ArrayObject uses fixed payload and passes object-size gate.
- **E3 — Mutation:** write preserves size.
- **E4 — No resize:** selector negative tests and no internal resize API.
- **E5 — Literal semantics:** contextual and explicit typed/hole forms resolve correctly.
- **E6 — Compatibility:** bare runtime literal still resolves to List.
- **E7 — Direct lowering:** fixed Array disassembly has Array construction and no List construction.
- **E8 — Dynamic spread:** spread disassembly/runtime uses private Array builder and no List builder.
- **E9 — GC:** forced-GC test including dynamic builder.
- **E10 — Equality/key law:** structural equality plus Map/Set rejection.
- **E11 — Predecessor regression:** P1 focused suite remains green.
- **E12 — Broader gate:** exact broad result, including baseline failures.

---

# 23. STOP AND CONSULT triggers

Escalate immediately if any of these occur:

1. P1 has not landed but implementation would require recreating its neutral literal architecture.
2. P1's checked sequence product cannot express Array without compiler-side family inference.
3. `Array<T>` cannot be formed without adding N.
4. a new TypeStore variant seems necessary for Array.
5. runtime Array appears to require per-object TypeId T.
6. published Array storage appears to require `Vec<Value>`.
7. the only dynamic spread design stages through List.
8. native Array storage cannot be admitted under current decision governance.
9. generic `InstanceObject` allocation is required for Array despite native representation.
10. adding Array forces a redesign of `Collection`.
11. adding Array forces a redesign of canonical Iterator/Iterable.
12. fixed literal construction can only be implemented by calling an overrideable public constructor.
13. dynamic spread requires holding a heap borrow across user iterator callbacks.
14. builder GC safety cannot be proven through current rooting architecture.
15. Array equality appears to require structural hashing for correctness.
16. const Array requires a separate public classifier.
17. bare runtime literals must switch to Array for tests to pass.
18. P3 static cardinality logic leaks into P2.
19. one principled correction fails to resolve the same semantic/inference defect.
20. the proposed fix would change unrelated collection semantics merely to make Array easier.

---

# 24. Completion checklist

## Lifecycle

- [ ] P2 marked ACTIVE only when started.
- [ ] start SHA recorded.
- [ ] P1 predecessor status recorded.
- [ ] native-floor authorization recorded.
- [ ] P2 walkthrough written.
- [ ] checkpoint updated with actual state.
- [ ] P3/P4 handoff written if required.

## Identity

- [ ] `UniverseKey::Array`.
- [ ] canonical `universe.collections.array`.
- [ ] source-owned identity.
- [ ] exposure/prelude behavior correct.
- [ ] `Array<T>` one-parameter type form.
- [ ] `Array -> Collection`.
- [ ] `CoreDeclarationIds::array`.

## Runtime

- [ ] `array_class`.
- [ ] `ArrayObject`.
- [ ] fixed payload.
- [ ] object-size gate.
- [ ] heap accessors.
- [ ] `Value::class`.
- [ ] GC trace.
- [ ] diagnostic rendering.

## Primitive floor

- [ ] authorization exists.
- [ ] size primitive.
- [ ] indexed read.
- [ ] indexed write.
- [ ] no resize primitive.
- [ ] frozen write guard if frozen seam landed.
- [ ] floor census updated.

## Source protocol

- [ ] size/read APIs.
- [ ] indexed replacement.
- [ ] no resize API.
- [ ] iterator.
- [ ] Iterable conformance.
- [ ] structural equality.
- [ ] cross-kind inequality.
- [ ] constructor policy follows ruling.

## Key law

- [ ] Array rejected by Map.
- [ ] Array rejected by Set.
- [ ] stale mutable-key wording corrected.

## Semantics

- [ ] contextual `Array<Int>`.
- [ ] contextual `Array<_>`.
- [ ] explicit `<Int>`.
- [ ] explicit `<_>`.
- [ ] typed empty Array.
- [ ] unresolved empty hole diagnosed.
- [ ] ordinary elements constrained to T.
- [ ] spread Item constrained to T.
- [ ] runtime spread cardinality accepted.
- [ ] bare runtime literal remains List.

## Compiler

- [ ] fixed Array direct lowering.
- [ ] no compiler family inference.
- [ ] missing semantic product fails closed.
- [ ] dynamic private builder.
- [ ] builder GC trace.
- [ ] no List staging.
- [ ] finalization directly to Array.

## Verification

- [ ] type identity tests.
- [ ] object-model tests.
- [ ] collection contract.
- [ ] mutation tests.
- [ ] equality tests.
- [ ] key rejection.
- [ ] fixed disassembly.
- [ ] spread disassembly.
- [ ] spread runtime tests.
- [ ] forced GC.
- [ ] P1 regression.
- [ ] collection regression.
- [ ] broad gate truthfully recorded.
- [ ] `git diff --check`.

---

# 25. Final implementer guidance

Keep one distinction in view throughout P2:

```text
Array is fixed after construction.
Array construction is not fixed-size during construction.
```

That is why the architecture has two representations:

```text
private growable builder
        ↓ finalize once
published fixed ArrayObject
```

The builder is implementation machinery. The Array is the language value. Do not erase that distinction by making Array growable temporarily, and do not replace it with a public List staging value.

Likewise, keep the type distinction explicit:

```text
Array<T>        runtime extent
Vector<T,N>     static extent
```

If any local implementation technique begins to blur those meanings, reject the technique rather than weakening the language model.

The desired end state of P2 is simple at the surface:

```telos
let a: Array<Int> = [1, 2, 3]
let b: Array<_> = [1, 2, 3]
let c = <Int>[1, 2, 3]
let d = <_>[1, 2, 3]

a[1] = 9
// size is still 3
```

while, during P2:

```telos
let legacy = [1, 2, 3]
// still List<Int>; P4 owns the default switch
```

Everything below that surface should converge on one semantic fact:

> `Array<T>` is a canonical, runtime-sized, fixed-extent, element-mutable collection, selected by semantic analysis and constructed directly by the runtime.

That is the P2 acceptance standard.
