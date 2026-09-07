# Phalcom Semantic Correctness Program — Technical 04 Implementation Plan
## Receiver, Inheritance, and `Self` Generic Specialization

**Repository:** `aureat/phalcom-lang`  
**Verified planning baseline:** `main` at `9b30ec324d4361128f285154fe236e25746df750`  
**Execution prerequisite:** complete the corrective single-semantic-world LSP retirement, then re-pin the implementation SHA  
**Companion specification:** `phalcom-semantic-correctness-technical-04-receiver-inheritance-self-specialization-spec.md`

## Goal

Implement complete receiver/class generic specialization and inherited `Self` semantics on top of the existing `TypeEnvironment`, declaration metadata, canonical dispatch, Technical 02 application funnel, and Technical 03 inference engine.

## Global constraints

1. Finish the corrective LSP retirement before implementing 04 on `main`.
2. Do not restore or update legacy LSP semantic code to support 04.
3. Do not change canonical callable identity for specializations.
4. Do not mutate declaration surfaces for a call-site receiver.
5. Use `TypeParameterId`, never generic parameter spelling.
6. Reuse `TypeEnvironment`, `TypeView`, `TypeSubstitution`, and existing `SelfTypeTerm`.
7. Specialize declaration parameters before method-local inference.
8. Do not weaken `TypeKnowledge` or proof authority.
9. Every behavioral task is RED → implementation → focused GREEN.
10. Do not unignore unrelated golden scenarios.

## File map

### Create

```text
phalcom-semantic/src/types/specialization.rs

phalcom-semantic/tests/semantic/foundations/receiver_specialization.rs
```

### Modify

```text
phalcom-semantic/src/types/mod.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/tests/semantic/foundations/mod.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
phalcom-semantic/tests/semantic/golden/mod.rs
phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
```

Potentially modify only if source characterization proves necessary:

```text
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/hierarchy_product.rs
```

Do not start by modifying those two. The current architecture already has enough information to keep nominal dispatch and hierarchy-edge identity separate from specialization.

## Execution order

```text
Task 0  single-world retirement gate
   ↓
Task 1  RED source-level receiver-specialization matrix
   ↓
Task 2  canonical ReceiverSpecialization domain
   ↓
Task 3  workspace generic-supertype hierarchy wiring
   ↓
Task 4  owner-relative / multi-hop environment projection
   ↓
Task 5  complete callable-signature specialization
   ↓
Task 6  replace direct-only CheckingContext specialization
   ↓
Task 7  compose receiver specialization with Technical 03
   ↓
Task 8  fields/getters/setters/indexers
   ↓
Task 9  constructor/class-side Self
   ↓
Task 10 super dispatch + nested Self
   ↓
Task 11 incremental dependency/reuse proof
   ↓
Task 12 activate GOLDEN-01
   ↓
Task 13 structural cleanup/audit
   ↓
Task 14 full verification
```

# Task 0 — Close Part 3 before semantic 04 begins

This is mandatory because the verified `main` baseline is still halfway through the retirement cut.

Current `Backend` still owns the old semantic/index ownership shape while `AnalysisService` has already moved toward publication-based construction. Implement the corrective retirement plan rather than restoring the old APIs.

### Step 0.1

Required final architecture:

```text
Backend
  └── AnalysisService
        └── one SemanticPublication
              └── Arc<phalcom_semantic::SemanticSnapshot>

worker
  └── one persistent SemanticWorkspaceSession
```

### Step 0.2

Before 04, verify:

```bash
cargo check -p phalcom-lsp --lib
cargo check -p phalcom-semantic

test ! -d phalcom-lsp/src/semantic

rg -n \
  'SemanticDb|SemanticEngine|CompilerSemanticSnapshot|WorkspaceIndex|pub mod semantic' \
  phalcom-lsp/src
```

Expected final search result:

```text
no legacy semantic owner
```

Allow references in historical/docs tests only where deliberately asserted absent.

### Step 0.3

Re-pin Technical 04's implementation SHA:

```bash
git rev-parse HEAD
git log -1 --oneline
git status --short
```

Write that SHA into both 04 documents before coding.

### Step 0.4

