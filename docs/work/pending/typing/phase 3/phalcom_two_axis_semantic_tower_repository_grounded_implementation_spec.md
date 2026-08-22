# Phalcom Two-Axis Semantic Tower
## Repository-Grounded Implementation Specification

**Status:** implementation specification
**Repository:** `aureat/phalcom-lang`
**Repository baseline inspected:** `ab9142700e53fb012a5da71a1074bd67ec8f44b4` (`docs: add static typing requirements analysis`)
**Investigation date:** 2026-08-22
**Primary normative semantic source:** `docs/spec/typing/ontology.md`
**Implementation target:** Phalcom semantic/type-analysis infrastructure, compiler/checker integration, project/module analysis, LSP consumption, and stable future-reification boundary
**Runtime type/kind reflection:** deliberately **not implemented by this plan**; its semantic contract is fixed here so a later runtime-reflection implementation reuses the semantic engine rather than inventing another type system.

---

# 1. Executive summary

Phalcom already contains both halves of the intended ontology, but they currently exist as largely independent mechanisms:

```text
RUNTIME / OBJECT AXIS                    STATIC / SEMANTIC AXIS
=====================                    ======================

Value                                     expression/value fact
  │                                         │
  │ .class                                  │ :
  ▼                                         ▼
ClassObject                               TypeId
  │                                         │
  │ .class                                  │ ::
  ▼                                         ▼
metaclass                                KindId
```

The runtime half is mature. `phalcom-core` implements the Smalltalk-style class/metaclass tower, `Value::class()` is the runtime classification operation, and class-side dispatch follows the parallel metaclass hierarchy. This plan **must not replace, wrap, flatten, emulate, or duplicate that tower**.

The semantic half is young but already has the correct seeds: canonical `TypeId`, `KindId`, `KindData::Type`, `KindData::Arrow`, interned `TypeData`, epistemic `TypeKnowledge`, semantic snapshots, declaration identities, semantic dependency graphs, and a first functional checker. The main problem is that the first checker still treats nearly every `TypeId` as if it had kind `Type`, treats a class-name expression as if its ordinary value type were the nominal instance type it denotes, has only instance-side semantic surfaces, and checks one program in isolation.

This implementation establishes the two axes as explicit, orthogonal semantic machinery:

```text
expression e  ⇓  value v
value v       :  proper type T
TypeForm F    :: kind K
```

and adds a separate denotation relation:

```text
class object Int   --denotes-->  nominal type form Int :: Type
class object List  --denotes-->  type constructor List :: Type -> Type
```

The term **`TypeForm`** is used for the common semantic/behavioral category of values that denote type-level forms. `Type` is reserved for the atomic kind. The stale typing documents that use `@protocol class Type` are not implementation authority for this work; where they conflict with `docs/spec/typing/ontology.md` and this ratified architecture, the ontology wins.

The implementation is organized into ten work packages:

1. Reconcile documentation and freeze the semantic contract.
2. Make kinds total and make type application kind-correct.
3. Add declaration type forms, generic signatures, and core generic metadata without inventing source syntax.
4. Separate ordinary value type from semantic denotation.
5. Make semantic member surfaces and dispatch correctly distinguish instance side and class side.
6. Complete existing annotation lowering through the canonical type/kind algebra.
7. Add generic substitution, applied-member views, and safe generic subtyping.
8. Build one project/module-aware semantic analysis pipeline over existing linker graphs and declaration shells.
9. Make compiler, CLI, and LSP consume the shared semantic analysis and add a stable compiled semantic-descriptor boundary.
10. Execute exhaustive correctness, object-model regression, incremental, performance, and repository verification gates.

The completed milestone does **not** add `AppliedType`, `UnionType`, `AtomicKind`, or `FunctionKind` runtime heap objects. It makes their future meaning fully determined by canonical semantic data so that runtime reflection later becomes a reification/presentation layer only.

---

# 2. Normative semantic decisions

These decisions are locked for this implementation. An implementation agent must not reinterpret them as suggestions.

## 2.1 One value/object universe, multiple semantic relations

Phalcom keeps one runtime value/object universe while maintaining distinct relations:

```text
e ⇓ v     expression evaluation
v : T     ordinary value typing
T :: K    kind classification
```

Runtime classification remains:

```text
v.class
```

`.class`, `:`, and `::` are not aliases and must never be implemented as aliases.

## 2.2 `Type` is the atomic kind

Examples:

```text
Int            :: Type
String         :: Type
List<Int>      :: Type
List           :: Type -> Type
Map            :: Type -> Type -> Type
Map<String>    :: Type -> Type
```

`Type` is not the common behavioral protocol formerly proposed in stale typing documentation.

## 2.3 `TypeForm` is the common type-denoting abstraction

Use **`TypeForm`** as the architectural name for a value/entity that denotes a canonical type-level form.

Examples of eventual surface values satisfying/playing this role:

```text
Class object Int
Class object List
TypeParameter
AppliedType
UnionType
TupleType
RecordType
CallableType
```

`TypeForm` is **not** a superclass inserted above `Class`, and this milestone does not need to introduce a surface `@protocol class TypeForm` yet. Runtime protocol/reflection exposure is deferred.

## 2.4 `TypeId` identifies canonical type-level forms

`TypeId` is retained. Do not introduce a second `TypeFormId` solely to distinguish constructors from proper types.

The clarified invariant is:

> A `TypeId` identifies one canonical type-level form within one `TypeStore`. Its `KindId` determines whether that form is a proper type (`Type`) or a constructor/higher-kinded form (`K1 -> K2`).

Therefore all of the following may be `TypeId`s:

```text
Int
List
List<Int>
Map
Map<String>
T
F
Int | String
(Int, String)
Int -> String
```

but only forms whose `kind_of(id) == KindId::TYPE` are valid as ordinary value types in `TypeKnowledge::Known`.

## 2.5 Class objects directly denote type forms

No `ClassType` wrapper is permitted.

```text
runtime class object Int
    denotes
semantic nominal form Int :: Type

runtime class object List
    denotes
semantic constructor List :: Type -> Type
```

The exact class object remains the eventual reflected value for a bare nominal/class type form.

## 2.6 A class object’s ordinary value type is distinct from the type form it denotes

The current checker conflates these. The new model does not.

```text
expression 42
    ordinary value type = Int
    denotation          = none

expression Int
    ordinary value type = ClassObject(Int)     // internal semantic form; no new syntax
    denotation          = Int :: Type

expression List
    ordinary value type = ClassObject(List)
    denotation          = List :: Type -> Type
```

`ClassObject(Int)` in this document names an **internal static type representation**, not a wrapper allocated at runtime and not a proposed `Class<Int>` source spelling.

## 2.7 Denotation is first-class semantic data

Expression analysis must be able to represent both:

```text
ordinary type knowledge
optional type/kind denotation
```

A type-denoting value is still an ordinary Phalcom value; its denotation is additional semantic information.

## 2.8 Type application is canonical, kind-checked, and non-overridable

The semantic operation behind existing generic-application syntax is trusted language semantics. It must not invoke arbitrary user method dispatch during static analysis.

No new syntax is introduced by this implementation.

The semantic kernel must support partial application:

```text
Map                 :: Type -> Type -> Type
Map<String>         :: Type -> Type
Map<String, Int>    :: Type
```

even if a particular source context does not yet admit an unsaturated constructor.

## 2.9 Higher-kinded architecture now; higher-kinded syntax later

The representation must permit:

```text
F      :: Type -> Type
Higher :: (Type -> Type) -> Type
```

but this plan does not add syntax for declaring higher-kinded parameters, type lambdas, kind variables, `Constraint`, or kind polymorphism.

## 2.10 Runtime reflection is downstream

Future runtime reflection must follow this dependency direction:

```text
source / declarations / inference
              │
              ▼
       phalcom-semantic
              │
              ├── TypeStore / TypeId
              ├── KindId / KindData
              ├── generic application
              ├── substitution
              ├── relation solving
              ├── declaration surfaces
              └── semantic snapshots
              │
              ▼
 stable VM-independent semantic descriptors
              │
              ▼
      future runtime reification
              │
              ▼
 ordinary Phalcom values/objects
```

Reflection must never parse display strings, rediscover generic arity from runtime method shape, or construct an independent type algebra.

---

# 3. Non-negotiable prohibitions

Implementation must not do any of the following:

1. Add a `ClassType` wrapper around class objects.
2. Introduce a second runtime object universe for types or kinds.
3. restructure the existing `Object` / `Behavior` / `Class` / `Metaclass` tower.
4. Treat `.class`, value typing, and kind classification as the same relation.
5. Persist raw `TypeId` or `KindId` numeric IDs into compiled/runtime artifacts.
6. Clone runtime classes to represent generic specializations.
7. Add generic type metadata to every ordinary runtime instance.
8. Treat `Unknown` as a semantic type.
9. Treat `Dynamic` as a fake universal nominal class.
10. Add `Any`, intersections, `Self`, F-bounds, type lambdas, kind polymorphism, or other unratified features merely because the new architecture could support them.
11. Add new surface syntax in order to make an internal test convenient.
12. Put types into selector identity or ordinary dispatch keys.
13. Add type-based overload resolution.
14. Automatically validate runtime values because an annotation exists.
15. Implement static class-side dispatch by inventing semantic metaclass objects; represent the side explicitly and mirror the runtime lookup rule.
16. Implement `super` by changing the receiver to an instance/class object of the superclass. Runtime `super` keeps the receiver and changes the lookup start; static semantics must model the same fact.
17. Delete or reinterpret the LSP `ValueShape` domain as the language type system. Its source explicitly says it is advisory and not a language type.
18. Make runtime reflection a prerequisite for completing the semantic tower.

If implementation reaches a point where a public syntax or semantic law not fixed above is required, stop at an internal API boundary and mark the surface decision deferred.

---

# 4. Repository state and evidence

All paths in this section refer to baseline `ab9142700e53fb012a5da71a1074bd67ec8f44b4`.

## 4.1 Normative ontology already exists

`docs/spec/typing/ontology.md` defines the one-value-universe/multiple-level model, the `e ⇓ v`, `v : T`, `T :: K` judgments, class/type distinctions, and the eventual reflection bridge.

Relevant anchors:

- `docs/spec/typing/ontology.md:1+` — one value universe, value/type/kind ladder.
- `docs/spec/typing/ontology.md` section **Two orthogonal structures** — runtime object model vs semantic model.
- `docs/spec/typing/ontology.md` section **Reflection** — class/type/kind reification examples.

The older `docs/spec/typing/01-protocol-foundation.md`, `02-type-expression-foundation.md`, `03-type-parameters-and-generic-signatures.md`, `STATUS.md`, and related example copies predate this ontology. In particular, Document 02 defines `Type` as a protocol. That naming/level decision is superseded for this implementation.

## 4.2 AST already represents several rich annotation forms

`phalcom-ast/src/ast.rs:405-446` already contains:

```rust
pub enum TypeAnnotationExpr {
    Reference(StaticSymbolRef),
    Application { ... },
    Union { ... },
    Tuple { ... },
    Callable { ... },
}
```

Therefore generic application, tuple, and callable annotations do **not** require new syntax or new AST variants for this milestone.

Class members already retain `is_static`/`@class` information; semantic collection currently fails to use it consistently.

## 4.3 Type store has the right structural forms but incorrect kind assumptions

`phalcom-semantic/src/types/store.rs:32+` contains `TypeData::{Never, Unit, Nominal, Applied, Union, Tuple, Record, Callable, Parameter, Infer}`.

`phalcom-semantic/src/types/store.rs:52+` contains separate kind interning and `type_kinds`.

