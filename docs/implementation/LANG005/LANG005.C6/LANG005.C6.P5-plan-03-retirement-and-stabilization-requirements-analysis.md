---
id: LANG005.C6.P5.PART3.REQUIREMENTS
category: LANG
program: LANG005
checkpoint: LANG005.C6
plan: LANG005.C6.P5
part: 3-of-3
kind: part-requirements-analysis
analysis_pass: 4-of-4
status: FINAL
completion: COMPLETE
verification: REPOSITORY_INSPECTED_REMOTE_PART3_LOCAL_EXECUTION_UNAVAILABLE
prepared: 2026-09-17
repository: aureat/phalcom-lang
repository_branch: main
repository_revision: 06e37b6830cd9114d03f2a794d0b9a973f8cdc82
repository_revision_subject: "feat(lang005): complete P4 iterator iterable law staging"
shared_header: LANG005.C6.P5-plan-00-shared-header.md
predecessor_part: LANG005.C6.P5-plan-02-builtins-and-iterator-surface.md
scope_tasks:
  - T12
  - T13
  - T14
  - T15
scope_gates:
  - G5
  - G6
---

# LANG005.C6.P5 Part III — Retirement and Stabilization Requirements Analysis

> **Purpose:** derive the complete implementation requirements for the final P5 phase before writing `LANG005.C6.P5-plan-03-retirement-and-stabilization.md`.
>
> **This is not the implementation plan.** It defines what Part III must accomplish, why each requirement exists, which repository owners are involved, which architectural choices are already fixed, where the remaining uncertainty lies, and what evidence is required before P5 can be called complete.
>
> **Part III is destructive by design.** Parts I and II establish the replacement system. Part III is where the compatibility substrate is finally removed. That means the burden of proof is higher than in a normal cleanup: every deletion must follow a demonstrated consumer migration, every public-name change must preserve canonical trait identity, and every runtime shortcut that previously relied on cursor selectors must be replaced before those selectors disappear.

---

# 0. Executive conclusion

Part III is not a documentation sweep after the “real” iterator work. It owns three architectural closures that are required for P5 to have a single production model:

1. **Positional expansion must stop using runtime cursor-method discovery.** The current generic `*` path is still compiled as `iterate(None) -> iteratorValue(cursor) -> iterate(cursor)` and is admitted by `VM::has_iteration_protocol`, which checks for the two selector names at runtime. This is the last confirmed VM-level structural cursor proof path and therefore blocks deletion of `iterate` / `iteratorValue`.
2. **The legacy `Iterable` class is a real runtime identity, not only a source shim.** `CoreClasses`, `UniverseKey`, bootstrap sealing, primitive/invariant inventories, and six kernel collection superclass relations still encode `Iterable` as a native class. Deleting `collections/iterable.ph` without removing that identity would leave a ghost runtime hierarchy contradicting the trait model.
3. **Current-facing tooling and documentation still explain iteration through inheritance and cursor selectors.** The AST `for` comment, LSP hover text, guides, current specs, collection-spread documentation, and some REPL documentation still expose the retired mechanics. These surfaces must be cut over with the runtime/public-name transition.

The principal unresolved implementation risk is positional expansion of operands whose **argument shape is dynamic** and/or whose **type evidence is non-proven**. Dynamic call shape and iteration eligibility are separate concerns. P5 may keep dynamic rest-pack shape construction, but it may not use that as justification for runtime trait proof or structural `iterator`/`next` probing. If the final language contract requires genuinely dynamically typed arbitrary iterable expansion and the existing runtime cannot execute that contract from precomputed evidence, Part III must STOP/CONSULT rather than invent a second iteration authority.

A second integration risk is developer tooling after superclass removal. The semantic/LSP stack already has trait-aware editor member projection. The REPL snapshot/completion stack is still inheritance-oriented. Part III must not preserve a fake `Iterable` superclass simply to keep inherited completions, and it must not independently reimplement trait solving in the REPL.

Subject to those two seams, the final phase is mechanically well bounded: migrate expansion, migrate repository source consumers such as Sheetcalc `RefRange`, remove legacy classes/selectors and native identity, cut package/public names to the canonical traits, update current-facing tooling/docs/comments, run the final production census, and produce P5 completion records.

---

# 1. Authority and input hierarchy

Part III must be executed under the following authority order.

## 1.1 Primary architectural authority

1. `LANG005.C6.P5-plan-00-shared-header.md`
2. `LANG005.C6.P5-requirements-analysis.md`
3. `docs/specs/iterator.md`
4. P4 canonical `Iterator` / `Iterable` trait law in `phalcom-core/core/universe/src/collections/iterator_laws.ph`
5. ratified current callable/argument specifications, especially `docs/specs/callables/arguments.md`
6. actual live repository behavior at the implementation revision

## 1.2 Predecessor contracts

Part III additionally consumes the actual handoffs from:

- Part I — semantic iteration authority, lowering metadata, compiler trait-invocation execution, canonical `for` behavior;
- Part II — production built-in `Iterable` conformances, stateful iterators/adapters, defaults/terminals, eager wrappers, Map/Range/String decisions, and boundedness propagation.

The implementation plan must begin by reading the **actual completed handoff state**, not only these planning artifacts. If G4 did not prove a canonical replacement for every built-in/source behavior that Part III intends to delete, Part III is not authorized to perform that deletion.

## 1.3 Superseded historical authority

The following may explain why the current code looks the way it does, but may not override P5:

- ADR 0029 cursor protocol;
- ADR 0041 bare-cursor / runtime Iterable root;
- historical COLL001/COLL004 iterator-pipeline implementation documents that specify reusable stateless cursor descriptors;
- `../../../__unstructured__/spec/current/iteration.md` cursor wording where it conflicts with `docs/specs/iterator.md` and the final P5 requirements;
- `../../../__unstructured__/spec/iterator.md` where it reflects the pre-P5 class/cursor model.

Historical records should normally remain historical rather than being rewritten to pretend the old decision never existed.

---

# 2. Part-III entry preconditions

Part III may begin only after Part II G4 is green or explicitly classified as non-blocking according to the shared baseline policy.

The handoff must identify, at minimum:

```text
canonical iterator_laws module identity
actual iterator/adaptor source module paths
all core source Iterable conformance declarations
all core source Iterator conformance declarations
Map Item = (key: K, value: V) exact labeled tuple type
Map Entry legacy status
RangeIterator implementation and supported +1 subset
StringByteIterator / StringCodePointIterator implementation and Item types
final Iterator lazy-default selector inventory
final Iterable terminal/default selector inventory
final concrete eager-wrapper inventory
final boundedness propagation table
remaining iterate/iteratorValue hits intentionally deferred to Part III
remaining legacy `is Iterable` / legacy Iterator-class users
G3/G4 focused test outcomes
any approved deviations or consultation outcomes
```

