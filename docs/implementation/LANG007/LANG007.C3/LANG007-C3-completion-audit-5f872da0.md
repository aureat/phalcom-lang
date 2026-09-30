# LANG007.C3 Completion Audit — Revision `5f872da0`

**Project:** Phalcom  
**Audit target:** `main` / `5f872da09c33c54c46d1b10c1ec6d763b1fd2506`  
**Commit subject:** `fix(semantic): refresh inferred conformance plans`  
**Audit date:** 2026-09-29  
**Disposition:** **C3 is not complete and is not presently closable.**  
**Recommended lifecycle state:** `IN_PROGRESS / PARTIAL / UNVERIFIED`

---

## 1. Executive conclusion

The C3 implementation is technically substantial and several of its hardest architectural decisions are now implemented in a disciplined way. In particular, Phalcom has a coherent canonical carrier for non-row static terms, a separate `RecordRowId` domain, explicit heterogeneous static arguments, first-class pack kinds, source-backed intrinsic structural facades, a non-injective projection solver that now retains residual proof obligations, correct source-aware complete-pack reassembly, increasingly strong Tuple/Record intrinsic projections, guarded/derived trait-dispatch evidence, and a canonical callable-signature authority that now publishes inferred trait-default results without introducing a second result owner.

Those are meaningful achievements. They do **not**, however, justify closing C3.

The current repository has at least four separate categories of closure blockers:

1. **A newly committed Tuple/Record typing specification is not the semantics implemented by the compiler.** It states that structural shapes are closed by default and that width subtyping requires an explicit open remainder. The live relation engine and accepted regression tests still implement classic immutable Record width subtyping for a closed super-record. The current AST also cannot represent several of the new spec's required forms, including optional structural members and unconstrained/typed structural remainders. This is a mathematical and normative contradiction, not documentation polish.

2. **P4.R1 is only through T3/G2.** T4/CV-05–CV-07 remains explicitly next. Cross-module inferred-result metadata, general dependent-consumer invalidation, same-result interface fingerprint stability, and cold/incremental parity remain to be accepted. T5 still owns omitted-result recursion and `Iterator.take` causal attribution; T6 still owns restoration of the natural iterator chain; T7 still owns normative callable-rule closure.

3. **The iterator surface still contains live type-safety and runtime defects independent of the R1 work.** `ChainIterator<I,J>` publishes `Item = I::Item` but returns values from `J` without constraining `J::Item = I::Item`; several adapters still erase their `Item` to `Dynamic`; and `Iterable.toMap` contains an extra `it.next` inside a `while let` loop, consuming a second element per iteration. The aggregate collection gate currently fails earlier, so downstream collection regressions can remain masked.

4. **Broad verification is not complete.** The latest R1 delivery ran focused selectors only. Existing P4 records still show unpassed collection aggregates, an incomplete workspace test attempt, failed Clippy/format gates, and unresolved current semantic regressions. The focused test quality is often high, but it is insufficient to establish end-to-end completeness.

Accordingly, the strongest defensible state is:

```yaml
C3:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: UNVERIFIED
  closable_now: false

P4:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: UNVERIFIED

P4_R1:
  T1: COMPLETE
  T2_G1: IMPLEMENTED / FOCUSED_TESTED
  T3_G2: IMPLEMENTED / FOCUSED_TESTED
  T4_G3: NOT ACCEPTED
  T5_G4: NOT ACCEPTED
  T6_G5: NOT ACCEPTED
  T7_final: NOT ACCEPTED
```

The most urgent architectural decision is not about the new inferred-result machinery. It is the **structural subtyping law**: either the late Tuple/Record spec is authoritative and the implementation must move to explicit openness, or the existing immutable width-subtyping model is authoritative and the new spec must be corrected. C3 cannot credibly claim type-theoretic closure while both models coexist.

---

# 2. Audit scope and method

## 2.1 Sources inspected

This audit reconciled four classes of evidence rather than trusting any one lifecycle document:

1. **Current repository implementation at `5f872da0`.**
   - `phalcom-semantic/src/types/relation.rs`
   - `phalcom-semantic/src/types/store.rs`
   - `phalcom-semantic/src/checker/inference.rs`
   - `phalcom-semantic/src/session.rs`
   - `phalcom-semantic/src/signature.rs`
   - `phalcom-semantic/src/db/fingerprint.rs`
   - `phalcom-semantic/src/impls.rs`
   - `phalcom-semantic/src/trait_dispatch.rs`
   - `phalcom-semantic/src/traits.rs`
   - AST type syntax in `phalcom-ast/src/ast.rs`
   - Universe Tuple, Record, Iterator, and Iterable source
   - representative semantic, incremental, core, and typing-integration tests.

2. **Current C3 lifecycle records.**
   - `LANG007.C3-CHECKPOINT.md`
   - P4 walkthrough/handoff
   - P4.R1 plan
   - INC-C3-21 and INC-C3-22.

3. **The prior P4 audit and remediation packets.**
   - projection-solver soundness
   - complete-pack canonicalization
   - nested derived-conformance runtime layout
   - semantic regression classification.

4. **Current normative or purportedly normative specifications.**
   - `docs/specs/tuple-and-record-typing.md`
   - `docs/specs/iterator.md`
   - existing Record-row and typing integration authorities.

The live source was used as implementation truth. Lifecycle documents were treated as claims to verify, not as proof that an implementation exists.

## 2.2 Verification boundary

This was a **source-and-evidence audit**, not a new full test run. The user reported that focused implementation tests passed at `5f872da0` and that no additional tests were run during delivery. I independently inspected the current source and committed tests, but I did not execute a new workspace suite in this audit.

That distinction matters. A source-visible defect such as the extra `it.next` in `Iterable.toMap` can be identified deterministically without running the program; a broad statement such as "the whole semantic suite now passes" cannot.

---

# 3. Status reconciliation: which records are authoritative?

## 3.1 Current checkpoint state is internally clear at the top level

The current checkpoint says C3/P4 is:

```text
IN_PROGRESS / PARTIAL / UNVERIFIED
```

and records T4/CV-05–CV-07 as the next P4.R1 scope. This is consistent with the current code history.

That status is correct.

## 3.2 The handoff and walkthrough contain superseded R1-era statements

The P4 handoff and T13 walkthrough preserve the earlier INC-C3-20 Outcome-B state in which unannotated trait-default results were *not* canonical contracts consumable by dispatch. That was subsequently superseded by INC-C3-21/P4.R1, which explicitly authorizes canonical inferred results for body-bearing callables including trait defaults.

The current checkpoint partially reconciles this by recording INC-C3-21/22 later in the file, but the older handoff remains easy to misread as current doctrine.

**Finding:** lifecycle documentation is historically useful but not self-consistent as a current state surface.

### Required correction

The handoff/walkthrough should acquire a conspicuous supersession note such as:

```text
Historical T13 state:
    INC-C3-20 Outcome B

Superseded for inferred-result architecture by:
    INC-C3-21
    INC-C3-22
    LANG007.C3.P4.R1

Current R1 state:
    T1–T3 complete/focused-tested
    T4 next
```

This avoids forcing future implementers to infer temporal precedence from append-only prose.

---

# 4. Severity summary

