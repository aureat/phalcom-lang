# Phalcom Canonical Universe Source and Verified Bootstrap Completion

## Repository-Grounded Implementation Specification

> **Status:** Proposed implementation specification  
> **Repository:** `aureat/phalcom-lang`  
> **Grounded revision:** `2705620c4e26e9447ef9742edb70cd271f9d25a6` (`main`, `docs(typing): rebase Spec 07 execution plan`, 2026-08-23)  
> **Primary scope:** complete the canonical `phalcom-core/core/universe/src/**/*.ph` corpus; make every bootstrap/native class and method visible and accurately declared in Phalcom source; make bootstrap mechanically verify source/native/runtime agreement; require complete type annotations and Phaldoc for the core surface; remove synthetic declaration overlays that exist only because the universe source is incomplete.  
> **Relationship to earlier plan:** this document deliberately narrows and supersedes the *universe-source and bootstrap-completion workstream* of `phalcom-canonical-native-universe-bootstrap-implementation-spec.md`. It does not replace the broader typing Specs 01–07 or the already-landed native metadata architecture.

---

# 1. Executive Summary

The principal defect addressed by this specification is no longer the absence of a native metadata architecture. Much of that architecture now exists. The defect is that the **canonical Phalcom universe source still does not actually describe the complete universe**.

Today a developer can open `phalcom-core/core/universe/src/scalar/number.ph` and find only:

```phalcom
class Number {}
class Int is Number {}
class Float is Number {}
```

while those classes have a substantial bootstrap-installed protocol. Many reflection modules contain only a module-level documentation marker and no class declaration at all. Rich source modules such as `String`, `List`, `Map`, `Set`, `Tuple`, `Record`, `Bytes`, `Behavior`, and `Method` call `_$...` implementation selectors that are real parts of the core implementation surface but are invisible in the containing class source. `BuiltinInterfaceBuilder` compensates for missing source by hard-coding class names into module interfaces, and VM bootstrap separately maintains a hard-coded `include_str!` source list. Runtime startup still defaults to `NativeInstallMode::Dual`, installing the legacy primitive floor when descriptor coverage is incomplete and then installing registered primitive descriptors as well.

That is the architecture this implementation must finish.

The target is simple to state:

> **A Phalcom developer must be able to open the canonical universe project and see the complete semantic presentation of the core language.**

For every universe class, the appropriate `.ph` module must show:

- the class declaration and correct superclass/generic declaration;
- whether the class identity is primordial/native;
- every language-visible method owned by that class;
- every internal implementation selector that source code relies on;
- whether each method is implemented in Rust or in Phalcom;
- class-side placement, constructor/factory status, and visibility annotations;
- parameter labels and source-friendly parameter names;
- parameter and return types;
- generic parameters and `where` constraints where applicable;
- durable Phaldoc describing the class and each method;
- real Phalcom bodies for derivable behavior;
- declaration-only `@native` members for irreducible Rust primitives;
- reference bodies only when they are intentionally useful as explanatory source and are never executed in place of Rust.

Bootstrap must then prove that this source presentation is true.

The final startup path is:

```text
UNIVERSE_CLASS_RELATIONS + UNIVERSE_BINDINGS
                 │
                 ▼
      create primordial class identities
                 │
                 ▼
BuiltinProjectSourceProvider(Universe)
                 │
                 ▼
 parse every canonical universe module once
                 │
                 ▼
  build complete UniverseSourceIndex
                 │
        ┌────────┴─────────┐
        ▼                  ▼
class/source parity   @native member parity
        │                  │
        └────────┬─────────┘
                 ▼
verify against NATIVE_SURFACES / PRIMITIVES
                 │
                 ▼
       descriptor-only native install
                 │
                 ▼
attach MethodSemanticIndex metadata
                 │
                 ▼
compile/execute real .ph implementations
  (`@native` declarations emit no code)
                 │
                 ▼
post-bootstrap class/global/layout invariants
                 │
                 ▼
          publish verified universe
```

Completion is not “the files have more comments.” Completion is a **closed, mechanically checked triangle**:

```text
canonical universe .ph source
          ↕
canonical Rust native descriptors
          ↕
live bootstrapped runtime surface
```

No fourth handwritten registry may be required to make those three agree.

---

# 2. Why This Is a Separate Implementation Program

The earlier native-universe plan correctly designed attributes, native declarations, descriptor generation, source/native verification, and bootstrap convergence, but it treated actual universe-file migration as one section among many. That priority is now backwards.

Current `main` has already landed major pieces that the old plan proposed:

- `phalcom-native-decl` exists;
- `phalcom-native-surface/src/generated.rs` exists;
- `NATIVE_SURFACES` is a generated canonical native surface;
- `phalcom-native-meta::UNIVERSE_CLASS_RELATIONS` exists;
- the runtime has a distributed `PRIMITIVES` descriptor registry;
- `phalcom-semantic` has canonical `TypeStore`, type terms, generic signatures, callable semantic signatures, and current Spec-04/04.5 foundations;
- `phalcom-diagnostics` exists;
- `RuntimeTypingRegistry` / `MethodSemanticIndex` exist;
- `BuiltinProjectSourceProvider` and `UNIVERSE_NODES` already model the universe as a builtin project.

What remains conspicuously incomplete is the **source presentation** and the **bootstrap proof that presentation is exact**.

This specification therefore makes the source corpus the main workstream and treats compiler/native/bootstrap changes as supporting infrastructure required to make that corpus authoritative.

---

# 3. Non-Negotiable Language Invariants

This work must preserve the following previously ratified invariants.

1. Static type metadata never participates in selector identity, ordinary dispatch, runtime class/metaclass identity, allocation layout, or inline-cache identity.
2. `List<Int>` and other generic applications are semantic type applications, not specialized runtime classes.
3. Source and Rust native declarations converge by stable callable identity; they are not two runtime implementations competing for dispatch.
4. The runtime remains one object model. `@native class` presents/completes an existing primordial class identity; it does not create a second “native type” hierarchy.
5. `_$selector` denotes the implementation selector namespace. It does **not** by itself mean “implemented in Rust.” A Phalcom implementation may be internal.
6. `@native` means Rust/native implementation ownership. `@internal` means implementation-namespace visibility/assertion. They are orthogonal.
7. Real derivable behavior remains in `.ph` even if it calls native hooks.
8. Types on native declarations are semantic contracts and documentation/tooling facts. They do not change dispatch.
9. The canonical semantic type system from Specs 01–04.5 is the comparison substrate. This work must not create a second `NormalizedContractType` universe.
10. Temporary inference variables remain session-local. Native/source declarations publish only canonical type terms.
11. Runtime guards, contracts, effects, and future proofs remain separate products; this universe-completion work must not collapse them into a single “native correctness” flag.
12. No missing or unsupported semantic fact may be silently converted into a successful source/native match.

---

# 4. Current Repository State That Motivates the Work

## 4.1 The canonical project exists but its files are incomplete

`phalcom-modules/src/builtin.rs` already declares `UNIVERSE_NODES` for the builtin `universe` project. The project has these package families:

```text
universe
├── object
├── scalar
├── errors
├── callable
├── option
├── concurrency
├── collections
└── reflection
    └── typing
```

This is the correct source organization to complete. Do not introduce another “core declarations” directory.

## 4.2 Several source modules are rich implementations over an invisible native floor

Representative examples:

- `scalar/string.ph` implements Unicode/search/trim/string wrappers but calls `_$byteCount`, `_$byteAt(_)`, and `_$slice(_,_)` without declaring those methods.
- `object/behavior.ph` implements `attributes` and `attributesOfType(_)` over `_$attributes`, but the native/internal member is absent from source.
- `callable/method.ph` does the same for method attributes.
- `collections/list.ph` calls `_$length`, `_$at(_)`, `_$set(_,_)`, `_$push(_)`, `_$replaceSlice(_,_,_)`, and other floor operations without declaring them.
- `collections/map.ph` calls `_$size`, `_$get(_)`, `_$put(_,_)`, `_$has(_)`, `_$remove(_)`, `_$keyAt(_)`, and `_$valueAt(_)` without native declarations.
- `collections/set.ph`, `tuple.ph`, `record.ph`, `bytes.ph`, `range.ph`, and `fiber.ph` have the same presentation gap.

These files already embody Phalcom's intended “small native floor, rich source protocol” philosophy. The fix is **not** to move their real source algorithms to Rust. The fix is to expose the native floor as declarations inside the same class source.

## 4.3 `_$` is not synonymous with `@native`

`collections/range.ph` currently contains a real Phalcom implementation:

```phalcom
_$sliceBounds(_ size) {
  ...
}
```

This method belongs to the internal implementation namespace but is not a Rust primitive. The target declaration must therefore be conceptually:

```phalcom
@internal
_$sliceBounds(_ size: Int) -> Result<(Int, Int), SliceError> {
  ...
}
```

and **must not** carry `@native`.

By contrast, a raw representation hook such as `String#_$byteAt(_)` is both internal and native:

```phalcom
@internal
@native
_$byteAt(_ index: Int) -> Option<Int>
```

This distinction must be enforced mechanically throughout the universe corpus.

## 4.4 Some modules are only placeholders

Examples on current `main` include files whose entire useful content is a module-level documentation marker, such as:

```text
reflection/selector.ph
reflection/message.ph
reflection/attribute.ph
reflection/project-manifest.ph
reflection/typing/type-descriptor.ph
callable/closure.ph
callable/family.ph
```

Those classes exist at runtime and/or through generated native metadata, but the canonical Phalcom module does not declare them. This is precisely the source-authority gap this specification closes.

