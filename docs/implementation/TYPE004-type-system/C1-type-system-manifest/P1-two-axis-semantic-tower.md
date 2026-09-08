# Phalcom Two-Axis Semantic Tower Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Phalcom’s two-axis semantic tower so runtime classification (`v.class`) remains orthogonal to static value typing (`v : T`) and kind classification (`T :: K`), while making type forms, kinds, denotation, generic application, class-side dispatch, project-wide checking, compiler/CLI integration, and LSP diagnostics share one canonical semantic engine.

**Architecture:** `phalcom-semantic` owns canonical type forms, kinds, denotation, substitution, relations, declaration typing, project-wide semantic analysis, and immutable static semantic snapshots. `phalcom-modules` owns source/module identity, parse-once source artifacts, interfaces, linking, declaration shells, and semantic dependency graphs. `phalcom-core` consumes analyzed programs and stable semantic export descriptors; `phalcom-lsp` consumes the same static semantic snapshot while retaining its separate advisory `ValueShape` engine. Runtime heap objects, `Value::class()`, and the metaclass tower are unchanged in this milestone.

**Tech Stack:** Rust workspace; `phalcom-ast`; `phalcom-modules`; `phalcom-native-meta`; `phalcom-native-macros`; `phalcom-semantic`; `phalcom-core`; `phalcom-lsp`; `thiserror`; `Arc`; deterministic `BTreeMap`; existing `graphify` workflow.

**Spec:** `docs/work/pending/typing/phalcom_two_axis_semantic_tower_repository_grounded_implementation_spec.md`

**Repository baseline verified:** `main` at `ab9142700e53fb012a5da71a1074bd67ec8f44b4` when this plan was produced.

## Global Constraints

- Repository crates live directly at repository root: `phalcom-semantic/...`, `phalcom-modules/...`, `phalcom-core/...`; **never use a `crates/` prefix**.
- `Type` is the atomic kind.
- `TypeForm` is the semantic/behavioral role of values that denote type-level forms.
- `TypeId` identifies one canonical type-level form within one `TypeStore`; its `KindId` determines whether it is a proper type or constructor.
- `TypeKnowledge::Known` may only describe proper value types whose kind is `Type`.
- Ordinary value type and semantic denotation are distinct facts.
- Class objects directly denote nominal type forms/type constructors; never introduce a `ClassType` runtime wrapper.
- `TypeData::ClassObject { declaration }` is an internal static value type, not a public reflected type-form descriptor.
- Type application is canonical, kind-checked, and non-overridable.
- Internal semantic algebra supports partial/higher-kinded application now; no new higher-kinded source declaration syntax is introduced.
- No runtime type/kind heap objects are added in this milestone.
- No runtime `Value` layout/tag change.
- No metaclass tower change.
- No generic specialization by cloning runtime classes.
- No per-instance generic type token.
- `Unknown` remains epistemic state, never a semantic type.
- `Dynamic` remains explicit static escape state, never a fake nominal top type.
- Generic declaration parameters are invariant until explicit variance metadata is designed.
- Selector identity remains independent of typing.
- `super` changes lookup start while retaining the actual receiver.
- `phalcom-lsp::semantic::ValueShape` remains advisory and is not converted into the language type system.
- Raw `TypeId`, `KindId`, or `TypeParameterId` must never be persisted into stable compiler/runtime artifacts.
- If a task would require unapproved public syntax or a new semantic law, stop at an internal API boundary rather than inventing it.
- Follow test-first implementation: failing focused test → minimal implementation → focused pass → commit.

---

# File and Ownership Map

## Documentation

- Modify: `docs/spec/typing/ontology.md`
- Modify: `docs/spec/typing/README.md`
- Modify: `docs/spec/typing/STATUS.md`
- Modify: `docs/spec/typing/02-type-expression-foundation.md`
- Modify copied historical typing examples only to add supersession banners where current agents may read them.

## `phalcom-semantic`

- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-semantic/src/source.rs`
- Modify: `phalcom-semantic/src/diagnostic.rs`
- Modify: `phalcom-semantic/src/dispatch.rs`
- Modify: `phalcom-semantic/src/surface.rs`
- Modify: `phalcom-semantic/src/identity.rs` only if dispatch request identity helpers belong there.
- Create: `phalcom-semantic/src/declarations.rs`
- Create: `phalcom-semantic/src/resolver.rs`
- Create: `phalcom-semantic/src/workspace.rs`
- Create: `phalcom-semantic/src/export.rs`

### `phalcom-semantic/src/types`

- Modify: `phalcom-semantic/src/types/mod.rs`
- Modify: `phalcom-semantic/src/types/id.rs`
- Modify: `phalcom-semantic/src/types/kind.rs`
- Modify: `phalcom-semantic/src/types/store.rs`
- Modify: `phalcom-semantic/src/types/evidence.rs`
- Modify: `phalcom-semantic/src/types/annotation.rs`
- Modify: `phalcom-semantic/src/types/native.rs`
- Modify: `phalcom-semantic/src/types/relation.rs`
- Modify: `phalcom-semantic/src/types/constraint.rs`
- Create: `phalcom-semantic/src/types/application.rs`
- Create: `phalcom-semantic/src/types/parameter.rs`
- Create: `phalcom-semantic/src/types/denotation.rs`
- Create: `phalcom-semantic/src/types/substitution.rs`

### `phalcom-semantic/src/checker`

- Modify: `phalcom-semantic/src/checker/mod.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/typed_expr.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/checker/result.rs` if `TypeCheckReport` is retained as low-level body-check report.

## `phalcom-native-meta`

- Modify: `phalcom-native-meta/src/types.rs`
- Modify: `phalcom-native-meta/src/universe.rs`
- Modify: `phalcom-native-meta/src/lib.rs` re-exports if required.

## `phalcom-native-macros`

- Modify: `phalcom-native-macros/src/lib.rs` so emitted type-parameter metadata carries kind information and remains compatible with new native semantic normalization.

## `phalcom-modules`

- Modify: `phalcom-modules/src/source.rs`
- Modify: `phalcom-modules/src/resolver.rs`
- Modify: `phalcom-modules/src/builtin_interface.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Modify: `phalcom-modules/src/linker.rs` only if declaration-level semantic edges can be derived cleanly at link time; otherwise workspace extends `linked.graphs.semantics` without changing linker.
- Existing reuse: `phalcom-modules/src/declaration.rs`
- Existing reuse: `phalcom-modules/src/graph.rs`

## `phalcom-core`

- Modify: `phalcom-core/src/modules/compile.rs`
- Modify: `phalcom-core/src/modules/artifact.rs` only if stable semantic descriptors are immediately attached to compiled artifacts.
- Modify: `phalcom-core/bin/phalcom/cli.rs`

### Explicit runtime files that must remain semantically unchanged

- `phalcom-core/src/value/repr.rs`
- `phalcom-core/src/value/mod.rs`
- `phalcom-core/src/heap/object.rs`
- `phalcom-core/src/heap/class.rs`
- `phalcom-core/src/universe/core_classes.rs`

Import-only or formatting-only churn is acceptable; behavioral changes are not.

## `phalcom-lsp`

- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/diagnostics.rs`
- Modify relevant worker/database files where published advisory snapshots are assembled.
- Reuse: `phalcom-lsp/src/semantic/ids.rs::DocumentModuleMap`
- Preserve: `phalcom-lsp/src/semantic/snapshot.rs` as the advisory/runtime-shape snapshot domain.

## New semantic integration tests

- Create: `phalcom-semantic/tests/kinds.rs`
- Create: `phalcom-semantic/tests/declaration_types.rs`
- Create: `phalcom-semantic/tests/denotation.rs`
- Create: `phalcom-semantic/tests/type_annotations.rs`
- Create: `phalcom-semantic/tests/class_side_dispatch.rs`
- Create: `phalcom-semantic/tests/substitution.rs`
- Create: `phalcom-semantic/tests/workspace.rs`
- Create: `phalcom-semantic/tests/export.rs`

---

# Task 1: Freeze the Current Ontology and Remove Stale `Type`-Protocol Authority

**Files:**
- Modify: `docs/spec/typing/ontology.md`
- Modify: `docs/spec/typing/README.md`
- Modify: `docs/spec/typing/STATUS.md`
- Modify: `docs/spec/typing/02-type-expression-foundation.md`

**Interfaces:**
- Consumes: ratified ontology and this implementation plan.
- Produces: one normative documentation authority for every later task.

- [ ] **Step 1: Add the normative terminology section to `ontology.md`**

Add a section near the beginning containing exactly these semantic claims:

```markdown
## Normative terminology and precedence

- `Type` is the atomic kind of proper types.
- `TypeForm` names the common semantic/behavioral role of values that denote
  type-level forms.
- `TypeForm` is not a superclass inserted into the runtime object hierarchy.
- A `TypeId` identifies a canonical type-level form within one `TypeStore`;
  the associated `KindId` determines whether that form is a proper type or a
  type constructor.
- The ordinary type of a class-object value is distinct from the type form
  denoted by that value.
- Runtime reflection reifies this semantic model; it does not define it.
- Older typing documents that define `Type` as a protocol are historical where
  they conflict with this ontology.