The critical current shortcuts are in `phalcom-semantic/src/types/store.rs:100-190`:

- `kind_of()` silently falls back to `KindId::TYPE`.
- `nominal()` always stamps `Type`.
- `applied()` always stamps `Type`.
- tuple/record/callable/inference constructors manually stamp kinds after generic `intern()`.

This is the first implementation target.

## 4.4 Kind representation already supports arrows

`phalcom-semantic/src/types/kind.rs` currently contains:

```rust
pub enum KindData {
    Type,
    Arrow { parameters: Box<[KindId]>, result: KindId },
}
```

No new fundamental kind AST is required for `Type`, `Type -> Type`, multi-argument constructor kinds, or higher-kinded parameter kinds.

## 4.5 Evidence already separates Known / Unknown / Dynamic

`phalcom-semantic/src/types/evidence.rs` contains:

```rust
pub enum TypeKnowledge {
    Known(TypeEvidence),
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}
```

This distinction must be preserved. `TypeKnowledge` will continue to answer the **ordinary value typing** question; denotation becomes a separate field/domain.

## 4.6 Typed expressions currently have no denotation

`phalcom-semantic/src/checker/typed_expr.rs:1-78` currently stores:

```rust
pub struct TypedExpression {
    pub knowledge: TypeKnowledge,
    pub constraints: Vec<TypeConstraint>,
    pub provenance: EvidenceSet,
}
```

There is no representation for “this value denotes type form `Int`” or “this value reifies kind `Type`.”

## 4.7 Class-name expressions are currently conflated with instance types

`phalcom-semantic/src/checker/expression.rs`, `Expr::Var`, currently does:

```rust
if let Some(decl) = ctx.resolver.resolve_type_name(...) {
    let ty = ctx.store.nominal(decl);
    TypedExpression::known(ty, ...)
}
```

Thus source expression `Int` is assigned ordinary type `Int`. Under the two-axis model it must instead have an internal class-object value type and a separate `Int :: Type` denotation.

`Expr::SelfVar`/`Expr::SuperVar` also currently ignore instance-vs-class side.

## 4.8 Semantic surfaces are currently instance-side-only in practice

`phalcom-semantic/src/surface.rs` has one `fields` map and one `callable_signatures` map. `add_field()` and `add_callable()` create identities using `DispatchSide::Instance` unconditionally.

`phalcom-semantic/src/identity.rs` already defines:

```rust
pub enum DispatchSide {
    Instance,
    Class,
}
```

The missing capability is therefore propagation and indexing, not a new object-model concept.

`phalcom-semantic/src/checker/declaration.rs:1-180` currently reads class members but does not route `FieldDef.is_static`, `MethodDef.is_static`, `GetterDef.is_static`, or `SetterDef.is_static` into separate surfaces.

## 4.9 Semantic dispatch currently ignores side and inheritance

`phalcom-semantic/src/dispatch.rs` contains `DispatchTarget { selector, side }`, but the actual `SurfaceDispatchResolver::resolve_dispatch(receiver, selector)` maps `TypeId -> DeclarationId` and checks one surface without a side parameter or superclass walk.

`CheckingContext::resolve_dispatch` in `phalcom-semantic/src/checker/context.rs` has a temporary applied-type rule: if the receiver is `TypeData::Applied`, retry lookup on its origin. No generic substitution occurs.

## 4.10 Current generic subtyping is implicitly covariant

`phalcom-semantic/src/types/relation.rs`, `TypeData::Applied` branch, currently accepts applied subtyping when origins subtype and every corresponding type argument subtypes.

That is an accidental covariance rule. Until declaration-site variance is implemented explicitly, generic arguments must be invariant.

## 4.11 Source annotation resolution is incomplete despite AST support

`phalcom-semantic/src/types/annotation.rs` currently resolves references and unions, then emits `AnnotationUnsupported` for applications, tuples, and callables.

The implementation should complete the existing forms instead of extending grammar.

## 4.12 Native metadata already has structural type specifications

`phalcom-native-meta/src/types.rs` contains:

```rust
pub enum TypeExprSpec {
    Unknown,
    Never,
    SelfType,
    Universe(UniverseKey),
    Parameter(&'static str),
    Applied { ... },
    Union(...),
    Tuple(...),
}
```

and callable/type-parameter specs.

`phalcom-semantic/src/types/native.rs` currently normalizes only a subset and maps the rest to `UnknownReason::OpaqueNative`.

Core generic/type-form metadata should extend this existing machinery, not introduce hardcoded checker-only generic classes.

## 4.13 Core collection classes do not yet declare source generics

For example, `phalcom-core/core/universe/src/collections/list.ph` begins:

```phalcom
class List {
    ...
}
```

There is no generic parameter declaration in current source. This plan therefore **must not invent or require class generic-declaration syntax**. Core generic signatures are supplied as trusted semantic/native metadata until a separately approved source-generic declaration implementation lands.

## 4.14 Project/module graph infrastructure already exists

`phalcom-modules/src/graph.rs` already has:

- `ReferenceGraph`
- `SemanticGraph`
- `RuntimeDependencyGraph`
- declaration-capable `SemanticNodeId`
- `SemanticEdgeKind::{ModuleInterface, TypeReference, Superclass, ProtocolReference, ConstraintReference, CallbackSignature, AdtReference}`
- deterministic SCC computation.

`phalcom-modules/src/declaration.rs` already has `DeclarationShellTable`, which predeclares stable declaration identities and realizes semantic SCCs while separately rejecting inheritance cycles.

Do not create a checker-private project graph.

## 4.15 Linker semantic graph is not yet declaration-complete

`phalcom-modules/src/linker.rs:300+` currently populates module-interface semantic edges for imports. Declaration-level type/superclass edges are not yet populated from type annotations/declarations.

The semantic workspace may extend a clone of the linked semantic graph with declaration edges; it must use the existing graph types and SCC algorithm.

## 4.16 Compiler type checking is currently single-program/local

`phalcom-core/src/modules/compile.rs:~513+`, `run_semantic_typecheck`, creates a fresh `TypeStore`, `MapTypeHierarchy`, and `SimpleTypeResolver`, hardcodes core names, scans the supplied one program, and calls `check_program`.

`ProgramCompiler::discover_and_link` (`phalcom-core/src/modules/compile.rs:360-450`) links all reachable modules and constructs `CompiledProgram` without invoking project-wide semantic analysis.

## 4.17 CLI check still invokes the one-program helper

`phalcom-core/bin/phalcom/cli.rs:304+`, `cmd_check`, parses one source and calls:

```rust
run_semantic_typecheck(&phalcom_modules::ModuleId::core(), &program)
```

Its rustdoc still says syntax-only.

## 4.18 LSP has the adapter but does not publish static semantic diagnostics

`phalcom-lsp/src/diagnostics.rs:35+` already converts `SemanticDiagnostic` into LSP diagnostics.

`phalcom-lsp/src/backend.rs:306+`, `publish_diagnostics_for`, publishes syntax diagnostics only.

`phalcom-lsp/src/semantic/facts.rs` explicitly says `ValueShape` is an advisory runtime shape and “deliberately not a language type.” Preserve this separate domain.

## 4.19 Runtime object model is already the source of truth

`docs/spec/current/object-model.md` fixes:

- every surface object has exactly one class;
- every class is an object with a metaclass;
- class-side dispatch uses the same lookup algorithm beginning at the metaclass;
- metaclass inheritance mirrors class inheritance;
- the tower closes through `Metaclass` / `Metaclass class`.

`phalcom-core/src/heap/class.rs`, `phalcom-core/src/universe/core_classes.rs`, and `Value::class()` implement this model.

Current semantic work must preserve it. No runtime type-reflection classes are added in this milestone.

---

# 5. Target architecture after this implementation

```text
                              SOURCE / AST
                                  │
                                  ▼
                    linked declaration identities
                                  │
                                  ▼
             ┌────────────────────────────────────┐
             │        SEMANTIC WORKSPACE           │
             │                                    │
             │  DeclarationTypeTable              │
             │  TypeStore                         │
             │  KindStore (inside TypeStore)      │
             │  Type hierarchy                    │
             │  instance/class surfaces           │
             │  semantic graph + SCCs             │
             │  diagnostics                       │
             └────────────────────────────────────┘
                        │                 │
                        │                 │
                     value : T         T :: K
                        │                 │
                        ▼                 ▼
                  TypeKnowledge        KindId
                        │
                        │ separate from
                        ▼
                SemanticDenotation
                  ├─ TypeForm(TypeId)
                  └─ Kind(KindId)


       expression Int
             │
             ├─ ordinary type ──▶ ClassObject(Int) :: Type
             │
             └─ denotes ─────────▶ Int :: Type

       expression List
             │
             ├─ ordinary type ──▶ ClassObject(List) :: Type
             │
             └─ denotes ─────────▶ List :: Type -> Type

       annotation List<Int>
             │
             └─ canonical type application
                        │
                        ▼
                  List<Int> :: Type
```

The future runtime reflection bridge begins only after this semantic representation is stable:

```text
Semantic TypeId / KindId
        │
        │ export structural, stable metadata
        ▼
CompiledTypeRef / CompiledKindRef
        │
        │ future runtime materialization
        ▼
existing Class object OR synthetic descriptor object
```

---

# 6. Work package 0 — documentation reconciliation and semantic guardrails

## Goal

Make it impossible for implementation agents to follow the stale `@protocol Type` design or accidentally treat runtime reflection as the type system.

## Files to modify

### `docs/spec/typing/ontology.md`

Add a short normative terminology/precedence section near the beginning:

```markdown
## Normative terminology and precedence

- `Type` is the atomic kind of proper types.
- `TypeForm` names the common semantic/behavioral role of values that denote
  type-level forms.
- `TypeForm` is not a superclass inserted into the object hierarchy.
- `TypeDescriptor` is reserved as a future implementation base for synthetic
  reflected type-form values.
- Where older typing design documents define `Type` as a protocol or otherwise
  collapse value type and kind levels, this ontology supersedes those passages.
- Runtime reflection reifies this semantic model; it does not define it.
```

Also add the eight ratified decisions from Section 2 as a compact invariant block.

### `docs/spec/typing/README.md`

Change the status page so `ontology.md` is listed as the current semantic foundation. Mark Documents 01–03 as historical/stale where they conflict with the ontology. Do not delete them; preserve design history.

### `docs/spec/typing/STATUS.md`

Add a supersession banner at the top. Do not continue calling the old `Type`-protocol decision “locked.”

### `docs/spec/typing/02-type-expression-foundation.md`

Add a top-of-file notice only; do not rewrite the historical document in this change:

```markdown
> Superseded terminology: the current ontology reserves `Type` for the atomic
> kind and uses `TypeForm` for the common type-denoting role. See `ontology.md`.
```

Apply the same notice to copied/example versions if they are still retained as agent references.

## Tests/verification

No code test. Require repository search after edits:

```sh
rg '@protocol\s+class Type|`Type` is a signature-only `Protocol`' docs/spec/typing examples/phalcom-typing
```

Every remaining match must either be in an explicitly superseded historical document or intentionally discussing the history.

## Acceptance criteria

- Agents have one current semantic authority.
- `Type` cannot be interpreted as both atomic kind and behavioral protocol by reading current docs.
- No surface runtime `TypeForm` class/protocol is added yet.

---

# 7. Work package 1 — total kinded `TypeStore` and canonical type application

## Goal

Make every canonical type form have exactly one explicit kind and make application derive kinds rather than assuming saturation.

## Files to modify