Re-run Technical 03 and existing generic baseline.

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_inference_proof_integrity

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::generics

cargo test -p phalcom-semantic --test semantic \
  golden_01_generic_self_chain -- --ignored --nocapture
```

The golden test should remain RED/gated before 04 implementation.

No 04 source change before this gate.

# Task 1 — Build the RED receiver-specialization matrix

Create:

```text
phalcom-semantic/tests/semantic/foundations/receiver_specialization.rs
```

Register it in:

```text
phalcom-semantic/tests/semantic/foundations/mod.rs
```

Add focused scenarios independently before touching production code.

### 1.1 Direct generic receiver

```phalcom
class Box<T> {
  value(_ x: T) -> T { x }
}

class Probe {
  @class
  run(_ box: Box<Int>) {
    let value = box.value(1)
  }
}
```

Assert:

```text
effective parameter = Int
effective return    = Int
```

This may already be GREEN.

Keep it as a characterization guard.

### 1.2 Simple inherited generic receiver

```phalcom
class Parent<T> {
  value(_ x: T) -> T { x }
}

class Child<T> is Parent<T> {}

run(_ child: Child<Int>) {
  let x = child.value(1)
}
```

Expected:

```text
x = Int
```

This is the first likely RED.

### 1.3 Transformed generic inheritance

```phalcom
class Parent<T> {
  value() -> T { ... }
}

class Child<T> is Parent<List<T>> {}
```

Receiver:

```text
Child<Int>
```

must expose:

```text
Parent<List<Int>>.value() -> List<Int>
```

### 1.4 Multi-hop transformation

Use:

```text
Leaf<Int>
    -> Middle<Option<Int>>
    -> Base<List<Option<Int>>>
```

Assert a `Base` method returns the fully materialized type.

### 1.5 Inherited nested `Self`

```phalcom
class Parent {
  wrap() -> Box<Self> { ... }
}

class Child is Parent {}
```

Expected:

```text
Child.wrap() -> Box<Child>
```

### 1.6 Class generic + method generic

```phalcom
class Pairer<T> {
  pair<U>(_ value: U) -> (T, U) { ... }
}
```

Receiver:

```text
Pairer<Int>
```

Call:

```text
pair("x")
```

Expected:

```text
(Int, String)
```

### 1.7 Class generic inside callable-generic constraint

Add a `where` fixture accepted by the current parser:

```text
U <: T
```

and prove receiver substitution changes it to:

```text
U <: Animal
```

for `Holder<Animal>`.

### 1.8 Generic inherited field

Exercise an exact field read/write inside permitted visibility and assert the field contract specializes through an ancestor.

### 1.9 Generic constructor

Characterize:

```phalcom
Box<Cat>.new(Cat.new())
```

Assert exact `Box<Cat>` if this syntax is accepted by the current expression parser.

If this test fails earlier in expression classification rather than specialization, preserve that exact failure; Task 9 owns the class-side correction.

### 1.10 `super`

Add:

```phalcom
class Child<T> is Parent<List<T>> {
  f() {
    super.value()
  }
}
```

and prove `Parent`'s `T` is `List<T>` in the child context.

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::receiver_specialization -- --nocapture
```

Commit the RED tests before implementation.

Suggested commit:

```text
test(semantic): characterize receiver specialization gaps
```

# Task 2 — Add the canonical specialization domain

Create:

```text
phalcom-semantic/src/types/specialization.rs
```

Modify:

```text
phalcom-semantic/src/types/mod.rs
```

Add:

```rust
pub struct ReceiverSpecialization { ... }

pub struct ReceiverSpecializationStep { ... }

pub enum ReceiverSpecializationFailure { ... }
```

Add the direct decomposition helper:

```rust
fn receiver_application(
    store: &TypeStore,
    receiver: TypeId,
) -> Option<(DeclarationId, Box<[TypeId]>)>;
```

Rules:

```text
Nominal D
    -> (D, [])

Applied(Nominal D, [A...])
    -> (D, [A...])

other forms
    -> unsupported unless explicitly handled
```

Add:

