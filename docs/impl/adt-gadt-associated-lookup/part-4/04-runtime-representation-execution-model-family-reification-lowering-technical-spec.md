# Phalcom ADT/GADT + Associated Lookup
## Part 04 — Runtime Representation, Execution Model, Family Reification, and Lowering

**Status:** Technical specification / runtime architecture contract  
**Series:** ADT/GADT + Associated Lookup, Part 4 of 6  
**Repository:** `aureat/phalcom-lang`  
**Verified repository branch:** `feat/adts`  
**Verified repository commit:** `f453a26a0e4e2c97640aeb3ac3cc212d8fb102f0`  
**Commit subject:** `feat: implement ADT semantic support`  
**Verified on:** 2026-08-31  
**Intended repository path:** `docs/impl/adt-gadt-associated-lookup/part-4/04-runtime-representation-execution-model-family-reification-lowering-technical-spec.md`

---

# 1. Executive Summary

Parts 1–3.5 establish the source grammar, semantic declaration identities, exact-case types, associated-family resolution model, generic/GADT specialization rules, captured capabilities, and the Option-A reification boundary. Part 4 defines how those already-proven meanings become executable values and bytecode without allowing the compiler or VM to reinterpret the source.

The runtime architecture chosen here is:

```text
static semantic identity
    VariantId / VariantConstructorId / CallableId / AssociatedFamilyId
    ExactCase / AssociatedResolution / FamilyApplicationResolution
                    │
                    │ compiler-owned projection only
                    ▼
backend lowering identity
    exact enum/variant/callable/family target specs
                    │
                    │ materialization/linking
                    ▼
runtime identity
    RuntimeEnumId / RuntimeVariantId / physical discriminant
    hidden case-behavior class / exact method handle / frozen family descriptor
                    │
                    ▼
physical value
    bare singleton      -> immediate case value, allocation-free
    constructor case    -> fresh `AdtCaseObject` in the current bytecode VM
    associated family   -> small capture object + shared immutable descriptor
    exact behavior ref  -> existing exact `BoundMethodObject`
    exact variant ctor  -> tiny compiler-generated callable thunk
```

This deliberately preserves:

```text
static semantic identity
!= runtime metadata identity
!= physical representation
!= allocation strategy
```

The current interpreter uses one 16-byte tagged `Value` everywhere. Part 4 therefore does not pretend that the bytecode VM itself is an unboxed native ABI. It adds a compact immediate representation for canonical singleton variants and a precise heap `AdtCaseObject` for fresh constructor results. A later AOT/native backend may represent a statically known `Option<Int>` as a conventional tagged union, apply niche optimization, scalar-replace a case, or stack-allocate it. Those are backend optimizations, not changes to `ExactCase` semantics.

The most important runtime decisions are:

1. **Bare singleton variants are canonical and may be immediate.** `Option::None` can be a tag-only `Value` with no heap object.
2. **Explicit zero-argument constructors remain fresh constructions.** `Option::None()` allocates a fresh case object in the current VM. It is not rewritten to the singleton just because it has no payload.
3. **Payload constructors allocate one case object in the current VM.** The object stores `RuntimeVariantId` plus immutable payload `Value`s. No separate wrapper is added merely because the static type is an enum.
4. **Generic ADTs are erased in the bytecode VM.** One runtime enum/variant descriptor exists per declaration/case, not per `Option<Int>` specialization. Generic/GADT proofs remain static.
5. **Enum roots are normal runtime behavior classes; exact cases use hidden final behavior classes.** Hidden classes exist to preserve ordinary `.` dispatch and `value.class` relationships. They are runtime metadata, not source declarations and not `TypeData::ExactCase`.
6. **Direct known associated invocation is direct.** `Option::Some(42)` lowers to exact variant construction. `System::print(1)` lowers to an exact behavioral target. Neither materializes a family.
7. **First-class families are frozen capabilities.** A reified family stores only the access-filtered, statically authorized operation set. Runtime invocation never re-runs associated hierarchy lookup or lexical access checks.
8. **Dynamic-shape family invocation routes only within the Part-3 candidate set.** It uses exact-shape-first then canonical compatible-rest ordering. It never searches the owner, class hierarchy, unrelated methods, or `doesNotUnderstand`.
9. **The compiler must receive a lowering projection from `SemanticSnapshot`.** The current `ProgramCompiler::compile_analyzed` only rejects semantic errors and exports metadata; the source compiler later recompiles source without expression-level semantic resolution. Part 4 closes that gap.
10. **GADT equality evidence is erased.** Runtime case identity and payload are enough. The theorem used to prove `Expr<Int>::Int` is not stored on each value.

Part 4 also defines the minimal case-test/payload-extraction runtime contract that Part 5 will consume for `match`, without implementing match semantics or exhaustiveness here.

---

# 2. Normative Inputs and Precedence

Part 4 consumes these documents as architectural inputs:

```text
Part 01
    01-surface-syntax-parser-ast-selector-family-grammar-technical-spec.md
    01-surface-syntax-parser-ast-selector-family-grammar-implementation-plan.md

Part 02
    02-declaration-model-family-reservation-variant-identity-exact-case-types-contracts-technical-spec.md
    02-declaration-model-family-reservation-variant-identity-exact-case-types-contracts-implementation-plan.md

Part 03
    03-associated-resolution-family-values-generic-specialization-invocation-typing-technical-spec.md
    03-associated-resolution-family-values-generic-specialization-invocation-typing-implementation-plan.md

Part 03.5
    03.5-g1-boundary-option-b-ready-family-type-source-design-spec.md
    03.5-part3-forward-compatibility-integration-and-verification-checklist.md
```

Precedence for Part 4 is:

1. ratified project/user decisions;
2. Part 3.5 for the G1/Option-A boundary;
3. Part 3 for associated resolution and family application products;
4. Part 2 for declaration/exact-case/GADT identity;
5. Part 1 for AST/source grammar;
6. current repository implementation for exact concrete symbols and runtime seams.

Part 4 may not reinterpret source-level associated lookup because a legacy runtime mechanism is convenient.

---

# 3. Fresh Repository Archaeology and Reconciliation

## 3.1 Verified branch state

The connected repository branch was re-read immediately before drafting this specification:

```text
branch: feat/adts
HEAD:   f453a26a0e4e2c97640aeb3ac3cc212d8fb102f0
parent: 2c8b5840fc5a864968cb2a832540fbcba868d9f8
subject: feat: implement ADT semantic support
```

The connected repository exposes the committed branch. Uncommitted local developer work is not observable through this repository connection. Therefore an implementing agent must repeat the preflight against the actual local tree before editing and treat any later Part-3 implementation as implementation truth for names, provided its semantics satisfy Parts 3/3.5.

## 3.2 Part 1 status at this baseline

The branch contains the dedicated enum and associated-expression front-end required by Part 1. New source syntax is not represented as the legacy `MethodRefExpr`/sealed-class expansion model.

The compiler still contains explicit staging failures for:

```text
EnumNotLoweredYet
AssociatedLookupNotLoweredYet
AssociatedInvokeNotLoweredYet
```

That is the correct pre-Part-4 state.

## 3.3 Part 2 status at this baseline

Part 2 semantic infrastructure is implemented. Exact current names include:

```rust
VariantId
VariantFamilyId
AssociatedFamilyId
VariantFieldId
VariantConstructorId
CallableOwnerId
CallableId

VariantShape
VariantVisibility
VariantFieldSemantic
VariantConstructorParameter
VariantConstructorSignature
VariantInfo
EnumInfo
EnumSemanticTable

AssociatedFamilyKind
AssociatedMemberId
AssociatedFamilyInfo
AssociatedSurface
AssociatedFamilyTable
```

`CallableOwnerId` distinguishes declaration-owned behavior from variant-owned case behavior. Variant constructors remain separate `VariantConstructorId` identities.

## 3.4 Part 3 status at this baseline

The repository contains the Part 3 and Part 3.5 design documents, but the committed branch does **not** yet contain the planned expression-level associated resolver and its lowering handoff products.

The following are therefore **required Part-3 prerequisites**, not Part-4-owned semantic work:

```text
InvocationTargetId
AssociatedResolution
AssociatedResolutionIndex
FamilyApplicationResolution
FamilyApplicationResolutionIndex
captured AssociatedValueDenotation
formal structural Family type
lane-preserving semantic rest selection
frozen dynamic candidate sets
Option-A reification finalization
```

Part 4 must not implement a second version of those algorithms in `phalcom-core`.

## 3.5 Current `Value` representation

`phalcom-core/src/value/repr.rs` implements a fixed two-word, 16-byte tagged `Value` with immediate tags for nil/unit/bool/int/float/symbol/object/option state. Heap objects are referenced through non-moving generational handles.

Consequences:

- Part 4 must not widen all `Value`s just to add ADTs.
- An additional immediate tag is cheap enough for canonical bare singleton cases.
- Payload-bearing cases naturally use a heap handle in the current interpreter.
- A future native backend is not required to copy this representation.

## 3.6 Current heap/GC architecture

`Heap` is a non-moving slotmap arena. `heap/trace.rs` uses an intentionally exhaustive `Object` match. `vm/gc.rs` similarly exhaustively destructures the `VM` root set.

Consequences:

- adding `Object::AdtCase` or `Object::AssociatedFamily` forces explicit GC edge classification;
- adding an ADT registry containing class handles forces an explicit VM-root decision;
- payload values must be traced through `Value::gc_obj_ref`;
- runtime metadata must not hide live heap handles in untraced Rust-side tables.

## 3.7 Current ordinary dispatch

Ordinary `.` invocation compiles through `Bytecode::Invoke` / pack invocation. The VM performs inline-cache lookup, exact method lookup, rest lookup, hierarchy traversal and eventually `doesNotUnderstand`.

That remains correct for `.` and is explicitly **not** the runtime algorithm for `::`.

## 3.8 Current legacy family machinery

The VM currently contains:

```rust
FamilyObject
MethodFamilyObject
BoundMethodFamilyObject
BoundMethodObject
Bytecode::MakeFamily
```

The legacy family path stores a receiver plus exact/pattern specification and uses runtime behavioral lookup. It is useful as representation precedent, but it cannot be the semantic implementation of new static associated families.

Part 4 reuses two safe pieces:

- `BoundMethodObject` as a compact exact behavioral callable representation once the exact method handle has already been bound;
- existing pack/rest runtime machinery for argument-shape extraction, after restricting it to a frozen descriptor.

It does **not** reuse legacy dynamic family discovery.

## 3.9 Current whole-program semantic-to-codegen gap

`ProgramAnalyzer` returns:

```rust
AnalyzedProgram {
    semantic: Arc<SemanticSnapshot>,
    ...
}
```

But `ProgramCompiler::compile_analyzed` currently:

1. rejects snapshots containing errors;
2. creates `CompiledModule` values with an empty `ModuleMaterializationPlan`;
3. exports limited semantic metadata;
4. does not retain expression-level semantic resolution for code generation.

Later, `VM::compile_program_module_closure` recompiles the retained source through:

```rust
compile_closure_as_with_bindings(...)
```

with linked bindings but no formal expression-resolution input.

This is a material architecture gap. Part 4 must add a compact immutable lowering projection to `CompiledModule` and pass it into the source compiler. The solution must be a projection from formal semantic analysis, not a second resolver.

---

# 4. Requirements Analysis

Part 4 must satisfy four independent requirements simultaneously.

## 4.1 Semantic fidelity

The runtime must preserve these distinctions:

```text
@variant None
@variant None()
@variant None(_)

VariantId
VariantConstructorId
CallableId
AssociatedFamilyId

ExactCase
runtime variant descriptor
physical discriminant
```

Representations may share machinery, but no identity may be collapsed simply because two values happen to need zero payload words.

## 4.2 Dynamic-language compatibility

Phalcom remains a dynamic language. Static proof enables optimized/direct execution, but lack of proof at a genuinely dynamic boundary does not make runtime execution impossible by definition.

The implementation therefore needs:

- precise static direct target lowering when proof exists;
- callable/family values that survive storage and `Object`/Dynamic erasure;
- runtime argument-shape routing for frozen family capabilities when shape is not statically known;
- normal runtime errors for genuinely dynamic call-shape failures;
- no runtime theorem solver.

## 4.3 Existing VM compatibility

The first implementation must fit:

```text
16-byte Value
non-moving handle heap
class-based ordinary dispatch
existing MethodObject / ClosureObject / BoundMethodObject
PackBuilder / rest-lane runtime call support
module materialization + on-demand source compilation
```

Part 4 should not rewrite the entire VM or introduce a second object system.

## 4.4 Future AOT/native compatibility

The semantic/runtime contract must admit a future backend where:

```text
Option<Int>
```

is physically:

```text
u8 tag + aligned payload
```

or niche-optimized, stack-resident, scalar-replaced, or monomorphized internally.

Therefore bytecode-VM heap allocation is an implementation strategy, not part of `ExactCase` semantics.

---

# 5. Goals

Part 4 is complete when the implementation architecture can:

1. lower a source enum declaration to runtime enum/case metadata and executable case behavior;
2. construct and execute singleton, zero-argument, and payload variants without conflating them;
3. execute ordinary enum-root and case-specific `.` behavior through the existing message model;
4. lower direct associated behavioral calls to exact pre-resolved targets without family materialization;
5. lower direct variant calls to exact construction without family materialization;
6. reify exact behavioral members and exact variant constructors as callable values;
7. reify whole associated families as frozen capability values;
8. invoke stored family values using `FamilyApplicationResolution` rather than re-running associated resolution;
9. route dynamic argument packs only through the statically frozen candidate set;
10. preserve capture-time visibility as a capability boundary;
11. retain compact runtime descriptors suitable for future reflection;
12. define physical discriminants without confusing them with semantic identities;
13. erase GADT proof evidence at runtime;
14. provide case-test and payload-projection runtime primitives for Part 5;
15. keep current `Value` width unchanged;
16. make GC tracing/root ownership explicit;
17. keep generic semantic schemas independent from runtime erasure strategy;
18. produce backend-neutral target information suitable for a later native compiler.

---

# 6. Non-Goals

Part 4 does not implement:

- match syntax or pattern resolution;
- exhaustiveness/redundancy checking;
- branch-local GADT equality introduction;
- source syntax for `Case<...>`;
- source syntax for `Family<#{...}>`;
- `forall` / universal first-class types;
- monomorphized native layout generation;
- niche optimization in the bytecode VM;
- stack allocation/escape analysis;
- public ABI-stable discriminant annotations;
- `@repr(C)` or FFI layouts;
- core `Option` migration;
- final reflection API spelling;
- final removal/deprecation of legacy `>>`/method-family behavior;
- LSP presentation completion.

Those remain Parts 5–6 or future optimization/ABI work.

---

# 7. Global Runtime Invariants

## I-RT-1 — Semantic IDs never become physical tags

```text
VariantId != RuntimeVariantId != CaseDiscriminant
```

`VariantTypeId` is a `TypeStore` handle and must never be stored in a runtime `Value`.

## I-RT-2 — Exact cases are not source classes

`TypeData::ExactCase` remains a semantic type. A hidden runtime class may model ordinary behavior/reflection, but it does not create a source `DeclarationId` and does not change semantic subtype construction.

## I-RT-3 — Direct associated calls do not materialize families

Known direct target:

```phalcom
Option::Some(42)
System::print("x")
```

must lower directly.

## I-RT-4 — Runtime never re-resolves `::`

The runtime may bind an exact pre-resolved symbolic target to a current method/variant handle. It must not:

```text
look up family base on owner
walk inheritance to discover members
try method then variant
re-run visibility
fall back to dNU
```

## I-RT-5 — Frozen family capability

A family value exposes exactly the capture-authorized runtime descriptor. Runtime dynamic routing may only choose entries named by the `FamilyApplicationResolution` candidate set or, after explicit Dynamic erasure, the captured descriptor itself.

## I-RT-6 — Constructor freshness is preserved

Bare singleton acquisition may be canonical. Constructor invocation is an event. In the current interpreter every `VariantShape::Constructor` invocation creates a fresh case object, including zero-argument constructors.

## I-RT-7 — Payloads are immutable case data

Variant payload slots are initialized during construction and are not ordinary mutable instance fields.

## I-RT-8 — GADT proofs are static

No `CaseTypeEnvironment`, type equality proof node, `TypeId`, or inference variable is stored in an `AdtCaseObject`.

## I-RT-9 — Generic specialization is not runtime identity

`Option<Int>::Some(1)` and `Option<String>::Some("x")` use the same runtime `Option::Some(_)` descriptor in the bytecode VM.

## I-RT-10 — `.` remains normal dispatch

Once an ADT value exists, ordinary:

```phalcom
value.method(...)
```

uses the same method dispatch machinery as other values.

---

# 8. Runtime Identity Model

## 8.1 Runtime IDs

Add compact VM-local IDs:

```rust
#[repr(transparent)]
pub struct RuntimeEnumId(u32);

#[repr(transparent)]
pub struct RuntimeVariantId(u32);

#[repr(transparent)]
pub struct CaseDiscriminant(u32);
```

Properties:

- `RuntimeEnumId` and `RuntimeVariantId` are valid only in one materialized VM/program world;
- they are not serialized as source identity;
- they are cheap payloads for `Value`/heap objects;
- the registry maps stable semantic IDs to them during materialization/execution linkage.

## 8.2 Physical discriminants

Each enum receives a dense per-enum discriminant assignment in declaration order:

```text
0, 1, 2, ...
```

The discriminant is stored in the runtime variant descriptor and may be used by a future switch/jump-table optimizer.

Normative rule:

> A physical discriminant is an implementation detail of one compiled artifact/runtime layout, not the language-level identity of a variant.

Part 6 reflection must not expose it as stable semantic identity. Future explicit ABI/discriminant annotations require a separate specification.

## 8.3 Determinism

For one compiled artifact, discriminant mapping must be deterministic from the enum declaration product. Source-range changes do not affect it. Reordering variants may change physical discriminants and therefore the runtime-layout fingerprint.

---

# 9. Runtime Enum and Variant Descriptors

Introduce an internal registry, recommended in `phalcom-core/src/adt.rs`:

```rust
pub struct RuntimeAdtRegistry {
    enums: Vec<RuntimeEnumDescriptor>,
    variants: Vec<RuntimeVariantDescriptor>,
    enum_by_declaration: HashMap<DeclarationId, RuntimeEnumId>,
    variant_by_semantic_id: HashMap<VariantId, RuntimeVariantId>,
    variant_by_behavior_class: HashMap<ClassId, RuntimeVariantId>,
}
```

Conceptual descriptors:

```rust
pub struct RuntimeEnumDescriptor {
    pub semantic_owner: DeclarationId,
    pub runtime_id: RuntimeEnumId,
    pub root_class: ClassId,
    pub variants: Box<[RuntimeVariantId]>,
}

pub struct RuntimeVariantDescriptor {
    pub semantic_id: VariantId,
    pub runtime_id: RuntimeVariantId,
    pub enum_id: RuntimeEnumId,
    pub discriminant: CaseDiscriminant,
    pub shape: RuntimeVariantShape,
    pub payload_arity: u16,
    pub behavior_class: ClassId,
    pub singleton: Option<Value>,
}

pub enum RuntimeVariantShape {
    Singleton,
    Constructor,
}
```

The exact implementation may split stable semantic keys into an artifact table and leave only runtime IDs in the live descriptor. The required invariant is that per-value storage contains compact runtime identity, not `VariantId`/`TypeId` graphs.

---

# 10. Enum Root and Exact-Case Runtime Class Model

## 10.1 Enum root

Every source enum declaration creates one normal runtime class object for ordinary behavior and class identity.

Example:

```phalcom
enum Shape {
    describe -> String { ... }
    @variant Circle(_ radius: Float)
}
```

Runtime:

```text
Shape root class
    superclass = Object
    instance methods = shared/default enum behavior
    class-side methods = enum root behavioral associated methods
```

The enum root is not directly instantiable as a generic empty instance. Construction happens through variants.

## 10.2 Hidden per-variant behavior class

Each `VariantId` receives one hidden final runtime behavior class:

```text
Shape::<Circle(_) runtime behavior>
    superclass = Shape root class
    own instance methods = Circle case-local behavior
```

This class:

- is not inserted as a source/module declaration;
- has no semantic `DeclarationId` of its own;
- is not used by the type checker to represent `ExactCase`;
- is never an associated variant constructor method;
- exists to make ordinary `.` dispatch and runtime `class`/`is-a` relationships efficient and compatible with the existing VM.

## 10.3 Why hidden behavior classes are preferable to tag-special dispatch

Alternative: teach every ordinary method send to inspect ADT tags and manually merge case/root behavior tables.

Rejected because it would fork the object dispatch engine and make every future optimization understand a second behavioral inheritance system.

The hidden class model lets current ordinary dispatch remain:

```text
case behavior class
    -> enum root class
    -> Object
```

while static semantic exact-case identity remains independent.

---

# 11. Physical Case Representation

## 11.1 Bare singleton

For:

```phalcom
@variant None
```

use an allocation-free immediate representation in the bytecode VM.

Add a `ValueTag::AdtSingleton` (exact name may follow current tag naming convention). Its payload stores `RuntimeVariantId`.

Conceptually:

```text
Value {
    tag = AdtSingleton,
    payload = runtime_variant_id,
}
```