- `phalcom-semantic/src/types/kind.rs`
- `phalcom-semantic/src/types/store.rs`
- `phalcom-semantic/src/types/id.rs` documentation
- `phalcom-semantic/src/types/mod.rs`
- `phalcom-semantic/src/lib.rs`

## New file

Recommended:

- `phalcom-semantic/src/types/application.rs`

Keep application/kind-check errors out of `store.rs` if the store would otherwise become too large.

## 7.1 Redefine `TypeId` documentation

In `types/id.rs`, replace “canonical type identifier” wording with the precise lifetime/meaning:

```rust
/// Snapshot/store-local canonical identifier for a type-level form.
///
/// A `TypeId` may identify a proper type (`kind == Type`) or an unsaturated
/// type constructor/higher-kinded form. The associated `KindId` determines
/// which. IDs are meaningful only with the `TypeStore` that allocated them.
pub struct TypeId(pub u32);
```

Do not make the ID globally stable.

## 7.2 Make type-kind storage total

Current:

```rust
type_kinds: HashMap<TypeId, KindId>
```

Replace with a dense parallel vector:

```rust
type_kinds: Vec<KindId>
```

Invariant:

```text
types.len() == type_kinds.len()
```

Every type interning path must assign its kind at creation.

Replace `intern(TypeData)` + `set_kind()` with one private operation:

```rust
fn intern_with_kind(&mut self, data: TypeData, kind: KindId) -> TypeId
```

Behavior:

1. If `data` is new, append both `data` and `kind`.
2. If `data` is already interned, verify its stored kind equals `kind`.
3. A mismatched kind is an internal programming invariant violation, never a user-source diagnostic.
4. User-source paths must validate kinds before calling this function, so malformed source cannot trigger an internal panic.

Delete public `set_kind()`.

Replace:

```rust
pub fn kind_of(&self, ty: TypeId) -> KindId {
    self.type_kinds.get(&ty).copied().unwrap_or(KindId::TYPE)
}
```

with total indexing:

```rust
pub fn kind_of(&self, ty: TypeId) -> KindId {
    self.type_kinds[ty.index()]
}
```

There must be no “missing kind means `Type`” behavior.

## 7.3 Canonical arrow-kind construction

Add to `kind.rs` / `TypeStore`:

```rust
pub fn arrow_kind(&mut self, parameters: impl Into<Box<[KindId]>>, result: KindId) -> KindId
```

Canonicalization rules:

- zero parameters normalize to `result`;
- right-associated result arrows flatten:

```text
Arrow([Type], Arrow([Type], Type))
==
Arrow([Type, Type], Type)
```

- arrow kinds used as **parameters** do not flatten into outer parameters:

```text
(Type -> Type) -> Type
```

must remain:

```text
Arrow([Arrow([Type], Type)], Type)
```

This makes `Type -> Type -> Type` one canonical n-ary representation while preserving higher-order parameter kinds.

## 7.4 Kind application

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum KindApplicationError {
    NotApplicable { kind: KindId },
    TooManyArguments { supplied: usize, accepted: usize },
    ArgumentKindMismatch {
        index: usize,
        expected: KindId,
        actual: KindId,
    },
}

pub fn apply_kind(
    &mut self,
    callee: KindId,
    arguments: &[KindId],
) -> Result<KindId, KindApplicationError>
```

Rules:

```text
Type applied to anything              -> NotApplicable
(Type -> Type) [Type]                 -> Type
(Type -> Type -> Type) [Type]         -> Type -> Type
(Type -> Type -> Type) [Type, Type]   -> Type
(Type -> Type) [Type, Type]           -> TooManyArguments
((Type -> Type) -> Type) [Type]       -> ArgumentKindMismatch
```

Exact `KindId` equality is enough in this milestone. Kind subtyping and kind polymorphism are deferred.

## 7.5 Add internal class-object proper type

Extend `TypeData`:

```rust
/// Proper static type of the runtime class-object value for `declaration`.
/// This is not a runtime wrapper and has no surface syntax in this milestone.
ClassObject { declaration: DeclarationId },
```

Its kind is always `Type`.

Add:

```rust
pub fn class_object_type(&mut self, declaration: DeclarationId) -> TypeId
```

This form describes ordinary values such as class object `Int`. It is **not** the nominal type form denoted by `Int`.

## 7.6 Make nominal-form construction kind-explicit

Current `nominal(declaration)` cannot remain the API for both `Int` and `List`, because `List` may be `Type -> Type`.

Recommended API:

```rust
pub fn nominal_form(&mut self, declaration: DeclarationId, kind: KindId) -> TypeId
pub fn nominal_type(&mut self, declaration: DeclarationId) -> TypeId
```

`nominal_type()` is only a convenience for declarations proven to have kind `Type`. Production declaration resolution should normally obtain the canonical form from the declaration type table introduced in work package 2.

Migrate ambiguous production uses of `store.nominal(decl)` to one of:

- declaration table lookup, or
- explicit `nominal_type(decl)` where the declaration is known non-generic/proper.

Do not allow whichever call happens first to accidentally determine a declaration’s kind.

## 7.7 Checked canonical type application

In `types/application.rs` add:

```rust
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TypeApplicationError {
    NotAConstructor { origin: TypeId, kind: KindId },
    TooManyArguments { supplied: usize, accepted: usize },
    ArgumentKindMismatch {
        index: usize,
        expected: KindId,
        actual: KindId,
    },
}

pub fn apply_type_form(
    store: &mut TypeStore,
    origin: TypeId,
    arguments: &[TypeId],
) -> Result<TypeId, TypeApplicationError>
```

or make the function a `TypeStore` method if preferred.

Rules:

1. `arguments.is_empty()` returns `origin`.
2. Read `origin_kind = store.kind_of(origin)`.
3. Apply argument kinds through `apply_kind`.
4. Flatten nested `TypeData::Applied` into one base origin plus one ordered argument vector.
5. Intern the flattened `TypeData::Applied` with the computed residual/result kind.
6. Partial application is allowed internally.
7. Applying a proper `Type` form to arguments is rejected.
8. Applying an argument of the wrong kind is rejected before interning.
9. Canonical identity must satisfy:

```text
apply(apply(Map, [String]), [Int])
===
apply(Map, [String, Int])
```

at the `TypeId` level.

## 7.8 Proper-type constructors

`union`, `tuple`, `record`, and `callable` are all proper types in this milestone.

Before interning, validate that every child position which semantically expects a value type has kind `Type`:

- union members: `Type`
- tuple elements: `Type`
- record fields: `Type`
- callable parameter types: `Type`
- callable result: `Type`

Do not silently admit an unsaturated constructor such as bare `List` as a tuple element type.

## Tests — new `phalcom-semantic/tests/kinds.rs`

Required tests:

1. `KindId::TYPE` remains canonical ID 0.
2. identical arrow kinds intern to same `KindId`.
3. right-associated arrows normalize.
4. higher-order arrow parameters do not flatten incorrectly.
5. every interned `TypeId` has a kind; no fallback path exists.
6. `ClassObject(Int)` has kind `Type` and is distinct from nominal form `Int`.
7. `Int :: Type`.
8. fake `List :: Type -> Type`.
9. fake `Map :: Type -> Type -> Type`.
10. full application yields `Type`.
11. partial application yields residual arrow kind.
12. nested/one-shot application canonicalize to the same `TypeId`.
13. wrong-kind argument is rejected.
14. too many arguments rejected.
15. proper type used as constructor rejected.
16. tuple/record/callable/union reject constructor-kinded children.

## Acceptance criteria

- No `unwrap_or(KindId::TYPE)` or equivalent remains in kind lookup.
- `TypeStore` cannot create an `Applied` form without deriving its kind.
- `TypeId` can correctly represent constructors.
- No source syntax or runtime object change occurred.

---

# 8. Work package 2 — declaration type forms, type parameters, and trusted core generic signatures

## Goal

Give every class/protocol-capable declaration one canonical type-form identity, class-object proper type, kind, and generic parameter list without introducing generic declaration syntax.

## New files

- `phalcom-semantic/src/types/parameter.rs`
- `phalcom-semantic/src/declarations.rs`

## Files to modify

- `phalcom-semantic/src/types/mod.rs`
- `phalcom-semantic/src/lib.rs`
- `phalcom-semantic/src/types/store.rs`
- `phalcom-native-meta/src/types.rs`
- `phalcom-native-meta/src/universe.rs`
- `phalcom-semantic/src/types/native.rs`

## 8.1 Type-parameter identity

Define:

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
```

`TypeParameterId` remains store/snapshot-local. Stable identity for serialization is owner + declaration index, not the raw integer.

Extend `TypeStore` with canonical parameter storage:

```rust
type_parameters: Vec<TypeParameterData>,
parameter_to_id: HashMap<(TypeParameterOwner, u16), TypeParameterId>,
```

Add APIs:

```rust
pub fn intern_type_parameter(&mut self, data: TypeParameterData) -> TypeParameterId
pub fn type_parameter(&self, id: TypeParameterId) -> &TypeParameterData
pub fn parameter_form(&mut self, id: TypeParameterId) -> TypeId
```

`parameter_form(id)` uses the parameter’s declared kind, not always `Type`.

## 8.2 Declaration typing table

Define:

```rust
#[derive(Clone, Debug)]
pub struct GenericSignature {
    pub owner: TypeParameterOwner,
    pub parameters: Box<[TypeParameterId]>,
}

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

Core invariant:

```text
entries[d].kind == store.kind_of(entries[d].form)
store.kind_of(entries[d].class_object_type) == Type
```

For a declaration with parameter kinds `[K1, ..., Kn]`:

```text
declaration form kind = K1 -> ... -> Kn -> Type
```

For zero parameters:

```text
kind = Type
```

The class-object type remains kind `Type` regardless of declaration genericity.

## 8.3 Do not add source generic syntax here

Current `ClassDef` has no generic-parameter field. Leave it that way in this work package.

Every current source class is registered with zero semantic type parameters unless a separately existing trusted metadata path says otherwise.

This is intentional architecture-first support, not a claim that generic source declarations are unsupported forever.

## 8.4 Trusted builtin generic signatures

Add explicit VM-free/native semantic metadata for core type-form signatures.

Recommended additions in `phalcom-native-meta/src/types.rs`:

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

Store a canonical table next to `UNIVERSE_BINDINGS`, for example:

```rust
pub const UNIVERSE_TYPE_FORMS: &[UniverseTypeFormSpec] = ...;
```

Initial generic metadata required by current semantics:

```text
List<T>       :: Type -> Type
Set<T>        :: Type -> Type
Map<K, V>     :: Type -> Type -> Type
Option<T>     :: Type -> Type
```

Only add additional generic core declarations when repository semantics already establish them. Do not infer genericity from class names or collection behavior.

This metadata does **not** change runtime class layout or `list.ph` syntax.

## 8.5 Bootstrap declaration forms once per semantic store

Add a helper such as:

```rust
pub fn register_universe_type_forms(
    store: &mut TypeStore,
    declarations: &mut DeclarationTypeTable,
    universe_decl: impl Fn(UniverseKey) -> DeclarationId,
)
```

For every universe class:

1. allocate declared parameter IDs from metadata;
2. construct the declaration kind;
3. intern nominal form with that kind;
4. intern class-object proper type;
5. record `DeclarationTypeInfo`.

Non-listed universe classes receive zero parameters and kind `Type`.

## Tests — new `phalcom-semantic/tests/declaration_types.rs`

Required:

- `Int` declaration form kind `Type`.
- `List` form kind `Type -> Type`.
- `Map` form kind `Type -> Type -> Type`.
- `List` class-object proper type kind `Type`.
- declaration form and class-object type are distinct IDs/data.
- `List` owns one parameter with stable owner/index.
- `Map` owns two ordered parameters.
- repeated universe bootstrap is deterministic/canonical within a store.
- source class without approved generic syntax receives kind `Type`.

## Acceptance criteria

- No checker hardcode decides “List is generic because its name is List.”
- Core genericity comes from authoritative native semantic metadata.
- User source grammar remains unchanged.

---

# 9. Work package 3 — denotation and two-axis expression facts

## Goal

Make the semantic engine able to answer two independent questions for an expression:

1. What ordinary type does its runtime value have?
2. Does the value denote a type form or reify a kind?

## New file

- `phalcom-semantic/src/types/denotation.rs`

## Files to modify

- `phalcom-semantic/src/checker/typed_expr.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/statement.rs`
- `phalcom-semantic/src/types/mod.rs`
- `phalcom-semantic/src/lib.rs`

## 9.1 Denotation enum

Add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticDenotation {
    /// Value denotes a canonical type-level form.
    TypeForm(TypeId),
    /// Value reifies a canonical kind. Runtime kind values are deferred, but
    /// the semantic domain is defined now.
    Kind(KindId),
}
```

