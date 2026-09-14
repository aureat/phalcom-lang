# Implementer Handoff — LANG005.C1.P2 Representation-Aware Product Optimization

You are the implementing agent for **LANG005.C1.P2 — Representation-Aware Product Optimizer, Scalar Replacement, and Allocation Sinking** in the Phalcom repository.

You are implementing an already-decided architecture. You are **not** the language designer and you are **not** being asked to select among competing semantic designs. The language semantics, optimization boundaries, representation model, fallback behavior, and checkpoint decomposition have already been decided.

Your job is to:

1. inspect the current repository and the completed P1 implementation;
2. reconcile mechanical symbol/path drift;
3. implement P2 checkpoint-by-checkpoint;
4. preserve every semantic invariant listed here;
5. verify each checkpoint with hostile evidence before proceeding;
6. record implementation state and evidence;
7. stop and classify an incident rather than inventing a new semantic rule if the repository contradicts a governing assumption.

Do not opportunistically redesign Phalcom while implementing this work.

---

# 1. Governing implementation documents

The primary detailed implementation plan is:

```text
LANG005.C1.P2-representation-aware-product-optimizer-scalar-replacement-plan.md
```

Read it completely before modifying code.

Also read:

```text
PDR-0035
LANG005.C1.P1 plan
LANG005.C1.P1 diagnosis / completion guidance
LANG005.C1.P3 plan — for boundary awareness only, NOT implementation
docs/internals/data-representation/
```

The P2 plan is the implementation checklist.

This handoff is an **architectural amendment and implementer guidance document**. Where it deliberately corrects the P2 plan, this handoff takes precedence.

The most important correction is:

> **Do not identify optimizer bindings using `(SourceRange, name)` or by reconstructing lexical name resolution in the compiler. Use canonical semantic `BindingId` identity.**

This correction is explained in detail below.

---

# 2. First action: inspect the actual repository, do not trust the plan's historical repository snapshot

The P2 plan was originally written before P1 had landed. That historical repository-grounding section is now stale.

P1 has since introduced the real first-class `data` and product subsystem, including architecture equivalent to:

```text
DataComponentId
DataConstructorId

ProductLayout
ProductLayoutId
ProductLayoutSpec
ProductStorage

DataObject
RuntimeDataDescriptorId

DataDeclarationLoweringSpec
DataConstructionLoweringSpec

LoadDataSingleton
ConstructData
GetDataComponent

general enum payload ProductStorage
```

Do not reproduce the pre-P1 investigation from the P2 plan.

Instead, before changing anything:

```bash
git status --short
git rev-parse HEAD
git log --oneline -20
```

Preserve unrelated local changes.

Do not reset, clean, stash, rewrite, or discard existing work unless explicitly instructed by the user.

Then inspect the current final implementations of at least:

```text
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/checker/data_declaration.rs

phalcom-core/src/modules/semantic_lowering.rs

phalcom-core/src/product/
  mod.rs
  layout.rs
  registry.rs
  storage.rs

phalcom-core/src/heap/data.rs
phalcom-core/src/heap/adt.rs

phalcom-core/src/data.rs
phalcom-core/src/vm/data.rs
phalcom-core/src/vm/adt.rs

phalcom-core/src/compiler/lib/data_decl.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/state.rs
phalcom-core/src/compiler/lib/scope.rs

phalcom-core/src/bytecode.rs

docs/internals/data-representation/
```

Also inspect the final P1 regression tests.

Do not assume symbol names from the plan if the committed P1 implementation chose equivalent names.

Record the final mapping before P2 begins.

---

# 3. Hard prerequisite: P1 must be green

P2 is an optimization layer over P1.

P1 is the **canonical semantic and materialization implementation**.

P2 is not allowed to repair P1 by inventing another representation model.

Before P2 implementation, establish a baseline:

```text
P1 semantic tests
P1 data runtime tests
P1 product storage tests
P1 enum/ADT tests
P1 NativeOption tests
P1 GC tests
```

Run the focused P1 gates recorded by the landed implementation.

If current P1 fails independently of P2:

```text
classify as P1 CONTRACT / BASELINE incident
```

Do not bury the defect inside the optimizer.

A P2 optimization is correct precisely when:

```text
Optimized execution
    === observable semantics ===
P1 canonical execution
```

P1 with optimization disabled is the oracle.

---

# 4. Fundamental P2 architecture

Do not introduce a general SSA compiler.

Do not introduce a register VM.

Do not create another type checker.

Do not create another name resolver.

The architecture is:

```text
semantic analysis
        |
        v
P1 semantic/lowering products
        |
        | exact identities and construction facts
        v
P2 transient optimizer proof plan
        |
        +-------------------------+
        |                         |
        v                         v
canonical P1 emission       virtual product emission
        |                         |
        +------------+------------+
                     |
                     v
               ordinary bytecode
                     |
                     v
         existing superinstruction fusion
                     |
                     v
                     VM
```

P2 removes unnecessary materialization.

It does **not** redefine products.

---

# 5. The single most important architectural correction: use `BindingId`, not `(range, name)`

The attached plan contains an obsolete design:

```rust
pub struct ProductBindingSite {
    pub range: SourceRange,
    pub name: Box<str>,
}
```

and describes a compiler-local lexical environment that reconstructs which source name denotes which binding.

**Do not implement that design.**

Phalcom already has canonical semantic binding identity:

```rust
pub struct BindingId(pub u32);
```

and semantic analysis already resolves local variables, parameters, patterns, shadowing, and references.

The optimizer must consume that identity.

## 5.1 Correct authority split

Semantic analysis answers:

```text
Which binding is this?
```

P2 answers:

```text
How is this already-resolved binding used?
```

That distinction is absolute.

Use:

```text
semantic authority:
    occurrence -> BindingId

P2:
    BindingId -> ProductUseSummary
```

Do not use:

```text
name + lexical walk -> guessed binding
```

---

# 6. Recommended optimizer identity model

A `ProductFunctionPlan` is scoped to one analyzed body/callable.

Within that plan, use:

```rust
BindingId
```

as binding identity.

If an identity must escape that per-body scope, qualify it with the canonical body/callable owner:

```rust
struct ProductBindingKey {
    body: BodyId, // or current canonical callable/body identity
    binding: BindingId,
}
```

Do not invent a global integer.

Do not use `SourceRange` as semantic identity.

A range is useful for:

```text
diagnostics
bytecode span attribution
source-site lookup
```

but never for answering:

```text
which variable declaration does this expression denote?
```

A better optimizer shape is conceptually:

```rust
pub(crate) struct ProductFunctionPlan {
    pub binding_decisions:
        BTreeMap<BindingId, ProductOptimizationDecision>,

    pub use_sites:
        BTreeMap<LoweringSite, ProductUseKind>,

    pub ephemeral_projections:
        BTreeMap<LoweringSite, VirtualShapePlan>,
}
```

Use the actual existing backend source-site identity instead of `LoweringSite` if current P1 uses another type.

---

# 7. How to get BindingId into the backend correctly

Do not make the compiler read a variable name and re-run lexical resolution.

Inspect the current formal semantic products.

Phalcom already owns semantic attachment structures around:

```text
CallableAnalysis
BindingId
ExpressionId
SourceSiteId
formal_bindings
formal_expressions
SourceSemanticIndex
```

The exact current implementation may expose enough information directly.

If the compiler does not currently receive binding identity, extend the existing semantic-to-codegen projection boundary:

```text
SemanticSnapshot
        |
        v
ModuleLoweringSemantics
```

with the smallest backend-facing binding attachment necessary.

A valid structural design is:

```rust
pub struct BindingOccurrenceLowering {
    pub binding: BindingId,
}

pub struct CallableBindingLowering {
    pub declarations:
        BTreeMap<LoweringSite, BindingId>,

    pub reads:
        BTreeMap<LoweringSite, BindingId>,

    pub writes:
        BTreeMap<LoweringSite, BindingId>,
}
```

Do **not** copy that spelling blindly.

Use existing `ExpressionId`, `SourceSiteId`, `LoweringSite`, or equivalent where possible.

The invariant is:

> `ModuleLoweringSemantics` projects semantic identity. The optimizer consumes it. The compiler does not rediscover it.

The source index may be useful as a reference model for this projection, but the compiler should not become directly dependent on an editor-specific reverse index if `CallableAnalysis` can provide the same identity more directly.

---

# 8. Use analysis may walk AST structure — identity resolution may not

P2 still needs to determine things such as:

```text
this binding is captured
this occurrence is an assignment
this occurrence is a direct component projection
this occurrence is an opaque whole-value use
this occurrence is a match scrutinee
this use appears under a loop
```

That is legitimately backend structural analysis.

For example:

```text
semantic:
    expression E17 -> BindingId(4)

AST:
    E17 is receiver of a data-component GetProperty

P2:
    BindingId(4) receives ProductUseKind::DataProjection
```

This is correct.

This is not:

```text
AST:
    variable name == "p"

compiler lexical stack says nearest "p" is candidate X

therefore this use belongs to X
```

Delete that second-resolver idea entirely.

---

# 9. Source evaluation order is immutable

P1 had a real defect here during implementation and it was corrected.

Do not regress it.

These are three distinct orders:

```text
source evaluation order
logical component order
materialized ProductStorage order
```

Example:

```phalcom
data Pair(first: Int, second: Int)

Pair(
  second: side2(),
  first: side1(),
)
```

Correct P2 virtual construction:

```text
evaluate side2()
store result into logical component "second"

evaluate side1()
store result into logical component "first"
```

Not:

```text
evaluate first
evaluate second
```

The optimizer must consume P1's canonical argument-to-component mapping.

Never normalize evaluation order merely because storage has been normalized.

---

# 10. Constructor arguments must execute exactly once

Scalar replacement removes the aggregate.

It does **not** remove constructor semantics.

Given:

```phalcom
Point(
  x: effectA(),
  y: effectB(),
)
```

both expressions execute:

```text
once
in source order
with identical errors
with identical suspension behavior
```

even when only:

```phalcom
p.x
```

is later used.

Do not dead-code eliminate an unprojected product component.

Component expressions may:

```text
raise
mutate
allocate
send messages
await/suspend
capture
trigger GC
```

The optimization removes storage, not effects.

---

# 11. Unrestricted Phalcom `Int` is not a raw `i64`

This is a critical P1 lesson.

Phalcom `Int` includes arbitrary-precision heap-backed integers.

Do not infer:

```text
semantic type Int
    =>
virtual native i64 slot
```

P2 virtual leaves are always ordinary runtime:

```rust
Value
```

This is deliberate.

A virtual component containing:

```phalcom
9223372036854775808
```

must work exactly like canonical P1 execution.

P1's `ProductLayout` may use specialized primitive storage only when the representation layer has an actual proof that encoding is valid.

P2 does not reproduce P1 physical layout policy in frame slots.

---

# 12. Virtual leaves are `Value` stack roots

The runtime stack is already:

```text
GC-visible Value slots
```

Use that.

A virtual Point:

```phalcom
Point(x: 1, y: someObject)
```