| ID | Severity | Finding | C3 impact |
|---|---|---|---|
| **C3-AUD-01** | **BLOCKER / TYPE THEORY** | New Tuple/Record spec says closed structural shapes do not width-subtype, while live Record relation and accepted tests do | C3 cannot claim coherent structural typing |
| **C3-AUD-02** | **BLOCKER / FEATURE GAP** | New optional-member and explicit structural-remainder forms are not representable in the live AST/type model | Normative spec is ahead of implementation |
| **C3-AUD-03** | **HIGH / R1 COMPLETION** | P4.R1 T4–T7 remain incomplete; metadata/fingerprint/incremental/recursion/natural-chain acceptance is open | Direct closure blocker |
| **C3-AUD-04** | **HIGH / INCREMENTAL RISK** | Callable signature product fingerprint includes inferred evidence status/origin, so same type with changed evidence shape may churn the public result fingerprint | CV-05/06 risk; unnecessary invalidation possible |
| **C3-AUD-05** | **HIGH / SOUNDNESS** | `ChainIterator<I,J>` advertises `I::Item` while yielding `J` items without an equality/conformance constraint | Published type contract can disagree with runtime values |
| **C3-AUD-06** | **HIGH / RUNTIME CORRECTNESS** | `Iterable.toMap` advances the iterator twice per `while let` iteration | Can skip every second input entry |
| **C3-AUD-07** | **HIGH / VERIFICATION** | `corpus::collections` currently stops at an earlier failure, masking later collection fixtures | Aggregate green status cannot be inferred from focused tests |
| **C3-AUD-08** | **MEDIUM-HIGH / ITERATOR TYPING** | FlatMap/Flatten/Zip/Indexed adapter items remain `Dynamic` despite richer accepted contracts elsewhere | Loss of static guarantees and downstream conformance precision |
| **C3-AUD-09** | **MEDIUM / PACK RELATION** | Pointwise `TypePack`/`LabeledTypePack` subtype arms appear behind the general `KindId::TYPE` gate for non-equal terms | Dead/misleading relation logic or missing kinded relation |
| **C3-AUD-10** | **MEDIUM / PERFORMANCE** | R1 refresh scans callable bodies iteratively and rebuilds conformance plans broadly | Potential O(C²)+O(I×R) analysis cost; not measured |
| **C3-AUD-11** | **MEDIUM / TRAIT RUNTIME COVERAGE** | Rest/module-export conformance-environment forwarding probes never produced a specialization recipe | Those forwarding seams remain unproven |
| **C3-AUD-12** | **MEDIUM / REGRESSION FLOOR** | Multiple known historical-current semantic regressions remain unresolved, including generic call inference | Reduces confidence in generic/trait integration |
| **C3-AUD-13** | **MEDIUM / TEST THEORY** | No systematic algebraic/property suite covers structural subtype reflexivity/transitivity/open-closed laws and pack/remainder separation | Edge cases can escape example-based tests |
| **C3-AUD-14** | **MEDIUM / CERTIFICATION** | G5/G6 are not green; workspace test result is incomplete and lint/format gates remain red | Release/checkpoint closure unsupported |

---

# 5. Findings in detail

## C3-AUD-01 — Closed Record semantics conflict with the newly committed specification

**Severity:** BLOCKER / TYPE THEORY  
**Confidence:** Confirmed from current source and tests.

The late `docs/specs/tuple-and-record-typing.md` defines structural types as **closed by default** and requires an explicit remainder to admit width.

Its intended law is effectively:

```text
{id: Int, name: String} </: {id: Int}

{id: Int, name: String} <: {id: Int, ...}
```

The live compiler implements something else.

`phalcom-semantic/src/types/relation.rs::check_record_row_subtype` compares every field required by the super-row and then, when the super-row tail is `Closed`, returns success without rejecting extra source fields.

In simplified form:

```rust
for sup_field in sup_row.fields {
    let Some(sub_field) = sub_row.field(&sup_field.name) else {
        return Refuted;
    };

    prove(sub_field.ty <: sup_field.ty)?;
}

match sup_row.tail {
    Closed => Proven,
    ...
}
```

Therefore:

```text
{a: Int, b: String} <: {a: Int}
```

is currently true.

The semantic test suite explicitly locks that law in:

```rust
fn test_record_subtyping_immutable_width() {
    // sub: { a: Int, b: String }
    // sup: { a: Int }

    assert!(is_subtype(sub, sup));
    assert!(!is_subtype(sup, sub));
}
```

Core typing integration likewise contains a positive test named:

```text
width_subtyping_accepts_extra_fields
```

with the comment that immutable width subtyping allows a wider Record to satisfy a narrower prefix.

This is not a minor doc drift. It changes the structural subtype relation itself.

### Type-theoretic consequence

Two different languages are currently described:

#### Model A — conventional immutable structural records

```text
{a: A, b: B} <: {a: A}
```

This is classic width + covariant depth subtyping.

#### Model B — exact closed shapes plus explicit openness

```text
{a: A, b: B} </: {a: A}
{a: A, b: B} <: {a: A, ...}
```

Here a closed record is an exact row and an open target is a separate structural type.

Both models can be coherent. They are not equivalent, and a compiler cannot implement both under the same syntax.

### Why this blocks C3

C3's own closure invariant requires the authoritative specifications to agree with shipped semantics. Structural Tuple/Record semantics are specifically within P4's completion scope. Closing C3 while the spec and relation engine define different lattices would institutionalize ambiguity in:

- parameter substitutability;
- overload/dispatch applicability;
- generic constraints;
- associated type equality/subtyping;
- row-polymorphic APIs;
- Record pattern typing;
- inference;
- future mutability/read-only rules;
- interface fingerprinting.

### Required decision

Choose one authority explicitly.

If **Model A** is intended, correct or revert the late spec's "closed means exact/no width" law and keep the existing row relation.

If **Model B** is intended, this is an implementation project, not a doc correction. It requires AST/type representation, relation rules, inference, diagnostics, metadata, formatting, LSP, and regression migration.

Do not "split the difference" by preserving current width behavior while merely adding `...` as decorative syntax.

---

## C3-AUD-02 — The new structural remainder and optional-member model is not implemented

**Severity:** BLOCKER / FEATURE GAP  
**Confidence:** Confirmed from current AST.

The new Tuple/Record spec describes a richer shape language, including:

```text
(Int, ...)
(Int, *: Number)
(name: String, **: Object)
(Int, ***: Dynamic)

{id: Int, ...}
{id: Int, **: Object}

{name?: String}
(name?: String)
```

The live AST cannot encode those semantics.

### Tuple AST today

Conceptually:

```rust
TypeAnnotationExpr::Tuple {
    elements: Vec<TypeTupleItem>,
}

enum TypeTupleItem {
    Element(TypeTupleElement),
    Expansion(TypePackExpansionSyntax),
}

struct TypeTupleElement {
    label: Option<String>,
    ty: TypeAnnotation,
}
```

There is no member-optional flag.

The `Expansion` arm is the existing `*` / `**` / `***` **pack expansion mechanism**, not a homogeneous open-remainder declaration.

### Record AST today

Conceptually:

```rust
TypeAnnotationExpr::Record {
    fields: Vec<RecordTypeField>,
    tail: Option<RecordRowTail>,
}

struct RecordTypeField {
    name: String,
    ty: TypeAnnotation,
}

struct RecordRowTail {
    name: String,
}
```

This can represent a named row-polymorphic tail, but not:

- an unconstrained anonymous `...`;
- a typed homogeneous remainder;
- optional fields.

### Architectural danger

The fastest wrong implementation would be to reuse pack expansion objects as structural openness. That would violate already-established C2/C3 boundaries:

```text
... is not pack syntax
LabeledTypePack != RecordRow
CompleteTypePack != Tuple
GenericParameterMode is separate from KindId
RecordRow remains row-owned
```