## 4.5 `number.ph` and several primordial roots are skeletons

`scalar/number.ph` contains only empty `Number`, `Int`, and `Float` declarations even though numeric arithmetic, conversion, equality, ordering, hashing, construction, and other behavior exist in bootstrap/native code.

`scalar/symbol.ph` and `object/metaclass.ph` are similarly skeletal.

## 4.6 `BuiltinInterfaceBuilder` currently masks missing source declarations

`phalcom-modules/src/builtin_interface.rs` builds an interface from source, then hard-codes a large module-path → class-name overlay for primordial/native classes. Examples include:

```text
reflection.selector           Selector, SelectorPattern
reflection.message            Message
reflection.attribute          Attribute
reflection.typing.kind        KindDescriptor, AtomicKind, FunctionKind, Type
reflection.typing.type_descriptor
                              TypeDescriptor, AppliedType, UnionType, TupleType,
                              RecordType, CallableType, TypeLambda, SpecialType, SelfType
reflection.typing.result      TypingResult and all result-state classes
object.object                 Object
object.behavior               Behavior
object.class                  Class
object.metaclass              Metaclass
scalar.number                 Number, Int, Float
callable.method               Method, BoundMethod
callable.family               Family, MethodFamily, BoundMethodFamily
option.option                 Option, Some, None, Unit
collections.list              List
collections.map               Map
...
```

The overlay is compatibility scaffolding. In the final architecture, those class declarations must come from the `.ph` modules themselves.

## 4.7 The builtin catalog and physical source tree can already drift

The physical source tree currently contains `phalcom-core/core/universe/src/scalar/uri.ph`, but `UNIVERSE_NODES` does not register `scalar.uri`; the canonical catalog registers `reflection.uri` instead. This is exactly the kind of source-corpus drift that should become impossible.

The implementation must decide whether an orphan file is stale and should be deleted, or belongs in the builtin catalog. It must never remain silently outside the canonical project.

## 4.8 Class ownership is currently inconsistent in compatibility overlays

`BuiltinInterfaceBuilder` currently injects `Unit` under `option.option`, while the current source definition of `Unit` lives in `collections/tuple.ph`. Similar discrepancies must be found by census rather than preserved as historical accidents.

Canonical ownership must be determined by the source project and then verified against exported/bootstrap identities.

## 4.9 Bootstrap still has two native installation paths

`VM::new()` currently defaults to:

```rust
Self::new_with_native_install_mode(NativeInstallMode::Dual)
```

and does:

```text
if Dual OR descriptor floor incomplete:
    Universe::install_primitives(...)

install_registered_primitives(...)
```

`descriptor_floor_is_complete()` compares distributed descriptors against transitional `NATIVE_MEMBERS`.

The final bootstrap must be descriptor-only.

## 4.10 Bootstrap maintains a second source-corpus list

`VM::run_universe_modules()` currently owns a static `SOURCES` array of direct `include_str!` calls. The builtin source provider already knows the universe project. This duplicate source list must disappear.

## 4.11 Current census tests diagnose drift but do not yet close it

`phalcom-core/tests/spec03_5_census.rs` computes generated, descriptor, and legacy key sets and prints their differences, but currently only asserts that generated and descriptor sets are nonempty. Final completion requires equality assertions and source-anchor parity as well.

---

# 5. Definition of a Complete Universe Module

A universe module is **complete** only if every one of the following conditions holds.

## 5.1 Module completeness

Every physical `.ph` file under `phalcom-core/core/universe/src` is exactly one of:

1. a canonical `UNIVERSE_NODES` module/package;
2. an explicitly excluded generated/test fixture outside the source root; or
3. removed as stale.

There may be no orphan source files.

Every `UNIVERSE_NODES` module/package must resolve to a physical/bundled source file and parse successfully.

## 5.2 Class completeness

Every language-visible class/value identity exported by the universe must have one canonical source declaration in its owning `.ph` module.

Every bootstrap-created class in `UNIVERSE_CLASS_RELATIONS` must have a source presentation unless it is explicitly a non-language implementation class that the language design intends to hide. The default is **present it**, including universe-only/non-prelude classes.

For a primordial class, the source declaration uses `@native class` once class-level native completion is supported:

```phalcom
/// Root of ordinary Phalcom values.
@native
class Object {
  ...
}
```

For a source-created class, do not add `@native` merely because it lives in the universe project.

## 5.3 Member completeness

For every class owned by the module, source must account for every member in the class's intended core surface.

A member is one of four categories:

```text
NATIVE_PUBLIC       Rust implementation, public/protected language surface
NATIVE_INTERNAL     Rust implementation, _$ implementation namespace
SOURCE_PUBLIC       real Phalcom implementation
SOURCE_INTERNAL     real Phalcom implementation in _$ namespace
```

No member may remain “runtime-only and invisible” merely because it is installed during bootstrap.

## 5.4 Signature completeness

Every core member must have the best canonical source signature the language type system can express:

- generic binders;
- parameter lanes and labels;
- parameter names;
- parameter types;
- return type;
- `Self` where semantically correct;
- `where` constraints where required;
- callable/block parameter types where supported;
- `Option`, `Result`, tuple/record/callable types rather than informal comments;
- no fake `Dynamic` just to make the file type-check.

At final completion there should be no untyped public/native declaration whose type is already known in Rust/native metadata or language specification.

## 5.5 Documentation completeness

Every core class and every core member gets Phaldoc.

Use the existing Phaldoc convention:

```text
//! module-level documentation
/// item-level documentation
```

Plain `//` is reserved for local implementation reasoning.

Documentation must explain semantic behavior, not restate machine-readable annotations.

## 5.6 Annotation completeness

Every bootstrap-installed native member must carry `@native` in its source anchor.

Every authored `_$...` member must carry `@internal`, regardless of whether its body is Rust or Phalcom.

Every class-side member must carry `@class` unless its syntax inherently establishes class-side placement through another ratified construct.

Constructors/factories must use the already-ratified constructor semantics; do not label arbitrary class-side factories `@constructor` simply because they allocate.

---

# 6. Canonical Source Forms

## 6.1 Primordial native class

```phalcom
/// Immutable UTF-8 string value.
@native
class String {
  ...
}
```

`@native class` means “this source declaration presents/completes the pre-existing primordial runtime class identity.” It must not allocate a fresh class.

## 6.2 Public native method

```phalcom
/// Concatenates this string with `other`.
///
/// @param other — string appended after the receiver
/// @returns the concatenated string
@native
+(_ other: String) -> String
```

No source body is needed for an irreducible primitive.

## 6.3 Internal native method

```phalcom
/// Returns the UTF-8 byte at `index`, or `None` when out of bounds.
@internal
@native
_$byteAt(_ index: Int) -> Option<Int>
```

## 6.4 Internal source implementation method

```phalcom
/// Normalizes the range into sequence slice bounds.
@internal
_$sliceBounds(_ size: Int) -> Result<(Int, Int), SliceError> {
  ...
}
```

No `@native` appears because the Phalcom body is the implementation.

## 6.5 Public source wrapper over native floor

```phalcom
/// Returns the number of UTF-8 storage bytes.
size -> Int {
  _$byteCount
}
```

Do not mark the wrapper `@native`.

## 6.6 Class-side native method

```phalcom
/// Creates an empty mutable list.
@class
@native
new() -> List<T>
```

The exact generic result is illustrative; implementation must use the ratified source semantics and canonical signature table.

## 6.7 Reference-bodied native method

Reference bodies are permitted only if the body is useful explanatory source and is guaranteed not to replace Rust execution:

```phalcom
@native
somePrimitive(_ x: T) -> U {
  // reference formulation only
  ...
}
```

The default for irreducible floor operations is declaration-only. Do not invent fake implementations merely to make source look complete.

---

# 7. Annotation Policy

The implementation must provide and enforce this matrix.

| Source fact | Required annotation | Forbidden/misleading annotation |
|---|---|---|
| primordial class identity created by Rust/bootstrap | `@native` on class | creating a fresh class and pretending it is primordial |
| Rust-implemented instance method | `@native` | source body treated as live implementation |
| Rust-implemented class-side method | `@class @native` | missing side annotation |
| Rust-implemented `_$` member | `@internal @native` | `@native` without `@internal`; public `_$` |
| Phalcom-implemented `_$` member | `@internal` | `@native` |
| ordinary Phalcom wrapper/algorithm | none of `@native/@internal` unless namespace requires | `@native` because it calls a primitive |
| private source helper | `@private` | `@internal` unless selector is implementation namespace |
| protected source API | `@protected` | `@internal` |
| constructor | ratified constructor annotation/syntax | treating all `new` methods as constructors |

`@internal` is an assertion about implementation namespace and visibility. It is not an authorization capability. Compiler privilege continues to come from canonical universe/core compilation identity.

## 7.1 Mandatory bootstrap-installed method annotation audit

Every method installed directly by bootstrap/native Rust must be represented by a source declaration and pass an annotation audit. There are no unannotated bootstrap-installed methods in the completed system.

For each such method, the census must prove:

```text
@native                 present
@class                  present iff class-side
@internal               present iff implementation selector namespace
@constructor            present only when the language constructor semantics actually apply
@private/@protected     consistent with declared visibility and never used as a substitute for @internal
@total                  present only when termination is proven under Spec 05; never inferred from being native
@requires/@ensures      present only for real language contracts; never synthesized merely from Rust assertions
parameter types         complete
return type             complete
generic binders/where   complete when applicable
Phaldoc                  present
```

Machine-only native facts such as ABI, Rust provenance, coarse effects, raises metadata, intrinsic identity, and trust remain in the primitive descriptor unless a separate ratified source annotation exists. The source must not duplicate these merely to satisfy an “annotation count.”

