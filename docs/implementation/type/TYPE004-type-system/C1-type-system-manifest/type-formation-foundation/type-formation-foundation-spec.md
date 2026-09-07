# Phalcom First Functional Typing System
## Repository-Grounded Detailed Implementation Specification

**Status:** Implementation specification / engineering handoff
**Repository:** `aureat/phalcom-lang`
**Repository investigation date:** 2026-08-22
**Repository snapshot investigated:** current `main` exposed by the connected GitHub repository during this investigation; blob results were rooted around commit `13e5cb20842d8f71a0c45947f0ad037f1c823a00`
**Primary milestone:** first functional source-level static typing layer integrated with compiler semantics and the LSP, while preserving Phalcom's dynamic runtime semantics
**Runtime type-contract enforcement:** explicitly deferred from this milestone
**Generic declarations / higher-kinded typing / full reflective type-object runtime:** architecturally prepared, not required for the first completion gate unless a later section explicitly says otherwise

---

# 0. How to read this specification

This document is intended to be handed directly to an engineering agent. It is deliberately implementation-specific.

The requirements use the following terms:

- **MUST**: required for the milestone to be considered correct.
- **MUST NOT**: prohibited because it would create semantic divergence, unsoundness, or architectural debt.
- **SHOULD**: strong recommendation; deviation requires a documented reason.
- **MAY**: optional implementation choice that does not change the required observable behavior.
- **Deferred**: deliberately outside the first functional milestone, but the implementation must not block it.

This specification distinguishes three categories of information:

1. **Current repository behavior** — verified in the repository.
2. **Previously proposed typing design** — design documents/examples that exist in the repository but are not current implementation.
3. **Normative implementation decisions in this document** — the architecture to implement now.

Do not treat `examples/phalcom-typing/` or `docs/spec/typing/` as evidence that the corresponding type system is already implemented. Those trees are design/prototype material. Current compiler, AST, LSP, module, and native-metadata code is the implementation baseline.

---

# 1. Milestone objective

Implement Phalcom's first functional static typing layer such that the following source is understood semantically:

```phalcom
const count: Int = 1

class User {
  _name: String

  name -> String {
    _name
  }

  rename(to value: String) -> String {
    _name = value
    value
  }
}
```

and a provable contradiction such as:

```phalcom
const count: String = 1
```

is reported consistently by the language semantic checker and by the LSP.

The milestone must establish the reusable architecture for later support of:

```phalcom
List<Int>
Map<String, User>
A | B
(Int, String) -> Bool

class Producer<+T> { ... }
class Consumer<-T> { ... }
```

and higher-kinded abstractions such as constructors of conceptual kind:

```text
Type -> Type
(Type -> Type) -> Type
```

but it must not implement those later features by adding ad-hoc rules to the first checker.

The runtime remains dynamically executing. Type information does not become part of ordinary selector identity, method lookup, overload resolution, object layout, allocation identity, or automatic production runtime validation.

---

# 2. Locked semantic principles

The implementation must preserve the following language principles.

## 2.1 One Phalcom semantic universe

Phalcom must not grow a TypeScript-like second language with unrelated semantics.

A class object that denotes a type in an annotation is still conceptually the same Phalcom class object. Future synthetic type descriptors are ordinary language-visible semantic objects. The compiler is permitted to represent these objects as canonical IDs, interned nodes, compact metadata, lazy descriptors, or other optimized forms until observation requires materialization.

The invariant is:

> Representation may differ by phase; language meaning must not.

Static analysis may use a restricted, staged evaluator for type-position expressions. That is an evaluation-phase restriction, not a second language.

The first milestone deliberately restricts source annotations to statically resolvable type references. It must not execute arbitrary user code during type checking.

## 2.2 Dynamic execution plus static semantic reasoning

Normal execution remains dynamic.

The compiler performs static consistency checking where it has sufficient evidence. A production run does not automatically insert type checks merely because an annotation exists.

Later typed/debug execution may instrument residual obligations. That is a different feature.

## 2.3 Developer declaration plus compiler evidence

An annotation expresses developer intent.

Inference expresses compiler-derived evidence.

If the compiler can prove that these conflict, it reports a contradiction. It does not silently replace the annotation with the inferred type and it does not silently replace inference with the annotation.

Example:

```phalcom
const value: String = 1
```

must conceptually produce:

```text
declared: String
actual/synthesized: Int
relation: Int is not assignable to String
result: refuted
```

## 2.4 Unknown is epistemic

`Unknown` is not a normal value type.

It means:

> The semantic engine has not established a type here.

Do not intern `Unknown` as a permanent `TypeId`.

## 2.5 Dynamic is deliberate static escape

`Dynamic` means:

> Static reasoning is intentionally not required at this boundary.

At runtime, the value still has an ordinary runtime class/type.

`Dynamic` is represented in type knowledge/checking state, not as a fake universal nominal class.

## 2.6 Never is a real bottom type

`Never` has no inhabitants and is a subtype of every value type:

```text
Never <: T
```

It is a canonical semantic type.

## 2.7 Any remains unresolved as a public language decision

The repository and previous blueprint mention `Any`, but the language discussion has not conclusively established whether a separate top type is necessary or whether an existing universal object type should fill that role.

Therefore:

- the first milestone MUST NOT require a user-facing `Any`;
- the internal relation API MUST be extensible to a true top type later;
- do not overload `Dynamic` as top;
- do not overload `Unknown` as top.

If `Any` is ratified later, it can become a canonical special type without changing checker architecture.

## 2.8 Unit is a real type, but final product equivalence is not a blocker

`Unit` already exists in the runtime universe catalog.

The intended language direction permits the singular unit value to be represented by zero-arity product spellings such as `()` and `#{}`. Current product lowering is not fully complete, so final product/unit equivalence is outside the first checker completion gate.

The type store must not encode an architecture that would make that equivalence impossible later.

## 2.9 Selector identity is type-independent

Types never alter selector identity in this milestone.

For:

```phalcom
foo(x: Int)
foo(x: String)
```

the type annotation does not create distinct selector identities. If both declarations otherwise encode the same selector, existing duplicate-selector rules still apply.

Typed dispatch is a separate future feature.

---

# 3. Repository baseline discovered during investigation

The implementation agent must begin from these facts rather than rediscovering them.

## 3.1 Workspace crates

Current workspace members include:

```text
phalcom-ast
phalcom-common
phalcom-type-syntax
phalcom-native-meta
phalcom-native-macros
phalcom-native-surface
phalcom-core
phalcom-repl
phalcom-lsp
phalcom-modules
```

A new VM-free semantic crate can be added without creating a dependency cycle.

Recommended dependency direction:

```text
phalcom-common
      ^
      |
phalcom-ast
      ^
      |
phalcom-modules ---- phalcom-native-meta
      \                 /
       \               /
        v             v
         phalcom-semantic
           ^        ^
           |        |
     phalcom-core  phalcom-lsp
```

`phalcom-lsp` must remain independent of `phalcom-core`.

## 3.2 `phalcom-modules` already owns canonical project/module identities

Do not invent a new canonical module identity in the type checker.

Reuse:

```rust
phalcom_modules::ModuleId
```

The module system also already defines:

```rust
pub struct DeclarationId {
    pub module: ModuleId,
    pub name: Box<str>,
}
```

and:

```rust
pub enum DeclarationKind {
    Class,
    Protocol,
    Adt,
    Alias,
}
```

It also has declaration-shell support for semantic strongly connected components.

This is the correct foundation for language-wide declaration identity.

## 3.3 The module system already has a semantic graph

`phalcom-modules/src/graph.rs` separates:

```text
reference dependencies
semantic/interface dependencies
runtime initialization dependencies
```

and already reserves semantic edge kinds including:

```rust
ModuleInterface
TypeReference
Superclass
ProtocolReference
ConstraintReference
CallbackSignature
AdtReference
```

Type resolution and type-interface dependency tracking must use this graph instead of creating a checker-only module graph.

## 3.4 The LSP already has mature semantic inference

`phalcom-lsp/src/semantic/facts.rs` explicitly describes `ValueShape` as:

> Advisory runtime value shape. This is deliberately not a language type.

That distinction must be preserved.

Existing shape forms include:

```text
Unknown
Instance(ClassId)
ClassObject(ClassId)
Module(ModuleId)
Tuple
ExactList
Record
List
Set
Map
Range
Callable
Selector
SelectorPattern
Family
Method
MethodFamily
BoundMethod
BoundMethodFamily
Union
```

The LSP also already has:

- `Confidence`;
- compact provenance;
- local binding facts;
- field facts;
- parameter facts;
- callable summaries;
- call dependency tracking;
- structured flow;
- bounded union widening;
- immutable published snapshots;
- incremental body-level invalidation;
- worker cancellation.

This machinery must be reused or extracted, not replaced.

## 3.5 The LSP already retains parsed ASTs

`phalcom-lsp/src/semantic/snapshot.rs` has:

```rust
pub struct FileSourceSnapshot {
    pub module: ModuleId,
    pub text: Arc<str>,
    pub program: Arc<phalcom_ast::ast::Program>,
    pub surface: ModuleSurface,
    pub scopes: ScopeGraph,
    pub callables: BTreeMap<CallableId, MemberAstRef>,
}
```

This is the correct architectural pattern for the compiler frontend too.

## 3.6 The project compiler currently reparses source

The project/module compiler parses source to construct interfaces, but the VM-facing bytecode compiler later parses the same source text again before AST-to-bytecode lowering.

Do not add a third parse for static typing.

The typing implementation must create a reusable parsed-source artifact so:

```text
parse once
  |
  +-> module interface
  +-> semantic checker
  +-> bytecode compiler
```

all consume the same `Arc<Program>`.

## 3.7 Native metadata is richer than the current LSP native surface

`phalcom-native-meta` already contains structured metadata:

```rust
TypeExprSpec
ParameterTupleSpec
CallableTypeSpec
PrimitiveSurfaceSpec
```

including:

- parameter types;
- return types;
- callable type parameters;
- raises;
- effects;
- return flow;
- universe keys.

The current LSP native surface mainly consumes `NativeReturnShape`, which is much less expressive.

The future canonical checker must normalize `phalcom-native-meta` into the shared type store. `NativeReturnShape` remains useful as a cheap advisory shape summary, not as the normative type contract.

## 3.8 `phalcom-type-syntax` is a native-metadata syntax parser

Its module documentation explicitly describes it as symbolic type/callable syntax for native metadata.

It has its own lexer and no source spans.

It must not become the authoritative source annotation AST merely because it already has `TypeExpr`.

It may remain:

```text
native metadata text
    -> phalcom-type-syntax
    -> phalcom-native-meta
    -> shared semantic normalization
```

while source annotations use the normal Phalcom lexer/parser.

## 3.9 Current typing design examples are not current implementation

The repository contains substantial future typing designs and example `.ph` files. They are useful requirements input, but they are not evidence of compiler support.