A structural remainder is a **shape constraint**, not simply a pack splice.

If the new spec is authoritative, introduce an explicit structural-remainder semantic representation or a carefully defined encoding into existing row/pack terms with a proof that the encoding preserves these distinctions. Do not overload `TypePackExpansionSyntax` merely because the spelling contains `*`.

---

## C3-AUD-03 — P4.R1 is not complete beyond T3/G2

**Severity:** HIGH / DIRECT COMPLETION BLOCKER

The current R1 plan is explicit.

### Completed

```text
T1 — read-only authority census
T2/G1 — canonical inferred result publication
T3/G2 — explicit/symbolic/conformance lifecycle controls
```

The latest implementation correctly uses the existing `CallableSignatureTable` as the result authority. `ConformanceWitnessPlan` is derived, replaceable state. Consumers depend on the plan. Changed plans invalidate and reanalyze dependent bodies. This is architecturally sound.

### Still open: T4/G3

T4 requires:

```text
changed inferred result
    -> dependent invalidation

same-result inferred body edit
    -> stable public result/interface fingerprint

explicit-result body edit preserving declared result
    -> stable result interface fingerprint

cross-module metadata
    -> round trip canonical inferred result

incremental changed/same-result edit
    -> cold parity with a dependent consumer
```

The current checkpoint itself records CV-05–CV-07 as next.

### Still open: T5/G4

T5 must establish the recursion boundary:

```text
direct omitted-result recursion
mutual omitted-result recursion
seeded cycles via explicit contract
Iterator.take causal attribution
```

The implementation must distinguish a genuine recursive semantic dependency from an artifact of analysis scheduling.

This is particularly important because the result refresh function already performs bounded repeated reanalysis. Without T5, it is not yet proven that the implementation preserves the promised "no general recursive result fixpoint" rule under all cycles.

### Still open: T6/G5

The natural chain:

```phalcom
(0..).iter.take(10).toList
```

remains the integration discriminator.

This matters because it composes:

```text
iterator result typing
trait default inferred results
generic Self/result specialization
guarded Derived Iterable evidence
ordinary dispatch
lowering
runtime library behavior
```

A known-receiver surrogate is good evidence for T13's guarded dispatch machinery; it is not a substitute for the natural end-to-end chain after R1 deliberately changes the result architecture.

### Still open: T7/final

The callable typing specification must be updated once the actual recursion and metadata rules are known, then the directly blocked collection selector must be rerun under the R1 authorization.

Therefore R1 itself cannot yet be marked complete.

---

## C3-AUD-04 — Same-type inferred results may still churn interface fingerprints

**Severity:** HIGH / INCREMENTAL CORRECTNESS & PERFORMANCE RISK  
**Confidence:** Strong risk from current fingerprint code; T4 is the designed acceptance gate for exactly this issue.

`CallableSemanticSignature` contains:

```rust
declared_return
return_validation
inferred_return: Option<TypeKnowledge>
```

The product fingerprint hashes `inferred_return`.

`hash_type_knowledge` hashes, for a known type:

```text
TypeId
EvidenceStatus
EvidenceOrigin
```

even when source provenance is excluded.

This means two implementations that expose the same semantic return type can produce different callable signature fingerprints if body analysis changes the evidence origin or status.

For example:

```phalcom
// version A
value() { "text" }

// version B
value() { helperReturningString() }
```

Both may publish `String`, yet body-derived evidence can have a different origin.

R1's current same-result test changes one String literal to another:

```text
"text" -> "other text"
```

That is useful, but it is not adversarial against origin/status churn.

### Why this matters

The public callable contract is the type-level result. Body proof provenance is analysis evidence.

If the interface fingerprint changes merely because the source of proof changed while the published contract remains `String`, then:

- downstream signatures can be invalidated unnecessarily;
- conformance plans may rebuild;
- clients may reanalyze;
- incremental work scales with body refactors that did not change the interface.

This is especially expensive after INC-C3-22 because conformance plans now depend directly on callable signature fingerprints.

### Required CV-05 discriminator

Add at least:

```text
same inferred TypeId
different body evidence origin
same callable public product fingerprint
same dependent client product
```

and separately decide whether `EvidenceStatus` is part of the public interface. If status affects legality, keep exactly the semantically meaningful status bit; if it only reflects body proof quality, keep it out of interface identity.

A clean model is:

```text
declaration/body analysis product
    contains proof/evidence provenance

callable public signature fingerprint
    contains canonical published result contract
```

Do not let body-local evidence detail become accidental ABI/interface identity.

---

# 6. TypePack and CompleteTypePack audit

## 6.1 Architecture is largely strong

The established separation is good:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordRow
Tuple
```

are not aliases for each other.

Important invariants are preserved:

```text
LabeledTypePack != RecordRow
CompleteTypePack != Tuple
* / ** / *** are expansion lanes
... is not pack syntax
whole-pack inference does not imply pack partition search
RecordRow stays outside ordinary TypeId argument storage where required
```

This is the correct direction for a language with heterogeneous tuple products and true record rows.

## 6.2 Projection-solver soundness defect is genuinely repaired

The old bug was serious.

A non-injective equality such as:

```text
Tuple<X>::Element == Tuple<Y>::Element
```

cannot imply:

```text
X == Y
```

Likewise:

```text
Positionals(P) == Positionals(Q)
```

does not prove `P == Q`.

The earlier solver conflated:

```text
"no substitution was produced"
```

with:

```text
"the obligation was proven"
```

and could finish `Solved` after unrelated variables acquired substitutions.

The remediation added explicit residual obligations owned by the existing inference session. That is the correct mathematical repair:

```text
non-injective projection stuck
    -> residual proof obligation

independent substitutions happen
    -> retry projection

projection normalizes equal
    -> discharge

projection normalizes unequal
    -> conflict

still stuck at finish
    -> not Solved
```

This restores the key logical distinction between **solver progress** and **constraint proof**.

The direct solver tests described in the remediation packet are the right form of tests. Testing only normalized `TypeStore` expressions would not prove solver completion behavior.

## 6.3 Complete-pack reassembly law is now correct

The canonical law:

```text
Complete(Positionals(P), Labeled(P)) ≡ P
```

is valid when `Positionals` and `Labeled` are the canonical intrinsic operations over the **same** complete-pack subject.

The remediation correctly made source-backed projections participate without discarding their source declaration identity.

The negative guards are important:

```text
different subjects
    -> no collapse

unrelated user member named Positionals
    -> no collapse

wrong result kind
    -> no collapse
```

This is now a well-scoped normalization law rather than a name-based rewrite.

## 6.4 Potential dead relation logic for pack subtyping

The relation implementation contains pointwise arms for:

```text
TypePack
LabeledTypePack
```

but the general subtype entry first requires both sides to be proper `Type`-kinded terms for non-identical inputs.

Since packs have their own kinds, distinct pack terms appear unable to reach those pointwise subtype arms through ordinary `is_subtype`.

That produces an architectural ambiguity:

- if packs intentionally have no general subtype lattice, the arms are misleading/dead and should be removed or moved to a kinded relation API;
- if pointwise pack subtyping is intended, the general proper-Type gate is too strong for that operation.

### Recommendation

Make the law explicit instead of leaving unreachable-looking code.

One possible separation:

```rust
check_type_subtype(TypeId, TypeId)          // requires proper Type
check_static_term_relation(TypeId, TypeId)  // kind-aware equality/order where defined
check_pack_relation(...)
```

This should **not** mean introducing a general lattice over arbitrary static domains. It means each kind's permitted relation is explicit.

## 6.5 Missing pack law/property suite

Example regressions exist, but C3 would benefit from a compact algebraic test module proving laws such as:

```text
project_complete(P).positionals/labeled preserve lane order
Complete(Positionals(P), Labeled(P)) = P

