# LANG007.C3.P5 — Full Implementation Audit

## Executive verdict

**P5 is not ready for closure, and LANG007.C3 must remain `IN_PROGRESS / PARTIAL`.**

The implementation is substantial and several major architectural choices were implemented correctly, but the audit found **multiple P5-causal semantic defects that are more fundamental than the single T12 failure currently recorded in the handoff**. In particular, the implementation introduced structural presence/remainder semantics at the canonical `TypeStore` level without carrying those semantics consistently through subtype checking, inference, generic materialization, scoped/type-lambda representation, and trait-contract specialization.

The most serious confirmed defect is a direct **type-soundness violation**: an optional source member currently satisfies a required target member if the labels and value types match. In effect, the implementation can accept:

```text
{ a?: Int } <: { a: Int }
```

and the analogous labeled-Tuple relation. That contradicts P5's central rule that optionality is member presence, and that a required target field must be *provably present*. P5 explicitly requires this distinction. :chatgpt-content-reference{index="0"}

The implementation also loses the new structural distinctions through multiple generic paths: Tuple remainders are dropped by `TypeView` and `materialize_type`; typed anonymous Record remainders are left unsubstituted; generic inference has no representation for structural presence or Tuple remainder state; scoped Tuple lowering explicitly discards anonymous remainders; scoped typed Record remainders referring to bound parameters degrade to unconstrained remainders; and trait generic substitution stops specializing an applied type as soon as it encounters a `DynamicType` argument.

Separately, the Iterator work is **not as complete as the walkthrough says**. The natural Range chain is genuinely much healthier and its test is a good end-to-end proof, but the public mapper/predicate contracts required by T7 were not implemented, and the precise `toList`, `partition`, and `toSet` terminal contracts required by T8 remain Dynamic-backed/unannotated. The walkthrough currently groups CV-18..CV-22 as PASS. :chatgpt-content-reference{index="1"} That classification should be revised.

The current checkpoint status—`IN_PROGRESS / PARTIAL / FOCUSED_TESTED`—is therefore correct, but its description substantially overstates which P5 tasks are actually complete. :chatgpt-content-reference{index="2"}

---

# 1. Audit basis

I audited current `main` through the connected repository and reconstructed the implementation history from the P5 planning baseline.

| Revision | Role |
|---|---|
| `6dcdcec08e55c2eb67eb60fff959cfb71fc3e939` | P5 planning baseline |
| `cbe62b4c59ede2cd168e8c3b86f7fa9e317339af` | Major T1–T7 implementation |
| `887269afb69e1f7f12b87eb0ce2bb23291a8fb87` | T8/T9 iteration closure |
| `d6def47c50a989ae5e4ce945b7fa7198d8764716` | T10 metadata/schema v10 |
| `42784e071861d17f5506255f11df2fed2beb0464` | T11 VM environment transport |
| `f7ddedf62840573fdfb4e9f19507cbcea2b573ee` | T11/T12 semantic dispatch correction |
| `e4418ae7364f202f74116b30b58f66496381a860` | Current `main`; documentation-only after `f7ddedf6` |

Thus the current production implementation is effectively the code at `f7ddedf6`; the latest commit only updates records.

The older on-the-ground continuation plan is correctly interpreted as historical. At that point T8 was partial and four iteration tests still failed. :chatgpt-content-reference{index="3"} Those specific iteration failures were subsequently addressed.

### Verification limitation

I independently reviewed repository history and current source, but I **did not rerun Cargo locally** because repository/computer execution was declined when Work mode was offered. The recorded test results were therefore audited structurally rather than independently re-executed. I also attempted to inspect GitHub commit/CI status, but the GitHub integration denied status access with HTTP 403.

Consequently:

- source-code findings below are independently confirmed;
- recorded PASS counts remain historical evidence;
- I do **not** independently certify those command executions.

This limitation does not affect the confirmed semantic bugs below because they follow directly from current production control flow.

---

# 2. Severity summary

| ID | Severity | Finding | Closure impact |
|---|---|---|---|
| **P5-AUD-001** | **S0 Critical** | Optional source member satisfies required target member | Soundness blocker |
| **P5-AUD-002** | **S0 Critical** | Generic inference erases structural presence/remainders | Soundness/completeness blocker |
| **P5-AUD-003** | **S1 Major** | `TypeView` / generic materialization lose Tuple remainders and fail to specialize typed Record remainder | Generic/witness blocker |
| **P5-AUD-004** | **S1 Major** | Scoped/type-lambda lowering cannot faithfully represent P5 structural remainders | Generic type-formation blocker |
| **P5-AUD-005** | **S1 Major** | `List<Dynamic>` relation invents static refutation rather than Dynamic boundary | Dynamic semantics blocker |
| **P5-AUD-006** | **S1 Major** | Trait generic substitution aborts specialization around `DynamicType` | Conformance correctness blocker |
| **P5-AUD-007** | **S2 Substantial** | Illegal optional positional Tuple is representable and persistable | Canonical invariant/schema gap |
| **P5-AUD-008** | **S1 Major** | Owner + method generic call specialization is incomplete | Existing T12 blocker |
| **P5-AUD-009** | **S1/S2** | Dynamic receiver-owner bindings cannot be represented in `CallSpecialization` | Dynamic/conformance integration gap |
| **P5-AUD-010** | **S1 Major** | Required typed Iterator mapper/predicate contracts were not implemented | T7/CV-19 incomplete |
| **P5-AUD-011** | **S1/S2** | Precise `toList`/`partition`/`toSet` source contracts remain missing | T8 incomplete |
| **P5-AUD-012** | **S1/S2** | Scoped generic applications have no nested-Dynamic argument carrier | Dynamic model incomplete |
| **P5-AUD-013** | S4 | Dead/misleading pack subtype arms remain | Architecture hygiene |
| **P5-AUD-014** | S2 verification | Required P5-specific incremental differential coverage is missing | CV-26 unverified |