Do not make implementation decisions based on the assumption that `Type`, `Protocol`, `AppliedType`, generic signatures, or reflective type application already exist in the runtime.

## 3.10 Existing `CompileMode` has a different meaning

Current:

```rust
CompileMode::Debug
CompileMode::Release
CompileMode::Unchecked
```

controls `@requires`, `@ensures`, and `@invariant` weaving/metadata behavior.

Do not reuse or reinterpret these values as static typing modes.

The static type checker is a separate semantic concern.

---

# 4. Corrections to the previous first-typing blueprint

The previous blueprint is directionally correct but must be amended as follows.

## 4.1 Do not make `phalcom-semantic` own `ModuleId`

`ModuleId` already has a canonical owner: `phalcom-modules`.

`phalcom-semantic` should re-export or consume it.

## 4.2 Prefer `DeclarationId` over inventing another nominal class key

The canonical nominal type identity should ultimately be based on:

```rust
phalcom_modules::DeclarationId
```

not the LSP's URI-backed `ClassId`.

The LSP's current `ClassId` is an adapter-era identity and should be migrated gradually.

## 4.3 Do not use `phalcom-type-syntax::TypeExpr` directly in source AST

Source annotations need exact `SourceRange`s and normal Phalcom static-reference semantics.

Create source annotation nodes in `phalcom-ast`.

## 4.4 Do not model provenance inside `TypeId`

Two occurrences of `Int` inferred from different evidence must resolve to the same canonical `TypeId`.

Evidence is separate.

## 4.5 Do not model `Unknown` as `TypeNode`

Keep uncertainty in `TypeKnowledge`.

## 4.6 Do not require a full solver before useful typing works

The first checker can support nominal declarations, assignments, arguments, fields, and returns before generic inference exists.

Architect `InferVarId` separately now, but introduce active constraint solving only when required.

## 4.7 Do not insert the checker into the VM bytecode compiler

The checker belongs in a VM-independent semantic phase after resolution/linking and before bytecode emission.

## 4.8 Do not duplicate current control-flow analysis

The existing LSP flow model is already substantial.

The semantic extraction must preserve and share it.

---

# 5. First functional milestone scope

The first completion gate MUST include:

1. source annotation storage in the AST;
2. parser support for annotations on:
   - `let`/`const` simple-name bindings;
   - fields;
   - method parameters;
   - setter parameters;
   - index parameters;
   - method/getter/setter/index return types;
3. canonical `TypeStore`;
4. canonical nominal type identity based on module/declaration identity;
5. `Never`;
6. `Dynamic` as knowledge/checking state;
7. type-reference resolution across:
   - current module declarations;
   - selective imports;
   - whole-module aliases;
   - builtin universe types;
8. minimal nominal subtyping through class inheritance;
9. union representation required for native metadata and later flow joins;
10. annotation checking for:
    - initializers;
    - reassignment;
    - field writes;
    - call arguments when a declared signature is available;
    - explicit returns;
    - reachable implicit/tail returns;
11. source callable declared-signature representation;
12. structured diagnostics shared by compiler and LSP;
13. LSP display of declared type plus inferred/advisory information;
14. compiler integration before bytecode lowering;
15. parse-once program ingestion for project compilation;
16. incremental invalidation that recognizes annotation edits as declaration-surface edits;
17. normalization of usable native type contracts from `phalcom-native-meta`;
18. tests proving compiler/LSP consistency.

The milestone MUST NOT require:

- runtime type enforcement;
- runtime type descriptor materialization;
- generic class declaration syntax;
- higher-kinded type inference;
- variance checking;
- structural protocol conformance;
- type aliases;
- intersections;
- fully typed blocks/closures;
- typed pattern destructuring;
- exhaustive ADT typing;
- SMT/prover integration.

Those are explicitly deferred.

---

# 6. New crate: `phalcom-semantic`

Create:

```text
phalcom-semantic/
  Cargo.toml
  src/
    lib.rs
    identity.rs
    source.rs
    surface.rs
    scope.rs
    dispatch.rs
    shape/
      mod.rs
      facts.rs
      analyzer.rs
      flow.rs
      callable.rs
    types/
      mod.rs
      id.rs
      kind.rs
      store.rs
      annotation.rs
      evidence.rs
      relation.rs
      native.rs
    checker/
      mod.rs
      context.rs
      expression.rs
      statement.rs
      declaration.rs
      call.rs
      result.rs
    diagnostic.rs
    snapshot.rs
    invalidation.rs
```

The exact physical split may vary, but responsibilities must not collapse into one monolithic module.

## 6.1 Cargo dependencies

Recommended:

```toml
[dependencies]
phalcom-ast = { path = "../phalcom-ast" }
phalcom-common = { path = "../phalcom-common" }
phalcom-modules = { path = "../phalcom-modules" }
phalcom-native-meta = { path = "../phalcom-native-meta" }
phalcom-native-surface = { path = "../phalcom-native-surface" } # only if needed for transition
thiserror = { workspace = true }
serde = { workspace = true } # only if diagnostics/snapshots require it
```

The crate MUST NOT depend on:

```text
phalcom-core
tower-lsp
tokio
VM heap types
ObjRef
ClassKey
```

## 6.2 Consumers

Add:

```toml
phalcom-semantic = { path = "../phalcom-semantic" }
```

to:

```text
phalcom-core
phalcom-lsp
```

Eventually remove duplicated LSP semantic models as extraction progresses.

---

# 7. Parse-once source artifact

## 7.1 Required artifact

Introduce a VM-free parsed source representation.

Recommended shape:

```rust
#[derive(Clone)]
pub struct ParsedSourceUnit {
    pub module: phalcom_modules::ModuleId,
    pub kind: phalcom_modules::ModuleKind,
    pub source: Option<phalcom_modules::SourceLocation>,
    pub text: Arc<str>,
    pub program: Arc<phalcom_ast::ast::Program>,
}
```

A richer implementation may additionally retain parser diagnostics.

## 7.2 `ModuleResolver` changes

Current `load_interface()` parses internally and caches only the interface.

Refactor toward:

```rust
pub fn load_parsed_source(
    &mut self,
    module: &ModuleId
) -> Result<Arc<ParsedSourceUnit>, ModuleLoadError>
```

and make:

```rust
pub fn load_interface(...)
```

derive from/cached alongside the parsed source.

Do not parse the source twice merely because the public interface API remains.

## 7.3 `CompiledModule` changes

Add a parsed program reference:

```rust
pub struct CompiledModule {
    ...
    pub source_text: Option<Arc<str>>,
    pub program: Option<Arc<Program>>,
    ...
}
```

If every source-backed module always has a program, prefer a stronger representation that makes the invariant explicit.

## 7.4 VM compiler delegation

Introduce an AST-based entry:

```rust
pub fn compile_program_as_with_bindings(
    &mut self,
    module: ObjRef,
    source_text: Arc<str>,
    program: Arc<Program>,
    kind: UnitKind,
    bindings: Option<CompileBindings>,
) -> PhResult<ObjRef>
```

or equivalent.

Then legacy source entry:

```rust
compile_closure_as_with_bindings(source, ...)
```

parses once and delegates.

Project/module compilation uses the already-retained AST.

## 7.5 Verification

Add instrumentation tests proving that project execution does not parse each module again during bytecode lowering.

The milestone must not regress REPL behavior.

---

# 8. Canonical semantic identities

## 8.1 Module identity

Canonical:

```rust
pub use phalcom_modules::ModuleId;
```

Do not introduce another semantic `ModuleId`.

## 8.2 Declaration identity

Canonical nominal declarations use:

```rust
pub use phalcom_modules::DeclarationId;
```

## 8.3 Dispatch side

Move or define VM-free dispatch side in the shared semantic crate:

```rust
pub enum DispatchSide {
    Instance,
    Class,
}
```

If another VM-free crate already has an equivalent, use one canonical definition and adapters.

## 8.4 Callable identity

Define:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallableId {
    pub owner: DeclarationId,
    pub selector: phalcom_common::selector::Selector,
    pub side: DispatchSide,
}
```

If storing the selector string is materially cheaper for existing maps, a canonical encoded selector may remain, but all construction must pass through the structural selector API.

Do not include type annotations in callable identity.

## 8.5 Field identity

Define:

```rust
pub struct FieldId {
    pub owner: DeclarationId,
    pub name: Box<str>,
    pub side: DispatchSide,
}
```

## 8.6 Binding identity

Current LSP `BindingId(u32)` is snapshot-local.

That is acceptable.

The shared checker may use:

```rust
pub struct BindingId(u32);
```

within a `SourceSnapshot`.

It MUST NOT assume that the numeric ID survives reparsing.

Cross-generation caches use declaration/module identity plus source generation/fingerprint, not raw `BindingId`.

## 8.7 LSP migration

`phalcom-lsp/src/semantic/ids.rs` currently contains a legacy URI-backed `ModuleId(String)` and a newer semantic document mapping.

Migration order:

1. keep `DocumentModuleMap` as an editor boundary;
2. store `phalcom_modules::ModuleId` inside shared semantic state;
3. adapt URI -> semantic module before querying;
4. replace LSP `ClassId` with shared declaration identity;
5. replace LSP callable/field IDs;
6. remove legacy URI-backed module IDs once no internal semantic table uses them.

Do not do a flag-day rewrite unless tests prove it is lower risk.

---

# 9. Source annotation AST

## 9.1 New source type nodes

Add to `phalcom-ast/src/ast.rs`.

Recommended:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct TypeAnnotation {
    pub expr: TypeAnnotationExpr,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeAnnotationExpr {
    Reference(StaticSymbolRef),

    // Reserved structural forms. Parser enablement is phase-gated.
    Application {
        origin: Box<TypeAnnotation>,
        arguments: Vec<TypeAnnotation>,
        range: SourceRange,
    },

    Union {
        members: Vec<TypeAnnotation>,
        range: SourceRange,
    },

    Tuple {
        elements: Vec<TypeTupleElement>,
        range: SourceRange,
    },

    Callable {
        parameters: Vec<TypeCallableParameter>,
        result: Box<TypeAnnotation>,
        range: SourceRange,
    },
}
```

For the first functional parser, `Reference` is the required source form.

The extra forms may be omitted from the first AST patch if the team prefers zero dead variants. If omitted, reserve their design in documentation and add them only when their parser semantics land.

## 9.2 Special types

Do not add lexer keywords for:

```text
Never
Dynamic
Any
Unit
```

in the first implementation.

Treat them as normal names in annotation context and resolve semantically.

Reasons:

- `Unit` is already a real class name.
- `Never` and `Dynamic` are type-context names.
- adding keywords would change ordinary expression grammar unnecessarily.
- future reflective type objects may bind these names normally.

## 9.3 `ParameterDef`

Current shape lacks annotations.

Add:

```rust
pub struct ParameterDef {
    pub name: String,
    pub name_range: SourceRange,
    pub label: Option<String>,
    pub label_range: Option<SourceRange>,
    pub rest_mode: RestMode,
    pub annotation: Option<TypeAnnotation>,
    pub range: SourceRange,
}
```