Positionals(Complete(A,B)) = A
Labeled(Complete(A,B)) = B

pack concat identity
pack concat associativity after canonical flattening

labeled duplicate rejection is deterministic
source provenance does not alter canonical term identity
different intrinsic operation identity prevents accidental rewrite

pack projection equality never reverse-solves subject
whole-pack inference never turns into arbitrary partition search
```

Property-style generation over small packs would be more valuable than dozens of hand-picked examples.

---

# 7. Tuple typing audit

## 7.1 Canonical tuple representation is improved

The P4 direction is sound:

```text
source family:
    Tuple<***Es>

semantic target:
    structural tuple / TupleFromPacks

behavior owner:
    canonical Universe Tuple declaration

runtime owner:
    preinstalled Tuple runtime class
```

This avoids turning every tuple shape into a nominal class while still giving all structural tuple shapes one behavior family.

The owner/source/index tests described in the walkthrough are valuable because they verify that equivalent source spellings converge before overlap/member resolution.

## 7.2 `Tuple::Element` has a coherent concrete law

For a fully enumerable Tuple shape:

```text
Tuple::Element = union(all positional payload types,
                       all labeled payload types)
```

and:

```text
Element(()) = Never
```

For an unenumerable symbolic pack-backed Tuple, retaining a symbolic source-backed projection is preferable to inventing a partial union.

That is mathematically coherent.

## 7.3 Tuple iterator specialization is now correct

Current source preserves:

```text
Tuple<***Es>
    -> TupleIterator<***Es>
    -> Iterator::Item = Tuple<***Es>::Element
```

This is a material improvement over `Dynamic`.

Runtime-shaped operations such as `rawFromList`, runtime slicing, and runtime-arity zip may legitimately stay behind a `Dynamic` boundary if the language has not introduced existential tuple lengths or dependent arithmetic.

## 7.4 New open-Tuple spec is not implemented

The new structural typing spec requires much more:

```text
(Int, ...)
(Int, *: Number)
(name: String, **: Object)
...
```

Current tuple relation uses exact cardinality and label matching for concrete Tuple types. There is no open tuple remainder relation corresponding to the spec.

If explicit openness is adopted, the relation needs a precise lane calculus.

For example:

```text
(A, B) <: (A, ...)
```

looks obvious, but the following need defined answers:

```text
(A, x: X) <: (A, ...)

(A, x: X) <: (A, *: Object)

(A, x: X) <: (A, **: Object)

(A, x: X) <: (A, ***: Object)

(A, B, x: X) <: (A, *: Object, **: Object)

(A, B) <: (*: Object)
```

The implementation must not infer these ad hoc from pack syntax.

---

# 8. Record typing, rows, and width subtyping

## 8.1 Existing row engine has good foundations

The row subsystem has several sound design properties:

- fields canonicalize independent of source order;
- duplicate fields are rejected;
- row tails have their own identity/domain;
- occurs checks reject `R = {next: T, **R}`;
- lacks constraints prevent illegal duplicate extension;
- solver-local remainders are not eagerly interned into the canonical `TypeStore`;
- open row inference is separate from ordinary `InferVarId`;
- `RecordRow` is not smuggled into `TypeData::Parameter`.

These are strong implementation choices.

## 8.2 Existing Record relation is classic width + covariant depth

The current relation means:

```text
S <: T
```

for Records if every field required by `T` exists in `S` and the corresponding value type is a subtype.

This is valid for immutable records.

It also means a closed target is a **minimum required field set**, not an exact shape.

That conflicts with the new spec, as described in C3-AUD-01.

## 8.3 Open-row subtyping remains conservative

When the target row has a symbolic parameter/projection tail, the relation generally proves only identity-compatible tail cases and otherwise blocks/defer rather than inventing a proof.

That is safer than pretending an unknown residual row is compatible.

However, if the new explicit-open model is adopted, this conservative behavior will need to be generalized systematically rather than patched case-by-case.

## 8.4 Optional fields require a first-class rule

If:

```text
{name?: String}
```

is adopted, do **not** define it as:

```text
{name: String | Unit}
```

That changes "field may be absent" into "field is definitely present and may contain Unit."

A sound optional-field model needs to represent presence separately.

Typical relation laws would need explicit ratification, for example:

```text
{name: String} <: {name?: String, ...}

{} <: {name?: String, ...}

{name: Int} </: {name?: String, ...}
```

For a closed exact record, you must additionally decide whether:

```text
{} <: {name?: String}
```

is legal and whether any *other* undeclared field is allowed. Under the new spec's "closed exact" interpretation, undeclared fields would not be allowed even though optional declared fields may be absent.

This is exactly why optionality cannot be bolted onto the existing field map without a formal shape model.

---

# 9. Iterator and Iterable audit

## 9.1 Primitive trait architecture is good

The canonical architecture is sound:

```phalcom
trait Iterable {
    type Item
    type Iterator
    iterator -> Self::Iterator
}

trait Iterator {
    type Item
    next -> Option<Self::Item>
}
```

and the blanket law:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
{
    type Item = <I as Iterator>::Item
    type Iterator = I
    iterator { self }
}
```

is a clean use of guarded generic conformance rather than a compiler special case.

The T13 work also successfully moved ordinary trait dispatch to consume canonical Exact/Forwarded/Derived proof evidence instead of inventing dispatch-local proof search.

## 9.2 `ChainIterator` is still unsound

Current source:

```phalcom
class ChainIterator<I, J> {
  _first: I
  _second: J

  next {
    ...
    if _phase == 1 {
      let step = _second.next
      if step.isSome { return step }
      ...
    }
  }
}

impl<I, J> Iterator for ChainIterator<I, J> {
  type Item = <I as Iterator>::Item
}
```

Missing from the published contract are constraints equivalent to:

```text
I |= Iterator
J |= Iterator
<J as Iterator>::Item == <I as Iterator>::Item
```

Therefore a chain of an `Iterator<Int>` and an `Iterator<String>` can claim:

```text
Iterator::Item = Int
```

while the implementation can return a String-bearing `Some`.

That is a genuine type-soundness problem.

It was already identified by the earlier LANG005 iterator audit and remains in current source.

### Required test

At minimum:

```text
chain(Int iterator, Int iterator)     -> accepted
chain(Int iterator, String iterator)  -> rejected
```

The negative must fail at semantic contract formation/application, not at runtime.

## 9.3 Several adapter results remain erased to `Dynamic`

Current source still contains:

```phalcom
impl<I, S> Iterator for FlatMapIterator<I, S> {
  type Item = Dynamic
}

impl<I> Iterator for FlattenIterator<I> {
  type Item = Dynamic
}

impl<I, J> Iterator for StrictZipIterator<I, J> {
  type Item = Dynamic
}

impl<I> Iterator for IndexedIterator<I> {
  type Item = Dynamic
}
```

Some runtime-shaped Tuple zip machinery is intentionally dynamic, but the generic iterator adapter surface had a stronger intended contract:

```text
filter preserves Item
filterMap maps to U
flatMap maps to inner Iterable::Item
flatten requires outer Item |= Iterable
chain requires equal Item types
strict zip produces (I::Item, J::Item)
indexed produces (Int, I::Item)
```

