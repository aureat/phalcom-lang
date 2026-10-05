---
id: LANG007.C4.guidance
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: checkpoint-guidance
title: "Structural Static Upgrade"
status: PROPOSED
prepared_against_revision: 8c3a98aab41ed8631e9f8746a6b29984cf77581d
prepared: 2026-10-04
---

# LANG007.C4 — Implementation Guidance

## Structural Static Upgrade

## 0. Purpose

This document is the compact architectural guidance for all `LANG007.C4`
implementation plans.

Use it with:

```text
LANG007.C4-CHECKPOINT.md
the active C4 numbered plan
the active plan requirements analysis
the predecessor walkthrough/handoff
the live repository
the current authoritative specs
```

This guidance is not a replacement for the active plan.

It fixes the checkpoint-wide architecture and plan boundaries that later
implementers must preserve.

---

# 1. C4 in one sentence

Turn Telos's C1–C3 structural/type-domain foundation into one bounded,
deterministic static semantic execution system with first-class efficient
Packs, generalized structural reflection, reusable pattern/proof semantics,
durable residualization/materialization, and a hard public ontology migration
without building a second VM or second type system.

---

# 2. Read this first

C4 has one overriding architectural rule:

> **Static Telos is ordinary Telos semantic code evaluated by a bounded,
> deterministic semantic evaluator over immutable semantic static values.**

Correct:

```text
existing semantic declaration identity
existing type/relation engine
existing row calculus
existing pattern/GADT resolution
existing query budget/cancellation
existing incremental DB
existing optimized compiler/runtime products
        ↓
C4 static evaluator / reflection / specialization
```

Incorrect:

```text
"compile time"
therefore
"run the VM"
```

Incorrect:

```text
"static match"
therefore
"build another pattern resolver"
```

Incorrect:

```text
"structural reflection"
therefore
"recursively expand every contained type's shape"
```

Incorrect:

```text
"Pack looks recursive in source"
therefore
"represent it as a linked cons list"
```

Incorrect:

```text
"proof predicate returns Bool"
therefore
"Unknown == false"
```

---

# 3. Non-negotiable inherited C1–C3 rules

Do not redesign C1–C3 merely because C4 exposes a richer public structural
surface.

Preserve the canonical existing authorities for:

```text
TypeId / ProperTypeId
static-domain kinds
TypePack / LabeledTypePack / CompleteTypePack internal terms
GenericParameterMode / generic binder headers
checked pack substitution/materialization
RecordRow identity and solver
AssociatedSurface/type-member lookup
VariantId / VariantFamilyId
GADT branch refinement
rich relation outcomes
semantic query/incremental identity
metadata schema-v10 baseline
```

Important distinctions remain:

```text
LabeledTypePack != RecordRow
CompleteTypePack != Tuple
static value != runtime Value
Type == semantic type identity, not reflected Shape expansion
```

C4 may migrate the **public ontology** away from old TypePack/RecordFields
names while retaining optimized internal C1–C3 nodes.

If a C4 plan appears to require deleting/replacing those internal authorities
rather than adapting them, STOP AND CONSULT.

---

# 4. Binding and field foundation

P1 establishes semantic ground that every later plan inherits.

## 4.1 Binding surface

Canonical:

```telos
let x = runtimeExpr
let mut y = runtimeExpr
let mut z
const c = staticExpr
```

Meaning:

```text
let      Runtime + Immutable
let mut  Runtime + Mutable
const    Static
```

Do not encode all three states in one `is_const` or `mutable` boolean.

Prefer explicit phase and runtime mutability.

## 4.2 Initialization

`let mut` may be declared without a value.

Its state is not `None`.

Canonical flow lattice:

```text
Uninitialized
MaybeInitialized
DefinitelyInitialized
```

Policy:

```text
Uninitialized read
    compile-time error when proven

MaybeInitialized read
    legal source
    runtime checked read

DefinitelyInitialized read
    runtime check omitted
```

Do not turn C4 into strict whole-program definite assignment.

Do not make closure/global uncertainty a compile error merely because local
analysis cannot prove it.

## 4.3 Runtime representation

An uninitialized binding must have private runtime representation distinct from
guest-visible `None`.

A proven local read remains branch-free.

The compiler consumes semantic read-safety facts; it does not solve control
flow again.

## 4.4 Fields

Canonical:

```telos
class C {
    _immutable: T
    mut _mutable: U
}
```