```rust
fn declaration_environment(
    declarations: &DeclarationTypeTable,
    store: &TypeStore,
    declaration: &DeclarationId,
    arguments: &[TypeId],
) -> Result<TypeEnvironment, ReceiverSpecializationFailure>;
```

Required assertions:

```text
argument count == declaration generic parameter count
kind matches parameter kind
binding key = TypeParameterId
```

Bind actual receiver as:

```rust
environment.bind_self(actual_receiver);
```

Do not expose parameter names in this layer.

Add low-level unit tests in the new module.

Run:

```bash
cargo test -p phalcom-semantic --lib receiver_specialization
```

Commit:

```text
feat(semantic): add canonical receiver specialization domain
```

# Task 3 — Wire source generic superclass templates into the workspace hierarchy

Modify:

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/session.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
```

### 3.1 Fix template registration semantics

Make `MapTypeHierarchy::insert_template(...)` mean only:

```text
register this class's type-level direct superclass template
```

It must not create an artificial nominal parent.

### 3.2 Preserve nominal hierarchy separately

Continue:

```rust
hierarchy.insert(
    class_decl.clone(),
    super_decl.clone(),
);
```

### 3.3 Register the already-resolved template

Immediately after the source hierarchy edge is known, add conceptually:

```rust
if let Some(template) =
    declarations.supertype_template(&class_decl).cloned()
{
    hierarchy.insert_template(template);
}
```

Do not reparse the superclass annotation here.

The canonical declaration product has already done that work.

### 3.4 Add a source-level relation test

Prove from an actual parsed workspace:

```phalcom
class Names<T> is Sequence<T> {}
```

that:

```text
Names<Int> <: Sequence<Int>
```

without manually inserting the template in the test.

### 3.5 Add transformed-parent relation test

Prove a transformed source-level parent application specializes correctly, e.g.:

```text
Names<Int> <: Sequence<List<Int>>
```

for the corresponding source declaration.

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generics_core

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::receiver_specialization
```

Commit:

```text
fix(semantic): publish generic supertype templates into hierarchy
```

# Task 4 — Implement owner-relative inheritance projection

In:

```text
phalcom-semantic/src/types/specialization.rs
```

add:

```rust
pub fn specialize_receiver_to_owner(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    hierarchy: &dyn TypeHierarchy,
    receiver: TypeId,
    member_owner: &DeclarationId,
) -> Result<ReceiverSpecialization, ReceiverSpecializationFailure>;
```

Algorithm:

```text
1. Extract receiver declaration + generic arguments.

2. Build environment for receiver declaration.

3. Save actual receiver as Self binding.

4. If current declaration == member owner:
       return environment.

5. Read nominal direct superclass.

6. Read current declaration's GenericSupertypeTemplate.

7. If a generic template exists:
       materialize template under current environment.
   Else:
       materialize plain superclass declaration type.

8. Decompose materialized superclass into:
       parent declaration
       parent arguments.

9. Verify parent declaration matches nominal hierarchy edge.

10. Build parent declaration environment from parent arguments.

11. Restore Self binding = original actual receiver.

12. Append specialization path step.

13. Repeat until member owner.

14. Detect repeated declarations and fail Cycle.

15. If member owner is not reachable:
       fail OwnerNotReachable.
```

Do not recursively call dispatch during this traversal.

The hierarchy is already the source of superclass topology.

Add pure tests for:

```text
direct owner
one generic hop
transformed one-hop
three-hop transformed inheritance
reordered generic parameters
non-generic edge inside generic chain
cycle rejection
wrong owner rejection
```

Commit:

```text
feat(semantic): project generic receivers to member owners
```

# Task 5 — Specialize complete callable contracts

Still in the specialization layer, add:

```rust
pub fn specialize_callable_signature(
    store: &mut TypeStore,
    signature: &CallableSignature,
    specialization: &ReceiverSpecialization,
) -> CallableSignature;
```

Do not mutate the supplied signature.

### 5.1 Parameters

Replace each known parameter type using:

```text
TypeView(param_type, environment).materialize(store)
```

preserving its `TypeKnowledge` status/origin.

### 5.2 Return

Apply the same operation.

### 5.3 Callable generic parameters