`Value::class(vm)` resolves the runtime variant descriptor and returns its hidden behavior class.

Properties:

- repeated `Option::None` yields identical canonical bits;
- no heap allocation;
- no payload storage;
- `===` naturally observes canonical singleton identity;
- GC has no edge for the immediate itself; the ADT registry roots its behavior class.

## 11.2 Explicit zero-argument constructor

For:

```phalcom
@variant None()
```

each invocation creates a fresh `AdtCaseObject`:

```rust
pub struct AdtCaseObject {
    pub variant: RuntimeVariantId,
    pub payload: Box<[Value]>,
}
```

with an empty payload array.

The two cases are therefore distinguishable even when both carry zero user fields:

```phalcom
Example::None === Example::None     # canonical singleton
Example::None() === Example::None() # false in the current identity-observable VM
```

This is the safest interpretation of “fresh semantic case construction per invocation” given the current runtime's representation-identity operator.

A future optimizer may eliminate or coalesce allocation only when it proves identity/reflection cannot observe the change.

## 11.3 Payload constructor

For:

```phalcom
@variant Some(_ value: T)
```

construction allocates one `AdtCaseObject` with payload values in declaration order.

No ordinary `InstanceObject` field dictionary/header is needed beyond the heap object's own object tag. The runtime class is derived from `variant -> behavior_class`.

## 11.4 Payload immutability

`GetField(slot)` may be generalized to read `AdtCaseObject.payload[slot]` for compiler-generated case-method field access.

`SetField` must not mutate ADT payload. A source-visible attempted write that reaches this path must produce a dedicated immutable-case-data error; internal compiler-generated writes are forbidden after construction.

---

# 12. GC and Memory Safety

## 12.1 `AdtCaseObject`

`heap/trace.rs` must trace every payload `Value` through `Value::gc_obj_ref`.

The `RuntimeVariantId` itself is an integer and not a heap edge.

## 12.2 Hidden classes and registry

`RuntimeAdtRegistry` contains `ClassId` handles for enum roots/hidden case classes. Because those classes may not be reachable through module globals, they are VM roots.

`VM::collect_roots` must explicitly enumerate ADT registry class handles.

## 12.3 Family values

A runtime associated-family capture may retain:

- a bound lookup-owner class value;
- a descriptor heap handle if descriptors live in the heap;
- exact bound method handles if the chosen representation stores them directly.

Every such edge must be covered by exhaustive tracing.

## 12.4 Allocation safepoint discipline

Variant construction follows the existing rule that `Heap::alloc` latches collection rather than collecting mid-opcode. No special ADT allocator may trigger collection while payload values exist only in Rust locals.

---

# 13. Generic ADT Runtime Representation

## 13.1 Bytecode VM policy: erased generic layout

The initial VM uses one runtime enum descriptor and one runtime variant descriptor per source declaration/case.

It does not create:

```text
Option<Int> runtime class
Option<String> runtime class
Some<Int> runtime case class
Some<String> runtime case class
```

Generic arguments are static typing information.

## 13.2 Consequence for payload layout

Every payload slot is a normal 16-byte `Value`, so a generic payload needs no runtime layout specialization.

Recursive ADTs are naturally safe because recursive values pass through `Value`; heap-referential cases do not require an infinitely recursive inline Rust struct.

## 13.3 GADT specialization

`Expr::Int(1)` and `Expr::Bool(true)` store only runtime case identity and payload. They do not store `T = Int` or `T = Bool` proof objects.

## 13.4 Future native backend

A native compiler may choose:

- erased boxed generic layout;
- specialized concrete layout;
- hybrid specialization;
- dictionary-driven code;
- niche encoding;
- scalar replacement.

This is a backend policy. It must preserve the same semantic `VariantId`/`ExactCase` meaning.

---

# 14. Boxing and Dynamic Erasure

## 14.1 Normative rule

> ADT values are not semantically boxed. Boxing/materialization belongs to a physical representation boundary.

## 14.2 Current bytecode VM

The bytecode VM already represents all program values through the erased 16-byte `Value`. Therefore:

```phalcom
const x: Option<Int> = Option::Some(10)
const y: Object = x
```

requires no **second** wrapper allocation. `x` is already represented by a `Value` containing an `AdtCaseObject` handle.

The initial VM allocation is a consequence of the interpreter's universal representation and observable constructor identity, not a property of `Option<Int>` in the language type system.

## 14.3 Future native/AOT boundary

A native backend may keep:

```text
Option<Int> = { tag, inline Int payload }
```

in registers/stack. Conversion to `Object`/Dynamic, reflection, address-taking, capture, or identity observation may materialize a boxed/runtime-object representation.

That transition is the place where boxing cost belongs.

---

# 15. Runtime Method Behavior on Cases

## 15.1 Root behavior

Enum-root bodyful instance behavior installs on the root enum class.

## 15.2 Case override/addition

Case-local methods install on the hidden behavior class associated with `VariantId`.

## 15.3 Dispatch

Ordinary:

```phalcom
shape.area()
```

remains an ordinary send. The receiver's runtime class is the hidden case behavior class, so existing class-hierarchy dispatch finds:

```text
case-local override
or enum-root shared/default implementation
or Object behavior
```

No tag-switch is inserted into every ordinary method call.

## 15.4 Case payload field access

Inside a case method, the compiler already knows the semantic `VariantFieldId`/field index from Part 2. It lowers to fixed payload-slot access. The VM does not resolve field names dynamically.

---

# 16. Semantic-to-Codegen Handoff

This is a required Part-4 architecture layer.

## 16.1 Why `SemanticSnapshot` itself should not be threaded into the VM compiler

The source compiler currently runs on demand inside `VM`, mutating the heap while creating closures/constants. Retaining the entire semantic snapshot inside the runtime would:

- keep large type/explanation graphs alive solely for code generation;
- entangle runtime execution with static query objects;
- encourage backend code to inspect `TypeData` and re-derive semantics;
- make AOT extraction harder.

Instead, `ProgramCompiler::compile_analyzed` must project the formal snapshot into a compact, immutable, backend-facing lowering product.

## 16.2 Recommended product

Create a core-side module such as:

```text
phalcom-core/src/modules/semantic_lowering.rs
```

with:

```rust
pub struct ModuleLoweringSemantics {
    pub module: ModuleId,
    pub enums: Box<[EnumLoweringSpec]>,
    pub associated: LoweringSiteIndex<AssociatedLoweringSpec>,
    pub family_applications: LoweringSiteIndex<FamilyApplicationLoweringSpec>,
}

pub struct EnumLoweringSpec {
    pub owner: DeclarationId,
    pub variants: Box<[VariantLoweringSpec]>,
}

pub struct VariantLoweringSpec {
    pub id: VariantId,
    pub shape: VariantShape,
    pub payload_fields: Box<[VariantFieldLoweringSpec]>,
}

pub struct VariantFieldLoweringSpec {
    pub id: VariantFieldId,
    pub local_name: Box<str>,
    pub slot: u16,
}
```

`payload_fields` is projected from Part-2 `VariantFieldSemantic` in declaration order. It is the canonical immutable field-to-slot layout used by code generation; case-body lowering does not infer payload layout from assignments or rebuild it from source.

The product is built **only** from formal semantic products.