The same audit covers methods attached during bootstrap by executing universe source: they must have complete source signatures and Phaldoc, but they do **not** receive `@native` unless Rust owns their implementation.

---

# 8. Documentation Policy

## 8.1 Every universe module

Each module begins with durable module documentation:

```phalcom
//! Numeric value types and primitive arithmetic protocol.
```

Package files document the package and its conceptual scope.

## 8.2 Every class

Every class receives an outer Phaldoc block covering:

- semantic role;
- identity/value/reference character where relevant;
- mutability/immutability where relevant;
- relationship to immediate values or hidden runtime identity where relevant;
- important protocol laws;
- whether construction is restricted;
- generic meaning of type parameters.

Do not put migration tickets, task IDs, or historical implementation chronology in class docs.

## 8.3 Every method

Every method receives a summary. Add detail when behavior is not obvious.

Use `@param`, `@returns`, `@throws`, `@see`, and examples where they add information. Do not duplicate a type annotation in prose merely to say “`x` is an Int.” Do not repeat executable `@requires`/`@ensures` in prose.

Internal native hooks also need docs because they are essential to understanding how the source implementation is built over the primitive floor. Their docs should explain representation-level semantics and assumptions without promising them as user API.

## 8.4 Historical-comment cleanup

Current universe files contain many comments such as:

```text
U-CORE-4
DEFERRED.md #18
CB-1
R-INV-5.3
"do not add until phase X"
```

During migration:

- durable semantic rationale becomes `///`/`//!` or a short local `//`;
- implementation-history/task archaeology moves to ADR/PDR/work docs;
- stale phase instructions are deleted;
- exact source semantics remain beside the code.

A finished universe module should read like a standard library/kernel implementation, not a project-management log.

---

# 9. Type Annotation Policy

## 9.1 Types are required where authoritative knowledge exists

Rust primitive metadata already carries parameter and return contracts for the descriptorized surface. The source anchor must express the same contract in Phalcom syntax.

Source wrappers should also be explicitly typed where their contract is stable. This is especially important in the universe because the universe is the canonical source used by semantic tooling and documentation.

## 9.2 Generic core types must stop presenting as monomorphic source shells

The final source presentation must express generic semantics for types such as:

```text
Option<T>
Some<T>
List<T>
Set<T>
Map<K, V>
Iterable<T>          if this is the ratified class/protocol model
Result<T, E>
Ok<T, E>
Err<T, E>
```

The exact binder/variance declarations must follow revised Spec 04 and the canonical declaration model.

This work must not invent a runtime specialized class. Generic declarations are semantic metadata over the existing runtime class identities.

## 9.3 Block/callable parameters

Methods such as `map`, `flatMap`, `filter`, `reduce`, `ifSome`, `ifNone`, `on`, `ensure`, and iteration helpers should use callable types once the current source type syntax can publish them correctly.

Expected examples are conceptual:

```phalcom
map<U>(_ transform: (T) -> U) -> List<U>
filter(_ predicate: (T) -> Bool) -> List<T>
```

The implementation agent must derive exact signatures from the ratified language semantics, current method behavior, and canonical native/source signature machinery.

## 9.4 No fake `Unknown` declarations

If a source type cannot yet be published because a Spec-04 semantic publication gate remains incomplete, the migration records an explicit blocker such as:

```text
Blocked(S4_GENERIC_METHOD_PUBLICATION)
Blocked(S5_TYPE_LAMBDA_BINDER_LOWERING)
Blocked(S7_RECORD_ROW_SEMANTICS)
```

It must not “solve” the blocker by writing a misleading broad type or by treating `Unknown` as successful verification.

Final completion requires those blockers resolved for the universe surface.

---

# 10. Workstream U0 — Build an Exhaustive Universe Census

This is the first implementation task. Do not begin editing dozens of `.ph` files by hand before producing a machine inventory.

## U0.1 Add a VM-free source census

Create a semantic/corpus utility under a suitable existing crate, preferably `phalcom-semantic::core_surface` or `phalcom-modules` plus semantic projection, that can enumerate:

```rust
pub struct UniverseSourceCensus {
    pub modules: Vec<UniverseModuleRow>,
    pub classes: Vec<UniverseClassRow>,
    pub members: Vec<UniverseMemberRow>,
}
```

Each module row records:

```text
ModuleId
SourceLocation
package/module kind
physical/bundled source existence
parse status
module Phaldoc presence
```

Each class row records:

```text
ModuleId
class name
source span
source-created vs @native/primordial
superclass syntax
generic signature
where constraints
class Phaldoc presence
corresponding UniverseKey, if any
corresponding UNIVERSE_CLASS_RELATIONS row, if any
corresponding UNIVERSE_BINDINGS entry, if any
```

Each member row records:

```text
owner class
side
canonical selector
source implementation category
annotations
source body presence
parameter names/labels/types
return type
generic signature
where constraints
Phaldoc presence
native descriptor match, if any
runtime installation match, when tested in VM
```

## U0.2 Use canonical identities

Census member identity is:

```text
(owner UniverseKey or source class identity, dispatch side, canonical selector)
```

Do not use Rust function names or bare method base names.

## U0.3 Add a human-readable report for migration

A test/helper command may render deterministic Markdown/JSON for implementers:

```text
module | class | selector | source-kind | descriptor? | runtime? | typed? | documented?
```

This report is a migration aid, not a checked-in handwritten source of truth.

## U0.4 Establish initial red tests

Before source migration, add tests that deliberately expose current incompleteness but can be staged/ignored behind a named migration gate. The report should identify:

- placeholder modules with no expected class declarations;
- native descriptors with no source anchor;
- source calls to `_$` selectors with no member declaration in the owning class;
- bootstrap classes with no canonical source declaration;
- source classes whose module ownership conflicts with current compatibility overlays;
- physical `.ph` files absent from `UNIVERSE_NODES`;
- `UNIVERSE_NODES` entries missing physical source;
- undocumented classes/members;
- untyped known native/source members.

Do not freeze the current incomplete state as a golden manifest.

---

# 11. Workstream U1 — Finish Attribute and Declaration Infrastructure

The universe corpus cannot become complete until source can express what it needs.

## U1.1 Implement `@internal`

Current `BuiltinAttr` on `main` includes `Native` and `Total` but does not yet include `Internal`. Add it to:

```text
phalcom-ast/src/ast.rs
phalcom-core/src/compiler/attributes.rs
compiler-only attribute handling
LSP semantic token/attribute consumers as needed
```

Enforce:

```text
authored _$selector in privileged universe source -> requires @internal
authored __field in privileged universe source    -> requires @internal
@internal ordinary selector                       -> error
@internal _ordinarySourceField                    -> error
@internal + @private/@protected                    -> visibility conflict
```

Compiler-generated internal members remain exempt from requiring a source annotation.

## U1.2 Support declaration-only callable members

Native anchors need to exist without fake executable bodies.

Introduce or complete an explicit member-body representation:

```rust
pub enum MemberBody {
    Declaration,
    Block(Vec<Statement>),
}
```

Apply it to method/getter/setter forms that can be native anchors.

Parser grammar must distinguish:

```phalcom
@native
foo(_ x: T) -> U
```

from:

```phalcom
foo(_ x: T) -> U {}
```

The first has no source implementation. The second has a real empty source body.

## U1.3 Restrict declaration-only members semantically

The parser may represent declaration-only members generally, but current executable compilation must reject a declaration that survives native/protocol/abstract handling.

For this workstream, canonical universe `@native` declarations are the supported use case.

## U1.4 Support class-level `@native`

A source declaration such as:

```phalcom
@native
class String { ... }
```

must assert that bootstrap already created the exact primordial class identity.

Compiler/bootstrap behavior:

1. resolve class name to canonical `UniverseKey`;
2. resolve `UniverseKey` to existing `ClassId`;
3. verify source superclass against `UNIVERSE_CLASS_RELATIONS`;
4. attach source-defined methods to that same class;
5. never allocate a second class;
6. preserve special globals whose public binding is not the class object.

## U1.5 Preserve the `None` binding

The `None` name currently denotes immediate absence, while a hidden `None` class identity exists in the runtime. A canonical native source declaration must not overwrite the public global.

Native class completion needs an explicit “preserve established public binding” rule. Add a regression test before adding a `None` source presentation.

## U1.6 Make source selector projection canonical

Native verification, compiler member identity, semantic analysis, and LSP must use one AST → canonical selector projection. Do not duplicate encoding rules in each consumer.

---

# 12. Workstream U2 — Canonical Class Ownership and Module Completion

Before adding methods, settle which class belongs in which universe module.

## U2.1 Source owns presentation location

The `.ph` declaration is the authority for where a class is presented in the universe project. Native metadata owns runtime identity and primordial relations, not documentation/module layout.

The source census cross-checks:

```text
source class declaration
↔ UniverseKey / UNIVERSE_BINDINGS
↔ UNIVERSE_CLASS_RELATIONS
```

but must not require a hard-coded path→class overlay after migration.

## U2.2 Resolve current catalog inconsistencies

At minimum investigate and resolve:

- physical `scalar/uri.ph` vs canonical `reflection.uri` node;
- `Unit` source ownership vs current `option.option` compatibility injection;
- any native class injected by `BuiltinInterfaceBuilder` into a module that differs from its actual source declaration;
- any source-only class currently present in a module but exported through an unexpected package.

The resolution may move/delete source or update package exposures, but must leave exactly one canonical declaration.

## U2.3 Delete source-interface class injection incrementally