Part III must not reopen the following architecture merely because retirement exposes a mechanical inconvenience:

- one-shot stateful iterator lifecycle;
- sticky exhaustion;
- `Iterator.iterator { self }` identity;
- reusable-source fresh iterator acquisition;
- Map direct item labeled tuple shape;
- Range forward-integer-only scope;
- String itself non-Iterable;
- String byte/codepoint view Item types;
- Iterator-vs-Iterable default ownership;
- direct built-in eager-vs-lazy selector policy;
- conservative `Bounded / Unbounded / Unknown` lattice;
- semantic proof / lowering / runtime anti-authority boundary from Part I.

Any need to alter those is a plan-set amendment, not a Part-III local decision.

---

# 3. Repository baseline re-grounding

The remote repository was rechecked during this analysis. `main` remains:

```text
06e37b6830cd9114d03f2a794d0b9a973f8cdc82
feat(lang005): complete P4 iterator iterable law staging
```

No implementation from Parts I or II is present at this planning baseline; therefore all Part-III file/path findings below describe the **pre-P5 live state**. The eventual implementation session must re-run the census after Parts I and II land and treat their handoffs as the local truth.

Important consequence: a cursor hit listed here may legitimately disappear before Part III begins. Part III should not recreate it merely because this analysis names it.

---

# 4. Final-phase architecture in one diagram

The desired end state is:

```text
source language
    |
    |  Iterable / Iterator conformance evidence
    v
phalcom-semantic
    |
    |  proven implicit iteration products
    |  associated Item / Iterator normalization
    |  selected / assumed / derived evidence
    v
ModuleLoweringSemantics
    |
    |  executable iterator() and next invocation specs
    v
compiler
    |
    |  direct call emission + Option branch/payload extraction
    v
VM
    |
    |  executes preselected calls only
    |  no trait proof
    |  no cursor selector discovery
    v
ordinary stateful iterator objects
```

And specifically for positional expansion:

```text
*source
  |
  +-- Unit  ------------------> direct empty positional lane
  |
  +-- Tuple ------------------> direct Tuple positional lane only
  |
  +-- canonical Iterable -----> semantic proof
                                 source.iterator once
                                 iterator.next repeatedly
                                 append yielded Item values
  |
  +-- no proof ---------------> semantic failure / approved dynamic policy
                                 NEVER cursor-method probing
                                 NEVER VM trait solving
```

Tuple is intentionally special even though Tuple is also an Iterable: `*tuple` contributes the Tuple's **positional argument lane**, which is not necessarily the same abstraction as “iterate every Tuple element”. The product/argument model therefore retains its direct fast lane.

---

# 5. Requirements taxonomy for this analysis

The global P5 requirements remain normative. To make the final implementation plan patch-grade, this document introduces **Part-III derived planning requirements** with `P3R-*` identifiers. These do not replace or renumber `P5-*`; they refine what the global requirements mean at the live repository seams discovered in this pass.

Families:

```text
P3R-EXP-*     positional expansion semantic/lowering/runtime migration
P3R-SRC-*     repository source consumer migration and cursor census
P3R-RT-*      runtime class/native metadata retirement
P3R-NAME-*    public declaration/module/name cutover
P3R-TOOL-*    LSP/REPL/reflection/tooling behavior after retirement
P3R-DOC-*     current documentation/comments/help cleanup
P3R-VERIFY-*  G5/G6 acceptance and final census
P3R-REC-*     completion records / checkpoint / walkthrough / handoff
```

---

# 6. T12 current state — positional expansion still owns a cursor runtime

The current generic positional expansion architecture is centralized enough to migrate cleanly, but it still directly violates the final P5 model.

## 6.1 Compiler owner

`phalcom-core/src/compiler/lib/expr.rs::compile_positional_expansion` currently:

1. checks boundedness;
2. evaluates the source once and stores it in a scratch local;
3. invokes a Tuple/Unit fast-lane probe;
4. on generic fallback, sends `iterate(None)`;
5. treats the returned cursor/`None` as loop state;
6. sends `iteratorValue(cursor)` for each item;
7. sends `iterate(cursor)` to advance;
8. appends each produced item into the argument-pack or List destination.

The helper is shared by positional expansion into:

- dynamic argument packs;
- List literal spread.

That shared ownership is desirable and should remain shared after migration.

## 6.2 VM owner

`phalcom-core/src/vm/dispatch.rs` currently gives `PackTryExpandTuplePositionals` and `ListTryExpandTuplePositionals` two logically separate responsibilities:

1. perform the direct Unit/Tuple positional-lane fast path;
2. if not Unit/Tuple, call `VM::has_iteration_protocol` to decide whether generic cursor fallback is permitted.

`has_iteration_protocol` interns and looks up:

```text
iterate(_)
iteratorValue(_)
```

on the runtime value.

This is not ordinary runtime execution of already-proven behavior. It is **runtime structural protocol discovery**, and therefore it must disappear before P5 can satisfy `P5-RT-13`, `P5-RT-14`, `P5-RETIRE-12`, and `P5-RETIRE-13`.

## 6.3 Semantic owner

`phalcom-semantic/src/checker/call.rs` currently represents `PackItem::Expand` as an `ApplicationArgument::Expansion` and correctly marks the final call shape as dynamic (`DynamicRestPack`) because expansion changes positional count at runtime.

However, no live semantic product was found that separately proves:

```text
expansion source |= Iterable
<source as Iterable>::Iterator |= Iterator
Item equality/projection
selected iterator invocation
selected next invocation
```

The operand expression is analyzed, but generic expansion eligibility is currently deferred to the compiler/VM cursor path.

This is the central T12 gap.

---

# 7. T12 core semantic requirement — separate dynamic call shape from iteration evidence

`P3R-EXP-01` — The implementation must distinguish **dynamic argument shape** from **iteration eligibility**.

An expansion can make the eventual selector/rest shape dynamic while its operand still has exact semantic type/conformance evidence. For example, a statically known `List<Int>` produces a dynamic number of positional arguments but is not semantically “unknown” as an iterable source.

Therefore:

```text
DynamicReason::DynamicRestPack
```

must not be interpreted as permission to perform runtime trait/protocol discovery.

`P3R-EXP-02` — Positional expansion must publish or reuse a semantic implicit-iteration resolution product for non-Unit/non-Tuple sources.

The product should be keyed by stable source expression identity and carry the same essential facts as a Part-I `for` lane:

```text
source ExpressionId / SourceRange
source normalized type
proven Iterable evidence
normalized Item type
normalized Iterator type
projected Iterator conformance/equality evidence
iterator() invocation authority
next invocation authority
dependency/fingerprint provenance
```

The exact Rust type may differ from the Part-I lane structure. The important rule is **one semantic authority, not duplicate proving logic**.

`P3R-EXP-03` — Prefer extracting/reusing a common internal “resolve implicit iteration source” routine from Part I rather than copying the proof sequence into call analysis.

The published products may remain distinct:

```text
ForIterationLaneResolution
PositionalExpansionIterationResolution
```

because their source-site ownership and downstream consumers differ. Their proof semantics must not diverge.

`P3R-EXP-04` — Expansion proof must support the same evidence classes already supported by canonical `for`:

- exact conformance;
- generic assumed conformance;
- derived `Iterator => Iterable` evidence;
- abstract requirement execution where already supported by the trait/lowering architecture.

Derived evidence must reuse the existing P3/P4 derived environment transport; it must not be mislabeled as an exact `TraitDispatchSite`.

`P3R-EXP-05` — A structural lookalike that merely defines `iterator` / `next`, or old `iterate` / `iteratorValue`, is not a canonical iterable expansion source without conformance evidence.

This closes the runtime-duck-typing loophole at the same time the cursor probe is removed.

---

# 8. T12 direct product lanes — Unit and Tuple remain special

`P3R-EXP-06` — `Unit` positional expansion remains a direct zero-value lane.

No Iterable proof should be required simply to expand `()`.

`P3R-EXP-07` — `Tuple` positional expansion remains a direct product-lane operation and contributes only the Tuple positional lane.

Do not route `*tuple` through `Tuple.iterator` merely because Tuple now conforms to Iterable. A complete Tuple can carry labeled entries, and the callable argument specification explicitly gives `*` only the positional lane.

`P3R-EXP-08` — The Unit/Tuple fast path may remain a VM bytecode/helper, but after T12 its semantics must be limited to product-lane detection/copying.

Acceptable end shape:

```text
TryAppendTuplePositionals(source, target)
    -> true  if Unit/Tuple was handled
    -> false otherwise
```

Unacceptable end shape:

```text
false otherwise IF runtime value responds to iterator protocol
error otherwise
```

Whether the existing bytecode names are retained or mechanically renamed is implementation-flexible. Their authority must change.

`P3R-EXP-09` — `RuntimeError::NonIterableStarOperand` must be re-audited after semantic migration.

If no approved runtime-dynamic expansion path still emits it, remove or retire the dead error rather than retaining unreachable cursor-era validation. If an approved dynamic path genuinely needs the runtime error, document the exact authority that reaches it.

---

# 9. T12 lowering requirement — expansion gets executable iteration metadata

`P3R-EXP-10` — Proven positional expansion iteration must be projected into `ModuleLoweringSemantics` rather than rediscovered by the compiler.

Recommended conceptual shape:

```rust
struct PositionalExpansionLoweringSpec {
    item_type: TypeId,
    iterator_type: TypeId,
    iterator: IterationInvocationSpec,
    next: IterationInvocationSpec,
}
```

This may reuse the Part-I executable invocation enum/type if the ownership fits.

`P3R-EXP-11` — Expansion lowering should have a dedicated lowering-site identity or dedicated map keyed by source site. Do not fabricate source AST method calls to make existing expression lowering accidentally apply.

`P3R-EXP-12` — Expansion lowering must participate in the same dependency/fingerprint regime as the semantic evidence that produced it. Changes to source type, Iterable/Iterator conformance, associated `Item` / `Iterator`, selected witnesses/defaults, or derived evidence must invalidate the relevant expansion lowering.

`P3R-EXP-13` — Body-only changes unrelated to the expansion proof should not force evidence churn beyond the repository's existing dependency model.

This is the expansion analogue of the Part-I incremental requirement; Part III should reuse existing machinery rather than design a new cache.

---

# 10. T12 compiler requirement — canonical stateful expansion loop

For the generic Iterable branch, the compiler target is:

```text
evaluate source exactly once
invoke selected Iterable.iterator exactly once
loop:
    invoke selected Iterator.next exactly once
    if None: finish expansion
    if Some(item): append payload
    repeat
```

`P3R-EXP-14` — Iterator acquisition occurs once per expansion occurrence.

For a reusable source, that obtains a fresh traversal. For an Iterator source using derived Iterable conformance, `iterator` aliases the current iterator object and expansion consumes its current state.

`P3R-EXP-15` — Each attempted element invokes `next` exactly once.

Do not call `next` once to test and again to retrieve.

`P3R-EXP-16` — `Option<Item>` must be eliminated by canonical option/tag semantics, preserving `Some(None)` when `Item` itself can contain `None`.

Use the same proven-Some payload extraction discipline established in Part I. Do not revive `unwrapOr` as an iteration sentinel.

`P3R-EXP-17` — Iterator exceptions/errors propagate as ordinary execution failures. Exhaustion alone is `None`.

`P3R-EXP-18` — Fiber suspension/yield through source `iterator` / `next` or downstream append-safe user code must preserve the existing direct VM activation semantics. Do not introduce a synchronous native loop over an Iterable.

`P3R-EXP-19` — The source scratch local and iterator scratch local must remain GC roots for the whole expansion loop, consistent with current compiler scratch-local discipline.

---

# 11. T12 evaluation-order and boundedness invariants

`P3R-EXP-20` — Preserve the argument-expansion rule that the operand expression is evaluated exactly once at its source position.

`P3R-EXP-21` — Preserve the current eager contribution ordering: a generic positional expansion is fully exhausted before evaluation proceeds to a later argument/literal entry.

Do not transform expansion into a lazy deferred pack contribution.

`P3R-EXP-22` — Preserve encounter order from the canonical iterator.

`P3R-EXP-23` — Continue to use the existing boundedness service before eager generic expansion.

- proven `Unbounded` => compile-time rejection;
- `Unknown` => legal;
- `Bounded` => legal.

Unknown boundedness is not the same thing as unknown iterability/type evidence.

`P3R-EXP-24` — No hidden runtime element cap or safety truncation may be introduced for Unknown sources.

`P3R-EXP-25` — List-literal positional spread and outgoing argument-pack positional spread must share the same canonical Iterable proof/execution semantics after the Unit/Tuple special lane.

---

# 12. T12 dynamic-type boundary — mandatory consultation seam

This is the most important STOP/CONSULT condition in Part III.