It may retain stable semantic IDs such as `VariantId`, `CallableId`, and `InvocationTargetId`; it must not retain solver variables or require the VM to inspect `TypeStore` for target discovery.

## 16.3 Site key

The current AST intentionally contains no semantic IDs. The checker already records source ranges for expression analyses.

For v1, the lowering site key is:

```text
module + source_id/source file + exact SourceRange + expression category
```

Building the index validates uniqueness. A missing or duplicate formal resolution at an associated/family call site is an internal compiler error.

This is an attachment lookup, not semantic re-resolution.

## 16.4 Compiler API

`CompiledModule` gains:

```rust
pub lowering: Arc<ModuleLoweringSemantics>
```

`VM::compile_program_module_closure` passes that product to a new compiler entry point, conceptually:

```rust
compile_closure_as_with_bindings_and_semantics(...)
```

`Compiler` stores an optional read-only lowering view.

Raw REPL/compiler entry points that do not run the semantic pipeline may continue to compile ordinary code. Encountering `enum`/new `::` without lowering semantics produces a structured compiler integration error rather than reconstructing the meaning from AST.

---

# 17. Backend Target Specs

Compiler bytecode must not embed `VariantTypeId` or raw semantic arena indices.

The lowering layer converts Part-3 resolutions into exact symbolic executable target specs.

Conceptually:

```rust
pub enum ExecutableInvocationTarget {
    Behavioral {
        lookup_owner: DeclarationId,
        callable: CallableId,
        operation: FamilyOperationShape,
        rest_mode: ExecutableRestMode,
    },
    VariantConstructor {
        variant: VariantId,
    },
}
```

For a behavioral target, `lookup_owner` is separate from the defining `CallableId` because inherited associated behavior executes the defining method with the lookup-owner class object as receiver. `operation` preserves the already-selected incoming call shape. `rest_mode` is the lane-preserving lowering of Part-3 canonical rest metadata (`None`, `Positional`, `Labeled`, `Complete`) and is used only to bind/call the already-selected target; it never causes runtime family ranking.

Runtime binding of this exact key to a method handle is address linkage, not semantic family lookup. The VM performs an exact defining-owner/selector lookup in either the exact-method or rest-method table according to `rest_mode`; it does not walk the hierarchy, choose a family member, or invoke dNU.

---

# 18. Direct Associated Invocation

## 18.1 Variant constructor

Source:

```phalcom
Option::Some(42)
```

Part 3 provides:

```text
InvocationTargetId::VariantConstructor(...)
```

Part 4 lowers directly to variant construction.

No family object exists on the runtime path.

## 18.2 Behavioral target

Source:

```phalcom
System::print("x")
```

Part 3 provides a defining `CallableId`, lookup owner and specialized call result.

Part 4 emits an exact associated behavioral invocation opcode/target reference. VM binding:

1. map defining declaration to exact runtime class/metaclass;
2. read the exact preselected method/rest slot;
3. bind receiver to the lookup-owner class object;
4. call that exact method;
5. do not walk hierarchy;
6. do not try another selector;
7. do not invoke dNU.

Each `ExecutableInvocationTarget` has a parallel cache entry:

```rust
#[derive(Debug, Clone, Copy)]
pub struct AssociatedTargetCache {
    pub receiver: Value,
    pub method: ObjRef,
    pub world_version: u64,
}
```

A hit is valid only when `world_version` matches. The cache is non-owning: the lookup-owner class is retained by `VM::classes` or `RuntimeAdtRegistry`, and the defining class retains the method object. On invalidation, binding repeats only the exact declaration/selector address lookup; it never repeats semantic family selection.

---

# 19. Exact Associated Callable Reification

## 19.1 Exact behavioral member

For:

```phalcom
const p = System::print::(_)
```

resolve the exact behavioral target, then reuse the existing:

```rust
BoundMethodObject {
    method,
    receiver,
}
```

This is safe representation reuse because the semantic target has already been selected. The `BoundMethodObject` performs no family discovery.

## 19.2 Exact variant constructor

For:

```phalcom
const ctor = Option<Int>::Some::(_)
```

the runtime value must remain callable without pretending the target is a method.

The preferred v1 representation is a tiny compiler-generated closure/thunk whose body executes exact variant construction.

Benefits:

- no `VariantConstructorId -> CallableId` collapse;
- no new public runtime callable class is required in Part 4;
- ordinary closure calling/erasure works;
- reification pays allocation only when the abstraction is actually created;
- future Option-B universal schemes can reuse the semantic constructor schema while choosing another runtime thunk strategy.

Immediate call of a syntactically exact constructor reference may be optimized to direct construction without allocating the thunk.

---

# 20. First-Class Associated Family Runtime Representation

## 20.1 Two-layer representation

A family has two runtime layers:

```text
immutable executable family descriptor
    shared operation table / target specs

associated family capture object
    descriptor reference
    optional bound lookup-owner receiver/context
```

The first implementation uses these concrete structures:

```rust
pub struct ExecutableFamilyDescriptor {
    pub entries: Box<[ExecutableFamilyEntry]>,
}

pub struct ExecutableFamilyEntry {
    pub operation: FamilyOperationShape,
    pub member_kind: FamilyMemberTypeKind,
    pub target: ExecutableFamilyTarget,
}

pub enum ExecutableFamilyTarget {
    Singleton { variant: VariantId },
    Behavioral { target: ExecutableInvocationTarget },
    VariantConstructor { variant: VariantId },
}

pub struct AssociatedFamilyObject {
    pub descriptor: Arc<ExecutableFamilyDescriptor>,
    pub bound_owner: Option<Value>,
}
```

`Chunk::executable_semantics.family_descriptors` owns the original `Arc`; `MakeAssociatedFamily` clones it into the capture object. Descriptor entries contain symbolic semantic/executable IDs only, never heap handles, so the descriptor itself adds no GC edges. `bound_owner` is the only family-capture `Value` edge and is traced normally. Exact runtime method handles may be cached separately with normal invalidation, but are not semantic family membership.

## 20.2 Capability freezing

The descriptor is generated from the exact Part-3 capture:

```text
access-filtered
lookup-owner-specialized
exact member set
```

Two captures of the same nominal `AssociatedFamilyId` under different visibility contexts may therefore use different runtime descriptors.

No runtime code can append members discovered from the owner.

## 20.3 Structural abstraction

A physical family object may possess a wider capability set than a function's static structural family parameter exposes.

A statically analyzed call must use the operation/candidate set from `FamilyApplicationResolution`, not opportunistically route through every physical entry.

If nominal/captured denotation was lost, `target = None` is legitimate. Runtime invocation uses the actual family object's operation table and the statically authorized operation key/candidate mask. It does not reconstruct nominal provenance.

## 20.4 Capture identity

Part 4 does not declare family reification a canonical singleton operation. Each explicit reification may allocate a small capture object. The immutable descriptor/table is shared.

This avoids accidentally specifying `===` identity for family values while keeping the large operation table allocation-free per capture.

---

# 21. Family Invocation

## 21.1 Static known shape

Given:

```phalcom
const f = Option<Int>::Some::*
const x = f(1)
```

Part 3 publishes a static `FamilyApplicationResolution` operation.

Lowering:

```text
evaluate f once
push arguments
InvokeAssociatedFamilyStatic(operation-id, argc)
```

The runtime:

1. verifies the value is an associated-family capability in the typed path;
2. indexes/finds the exact operation in its immutable descriptor;
3. invokes that target directly.

It does not query `Option` or associated surfaces.

## 21.2 Dynamic pack

Given:

```phalcom
f(*args)
```

Part 3 freezes candidates.

The bytecode carries a candidate-set descriptor/mask. Runtime:

```text
runtime argument shape
    ↓
exact operation among frozen candidates?
    yes -> choose it
    no
    ↓
compatible rest candidate among frozen candidates
    ↓
invoke exact target
```

No other entry is eligible even if the physical family object contains it.

## 21.3 Rest lanes

Runtime shape compatibility mirrors the canonical lane-aware semantics established in Part 3:

```text
positional rest
labeled rest
complete/split rest
```

The VM may reuse existing `Signature`/`RestLayout` utilities for shape tests. It must not use legacy family hierarchy search as evidence.

## 21.4 Dynamic-erased call

When an associated family is intentionally erased to Dynamic/Object and invoked through a genuinely dynamic call path, the family object's **captured descriptor** is the authority. The runtime may route among that captured set by call shape.

This preserves dynamic usefulness without restoring lexical access or owner lookup.

---

# 22. Bytecode Additions

The first bytecode implementation uses these concrete opcode shapes. Fixed-shape argument counts follow the existing `Invoke(u8, u16)` / `SuperSend(u8, ...)` convention; dynamic packs stay on the pack path.

```rust
// enum/runtime materialization
Bytecode::Enum(u16)
Bytecode::VariantMethod { variant: u16, selector: u16 }
Bytecode::FinalizeEnum(u16)

// case values
Bytecode::LoadVariantSingleton(u16)
Bytecode::ConstructVariant { variant: u16, arity: u8 }

// exact associated behavior
Bytecode::MakeResolvedBoundMethod(u16)
Bytecode::InvokeResolvedAssociated { target: u16, arity: u8 }

// first-class family
Bytecode::MakeAssociatedFamily(u16)
Bytecode::InvokeAssociatedFamilyStatic { operation: u16, arity: u8 }
Bytecode::InvokeAssociatedFamilyPack { candidates: u16 }

// Part-5 runtime primitives, not match semantics
Bytecode::IsVariant(u16)
Bytecode::GetVariantPayload(u16)
```

Notes:

- `Enum` indexes an `ExecutableEnumSpec`, creates/registers the enum root and hidden case behavior classes, and leaves the root class value on the stack for root-member installation.
- `VariantMethod` uses the same compiled-method stack discipline as existing `Method`, but installs the method on the hidden class for the indexed `VariantId`; it never publishes a variant as a metaclass method.
- `FinalizeEnum` finalizes root/hidden behavior tables after all enum and case methods are installed.
- `LoadVariantSingleton`/`ConstructVariant` consume symbolic variant references from the executable semantic pool, not raw VM-local IDs baked before materialization.
- `MakeAssociatedFamily` is **not** legacy `MakeFamily`.
- `InvokeResolvedAssociated` binds and invokes the exact formal target and bypasses ordinary dispatch and dNU.
- Part 5 may later introduce fused branch instructions (`JumpIfNotVariant`) without changing the runtime case contract.

---

# 23. Chunk / Executable Side Tables

`Chunk` currently stores:

```text
code
constants: Vec<Value>
spans
inline caches
global caches
```

Do not encode semantic target specs as arbitrary heap `Value` constants. Add one typed pool directly to `Chunk`:

```rust
pub struct ExecutableSemanticPool {
    pub enum_specs: Vec<Arc<ExecutableEnumSpec>>,
    pub variant_targets: Vec<VariantId>,
    pub associated_targets: Vec<ExecutableInvocationTarget>,
    pub associated_target_caches: Vec<Cell<Option<AssociatedTargetCache>>>,
    pub family_descriptors: Vec<Arc<ExecutableFamilyDescriptor>>,
    pub family_operations: Vec<FamilyOperationShape>,
    pub family_candidate_sets: Vec<ExecutableFamilyCandidateSet>,
}

pub struct Chunk {
    ...
    pub executable_semantics: ExecutableSemanticPool,
}
```

Each insertion API returns a checked `u16` index and reports a compiler integration error rather than truncating if a pool exceeds `u16::MAX` entries.

Requirements:

- bytecode operands remain compact `u16`/`u32` indices;
- side tables contain stable symbolic IDs, not live heap handles prior to runtime binding;
- `associated_target_caches.len() == associated_targets.len()` is invariant; cached handles are non-owning and valid only while `world_version` matches;
- disassembly must render symbolic selector/owner information for debugging.

---

# 24. Enum Declaration Lowering

Create a dedicated compiler module rather than forcing enum source through class expansion.

Conceptual flow:

```text
Statement::Enum
    ↓ lookup EnumLoweringSpec from formal lowering bundle
create enum root runtime class
    ↓
create/register hidden behavior class for each VariantId
    ↓
register RuntimeEnumDescriptor / RuntimeVariantDescriptor
    ↓
compile enum-root shared/class-side behavior
    ↓
compile each variant body method onto its hidden class
    ↓
finalize ordinary behavior tables
```

The compiler must not synthesize ordinary source class declarations for variants.

`@variant` constructors are not installed as class-side methods. Associated construction uses runtime variant descriptors.

---

# 25. Runtime Case Testing Contract for Part 5

Part 4 provides these runtime operations:

```rust
fn runtime_variant_of(value: Value) -> Option<RuntimeVariantId>;
fn value_is_variant(value: Value, expected: RuntimeVariantId) -> bool;
fn case_payload_len(value: Value) -> Option<usize>;
fn case_payload_at(value: Value, index: usize) -> Result<Value, RuntimeError>;
fn case_behavior_class(value: Value) -> Option<ClassId>;
```

They support both:

- immediate singleton values;
- heap `AdtCaseObject`s.

Part 5 may lower:

```text
evaluate scrutinee once
IsVariant
conditional jump
GetVariantPayload
```

No GADT proof object is required at runtime.

---

# 26. Visibility and Capability Boundary

Construction visibility is enforced in Part 3 before lowering.

Part 4 rules:

1. direct inaccessible construction never reaches codegen as a valid target;
2. a captured family descriptor contains only members authorized at capture;
3. exact captured references remain usable later without lexical re-check;
4. Dynamic/Object erasure does not add members;
5. runtime reflection may inspect a captured capability in Part 6 but may not reconstruct an uncaptured private constructor from `AssociatedFamilyId` alone.

The VM is not an access-control resolver for static `::`.

---

# 27. Source and LSP Implications

Part 4 adds no editor-owned semantic authority and no new source-resolution algorithm.

