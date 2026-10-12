---
id: COLL006.C1.P4-implementation-requirements-analysis
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: requirements-analysis
title: "P4 Static Homogeneous Sequences and Atomic Bracket-Default Cutover — Implementation Requirements"
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-10-08
prepared_against_branch: feat/telos-collection-upgrade
prepared_against_revision: 298084a44968402fc47d826ee9f99df1c9205e6a
main_revision_observed: c2dd635ee34f2cef3e64e86c87a5e77992f29fa7
depends_on:
  - COLL006.C1.P2
  - COLL006.C1.P3
---

# COLL006.C1.P4 — Static Homogeneous Sequences and Atomic Bracket-Default Cutover

## Implementation Requirements Analysis

> **Relationship to the prior audit**
>
> This document continues and deepens `COLL006.C1-P3-audit-and-P4-requirements-analysis.md`.
> The earlier report remains the P3 code-review/audit record. This document is the
> implementation-grade requirements package for the work that must happen before
> and during P4.
>
> It does **not** authorize P4 implementation while P3 is still incomplete.

---

# 0. Executive conclusion

P4 is architecturally ready to be specified, but it is **not ready to be implemented**.

The live `feat/telos-collection-upgrade` branch remains at:

```text
298084a44968402fc47d826ee9f99df1c9205e6a
feat(COLL006): stage sequence products by inference owner
```

Current `main` is:

```text
c2dd635ee34f2cef3e64e86c87a5e77992f29fa7
```

The feature branch remains:

```text
5 commits ahead
14 commits behind
merge base 61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1
```

P3 still has blocking semantic correctness work:

```text
canonical negative-Vector-extent rejection
inference-owned sequence publication integration
Array<T> public homogeneous API typing
mutable-descendant Vector hash/key stability policy
remaining semantic/runtime/durability evidence
lifecycle reconciliation
```

P4 must begin only after those are accepted and the feature line is converged with
the current integration target.

Once the entry gates are satisfied, P4 should be implemented as **one bounded
extension of existing C4 static-value/static-constant machinery plus one atomic
source-language default migration**.

The central architecture is:

```text
checked neutral bracket literal
        │
        ▼
SequenceLiteralResolution
        │
        ├──────── runtime codegen path ────────► Array / Vector runtime construction
        │
        └──────── static evaluator path
                         │
                         ▼
                 StaticValueData::Sequence
                         │
                         ▼
                MaterializationDescriptor
                         │
                         ▼
           canonical owner materialization cache
                         │
             ┌───────────┴────────────┐
             ▼                        ▼
      immutable VectorObject   frozen ArrayObject
```

The implementation must **not** introduce:

```text
ConstArray
ConstVector
static List semantics by accident
a collection-specific static evaluator
a collection-specific static-value store
runtime re-execution of initializer AST
phase-based overload preference
Pack-as-sequence encoding
a second nominal type identity for Array/Vector
```

A particularly important refinement from the deeper repository pass is that a
new static sequence node is not complete merely when it can be interned and
evaluated. It must participate in **all canonical C4 static-value operations**:

```text
static_value_type
semantic_eq
semantic_hash
content_fingerprint
materialization descriptor projection
```

Otherwise static sequences would be only partially first-class and would break
incremental invalidation, constant equality/hash semantics, or type projection.

A second refinement is that source-project import/re-export identity is already
implemented through:

```text
StaticConstantId
    ↓
StaticConstantLoweringSpec
    ↓
RuntimeStaticConstantRef
    ↓
canonical owner ModuleObject.static_constants
```

P4 should extend this path. It does **not** need a new import representation,
runtime global slot, or collection-specific load opcode.

---

# 1. Authority and scope

## 1.1 C1 authority

The following C1 rulings are fixed:

```text
List<T>
    homogeneous
    runtime extent
    dynamically resizable
    explicitly selected after cutover

Array<T>
    homogeneous
    runtime extent
    fixed after construction
    ordinary runtime instances permit element replacement

Vector<T, N>
    homogeneous
    exact static extent N
    immutable
```

Literal selection after P4:

```text
<T>[...]          -> Array<T>
<T, N>[...]       -> Vector<T, N>

expected List     -> List
expected Array    -> Array
expected Vector   -> Vector

otherwise:
    runtime phase -> Array
    static phase  -> Vector
```

Phase defaulting is not an overload tie-breaker.

## 1.2 C4 authority

C4 remains the owner of:

```text
StaticValueStore
StaticValueRef / StaticValueData
StaticEvaluator
StaticEvalOutcome
StaticConstantId
static constant query/dependency/cycle semantics
static-value semantic equality/hash
static-value content fingerprinting
materialization descriptors
canonical source-owner runtime materialization
query budgets and cancellation
```

P4 may extend these authorities with sequence-specific cases.

P4 may not replace them.

## 1.3 Historical collection analysis that is not P4 authority

The repository contains:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/collections/
    telos-collections-requirements-analysis.md
```

That record contains an older collection proposal using concepts such as:

```text
Array<T, N>
Vector<T>
```

Those shapes conflict with the now-ratified COLL006.C1 architecture:

```text
Array<T>
Vector<T, N>
```

That older document may be used for repository archaeology and generic C4
integration observations only.

It must **not** be used as semantic authority for P4.

## 1.4 P4 mandatory vertical

P4's required static collection vertical is:

```text
checked bracket literal
    -> static Array or static Vector value
    -> canonical materialization
```

P4 does **not** automatically make arbitrary runtime constructors static-callable.

For example, P4 must not assume that:

```telos
const x = Array.new(...)
const y = Vector.new(...)
```

are legal static calls merely because bracket construction is implemented.

Arbitrary constructor/static-method evaluation is legal only if the existing C4
static checker already authorizes the exact call and its semantics are independently
defined.

---

# 2. P4 entry gates

# 2.1 G0 — P3 semantic acceptance

P4 source implementation must not begin until all of the following are true.

## Vector formation

- concrete negative `N` is rejected at canonical formation;
- partial/symbolic `N` remains legal;
- substitution cannot manufacture a closed negative Vector;
- scoped application cannot bypass validation;
- aliases cannot bypass validation;
- inference materialization cannot bypass validation;
- imported metadata cannot publish an invalid closed Vector.

## Sequence product lifetime

- locally valid sequence facts are not committed while required inference owners remain unresolved;
- declaration inference ownership is integrated;
- explicit-head inference ownership is integrated;
- ordinary generic-call ownership is integrated;
- deferred/borrowed child ownership is integrated;
- union shared ownership is integrated;
- static expected-type ownership is integrated;
- flow probes cannot mutate parent publication state;
- unresolved/cancelled/budget/internal-failure owners publish no executable sequence capability;
- final callable/static products contain committed sequence entries only.

## Array typing

- public reads expose homogeneous `T`/`Option<T>` contracts as appropriate;
- public replacements accept `T`;
- wrong-type replacement is rejected semantically;
- existing bounds behavior is unchanged;
- native metadata is regenerated.

## Runtime correctness

- Vector operand evaluation is exactly once and source ordered;
- construction failure unwinds private builders safely;
- GC while constructing a Vector is safe;
- missing symbolic proof fails closed;
- inconsistent symbolic proof fails closed;
- zero extent works;
- large exact semantic extent does not truncate;
- mutable-descendant key/hash policy is explicit and tested.

## Durability

- Vector generic parameter metadata distinguishes `T: Type` and `N: Int`;
- reflection shows the static parameter domain;
- import/re-export preserves canonical Vector identity;
- cold/incremental analysis agrees when N changes;
- cold/incremental analysis agrees when spread exactness changes.

# 2.2 G1 — Branch convergence

Before the final P4 migration census:

1. converge `feat/telos-collection-upgrade` with the current intended integration base;
2. resolve overlapping parser/checker/Universe/native-surface changes intentionally;
3. rerun P3 acceptance gates;
4. update P4's implementation anchor.

Do not freeze a repository-wide migration inventory against a branch known to be
14 commits behind the integration line.

# 2.3 G2 — C4 seam revalidation

Immediately before P4 source edits, re-read:

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/materialization.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/static_constant_demand.rs
phalcom-semantic/src/db/fingerprint.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/modules/materialize.rs
phalcom-core/src/compiler/lib/expr.rs
```