As each module gains real class declarations, remove the corresponding hard-coded branch from `phalcom-modules/src/builtin_interface.rs`.

Do not wait until the end and keep both sources active. Each module migration should prove that source-derived interfaces now contain the expected declarations without injection.

---

# 13. Workstream U3 — Object and Scalar Universe Completion

This wave establishes the pattern for primordial classes and native/source layering.

## U3.1 `object/object.ph`

Target `Object` as a complete root presentation.

Tasks:

- mark the primordial class appropriately with class-level `@native`;
- add class Phaldoc;
- retain real source implementations such as `is(_)`, `is!(_)`, and derived comparison wrappers where still semantically correct;
- add declaration-only anchors for every bootstrap-installed `Object` native method from the generated descriptor census;
- annotate `_$` implementation hooks with `@internal @native`;
- type all source and native members;
- document every member;
- preserve source-vs-native ownership exactly—do not convert derived behavior into native declarations.

## U3.2 `object/behavior.ph`

- declare `Behavior` as primordial/native if bootstrap owns the identity;
- expose all native behavior/reflection methods from descriptor census;
- declare `_$attributes` and other raw hooks explicitly;
- retain `attributes`/`attributesOfType(_)` wrappers as real `.ph`;
- add exact types and docs.

## U3.3 `object/class.ph`

- present primordial `Class` identity;
- expose `_$new()` and any other native class behavior explicitly;
- keep `new()` as a source wrapper if that is still the intended semantic layering;
- distinguish class-side protocol on represented classes from instance-side methods on class objects;
- document allocation semantics and constructor distinction.

## U3.4 `object/metaclass.ph`

Replace the empty shell with the actual presented metaclass protocol. Do not invent methods that are inherited; declare only owned members plus durable docs.

## U3.5 `object/ellipsis.ph` and `object/ordering.ph`

These are source-owned classes/values unless the census shows native hooks. Keep them source-defined. Add complete types and Phaldoc. Do not add `@native` simply because they are core.

## U3.6 `scalar/number.ph`

This is a priority file because current source is almost empty.

For `Number`, `Int`, and `Float`:

- present each primordial identity;
- declare all native arithmetic/comparison/conversion/hash/rendering methods actually owned by each class;
- add exact source types;
- preserve distinctions between inherited and overridden methods;
- document numeric semantics, mixed numeric behavior, and return types;
- do not infer exact return type from method name when Rust metadata/language semantics say otherwise;
- audit class-side constructors/factories separately from instance arithmetic.

The file must cease being an empty shell around a hidden Rust protocol.

## U3.7 `scalar/string.ph`

Keep the existing rich source implementation. Add the invisible native floor at the top of the class in a dedicated, readable section:

```phalcom
@internal
@native
_$byteCount -> Int

@internal
@native
_$byteAt(_ index: Int) -> Option<Int>

@internal
@native
_$slice(_ start: Int, _ end: Int) -> String
```

Also add source anchors for public native methods owned by `String` according to `NATIVE_SURFACES`.

Then type/document all existing source methods and helper view classes.

## U3.8 `scalar/bool.ph`

- present `Bool`, `True`, and `False` identities correctly;
- make the native control-flow floor visible through declarations;
- preserve `toString` as real source-derived behavior;
- remove historical “sacred selector” project-log prose from the source and retain only durable semantic explanation;
- type/document all methods.

## U3.9 `scalar/symbol.ph`

Replace `class Symbol {}` with the complete native/source surface owned by Symbol. Add docs/types and class presentation annotation.

---

# 14. Workstream U4 — Callable and Option Completion

## U4.1 `callable/function.ph`

- present `Function` and its native call/control gateways;
- retain `attempt()` as real source code;
- remove stale List-related migration commentary that does not belong in this module;
- type/document every callable member;
- represent rest-call gateways accurately.

## U4.2 `callable/closure.ph`

Replace the documentation-only placeholder with a canonical `Closure` class presentation and its owned native members. Do not duplicate inherited `Function` members as owned declarations.

## U4.3 `callable/method.ph`

Present:

```text
Method
BoundMethod
```

according to current runtime/native ownership.

- declare native reflection/invocation/binding/provenance methods;
- declare `_$attributes` if it is an internal native hook;
- retain source wrappers `attributes` and `attributesOfType(_)`;
- type/document every method;
- include implementation provenance methods introduced by Spec 03.5.

## U4.4 `callable/family.ph`

Replace the placeholder with canonical declarations for the actual family tower, currently represented by compatibility overlay names such as:

```text
Family
MethodFamily
BoundMethodFamily
```

Use the generated native surface and runtime class relations to determine ownership/superclass, not the old overlay list alone.

## U4.5 `option/option.ph`

This file should become the exemplary generic source/native hybrid.

- present `Option<T>` and `Some<T>` using the ratified generic declaration model;
- expose native `match(...)`, `Some` construction, and any other primitive floor operations through `@native` declarations;
- retain all derivable combinators as real `.ph`;
- add exact callable generic types to combinators;
- add class/method docs;
- present `None` without clobbering the immediate `None` global;
- resolve `Unit` canonical module ownership instead of preserving the current compatibility mismatch.

`Result`, `Ok`, and `Err` remain source-defined if that is still current architecture. Give them full generic signatures/docs without `@native` unless a native census proves otherwise.

---

# 15. Workstream U5 — Collections Completion

Collections are the largest proof that the architecture works because they mix native storage hooks with extensive Phalcom algorithms.

## U5.1 `collections/iterable.ph`

- make generic element semantics explicit;
- document and type the canonical iteration protocol `iterate(previousCursor)` / `iteratorValue(cursor)`;
- type/document all combinators;
- do not introduce a separate hidden `Iterable<T>` runtime dispatch mechanism;
- ensure `for` semantics continue to use the two-selector protocol.

## U5.2 `collections/list.ph`

Add declarations for the complete raw native floor, including every internal selector used by the source and every public native member in the descriptor census.

Expected categories include operations like:

```text
_$length
_$at(_)
_$set(_,_)
_$push(_)
_$replaceSlice(_,_,_)
```

but implementation must use the actual generated census rather than this illustrative subset.

Then:

- declare `List<T>` generics;
- type every wrapper/algorithm;
- document all public, private, and internal members;
- preserve mutable-flow semantics in source bodies;
- keep iteration protocol source-owned where currently derived.

## U5.3 `collections/map.ph`

Present `Map<K,V>` and its native storage/hash table floor. Explicitly declare all raw hooks used in the file, including lookup, put, membership, removal, and indexed key/value observation.

Type `get`, strict indexing, views, insertion, and iteration so `Option<V>` and key/value relationships are represented correctly.

## U5.4 `collections/set.ph`

Present `Set<T>`, its native hash-storage hooks, and typed source wrappers/structural equality.

## U5.5 `collections/tuple.ph`

Resolve `Unit` ownership first. Then present `Tuple` native layout observations and source protocol.

Declare internal tuple observations such as size/positionals/labels/access/slice/from-list according to actual descriptors. Type positional/labeled operations with the strongest currently expressible source contracts; do not fake row precision before the relevant row semantics are implemented.

## U5.6 `collections/record.ph`

Present the native record observation floor (`_$size`, labels, values, etc.) and source equality/hash/get behavior.

Coordinate exact record typing with the canonical row semantics work. Until row tails/structural row publication is complete, mark precision blockers explicitly; final completion requires the proper row-backed type model, not a stringly approximation.

## U5.7 `collections/range.ph`

This file must demonstrate the `@internal`/`@native` distinction:

- native representation readers such as `_$lower`, `_$upper`, `_$upperInclusive` get `@internal @native` declarations;
- source-implemented `_$sliceBounds(_)` gets `@internal` only;
- public progression/iteration methods remain source bodies;
- all members get types/docs.

## U5.8 `collections/bytes.ph`

This is a large hybrid module. Declare the entire native byte-buffer and resource floor used by source, including the actual descriptorized equivalents of operations such as:

```text
_$size
_$at(_)
_$set(_,_)
_$fill(_)
_$utf8
_$utf8Lossy
_$slice(_,_)
_$copyInto(_,_)
_$equalsConstantTime(_)
_$fromString(_)
```

as well as resource hooks such as registration/close/status when owned by classes in the same module.

Keep validation, composition, path/resource/stream algorithms in source. Type and document every class and member in the file, not only `Bytes`.

---

# 16. Workstream U6 — Errors and Concurrency Completion

## U6.1 Error modules

`errors/error.ph` must present the native Error root surface, including native message/raise behavior, while retaining source constructors/fields and source-defined error classes.

For every error module:

```text
error.ph
argument.ph
indexing.ph
contracts.ph
unsupported.ph
unimplemented.ph
```

- add module/class/method Phaldoc;
- add complete types;
- mark only primordial/native classes as `@native`;
- expose native methods where the descriptor census says they exist;
- preserve source-only error classes as source classes.

## U6.2 `concurrency/fiber.ph`

Perform a full native/source classification of the Fiber surface.

This module is large and historically carried primitives that escaped floor census. It therefore receives extra strictness:

- every bootstrap-installed Fiber method must have an exact source anchor;
- every native effect/raise/flow fact remains descriptor-owned and may be presented in tooling without duplicating it in source annotations unless the language has a ratified source annotation for that fact;
- source wrappers/algorithms remain `.ph`;
- all native/internal methods are documented;
- tests compare the entire Fiber source/native/runtime key set, not a manually selected subset.

---

# 17. Workstream U7 — Reflection and Typing Reflection Completion

This workstream removes the largest concentration of documentation-only placeholders.

## U7.1 General rule

Every reflection module in `UNIVERSE_NODES` becomes a real source presentation. A file containing only:

```phalcom
@!documentation("...")
```

is not complete when a runtime class is injected into its interface.

## U7.2 Reflection modules to complete

Current canonical nodes include:

```text
reflection/module.ph
reflection/package-object.ph
reflection/project.ph
reflection/project-manifest.ph
reflection/package-info.ph
reflection/package-author.ph
reflection/package-requirement.ph
reflection/resolved-project-dependency.ph
reflection/module-dependency.ph
reflection/export-table.ph
reflection/export.ph
reflection/export-kind.ph
reflection/child-module-table.ph
reflection/module-identity.ph
reflection/package-identity.ph
reflection/project-identity.ph
reflection/uri.ph
reflection/selector.ph
reflection/message.ph
reflection/attribute.ph
reflection/implementation.ph
```

For each file:

1. identify all canonical classes/values owned by the module from runtime identities, generated native surface, and current module interface;
2. author the class declarations in `.ph`;
3. annotate primordial/native classes;
4. declare every owned native method;
5. keep any source methods as real implementations;
6. add complete signatures;
7. add Phaldoc for module, classes, and methods;
8. remove the corresponding `BuiltinInterfaceBuilder` injection row.

## U7.3 Typing reflection modules to complete

The `reflection/typing` package currently contains:

```text
kind.ph
type-descriptor.ph
type-parameter.ph
generic-signature.ph
signature.ph
type-use.ph
result.ph
evidence.ph
context.ph
```

These must present the runtime reflection tower implemented by Specs 02/03/03.5, including current classes such as:

```text
KindDescriptor
AtomicKind
FunctionKind
Type
TypeDescriptor
AppliedType
UnionType
TupleType
RecordType
CallableType
TypeLambda
SpecialType
SelfType
TypeParameter
GenericSignature
GenericConstraint
CallableSignature
CallableParameter
FieldSignature
TypeUse
TypingResult and result-state variants
TypeRelationResult and relation-state variants
MemberLookupResult and member-state variants
RelationEvidence
RelationFailure
DynamicBoundary
ReflectionCapability
TypingContext
Typing
```

The above list is a grounding seed from current compatibility interface code, not a new handwritten authority. The implementation must reconcile it against `UNIVERSE_BINDINGS`, `UNIVERSE_CLASS_RELATIONS`, generated native surfaces, and the live runtime, then let source become authoritative for presentation.

## U7.4 Correct current runtime reflection bugs while presenting them

Source declarations make inconsistencies visible. In particular, kind reflection must agree with semantic truth for generic constructors such as `Option`/`Some`, not only historically hard-coded List/Set/Map cases.

Do not change semantic design to match a runtime projection bug. Repair the runtime projection and test source/semantic/runtime agreement.

---

# 18. Workstream U8 — Complete Package Files and Exports

Every `package.ph` becomes part of the documentation and integrity story.

For each package:

- add `//!` package documentation;
- expose every canonical child module exactly once;
- ensure exposed children match `UNIVERSE_NODES.children`;
- ensure source class exports/re-exports match intended public universe presentation;
- ensure prelude membership remains independently controlled by `UNIVERSE_BINDINGS` and is not inferred merely from package exposure.

Add tests that compare package `expose` declarations with `UNIVERSE_NODES` instead of maintaining two unchecked lists.

---

# 19. Workstream B0 — Build the Canonical `UniverseSourceIndex`

Once the corpus can express native declarations, bootstrap needs one parsed semantic index.

Target shape, in a VM-free layer:

```rust
pub struct UniverseSourceIndex {
    pub modules: BTreeMap<ModuleId, UniverseModulePresentation>,
    pub classes: BTreeMap<UniverseSourceClassKey, UniverseClassPresentation>,
    pub native_members: BTreeMap<PrimitiveKey, UniverseNativeMemberPresentation>,
    pub internal_source_members: BTreeMap<UniverseMemberKey, UniverseSourceMemberPresentation>,
}
```

A native member presentation retains:

```text
ModuleId
SourceLocation
owner class
side
canonical selector
member kind
source range/name range
parameter names/labels
canonical semantic signature/generic signature
where constraints
documentation span/text
@internal state
body kind (declaration/reference)
```

The index is built from `BuiltinProjectSourceProvider(BuiltinProject::Universe)` parsed units, not from `core/core.ph` and not from a VM source list.

---

# 20. Workstream B1 — Source ↔ Native Descriptor Verification

## B1.1 Identity matching

For every source `@native` member, find exactly one generated/runtime primitive descriptor by:

```text
(owner UniverseKey, dispatch side, canonical selector)
```

Rust function name is provenance only.

## B1.2 Verify structural agreement

The verifier checks:

- owner;
- dispatch side;
- selector;
- member kind;
- positional/labeled/rest parameter shape;
- generic binders;
- parameter types;
- return type;
- `where` constraints where represented by native/source semantic signatures;
- internal visibility;
- lifecycle/source-native provenance rules where relevant.

Use the canonical `TypeStore` / semantic type terms / `CallableSemanticSignature`. Do not create a second type model just for bootstrap.

## B1.3 Distinct verification outcomes

Use explicit states rather than boolean “matched”:

```text
Verified
Mismatch
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

A source type that cannot yet be lowered because a semantic publication gate is incomplete is `Blocked`, not “equal enough.”

## B1.4 Reverse completeness

Final strict mode requires:

```text
required descriptor keys == source @native member keys
```

Any intentionally hidden native primitive needs an explicit machine-level hidden/presentation policy. Hidden must be rare; do not maintain a loose exemption string list.

## B1.5 Class completeness

Likewise require every intended primordial universe class to have exactly one source class presentation and correct superclass relation.

---

# 21. Workstream B2 — Replace the Bootstrap Source List

`VM::run_universe_modules()` must stop owning direct `include_str!` entries.

Use:

```rust
BuiltinProjectSourceProvider::new(BuiltinProject::Universe)
```

and canonical module enumeration/order.

Requirements:

1. one source corpus authority;
2. one parse result per module per bootstrap;
3. source verification and later compilation operate on the same parsed program;
4. diagnostics use canonical `phalcom://universe/...` identity or physical workspace source when appropriate;
5. no source file can be accidentally omitted from runtime bootstrap while still appearing to tooling.

If current runtime still needs a compatibility topological execution order, derive/store that order in the builtin project/module layer, not as another source-text array inside VM bootstrap.

---

# 22. Workstream B3 — Parse Once, Verify Once, Compile the Same AST

Current bootstrap source verification must not parse a file and then later compile a separately reparsed string.

Add/complete an AST-taking compiler entry point so bootstrap can pass the verified `Program` directly into compilation.

Conceptually:

```rust
compile_parsed_program_as_with_bindings(
    module,
    parsed_unit,
    bindings,
)
```

This guarantees that the declarations stripped as `@native` are the exact declarations that preflight verified.

---

# 23. Workstream B4 — Make Bootstrap Explicitly Phased and Fallible

Introduce a testable fallible bootstrap path, preserving `VM::new()` as a convenience wrapper if desired.

Recommended phases:

```text
B4.1 allocate kernel heap + primordial class tower
B4.2 initialize core module / universe package identities
B4.3 stamp irreducible fixed layouts required before source compile
B4.4 validate native descriptor registry
B4.5 load + parse canonical universe source project
B4.6 build UniverseSourceIndex
B4.7 verify class/source/native contracts
B4.8 build/register native semantic metadata
B4.9 install descriptor primitives exactly once
B4.10 finalize native-only base-name indexes needed for source execution
B4.11 compile/execute real source implementations
B4.12 resolve source-exported semantic roots
B4.13 verify globals, class relations, layouts, methods, typing side tables
B4.14 freeze/publish verified universe
```

No universe source body executes before B4.7 succeeds.

No primitive is installed before the source/native preflight has at least validated the strict structural surface required by the selected migration mode.

---

# 24. Workstream B5 — Descriptor-Only Native Installation

Final default startup must not be `Dual`.

Migration:

1. make `spec03_5_census` assert generated == descriptor for migrated surface;
2. migrate remaining legacy primitive registrations into descriptors;
3. make source anchors complete;
4. assert descriptor == required source-native anchors;
5. switch bootstrap default to descriptor-only;
6. remove legacy fallback;
7. delete `descriptor_floor_is_complete()` once there is no compatibility floor to compare;
8. delete ordinary-language responsibilities from `Universe::install_primitives`;
9. remove `NATIVE_MEMBERS` as a runtime/bootstrap dependency.

A primitive must never be installed through both paths in the final system.

---

# 25. Workstream B6 — Native Method Semantic Metadata

For every verified native source/descriptor pair, create the canonical callable semantic record used by runtime reflection.

Source contributes:

```text
parameter local names
source spans
Phaldoc/source identity
generic/source signature presentation
```

Native descriptor contributes:

```text
implementation provenance
ABI
effects
raises
return-flow metadata
intrinsic id
trust/lifecycle
machine native signature projection
```

The verifier proves the overlapping type/shape facts agree.

Installed `MethodObject`s are associated through `MethodSemanticIndex`; do not bloat each method object with a duplicate full metadata graph.

---

# 26. Workstream B7 — Native Class Completion and Layout Verification

When canonical source declares a primordial class:

```phalcom
@native
class Error { ... }
```

bootstrap/compiler must complete the existing class rather than create a new one.

Verify:

- exact class identity;
- exact superclass;
- fixed native slot layout compatibility;
- source-added fields do not alias inherited/native fixed slots incorrectly;
- source methods attach to intended class side;
- source declarations cannot silently change native representation.