- `phalcom-semantic` Part-3 products remain the source of go-to-definition, hover, completion, exact-case presentation, and associated target identity.
- `ModuleLoweringSemantics`, `ExecutableSemanticPool`, `RuntimeEnumId`, `RuntimeVariantId`, and `CaseDiscriminant` are backend/runtime products and must not appear as LSP semantic identities.
- The source index must continue to attach `VariantId`, `CallableId`, and `AssociatedFamilyId` from formal analysis; it must not inspect runtime descriptors or bytecode to infer a target.
- Hover for a constructed variant continues to present the semantic exact-case/root type chosen by Part 3, not the hidden runtime behavior class.
- Hidden case behavior classes have no source definition target of their own. Navigation on variant syntax goes to the `@variant` declaration; navigation on case-local behavior goes to the source callable declaration already indexed by semantics.
- The range-based lowering-site key is a backend attachment mechanism only. LSP/source-index code must not depend on it as a semantic identity.
- Disassembly may show stable semantic owner/selector names for debugging, but must not expose VM-local runtime IDs as user-facing identity.

Part 6 owns final editor presentation, completion polish, reflection UI, and migration cleanup. Part 4 only preserves the information boundaries required for that work.

---

# 28. Diagnostics and Failure Ownership
## 28.1 Semantic user errors
Missing family/member, inaccessible construction, generic underconstraint, GADT owner conflict, static family call mismatch and similar errors belong to `phalcom-semantic` and must be diagnosed before Part-4 lowering.

## 28.2 Compiler integration errors
Add structured compiler errors for impossible handoff failures, e.g.:

```text
MissingEnumLoweringSemantics
MissingAssociatedResolution
MissingFamilyApplicationResolution
AmbiguousLoweringSiteAttachment
UnmaterializedExecutableTarget
```

These are compiler/internal integration failures, not alternative user-facing type diagnostics.

## 28.3 Runtime dynamic errors
Legitimate dynamic failures include:

```text
no captured family operation matches runtime pack shape
runtime value is not an associated-family capability at a Dynamic call boundary
wrong runtime arity for a dynamically invoked exact constructor thunk
```

They must mention the frozen/captured family rather than claiming ordinary message lookup failed.

---

# 29. Incremental and Fingerprint Implications
Part 4 introduces no second semantic query graph.

## 29.1 Semantic dependencies
Part 3 remains responsible for recording dependencies on:

- enum declaration products;
- associated surfaces;
- callable signatures;
- hierarchy edges/generic supertype templates;
- family application resolution inputs.

## 29.2 Lowering projection fingerprints
The compact `ModuleLoweringSemantics` should have a deterministic fingerprint derived from semantic products relevant to execution:

```text
enum variant identities/order/shapes/payload arity
behavioral target identities
captured family member sets
static family operations
dynamic candidate sets
```

Exclude:

```text
source ranges except as attachment keys
type explanation prose
GADT proof graph history
hover presentation
advisory facts
```

## 29.3 Runtime layout fingerprint
`ModuleMaterializationPlan` currently uses a placeholder zero fingerprint in runtime registration. Part 4 should introduce a real plan/layout fingerprint covering enum runtime descriptors and declaration blueprints.

Body-only method changes with unchanged enum/family signature should not change enum layout identity, although the code chunk itself changes.

---

# 30. Performance Contract
## 30.1 Construction
Bare singleton:

```text
0 heap allocations
O(1) descriptor-id value creation
```

Zero-arg/payload constructor in bytecode VM:

```text
1 case-object allocation
O(payload count) payload copy
O(1) variant descriptor lookup/cache
```

## 30.2 Direct associated call
Must not allocate a family object.

Behavioral target:

```text
exact symbolic target bind/cache
exact method activation
no hierarchy search
no dNU
```

Variant target:

```text
direct construction
```

## 30.3 Exact reference reification
Behavioral:

```text
one BoundMethod allocation when value actually escapes/reifies
```

Variant constructor:

```text
one tiny closure/thunk allocation when reified
```

Immediate application may elide both.

## 30.4 Family capture
```text
one small capture object
shared immutable operation descriptor/table
```

No per-capture hash map rebuild.

## 30.5 Family invocation
Static operation:

```text
O(1) indexed/sorted-table operation selection
```

Dynamic pack:

```text
O(k) worst case over frozen candidates in v1
```

where `k` is the candidate count, not the class/owner method count. A future descriptor may precompute exact-shape hash indexes without semantic changes.

---

# 31. AOT / Native Compilation Contract
A future backend should be able to consume the same backend-facing lowering semantics and implement:

```text
ConstructVariant
LoadVariantSingleton
InvokeResolvedAssociated
MakeAssociatedFamily
InvokeAssociatedFamily[Static|Dynamic]
IsVariant
GetVariantPayload
```

without asking the type checker which variant/member/family was meant.

Native representation may differ radically:

```text
register tag
stack union
niche encoding
specialized payload struct
boxed erased object
```

The semantic target/proof pipeline remains unchanged.

This makes Part 4 a runtime abstraction contract rather than a VM-only language definition.

---

# 32. Reflection Handoff to Part 6
Part 4 records enough runtime metadata for Part 6 to expose reflection safely:

```text
enum declaration identity mapping
variant selector
variant shape
physical discriminant (internal)
payload arity/order
hidden behavior class
family operation descriptors
```

Part 4 does **not** define the public reflection API.

Part 6 must preserve:

```text
semantic VariantId
!= physical CaseDiscriminant
```

and must not expose `RuntimeVariantId`, `VariantTypeId`, slotmap handles, or type-store arena indices as stable language identity.

---

# 33. Interaction With Existing `Option` / `Some` / `None`
The current VM already has special `Value` option-state representation and core `Some`/`None` behavior. Part 4 does not migrate those built-ins.

Rules:

- newly declared source enums use the new ADT runtime path;
- current core option representation remains compatible during staging;
- Part 6 decides whether core `Option` is migrated onto the general enum machinery or kept as a representation-specialized builtin with the same semantic surface;
- no new ADT rule may depend on the special existing `Option` encoding.

---

# 34. Part 4 / Part 5 Boundary
Part 4 owns:

```text
case construction
runtime case identity
case tag/discriminant descriptors
payload storage
payload extraction primitive
case-test primitive
case behavior runtime class
```

Part 5 owns:

```text
pattern identity resolution
reachable case universe
exhaustiveness
redundancy
branch refinement
GADT equality introduction
branch-local proof environment
match result join
match lowering sequence
```

Part 5 must not redesign ADT storage. Part 4 must not decide exhaustiveness.

---

# 35. Testable Acceptance Criteria
Part 4 is accepted when implementation tests can establish all of the following.

## 35.1 Runtime identity/layout
- `@variant None` is a canonical singleton runtime value;
- singleton acquisition performs no heap case allocation;
- `@variant None()` produces a fresh runtime case object each invocation;
- `@variant None(_)` stores payload values in declaration order;
- singleton, zero-arg constructor and payload constructor have distinct runtime variant descriptors;
- physical discriminant is deterministic per compiled artifact;
- no `VariantTypeId` appears in runtime object/value storage.

## 35.2 Behavior
- enum-root shared behavior works on every case;
- case-local override shadows root behavior through ordinary `.` dispatch;
- case-added behavior is available only on the appropriate runtime case class;
- payload reads in case behavior use immutable payload slots;
- payload writes are rejected.

## 35.3 Direct associated lowering
- direct variant invocation produces no `MakeAssociatedFamily` bytecode;
- direct behavioral associated invocation produces no ordinary `Invoke` family lookup/dNU path;
- inherited behavioral associated target invokes the defining method with lookup-owner class receiver;
- exact rest target remains exact and does not search unrelated rest families.