Leaving these as `Dynamic` is not merely lower IDE quality. It breaks trait-associated type precision and can hide incompatible compositions that the type system is otherwise designed to reject.

## 9.4 `Iterable.toMap` consumes an extra element

Current source at `iterator_laws.ph`:

```phalcom
toMap -> Result<Map<Dynamic, Dynamic>, DuplicateKeyError> {
  let result = Map<Dynamic, Dynamic>.new()

  let it = self.iterator
  while let Some(entry) = it.next {
    let key = entry.key
    if result.includes(key) {
      return Result::Error(DuplicateKeyError.new(key))
    }
    result.insert(entry.value, for: key)
    step = it.next
  }
  return Result::Ok(result)
}
```

The final line inside the loop:

```phalcom
step = it.next
```

is not used by the `while let` condition.

On the next iteration, the condition evaluates:

```phalcom
it.next
```

again.

Therefore the loop advances twice per body execution.

For input:

```text
a, b, c, d
```

the body can observe:

```text
a, c
```

while `b` and `d` are consumed by the unused assignment.

This is a deterministic source-level correctness defect.

The active `eager_operations.ph` fixture expects a two-entry list to create a map of size two. However, the aggregate collections gate currently stops earlier at the `TakeIterator.toList` path, demonstrating why aggregate ordering is not sufficient coverage.

### Required repair/test

Delete the extra read:

```phalcom
while let Some(entry) = it.next {
    ...
}
```

and run a dedicated exact selector with at least 3–4 entries so a double-advance bug is impossible to miss.

## 9.5 Natural chain remains a valuable integration gate

Once R1 T4/T5 are complete, the natural chain should be reinstated:

```phalcom
(0..).iter.take(10).toList
```

It is particularly useful because it joins:

- unannotated trait-default return publication;
- `Self`/generic result specialization;
- guarded derived `Iterable` evidence;
- generic iterator adapters;
- canonical trait lowering;
- actual collection conversion.

A hand-constructed known receiver cannot exercise all of these dependencies.

---

# 10. Trait and conformance audit

## 10.1 Canonical authority after INC-C3-21/22 is well chosen

The current rule is:

```text
CallableSignatureTable
    = canonical callable result authority

ConformanceWitnessPlan
    = derived projection

TraitSurface
    = declaration/requirement surface, not a second inferred-result owner
```

This avoids a common architectural failure where:

```text
signature.result
trait_surface.result
conformance_plan.result
dispatch.result
```

all become independently mutable caches.

The current R1 implementation does not do that.

## 10.2 Focused lifecycle test is unusually strong

`inferred_trait_result_replaces_conformance_plans_and_rechecks_clients` validates many important properties in one incremental scenario:

- canonical inferred result on the trait default;
- explicit and bodyless controls;
- compatible and incompatible witnesses;
- plan requirement views;
- plan dependency on the callable signature;
- plan dependency on the trait surface;
- caller dependency on the conformance plan;
- same-result body edit preserving plan fingerprint;
- unrelated cross-module plan pointer reuse;
- changed result replacing the plan;
- stale diagnostic retirement;
- client reanalysis;
- changed-result cold parity.

This is high-value testing.

## 10.3 It is still synthetic and does not close T4

The test does not yet establish:

- cross-module serialized metadata roundtrip of the inferred public result;
- a general dependent consumer outside the conformance-specific path;
- same Type with changed body evidence origin;
- explicit-result body edit interface stability;
- full source/import/incremental interaction.

Those are exactly T4's remaining CV-05–CV-07 obligations.

## 10.4 Derived conformance runtime slot repair is soundly designed

The CV50 failure was real: nested proofs for distinct source constraints were being projected into the same runtime evidence slot.

The repair now:

1. reads the selected source implementation's generic signature;
2. constructs its canonical `GenericEvidenceLayout`;
3. aligns nested evidence to the original source-constraint sequence;
4. ignores non-conformance placeholders for slot consumption;
5. keys executable conformance slots by canonical `GenericEvidenceSlotKey`.

That is the right architecture because runtime layout is a projection of already-selected semantic evidence, not a second proof search.

## 10.5 Rest/module-export transport is still unproven

The remediation probes for specialized rest dispatch and module-export forwarding stopped earlier because semantic analysis did not publish a callable/specialization recipe at the probe sites.

Therefore they establish only:

```text
no recipe reached the suspected forwarding path
```

not:

```text
the forwarding path preserves a non-empty conformance environment
```

Do not mark these seams proven.

A future test must first produce a real specialization recipe and then verify that the exact runtime type/conformance environments survive the forwarding boundary.

---

# 11. Inferred result refresh: correctness and complexity

## 11.1 Correctness direction

`refresh_inferred_callable_results` now updates canonical signatures after body analysis and reanalyzes consumers of changed result contracts.

For trait defaults it republishes the canonical callable signature query.

After result stabilization, conformance plans are rebuilt from the current canonical signature table and changed plan fingerprints are compared against the prior snapshot. Reverse dependency closure is used to find client bodies that consumed changed plans.

This is coherent with INC-C3-22.

## 11.2 Complexity risk

The implementation is deliberately conservative.

At a high level:

```text
for up to O(C) iterations:
    scan O(C) callable analyses

    if results changed:
        walk semantic-work modules
        walk class members
        reanalyze affected callables

then:
    rebuild lookup-eligible conformance plans
        for each impl
            for each trait requirement
                resolve/specialize/check candidate
```

The bounded result loop can therefore approach **O(C²)** scanning even before body-analysis cost.

Conformance refresh adds approximately:

```text
O(number_of_impls × requirements_per_trait × witness_resolution_cost)
```

and may happen after an earlier provisional conformance publication in the same semantic pass.

The code also clones the callable signature table for some reanalysis boundaries.

None of this is necessarily unacceptable for the current compiler scale. It is, however, a real incremental-performance risk introduced at a central analysis seam.

## 11.3 Memory implications

The architecture correctly reuses `Arc` products when fingerprints are unchanged. That is good.

Potential transient cost still includes:

- previous and current conformance plan maps;
- cloned signature snapshots;
- dependency closure sets;
- replacement callable analyses;
- residual query products retained across the revision.

No evidence suggests a leak, but no R1-specific memory/resource measurement is recorded.

## 11.4 Recommended optimization boundary

Do not optimize before T4/T5 correctness is accepted.

Afterward, prefer:

```text
changed callable signature
    -> reverse dependency index
    -> affected conformance plans only
    -> affected clients only
```

rather than repeatedly scanning all candidate products.

An SCC/worklist can also distinguish acyclic return propagation from true omitted-result recursion and reduce repeated global scans.

Performance tests should record:

```text
N independent callables, one result edit
    -> O(1) or bounded-local recomputation

N independent conformances, one trait result edit
    -> only conformances depending on that requirement rebuild

same-result body edit
    -> no downstream plan/client reanalysis
```

---

# 12. Type-theoretic assessment

## 12.1 Strong foundations

The following decisions are mathematically disciplined:

### Canonical static terms

```text
TypeId + KindId
```

for non-row static terms avoids proliferating arenas and identity systems.

### Proper type proof boundary

`ProperTypeId` prevents arbitrary static-domain inhabitants, packs, and other kinded terms from silently becoming runtime/proper types.

### Row separation

`RecordRowId` remains a separate domain. This is correct because row variables have different unification structure and occurs/lacks constraints from ordinary first-order type terms.

### Non-injective projections

Residual proof obligations are necessary. The solver now correctly refuses to interpret non-progress as proof.

### Nominal static domains