If another C4 plan has already generalized structural materialization, P4 must
reuse the landed architecture.

# 2.4 G3 — Decision amendment path confirmed

Current accepted literal decisions still define bare brackets in terms of List.

Before final default cutover, establish the repository's accepted mechanism for
amending/superseding:

```text
TDR-0025
TDR-0028
```

The decision update can be prepared in parallel with implementation.

The final semantic flip cannot be accepted while normative docs still contradict
the executable language.

---

# 3. P3 corrective implementation requirements inherited by P4

This section sharpens the P3 changes that P4 depends upon.

# 3.1 Centralized negative-extent validation

The live canonical generic application boundary is:

```rust
TypeStore::apply_static_form_with_arena
```

It already:

```text
kind-checks arguments
beta-reduces lambdas
flattens partial Applied forms
builds the final argument vector
interns TypeData::Applied
```

That makes it the correct bounded location for the canonical Vector application
well-formedness rule.

The validation must occur **after applied-form flattening** and **before interning
the final Applied node**.

Conceptually:

```rust
let (final_origin, final_args) = flatten_application(...);

validate_nominal_application(
    final_origin,
    &final_args,
    residual_kind,
)?;

Ok(intern(Applied { ... }))
```

Required Vector check:

```rust
fn validate_vector_application(
    store: &TypeStore,
    origin: TypeId,
    args: &[StaticArgument],
) -> Result<(), TypeApplicationError> {
    if store.nominal_origin_declaration(origin)
        != Some(&universe_declaration(UniverseKey::Vector))
    {
        return Ok(());
    }

    // Partial application remains legal.
    if args.len() < 2 {
        return Ok(());
    }

    let StaticArgument::Term(n) = args[1] else {
        return Ok(()); // ordinary kind checking owns wrong-domain rejection
    };

    if let TypeData::ConstInt(value) = store.get(n) {
        if value.is_negative() {
            return Err(TypeApplicationError::InvalidStaticArgument { ... });
        }
    }

    Ok(())
}
```

The exact error variant is flexible, but `MalformedLambda` is not an adequate
public formation error for this case.

A dedicated formation error should preserve enough information to produce:

```text
Vector extent must be nonnegative; got -1
```

## Why this boundary is sufficient

The following existing paths already route through application formation:

```text
ordinary application
application extension after partial application
type-lambda beta reduction
type substitution
inference materialization
```

The implementation must verify rather than assume that metadata reconstruction
and every scoped form eventually hit the same boundary.

# 3.2 Sequence publication transaction

`checker/sequence_publication.rs` is currently a detached helper.

P3 must move from:

```text
literal locally valid
    -> ctx.sequence_literals.insert(...)
```

to:

```text
literal locally valid
    -> stage candidate with actual owner dependencies
    -> owner settlement
    -> commit or discard
```

Required `CheckingContext` state:

```text
committed sequence index
pending sequence publication transaction
```

The committed index remains the compiler-facing product.

The pending transaction remains query-local and non-durable.

Required finalization invariant:

```text
CheckedStaticExpression.sequence_literals
CheckedStaticBody.sequence_literals
CallableAnalysis.sequence_literals
ModuleLoweringSemantics.sequence_literals
```

contain committed entries only.

# 3.3 Array public typing

The raw native floor may internally move `Value`, but source-owned signatures
must describe the homogeneous family.

Required semantic result shape:

```text
Array<T>.strict subscript read -> T
Array<T>.safe read             -> Option<T> or established equivalent
Array<T>.replacement value     -> T
```

The exact method names may remain as currently authored.

Do not use `Dynamic` merely because the VM representation is dynamically tagged.

# 3.4 Mutable-descendant Vector key stability

P3 must select one bounded policy before P4 makes Vector much more common.

A Vector is shallowly immutable, not necessarily deeply immutable.

If:

```text
Vector<Array<Int>, 1>
```

is admitted as a hash key and its Array child mutates, structural hash changes.

Therefore choose one of:

```text
A. recursively reject content-unstable descendants at key admission;
B. temporarily reject Vector as a Map/Set key;
C. implement an already-ratified stable-hashability contract.
```

Do not invent a broad Hashable trait solely to close P3.

If A cannot be implemented cleanly with current value categories, use B and
record the deferral.

---

# 4. P4 semantic decision register

# 4.1 Fixed

The following are fixed by C1 and require consultation to change:

```text
runtime default = Array
static default = Vector
List remains explicit resizable family
Array has runtime extent
Vector carries static N
Vector immutable
explicit const Array supported
const Array remains Array
no ConstArray classifier
no phase-default overload tie-break
semantic checker owns family selection
compiler consumes checked family/product
C4 owns static identity/evaluation/materialization
```

# 4.2 Newly clarified by this analysis

## Static sequence value is one shared representation

Array and Vector static values share static storage machinery but preserve
distinct result types.

## Static List is not silently introduced

C1 explicitly specifies:

```text
const Vector
explicit const Array
```

It does not define frozen List, const List, or a static resizable List model.

Therefore P4 must not infer a new semantic rule such as:

```text
const x: List<Int> = [...]
    -> frozen List
```

without an explicit ruling.

Recommended P4 behavior:

```text
static List literal requested explicitly/contextually
    -> deterministic unsupported-static-sequence diagnostic
       unless a separate ruling authorizes const List
```

This recommendation is derived from the current C1 scope; the source records do
not themselves specify a complete const-List law.

## Arbitrary static collection constructors are not implied

Bracket construction is mandatory.

Other constructors remain governed by existing static-call authorization.

# 4.3 Mechanically flexible

Implementers may choose:

```text
StaticSequenceId internal representation
Vec vs Box slice intern storage
private helper names
exact materialization descriptor enum names
whether nested sequence descriptors are recursive enum values or descriptor IDs
private GC rooting helper
exact test module decomposition
```

provided all invariants in this document hold.

---

# 5. Static sequence value model

# 5.1 Required representation