becomes logically:

```text
slot H     -> Value(Int 1)
slot H + 1 -> Value(ObjRef ...)
```

not raw:

```text
slot H -> i64 bits
```

This preserves:

```text
GC
fiber parking
exception unwinding
upvalues
stack inspection
method ABI
```

Do not introduce untagged machine scalars in stack slots.

That is explicitly outside P2.

---

# 13. Persistent virtual binding storage

For an eligible immutable local with `N` leaves:

```text
source binding p
    |
    v
head slot H
leaf 1 at H+1
leaf 2 at H+2
...
```

The important invariant is:

```text
source binding identity != source binding physical slot
```

`BindingId` identifies the source variable.

`head_slot` identifies the current bytecode representation.

Keep that distinction.

A structurally appropriate compile-time record is:

```rust
struct ActiveVirtualProduct {
    binding: BindingId,
    head_slot: u16,
    leaf_count: u16,
    shape: VirtualShapePlan,
    materialization_spec: u16,
}
```

The exact fields may adapt to current code.

---

# 14. Hidden leaf locals must not become user-visible names

The plan suggests fresh names like:

```text
$product#17
```

That is only acceptable if it uses the existing compiler scratch-symbol mechanism and cannot participate in source lookup.

Prefer the compiler's current scratch reservation conventions.

Do not make hidden leaves ordinary user-resolvable lexical declarations.

Existing code already has scratch-symbol facilities around pack lowering. Reuse that machinery.

The correct semantics are:

```text
source binding table:
    p -> BindingId / head slot

compiler runtime slot metadata:
    head
    hidden leaf
    hidden leaf
```

A user must never be able to refer to one of the hidden leaves.

---

# 15. Persistent versus ephemeral slots

Treat these separately.

## Persistent virtual local

Lives for normal lexical lifetime:

```phalcom
const p = Point(x: a, y: b)

use(p.x)
use(p.y)
```

Its leaf slots remain live until `p` leaves scope.

Do not release them early.

## Ephemeral constructor projection

Example:

```phalcom
Point(x: a(), y: b()).x
```

Use a temporary contiguous scratch region:

```text
reserve
evaluate/store a
evaluate/store b
load requested leaf
release scratch while preserving result
```

Reuse existing:

```text
ReserveScratchLocal
ReleaseScratchLocal
reserve_pack_scratch
release_pack_scratch_from
```

semantics.

Do not invent a second stack-insertion mechanism.

---

# 16. Exception and non-local-control safety needs explicit hostile tests

A common implementation mistake is assuming:

```text
"ReleaseScratchLocal is emitted later, therefore scratch always disappears."
```

That is false when evaluation exits abnormally.

Test:

```text
scratch reserved
constructor argument raises
same-frame handler catches
execution continues
```

The handler/unwinder must restore a valid stack shape even though the later release bytecodes did not execute.

Also test:

```text
non-local block return
fiber suspension
yield/await
GC
nested scratch region
```

while virtual scratch slots exist.

Do not regard ordinary happy-path stack-height tests as sufficient.

---

# 17. Capture handling is deliberately conservative

P2 does **not** implement multi-component upvalues.

Any candidate captured by a runtime block:

```phalcom
const p = Point(x: 1, y: 2)

const f = {
  p
}
```

falls back.

Even:

```phalcom
const f = {
  p.x
}
```

falls back in P2.

Do not invent:

```text
capture leaf H
capture leaf H+1
reconstruct binding in closure
```

That is a separate ABI problem.

The semantic occurrence identity makes capture detection reliable:

```text
nested body occurrence refers to outer BindingId
    =>
captured = true
```

---

# 18. Mutable/reassigned bindings fall back

P2 handles immutable product locals.

If a binding can be reassigned:

```phalcom
var p = Point(...)
p = Point(...)
```

do not virtualize it.

Do not attempt:

```text
per-assignment virtual-shape mutation
phi merging
shape-version tracking
```

That is beyond this checkpoint.

Use existing semantic mutability/binding information where available.

Do not infer mutability solely from a token spelling if canonical semantic binding metadata already exists.

---

# 19. Alias propagation is deliberately out of scope

This remains canonical:

```phalcom
const p = Point(x: 1, y: 2)
const q = p
q.x
```

Do not optimize `p` and treat `q` as an implicit alias to the same leaf region.

That would introduce:

```text
virtual-value SSA aliasing
lifetime sharing
multi-name materialization ownership
```

which P2 intentionally avoids.

Fallback to P1.

P3/later optimization may broaden this.

---

# 20. Standalone/no-semantic-lowering compiles must remain conservative

P1 has compatibility paths that can synthesize some lowering information for standalone or unlinked compilation.

P2 must not treat that as equivalent to formal semantic proof.

Optimization eligibility requires canonical semantic facts:

```text
resolved constructor identity
resolved component identity
resolved BindingId
resolved match candidates
```

If the compiler is operating without the semantic lowering required to prove those facts:

```text
ProductOptimizationDecision::Materialize(
    NoResolvedConstruction
)
```

Use canonical P1 lowering.

Do not guess.

---

# 21. Product optimization failure is never a user diagnostic

The product optimizer is optional backend optimization.

If proof is incomplete:

```text
fallback
```

not:

```text
compile error
```

Examples:

```text
missing optimization binding attachment
unsupported use shape
too many leaves
possible capture
multiple whole-value uses
dynamic pack
generic ambiguity
```

all mean:

```text
canonical P1 materialization
```

Only a contradiction after the optimizer has committed to `Virtualize` may be an internal compiler invariant failure.

---