```

- [ ] **Step 2: Add the non-negotiable implementation invariants**

Add a compact block covering:
- no `ClassType` wrapper;
- no runtime type/kind heap objects in this milestone;
- `.class`, `:`, and `::` remain distinct relations;
- no raw semantic IDs in persisted artifacts;
- no new syntax.

- [ ] **Step 3: Mark stale documents as historical**

At the top of `02-type-expression-foundation.md` add:

```markdown
> **Superseded terminology:** current Phalcom typing ontology reserves `Type`
> for the atomic kind and uses `TypeForm` for the common type-denoting role.
> This document is retained as design history. See `ontology.md`.
```

Update `README.md` and `STATUS.md` so no current status table calls the old `@protocol class Type` decision “locked.”

- [ ] **Step 4: Verify stale authority is discoverable but clearly superseded**

Run:

```bash
rg '@protocol\s+class Type|signature-only `Protocol`|class Type \{' docs/spec/typing
```

Expected:
- historical matches may remain;
- every historical document that can be mistaken for current authority has a supersession banner.

- [ ] **Step 5: Commit**

```bash
git add docs/spec/typing
git commit -m "docs(typing): ratify two-axis type and kind ontology"
```

---

# Task 2: Make `TypeStore` Totally Kinded and Add Canonical Kind Application

**Files:**
- Modify: `phalcom-semantic/src/types/id.rs`
- Modify: `phalcom-semantic/src/types/kind.rs`
- Modify: `phalcom-semantic/src/types/store.rs`
- Modify: `phalcom-semantic/src/types/evidence.rs`
- Modify: `phalcom-semantic/src/types/mod.rs`
- Create: `phalcom-semantic/src/types/application.rs`
- Create: `phalcom-semantic/tests/kinds.rs`

**Interfaces:**
- Produces:
  - `TypeStore::arrow_kind(...) -> KindId`
  - `TypeStore::apply_kind(...) -> Result<KindId, KindApplicationError>`
  - `TypeStore::apply_type_form(...) -> Result<TypeId, TypeApplicationError>` or exported free wrapper
  - `TypeStore::class_object_type(DeclarationId) -> TypeId`
  - `TypeStore::is_proper_type(TypeId) -> bool`
  - explicit nominal-form construction
- Later tasks must use these APIs instead of `store.applied()` and bare `store.nominal()` assumptions.

- [ ] **Step 1: Write failing kind tests**

Create `phalcom-semantic/tests/kinds.rs` with tests equivalent to:

```rust
#[test]
fn kind_application_returns_residual_arrow_kind() {
    let mut store = TypeStore::new();
    let map_kind = store.arrow_kind(
        vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(),
        KindId::TYPE,
    );

    let residual = store.apply_kind(map_kind, &[KindId::TYPE]).unwrap();

    assert_eq!(
        store.get_kind(residual),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );
}

#[test]
fn nested_type_application_canonicalizes() {
    let mut store = TypeStore::new();
    let map_decl = test_decl("Map");
    let string_decl = test_decl("String");
    let int_decl = test_decl("Int");

    let map_kind = store.arrow_kind(
        vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(),
        KindId::TYPE,
    );
    let map = store.nominal_form(map_decl, map_kind);
    let string = store.nominal_type(string_decl);
    let int = store.nominal_type(int_decl);

    let partially = store.apply_type_form(map, &[string]).unwrap();
    let nested = store.apply_type_form(partially, &[int]).unwrap();
    let direct = store.apply_type_form(map, &[string, int]).unwrap();

    assert_eq!(nested, direct);
    assert_eq!(store.kind_of(direct), KindId::TYPE);
}
```

Also add:
- wrong-kind parameter test;
- too-many-arguments test;
- applying proper `Int` as constructor test;
- `ClassObject(Int)` distinct from nominal `Int`;
- every interned type has an explicit kind.

- [ ] **Step 2: Run tests and confirm failure**

```bash
cargo test -p phalcom-semantic --test kinds
```

Expected: compile/test failures because APIs and `ClassObject` do not exist.

- [ ] **Step 3: Redefine `TypeId` documentation**

In `phalcom-semantic/src/types/id.rs`:

```rust
/// Store/snapshot-local canonical identifier for a type-level form.
///
/// A `TypeId` may identify a proper type (`kind == Type`) or an unsaturated
/// type constructor/higher-kinded form. The associated `KindId` determines
/// which. The integer is meaningful only with the `TypeStore` that allocated it.
pub struct TypeId(pub u32);
```

Document the same store-local rule for `KindId` and `TypeParameterId`.

- [ ] **Step 4: Add `ClassObject` to `TypeData`**

In `store.rs` add:

```rust
ClassObject {
    declaration: DeclarationId,
},
```

Document it as:
- proper static type of a runtime class-object value;
- internal semantic representation;
- not a runtime wrapper;
- not surface syntax.

- [ ] **Step 5: Replace sparse type-kind map with dense storage**

Change:

```rust
type_kinds: HashMap<TypeId, KindId>,
```

to:

```rust
type_kinds: Vec<KindId>,
```

Delete public `set_kind()`.

Add:

```rust
fn intern_with_kind(&mut self, data: TypeData, kind: KindId) -> TypeId {
    if let Some(&id) = self.type_to_id.get(&data) {
        debug_assert_eq!(self.type_kinds[id.index()], kind);
        return id;
    }

    let id = TypeId(self.types.len() as u32);
    self.types.push(data.clone());
    self.type_kinds.push(kind);
    self.type_to_id.insert(data, id);
    debug_assert_eq!(self.types.len(), self.type_kinds.len());
    id
}
```

Replace `kind_of()` with direct indexing:

```rust
#[inline]
pub fn kind_of(&self, ty: TypeId) -> KindId {
    self.type_kinds[ty.index()]
}
```

- [ ] **Step 6: Bootstrap `Never` and `Unit` through kinded interning**

`TypeStore::new()` must create:

```rust
store.never_id = store.intern_with_kind(TypeData::Never, KindId::TYPE);
store.unit_id = store.intern_with_kind(TypeData::Unit, KindId::TYPE);
```

No fallback.

- [ ] **Step 7: Add canonical arrow construction**

Implement as a `TypeStore` method because the store owns kind interning:

```rust
pub fn arrow_kind(
    &mut self,
    parameters: Box<[KindId]>,
    result: KindId,
) -> KindId
```

Rules:
- empty parameter list returns `result`;
- if `result` is `KindData::Arrow`, flatten only the result-side arrow;
- do not flatten an arrow kind that appears as an input parameter.

- [ ] **Step 8: Add store-owned `apply_kind`**

Implement:

```rust
pub fn apply_kind(
    &mut self,
    callee: KindId,
    arguments: &[KindId],
) -> Result<KindId, KindApplicationError>
```

`KindApplicationError`:

```rust
pub enum KindApplicationError {
    NotApplicable { kind: KindId },
    TooManyArguments { supplied: usize, accepted: usize },
    ArgumentKindMismatch {
        index: usize,
        expected: KindId,
        actual: KindId,
    },
}
```

Algorithm:
1. `Type` with non-empty args → `NotApplicable`.
2. `Arrow { parameters, result }`:
   - reject if supplied > parameters.len();
   - compare supplied argument kinds exactly;
   - if all parameters consumed, return `result`;
   - otherwise intern `Arrow(remaining_parameters, result)` via `arrow_kind`.

- [ ] **Step 9: Add explicit nominal-form APIs**

Implement:

```rust
pub fn nominal_form(
    &mut self,
    declaration: DeclarationId,
    kind: KindId,
) -> TypeId

pub fn nominal_type(
    &mut self,
    declaration: DeclarationId,
) -> TypeId

pub fn class_object_type(
    &mut self,
    declaration: DeclarationId,
) -> TypeId