Leave callable-owned `TypeParameterId`s untouched.

### 5.4 Constraints

Add a helper:

```rust
fn specialize_generic_constraint(
    store: &mut TypeStore,
    constraint: &GenericConstraint,
    environment: &TypeEnvironment,
) -> GenericConstraint;
```

Handle:

```rust
Subtype { lower, upper }
Equivalent { left, right }
```

For:

```rust
TypeTerm::Canonical(ty)
```

materialize `ty` through the receiver environment.

For:

```rust
TypeTerm::SelfType(...)
```

materialize against the `Self` binding.

For:

```rust
TypeTerm::Infer(_)
```

preserve it.

The callable `GenericSignature.owner` and `.parameters` must remain exactly unchanged.

### 5.5 Add postcondition checks

In debug/test builds:

```text
no member-owner declaration parameter should remain in a completely
specialized signature if the actual receiver supplied all required
arguments.
```

Callable-owned parameters are expected to remain.

Add focused unit tests.

Commit:

```text
feat(semantic): specialize complete callable contracts by receiver
```

# Task 6 — Replace `CheckingContext`'s direct-only specialization

Modify:

```text
phalcom-semantic/src/checker/context.rs
```

Current local mechanisms equivalent to:

```text
substitution_for_applied_receiver
specialize_dispatch_signature
```

must be replaced by the canonical specialization module.

Do not retain both implementations.

Target inside `resolve_dispatch_target`:

```text
resolved raw dispatch
    ↓
record canonical selected callable/declaration dependencies
    ↓
specialize_receiver_to_owner(
    receiver,
    resolved callable owner / declaring class
)
    ↓
specialize_callable_signature(...)
    ↓
return resolved dispatch with specialized signature
```

Retain the original:

```text
ResolvedDispatch.callable
ResolvedDispatch.declaring_class
```

identity.

Only the signature view changes.

### Dependencies

Record:

```text
DeclarationShell for every path declaration
HierarchyEdge for every traversed edge
DeclarationSurface for selected member owner
CallableSignature for selected callable
```

Do not make the specialization module depend on the semantic DB.

The context records query dependencies after receiving the traversal path.

### Failure handling

Do not silently retain the raw signature.

If the path is inconsistent:

```text
record a structured Blocked/InternalFailure outcome
```

through the existing checker-status infrastructure.

If introducing an explicit dispatch-specialization outcome is necessary after RED tests, do that in this task rather than returning `Option`.

Run the receiver-specialization matrix.

Commit:

```text
fix(semantic): make dispatch specialization inheritance aware
```

# Task 7 — Prove composition with Technical 03

Modify production `checker/call.rs` only if a small interface change is actually needed.

The preferred outcome is that no generic algorithm changes are necessary.

The call path should already receive:

```text
receiver-specialized CallableSignature
```

before:

```rust
apply_generic_callable_inner(...)
```

Technical 03 then instantiates only callable-owned generic parameters.

Add tests:

```phalcom
class Box<T> {
  pair<U>(_ u: U) -> (T, U) { ... }
}
```

for:

```text
Box<Int>.pair(String)
    -> (Int,String)
```

Also test:

```phalcom
class Parent<T> {
  choose<U>(_ value: U) -> (T,U) where U <: T
}

class Child<T> is Parent<T> {}
```

and ensure:

```text
Child<Animal>
    ↓
constraint U <: Animal
```

### Critical proof regression

Use an `Assumed` receiver input and an established method argument.

Ensure the final generic result authority still obeys Technical 03 / call-premise authority.

Receiver substitution itself must not upgrade assumed evidence.

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_inference_proof_integrity

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::generics

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::receiver_specialization
```

Commit:

```text
test(semantic): compose receiver specialization with generic inference
```

# Task 8 — Extend receiver specialization to member forms

Modify:

```text
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
```

### Getter/setter/indexer

These already resolve callable signatures.

They should automatically benefit from Task 6.

Add tests to prove that instead of adding syntax-specific substitution.

### Direct field access

Locate the exact field-resolution path and replace:

```text
read raw DeclarationSurface field TypeKnowledge
```

with:

```text
resolve field owner
    ↓