Add one canonical static-value category.

Conceptually:

```rust
structural_id!(StaticSequenceId);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StaticSequenceData {
    result_type: TypeId,
    elements: Box<[StaticValueRef]>,
}

pub enum StaticValueData {
    ...
    Sequence(StaticSequenceId),
}
```

`result_type` is mandatory.

Payload contents are insufficient to distinguish:

```text
Array<Int>[1,2]
Vector<Int,2>[1,2]
```

# 5.2 Required store fields

Conceptually:

```rust
sequences: Vec<StaticSequenceData>,
sequence_ids: HashMap<StaticSequenceData, StaticSequenceId>,
```

Cloning the `StaticValueStore` must preserve sequence indexes and the store
lineage exactly like existing structural categories.

# 5.3 Interning API

Recommended bounded API:

```rust
pub fn intern_sequence(
    &mut self,
    result_type: TypeId,
    elements: Vec<StaticValueRef>,
    types: &TypeStore,
) -> Result<StaticValueRef, StaticValueStoreError>
```

or an equivalent API with sufficient canonical type information.

Required checks:

1. all child refs belong to this `StaticValueStore`;
2. `result_type` exists in the supplied TypeStore;
3. `result_type` has proper kind `Type`;
4. nominal origin is canonical Array or canonical Vector;
5. if result is Vector, its static extent is a concrete nonnegative exact Int;
6. Vector extent equals `elements.len()` exactly;
7. no host integer truncation occurs while checking the cardinality.

Do not turn `usize` into Telos Int by unchecked cast.

# 5.4 Homogeneity validation boundary

The semantic checker is the authority for element assignability.

`intern_sequence` should not become a second full type checker.

Recommended division:

```text
checker:
    proves each element/spread item satisfies element contract

static evaluator:
    evaluates only checked operands

store:
    validates representation identity and cardinality invariants
```

If the store can cheaply validate child types using canonical semantic tables,
a defensive assertion is acceptable, but it must not duplicate generic
inference or overload resolution.

# 5.5 Accessors

Add canonical accessors such as:

```rust
pub fn get_sequence(
    &self,
    id: StaticSequenceId,
) -> Result<&StaticSequenceData, StaticValueStoreError>

pub fn sequence_elements(
    &self,
    id: StaticSequenceId,
) -> Result<&[StaticValueRef], StaticValueStoreError>
```

Do not expose mutable sequence backing.

---

# 6. Full first-class static-value integration

A `Sequence` arm must be added to every exhaustive static-value operation.

# 6.1 `static_value_type`

Required:

```text
Sequence -> retained result_type
```

after validating that the result type remains a proper, recognized Array/Vector
type.

Do not reconstruct type from child values and current phase.

# 6.2 `semantic_eq`

Required law:

```text
same result type
AND
same element count
AND
semantic_eq for every corresponding child
```

Therefore:

```text
Array<Int>[1,2] != Vector<Int,2>[1,2]
```

even when the children are equal.

# 6.3 `semantic_hash`

Required hash law:

```text
semantic_eq(a,b) => semantic_hash(a) == semantic_hash(b)
```

Hash the sequence category, result type/family identity, length, and recursively
semantic-hashed children.

Do not hash store-local sequence ID.

# 6.4 `content_fingerprint`

Static constant invalidation uses:

```rust
StaticValueStore::content_fingerprint
```

recursively.

Sequence fingerprint must include:

```text
sequence tag
result type identity sufficient for the current snapshot fingerprint model
length
child content fingerprints in order
```

This is what makes:

```text
const x = [1,2]
       ↓ edit
const x = [1,2,3]
```

invalidate downstream consumers.

# 6.5 `domain_of`

Do not pretend structural sequence values have a primitive value-domain answer
without semantic products.

Unless C4 has generalized this API by implementation time, Sequence should
follow the existing structural-value policy:

```text
StructuralValueNeedsSemanticProducts
```

while `static_value_type` remains the ordinary type projection.

# 6.6 Exhaustive matches

Audit every `StaticValueData` match in:

```text
semantic store
static evaluator
materialization
fingerprinting
tests
debug/presentation helpers
```

Use compiler exhaustiveness as a discovery aid, not as proof that semantic
coverage is complete.

---

# 7. Static evaluator integration

# 7.1 `EvaluatorBody` must expose sequence products

All evaluator body forms already retain sequence indexes:

```text
CheckedStaticExpression
CheckedStaticConstantInitializer
CheckedStaticBody
```

Add/reuse one accessor:

```rust
fn sequence_literals(
    &self
) -> &SequenceLiteralResolutionIndex
```

No evaluator path should inspect checker internals beyond the published product.

# 7.2 New bracket-literal evaluation arm

`StaticEvaluator::run` currently rejects unmatched expressions as
`UnsupportedExpression`.

Add an `Expr::BracketLiteral` arm that:

1. obtains the exact `ExpressionId`;
2. looks up `SequenceLiteralResolution`;
3. rejects missing resolution;
4. validates that the resolution family is statically supported;
5. schedules element evaluation in source order;
6. performs only the construction lane authorized by the product;
7. interns the resulting static sequence.

# 7.3 Work queue shape

A mechanically reasonable shape is:

```rust
enum Work<'a> {
    ...
    FinishStaticSequence {
        expression: ExpressionId,
        element_count: usize,
    },
    ...
}
```

Expanded sequences may require a richer work item.

The evaluator must preserve its existing non-recursive guest execution model.

# 7.4 Fixed construction

For:

```rust
SequenceConstructionPlan::Fixed
```

the evaluator:

```text
evaluates each child once
collects resulting StaticValueRef values
verifies count against checked plan
interns one sequence
```

Evaluation order must match lexical source order.

# 7.5 Exact spread construction

For:

```rust
SequenceConstructionPlan::Expanded
```

the evaluator must consume `exact_expansions`.

It may expand only semantic lanes that have a defined static-value representation.

Required first lanes:

```text
closed positional Tuple
static Vector sequence
```

If P3 publishes additional exact lanes by P4 start, revalidate them.

Do not invoke runtime `Iterable.iterator` from the static evaluator.

# 7.6 Vector identity spread

For:

```rust
VectorIdentitySpread
```

the evaluator may copy/reuse the source static Vector's logical child sequence
after verifying:

```text
source static value is Vector
source result extent matches proof
source child count matches proof
result result_type matches checked literal result
```

Static identity reuse of the exact same `StaticValueRef` is not required because
the source and destination source-level construction are distinct values, though
interning may canonicalize equal static values.

# 7.7 Static Array

For an explicit/contextual:

```telos
const a: Array<Int> = [...]
```

the static semantic value's `result_type` is `Array<Int>`.

The static value itself is immutable canonical compile-time data even though its
runtime materialization is a frozen ordinary Array object.

# 7.8 Static Vector

For:

```telos
const v = [...]
```

after the default cutover:

```text
family = Vector
N = exact checked count
result_type = Vector<T,N>
```

# 7.9 Static List boundary

P4 must not construct `StaticValueData::Sequence` with a List result unless an
explicit ruling adds static List semantics.