The callable system correctly has a runtime dynamic-pack shape because expansion changes arity. That does **not** answer what happens if the expansion operand's type itself is Dynamic/Unknown and no canonical Iterable evidence can be proven.

`P3R-EXP-26` — Part III must not implement any of the following as a fallback:

```text
runtime lookup of iterator / next by selector name
runtime lookup of iterate / iteratorValue
VM scan of conformance tables
runtime semantic solver invocation
"respondsTo"-style structural acceptance
special-case acceptance because the value's native class used to inherit Iterable
```

`P3R-EXP-27` — If the ratified language surface requires arbitrary Dynamic positional expansion to remain legal, implementation must identify an **already-ratified executable capability token/evidence representation** that can accompany the value. If no such capability exists, STOP AND CONSULT.

`P3R-EXP-28` — It is acceptable for the final static semantic model to reject a source that cannot prove `Iterable` if that follows the final P5 trait-evidence contract. It is not acceptable for Luna to silently choose this language behavior merely to simplify implementation. If current tests/specs require runtime-dynamic duck expansion, the architect must resolve the design conflict.

The Part-III implementation plan should therefore make T12 begin with a targeted classification test matrix before deleting the old VM probe.

---

# 13. T13 repository-source migration — close every live legacy consumer

Part III must not assume the Universe is the only user of the old class/cursor protocol.

`P3R-SRC-01` — Perform a live source census after G4 for:

```text
class ... is Iterable
class ... is Iterator
iterate(
iteratorValue(
SourceIterator
IndexedIterable
ZippedIterable
MapIterator / FilterIterator / old cursor adapter class names
$for_cursor
has_iteration_protocol
PackTryExpandTuplePositionals / ListTryExpandTuplePositionals generic fallback
```

Class-name searches must distinguish final stateful classes from cursor-era classes with the same or similar names.

`P3R-SRC-02` — Every live production source consumer must be migrated before deleting its legacy provider.

No “it probably is only an example” exception: repository examples are executable/reference code and are part of the final census unless explicitly archival.

---

# 14. T13 confirmed external consumer — Sheetcalc `RefRange`

The live repository currently contains:

```text
examples/sheetcalc/src/grid/ref-range.ph
```

with:

```phalcom
class RefRange is Iterable
```

and an `iteratorValue(_ cursor)` method relying on inherited legacy `Iterable.iterate`.

`P3R-SRC-03` — Migrate `RefRange` to explicit canonical trait conformance.

The final semantic shape should be equivalent to:

```text
RefRange |= Iterable
RefRange.Item = Ref
RefRange.Iterator = RefRangeIterator
RefRange.iterator -> fresh RefRangeIterator
RefRangeIterator |= Iterator
RefRangeIterator.Item = Ref
RefRangeIterator.next -> Option<Ref>
```

`P3R-SRC-04` — Preserve row-major encounter order and normalized rectangle semantics.

The iterator should retain whatever source state is required to compute row/column progression without reintroducing a cursor token protocol.

`P3R-SRC-05` — Reusable `RefRange.iterator` calls must return independent traversals.

`P3R-SRC-06` — `RefRangeIterator` must be sticky after exhaustion.

`P3R-SRC-07` — Existing `size`, `contains`, `topLeft`, `bottomRight`, parsing, and display behavior are not P5 redesign targets.

`P3R-SRC-08` — Update live Sheetcalc documentation/comments that say “extends Iterable” or explain inherited `iterate` / `iteratorValue` behavior.

---

# 15. T13 final source census and deletion readiness

Before T14 can begin, T13 must prove all of the following:

`P3R-SRC-09` — No live production class relies on legacy `Iterable` inheritance for traversal/default algorithms.

`P3R-SRC-10` — No live production class relies on the legacy source `Iterator` class for lazy behavior.

`P3R-SRC-11` — No live production implementation defines `iterate` / `iteratorValue` as its canonical traversal contract.

`P3R-SRC-12` — No compiler/runtime path emits/sends those selectors for iteration.

`P3R-SRC-13` — Any remaining textual hits are classified into:

```text
historical accepted ADR / historical implementation record
superseded document with explicit historical context
negative test proving old surface is unavailable
unrelated use of the English word cursor
bug / missed migration
```

Only the first four categories may survive G6.

---

# 16. T14 live runtime identity — `Iterable` is embedded in the kernel class graph

At the inspected baseline, `Iterable` exists in all of the following runtime-authoritative places:

```text
phalcom-core/src/universe/core_classes.rs
    CoreClasses.iterable_class
    make_core_class("Iterable", Object)
    List/Map/Set/Tuple/Range/Bytes initially wired through it
    UniverseKey::Iterable resolution

phalcom-native-meta/src/universe.rs
    UniverseKey::Iterable
    UNIVERSE_CLASS_RELATIONS: Iterable -> Object
    List/Map/Set/Tuple/Range/Bytes -> Iterable
    source_path/name mappings

phalcom-core/src/vm/bootstrap.rs
    add_class!(iterable_class) / sealed-class registration

phalcom-core/src/universe/invariants.rs
    core class inventory row for Iterable
    native relation verification

phalcom-core/src/universe/primitives.rs
    core-class/native inventory participation
```

This confirms that T14 is a runtime identity migration, not merely source-file deletion.

---

# 17. T14 kernel superclass retirement requirements

`P3R-RT-01` — After canonical trait conformances are proven, kernel collection runtime inheritance must cease to encode iterability.

Default target:

```text
List  -> Object
Map   -> Object
Set   -> Object
Tuple -> Object
Range -> Object
Bytes -> Object
```

If a newer class-specific normative superclass exists at implementation time, use that instead. Do not invent an `IterableBase`, `SequenceBase`, or other hidden inheritance replacement.

`P3R-RT-02` — Update both initial construction and canonical native-meta relation authority. It is insufficient to change only the `make_core_class` parent because `UNIVERSE_CLASS_RELATIONS` later applies the canonical graph.

`P3R-RT-03` — Preserve the metaclass parallel-superclass invariant automatically through the existing class-relation machinery.

Do not manually special-case metaclass parents for this migration.

`P3R-RT-04` — Class identity for List/Map/Set/Tuple/Range/Bytes themselves remains stable. P5 removes only the legacy Iterable intermediary, not the collection classes.

`P3R-RT-05` — Native primitive binding must remain attached to the concrete classes that own those primitives. Removing Iterable must not redistribute primitive methods into traits or Object.

---

# 18. T14 complete native identity removal matrix

`P3R-RT-06` — Remove `CoreClasses.iterable_class` and every resolver/inventory use after no consumer remains.

`P3R-RT-07` — Remove `UniverseKey::Iterable` from `phalcom-native-meta` after the legacy native class is no longer canonical.