`range` must cover the complete parameter including annotation.

`name_range` remains only the local binding token.

## 9.4 `FieldDef`

Add:

```rust
pub annotation: Option<TypeAnnotation>,
```

The field `range` must include annotation and initializer.

## 9.5 `LetBinding`

Add:

```rust
pub annotation: Option<TypeAnnotation>,
```

This attaches to the binding pattern as a whole.

First checker support is restricted to:

```rust
Pattern::Name
```

See §13.6.

## 9.6 `MethodDef`

Add:

```rust
pub return_annotation: Option<TypeAnnotation>,
```

## 9.7 `GetterDef`

Add:

```rust
pub return_annotation: Option<TypeAnnotation>,
```

## 9.8 `SetterDef`

Add:

```rust
pub return_annotation: Option<TypeAnnotation>,
```

Even if the first checker derives setter behavior specially, preserve a uniform callable AST.

## 9.9 `IndexMethodDef`

Add:

```rust
pub return_annotation: Option<TypeAnnotation>,
```

## 9.10 Closure parameters

Do not modify `ClosureParameter` in the first milestone.

Typed closure parameters are deferred.

This must be documented in diagnostics/tooling so users are not led to expect:

```phalcom
|x: Int| { ... }
```

yet.

## 9.11 Struct literal migration requirement

After changing AST structs, run a repository-wide sweep for every Rust struct literal of:

```text
ParameterDef
FieldDef
MethodDef
GetterDef
SetterDef
IndexMethodDef
LetBinding
```

Production synthesis sites include parser code, compiler attribute expansion, class lowering helpers, LSP selector/surface helpers, and tests.

Every synthesized node must deliberately set:

```rust
annotation: None
return_annotation: None
```

or propagate an annotation when semantically required.

Never rely on `Default` to conceal a forgotten propagation path.

`cargo check --workspace` is a required gate after this migration.

---

# 10. Annotation grammar

## 10.1 First functional grammar

The first semantic milestone accepts type references:

```text
TYPE_REF :=
    IDENTIFIER
    ( "." IDENTIFIER )*
```

Examples:

```phalcom
Int
String
User
accounts.User
universe.Int
Never
Dynamic
```

The parser uses the ordinary Phalcom token stream.

## 10.2 Binding declarations

Required:

```phalcom
const value: Int = 1
let value: Int = expression
```

Grammar:

```text
binding-decl :=
    ("let" | "const")
    pattern
    (":" type-ref)?
    ("=" expr)?
```

Annotation parse point: immediately after `parse_pattern()` and before `=`.

## 10.3 Fields

Required:

```phalcom
_name: String
_name: String = "default"
const _id: Int
const _id: Int = 1
```

Grammar:

```text
field-decl :=
    ("const")?
    FIELD_IDENTIFIER
    (":" type-ref)?
    ("=" expr)?
```

Annotation parse point: after field identifier, before `=`.

## 10.4 Parameters

Required forms:

```phalcom
run(value: Int)
run(_ value: Int)
run(label value: Int)
run(*values: Int)
run(**options: Map)
run(***all: Object)
```

The current selector grammar has an important ambiguity that MUST be handled deliberately.

Today `parse_selector_params()` rejects a colon immediately following the first identifier because legacy external-label syntax used colons.

With typing, this source:

```phalcom
run(value: Int)
```

must mean:

```text
external label: value
local name: value
annotation: Int
```

Therefore remove/replace the old unconditional colon error.

Exact cases:

### Same external/local labeled parameter

Input:

```phalcom
value: Int
```

Result:

```rust
name = "value"
label = Some("value")
annotation = Some(Int)
```

### Explicit external/local split

Input:

```phalcom
to value: String
```

Result:

```rust
name = "value"
label = Some("to")
annotation = Some(String)
```

### Positional

Input:

```phalcom
_ value: Int
```

Result:

```rust
name = "value"
label = None
annotation = Some(Int)
```

### Rest

Parse annotation after the rest binding name.

The annotation describes the semantic parameter binding according to future rest policy. For the first milestone, the checker may conservatively treat the rest container as unknown unless a precise rest-container type rule is implemented.

## 10.5 Setter parameter

Required:

```phalcom
name=(put value: String) {
    ...
}
```

Update the setter-specific parser path that currently constructs `ParameterDef` manually.

## 10.6 Index setter parameter

Required:

```phalcom
[_ key]=(put value: String) {
    ...
}
```

Update the index-setter-specific manual `ParameterDef` construction.

## 10.7 Return types

Required:

```phalcom
run() -> Int {
    ...
}

name -> String {
    ...
}

[_ index] -> Object {
    ...
}

name=(put value: String) -> String {
    ...
}
```

Parse point:

```text
after the complete selector/parameter declaration
before parse_method_block()
```

## 10.8 Constructors

The parser may preserve an explicit return annotation, but the first checker MUST diagnose it on constructor declarations unless the language decision is explicitly changed.

Recommended diagnostic:

```text
type.constructor.explicit_return
```

Rationale:

- constructor/factory result is semantically determined by constructor behavior;
- current compiler has special constructor lowering;
- explicit constructor return contracts can be designed later with `Self`.

Do not silently ignore a written annotation.

## 10.9 Destructuring binding annotation

The parser may accept:

```phalcom
let (a, b): Pair = value
```

into `LetBinding.annotation`, but the first checker MUST reject it with a precise unsupported-feature diagnostic.

Recommended:

```text
type.annotation.destructuring_not_supported
```

Do not guess that the annotation applies independently to every bound name.

## 10.10 Variant declaration labels

Current variant syntax uses:

```phalcom
@variant Name(foo:, bar:)
```

Those colons are variant-field label grammar, not type annotations.

Do not change variant parsing in this milestone.

## 10.11 Type application / union / callable syntax

Reserve, but do not claim first-milestone support unless implemented end-to-end:

```phalcom
List<Int>
Int | String
(Int, String) -> Bool
```

Do not accept syntax into the normal language and then silently erase its semantics.

A preparatory parser implementation is acceptable only if unsupported forms produce a stable, explicit semantic diagnostic and tests lock that behavior.

---

# 11. Source semantic surfaces

Source surfaces must retain annotations independent of the canonical type store.

## 11.1 `ParamSurface`

Extend:

```rust
pub struct ParamSurface {
    ...
    pub annotation: Option<TypeAnnotation>,
}
```

If cloning full annotation trees becomes expensive, store an AST reference/index instead. The requirement is that source syntax and range remain recoverable.

## 11.2 `FieldSurface`

Add annotation reference.

## 11.3 `MemberSurface`

Add:

```rust
pub return_annotation: Option<TypeAnnotation>,
```

Keep:

```rust
native_return
```

temporarily for advisory shape compatibility.

Do not treat `native_return` as the canonical type contract.

## 11.4 Surface vs resolved signature

Source surface stores syntax.

Resolved type metadata lives in a semantic layer:

```rust
pub struct ResolvedCallableSignature {
    pub callable: CallableId,
    pub parameters: Box<[ResolvedParameterType]>,
    pub result: TypeKnowledge,
    pub declaration: SignatureSource,
}
```

Do not store generation-local `TypeId`s permanently inside the AST.

---

# 12. Canonical type store

## 12.1 IDs

Define compact IDs:

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeId(u32);

#[repr(transparent)]
pub struct KindId(u32);

#[repr(transparent)]
pub struct TypeParameterId(u32);

#[repr(transparent)]
pub struct InferVarId(u32);
```

`InferVarId` MUST remain distinct from `TypeId`.

## 12.2 Kind-aware architecture

Even though first source generics are deferred, the store must be able to distinguish a fully formed value type from a type constructor.

Minimum kind model:

```rust
pub enum KindData {
    Type,
    Arrow {
        parameters: Box<[KindId]>,
        result: KindId,
    },
}
```

Canonical builtins:

```text
TypeKind = Kind::Type
```

Future examples:

```text
Int          : Type
List         : Type -> Type
Map          : (Type, Type) -> Type
Functor      : (Type -> Type) -> Type
```

Do not implement monads as a compiler special case. The kind/type parameter system must eventually be expressive enough to describe them.

## 12.3 Type data

Recommended long-term shape:

```rust
pub enum TypeData {
    Never,
    Unit,

    Nominal {
        declaration: DeclarationId,
    },

    Applied {
        origin: TypeId,
        arguments: Box<[TypeId]>,
    },

    Union(Box<[TypeId]>),

    Tuple(Box<[TupleTypeElement]>),

    Record(Box<[RecordTypeField]>),

    Callable(CallableType),