If semantic checking selects List in a static constant initializer, produce a
deterministic unsupported-static-collection result/diagnostic rather than
silently freezing or changing family.

# 7.10 Failure atomicity

StaticValueStore is append-only.

P4 does not need arena rollback for child values already interned before a
failure.

The required atomicity is:

```text
failed/cancelled/budgeted sequence evaluation
    -> no successful root StaticValueRef published as the constant result
    -> no materialization descriptor projected
    -> no runtime constant published
```

This matches existing C4 arena semantics.

---

# 8. Static evaluator proof discipline

The evaluator is an execution engine for checked static semantics, not a second
semantic analyzer.

Forbidden evaluator behavior:

```text
select Array/Vector from phase
infer T from runtime/static child values
recompute N from source syntax as semantic authority
resolve Iterable dynamically
retry a rejected spread through another lane
repair missing SequenceLiteralResolution
construct List as fallback
```

Required fail-closed behavior:

```text
missing checked product        -> internal/invalid static outcome
proof/storage disagreement     -> invalid/internal static outcome
unsupported static spread lane -> deterministic unsupported outcome
cancelled                      -> Cancelled
budget exhausted               -> BudgetExceeded
```

---

# 9. Materialization descriptor extension

# 9.1 Current boundary

Current descriptors support primitive materialization and classify structural
values as deferred.

P4 should extend that same enum.

# 9.2 Recommended representation