User-defined `type data` domains are nominal while their constructors can still be structurally unified within the same domain/case identity.

### Conformance as proof, not dispatch guesswork

Guarded/derived dispatch asks the canonical conformance engine for evidence. Lowering consumes selected evidence. This respects proof authority.

## 12.2 Current theoretical hole: structural exactness vs width

The language does not currently have one consistent structural shape theory.

Until that is resolved, it is impossible to state a single correct theorem such as:

```text
Record subtyping is reflexive, transitive, and width-covariant
```

or:

```text
closed Record types denote exact finite rows
```

because the docs and code disagree.

## 12.3 Dynamic must remain a boundary, not a proof

The project has correctly resisted treating `Dynamic` as evidence that an associated projection is statically satisfied.

That rule should remain.

Using `Dynamic` to paper over iterator adapter typing may be an accepted dynamic boundary for truly runtime-shaped data, but it should not be used where generic constraints can express the exact type.

## 12.4 Recursion semantics still need proof

The result refresh now resembles a bounded iterative dependency process.

That is fine for propagating acyclic dependencies in arbitrary processing order, but it must not accidentally become the semantics of recursive result inference.

T5 should establish:

```text
f() { g() }
g() { f() }
```

with both results omitted does **not** acquire a type merely because repeated iteration stabilizes on a permissive unknown/union state.

By contrast:

```text
f() -> Int { g() }
g() { f() }
```

has an explicit seed and may be analyzable.

The cycle policy belongs in the semantic dependency graph, not in incidental iteration count.

---

# 13. Test-coverage audit

## 13.1 Where coverage is strong

| Area | Coverage quality | Assessment |
|---|---|---|
| Projection non-injectivity | Direct solver-level residual tests | **Strong** |
| Complete-pack source identity | Internal/source-backed/mixed/negative canonicalization tests | **Strong focused evidence** |
| Tuple iterator specialization | Source + semantic + core focused selectors | **Strong focused evidence** |
| Tuple/Record intrinsic owner identity | Canonical owner/equivalent spelling/collision tests | **Strong** |
| Record row occurs/lacks/canonical order | Law-oriented semantic tests | **Strong** |
| Trait exact/derived dispatch | Concrete, symbolic, ineligible, ambiguity, lowering parity | **Strong focused evidence** |
| Inferred trait result → plan lifecycle | Dense incremental/cold-parity test | **Strong but narrow** |
| Nested conformance slot layout | Exact runtime regression | **Strong for CV50** |
| Metadata schema compatibility for earlier C3 features | Focused v8/v9 compatibility tests | **Good** |

## 13.2 Where coverage is insufficient

| Area | Missing evidence |
|---|---|
| New Tuple/Record spec | Parser/AST/semantic tests for optional fields and explicit remainders do not exist because machinery does not exist |
| Record closed/open subtype law | Existing tests assert the opposite of the late spec |
| Tuple open subtype law | No representation or law tests |
| Inferred result fingerprint stability | Same type through different evidence origin/status not tested |
| R1 cross-module metadata | T4 not accepted |
| R1 general dependent consumers | T4 not accepted |
| Omitted-result recursion | T5 not accepted |
| Natural iterator chain | T6 not accepted |
| Iterator chain item equality | Known live source unsoundness; missing negative |
| `toMap` double advance | Aggregate collection test can be masked by earlier fixture |
| Adapter Item typing | Multiple adapters still `Dynamic` |
| Rest/module-export conformance environment | Probe never reaches a specialization recipe |
| Performance | No R1-specific scaling/resource test |
| Full workspace | Last test attempt incomplete |
| Clippy/format | Broad gates red |
| Current semantic regressions | Several earlier-green failures remain unresolved |

## 13.3 Aggregate corpus tests are not enough

The current collection gate is especially illustrative.

If the aggregate executes fixtures in order:

```text
fixture A -> fails
fixture B -> not reached
fixture C -> not reached
```

then fixing B/C is neither proved nor disproved.

P4 records correctly say the aggregate is not green, but the practical consequence should be stronger: **every critical migrated/semantic feature needs a focused exact selector independent of aggregate ordering.**

`Iterable.toMap` demonstrates why.

---

# 14. Required algebraic/property test suite

A completion-grade type system should test laws, not only examples.

## 14.1 Proper Type relation

For generated small proper types where applicable:

```text
reflexivity:
    T <: T

transitivity:
    A <: B and B <: C => A <: C
```

Antisymmetry should **not** be assumed globally because nominal equivalences/aliases and representation-normalized terms can complicate identity; test canonical equivalence separately.

## 14.2 Record rows

Under whichever structural model is ratified:

```text
field permutation does not affect canonical closed-row identity
duplicate field formation rejects
depth covariance is transitive
tail occurs check holds
lacks constraints are preserved through substitution
open-tail substitution preserves field constraints
no speculative row term enters durable TypeStore identity
```

If classic width is retained:

```text
{a:A,b:B} <: {a:A}
```

If explicit openness is adopted:

```text
{a:A,b:B} </: {a:A}
{a:A,b:B} <: {a:A,...}
```

and that exact distinction must be tested throughout inference and call applicability.

## 14.3 Optional members

If adopted:

```text
required field satisfies optional field
absence satisfies optional field where permitted
wrong present type refutes
optional absence is not encoded as Unit
extra undeclared fields obey closed/open remainder law
```

## 14.4 Pack algebra

```text
Complete(Positionals(P), Labeled(P)) = P
Positionals(Complete(A,B)) = A
Labeled(Complete(A,B)) = B

concat identity
canonical flattening associativity
label order preservation
duplicate label rejection
source declaration provenance does not alter operation identity
unrelated same-name member cannot trigger intrinsic rewrite
```

## 14.5 Projection inference

```text
F<X> = F<Y> never reverse-solves X=Y for non-injective F
stuck residual prevents Solved
independent subject solution retries residual
equal normalized result discharges
unequal normalized result conflicts
zero-variable symbolic residual still prevents Solved
```

## 14.6 Callable inferred-result incrementality

```text
same type / different literal
same type / literal vs helper call
same type / branch vs direct return
same type / changed source ranges
same type / changed evidence origin
changed type
explicit return / arbitrary compatible body change
```

Measure both:

```text
product fingerprint
recomputed dependent count
```

---

# 15. Performance and memory assessment

## 15.1 TypeStore canonicalization

Pack and row canonicalization appear structurally bounded and rely on interning, which is appropriate.

Risks to watch:

- concat normalization repeatedly rebuilding arrays;
- source-backed projections retaining extra source identity objects;
- repeated symbolic projection normalization;
- cloning stores for query/probe paths.

No current evidence shows an asymptotic leak in C3's pack representation.

## 15.2 Row solver

The row solver correctly avoids interning speculative remainders during solving. That is a significant memory-quality feature.

The fixed-point interaction between ordinary inference and row inference can still become expensive for deeply nested mixed equations, but the architecture is preferable to collapsing rows into the ordinary type solver.

Recommended stress dimensions:

```text
row field count
nested generic depth
number of row variables
cross-frame equations
open-tail chain length
```

## 15.3 Conformance plans

The current plan structure is rich:

- requirement projections;
- proof provenance;
- nested evidence;
- associated type plan;
- dependencies;
- fingerprints.

This is correct but can be memory-heavy at scale.

Pointer reuse for unchanged `Arc<ConformanceWitnessPlan>` products is important and is already tested in the R1 lifecycle scenario.

The next step is not compression; it is measuring:

```text
plans retained per revision
bytes per plan
number of dependency edges per plan
rebuild count after a one-signature edit
```