pub fn is_proper_type(
    &self,
    form: TypeId,
) -> bool
```

`nominal_type` is only convenience for proven zero-parameter declarations.

- [ ] **Step 10: Implement canonical checked type application**

In `application.rs` define:

```rust
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TypeApplicationError {
    #[error("type form is not applicable")]
    NotAConstructor { origin: TypeId, kind: KindId },

    #[error("too many type arguments")]
    TooManyArguments { supplied: usize, accepted: usize },

    #[error("type argument kind mismatch at index {index}")]
    ArgumentKindMismatch {
        index: usize,
        expected: KindId,
        actual: KindId,
    },
}
```

Expose through `TypeStore`:

```rust
pub fn apply_type_form(
    &mut self,
    origin: TypeId,
    arguments: &[TypeId],
) -> Result<TypeId, TypeApplicationError>
```

Algorithm:
1. empty args → return origin;
2. flatten existing `TypeData::Applied` origin + old args;
3. compute current origin kind;
4. collect new argument kinds;
5. call `apply_kind`;
6. intern flattened `TypeData::Applied` with residual/result kind.

- [ ] **Step 11: Make composite proper-type constructors validate children**

Before interning:
- `union`: every member must be proper `Type`;
- `tuple`: every element type must be proper;
- `record`: every field type must be proper;
- `callable`: every parameter/result must be proper.

Use internal assertions for programming errors and checked callers for source errors.

- [ ] **Step 12: Remove or rewrite collection-specific application helpers**

Current `list_of`, `map_of`, `set_of` encode bare nominal origin as kind `Type`.

Preferred action: remove them and migrate call sites to `apply_type_form`.

If temporary compatibility helpers are kept, require an already-canonical constructor form:

```rust
pub fn list_of(
    &mut self,
    list_form: TypeId,
    element: TypeId,
) -> Result<TypeId, TypeApplicationError> {
    self.apply_type_form(list_form, &[element])
}
```

Do not accept a `DeclarationId` and reconstruct a bare nominal form.

- [ ] **Step 13: Enforce proper-type evidence boundary**

In `evidence.rs` update docs:

```rust
/// `Known` is only valid for proper value types whose kind is `Type`.
Known(TypeEvidence),
```

Do not make `TypeKnowledge::known()` silently perform kind checking without a store. Instead:
- introduce `TypeStore::known_type(ty, authority)` if useful; or
- ensure all public semantic boundaries validate `store.is_proper_type(ty)` before constructing `Known`;
- add `debug_assert!(store.is_proper_type(...))` in subtype/assignability entry points.

- [ ] **Step 14: Run focused tests**

```bash
cargo test -p phalcom-semantic --test kinds
cargo test -p phalcom-semantic --lib types::
```

Expected: PASS.

- [ ] **Step 15: Commit**

```bash
git add phalcom-semantic/src/types phalcom-semantic/tests/kinds.rs
git commit -m "feat(semantic): make type forms explicitly kinded"
```

---

# Task 3: Add Declaration Type Forms, Type Parameters, and Trusted Core Generic Metadata

**Files:**
- Create: `phalcom-semantic/src/types/parameter.rs`
- Create: `phalcom-semantic/src/declarations.rs`
- Modify: `phalcom-semantic/src/types/store.rs`
- Modify: `phalcom-semantic/src/types/mod.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-native-meta/src/types.rs`
- Modify: `phalcom-native-meta/src/universe.rs`
- Modify: `phalcom-native-macros/src/lib.rs`
- Modify: `phalcom-semantic/src/types/native.rs`
- Create: `phalcom-semantic/tests/declaration_types.rs`

**Interfaces:**
- Produces:
  - `TypeParameterOwner`
  - `TypeParameterData`
  - `GenericSignature`
  - `DeclarationTypeInfo`
  - `DeclarationTypeTable`
  - trusted `UniverseTypeFormSpec`
  - universe bootstrap helper
- Consumed by checker context, annotations, native normalization, substitution, workspace.

- [ ] **Step 1: Write failing declaration-kind tests**

Create tests proving:
- `Int` form kind is `Type`;
- `List` kind is `Type -> Type`;
- `Set` kind is `Type -> Type`;
- `Map` kind is `Type -> Type -> Type`;
- `Option` kind is `Type -> Type`;
- `Some` kind is also `Type -> Type` so generic inheritance remains kind-consistent;
- class-object proper types all have kind `Type`;
- declaration form and class-object type are distinct;
- parameter owner/index are stable within a store.

- [ ] **Step 2: Run tests and confirm failure**

```bash
cargo test -p phalcom-semantic --test declaration_types
```

- [ ] **Step 3: Add parameter identity/data**

In `parameter.rs`:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypeParameterOwner {
    Declaration(DeclarationId),
    Callable(CallableId),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TypeParameterData {
    pub owner: TypeParameterOwner,
    pub index: u16,
    pub name: Box<str>,
    pub kind: KindId,
}

#[derive(Clone, Debug)]
pub struct GenericSignature {
    pub owner: TypeParameterOwner,
    pub parameters: Box<[TypeParameterId]>,
}
```

- [ ] **Step 4: Add parameter interning to `TypeStore`**

Add:

```rust
type_parameters: Vec<TypeParameterData>,
parameter_to_id: HashMap<(TypeParameterOwner, u16), TypeParameterId>,
```

and:

```rust
pub fn intern_type_parameter(
    &mut self,
    data: TypeParameterData,
) -> TypeParameterId

pub fn type_parameter(
    &self,
    id: TypeParameterId,
) -> &TypeParameterData

pub fn parameter_form(
    &mut self,
    id: TypeParameterId,
) -> TypeId
```

`parameter_form` uses the parameter’s declared kind.

- [ ] **Step 5: Add `DeclarationTypeTable`**

In `declarations.rs`:

```rust
#[derive(Clone, Debug)]
pub struct DeclarationTypeInfo {
    pub declaration: DeclarationId,
    pub form: TypeId,
    pub class_object_type: TypeId,
    pub kind: KindId,
    pub generic_signature: Option<GenericSignature>,
}

#[derive(Clone, Debug, Default)]
pub struct DeclarationTypeTable {
    entries: HashMap<DeclarationId, DeclarationTypeInfo>,
}
```

APIs:

```rust
pub fn insert(&mut self, info: DeclarationTypeInfo)
pub fn get(&self, declaration: &DeclarationId) -> Option<&DeclarationTypeInfo>
pub fn form(&self, declaration: &DeclarationId) -> Option<TypeId>
pub fn class_object_type(&self, declaration: &DeclarationId) -> Option<TypeId>
pub fn kind(&self, declaration: &DeclarationId) -> Option<KindId>
```

Insertion invariant:
- `info.kind == store.kind_of(info.form)` must be validated by bootstrap caller;
- `info.class_object_type :: Type`.

- [ ] **Step 6: Add VM-free kind specs to native metadata**

In `phalcom-native-meta/src/types.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum KindSpec {
    Type,
    Arrow {
        parameters: &'static [KindSpec],
        result: &'static KindSpec,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TypeParameterDeclSpec {
    pub name: &'static str,
    pub kind: KindSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct UniverseTypeFormSpec {
    pub owner: UniverseKey,
    pub parameters: &'static [TypeParameterDeclSpec],
}
```

- [ ] **Step 7: Extend callable type-parameter metadata with kinds**

Change:

```rust
pub struct TypeParameterSpec {
    pub name: &'static str,
}
```

to:

```rust
pub struct TypeParameterSpec {
    pub name: &'static str,
    pub kind: KindSpec,
}
```

For current native macro syntax, emitted callable type parameters default to `KindSpec::Type`. No new macro syntax is added in this milestone.

Update `phalcom-native-macros/src/lib.rs::emit_callable_spec` accordingly.

- [ ] **Step 8: Add core universe generic signatures**

In `phalcom-native-meta/src/universe.rs` add:

```rust
pub const UNIVERSE_TYPE_FORMS: &[UniverseTypeFormSpec] = &[
    // List<T>
    // Set<T>
    // Map<K, V>
    // Option<T>
    // Some<T>
];
```

Use static `KindSpec::Type` parameter definitions.

Do **not** invent a generic type constructor for `None`; the runtime/source special case is explicitly deferred.

- [ ] **Step 9: Bootstrap declaration type forms**

Add a semantic helper that:
1. creates type-parameter IDs;
2. builds declaration kind from parameter kinds ending in `Type`;
3. interns nominal declaration form with that kind;
4. interns class-object proper type;
5. inserts `DeclarationTypeInfo`.

Non-generic universe declarations default to zero parameters and kind `Type`.

- [ ] **Step 10: Migrate native normalization to declaration table**

Change `normalize_native_type` to take enough context to resolve canonical declaration forms and type parameters.

Introduce internal split:

```rust
fn resolve_native_type_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    parameters: &HashMap<&str, TypeId>,
    universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId,
    spec: &TypeExprSpec,
) -> Result<TypeId, NativeTypeResolutionError>
```

Then `normalize_native_type` converts only proper forms into `TypeKnowledge::Known`.

Handle:
- `Never`
- `Unit`
- `Universe`
- `Parameter`
- `Applied`
- `Union`
- `Tuple`

Unsupported/unknown stays epistemically unknown rather than inventing a type.

- [ ] **Step 11: Migrate `register_standard_surfaces` away from bare generic nominals**

For `List`, `Map`, `Set`, `Option` and `Some`:
- obtain constructor form from `DeclarationTypeTable`;
- do not register bare constructor form as an ordinary instance receiver;
- native member signatures using type parameters should use canonical parameter forms where metadata exists.

Do not keep `List.add` permanently `Dynamic` if the generic parameter is now available; bind it to `T`.

- [ ] **Step 12: Run focused tests**

```bash
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-macros
cargo test -p phalcom-semantic --test declaration_types
```

- [ ] **Step 13: Commit**

```bash
git add phalcom-native-meta phalcom-native-macros phalcom-semantic
git commit -m "feat(semantic): register canonical declaration type forms"
```

---

# Task 4: Add Denotation and Preserve It Through Expression/Binding Semantics

**Files:**
- Create: `phalcom-semantic/src/types/denotation.rs`
- Modify: `phalcom-semantic/src/types/mod.rs`
- Modify: `phalcom-semantic/src/checker/typed_expr.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Create: `phalcom-semantic/tests/denotation.rs`

**Interfaces:**
- Produces:
  - `SemanticDenotation`
  - `ValueSemanticFact`
  - fact-aware local environment
- Consumes:
  - `DeclarationTypeTable`
  - proper `ClassObject` types
- Later dispatch task adds lookup mode; `ReceiverDispatch` does **not** live under `types/denotation.rs`.

- [ ] **Step 1: Write failing denotation tests**

Required tests:

```rust
#[test]
fn class_name_has_class_object_value_type_and_type_form_denotation() { ... }

#[test]
fn generic_class_name_denotes_constructor_kind() { ... }

#[test]
fn ordinary_literal_has_no_denotation() { ... }

#[test]
fn const_binding_preserves_denotation() { ... }

#[test]
fn reassignment_replaces_or_clears_denotation() { ... }