Do not call the first variant `Type(TypeId)`; that wording encourages confusion between proper types and constructor-kinded type forms.

## 9.2 Compact value fact for environments

Do not store an entire occurrence-specific `TypedExpression` in lexical environments. Introduce:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueSemanticFact {
    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
}
```

`TypedExpression` becomes:

```rust
pub struct TypedExpression {
    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub constraints: Vec<TypeConstraint>,
    pub provenance: EvidenceSet,
    pub receiver_dispatch: ReceiverDispatch,
}
```

The final `receiver_dispatch` field is described below for correct `super` semantics.

All existing constructors default `denotation = None` and `receiver_dispatch = Normal`.

Add methods:

```rust
pub fn with_denotation(self, denotation: SemanticDenotation) -> Self
pub fn fact(&self) -> ValueSemanticFact
pub fn type_form_denotation(&self) -> Option<TypeId>
pub fn kind_denotation(&self) -> Option<KindId>
```

`TypedExpression::ty()` remains the ordinary value type accessor.

## 9.3 Local environments preserve denotation

Change `LocalEnv`:

```rust
bindings: HashMap<String, ValueSemanticFact>
```

Change:

```rust
bind_local(name, TypeKnowledge)
lookup_local(name) -> &TypeKnowledge
```

into fact-aware APIs.

Keep convenience methods for callers that only need `.knowledge` if useful.

### Binding behavior

For:

```phalcom
const t = Int
```

the binding must retain both:

```text
ordinary type = ClassObject(Int)
denotation    = TypeForm(Int)
```

If an initializer has no denotation, binding denotation is `None`.

Annotations constrain ordinary value typing; they do not synthesize denotations.

## 9.4 Class-name expression semantics

In `Expr::Var` resolution:

1. lexical binding lookup still wins;
2. if a declaration name resolves as a class/type-form declaration:
   - ordinary value type = `DeclarationTypeInfo.class_object_type`;
   - denotation = `SemanticDenotation::TypeForm(DeclarationTypeInfo.form)`.

Thus:

```text
Int
  : ClassObject(Int)
  denotes Int :: Type

List
  : ClassObject(List)
  denotes List :: Type -> Type
```

This is the central code change that makes the two-axis ontology real.

## 9.5 Literal semantics stay ordinary

Examples:

```text
1
  : Int
  denotation = none

"x"
  : String
  denotation = none
```

The literal branches should continue returning proper nominal types from the declaration table.

## 9.6 Track current dispatch side in checker context

Add to `CheckingContext`:

```rust
pub current_side: DispatchSide,
```

Default at module scope may be `Instance` only as an inert value; it must be explicitly set while checking a member.

For each class member:

```text
@/is_static false -> DispatchSide::Instance
@/is_static true  -> DispatchSide::Class
```

## 9.7 Correct `self`

Inside instance-side member of `C`:

```text
self
  ordinary type = C (proper nominal instance type)
  denotation    = none
```

Inside class-side member of `C`:

```text
self
  ordinary type = ClassObject(C)
  denotation    = TypeForm(C declaration form)
```

This exactly matches runtime reality: the receiver of a class-side send is the class object.

## 9.8 Correct `super` without violating the object model

Current checker approximates `super` as `Object`/current nominal type. Replace that model.

Runtime Phalcom semantics keep `self` as receiver and only change lookup start. Static semantics must do the same.

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiverDispatch {
    Normal,
    Super {
        defining_class: DeclarationId,
        side: DispatchSide,
    },
}
```

`Expr::SuperVar` returns the **same ordinary value type and denotation as `self`**, plus `ReceiverDispatch::Super`.

Message-send synthesis detects this marker and asks dispatch resolution to begin at the direct superclass of the defining class, on the same side.

Do not bind `super` as a normal lexical value; the dispatch marker is ephemeral expression semantics.

## Tests — new `phalcom-semantic/tests/denotation.rs`

Required:

- `1` -> ordinary `Int`, no denotation.
- `Int` -> ordinary `ClassObject(Int)`, denotes nominal `Int :: Type`.
- `List` -> ordinary `ClassObject(List)`, denotes constructor `List :: Type -> Type`.
- `const t = Int; ...t...` preserves denotation.
- ordinary unknown value has no fake denotation.
- `Dynamic` knowledge does not imply denotation.
- class-side `self` is a class-object type and denotes its declaration form.
- instance-side `self` is the instance type and has no type-form denotation.
- `super` preserves receiver type/denotation while changing lookup start.

## Acceptance criteria

- No class-name expression is typed as if it were an instance of itself.
- Denotation survives straightforward lexical binding.
- No runtime object/code changes required.

---

# 10. Work package 4 — side-correct declaration surfaces and dispatch

## Goal

Make the static semantic model reflect the runtime object model’s two dispatch starting points without constructing a duplicate metaclass tower.

## Files to modify

- `phalcom-semantic/src/surface.rs`
- `phalcom-semantic/src/dispatch.rs`
- `phalcom-semantic/src/checker/declaration.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/types/relation.rs`

## 10.1 Split declaration member surfaces by side

Recommended shape:

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

APIs must require a side:

```rust
pub fn side(&self, side: DispatchSide) -> &MemberSurface
pub fn side_mut(&mut self, side: DispatchSide) -> &mut MemberSurface
pub fn add_field(&mut self, side: DispatchSide, ...)
pub fn get_field(&self, side: DispatchSide, ...)
pub fn add_callable(&mut self, side: DispatchSide, ...)
pub fn get_callable(&self, side: DispatchSide, ...)
```

`FieldId` / `CallableId` already contain `DispatchSide`; populate them correctly.

## 10.2 Register class members on the correct side

In `checker/declaration.rs`:

- `FieldDef.is_static` controls side.
- `MethodDef.is_static` controls side.
- `GetterDef.is_static` controls side.
- `SetterDef.is_static` controls side.
- index methods remain instance-side under current AST semantics.

No selector encoding change.

## 10.3 Extend hierarchy API with direct parent lookup

Current `TypeHierarchy` only provides `is_subclass` while `MapTypeHierarchy` already owns a direct-parent map.

Add:

```rust
fn superclass(&self, declaration: &DeclarationId) -> Option<DeclarationId>;
```

or an equivalent borrowed API compatible with trait object lifetimes.

`is_subclass` can continue to be a convenience built from this relation.

## 10.4 Dispatch owner must include side

Replace the effective mapping:

```text
TypeId -> DeclarationId
```

with:

```rust
pub struct DispatchOwner {
    pub declaration: DeclarationId,
    pub side: DispatchSide,
}
```

Register:

```text
proper instance nominal C     -> (C, Instance)
class-object proper type C    -> (C, Class)
```

A constructor-kinded bare type form like `List :: Type -> Type` is a denotation and is not itself used as the ordinary receiver value type. `ClassObject(List)` is the class-side receiver type.

## 10.5 Hierarchy lookup mirrors runtime, not metaclass objects

For ordinary instance lookup:

```text
C / Instance
C.superclass / Instance
...
```

For class-side lookup:

```text
C / Class
C.superclass / Class
...
```

This second traversal is the semantic mirror of runtime:

```text
(C class).superclass == (C.superclass) class
```

No `MetaclassType`, no semantic metaclass graph, and no duplicated class objects are introduced.

## 10.6 `super` lookup start

When `ReceiverDispatch::Super { defining_class, side }` is present, begin lookup at `hierarchy.superclass(defining_class)`, preserving the receiver’s actual ordinary type/denotation.

## Tests — new `phalcom-semantic/tests/class_side_dispatch.rs`

Required:

- same selector may exist independently on instance and class side.
- instance receiver resolves only instance-side entry.
- class-object receiver resolves only class-side entry.
- inherited instance method resolves through declaration superclass chain.
- inherited class-side method resolves through same declaration chain/side and matches runtime parallel-rule expectation.
- instance-side `super` begins above defining class while keeping receiver.
- class-side `super` begins above defining class on class side while keeping class-object receiver.
- selector identity is unchanged by side or type annotations.

## Runtime regression requirement

No changes should be required in:

- `phalcom-core/src/heap/class.rs`
- `phalcom-core/src/universe/core_classes.rs`
- `phalcom-core/src/value/mod.rs`

If an implementation attempts to modify runtime metaclass wiring to make static checking work, reject the approach.

---

# 11. Work package 5 — complete annotation lowering through the kinded algebra

## Goal

Use already-parsed annotation syntax to construct canonical, kind-checked type forms.

## Files to modify

- `phalcom-semantic/src/types/annotation.rs`
- `phalcom-semantic/src/diagnostic.rs`
- `phalcom-semantic/src/checker/declaration.rs`
- `phalcom-semantic/src/checker/statement.rs`

## 11.1 Separate “resolve a type form” from “resolve a value annotation”

An annotation syntax node can contain an origin that is constructor-kinded even though the completed annotation must classify values.

Introduce a recursive helper conceptually equivalent to:

```rust
fn resolve_type_form(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    current_module: &ModuleId,
    annotation: &TypeAnnotation,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> TypeFormResolution
```

`resolve_type_annotation(...) -> TypeKnowledge` becomes the outer value-annotation operation and verifies that a successfully resolved form has kind `Type`.

Do not place constructor-kinded `TypeId`s into `TypeKnowledge::Known`.

## 11.2 Reference resolution

For ordinary declaration references:

```text
Int   -> declaration table form Int :: Type
List  -> declaration table form List :: Type -> Type
```

At recursive type-form level either is legal.

At outer value-annotation level:

```text
x: Int        legal
x: List       error: unsaturated type constructor
```

unless a future language decision explicitly makes constructor-kinded annotations meaningful in that position.

Preserve existing handling of `Never`, `Unit`, and explicit `Dynamic` unless this implementation requires a direct conflict. Do not add `Any`.

## 11.3 Application resolution

For `TypeAnnotationExpr::Application`:

1. resolve origin as a type form;
2. resolve every argument as a type form;
3. call canonical checked `apply_type_form`;
4. translate kind/application failures into stable semantic diagnostics;
5. outer value-annotation resolution requires final kind `Type`.

Examples:

```text
List<Int>            legal
Map<String, Int>     legal
List<List>            kind mismatch: T expects Type, List has Type -> Type
Int<String>           origin is not a constructor
Map<String>           internally resolves to Type -> Type, then outer annotation rejects unsaturated form
```

## 11.4 Tuple resolution

Lower each `TypeTupleElement` to `TupleTypeElement` and call `store.tuple` after verifying each child is kind `Type`.