## 15.4 Result refresh

The R1 result refresh is the most obvious new CPU scaling concern.

A correctness-first full scan is acceptable temporarily, but C3 should not claim performance/resource closure without measuring it because the original acceptance contract includes incremental correctness and P3/P4 records repeatedly mention resource evidence.

---

# 16. Metadata and persistence audit

Earlier C3 metadata work is comparatively mature:

- static-domain cataloging;
- heterogeneous static arguments;
- scoped RecordRow v9 node;
- source-backed projection identity;
- parameter kind/mode separation;
- Unit/EmptyRecord zero-form identity;
- incremental fingerprint tests.

The open issue is **R1 inferred-result metadata**, not the general C3 metadata substrate.

T4 should prove:

```text
provider:
    unannotated callable -> inferred canonical R

export:
    metadata publishes R through existing callable record

consumer:
    imports provider
    observes R

incremental provider body edit:
    R changes -> consumer invalidates

same-result edit:
    exported callable interface fingerprint stable

cold analysis:
    exact same metadata-visible result
```

Do not add a second serialized "inferred result" field if the existing published return slot already represents the canonical contract.

---

# 17. Regression-floor assessment

The semantic regression classification work was useful because it stopped incorrectly attributing every failure to P4.

However, classification is not remediation.

The current records still identify several earlier-green failures whose causes are not isolated, including ordinary generic inference cases such as:

```text
generic_callable_return_inference_requires_exact_parameter_domain
generic_calls_in_one_body_do_not_share_type_variables
generic_identity_solves_parameter_from_argument_and_specializes_return
generic_conflict_is_local_to_call_and_sibling_remains_established
```

These predate P4 at the immediate predecessor, so they are not evidence that P4 introduced the bug. But they are highly relevant to confidence in:

- generic callable result specialization;
- caller-local inference variables;
- sibling-call isolation;
- inferred result publication.

C3 may be permitted to defer unrelated regressions by program policy, but a final type-system closure report must state clearly that these remain live known defects.

Do not translate:

```text
not caused by P4
```

into:

```text
not relevant to language correctness
```

Those are different statements.

---

# 18. Completion matrix

| C3 area | Implementation status | Verification status | Audit verdict |
|---|---|---|---|
| Unified `data` / `type data` foundation | Implemented | Focused historical gates | Acceptable |
| Nominal static domains | Implemented | Focused | Acceptable |
| Heterogeneous static arguments | Implemented | Focused | Acceptable |
| Pack kinds / whole-pack inference | Implemented | Substantial focused coverage | Good, with relation cleanup needed |
| Projection residual soundness | Repaired | Direct focused solver tests | Acceptable |
| Complete-pack source-aware reassembly | Repaired | Focused | Acceptable |
| Intrinsic Tuple/Record facade ownership | Implemented | Focused | Acceptable |
| Tuple element projection | Implemented | Focused | Acceptable |
| Record value projection | Implemented | Focused | Acceptable |
| Tuple iterator specialization | Implemented | Focused | Acceptable |
| Record structural subtyping | Implemented under old law | Tested under old law | **Conflicts with new spec** |
| Optional structural members | Not implemented | No coverage | **Blocker if spec authoritative** |
| Explicit tuple/record open remainders | Partial/absent | No matching coverage | **Blocker if spec authoritative** |
| Guarded/derived trait dispatch | Implemented | Strong focused matrix | Good |
| Nested runtime conformance evidence layout | Repaired | Focused named regressions | Good, not broad |
| Canonical inferred trait-default result | Implemented | Focused | Good |
| Conformance-plan refresh after inferred result | Implemented | Dense focused lifecycle test | Good |
| R1 interface fingerprint/metadata/cold parity | Incomplete | T4 pending | **Open** |
| Omitted-result recursion policy | Incomplete | T5 pending | **Open** |
| `Iterator.take` causal attribution | Incomplete | T5 pending | **Open** |
| Natural iterator chain | Blocked/not restored | T6 pending | **Open** |
| Iterator adapter type soundness | Incomplete | Known gaps | **High** |
| Collection `toMap` | Live source defect | Dedicated post-R1 test not established | **High** |
| G5 collection aggregate | Not green | Fails before full corpus | **Open** |
| Workspace test | Incomplete run | Unknown after core lib | **Open** |
| Clippy | Fails | Red | **Open** |
| Workspace format | Fails | Red | **Open** |
| Global diff check | Blocked by preserved unrelated file | Red/non-C3 | Lifecycle disposition needed |
| C3 closeout | Not reached | Not certified | **Not closable** |

---

# 19. Recommended remediation order

## Phase 0 — Resolve the structural type law before more "completion" work

### 0.1 Decide Record/Tuple exactness

Choose:

```text
A. conventional immutable width subtyping
```

or:

```text
B. closed exact shapes + explicit-open remainder subtyping
```

Record the decision in one authoritative spec.

### 0.2 Reconcile specs and tests immediately

If A:

- correct `tuple-and-record-typing.md`;
- retain row width tests;
- define what any remainder syntax is for if still desired.

If B:

- amend the implementation plan;
- add AST forms for optionality/remainders;
- implement type representation and relation laws;
- migrate old width tests;
- add metadata/LSP/formatting;
- prove inference/pattern interaction.

Do not leave this conflict until final documentation cleanup.

---

## Phase 1 — Fix live correctness defects that can mask later gates

### 1.1 `Iterable.toMap`

Remove the second advancement and add a dedicated multi-entry regression.

### 1.2 `ChainIterator`

Add proper Iterator/equality constraints or redesign the adapter signature to carry an explicit common item type.

At minimum, reject heterogeneous chains that cannot satisfy one `Iterator::Item`.

### 1.3 Classify adapter Dynamic boundaries

For each of:

```text
FlatMapIterator
FlattenIterator
StrictZipIterator
IndexedIterator
```

state whether Dynamic is intentional and semantically unavoidable.

Where the generic result is statically expressible, replace Dynamic with the real associated projection.

---

## Phase 2 — Complete P4.R1 T4/CV-05–CV-07

### 2.1 Fingerprint stability

Test same `TypeId`, different proof/evidence path.

### 2.2 Metadata roundtrip

Cross-module inferred trait/default and ordinary callable results.

### 2.3 General dependent consumer

Not only conformance-plan clients.

### 2.4 Cold/incremental parity

Changed and unchanged public result cases.

Only then mark G3.

---

## Phase 3 — Complete R1 T5 recursion and `Iterator.take`

Trace the exact dependency cycle.

Required cases:

```text
unseeded direct recursion
unseeded mutual recursion
explicitly seeded recursion
acyclic dependency chain
Iterator.take
```

If `Iterator.take` is blocked only because scheduling treats an acyclic dependency as recursive, fix the dependency owner/order.

Do not add a general widening/fixpoint merely to make it pass.

---

## Phase 4 — Restore the natural chain

Run the exact R1 discriminator:

```phalcom
(0..).iter.take(10).toList
```

Inspect:

```text
receiver type after each stage
canonical callable result
derived Iterable proof
nested Iterator proof
lowered TraitInvocationSpec
runtime result
```

Then run the directly blocked collection selector.

---

## Phase 5 — Add law-level structural tests

Regardless of which subtyping model is chosen, create a compact algebraic suite rather than relying on individual feature tests.

This should become the long-term regression floor for C3.

---

## Phase 6 — Performance/incremental measurement

After correctness:

- one inferred-result edit across N independent callables;
- one trait-default result edit across N conformances;
- same-result edit;
- result-changing edit;
- peak retained semantic products;
- dependency-edge counts.

