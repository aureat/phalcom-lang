# Phalcom ADT/GADT + Associated Lookup
## Part 04 — Runtime Representation, Execution Model, Family Reification, and Lowering
### Repository-Grounded Implementation Plan

> **For agentic workers:** use the repository's normal TDD/review workflow. For behavioral/runtime changes, write the focused failing test first. Before claiming completion, run the full verification commands in Task 24 and report exact output/failures.

**Goal:** Replace the explicit enum/associated lowering staging guards with execution driven exclusively by Part-2/Part-3 semantic products. Add an efficient ADT runtime representation, exact associated-target lowering, frozen first-class family values, dynamic-pack routing confined to frozen candidates, and the runtime case primitives Part 5 will consume.

**Verified planning repository:** `aureat/phalcom-lang`  
**Verified branch:** `feat/adts`  
**Verified HEAD:** `f453a26a0e4e2c97640aeb3ac3cc212d8fb102f0`  
**Commit subject:** `feat: implement ADT semantic support`  
**Spec:** `docs/impl/adt-gadt-associated-lookup/part-4/04-runtime-representation-execution-model-family-reification-lowering-technical-spec.md`

---

# 0. Preflight — Reconcile the Actual Local Tree Before Editing

Part 3 is **not implemented in the connected committed branch** used to write this plan. The first implementation task is therefore a hard prerequisite check, not optional archaeology.

Run:

```bash
git status --short
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git log -1 --oneline
```

Required branch intent:

```text
feat/adts or a descendant implementation branch
```

Do not reset/stash/discard uncommitted Part-3 work.

Inspect:

```bash
rg -n 'InvocationTargetId|AssociatedResolution|FamilyApplicationResolution|AssociatedValueDenotation|TypeData::Family' phalcom-semantic/src
rg -n 'AssociatedLookupNotLoweredYet|AssociatedInvokeNotLoweredYet|EnumNotLoweredYet' phalcom-core/src
rg -n 'RestMode|UnsupportedRestShape|FamilyApplicationCandidate' phalcom-semantic/src
```

Record a name map from the Part-3 plan to actual implementation names.

## Required Part-3 prerequisites

Do **not** begin runtime semantics until the current tree provides structural equivalents of:

```text
InvocationTargetId::{Behavioral, VariantConstructor}
AssociatedResolution
FamilyApplicationResolution
Captured associated exact/family denotation
FamilyOperationShape / structural family type
frozen Dynamic candidate list
visibility-filtered family capture
lane-preserving exact-before-rest selection
Option-A concrete/contextual reification finalization
```

If any is absent, finish it under Part 3 rather than adding a replacement resolver to `phalcom-core`.

## Prerequisite semantic tests

Use the actual Part-3 test modules. At minimum:

```bash
cargo test -p phalcom-semantic --test semantic associated
cargo test -p phalcom-semantic --test semantic gadt
cargo test -p phalcom-semantic --test semantic fingerprints
```

If filter names differ, enumerate the test target and record the exact replacements.

---

# 1. Current File Map and Ownership

Verified current runtime/compiler seams:

```text
phalcom-core/src/value/repr.rs
    16-byte tagged Value

phalcom-core/src/value/mod.rs
    class/type/render/equality behavior

phalcom-core/src/heap/object.rs
    Object enum; legacy Family/MethodFamily/BoundMethodFamily/BoundMethod

phalcom-core/src/heap/trace.rs
    exhaustive GC edge tracing

phalcom-core/src/heap/mod.rs
    object arena + typed accessors

phalcom-core/src/vm/mod.rs
    VM state and runtime roots

phalcom-core/src/vm/gc.rs
    exhaustive VM root enumeration

phalcom-core/src/vm/dispatch.rs
    bytecode loop and normal Invoke/dNU behavior

phalcom-core/src/vm/send.rs
    legacy family invocation helpers / pack activation

phalcom-core/src/primitive/family.rs
    runtime Family surface primitives

phalcom-core/src/bytecode.rs
    Bytecode + legacy MakeFamily

phalcom-core/src/chunk.rs
    code/constants/spans/IC side tables

phalcom-core/src/compiler/lib/mod.rs
    Compiler state and compile entry points

phalcom-core/src/compiler/lib/expr.rs
    expression lowering; associated staging guards

phalcom-core/src/compiler/lib/class_decl.rs
    runtime class/method lowering patterns to reuse

phalcom-core/src/compiler/lib/error.rs
    Enum/Associated staging errors

phalcom-core/src/modules/compile.rs
    AnalyzedProgram -> CompiledProgram; currently discards lowering semantics

phalcom-core/src/modules/artifact.rs
    declaration/materialization blueprints

phalcom-core/src/modules/materialize.rs
    VM materialization and on-demand source compilation

phalcom-semantic/src/snapshot.rs
    immutable formal snapshot

phalcom-semantic/src/enum_semantics.rs
    Part-2 enum/variant declaration products

phalcom-semantic/src/associated.rs
    Part-2 associated family surfaces
```

---

# 2. Target New Files

Create these focused files. If Part 3 lands a file with one of these exact responsibilities before implementation begins, reconcile by moving the Part-4 declarations into that landed module rather than creating duplicate ownership; record the mechanical path change in the implementation report.

```text
phalcom-core/src/adt.rs
    RuntimeEnumId / RuntimeVariantId / CaseDiscriminant
    RuntimeAdtRegistry
    runtime enum/variant descriptors

phalcom-core/src/heap/adt.rs
    AdtCaseObject

phalcom-core/src/heap/associated.rs
    AssociatedFamilyObject
    executable family descriptor/capture payloads if heap-owned

phalcom-core/src/compiler/lib/enum_decl.rs
    Statement::Enum lowering
    root/case behavior method emission

phalcom-core/src/compiler/lib/associated.rs
    AssociatedLookup/AssociatedInvoke lowering
    exact target/family lowering

phalcom-core/src/modules/semantic_lowering.rs
    SemanticSnapshot -> compact ModuleLoweringSemantics projection

phalcom-core/src/vm/adt.rs
    runtime variant registration, construction, case testing, payload access

phalcom-core/src/vm/associated.rs
    exact associated target binding
    frozen family invocation/routing

phalcom-core/tests/adt_runtime.rs
phalcom-core/tests/associated_lowering.rs
phalcom-core/tests/associated_family_runtime.rs
```