# 22. Exact data descriptors remain P1 authority

Do not implement runtime exact type construction in P2.

Do not manually call into:

```text
RuntimeDataRegistry
ProductLayoutRegistry
runtime typing registry
```

from optimization code to rebuild `Point<Int>`.

When materialization is required, use P1's canonical construction recipe / executable semantic spec and emit P1 construction semantics.

Conceptually:

```text
virtual Point<Int>
    |
    | whole-value boundary
    v
load leaves
    |
    v
P1 ConstructData(spec)
    |
    v
exact Point<Int> RuntimeDataDescriptor
```

This protects:

```text
phantom generics
exact specialization
behavior ClassId
runtime type descriptor
layout sharing
```

P2 must never reconstruct type identity from runtime component classes.

---

# 23. Materialization should be recipe-driven

An `ActiveVirtualProduct` should retain enough information to invoke the same canonical P1 construction semantics later.

Do not duplicate:

```text
descriptor interning
layout selection
component packing
nullary behavior
exact type metadata
```

inside the optimizer.

P2 knows:

```text
which leaves represent which logical components
```

P1 knows:

```text
how a logical data value becomes a runtime materialized value
```

Maintain that separation.

---

# 24. One opaque whole-use policy — important extra restriction for loops

The plan allows one static whole-value use site:

```text
whole_value_use_count <= 1
```

There is a performance trap here.

Consider:

```phalcom
const p = Point(x: 1, y: 2)

while condition {
  sink(p)
}
```

There is only one source use site, but rematerializing there creates a new DataObject every iteration.

Canonical P1 creates one DataObject.

That is semantically valid because data is identity-free, but it can be a catastrophic performance regression.

Therefore use this conservative rule:

> **Do not apply one-use allocation sinking when the whole-value use may execute repeatedly relative to the construction.**

Track lexical/control repetition sufficiently to reject obvious cases.

At minimum distinguish loop nesting at binding construction and whole-use occurrence.

Conceptually:

```rust
struct ProductUseContext {
    loop_depth: u16,
}
```

If:

```text
whole_use.loop_depth > binding_creation.loop_depth
```

treat that use as potentially repeated and fall back.

A product constructed inside a loop and consumed once in the same iteration may still qualify:

```phalcom
while condition {
  const p = Point(...)
  sink(p)
}
```

because canonical P1 would also construct each iteration.

Do not build full frequency analysis.

Conservative lexical loop-depth proof is sufficient for P2.

Record this as a P2 architectural rule and test it.

---

# 25. Branch sinking is desirable

This case should optimize:

```phalcom
const p = Point(x: a, y: b)

if needWhole {
  sink(p)
}
```

If `needWhole` is false:

```text
zero DataObject allocation
```

If true:

```text
materialize at sink
```

This is one of the strongest reasons allocation sinking exists.

Test both branches.

---

# 26. Whole-value boundaries

For P2, treat these as opaque data-value boundaries:

```text
ordinary call/send argument
ordinary method receiver
return
non-local return
global assignment
field assignment
index/collection insertion
pack expansion
reflection
.class
===
==
hash
comparison
unknown dynamic consumer
```

P3 may later eliminate some such materializations.

P2 stays conservative.

Do not opportunistically optimize `.class` simply because the exact type is statically known.

That belongs to P3's broader observation-boundary work.

---

# 27. Multiple whole-value uses fall back

Do not implement cached lazy materialization.

Example:

```phalcom
const p = Point(...)
sink1(p)
sink2(p)
```

P2 should not:

```text
virtualize
materialize for sink1
cache box
reuse for sink2
```

Although this could be legal for identity-free data, it adds state and lifetime complexity.

Use P1 canonical materialization at binding initialization.

---

# 28. Nested data flattening

Only after single-level virtualization is green.

Example:

```phalcom
data Point(x: Int, y: Int)

data Rect(
  topLeft: Point,
  bottomRight: Point,
)

const r = Rect(
  topLeft: Point(x: 1, y: 2),
  bottomRight: Point(x: 3, y: 4),
)
```

Target logical leaves:

```text
r + 0 => topLeft.x
r + 1 => topLeft.y
r + 2 => bottomRight.x
r + 3 => bottomRight.y
```

Valid:

```phalcom
r.topLeft.x
```

should load one leaf.

Valid:

```phalcom
sink(r.topLeft)
```

may materialize only nested `Point`.

Valid:

```phalcom
sink(r)
```

recursively materializes the children and then Rect through canonical P1 recipes.

---

# 29. Do not flatten arbitrary existing data aliases in P2

This:

```phalcom
const p = Point(...)
const r = Rect(topLeft: p, ...)
```

does not imply that P2 can splice `p`'s leaf region into `r`.

That is virtual alias propagation.

P2 only recursively flattens a component when the component expression itself is an eligible exact nested data construction.

This keeps ownership/lifetime simple.

---

# 30. Leaf-budget is mandatory

Keep the initial budget:

```rust
MAX_VIRTUAL_PRODUCT_LEAVES = 8
```

unless the C5 performance evidence explicitly justifies changing it.

Do not tune it while debugging correctness.

A nine-leaf candidate:

```text
falls back
```

It is not an error.

One eligibility function owns this policy.

Do not spread slightly different leaf-budget checks through:

```text
binding compiler
projection compiler
materialization compiler
match compiler
```

There must be one canonical decision.

---

# 31. Suggested pure eligibility boundary

Architecturally aim for a decision function like:

```rust
fn decide_product_optimization(
    candidate: &ProductCandidate,
    uses: &ProductUseSummary,
    context: &ProductControlContext,
) -> ProductOptimizationDecision
```