    Parameter(TypeParameterId),
}
```

The first checker actively requires:

```text
Never
Unit
Nominal
Union
```

`Applied`, `Tuple`, `Record`, `Callable`, and `Parameter` are implemented when needed by native normalization or subsequent phases.

## 12.4 Do not include

Do not add:

```rust
TypeData::Unknown
TypeData::Dynamic
```

`Dynamic` and unknown knowledge are outside the canonical value-type lattice.

## 12.5 Interning

`TypeStore` must intern structurally equivalent canonical nodes.

Required invariant:

```text
intern(Int) == intern(Int)
```

and:

```text
intern(Int | String) == intern(String | Int)
```

after normalization.

Use a hash-consing map:

```rust
HashMap<TypeData, TypeId>
```

or equivalent.

## 12.6 Stable semantic equality

`TypeId` equality within a store means canonical semantic equality.

Do not expose object allocation identity as type equality.

Future runtime descriptor object identity may be interned, but semantic equality is the primary rule.

## 12.7 Union normalization

When interning a union:

1. flatten nested unions;
2. remove duplicates;
3. remove `Never` when another member exists;
4. sort members by canonical deterministic key;
5. if zero members remain, return `Never`;
6. if one member remains, return that member directly.

Do not apply unsound class-hierarchy simplifications in the first implementation unless relation tests cover them.

---

# 13. Type knowledge and evidence

## 13.1 Core representation

Recommended:

```rust
pub enum TypeKnowledge {
    Known(TypeEvidence),
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}
```

## 13.2 Evidence

```rust
pub struct TypeEvidence {
    pub ty: TypeId,
    pub authority: EvidenceAuthority,
    pub provenance: EvidenceSet,
}
```

## 13.3 Authority

Do not reuse current `Confidence` as a soundness flag.

Introduce:

```rust
pub enum EvidenceAuthority {
    Declared,
    Proven,
    ExactSyntax,
    TrustedNative,
    Advisory,
}
```

Meaning:

- `Declared`: developer type declaration.
- `Proven`: result established by checker rules.
- `ExactSyntax`: literal/class construction fact known directly.
- `TrustedNative`: normalized native contract under trusted metadata.
- `Advisory`: useful LSP shape inference, not sufficient by itself for a hard static contradiction unless paired with a sound derivation.

## 13.4 Provenance

Keep provenance separate from `TypeId`.

Possible sources:

```rust
pub enum TypeOrigin {
    Annotation(SourceRange),
    Literal(SourceRange),
    Binding(SourceRange),
    Return(SourceRange),
    Call {
        site: SourceRange,
        target: CallableId,
    },
    FieldWrite(SourceRange),
    Native(PrimitiveKey),
    Refinement(SourceRange),
}
```

Cap UI-oriented provenance if necessary, but checker diagnostics must retain enough cause edges to explain errors.

## 13.5 Unknown reasons

Use structured reasons:

```rust
pub enum UnknownReason {
    NoAnnotation,
    DynamicDispatch,
    UnresolvedCallableReturn,
    AdvisoryWidening,
    DynamicPack,
    GenericInferenceDeferred,
    UnsupportedExpression,
    NativeMetadataMissing,
    IncompleteProject,
}
```

## 13.6 Dynamic reasons

```rust
pub enum DynamicReason {
    ExplicitAnnotation,
    DynamicBoundary,
    ExternalInterop,
}
```

---

# 14. Type annotation resolution

## 14.1 Resolver input

Input:

```rust
TypeAnnotation
ResolutionContext
```

Output:

```rust
Result<TypeKnowledge, SemanticDiagnostic>
```

A valid ordinary annotation should normally yield `Known`.

## 14.2 Context

```rust
pub struct TypeResolutionContext<'a> {
    pub module: &'a ModuleId,
    pub declaration: Option<&'a DeclarationId>,
    pub semantic_graph: &'a SemanticGraph,
    pub interfaces: &'a InterfaceUniverse,
    pub store: &'a mut TypeStore,
    pub type_parameters: &'a TypeParameterScope,
}
```

## 14.3 Namespace

The first type namespace contains:

- class declarations;
- future protocols;
- future ADTs;
- future aliases;
- builtin universe declarations;
- type parameters when generic declarations land;
- special type names such as `Never` and `Dynamic`.

Ordinary local value bindings do not become statically evaluable type expressions in v1.

## 14.4 Local declaration resolution

For:

```phalcom
class User { ... }
const user: User = ...
```

resolve to:

```rust
TypeData::Nominal {
    declaration: DeclarationId(module, "User")
}
```

## 14.5 Selective imports

For:

```phalcom
from accounts import User
const user: User = ...
```

resolve through the linked import/export identity to the target declaration.

Do not create a nominal type for the local alias spelling itself.

## 14.6 Module-qualified imports

For:

```phalcom
import accounts
const user: accounts.User = ...
```

resolve the module alias and exported declaration through the module linker.

## 14.7 Builtin universe types

Use `UniverseKey` / builtin module identity as the bridge.

Do not encode builtin types by unqualified strings inside the canonical store.

## 14.8 `Never`

If the resolved type-context spelling is the canonical `Never` special name, return the canonical bottom `TypeId`.

## 14.9 `Dynamic`

Return:

```rust
TypeKnowledge::Dynamic(DynamicReason::ExplicitAnnotation)
```

Do not intern a `Dynamic` TypeId.

## 14.10 Missing annotation

Missing annotation remains absence:

```rust
None
```

It is not rewritten to `Dynamic`, `Object`, or `Unknown` in source metadata.

The checker may derive `Unknown` knowledge from absence.

## 14.11 Unresolved annotation

This is a hard semantic error because the declaration itself is malformed:

```text
type.annotation.unresolved_name
```

Example:

```phalcom
const value: DoesNotExist = ...
```

The program must not silently continue by treating that annotation as `Dynamic`.

---

# 15. Native type normalization

## 15.1 Canonical source

Use:

```text
phalcom-native-meta
```

for normative native type metadata.

## 15.2 Normalizer

Implement:

```rust
pub struct NativeTypeNormalizer<'a> {
    store: &'a mut TypeStore,
    universe: &'a UniverseDeclarationMap,
}
```

with:

```rust
fn normalize_type(
    &mut self,
    spec: &TypeExprSpec,
    ctx: &NativeTypeContext
) -> TypeKnowledge
```

## 15.3 Rules

### `TypeExprSpec::Never`

-> canonical `Never`.

### `Universe(UniverseKey)`

-> canonical nominal declaration for that universe key.

### `SelfType`

-> receiver/owner type from context.

### `Union`

-> normalized canonical union.

### `Unknown`

-> `TypeKnowledge::Unknown(NativeMetadataMissing)`.

Never intern it as a type.

### `Parameter`

If the callable generic binder is not implemented in the current stage:

-> `Unknown(GenericInferenceDeferred)`.

When generic native call inference lands, bind to local `InferVarId`/type parameter identity.

### `Applied`

Normalize only when origin and arguments can be represented safely.

Before variance/generic declaration semantics land, same-origin applied types are invariant for relation purposes.

## 15.4 Return flow

`ReturnFlowSpec` is separate evidence.

Examples:

```text
Receiver
Argument(i)
Never
```

may provide stronger flow knowledge than an absent type expression.

`Never` flow means the call has no normal return path. Integrate this into control flow when safe.

## 15.5 `NativeReturnShape`

Keep current `phalcom-native-surface::NativeReturnShape` for migration and cheap advisory inference.

Long term it should be generated or derived from richer metadata where possible to prevent drift.

---

# 16. Type relations

Create separate APIs.

Do not implement one `compatible()` function and reuse it for all semantics.

Required public/internal relations:

```rust
fn equivalent(a: TypeId, b: TypeId, ctx: &RelationContext) -> bool;

fn is_subtype(
    sub: TypeId,
    sup: TypeId,
    ctx: &RelationContext
) -> RelationResult;

fn is_assignable(
    actual: &TypeKnowledge,
    expected: &TypeKnowledge,
    ctx: &RelationContext
) -> CheckResult;

fn is_consistent(
    a: &TypeKnowledge,
    b: &TypeKnowledge,
    ctx: &RelationContext
) -> CheckResult;
```

## 16.1 Result type

```rust
pub enum CheckResult {
    Proven,
    Refuted(RelationFailure),
    Unknown(RelationUnknown),
    Dynamic,
}
```

## 16.2 Base laws

Required:

```text
T <: T
Never <: T
```

Nominal subclass:

```text
Child <: Parent
```

when the canonical declaration hierarchy proves it.

## 16.3 Dynamic

Assignability involving explicit Dynamic yields:

```text
Dynamic
```

not `Proven`.

This distinction matters for future runtime obligation generation.

## 16.4 Unknown

Unknown evidence yields:

```text
Unknown
```

unless the relation can be decided independently.

Never report `Unknown` as a contradiction in ordinary compile mode.

## 16.5 Union

Required safe rules:

```text
A <: (B | C)
    iff A <: B OR A <: C

(A | B) <: C
    iff A <: C AND B <: C
```

If any necessary relation is unknown, return unknown unless another branch proves/refutes the composite conclusively.

## 16.6 Nominal inheritance source

Do not ask the VM heap.

Use the linked semantic declaration/superclass graph.

## 16.7 Applied types

Until variance is ratified/implemented:

```text
Origin<A> <: Origin<B>
```

only when arguments are semantically equivalent.

This is invariant and safe.

Later variance metadata changes this relation without changing API shape.

## 16.8 Object/top relationship

If all ordinary Phalcom runtime values are semantically instances beneath `Object`, nominal ancestry may naturally make:

```text
T <: Object
```

for nominal classes.

Do not use that fact to decide the unresolved `Any` question.

---

# 17. Checker architecture

## 17.1 Bidirectional entry points

Implement:

```rust
pub fn synthesize(
    expr: &Expr,
    ctx: &mut CheckContext
) -> TypeKnowledge;