These paths match the current `phalcom-core` organization (`heap/*`, `vm/*`, `compiler/lib/*`, `modules/*`). Do not merge semantic resolver code into `vm/dispatch.rs` or another broad file merely to avoid creating the focused modules above.

---

# 3. Task 1 — Build the Formal Semantic-to-Codegen Projection

**Create:**

```text
phalcom-core/src/modules/semantic_lowering.rs
```

**Modify:**

```text
phalcom-core/src/modules/mod.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/artifact.rs
phalcom-core/src/modules/materialize.rs
phalcom-semantic/src/snapshot.rs        # expose the Part-3 formal resolution indexes to the projection builder
```

## 3.1 Write failing projection tests first

Add unit/integration tests proving:

```text
- a valid AssociatedInvoke source site has exactly one lowering record;
- a valid AssociatedLookup exact ref has exactly one record;
- a stored family call has exactly one FamilyApplicationResolution record;
- a module enum declaration yields an EnumLoweringSpec;
- missing/duplicate site attachment is rejected as an internal projection error;
- source-range movement changes attachment keys but not range-free target fingerprint.
```

## 3.2 Add `ModuleLoweringSemantics`

Implement this compact immutable product:

```rust
pub struct ModuleLoweringSemantics {
    pub module: ModuleId,
    pub enums: Box<[EnumLoweringSpec]>,
    pub associated: BTreeMap<LoweringSite, AssociatedLoweringSpec>,
    pub family_applications: BTreeMap<LoweringSite, FamilyApplicationLoweringSpec>,
}
```

`LoweringSite` includes:

```text
module/source identity
SourceRange
associated/family application category
```

Do not key by source spelling/name alone.

## 3.3 Preserve stable target identity

The projection may carry:

```text
DeclarationId
VariantId
VariantConstructorId
CallableId
InvocationTargetId
FamilyOperationShape
```

It must not carry:

```text
solver metavariables
mutable inference session state
GADT proof graph nodes required only for typing
diagnostic prose
advisory ValueShape
```

## 3.4 Project enum declarations

`EnumLoweringSpec` should include at minimum:

```rust
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

Populate `payload_fields` from Part-2 `VariantFieldSemantic` in declaration order and assign `slot` from its stable `VariantFieldId`/index. This is the canonical compile-time payload layout used by case-body field lowering. Case/root behavioral callables remain source bodies paired with canonical callable identities; do not copy full type facts into runtime specs.

## 3.5 Store projection on `CompiledModule`

Change:

```rust
pub struct CompiledModule {
    ...
    pub lowering: Arc<ModuleLoweringSemantics>,
}
```

`ProgramCompiler::compile_analyzed` must build this from `analyzed.semantic` before discarding the snapshot.

## 3.6 Pass projection into on-demand compiler

In `VM::compile_program_module_closure`, replace the semantic-blind call with this entry point:

```rust
compile_closure_as_with_bindings_and_semantics(
    module,
    source,
    UnitKind::File,
    bindings,
    compiled_module.lowering.clone(),
)
```

Add to `Compiler`:

```rust
lowering: Option<Arc<ModuleLoweringSemantics>>
```

Clone the `Arc` at compiler construction boundaries; do not borrow the full `SemanticSnapshot` through the VM compiler.

## 3.7 Raw compiler path

Existing direct/REPL compiler APIs may pass `None` for ordinary code. New enum/associated AST encountered without formal lowering semantics must return a structured compiler integration error. Do not re-run associated resolution.

**Focused commands:**

```bash
cargo test -p phalcom-core semantic_lowering
cargo check -p phalcom-core
```

---

# 4. Task 2 — Add Runtime ADT IDs and Registry

**Create:**

```text
phalcom-core/src/adt.rs
```

**Modify:**

```text
phalcom-core/src/lib.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/gc.rs
```

Add:

```rust
#[repr(transparent)]
pub struct RuntimeEnumId(u32);

#[repr(transparent)]
pub struct RuntimeVariantId(u32);