It should not emit bytecode.

It should not modify Compiler state.

It should not query runtime objects.

It should be highly unit-testable.

All emitters consume its result.

This separation is important:

```text
proof
    !=
emission
```

When a test fails, we need to know whether:

```text
planner accepted wrong candidate
```

or:

```text
emitter mishandled a correctly accepted candidate
```

---

# 32. Enum virtualization has stricter semantics than data

Do not generalize data rematerialization rules to enums.

General enum case allocation identity has **not** been ruled representation-independent in C1.P2.

Therefore an enum case may be virtualized only if the whole case object cannot become observable.

Eligible conceptual example:

```phalcom
const r = Result::Ok(expensive())

match r {
  Result::Ok(x) => use(x)
  Result::Error(e) => useError(e)
}
```

No other use of `r`.

---

# 33. Enum candidate requirements

Require all of:

```text
exact resolved general variant
positive payload
not NativeOption
immutable local
not captured
not reassigned
no opaque expansion
zero whole-value uses
every use is resolved match scrutinee/payload use
no root pattern binding of whole case
```

Do not weaken these.

---

# 34. Absolutely no enum rematerialization

If allocation was elided for a general enum case, P2 must never later fabricate a fresh case object.

If code emission discovers an unexpected whole enum use after the planner accepted the candidate:

```text
internal optimizer invariant failure
```

not:

```text
construct a new case and continue
```

The latter could change observable identity.

This differs fundamentally from data.

---

# 35. NativeOption remains outside general enum virtualization

Do not touch the NativeOption special representation.

P1 deliberately routes variant construction through one central helper to preserve:

```text
general ADT ProductStorage
vs
NativeOption immediate encoding
```

P2 must retain that separation.

`Some`/`None` already have a better specialized representation.

Do not optimize them through virtual general-enum payload slots.

---

# 36. Exact enum match optimization consumes semantic match resolution

The backend must not solve matching.

The semantic layer already tells the compiler which variants a pattern can denote.

P2 may use:

```text
resolved candidate membership
```

to conclude:

```text
this arm cannot match our exact virtual VariantId
```

or:

```text
this arm can match; stage payload leaves
```

It may not perform:

```text
selector name comparison
variant base-name lookup
GADT solving
exhaustiveness solving
family overload resolution
```

in the compiler.

---

# 37. Virtual enum match architecture

A useful adapter is conceptually:

```rust
struct VirtualMatchScrutinee<'a> {
    variant: VariantId,
    payload_slots: &'a [u16],
}
```

Then compile arms using existing match scaffolding.

For each arm:

```text
resolved candidate set excludes exact variant
    -> emit arm failure/skip

resolved candidate set may include exact variant
    -> stage requested payload fields with GetLocal

child payload patterns
    -> existing pattern compiler
```

Do not rewrite the pattern subsystem.

Only replace the outer:

```text
IsVariant
GetVariantPayload
```

operations that the exact virtual case makes unnecessary.

---

# 38. Or-pattern and rollback behavior must remain untouched

The existing matcher has staging/rollback rules for bindings.

Virtual enum optimization must preserve them.

Do not directly bind payload leaves into final locals in a way that bypasses rollback.

Stage values in exactly the same logical place where:

```text
GetVariantPayload
```

would have supplied a `Value`.

Then let the existing child pattern machinery operate.

Test:

```text
overlapping arms
or-pattern alternatives
payload patterns that fail after staging
binding rollback
first-match semantics
```

---

# 39. GADT semantics belong to semantic analysis, not P2

When a GADT-related optimized test fails:

Do not modify:

```text
exact-case type theory
variant-local generic inference
case result specialization
exhaustiveness
type refinement
```

merely to get optimizer tests green.

First compare with:

```text
ProductOptimizationMode::Disabled
```

If Disabled is correct:

```text
the P2 optimizer is wrong
```

Fix planner/emission.

If Disabled is wrong:

```text
P1 / semantic baseline incident
```

Do not blur these categories.

---

# 40. Differential testing is the semantic oracle

Every meaningful optimization needs:

```text
Disabled
vs
Enabled
```

execution comparison.

Use the **same parsed source and same semantic snapshot/lowering products** if the harness permits.

Do not accidentally perform two unrelated semantic analyses and call differences optimizer differences.

Compare:

```text
return/result value
printed output
error kind
error message where stable
source span
side-effect log
control flow
```

Do not compare:

```text
heap ObjRef
allocation address
DataObject identity
```

because data deliberately has no allocation identity.

---

# 41. Allocation evidence must be deterministic

Do not infer allocation elimination solely from:

```text
heap live_count at end
```

A materialized object may have become unreachable or been collected.

Use at least two independent mechanisms:

```text
bytecode/disassembly assertion
+
targeted allocation instrumentation/count
```

For example:

```text
optimized projection-only Point:
    no ConstructData opcode
    DataObject allocation counter delta == 0
```

If P1 already has stable allocation instrumentation, reuse it.

If not, add a **test-only** narrow counter at:

```text
Heap::alloc_data
VM::construct_data_value
```

or the actual canonical allocation owner.

Do not add production language-visible counters.

---

# 42. Important P2 hard cases

You must write hostile tests for at least:

```text
labeled arguments in reversed declaration order
unselected constructor argument has a side effect
unselected constructor argument throws
large Int component
heap object component
closure component
nested data
one branch materialization
whole-use inside a loop -> bailout
capture -> bailout
reassignment -> bailout
alias -> bailout
dynamic pack -> bailout
leaf count 8 -> optimize
leaf count 9 -> bailout
GC while only virtual leaf retains object
fiber suspension with virtual leaf
same-frame caught exception while ephemeral scratch exists
non-local return during constructor evaluation
shadowing with same source variable name
sacred inline fast path
sacred inline fallback path
REPL cells with repeated source names
```