Use those measurements to decide whether targeted worklists are needed.

---

## Phase 7 — Broad certification

Run the complete intended P4 gates once the feature-level blockers are removed.

Do not report a green gate while annotating failing subcommands as "baseline."

A useful final matrix should distinguish:

```text
PASS
ACCEPTED_BASELINE_BLOCKED
FAIL_CURRENT
NOT_RUN
INCOMPLETE_RUN
```

rather than flattening them.

---

# 20. Suggested acceptance tests to add

## 20.1 Structural exact/open Record tests

If explicit openness is ratified:

```phalcom
let exact: { id: Int } = { id: 1 }                       // pass
let tooWide: { id: Int } = { id: 1, name: "a" }         // reject

let open: { id: Int, ... } = { id: 1, name: "a" }       // pass
let badDepth: { id: Number, ... } = { id: "x" }          // reject
```

## 20.2 Optional member tests

```text
required -> optional: pass
absent optional: pass
wrong present type: reject
extra unknown field under closed target: policy-specific reject
```

## 20.3 Tuple remainder tests

Test positional, labeled, complete, and unconstrained remainders independently.

## 20.4 Iterator chain soundness

```phalcom
const ints = ...
const strings = ...

ints.chain(ints)       // typed
ints.chain(strings)    // semantic rejection
```

## 20.5 `toMap` no-skipping regression

Use at least four unique entries and verify all four keys and values.

## 20.6 Inferred-result fingerprint adversary

Two bodies with the same published result type but different proof origin.

## 20.7 Rest/module-export conformance evidence

Force a real specialized callable recipe across the forwarding boundary and execute a nested requirement inside the callee.

---

# 21. Implementation quality assessment

## 21.1 Strong aspects

The C3 implementation repeatedly shows good architectural instincts:

- canonical semantic authority instead of parallel registries;
- stable identity keys;
- source identity preserved as provenance rather than used as semantic spelling identity;
- rows remain separate from proper types;
- no runtime pack solver;
- no VM type-data evaluator;
- no reverse inference through non-injective projections;
- exact dependency recording;
- cold/incremental comparisons in difficult areas;
- compiler lowering generally consumes semantic evidence instead of re-solving.

Those decisions should be preserved.

## 21.2 Weak aspects

The main quality weaknesses are now less about "wrong data structures" and more about **completion discipline**:

1. late normative changes are entering the tree without implementation reconciliation;
2. broad aggregate tests are allowed to mask later fixtures;
3. historical documents remain current-looking after architectural supersession;
4. some old known soundness issues in iterator adapters remain adjacent to new C3 work;
5. correctness-first global rescans are landing without resource measurement;
6. a few dense mega-tests carry too much acceptance weight without complementary law/property tests.

---

# 22. Final verdict

C3 should **not** be marked complete after `5f872da0`.

The newly implemented inferred trait-default result architecture is directionally correct and T3/G2 is substantially convincing. The previous projection solver, complete-pack canonicalization, and nested evidence-slot audit defects were also repaired in credible ways.

But closure would currently overstate the system in four independent dimensions:

```text
semantic coherence:
    Tuple/Record spec and implementation disagree

feature completeness:
    R1 T4–T7 remain open

language correctness:
    iterator adapter contract defects and toMap bug remain

verification:
    G5/G6 and broad regression floors are not green
```

The correct next step is **not** to run a giant workspace suite immediately.

The correct order is:

```text
1. resolve the Tuple/Record subtype authority conflict;
2. fix the deterministic iterator correctness defects;
3. complete R1 T4;
4. complete T5 recursion / Iterator.take attribution;
5. restore the natural chain;
6. run focused structural/type-theory law tests;
7. then perform the bounded/broad certification gates.
```

Only after those steps would a C3 closure review have a stable semantic object to certify.

---

# Appendix A — Key current source observations

## A.1 Record width behavior

Current semantic law effectively:

```text
for every field in target:
    source contains field
    source.field <: target.field

target tail closed:
    success
```

Extra source fields are therefore admitted.

## A.2 Tuple relation

Concrete tuple subtype comparison currently requires:

```text
same arity
same labels at each position
covariant payload subtype
```

There is no open remainder relation matching the late spec.

## A.3 `Iterable.toMap`

Current defect:

```phalcom
while let Some(entry) = it.next {
    ...
    step = it.next // extra advance
}
```

## A.4 `ChainIterator`

Current unsound contract:

```phalcom
impl<I, J> Iterator for ChainIterator<I, J> {
    type Item = <I as Iterator>::Item
}
```

without a `J::Item == I::Item` guarantee.

## A.5 Adapter erasure

Current source includes:

```phalcom
FlatMapIterator::Item = Dynamic
FlattenIterator::Item = Dynamic
StrictZipIterator::Item = Dynamic
IndexedIterator::Item = Dynamic
```

## A.6 Callable signature fingerprint

Current product fingerprint includes the `inferred_return` `TypeKnowledge`, whose known form includes:

```text
TypeId
EvidenceStatus
EvidenceOrigin
```

This is the main T4 same-result fingerprint risk.

---

# Appendix B — Closure checklist

A future C3 closure review should not begin until all boxes below are either checked or explicitly removed by an accepted architecture amendment.

### Structural typing

- [ ] One authoritative Record/Tuple width/open law.
- [ ] Spec, AST, semantic model, relation engine, tests agree.
- [ ] Optional members implemented or removed from current normative spec.
- [ ] Explicit remainder forms implemented or removed from current normative spec.
- [ ] Algebraic subtype law tests pass.

### Packs

- [x] Projection residual obligations prevent false `Solved`.
- [x] Complete-pack source-backed reassembly law.
- [ ] Pack subtype/equality API intent clarified; unreachable relation logic resolved.
- [ ] Property suite for projection/concat/canonicalization.

### Tuple/Record

- [x] Canonical structural behavior owner.
- [x] Tuple `Element` exact concrete behavior.
- [x] Record `Value` closed/open projection behavior.
- [x] Tuple iterator specialization.
- [ ] Structural subtype law reconciled.
- [ ] New optional/remainder spec reconciled.

### Traits/conformance

- [x] Guarded Derived dispatch through canonical proof authority.
- [x] Lowering consumes selected evidence.
- [x] Nested evidence runtime slot mapping repaired.
- [x] Canonical inferred trait-default result publication.
- [x] Conformance plan refresh/replacement and focused client revalidation.
- [ ] Rest/module-export evidence transport proven by executable specialized recipes.

### R1

- [x] T1.
- [x] T2/G1.
- [x] T3/G2.
- [ ] T4/G3 CV-05–CV-07.
- [ ] T5/G4 recursion and `Iterator.take`.
- [ ] T6/G5 natural chain.
- [ ] T7/final scoped spec + collection selector.

### Iterator/library

- [ ] `ChainIterator` item soundness.
- [ ] `toMap` no double-advance.
- [ ] Adapter Dynamic boundaries justified or typed.
- [ ] Direct dedicated collection regressions not masked by aggregate ordering.

### Certification

- [ ] Required G5 aggregate(s) green or formally accepted under checkpoint policy.
- [ ] Workspace test run completes with captured exit.
- [ ] Clippy disposition.
- [ ] Format disposition.
- [ ] Diff-check disposition.
- [ ] Known current semantic regressions explicitly carried into closure record.
- [ ] Final walkthrough/handoff updated to current R1 architecture.
- [ ] No stale Outcome-B language presented as current rule.