There are therefore **at least six closure-blocking correctness defects**, independent of the already-known T12 failure.

---

# 3. P5-AUD-001 — Optional source presence unsoundly satisfies a required target

**Severity: S0 — type-soundness critical**

### Intended semantic law

P5 says optionality represents member **presence**, not value optionality:

```text
{ a?: Int }
```

means the field may not exist.

Therefore:

```text
{ a?: Int } </: { a: Int }
```

A required target member must be *provably present*. The structural specification also states this directly.

### Actual Record implementation

Current:

`phalcom-semantic/src/types/relation.rs:824–855`

Conceptually:

```rust
for target_field in sup_row.fields.iter() {
    if let Some(source_field) =
        sub_row.fields.iter().find(|f| f.name == target_field.name)
    {
        let outcome = check_subtype_impl(
            store,
            hierarchy,
            source_field.ty,
            target_field.ty,
            ...
        );

        if !outcome.is_proven() {
            return outcome;
        }
    } else if target_field.presence.is_required() {
        return Refuted(...);
    }
    ...
}
```

The requiredness check occurs **only when the source field is missing entirely**.

When the source contains:

```text
a?: Int
```

and the target contains:

```text
a: Int
```

the first branch executes. Only:

```text
Int <: Int
```

is checked.

`source_field.presence` is ignored.

### Former Labeled Tuple Presence (Removed by Remediation)

Previously, Tuple elements carried presence variability and exhibited a similar defect in `phalcom-semantic/src/types/relation.rs`. Under the P5 Structural Presence Remediation, Tuple presence variability was identified as an unintended feature and completely removed: Tuple fixed members are always required, and Tuple variability is expressed exclusively through positional and labeled remainders. Optionality is strictly a Record field-presence feature. The Record half of this issue remains a subtype-soundness fix (`{ a?: Int } </: { a: Int }`).

### Consequence

A function requiring:

```text
{ user: User }
```

may statically receive a value whose type promises only:

```text
{ user?: User }
```

and then access `user` as if it certainly exists.

That is not loss of precision. It is a structural subtype **soundness defect**.

### Missing regression

The recorded CV-03/CV-04 tests evidently exercise optional absence/present typing but do not test the crucial direction:

```text
optional source -> required target = false
```

The walkthrough's blanket `CV-01..CV-09 PASS` therefore overstates the structural relation's correctness. :chatgpt-content-reference{index="4"}

### Required repair

Presence compatibility must precede or accompany value-type comparison:

```text
target required:
    source must be Required
    AND source.type <: target.type

target optional:
    source Required or Optional accepted
    provided any possible present value <: target.type
```

Add both Record and labeled-Tuple negative laws.

---

# 4. P5-AUD-002 — Generic inference cannot represent the structural semantics P5 introduced

**Severity: S0 — systemic type-system correctness defect**

This is the broadest architectural problem found.

P5 added first-class:

- required/optional presence;
- positional Tuple remainder state;
- labeled Tuple remainder state;
- anonymous Record remainder state.

But the principal generic inference IR was not correspondingly extended.

## Tuple inference IR

`phalcom-semantic/src/checker/inference.rs`

`InferenceTupleElement` effectively contains:

```rust
struct InferenceTupleElement {
    label: Option<Box<str>>,
    term: InferenceTerm,
}
```

There is **no presence** field.

`InferenceTerm::Tuple` also has no positional/labeled remainder state.

When canonical:

```rust
TypeData::Tuple(TupleType {
    elements,
    positional_remainder,
    labeled_remainder,
})
```

is converted into inference terms, the code retains only:

```text
label
type term
```

and discards:

```text
presence
positional_remainder
labeled_remainder
```

When inference materializes the Tuple again, it creates every element as:

```rust
presence: StructuralMemberPresence::Required
```

and reconstructs a closed Tuple.

## Record inference IR

`phalcom-semantic/src/checker/row_inference.rs`

`InferenceRecordField` similarly contains only:

```rust
name
term
```

with no presence.

Record-row conversion therefore erases optionality.

The reconstruction path hard-codes:

```rust
presence: Required
```

for inferred fields.

### Consequences

A generic operation receiving:

```text
{ value?: T }
```

can have its contract turned into:

```text
{ value: T }
```

during generic inference.

Likewise an open Tuple such as conceptually:

```text
(Int, *: String)
```

can be materialized as merely:

```text
(Int)
```

after passing through inference.

That can cause both:

- **unsound strengthening** — optional → required;
- **incorrect rejection/closure** — open remainder → closed.

This directly violates INV-03 and INV-05, which require presence and remainder state to remain canonical semantic facts. :chatgpt-content-reference{index="5"}

### Architectural significance

This cannot be repaired by adding a relation special case.

The inference representation itself is insufficient.

`InferenceTerm::Tuple` and inferred Record fields need to carry the structural shape dimensions or the inference architecture needs another canonical-shape-preserving representation.