pub fn check(
    expr: &Expr,
    expected: &TypeKnowledge,
    ctx: &mut CheckContext
) -> CheckOutcome;
```

## 17.2 Why bidirectional

Expected types will later guide:

- empty collection literals;
- callable/block types;
- generic arguments;
- overload-like constraints if ever specified.

Even in v1, it gives clean assignment/return/argument checking.

## 17.3 Checker context

Recommended:

```rust
pub struct CheckContext<'a> {
    pub module: &'a ModuleId,
    pub current_class: Option<&'a DeclarationId>,
    pub current_callable: Option<&'a CallableId>,
    pub dispatch_side: Option<DispatchSide>,

    pub types: &'a mut TypeStore,
    pub relations: RelationContext<'a>,
    pub scopes: &'a ScopeGraph,
    pub surfaces: &'a SemanticSurfaceUniverse,
    pub dispatch: &'a DispatchResolver,

    pub flow: &'a mut TypedFlowState,
    pub diagnostics: &'a mut Vec<SemanticDiagnostic>,
}
```

## 17.4 Checker output

Do not only return a type.

Capture:

```rust
pub struct CheckOutcome {
    pub knowledge: TypeKnowledge,
    pub obligations: Vec<TypeObligation>,
}
```

The first milestone may immediately discharge obligations instead of retaining a large graph, but the API must not prevent future residual runtime checks.

---

# 18. Sound bridge from `ValueShape`

The checker may reuse advisory shape analysis, but only through an explicit soundness gate.

Implement:

```rust
fn shape_to_synthesized_type(
    value: &InferredValue,
    context: &ShapeTypeBridgeContext
) -> Option<TypeEvidence>
```

## 18.1 Safe initial mappings

Safe when provenance/derivation is trusted:

```text
Instance(Int)     -> nominal Int
Instance(Float)   -> nominal Float
Instance(String)  -> nominal String
Instance(Bool)    -> nominal Bool
Instance(C)       -> nominal C
```

for direct literal/self/constructor facts.

## 18.2 Not automatically sound

Do NOT turn these into hard contracts merely because `ValueShape` has them:

- parameter shapes inferred from observed call sites;
- field shapes inferred only from observed writes;
- heuristic use-site constraints;
- a missing dynamic send;
- a widened union;
- an interprocedural shape whose source contains dynamic behavior.

Such facts remain advisory unless the normative checker independently proves them.

## 18.3 Confidence is not proof

Current:

```text
Exact
Flow
Interprocedural
Heuristic
```

is useful presentation metadata.

Do not equate `Confidence::Exact` with a formal proof result globally.

---

# 19. Expression synthesis table

Every current `Expr` variant must have a deliberate checker policy.

No wildcard `_ => Unknown` may hide a newly added AST variant. Match exhaustively.

## 19.1 Integer literal

```rust
Expr::Int
```

synthesizes canonical universe `Int`.

Authority: `ExactSyntax`.

## 19.2 Float literal

-> `Float`.

## 19.3 String literal

-> `String`.

## 19.4 Boolean literal

-> `Bool`.

Known true/false value may remain in advisory flow; type checker only needs `Bool` initially.

## 19.5 Symbol

A normal symbol literal -> `Symbol`.

Exact selector/selector-pattern runtime classes may be resolved if the universe has distinct types; otherwise use the appropriate canonical nominal class already represented by runtime semantics.

## 19.6 Variable

Resolution order uses shared `ScopeGraph`.

If binding has a declared type:

-> return that type for reads, with flow refinement if valid.

If no declared type:

-> use proven current flow type if available.

If only advisory fact exists:

-> return advisory/unknown according to soundness gate.

## 19.7 Field read

If field has a declared annotation:

-> declared type, possibly refined only under sound rules.

If unannotated:

-> proven field type if checker can establish one; otherwise advisory/unknown.

Do not use arbitrary observed field writes as a permanent declaration contract.

## 19.8 `self`

Instance-side:

-> current class nominal type.

Class-side:

the first checker may return Unknown for the precise class-object type unless a canonical metatype model is available.

Do not pretend `Class<C>` semantics exist if they are not specified.

## 19.9 `super`

`super` is not a normal value outside send syntax.

For receiver checking, use current instance type plus a dispatch-start override.

Do not assign it an independent user-visible type.

## 19.10 Assignment

Synthesize RHS.

Check RHS against declared target type if one exists.

Assignment expression result follows current runtime semantics: use RHS type if assignment returns the written value, consistent with existing advisory analyzer behavior.

## 19.11 Range

At minimum synthesize nominal `Range`.

Typed element/bound application is deferred unless canonical applied types are active.

## 19.12 Unary operations

For `not`:

- operand should be statically Bool when provable;
- result is Bool.

For other unary operations:

- resolve the normal selector through shared dispatch;
- if target has a declared return type, use it;
- otherwise use trusted native return metadata;
- otherwise unknown/advisory.

Do not hardcode arithmetic type tables when dispatch already owns semantics.

## 19.13 Binary operations

Same principle: binary operators are message sends.

Special cases already guaranteed by language semantics may synthesize direct types, for example exact sameness comparison returning Bool.

Otherwise:

1. resolve normal operator selector;
2. include reflected operator path when language semantics use it;
3. synthesize declared/trusted return type;
4. join only sound alternatives;
5. if dynamic resolution remains possible, return Unknown rather than inventing a narrow type.

## 19.14 Comparison chain

Each comparison operand/send is checked.

Result type is Bool when the chain expression is semantically Boolean.

## 19.15 `IfLet`

Use existing pattern/condition flow machinery.

The expression result is the join of reachable branch result types if the AST represents it as an expression.

If branch value typing is not yet stable, return Unknown but still type-check statements inside branches.

## 19.16 `WhileLet`

Check iterable/pattern/refinement semantics; normal expression result may remain Unit/Unknown according to actual language runtime semantics.

Do not guess. Match current compiler/runtime value behavior.

## 19.17 Ellipsis

Resolve to existing `Ellipsis` universe class if present.

## 19.18 Unqualified call

Resolve existing lexical/global/implicit-self call semantics.

If a target callable is statically resolved:

- validate arguments against its declared signature;
- synthesize declared result.

If dispatch is dynamic:

- evaluate/check arguments for internal contradictions;
- return Unknown.

## 19.19 Method call

Algorithm:

1. synthesize receiver;
2. compute canonical selector from actual argument labels/shape;
3. if receiver nominal type is known, resolve dispatch;
4. validate each static argument against resolved parameter annotation when available;
5. synthesize result from declared return type;
6. otherwise use trusted native contract;
7. otherwise unknown/advisory.

Dynamic packs disable complete static argument checking for affected slots.

Do not report missing selector unless receiver knowledge is authoritative enough to prove the miss.

## 19.20 Implementation selector

Treat according to existing privileged/internal rules.

No new type semantics.

## 19.21 Property get

Property access is a getter send.

Use normal dispatch/signature logic.

## 19.22 Property set

Setter send.

Check written value against setter parameter type and/or declared field semantics where resolvable.

Result follows actual setter expression semantics.

## 19.23 Index get

Resolve normal subscript selector.

Check parameters.

Use declared/trusted return.

## 19.24 Index set

Check index arguments and `put` value.

Result follows existing runtime semantics; current advisory analyzer returns assigned value shape, so preserve that unless runtime specification says otherwise.

## 19.25 Block

First milestone:

- analyze body for internal type errors;
- return Unknown as a first-class block type unless callable/block type representation is implemented.

Do not infer a fake `Function` nominal contract that loses parameter/result structure and then use it for sound higher-order checking.

## 19.26 Method reference

May synthesize nominal runtime method/family class for simple reflection.

Full callable signature typing is deferred.

## 19.27 Tuple literal

If tuple type support is enabled in TypeStore:

-> canonical tuple of synthesized element types when every element is known.

Otherwise:

-> nominal Tuple plus advisory structural shape.

Do not lose exact advisory tuple information from LSP.

## 19.28 Record literal

Same policy.

Dynamic expansion may force Unknown structural type while retaining nominal Record.

## 19.29 Map literal

At minimum -> nominal Map.

If applied generic types are active, join key/value synthesized types.

Do not let an empty map force a bogus element type.

## 19.30 Set literal

At minimum -> nominal Set.

## 19.31 List literal

At minimum -> nominal List.

Current LSP exact list/element shapes remain available for editor hints.

Generic `List<T>` synthesis is enabled only after applied type semantics are active.

## 19.32 Membership

Result -> Bool when language semantics guarantee Boolean membership result.

Check underlying send if necessary.

## 19.33 `is` membership / trusted type tests

Result -> Bool.

Also produce true/false branch refinements where target is a statically resolved class/type and predicate semantics are trusted.

---

# 20. Statement checking

## 20.1 `let` / `const`

For:

```phalcom
const x: T = expr
```

1. resolve `T`;
2. synthesize/check `expr`;
3. compare actual to expected;
4. on refutation, emit `type.binding.initializer_mismatch`;
5. bind declared `T` into typed flow state;
6. retain actual evidence for diagnostics.

For an unannotated binding:

- infer current flow type when sound;
- do not create a permanent public contract.

## 20.2 Uninitialized `let`

Current runtime semantics make uninitialized `let` read the surface absence value (`None`).

Therefore:

```phalcom
let x: Int
```

must not be treated as delayed definite initialization unless language semantics are changed elsewhere.

First milestone options, in order of preference:

1. if canonical `None` type assignability is implemented, check `None` against annotation;
2. otherwise reject annotated uninitialized lets with a stable diagnostic.

Recommended diagnostic:

```text
type.binding.uninitialized_annotation
```

Do not silently pretend the initial value is `Int`.

## 20.3 `const` without initializer

Existing compiler/parser semantics already reject it.

Do not duplicate a type-system-specific diagnostic.

## 20.4 Expression statement

Type-check expression for nested contradictions.

Tail value contributes to callable result inference under current language semantics.

## 20.5 Return

If callable has explicit return annotation:

check returned expression against it.

Bare `return` uses the actual language absence/unit semantics. Do not coerce it to an arbitrary expected type.

## 20.6 Throw

Type-check thrown expression as far as existing Error semantics permit.

A `throw` terminates the normal flow path.

Future `Never` flow should model this.

## 20.7 Break / continue

No type result; preserve existing loop control.

## 20.8 For

Reuse existing iterable element-shape analysis.

If precise generic iterable type semantics are unavailable, pattern bindings may remain Unknown/advisory.

Still check the body and all annotated assignments.

## 20.9 Class

Class declarations are checked as declaration units.

Nested classes remain illegal according to existing parser/compiler rules.

## 20.10 Export

Exports affect semantic interface/invalidation but do not create runtime expression checks themselves.

---

# 21. Typed flow state

## 21.1 Do not build a second control-flow walker

The repository already has structured flow for:

- returns;
- breaks;
- continues;
- throws;
- conditional block sends;
- `and`/`or`;
- while fixed points;
- for loops;
- pattern binding;
- field writes;
- call events.

The type checker must reuse it.

## 21.2 Migration target

Move/extract the VM-free flow engine into `phalcom-semantic`.

A practical intermediate state can keep both advisory and typed facts in one traversal:

```rust
pub struct SemanticFlowState {
    pub value_shapes: BTreeMap<BindingId, InferredValue>,
    pub types: BTreeMap<BindingId, TypeKnowledge>,
}
```

or parameterize the flow walker over a domain.

Avoid a giant generic abstraction if it materially harms readability. The hard requirement is one source of branch/loop reachability semantics.

## 21.3 Declared binding rule

A mutable binding with declared type `T` keeps `T` as its contract after assignment.

Assignment may refine current exact value knowledge but cannot replace the declaration.

Example:

```phalcom
let x: Number = 1
x = 1.5
```

Current value may be `Float`; declared contract remains `Number`.

## 21.4 Flow refinement

Trusted test:

```phalcom
if x.is(User) {
    ...
}
```

may narrow a union/nominal supertype in the true branch if the existing type-test semantics guarantee it.

Refinement must retain provenance and be invalidated by writes to the mutable binding.

## 21.5 Merge

At control-flow joins:

- declared contracts remain unchanged;
- current inferred type facts are joined;
- unknown on one reachable branch prevents an unjustified proof;
- `Never` branch does not contribute a normal value.

## 21.6 Loop fixed point

Reuse existing bounded iteration/widening constants/strategy.

Typed widening must be conservative.

Do not widen to `Dynamic`; widen to Unknown knowledge.

---

# 22. Callable signatures

Keep advisory and normative signatures separate.

## 22.1 Existing advisory summary

Preserve current conceptual model:

```rust
CallableSummary {
    params: Vec<InferredValue>,
    returns: InferredValue,
    dependencies: ...,
    effects: ...,
}
```

This is editor/inference evidence.

## 22.2 Declared signature

Add:

```rust
pub struct DeclaredCallableSignature {
    pub callable: CallableId,
    pub parameters: Box<[DeclaredParameterType]>,
    pub result: Option<ResolvedTypeAnnotation>,
}
```

Parameter annotation absence remains `None`.

## 22.3 Checked signature

After body checking:

```rust
pub struct CheckedCallableSignature {
    pub callable: CallableId,
    pub declared: DeclaredCallableSignature,
    pub inferred_result: TypeKnowledge,
    pub status: SignatureCheckStatus,
}
```

## 22.4 Parameter call-site evidence

Do not convert observed callers into a declared parameter type.

This current LSP feature remains advisory.

If a parameter is unannotated, call-site evidence may improve hover/inlay hints but it does not create a hard public contract.

## 22.5 Return inference

An unannotated function may expose an inferred result to the LSP.

For static checking of callers, only use an inferred result as authoritative if the checker has proved all reachable normal returns under its supported semantics.

Otherwise caller result remains Unknown/advisory.

---

# 23. Argument checking

When a callable is resolved and a parameter has an annotation:

1. map actual pack items to formal selector slots;
2. preserve existing label/rest semantics;
3. synthesize actual expression;
4. `is_assignable(actual, expected)`;
5. report only `Refuted`;
6. retain `Unknown` as residual;
7. Dynamic skips hard checking for that relation.

Diagnostic:

```text
type.argument.incompatible
```

Primary span: actual argument expression.

Secondary span: parameter annotation.

Message should name selector and parameter.

Example:

```text
error[type.argument.incompatible]:
argument for `value` has type String, expected Int
```

Dynamic packs (`...`, computed labels) prevent complete static mapping. Do not guess argument positions.

---

# 24. Return checking

## 24.1 Explicit return

For:

```phalcom
run() -> String {
    return 1
}
```

emit:

```text
type.return.incompatible
```

Primary: returned expression.

Secondary: return annotation.

## 24.2 Tail expression

Current compiler body semantics return the last value-producing statement when falling off the body.

The type checker must check the reachable tail value against the declared return contract.

Do not check only explicit `return`.

## 24.3 No value

If a body can fall through without a value and runtime returns absence/None, include that path.

A declaration:

```phalcom
run() -> String {
}
```

must not be accepted merely because there is no explicit return to inspect.

## 24.4 Throw-only body

If every path terminates with `throw`/Never, then:

```text
Never <: declared result
```

and the body is valid for any declared result.

---

# 25. Field checking

## 25.1 Declaration initializer

```phalcom
_count: Int = 0
```

check initializer.

Diagnostic:

```text
type.field.initializer_mismatch
```

## 25.2 Constructor writes

Every statically visible write to annotated field is checked.

## 25.3 General writes

Same check.

Existing field evidence categories can remain for presentation:

```text
DeclarationInitializer
ConstructorInitialization
GeneralWrite
```

## 25.4 Unannotated fields

Observed writes remain advisory shape evidence.

Do not freeze the first observed type as a permanent field contract.

---

# 26. Local assignment checking

When assignment target resolves to a binding with declared annotation:

```phalcom
let x: Number = 1
x = "bad"
```

emit:

```text
type.assignment.incompatible
```

Existing immutability diagnostics take precedence for writes to `const`.

Do not produce a redundant type error when assignment is already illegal due to const semantics unless tests explicitly require multi-diagnostic reporting.

---

# 27. Special-type policies

## 27.1 Never

Canonical, no inhabitants.

Sources include:

- throw-only expressions/paths;
- native `ReturnFlowSpec::Never`;
- future exhaustive impossible branches.

## 27.2 Dynamic

Explicit type annotation boundary.

Example:

```phalcom
let payload: Dynamic = external()
```

Static calls on Dynamic remain dynamically resolved.

Do not report receiver-member absence based on Dynamic.

## 27.3 Unknown

Internal only in this milestone.

Do not expose an `Unknown` runtime type object merely because `phalcom-type-syntax` has a native metadata token named `Unknown`.

## 27.4 Unit

Resolve existing `Unit` nominal class/type.

Final zero-product equivalence is deferred.

## 27.5 None / Option

Respect current runtime absence semantics.

Do not conflate `None` with Unit.

A future `Option<T>` generic checker will refine this further.

## 27.6 Any

Not required.

Keep relation/store extensible.

---

# 28. Module semantic interfaces

Cross-module static checking requires type signatures to survive interface construction.

## 28.1 Do not put body facts in public interfaces

Interface metadata may include:

- declaration kind;
- generic signature later;
- superclass;
- parameter annotations;
- result annotations;
- field annotations where semantically visible.

Do not include:

- local variable inferred types;
- transient branch refinements;
- observed call-site parameter shapes;
- private implementation facts unless needed for same-package checking.

## 28.2 Interface representation

Recommended:

```rust
pub struct SemanticModuleInterface {
    pub module: ModuleId,
    pub declarations: BTreeMap<DeclarationId, SemanticDeclarationSignature>,
    pub exports: ...,
}
```

For the first milestone, this can live in `phalcom-semantic` and be built from linked module interfaces plus source surfaces.

## 28.3 Type dependency edges

Whenever an annotation references another declaration, add:

```rust
SemanticEdgeKind::TypeReference
```

Constraint/generic edges use their already-reserved categories later.

## 28.4 SCCs

Semantic declaration cycles may be legal even when runtime initialization cycles are not.

Use existing declaration shells/semantic SCC machinery.

Do not force type interfaces into runtime DAG ordering.

---

# 29. Declaration fingerprints and incremental invalidation

Current LSP declaration fingerprints do not include type annotations.

They must.

## 29.1 Parameter fingerprint

Add normalized annotation syntax:

```rust
pub annotation: Option<TypeAnnotationFingerprint>
```

## 29.2 Member fingerprint

Add return annotation fingerprint.

## 29.3 Field fingerprint

Add field annotation fingerprint.

## 29.4 Top-level binding interface

If top-level annotated bindings participate in exports, include their annotations in module semantic-interface fingerprints.

## 29.5 Range-insensitive

Fingerprints must ignore source offsets.

Formatting/comment movement cannot become a semantic declaration change.

## 29.6 Two-stage invalidation

Recommended future optimization:

```text
source annotation syntax changed
     |
     v