#[test]
fn flow_join_preserves_only_identical_denotation() { ... }
```

- [ ] **Step 2: Add denotation types**

In `types/denotation.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticDenotation {
    TypeForm(TypeId),
    Kind(KindId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueSemanticFact {
    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
}
```

Helpers:

```rust
impl ValueSemanticFact {
    pub fn new(knowledge: TypeKnowledge) -> Self
    pub fn with_denotation(self, denotation: SemanticDenotation) -> Self
    pub fn merge(left: &Self, right: &Self, merged_knowledge: TypeKnowledge) -> Self
}
```

Merge rule:
- preserve denotation only if both sides contain the exact same `SemanticDenotation`;
- otherwise `None`.

- [ ] **Step 3: Extend `TypedExpression`**

Add:

```rust
pub denotation: Option<SemanticDenotation>,
```

No dispatch marker yet.

Add:

```rust
pub fn with_denotation(...)
pub fn fact(&self) -> ValueSemanticFact
```

All existing constructors default to `None`.

- [ ] **Step 4: Give `CheckingContext` the declaration table**

Add:

```rust
pub declarations: &'a DeclarationTypeTable,
```

to `CheckingContext`.

Update constructor:

```rust
pub fn new(
    store: &'a mut TypeStore,
    hierarchy: &'a dyn TypeHierarchy,
    resolver: &'a dyn TypeResolver,
    declarations: &'a DeclarationTypeTable,
    current_module: ModuleId,
) -> Self
```

Update all current tests/callers.

- [ ] **Step 5: Store full semantic facts in `LocalEnv`**

Change:

```rust
bindings: HashMap<String, TypeKnowledge>
```

to:

```rust
bindings: HashMap<String, ValueSemanticFact>
```

Update:

```rust
bind_local(name, fact)
lookup_local(name) -> Option<&ValueSemanticFact>
```

Add convenience `lookup_local_knowledge` only if existing call sites benefit.

- [ ] **Step 6: Fix class-name expression synthesis**

Current behavior uses `store.nominal(decl)`.

Replace with:

```rust
let info = ctx.declarations.get(&decl)
    .expect("resolved declaration must have type info");

TypedExpression::known(
    info.class_object_type,
    EvidenceAuthority::Declared,
    *range,
)
.with_denotation(SemanticDenotation::TypeForm(info.form))
```

Literal branches obtain proper nominal declaration form from table and keep `denotation = None`.

- [ ] **Step 7: Preserve denotation through bindings**

For unannotated:

```phalcom
const t = Int
```

binding fact is initializer fact.

For an annotation plus initializer:
- annotation constrains ordinary value type;
- denotation comes from initializer value if assignable;
- annotation itself does not manufacture denotation.

- [ ] **Step 8: Define assignment semantics**

For:

```phalcom
let t = Int
t = String
```

the local fact after assignment must denote `String`, not stale `Int`.

For assignment from an ordinary non-denoting value, clear denotation.

If current scope model does not mutate bindings on assignment yet, add a targeted `LocalEnv::assign_existing` operation that finds the nearest existing binding and replaces its `ValueSemanticFact`.

- [ ] **Step 9: Preserve denotation through blocks only when result expression preserves it**

Replace lossy `synthesize_expr()` usage in tail-expression paths with `synthesize_typed_expr()`.

A block’s result:
- uses tail expression fact if last statement is expression;
- `throw` → `Never`, no denotation;
- non-expression tail → `Unit`, no denotation.

- [ ] **Step 10: Define control-flow join**

For `if let`/branching constructs:
1. merge ordinary types as today;
2. denotation survives only if all reachable result branches contain same denotation;
3. otherwise clear.

Do not union denotations.

- [ ] **Step 11: Run tests**

```bash
cargo test -p phalcom-semantic --test denotation
cargo test -p phalcom-semantic --test phase2_expression_engine
```

- [ ] **Step 12: Commit**

```bash
git add phalcom-semantic/src/types/denotation.rs \
        phalcom-semantic/src/checker \
        phalcom-semantic/tests/denotation.rs
git commit -m "feat(semantic): separate value typing from type-form denotation"
```

---

# Task 5: Make Instance/Class Surfaces, Inheritance, and `super` Semantically Correct

**Files:**
- Modify: `phalcom-semantic/src/surface.rs`
- Modify: `phalcom-semantic/src/dispatch.rs`
- Modify: `phalcom-semantic/src/types/relation.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/typed_expr.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Create: `phalcom-semantic/tests/class_side_dispatch.rs`

**Interfaces:**
- Produces:
  - `MemberSurface`
  - `DispatchOwner`
  - `DispatchLookup`
  - hierarchy `superclass(...)`
  - side-aware field and callable resolution
- Correct generic receiver rule:
  - `List<Int>` instance dispatch delegates to declaration `List`;
  - bare `List :: Type -> Type` is not an ordinary instance receiver;
  - `ClassObject(List)` dispatches class-side.

- [ ] **Step 1: Write failing side/`super` tests**

Tests:
- same selector independently on instance/class side;
- inherited instance method;
- inherited class method;
- inherited instance field;
- inherited class field;
- `ClassObject(Child)` resolves class-side inherited member;
- `Applied(List<Int>)` resolves instance-side `List` member;
- bare constructor-kinded `List` is rejected as ordinary receiver;
- `super` preserves receiver type but starts at parent;
- class-side `super` starts at parent/class side.

- [ ] **Step 2: Split `DeclarationSurface`**

Implement:

```rust
#[derive(Clone, Debug, Default)]
pub struct MemberSurface {
    pub fields: HashMap<String, TypeKnowledge>,
    pub field_ids: HashMap<FieldId, TypeKnowledge>,
    pub callables: HashMap<CallableId, TypeKnowledge>,
    pub callable_signatures: HashMap<Selector, CallableSignature>,
}

#[derive(Clone, Debug, Default)]
pub struct DeclarationSurface {
    pub id: Option<DeclarationId>,
    pub instance: MemberSurface,
    pub class: MemberSurface,
}
```

Add side-aware accessors.

- [ ] **Step 3: Route source members using `ClassMember::is_static()`**

In declaration collection:
- method/getter/setter/field use `member.is_static()`;
- indexer remains instance-side;
- variant does not produce direct callable surface.

- [ ] **Step 4: Add direct superclass API**

Extend trait:

```rust
pub trait TypeHierarchy {
    fn superclass(
        &self,
        declaration: &DeclarationId,
    ) -> Option<&DeclarationId>;

    fn is_subclass(
        &self,
        sub: &DeclarationId,
        sup: &DeclarationId,
    ) -> bool;
}
```

Implement on `MapTypeHierarchy`.

- [ ] **Step 5: Add dispatch request model**

In `dispatch.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchLookup {
    Normal,
    Super {
        defining_class: DeclarationId,
        side: DispatchSide,
    },
}

#[derive(Clone, Debug)]
pub struct DispatchRequest<'a> {
    pub receiver: TypeId,
    pub selector: &'a Selector,
    pub lookup: DispatchLookup,
}
```

`ReceiverDispatch` should not be in the type module.

- [ ] **Step 6: Extend `TypedExpression` with lookup mode only for `super`**

Add optional field:

```rust
pub dispatch_lookup: DispatchLookup
```

Default `Normal`.

`Expr::SuperVar` produces same ordinary value fact as `self`, but sets:

```rust
DispatchLookup::Super {
    defining_class: current_class,
    side: current_side,
}
```

- [ ] **Step 7: Add `current_side` to checker context**

```rust
pub current_side: DispatchSide,
```

Set explicitly while checking each member.

`self` rules:
- instance-side → proper nominal instance type, no denotation;
- class-side → class-object proper type + type-form denotation.

- [ ] **Step 8: Resolve dispatch owner from receiver form**

Implement logic:

```text
TypeData::ClassObject { declaration }
    -> DispatchOwner { declaration, side: Class }

TypeData::Nominal { declaration } with kind Type
    -> DispatchOwner { declaration, side: Instance }

TypeData::Applied { origin, .. } with final kind Type
    -> recursively identify base nominal declaration from origin
    -> DispatchOwner { declaration, side: Instance }
```

Never register every specialization in a map.

A constructor-kinded bare nominal form is not a valid ordinary receiver.

- [ ] **Step 9: Walk inheritance on the same side**

Lookup:
1. choose starting declaration:
   - normal → owner declaration;
   - super → direct superclass of defining class;
2. query selected `MemberSurface`;
3. if missing, walk `hierarchy.superclass()` preserving `DispatchSide`.

Use same algorithm for fields and callables.

- [ ] **Step 10: Keep runtime object model untouched**

Run:

```bash
git diff -- phalcom-core/src/value/repr.rs \
            phalcom-core/src/heap/object.rs \
            phalcom-core/src/heap/class.rs \
            phalcom-core/src/universe/core_classes.rs
```

Expected: no semantic/runtime changes.

- [ ] **Step 11: Run focused tests**

```bash
cargo test -p phalcom-semantic --test class_side_dispatch
scripts/test.sh invariants
```

- [ ] **Step 12: Commit**

```bash
git add phalcom-semantic/src/surface.rs \
        phalcom-semantic/src/dispatch.rs \
        phalcom-semantic/src/types/relation.rs \
        phalcom-semantic/src/checker \
        phalcom-semantic/tests/class_side_dispatch.rs
git commit -m "feat(semantic): model instance and class dispatch sides"
```

---

# Task 6: Complete Source Annotation Lowering Through the Kinded Algebra

**Files:**
- Modify: `phalcom-semantic/src/types/annotation.rs`
- Modify: `phalcom-semantic/src/diagnostic.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Create: `phalcom-semantic/tests/type_annotations.rs`

**Interfaces:**
- Produces:
  - recursive `resolve_type_form(...)`
  - top-level proper-value annotation `resolve_type_annotation(...)`
  - stable kind/application diagnostics

- [ ] **Step 1: Write failing annotation tests**

Cover:
- `List<Int>`;
- `Map<String, Int>`;
- tuple labels;
- callable labels/rest/result;
- union;
- bare `List` rejected as unsaturated value annotation;
- `Int<String>` rejected as non-constructor;
- `List<List>` rejected as argument-kind mismatch;
- unresolved reference retains current unresolved diagnostic.

- [ ] **Step 2: Add `TypeFormResolution`**

Recommended:

```rust
pub enum TypeFormResolution {
    Known(TypeId),
    Dynamic,
    Unknown(UnknownReason),
}
```

`Dynamic` is not a `TypeId`.

- [ ] **Step 3: Add recursive `resolve_type_form`**

Signature:

```rust
pub fn resolve_type_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    current_module: &ModuleId,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormResolution
```

Reference behavior:
- `Never`, `Unit` → proper canonical types;
- `Dynamic` → `Dynamic`;
- declaration → canonical declaration form from table.

- [ ] **Step 4: Lower application**

For `TypeAnnotationExpr::Application`:
1. resolve origin recursively;
2. resolve each argument recursively;
3. require each argument to be a semantic type form;
4. call `store.apply_type_form`;
5. translate application/kind errors to diagnostics.

- [ ] **Step 5: Lower tuple and callable forms**

Tuple:
- lower every child;
- require kind `Type`;
- preserve labels.

Callable:
- lower each parameter;
- preserve label/rest bit;
- lower result;
- all positions must be kind `Type`;
- intern `CallableType`.

- [ ] **Step 6: Keep union normalization canonical**

Every ordinary union member must be proper `Type`.

If explicit `Dynamic` participates, retain current escape behavior; do not invent a special union type for it.

- [ ] **Step 7: Enforce top-level value annotation kind**

`resolve_type_annotation(...) -> TypeKnowledge` wraps `resolve_type_form`.

If result is `Known(form)` but `store.kind_of(form) != KindId::TYPE`, emit:

```text
type.annotation.unsaturated_constructor
```

and return `Unknown`.

- [ ] **Step 8: Add diagnostic codes**

Add:
- `type.kind.expected_type`
- `type.application.not_constructor`
- `type.application.too_many_arguments`
- `type.application.argument_kind_mismatch`
- `type.annotation.unsaturated_constructor`

Preserve:
- `type.annotation.unresolved`
- `type.annotation.unsupported`

`AnnotationUnsupported` must no longer be emitted solely for Application/Tuple/Callable AST variants.

- [ ] **Step 9: Run tests**

```bash
cargo test -p phalcom-semantic --test type_annotations
cargo test -p phalcom-semantic --test checker
scripts/test.sh ast
```

- [ ] **Step 10: Commit**

```bash
git add phalcom-semantic/src/types/annotation.rs \
        phalcom-semantic/src/diagnostic.rs \
        phalcom-semantic/src/checker \
        phalcom-semantic/tests/type_annotations.rs
git commit -m "feat(semantic): lower kind-checked type annotations"
```

---

# Task 7: Add Generic Substitution, Applied Member Views, and Conservative Generic Relations

**Files:**
- Create: `phalcom-semantic/src/types/substitution.rs`
- Modify: `phalcom-semantic/src/types/mod.rs`
- Modify: `phalcom-semantic/src/dispatch.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/types/relation.rs`
- Modify: `phalcom-semantic/src/types/constraint.rs`
- Create: `phalcom-semantic/tests/substitution.rs`

**Interfaces:**
- Produces:
  - `TypeSubstitution`
  - applied-member signature substitution
  - invariant generic relation
  - class-object subtype relation

- [ ] **Step 1: Write failing substitution/relation tests**

Tests:
- direct `T -> Int`;
- nested `List<T>`;
- tuple/record/union/callable substitution;
- partial substitution leaves unbound parameter;
- `Box<T>.get() -> T` viewed on `Box<Int>` returns `Int`;
- `Box<T>.put(T)` viewed on `Box<Int>` accepts `Int`;
- `Box<Int>` not subtype of `Box<Number>` by default;
- `ClassObject(Sub) <: ClassObject(Super)`;
- existing callable variance remains.

- [ ] **Step 2: Add `TypeSubstitution`**

```rust
#[derive(Clone, Debug, Default)]
pub struct TypeSubstitution {
    bindings: HashMap<TypeParameterId, TypeId>,
}
```

Methods:
- `bind`
- `get`
- `apply`

- [ ] **Step 3: Implement recursive substitution**

Handle:
- `Parameter`
- `Applied`
- `Union`
- `Tuple`
- `Record`
- `Callable`

Leave:
- `Never`
- `Unit`
- `Nominal`
- `ClassObject`
- unresolved `Infer`

For `Applied`, rebuild using checked `apply_type_form`, never raw `store.applied`.

- [ ] **Step 4: Keep inference substitution separate**

Update `constraint.rs::substitute_type` to use new checked application APIs, but do not merge its `InferVarId -> TypeId` map with `TypeSubstitution`.

- [ ] **Step 5: Build substitution from declaration signature + applied arguments**

Add helper:

```rust
pub fn substitution_for_applied(
    declarations: &DeclarationTypeTable,
    store: &TypeStore,
    applied: TypeId,
) -> Option<TypeSubstitution>
```

Bind only supplied prefix; leave remaining parameters unbound for partial forms.

- [ ] **Step 6: Apply substitution to callable views lazily**

When dispatch resolves a callable on an applied proper type:
1. resolve declaration/member;
2. derive substitution;
3. clone only returned `CallableSignature`;
4. substitute parameter and return `TypeKnowledge::Known` types;
5. retain same callable identity.

Do not clone whole surface or class.

- [ ] **Step 7: Make applied generic subtyping invariant**

For two applied forms of same canonical base origin and equal arity:
- corresponding arguments must be canonically equal;
- do not recursively subtype arguments.

Do not infer covariance from origin inheritance.

- [ ] **Step 8: Add proper-kind precondition to subtyping**

At public boundary:

```rust
debug_assert!(store.is_proper_type(sub));
debug_assert!(store.is_proper_type(sup));
```

If a checked public API is exposed beyond internal checker use, return a relation error instead of panicking.

- [ ] **Step 9: Add class-object relation**

```text
ClassObject(Sub) <: ClassObject(Super)
iff hierarchy.is_subclass(Sub, Super)
```

This mirrors runtime metaclass inheritance semantically without introducing metaclass `TypeData`.

- [ ] **Step 10: Run tests**

```bash
cargo test -p phalcom-semantic --test substitution
cargo test -p phalcom-semantic --lib types::relation
```

- [ ] **Step 11: Commit**

```bash
git add phalcom-semantic/src/types \
        phalcom-semantic/src/dispatch.rs \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/tests/substitution.rs
git commit -m "feat(semantic): substitute generic applied member views"
```

---

# Task 8: Move Parse-Once Source Ownership into `phalcom-modules`

**Files:**
- Modify: `phalcom-modules/src/source.rs`
- Modify: `phalcom-modules/src/resolver.rs`
- Modify: `phalcom-modules/src/builtin_interface.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Modify: `phalcom-semantic/src/source.rs`
- Add/modify module tests for parse caching.

**Interfaces:**
- Produces:
  - `phalcom_modules::ParsedModuleUnit`
  - `ModuleResolver::load_parsed(...)`
- Semantic workspace consumes `Arc<ParsedModuleUnit>` without reparsing.
- `UnlinkedModuleInterface` remains a derived/cached artifact, not a field embedded in parsed unit.

- [ ] **Step 1: Write failing parse-cache tests**

Test:
- loading parsed module twice returns same cached `Arc`/same parse artifact;
- `load_interface` after `load_parsed` does not parse a second time;
- builtin interface generation uses same parsed artifact;
- invalidation/revision rebuild replaces cache entry when source generation changes.

Use an injectable/test counting source provider or existing resolver test instrumentation; do not add global mutable production counters.

- [ ] **Step 2: Add `ParsedModuleUnit` to `source.rs`**

```rust
#[derive(Clone, Debug)]
pub struct ParsedModuleUnit {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: Option<SourceLocation>,
    pub text: Arc<str>,
    pub program: Arc<Program>,
}
```

`source: Option` supports physical, builtin, synthetic, and inline modules.

- [ ] **Step 3: Re-export it from `phalcom-modules/src/lib.rs`**

Update:

```rust
pub use source::{
    EntryOwnership,
    FilesystemSourceProvider,
    ModuleKind,
    ParsedModuleUnit,
    SourceProvider,
    SourceUnit,
};
```

- [ ] **Step 4: Add parsed cache to `ModuleResolver`**

Use:

```rust
parsed_cache: HashMap<
    ModuleId,
    Result<Arc<ParsedModuleUnit>, ModuleLoadError>,
>,
interface_cache: HashMap<
    ModuleId,
    Result<UnlinkedModuleInterface, ModuleLoadError>,
>,
```

Expose:

```rust
pub fn load_parsed(
    &mut self,
    module: &ModuleId,
) -> Result<Arc<ParsedModuleUnit>, ModuleLoadError>
```

- [ ] **Step 5: Make `load_interface()` project from parsed cache**

`load_interface()`:
1. calls `load_parsed`;
2. builds `InterfaceBuilder::build(...)` from `parsed.program`;
3. caches interface separately.

- [ ] **Step 6: Unify builtin parsing**

`BuiltinInterfaceBuilder` currently parses `source_text()` itself.

Refactor provider/builder so builtin parsed source is obtained through the same parse-once artifact path. Preserve native interface overlay after `InterfaceBuilder::build`.

No duplicate call to `phalcom_ast::parse` for same builtin generation.

- [ ] **Step 7: Remove duplicate semantic parse artifact ownership**

Current `phalcom-semantic/src/source.rs::ParsedSourceUnit` duplicates the desired structure.

Prefer one of:
- re-export/type alias to `phalcom_modules::ParsedModuleUnit`; or
- thin conversion wrapper containing only `Arc<ParsedModuleUnit>`.

Do not retain duplicate `Arc<str>` and `Arc<Program>` ownership.

- [ ] **Step 8: Run tests**

```bash
cargo test -p phalcom-modules
cargo test -p phalcom-semantic --tests
```

- [ ] **Step 9: Commit**

```bash
git add phalcom-modules phalcom-semantic/src/source.rs
git commit -m "refactor(modules): retain parsed module units across analysis"
```

---

# Task 9: Build the Project-Aware Resolver and Semantic Workspace

**Files:**
- Create: `phalcom-semantic/src/resolver.rs`
- Create: `phalcom-semantic/src/workspace.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-semantic/src/checker/mod.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Reuse: `phalcom-modules/src/declaration.rs`
- Reuse: `phalcom-modules/src/graph.rs`
- Create: `phalcom-semantic/tests/workspace.rs`

**Interfaces:**
- Produces:
  - `LinkedTypeResolver`
  - `SemanticWorkspaceInput`
  - `SemanticAnalysis`
  - `analyze_workspace`
  - `analyze_single_module`
  - one coherent static snapshot per generation
- Removes public `check_program` program-level ownership model.

- [ ] **Step 1: Write failing workspace tests**

Required cases:
1. local declaration resolves;
2. selective imported type resolves to target declaration;
3. module-qualified imported type resolves;
4. re-export resolves to original canonical declaration;
5. same leaf name in two modules stays distinct;
6. cross-module superclass works;
7. semantic mutual reference SCC is accepted;
8. inheritance cycle rejected;
9. same imported type form gets same `TypeId` within generation;
10. changing one declaration removes stale old surface on new generation;
11. builtin declaration identity is canonical;
12. diagnostics are grouped by owning `ModuleId`.

- [ ] **Step 2: Implement `LinkedTypeResolver`**

In `resolver.rs`:

```rust
pub struct LinkedTypeResolver {
    // canonical indexes derived from LinkedProgram and declaration table
}
```

Implement existing `TypeResolver` trait.

Resolution order:
1. local declaration;
2. selective import binding;
3. module alias + member;
4. re-export;
5. builtin/prelude declaration;
6. qualified linked path.

Never fallback to a global leaf-name map.

Keep `SimpleTypeResolver` only for isolated unit tests.

- [ ] **Step 3: Define workspace input/output**

```rust
pub struct SemanticWorkspaceInput {
    pub linked: Arc<LinkedProgram>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub generation: u64,
}

pub struct SemanticAnalysis {
    pub snapshot: Arc<SemanticSnapshot>,
}
```

Do not duplicate diagnostics outside the snapshot. The snapshot is the single publication truth.

- [ ] **Step 4: Extend `SemanticSnapshot`**

Add:

```rust
pub declarations: Arc<DeclarationTypeTable>,
pub hierarchy: Arc<MapTypeHierarchy>,
pub diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
pub semantic_graph: Arc<SemanticGraph>,
```

Keep:
- generation;
- store;
- sources;
- surfaces;
- dispatch.

Prefer deterministic maps for published data.

- [ ] **Step 5: Implement workspace Phase A — universe bootstrap**

Create one `TypeStore`.

Register:
- canonical builtin declaration type forms;
- generic signatures;
- native/core hierarchy relationships;
- native/member surfaces.

Native hierarchy must include runtime-authoritative relationships such as:
- `Behavior -> Object`
- `Class -> Behavior`
- `Int -> Number`
- `Float -> Number`
- `Some -> Option`
and the rest already established by native surface metadata.

Do not rely exclusively on source `is` clauses because several bootstrapped classes are runtime-authored.

- [ ] **Step 6: Implement Phase B — predeclare every source declaration**

For all reachable parsed source declarations:
- create canonical `DeclarationId`;
- build `DeclarationBlueprint`;
- call `DeclarationShellTable::predeclare`;
- register source declaration type info.
Current source classes get zero generic parameters because source generic declaration syntax is not introduced here.

- [ ] **Step 7: Implement Phase C — construct `LinkedTypeResolver`**

Build it only after all declaration shells/type infos exist.

- [ ] **Step 8: Implement Phase D — enrich existing semantic graph**

Start from:

```rust
let mut semantic_graph = input.linked.graphs.semantics.clone();
```

Add declaration-level edges:
- superclass references → `SemanticEdgeKind::Superclass`;
- annotation references → `TypeReference`;
- callable type references → appropriate semantic edge where current graph taxonomy supports it;
- future constraints only when source representation actually exists.

Do not add these to runtime initialization graph.

- [ ] **Step 9: Implement Phase E — realize declaration shells**

Call:

```rust
shells.realize_semantic_graph(&semantic_graph)
```

Translate:
- missing shell;
- inheritance cycle
into structured semantic diagnostics or workspace-analysis error as appropriate.

Mutual non-inheritance type references remain legal SCCs.

- [ ] **Step 10: Implement Phase F — build hierarchy**

Resolve direct superclass identities canonically.

No unresolved superclass silently becomes `Object`.

Combine:
- source declared superclass relationships;
- runtime-authoritative native class relationships.

- [ ] **Step 11: Implement Phase G — collect all declaration surfaces before bodies**

Refactor current checker entry so surface collection does not execute method bodies.

Collect:
- instance fields;
- class fields;
- instance callables;
- class callables;
- native surfaces.

Then construct one `SurfaceDispatchResolver`.

- [ ] **Step 12: Implement Phase H — body checking**

For each reachable source module:
- create `CheckingContext` borrowing same store/resolver/declarations/hierarchy;
- check bodies;
- append diagnostics under owning `ModuleId`.

Do not create one `TypeStore` per module.

- [ ] **Step 13: Implement Phase I — local constraint solving**

Keep current `LocalConstraintSolver` per checking context/body as appropriate.

Do not introduce a second global solver in this task. Preserve architecture so future SCC/fixed-point solving can operate over the shared workspace.

- [ ] **Step 14: Implement Phase J — immutable publication**

Only after all tables are coherent:
- freeze store/tables/maps under `Arc`;
- build `SemanticSnapshot`;
- return `SemanticAnalysis { snapshot }`.

- [ ] **Step 15: Replace public `check_program`**

Remove `check_program` from `phalcom-semantic/src/lib.rs` public exports.

Add:

```rust
pub fn analyze_single_module(
    module: ModuleId,
    source: Arc<str>,
    program: Arc<Program>,
) -> SemanticAnalysis
```

It constructs a legitimate one-module linked/workspace context and delegates to `analyze_workspace`.

Do not preserve the old signature that accepts caller-owned store/hierarchy/resolver.

- [ ] **Step 16: Migrate semantic tests**

Whole-program tests use:
- `analyze_single_module`, or
- explicit `analyze_workspace`.

Low-level algebra tests may still instantiate `TypeStore`/`CheckingContext` directly.

- [ ] **Step 17: Run tests**

```bash
cargo test -p phalcom-semantic --test workspace
cargo test -p phalcom-semantic --tests
cargo test -p phalcom-modules
```

- [ ] **Step 18: Commit**

```bash
git add phalcom-semantic phalcom-modules
git commit -m "feat(semantic): analyze linked workspaces in one semantic generation"
```

---

# Task 10: Add Stable Export Descriptors and the `AnalyzedProgram` Compiler Seam

**Files:**
- Create: `phalcom-semantic/src/export.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-core/src/modules/compile.rs`
- Modify: `phalcom-core/src/modules/artifact.rs` only if needed
- Create: `phalcom-semantic/tests/export.rs`
- Add/modify compiler integration tests.

**Interfaces:**
- Produces:
  - `CompiledKindRef`
  - `CompiledTypeRef`
  - `CompiledTypeParameterOwner`
  - `export_kind`
  - `export_type_form`
  - `AnalyzedProgram`
  - `ProgramSemanticDiagnostics`
  - `ProgramCompileError::Semantic`

- [ ] **Step 1: Write failing export tests**

Cover:
- nominal export;
- applied export;
- union;
- tuple;
- record;
- callable;
- stable parameter owner/index;
- kind arrow export;
- inference variable export rejection;
- `ClassObject` export rejection as `NonExportableTypeForm`;
- no exported enum contains raw `TypeId`/`KindId`/`TypeParameterId`.

- [ ] **Step 2: Define stable kind descriptors**

In `export.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledKindRef {
    Type,
    Arrow {
        parameters: Box<[CompiledKindRef]>,
        result: Box<CompiledKindRef>,
    },
}
```

- [ ] **Step 3: Define stable type-form descriptors**

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledTypeRef {
    Never,
    Unit,
    Nominal(DeclarationId),
    Applied {
        origin: Box<CompiledTypeRef>,
        arguments: Box<[CompiledTypeRef]>,
    },
    Union(Box<[CompiledTypeRef]>),
    Tuple(Box<[CompiledTupleElement]>),
    Record(Box<[CompiledRecordField]>),
    Callable(CompiledCallableType),
    Parameter {
        owner: CompiledTypeParameterOwner,
        index: u16,
    },
}
```

Do not add `ClassObject`.

- [ ] **Step 4: Define export error**

Use semantically precise name:

```rust
pub enum SemanticExportError {
    InferenceVariable(InferVarId),
    NonExportableTypeForm { form: TypeId },
}
```

`ClassObject` hits `NonExportableTypeForm`.

- [ ] **Step 5: Implement exporters**

```rust
pub fn export_kind(
    store: &TypeStore,
    kind: KindId,
) -> CompiledKindRef

pub fn export_type_form(
    store: &TypeStore,
    form: TypeId,
) -> Result<CompiledTypeRef, SemanticExportError>
```

Stable parameter identity:
- declaration owner + index;
- callable owner + index.

- [ ] **Step 6: Add `AnalyzedProgram`**

In compiler/front-end seam:

```rust
pub struct AnalyzedProgram {
    pub project_universe: Arc<ProjectUniverse>,
    pub linked: Arc<LinkedProgram>,
    pub semantic: Arc<phalcom_semantic::SemanticSnapshot>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub entry: ModuleId,
}
```

Exact module placement may be `phalcom-core/src/modules/compile.rs` initially; split later only if file size demands it.

- [ ] **Step 7: Add analysis entry seam**

Prefer:

```rust
pub struct ProgramAnalyzer;

impl ProgramAnalyzer {
    pub fn analyze_entry_selection(
        entry: EntrySelection,
    ) -> Result<AnalyzedProgram, ProgramCompileError>;
}
```

This function owns:
- project/source discovery;
- parse-once module acquisition;
- linking;
- `analyze_workspace`.

`ProgramCompiler` should compile an `AnalyzedProgram`, not rediscover/check separately.

- [ ] **Step 8: Make compilation stages explicit**

Target flow:

```text
EntrySelection
    -> ProgramAnalyzer::analyze_entry_selection
    -> AnalyzedProgram
    -> ProgramCompiler::compile_analyzed
    -> CompiledProgram
```

If `ProgramCompiler::compile_entry_selection` remains public for compatibility, implement it by calling analyzer then `compile_analyzed`.

- [ ] **Step 9: Replace program semantic error variant**

Define:

```rust
#[derive(Clone, Debug, Default)]
pub struct ProgramSemanticDiagnostics {
    pub by_module: BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
}
```

Helpers:
- `is_empty`
- `has_errors`
- `for_module`
- `iter`

Replace:

```rust
ProgramCompileError::Type(Vec<SemanticDiagnostic>)
```

with:

```rust
ProgramCompileError::Semantic(ProgramSemanticDiagnostics)
```

- [ ] **Step 10: Remove `run_semantic_typecheck`**

Delete the ad-hoc local helper.

All:
- Project
- Package
- Module
- standalone Module
- Inline

must use the analyzer seam.

- [ ] **Step 11: Preserve parsed body ownership**

`AnalyzedProgram.sources` provides the body ASTs required by semantic analysis and compilation. Do not attempt to recover bodies from `LinkedProgram` interfaces.

- [ ] **Step 12: Run tests**

```bash
cargo test -p phalcom-semantic --test export
cargo test -p phalcom-core
```

Add compiler integration cases:
- project dependency type error caught;
- package dependency type error caught;
- inline mismatch caught;
- standalone mismatch caught;
- clean project compiles.

- [ ] **Step 13: Commit**

```bash
git add phalcom-semantic/src/export.rs \
        phalcom-semantic/tests/export.rs \
        phalcom-core/src/modules
git commit -m "feat(compiler): compile only from analyzed semantic programs"
```

---

# Task 11: Make `phalcom check` Reuse the Program Analyzer

**Files:**
- Modify: `phalcom-core/bin/phalcom/cli.rs`
- Add/modify CLI tests.

**Interfaces:**
- Consumes: `ProgramAnalyzer::analyze_entry_selection`
- Produces: one CLI semantic-check path; no checker-specific project discovery.

- [ ] **Step 1: Write failing CLI tests**

Cases:
- inline source mismatch;
- standalone file mismatch;
- file inside project resolves project context;
- project dependency mismatch appears;
- clean project returns success;
- JSON/text diagnostics include module/source ownership.

- [ ] **Step 2: Change `cmd_check` to analyzer seam**

Do not call `analyze_workspace` directly from CLI.

Convert CLI target into `EntrySelection`, then:

```rust
let analyzed = ProgramAnalyzer::analyze_entry_selection(entry)?;
```

Inspect `analyzed.semantic.diagnostics`.

- [ ] **Step 3: Render `ProgramSemanticDiagnostics` consistently**

Use the same grouping/ordering as compiler error rendering.

Do not flatten module ownership away.

- [ ] **Step 4: Correct stale CLI docs**

Remove syntax-only wording.

Do not add `--types=strict` unless separately ratified.

- [ ] **Step 5: Run CLI/core tests**

```bash
cargo test -p phalcom-core
```

- [ ] **Step 6: Commit**

```bash
git add phalcom-core/bin/phalcom/cli.rs
git commit -m "feat(cli): check linked projects with shared semantic analysis"
```

---

# Task 12: Publish Static Semantic Snapshots and Diagnostics Through LSP Without Replacing `ValueShape`

**Files:**
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/diagnostics.rs`
- Modify worker/database state files that own snapshot publication.
- Reuse: `phalcom-lsp/src/semantic/ids.rs::DocumentModuleMap`
- Preserve: `phalcom-lsp/src/semantic/snapshot.rs`

**Interfaces:**
- Consumes:
  - `phalcom_semantic::SemanticSnapshot`
  - canonical `phalcom_modules::ModuleId`
  - existing `DocumentModuleMap`
- Produces:
  - published static semantic diagnostics
  - coherent advisory + static snapshot generations

- [ ] **Step 1: Write failing LSP integration tests**

Cases:
1. static mismatch publishes `source = "phalcom-typecheck"`;
2. semantic diagnostic code preserved;
3. fix clears diagnostic;
4. syntax + semantic diagnostics coexist when parse recovery permits;
5. syntax-invalid new revision clears stale semantic diagnostics;
6. imported type mismatch resolves project identity;
7. edit exported declaration updates dependent module diagnostics;
8. unrelated module unaffected;
9. existing advisory semantic tests continue passing;
10. snapshot generation/source revisions remain coherent.

- [ ] **Step 2: Name the two snapshot domains explicitly**

In LSP code use qualified names or aliases:

```rust
type AdvisorySemanticSnapshot = crate::semantic::SemanticSnapshot;
type StaticSemanticSnapshot = phalcom_semantic::SemanticSnapshot;
```

Never replace advisory snapshot with static snapshot.

- [ ] **Step 3: Add static snapshot publication state to analysis worker**

The worker owns:
- current advisory snapshot;
- current `Arc<StaticSemanticSnapshot>`;
- source revision/generation stamp used to produce both.

No request handler performs workspace type checking synchronously.

- [ ] **Step 4: Reuse `DocumentModuleMap` for URI identity**

URI → canonical semantic `phalcom_modules::ModuleId` comes from existing map.

Do not introduce a new filename-to-module-ID scheme.

Builtin URIs use canonical builtin identity.

- [ ] **Step 5: Run static workspace analysis after coalesced edits**

Worker flow:

```text
ingest edits
  -> parse/update advisory source products
  -> resolve module/project identities
  -> build/update linked semantic workspace inputs
  -> analyze_workspace
  -> publish immutable static snapshot
  -> emit publication event
```

Use latest-wins worker behavior already present.

- [ ] **Step 6: Publish combined diagnostics**

`backend.rs::publish_diagnostics_for`:
- syntax diagnostics;
- semantic diagnostics for same source revision/generation;
- clear previously published semantic diagnostics by sending empty list when clean.

Do not publish stale static diagnostics after a syntax-invalid replacement.

- [ ] **Step 7: Fix related-information URI ownership**

`diagnostics.rs` must map labels to actual current/cross-module URI using `DocumentModuleMap`.

Do not emit placeholder `file:///`.

- [ ] **Step 8: Preserve advisory `ValueShape` behavior**

No conversion such as:

```text
ValueShape -> TypeId
```

is introduced.

Hover/completion may continue using advisory engine until separately migrated to consume static facts intentionally.

- [ ] **Step 9: Run LSP tests**

```bash
cargo test -p phalcom-lsp
scripts/test.sh lsp
```

- [ ] **Step 10: Commit**

```bash
git add phalcom-lsp
git commit -m "feat(lsp): publish shared static semantic diagnostics"
```

---

# Task 13: Full Semantic, Object-Model, Determinism, and Performance Verification

**Files:**
- Modify tests/docs only where failures expose genuine missing coverage.
- Do not add runtime semantic changes to satisfy static tests.

**Interfaces:**
- Consumes all previous tasks.
- Produces merge-ready acceptance evidence.

- [ ] **Step 1: Run semantic focused suite**

```bash
cargo test -p phalcom-semantic --tests
```

Expected: all old and new semantic tests pass.

- [ ] **Step 2: Run modules/native suites**

```bash
cargo test -p phalcom-modules
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-macros
```

- [ ] **Step 3: Run compiler/core suite**

```bash
cargo test -p phalcom-core
```

- [ ] **Step 4: Run LSP suite**

```bash
cargo test -p phalcom-lsp
scripts/test.sh lsp
```

- [ ] **Step 5: Run AST/no-new-syntax gate**

```bash
scripts/test.sh ast
```

Verify no parser/AST syntax extension was introduced for:
- generic class declarations;
- type lambdas;
- kind annotations;
- higher-kinded declarations.

- [ ] **Step 6: Run runtime object-model invariant gate**

```bash
scripts/test.sh invariants
```

Additionally inspect:

```bash
git diff -- phalcom-core/src/value/repr.rs \
            phalcom-core/src/value/mod.rs \
            phalcom-core/src/heap/object.rs \
            phalcom-core/src/heap/class.rs \
            phalcom-core/src/universe/core_classes.rs
```

Expected:
- no runtime type/kind object variants;
- no `Value` layout change;
- no class/metaclass bootstrap semantic change.

- [ ] **Step 7: Run workspace/full gates**

```bash
scripts/test.sh workspace
scripts/test.sh full
```

- [ ] **Step 8: Run formatting/lint/docs**

```bash
cargo fmt --check
cargo clippy --workspace
cargo doc --workspace --no-deps
```

- [ ] **Step 9: Verify deterministic outputs**

Add/run tests that analyze same workspace twice in fresh stores and compare:
- structural exported `CompiledTypeRef`/`CompiledKindRef`;
- diagnostic module ordering;
- diagnostic codes/ranges;
- canonical display strings if diagnostics expose them.

Do not assert cross-process persistence of raw `TypeId` integer values.

- [ ] **Step 10: Verify parse-once behavior**

Use resolver/builtin cache tests to prove:
- source revision parses once;
- interface building projects from parsed AST;
- semantic workspace reuses same `Arc<Program>`;
- builtin interface generation no longer reparses same module independently.

- [ ] **Step 11: Verify dependent invalidation**

LSP/workspace test:
1. module A exports class/signature used by B;
2. analyze;
3. modify A;
4. rebuild;
5. B diagnostic/fact updates;
6. unrelated C remains stable where current invalidation granularity permits.

- [ ] **Step 12: Verify kind/application invariants directly**

Required assertions:
- `Int :: Type`;
- `List :: Type -> Type`;
- `Map :: Type -> Type -> Type`;
- `Map<String> :: Type -> Type`;
- `Map<String, Int> :: Type`;
- `List<List>` wrong-kind;
- `apply(apply(Map, String), Int) == apply(Map, [String, Int])`.

- [ ] **Step 13: Verify two-axis facts**

Required assertions:
- `1 : Int`, no denotation;
- `Int : ClassObject(Int)`, denotes `Int :: Type`;
- `List : ClassObject(List)`, denotes `List :: Type -> Type`;
- class-side `self` is class-object typed and denotes declaration form;
- `super` preserves receiver and changes lookup start.

- [ ] **Step 14: Record static-analysis performance evidence**

Measure before/after or current milestone baseline for:
- `cargo test -p phalcom-semantic --tests`;
- representative multi-module `phalcom check`;
- LSP edit-to-static-publication benchmark/counter.

Requirements:
- `kind_of` O(1) dense indexing;
- no per-query workspace reparsing;
- no whole-surface clone per generic specialization;
- no runtime performance tax from this milestone.

Any material regression requires profiling note before merge.

- [ ] **Step 15: Run graphify impact checks**

Before final sign-off:

```bash
graphify query "TypeStore KindId semantic checker declaration surface project typing" --budget 2000
graphify affected "TypeStore"
graphify affected "TypedExpression"
graphify affected "DeclarationSurface"
graphify path "ProgramCompiler" "SemanticSnapshot"
graphify update . --no-cluster
```

Use repository-current equivalent if command syntax changed.

- [ ] **Step 16: Search for forbidden legacy patterns**

Run:

```bash
rg 'unwrap_or\(KindId::TYPE\)' phalcom-semantic
rg 'store\.nominal\(.*List|store\.nominal\(.*Map|store\.nominal\(.*Set|store\.nominal\(.*Option' phalcom-semantic
rg 'ProgramCompileError::Type' .
rg 'check_program' phalcom-semantic phalcom-core phalcom-lsp
rg 'file:///' phalcom-lsp/src/diagnostics.rs
```

Expected:
- no kind fallback;
- no generic collection bare-nominal reconstruction in production;
- no old compile error variant;
- no public old `check_program`;
- no placeholder diagnostic URI.

- [ ] **Step 17: Final commit**

```bash
git add .
git commit -m "test(semantic): verify two-axis semantic tower integration"
```

---

# Cross-Task Interface Summary

The following names and meanings are fixed by this plan so later tasks do not independently rename/reinterpret them.

## Type/kind core

```rust
TypeId
KindId
TypeData::ClassObject { declaration: DeclarationId }

TypeStore::kind_of(TypeId) -> KindId
TypeStore::is_proper_type(TypeId) -> bool
TypeStore::arrow_kind(Box<[KindId]>, KindId) -> KindId
TypeStore::apply_kind(KindId, &[KindId]) -> Result<KindId, KindApplicationError>
TypeStore::nominal_form(DeclarationId, KindId) -> TypeId
TypeStore::nominal_type(DeclarationId) -> TypeId
TypeStore::class_object_type(DeclarationId) -> TypeId
TypeStore::apply_type_form(TypeId, &[TypeId]) -> Result<TypeId, TypeApplicationError>
```

## Declaration/generics

```rust
TypeParameterOwner
TypeParameterData
GenericSignature
DeclarationTypeInfo
DeclarationTypeTable
```

## Expression semantics

```rust
SemanticDenotation::TypeForm(TypeId)
SemanticDenotation::Kind(KindId)

ValueSemanticFact {
    knowledge: TypeKnowledge,
    denotation: Option<SemanticDenotation>,
}
```

## Dispatch

```rust
DispatchSide::{Instance, Class}

DispatchLookup::{
    Normal,
    Super {
        defining_class: DeclarationId,
        side: DispatchSide,
    },
}

DispatchOwner {
    declaration: DeclarationId,
    side: DispatchSide,
}
```

## Project analysis

```rust
LinkedTypeResolver
SemanticWorkspaceInput
SemanticAnalysis { snapshot: Arc<SemanticSnapshot> }
analyze_workspace(...)
analyze_single_module(...)
```

`check_program` is removed from the public program-level API.

## Compiler seam

```rust
AnalyzedProgram
ProgramAnalyzer::analyze_entry_selection(...)
ProgramCompiler::compile_analyzed(...)
ProgramSemanticDiagnostics
ProgramCompileError::Semantic(...)
```

## Stable export

```rust
CompiledKindRef
CompiledTypeRef
CompiledTypeParameterOwner
SemanticExportError::InferenceVariable(...)
SemanticExportError::NonExportableTypeForm { ... }
export_kind(...)
export_type_form(...)
```

`CompiledTypeRef` does not contain `ClassObject`.

---

# Semantic Acceptance Matrix

| Expression/form | Ordinary value type | Denotation | Kind of denoted form |
|---|---|---|---|
| `1` | `Int` | none | n/a |
| `"x"` | `String` | none | n/a |
| `Int` | internal `ClassObject(Int)` | `TypeForm(Int)` | `Type` |
| `List` | internal `ClassObject(List)` | `TypeForm(List)` | `Type -> Type` |
| `List<Int>` annotation | n/a as runtime value in this milestone | canonical type form | `Type` |
| `Map<String>` internal type form | n/a | canonical type form | `Type -> Type` |
| class-side `self` in `List` | `ClassObject(List)` | `TypeForm(List)` | `Type -> Type` |
| instance-side `self` in `List` | saturated proper instance type in context | none | `Type` |
| `super` | same as `self` | same as `self` | unchanged; only lookup start changes |

---

# Deferred Semantics — Must Not Be Invented During Execution

The implementation must stop at internal representation boundaries instead of deciding any of these:

- source syntax for generic class parameters;
- source syntax for higher-kinded parameters;
- type lambdas;
- kind polymorphism;
- user-defined kinds;
- `Constraint` kind;
- `Any`/top type;
- intersections;
- `Self` full type semantics beyond current required checker behavior;
- F-bounds;
- declaration-site variance syntax/semantics;
- structural protocol conformance;
- type aliases;
- ADT exhaustiveness;
- type-based method overload resolution;
- runtime type/kind reflection objects;
- per-instance reified generic arguments;
- runtime generic validation;
- `None` generic-constructor semantics.

---

# Completion Definition

The milestone is complete only when all of the following are simultaneously true:

- [ ] Every canonical `TypeId` has one explicit `KindId`; no fallback exists.
- [ ] `TypeKnowledge::Known` is only used for proper `Type`-kinded value types.
- [ ] Bare generic class forms such as `List` are constructor-kinded, not ordinary instance types.
- [ ] Class-object static types are distinct from nominal type forms.
- [ ] Class-name expressions expose class-object value type + type-form denotation.
- [ ] Denotation is flow-safe and never stale after assignment/join.
- [ ] Generic application is canonical and kind-checked.
- [ ] Partial application works internally.
- [ ] Native generic metadata lowers through the same canonical algebra.
- [ ] `Option<T>`/`Some<T>` metadata is kind-consistent.
- [ ] Instance/class surfaces are separate.
- [ ] Generic applied receivers dispatch through origin declaration without registering every specialization.
- [ ] Inherited fields and callables work on both sides.
- [ ] `super` preserves receiver and changes lookup start.
- [ ] Generic substitution is separate from inference substitution.
- [ ] Generic declaration parameters are invariant by default.
- [ ] Project-wide resolution uses `LinkedTypeResolver`, not `SimpleTypeResolver`.
- [ ] Existing `SemanticGraph`/`DeclarationShellTable` are reused for SCC realization.
- [ ] Parsing is retained once per module revision and reused by interfaces/checker.
- [ ] `check_program` is removed from public whole-program API.
- [ ] Compiler flow explicitly includes `AnalyzedProgram`.
- [ ] `ProgramCompileError::Semantic` retains module-owned diagnostics.
- [ ] `phalcom check` reuses program analysis entry selection.
- [ ] `CompiledTypeRef`/`CompiledKindRef` contain no raw semantic IDs and exclude `ClassObject`.
- [ ] LSP publishes shared static semantic diagnostics.
- [ ] Existing advisory LSP `ValueShape` engine remains separate.
- [ ] Runtime heap/object/metaclass semantics are unchanged.
- [ ] All focused, workspace, LSP, invariant, formatting, lint, documentation, and full test gates pass.
- [ ] Graphify indexes/impact checks are refreshed.
- [ ] Performance evidence shows no unexplained material static-analysis regression and no runtime tax.

The resulting dependency direction must be:

```text
phalcom-ast
    owns syntax

phalcom-modules
    owns source/module identity, parse-once artifacts, interfaces,
    linking, declaration shells, dependency graphs

phalcom-semantic
    owns canonical type forms, kinds, denotation, generic substitution,
    relations, declaration typing, project checking, static snapshots,
    stable semantic export descriptors

phalcom-core
    analyzes entry selections, compiles AnalyzedProgram, and later may
    consume stable descriptors for runtime reification

phalcom-lsp
    consumes phalcom-semantic static snapshots for static diagnostics
    while retaining its separate advisory runtime-shape semantic engine
```

And the final architectural invariant remains:

> **Runtime reflection may expose Phalcom’s semantic tower, but it may not define a second one.**