specialize receiver to owner
    ↓
materialize field contract through environment
```

Field identity remains unchanged.

### Field writes

The declaration contract used for assignability must be specialized identically.

Do not alter field lifecycle state identity; `FieldId` remains declaration-owned.

Test:

```text
Parent<List<Int>>._field
```

read and write through a generic child.

Commit:

```text
feat(semantic): specialize generic member contracts uniformly
```

# Task 9 — Correct generic constructor/class-side specialization

Begin from RED characterization from Task 1.

Do not assume a representation before observing the current syntax path.

The acceptance law is fixed:

```phalcom
Box<Cat>.new(cat)
```

must result in:

```text
Box<Cat>
```

with:

```text
EvidenceOrigin::ConstructorSemantics
```

### Required production shape

Separate:

```text
class/type application
```

from:

```text
constructor value arguments
```

Do not let constructor arguments double as a second ad-hoc class-generic solver.

Audit `synthesize_unqualified_call` and remove any branch that manually:

```text
analyzes ordinary value arguments
collects their TypeIds as declaration generic arguments
checks generic arity
calls store.apply_type_form(...)
```

if that branch is being used as constructor class-generic inference rather than actual type-form application.

Replace it with one explicit declaration/type-application helper using canonical type application.

Then constructor dispatch receives the specialized class receiver and ordinary value arguments go through Technical 02.

Constructor `Self` is materialized from the specialized class receiver.

Add:

```text
Box<Int>.new(1) -> Box<Int>
Box<String>.new("x") -> Box<String>
inherited generic constructor if language permits it
constructor mismatch still diagnoses argument relation
```

Commit:

```text
fix(semantic): preserve class specialization through constructors
```

# Task 10 — `super` and nested `Self`

Add focused tests before changes.

For a generic `super` call, the actual receiver remains the subclass but target lookup begins at the superclass.

The specialization algorithm therefore receives:

```text
actual receiver = Child<Int>
member owner    = Parent
```

and computes the same parent environment as ordinary inherited dispatch.

Do not bind `Self` to `Parent`.

Test:

```phalcom
class Parent<T> {
  value() -> T { ... }
  selfBox() -> Box<Self> { ... }
}

class Child<T> is Parent<List<T>> {
  probe() {
    (super.value(), super.selfBox())
  }
}
```

Expected:

```text
(List<T>, Box<Child<T>>)
```

Commit:

```text
feat(semantic): specialize generic super sends and inherited Self
```

# Task 11 — Add incremental dependency proof

Modify:

```text
phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
```

Create a persistent `SemanticWorkspaceSession`.

Revision A:

```phalcom
class Parent<T> {
  value() -> T { ... }
}

class Child<T> is Parent<T> {}

class Consumer {
  @class
  use(_ value: Child<Int>) {
    value.value()
  }

  @class
  independent() { 42 }
}
```

Revision B:

```phalcom
class Child<T> is Parent<List<T>> {}
```

Required assertions:

```text
Consumer.use semantic product recomputes

its result changes Int -> List<Int>

Parent/Child relevant hierarchy/declaration products change

Consumer.independent Arc is reused where its direct/dependency
fingerprints are unchanged

canonical callable IDs remain unchanged

TypeStore identity remains stable
```

Then perform a trivia-only edit around `Child`.

Required:

```text
semantic inherited specialization product unchanged
unrelated semantic products reusable
source presentation may refresh independently
```

If the test shows generic-supertype metadata is omitted from a relevant fingerprint, modify:

```text
phalcom-semantic/src/db/fingerprint.rs
```

at this task.

Do not eagerly change fingerprints before the RED test demonstrates the omission.

Commit:

```text
test(semantic): prove incremental generic specialization dependencies
```

# Task 12 — Activate `golden_01_generic_self_chain`

Modify:

```text
phalcom-semantic/tests/semantic/golden/mod.rs
```

Remove only:

```rust
#[ignore = "semantic gate waits for generic inheritance and nested Self specialization"]
```

from:

```text
golden_01_generic_self_chain
```

Do not remove other ignores.

Strengthen the test where the fixture API permits it.

At minimum prove:

```text
maker.echo(Cat.new()) -> Cat