---

# 5. P5-AUD-003 — Two major materialization paths erase new structural state

**Severity: S1 — generic/witness correctness**

P5 correctly extended `TypeSubstitution`; that path actually preserves Tuple remainders and substitutes typed Record remainders.

For example, `phalcom-semantic/src/types/substitution.rs:136–156` correctly computes:

```rust
let pos_rem = match tuple.positional_remainder {
    Typed(t) => Typed(substitute(t)),
    rem => rem,
};

let lab_rem = ...
store.open_tuple(..., pos_rem, lab_rem)
```

and `apply_record_row_with_arena` specializes a typed anonymous remainder.

However, two other materialization authorities were not updated equivalently.

## 5.1 `TypeView::materialize_view`

`phalcom-semantic/src/types/environment.rs:215–242`

Tuple:

```rust
TypeData::Tuple(elems) => {
    ...
    store.tuple(subst_elems.into_boxed_slice())
}
```

`store.tuple` creates a **closed Tuple**.

Both:

```text
positional_remainder
labeled_remainder
```

are discarded.

Record:

```rust
let row = store.record_row(row_id);
(fields, row.tail)
...
record_row_type_checked(subst_fields, tail)
```

The field types are recursively materialized, but the tail is copied verbatim.

So:

```text
{ fixed: Int, **: T }
```

under:

```text
T := String
```

retains an unspecialized `T` in its typed anonymous remainder.

## 5.2 `materialize_type`

`phalcom-semantic/src/types/instantiation.rs:70–190`

The same Tuple defect exists:

```rust
TypeData::Tuple(elements) => {
    ...
    Ok(store.tuple(elements.into_boxed_slice()))
}
```

Again remainder lanes disappear.

And `materialize_row_tail` at `335–359` only materializes:

```rust
RecordRowTail::Parameter(...)
```

Everything else returns unchanged:

```rust
let RecordRowTail::Parameter(parameter) = tail else {
    return Ok((fields, tail));
};
```

That means a typed anonymous remainder is not recursively specialized.

### Why this matters

`TypeView` is not peripheral. It participates in:

- trait surface instantiation;
- witness candidate specialization;
- inferred return specialization.

`materialize_type` feeds:

- generic constraint discharge;
- conformance;
- generic callable specialization.

Thus P5's structural semantics disappear precisely at use-site specialization boundaries.

---

# 6. P5-AUD-004 — Scoped/type-lambda structural representation is incomplete

**Severity: S1**

P5 requires structural semantics to survive generic/scoped type forms.

Current scoped representation:

`phalcom-semantic/src/types/type_lambda.rs`

contains:

```rust
ScopedTypeData::Tuple(Box<[ScopedTupleElement]>)
```

where each element has presence, but the Tuple has **no anonymous remainder state**.

### Tuple source lowering explicitly discards remainders

`phalcom-semantic/src/types/annotation.rs:1419–1522`

On encountering:

```rust
TypeTupleItem::Remainder(...)
```

the implementation does this:

```rust
if let AnonymousRemainderSyntax::Typed { constraint, ... } = ... {
    let ty = lower_scoped_type_form(...);
    require_scoped_proper(...);
}
```

and then throws the result away.

It finally interns:

```rust
ScopedTypeData::Tuple(fixed_elements)
```

The source remainder has vanished.

This is not an accidental edge case—the comment currently acknowledges the incomplete behavior.

### Bound typed Record remainder is weakened

At approximately `annotation.rs:1589–1596`:

```rust
let ty = lower_scoped_type_form(...);

if let ScopedTypeData::Free(ty_id) = ... {
    AnonymousRecordRemainder::Typed(ty_id)
} else {
    AnonymousRecordRemainder::Unconstrained
}
```

Therefore:

```text
type lambda T => { value: Int, **: T }
```

cannot preserve the constraint if `T` is a scoped/bound variable.

Instead:

```text
**: T
```

becomes effectively:

```text
**...
```

### Result

Generic aliases/type lambdas cannot faithfully express all structural types that ordinary canonical types can express.

P5 therefore has **two structural type languages**:

1. the canonical TypeStore model;
2. a weaker scoped model.

That violates the stated canonical-flow design.

---

# 7. P5-AUD-005 — Dynamic applied-type relation invents a static contradiction

**Severity: S1**

P5's Dynamic design is explicit:

```text
List<Dynamic>
```

has a known outer type but uncertainty in one argument slot.

If relation depends on that slot, the result must remain a Dynamic boundary rather than fabricate proof or refutation. INV-13 explicitly requires this. :chatgpt-content-reference{index="6"}

Current:

`phalcom-semantic/src/types/relation.rs:320–385`

For same-origin applications:

```rust
let (Some(a_sub), Some(a_sup)) =
    (a_sub.as_type_id(), a_sup.as_type_id())
else {
    if a_sub == a_sup {
        continue;
    }

    return RelationOutcome::Refuted(...);
};
```

`StaticArgument::DynamicType::as_type_id()` returns `None`.

Therefore:

```text
List<Dynamic> vs List<Int>
```

or:

```text
List<Dynamic> vs List<Any>
```

takes the `else` branch and becomes:

```text
Refuted(TypeMismatch)
```

unless both arguments are literally `DynamicType`.

That is an invented static contradiction.

The intended result is a Dynamic/uncertain relation boundary whenever variance/relation depends on that slot.