Stored fields are explicit only.

Assignment never creates storage.

Immutable instance fields use construction lifecycle:

```text
default initializer
or
one legal constructor-lifecycle write
```

Every successful normal construction exit establishes all required immutable
fields.

Do not implement first-write-anywhere freeze semantics.

## 4.5 Constructor-time early use

Constructor-exit proof alone does not guarantee that an ordinary method called
during construction cannot observe an uninitialized field.

P1 must retain a sound backstop:

```text
static restriction
or
runtime uninitialized-field check
```

The plan currently prefers the minimal runtime backstop unless the live
repository already proves the stronger restriction.

Never allow uninitialized storage to surface as a value satisfying the final
field contract.

---

# 5. Static value architecture

P2 owns the first real static evaluator substrate.

## 5.1 Dedicated semantic values

Conceptually:

```rust
enum StaticValueData {
    Unit,
    Bool(bool),
    Int(...),
    Float(...),
    String(...),
    Symbol(...),
    Type(TypeId),
    StaticTerm(TypeId),
    Pack(...),
    Tuple(...),
    Record(...),
    Data(...),
    Variant(...),
}
```

Exact representation may differ.

Forbidden canonical representation:

```text
phalcom_core::Value
ObjRef
VM heap object
runtime allocation identity
```

## 5.2 Reduced values are DAGs

Reduced structural static values should form an acyclic DAG.

Recursive semantic concepts use stable references:

```text
TypeId
VariantId
DefinitionId
stable semantic definition/reference
```

Do not introduce arbitrary cyclic `StaticValue` graphs unless a future
ratified use case requires them.

This keeps:

```text
equality
hashing
memoization
metadata
incremental fingerprints
materialization
```

tractable and deterministic.

## 5.3 Static strings

Static String semantics are content/value semantics.

Interning is an implementation detail.

Never expose static arena/intern identity through Telos equality or identity.

## 5.4 Closed const

A closed:

```telos
const x = ...
```

must fully reduce.

If it remains symbolic, C4 reports a compile-time failure.

Do not rerun the initializer at runtime.

---

# 6. Static evaluator architecture

## 6.1 Explicit evaluator control

Conceptually:

```rust
enum EvalControl {
    Value(StaticValueId),
    Return(StaticValueId),
    Break,
    Continue,
}
```

The evaluator must model source control semantically rather than by invoking
runtime methods.

## 6.2 Bounded and deterministic

Reuse existing:

```text
QueryBudget
CancellationToken
incremental semantic query machinery
```

Do not invent arbitrary hidden recursion/loop caps.

If structural value/residual growth needs a separate resource budget, make it a
first-class deterministic semantic budget.

## 6.3 No arbitrary runtime execution

Static evaluation does not invoke:

```text
runtime VM dispatch
native methods
I/O
filesystem
network
processes
clock
randomness
fibers
mutable runtime globals
runtime allocation identity
```

Only explicitly supported semantic operations reduce.

Unsupported runtime behavior residualizes or diagnoses according to static
context rules.

## 6.4 Untaken branches

Untaken static branches are not evaluated.

C4 static semantics remain ordinary control-flow semantics, not eager
normalization of both arms.

---

# 7. Static control flow

P6 owns control-flow integration.

Supported semantic constructs:

```text
if
match
for
while
return
break
continue
```

## 7.1 `while`

`while` must survive to semantic/static analysis as a real control-flow
construct.

Absolutely no special meaning attaches to a method/selector named:

```text
whileTrue
```

A user-defined `whileTrue` is just an ordinary method.

Runtime lowering may later compile semantic `while` into efficient jumps.

## 7.2 Static `for`

Static structural iteration is not the runtime Iterator protocol.

It iterates semantic structural products such as Pack/Product/Variants through
compiler-known structure.

Runtime iteration remains independently defined.

## 7.3 Heterogeneous binder refinement

For:

```text
[Int, String, Bool]
```

a static structural loop should specialize/refine the binder per iteration:

```text
iteration 0 -> Int
iteration 1 -> String
iteration 2 -> Bool
```

Do not erase exact correlation into a lossy union when the evaluator knows the
exact static entry.

---

# 8. Pattern and GADT guidance

One Telos pattern language.

Reuse:

```text
MatchResolution
MatchArmResolution
PatternResolution
ResolvedVariantPattern
ResolvedVariantCandidate
BranchProofEnvironment
existing GADT equalities/refinements
```