maker.wrap(...) -> Box<Cat>

Box<Cat> assignable to Box<Animal>
because Box is covariant

CatNode.new().boxed() -> Box<CatNode>

Box<CatNode>.value() -> CatNode
```

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  golden_01_generic_self_chain -- --nocapture
```

Commit:

```text
test(semantic): activate generic receiver and Self golden
```

# Task 13 — Remove superseded partial mechanisms

Search:

```bash
rg -n \
  'substitution_for_applied_receiver|specialize_dispatch_signature|specialize_self_type|TypeParameterOwner::Declaration|supertype_template|insert_template' \
  phalcom-semantic/src
```

Classify every result.

Delete the old context-local direct-only substitution helper once no caller remains.

Keep the canonical low-level `specialize_self_type` only if it remains part of the new specialization implementation; otherwise fold its recursion into `TypeEnvironment`/`TypeView` and delete the duplicate.

Audit for forbidden generic mapping:

```bash
rg -n \
  'type_parameter.*name|parameter.*name.*argument|zip.*generic|HashMap<String.*TypeId' \
  phalcom-semantic/src/checker \
  phalcom-semantic/src/types
```

`HashMap<String, TypeId>` remains legitimate in source annotation resolver scope.

It is forbidden in call-site specialization.

Audit for surface mutation:

```bash
rg -n \
  'register_surface|add_callable|callable_signatures.*insert' \
  phalcom-semantic/src/checker
```

No call-site specialization should write a specialized signature back into a declaration surface.

Audit for LSP copies:

```bash
rg -n \
  'generic.*special|special.*receiver|SelfType|TypeSubstitution' \
  phalcom-lsp/src
```

After the retirement there should be no semantic implementation there.

Commit:

```text
refactor(semantic): retire partial receiver specialization paths
```

# Task 14 — Full closure verification

Run all Technical-04 focused suites:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::receiver_specialization

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generics_core

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_inference_proof_integrity

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::generics

cargo test -p phalcom-semantic --test semantic \
  golden_01_generic_self_chain
```

Then predecessors:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application
```

Then incremental:

```bash
cargo test -p phalcom-semantic --test semantic \
  incremental
```

Then full semantic target:

```bash
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-semantic --lib
cargo check -p phalcom-semantic
```

Because retirement is the prerequisite, also:

```bash
cargo check -p phalcom-lsp --lib
cargo test -p phalcom-lsp --test semantic_boundary
```

Finally:

```bash
cargo fmt --all -- --check
git diff --check
```

Run clippy as a non-silent gate:

```bash
cargo clippy -p phalcom-semantic --all-targets -- -D warnings
```

If the pre-existing generated-native-surface `clippy::deref_addrof` baseline still exists, record it precisely rather than absorbing generated cleanup into Technical 04.

# Final implementation verification checklist

Before claiming Technical 04 complete, verify:

```text
[ ] Part 3 single-semantic-world retirement is complete.
[ ] Implementation SHA is re-pinned in both 04 documents.
[ ] No legacy phalcom-lsp semantic owner remains.
[ ] Receiver specialization is owner-relative.
[ ] Multi-hop transformed generic inheritance works.
[ ] Self remains bound to the actual receiver through inherited calls.
[ ] Callable-owned generics are not replaced by declaration substitution.
[ ] Enclosing generic parameters inside method constraints are specialized.
[ ] Getter/setter/indexer/direct-field contracts specialize identically.
[ ] Generic constructor Self preserves the applied class receiver.
[ ] Raw declaration surfaces remain generic and immutable.
[ ] No specialization is keyed by parameter name.
[ ] No silent fallback to raw unspecialized signatures remains.
[ ] Hierarchy/declaration dependencies drive invalidation.
[ ] Unrelated callable products remain reusable.
[ ] golden_01_generic_self_chain is active and green.
[ ] No unrelated golden ignore was removed.
[ ] Technical 01–03 tests remain green.
[ ] Full semantic test target passes.
[ ] phalcom-lsp compiles after retirement.
[ ] fmt and diff checks pass.
[ ] clippy result is recorded without hiding any baseline issue.
```