### Why existing tests missed it

The CV-10..CV-14 tests currently emphasize:

- formation;
- distinction from `Any`;
- materialization;
- fixed/dependent results.

I found no corresponding same-head Dynamic-argument relation discriminator.

So the recorded “Dynamic Generic Argument Envelope PASS” proves formation substantially better than it proves relation semantics. :chatgpt-content-reference{index="7"}

---

# 8. P5-AUD-006 — Trait generic specialization gives up when one argument is Dynamic

**Severity: S1**

`phalcom-semantic/src/traits.rs:834–860`

For:

```rust
TypeData::Applied { origin, arguments }
```

trait-contract substitution currently does:

```rust
for argument in arguments {
    let Some(argument) = argument.as_type_id() else {
        return TypeTerm::Canonical(ty);
    };

    ...
}
```

A `DynamicType` argument has no `TypeId`.

Therefore encountering one Dynamic slot returns the **entire original applied type unchanged**.

Example:

```text
Pair<Dynamic, T>
T := Int
```

should become:

```text
Pair<Dynamic, Int>
```

but this path leaves:

```text
Pair<Dynamic, T>
```

unchanged.

This violates P5's central dependent-vs-independent materialization rule.

The same helper also reconstructs Tuple types with:

```rust
store.tuple(...)
```

and therefore drops Tuple remainder state as described above.

This function feeds trait generic contract instantiation, so generic conformance can operate on stale or structurally weakened requirements.

---

# 9. P5-AUD-007 — Illegal optional positional Tuple is a canonical and durable state

**Severity: S2**

P5 explicitly rejects optional positional Tuple elements.

Yet `phalcom-semantic/src/types/store.rs` defines:

```rust
pub fn optional_unlabeled(ty: TypeId) -> Self {
    Self {
        label: None,
        ty,
        presence: Optional,
    }
}
```

The canonical Tuple constructors do not universally forbid that state.

### Metadata inconsistency

Schema v10's `OpenTuple` validator correctly rejects optional positional elements.

But the older/closed:

```text
TypeNode::Tuple
ScopedTypeNode::Tuple
```

forms now also carry member presence, and their v10 validation paths do not enforce the same restriction.

Thus schema-v10 metadata can represent a closed Tuple containing:

```text
optional + unlabeled
```

even though the language says that state cannot exist.

This is exactly the kind of malformed durable state a metadata validator is supposed to prevent.

### Repair

Make the invariant canonical:

```text
label == None => presence == Required
```

at the store formation boundary, then duplicate the validation defensively in all metadata forms.

The existence of `optional_unlabeled` should probably disappear entirely.

---

# 10. P5-AUD-008 — Existing T12 owner/method specialization failure is real and architecturally significant

**Severity: S1**

The handoff records one remaining generic-conformance test:

`receiver_owner_evidence_is_published_without_owner_inference`

with 17/18 passing. :chatgpt-content-reference{index="8"}

The fixture combines:

```text
Container<T>
where T |= Printable
```

with:

```text
combine<U>(...)
where U |= Printable
```

The call should publish evidence for:

```text
T
U
```

The implementation currently has two publication stages:

### Method-domain publication

`checker/call.rs:3170–3232`

`publish_call_specialization` constructs method/declaration bindings and discharges conformance obligations.

### Owner finalization

`checker/call.rs:3242–3300`

`finalize_owner_call_specialization` takes the previously published specialization, adds receiver-owned bindings, redischarges the combined contract, and republishes it.

But it explicitly returns when:

```rust
callable_has_generic_domains && existing.is_none()
```

The failing test demonstrates a path where the method-local specialization does not exist by the time owner finalization requires it.

Because `CallSpecialization` feeds semantic lowering/runtime environment recipes, this is not merely missing test metadata.

It represents an incomplete ownership/inference composition lifecycle.

---

# 11. P5-AUD-009 — Dynamic receiver-owner bindings fall outside `CallSpecialization`

**Severity: S1/S2**

P5 correctly introduced a knowledge-bearing receiver environment:

```text
exact parameter -> TypeId
Dynamic parameter -> dynamic binding
```

`TypeEnvironment` therefore has both:

```rust
bindings: HashMap<TypeParameterId, TypeId>
dynamic_bindings: HashSet<TypeParameterId>
```

But owner finalization only calls:

```rust
environment.get_param(parameter)
```

and considers failure to return a `TypeId` to mean:

> receiver specialization is missing a required owner binding

There is no equivalent:

```text
get_dynamic(parameter)
```

path into `CallSpecialization`.

More fundamentally, `CallSpecialization.bindings` itself represents exact:

```text
(TypeParameterId, TypeId)
```

bindings only.

So a receiver such as conceptually:

```text
Container<Dynamic>
```

has a valid semantic receiver environment but no representable owner binding in the call-specialization product.

This is another point where P5 introduced a richer semantic state at one level without extending a downstream carrier.

---

# 12. P5-AUD-010 — T7 mapper/predicate contracts were not implemented

**Severity: S1 — implementation gap**

P5 T7 specifically requires typed mapper/predicate/callback contracts and result generic `U` inference. The task's output explicitly includes:

- typed mapper/predicate/callback contracts;
- result generic `U` inference. :chatgpt-content-reference{index="9"}

Current `iterator_laws.ph:9–26` still says:

```phalcom
map(_ f) { MappedIterator.new(self, f) }
filter(_ pred) { FilterIterator.new(self, pred) }
filterMap(_ f) { FilterMapIterator.new(self, f) }

skipWhile(_ pred) { SkipWhileIterator.new(self, pred) }
takeWhile(_ pred) { TakeWhileIterator.new(self, pred) }
mapWhile(_ f) { MapWhileIterator.new(self, f) }
inspect(_ f) { InspectIterator.new(self, f) }
```

The downstream adapter constructors are more precise, for example:

```phalcom
MappedIterator<I, U>.new(
    _ mapper: (<I as Iterator>::Item) -> U
)
```

but that does **not** make the public `Iterator.map` declaration carry the required contract.

The intended public shape was essentially:

```phalcom
map<U>(
    _ f: (Self::Item) -> U
) -> MappedIterator<Self, U>
```

with analogous definitions for `filterMap`, `mapWhile`, predicate methods, etc.

### Coverage problem

`iterator_adapter_fixture_preserves_item_projections_and_chain_equality` exercises:

- chain;
- flatMap;
- flatten;
- zip;
- indexed.

It does not establish `map` / `filterMap` / `mapWhile` `U` inference or the predicate callable domains.

So CV-19 has not actually been established.

---

# 13. P5-AUD-011 — Terminal Iterator contracts remain materially imprecise

**Severity: S1/S2**

P5 required terminal precision where semantically expressible.

Current `iterator_laws.ph`:

### `partition`

Lines 258 onward:

```phalcom
partition(where predicate) {
  let accepted = List<Dynamic>.new()
  let rejected = List<Dynamic>.new()
  ...
  return (accepted, rejected)
}
```

No predicate contract and no:

```text
(List<Self::Item>, List<Self::Item>)
```

result contract.

### `toSet`

```phalcom
toSet {
  let result = Set<Dynamic>.new()
  ...
}
```

No:

```text
Set<Self::Item>
```

contract.

Current `Set<T>` has no evident type constraint preventing this precision.

### `toList`

Lines 327–337:

```phalcom
toList {
  let result = List<Dynamic>.new()
  ...
  return result
}
```

No:

```text
-> List<Self::Item>
```

contract.

The P5 implementation plan explicitly calls for direct `toList` result-contract and `partition`/`toSet` type controls. Those requirements were not satisfied merely because the natural runtime chain executes.

### Important distinction

The excellent `unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence` test proves:

- the actual `.take(10).toList` expression has an evidenced dispatch;
- the target is `Iterable::toList`;
- evidence is `Derived(GuardedRule)`.

That is meaningful progress.

But the test does **not** assert the result is:

```text
List<Int>
```

rather than:

```text
List<Dynamic>
```

So CV-22's dispatch/evidence half can pass while T8's terminal typing objective remains incomplete.

---

# 14. P5-AUD-012 — Scoped generic applications still lack nested-Dynamic representation

**Severity: S1/S2**

Canonical generic applications now correctly have:

```rust
StaticArgument::DynamicType
```

But `ScopedTypeData::Applied` is still:

```rust
Applied {
    origin: ScopedTypeId,
    arguments: Box<[ScopedTypeId]>,
}
```

There is no scoped heterogeneous argument equivalent containing:

```text
DynamicType
```

`lower_scoped_type_form` treats source `Dynamic` as:

```text
TypeFormationOutcome::Dynamic
```

A scoped generic application recursively lowers its arguments expecting a `ScopedTypeId`; a Dynamic argument therefore propagates the whole Dynamic formation outcome rather than becoming a nested argument marker.

Consequently forms involving both a scoped binder and nested Dynamic, conceptually:

```text
Pair<T, List<Dynamic>>
```

inside a type lambda/generic scoped context cannot be represented with the same semantics as their canonical non-scoped counterpart.

Metadata shows the same asymmetry:

- top-level `StaticArgumentRef` has `DynamicType`;
- scoped applied nodes remain scoped-node IDs only.

This leaves the nested-Dynamic implementation incomplete across the full type-form language.

---

# 15. P5-AUD-013 — Dead pack-subtyping logic remains

**Severity: S4**

P5 says ordinary `<:` applies only to proper `Type` terms.

The outer kind gate appears to preserve that.

However `relation.rs` still contains branches such as:

```rust
TypeData::TypePack(...)
TypeData::LabeledTypePack(...)
TupleFromPacks(...)
CompleteTypePack(...)
```

inside the ordinary subtype relation.

Because the proper-Type guard prevents unequal pack terms reaching these paths in normal operation, I do **not** classify this as an active soundness bug.

But P5 explicitly wanted misleading pack-subtype paths removed or moved to clearly named domain compatibility/equality operations.

This is architectural residue and increases the risk of a later caller bypassing the intended boundary.

---

# 16. P5-AUD-014 — Incremental P5 semantics are not sufficiently certified

**Severity: S2 verification gap**

The production fingerprint work itself looks good.

I confirmed metadata fingerprinting includes:

- structural member presence;
- anonymous Record remainder;
- Tuple remainder state;
- `DynamicType`.

The metadata test:

`structural_member_presence_and_remainder_affect_fingerprints`

is useful.

However I did not find focused P5 differential tests establishing:

```text
cold(new) == incremental(old -> new)
```

for edits such as:

```text
required -> optional
closed -> open
typed remainder Int -> String
List<Any> -> List<Dynamic>
Dynamic -> exact generic argument
```