re-resolve declaration interface
     |
     +-- canonical semantic signature unchanged
     |      -> avoid dependent recheck
     |
     +-- signature changed
            -> invalidate semantic dependents
```

The first implementation may conservatively invalidate dependents on any normalized annotation change.

## 29.7 Body-only edits

Preserve current precise callable invalidation.

Adding types must not regress every edit into workspace-wide reanalysis.

---

# 30. Snapshot architecture

Keep the LSP's strong immutable publication model.

## 30.1 Shared semantic snapshot

Recommended core:

```rust
pub struct SemanticSnapshot {
    pub generation: SemanticGeneration,
    pub sources: Arc<...>,
    pub declarations: Arc<...>,
    pub shapes: Arc<...>,
    pub type_store: Arc<TypeStoreSnapshot>,
    pub declared_signatures: Arc<...>,
    pub checked_signatures: Arc<...>,
    pub diagnostics: Arc<...>,
    pub graph: Arc<...>,
}
```

## 30.2 LSP adapter

URI mapping stays in LSP.

Request handlers must not acquire mutable checker state.

## 30.3 Compiler

Compiler can use a one-shot semantic build or the same snapshot builder without editor URI machinery.

---

# 31. Diagnostics

## 31.1 Shared diagnostic model

Create:

```rust
pub struct SemanticDiagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub primary: DiagnosticLabel,
    pub secondary: Vec<DiagnosticLabel>,
    pub notes: Vec<String>,
}
```

This type is VM/LSP independent.

## 31.2 Labels

```rust
pub struct DiagnosticLabel {
    pub module: ModuleId,
    pub range: SourceRange,
    pub message: String,
}
```

## 31.3 Initial stable codes

Required or equivalent stable names:

```text
type.annotation.unresolved_name
type.annotation.unsupported_form
type.annotation.kind_mismatch
type.annotation.destructuring_not_supported
type.binding.initializer_mismatch
type.binding.uninitialized_annotation
type.assignment.incompatible
type.argument.incompatible
type.return.incompatible
type.field.initializer_mismatch
type.field.assignment_mismatch
type.constructor.explicit_return
```

## 31.4 Contradiction structure

Example:

```phalcom
const x: String = 1
```

Desired information:

```text
error[type.binding.initializer_mismatch]:
initializer has type Int, but binding `x` is declared String

primary:
    `1`
    inferred as Int from integer literal

secondary:
    `String`
    declared binding type
```

## 31.5 Unknown is not an error

If actual type cannot be established:

```text
no contradiction diagnostic
```

The LSP may present:

```text
declared: User
inferred: unknown
```

## 31.6 Dynamic is not an error

It may produce an informational obligation in a future strict checker, but not a first-milestone error.

## 31.7 CLI renderer

Do not route semantic errors through:

```rust
CompilerError::Message(String)
```

as the primary architecture.

Add shared diagnostic rendering/plumbing so source spans are preserved.

## 31.8 LSP renderer

Map shared semantic diagnostics to `tower_lsp::lsp_types::Diagnostic`.

Use the same code/message/source ranges.

Compiler and LSP tests must prove parity for representative errors.

---

# 32. Compiler integration

## 32.1 Pipeline

Target:

```text
source
  |
  v
parse once
  |
  v
source/interface extraction
  |
  v
module resolution + linking
  |
  v
semantic surface + scope construction
  |
  v
annotation resolution
  |
  v
static checker
  |
  +-- refuted -> compile diagnostics / abort
  |
  +-- no refutation
          |
          v
      bytecode lowering
          |
          v
          VM
```

## 32.2 Static checking in normal compilation

Annotated contradictions are compile-time errors in normal compilation.

They are not limited to a special typed runner.

## 32.3 Existing `CompileMode`

Do not bind type checking to:

```text
Debug
Release
Unchecked
```

Those modes retain current contract-weaving meaning.

If a future option disables semantic checking for experimentation, it must be a separately named option and is not part of this milestone.

## 32.4 `phalcom check`

Today this command is syntax-only.

Upgrade path:

```text
phalcom check
  -> parse
  -> module resolution/linking where context exists
  -> semantic checking
  -> diagnostics
  -> no bytecode/run