C4 may add only the minimum structural Pack/Product forms needed.

Do not build:

```text
StaticPattern
StaticVariantResolver
StaticGadtEnvironment
```

as independent semantic authorities.

The static evaluator is a consumer of resolved patterns.

---

# 9. Proof predicate guidance

C4 exposes static predicates such as:

```text
Equivalent<A, B>
Subtype<A, B>
Conforms<A, Trait>
```

Public result:

```text
Bool
```

Internal proof result remains richer.

Canonical mapping:

```text
proven      -> true
refuted     -> false
unknown     -> residual
blocked     -> residual or diagnostic
dynamic     -> residual where legal
cancelled   -> evaluator outcome, never Bool
budget      -> evaluator outcome, never Bool
internal    -> compiler failure, never Bool
```

Key rule:

> Failure to prove is not proof of false.

Generic example:

```telos
type IsNumber<T>: Bool =
    Subtype<T, Number>
```

While `T` is abstract:

```text
Subtype<T, Number>
```

may remain residual.

On later specialization it may reduce to true or false.

If a closed context requires a concrete Bool and the proof remains unresolved,
diagnose rather than inventing a third public Bool value.

Tooling may explain unresolved proof state.

Uncertainty is compiler epistemology, not a user-level Bool variant.

---

# 10. Residual IR guidance

P7 owns normalized semantic residualization.

## 10.1 Use semantic residual nodes

Conceptually:

```rust
enum StaticResidualData {
    Parameter(...),
    Apply { definition, arguments },
    Intrinsic { operation, arguments },
    If { condition, then_term, else_term },
    Match { ...resolved semantic pattern refs... },
    Product { ... },
}
```

Exact node set may evolve.

## 10.2 Do not serialize AST

AST contains:

```text
source sugar
unresolved names
source ranges
presentation forms
```

and is not the durable static semantic contract.

## 10.3 Do not serialize evaluator frames

Evaluator stack/locals/budget/memo state are implementation state.

They are not residual semantics.

## 10.4 Residuals should be stable enough for

```text
memoization
incremental fingerprints
specialization
cross-module metadata
diagnostics/tooling
```

---

# 11. Recursive static computation

C4 intentionally supports recursive generic/static computation.

## 11.1 No host recursion dependency

Use explicit evaluator frames.

A deeply recursive Telos static algorithm must not require equivalent Rust call
stack depth.

## 11.2 Cycle detection

Memo/application key conceptually includes:

```text
stable static definition identity
canonical static arguments
relevant specialization/proof environment
```

Exact repeat:

```text
F<X> -> F<X>
```

is a recursive cycle.

Changing arguments:

```text
Count<P4>
 -> Count<P3>
 -> Count<P2>
 -> ...
```

is not a cycle.

It remains bounded by ordinary evaluator budget.

## 11.3 Do not cache incomplete outcomes as success

Never treat:

```text
Cancelled
BudgetExceeded
Blocked
InternalFailure
```

as a successful reduced result.

---

# 12. Pack architecture

Pack is not merely another Tuple.

It is a specialized immutable structural transport type for:

```text
variadic capture
argument construction
spreading
rest
structural reflection
static structural traversal
```

## 12.1 Flat storage

Preferred logical representation:

```text
PositionalPack
    contiguous values / slice

LabeledPack
    contiguous labels
    contiguous values

CompletePack
    positional lane + labeled lane
```

Do not use a recursively nested cons representation.

## 12.2 O(1) views

Source:

```telos
PackView::Entry(value, rest)
```

is one-step decomposition.

Implementation:

```text
same backing
start + 1
length - 1
```

`rest` is O(1).

Do not allocate every suffix.

## 12.3 Call path

Ordinary fixed call:

```telos
foo(a, b, c)
```

must not allocate/build a first-class Pack.

Compile directly to fixed argument window.

First-class Pack is used when semantics require actual packing/capture/spread.

## 12.4 Multi-spread

```telos
foo(*a, *b, *c)
```

must not repeatedly concatenate immutable Packs.

Lower as argument segments and materialize/copy final argument window once.

Effective complexity:

```text
O(total arguments)
```

## 12.5 Rest capture

Prefer zero-copy/slice-backed rest capture where lifetime/layout permits.

Do not finalize rest through Tuple merely because current runtime does.

## 12.6 Labeled lane

Parallel label/value lanes are a good fit for current call infrastructure.