nor evidence that only relevant dependents are invalidated.

That matters because CV-26 explicitly calls for cold/import/incremental equivalence.

Given the representation bugs above, an incremental PASS obtained only over ordinary closed shapes would not be sufficient anyway.

---

# 17. Additional audit result — scoped representation and inference are the common failure pattern

A significant architectural pattern emerged.

P5 correctly upgraded the **canonical TypeStore**:

```text
TupleType
StructuralMemberPresence
AnonymousTupleRemainder
RecordRowTail::Anonymous
StaticArgument::DynamicType
```

but several adjacent semantic carriers retained their old shape:

```text
InferenceTerm::Tuple
InferenceRecordField
ScopedTypeData::Tuple
ScopedTypeData::Applied
CallSpecialization
```

This creates a semantic “funnel”:

```text
rich canonical type
       ↓
old narrower intermediate representation
       ↓
reconstructed type
```

where information disappears.

That is the main architectural cause behind several apparently independent findings.

The correct remediation is **not** to patch each failing fixture.

Every carrier capable of transporting a P5 type contract must be inventoried and either:

1. extended to carry the distinction; or
2. replaced by a canonical reference/view that cannot erase it.

---

# 18. Systems that audited positively

The audit did not find every P5 subsystem defective.

## 18.1 Canonical Dynamic formation

The core `StaticArgument::DynamicType` choice is sound architecturally.

It correctly avoids:

- `TypeData::Dynamic`;
- synthetic Dynamic `TypeId`;
- rewriting Dynamic to `Any`.

`TypeStore` interning includes the Dynamic marker in structural equality/hash, so equivalent `List<Dynamic>` forms are canonicalized rather than creating a Dynamic object per use.

## 18.2 Exact TypeSubstitution

`phalcom-semantic/src/types/substitution.rs` is notably better than the newer parallel materializers.

It:

- preserves member presence;
- preserves and specializes Tuple remainders;
- specializes typed anonymous Record remainders;
- preserves `DynamicType` arguments.

This should probably serve as an implementation reference when repairing `TypeView`, `materialize_type`, and trait substitution.

## 18.3 Metadata v10 is substantial

Schema v10 correctly adds durable forms for:

- open Tuple;
- anonymous Record;
- structural presence;
- Tuple remainders;
- Record remainders;
- Dynamic argument slots.

It also:

- enforces schema floors;
- validates Dynamic argument kind through application kind checking;
- fingerprints P5 structural distinctions;
- maps `DynamicType` into runtime overlay representation;
- renders new forms in runtime inspection.

The finding above is a specific validator/canonical-state hole, not a rejection of the schema design.

## 18.4 Generic evidence recursion is conservative in the reviewed engine

The `generic_evidence.rs` recursion machinery uses Visiting/Blocked behavior and does not appear to simply treat recursive goals as proven.

Derived-rule candidates require their nested obligations to succeed, and multiple successful candidates are handled as coherence issues.

I found no independent “recursive proof accepts itself” defect.

However, this engine consumes materialized constraints, so findings P5-AUD-003/006 can still feed it incorrect contracts.

## 18.5 Natural Iterator → Iterable evidence path is materially improved

The current `unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence` test is a good semantic test.

It inspects the actual source expression and verifies:

```text
TraitDispatchSite::Evidenced
Iterable::toList
Derived evidence
GuardedRule origin
```

The adjacent known-ListIterator test goes further and verifies nested exact evidence and semantic dependency publication.

Those are meaningful improvements over the earlier P4 state.

## 18.6 Rest runtime environment forwarding appears architecturally correct

The T11 VM changes forward:

```text
type_environment
conformance_environment
```

through specialized rest activation into the callee frame.

The reviewed runtime path transports selected evidence rather than conducting VM-side trait search, satisfying the core INV-23/24 ownership boundary.

The core test:

`p4_specialized_rest_dispatch_preserves_conformance_environment`

also executes the specialized rest call at runtime.

The major weakness is upstream **production of the complete `CallSpecialization` recipe**, not the VM transport once the recipe exists.

---

# 19. Performance and memory audit

I found no P5-specific catastrophic memory or runtime-performance issue.

### Canonical type memory

The new canonical forms derive structural equality/hash and participate in the existing TypeStore interning scheme.

Equivalent:

```text
List<Dynamic>
```

or structurally identical open products therefore reuse canonical nodes.

No standalone Dynamic type allocation was introduced.

### Rest dispatch

The P5 runtime change mainly forwards environment identities already computed semantically. It does not introduce runtime proof search.

That is the correct cost model.

### Structural subtype relation

There is a moderate existing complexity concern: Tuple/Record relation uses repeated linear searches such as:

```rust
sub_lab.iter().find(...)
sup_lab.iter().any(...)
```

making labeled structural comparison potentially **O(n²)** in member count.

For ordinary Record sizes this is unlikely to be material, and P5 is not a performance plan. I would not block C3 over it.

If very large structural record types become common, canonical sorted fields could support a linear merge.

### More important than runtime performance: semantic recomputation

The plan asks for bounded observations of:

- callable bodies recomputed;
- conformance plans refreshed;
- unrelated body reuse.

I did not find final P5 evidence for these new structural/Dynamic edit classes.

That is a verification deficiency rather than a demonstrated regression.

---

# 20. Current coverage reclassification