## 35.4 Exact reification
- exact behavioral ref produces a callable bound to the exact method/lookup-owner receiver;
- exact variant constructor ref is callable and still semantically/runtimely distinct from a method;
- immediate exact-ref application may bypass reification without changing result.

## 35.5 Family capability
- family capture contains only Part-3-authorized entries;
- inaccessible entries never appear at runtime;
- two different access-filtered captures of the same nominal family remain distinct descriptors/capabilities;
- static family call invokes the Part-3-selected operation;
- structural family call with `target=None` does not re-resolve nominal family identity;
- dynamic pack routes only among its frozen candidate set;
- exact shape wins before compatible rest;
- dynamic-erased family call remains confined to the captured descriptor;
- no dNU path is reachable from new associated-family invocation.

## 35.6 GC
- case payload objects survive GC when reachable only through an ADT value;
- objects referenced by case payload are traced;
- hidden case classes survive GC;
- family bound owner/descriptor/target handles survive GC;
- unreachable case/family objects are collectable;
- singleton immediate values add no spurious heap edge.

## 35.7 Semantic handoff
- compiler lowering fails internally if a new associated AST node lacks a formal Part-3 resolution;
- compiler never calls an associated semantic resolver;
- `CompiledModule` carries the lowering projection needed by on-demand source compilation;
- VM never receives a `TypeStore` requirement for target discovery;
- GADT proof/equality objects are absent from case runtime storage.

## 35.8 Part-5 primitive contract
- `runtime_variant_of` works for singleton and constructor values;
- `IsVariant` distinguishes every exact variant including `None` vs `None()`;
- `GetVariantPayload` extracts constructor fields and rejects singleton/non-case misuse predictably.

---

# 36. Architecture-Negative Checks
The implementation is invalid if any of these become true:

```text
compiler associated lowering calls semantic family resolution from AST names
VM associated invocation walks a class hierarchy to discover the target
new associated family invocation calls doesNotUnderstand
VariantConstructorId is converted into CallableId
@variant None() is cached as the singleton
all ADT values are forced through an extra DynamicObject wrapper
ExactCase is materialized as a source DeclarationId
GADT equality TypeIds are stored on each case object
runtime family capture re-checks lexical visibility on later calls
runtime dynamic pack can discover a member outside the frozen candidate set
new associated syntax lowers through legacy Bytecode::MakeFamily
```

Recommended repository checks are listed in the implementation plan.

---

# 37. Design Decisions and Rejected Alternatives
## D04-1 — Immediate bare singletons

**Decision:** use a new compact immediate singleton-case `Value` tag in the bytecode VM.

**Rejected:** allocate one heap object for every bare singleton case. Correct but unnecessarily expensive and contrary to the existing tagged-value architecture.

## D04-2 — Fresh zero-argument constructor objects

**Decision:** every explicit constructor invocation is fresh in the current VM, even with zero payload fields.

**Rejected:** reuse singleton bits/object for `None()` because current identity is observable and the source semantics intentionally distinguish construction from singleton acquisition.

## D04-3 — Hidden runtime behavior class per variant

**Decision:** use hidden final runtime classes solely for ordinary behavior/reflection relationships.

**Rejected:** synthesize source declarations/exact-case nominal classes in `phalcom-semantic`.

**Rejected:** implement a second tag-aware ordinary method dispatcher.

## D04-4 — Erased generic runtime descriptors

**Decision:** one runtime descriptor per declaration/case in the VM.

**Rejected:** create runtime class/descriptor per semantic generic specialization. That would conflate typing applications with runtime identity and make future `forall` unnecessarily expensive.

## D04-5 — Compact lowering projection

**Decision:** project `SemanticSnapshot` into per-module lowering semantics retained by `CompiledModule`.

**Rejected:** thread the full semantic DB/snapshot into the VM source compiler.

**Rejected:** have `phalcom-core` inspect names/types and re-run associated resolution.

## D04-6 — Exact behavior binding, not ordinary dispatch

**Decision:** bind a Part-3-selected `CallableId` directly to the defining runtime method slot and use the lookup-owner class object as receiver.

**Rejected:** emit normal `Invoke` and trust runtime method lookup to find the same method.

## D04-7 — Reuse `BoundMethodObject` only after resolution

**Decision:** reuse exact bound-method representation for exact behavioral refs.

**Rejected:** reuse legacy family search semantics.

## D04-8 — Variant constructor reference as closure thunk

**Decision:** represent reified variant constructors as small compiler-generated callables in v1.

**Rejected:** install variants as class-side methods.

## D04-9 — New frozen associated-family runtime object

**Decision:** introduce a separate internal associated-family object/descriptor path, even if it shares the existing surface `Family` runtime class temporarily.

**Rejected:** use `FamilyObject { receiver, selector/pattern }`, because that object intentionally discovers behavior dynamically.

## D04-10 — Shared descriptor + small capture object

**Decision:** operation tables are shared; explicit family reification creates a small capture.

**Rejected:** rebuild a hash map on every capture.

## D04-11 — Dynamic pack is capability-local routing

**Decision:** runtime pack shape selects only from the candidate set frozen by Part 3.

**Rejected:** runtime owner lookup / hierarchy search / dNU.

## D04-12 — No second boxing on Object conversion in VM

**Decision:** current VM values are already erased `Value`s; conversion to `Object` does not wrap an `AdtCaseObject` again.

**Rejected:** define ADTs as inherently boxed because the interpreter uses heap cases.

## D04-13 — GADT proofs erased

**Decision:** proof/equality state is compile-time only.

**Rejected:** attach theorem metadata to each runtime case.

## D04-14 — Physical discriminants are unstable implementation data

**Decision:** deterministic within an artifact, not public ABI.

**Rejected:** expose `u32` tag as `VariantId` or stable reflection identity.

---

# 38. Explicit Deferred Work
The following are deliberate future work, not gaps in the Part-4 runtime contract:

- niche/null-pointer optimization;
- AOT/native layout engine;
- stack allocation and scalar replacement;
- specialized generic layouts;
- stable FFI discriminants/layout annotations;
- public exact-case reflection type descriptors;
- public associated-family reflection API;
- public constructor-reference runtime class naming;
- core `Option` migration;
- final legacy Family/MethodFamily deprecation/removal;
- `forall` runtime/type-scheme implementation;
- match decision-tree optimization beyond Part-4 case primitives.

---

# 39. Part 4 Completion Boundary
The Part-4 implementation is complete only when the runtime can execute the meanings Part 3 has already resolved, without asking semantic questions again.

The final pipeline must be:

```text
SOURCE
    ↓
Part 1 AST
    ↓
Part 2 declaration semantics
    ↓
Part 3 associated/family resolution
    ↓
Part 4 ModuleLoweringSemantics projection
    ↓
Part 4 bytecode/executable target specs
    ↓
Part 4 runtime ADT/family descriptors
    ↓
EXECUTION
```

And the handoff to Part 5 must be exactly:

```text
runtime_variant_of(value)
value_is_variant(value, variant)
case_payload_at(value, index)
physical case descriptor/discriminant
```

with exhaustiveness, pattern applicability and GADT proof refinement still wholly owned by `phalcom-semantic`.