`P3R-RT-08` — Rewrite six native superclass relations away from `UniverseKey::Iterable`.

`P3R-RT-09` — Remove bootstrap sealed/global class handling for the legacy native Iterable class.

`P3R-RT-10` — Remove invariant/floor/core-class inventory rows that assert Iterable is a runtime class.

`P3R-RT-11` — Remove primitive/native-surface inventory references whose only purpose is the legacy class identity.

`P3R-RT-12` — Audit reflection/native metadata that assumes “is Iterable” can be answered by superclass ancestry. Iterability after P5 is trait conformance evidence, not `isA(Iterable)`.

`P3R-RT-13` — No tombstone native class named Iterable may remain merely to preserve old `ClassId` ordering unless an external ABI explicitly requires it. No such ABI authority was identified in this analysis; discovering one is STOP/CONSULT.

---

# 19. T14 legacy source module/class retirement

At baseline, `phalcom-core/core/universe/src/collections/iterable.ph` contains the old:

- `@native class Iterable<T>`;
- cursor defaults;
- `IndexedIterable`;
- `ZippedIterable`;
- source `class Iterator is Iterable`;
- `SourceIterator`;
- cursor-based lazy adapter classes.

Part II is expected to move canonical behavior elsewhere while deliberately retaining this file as migration runway.

`P3R-RT-14` — T14 may delete or empty-retire `collections/iterable.ph` only after T12/T13 prove no live consumer remains.

Preferred end state is deletion of the obsolete module rather than an empty compatibility shell.

`P3R-RT-15` — Delete the legacy source `Iterator` class and all cursor adapter classes not reused as final stateful classes.

If Part II intentionally reused a class name (for example `FilterIterator`) for a new stateful implementation in another module, final census must distinguish by defining module and semantics.

`P3R-RT-16` — Do not keep `SourceIterator` as a compatibility allocation. Canonical `.iter`/`.iterator` semantics are already owned by the final traits/source implementations.

`P3R-RT-17` — Delete `IndexedIterable` / `ZippedIterable` only after their final canonical replacements from Part II are proven and every call site has migrated.

---

# 20. T14 public name and module topology cutover

At baseline:

```phalcom
// collections/package.ph
expose .iterable
expose .iterator_laws
...
```

and the builtin module graph contains both children.

`P3R-NAME-01` — Final public `Iterator` must resolve to the P4 canonical trait declaration, not a source class.

`P3R-NAME-02` — Final public `Iterable` must resolve to the P4 canonical trait declaration, not a native/source class.

`P3R-NAME-03` — Preserve the canonical declaration identities from `collections.iterator_laws` unless a proven module-system blocker requires relocation.

Do not move/rename traits simply so the filename looks prettier.

`P3R-NAME-04` — Once `collections.iterable` has no migration consumer, stop exposing it from `collections/package.ph`.

`P3R-NAME-05` — Remove the obsolete `collections.iterable` node/source mapping from `phalcom-modules/src/builtin.rs` if the module is deleted.

`P3R-NAME-06` — Update the `collections` package child list accordingly.

`P3R-NAME-07` — Package-level exposure of `Iterator` / `Iterable` must resolve to the trait declarations without same-scope ambiguity.

`P3R-NAME-08` — The prelude/root convenience policy, if it exposes these names at the implementation revision, must point to the same canonical trait declarations. Do not create aliases with new semantic identity.

`P3R-NAME-09` — Import/definition/hover tests should verify declaration identity, not merely that a string named `Iterable` is present.

---

# 21. T14/T15 reflection and editor semantics after superclass removal

The final architecture changes what it means to explain why a collection has iterator/default behavior.

Before P5:

```text
List
  superclass -> Iterable
  inherited method -> map / each / ...
```

After P5:

```text
List
  superclass -> Object
  explicit Iterable conformance
  effective trait default / inherent witness selected semantically
```

`P3R-TOOL-01` — Reflection/editor output must not continue to claim that List/Map/etc. inherit traversal behavior from an Iterable superclass.

`P3R-TOOL-02` — Trait conformance/effective behavior must be explained through existing trait/conformance products.

No fake superclass edge may be retained for tooling convenience.

`P3R-TOOL-03` — LSP member completion/hover/definition must continue to use `phalcom-semantic` editor projections and the canonical trait resolver. Do not add a second LSP trait solver.

The C4 implementation already established trait-aware editor member projection; Part III should leverage that architecture.

`P3R-TOOL-04` — Trait defaults/conformance-local witnesses must remain absent from target runtime class dictionaries if that is their established representation. Do not copy defaults into concrete classes just so runtime reflection/completion can find them.

This preserves the C4 anti-authority/runtime representation contract.

---

# 22. REPL-specific tooling seam

The REPL has a distinct architecture from the semantic LSP:

```text
ReplSnapshot
    captures live runtime class dictionaries
    walks superclass chain
    tags members with own_depth

ReplOracle
    sorts completion candidates by (own_depth, name)
```

The current implementation test only requires List to have some inherited members, so `Object` can satisfy the raw depth assertion after Iterable removal. However, historical REPL docs explicitly describe Iterable's methods as an inheritance tier.

`P3R-TOOL-05` — Update REPL documentation that uses `List -> Iterable -> Object` as the effective member model.

`P3R-TOOL-06` — Audit whether current REPL completion promises trait-default/effective trait members for canonical typed values.

If yes, removing the superclass will reveal a functional gap, because trait defaults are intentionally detached from class dictionaries.

`P3R-TOOL-07` — If REPL effective-trait completion is required, it must consume/reuse the existing semantic/editor authority or another already-canonical semantic product. It must not infer conformance from selector coincidence or implement a REPL-only solver.

`P3R-TOOL-08` — If wiring semantic trait projection into the REPL would require a material REPL architecture expansion beyond P5's accepted scope, STOP AND CONSULT with a concrete compatibility impact report. Do not preserve `iterable_class` as a workaround.

`P3R-TOOL-09` — Runtime inheritance completion may continue to show genuine Object/inherent members; only the fake trait-via-superclass explanation must disappear.

---

# 23. Generated/static tooling tables

A repository note indicates `tools/vsphalcom/src/generated/core-table.json` is stale and separate regeneration work has existed historically.

`P3R-TOOL-10` — Do not expand Part III into a wholesale generated core-table regeneration unless live current tooling/tests prove the table is an authoritative consumer of the removed Iterable class.

`P3R-TOOL-11` — Classify every generated/static metadata hit by actual ownership:

```text
live authoritative input -> update in P5
live generated derivative with canonical generator -> regenerate narrowly
known stale/deferred artifact not consumed by current stack -> document, do not opportunistically rewrite
historical snapshot -> leave historical
```

This prevents runtime-class retirement from becoming an unrelated tooling modernization project.

---

# 24. T15 current-facing documentation and source comments

Confirmed current-facing stale surfaces include, at the inspected baseline:

```text
phalcom-ast/src/ast.rs
    For statement comment names iterate/iteratorValue

phalcom-lsp/src/hover.rs
    hard-coded `for` hover teaches cursor protocol

docs/spec/current/iteration.md
    current cursor protocol description

docs/spec/current/control-flow.md
    cursor lowering description

docs/guide/collections.md
    two-selector iterability table / structural opt-in

docs/guide/control-flow.md
    cursor lowering explanation

docs/spec/collections/06-rest-spread-and-pack-operators.md
    `*` generic cursor expansion wording

docs/wiki/collections/traversal.md
    cursor/pipeline mechanics where presented as current

docs/spec/iterator.md
    competing older iterator document requiring classification

phalcom-core/tests/core/language/corpus.rs
    iteration/disassembly comments name old selectors

examples/sheetcalc docs/comments
    inherited legacy Iterable explanation
```

`P3R-DOC-01` — Every current-facing description of `for` must explain canonical `Iterable.iterator` + stateful `Iterator.next`, while preserving the direct-jump compiler/control-flow property where relevant.

`P3R-DOC-02` — Every current-facing description of user-defined iterability must require explicit trait conformance and associated-type law, not method coincidence.

`P3R-DOC-03` — Positional expansion docs must retain the Unit/Tuple lane special cases and describe generic source expansion through canonical iteration without naming deleted cursor selectors.

`P3R-DOC-04` — Current docs must describe reusable-source freshness versus one-shot Iterator identity/exhaustion.

`P3R-DOC-05` — Update source comments in AST/compiler/tests/LSP that would mislead future maintainers even if they do not affect runtime behavior.

`P3R-DOC-06` — Do not insert planning IDs, “temporary for P5”, agent directions, or implementation-history narration into production library comments.

---

# 25. Historical documentation policy

`P3R-DOC-07` — Accepted historical ADRs that chose cursor iteration remain historical records.

Do not rewrite ADR 0029/0041 decision bodies to describe the new system as if it had always existed.

`P3R-DOC-08` — Where repository convention supports supersession metadata/notes, it is acceptable to mark historical cursor decisions as superseded by the newer canonical iteration specification. The note should preserve the old decision and point to the new authority.

`P3R-DOC-09` — Historical implementation plans/walkthroughs may retain cursor names because they document what was implemented at the time.

`P3R-DOC-10` — A final grep hit in a historical file is acceptable only if the file's historical status is unambiguous. A file located under a “current” or guide/wiki current-facing path must be updated or reclassified.

---

# 26. G5 retirement gate requirements

G5 is not merely “tests pass after deleting Iterable”. It is the authorization barrier between destructive retirement and final stabilization.

G5 must prove:

`P3R-VERIFY-01` — T12 generic positional expansion no longer emits or discovers `iterate` / `iteratorValue`.

`P3R-VERIFY-02` — VM contains no iteration-protocol proof helper such as `has_iteration_protocol`.

`P3R-VERIFY-03` — Unit/Tuple positional expansion still preserves product-lane semantics.

`P3R-VERIFY-04` — statically proven Iterable expansion works for:

- concrete source;
- generic assumed `Iterable` source;
- `Iterator` source through derived `Iterable` conformance;
- source whose Item can itself be Option/None-like data;
- List-literal spread and outgoing pack spread.

`P3R-VERIFY-05` — structural lookalike without conformance is not accepted as a canonical generic iterable source.

`P3R-VERIFY-06` — boundedness still rejects proven-unbounded eager expansion and permits Unknown.

`P3R-VERIFY-07` — repository live source has no legacy `is Iterable` / cursor implementation dependency.

`P3R-VERIFY-08` — `collections.iterable` has no remaining live migration consumer.

`P3R-VERIFY-09` — `CoreClasses`, `UniverseKey`, bootstrap, primitives, invariants, and native class relations contain no legacy runtime Iterable identity.

`P3R-VERIFY-10` — List/Map/Set/Tuple/Range/Bytes have their final non-iteration runtime superclass and still execute their canonical trait behavior.

`P3R-VERIFY-11` — public `Iterator` / `Iterable` resolve to canonical traits with no class ambiguity.

`P3R-VERIFY-12` — no trait-default/witness execution was achieved by copying methods into class dictionaries or by adding VM proof search.

Only after these are true should T15 perform the broad current-facing cleanup/final census.

---

# 27. G6 final focused stabilization requirements

G6 is the final focused P5 acceptance gate, not a claim that every repository-wide test in unrelated programs is green.

It must combine:

## 27.1 Semantic evidence

- canonical for-lane evidence from Part I remains green;
- positional-expansion evidence is green;
- exact / assumed / derived conformance cases remain green;
- associated Item/Iterator equality remains green;
- incremental/fingerprint iteration cases remain green.

## 27.2 Core/runtime evidence

- stateful source iterators/adapters remain green;
- direct `for` and expansion contain no cursor sends;
- strict zip mismatch/error behavior remains green;
- Map tuple Item behavior remains green;
- Range and String views remain green;
- GC nested/adaptor tests remain green;
- no runtime trait proof/conformance search is introduced.

## 27.3 Tooling evidence

- LSP hover/current semantic editor behavior uses final terminology;
- definition/completion identity for Iterable/Iterator is canonical;
- REPL behavior is classified and either verified against final semantics or escalated according to `P3R-TOOL-08`;
- no current-facing tool explains iterability by the deleted superclass.

## 27.4 Documentation/census evidence

- final current docs/guides/comments updated;
- final production cursor grep classified clean;
- final runtime-class/public-name grep classified clean;
- historical hits explicitly remain historical.

## 27.5 Repository hygiene

- `git diff --check`;
- focused affected crate `cargo check` / tests according to shared verification budget;
- no unrelated broad formatter churn;
- no temporary probes or compatibility shims left behind.

---

# 28. Global P5 coverage obligations owned or closed by Part III

The final requirements analysis assigns the following existing global coverage rows directly to Part III.