Preserve labels exactly.

## 11.5 Callable resolution

Lower `TypeCallableParameter` values to `CallableParameterType`, preserving:

- label;
- type;
- rest bit.

Resolve result as proper `Type`, then intern `CallableType`.

This uses the existing AST form. No new callable syntax.

## 11.6 Union resolution

Continue canonical union normalization. Require each ordinary member form to have kind `Type`.

Do not redesign `Dynamic` union behavior in this work package. Preserve the current explicit-escape compatibility behavior and leave the full special-type lattice to its own design.

## 11.7 Diagnostics

Add stable codes to `DiagnosticCode`:

```text
type.kind.expected_type
    outer annotation or child position requires kind Type

type.application.not_constructor
    attempted application of a non-arrow kind

type.application.too_many_arguments
    supplied more type arguments than current constructor accepts

type.application.argument_kind_mismatch
    argument kind differs from corresponding parameter kind

type.annotation.unsaturated_constructor
    top-level value annotation resolved to an arrow kind
```

Keep existing:

```text
type.annotation.unresolved
type.annotation.unsupported
```

`AnnotationUnsupported` should no longer be emitted merely because a parsed annotation is `Application`, `Tuple`, or `Callable`.

Diagnostics must contain the source range of the offending origin/argument where the AST provides it.

## Tests — extend `phalcom-semantic/tests/checker.rs` and add `type_annotations.rs`

Required:

- `const xs: List<Int> = ...` resolves without “unsupported annotation.”
- `Map<String, Int>` resolves.
- tuple annotation resolves, labels preserved.
- callable annotation resolves, labels/rest preserved.
- union normalization still passes existing tests.
- `List<List>` emits argument-kind mismatch.
- `Int<String>` emits not-constructor.
- bare `List` in ordinary value annotation emits unsaturated-constructor.
- unresolved reference retains existing diagnostic.
- no parser changes are necessary for these cases.

---

# 12. Work package 6 — generic substitution, applied member views, and safe subtyping

## Goal

Make applied types semantically useful without cloning declarations/classes or pretending all generic parameters are covariant.

## New file

- `phalcom-semantic/src/types/substitution.rs`

## Files to modify

- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/dispatch.rs`
- `phalcom-semantic/src/types/relation.rs`
- `phalcom-semantic/src/types/constraint.rs` only to reuse/shared helpers, not to merge generic and inference substitutions.

## 12.1 Keep two substitution domains distinct

Current `LocalConstraintSolver` owns:

```text
InferVarId -> TypeId
```

Generic substitution needs:

```text
TypeParameterId -> TypeId
```

Do not reuse one map type and blur their lifetimes.

Define:

```rust
#[derive(Clone, Debug, Default)]
pub struct TypeSubstitution {
    bindings: HashMap<TypeParameterId, TypeId>,
}
```

Add:

```rust
pub fn bind(&mut self, parameter: TypeParameterId, argument: TypeId)
pub fn get(&self, parameter: TypeParameterId) -> Option<TypeId>
pub fn apply(&self, store: &mut TypeStore, form: TypeId) -> TypeId
```

## 12.2 Recursive substitution coverage

`apply()` must recursively handle:

- `Parameter`
- `Applied` using checked canonical application
- `Union`
- `Tuple`
- `Record`
- `Callable`

and leave unchanged:

- `Never`
- `Unit`
- `Nominal`
- `ClassObject`
- unresolved `Infer` unless inference solver is invoked separately.

Do not resolve `InferVarId` through generic substitution.

## 12.3 Derive substitution from applied form

For canonical applied form:

```text
Origin<A, B>
```

read `Origin`’s `GenericSignature` and bind the applied prefix in declaration order.

Partial application produces a partial substitution; unbound parameters remain parameter forms.

## 12.4 Applied member view

Replace the current “if applied, dispatch on origin unchanged” shortcut.

Lookup process:

1. resolve base origin declaration and side;
2. resolve member in declaration/inheritance surfaces;
3. build substitution from applied arguments;
4. lazily substitute parameter types and return type in the returned `CallableSignature`;
5. do not mutate or clone the stored declaration surface wholesale.

Example semantic fixture:

```text
Box<T>.get() -> T
Box<Int>.get() -> Int
```

Both views point back to the same declaration/callable identity; only the returned signature view is substituted.

## 12.5 No runtime specialization

No `ClassObject` clone, method dictionary copy, static slot copy, or metaclass is created for `Box<Int>`.

Applied member views are semantic projections only.

## 12.6 Fix generic subtyping

Until variance metadata is explicitly implemented, applied parameters are **invariant**.

Replace current recursive covariance:

```text
A <: B for every corresponding argument
```

with conservative first-version rule:

```text
Origin<A...> <: Origin<B...>
    only when origins are semantically the same applicable declaration
    and corresponding arguments are canonically equal
```

Do not infer generic covariance from origin inheritance.

Keep ordinary callable variance exactly where already defined:

- parameter types contravariant;
- return type covariant.

Those are callable-type semantics, not declaration-site generic variance.

## 12.7 Proper-type relation precondition

`is_subtype` should only be used on forms of kind `Type`.

Add debug/internal assertions or a checked wrapper at public boundaries so constructor-kinded forms do not accidentally enter value-subtyping logic.

## 12.8 Class-object type hierarchy

Implement:

```text
ClassObject(Sub) <: ClassObject(Super)
```

when `Sub` is a subclass of `Super`.

This models the observable class-object value hierarchy consistently with the runtime parallel metaclass rule without reifying semantic metaclasses.

## Tests — new `phalcom-semantic/tests/substitution.rs`

Required:

- parameter substitution direct.
- nested `List<T>` substitution.
- nested tuple/record/union/callable substitution.
- partial substitution leaves unbound parameter.
- applied member return substitution.
- applied member parameter substitution.
- canonical identity maintained after substitution.
- invariant generic subtyping: `Box<Int>` is not subtype of `Box<Number>` by default even when `Int <: Number`.
- identical applied type is reflexive subtype.
- class-object hierarchy mirrors class hierarchy.
- existing callable contravariance/covariance tests remain passing.

---

# 13. Work package 7 — project/module-aware semantic workspace

## Goal

Replace one-file ad-hoc environments with one coherent type/kind/declaration universe per linked analysis generation.

## Existing infrastructure to reuse

- `phalcom-modules::LinkedProgram`
- `phalcom-modules::ModuleGraphs`
- `SemanticGraph`
- `DeclarationShellTable`
- `DeclarationId`
- `ParsedSourceUnit`
- `phalcom-semantic::SemanticSnapshot`

## New file

- `phalcom-semantic/src/workspace.rs`

Potential additional file:

- `phalcom-semantic/src/resolver.rs` if project-aware resolution would make `types/annotation.rs` too large.

## Files to modify

- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/checker/mod.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/lib.rs`
- `phalcom-modules/src/resolver.rs`
- possibly `phalcom-modules/src/source.rs` for parse-once artifact ownership.

## 13.1 Parse source once

Current project discovery loads/parses interfaces through `ModuleResolver::load_interface`, then `ProgramCompiler::discover_and_link` separately reads source text. A shared static checker would otherwise parse every body again.

Introduce a module-layer parsed source artifact, because `phalcom-modules` may depend on `phalcom-ast` while it must not depend on `phalcom-semantic`.

Recommended:

```rust
pub struct ParsedModuleUnit {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
    pub text: Arc<str>,
    pub program: Arc<Program>,
    pub interface: UnlinkedModuleInterface,
}
```

`ModuleResolver` should cache parsed units. `load_interface()` becomes a projection from the parsed cache rather than a separate parse path.

Builtin providers should expose equivalent parsed units where source is available.

`phalcom-semantic::ParsedSourceUnit` may be constructed from this artifact without reparsing. Do not make `phalcom-modules` depend on `phalcom-semantic`.

## 13.2 Project-aware resolver

Keep `SimpleTypeResolver` for unit tests only. Add production `LinkedTypeResolver`:

```rust
pub struct LinkedTypeResolver { ... }
```

It must resolve through canonical linked identities:

1. local declaration name -> local `DeclarationId`;
2. selective imported binding -> canonical exported `SymbolId`/`DeclarationId`;
3. whole-module import alias + member -> linked target module export;
4. re-export -> canonical target identity;
5. prelude universe declarations -> canonical builtin declaration identity;
6. qualified reference -> linked path, never leaf-name fallback.

No `HashMap<String, DeclarationId>` with unqualified global names may be the production project resolver.

## 13.3 Workspace input/output

Define conceptually:

```rust
pub struct SemanticWorkspaceInput {
    pub linked: Arc<LinkedProgram>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedSourceUnit>>,
    pub generation: u64,
}

pub struct SemanticAnalysis {
    pub snapshot: Arc<SemanticSnapshot>,
    pub diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
}

pub fn analyze_workspace(input: SemanticWorkspaceInput) -> SemanticAnalysis
```

Use `BTreeMap` at publication boundaries for deterministic output.

## 13.4 Extend semantic snapshot

Current snapshot stores generation, type store, sources, surfaces, dispatch.

Add at minimum:

```rust
pub declarations: Arc<DeclarationTypeTable>,
pub hierarchy: Arc<MapTypeHierarchy>,
pub diagnostics: Arc<BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>>,
pub semantic_graph: Arc<SemanticGraph>,
```

If `MapTypeHierarchy` remains mutable by API, publish an immutable wrapper/type instead.

Every `TypeId`, `KindId`, `TypeParameterId`, and snapshot-local binding ID is interpreted only under this snapshot generation/store.

## 13.5 Analysis pipeline

Implement this explicit phase ordering:

### Phase A — bootstrap canonical universe type/kind metadata

- create `TypeStore`;
- register `Type` kind and arrow kinds lazily/canonically;
- register core declaration type forms/generic signatures from native metadata;
- register trusted native member surfaces once.

### Phase B — predeclare all source declarations

For every reachable parsed source class declaration:

- create canonical `DeclarationId`;
- predeclare through `DeclarationShellTable`;
- current source class generic signature is empty unless an already-approved/source-supported metadata path exists;
- register source declaration form with kind `Type`;
- register class-object proper type.

All declarations exist before any body annotation is resolved.

### Phase C — build linked type resolver

Use `LinkedProgram` imports/exports/bindings and declaration table.

### Phase D — enrich the existing semantic graph

Start from:

```rust
linked.graphs.semantics.clone()
```

Add declaration-level edges discovered from parsed declarations/annotations:

- superclass -> `Superclass`;
- annotation references -> `TypeReference`;
- generic restrictions when later present -> `ConstraintReference`;
- callable type references -> `CallbackSignature` where useful.

Do not add these edges to the runtime initialization graph.

Use the existing `SemanticGraph`, not a new checker graph type.

### Phase E — realize declaration shells / reject inheritance cycles

Use `DeclarationShellTable::realize_semantic_graph`.

Mutually recursive type references may be legal semantic SCCs; superclass cycles remain illegal.

### Phase F — construct hierarchy

Populate canonical direct superclass relationships from resolved declaration identities.

No unresolved superclass may silently become `Object` in production analysis. Emit a semantic/link diagnostic when resolution fails according to current language rules.

### Phase G — collect all declaration surfaces before bodies

Split current `register_class_surface` into a pure surface-collection operation that does not run method bodies.

Collect every reachable class’s instance/class fields and callable signatures using the same store/resolver.

Publish/register native and source surfaces into one `SurfaceDispatchResolver`.

### Phase H — check bodies

Only after all reachable interfaces/surfaces exist, check bodies.

Each module receives a `CheckingContext` borrowing the shared semantic world.

Do not create a new `TypeStore` per module.