For enums additionally:

```text
two variants with identical payload layouts
GADT exact case
variant-local generic
or-pattern
wildcard
root whole-value pattern binding -> bailout
scrutinee used inside arm body -> bailout
method send -> bailout
=== -> bailout
capture -> bailout
NativeOption -> untouched
```

---

# 43. Control-flow repetition and allocation sinking

Record this explicitly in implementation state:

```text
one static whole-value use
    does not necessarily mean
one dynamic materialization
```

Reject obvious repeated-use sinking as described earlier.

This is not a language semantic issue.

It is a P2 profitability/safety policy.

Do not knowingly replace:

```text
1 eager allocation
```

with:

```text
N allocations in a loop
```

under the label "optimization".

---

# 44. Sacred inline optimization interaction

Phalcom may compile one source block:

```text
as ordinary closure
as inlined sacred fast path
as fallback/deopt path
```

Do not let active optimizer state leak across these copies.

The proof plan may be immutable and reusable.

The codegen state must be separate.

Bad:

```text
global map keyed by SourceRange
    -> mutable ActiveVirtualProduct
```

Good:

```text
immutable semantic/use plan
+
FunctionState-local active slot representation
```

Identical source ranges appearing in multiple compiled copies are not one runtime slot environment.

---

# 45. Superinstruction fusion comes after P2

Do not teach the product optimizer to perform existing peephole fusion.

Do not teach `fuse_superinstructions()` about high-level product semantics unless a concrete correctness requirement emerges.

The intended ordering is:

```text
P2 optimized canonical emission
        |
        v
ordinary bytecodes
        |
        v
existing fuse_superinstructions()
```

Tests must cover branch targets and fused local invokes if the new `GetLocal` patterns trigger existing fusion.

---

# 46. REPL and incremental compilation

The product plan is transient.

Do not store:

```text
ProductFunctionPlan
VirtualProductKind
MaterializationReason
```

as semantic DB query products.

Do not add:

```text
QueryKey::ProductOptimization
```

P2 policy belongs to backend codegen.

Each REPL cell/compile unit gets fresh planner state.

Semantic source identity remains canonical.

Editing a data component changes P1 lowering, which naturally changes the next derived optimization plan.

An unrelated method edit must not cause a new semantic optimizer product because there should be none.

---

# 47. Optimizer mode must be invisible

`ProductOptimizationMode` exists for:

```text
tests
differential validation
benchmarking
```

Production compilation defaults to Enabled after checkpoints allow it.

Do not expose:

```text
CLI language switch
runtime property
reflection API
environment-controlled language semantics
```

unless explicitly requested later.

Optimizer choice is not part of Phalcom program semantics.

---

# 48. C0 execution order

Implement C0 exactly as a no-behavior-change checkpoint.

## C0.1 Reconcile P1

Record actual final symbols.

Do not rewrite them.

## C0.2 Add mode

Before any transformation:

```text
Enabled
Disabled
```

must emit equivalent canonical code.

Prefer exact chunk/executable-semantic equivalence where practical.

At minimum opcode + relevant semantic pool behavior must be equivalent.

## C0.3 Add planner

Planner produces decisions.

Emitter ignores them at first.

Only when C0 evidence proves:

```text
planner exists
no behavior changed
```

may C1 begin.

---

# 49. C1 execution order

Do not activate broad optimization yet.

First implement:

```text
ActiveVirtualProduct state
slot reservation
ephemeral scratch
constructor-to-leaf emitter
projection emitter
data rematerialization emitter
```

Then hostile-test:

```text
stack
capture interaction
GC
fibers
exceptions
non-local returns
```

Only after that activate optimization in C2.

This sequencing is essential.

---

# 50. C2 execution order

Activate in increasing complexity.

### First

```phalcom
Point(x: a(), y: b()).x
```

No persistent binding.

### Then

```phalcom
const p = Point(...)
p.x
p.y
```

Persistent virtual local.

### Then

```phalcom
const p = Point(...)
p.x
sink(p)
```

One-use allocation sinking.

Do not implement all three simultaneously and debug them as one feature.

---

# 51. C3 execution order

Only after C2 differential matrix is fully green:

```text
nested shape construction
nested component path
selective nested materialization
outer recursive materialization
leaf-budget bailouts
```

Keep:

```text
Tuple
Record
general enum
class
```

as scalar `Value` leaves in P2.

P3 will later generalize transparent structural products.

---

# 52. C4 execution order

Only after data optimization is stable:

```text
strict enum eligibility proof
virtual enum payload slots
virtual exact-match adapter
GADT / identity / NativeOption qualification
```

Do not reuse data rematerialization logic for enum values.

---

# 53. C5 is not optional cleanup

C5 is part of P2 implementation.

It must verify:

```text
sacred inlining
fallback copy
superinstruction fusion
incremental compilation
REPL
source/LSP invisibility
allocation matrix
benchmarks
workspace tests
Clippy
negative architecture searches
```

Do not declare P2 done after C4.

---

# 54. State-file requirements

Continue the existing LANG005 C1 state record if one exists.

If the current checkout has no canonical C1 state file, create:

```text
docs/implementation/LANG005/LANG005.C1-/implementation-state.md
```

Do not create multiple competing state files.

Maintain:

```markdown
## C1.P2 base revision

## P1 takeover interface map

## Architectural amendments applied
- canonical BindingId optimizer identity
- source-order construction invariant
- loop-repeat allocation-sinking restriction
- etc.

## Checkpoint status

## Established invariants

## Evidence ledger

## Differential matrix

## Allocation / opcode matrix

## Benchmark evidence

## Negative searches

## Active incident

## Next resume action
```

Do not store chain-of-thought.

---

# 55. Incident protocol

If a checkpoint fails, do not immediately patch surrounding systems.

Classify first.

Use:

```text
PROOF
    planner accepted an unsafe candidate

EMISSION
    candidate was valid, generated bytecode was wrong

P1 CONTRACT
    canonical disabled execution is wrong

BINDING IDENTITY
    backend lacks/corrupts semantic BindingId attachment

SCRATCH / STACK
    ReserveScratchLocal / ReleaseScratchLocal invariant failed

MATCH BACKEND
    virtual exact-case adapter diverged from canonical match lowering

GC / FIBER
    Value leaf was not preserved as runtime root

BASELINE
    failure exists before P2

PLAN DRIFT
    current architecture differs mechanically from plan
```

Fix the narrow owner.

Do not alter language semantics.

---

# 56. Specific implementation sketches

These are architecture demonstrations, not copy-paste requirements.

## 56.1 Correct planner identity

```rust
struct ProductFunctionPlan {
    decisions: BTreeMap<BindingId, ProductOptimizationDecision>,
    uses: BTreeMap<ExpressionId, ProductUseKind>,
    ephemeral: BTreeMap<ExpressionId, VirtualShapePlan>,
}
```

If compiler lowering uses another canonical site identity, substitute it.

The invariant is what matters.

---

## 56.2 Correct planner traversal

Pseudo-logic:

```rust
fn visit_expression(
    expression: &Expr,
    semantic_site: SemanticExpressionSite,
    ctx: UseContext,
    plan: &mut ProductPlanBuilder,
) {
    let resolved_binding = semantic_site.binding_id();

    match (resolved_binding, expression, ctx) {
        (Some(binding), Expr::Var(_), UseContext::DataProjection(path)) => {
            plan.record_projection(binding, semantic_site.expression_id(), path);
        }

        (Some(binding), Expr::Var(_), UseContext::AssignmentTarget) => {
            plan.record_reassignment(binding);
        }

        (Some(binding), Expr::Var(_), UseContext::NestedRuntimeBlock) => {
            plan.record_capture(binding);
        }

        (Some(binding), Expr::Var(_), _) => {
            plan.record_whole_value_use(binding, semantic_site.expression_id());
        }

        _ => {
            visit_children(...);
        }
    }
}
```

What is deliberately absent:

```rust
lookup_name("p")
nearest_lexical_scope_with_name("p")
```

---

# 57. Correct virtual data construction

Pseudo-emission:

```rust
fn compile_virtual_data_constructor_into(
    &mut self,
    construction: &DataConstructionLoweringSpec,
    source_arguments: &[Expr],
    leaves: &VirtualLeafRegion,
) -> Result<(), CompilerError> {
    for source_argument in source_arguments {
        let component =
            construction.component_for_source_argument(source_argument.site())?;

        self.compile_expr(source_argument)?;

        let target_leaf = leaves.for_component(component);

        self.emit_set_local(target_leaf);
        self.discard_expression_copy_if_required();
    }

    Ok(())
}
```

Do not evaluate arguments by `component_index`.

---

# 58. Correct scalar projection

For:

```phalcom
p.x
```

with:

```text
BindingId(p)
    -> ActiveVirtualProduct
    -> x = leaf offset 1
```

emit:

```text
GetLocal(head_slot + 1)
```

Do not emit:

```text
GetLocal(head_slot)
GetDataComponent
```

Do not materialize `p`.

---

# 59. Correct data whole-use materialization

Conceptually:

```rust
fn emit_virtual_data_materialization(
    &mut self,
    product: &ActiveVirtualProduct,
) -> Result<(), CompilerError> {
    for constructor_parameter in
        self.materialization_recipe(product.materialization_spec)?
    {
        self.emit_virtual_component_value(
            product,
            constructor_parameter.component,
        )?;
    }

    self.emit(Bytecode::ConstructData {
        constructor: product.materialization_spec,
        arity: product.logical_arity(),
    });

    Ok(())
}
```

The actual stack order must follow current P1 bytecode contract.

Do not pack `ProductStorage` in compiler code.

---

# 60. Correct nested materialization

Given:

```text
Rect
    topLeft -> virtual Point
    bottomRight -> virtual Point
```

whole Rect materialization:

```text
materialize topLeft using Point P1 recipe
materialize bottomRight using Point P1 recipe
ConstructData(Rect recipe)
```

Projection:

```phalcom
r.topLeft.x
```

goes directly:

```text
GetLocal(x leaf)
```

Projection:

```phalcom
r.topLeft
```

materializes Point only.

---

# 61. Correct enum virtualization

Constructor:

```phalcom
const r = Result::Ok(payload())
```

eligible only after proof.

Then:

```text
reserve payload leaf
evaluate payload()
store leaf

NO ConstructVariant
```

Match:

```phalcom
match r {
  Result::Ok(x) => ...
  Result::Error(e) => ...
}
```

uses semantic candidate information:

```text
Ok arm:
    known exact VariantId == Ok
    stage payload with GetLocal

Error arm:
    candidate cannot match
    skip/fail arm
```

No new case object exists.

---

# 62. Things you must NOT "improve"

Do not implement any of the following while doing P2:

```text
Tuple/Record virtualization
general SSA
register allocation
unboxed frame Ints
captured virtual products
virtual alias propagation
multi-use cached materialization
interprocedural escape analysis
multi-value return ABI
method specialization
trait devirtualization
general match decision DAG
enum rematerialization
NativeOption rewrite
runtime-generic reification redesign
List<Data> inline storage
JIT
```

P3 and later work have owners.

Scope discipline is part of correctness.

---

# 63. Required deterministic qualification matrix

At C5, explicitly verify:

| Fixture | Disabled | Enabled |
|---|---|---|
| `Point(...).x` | one materialized data object | zero |
| local Point + projections | one eager object | zero |
| local Point + one branch whole use, branch false | one eager object | zero |
| same branch true | one eager object | one sunk object |
| whole use inside loop outside binding | one eager object | **must bail out** |
| nested Rect/Point projections | nested materialization | zero product allocations |
| nine-leaf data | P1 | P1 |
| captured data | P1 | P1 |
| reassigned data | P1 | P1 |
| alias | P1 | P1 |
| eligible exact general enum match | one case object | zero |
| enum method/equality/identity use | canonical object | canonical object |
| NativeOption | P1 special representation | identical |

This matrix is a hard release gate.

---

# 64. Benchmark interpretation

Criterion is supporting evidence.

It is not the proof of optimization.

First establish:

```text
correct bytecode delta
correct allocation delta
correct Enabled/Disabled semantics
```

Then benchmark.

Record:

```text
runtime
compiler time
frame-slot pressure
allocation count
```

Pay attention to optimizer-planning cost.

Do not accept a transformation that saves one allocation but materially degrades common compile throughput without documenting evidence.

Do not silently change the eight-leaf threshold based on intuition.

---

# 65. Negative architecture gates

At final verification, ensure:

```text
no ProductOptimization query in semantic DB

no VirtualProduct runtime heap object

no user-visible optimization mode

no compiler-side declaration-name special cases

no anonymous Tuple/Record takeover

no unboxed parameter/return ABI

no general enum rematerializer

no NativeOption generalization
```

Additionally add the post-plan identity gate:

```bash
rg 'ProductBindingSite.*name|range.*name' \
  phalcom-core/src/compiler
```

Investigate any optimizer binding identity still based on name/range.

And:

```bash
rg 'lookup.*name|resolve.*name' \
  phalcom-core/src/compiler/lib/product_opt.rs
```

There should be no optimizer-local semantic variable resolver.

Name strings may appear only for diagnostics/debugging, never as proof identity.

---

# 66. Final validation

Run focused tests checkpoint by checkpoint.

At completion run at least:

```bash
cargo +stable fmt --all -- --check

cargo +stable check --workspace --all-targets

cargo +stable test -p phalcom-semantic
cargo +stable test -p phalcom-core
cargo +stable test -p phalcom-lsp

cargo +stable test --workspace --all-targets

cargo +stable clippy --workspace --all-targets -- -D warnings
```

Use the repository's pinned toolchain/environment if it differs from the examples.

Run current CI-specific gates relevant to touched code.

Do not declare COMPLETE because one focused filter is green.

---

# 67. What "P2 complete" means

You are finished only when all of these statements are true:

```text
P1 canonical path remains intact.

Disabled mode reproduces P1.

Optimizer uses canonical semantic BindingId identity.

No backend name resolver exists.

Data arguments evaluate once in source order.

Virtual leaves are ordinary Value frame slots.

Ephemeral scratch survives exceptions, captures around it, GC, and fibers.

Simple constructor→projection eliminates allocation.

Eligible local data scalar replacement eliminates allocation.

One safe non-repeating opaque data use sinks materialization.

Repeated-loop whole-use sinking conservatively bails out.

Nested data flattening works within the leaf budget.

Materialization always uses canonical P1 construction semantics.

General enum cases virtualize only when the whole case cannot be observed.

Virtual enum matching consumes resolved semantic candidates.

No general enum rematerialization exists.

GADT semantics remain unchanged.

NativeOption remains unchanged.

Sacred fast/fallback copies have independent active codegen state.

REPL/incremental compilation gets fresh transient optimizer state.

No optimizer policy is added to semantic DB.

Allocation/opcode differential tests pass.

Performance evidence is recorded.

Workspace gates pass.

No P2 incident remains.
```

The handoff to P3 is then:

> The compiler has a sound, backend-only virtual-product substrate. A proven identity-free data product can live as ordinary `Value` leaves, nested data can flatten recursively, component projections can bypass materialization, and canonical P1 materialization can be inserted at a controlled observation boundary. Exact general enum cases can also avoid allocation under a stricter proof that no whole case value is observable. All semantic identity comes from the semantic layer; optimization never becomes a second resolver or type checker.

---

# 68. Final instruction on decision authority

You are expected to make normal **mechanical implementation choices**:

```text
private helper names
module organization
small refactors
error helper extraction
test file placement
```

when they do not alter architecture.

You are **not** authorized to make semantic or architectural choices such as:

```text
changing when materialization is observable
changing enum identity semantics
adding a runtime guard instead of proof
making Tuple/Record part of P2
changing data equality
changing type inference
adding alias propagation
changing the ABI
changing the leaf budget without evidence
inventing a new BindingId alternative
making a P1 representation observable
```

If the current repository makes one of the governing decisions impossible:

1. stop that checkpoint;
2. preserve the working tree;
3. record a `PLAN DRIFT` or appropriately classified incident;
4. explain the exact conflicting code/invariant;
5. do not choose an alternative architecture yourself.

Implement the architecture given here and in the P2 plan—not a nearby architecture that happens to be easier.