#[repr(transparent)]
pub struct CaseDiscriminant(u32);
```

Add descriptors:

```rust
RuntimeEnumDescriptor
RuntimeVariantDescriptor
RuntimeVariantShape
RuntimeAdtRegistry
```

Required indexes:

```text
DeclarationId -> RuntimeEnumId
VariantId -> RuntimeVariantId
RuntimeVariantId -> descriptor
ClassId -> RuntimeVariantId (reverse behavior-class mapping)
```

Add `VM` field:

```rust
adt_registry: RuntimeAdtRegistry
```

Update the exhaustive `VM::collect_roots` destructure immediately when the field is added.

Registry roots every root/hidden case class handle it owns.

Tests:

- registration deterministic;
- duplicate semantic variant registration rejected/internal;
- one enum assigns dense discriminants in declaration order;
- different enums each start their local discriminants independently;
- runtime IDs differ from semantic/type-store IDs.

---

# 5. Task 3 — Extend Module Runtime Blueprints for Enums

**Modify:**

```text
phalcom-core/src/modules/artifact.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/materialize.rs
```

Extend:

```rust
RuntimeDeclarationBlueprint
```

with this dedicated enum blueprint:

```rust
RuntimeDeclarationBlueprint::Enum(EnumBlueprint)
```

`EnumBlueprint` contains stable declaration/variant shape information from `ModuleLoweringSemantics`, not AST guesses.

Materialization Phase 3 must reserve the enum's global binding exactly as it does for classes/globals.

Do not create variant globals.

Add a real `ModulePlanFingerprint` input for enum blueprint structure instead of continuing to treat the plan as fingerprint `0` once runtime ADT layout depends on it.

Tests:

```text
- enum global slot is declared during materialization;
- no variant is separately declared as module global;
- changing variant order/shape changes runtime-plan fingerprint;
- changing only a case method body does not change enum-layout fingerprint.
```

---

# 6. Task 4 — Add `AdtCaseObject` and Immediate Singleton Value Tag

**Create:**

```text
phalcom-core/src/heap/adt.rs
```

**Modify:**

```text
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/value/repr.rs
phalcom-core/src/value/mod.rs
```

## 6.1 Heap case

Add:

```rust
pub struct AdtCaseObject {
    pub variant: RuntimeVariantId,
    pub payload: Box<[Value]>,
}
```

Add:

```rust
Object::AdtCase(Box<AdtCaseObject>)
```

Add typed heap accessors following `InstanceObject`/`ClosureObject` conventions.

## 6.2 Immediate singleton

In `phalcom-core/src/value/repr.rs`, add `ValueTag::AdtSingleton = 8` and update every exhaustive tag operation in the same file: `ValueTag::from_u8`, `PartialEq`, `Hash`, and any debug/render match introduced by the implementation.

Add constructors/accessors:

```rust
Value::adt_singleton(RuntimeVariantId) -> Value
Value::adt_singleton_id(self) -> Option<RuntimeVariantId>
```

The payload word stores `RuntimeVariantId.0`; `gc_obj_ref` continues to return `None` for this tag. Preserve the existing 16-byte size static assertion/tests and add a round-trip tag/payload test.

## 6.3 Value surface integration

In `phalcom-core/src/value/mod.rs` update the current exhaustive surface seams:

```text
Value::type_name
    AdtSingleton -> "case"
    Object::AdtCase -> still enters the ordinary object branch

Value::class
    AdtSingleton(runtime_variant)
        -> VM.adt_registry.variant(runtime_variant).behavior_class
    Object::AdtCase(case)
        -> VM.adt_registry.variant(case.variant).behavior_class

Value::to_context
    Object::AdtCase -> CallContext::Instance
    AdtSingleton -> CallContext::Immediate

Value::value_eq
    AdtSingleton -> compare RuntimeVariantId
    Object::AdtCase -> retain ordinary ObjRef identity unless user behavior dispatch has already selected another equality method