```

For standalone source without project context, perform all checks that can be resolved and report incomplete-project unknowns conservatively.

Do not silently change the meaning of existing `--release`/`--unchecked`.

---

# 33. LSP integration

## 33.1 One semantic result

The LSP must consume the same canonical type relations/check results used by compiler semantic checking.

Do not implement a separate editor-only annotation checker.

## 33.2 Hover

For an annotated binding, hover should distinguish:

```text
Declared: Number
Current inferred: Int
Status: compatible
```

For contradiction:

```text
Declared: String
Current inferred: Int
Status: incompatible
```

For uncertain:

```text
Declared: User
Current inferred: unknown
```

## 33.3 Inlay hints

Do not repeat source-written annotations as redundant inlay hints.

Use existing inferred hints only where they add information.

Future annotation-value analysis may classify:

```text
exactly inferred
widened
public boundary
ambiguous
```

but that is not a completion requirement.

## 33.4 Completion

When receiver has an authoritative declared/proven nominal type, completion may use it.

Fallback to current `ValueShape` inference for untyped/incomplete code.

## 33.5 Signature help

Prefer declared parameter/result types.

Show advisory inferred information only as secondary presentation.

## 33.6 Diagnostics publication

Semantic diagnostics must be included with parser diagnostics without suppressing syntax recovery.

Do not run checker on structurally unrecoverable AST fragments if invariants cannot be trusted.

## 33.7 Performance

LSP request paths stay snapshot-only and nonblocking.

All checking work remains worker-side/cancellable.

---

# 34. Attribute/decorator expansion boundary

This is a critical repository-specific constraint.

Current bytecode compiler runs `expand_class_attributes()` and may synthesize members before lowering.

The LSP currently models some attribute effects from source without using VM compiler expansion.

The type checker MUST NOT make VM-owned mutable AST expansion the canonical semantic declaration model.

## 34.1 First milestone rule

Check source-authored declarations plus already-exposed semantic/native surfaces.

Honor current semantic placement effects such as:

```text
@class
@constructor
visibility
```

through source surface construction.

## 34.2 Generated members

For compiler-generated members from features such as `@construct`, `@data`, or variants:

- do not invent signature types if source semantics do not expose them yet;
- do not create false hard diagnostics;
- preserve current dynamic behavior.

## 34.3 Follow-up extraction

Long-term, create a VM-independent semantic expansion layer whose output describes generated declaration signatures without necessarily generating compiler AST.

Example conceptual trait:

```rust
trait SemanticAttributeExpander {
    fn expand_signatures(
        &self,
        declaration: &SourceDeclaration,
        ctx: &SemanticExpansionContext,
    ) -> Vec<GeneratedSemanticDeclaration>;
}
```

Bytecode AST synthesis remains a compiler concern.

This separation prevents tooling/compiler divergence.

---

# 35. Reflection persistence preparation

Even though runtime type objects are deferred, the compiler must not destroy source type metadata.

## 35.1 Preserve source record

Retain:

```rust
pub struct ResolvedTypeAnnotation {
    pub source: TypeAnnotation,
    pub resolved: TypeKnowledge,
}
```

or equivalent.

## 35.2 Compiled artifacts

`CompiledProgram` / semantic artifacts should retain enough annotation metadata to support later runtime reflection encoding without reparsing source.

## 35.3 No runtime wrappers

Do not wrap ordinary runtime values to carry generic/type annotations.

A value annotated:

```phalcom
const values: List = ...
```

remains its ordinary runtime object.

## 35.4 Future descriptor materialization

Canonical `TypeId` -> runtime reflective object mapping is a future bridge.

Design TypeStore so descriptors can be materialized lazily and interned.

---

# 36. Performance requirements

## 36.1 Parse count

Project/module source should be parsed once per source revision per frontend ingestion path.

## 36.2 Type interning

Avoid recursively cloned type trees in facts/summaries.

Use compact `TypeId`.

## 36.3 No per-query recomputation

LSP hover/completion/signature must read published results.

## 36.4 Incrementality

A body-only edit should preserve existing narrow callable invalidation where possible.

## 36.5 Native normalization cache

Normalize static native metadata once per semantic universe generation, not per hover/call.

## 36.6 Relation memoization

Nominal ancestry/subtyping may be cached by:

```text
(TypeId, TypeId, hierarchy_generation)
```

Do not add complex memoization before measurement.

## 36.7 Diagnostics

Store structured evidence, but cap nonessential presentation provenance to prevent large graphs.

## 36.8 Benchmark gates

Record before/after:

- LSP source update latency;
- semantic worker passes;
- hover request latency;
- project compile parse count;
- project compile wall time on representative fixtures.

No typing feature should reintroduce whole-workspace analysis on every keystroke.

---

# 37. File-by-file implementation plan

This section is normative enough to execute directly.

## 37.1 Root `Cargo.toml`

Add:

```text
phalcom-semantic
```

to workspace members/default members.

## 37.2 `phalcom-semantic/Cargo.toml`

Create VM-free dependencies as described in §6.

## 37.3 `phalcom-ast/src/ast.rs`

Add `TypeAnnotation` and first source type-reference representation.

Add annotation fields to:

```text
LetBinding
ParameterDef
FieldDef
MethodDef
GetterDef
SetterDef
IndexMethodDef
```

Update derives only where valid.

## 37.4 `phalcom-ast/src/parser.rs`

Implement helper:

```rust
fn parse_type_annotation(&mut self) -> ParserResult<TypeAnnotation>
```

For first milestone parse a qualified static reference.

Modify:

```text
parse_binding
parse_field_decl
parse_selector_params
setter branch in parse_class_member
parse_index_member
method/getter/member return parsing
```

Preserve error recovery.

## 37.5 `phalcom-ast/tests/...`

Add parser/snapshot fixtures for every supported annotation position and error condition.

## 37.6 `phalcom-modules/src/resolver.rs`

Add parsed-source caching/API.

Avoid dropping `Arc<Program>` after interface build.

## 37.7 `phalcom-modules/src/interface.rs`

Where required, enrich declaration interface metadata with declaration kind/type-surface references or provide a semantic-layer builder that can consume source surfaces after linking.

Do not move checker logic into this crate.

## 37.8 `phalcom-modules/src/linker.rs`

Add/extend declaration-reference resolution if needed:

```rust
resolve_declaration_reference(...)
```

returning canonical declaration identity.

Do not overload value `SymbolId` if declaration kind validation matters.

## 37.9 `phalcom-modules/src/graph.rs`

Use existing `TypeReference` edges.

No new parallel graph.

## 37.10 `phalcom-semantic/src/types/store.rs`

Implement TypeStore, interning, special IDs, kind lookup.

## 37.11 `phalcom-semantic/src/types/relation.rs`

Implement relation kernel.

## 37.12 `phalcom-semantic/src/types/evidence.rs`

Implement knowledge/evidence separation.

## 37.13 `phalcom-semantic/src/types/annotation.rs`

Implement source annotation resolution.

## 37.14 `phalcom-semantic/src/types/native.rs`

Normalize native metadata.

## 37.15 LSP semantic extraction

Move or adapt VM-free modules from:

```text
phalcom-lsp/src/semantic/facts.rs
callable.rs
scope.rs
dispatch.rs
flow.rs
analyzer.rs
surface.rs
invalidation.rs
snapshot.rs
```

incrementally.

Keep LSP-specific:

```text
URI mapping
occurrence presentation
tower-lsp diagnostics
worker/server wiring
editor formatting
```

The implementation may initially re-export moved types to minimize churn.

## 37.16 `phalcom-lsp/src/semantic/invalidation.rs`

Extend declaration fingerprints with type annotations.

Eventually move core invalidation model to shared crate.

## 37.17 `phalcom-lsp/src/hover.rs`

Consume declared/checked types in addition to advisory shapes.

## 37.18 `phalcom-lsp/src/inlay_hints.rs`

Suppress hints redundant with written annotations.

## 37.19 `phalcom-lsp/src/diagnostics.rs`

Accept shared semantic diagnostics in addition to parser errors.

## 37.20 `phalcom-core/src/modules/compile.rs`

Retain parsed programs in `CompiledModule`.

Invoke semantic checker after linking.

Add semantic diagnostics to `ProgramCompileError` through a structured variant, e.g.:

```rust
Semantic(Vec<SemanticDiagnostic>)
```

or a report object.

## 37.21 `phalcom-core/src/interpret.rs`

Use retained AST for compiled programs.

Keep source-parsing entry points for REPL/legacy convenience.

## 37.22 `phalcom-core/src/compiler/lib/mod.rs`

Add AST-based compile entry if required.

Do not embed the checker into bytecode emission.

## 37.23 `phalcom-core/src/compiler/attributes.rs`

Update every synthesized AST struct literal for new annotation fields.

Where a generated member semantically inherits a field annotation, do so only if the attribute specification actually defines that contract.

Otherwise set `None`.

## 37.24 `phalcom-core/src/compiler/lib/class_decl.rs`

Update struct literals/helpers.

No type-specific dispatch changes.

## 37.25 `phalcom-core/bin/phalcom/cli.rs`

Upgrade `check` only after semantic checker pipeline is available.

Render structured semantic diagnostics.

Do not change current contract compile-mode meanings.

---

# 38. Recommended implementation phases

Each phase must leave the workspace compiling and tests passing.

## Phase A — semantic crate scaffold and identity bridge

Implement:

- crate;
- canonical aliases/IDs;
- TypeId/KindId/InferVarId;
- TypeStore skeleton;
- basic intern tests.

No source syntax yet.

### Gate

```sh
cargo check --workspace
cargo test -p phalcom-semantic
```

## Phase B — source annotation AST

Add AST fields and type-reference syntax nodes.

Update all struct literals.

No semantic effect yet.

### Gate

`cargo check --workspace` must prove every constructor site was consciously migrated.

## Phase C — parser

Implement all annotation positions in §10.

Add parser snapshots.

### Gate

Existing untyped source parses identically modulo added `None` fields.

## Phase D — parse-once source pipeline

Introduce `ParsedSourceUnit`.

Retain `Arc<Program>` through modules/compiler.

Eliminate project double parse.

This can land before or after checker logic but should land before full integration.

## Phase E — canonical type resolution

Implement:

- nominal references;
- builtin mapping;
- Never;
- Dynamic knowledge;
- unresolved-name diagnostics;
- module/import resolution.

## Phase F — minimal relation kernel

Implement:

- equality;
- Never;
- nominal inheritance;
- union;
- Dynamic/Unknown result handling.

## Phase G — declaration signatures

Resolve parameter/field/return annotations into semantic signature tables.

Add type-reference semantic graph edges.

## Phase H — expression checker

Implement literal/variable/self/basic-send synthesis plus assignment checking.

Do not start with every advanced expression at once.

Use exhaustive match with intentionally Unknown policies for deferred forms.

## Phase I — flow/body checking

Integrate with existing flow traversal.

Add:

- tail return checking;
- explicit return;
- branch reachability;
- mutable assignment;
- field writes.

## Phase J — call checking

Validate resolved arguments.

Use declared/trusted native results.

## Phase K — compiler integration

Run semantic checking after linking.

Abort on refuted semantic errors.

## Phase L — LSP integration

Publish same diagnostics and declared-type presentation.

Preserve advisory inference fallback.

## Phase M — native metadata bridge

If not already done earlier, switch normative native signatures from `NativeReturnShape` to rich native metadata where supported.

## Phase N — performance/invalidation hardening

Measure, fix regressions, finalize fingerprints and snapshot reuse.

---

# 39. Required tests

## 39.1 AST/parser tests

Must cover:

```phalcom
const x: Int = 1
let x: Int = 1
_name: String
const _id: Int = 1

run(value: Int) { }
run(_ value: Int) { }
run(to value: String) { }
run(*values: Object) { }