Classes with hand-stamped fixed layouts in current `VM::new()`—for example `Message`, `Error`, `MessageNotUnderstood`, resource/error classes, and any others found by census—need explicit layout parity tests.

Longer-term cleanup may move layout facts into one canonical metadata representation, but this workstream's immediate requirement is to **verify**, not duplicate them in `.ph` as fake fields.

---

# 27. Workstream B8 — Special Binding and Immediate-Value Tests

Bootstrap must explicitly prove that source presentation does not corrupt special runtime bindings.

At minimum test:

```text
None global == immediate absence
None global != hidden None class object
true.class == True
false.class == False
numeric immediates map to intended classes
Symbol/string/immediate class identities remain stable
universe.X and prelude X share exact class object where both are exported
universe-only classes remain accessible without accidental prelude leakage
```

Adding source presentation must never change these object identities.

---

# 28. Workstream T0 — Remove Synthetic Interface Authority

Once a module is complete, `phalcom-modules/src/builtin_interface.rs` must derive its declarations from source rather than inject primordial names for that module.

Final state:

- no giant path→class-name match for ordinary universe source declarations;
- root/prelude/binding metadata may still use `UNIVERSE_BINDINGS` for runtime identity/export policy;
- class presentation, source location, docs, generic syntax, and method declarations come from source.

Add a test that fails if an interface declaration exists only because of a native overlay when the class is supposed to be source-presented.

---

# 29. Workstream T1 — Retire `core/core.ph` as a Competing Authority

`phalcom-core/core/core.ph` is a legacy concatenated/parallel representation of much of the same core source. The canonical authority is the universe project.

Migration:

1. identify every consumer of `core/core.ph`;
2. migrate LSP/tooling/bootstrap consumers to `BuiltinProjectSourceProvider(Universe)` and real module identities;
3. if a compatibility artifact is temporarily required, generate it from canonical universe sources rather than hand-editing it;
4. delete or clearly demote the file once no consumer requires it.

No implementation is complete while a developer can edit `core/core.ph` and change one tool's view without changing the canonical universe modules.

---

# 30. Workstream T2 — LSP and Documentation Presentation

The LSP remains VM-free.

For a native method, tooling should merge:

```text
canonical .ph source declaration
+ generated native implementation metadata
+ canonical semantic signature
= one semantic member
```

Go-to-definition lands in the `.ph` module.

Hover can display:

- selector;
- generic/type signature;
- Phaldoc summary/details;
- native/source implementation kind;
- internal/public visibility;
- effects/raises/lifecycle when available from native metadata;
- optional Rust implementation provenance through an implementation-navigation action.

Do not create a synthetic duplicate native member when a source `@native` declaration already exists.

---

# 31. Exhaustive Corpus Work Matrix

The implementation must track every canonical module. The matrix below is an ownership/checklist seed; the machine census is authoritative.

## 31.1 Object package

| File | Required outcome |
|---|---|
| `object/package.ph` | package docs; exposures exactly match catalog |
| `object/object.ph` | complete `Object` source/native surface, types/docs |
| `object/behavior.ph` | complete `Behavior` source/native surface, raw attribute hooks declared |
| `object/class.ph` | complete `Class` surface, allocation floor visible |
| `object/metaclass.ph` | replace empty shell with canonical presentation |
| `object/ellipsis.ph` | source singleton fully typed/documented; no fake native markers |
| `object/ordering.ph` | source ordering value/class fully typed/documented |

## 31.2 Scalar package

| File | Required outcome |
|---|---|
| `scalar/package.ph` | package docs/catalog parity |
| `scalar/number.ph` | complete `Number`/`Int`/`Float` native protocol |
| `scalar/string.ph` | retain algorithms; add all native/internal anchors; type/docs everywhere |
| `scalar/bool.ph` | expose native control floor; retain source display; type/docs |
| `scalar/symbol.ph` | complete native surface |
| `scalar/uri.ph` | resolve orphan/stale ownership; delete or register intentionally |

## 31.3 Callable package

| File | Required outcome |
|---|---|
| `callable/package.ph` | package docs/catalog parity |
| `callable/function.ph` | complete Function floor + source `attempt`; remove stale unrelated comments |
| `callable/closure.ph` | real Closure declaration/surface instead of docs-only placeholder |
| `callable/method.ph` | Method + BoundMethod surface, reflection/provenance hooks |
| `callable/family.ph` | Family/MethodFamily/BoundMethodFamily canonical declarations |

## 31.4 Option package

| File | Required outcome |
|---|---|
| `option/package.ph` | package docs/catalog parity |
| `option/option.ph` | generic Option/Some/None presentation, native floor anchors, source combinators, Result family types/docs, resolve Unit ownership |

## 31.5 Collections package

| File | Required outcome |
|---|---|
| `collections/package.ph` | package docs/catalog parity |
| `collections/iterable.ph` | typed/documented iteration protocol and generic combinators |
| `collections/list.ph` | complete List native floor + source API |
| `collections/map.ph` | complete Map native floor + source API/views |
| `collections/set.ph` | complete Set native floor + source API |
| `collections/tuple.ph` | complete Tuple/Unit ownership, native observations, source API |
| `collections/record.ph` | complete Record native observations + source API |
| `collections/range.ph` | native readers + source-internal helpers correctly distinguished |
| `collections/bytes.ph` | complete Bytes/resource native floor + all source classes typed/documented |

## 31.6 Errors package

| File | Required outcome |
|---|---|
| `errors/package.ph` | package docs/catalog parity |
| `errors/error.ph` | Error/native root + source constructors + all resident error classes |
| `errors/argument.ph` | complete source error declarations/docs/types |
| `errors/indexing.ph` | complete source error declarations/docs/types |
| `errors/contracts.ph` | complete contract error declarations/docs/types |
| `errors/unsupported.ph` | singleton/source error presentation, docs/types |
| `errors/unimplemented.ph` | source error presentation, docs/types |

## 31.7 Concurrency package

| File | Required outcome |
|---|---|
| `concurrency/package.ph` | package docs/catalog parity |
| `concurrency/fiber.ph` | exhaustive Fiber native/source surface; strict census parity; all docs/types |

## 31.8 Reflection package

Every current node becomes a real presentation rather than relying on compatibility injection:

```text
reflection/module.ph
reflection/package-object.ph
reflection/project.ph
reflection/project-manifest.ph
reflection/package-info.ph
reflection/package-author.ph
reflection/package-requirement.ph
reflection/resolved-project-dependency.ph
reflection/module-dependency.ph
reflection/export-table.ph
reflection/export.ph
reflection/export-kind.ph
reflection/child-module-table.ph
reflection/module-identity.ph
reflection/package-identity.ph
reflection/project-identity.ph
reflection/uri.ph
reflection/selector.ph
reflection/message.ph
reflection/attribute.ph
reflection/implementation.ph
reflection/package.ph
```

## 31.9 Typing reflection package

```text
reflection/typing/package.ph
reflection/typing/kind.ph
reflection/typing/type-descriptor.ph
reflection/typing/type-parameter.ph
reflection/typing/generic-signature.ph
reflection/typing/signature.ph
reflection/typing/type-use.ph
reflection/typing/result.ph
reflection/typing/evidence.ph
reflection/typing/context.ph
```

All runtime reflection classes and methods must be source-presented, typed, and documented.

---

# 32. Test Strategy — Source Corpus

## 32.1 Physical corpus parity

Add a repository test that walks `phalcom-core/core/universe/src` and compares it with `UNIVERSE_NODES`.

Assert:

```text
physical canonical files == provider nodes
```

with package-file normalization handled explicitly.

No orphan files; no missing files.

## 32.2 Package exposure parity

For each package:

```text
package.ph expose children == UNIVERSE_NODES.children
```

unless a documented semantic distinction requires an explicit exception type rather than a loose string list.

## 32.3 Class presentation parity

Assert every expected universe/primordial class has exactly one canonical source declaration.

Assert no duplicate source presentation of the same `UniverseKey`.

Assert source superclass matches `UNIVERSE_CLASS_RELATIONS` for primordial classes.

## 32.4 Source-native member parity

Final strict assertion:

```text
source @native keys == required NATIVE_SURFACES keys == PRIMITIVES keys
```

where explicitly hidden machine primitives are represented by a typed policy, not an ad hoc exclusion list.

## 32.5 Internal namespace parity

For every authored source member:

```text
selector starts _$  => @internal
@internal            => selector starts _$ OR implementation field rule
```

Then:

```text
@internal + @native  => matching native descriptor with Internal visibility
@internal only       => no native descriptor for that key; body must be real source implementation
```

This catches accidental marking of `Range#_$sliceBounds(_)` as native and accidental omission of `@native` on a raw Rust floor hook.

## 32.6 Declaration/body parity

Assert:

```text
@native declaration body = Declaration or explicitly permitted ReferenceBody
source-owned member       = executable Block
```

and prove native declarations emit no executable method that overwrites the installed native primitive.

## 32.7 Typing completeness

For every core class/member with known contract:

- source type lowering succeeds;
- canonical signature is publishable;
- native/source overlapping signatures compare equal;
- no canonical inference variables survive;
- no `Unknown` is accepted as verification success.

## 32.8 Documentation completeness

Add a source-doc lint/test for the universe corpus:

- every module/package has `//!` or accepted module metadata doc during migration;
- every class has `///`;
- every public/protected/native/internal member has `///`;
- final strict mode also requires docs on private helpers, because this is the language kernel reference source;
- no dangling doc blocks;
- no duplicate doc contract facts where machine annotations are authoritative;
- selector-keyed association survives overloaded arity/labels.