```

When `Object::AssociatedFamily` lands in Task 12, add it to `Value::class` and `Value::to_context` in the same exhaustive audit.

## 6.4 Equality/identity

Do not add content equality that collapses constructor cases.

`===`/same-bits behavior should establish:

```text
same singleton variant -> identical
separate constructor allocations -> non-identical
```

Normal `==` remains governed by ordinary language equality/message semantics; do not invent structural ADT equality in Part 4.

## 6.5 Tracing

In `heap/trace.rs`:

```rust
Object::AdtCase(case) => {
    for value in &case.payload {
        trace_value(*value, push);
    }
}
```

The runtime variant ID is not a heap edge.

**Focused tests:**

```bash
cargo test -p phalcom-core --test adt_runtime singleton
cargo test -p phalcom-core --test adt_runtime constructor_identity
cargo test -p phalcom-core --test gc adt
```

---

# 7. Task 5 — Create Enum Root and Hidden Case Behavior Classes

**Create:**

```text
phalcom-core/src/vm/adt.rs
phalcom-core/src/compiler/lib/enum_decl.rs
```

**Modify:**

```text
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/api.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/class_decl.rs   # shared helper extraction only
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/bytecode.rs
```

## 7.1 Extract reusable class creation/method helpers

Do not clone the entire class compiler. Extract the minimum helpers for:

```text
emit class/root creation
compile behavioral member closure
install method on known runtime class target
finalize base names
```

Class source semantics remain separate from enum source semantics.

## 7.2 Add the concrete enum bytecode/runtime path

Add these bytecodes:

```rust
Bytecode::Enum(u16)
Bytecode::VariantMethod { variant: u16, selector: u16 }
Bytecode::FinalizeEnum(u16)
```

`Enum(u16)` indexes an `ExecutableEnumSpec` in the chunk executable semantic pool and:

1. creates one root class with superclass `Object`;
2. stores it in the enum module binding;
3. creates one hidden final behavior class per `VariantLoweringSpec`;
4. registers root/variants with `RuntimeAdtRegistry`;
5. initializes the immediate singleton value in the descriptor for `VariantShape::Singleton`;
6. pushes the enum root class value so existing root `Method` installation discipline can be reused.

`VariantMethod` consumes the same compiled method object shape expected by existing `Method`, but installs it on the exact hidden case behavior class from the variant target table. `FinalizeEnum` finalizes root and hidden behavior tables after member installation.

Do not create source-visible globals for hidden case classes.

## 7.3 Prevent direct enum-root instantiation

Guard `NewInstance`/constructor paths so a raw enum root cannot be instantiated as an ordinary class instance.

Use the ADT registry/root-class reverse mapping. Emit a clear runtime/internal error if a dynamic path tries.

## 7.4 Compile behavior

Root behavior:

```text
instance member -> enum root class
@class member   -> enum root metaclass
```

Case behavior:

```text
instance member -> hidden behavior class for VariantId
```

Do not install variants as metaclass methods.

Tests:

```text
- enum root exists as module class binding;
- hidden case class not visible as module binding;
- singleton/case `.class` resolves to hidden behavior class;
- hidden class superclass is enum root;
- root behavior dispatch works;
- case override/addition works.
```

---

# 8. Task 6 — Make Case Payload Fields Work in Case Methods

**Modify:**

```text
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/error.rs
```

Extend `Bytecode::GetField(slot)` runtime handling:

```text
InstanceObject -> existing behavior
ClassObject    -> existing static slots
AdtCaseObject -> payload[slot]
```

Immediate singleton has zero payload slots; attempting field access is an internal mismatch unless future reflection deliberately exposes something.

`SetField` on `AdtCaseObject` must fail with a dedicated immutable case-payload error.

Tests:

```text
- case method can read positional payload field;
- case method can read labeled payload field by compiled slot;
- payload object reference survives GC;
- write is rejected;
- root method cannot fabricate a payload slot absent from semantic field proof.
```

---

# 9. Task 7 — Add Variant Construction and Singleton Load Bytecodes

**Modify:**

```text
phalcom-core/src/bytecode.rs
phalcom-core/src/chunk.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/adt.rs
phalcom-core/bin/phalcom/disasm.rs
```

## 9.1 Executable variant reference side table

Add a typed `ExecutableSemanticPool` field directly to `Chunk`. Do not store `VariantId` as a heap `Value` constant.

Add `ExecutableSemanticPool` to `Chunk` and expose typed insertion/accessors:

```rust
fn add_enum_spec(&mut self, spec: Arc<ExecutableEnumSpec>) -> u16;
fn enum_spec(&self, index: u16) -> &ExecutableEnumSpec;
fn add_variant_target(&mut self, variant: VariantId) -> u16;
fn variant_target(&self, index: u16) -> &VariantId;
fn add_associated_target(&mut self, target: ExecutableInvocationTarget) -> u16;
fn associated_target(&self, index: u16) -> &ExecutableInvocationTarget;
```

Define in `phalcom-core/src/chunk.rs`:

```rust
#[derive(Debug, Clone, Default)]
pub struct ExecutableSemanticPool {
    pub enum_specs: Vec<Arc<ExecutableEnumSpec>>,
    pub variant_targets: Vec<VariantId>,
    pub associated_targets: Vec<ExecutableInvocationTarget>,
    pub family_descriptors: Vec<Arc<ExecutableFamilyDescriptor>>,
    pub family_operations: Vec<FamilyOperationShape>,
    pub family_candidate_sets: Vec<ExecutableFamilyCandidateSet>,
}
```

Add `pub executable_semantics: ExecutableSemanticPool` to `Chunk`. Every `add_*` helper checks the `u16` index limit before insertion and returns `CompilerError::ExecutableSemanticPoolOverflow { kind }` on overflow.

## 9.2 Bytecodes

Add exactly:

```rust
Bytecode::LoadVariantSingleton(u16)
Bytecode::ConstructVariant { variant: u16, arity: u8 }
```

Use the existing fixed-shape call convention: `Bytecode::Invoke` and `SuperSend` carry argument counts as `u8`; `ConstructVariant` and other fixed-shape associated invoke opcodes use `u8` too. Dynamic packs use their existing pack representation instead of widening the fixed-shape arity field.

## 9.3 Runtime singleton load

Resolve semantic `VariantId` through `RuntimeAdtRegistry`; assert descriptor shape is `Singleton`; push canonical immediate singleton `Value`.

## 9.4 Runtime construction

Resolve descriptor; assert constructor shape; remove/copy the exact argument count from the stack; allocate fresh `Object::AdtCase`; push its value.

The constructor opcode itself never runs generic inference or visibility checks.

## 9.5 Tests

Disassembly and runtime tests must distinguish:

```text
Option::None      -> LoadVariantSingleton
Option::None()    -> ConstructVariant arity=0
Option::Some(1)   -> ConstructVariant arity=1
```

---

# 10. Task 8 — Lower Direct Associated Variant Invocation

**Create/modify:**

```text
phalcom-core/src/compiler/lib/associated.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/mod.rs
```

When compiling `Expr::AssociatedInvoke`:

1. look up the exact `AssociatedLoweringSpec` by formal source site;
2. require a static `InvocationTargetId` from Part 3;
3. compile arguments with existing evaluation/pack rules;
4. for `VariantConstructor`, emit `ConstructVariant`;
5. for behavioral target, use Task 10;
6. never call an associated resolver;
7. never emit `MakeAssociatedFamily` for direct invocation.

For a static singleton getter `AssociatedLookup`, emit `LoadVariantSingleton`.

Add architecture test by disassembly asserting no `MakeFamily`/`MakeAssociatedFamily` appears in direct constructor bytecode.

---

# 11. Task 9 — Add Exact Behavioral Target Specs and Binding

**Create:**

```text
phalcom-core/src/vm/associated.rs
```

**Modify:**

```text
phalcom-core/src/chunk.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/bytecode.rs
```

Add these executable target types:

```rust
pub enum ExecutableRestMode {
    None,
    Positional,
    Labeled,
    Complete,
}