Duplicate detection may use:

```text
small n  -> linear scan
large n  -> temporary hash/index
```

Do not require a permanent map in every Pack merely for duplicate checks.

## 12.7 Virtualization

A first-class Pack may be:

```text
not materialized
compiler-virtual
materialized
```

depending on escape/use.

C4 should reuse existing scalar replacement/allocation-sinking substrate where
appropriate.

---

# 13. Pack static semantics

Static Pack should preserve the same logical semantics and asymptotics.

Conceptually:

```text
StaticPack backing
start
length
```

A static PackView does not construct recursive view chains.

Exact static Pack type shape may remain rich:

```telos
Pack<Int, Int, String>
```

while runtime/static storage remains compact and flat.

Rich type shape must not imply nested physical structure.

---

# 14. Product/Sum reflection

P5 exposes compiler-known structural information as ordinary intrinsic Telos
data.

Intended concepts:

```text
Product
OrderedProduct
UnorderedProduct
Sum

ProductShape
OrderedShape
UnorderedShape

ProductEntry / FieldInfo
Variant
Variants
```

## 14.1 Sealed compiler-derived evidence

A developer cannot claim arbitrary:

```text
T: Product
```

with a user implementation.

Compiler-derived structural evidence is authoritative.

## 14.2 Derive from lower compiler facts

Correct direction:

```text
DataInfo / EnumInfo / row/type representation
        ↓
structural classification
        ├─ sealed Product/Sum capability
        └─ ProductShape/Variant reflection
```

Forbidden:

```text
Product conformance
    ↔
Shape reflection
```

which creates recursive semantic queries.

## 14.3 Shallow reflection

For:

```telos
data Node {
    Next(Node)
}
```

reflection contains:

```text
field type = Type(Node)
```

It does not recursively reflect `Node::Shape`.

Likewise:

```telos
data List<T> {
    Empty
    Cons(T, List<T>)
}
```

`Cons` reflection stops at immediate payload Type references.

---

# 15. Sum / Variant guidance

Use existing variant semantic identity.

A name may identify multiple variant families/candidates.

Do not resolve variants purely by Symbol.

Public C4 Variant metadata is structural:

```text
name
immediate payload shape
ordinary structural identity/info
```

Do not expose matcher-internal GADT proof state in C4.

`Variants` should naturally use a positional Pack:

```telos
type Variants = PositionalPack<Variant>
```

where possible.

---

# 16. Generalized FieldKey

P4 owns:

```telos
type FieldKey = Symbol | String | Int | Bool
```

## 16.1 Stable total order

Language-defined ordering:

```text
Symbol < String < Int < Bool
```

Within:

```text
Symbol  canonical content
String  content
Int     numeric
Bool    false < true
```

Never use:

```text
intern ID
enum discriminant
hash-table order
process-local address
```

as semantic order.

## 16.2 Full-stack migration

FieldKey impacts:

```text
AST record keys
row interning
row inference
lacks constraints
substitution
relation/subtyping
matching
reflection
fingerprints
metadata
runtime RecordProductShape
diagnostics
```

Do not patch only reflection.

## 16.3 Tuple/Pack labels remain Symbol

Do not accidentally generalize every label concept.

Keep:

```text
Tuple/Pack/call labels -> Symbol
Record fields         -> FieldKey
```

unless a later ratified decision changes this.

---

# 17. Static materialization

C4 semantic value materialization must have one explicit seam.

Correct:

```text
StaticValue
    ↓
MaterializationDescriptor
    ↓
compiler lowering
    ↓
runtime representation
```

Incorrect:

```text
compiler asks evaluator opportunistically during lowering
```

Incorrect:

```text
runtime reruns const initializer
```

Incorrect:

```text
semantic evaluator allocates VM objects
```

Materialization descriptor must be independent of runtime heap identity.

---

# 18. Cross-module static semantics

P8 owns durable import/export.

Export stable meaning:

```text
static definition identity
parameter/result domains
checked semantic body/residual
stable dependency fingerprints
structural/static values
capability/proof requirements
```

Do not export:

```text
raw AST as semantic contract
evaluator call stack
arena/session IDs
runtime ObjRef
```

Importer specializes checked semantic representation using static arguments.

Const value changes must invalidate importing clients even if nominal type is
unchanged.

---

# 19. Equality/hash guidance

Structural equality is a semantic operation.