Until Phaldoc has a full AST representation, the lint may use a dedicated source trivia scanner. It must not change runtime semantics.

---

# 33. Test Strategy — Bootstrap

## 33.1 Preflight-before-execution test

Inject a fixture mismatch between source native declaration and descriptor.

Assert bootstrap fails before:

- installing the mismatched primitive;
- executing any universe source body;
- mutating semantic roots dependent on universe execution.

## 33.2 Descriptor-only startup test

Create VM with descriptor-only mode and require complete startup. Once green, make it the default.

Then remove/fail any path that depends on legacy installation.

## 33.3 Exactly-once installation

Record all installed native method keys and assert each required primitive is installed exactly once.

## 33.4 Runtime surface parity

After bootstrap, enumerate live methods owned by primordial classes and compare native method identity/provenance against descriptor/source census.

Source-defined methods are expected additional rows; native-owned rows must match one-to-one.

## 33.5 Class relation parity

Retain and strengthen the current `UNIVERSE_CLASS_RELATIONS` runtime test.

Also compare source-declared superclass templates for primordial classes.

## 33.6 Global identity invariants

Test `None`, booleans, universe/prelude class identity, semantic roots, and universe-only classes.

## 33.7 Layout invariants

For fixed-layout primordial classes, assert source completion leaves field slots/counts unchanged and compatible.

## 33.8 Base-name/selector indexes

After native install and after source completion, verify selector/base-name indexes contain both native and source members and are deterministic/idempotent.

## 33.9 Runtime typing registry

For representative and then exhaustive native methods:

- `MethodSemanticIndex` lookup succeeds;
- callable record resolves;
- type authority is trusted native where appropriate;
- source span points to canonical `.ph` declaration;
- native implementation provenance points to Rust descriptor source;
- both refer to the same semantic callable identity.

---

# 34. Test Strategy — Tooling

## 34.1 Builtin interface test

Build every universe module interface solely from source after migration and assert all expected declarations/exports remain.

## 34.2 Go-to-definition

For representative classes/methods across every package, go-to-definition lands in the actual universe `.ph` module, not a synthetic overlay or `core/core.ph`.

## 34.3 Hover

Native method hover contains:

```text
canonical source signature
Phaldoc
native implementation marker/provenance
machine effects/raises metadata when available
```

with no duplicate synthetic member.

## 34.4 Completion visibility

Internal `_$` members remain available to semantic analysis but are excluded from ordinary user completion outside privileged/internal presentation modes.

---

# 35. Migration Sequence

The sequence below is designed so every commit leaves a coherent architecture rather than creating a giant all-at-once source rewrite.

## Phase 0 — Census and red-gap report

- [ ] Add exhaustive `UniverseSourceCensus`.
- [ ] Compare physical corpus with `UNIVERSE_NODES`.
- [ ] Compare compatibility interface classes with actual source classes.
- [ ] Compare generated native surface/descriptors with source-native anchors.
- [ ] Produce deterministic migration report.
- [ ] Record blockers by semantic gate rather than filling unknowns.

**Gate:** every missing source presentation is mechanically enumerable.

## Phase 1 — Source declaration infrastructure

- [ ] Add `@internal` builtin attribute and legality.
- [ ] Add declaration-only member representation/syntax.
- [ ] Make `@native` class completion legal only for canonical privileged universe source.
- [ ] Add canonical source selector projection.
- [ ] Add special-global preservation for native class completion.
- [ ] Add tests for all annotation/body combinations.

**Gate:** source can truthfully express every native/internal class/member category.

## Phase 2 — Object + scalar source migration

- [ ] Complete `Object`.
- [ ] Complete `Behavior`.
- [ ] Complete `Class`.
- [ ] Complete `Metaclass`.
- [ ] Type/document `Ellipsis`/`Ordering`.
- [ ] Complete `Number`/`Int`/`Float`.
- [ ] Complete `String` floor declarations + source signatures/docs.
- [ ] Complete `Bool`/`True`/`False`.
- [ ] Complete `Symbol`.
- [ ] Remove corresponding builtin interface injections.

**Gate:** the fundamental object/scalar tower is understandable entirely from canonical source.

## Phase 3 — Callable + Option source migration

- [ ] Complete Function/Closure/Method/BoundMethod/family classes.
- [ ] Complete Option/Some/None generic presentation.
- [ ] Type/document Result/Ok/Err source surface.
- [ ] Resolve Unit ownership.
- [ ] Remove corresponding interface injections.

**Gate:** core callable/absence behavior has complete source/native presentation.

## Phase 4 — Collections source migration

- [ ] Complete Iterable.
- [ ] Complete List.
- [ ] Complete Map.
- [ ] Complete Set.
- [ ] Complete Tuple/Unit.
- [ ] Complete Record.
- [ ] Complete Range with native-vs-source-internal distinction.
- [ ] Complete Bytes and all resident resource/path/stream classes.
- [ ] Remove corresponding interface injections.

**Gate:** every collection's native representation floor is visible beside its real source algorithms.

## Phase 5 — Errors + concurrency source migration

- [ ] Complete all error modules.
- [ ] Complete Fiber exhaustively.
- [ ] Remove corresponding interface injections.

**Gate:** no runtime error/concurrency method remains invisible in source.

## Phase 6 — Reflection source migration

- [ ] Replace every reflection placeholder with actual class declarations.
- [ ] Complete module/package/project/selector/message/attribute/implementation reflection.
- [ ] Remove corresponding interface injections.

**Gate:** general runtime reflection is source-navigable.

## Phase 7 — Typing reflection source migration

- [ ] Complete every reflection/typing module.
- [ ] Verify runtime kind/type projections against semantic truth.
- [ ] Fix Option/Some and any other generic-kind projection mismatches.
- [ ] Remove typing reflection interface injections.

**Gate:** runtime typing reflection API is completely visible and documented in canonical universe source.

## Phase 8 — Bootstrap strict preflight

- [ ] Build `UniverseSourceIndex` from provider parsed units.
- [ ] Verify class/native contracts before install/execute.
- [ ] Parse once and compile same AST.
- [ ] Replace VM hard-coded source list.
- [ ] Add fallible bootstrap test seam.

**Gate:** deliberate source/native mismatch prevents bootstrap before execution.

## Phase 9 — Descriptor-only native floor

- [ ] Finish descriptor migration.
- [ ] Strengthen census equality assertions.
- [ ] Switch default startup away from Dual.
- [ ] Remove legacy fallback and `descriptor_floor_is_complete()`.
- [ ] Remove `NATIVE_MEMBERS` bootstrap dependency.

**Gate:** every native method installs once from descriptor registry and has one source anchor.

## Phase 10 — Runtime typing + tooling convergence

- [ ] Attach verified source/native callable records to installed methods.
- [ ] Make LSP source-backed for all universe declarations.
- [ ] Retire `core/core.ph` authority.
- [ ] Remove remaining synthetic native class/member overlay paths.

**Gate:** runtime reflection, semantic analysis, docs, and LSP all navigate the same source declaration.

## Phase 11 — Documentation and strict completeness

- [ ] Add/finalize Phaldoc on every module/class/member.
- [ ] Remove historical task commentary from universe source.
- [ ] Enable strict documentation lint.
- [ ] Enable strict typing completeness lint.
- [ ] Enable strict class/member/native parity tests.

**Gate:** zero missing declarations, zero missing known signatures, zero missing docs, zero orphan files, zero required native descriptors without source anchors.

---

# 36. Per-File Rust Change Map

The exact implementation may discover small additional consumers, but these are the primary ownership points.