pub struct BehavioralAssociatedTargetSpec {
    pub lookup_owner: DeclarationId,
    pub callable: CallableId,
    pub operation: FamilyOperationShape,
    pub rest_mode: ExecutableRestMode,
}
```

The target carries both lookup owner and defining callable owner.

Implement:

```rust
fn bind_behavioral_associated_target(
    &mut self,
    spec: &BehavioralAssociatedTargetSpec,
) -> Result<ResolvedBehavioralAssociatedTarget, RuntimeError>;
```

Binding algorithm:

```text
DeclarationId -> module handle + class symbol
CallableId owner -> defining class/metaclass
lookup owner -> receiver class object
`callable.selector` + `rest_mode` -> exact direct method/rest-method slot
`operation` -> already-selected incoming call shape for rest ABI packing
```

Forbidden operations:

```text
hierarchy walk
family-base search
rest candidate ranking
dNU
visibility check
```

A direct hash access to the defining class's exact method/rest slot is runtime address binding, not semantic resolution.

Add a cache parallel to `Chunk::executable_semantics.associated_targets`:

```rust
#[derive(Debug, Clone, Copy)]
pub struct AssociatedTargetCache {
    pub receiver: Value,
    pub method: ObjRef,
    pub world_version: u64,
}
```

`ExecutableSemanticPool` keeps `associated_target_caches: Vec<Cell<Option<AssociatedTargetCache>>>`, appending one empty cell with every associated target. A cache hit is valid only when `world_version` matches. The cache is non-owning: the lookup-owner class is rooted by `VM::classes`/`RuntimeAdtRegistry`, and the defining class owns the method handle; document this invariant beside the cache.

---

# 12. Task 10 — Add Direct Behavioral Associated Invocation Bytecode

**Modify:**

```text
phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/associated.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/associated.rs
```

Add:

```rust
Bytecode::InvokeResolvedAssociated { target: u16, arity: u8 }
```

VM execution:

1. bind/cache exact target spec;
2. place lookup-owner receiver in the normal method receiver position;
3. activate the exact method object;
4. preserve ordinary method frame semantics/access owner;
5. do not execute the normal `Invoke` lookup path.

Tests:

```text
- local class-side associated method direct call;
- inherited associated method uses Base method body with Derived class receiver;
- nearest override target is exactly the Part-3-selected CallableId;
- no hierarchy lookup occurs at runtime;
- no dNU occurs if runtime owner happens to expose another similarly named method.
```

Prove the negative path with unit tests that call `bind_behavioral_associated_target` against a hierarchy where normal lookup would choose a different override; assert the returned method handle is the defining `CallableId` target. Add disassembly assertions that the call site contains `InvokeResolvedAssociated`, not `Invoke`.

---

# 13. Task 11 — Reify Exact Associated Callables

**Modify:**

```text
phalcom-core/src/compiler/lib/associated.rs
phalcom-core/src/vm/associated.rs
phalcom-core/src/heap/object.rs (only if helper/accessor needed)
```

## 13.1 Behavioral exact refs

For Part-3 exact behavioral resolution:

1. bind exact method handle;
2. allocate existing `BoundMethodObject { method, receiver }`;
3. push it.

Do not invoke the method at acquisition.

## 13.2 Variant constructor refs

Implement a compiler-generated callable thunk path.

Add this helper in `phalcom-core/src/compiler/lib/associated.rs`:

```rust
fn compile_variant_constructor_thunk(
    &mut self,
    variant: &VariantId,
    operation: &FamilyOperationShape,
    range: SourceRange,
) -> Result<ObjRef, CompilerError>
```

The thunk body consists of argument loads plus `ConstructVariant` and return. It is a normal closure runtime value but preserves the semantic constructor identity in lowering metadata.

Do not install a method on the enum metaclass.

## 13.3 Immediate exact-ref call optimization

After correctness tests pass, optionally recognize:

```phalcom
(Option<Int>::Some::(_))(1)
```

when Part-3 resolution proves the exact constructor and emit direct construction.

The optimization must be syntactic/codegen only and must not alter semantic analysis.

---

# 14. Task 12 — Introduce Frozen Associated Family Descriptor and Capture Object

**Create:**

```text
phalcom-core/src/heap/associated.rs
```

**Modify:**

```text
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/chunk.rs
```

Add an internal object distinct from legacy `FamilyObject`:

```rust
Object::AssociatedFamily(Box<AssociatedFamilyObject>)
```

Add these concrete structures:

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

Store `Arc<ExecutableFamilyDescriptor>` entries in `Chunk::executable_semantics.family_descriptors`. `MakeAssociatedFamily` clones that `Arc` into the heap capture. Descriptor entries contain no `ObjRef`; only `bound_owner` is a GC edge. The descriptor is built from Part-3 captured denotation, not from `AssociatedFamilyId` alone.

`Value::class` may map this internal object to the current runtime `Family` class as a staging surface; Part 6 decides final public reflection/migration. The object payload must stay distinct so legacy lookup semantics cannot accidentally run.

Tracing:

- trace `bound_owner` through `Value::gc_obj_ref` when present;
- do not trace the `Arc<ExecutableFamilyDescriptor>` because it contains only Rust-owned immutable symbolic IDs and no `ObjRef`/`Value` handles.

---

# 15. Task 13 — Lower Whole-Family Reification

**Modify:**

```text
phalcom-core/src/compiler/lib/associated.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/associated.rs
phalcom-core/bin/phalcom/disasm.rs
```

Add:

```rust
MakeAssociatedFamily(descriptor_ref)
```

Compiler:

1. obtain `AssociatedResolution::Family` (actual Part-3 name);
2. convert the captured member list into immutable executable descriptor entries;
3. preserve lookup owner for behavioral family binding;
4. emit `MakeAssociatedFamily`;
5. never emit legacy `MakeFamily`.

Runtime:

- allocate one small capture object;
- do not rediscover family members;
- for a behavioral family, resolve and store the lookup-owner class value in `bound_owner` at capture time; variant families store `None`;
- intern `ExecutableFamilyDescriptor` values by exact ordered entry equality inside `ExecutableSemanticPool` so captures with identical authorized operation/target sets clone the same `Arc`; different access-filtered sets never deduplicate.

Tests:

```text
- variant family with singleton + zeroarg + unary constructor;
- behavioral family with inherited exact members;
- capture with private member filtered out;
- two lexical access contexts produce different descriptors when member set differs;
- `MakeFamily` is absent from disassembly.
```

---

# 16. Task 14 — Static Invocation of a Stored Family

**Modify:**

```text
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/associated.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/associated.rs
```

Add:

```rust
Bytecode::InvokeAssociatedFamilyStatic { operation: u16, arity: u8 }
```

When ordinary call compilation sees a Part-3 `FamilyApplicationResolution::Static` at the call site:

1. compile callee exactly once;
2. compile arguments exactly once;
3. emit the operation-specific family opcode;
4. runtime selects only the requested operation from the capture descriptor;
5. if Part 3 recorded `target = Some`, the compiler may use it to validate/optimize, but the family value remains the capability authority after storage/abstraction;
6. if `target = None`, do not attempt nominal reconstruction.

Tests must include structural abstraction where target provenance is intentionally absent but operation invocation succeeds through the runtime family object.

---

# 17. Task 15 — Dynamic-Pack Family Invocation Over Frozen Candidates

**Modify:**

```text
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/associated.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/associated.rs
phalcom-core/src/vm/send.rs      # extract/reuse shape utilities only
```

Add executable candidate-set descriptors from Part 3:

```rust
pub struct ExecutableFamilyCandidateSet {
    pub candidates: Box<[ExecutableFamilyCandidate]>,
}
```

Add:

```rust
InvokeAssociatedFamilyPack { candidates: u16 }
```

Runtime algorithm:

```text
read PackBuilder/runtime call shape
filter/consult ONLY encoded candidate entries
exact operation match first
then compatible rest in semantic candidate order
invoke selected exact target
```

Reuse existing `RestLayout::accepts`/pack extraction if compatible.

Do **not** call:

```text
lookup_method_in_hierarchy
lookup_rest_method on owner/class surface
activate_family_with_kind legacy owner discovery
forward_does_not_understand
```

Dynamic failure when no frozen candidate accepts the pack gets a dedicated associated-family runtime error.

Add negative test where the runtime owner has an additional callable of a matching shape that was **not** captured; dynamic pack must still fail rather than discover it.

---

# 18. Task 16 — Dynamic/Object-Erased Family Call Fallback

**Modify:**

```text
phalcom-core/src/primitive/family.rs
phalcom-core/src/vm/associated.rs
```

Static typed family calls must use Tasks 14–15 and avoid ordinary `.call` dispatch.

For a family deliberately erased to Dynamic/Object and later invoked dynamically, extend the existing runtime `Family` surface primitive to recognize `Object::AssociatedFamily` and route only within its captured descriptor.

Keep legacy `Object::Family` behavior unchanged until Part 6 migration.

Tests:

```text
- precise family -> Object -> dynamic call still works;
- private/uncaptured operation does not become visible after erasure;
- legacy Family behavior remains unchanged;
- no owner hierarchy lookup is used for AssociatedFamily.
```

---

# 19. Task 17 — Add Part-5 Runtime Case Primitives

**Modify:**

```text
phalcom-core/src/vm/adt.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/bin/phalcom/disasm.rs
```

Implement API helpers:

```rust
runtime_variant_of(Value) -> Option<RuntimeVariantId>
value_is_variant(Value, RuntimeVariantId) -> bool
case_payload_at(Value, usize) -> Result<Value, RuntimeError>
```

Support both immediate singleton and heap case values.

Add general bytecodes:

```rust
IsVariant(variant_ref)
GetVariantPayload(index)
```

Do not add `match` AST or exhaustiveness logic.

Tests:

```text
- singleton matches only its exact VariantId;
- None and None() are distinct;
- payload constructor test + projection;
- non-ADT value returns false for IsVariant;
- invalid payload extraction produces internal/runtime misuse error;
- no theorem/type metadata needed.
```

---

# 20. Task 18 — Remove Part-4 Staging Guards Only When Their Paths Are Real

**Modify:**

```text
phalcom-core/src/compiler/lib/error.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/mod.rs
```

Replace:

```text
EnumNotLoweredYet
AssociatedLookupNotLoweredYet
AssociatedInvokeNotLoweredYet
```

with real lowering.

Do not remove a guard before its corresponding integration test passes.

Add internal errors for missing formal handoff:

```rust
MissingEnumLoweringSemantics
MissingAssociatedResolution
MissingFamilyApplicationResolution
AmbiguousLoweringSiteAttachment
```

These errors should indicate a compiler/semantic pipeline defect, not retry another resolver.

---

# 21. Task 19 — Complete GC Root and Trace Audit

**Modify:**

```text
phalcom-core/src/heap/trace.rs
phalcom-core/src/vm/gc.rs
phalcom-core/tests/gc.rs
phalcom-core/tests/f2_pack_gc.rs        # only if shared pack machinery changed
```

Run exhaustive object/root review after all runtime fields are final.

Required GC scenarios:

1. `AdtCaseObject` payload contains the only reference to a heap string/list; payload child survives.
2. case becomes unreachable; case + unique payload child collect.
3. hidden behavior class has no module global; registry root keeps it alive.
4. associated family bound owner survives through capture.
5. associated family descriptor/target handles survive if heap-held.
6. associated family/captured child collects when unreachable.
7. singleton immediate introduces no false heap reference.
8. GC during/re-entering dynamic family call follows existing temp-root discipline.

Run:

```bash
cargo test -p phalcom-core --test gc
cargo test -p phalcom-core --test f2_pack_gc
```

---

# 22. Task 20 — Disassembly and Debuggability

**Modify:**

```text
phalcom-core/bin/phalcom/disasm.rs
phalcom-core/tests/disasm_golden.rs
```

Render new opcodes with stable symbolic information where possible:

```text
LOAD_VARIANT_SINGLETON Option::None
CONSTRUCT_VARIANT Option::Some(_) argc=1
INVOKE_ASSOC System::print(_) lookup=System define=System
MAKE_ASSOC_FAMILY Derived::build::* entries=3
INVOKE_ASSOC_FAMILY op=method(_)
IS_VARIANT Expr::Int(_)
GET_VARIANT_PAYLOAD 0
```

Do not print VM-local `RuntimeVariantId` as if it were semantic identity.

Golden tests should make direct-call non-reification visually obvious.

---

# 23. Task 21 — Incremental / Fingerprint Integration

**Modify:**

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/artifact.rs
phalcom-core/src/modules/registry.rs
phalcom-core/src/modules/compile.rs
phalcom-semantic/src/db/fingerprint.rs      # only if Part 3 has not completed resolution fingerprints
```