## 19.1 Pack

Compare:

```text
lane/shape
length
positional order
labels/order where significant
element semantic equality
```

## 19.2 Type inside structural values

Use canonical semantic Type equality/hash.

Do not compute:

```text
Type equality
    by
recursive Shape reflection
```

This is critical for recursive types.

## 19.3 Float

Pack/StaticValue do not invent their own Float equality/hash semantics.

Use the globally defined Telos Float semantics and preserve:

```text
a == b => hash(a) == hash(b)
```

---

# 20. Recursion and reflection safety

These checkpoint-wide rules are non-negotiable.

```text
R1  evaluator recursion uses explicit frames
R2  exact repeated application key is a cycle
R3  changing-argument structural recursion is allowed subject to budget
R4  reduced structural StaticValues form an acyclic DAG
R5  recursive semantic entities use stable references
R6  reflected shapes describe immediate structure only
R7  PackView is one-step, not recursive materialization
R8  constructing Pack/ProductShape never implicitly reflects its own type
R9  capability/reflection share lower structural classification
R10 equality/hash never recursively expand Shape
R11 metadata preserves references/DAG sharing
```

If a plan requires weakening one of these, STOP AND CONSULT.

---

# 21. Public ontology migration

P9 closes C4 with a hard public cutover.

Target source vocabulary centers on:

```text
Pack
PositionalPack
LabeledPack
CompletePack

Product
OrderedProduct
UnorderedProduct

ProductShape
OrderedShape
UnorderedShape

Sum
Variant
Variants
```