Conceptually:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SequenceMaterializationKind {
    Vector,
    FrozenArray,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaterializationDescriptor {
    ...
    Sequence {
        kind: SequenceMaterializationKind,
        elements: Arc<[MaterializationDescriptor]>,
    },
    DeferredStructural(DeferredStructuralKind),
    Unsupported(String),
}
```

Equivalent descriptor-ID graphs are mechanically acceptable.

# 9.3 Descriptor must be self-contained

Do not put these into a runtime-facing materialization descriptor:

```text
StaticValueRef
StaticValueStoreId
ExpressionId
InferenceContextId
query-local TypeId as runtime identity
AST node
checker pointer/reference
```

The descriptor is a self-contained lowering product.

# 9.4 Runtime family identification

The descriptor must encode enough to choose:

```text
VectorObject
frozen ArrayObject
```

Do not infer family at runtime from element count.

# 9.5 Nested sequences

P4 should support recursively nested homogeneous sequences if every leaf is
currently materializable.

Example:

```telos
const matrix: Vector<Vector<Int,2>,2> = [
    <Int,2>[1,2],
    <Int,2>[3,4],
]
```

may materialize recursively.

# 9.6 Deferred nested structural children

Suppose C4 can represent:

```text
Tuple
Record
Data
Variant
Pack
```

statically but still defers their runtime materialization.

Then:

```telos
const xs: Vector<RecordType, 2> = [...]
```

may be a valid **static semantic value** while runtime access remains
non-materializable.

P4 must not secretly implement all structural C4 materialization to make this
case work.

Recommended descriptor projection:

```text
if all children materializable:
    Sequence(...)
else:
    DeferredStructural(Sequence)
    or an equivalent explicit deferred result
```

The outer static constant remains semantically valid.

# 9.7 Capability taxonomy

If the current `MaterializationCapability` needs widening, prefer a generic
category such as:

```text
MaterializableStructural
```

rather than a phase-number-specific name.

Do not imply that Tuple/Record/Data/Variant/Pack have been generalized if only
Sequence is supported.

---

# 10. Runtime materialization

# 10.1 Canonical owner path

Reuse:

```text
StaticConstantLoweringSpec
        ↓
VM::ensure_static_constant_materialized
        ↓
canonical source owner ModuleObject.static_constants
```

This already guarantees:

```text
owner identity
import alias identity
re-export identity
one materialized runtime value per owner/name
```

# 10.2 No runtime initializer execution

The runtime materializer receives descriptors only.

It must never evaluate:

```text
BracketLiteral AST
initializer AST
static expression AST
type inference
sequence proof
```

# 10.3 Vector materialization

Materialize:

```rust
Object::Vector(VectorObject::new(elements.into_boxed_slice()))
```

or the current heap helper equivalent.

No List/Array staging allocation.

# 10.4 Frozen Array materialization

Prefer a construction API that cannot expose the object in a mutable state:

```rust
ArrayObject::from_vec_frozen(elements)
```

or:

```rust
let mut object = ArrayObject::from_vec(elements);
object.freeze();
publish(object);
```

where publication occurs only after freezing.

`set_frozen` remains runtime-internal.

# 10.5 Publication atomicity

`ensure_static_constant_materialized` currently inserts into the owner module
only after `materialize_static_constant_value` succeeds.

Preserve that property:

```text
child/materialization failure
    -> no owner static_constants insertion
```

# 10.6 GC rooting during recursive materialization

This is a mandatory implementation concern.

A recursive sequence descriptor may allocate heap-backed children before the
parent is allocated.

Every intermediate heap `Value` must remain GC-rooted across subsequent
allocations.

Acceptable approaches include:

```text
private rooted VM stack/local sequence
temporary GC-visible builder
existing compiler/runtime rooting helper
another already-proven VM root container
```

Unacceptable:

```rust
let mut values = Vec<Value>; // unrooted heap refs
for child in children {
    values.push(materialize(child)?); // later allocation can GC earlier values
}
```

unless the VM's allocation discipline explicitly proves those host-held Values
are roots.

Add forced-GC tests during nested sequence materialization.

# 10.7 Resource failure

Sequence allocation must fail through current controlled allocation/resource
policy.

Do not:

```text
truncate count
panic on reserve failure
wrap u64/usize
publish half-built constant
```

---

# 11. Compiler constant-read integration

# 11.1 Current primitive vertical

Compiler reads of primitive static constants can embed immediate/constant-pool
values directly.

Structural values currently produce:

```text
StaticConstStructuralDeferred
```

# 11.2 Sequence read requirement

A materializable sequence constant must use the canonical owner-held runtime
value.

Do **not** compile a sequence constant by independently allocating a fresh
Vector/Array in every referring chunk.

For same-module reads, a mechanically viable path is:

```text
find StaticConstantLoweringSpec
ensure_static_constant_materialized(spec)
resolve owner-held Value
embed/reference that canonical Value
```

provided GC/root ownership remains correct.

For imported/re-exported reads, existing `RuntimeLinkedRead::StaticConstant`
already carries the canonical owner reference.

# 11.3 Identity test

Required:

```telos
// provider
export const values = [1,2,3]

// multiple reads/imports/re-export
```

All runtime reads must resolve the same owner-held object identity where object
identity is observable internally, while public semantic behavior remains value
semantics where specified.

# 11.4 Compiler must not regain semantic authority

Compiler may consume:

```text
descriptor
canonical static-constant owner reference
checked sequence lowering product
```

It may not decide:

```text
this bracket must be Vector because it appears in const
```

That decision belongs to semantic analysis.

---

# 12. Const Array mutation semantics

# 12.1 Why ordinary binding immutability is insufficient

Existing `AssignmentToImmutable` protects:

```telos
let x = ...
x = newValue
```

It does not protect:

```telos
const a: Array<Int> = [...]
a[0] = 3
```

because the latter is a subscript-set invocation on the bound object, not a
rebinding of `a`.

# 12.2 Required two-layer safety

## Layer A — semantic direct-use diagnostic

When checker provenance proves that the mutation receiver is a direct static
constant whose runtime classifier is Array, diagnose the mutation.

## Layer B — runtime frozen guard

Any path that loses constant provenance still reaches:

```text
ArrayObject::set
    -> Frozen
    -> RuntimeError::FrozenMutation
```

This is the universal alias defense.

# 12.3 Const provenance

The direct semantic check must use stable binding identity/provenance.

Do not detect this by:

```text
variable spelling
AST search for `const`
compiler global_static_constants
```

Recommended implementation:

- retain binding phase (`Runtime` / `Static`) in checker-visible binding metadata; or
- retain a checker-local static-binding identity set populated from canonical declaration/module facts.

This should work for:

```text
local const
module const
imported const
renamed imported const
```

where the binding itself still directly denotes the static constant.

# 12.4 Mutation boundary

`synthesize_set_index_expr` already analyzes the receiver and obtains exact
binding-read identity on a direct variable receiver.

Before resolving/executing the setter, P4 can check:

```text
receiver directly denotes static const
AND
receiver type nominal origin == canonical Array
```

then issue a source diagnostic.

# 12.5 Diagnostic

Do not reuse:

```text
AssignmentToImmutable
```

because that message tells the user to use `let mut`, which would be incorrect.

Use a dedicated diagnostic such as:

```text
StaticConstantMutation
ConstValueMutation
```

with meaning:

```text
cannot mutate a materialized const Array
```

# 12.6 Named mutation method

If the public Array mutator:

```text
at(_, put:)
```

has a stable canonical callable identity available at semantic application time,
the same direct-const guard should cover it.

Do not invent a general mutation-effect system solely for P4.

Subscript assignment plus the frozen runtime guard is the minimum mandatory
safety vertical.

# 12.7 Aliases

Example:

```telos
const a: Array<Int> = [1,2]

let b = a
b[0] = 9
```

Whether checker provenance follows this alias deeply is not required for P4.

Runtime must reject it.

Likewise for Dynamic/reflection paths.

# 12.8 Hashability

A const Array does not become hashable merely because this materialized instance
is frozen.

Array remains a mutable/content-unstable collection family under the C1 key
policy.

---

# 13. Phase-sensitive default selection

# 13.1 Current P3 branch

Current behavior:

```rust
None => contextual_family.unwrap_or(SequenceLiteralFamily::List)
```

P4 changes only the no-explicit/no-context fallback.

# 13.2 Required helper

Conceptually:

```rust
fn phase_default(phase: EvaluationPhase) -> SequenceLiteralFamily {
    match phase {
        EvaluationPhase::Runtime => SequenceLiteralFamily::Array,
        EvaluationPhase::Static => SequenceLiteralFamily::Vector,
    }
}
```

Selection remains:

```text
explicit literal head
    ↓ if absent
unique contextual family
    ↓ if absent
phase default
```

# 13.3 Empty literals

Runtime:

```telos
let x = []
```

has Array family, but no element type evidence.

If no contextual type exists, preserve deterministic underconstraint/no-type
diagnostics.

Static:

```telos
const x = []
```

has exact Vector extent `0`, but element type remains underconstrained.

Do not infer `Never`, `Dynamic`, or an arbitrary type unless separately ratified.

With context:

```telos
const x: Vector<Int,_> = []
// Vector<Int,0>
```

is valid.

# 13.4 Explicit families override phase

```telos
let x: Vector<Int,3> = [1,2,3]
// Vector despite runtime phase

const a: Array<Int> = [1,2,3]
// Array despite static phase
```

# 13.5 Overload ambiguity

Given:

```telos
f(_ a: Array<Int>)
f(_ v: Vector<Int,3>)

f([1,2,3])
```

the call remains ambiguous.

The runtime default must not bias candidate selection toward Array.

Explicit forms disambiguate:

```telos
f(<_>[1,2,3])
f(<_,_>[1,2,3])
```

If the live call analyzer cannot preserve this distinction after branch
convergence, STOP AND CONSULT before P4 cutover.

---

# 14. Static List boundary

The repository establishes:

```text
List = runtime-resizable family
P4 static support = Vector + explicit Array
```

It does not establish frozen List.

Therefore add an explicit P4 gate test for:

```telos
const xs: List<Int> = [1,2,3]
```

Recommended expected result:

```text
semantic/static diagnostic:
List is a dynamically resizable runtime collection and is not a supported const
sequence; use Vector for immutable static default or Array for explicit frozen
materialization
```

If the user chooses instead to support const List, stop and amend P4 scope
because that requires:

```text
frozen List semantics or copying semantics
runtime alias mutation policy
hashability policy
constructor/method policy
new migration/documentation rules
```

It is not a mechanical extension of current P4.

---

# 15. Migration census

# 15.1 A durable census artifact is required

Create:

```text
docs/implementation/COLL006/COLL006.C1/
    COLL006.C1.P4-bracket-migration-census.md
```

or the repository-conventional equivalent.

Do not keep the census only in shell output or agent notes.

# 15.2 Discovery method

Use AST/parser-based discovery for `.ph` source where possible.

Regex/search may supplement but not replace AST classification.

A literal `[` token is ambiguous with:

```text
subscript read
subscript write
selector declaration
type syntax
literal syntax
documentation text
```

# 15.3 Required classifications

Every first-party bare bracket literal must be classified as one of:

| Class | Meaning | Required action |
|---|---|---|
| `LIST_RESIZING` | append/remove/insert/size-changing semantics required | make List explicit |
| `LIST_IDENTITY` | code/tests require nominal List behavior | make List explicit |
| `FIXED_RUNTIME` | fixed sequence semantics acceptable | allow Array default |
| `STATIC_IMMUTABLE` | static exact immutable sequence intended | allow Vector default |
| `READ_ITERATE_ONLY` | only indexed reads/iteration; family likely flexible | review and accept/migrate deliberately |
| `LEGACY_TEST` | intentionally asserts old bare-List rule | rewrite/retire |
| `AMBIGUOUS` | cannot classify mechanically | manual decision required |

# 15.4 Scope

Inventory at least:

```text
phalcom-core/core/universe/**/*.ph
tests/**/*.ph
examples/**/*.ph
packages/**/*.ph
benchmarks source fixtures
language corpus
semantic fixtures
compiler/runtime fixtures
LSP/parser fixtures containing source programs
current specs/examples
accepted decision examples affected by meaning
```

Generated Rust/native metadata should be regenerated from authored sources.

# 15.5 Mutation does not automatically imply List

This is important.

Code that does:

```telos
xs[0] = value
```

may be compatible with Array.

Only structural size change or explicit nominal List contracts prove List is
required.

# 15.6 Migration before cutover

All `LIST_RESIZING` and `LIST_IDENTITY` sites must become explicit before the
phase default flips.

---

# 16. Decision/spec amendment

Before P4 acceptance, amend/supersede the legacy List-literal decisions.

Normative target text should state:

```text
[...] is a contextual homogeneous sequence literal.

Explicit head:
    <T>[...]     -> Array<T>
    <T,N>[...]   -> Vector<T,N>

Unique contextual expectation:
    List / Array / Vector -> expected family

No unique family:
    runtime phase -> Array
    static phase  -> Vector

Phase default never resolves overload ambiguity.
```

Also update current syntax and collection taxonomy documentation.

Historical docs may remain for archaeology if clearly marked superseded.

---

# 17. Cross-module source-project durability

# 17.1 Existing path

Source-project static constants already project through canonical owner identity.

P4 must prove sequence descriptors work with:

```text
selective import
renamed import
qualified module access
re-export
multiple consumers
```

# 17.2 Required scenario

Provider:

```telos
export const vector = [1, 2, 3]
export const array: Array<Int> = [4, 5, 6]
```

Consumer A:

```telos
import provider { vector, array }
```

Consumer B:

```telos
import provider { vector as values }
export values
```

Required facts:

```text
vector remains Vector<Int,3>
array remains Array<Int>
array runtime object is frozen
provider initializer is not re-executed at runtime
all references resolve canonical provider StaticConstantId/RuntimeStaticConstantRef
materialized heap-backed value survives GC
```

# 17.3 Artifact-only durability is not silently claimed

General cross-artifact serialized static-value/materialization support belongs to
the broader C4 durability program unless already landed.

P4 must prove the current source-project/module pipeline it actually uses.

Do not claim full C4.P8 artifact durability merely because source modules import
each other successfully.

---

# 18. Incremental correctness

# 18.1 Static value fingerprints

Because static constant product fingerprinting recursively calls
`content_fingerprint`, adding Sequence there is mandatory.

# 18.2 Edit matrix

Minimum P4 incremental matrix:

| Before | Edit | After | Required effect |
|---|---|---|---|
| `const x=[1,2]` | add element | `const x=[1,2,3]` | Vector N/value/fingerprint changes |
| `const x=[1,2]` | change element | `const x=[1,9]` | value/fingerprint changes, type may stay |
| `let x=[1,2]` | add element | runtime Array | family stays Array; runtime product changes |
| `let x: List<_>=[1]` | remove annotation | `let x=[1]` | List -> Array |
| `const x: Array<_>=[1]` | remove annotation | `const x=[1]` | Array -> Vector |
| runtime `let` | change declaration to `const` | same literal | Array -> Vector phase default |
| exact Vector spread | source becomes Array/List | invalid static cardinality | stale exact proof removed |
| valid static sequence | child becomes unsupported | materialization capability changes | stale descriptor removed |
| invalid | repair | valid | no stale diagnostic/product |
| valid | break | invalid | no LKG executable product published as current |

# 18.3 Comparison method

Cold/incremental parity compares semantic meaning:

```text
formatted canonical result type
family
extent
static value content fingerprint
materialization capability
diagnostics
construction plan
dependency set
```

Do not compare unrelated raw store-local IDs.

---

# 19. Diagnostics requirements

P4 needs precise diagnostics for new semantic boundaries.

Recommended additions/reuse:

| Condition | Required diagnostic meaning |
|---|---|
| direct const Array mutation | cannot mutate const Array |
| unsupported static List | List has no P4 const/static sequence semantics |
| missing checked static sequence product | internal semantic invariant, not user fallback |
| unsupported nested runtime materialization | static value valid but runtime materialization unavailable |
| overload family ambiguity | existing ambiguity mechanism; no phase tie-break |
| empty static Vector underconstrained T | inference/no-type evidence |
| Vector extent mismatch | inherited P3 generic inference conflict |
| invalid negative N | inherited P3 application formation diagnostic |

Do not create P4-specific duplicate codes where an existing semantic diagnostic
already says the correct thing.

---

# 20. P4 task decomposition

The implementation plan should use the following dependency structure or a
semantically equivalent one.

```text
T0  preflight + convergence + P3 gate
 │
 ├───────────────┐
 ▼               ▼
T1 static store  T7 migration census
 │               │
 ▼               ▼
T2 semantics     T8 source migration
 │               │
 ▼               ▼
T3 evaluator     T9 decision/spec amendment
 │
 ▼
T4 descriptor
 │
 ▼
T5 runtime materialization
 │
 ▼
T6 const mutation + source-project durability
 │               │
 └───────┬───────┘
         ▼
T10 atomic default flip
         │
         ▼
T11 hostile ambiguity/incremental/GC certification
         │
         ▼
T12 lifecycle reconciliation + P5 handoff
```

No task after T10 may be used to retroactively justify an unsafe cutover.

---

# 21. T0 — Preflight, convergence, and acceptance baseline

## Goal

Establish a trustworthy implementation base.

## Required work

1. record exact feature branch revision;
2. converge with current intended base;
3. run P3 closure gates;
4. verify C4 static-value/materialization APIs;
5. verify accepted decision supersession mechanism;
6. verify current parser/AST still has neutral `BracketLiteralExpr`;
7. verify current EvaluationPhase owner;
8. verify current module static-constant materialization path;
9. verify current P4 migration surface.

## Exit

A durable P4 plan/checkpoint entry records:

```text
anchor revision
P3 accepted revision
main/base convergence revision
known baseline failures
exact focused gate names/results
```

---

# 22. T1 — Static sequence storage

## Modify

Likely:

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_value/mod.rs
focused static-value tests
```

## Implement

```text
StaticSequenceId
StaticSequenceData
StaticValueData::Sequence
store vectors/interner
intern_sequence
sequence accessors
foreign-child rejection
Array/Vector result validation
Vector cardinality validation
```

## Required law tests

```text
same Array result + same children interns identically
same Vector result + same children interns identically
Array vs Vector do not intern as same value
Vector<Int,2> with 3 children rejected
Vector<Int,3> with 3 children accepted
foreign child rejected
zero-length Vector<Int,0> accepted
large semantic N does not truncate while validation fails/succeeds predictably
```

---

# 23. T2 — Static value semantic operations

## Modify

All static-value exhaustive operations.

## Implement/test

```text
static_value_type
semantic_eq
semantic_hash
content_fingerprint
domain_of
debug/inspection where applicable
```

## Required laws

```text
semantic equality is reflexive for valid sequence values
equal sequences have equal semantic hash
different element order differs
different family/result type differs
fingerprint changes when any child changes
fingerprint changes when result family/type changes
fingerprint is independent of store-local SequenceId allocation order
```

---

# 24. T3 — Static evaluator sequence execution

## Modify

Likely:

```text
phalcom-semantic/src/static_eval/machine.rs
checked body accessor(s)
static evaluator tests
static constant tests
```

## Implement

```text
EvaluatorBody::sequence_literals
Expr::BracketLiteral arm
fixed sequence work item
exact expansion execution
VectorIdentitySpread execution
static Array/Vector family validation
static List rejection/consult boundary
```

## Required behavioral tests

```text
const bare [1,2,3] -> static Vector after cutover test mode
const explicit Array -> static Array
nested static sequence
empty typed Vector
operand order
operand once
cancel during sequence
budget during sequence
missing sequence resolution fails closed
tampered cardinality proof fails closed
unsupported spread lane fails closed
```

For implementation staging before T10, explicit/contextual Vector/Array forms can
exercise T3 while global bare default remains List.

---

# 25. T4 — Materialization descriptor

## Modify

```text
phalcom-semantic/src/materialization.rs
materialization tests
```

## Implement

```text
Sequence descriptor
Vector/FrozenArray kind
recursive child descriptor projection
materializable/deferred capability classification
```

## Required tests

```text
primitive Vector descriptor
primitive frozen Array descriptor
nested sequence descriptor
Tuple child remains deferred if Tuple materialization still deferred
Record/Data/Variant/Pack child remains deferred unless canonical C4 support landed
descriptor contains no StaticValueRef/VM identity
```

---

# 26. T5 — VM/runtime materialization

## Modify

Likely:

```text
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/heap/array.rs
heap allocation helper if needed
core static constant tests
GC tests
```

## Implement

```text
recursive descriptor materialization
Vector allocation
frozen Array allocation
rooting of intermediate child Values
canonical owner cache reuse
no partial publication
```

## Required tests

```text
const Vector executes and reads at runtime
const Array executes and reads
const Array reports Array classifier
Vector reports Vector classifier
repeated read reuses value
forced GC before/after read
forced GC during nested sequence materialization
materialization failure leaves owner cache empty
no initializer bytecode emitted/executed
```

---

# 27. T6 — Direct const mutation and module durability

## Semantic mutation work

Add stable const receiver provenance and direct Array mutation diagnostic.

Required tests:

```telos
const a: Array<Int> = <Int>[1,2]
a[0] = 3
```

semantic error.

Alias:

```telos
let b = a
b[0] = 3
```

must fail at runtime even if semantic provenance is intentionally not propagated.

## Module work

Test:

```text
selective import
renamed import
qualified import
re-export
same owner identity
GC
```

No new module runtime slot for the const value.

---

# 28. T7 — Repository bracket census

No semantic cutover yet.

Produce the durable migration census.

Run source classification tooling against the **converged** tree.

Manual-review all ambiguous sites.

Exit requires zero unclassified first-party bare bracket literals in scoped
source roots.

---

# 29. T8 — Source migration

Convert every `LIST_RESIZING` / `LIST_IDENTITY` site to explicit List.

Preferred syntax should use existing supported contextual or constructor forms,
not invent migration-only syntax.

Run affected package/tests after each coherent migration slice.

Do not flip defaults yet.

---

# 30. T9 — Decision/spec amendment

Land or prepare the accepted semantic amendment according to repository policy.

Update:

```text
literal decision
collection taxonomy
syntax documentation
List guidance
Array guidance
Vector guidance
const Array guidance
```

No executable semantic cutover is considered accepted while current normative
text says bare brackets intrinsically mean List.

---

# 31. T10 — Atomic default flip

This should be a small semantic diff because infrastructure and migration are
already complete.

Change:

```text
no explicit head
no unique contextual family
```

from:

```text
List
```

to:

```text
Runtime -> Array
Static  -> Vector
```

No parser change should be necessary for family selection.

No compiler inference change should be necessary.

If either parser or compiler needs to reproduce family selection, STOP.

---

# 32. T11 — Certification

Run the hostile matrix.

## Literal matrix

```text
runtime bare
static bare
explicit Array
explicit Vector
contextual List
contextual Array
contextual Vector
empty
single
large fixed
spread
nested
```

## Ambiguity matrix

```text
Array vs Vector overload
List vs Array overload
List vs Vector overload
three-family overload
explicit head disambiguation
annotation disambiguation
```

## Const matrix

```text
Vector primitive
Vector nested
Array primitive
Array nested
direct mutation
alias mutation
import/re-export
unsupported nested structural child
```

## Incremental matrix

Use §18.

## GC matrix

```text
materialized Vector child graph
materialized frozen Array child graph
nested sequences
imported constant
constant cached only by module owner
```

---

# 33. T12 — Lifecycle closure

Reconcile:

```text
COLL006/PROGRAM.md
COLL006/STATUS.md
COLL006.C1-CHECKPOINT.md
P4 implementation plan
P4 walkthrough
P4 handoff
migration census
decision/spec amendment references
```

Record:

```text
final revision
exact gates/results
known baseline failures
approved deferrals
P5 entry assumptions
```

Only after this may P5 be declared the active C1 plan.

---

# 34. File/symbol impact map

| Concern | Primary current owner | P4 action |
|---|---|---|
| phase default | `checker/sequence_literal.rs::synthesize` | flip only at T10 |
| sequence checked product | `checker::analysis` | preserve shape unless required |
| publication lifecycle | `checker/sequence_publication.rs`, `CheckingContext` | must be closed in P3 |
| static sequence value | `static_value/store.rs` | add Sequence |
| static sequence equality/hash | `StaticValueStore` | exhaustive integration |
| static constant fingerprint | `db/fingerprint.rs` via `content_fingerprint` | covered by Sequence fingerprint |
| static evaluator | `static_eval/machine.rs` | consume checked product |
| descriptor | `materialization.rs` | supported sequence descriptor |
| lowering projection | `modules/semantic_lowering.rs::project_static_constants` | reuse |
| runtime const cache | `modules/static_constant.rs` | extend descriptor materialization |
| imports/re-exports | `modules/materialize.rs` | reuse canonical references |
| Array frozen storage | `heap/array.rs` | use/strengthen construction helper |
| Vector storage | `heap/vector.rs` | reuse |
| direct const mutation | `checker/expression.rs::synthesize_set_index_expr` + binding provenance | new guard |
| literal migration | repository `.ph` corpus | census + explicit List migration |
| normative decisions | TDR-0025 / TDR-0028 | amend/supersede |

---

# 35. Performance requirements

P4 is not the final collection performance phase, but it must avoid obvious
architectural regressions.

# 35.1 Static store

Target:

```text
sequence interning O(n) expected
semantic equality O(n) worst-case child traversal
semantic hash O(n)
content fingerprint O(n)
```

Do not accidentally perform full type inference per element during interning.

# 35.2 Static evaluator

Charge query budget proportional to:

```text
literal element count
expanded child count
recursive nested work
```

No hidden unbudgeted whole-sequence loops.

# 35.3 Runtime materialization

Target:

```text
one final dense allocation per sequence
one pass over children
no intermediate List
no re-run initializer
constant cache prevents repeat whole-sequence materialization
```

Nested descriptors may allocate nested children, but each node should be built
once.

# 35.4 Memory

Do not duplicate entire static sequence payload in multiple parallel
representations merely to materialize it.

Descriptor duplication is acceptable only where required by the existing
snapshot/lowering ownership model.

---

# 36. Security/correctness failure model

P4 must fail safely for:

```text
host allocation failure
query cancellation
query budget exhaustion
invalid/tampered sequence proof
descriptor mismatch
missing canonical module owner
GC during recursive materialization
invalid negative Vector type from malformed metadata
stale incremental product
unsupported structural child
```

No failure may:

```text
publish a mutable const Array
publish partially materialized owner constant
silently convert family
fall back to List
truncate exact N
execute initializer at runtime
```

---

# 37. Verification commands — requirements, not fabricated evidence

The exact commands must be selected from live test registration at implementation
time.

The P4 plan should establish focused selectors for at least:

```text
semantic static sequence store laws
semantic static evaluator sequences
semantic static constants/incremental
COLL006 sequence literal semantics
core static constant execution
core Array
core Vector
core module import/re-export parity
GC stress for static sequence materialization
native surface --check
workspace affected-crate checks
```

Do not write nonexistent exact test names into the walkthrough before the tests
exist.

Record actual selected counts and results.

---

# 38. STOP AND CONSULT triggers specific to P4

In addition to C1 global triggers, stop if:

1. P3 cannot produce committed-only sequence products before P4 evaluator work.
2. Static sequence support requires a second static store.
3. Static evaluator would need runtime method dispatch to implement bracket construction.
4. Static evaluator would need to re-infer collection family.
5. Runtime materialization would need access to AST/checker state.
6. Const Array safety would require a new public classifier.
7. Const Array safety would require copying at every alias boundary.
8. Supporting a nested sequence would require silently implementing all C4 structural materialization.
9. `const List` support becomes necessary for existing acceptance without an explicit semantic ruling.
10. Phase default must bias overload candidate resolution to make tests pass.
11. Migration census cannot distinguish a high-risk List-dependent site without semantic behavior choice.
12. Branch convergence changes the sequence-literal ownership architecture materially.
13. Materializing nested heap values cannot be rooted safely without changing GC ownership architecture.
14. Sequence content fingerprint cannot be made deterministic under current static-value identity rules.
15. Source-project imports can only work by duplicating runtime sequence objects instead of using canonical owner references.
16. Final decision amendment cannot be landed before executable cutover acceptance.

---

# 39. Acceptance checklist

P4 is complete only when every applicable item is checked.

## Entry

- [ ] P3 accepted.
- [ ] P3 publication lifecycle integrated.
- [ ] negative extent canonical formation closed.
- [ ] Array public typing closed.
- [ ] Vector hash/key policy closed.
- [ ] branch converged with current integration base.
- [ ] P3 gates rerun after convergence.

## Static value

- [ ] `StaticValueData::Sequence`.
- [ ] store/interner.
- [ ] foreign-child rejection.
- [ ] result-type validation.
- [ ] Vector extent/count validation.
- [ ] zero extent.
- [ ] `static_value_type`.
- [ ] `semantic_eq`.
- [ ] `semantic_hash`.
- [ ] `content_fingerprint`.
- [ ] deterministic clone/store behavior.

## Static evaluator

- [ ] checked product accessor.
- [ ] fixed bracket sequence execution.
- [ ] exact spread execution.
- [ ] Vector identity spread.
- [ ] source-order evaluation.
- [ ] exactly-once evaluation.
- [ ] cancellation.
- [ ] budget.
- [ ] missing-proof fail-closed behavior.
- [ ] static List boundary explicitly handled.

## Materialization

- [ ] self-contained sequence descriptor.
- [ ] Vector descriptor.
- [ ] frozen Array descriptor.
- [ ] nested sequence descriptor.
- [ ] unsupported nested structural child stays deferred.
- [ ] runtime Vector allocation.
- [ ] runtime frozen Array allocation.
- [ ] GC-safe intermediate rooting.
- [ ] no partial owner publication.
- [ ] materialized value cache reuse.
- [ ] no initializer AST/runtime execution.

## Const mutation

- [ ] direct const Array subscript mutation diagnosed.
- [ ] canonical named mutator guarded if feasible from existing callable identity.
- [ ] alias mutation rejected by runtime frozen guard.
- [ ] imported direct const retains diagnostic provenance where supported.
- [ ] const Array remains classifier Array.
- [ ] const Array not made hashable merely because frozen.

## Migration/cutover

- [ ] durable bracket census.
- [ ] zero unclassified scoped first-party literal sites.
- [ ] all List-resizing sites explicit.
- [ ] all List-identity sites explicit.
- [ ] legacy tests rewritten/retired.
- [ ] TDR-0025/TDR-0028 amended/superseded.
- [ ] runtime bare default = Array.
- [ ] static bare default = Vector.
- [ ] explicit List preserved.
- [ ] explicit Array preserved.
- [ ] explicit Vector preserved.
- [ ] contextual family precedence preserved.
- [ ] phase default not overload tie-break.

## Durability

- [ ] selective import.
- [ ] renamed import.
- [ ] qualified access.
- [ ] re-export.
- [ ] canonical owner value reused.
- [ ] GC survives owner-only rooting.
- [ ] cold/incremental family edit parity.
- [ ] cold/incremental extent edit parity.
- [ ] cold/incremental spread-proof edit parity.
- [ ] content fingerprint invalidates transitive static dependents.

## Closure

- [ ] focused gates recorded.
- [ ] generated native metadata current.
- [ ] scoped diff reviewed.
- [ ] no unrelated migration.
- [ ] checkpoint updated.
- [ ] PROGRAM updated.
- [ ] STATUS updated.
- [ ] P4 walkthrough complete.
- [ ] P4 handoff complete.
- [ ] P5 entry assumptions explicit.

---

# 40. Recommended implementation order

The most risk-reducing sequence is:

```text
P3 closure
    ↓
branch convergence
    ↓
T1 static sequence storage
    ↓
T2 static-value laws/fingerprint
    ↓
T3 evaluator
    ↓
T4 descriptor
    ↓
T5 runtime materialization
    ↓
T6 const mutation + module durability
    ↓
T7/T8 census and source migration
    ↓
T9 decision amendment
    ↓
T10 atomic default cutover
    ↓
T11 hostile/incremental/GC certification
    ↓
T12 lifecycle closure
```

Do not start with the default flip.

The default flip should be one of the final and smallest code changes in P4.

---

# 41. Final engineering judgment

P4 is not primarily a parser feature and not primarily a runtime collection
feature.

It is a **semantic-authority convergence phase**.

Its correctness depends on keeping four boundaries clean:

```text
1. checker owns collection-family and cardinality meaning;
2. C4 static evaluator owns compile-time execution;
3. StaticValueStore owns canonical compile-time value identity;
4. runtime materializer consumes self-contained checked descriptors only.
```

The source-language migration is safe only after those boundaries are stable.

The most dangerous implementation shortcuts would be:

```text
flip bare brackets first;
re-infer Vector/Array in compiler;
materialize static sequences by executing initializer AST at runtime;
encode sequence as Pack;
create ConstArray;
treat frozen Array as generally immutable/hashable;
silently add const List;
allow pending inference-owned sequence products into static evaluator input.
```

The implementation plan should be considered invalid if it relies on any of
those shortcuts.

P4 acceptance should leave the repository in a state where P5 can treat:

```text
List
Array
Vector
phase defaults
const Array/Vector materialization
```

as stable foundations rather than provisional behavior.