Add deterministic fingerprints for:

```text
ModuleLoweringSemantics
EnumBlueprint/runtime layout
Executable family descriptor member set
Dynamic candidate set
```

Tests:

```text
- changing case method body only: enum layout fingerprint stable;
- changing variant selector: layout + dependent target projection changes;
- changing payload arity: layout changes;
- changing GADT equality with same physical variant shape: semantic dependent code invalidates, but raw runtime descriptor layout need not change;
- changing associated visibility/member set: family capture descriptor fingerprint changes;
- changing unrelated method body with unchanged signature does not change captured family membership fingerprint;
- source-range-only movement does not alter range-free runtime target fingerprint.
```

Do not introduce a VM semantic query graph. Codegen consumes frozen products.

---

# 24. Task 22 — Runtime and Compiler Architecture-Negative Tests

Add automated searches/assertions where possible.

## 24.1 Compiler associated module must not resolve semantics

```bash
! rg -n 'resolve_associated|effective_associated_family|AssociatedSurface.*lookup|resolve_dispatch_target' \
  phalcom-core/src/compiler/lib/associated.rs
```

Allow only target attachment/projection lookup by source site.

## 24.2 New associated lowering must not use legacy `MakeFamily`

```bash
! rg -n 'Bytecode::MakeFamily' phalcom-core/src/compiler/lib/associated.rs
```