run() -> Int { 1 }
name -> String { "x" }
name=(put value: String) -> String { value }
[_ i: Int] -> String { "x" }
[_ i: Int]=(put value: String) -> String { value }
```

Negative:

- missing annotation name;
- malformed qualified name;
- colon after destructuring if rejected parser-side;
- explicit constructor result diagnostic later;
- variant labels unchanged;
- old untyped parameter syntax unchanged.

## 39.2 TypeStore tests

- same nominal interns once;
- union ordering canonical;
- nested unions flatten;
- duplicates removed;
- Never removed from nonempty union;
- zero-member normalized union -> Never;
- TypeId stable within store;
- kind lookup deterministic.

## 39.3 Resolution tests

Local class:

```phalcom
class User {}
const u: User = ...
```

Selective import.

Module-qualified import.

Builtin `Int`.

Unknown type name.

Dynamic.

Never.

## 39.4 Relation tests

```text
Int <: Int
Never <: Int
Int <: Number
Int <: Object
String !<: Int
(Int | String) <: Object
Int <: (Int | String)
```

Unknown/Dynamic result tests.

## 39.5 Binding checker

Valid:

```phalcom
const x: Int = 1
```

Invalid:

```phalcom
const x: String = 1
```

Mutable reassignment valid/invalid.

Unannotated remains accepted.

## 39.6 Return checker

Valid exact.

Valid subtype.

Invalid explicit return.

Invalid tail return.

Throw-only valid for arbitrary result.

Fallthrough absence mismatch.

## 39.7 Field checker

Initializer mismatch.

Constructor write mismatch.

General write mismatch.

Unannotated mixed writes do not become hard errors.

## 39.8 Call checker

Positional parameter.

Labeled parameter.

External/local split.

Setter value.

Index parameter.

Dynamic pack remains Unknown rather than false error.

## 39.9 Native metadata tests

- universe nominal normalization;
- Never flow;
- unknown metadata remains Unknown knowledge;
- receiver return flow;
- argument-return flow where implemented;
- union normalization.

## 39.10 Flow tests

- branch narrowing;
- assignment invalidates narrowing;
- loop merge conservative;
- unreachable Never branch does not contaminate normal result;
- existing shape flow tests continue to pass.

## 39.11 Cross-module tests

Module A exports annotated class/method.

Module B imports and correctly checks call.

Changing exported annotation invalidates B.

Changing private body without signature change does not unnecessarily recheck unrelated module in the optimized path.

## 39.12 LSP tests

Hover declared+inferred.

Diagnostic exact range.

Compiler/LSP same diagnostic code.

No redundant inlay for written type.

Untyped completion still works.

Body-only edit preserves narrow invalidation.

## 39.13 Compiler tests

Normal run rejects provable type mismatch before bytecode execution.

Unknown case still compiles.

Dynamic case still compiles.

Release/Unchecked contract modes do not disable static type contradiction checks.

## 39.14 Parse-count regression

Instrument parser calls for project compile and assert no second bytecode-lowering parse after retained AST migration.

---

# 40. Diagnostic examples

## 40.1 Binding

Source:

```phalcom
const port: String = 8080
```

Expected:

```text
error[type.binding.initializer_mismatch]: initializer for `port` is not assignable to declared type `String`
  --> file.ph:1:22
   |
 1 | const port: String = 8080
   |             ------   ^^^^ has type Int
   |             |
   |             declared String
```

Exact renderer formatting may follow current diagnostic style.

## 40.2 Argument

```phalcom
send(port: Int) { ... }

send("8080")
```

Expected semantic information:

```text
actual: String
expected: Int
parameter: port
selector: send(port)
```

## 40.3 Return

```phalcom
port() -> Int {
  "8080"
}
```

Primary: `"8080"`.

Secondary: `Int`.

## 40.4 Unknown

```phalcom
const user: User = plugin.load()
```

If `plugin.load()` cannot be typed:

```text
no error
knowledge = Unknown
residual obligation = optional future typed-runner check
```

LSP may explain uncertainty.

---

# 41. Checker correctness rules

## 41.1 Never diagnose from heuristics alone

A hard error requires a sound contradiction.

## 41.2 Do not infer public contracts from callers

Observed calls do not define a parameter's contract.

## 41.3 Do not assume closed-world dynamic dispatch

Absence of a currently known send target does not prove impossibility when receiver/dynamic semantics are uncertain.

## 41.4 Do not confuse class inheritance with the final full subtype relation

First milestone nominal subtyping uses inheritance.

Future protocols, unions, generics, special types, and structural conformance extend type relations.

Keep APIs general.

## 41.5 Do not use runtime class inheritance as an annotation resolver

Resolve source declaration identity statically through module/linker semantics.

---

# 42. Higher-kinded and generic compatibility requirements

The first milestone is not complete if it paints the architecture into a corner.

## 42.1 Kinds

TypeStore must have a place to record kinds.

## 42.2 Applied types

Canonical node shape must be possible:

```rust
Applied {
    origin: TypeId,
    arguments: Box<[TypeId]>
}
```

## 42.3 Partial application

Current first milestone does not support it.

Do not hardcode an assumption that every type constructor is always saturated forever.

A later design may choose explicit type lambdas/partial application.

## 42.4 Variance

Future declaration syntax is intended to use unary-style markers:

```phalcom
class Producer<+T>
class Consumer<-T>
class Cell<T>
```

Do not build relation code around stale `in`/`out` spellings from older design documents.

Internal variance enum can already be:

```rust
pub enum Variance {
    Covariant,
    Contravariant,
    Invariant,
}
```

but no active variance checking is required now.

## 42.5 Local generic inference

Future solver flow:

1. instantiate callable type parameters with fresh `InferVarId`s;
2. add expected-result constraints;
3. map actual arguments;
4. add subtype/equality constraints;
5. solve bounds;
6. substitute result.

The current first checker APIs must allow this to be inserted without redesign.

---

# 43. Runtime typed-runner preparation

Not implemented now.

But represent checking outcomes so later code can map:

```text
Proven
  -> no runtime check required

Refuted
  -> compile error

Unknown
  -> possible runtime check obligation

Dynamic
  -> deliberately unchecked or boundary-policy check
```

Do not insert runtime bytecodes in this milestone.

Do not change ordinary VM dispatch.

---

# 44. Risks and mitigations

## Risk 1 — semantic crate becomes an LSP dump

Mitigation:

- keep URI/tower-lsp/server logic out;
- expose VM/editor-independent queries.

## Risk 2 — checker trusts `ValueShape` too much

Mitigation:

- explicit authority bridge;
- hard diagnostics require sound evidence.

## Risk 3 — AST annotations create massive constructor churn

Mitigation:

- migrate struct literals in one dedicated commit;
- use `cargo check --workspace`;
- no hidden defaults.

## Risk 4 — annotation edit causes whole workspace rebuild

Mitigation:

- extend current declaration fingerprints;
- preserve body-only callable frontier;
- later compare resolved interface hash.

## Risk 5 — compiler and LSP diverge on IDs

Mitigation:

- converge on `phalcom_modules::ModuleId` + `DeclarationId`;
- URI identity stays at editor boundary.

## Risk 6 — native type truth remains duplicated

Mitigation:

- normalize `phalcom-native-meta`;
- treat `NativeReturnShape` as transition/advisory.

## Risk 7 — attribute-generated APIs differ between compiler and tooling

Mitigation:

- first milestone checks source-authored surface conservatively;
- later extract VM-independent semantic attribute expansion.

## Risk 8 — type syntax becomes a second language

Mitigation:

- use ordinary Phalcom lexer;
- source annotation syntax lowers to ordinary type-denoting semantic entities;
- restrict static evaluation by phase/effect, not unrelated semantics.

## Risk 9 — current `check` command semantics surprise users

Mitigation:

- document upgrade from syntax check to semantic check;
- keep output codes/formats stable where possible;
- add regression tests.

---

# 45. Completion criteria

The first functional typing milestone is complete only when all of these are true.

## Source

- [ ] supported annotations parse at every specified declaration site;
- [ ] untyped source remains backward-compatible;
- [ ] annotations preserve source ranges;
- [ ] unsupported type forms fail explicitly, not silently.

## Identity

- [ ] shared checker uses `phalcom_modules::ModuleId`;
- [ ] nominal identities derive from canonical declaration identity;
- [ ] LSP URI keys are not canonical language identities.

## Types

- [ ] canonical `TypeStore` exists;
- [ ] Unknown is not a canonical type;
- [ ] Dynamic is not conflated with top/unknown;
- [ ] Never is bottom;
- [ ] nominal subtype uses static hierarchy;
- [ ] union normalization/relations are tested.

## Checker

- [ ] initializer mismatch detected;
- [ ] assignment mismatch detected;
- [ ] field mismatch detected;
- [ ] argument mismatch detected where target signature is known;
- [ ] explicit and tail return mismatch detected;
- [ ] unknown does not produce false errors;
- [ ] Dynamic does not produce false errors;
- [ ] throw/Never paths are modeled conservatively.

## Shared architecture

- [ ] compiler and LSP use the same relation/checker implementation;
- [ ] LSP advisory `ValueShape` remains available;
- [ ] no second control-flow semantics is introduced;
- [ ] native rich metadata has a normalization path.

## Compiler

- [ ] semantic checking occurs before bytecode lowering;
- [ ] normal dynamic execution semantics remain unchanged;
- [ ] contract `CompileMode` meanings remain unchanged;
- [ ] project source is not reparsed for bytecode after retained-AST migration.

## LSP

- [ ] declared type appears in hover;
- [ ] inferred/advisory type remains visible where useful;
- [ ] semantic diagnostics publish with exact spans;
- [ ] compiler/LSP diagnostic codes agree;
- [ ] request path remains snapshot-only/nonblocking.

## Performance

- [ ] body-only edit does not cause unnecessary full-workspace checking;
- [ ] no per-hover type recomputation;
- [ ] benchmark/perf counters show no unbounded regression.

## Tests

- [ ] `cargo check --workspace`;
- [ ] new `phalcom-semantic` unit tests;
- [ ] AST/parser tests;
- [ ] module/linker type-resolution tests;
- [ ] checker tests;
- [ ] LSP integration tests;
- [ ] compiler integration tests;
- [ ] native metadata tests;
- [ ] parse-count regression test;
- [ ] existing test suites remain green.

---

# 46. Explicitly deferred follow-up phases

After this milestone, recommended order is:

1. **Applied types and source generic application**
   - `List<Int>`;
   - canonical application validation.

2. **Generic declaration signatures**
   - class/method type parameters;
   - `+T` / `-T` variance metadata.

3. **Substitution and local generic inference**
   - `InferVarId`;
   - constraints;
   - bounds.

4. **Structural protocols**
   - complete-selector conformance;
   - class-side requirements.

5. **Callable/block types**
   - typed closures;
   - higher-order APIs.

6. **Flow-rich special types**
   - Option/Result/variants;
   - exhaustiveness.

7. **Higher-kinded parameters**
   - kind checking;
   - type constructors as parameters;
   - type lambdas/partial application decision.

8. **Runtime reflection**
   - `Type` protocol/object surface;
   - lazy descriptor materialization;
   - annotation metadata on methods/fields/parameters.

9. **Typed runner/runtime obligations**
   - instrument unresolved static obligations;
   - source-aware runtime failures.

10. **Optimization**
    - specialization;
    - representation selection;
    - devirtualization where semantically proven.

---

# 47. Final implementation invariant

The engineering team should use this invariant to reject tempting shortcuts:

> Phalcom has one dynamic object/message language. Static typing is a canonical semantic analysis of that language, not a parallel erased type language. Source annotations state intent; static inference supplies evidence; proven contradictions are errors; uncertainty remains explicit; runtime execution remains dynamically dispatched unless a separate language feature explicitly changes it.

A correct implementation therefore has:

```text
one parser
one module/declaration identity model
one selector model
one control-flow meaning
one shared semantic checker
two complementary knowledge layers:
    advisory runtime shape
    normative language type
one dynamic runtime
```

and not:

```text
LSP type guesses
+
compiler type guesses
+
native type strings
+
runtime classes
```

that happen to use similar names.

That convergence is the central goal of this milestone.