The current walkthrough's summary should not survive this audit unchanged.

| CV | Audit disposition | Reason |
|---|---|---|
| CV-01 closed structural negative | **VERIFIED WITH DEFECTS NEARBY** | exact extras appear enforced |
| CV-02 explicit open success | **VERIFIED WITH DEFECTS NEARBY** | basic openness works |
| CV-03 optional absence | **FAILED / INCOMPLETE** | optional-source → required-target law missing |
| CV-04 optional presence | **FAILED / INCOMPLETE** | relation ignores source presence |
| CV-05 captured vs anonymous | **PARTIAL** | canonical distinction exists; scoped/materialization paths erase/weakens it |
| CV-06 typed remainder | **FAILED cross-layer** | direct relation exists; generic/scoped specialization broken |
| CV-07 Tuple lane independence | **FAILED cross-layer** | direct relation exists; materialization closes lanes |
| CV-08 structural canonicalization | **PARTIAL** | store identity good; illegal positional optional representable |
| CV-09 proper-Type boundary | **PASS with cleanup debt** | gate intact; stale pack arms remain |
| CV-10 nested Dynamic formation | **PARTIAL** | canonical path works; scoped path does not |
| CV-11 Dynamic kind safety | **MOSTLY PASS** | canonical/schema kind validation good |
| CV-12 bare dependent Dynamic result | **PARTIAL** | simple path works; call specialization does not carry all Dynamic owner bindings |
| CV-13 nested dependent result | **PARTIAL** | normal materializer works; trait/scoped paths incomplete |
| CV-14 fixed result remains Known | **PARTIAL** | intended mechanism exists; mixed-Dynamic trait substitution can abort |
| CV-15 metadata round trip | **PARTIAL** | substantial v10 support; illegal closed positional-option state accepted |
| CV-16 witness specialization | **FAILED cross-layer** | remainder materialization incomplete |
| CV-17 witness negative | **NOT SUFFICIENTLY CERTIFIED** | direct negative exists but specialized structural contracts can already be wrong |
| CV-18 projection/bound legality | **PASS / substantial evidence** | source bounds greatly improved |
| CV-19 mapper `U` inference | **FAILED / NOT IMPLEMENTED** | public mapper contracts still untyped |
| CV-20 predecessor adapter semantics | **PASS for covered adapters** | chain/flatMap/flatten/zip/indexed strong |
| CV-21 `toMap` single advance | **RECORDED PASS** | runtime test coverage reasonable |
| CV-22 natural Range chain | **PASS for dispatch/runtime; incomplete type precision** | evidence path good; `toList` result still Dynamic-backed |
| CV-23 bootstrap closure | **RECORDED PASS** | zero diagnostics claimed; not independently rerun |
| CV-24 evidence recipe | **PARTIAL** | rest probe works; generic owner+method publication still fails elsewhere |
| CV-25 runtime transport | **PASS for reviewed rest path** | transport architecture good |
| CV-26 incremental parity | **INCOMPLETE** | no sufficient P5 structural/Dynamic differential matrix |
| CV-27 P4.R1 lifecycle | **PARTIAL / regression risk** | provider ownership preserved, but new materialization paths corrupt shape |
| CV-28 failure classification | **NOT COMPLETE** | T12 itself remains open and this audit adds P5-causal failures |

The P5 plan requires every CV-01..CV-27 obligation to be dispositioned before T12 can be accepted. :chatgpt-content-reference{index="10"}

**T12 is therefore not close to green in its current semantic sense, even if most existing test selectors pass.**

---

# 21. Task-level audit disposition

| Task | Recorded state | Audit state |
|---|---|---|
| T1 baseline | Complete | **Accepted** |
| T2 structural spec | Complete | **Accepted**; specification is clearer than implementation |
| T3 structural representation | Complete | **Partial**; canonical core exists, scoped/inference carriers incomplete |
| T4 Dynamic generic slots | Complete | **Partial**; canonical representation correct, scoped carrier absent |
| T5 structural relation | Complete | **Failed**; presence soundness bug |
| T6 Dynamic consumers/persistence | Complete | **Partial/Failed**; relation + trait specialization defects |
| T7 witness + Iterator contracts | Complete | **Partial/Failed**; mapper contracts missing; materialization defects |
| T8 bootstrap/terminal precision | Complete | **Partial**; bootstrap improved, terminal contracts incomplete |
| T9 natural chain | Complete | **Substantially accepted** |
| T10 metadata/incremental/tooling | Complete | **Metadata mostly accepted; incremental incomplete** |
| T11 rest evidence transport | Complete | **Runtime transport accepted; semantic recipe lifecycle remains incomplete globally** |
| T12 focused matrix | In progress | **Failed/open** |
| T13 residual classification | Not final | **Cannot begin final classification yet** |
| T14 certification | Not run/final | **Must not begin as closure certification** |
| T15 closure | Not run | **Blocked** |

---

# 22. Documentation accuracy findings

The walkthrough is useful but currently too optimistic.

It reports:

> CV-01..CV-09 PASS  
> CV-10..CV-14 PASS  
> CV-18..CV-22 PASS

while simultaneously recording CV-16..17 as partial. :chatgpt-content-reference{index="11"}

Following this audit:

- CV-03/04 need reopening;
- CV-05/06/07 need cross-layer reopening;
- CV-10/12/13/14 need scoped/conformance reopening;
- CV-19 needs implementation;
- CV-22 should be described as **natural-chain dispatch/runtime PASS**, not total Iterator precision;
- CV-26 remains open.