Legacy compiler paths elsewhere may remain until Part 6.

## 24.3 New family runtime path must not use dNU

```bash
! rg -n 'forward_does_not_understand|doesNotUnderstand' phalcom-core/src/vm/associated.rs
```

## 24.4 No constructor-to-method collapse

```bash
! rg -n 'VariantConstructorId.*CallableId|CallableId.*VariantConstructorId' phalcom-core phalcom-semantic/src/checker/associated.rs
```

Review legitimate enum declarations/imports manually if search matches type names in the same file.

## 24.5 No runtime theorem storage

```bash
! rg -n 'CaseTypeEnvironment|GenericConstraint|TypeId|VariantTypeId' \
  phalcom-core/src/heap/adt.rs phalcom-core/src/adt.rs
```

`VariantId` may appear in registry/codegen mappings; `VariantTypeId` must not.

---

# 25. Task 23 — Focused Integration Test Matrix

Create/extend:

```text
phalcom-core/tests/adt_runtime.rs
phalcom-core/tests/associated_lowering.rs
phalcom-core/tests/associated_family_runtime.rs
phalcom-core/tests/gc.rs
phalcom-core/tests/disasm_golden.rs
```

## 25.1 ADT values

```phalcom
enum Example {
    @variant None
    @variant None()
    @variant Some(_ value: Int)
}
```

Assert:

```text
None canonical
None() fresh
Some payload preserved
all three different case identities
root type behavior works
case override works
```

## 25.2 Generic erasure

```phalcom
enum Box<T> { @variant Item(_ value: T) }

const a = Box::Item(1)
const b = Box::Item("x")
```

Assert one runtime `VariantId -> RuntimeVariantId` descriptor is used while payload values differ.

Do not assert loss of static exact-case types; semantic tests cover those.

## 25.3 GADT runtime erasure

```phalcom
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
```

Assert runtime storage contains only case + payload; no type index object allocation.

## 25.4 Direct call no reification

Disassemble/run:

```phalcom
Option::Some(42)
System::print("x")
```

Assert no `MAKE_ASSOC_FAMILY` instruction.

## 25.5 Exact refs

```phalcom
const ctor = Option<Int>::Some::(_)
const x = ctor(1)
```

and inherited behavioral exact ref.

## 25.6 Whole family

Test singleton + zeroarg + payload constructor family and inherited behavioral family.

## 25.7 Dynamic pack confinement

Construct a frozen family with candidate A, make runtime owner also expose B, call via a pack matching only B. Result must be frozen-family mismatch, not B invocation.

## 25.8 Visibility capability

Capture family from privileged context, pass value to another function, call captured allowed member successfully. Separately prove member omitted at capture cannot be recovered by later Dynamic call.

---

# 26. Task 24 — Full Verification Before Completion

Run fresh commands after the final implementation diff.

## 26.1 Formatting

```bash
cargo fmt --all -- --check
```

## 26.2 Focused semantic prerequisites

Use actual Part-3 filters, then at minimum:

```bash
cargo test -p phalcom-semantic --test semantic
```

## 26.3 Core focused suites

```bash
cargo test -p phalcom-core --test adt_runtime
cargo test -p phalcom-core --test associated_lowering
cargo test -p phalcom-core --test associated_family_runtime
cargo test -p phalcom-core --test family_selector_runtime
cargo test -p phalcom-core --test gc
cargo test -p phalcom-core --test f2_pack_gc
cargo test -p phalcom-core --test disasm_golden
```

## 26.4 Core full suite

```bash
cargo test -p phalcom-core
```

## 26.5 Dependent crates

```bash
cargo test -p phalcom-lsp
```

No new LSP semantic resolver is expected; this is a regression check.

## 26.6 Workspace

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If a failure predates Part 4:

1. record exact command/output;
2. reproduce on the starting baseline when practical;
3. do not claim the workspace is green unless the fresh command is green.

## 26.7 Architecture searches

Run Task 22 searches after formatting so the report reflects the final tree.

---

# 27. Task 25 — Documentation Consistency Pass

**Create repository docs:**

```text
docs/impl/adt-gadt-associated-lookup/part-4/04-runtime-representation-execution-model-family-reification-lowering-technical-spec.md
docs/impl/adt-gadt-associated-lookup/part-4/04-runtime-representation-execution-model-family-reification-lowering-implementation-plan.md
```

Update only active docs that would otherwise incorrectly claim:

```text
new :: lowers through legacy MakeFamily
enums have no runtime implementation
variant cases are synthetic source classes
```