## AST/parser

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
potential shared selector projection module in phalcom-ast
```

Changes:

- `BuiltinAttr::Internal`;
- declaration-only `MemberBody`;
- class/member `@native` presentation syntax handling;
- source selector projection;
- tests.

## Compiler attributes/class lowering

```text
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/error.rs
other member-body consumers
```

Changes:

- `@internal` legality;
- privileged `@native` class/member rules;
- native class completion;
- special-global preservation;
- declaration bodies never reaching executable lowering;
- layout checks;
- structured diagnostics.

## Semantic core surface

```text
phalcom-semantic/src/core_surface/**
phalcom-semantic/src/types/** as consumers, not a new parallel model
phalcom-semantic/src/signature.rs / callable signature tables as appropriate
```

Changes:

- `UniverseSourceIndex` / census;
- source-native callable merge;
- source class/native class conformance;
- canonical signature comparison;
- explicit blocked/failure states;
- documentation/source presentation data if semantic layer owns it.

## Modules/builtin provider

```text
phalcom-modules/src/builtin.rs
phalcom-modules/src/builtin_interface.rs
phalcom-modules/tests/builtin_catalog.rs
```

Changes:

- canonical node enumeration API;
- physical-corpus parity test;
- package exposure parity;
- remove class-name injection as modules become source-complete;
- provider parsed units become bootstrap source authority.

## Native surface/registry

```text
phalcom-native-surface/src/generated.rs  # generated output, not hand-edited
phalcom-native-surface/src/lib.rs
phalcom-core/src/native/registry.rs
phalcom-core/src/native/install.rs
phalcom-core/tests/spec03_5_census.rs
```

Changes:

- strict generated/descriptor/source parity;
- descriptor-only installation;
- installed method refs returned/registered;
- legacy compatibility deletion.

## Bootstrap

```text
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/vm/mod.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/universe/**
```

Changes:

- fallible phased bootstrap;
- provider source loading;
- source preflight;
- parse-once compile;
- descriptor-only default;
- legacy installer deletion;
- strengthened runtime invariants.

## Runtime typing

```text
phalcom-core/src/typing/registry.rs
phalcom-core/src/typing/side_table.rs
phalcom-core/src/modules/materialize.rs
```

Changes:

- source-backed trusted-native callable registration;
- safe metadata pool allocation where still needed;
- reflection tests.

## LSP

```text
phalcom-lsp/src/semantic/core_source.rs
phalcom-lsp/src/semantic/surface.rs
phalcom-lsp/src/selectors.rs or shared replacement
hover/completion/definition/index consumers
```

Changes:

- remove synthetic core/native overlay authority;
- consume actual universe modules;
- merge source + generated native provenance;
- source-backed docs/navigation;
- internal completion filtering.

---

# 37. Required Diagnostics

Add structured diagnostics for source-corpus and bootstrap integrity. Suggested stable concepts:

```text
universe.orphan_source_file
universe.missing_source_file
universe.package_catalog_mismatch
universe.duplicate_class_presentation
universe.missing_class_presentation
universe.class_identity_mismatch
universe.superclass_mismatch
universe.native_anchor_missing
universe.native_anchor_orphan
universe.native_anchor_duplicate
universe.native_side_mismatch
universe.native_selector_mismatch
universe.native_visibility_mismatch
universe.native_parameter_shape_mismatch
universe.native_parameter_type_mismatch
universe.native_return_type_mismatch
universe.source_signature_blocked
universe.internal_annotation_required
universe.internal_annotation_mismatch
universe.native_annotation_required
universe.native_body_illegal
universe.documentation_missing
universe.type_annotation_missing
bootstrap.native_install_duplicate
bootstrap.special_binding_clobbered
bootstrap.native_layout_mismatch
```

Mismatch diagnostics should show both source and Rust provenance.

---

# 38. Performance Requirements

The correctness work must not make startup pathologically expensive.

Requirements:

1. parse each universe source module once per bootstrap revision;
2. cache/provider reuse where existing immutable builtin source allows it;
3. build descriptor lookup maps once, keyed by canonical primitive identity;
4. canonicalize source type syntax through existing semantic stores, not repeated string parsing per comparison;
5. source/native verification linear or near-linear in number of classes/members;
6. no VM execution required by LSP to obtain core surface;
7. no reference-body execution during verification;
8. deterministic ordering for diagnostics/tests.

Do not optimize away verification before measuring it. The native/core surface is small compared with ordinary program analysis.

---

# 39. Deletion Ledger

The project is not complete until these compatibility mechanisms are deleted or cease to be authoritative.

| Compatibility mechanism | Deletion condition |
|---|---|
| hard-coded class-name injection in `BuiltinInterfaceBuilder` | every injected class has canonical source declaration |
| `VM::run_universe_modules` static `SOURCES` `include_str!` list | provider-based canonical loading + order is green |
| `NativeInstallMode::Dual` as default | descriptor-only startup passes full workspace/invariant tests |
| legacy ordinary-language entries in `Universe::install_primitives` | descriptor census complete |
| `descriptor_floor_is_complete()` fallback comparison | no legacy floor remains |
| `NATIVE_MEMBERS` as bootstrap/runtime authority | generated canonical surface + descriptors + source anchors are strict-equal |
| synthetic LSP core/native member overlays | universe source declarations are complete |
| `phalcom-core/core/core.ph` as editable canonical source | all consumers use universe project modules |
| placeholder `.ph` modules containing only metadata docs for runtime classes | actual class/member declarations authored |
| historical migration/task commentary in universe source | durable docs extracted/rewritten |
| untyped known native method declarations | source signature publication complete |
| undocumented core class/member | strict Phaldoc lint green |

---

# 40. Acceptance Gates

## V0 — Corpus closed

- every physical universe source file belongs to canonical catalog or is intentionally removed;
- every canonical node has source;
- package children/exposures agree.

## V1 — Class presentation closed

- every intended universe class has one source declaration;
- every primordial source class resolves to exact `UniverseKey`/runtime identity;
- source superclass agrees with canonical relation.

## V2 — Native member presentation closed

- every required native descriptor has one `@native` source anchor;
- every `@native` source anchor has one descriptor;
- side/selector/visibility/member kind agree.

## V3 — Type contracts closed

- every native/source core member has canonical source type signature where semantically representable;
- source/native signatures compare structurally through canonical semantic types;
- no successful match relies on `Unknown`.

## V4 — Internal namespace closed

- every authored `_$` declaration has `@internal`;
- native `_$` declarations also have `@native`;
- source `_$` bodies do not have `@native`.

## V5 — Documentation closed

- every module/package/class/member has Phaldoc;
- no placeholder-only runtime class module remains;
- no task-archaeology comments dominate stable source.

## V6 — Bootstrap preflight closed

- source/native mismatch fails before install/execution;
- verified AST is the AST compiled;
- canonical provider is source authority.

## V7 — Native install closed

- descriptor-only startup is default and green;
- every native method installs exactly once;
- legacy native floor authority removed.

## V8 — Runtime identity/layout closed

- class tower, special globals, fixed layouts, base-name indexes, and semantic roots remain correct after source completion.

## V9 — Runtime typing closed

- installed native methods resolve through `MethodSemanticIndex` to source-backed trusted-native callable records.

## V10 — Tooling closed

- LSP uses actual universe modules;
- navigation/hover/docs are source-backed;
- no required native class/member exists only through synthetic overlay.

## V11 — Full workspace verification

Run and record actual outputs for repository-standard checks, at minimum:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Use repository CI policy if its exact command differs. Do not claim these gates without executing them in the implementation checkout.

---

# 41. Example End State: `String`

Illustrative source shape:

```phalcom
//! UTF-8 string values and source-derived text operations.

/// Immutable UTF-8 string value.
@native
class String {
  /// Concatenates this string with `other`.
  @native
  +(_ other: String) -> String

  /// Returns a stable hash of the string contents.
  @native
  hash -> Int

  /// Returns the number of bytes in the UTF-8 representation.
  @internal
  @native
  _$byteCount -> Int

  /// Returns the byte at `index`, or `None` when `index` is out of bounds.
  @internal
  @native
  _$byteAt(_ index: Int) -> Option<Int>

  /// Returns the byte range `[start, end)` as a string.
  @internal
  @native
  _$slice(_ start: Int, _ end: Int) -> String

  /// Returns this string unchanged.
  toString -> String {
    self
  }

  /// Returns the number of UTF-8 storage bytes.
  size -> Int {
    _$byteCount
  }

  /// Returns whether this string contains no bytes.
  isEmpty -> Bool {
    _$byteCount == 0
  }

  // Existing search/Unicode/trim/etc. implementations remain real Phalcom.
}
```

The exact public native member list and exact types come from the canonical descriptor/source semantic census, not this shortened example.

---

# 42. Example End State: `Range`

This example demonstrates implementation namespace without pretending every internal method is native:

```phalcom
/// Immutable range bounds and forward integer iteration.
@native
class Range is Iterable<Int> {
  /// Native lower-bound observation.
  @internal
  @native
  _$lower -> Option<Int>

  /// Native upper-bound observation.
  @internal
  @native
  _$upper -> Option<Int>

  /// Whether the upper bound is inclusive.
  @internal
  @native
  _$upperInclusive -> Bool

  /// Normalizes the range against a finite sequence length.
  @internal
  _$sliceBounds(_ size: Int) -> Result<(Int, Int), SliceError> {
    // real source implementation
    ...
  }

  /// Advances the public iteration cursor protocol.
  iterate(_ previous: Option<Int>) -> Option<Int> {
    ...
  }

  iteratorValue(_ cursor: Int) -> Int {
    cursor
  }
}
```

Again, exact types must follow the current ratified semantics; the point is the annotation distinction.

---

# 43. Example End State: Reflection Placeholder

A file that is currently only:

```phalcom
@!documentation("First-class dispatch selector representation.")
```

must become a real source presentation, conceptually:

```phalcom
//! First-class dispatch selector representation.

/// Canonical first-class selector value.
@native
class Selector {
  // every owned native/source member, typed and documented
}

/// Structural selector pattern used by family/reflection operations.
@native
class SelectorPattern {
  // every owned native/source member, typed and documented
}
```

The exact members are filled from the generated/native/runtime census.

---

# 44. Final Completion Definition

This implementation is complete when **the universe source is sufficient to understand the universe**.

A developer should no longer need to inspect:

```text
universe/primitives.rs
NATIVE_MEMBERS
BuiltinInterfaceBuilder hard-coded overlays
VM bootstrap include lists
Rust primitive modules
```

just to discover that a core class or method exists.

Rust remains essential for implementation and machine metadata, but it is no longer the only place the language surface is visible.

The final architecture is:

```text
                    AUTHORITATIVE PRESENTATION
          phalcom-core/core/universe/src/**/*.ph
              classes + members + types + docs
                         │
                         │ verified by identity/signature
                         ▼
                   NATIVE IMPLEMENTATION
          #[primitive(...)] / generated surface
                         │
                         │ installs exactly once
                         ▼
                    LIVE RUNTIME
              primordial classes + methods
                         │
                         │ indexed by
                         ▼
               canonical semantic metadata
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
          reflection                LSP/docs
```

The source and Rust declarations intentionally remain independently authored where they describe overlapping facts, because that independence is what makes verification valuable. The runtime is not allowed to silently diverge from either.

The strongest acceptance statement is therefore:

> **Every canonical universe class and member is source-visible, correctly annotated, correctly typed, documented, mechanically matched to its native implementation when native, installed exactly once, and proven by bootstrap tests to preserve the runtime object model.**

That is the point at which Phalcom's universe stops being a partial source façade over hidden bootstrap machinery and becomes the actual canonical, navigable, semantically rich definition of the language core.