The handoff's proposed next step—finish the one T12 test and proceed directly into T13/T14—is no longer sufficient. :chatgpt-content-reference{index="12"}

The checkpoint must be amended before another implementer interprets the current state as “one failing test from certification.”

---

# 23. Required remediation order

The implementation should **not** proceed to broad T14 certification yet.

The safest corrective sequence is:

1. **Repair structural presence relation immediately.** Add direct Record and labeled-Tuple optional→required negative controls.
2. **Perform a structural carrier census.** Extend inference, `TypeView`, `GenericInstantiation`, trait substitution, scoped types, and metadata carriers so presence/remainders cannot disappear.
3. **Unify materialization behavior.** There are currently multiple recursive type-rewriters with inconsistent P5 coverage. Reuse a single recursive structural-materialization primitive where possible.
4. **Complete scoped structural representation.** Tuple lane remainders and typed anonymous Record remainders must be able to reference scoped/bound types.
5. **Complete scoped Dynamic representation.** Do not let nested Dynamic collapse a scoped applied form.
6. **Correct Dynamic relation semantics.** Mismatched Dynamic slots must produce `DynamicBoundary`, preserving declared variance.
7. **Repair mixed exact+Dynamic trait substitution.**
8. **Fix `CallSpecialization` owner/method merge**, including a deliberate representation for Dynamic owner bindings.
9. **Implement the actual T7 public callable contracts** for map/filter/filterMap/mapWhile/takeWhile/etc.
10. **Implement or explicitly classify T8 terminal precision** for `toList`, `partition`, and `toSet`.
11. **Build a new focused structural/generic matrix** that exercises all of the above through generic aliases, generic call inference, trait witness specialization, imported metadata, and incremental refresh.
12. Only after that should T12 be reclassified, followed by T13 and T14.

This is large enough that I would treat it as a **P5 corrective closure slice**, not as opportunistic fixes inside the current T12 loop.

---

# 24. Mandatory new regression cases

At minimum, the next implementation should add direct discriminators for these previously uncovered laws:

```text
# Presence soundness
{ a?: Int } </: { a: Int }
{ a: Int } <: { a?: Int }

# Tuple optional members rejected by syntax
(a?: Int) => parse error
(Int?) => parse error

# Generic presence preservation
F<{ a?: T }>[T := Int]
    == { a?: Int }

# Generic Tuple remainder preservation
F<(Int, *: T)>[T := String]
    == (Int, *: String)

# Generic Record typed remainder
F<{ a: Int, **: T }>[T := String]
    == { a: Int, **: String }

# Scoped/type-lambda versions of both

# Dynamic relation
List<Dynamic> <: List<Int>
    => DynamicBoundary, not Refuted/Proven
# with covariant/invariant/contravariant controls

# Mixed Dynamic specialization
Pair<Dynamic, T>[T := Int]
    == Pair<Dynamic, Int>

# Dynamic owner call specialization
Container<Dynamic>.method<Exact>(...)
    preserves Dynamic owner + exact method knowledge

# Mapper contracts
Iterator<Int>.map(|x: Int| String)
    -> MappedIterator<..., String>

filterMap / mapWhile similarly infer U

# Predicate negative
Iterator<Int>.filter(|x| "not bool")
    rejected

# Terminal contracts
Iterator<Int>.toList
    -> List<Int>

Iterator<Int>.partition(...)
    -> (List<Int>, List<Int>)

Iterator<Int>.toSet
    -> Set<Int>
```

These tests should be at the semantic product level, not only runtime-output fixtures.

---

# 25. Final architecture assessment

The **direction** of P5 remains sound.

I would preserve these decisions:

- exact closed structural products;
- optionality as presence;
- captured vs anonymous remainder distinction;
- Dynamic outside standalone `TypeId`;
- `StaticArgument::DynamicType`;
- provider-owned callable inferred results;
- strict witness compatibility;
- semantic conformance proof selection;
- VM-only evidence transport;
- derived `Iterator -> Iterable` law;
- schema-v10 structural durability.

The implementation problem is not that those rulings were wrong.

The problem is that they were integrated **vertically into the canonical representation but not horizontally through every semantic carrier**.

That distinction matters. The remediation should finish the architecture, not redesign it.

---

# 26. Closure verdict

## LANG007.C3.P5

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
audit_result: FAILED_CLOSURE_AUDIT
broad_certification_ready: false
```

## LANG007.C3

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
closable: false
```

The P5 plan itself requires T12 to have no open P5-caused failure before T13/T14 closure work. :chatgpt-content-reference{index="13"} That condition is plainly not met.

### Highest-priority blockers

The immediate order is:

**P5-AUD-001 → 002/003/004 → 005/006/012 → 008/009 → 010/011 → incremental revalidation.**

The existing `receiver_owner_evidence_is_published_without_owner_inference` failure remains real, but **it is no longer the sole or even the highest-severity open issue**.

The most important correction to the project record is therefore:

> **P5 is substantially implemented, but its new structural and Dynamic semantics have not yet been propagated coherently through all generic, scoped, inference, relation, and conformance paths. Existing focused PASS counts miss multiple semantic counterexamples. C3 cannot proceed to certification until those cross-layer gaps are repaired and the focused matrix is rebuilt around the canonical P5 invariants.**