Do not perform the Part-6 documentation migration here.

---

# 28. Integration Order and Sequencing Dependencies

Use this dependency order:

```text
Part-3 formal resolution complete
        ↓
Task 1 semantic lowering projection
        ↓
Tasks 2–4 runtime IDs/value/object foundations
        ↓
Tasks 5–7 enum root/case classes + construction
        ↓
Tasks 8–10 direct associated invocation
        ↓
Task 11 exact callable reification
        ↓
Tasks 12–13 family descriptor/capture
        ↓
Tasks 14–16 family invocation + Dynamic erasure
        ↓
Task 17 Part-5 case primitives
        ↓
Tasks 18–23 staging removal, fingerprints, tests
        ↓
Tasks 24–25 final verification/docs
```

Do not start dynamic family routing before the Part-3 candidate-set representation is present and tested.

Do not remove staging guards before the corresponding implementation path is covered.

---

# 29. Recommended Commit Sequence

Keep changes reviewable. Recommended sequence:

1. `feat(core): project formal semantics for runtime lowering`
2. `feat(runtime): add ADT runtime identities and descriptors`
3. `feat(value): represent ADT singleton and case values`
4. `feat(gc): trace ADT runtime values and descriptors`
5. `feat(compiler): lower enum roots and case behavior`
6. `feat(runtime): construct and inspect ADT cases`
7. `feat(compiler): lower direct associated variant calls`
8. `feat(runtime): bind exact associated behavioral targets`
9. `feat(compiler): lower direct associated behavioral calls`
10. `feat(compiler): reify exact associated callables`
11. `feat(runtime): add frozen associated family descriptors`
12. `feat(compiler): lower associated family capture`
13. `feat(runtime): invoke static associated family operations`
14. `feat(runtime): route dynamic family packs over frozen candidates`
15. `feat(runtime): support dynamic-erased captured families`
16. `feat(runtime): expose ADT case primitives for match lowering`
17. `test(adts): cover runtime and lowering architecture`
18. `docs(adts): record Part 4 runtime contract`

If Part 3 remains uncommitted in the working tree, keep Part-4 changes logically separable even if the user's workflow does not create each commit immediately.

---

# 30. Explicit Part-3 / Part-4 Ownership Ledger

## Already implemented at verified `f453a26a`

```text
Enum/Variant AST and associated AST
VariantId / VariantConstructorId / CallableOwnerId
ExactCase semantic type
EnumInfo / VariantInfo / constructor signatures
GADT case equality environment
associated declaration surfaces/reservation
case/root semantic behavior model
```

## Required Part-3 prerequisite before runtime work

```text
AssociatedResolution
FamilyApplicationResolution
InvocationTargetId
captured associated denotation
formal family type
owner specialization
visibility-filtered capture
frozen dynamic candidates
canonical rest-lane semantic selection
Option-A reification completion
```

## Part-4-owned

```text
semantic lowering projection into CompiledModule
runtime enum/variant IDs and physical discriminants
immediate singleton representation
fresh constructor case object
hidden case behavior classes
enum/case lowering
exact associated behavioral target binding
variant constructor execution
exact callable runtime reification
frozen associated-family descriptor/capture
family static/dynamic invocation bytecodes
GC integration
Part-5 runtime case primitives
```

## Part-5-owned

```text
match/pattern semantics
exhaustiveness/redundancy
branch refinement
GADT equality introduction in match
match decision tree/lowering use of Part-4 primitives
```

## Part-6-owned

```text
core Option migration decision
public enum/family reflection
LSP completion/hover/definition finalization
legacy Family/MethodFamily migration/deletion
final docs/reference integration
```

---

# 31. Implementation Completion Report Template

The implementing agent's final report must include:

```text
1. starting branch and SHA
2. Part-3 prerequisite SHA/status and name reconciliations
3. final SHA(s), if commits created
4. files created/modified
5. semantic lowering projection design
6. RuntimeEnumId/RuntimeVariantId/discriminant design actually landed
7. singleton representation
8. zeroarg/payload constructor representation
9. enum root/hidden case behavior class implementation
10. generic-erasure behavior
11. exact behavioral target binding implementation
12. exact constructor-ref representation
13. family descriptor/capture representation
14. static family invocation behavior
15. dynamic pack frozen-candidate behavior
16. GC root/trace changes
17. Part-5 runtime primitives
18. incremental/fingerprint changes
19. focused test commands + exact results
20. workspace/clippy commands + exact results
21. architecture-negative search results
22. baseline failures, if any
23. deviations from this plan and rationale
24. Part-5 assumptions discovered/changed
```

The report must explicitly state whether each of these is true:

```text
- no new associated semantic resolver exists in phalcom-core
- new associated syntax does not lower through legacy MakeFamily
- direct known associated calls do not allocate a family object
- runtime dynamic family routing cannot discover uncaptured members
- VariantConstructorId remains distinct from CallableId
- GADT theorem metadata is absent from runtime case values
- no claim of workspace green is made without fresh command evidence
```

---

# 32. Definition of Done

Part 4 is done only when:

1. every valid new enum/associated source form that Part 3 resolves has a runtime lowering path;
2. compiler codegen consumes formal resolution attachments rather than re-deriving meaning;
3. singleton/zeroarg/payload variants preserve their semantic distinctions at runtime;
4. direct known calls bypass family reification;
5. first-class families are immutable frozen capabilities;
6. dynamic pack routing is candidate-confined and exact-before-rest;
7. ADT payloads and family captures are GC-correct;
8. ordinary case behavior uses existing `.` dispatch without a second behavioral resolver;
9. current VM remains 16-byte `Value` based;
10. GADT proofs are erased;
11. Part 5 has stable runtime case-test/payload APIs;
12. all staging errors owned by Part 4 are removed only where real implementations replace them;
13. focused/full verification is reported from fresh commands.