| Coverage ID | Part-III acceptance meaning |
|---|---|
| `P5-CV-58` | disassembly/runtime products show no cursor sends in canonical loop/expansion paths |
| `P5-CV-59` | no trait proof/runtime lookup in iteration hot paths; T12 must not replace cursor probing with trait probing |
| `P5-CV-64` | repository source legacy Iterable subclass such as `RefRange` is migrated |
| `P5-CV-65` | final repository production search has no cursor protocol implementation/use |
| `P5-CV-66` | final public `Iterable` / `Iterator` resolve to canonical traits |
| `P5-CV-93` | final rich iterator/default surface still requires no VM proof or native combinator floor after superclass deletion |
| `P5-CV-112` | positional `*` no longer depends on `iterate` / `iteratorValue` lookup |
| `P5-CV-113` | CoreClasses/bootstrap/invariants/native-meta census contains no `iterable_class` identity |
| `P5-CV-114` | current LSP hover / AST comments use iterator/next semantics, not cursor protocol |
| `P5-CV-115` | final production grep finds cursor names only in explicitly historical/superseded material |

Part III also acts as a regression gate for Part-I/II coverage that can be invalidated by deletion, notably trait dispatch, defaults, GC, boundedness, Map iteration, Range, String views, one-shot adapter behavior, and strict zip.

---

# 29. Part-III requirements-to-global traceability

| Part-III family | Primary global requirements |
|---|---|
| `P3R-EXP-*` | `P5-RT-01..14`, `P5-RETIRE-01/02/12/13`, `P5-PERF-*`, boundedness requirements |
| `P3R-SRC-*` | `P5-RETIRE-01/04/09/12/13`, collection/source requirements, `P5-CV-64/65` |
| `P3R-RT-*` | `P5-RETIRE-02/05/06/07/10/11`, `P5-RT-01`, `P5-CV-113` |
| `P3R-NAME-*` | `P5-NAME-01..06`, `P5-RETIRE-11`, `P5-CV-66` |
| `P3R-TOOL-*` | `P5-RETIRE-05/08/14`, `P5-NAME-06`, `P5-CV-114` |
| `P3R-DOC-*` | `P5-DOC-01..06`, `P5-RETIRE-03/12/14`, `P5-CV-115` |
| `P3R-VERIFY-*` | G5/G6, `P5-CV-58/59/64/65/66/93/112/113/114/115` |
| `P3R-REC-*` | checkpoint/walkthrough/handoff completion requirements in shared header |

---

# 30. Exact repository impact map for the implementation plan

This is a requirements-analysis impact map, not blanket permission to edit every file.