### Phase I — solve local constraints/fixed points

Keep current local solver for local expression constraints.

Recursive/interprocedural solving should use the repository’s committed fixed-point/SCC direction; do not encode recursion as global `Unknown`. This work package need not complete every Pyrefly-inspired solver feature, but its ownership boundary must allow them to run over the shared workspace.

### Phase J — publish immutable generation

Construct `Arc<SemanticSnapshot>` only after all tables/diagnostics for the generation are coherent.

Consumers never read half-updated mutable semantic state.

## 13.6 Refactor `check_program`

Do not leave two semantically divergent checkers.

Options:

- keep `check_program` as a test/convenience wrapper that constructs a single-module `SemanticWorkspaceInput` and delegates to the shared pipeline; or
- make lower-level body checking explicitly internal and expose `analyze_workspace` as the canonical API.

There must be one implementation of annotation resolution, surface registration, and body checking.

## Tests — new `phalcom-semantic/tests/workspace.rs`

Required fixtures:

1. local class reference resolves canonically.
2. selective imported type reference resolves to target declaration.
3. module-qualified imported type reference resolves.
4. re-exported type reference resolves to original declaration identity.
5. same leaf class names in different modules remain distinct.
6. cross-module superclass relation works.
7. mutually recursive type-reference declarations realize via semantic SCC.
8. inheritance cycle is rejected.
9. project uses one `TypeStore`; same imported form gets same `TypeId` within generation.
10. diagnostics carry owning `ModuleId`.
11. deleting/changing one declaration removes stale surface entries on next generation.
12. builtins are canonical and not recreated under each user module ID.

## Module tests

Extend `phalcom-modules/tests/declaration_shells.rs` only where graph/shell behavior itself changes. Keep semantic typing tests in `phalcom-semantic`.

## Acceptance criteria

- Production project analysis has no `SimpleTypeResolver` dependency.
- No `run_semantic_typecheck`-style fresh store per module.
- Import graph is reused, not reimplemented.
- Project/package source can be checked coherently before compilation/runtime initialization.

---

# 14. Work package 8 — compiler/CLI integration and stable semantic export boundary

## Goal

Make all compilation entry paths consume the same semantic analysis, and define the stable representation future runtime reflection will consume without implementing reflection itself.

## New file

Recommended:

- `phalcom-semantic/src/export.rs`

## Files to modify

- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/src/modules/artifact.rs` only as necessary to reference stable semantic metadata
- `phalcom-core/bin/phalcom/cli.rs`
- `phalcom-semantic/src/lib.rs`

## 14.1 Introduce analyzed-program seam

Do not compile directly from “linked but semantically unchecked” project data.

Conceptually:

```rust
pub struct AnalyzedProgram {
    pub linked: Arc<LinkedProgram>,
    pub semantic: Arc<SemanticSnapshot>,
    pub sources: BTreeMap<ModuleId, Arc<ParsedSourceUnit>>,
}
```

This type may live in `phalcom-semantic` or a thin compiler-side wrapper, but dependency direction must remain:

```text
phalcom-modules -> no semantic dependency
phalcom-semantic -> phalcom-modules
phalcom-core -> phalcom-semantic + phalcom-modules
```

`ProgramCompiler` should obtain/consume an analyzed program before producing `CompiledProgram`.

## 14.2 Replace `run_semantic_typecheck`

Remove it as the production entry point.

If retained temporarily for tests/backward API compatibility, implement it by constructing/delegating to the canonical workspace analyzer, and mark it deprecated/internal.

Do not duplicate core builtin registration inside it.

## 14.3 Project/package/module/inline parity

All entry selections must pass through semantic analysis:

```text
Project
Package
owned Module
standalone Module
Inline
```

The difference is only how the project/source/link context is built.

No entry path may silently skip type checking because source was reached through project discovery rather than inline compilation.

## 14.4 Structured program semantic errors

Current `ProgramCompileError::Type(Vec<SemanticDiagnostic>)` loses module/source ownership.

Replace with or wrap a module-owned diagnostic representation, e.g.:

```rust
pub struct ProgramSemanticDiagnostics {
    pub by_module: BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
}
```

Then:

```rust
ProgramCompileError::Type(ProgramSemanticDiagnostics)
```

or a better-named `Semantic` variant if compatibility permits.

CLI/LSP rendering can then map ranges against the correct source text.

## 14.5 `phalcom check`

Update `cmd_check` rustdoc: it is no longer syntax-only.

Behavior:

- inline source -> same inline analyzed-program pipeline;
- standalone source file -> standalone linked+semantic pipeline;
- source file inside project -> discover owning project and analyze linked project context;
- if path is an already-supported project/package form, reuse compiler entry selection behavior rather than adding a checker-specific path resolver.

Do not add `--types=strict` in this work package unless separately approved/current CLI already defines it.

Preserve current text/JSON diagnostic formatting contracts where possible; add module/source path to structured output when necessary without silently changing stable fields.

## 14.6 Stable compiled semantic descriptors

Raw `TypeId`/`KindId` cannot cross snapshot/store lifetimes.

Define VM-independent structural forms in `phalcom-semantic/src/export.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledKindRef {
    Type,
    Arrow {
        parameters: Box<[CompiledKindRef]>,
        result: Box<CompiledKindRef>,
    },
}

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

`ClassObject` may be exported only if a compiler consumer genuinely needs the static type of class-object values. Runtime annotation reflection normally needs declaration/type-form metadata, not this internal checking type. Keep it out of public persisted metadata unless required.

Define stable owner identity using existing `DeclarationId` / `CallableId`, never `TypeParameterId` raw integer.

## 14.7 Exporter

Add:

```rust
pub fn export_kind(store: &TypeStore, kind: KindId) -> CompiledKindRef
pub fn export_type_form(
    store: &TypeStore,
    form: TypeId,
) -> Result<CompiledTypeRef, SemanticExportError>
```

Rules:

- recursive structural conversion;
- stable declaration identities retained;
- `TypeData::Infer` is rejected;
- `Unknown` and `Dynamic` cannot appear because they are not `TypeId` forms;
- no display strings as identity;
- deterministic order for unions/records as canonicalized by store.

## 14.8 Do not wire runtime reflection yet

This work package may attach exported semantic metadata to `CompiledModule`/`ModuleMaterializationPlan` **only if needed by an immediate compiler consumer**.

Otherwise implement and test the exporter and leave runtime materialization wiring to the later reflection milestone.

Do **not** add:

```text
Object::AppliedType
Object::Kind
ValueTag::Type
UniverseKey::AppliedType
```

in this phase.

## Tests

### Core compiler integration

- inline source with type mismatch fails through shared analyzer.
- standalone file mismatch fails through shared analyzer.
- project dependency mismatch is detected even when entry module is clean.
- package module mismatch detected.
- clean linked project compiles.
- same diagnostic codes as semantic layer are preserved.

### Export tests — `phalcom-semantic/tests/export.rs`

- `Int` exports as stable nominal declaration.
- `List<Int>` exports structurally.
- partial constructor kind exports structurally.
- union/tuple/record/callable export deterministically.
- parameter exports by owner/index.
- inference variable export fails.
- no exported structure contains raw `TypeId`, `KindId`, `TypeParameterId`.

## Acceptance criteria

- Project compilation can no longer bypass semantic analysis.
- One shared semantic result feeds CLI/compiler.
- Stable future-runtime metadata boundary exists without runtime reflection implementation.

---

# 15. Work package 9 — LSP shared static semantic snapshot and diagnostics

## Goal

Make editor static diagnostics consume `phalcom-semantic` while preserving the existing advisory `ValueShape` engine for runtime-shape/editor intelligence.

## Files to modify

- `phalcom-lsp/src/backend.rs`
- `phalcom-lsp/src/diagnostics.rs`
- `phalcom-lsp/src/analysis_service.rs`
- `phalcom-lsp/src/semantic/engine.rs` / snapshot ownership files as appropriate after current inspection
- LSP integration tests

## 15.1 Preserve two semantic domains

Do not delete or rename away the meaning of:

```rust
ValueShape
```

It continues to answer advisory runtime-shape questions useful for completion/hover/flow.

Static typing comes from:

```rust
Arc<phalcom_semantic::SemanticSnapshot>
```

Initially these may coexist in the LSP worker. Long-term query unification is allowed, but do not fake unification by converting one domain into the other.

## 15.2 Worker ownership

The LSP’s mutable analysis worker should own rebuilding static semantic workspace state after source/project changes, then publish an immutable `Arc<phalcom_semantic::SemanticSnapshot>`.

Requests must not hold mutable analysis locks while rendering responses.

Use generation coherence:

```text
source revisions used to build static snapshot
==
snapshot generation publication set
```

A request must not combine static diagnostics from one generation with source ranges from another.

## 15.3 URI -> `ModuleId` mapping

Use project/module resolution already owned by module/workspace infrastructure.

Do not invent a new filename-based module identity inside LSP solely for typing.

Builtin virtual URIs must resolve to their builtin `ModuleId` identity.

## 15.4 Publish semantic diagnostics

`backend.rs:306+` currently publishes syntax only.

New flow:

```text
open/change document
    ↓
parse/recover
    ↓
update workspace source
    ↓
static semantic generation completes
    ↓
for affected module(s):
    syntax diagnostics
    + semantic diagnostics from canonical snapshot
    ↓
publishDiagnostics
```

When syntax is sufficiently malformed that a module contribution cannot be semantically analyzed, publish syntax diagnostics and suppress stale semantic diagnostics for that revision.

## 15.5 Fix related-information URIs

Current `semantic_diagnostic_to_lsp_diagnostic()` uses a placeholder `file:///` URI for labels.

As part of project-aware diagnostics, enrich diagnostic labels or adapter input with module/source ownership so related information points at the real URI.

If cross-module labels are not yet produced, at minimum use the current document URI instead of `file:///`.

## 15.6 Diagnostic clearing

Publishing a clean semantic generation must send an empty semantic diagnostic list for previously erroneous modules/documents so squiggles disappear.

## 15.7 Hover/inlay integration scope

Not required for initial acceptance:

- replacing all ValueShape hover rendering;
- displaying kinds;
- displaying denotation;
- runtime reflective TypeForm docs.

However expose narrow semantic query APIs so later hover can ask:

```text
ordinary type at occurrence
optional denotation
kind of denoted type form
```

without re-running the checker.

## LSP tests

Required integration cases:

1. type mismatch produces `publishDiagnostics` with `source = "phalcom-typecheck"`.
2. diagnostic code matches semantic code.
3. fixing mismatch clears static diagnostic.
4. syntax and static semantic diagnostics can coexist when parse recovery permits.
5. stale semantic diagnostic does not survive a syntax-invalid replacement revision.
6. cross-module imported type mismatch resolves using project identity.
7. editing an exported/type declaration updates dependent module diagnostics.
8. unrelated module does not receive bogus diagnostics.
9. all existing 26+ ValueShape semantic-analysis tests continue passing unchanged unless expected fixtures explicitly improve.
10. LSP request snapshot generation is internally coherent.

## Acceptance criteria

- No editor-only static type checker exists.
- LSP adapter consumes shared `SemanticDiagnostic` and shared semantic snapshot.
- Advisory shape and static type remain explicitly different domains.

---

# 16. Work package 10 — verification, performance, documentation, and completion gate

## Goal

Prove the new semantic tower is correct, object-model-preserving, project-aware, deterministic, incremental-safe, and free of runtime cost in ordinary execution.

## 16.1 Required semantic invariants

Add direct tests/assertions for:

```text
1. Every TypeId has exactly one KindId.
2. TypeKnowledge::Known only carries TypeIds of kind Type.
3. A declaration type form and its class-object proper type are distinct facts.
4. Class-name expression uses class-object type + type-form denotation.
5. Ordinary values do not acquire type-form denotation accidentally.
6. Applied type kind is derived by kind application.
7. Partial application retains residual kind.
8. Applied type identity is canonical across grouping.
9. Unknown is never interned into TypeStore.
10. Dynamic is never represented as a nominal fake type.
11. Generic parameters are invariant until variance metadata says otherwise.
12. Instance/class member surfaces remain separate.
13. Class-side inherited lookup mirrors runtime superclass relation.
14. super changes lookup start, not receiver identity.
15. TypeId/KindId never cross stable export boundary directly.
```

## 16.2 Object-model regression gates

Because the implementation claims not to modify runtime object semantics, run the dedicated invariant suite:

```sh
scripts/test.sh invariants
```

Also run `phalcom-core` tests covering:

- class/metaclass bootstrap;
- class-side inherited methods;
- `Value::class()` totality;
- selector/method lookup;
- object reflection.

Any runtime tower regression blocks merge even if static tests pass.

## 16.3 Focused crate tests

Required:

```sh
cargo test -p phalcom-semantic --tests
cargo test -p phalcom-modules
cargo test -p phalcom-core
cargo test -p phalcom-lsp
```

Run AST tests because annotation parsing is an important no-syntax-change contract:

```sh
scripts/test.sh ast
```

## 16.4 LSP gate

```sh
scripts/test.sh lsp
```

This must include the new static-diagnostic integration tests and existing semantic shape tests.

## 16.5 Formatting/lint/docs/workspace gates

Repository guidance requires at minimum:

```sh
cargo fmt --check
cargo clippy --workspace
scripts/test.sh workspace
```

For this cross-crate semantic/compiler/LSP change, also run:

```sh
scripts/test.sh full
```

Public new Rust items require professional rustdoc. Run `cargo doc` if not already included by workspace gate and verify no missing-doc warnings for affected crates.

## 16.6 Graph maintenance

Repository `AGENTS.md`/`CLAUDE.md` require graphify usage for codebase work.

Before implementation:

```sh
graphify query "TypeStore KindId semantic checker declaration surface project typing" --budget 2000
graphify affected "TypeStore"
graphify affected "TypedExpression"
graphify affected "DeclarationSurface"
graphify path "ProgramCompiler" "SemanticSnapshot"
```

After each substantial implementation slice, re-run affected/path checks.

Before completion:

```sh
graphify update . --no-cluster
```

or the repository-current equivalent.

## 16.7 Performance requirements

This change must not make ordinary runtime execution slower. No runtime object/VM hot path changes are required in the milestone.

Static-analysis performance requirements:

### TypeStore

- `kind_of(TypeId)` O(1) dense vector access;
- no allocation on kind lookup;
- hash-consed canonical types/kinds;
- no repeated structural normalization where canonical IDs suffice.

### Type application

- O(number of newly supplied arguments) kind validation plus interning;
- nested application flattening does not recursively rebuild arbitrary deep trees when origin already canonical.

### Substitution

- perform lazily for applied member views;
- do not materialize a complete copied declaration surface per specialization;
- add a snapshot-local memo such as `(applied_type, callable_id) -> substituted signature` only after profiling demonstrates benefit.

### Workspace

- parse once per source revision;
- reuse linker graph;
- deterministic SCC order;
- no whole-workspace reparsing per LSP query;
- immutable published snapshots.

### LSP

- rebuild static semantic state in worker, not request path;
- publish only affected diagnostics when practical;
- correctness first, then incremental frontier optimization using semantic/reference graph reverse dependencies.

Record before/after timings for:

- `cargo test -p phalcom-semantic --tests`;
- a representative multi-module `phalcom check` fixture;
- LSP open/change analysis benchmark or existing perf counters.

No fixed performance percentage is mandated in this spec; any material regression requires explanation/profiling before merge.

## 16.8 Determinism tests

Analyze identical workspace twice in separate fresh stores and verify stable **structural** outputs:

- declaration identities;
- diagnostic ordering/codes/ranges;
- exported `CompiledTypeRef`/`CompiledKindRef`;
- kind/type display used in diagnostics.

Raw `TypeId` integer values need not be a cross-process persistence contract, though deterministic allocation is desirable within identical traversal order.

---

# 17. Detailed file-by-file implementation map

This section is a concrete edit checklist.

## `docs/spec/typing/ontology.md`

**Modify:** add `Type` atomic-kind / `TypeForm` terminology and precedence.
**Do not:** add runtime TypeForm APIs.

## `docs/spec/typing/README.md`

**Modify:** make ontology current foundation; mark conflicting earlier docs historical.

## `docs/spec/typing/STATUS.md`

**Modify:** supersession note; remove implication that stale `Type` protocol naming remains current.

## `phalcom-semantic/src/types/id.rs`

**Modify docs:** redefine `TypeId`, clarify store/snapshot lifetime for all IDs.

## `phalcom-semantic/src/types/kind.rs`

**Modify:** add canonical arrow helper/error representation as appropriate.

## `phalcom-semantic/src/types/store.rs`

**Modify heavily:**

- add `ClassObject` TypeData variant;
- dense total type-kind vector;
- remove `set_kind` public mutation;
- replace generic `intern` usage with kinded interning;
- make nominal form kind explicit;
- checked composite type construction;
- canonical kind APIs;
- type-parameter storage if kept in store.

## `phalcom-semantic/src/types/application.rs` — **new**

**Add:** checked/canonical type-form application and `TypeApplicationError`.

## `phalcom-semantic/src/types/parameter.rs` — **new**

**Add:** `TypeParameterOwner`, `TypeParameterData`, `GenericSignature`.

## `phalcom-semantic/src/types/denotation.rs` — **new**

**Add:** `SemanticDenotation`, `ValueSemanticFact`, optional receiver-dispatch helper if not kept under checker.

## `phalcom-semantic/src/types/substitution.rs` — **new**

**Add:** generic `TypeParameterId -> TypeId` substitution.

## `phalcom-semantic/src/declarations.rs` — **new**

**Add:** `DeclarationTypeInfo`, `DeclarationTypeTable`, registration/bootstrap helpers.

## `phalcom-semantic/src/types/annotation.rs`

**Modify:** recursive type-form lowering for Application/Tuple/Callable/Union; require proper Type at value-annotation boundary.

## `phalcom-semantic/src/types/native.rs`

**Modify:** normalize native Parameter/Applied/Tuple structures through same canonical store and declaration table; stop treating structurally supported metadata as opaque by default.

## `phalcom-semantic/src/types/relation.rs`

**Modify:** proper-kind precondition; invariant generic arguments; class-object subtype relation; hierarchy direct-parent query.

## `phalcom-semantic/src/types/constraint.rs`

**Modify minimally:** update renamed TypeStore APIs. Keep inference substitution separate from generic substitution.

## `phalcom-semantic/src/checker/typed_expr.rs`

**Modify:** denotation and receiver-dispatch metadata; constructors/defaults.

## `phalcom-semantic/src/checker/context.rs`

**Modify:**

- local env uses `ValueSemanticFact`;
- add `current_side`;
- borrow/shared declaration table;
- dispatch through side-aware resolver;
- remove naive applied-origin fallback.

## `phalcom-semantic/src/checker/expression.rs`

**Modify:**

- declaration value expression -> class-object proper type + denotation;
- self side semantics;
- super receiver-dispatch semantics;
- migrate collection inference to registered constructor forms + checked application;
- use `synthesize_typed_expr` whenever denotation must propagate.

## `phalcom-semantic/src/checker/statement.rs`

**Modify:** preserve denotation in bindings and return typed expression/fact where needed.

## `phalcom-semantic/src/checker/declaration.rs`

**Modify:** split surface collection from body checking; use instance/class side; use project declaration table; set checker side per member.

## `phalcom-semantic/src/checker/mod.rs`

**Modify:** separate reusable surface/body phases; make one-program helper delegate to workspace machinery.

## `phalcom-semantic/src/surface.rs`

**Modify:** `MemberSurface` + instance/class sides.

## `phalcom-semantic/src/dispatch.rs`

**Modify:** side-aware `DispatchOwner`; inheritance walk; substituted applied views.

## `phalcom-semantic/src/snapshot.rs`

**Modify:** publish declaration type table, hierarchy, diagnostics, semantic graph with same generation/store.

## `phalcom-semantic/src/source.rs`

**Modify only if necessary:** conversion from parse-once module artifact.

## `phalcom-semantic/src/workspace.rs` — **new**

**Add:** project/module analysis coordinator and immutable publication.

## `phalcom-semantic/src/export.rs` — **new**

**Add:** stable `CompiledTypeRef`, `CompiledKindRef`, parameter-owner representation, exporter.

## `phalcom-semantic/src/diagnostic.rs`

**Modify:** kind/application diagnostics.

## `phalcom-semantic/src/types/mod.rs` / `phalcom-semantic/src/lib.rs`

**Modify:** exports for new canonical semantic APIs only; avoid exposing implementation-only mutable tables unnecessarily.

## `phalcom-native-meta/src/types.rs`

**Modify:** kind/type-form declaration specs.

## `phalcom-native-meta/src/universe.rs`

**Modify:** canonical core generic signature table. Do not change runtime class catalog semantics.

## `phalcom-modules/src/resolver.rs`

**Modify:** parse-once module cache/API.

## `phalcom-modules/src/source.rs` or suitable existing ownership file

**Potential new/modify:** `ParsedModuleUnit` if not placed in resolver.

## `phalcom-core/src/modules/compile.rs`

**Modify heavily:** consume analyzed workspace on every entry path; remove isolated hardcoded checker environment.

## `phalcom-core/src/modules/artifact.rs`

**Modify only if needed:** attach stable exported semantic metadata; never raw IDs.

## `phalcom-core/bin/phalcom/cli.rs`

**Modify:** `check` invokes shared project-aware analyzer and renders module-owned diagnostics; fix stale rustdoc.

## `phalcom-lsp/src/diagnostics.rs`

**Modify:** real URI ownership for semantic related info.

## `phalcom-lsp/src/backend.rs`

**Modify:** merge/publish syntax and canonical static semantic diagnostics.

## `phalcom-lsp/src/analysis_service.rs` and semantic worker files

**Modify:** maintain/publish `phalcom_semantic::SemanticSnapshot` alongside advisory runtime-shape snapshot.

## Explicitly unchanged in this milestone

Unless an unrelated compile refactor forces imports only, do not change runtime semantics in:

```text
phalcom-core/src/value/repr.rs
phalcom-core/src/value/mod.rs  // behavioral logic
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/class.rs
phalcom-core/src/universe/core_classes.rs
```

No type/kind runtime heap variants or core classes yet.

---

# 18. Test suite layout

Recommended new semantic integration files:

```text
phalcom-semantic/tests/kinds.rs
phalcom-semantic/tests/declaration_types.rs
phalcom-semantic/tests/denotation.rs
phalcom-semantic/tests/type_annotations.rs
phalcom-semantic/tests/class_side_dispatch.rs
phalcom-semantic/tests/substitution.rs
phalcom-semantic/tests/workspace.rs
phalcom-semantic/tests/export.rs
```

Continue running and extending:

```text
phalcom-semantic/tests/checker.rs
phalcom-semantic/tests/phase2_expression_engine.rs
phalcom-modules/tests/declaration_shells.rs
existing phalcom-core object-model/class-side tests
existing phalcom-lsp semantic + diagnostics tests
```

Testing philosophy:

1. Test algebra directly before parser/UI.
2. Test semantic expression facts directly before diagnostics rendering.
3. Test workspace identity/resolution before compiler integration.
4. Test compiler/CLI before LSP adapter.
5. Test LSP protocol output only after canonical semantic facts pass.
6. Test runtime object model as regression, not as the implementation of static typing.

---

# 19. Required end-to-end acceptance scenarios

## Scenario A — ordinary value vs class-object denotation

Source:

```phalcom
const n = 1
const c = Int
```

Expected semantic facts:

```text
n:
  ordinary type = Int
  denotation = None

c:
  ordinary type = ClassObject(Int)
  denotation = TypeForm(Int)
  kind(TypeForm(Int)) = Type
```

## Scenario B — generic constructor and application

Authoritative core metadata:

```text
List<T :: Type> :: Type -> Type
```

Annotation:

```phalcom
const xs: List<Int> = [1, 2, 3]
```

Expected:

```text
List                    :: Type -> Type
Int                     :: Type
List<Int>               :: Type
canonical applied ID    stable within snapshot/store
```

No runtime list instance receives an `Int` token.

## Scenario C — partial application internal support

Direct semantic test:

```text
Map                    :: Type -> Type -> Type
Map<String>            :: Type -> Type
Map<String, Int>       :: Type
```

If `Map<String>` appears where a value type is required, the annotation layer rejects it as unsaturated. The algebra itself remains valid.

## Scenario D — wrong higher kind

Direct/annotation semantic case:

```text
List<List>
```

Given:

```text
List parameter kind = Type
List argument kind  = Type -> Type
```

Expected diagnostic:

```text
type.application.argument_kind_mismatch
```

No `Applied` node is interned as a valid form for that application.

## Scenario E — class-side lookup

Source fixture:

```phalcom
class Parent {
  @class
  make() { ... }
}

class Child is Parent {}

Child.make()
```

Static resolution walks:

```text
Child / Class
Parent / Class
```

Runtime continues walking:

```text
Child class
Parent class
```

The semantic checker does not create `Child class` TypeData objects just to perform lookup.

## Scenario F — super

Inside `Child` class-side member:

```phalcom
super.make()
```

Receiver ordinary identity remains class object `Child`; lookup begins at `Parent / Class`.

Inside instance method the same principle applies with `Instance` side.

## Scenario G — cross-module typing

Module A exports class `User`. Module B imports it and annotates a parameter `User`.

Expected:

- one canonical `DeclarationId` for A.User;
- B’s annotation resolves to that declaration’s canonical type form;
- changing A’s relevant declaration surface invalidates B’s dependent semantic result;
- no duplicate B-local `User` nominal type is manufactured.

## Scenario H — IDE diagnostic

A project module contains:

```phalcom
const x: String = 1
```

Expected:

- compiler/CLI semantic diagnostic code is canonical;
- LSP publishes same code with source `phalcom-typecheck`;
- correcting initializer clears the diagnostic;
- no editor-only inference path decides the mismatch independently.

---

# 20. Deferred runtime reflection contract

This section is **not an implementation work package for this milestone**. It fixes the contract that the later runtime-reflection plan must obey.

## 20.1 Future runtime type-form values

Bare nominal/class type forms reify to existing class objects:

```text
reify(Int)  -> existing runtime Class object Int
reify(List) -> existing runtime Class object List
```

Synthetic forms may later use ordinary immutable objects/classes:

```text
List<Int>           -> AppliedType object
Int | String        -> UnionType object
(Int, String)       -> TupleType object
record type         -> RecordType object
Int -> String       -> CallableType object
```

## 20.2 Future kind values

```text
Type                -> atomic-kind reflected object
Type -> Type        -> function-kind reflected object
```

Exact runtime class names (`AtomicKind`, `FunctionKind`, `KindDescriptor`) may be finalized in the runtime-reflection specification, but they may not change semantic meaning.

## 20.3 Runtime representation policy

Preferred initial direction:

- keep `Value` representation unchanged;
- use ordinary heap references for reflected descriptors;
- use a VM-owned canonical runtime type/kind registry;
- materialize descriptor objects lazily/on demand;
- nominal reification returns existing class object;
- cache canonical descriptor identity deliberately;
- solve GC rooting/weakness/immortality explicitly at runtime-reflection design time.

## 20.4 Stable bridge

Future runtime must consume `CompiledTypeRef` / `CompiledKindRef` or equivalent structural stable metadata and re-intern it into runtime IDs.

It must **not** retain semantic `TypeId`/`KindId` from compiler snapshots.

## 20.5 No per-instance generic tax

For:

```phalcom
const xs: List<Int> = [1, 2, 3]
```

runtime `xs.class` remains `List` under baseline semantics. Static type `List<Int>` does not require a hidden type token in the list object.

If a future checked/debug runtime mode wants dynamic generic tokens, it requires a separate explicit design.

---

# 21. Suggested implementation/commit sequence

Keep changes reviewable. Recommended sequence:

## Commit 1 — semantic ontology/kind kernel

- docs precedence/TypeForm naming;
- total kind table;
- canonical arrow kinds;
- checked application;
- ClassObject static type form;
- kind tests.

Verification:

```sh
cargo test -p phalcom-semantic --tests
cargo fmt --check
```

## Commit 2 — declaration forms/core generic metadata

- parameter metadata;
- declaration type table;
- native universe type-form signatures;
- bootstrap tests.

## Commit 3 — denotation + class-side semantic facts

- ValueSemanticFact;
- TypedExpression denotation;
- class-name/self/super semantics;
- side-aware surfaces/dispatch;
- tests.

Run object-model invariants even though runtime should be untouched.

## Commit 4 — annotation lowering + substitution

- Application/Tuple/Callable lowering;
- kind diagnostics;
- generic substitution;
- applied member views;
- invariant generic subtyping.

## Commit 5 — project semantic workspace

- parse-once source artifact;
- linked resolver;
- declaration SCC enrichment;
- shared snapshot;
- workspace tests.

## Commit 6 — compiler/CLI + export boundary

- analyzed-program seam;
- all compiler entry paths;
- project-aware `check`;
- stable exported type/kind descriptors.

## Commit 7 — LSP integration

- shared static snapshot in worker;
- static diagnostic publication/clearing;
- URI ownership;
- cross-module invalidation tests.

## Commit 8 — full verification/docs/perf

- full gates;
- graphify update;
- benchmark notes;
- documentation cleanup;
- acceptance evidence.

Do not combine runtime reflection into these commits.

---

# 22. Completion checklist

An implementation is complete only when every item below is true.

## Semantic kernel

- [ ] `TypeId` documented as type-level-form identity.
- [ ] every `TypeId` has exactly one total `KindId`.
- [ ] `kind_of` has no fallback.
- [ ] canonical `Type` atomic kind retained as `KindId::TYPE`.
- [ ] canonical arrow kinds work.
- [ ] partial kind/type application works internally.
- [ ] wrong-kind application rejects cleanly.
- [ ] nested application canonicalizes.
- [ ] `ClassObject(declaration)` proper static type exists.

## Declaration model

- [ ] every declaration has a canonical type form.
- [ ] every class declaration has a class-object proper type.
- [ ] core `List`, `Set`, `Map`, `Option` generic signatures come from trusted metadata.
- [ ] no generic source syntax invented.
- [ ] type parameter identity is owner/index-based semantically.

## Two-axis expression semantics

- [ ] ordinary value type and denotation are separate.
- [ ] class-name expressions use class-object proper type.
- [ ] class-name expressions denote declaration type form.
- [ ] lexical binding can preserve denotation.
- [ ] class-side `self` is modeled correctly.
- [ ] `super` changes lookup start, not receiver.

## Dispatch/surfaces

- [ ] instance/class surfaces separated.
- [ ] source static member flags propagated.
- [ ] inherited instance lookup works.
- [ ] inherited class-side lookup mirrors runtime parallel rule.
- [ ] applied member views substitute generics lazily.
- [ ] selector identity unchanged.

## Type syntax already present

- [ ] generic application annotation lowering implemented.
- [ ] tuple annotation lowering implemented.
- [ ] callable annotation lowering implemented.
- [ ] union remains canonical.
- [ ] unsaturated constructor cannot enter value annotation.
- [ ] stable kind/application diagnostics exist.

## Relations

- [ ] subtype operation restricted to proper types.
- [ ] generic args invariant until variance is explicit.
- [ ] callable variance preserved.
- [ ] class-object type hierarchy implemented.

## Workspace

- [ ] one store per semantic generation/workspace.
- [ ] linked declaration identities used across modules.
- [ ] semantic graph/SCC infrastructure reused.
- [ ] declaration shells predeclared before bodies.
- [ ] all surfaces collected before body checking.
- [ ] project/package errors detected outside entry module.
- [ ] snapshots immutable/coherent.

## Compiler/CLI

- [ ] project/package/module/inline all run shared analyzer.
- [ ] isolated production `run_semantic_typecheck` removed/delegated.
- [ ] diagnostics retain module/source ownership.
- [ ] `phalcom check` docs match behavior.
- [ ] stable CompiledTypeRef/CompiledKindRef boundary exists.
- [ ] exporter rejects inference variables.
- [ ] raw semantic IDs are not persisted.

## LSP

- [ ] static semantic diagnostics publish.
- [ ] diagnostics clear after correction.
- [ ] project imported types resolve.
- [ ] dependent edits invalidate affected diagnostics.
- [ ] ValueShape remains separate advisory domain.
- [ ] no editor-only type checker.

## Runtime preservation

- [ ] no new runtime Type/Kind object variants in this milestone.
- [ ] no `Value` size/layout change.
- [ ] no metaclass tower change.
- [ ] no per-instance generic token.
- [ ] `scripts/test.sh invariants` passes.

## Repository gates

- [ ] semantic tests pass.
- [ ] modules tests pass.
- [ ] core tests pass.
- [ ] LSP tests pass.
- [ ] AST tests pass.
- [ ] formatting passes.
- [ ] clippy passes.
- [ ] workspace gate passes.
- [ ] full gate passes.
- [ ] graphify updated.
- [ ] relevant performance evidence recorded.

---

# 23. Final architecture invariant

After this implementation, the source tree should encode this model directly:

```text
                           PHALCOM

       OBJECT / RUNTIME AXIS                SEMANTIC AXIS
       =====================                =============

42 ──.class──▶ Int                      42 : Int

Int ──.class──▶ Int class               Int value
 │                                        │
 │                                        └─ denotes ─▶ Int :: Type
 │
 └─ runtime metaclass tower unchanged

List ─.class──▶ List class              List value
                                           │
                                           └─ denotes ─▶ List :: Type -> Type

annotation List<Int>
          │
          └─ canonical semantic application
                         │
                         ▼
                    List<Int> :: Type
```

And the code-level ownership must remain:

```text
phalcom-ast
    owns syntax

phalcom-modules
    owns module/declaration identity, linking, dependency graphs, parse/source ownership

phalcom-semantic
    owns canonical type forms, kinds, denotation, type relations,
    declaration typing, substitution, checking, semantic snapshots

phalcom-core
    consumes analyzed semantics for compilation and later consumes stable
    exported metadata for runtime reification

phalcom-lsp
    consumes immutable phalcom-semantic results for static typing while
    retaining separate advisory runtime-shape analysis
```

The decisive rule for all future work is:

> **Runtime reflection may expose Phalcom’s semantic tower, but it may not define a second one.**

Once this milestone is complete, a future runtime-reflection implementation can add ordinary Phalcom descriptor objects for synthetic type forms and kinds while reusing the exact canonical semantics established here. No fundamental type/kind law should need to be redesigned at that stage.