Legacy public:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordFields
```

does not remain as a permanent parallel ontology.

Internal optimized C1–C3 terms may remain if they are implementation details.

Do not equate public cleanup with deleting performant internal representation.

---

# 22. Source-defined structural algorithms

A key C4 objective is to make many structural utilities ordinary Telos source
rather than native/compiler primitives.

Examples include:

```text
count Pack entries
map/filter reflected variants
construct type-level structural results
look up reflected fields
derive structural predicates
compose ProductShape transforms
```

Compiler assistance is allowed where needed for:

```text
sealed structural evidence
canonical semantic operations
efficient Pack representation
materialization
specialization/fusion
```

Do not move ordinary algorithms native merely because their source definition
is recursive.

Optimize representation/evaluation instead.

---

# 23. Static source performance

Source-defined immutable Pack operations can accidentally become O(n²) if each
append copies.

C4 should recognize/fuse builder-like structural construction patterns where
necessary so canonical source algorithms remain effectively O(n).

Do not force developers to use a public mutable builder API solely to recover
performance.

The runtime/private argument builder remains an implementation mechanism.

---

# 24. P1 boundaries

P1 owns:

```text
let / let mut / const phase semantics
binding initialization lattice
runtime uninitialized-binding state
checked vs unchecked binding reads
immutable-default field syntax
explicit fields only
immutable field constructor lifecycle
module/source binding taxonomy
repository source cutover
P1-specific tooling/diagnostics/performance
```

P1 does not own:

```text
StaticValue evaluator
Pack value
FieldKey
Product reflection
static loops
residual IR
cross-module static metadata
```

If P1 starts implementing general static evaluation, it is drifting into P2.

---

# 25. P2 boundaries

P2 owns:

```text
StaticValue
StaticDefinition
static result domains
expression/block type-definition bodies
deterministic bounded evaluator foundation
closed const evaluation
semantic materialization descriptor seam
query budget/cancellation integration
```

P2 does not own:

```text
first-class Pack call semantics
FieldKey migration
Product reflection
full static control flow
recursive residualization
cross-module durable residual metadata
```

It must create the common substrate P3/P4/P5 consume.

---

# 26. P3 boundaries

P3 owns:

```text
first-class Pack runtime/static values
Positional/Labeled/Complete Pack
public Pack aliases/views
rest capture -> Pack
spread lowering
private builder convergence
fixed-call no-allocation path
O(1) views/slicing
source Pack-construction fusion as needed
```

P3 must not:

```text
turn fixed calls into Pack allocation
use recursive cons Pack storage
collapse Record into LabeledPack
```

---

# 27. P4 boundaries

P4 owns:

```text
FieldKey
heterogeneous record keys
canonical key order
row solver migration
record relation/subtyping migration
runtime RecordProductShape migration
metadata/fingerprint preparation
unordered product lookup
```

P4 must preserve current record optionality/width semantics.

It must not generalize Tuple/Pack labels to FieldKey without a new decision.

---

# 28. P5 boundaries

P5 owns:

```text
sealed Product/OrderedProduct/UnorderedProduct/Sum evidence
ProductShape/OrderedShape/UnorderedShape
Variant/Variants reflection
generalized type-member result domains
source-defined structural utilities where practical
```

P5 does not own general static control flow; P6 consumes the reflection model.

P5 must derive reflection/capabilities from lower semantic structure.

---

# 29. P6 boundaries

P6 owns:

```text
static if/match/for/while
return/break/continue
Pack structural patterns
Product/Unordered iteration
Equivalent/Subtype/Conforms
proof uncertainty residualization
existing pattern/GADT proof reuse
```

P6 must not introduce:

```text
whileTrue static intrinsic
runtime Iterator requirement for static for
Unknown -> false
static-only matcher
```

---

# 30. P7 boundaries

P7 owns:

```text
explicit evaluator frames
recursive static application
exact-key cycle detection
normalized semantic residual IR
memoization
residual specialization
growth controls
```

Memo key must include semantically relevant definition/argument/proof context.

Do not include irrelevant source ranges or transient arena addresses.

---

# 31. P8 boundaries

P8 owns:

```text
incremental static-definition query products
const value fingerprints
metadata schema evolution
stable StaticValue/Residual DAG
stable FieldKey metadata
cross-module specialization
materialization descriptor compiler consumption
cold/incremental parity
```

The compiler consumes frozen semantic results.

It does not run the evaluator opportunistically.

---

# 32. P9 boundaries

P9 owns:

```text
hard public ontology migration
Universe/core source updates
source-defined reflection algorithms
LSP/REPL/formatter/VS Code closure
docs/spec supersession
fuzz/regression closure
performance certification
checkpoint final acceptance
```

P9 should mostly consume P1–P8 architecture.

If P9 needs a new static-value, residual, FieldKey, Pack, or reflection identity
model, an earlier plan is incomplete: STOP AND CONSULT.

---

# 33. Testing doctrine

Testing is evidence gathering.

During P1–P8 BUILD mode, use the smallest discriminating test.

Canonical ladder:

```text
T0 exact reproducer / new regression
T1 direct feature test
T2 owning subsystem
T3 cross-layer integration
T4 affected crate
T5 workspace/release only at named certification
```

Do not run broad workspace tests reflexively after every edit.

P9 may deliberately broaden because it owns checkpoint closure.

Coverage design and actually executed verification are separate records.

---

# 34. Coverage dimensions that matter

Prioritize:

```text
semantic identity
closed/open static behavior
proof uncertainty
recursion/cycle detection
budget/cancellation
branch non-evaluation
pattern/GADT reuse
Pack zero-copy/slicing behavior
fixed-call no-allocation
FieldKey canonical order
recursive type reflection safety
structural equality/hash
metadata stability
cross-module specialization
cold/incremental equality
optimizer soundness
runtime-negative static behavior
tooling semantic identity
performance asymptotics
```

Do not mechanically add unrelated GC/concurrency tests unless a C4 change
actually affects those mechanisms.

---

# 35. Debugging budget

Mechanical:

```text
up to ~3 coherent correction cycles with concrete progress
```

Semantic:

```text
record:
Observed
Hypothesis
Evidence
Prediction
Discriminating test