## 30.1 T12 likely owners

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/context.rs and/or shared implicit-iteration helper from Part I
phalcom-semantic/src/checker/analysis.rs if a new published index is needed
phalcom-semantic/src/db/fingerprint.rs if publication fingerprinting needs explicit inclusion

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs if derived-evidence projection needs the same recipe path as Part I
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/scope.rs or shared trait-invocation emitter if needed
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/error.rs only if NonIterableStarOperand becomes dead/changes owner
phalcom-core/src/bytecode.rs only if the fast-lane opcode is renamed/recontracted
```

Tests likely include semantic call/expansion tests plus:

```text
phalcom-core/tests/core/collections/outgoing_packs.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
phalcom-core/tests/fixtures/language/collections/*
phalcom-core/tests/fixtures/language/collections/negative/*
compiler disassembly/lowering tests as appropriate
```

## 30.2 T13 likely owners

```text
examples/sheetcalc/src/grid/ref-range.ph
examples/sheetcalc tests / fixtures that cover ranges
live example/application files found by final census
source comments in those files
```

## 30.3 T14 likely owners

```text
phalcom-core/core/universe/src/collections/iterable.ph       # delete/retire
phalcom-core/core/universe/src/collections/package.ph
phalcom-modules/src/builtin.rs

phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-native-meta/src/universe.rs

any live reflection/prelude/native-surface maps discovered by exact census
```

## 30.4 T15 likely owners

```text
phalcom-ast/src/ast.rs
phalcom-lsp/src/hover.rs
semantic/LSP tests for final public trait identity and effective members
phalcom-repl docs/tests/code only if runtime-class removal exposes real behavior changes

docs/spec/current/iteration.md
docs/spec/current/control-flow.md
docs/guide/collections.md
docs/guide/control-flow.md
docs/spec/collections/06-rest-spread-and-pack-operators.md
docs/wiki/collections/traversal.md
docs/spec/iterator.md                                  # classify old/competing status
current navigation/index files that link the wrong authority

phalcom-core/tests/core/language/corpus.rs              # stale explanatory comments
examples/sheetcalc current docs

LANG005.C6-CHECKPOINT.md
LANG005.C6.P5-walkthrough.md
LANG005.C6.P5-handoff.md
```

---

# 31. STOP/CONSULT matrix for Part III

Part III must stop and consult rather than improvise if any of the following occurs.

| Trigger | Why it is architectural |
|---|---|
| Dynamic positional expansion cannot execute without runtime trait/method discovery | Would violate `P5-RT-14` and create a second authority |
| A live consumer requires cursor selectors after canonical migration | Replacement architecture is incomplete; deletion would be premature |
| Removing `iterable_class` breaks a class-layout/ABI/class-ID invariant outside documented runtime metadata | Runtime representation scope expands beyond planned class-edge removal |
| A collection has a newer normative superclass other than Object | Must respect newer class hierarchy authority |
| Public trait names cannot be exposed without moving/duplicating P4 declarations | Risks declaration-identity churn and duplicate semantic authority |
| REPL trait-default completion requires a new independent trait solver | Tooling must reuse semantic authority, not fork it |
| LSP final trait behavior is only available through class dictionaries | Contradicts established C4 semantic editor architecture; investigate before workaround |
| Positional expansion proof requires general callable/generic-inference redesign | T12 should reuse existing proof machinery, not absorb unrelated typing work |
| Removing old runtime error/opcode requires broad bytecode ABI migration | Rename/removal must be justified; semantic authority change is required, cosmetic opcode churn is not |
| Final grep reveals cursor mechanics in another live compiler/VM subsystem | Must classify and extend the retirement ledger before claiming completion |
| Existing Part-I/II handoff differs materially from this requirements analysis | Re-ground and amend; do not implement against stale architecture |
| Local baseline has materially changed from `06e37b6` | Re-audit impacted seams before destructive deletion |

---

# 32. Risk register

## R1 — dynamic expansion semantics

**Severity:** RED
**Likelihood:** medium
**Owner:** T12 / architect consultation if triggered

Current runtime behavior accepts generic expansion based on method lookup. Canonical P5 forbids that proof mechanism. The implementation plan must force this question to resolution before deleting the old probe.

## R2 — public name / module identity collision

**Severity:** RED if encountered
**Likelihood:** low-to-medium

Both `collections.iterable` and `collections.iterator_laws` are currently exposed. The final traits should retain P4 identity. If package exposure cannot present them without relocation/duplication, consultation is mandatory.

## R3 — runtime native-meta class graph

**Severity:** high
**Likelihood:** certain work, low uncertainty

`UniverseKey::Iterable` is canonical native metadata today. Removal must be synchronized across class creation, relation application, invariants, bootstrap, and primitive/native inventories.

## R4 — REPL effective-member degradation

**Severity:** medium/high
**Likelihood:** unknown until G5 audit

The REPL member snapshot is superclass based, whereas trait defaults are detached. Runtime superclass deletion may change completion behavior even if execution is correct.

## R5 — stale current documentation masquerading as authority

**Severity:** medium
**Likelihood:** high

There are multiple current/guide paths teaching cursor semantics. Final grep must classify current versus historical, not merely count hits.

## R6 — cursor names retained inside tests/comments

**Severity:** low functionally, high for P5 completion truth

Tests can remain green while comments/disassembly expectations teach deleted mechanics. T15 must update these deliberately.

## R7 — over-expansion into generated tooling/core tables

**Severity:** medium
**Likelihood:** medium

A stale generated core table exists. Do not turn P5 into unrelated regeneration unless current consumers require it.

---

# 33. What the Part-III implementation plan must decide mechanically

The next planning turn should be able to write T12–T15 without reopening language design. It still must pin the following implementation mechanics after reading the actual Part-I/II landed code:

1. exact shared semantic helper used to prove implicit iteration for `for` and expansion;
2. exact published `PositionalExpansionIterationResolution` shape or reuse strategy;
3. exact `ModuleLoweringSemantics` key/map and invocation-spec type reused from Part I;
4. exact compiler helper for emitting selected/assumed/derived iterator/next calls;
5. whether `PackTryExpandTuplePositionals` / `ListTryExpandTuplePositionals` keep their names with narrowed semantics or are renamed;
6. disposition of `NonIterableStarOperand` after static proof migration;
7. final live `RefRangeIterator` source/module location and conformance declarations;
8. full post-G4 census of remaining `is Iterable`, `iterate`, `iteratorValue`, legacy class/adaptor hits;
9. exact list of `CoreClasses` / `UniverseKey` / bootstrap / invariant / primitive rows to delete;
10. exact package exposure and builtin node edits preserving `iterator_laws` declaration identity;
11. exact LSP tests proving final trait identity/effective methods;
12. whether REPL effective trait completion is currently promised and, if so, the smallest reuse of semantic editor authority;
13. exact current-vs-historical documentation classification;
14. focused G5/G6 command list based on live test registration.

Those are implementation-plan details, not unresolved semantic policy except where explicitly marked STOP/CONSULT above.

---

# 34. Completion records requirements

`P3R-REC-01` — Part III owns the final P5 walkthrough, not Parts I or II.

The walkthrough must describe the final architecture and actual implementation outcome, including any accepted consultation decision for dynamic expansion or REPL integration.

`P3R-REC-02` — Update `LANG005.C6-CHECKPOINT.md` with P5 status only according to actual verified evidence.

Do not mark P5 complete if G5/G6 are blocked by an unresolved architectural seam.

`P3R-REC-03` — Produce `LANG005.C6.P5-handoff.md` for P6 / checkpoint continuation with:

```text
revision implemented
focused verification results
known unrelated baseline failures
final cursor/public-name census result
remaining historical cursor documents intentionally retained
any deferred dynamic/tooling issue explicitly approved
performance/runtime anti-authority evidence
next checkpoint/P6 entry condition
```

`P3R-REC-04` — If broad workspace certification is not part of P5's required verification budget, do not mislabel focused completion as full release certification. Use the shared completion truth table.

---

# 35. Final acceptance definition for Part III

Part III is complete only when all of the following statements are true simultaneously:

```text
[ ] positional * uses Unit/Tuple product lane OR canonical Iterable evidence
[ ] generic * executes iterator()/next from lowering metadata
[ ] no VM cursor/protocol proof remains
[ ] no runtime trait proof/search was added
[ ] boundedness semantics are unchanged
[ ] all live repo legacy Iterable subclasses are migrated
[ ] no live production iterate/iteratorValue implementation remains
[ ] no compiler iteration path emits iterate/iteratorValue
[ ] legacy source Iterable/Iterator classes are deleted/retired
[ ] CoreClasses contains no iterable_class
[ ] UniverseKey/native-meta contains no legacy Iterable class
[ ] List/Map/Set/Tuple/Range/Bytes no longer inherit Iterable at runtime
[ ] canonical collection behavior still works through trait/inherent semantics
[ ] collections package no longer exposes legacy iterable module as authority
[ ] public Iterator/Iterable identity is the canonical P4 traits
[ ] LSP/editor behavior explains effective trait behavior semantically
[ ] REPL does not retain/fabricate fake Iterable inheritance to preserve tooling
[ ] current AST/LSP/docs/guides/wiki comments teach iterator/next semantics
[ ] historical cursor documents are clearly historical/superseded, not rewritten as current
[ ] final production cursor/name grep is classified clean
[ ] G5 passes
[ ] G6 passes or has only explicitly accepted non-blocking baseline failures
[ ] checkpoint, walkthrough, and P5 handoff reflect actual verification truth
```

If any unchecked item is solved by retaining the old cursor/class mechanism behind a compatibility alias, P5 has not achieved its stated architecture.

---

# 36. Requirements-analysis conclusion

The final P5 phase is now sufficiently specified to write a Luna-grade T12–T15 implementation plan.

The live repository confirms that the remaining work is concentrated in a small number of high-impact seams:

- positional expansion is the last confirmed compiler/VM cursor consumer and must gain semantic evidence/lowering authority;
- Sheetcalc `RefRange` is a confirmed repository source consumer of legacy Iterable inheritance;
- native `Iterable` identity is encoded in `CoreClasses`, `UniverseKey`, class relations, bootstrap, primitives, and invariants;
- package topology still exposes both the legacy class module and canonical trait module;
- current-facing AST/LSP/docs still describe cursor iteration;
- REPL completion derives behavior from runtime inheritance and therefore requires an explicit post-retirement audit.

No new iteration model is required. Part III's job is to make every remaining consumer use the already-ratified canonical model, then delete the obsolete authority completely.

The next artifact should be:

```text
LANG005.C6.P5-plan-03-retirement-and-stabilization.md
```

and should translate these requirements into the stable task sequence:

```text
T12 -> T13 -> T14 -> G5 -> T15 -> G6
```

with T12's dynamic-expansion classification and T14/T15's tooling/public-name checks treated as explicit reviewer gates rather than hidden cleanup work.