one serious corrective attempt
then STOP AND CONSULT
```

Architecture:

```text
zero speculative redesign attempts
STOP AND CONSULT immediately
```

Never repeatedly rerun an unchanged failing command without a relevant state
change.

---

# 36. Global STOP / CONSULT triggers

Stop if any of these occur:

1. static evaluation appears to require running the runtime VM;
2. semantic StaticValue needs ObjRef/runtime identity;
3. a second pattern/GADT resolver is being introduced;
4. a second subtype/conformance engine is being introduced;
5. a second row solver is being introduced;
6. uncertainty must be coerced to false to continue;
7. Pack requires recursive cons storage to satisfy source semantics;
8. fixed calls would require first-class Pack allocation;
9. reflected recursive types require recursive Shape expansion;
10. Product capability must be user-forgeable for implementation convenience;
11. capability and reflection queries become mutually recursive;
12. semantic equality/hash requires reflected Shape recursion;
13. runtime initializer re-execution is being used instead of materialization;
14. raw AST/evaluator continuations are becoming durable residual metadata;
15. static metadata requires raw session/store IDs;
16. FieldKey canonical order would depend on intern/discriminant/hash order;
17. `whileTrue` is being given static-control significance;
18. static `for` would require the runtime Iterator protocol;
19. P3 needs to merge Record and LabeledPack;
20. P4 needs to generalize call/Tuple labels to arbitrary FieldKey;
21. public GADT proof metadata becomes necessary to complete C4 behavior;
22. the same nontrivial semantic failure survives one serious correction;
23. source/spec/test authorities materially conflict;
24. an invariant must be weakened to make tests green;
25. scope becomes materially more cross-cutting than the active plan.

---

# 37. Baseline failure policy

Classify observed failures:

```text
A — definitely caused by current C4 work
B — probably caused by current C4 work
C — unclear
D — clearly unrelated/baseline
```

Policy:

```text
A/B -> active responsibility
C   -> one bounded classification pass, then record/defer if nonblocking
D   -> record/defer immediately
```

The durable home is:

```text
LANG007.C4-CHECKPOINT.md
```

Do not absorb unrelated failures into C4 architecture.

---

# 38. Checkpoint bookkeeping

At each plan start:

```text
status          -> IN_PROGRESS when implementation actually begins
completion      -> PARTIAL
active_plan     -> active P<n>
record revision
verify predecessor handoff
```

At durable gates:

- record implemented invariant/interface;
- record actual verification;
- record consultation/amendment;
- record deferred failure.

Do not use CHECKPOINT as a chronological edit log.

At P9 closure, C4 may become:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

or stronger only if the actual named evidence justifies it.

Do not infer `RELEASE_COMPLETE` from plan completion.

---

# 39. Required plan completion artifacts

Every C4 plan normally produces:

```text
LANG007.C4.P<n>-walkthrough.md
LANG007.C4.P<n>-handoff.md
```

Walkthrough:

```text
what actually implemented/proved
```

Handoff:

```text
smallest complete context the next plan needs
```

The checkpoint remains the shared durable state authority.

---

# 40. Do not re-explore

Unless live evidence contradicts the architecture, do not spend implementation
time re-deciding:

```text
whether let should be immutable
whether let mut can be uninitialized
whether uninitialized means None
whether fields are explicit
whether fields are immutable by default
whether immutable fields initialize in constructor lifecycle
whether static Telos should run the VM
whether residuals should be raw AST
whether static matching needs a separate resolver
whether Unknown should be false
whether Product should be user-forgeable
whether static while should use whileTrue
whether const initializer reruns at runtime
whether TypePack public compatibility should remain indefinitely
whether Pack should be a recursive cons list
whether Pack rest should allocate
whether fixed calls should allocate Pack
whether recursive reflection should expand nested shapes
whether StaticValue should allow arbitrary cyclic containment
```

Those are already decided for C4.

---

# 41. Verify per plan

Before each numbered plan, verify live mechanical facts:

```text
exact AST/parser symbols
exact semantic query products
exact runtime bytecodes
exact metadata schema version
exact TypeMember/AssociatedSurface APIs
exact Pack/type-pack internal nodes
exact row representation
exact product runtime substrate
exact variant info representation
exact LSP source-index products
exact test module names
```

Mechanical drift is expected.

Architectural drift requires consultation.

---

# 42. Final architectural summary

```text
runtime source semantics
    let       -> immutable
    let mut   -> mutable
    const     -> static

semantic static world
    StaticValue
    StaticResidual
    deterministic bounded evaluator

one proof/type world
    existing relation/conformance engines

one pattern world
    existing resolved patterns + GADT proofs

one structural world
    compiler-known Product/Sum/row/type information
        ↓
    sealed structural capabilities
        +
    reflection values

one variadic transport world
    flat first-class Pack
    O(1) rest/views
    no fixed-call Pack allocation

one durable static world
    checked semantic definitions
    stable values/residuals
    dependency fingerprints
    materialization descriptors

recursive semantic graphs
    represented by stable references

reduced structural values
    acyclic DAGs

public C4 ontology
    Pack / Product / Sum
    not permanent legacy TypePack/RecordFields duality

no compile-time VM
no second matcher
no second relation engine
no recursive reflection expansion
no runtime const re-execution
```

If implementation remains inside these boundaries, C4 can add powerful
compile-time structural programming without destabilizing the semantic
authority established in C1–C3.
