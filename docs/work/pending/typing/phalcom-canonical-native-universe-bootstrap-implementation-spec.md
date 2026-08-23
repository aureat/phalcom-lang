# Phalcom Canonical Native Universe, Primitive Metadata, and Verified Bootstrap — Repository-Grounded Implementation Specification

> **Status:** Proposed implementation specification
> **Repository:** `aureat/phalcom-lang`
> **Grounded revision:** `edbced8914b924f95f4dbc83508c93ca77cd36ce` (`main`, “extend runtime typing metadata and reflection”, 2026-08-22)
> **Date:** 2026-08-23
> **Primary scope:** canonical universe source, native primitive declaration/registration, compiler attributes, bootstrap verification, runtime typing integration, and LSP semantic integration
> **Normative companion specifications:** `@internal`, `@native`, Rust `#[primitive(...)]`, and `@class` attribute specifications prepared with this design

---

## 1. Executive Summary

This specification defines the implementation program required to make Phalcom's native core *visible, typed, navigable, and mechanically verified* from canonical Phalcom source without turning that source into a second unverified implementation registry. The end state has two intentionally independent declarations of native behavior: the Rust `#[phalcom_native_macros::primitive(...)]` declaration is the machine-authoritative native implementation contract, while the canonical `phalcom-core/core/universe/src/**/*.ph` declaration is the language-facing source contract. Bootstrap verifies that the two agree before installing or executing the universe. Every other machine surface—runtime primitive installation tables, the VM-free LSP native surface, native reflection metadata, and native method typing metadata—is derived from those authoritative declarations rather than maintained as an additional handwritten registry.

The change is not merely a syntax addition. The repository currently contains three partially overlapping native systems. First, `phalcom-core/src/universe/primitives.rs` still manually installs a large native floor through `primitive!`, `primitive_internal!`, `primitive_static!`, `primitive_shape!`, and related macros. Second, the newer `phalcom-native-macros` + `phalcom-native-meta` + `phalcom-core/src/native/*` descriptor pipeline registers a subset of primitives through `#[primitive(...)]` and `linkme`. Third, `phalcom-native-surface/src/lib.rs` manually repeats class/member/visibility/return-shape information for LSP and runtime validation. The bootstrap currently invokes both `Universe::install_primitives(&mut vm)` and `install_registered_primitives(&mut vm)`. This is a migration architecture, not an acceptable final architecture.

The canonical universe source has a similar split. The actual universe is already a project/package tree under `phalcom-core/core/universe`, and `phalcom-modules::BuiltinProjectSourceProvider` already knows how to expose bundled universe modules using stable `phalcom://universe/...` source identities. Runtime bootstrap, however, still carries its own hard-coded `SOURCES` list in `VM::run_universe_modules`, while `phalcom-lsp/src/semantic/core_source.rs` still treats `phalcom-core/core/core.ph` as one bundled semantic core document and overlays the handwritten native surface. The implementation therefore centralizes source loading on the existing builtin-project provider rather than introducing another source-catalog crate.

The target boot process becomes a staged proof of consistency:

```text
construct primordial object/class tower
              │
              ▼
collect + validate Rust PrimitiveDescriptors
              │
              ▼
load and parse canonical universe source corpus
              │
              ▼
build NativeSourceIndex from @native/@internal/@class declarations
              │
              ▼
verify source ↔ native structural agreement
              │
              ▼
build/register native callable typing metadata
              │
              ▼
install descriptor-backed primitives
              │
              ▼
compile universe source
  - declaration-only @native members emit nothing
  - reference-bodied @native members emit nothing
  - real .ph wrappers and algorithms emit bytecode
              │
              ▼
execute universe modules
              │
              ▼
verify post-bootstrap runtime invariants
```

A failed source/native consistency check must fail *before* native installation and before universe code executes. This gives Phalcom three different verification layers: declaration correctness, installation correctness, and runtime object-graph correctness.

---

## 2. Goals

The implementation MUST produce the following externally visible properties.

1. Canonical universe modules contain source declarations for language-visible primordial/native classes and native methods. A developer navigating `String`, `Number`, `Option`, `List`, `Method`, `Fiber`, or another built-in type should find a real `.ph` declaration with Phaldoc, source-level parameter names, source type annotations, and the appropriate attributes.
2. Native primitive Rust functions use `#[phalcom_native_macros::primitive(...)]`. The legacy manual registration table is eliminated as the authority for ordinary language primitives.
3. Native implementation selectors such as `_$byteAt(_)` are explicit in canonical source and carry `@internal @native`. Their Rust descriptors carry `visibility = internal`. Namespace, source attribute, runtime visibility, and descriptor visibility must agree.
4. The language supports declaration-only native members, for example `@native +(_ other: Number) -> Number`, without treating an absent body as an empty executable body.
5. The language continues to support reference-bodied native anchors. Their bodies parse and are available to tooling, but they never replace the Rust implementation.
6. Real user-facing wrappers around a primitive floor remain ordinary executable Phalcom. A method such as `String#size`, if implemented as `_$byteCount`, must not be marked `@native` merely because it calls a native hook.
7. Native source declarations and native descriptors are structurally cross-checked: owner, dispatch side, selector, member kind, parameter lanes, parameter types, return type, and internal visibility.
8. Native descriptor-only facts—Rust source provenance, ABI, effects, raises, return-flow metadata, intrinsic identity, trust, and lifecycle—are not duplicated into `.ph` purely for cross-checking. Tooling merges them into the source-backed semantic member.
9. `phalcom-native-surface` becomes generated/derived from Rust `#[primitive(...)]` declarations. It must cease to be a handwritten member registry.
10. Runtime-installed native methods participate in the existing `RuntimeTypingRegistry` / `MethodSemanticIndex` model without bloating `MethodObject` with complete source/type descriptors.
11. The LSP consumes the actual universe project/module source corpus and merges source `@native` declarations with generated native metadata. Go-to-definition should land in the actual universe source module, not a synthetic `core.ph` substitute.
12. The bootstrap source corpus is loaded through one shared source-provider mechanism. Runtime and tooling must not maintain independent lists of universe source text.
13. The completed migration enforces a required-anchor bijection for language-visible primitives, with an explicit descriptor-level escape hatch only for intentionally hidden VM primitives.
14. Universe source comments are cleaned so stable source reads like language/library code, not accumulated implementation-task archaeology.

---

## 3. Non-Goals

This work MUST NOT introduce type-based dispatch. Selector identity remains the existing structural selector plus dispatch side; type annotations remain semantic metadata.

This work MUST NOT attempt to prove that a reference-bodied `@native` implementation is behaviorally equivalent to its Rust primitive. Structural contract verification is mandatory; semantic equivalence testing may be added later through property tests or differential execution, but it is not required for this implementation.

This work MUST NOT make `@internal` an authorization capability. The existing privileged-core identity check remains the security/authority boundary. A user module cannot acquire implementation namespace authority by spelling an attribute.

This work MUST NOT generate canonical `.ph` source declarations from Rust. That would destroy the useful independence between the language-facing source contract and the machine implementation contract. Build generation is appropriate for *derived machine surfaces* such as LSP native metadata; source anchors remain deliberately authored and verified.

This work MUST NOT move derivable high-level library behavior into Rust simply because native source declarations are being exposed. ADR-0019's primitive-floor principle remains in force: native code is the irreducible floor; `.ph` owns derivable protocol and wrappers.

This work MUST NOT replace the existing class-object/metaclass model, change allocation layout semantics, or add a second runtime type system.

---

# Part I — Current Repository State

## 4. Repository Baseline and Existing Architecture

This specification is grounded against commit `edbced8914b924f95f4dbc83508c93ca77cd36ce`. That revision is important because the runtime typing registry and reflection surface were extended immediately before this design work; the implementation should build on those landed abstractions rather than reconstruct an older architecture.

### 4.1 Workspace topology

The workspace already separates the relevant responsibilities:

```text
phalcom-ast              source AST and parser
phalcom-common           common selector/range utilities
phalcom-type-syntax      VM-free type-syntax parser/model
phalcom-native-meta      VM-free native primitive metadata and UniverseKey
phalcom-native-macros    #[primitive(...)] proc macro
phalcom-native-surface   VM-free native member surface (currently handwritten)
phalcom-modules          module/project identities and builtin source provider
phalcom-semantic         VM-free semantic analysis
phalcom-type-meta        serialized/runtime-loadable semantic typing metadata
phalcom-core             VM, compiler, native installer, bootstrap, primitives
phalcom-lsp              VM-free language server
```

The dependency shape already permits the desired design. `phalcom-lsp` deliberately does not depend on `phalcom-core`; it depends on AST/common/modules/semantic/native-surface instead. `phalcom-core` depends on native-meta, native-macros, native-surface, modules, semantic, and type-meta. A new VM-free shared native declaration parser can therefore sit below both the proc-macro crate and the native-surface build script without creating a dependency on the VM.

### 4.2 New descriptor path already exists

`phalcom-native-meta/src/primitive.rs` defines the modern machine metadata model: `NativeDispatch`, `NativeVisibility`, `NativeStability`, `RaisesSpec`, `NativeEffect`, `EffectSpec`, `ReturnFlowSpec`, `NativeIntrinsicId`, `NativeTrust`, `PrimitiveAbi`, `PrimitiveKey`, `NativeSourceSpec`, and `PrimitiveSurfaceSpec`.

`phalcom-core/src/native/descriptor.rs` wraps a `PrimitiveSurfaceSpec` with the Rust ABI entry point and Rust source provenance in `PrimitiveDescriptor`. `phalcom-core/src/native/registry.rs` exposes the `linkme` distributed slice `PRIMITIVES` and checks duplicate `(owner, side, selector)` keys. `phalcom-core/src/native/install.rs` sorts the distributed descriptors and installs native `MethodObject`s with the declared visibility and access owner.

This is the architecture that should survive the migration.

### 4.3 The proc macro is rich, but not universal yet

`phalcom-native-macros/src/lib.rs` already parses owner, selector, parameter tuple, return type, complete callable type, raises, effects, dispatch side, visibility, stability/lifecycle, ABI, return flow, intrinsic identity, and trust. It validates canonical selectors and type syntax before emitting `PrimitiveSurfaceSpec` and `PrimitiveDescriptor` values.

However, repository search for `phalcom_native_macros::primitive` finds descriptor usage only in a subset of primitive modules, including `nil.rs`, `int.rs`, `class.rs`, `symbol.rs`, `string.rs`, `object.rs`, `system.rs`, `boolean.rs`, `selector.rs`, `attribute.rs`, `selector_pattern.rs`, and `number.rs`. The primitive directory contains many additional modules—`block`, `bytes`, `error`, `family`, `fiber`, `float`, `list`, `map`, `method`, collections/reflection helpers, and others—that are still reached through the older installer. The migration therefore has to be exhaustive and mechanically audited rather than assuming descriptor coverage.

### 4.4 The legacy native installer remains large and authoritative in practice

`phalcom-core/src/universe/primitives.rs` imports primitive functions throughout `phalcom-core/src/primitive` and installs them with macros such as:

```text
primitive!
primitive_internal!
primitive_rest!
primitive_shape!
primitive_static!
primitive_static_internal!
```

The table includes fundamental Object, Message, Behavior, Class, Number, Int, Float, String, Bool, Symbol, collections, callables, fibers, reflection, and other protocols. In `VM::new()`, bootstrap calls both:

```rust
Universe::install_primitives(&mut vm);
crate::native::install::install_registered_primitives(&mut vm)
    .expect("registered primitives must install cleanly");
```

The final implementation must remove this duality. Migration commits must not leave one primitive installed by both systems.

### 4.5 `phalcom-native-surface` is currently a third handwritten declaration

`phalcom-native-surface/src/lib.rs` contains its own `NativeMemberKind`, `NativeDispatch`, `NativeVisibility`, `NativeReturnShape`, `NativeMember`, `NativeClass`, `NATIVE_CLASSES`, and `NATIVE_MEMBERS`. It therefore duplicates metadata already modeled more richly in `phalcom-native-meta`.

The duplication has already drifted. For example, `Object#_$attributes`, `Object#_$attach(_)`, and `Object#_$freezeAttributes()` appear in the handwritten surface as public even though their selector spelling belongs to the `_$` implementation namespace and the legacy runtime installer installs them through `primitive_internal!`. This is exactly the failure mode the generated-surface work must eliminate.

### 4.6 Native visibility validation is not strict enough

The current `#[primitive(...)]` parser requires an explicit visibility for selectors beginning `_$`, and it requires `visibility = internal` declarations to use a `_$` selector. It nevertheless accepts explicit `visibility = public` for a `_$` selector. The target language rule is stronger: implementation selectors are always internal. Rust compilation should fail before bootstrap if a `_$` primitive is declared public.

### 4.7 `@native` exists, but its semantics are provisional

`phalcom-core/src/compiler/attributes.rs` already registers `NativeExpander`. It is a no-op expander whose real effect is implemented by `expand_class_attributes`: marked members are legality-checked and then removed from `ClassDef::members` before accessor derivation, variant expansion, normal member attribute expansion, and invariant weaving. This ordering is useful and should be preserved.

The current implementation has two limitations important to this project. First, `NativeExpander` is member-only; class-level `@native` is currently illegal. Second, the compiler does not use native registry knowledge when dropping a member. A user module can therefore spell `@native` and have a member disappear even though user-defined Rust extension binding has not been specified. The target implementation must restrict source-native anchors to privileged native/universe source until a future extension system explicitly opens that capability.

### 4.8 `@internal` does not yet exist as a builtin attribute

`phalcom-ast/src/ast.rs` currently defines 16 `BuiltinAttr` variants: `Construct`, `Constructor`, `Class`, `Get`, `Set`, `Data`, `Sealed`, `Variant`, `Invariant`, `Requires`, `Ensures`, `On`, `Native`, `Ignore`, `Private`, and `Protected`. `@internal` must be added as a first-class builtin.

The underlying namespace semantics already exist. `Compiler::compiling_privileged_core()` compares the compiling module handle with the actual core module handle. `Compiler::compile_class_impl` rejects source-authored `_$` methods/getters/setters and `__` fields outside that privileged module before attribute expansion. `member_visibility()` treats a `_$` selector as `MemberVisibility::Internal` independently of attributes. This is the correct authority model to preserve.

### 4.9 Declaration-only callable syntax does not yet exist

`MethodDef`, `GetterDef`, and `SetterDef` currently store `body: Vec<Statement>`. `Parser::parse_class_member` always calls `parse_method_block()` after the signature and optional return annotation. As a result, there is no AST distinction between “there is no source implementation” and “there is an executable empty body.” This must be fixed before canonical native declarations can omit braces.

### 4.10 The universe source is already a real project

The canonical source tree is not hypothetical. `phalcom-core/core/universe/project.toml` defines the universe project, and `src/package.ph` plus child `package.ph` files expose the object/scalar/callable/option/collections/errors/reflection/concurrency hierarchy. `phalcom-modules/src/builtin.rs` already defines `BuiltinProjectSourceProvider`, returns stable `phalcom://universe/...` source identities, embeds the universe source, and can return parsed `ParsedModuleUnit`s. `phalcom-modules/src/builtin_interface.rs` caches parsed units and source-derived interfaces.

This existing source provider should become the common runtime/tooling source seam. A new `phalcom-universe-source` crate is unnecessary.

### 4.11 Runtime bootstrap still maintains its own source list

Despite the provider above, `VM::run_universe_modules` in `phalcom-core/src/vm/bootstrap.rs` has a separate `static SOURCES: &[(&str, &str)]` with direct `include_str!` calls and a manually ordered list. Each source is currently compiled into the same core module handle so it can complete bootstrapped class stubs and use privileged implementation namespaces.

The source provider and the bootstrap list therefore represent the same physical corpus in two different ways. Runtime should load through the provider while preserving the current privileged compilation semantics until the runtime's built-in project initialization is fully unified with ordinary module execution.

### 4.12 LSP still uses the old single-core model

`phalcom-lsp/src/semantic/core_source.rs` embeds `../../../phalcom-core/core/core.ph` as `BUNDLED_CORE_SOURCE`. `build_core_surface()` builds a source surface and then adds `NATIVE_CLASSES`/`NATIVE_MEMBERS`. Critically, it performs:

```rust
if class.member(native.selector, side).is_some() {
    continue;
}
```

This models source and native declarations as alternatives. The target model is a merge: a source `@native` declaration supplies canonical source identity, documentation, names, and written types; generated native metadata enriches that same semantic member.

### 4.13 The LSP has a concrete implementation-field classification bug

`phalcom-lsp/src/semantic/surface.rs` currently marks a field as implementation storage when `field.name.starts_with("_$")`. The language namespace is `__field` for implementation storage and `_$selector` for implementation selectors. The field check must be corrected to `starts_with("__")` as part of the `@internal` work.

### 4.14 Runtime typing already has the correct side-table shape

`phalcom-core/src/typing/registry.rs` owns `RuntimeTypingRegistry`, including a `MethodSemanticIndex`. `phalcom-core/src/typing/side_table.rs` maps live method-object `ObjRef`s to `RuntimeCallableRef { pool, record }`. `phalcom-type-meta/src/declaration.rs` already distinguishes `PublishedTypeAuthority::TrustedNative` and defines `CallableSemanticRecord`/`CallableParameterRecord`.

The native implementation must use this infrastructure. It should not add full native descriptors to `MethodObject`.

---

# Part II — Normative Semantic Decisions

## 5. Authority Model

There MUST NOT be several peers that can silently disagree about the same fact. Instead, each category of information has one authority and may be cross-checked against independently authored representations.

| Fact | Authority | Verified/merged representation |
|---|---|---|
| Rust implementation function | Rust function | `NativeSourceSpec` provenance |
| Native primitive identity | Rust `#[primitive]` | source `@native` anchor |
| Owner | Rust descriptor | enclosing source native class |
| Dispatch side | Rust descriptor | source `@class` placement |
| Selector | structural identity in both | source/native verifier |
| Native parameter/return contract | Rust descriptor | source type annotations |
| Internal runtime visibility | `_$` namespace + descriptor | source `@internal` assertion |
| Effects / raises / ABI / flow / intrinsic / trust | Rust descriptor | merged into LSP/reflection |
| Human parameter names | canonical `.ph` | semantic member metadata |
| Phaldoc | canonical `.ph` | LSP/docs |
| Reference implementation body | canonical `.ph` | tooling only, not installed |
| Real wrapper implementation | canonical `.ph` | compiled bytecode |
| Primitive runtime installation | descriptor registry | verified source anchor |
| VM-free native surface | generated | never handwritten authority |

The deliberate duplication is therefore narrow and useful: Rust and `.ph` independently describe the same native callable at different abstraction levels, and bootstrap proves structural agreement.

## 6. Native Source Member Forms

### 6.1 Declaration-only native primitive

Use declaration-only form for irreducible primitive behavior:

```phalcom
@internal
@native
_$byteAt(_ index: Int) -> Option<Int>
```

or:

```phalcom
@native
+(_ other: Number) -> Number
```

There is no `{}`. This declaration creates no executable body.

### 6.2 Reference-bodied native primitive

Use a body only when it is a truthful and useful source-level explanation:

```phalcom
@native
toString -> String {
  "[" + joined(", ") + "]"
}
```

The body is parsed and indexed, and may later be semantically checked or differentially tested, but it is never installed by ordinary compilation.

### 6.3 Real executable source wrapper

A real wrapper has no `@native`:

```phalcom
size -> Int {
  _$byteCount
}
```

The wrapper emits bytecode and remains dynamically overridable according to ordinary Phalcom semantics.

### 6.4 Empty body is not declaration-only

These forms are semantically distinct:

```phalcom
@native
foo() -> Unit
```

```phalcom
foo() -> Unit {}
```

The first has no source implementation. The second has an executable empty body whose ordinary fall-through result follows current method-body semantics.

## 7. `@internal`

`@internal` is an explicit source assertion, not the source of authority. In canonical authored universe source:

```text
_$selector  requires @internal
__field     requires @internal
```

The inverse also holds:

```text
@internal ordinarySelector  => error
@internal _sourceField       => error
```

Outside privileged core/universe compilation, the existing implementation namespace reservation remains decisive. `@internal` cannot bypass `InternalNamespaceReserved`.

`@private` and `@protected` must not combine with `@internal`, because implementation visibility is a distinct visibility category rather than a private/protected modifier.

Compiler-synthesized internal selectors are exempt from the source-attribute requirement because no author wrote them. The existing `allow_synthetic_internal` route in `compile_class_impl` is the correct seam to preserve.

## 8. `@native`

`@native` is a declaration assertion. It does not name or install a Rust function. Native matching derives identity structurally from source:

```text
(owner UniverseKey, dispatch side, canonical selector)
```

`@native` is initially legal only in privileged canonical native source. User-defined FFI/native plugin registration is a separate future capability.

Member-level `@native` means the native runtime owns the implementation. Class-level `@native` means the primordial class identity already exists and the source declaration completes/presents that identity.

## 9. `@native class` and Stub Completion

The current compiler already contains a core-only “stub completion” path: a bootstrapped class present in `vm.classes` but not yet present in `field_layouts` may be completed by source in the core module. `@native class` should formalize and verify that path; it should not introduce a separate class-creation mechanism.

For a privileged source class carrying `@native`, the compiler/bootstrap must assert:

1. `UniverseKey::from_name(class.name)` resolves.
2. The corresponding primordial `ClassId` already exists.
3. The compiler is completing that exact class identity, not allocating a fresh class.
4. Any written superclass clause is compatible with the bootstrapped superclass.
5. The source class may contain ordinary real `.ph` members and `@native` anchor members.

After migration, completing a known primordial class in canonical universe source without `@native` should become a source-integrity error. Source-defined core classes that are not precreated by Rust remain ordinary source classes and must not carry `@native`.

### 9.1 Special global binding preservation (`None` and similar identities)

The current `option.ph` deliberately avoids `class None {}` because class lowering emits a global definition and would overwrite the public `None` immediate with the hidden `None` class object. `@native class` must solve this *generally*, not by hard-coding parser syntax for `None`.

For native stub completion, the compiler must distinguish class identity from public global binding. If the core class registry contains the primordial class but the existing global is not `Value::obj(existing_class)`, native completion MUST preserve the existing global rather than emitting a rebinding. This allows a future canonical declaration such as `@native class None { ... }` to present the hidden class row without destroying the language's immediate `None` value. The same rule also handles a native class intentionally hidden from globals.

The compiler should assert this only for `@native` stub completion; ordinary source class declarations continue to define their global normally.

## 10. `@class`

`@class` already implements class-side placement and remains orthogonal to native/internal status:

```phalcom
@class
@internal
@native
_$allocate(_ size: Int) -> Object
```

The native-source verifier must derive the side from the unexpanded source attributes because parser-created method nodes still have `is_static = false` until `ClassExpander` runs. The verifier therefore cannot rely only on post-expansion `is_static` state.

## 11. Rust `#[primitive(...)]`

The Rust attribute is the machine declaration of a native primitive. There is no Phalcom-language `@primitive` in this design. Canonical Phalcom source uses `@native`; Rust uses `#[phalcom_native_macros::primitive(...)]`.

Every language primitive must ultimately use the Rust attribute. The completed migration must not rely on `Universe::install_primitives` for ordinary language callables.

---

# Part III — Target Architecture

## 12. End-State Data Flow

```text
Rust primitive module
  #[primitive(...)]
         │
         ├─────────────── compile-time proc macro ───────────────┐
         │                                                       │
         │                                              PrimitiveDescriptor
         │                                                       │
         │                                              linkme::PRIMITIVES
         │                                                       │
         └──── build-time source scanner ──> generated VM-free surface
                                                                 │
Canonical universe .ph                                          │
  @native / @internal / @class                                   │
         │                                                       │
         └──────────────> NativeSourceIndex <────────────────────┘
                                 │
                                 ▼
                         NativeContractVerifier
                                 │
               ┌─────────────────┴─────────────────┐
               ▼                                   ▼
      Runtime semantic metadata             LSP semantic merge
               │                                   │
               ▼                                   ▼
       descriptor installer                  source-backed hover/
               │                             completion/navigation
               ▼
         MethodObject
               │
               ▼
       MethodSemanticIndex
```

The architecture intentionally does **not** contain:

```text
handwritten NATIVE_MEMBERS
handwritten native LSP return-shape table
manual per-primitive Universe::install_primitives entries
another independent universe source include list in VM bootstrap
```

## 13. New Shared Native Declaration Layer

Create a new workspace crate:

```text
phalcom-native-decl/
  Cargo.toml
  src/lib.rs
  src/parse.rs
  src/model.rs
  src/validate.rs
  src/emit.rs
```

Purpose: own the codegen-neutral parser and validator for the Rust `#[primitive(...)]` attribute grammar.

Suggested dependency graph:

```text
phalcom-native-meta
        ▲
        │
phalcom-native-decl ──> phalcom-common
        │              phalcom-type-syntax
        │              syn / proc-macro2 / quote
        │
   ┌────┴────────────┐
   ▼                 ▼
phalcom-native-   phalcom-native-surface
macros            build.rs
```

`phalcom-native-decl` MUST NOT depend on `phalcom-core` or the VM.

The shared parser should produce owned data, for example:

```rust
pub struct PrimitiveDecl {
    pub owner: UniverseKey,
    pub selector: String,
    pub params: OwnedParameterTuple,
    pub returns: OwnedTypeExpr,
    pub callable: OwnedCallableType,
    pub raises: OwnedRaises,
    pub effects: OwnedEffects,
    pub side: NativeDispatch,
    pub visibility: NativeVisibility,
    pub stability: NativeStability,
    pub since: Option<String>,
    pub deprecated_since: Option<String>,
    pub replacement: Option<String>,
    pub abi: PrimitiveAbi,
    pub flow: ReturnFlowSpec,
    pub intrinsic: Option<NativeIntrinsicId>,
    pub trust: NativeTrust,
    pub anchor: NativeAnchorPolicy,
}
```

The proc macro uses this model to emit `'static` native-meta structures and the function-pointer-bearing `PrimitiveDescriptor`. The build script uses the same model to emit VM-free static surface data. This prevents the generator from becoming another parser whose interpretation can drift from the proc macro.

## 14. Native Anchor Policy

Add to `phalcom-native-meta/src/primitive.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum NativeAnchorPolicy {
    Required,
    Hidden,
}
```

Add:

```rust
pub anchor: NativeAnchorPolicy,
```

to `PrimitiveSurfaceSpec`.

The Rust attribute gains:

```text
anchor = required
anchor = hidden
```

with `required` as the semantic default.

`Hidden` is for primitives that intentionally have no canonical language-facing universe declaration. It must be rare and explicit. Do not maintain a separate global `EXEMPT_NATIVE_KEYS` list.

### 14.1 Migration gate strategy

The new field may default to `Required` immediately, but reverse-completeness enforcement must be staged. During early migration the verifier first guarantees “every source anchor resolves to a descriptor.” Once all descriptor-backed primitives have source anchors and every legacy primitive has been migrated, enable the reverse gate “every required descriptor has exactly one source anchor.” This avoids introducing a temporary third `Legacy` policy that would itself need later removal.

---

# Part IV — AST, Parser, and Attribute Infrastructure

## 15. Add `MemberBody`

### Files

Modify:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
```

Potential dependent updates will occur in:

```text
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-lsp/src/semantic/surface.rs
phalcom-semantic/**
```

### AST shape

Replace `Vec<Statement>` bodies on `MethodDef`, `GetterDef`, and `SetterDef` with an explicit enum:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum MemberBody {
    Declaration,
    Block(Vec<Statement>),
}

impl MemberBody {
    pub fn is_declaration(&self) -> bool { ... }
    pub fn statements(&self) -> Option<&[Statement]> { ... }
    pub fn statements_mut(&mut self) -> Option<&mut Vec<Statement>> { ... }
}
```

Do not use an empty vector as the declaration sentinel. `foo() {}` must remain executable empty source.

Initially, index members can retain their existing body representation because `@native` currently has no index-member contract in `phalcom-native-surface` and no agreed native index kind. Generalize index declarations later if a real primitive requires them.

### Parser behavior

Refactor `Parser::parse_class_member` so the signature parser ends before body selection. After a method/getter/setter signature and optional return annotation:

- `{` parses `MemberBody::Block` through the current block parser;
- newline, closing `}`, or EOF in the syntactically valid class-member boundary parses `MemberBody::Declaration`;
- any other token reports an expected-body/member-terminator diagnostic.

The parser MUST NOT consult `@native` to decide whether declaration syntax is grammatically allowed. Parsing records syntax; compilation/semantic validation decides whether a declaration-only member is legal in context. This leaves the syntax reusable for a future protocol/abstract declaration facility without another grammar rewrite.

### Compiler legality

After native-anchor stripping, no `MemberBody::Declaration` may reach executable lowering. Add a defensive compiler error, for example:

```text
member.declaration_requires_implementation
```

with guidance that declaration-only members currently require `@native` in privileged universe source.

## 16. Add `BuiltinAttr::Internal`

Modify `phalcom-ast/src/ast.rs`:

```rust
pub enum BuiltinAttr {
    ...
    Internal,
}
```

Update `BuiltinAttr::name()` and `BuiltinAttr::parse()`.

Modify the fixed-size registry in `phalcom-core/src/compiler/attributes.rs` from 16 slots to the new enum count. Prefer eliminating the magic literal by defining a compile-time count next to `BuiltinAttr` or by changing the registry representation to a `HashMap<BuiltinAttr, ...>` only if the performance/complexity tradeoff is justified. The minimal change is a named `BUILTIN_ATTR_COUNT` constant.

Add `InternalExpander` with legal targets:

```text
Method
Getter
Setter
Field
```

The expander itself is a no-op. `@internal` semantics are declaration-integrity semantics and must be checked before native members are removed.

Add `internal` to `COMPILER_ONLY_ATTRS` in `phalcom-core/src/compiler/lib/class_decl.rs` so the compiler does not attempt to instantiate a user-visible runtime `Attribute` object merely because the source assertion is retained in the AST.

## 17. Add Attribute Argument Policies

The current attribute framework permits argument-bearing spellings such as `@native(foo)` because expanders receive `args` but do not centrally validate arity. This update should establish a reusable policy rather than adding one-off checks.

Recommended interface:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeArgPolicy {
    Any,
    None,
    Exact(usize),
    AtLeast(usize),
}

pub trait AttributeExpander {
    fn legal_targets(&self) -> &'static [Target];
    fn argument_policy(&self) -> AttributeArgPolicy {
        AttributeArgPolicy::Any
    }
    fn expand(
        &self,
        ctx: &mut ExpandCtx,
        member: &mut ClassMember,
        args: &[Expr],
    ) -> Result<(), CompilerError>;
}
```

At minimum, the following must use `None`:

```text
@native
@internal
@class
@private
@protected
@constructor
```

Do not blindly assign `None` to contract or derive attributes whose current syntax accepts expressions. Existing behavior outside this change must remain stable.

Validation must happen before a subtractive attribute can drop its target.

## 18. Early Declaration-Integrity Validation

Create a focused helper, preferably in a new file:

```text
phalcom-core/src/compiler/declaration_integrity.rs
```

or a tightly scoped private section of `class_decl.rs` if maintainers prefer fewer modules.

Recommended API:

```rust
pub(crate) struct DeclarationAuthority {
    pub privileged_core: bool,
    pub allow_synthetic_internal: bool,
}

pub(crate) fn validate_class_declaration_integrity(
    class: &ClassDef,
    authority: DeclarationAuthority,
) -> Result<(), CompilerError>;
```

This function runs at the beginning of `compile_class_impl`, before `expand_class_attributes` and therefore before `@native` stripping.

It must enforce:

```text
ordinary module + _$selector              => existing reserved-namespace error
ordinary module + __field                 => existing reserved-namespace error
ordinary module + @internal               => no privilege grant; fail when namespace used
ordinary module + @native                 => attr.native_requires_privileged_core

privileged authored _$selector + @internal  => valid
privileged authored __field + @internal     => valid
privileged authored _$selector no @internal => attr.internal_required
privileged authored __field no @internal    => attr.internal_required
@internal + ordinary selector               => attr.internal_requires_implementation_namespace
@internal + _sourceField                    => attr.internal_requires_implementation_namespace
@internal + @private/@protected             => member.visibility_conflict
```

Compiler-generated internal declarations pass `allow_synthetic_internal = true` and must not be forced to synthesize source attributes.

## 19. Strengthen `@native` Privilege

Before the existing legality-check-then-drop pass, reject `@native` in nonprivileged compilation. This closes a current dangerous behavior where an ordinary user method can be silently removed by spelling a compiler-recognized attribute.

Suggested diagnostic:

```text
attr.native_requires_privileged_core
```

or a repository-standard structured `CompilerError` variant carrying the source range.

A future native-extension/plugin facility may introduce another authority token. Do not pre-authorize it in this change.

## 20. Class-Level `@native`

Change `NativeExpander::legal_targets()` to include `Target::Class`, but do not implement class semantics inside `AttributeExpander::expand`; class native identity requires whole-class/bootstrap context.

Before normal class stub completion, detect class-level `@native` and validate:

```rust
let has_native_class_attr = class_def.attributes.iter().any(...BuiltinAttr::Native...);
```

Rules:

- `@native class` outside privileged core: error.
- `@native class` with no corresponding primordial `UniverseKey`/`ClassId`: `native.class_identity_mismatch`.
- primordial stub completion without `@native` in canonical universe: initially warning/test failure during migration, finally hard error.
- `@native class` must never allocate a fresh user class.
- class body remains otherwise normal; only its native members are stripped.

## 21. Centralize Source Selector Projection

The source/native verifier, compiler duplicate-selector logic, and LSP must not maintain subtly different selector construction rules.

Today LSP has `phalcom-lsp/src/selectors.rs::selector_from_member`, while compiler code builds selectors through `encode_selector`, rest-specific helpers, and `SignatureKind`. Introduce a VM-free AST selector projection in `phalcom-ast`, for example:

```text
phalcom-ast/src/selector.rs
```

API:

```rust
pub fn selector_from_member(member: &ClassMember) -> Result<Selector, SelectorProjectionError>;
```

It should cover method/getter/setter and the existing index forms using `phalcom-common::selector::Selector`. Rest-shape construction must match the compiler's canonical selector encoding.

Then:

- LSP delegates to `phalcom_ast::selector_from_member`;
- native source indexing delegates to the same function;
- compiler may gradually reuse it in duplicate-selector validation and installation paths.

This is a semantic deduplication, not a cosmetic refactor.

---

# Part V — Rust Primitive Declaration and Generation

## 22. Extract `PrimitiveAttrArgs` Parsing

Move codegen-neutral parsing and validation out of `phalcom-native-macros/src/lib.rs` into `phalcom-native-decl`.

The proc macro should become responsible primarily for:

1. obtaining the Rust function item;
2. calling the shared declaration parser;
3. validating the function signature against ABI;
4. emitting static metadata values;
5. emitting the `PrimitiveDescriptor` into `PRIMITIVES`;
6. stamping `NativeSourceSpec` with module path, Rust name, file, and line.

The shared crate should own:

- keyword parsing;
- duplicate option detection;
- selector canonicalization;
- parameter-lane validation;
- type-syntax parsing;
- `types` consistency checks;
- visibility rules;
- lifecycle rules;
- anchor policy parsing;
- effect/flow/intrinsic/trust parsing.

## 23. Enforce Implementation Visibility in the Shared Validator

Replace the current “`_$` requires explicit public or internal” rule with the final invariant:

```text
selector starts with "_$"
    => visibility must be explicitly internal

visibility = internal
    => selector must start with "_$"
```

Invalid:

```rust
#[primitive(
    Object,
    "_$attach(_)",
    visibility = public,
    ...
)]
```

This must fail at Rust compile time.

## 24. Generate `phalcom-native-surface`

### Files

Modify/create:

```text
phalcom-native-surface/Cargo.toml
phalcom-native-surface/build.rs             # new
phalcom-native-surface/src/lib.rs
```

Add normal dependency:

```text
phalcom-native-meta
```

Add build dependencies:

```text
phalcom-native-decl
syn
quote/proc-macro2 as required by shared emitter
```

The build script recursively scans `../phalcom-core/src/primitive/**/*.rs`, parses each Rust file with `syn::parse_file`, finds attributes whose path terminates in `primitive`, and passes the attribute token stream to `phalcom-native-decl`.

For every scanned file, emit:

```text
cargo:rerun-if-changed=<path>
```

Emit generated Rust into `OUT_DIR`, for example:

```text
native_surface.rs
```

`src/lib.rs` becomes a small stable API wrapper:

```rust
pub use phalcom_native_meta::{...};
include!(concat!(env!("OUT_DIR"), "/native_surface.rs"));
```

The generated surface should expose rich primitive metadata, preferably:

```rust
pub static NATIVE_PRIMITIVE_SURFACES: &[PrimitiveSurfaceSpec] = ...;
```

or an equivalent generated wrapper that references the same native-meta enums and symbolic type structures.

Do **not** preserve a parallel lossy `NativeReturnShape` table as the primary semantic contract. Return shape can be derived from the actual symbolic return type/flow metadata where useful.

### 24.1 Generated class information

Do not regenerate a second hard-coded `NATIVE_CLASSES` hierarchy from primitive ownership. Canonical class declarations belong in universe source; primordial binding identity belongs in `phalcom-native-meta::UNIVERSE_BINDINGS` and `UniverseKey`. During transition, `NATIVE_CLASSES` may remain as compatibility fallback, but the deletion gate requires the LSP to get class source/hierarchy from the universe source/binding model instead.

### 24.2 Generator parity test

After descriptor migration is complete, add an integration test comparing generated surface keys to runtime distributed descriptor keys:

```text
set(NATIVE_PRIMITIVE_SURFACES.key)
    ==
set(PRIMITIVES.surface.key)
```

This is a high-value test: the two sides are produced from the same Rust attributes by two independent consumption paths, so it validates the build scanner and proc macro agree.

---

# Part VI — Native Source Index and Contract Verification

## 25. New `phalcom-core::native` Source Modules

Create:

```text
phalcom-core/src/native/source.rs
phalcom-core/src/native/verify.rs
phalcom-core/src/native/typing.rs       # if native metadata lowering is kept with native subsystem
```

Export through `phalcom-core/src/native/mod.rs`.

### 25.1 Owned source key

`PrimitiveKey` contains a `&'static str` selector and is ideal for generated native metadata, not parsed source. Define an owned source key:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeAnchorKey {
    pub owner: UniverseKey,
    pub side: NativeDispatch,
    pub selector: String,
}
```

Provide comparison/conversion helpers against `PrimitiveKey` without leaking Rust function identity.

### 25.2 Anchor record

```rust
pub enum NativeReferenceBodyKind {
    Declaration,
    ReferenceBody,
}

pub struct NativeMemberAnchor {
    pub key: NativeAnchorKey,
    pub module: phalcom_modules::ModuleId,
    pub source: phalcom_modules::SourceLocation,
    pub class_range: SourceRange,
    pub member_range: SourceRange,
    pub name_range: SourceRange,
    pub kind: NativeMemberKind,
    pub internal: bool,
    pub params: Vec<SourceNativeParameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body_kind: NativeReferenceBodyKind,
}
```

`NativeMemberKind` can be a new VM-free enum in `phalcom-native-meta` or an AST-local mapping; avoid preserving the duplicate enum only because `phalcom-native-surface` currently has it.

### 25.3 Native class anchor

```rust
pub struct NativeClassAnchor {
    pub key: UniverseKey,
    pub module: ModuleId,
    pub source: SourceLocation,
    pub range: SourceRange,
    pub name_range: SourceRange,
    pub written_superclass: Option<StaticSymbolRef>,
}
```

### 25.4 Source index

```rust
pub struct NativeSourceIndex {
    pub classes: BTreeMap<UniverseKey, NativeClassAnchor>,
    pub members: BTreeMap<NativeAnchorKey, NativeMemberAnchor>,
}
```

Insertion must detect duplicates immediately and retain both locations for diagnostics.

## 26. Build the Index from Parsed Builtin Universe Units

Use the existing:

```rust
BuiltinProjectSourceProvider::new(BuiltinProject::Universe)
```

and its `load_parsed` API. Expose a stable enumeration helper from `phalcom-modules/src/builtin.rs`, such as:

```rust
pub fn nodes(&self) -> &'static [BuiltinNodeSpec];
pub fn module_ids(&self) -> impl Iterator<Item = ModuleId>;
```

Do not make `phalcom-core` copy `UNIVERSE_NODES`.

The index builder walks every class statement in every parsed universe module. For each class:

- resolve its name through `UniverseKey::from_name` if it carries `@native`;
- record class anchor metadata;
- walk its members;
- find member `@native` attributes;
- derive dispatch side from `@class`/constructor source metadata before expansion;
- derive canonical selector through the shared AST selector projection;
- classify body as declaration/reference;
- capture source type annotations and parameter lane metadata;
- validate/source-record `@internal` state.

The index builder must not execute the reference body.

## 27. Type Normalization

Do not compare source and Rust type strings.

Source has `phalcom_ast::TypeAnnotationExpr`; native metadata has `phalcom_native_meta::TypeExprSpec` / `ParameterTupleSpec` / `CallableTypeSpec`. Add a common *owned normalized comparison form* in a VM-free layer that can depend on both AST and native-meta. `phalcom-semantic` is the best existing home because it already depends on `phalcom-ast`, `phalcom-modules`, `phalcom-native-meta`, and `phalcom-type-meta`.

Create, for example:

```text
phalcom-semantic/src/native_contract.rs
```

with:

```rust
pub enum NormalizedContractType {
    Unknown,
    Never,
    SelfType,
    Nominal(UniverseKey),
    Parameter(Box<str>),
    Applied {
        origin: Box<NormalizedContractType>,
        args: Box<[NormalizedContractType]>,
    },
    Union(Box<[NormalizedContractType]>),
    Tuple(...),
    Callable(...),
}
```

Provide:

```rust
pub fn normalize_source_type(...);
pub fn normalize_native_type(...);
pub fn normalize_source_callable(...);
pub fn normalize_native_callable(...);
```

Nominal source references inside canonical universe source must resolve using the universe semantic namespace, not naïve text comparison. Type aliases, generics, and future qualified names should lower through the same stable semantic type model used by the broader type checker when possible.

### 27.1 Callable-type structural gap

`TypeAnnotationExpr` supports callable syntax while `TypeExprSpec` currently stores callable information separately in `CallableTypeSpec` and does not expose a nested `Callable` `TypeExprSpec` variant. During implementation, audit primitives accepting callable-typed arguments. If nested callable types are required in `params`/`returns`, extend `TypeExprSpec` with a callable node rather than degrading source types to strings or `Unknown`.

## 28. Verification Algorithm

Add:

```rust
pub struct NativeVerificationReport { ... }

pub fn verify_native_contracts(
    descriptors: &[&'static PrimitiveDescriptor],
    source: &NativeSourceIndex,
    mode: NativeVerificationMode,
) -> Result<VerifiedNativeUniverse, NativeVerificationError>;
```

Suggested modes during migration:

```rust
pub enum NativeVerificationMode {
    AnchorsMustResolve,
    Complete,
}
```

This is a *bootstrap/test mode*, not persisted primitive metadata. Remove the transitional mode once migration is complete if it has no continuing diagnostic use.

For each source anchor:

1. Find exact descriptor by owner + side + selector.
2. Verify member category (method/getter/setter).
3. Verify parameter lane structure, including labels/rest capture.
4. Normalize and compare each parameter type.
5. Normalize and compare return type.
6. Verify `_$`/`@internal`/descriptor `Internal` agreement.
7. Record the matched descriptor and source anchor as one verified callable.

In `Complete` mode, also iterate descriptors with `anchor == Required` and require exactly one source anchor.

The verifier should return an indexed result that later installation/LSP/typing code can consume without repeating matching work.

## 29. Diagnostics

Prefer structured error variants with source span and native source provenance rather than free-form strings. Required diagnostic concepts:

```text
native.duplicate_anchor
native.orphan_anchor
native.missing_anchor
native.class_identity_mismatch
native.kind_mismatch
native.side_mismatch
native.selector_mismatch
native.visibility_mismatch
native.parameter_shape_mismatch
native.parameter_type_mismatch
native.return_type_mismatch
```

A mismatch diagnostic should include both representations. Example:

```text
native.parameter_type_mismatch:
String#_$byteAt(_), argument 0

source:  Int
native:  Number

source declaration:
  phalcom://universe/scalar/string:<line>:<col>

native implementation:
  phalcom-core/src/primitive/string.rs:<line>
```

`NativeSourceSpec` already carries Rust file and line for runtime descriptors.

---

# Part VII — Bootstrap Redesign

## 30. Introduce a Fallible Bootstrap Entry Point

`VM::new() -> Self` currently uses `expect`/assertions throughout bootstrap. To test native consistency failures cleanly, introduce:

```rust
pub fn try_new() -> PhResult<Self>
```

or a dedicated:

```rust
pub fn try_bootstrap() -> Result<Self, BootstrapError>
```

with:

```rust
pub fn new() -> Self {
    Self::try_new().expect("VM bootstrap must succeed")
}
```

Compatibility callers retain the current infallible API while tests and tools can observe structured bootstrap failures.

If `PhError` is too runtime-oriented for pre-execution native/source failures, define `BootstrapError` and convert it at the outer boundary.

## 31. New Bootstrap Order

Refactor `phalcom-core/src/vm/bootstrap.rs` into named private phases. Avoid one giant `VM::new()` sequence as additional validation grows.

Recommended shape:

```rust
fn bootstrap_kernel(&mut self) -> Result<(), BootstrapError>;
fn load_universe_source(&self) -> Result<UniverseSourceCorpus, BootstrapError>;
fn verify_native_universe(...) -> Result<VerifiedNativeUniverse, BootstrapError>;
fn install_native_universe(...) -> Result<(), BootstrapError>;
fn execute_universe_source(...) -> Result<(), BootstrapError>;
fn finalize_semantic_roots(...) -> Result<(), BootstrapError>;
fn verify_runtime_invariants(...) -> Result<(), BootstrapError>;
```

The ordering is normative:

### Phase A — primordial identities

Build heap, class/metaclass tower, core module, canonical builtin universe package identity, fixed native layouts that are needed before source compilation, and special immediate/global roots.

### Phase B — native registry validation

Validate duplicate `PrimitiveKey`s and descriptor-level invariants. This includes the strengthened implementation-visibility rule even though the proc macro should already enforce it.

### Phase C — source corpus parse

Load every canonical universe source unit through `BuiltinProjectSourceProvider`; fail on parse errors.

### Phase D — source/native preflight

Build `NativeSourceIndex`, verify class anchors and member anchors, and perform completeness checks appropriate to the migration stage.

No primitive should be installed yet. No universe source body should have executed yet.

### Phase E — native typing metadata

Build/register the native semantic metadata pool and retain callable record IDs indexed by verified primitive key.

### Phase F — primitive installation

Install all descriptor-backed primitives. For each installed `MethodObject`, register the matching `RuntimeCallableRef` in `MethodSemanticIndex`.

### Phase G — base-name finalization

Retain `finalize_all_core_base_names()` if it is still required for kernel classes with native-only protocol before source execution. Re-evaluate after source/native class migration, but do not remove it merely as cleanup.

### Phase H — source compilation/execution

Compile canonical universe modules. `@native` declarations disappear before executable member lowering; real source wrappers and algorithms compile normally.

### Phase I — late semantic roots and runtime invariants

Resolve source-exported semantic roots (`unsupported`, `ellipsis`, `Ordering`, etc.), finalize prelude names, verify the `None` global, verify class-tower/kernel invariants, and perform existing post-bootstrap checks.

## 32. Reuse `BuiltinProjectSourceProvider`; Remove Runtime Source Duplication

`VM::run_universe_modules` must stop embedding its own source strings. Replace its direct `include_str!` access with provider-loaded source/parsed units.

During the first implementation, preserve the current execution order if changing module initialization semantics would introduce unrelated risk. Move that order into one canonical place in `phalcom-modules`, or derive it from existing builtin module/interface dependencies. The VM should consume module IDs, not maintain path→source text itself.

A useful transitional API in `phalcom-modules`:

```rust
pub fn universe_bootstrap_modules() -> &'static [BuiltinModuleRef];
```

where each ref can be resolved through the provider. Longer term, ordinary package/linker initialization should own topological order, but that larger module-runtime convergence should not block native source verification.

## 33. Do Not Parse Universe Source Twice Unnecessarily

`BuiltinProjectSourceProvider::load_parsed` already caches `ParsedModuleUnit`. Preflight should retain those parsed units. Runtime compilation currently calls `compile_closure_as_with_bindings`, which reparses source text. Add an AST-taking compiler entry point:

```rust
pub fn compile_program_as_with_bindings(
    &mut self,
    module: ObjRef,
    source: Arc<String>,
    program: Program,
    kind: UnitKind,
    bindings: Option<CompileBindings>,
) -> PhResult<ObjRef>;
```

or equivalent. `compile_closure_as_with_bindings` remains the text API and delegates after parsing. Bootstrap can feed the already parsed `Program` into the AST entry point.

This both removes redundant work and ensures the exact AST verified during preflight is the AST later subjected to native-member dropping.

## 34. Native Class Completion and Global Binding

Modify the class lowering path in `phalcom-core/src/compiler/lib/class_decl.rs` around the existing stub-completion branch. For `@native class` completion:

- resolve `name_key` to the existing primordial `ClassId`;
- reject fresh allocation;
- preserve established superclass;
- apply source-declared field/layout rules only where compatible with the native class's fixed representation contract;
- when emitting the class's name/global result, do not overwrite a preexisting non-class binding (e.g. immediate `None`);
- still allow the source declaration to attach real methods and finalize class indexes.

Native classes with Rust-fixed slots need explicit compatibility checking. A source declaration must not silently infer fields that conflict with fixed slots stamped in bootstrap. Where source wants to document a Rust-owned implementation slot, use explicit `__` implementation-field declarations only after layout semantics for those declarations are defined. Do not convert ordinary source fields such as `_message` to `__message` mechanically.

---

# Part VIII — Runtime Typing Integration

## 35. Build Native Callable Metadata from Verified Contracts

`phalcom-type-meta::CallableSemanticRecord` already supports parameter local names, external labels, rest mode, source spans, return types, and `PublishedTypeAuthority`. Use the verified source/native pair to build records where:

```text
parameter local_name     comes from source anchor
external label           comes from selector/source parameter
parameter type           verified source/native type
return type              verified source/native type
source span              canonical .ph declaration
PublishedTypeAuthority   TrustedNative
```

If a primitive is `anchor = hidden` and has no source parameter names, synthesize stable internal parameter names only for metadata mechanics and leave source span absent. Hidden primitives are not user API documentation.

## 36. Register a Native Metadata Pool

Add a helper around `RuntimeTypingRegistry` rather than assuming pool ID `0`. The current program materializer constructs a loaded pool using `MetadataPoolId(0)` and then calls `register_pool`; native bootstrap metadata introduces another pool and therefore makes fixed IDs unsafe.

Recommended API evolution:

```rust
impl RuntimeTypingRegistry {
    pub fn next_pool_id(&self) -> MetadataPoolId;

    pub fn load_and_register_bundle(
        &mut self,
        bundle: Arc<SemanticMetadataBundle>,
        limits: &ValidationLimits,
    ) -> Result<MetadataPoolId, ...>;
}
```

or an equivalent reservation API that ensures the loader and registry agree on the assigned ID.

Update `phalcom-core/src/modules/materialize.rs` Phase 8 to use the same helper, eliminating implicit assumptions that ordinary program metadata is always pool zero.

## 37. Index Installed Native `MethodObject`s

Change native installation so the created method object is available to typing registration. Options:

```rust
fn install_one(...) -> Result<ObjRef, NativeInstallError>
```

or:

```rust
pub struct InstalledPrimitive {
    pub key: PrimitiveKey,
    pub method: ObjRef,
}
```

The installer then executes:

```rust
vm.typing_registry.method_semantics.insert(
    method,
    RuntimeCallableRef { pool, record },
);
```

using the verified primitive-key→callable-record map built before installation.

Do not add the full `PrimitiveDescriptor`, `TypeExprSpec`, or source body to `MethodObject`.

## 38. Reflection Expectations

Native method reflection should become indistinguishable from source methods with respect to published type metadata where the contract is known. Reflection may additionally expose implementation status/effects later, but type lookup must flow through the same `MethodSemanticIndex` path.

Tests should prove that a reflected native method such as `String#_$byteAt(_)` or a public arithmetic method resolves to a callable record with `TrustedNative` type authority and exact parameter/return forms.

---

# Part IX — LSP and Semantic Integration

## 39. Replace the Single `core.ph` Native-Surface Model

`phalcom-lsp/src/semantic/core_source.rs` currently treats `core.ph` as one semantic core source and overlays `NATIVE_CLASSES`/`NATIVE_MEMBERS`. Replace this model with universe-project module ingestion.

The LSP should resolve the universe source origin in this order:

1. explicitly configured universe/sysroot project root, if supported by current settings;
2. workspace checkout `phalcom-core/core/universe` when editing Phalcom itself;
3. bundled `BuiltinProjectSourceProvider(BuiltinProject::Universe)` fallback.

Each module retains logical `ModuleId::builtin(BuiltinProject::Universe, path)` identity and a physical source location when one exists. Bundled sources use canonical `phalcom://universe/...` URIs.

Do not flatten all files into one synthetic module merely to preserve the old `CORE_MODULE_URI` representation. The module system already models the universe as a builtin project.

## 40. Preserve Workspace Source Overrides

The current `CoreSource::select` lets a workspace/core file override the bundled copy. Preserve that developer experience at project granularity. Introduce an abstraction such as:

```rust
pub enum UniverseSourceOrigin {
    Configured(PathBuf),
    Workspace(PathBuf),
    Bundled,
}
```

with module-specific loading. For `Configured`/`Workspace`, parse the physical `.ph` path corresponding to a builtin module ID but preserve the logical builtin universe identity in the semantic graph. This lets go-to-definition open editable repository source while import/type identity remains canonical.

## 41. Enrich `MemberSurface`

Replace the lossy:

```rust
pub native_return: Option<NativeReturnShape>
```

with a richer native semantic attachment:

```rust
pub struct NativeMemberSurface {
    pub primitive: &'static PrimitiveSurfaceSpec,
    pub rust_source: Option<NativeImplementationSource>,
}

pub struct MemberSurface {
    ...
    pub native: Option<NativeMemberSurface>,
    ...
}
```

If lifetime/static constraints make direct references awkward, generated native-surface can expose a copyable/static wrapper or an owned Arc-backed representation. The important architectural rule is that the metadata comes from the generated primitive surface, not another manually reduced table.

## 42. Merge Source `@native` + Native Descriptor

For a source member with `@native`:

```text
source member location/names/docs/types
           +
generated native primitive metadata
           =
one MemberSurface
```

For a source member without `@native`, do **not** automatically overlay a primitive merely because selector/owner/side happen to match. Such a member is real executable source behavior. In canonical universe code this distinction is important for wrappers and derivable overrides.

After strict migration, a required primitive should always have its `@native` source anchor, so synthetic fallback members should disappear for language-visible primitives.

## 43. Fix Implementation Field Classification

In `phalcom-lsp/src/semantic/surface.rs`, change:

```rust
field.name.starts_with("_$")
```

to:

```rust
field.name.starts_with("__")
```

Add tests for `_sourceField`, `__implementationField`, and `_$selector` so the two implementation namespaces cannot regress into one another.

## 44. Phaldoc and Navigation

The LSP already has source-oriented hover/documentation behavior. Once native semantic members are source-backed, ordinary go-to-definition should land at the `.ph` declaration. Phaldoc should be harvested from that declaration.

Native metadata should enrich hover with structured information such as:

```text
native implementation
internal/public visibility
parameter/return types
effects
raises
stability
```

without forcing authors to repeat machine facts in doc comments.

An implementation-navigation command may additionally offer the Rust source location from `NativeSourceSpec` or the generated scanner's provenance, but `.ph` remains the primary language definition target.

## 45. Internal Members in Completion

`MemberVisibility::Internal` should remain available to semantic queries but be hidden from ordinary user completion unless the request comes from privileged universe source or an explicit “show internals” tooling mode. This is a presentation decision, not dispatch enforcement.

---

# Part X — Primitive Migration

## 46. Build an Exhaustive Migration Census First

Before editing primitive modules, produce an automated census from the current repository. The census should enumerate every legacy registration in `phalcom-core/src/universe/primitives.rs` and every existing `#[primitive]` declaration, keyed by canonical `(owner, side, selector)`.

The census is a migration tool, not a new permanent source of truth. It may be implemented as a test utility or temporary executable under `phalcom-core/bin`, but its output should not become another checked-in hand-maintained manifest.

For each primitive, record:

```text
owner
side
selector
member kind
Rust function
legacy registration present?
#[primitive] present?
visibility
parameter metadata complete?
return metadata complete?
source @native anchor present?
```

The first deliverable is a zero-ambiguity list of every primitive still dependent on legacy installation.

## 47. Migration Unit Rule

Migrate by coherent primitive module, not by scattered functions. In one reviewable change for a module:

1. Add complete `#[primitive(...)]` metadata to every language primitive in that module.
2. Add/update canonical source `@native` anchors for those primitives.
3. Add types and Phaldoc to source declarations.
4. Mark implementation selectors `@internal` and Rust `visibility = internal`.
5. Remove the corresponding legacy `primitive!*` registrations from `universe/primitives.rs`.
6. Run source/native verification for those keys.
7. Run module/runtime/LSP tests.

Never leave one primitive installed by both mechanisms after a migration commit.

## 48. Recommended Migration Order

### Wave 1 — scalar exemplars

```text
Number / Int / Float
String
Bool
Symbol
Option / Some / None-facing operations
```

This wave exercises public arithmetic, getters, class-side constructors/factories, internal raw operations, immediate values, and source wrappers.

### Wave 2 — object model and reflection kernel

```text
Object
Behavior
Class
Metaclass-related native hooks
Message
Selector
SelectorPattern
Attribute
Method / MethodFamily / Family / bound variants
```

This wave exercises internal dispatch hooks, shape primitives, reflection, and class-side/instance-side distinctions.

### Wave 3 — collections and binary storage

```text
List
Map
Set
Tuple
Record
Range
Bytes
Iterable-related irreducible hooks
```

This wave should aggressively preserve high-level `.ph` algorithms while exposing only raw allocation/storage operations as internal native floor.

### Wave 4 — callable/control/runtime services

```text
Block / Function / Closure call gateways
Error primitives
System
Module/package/project reflection primitives
Fiber/scheduler primitives
Resource primitives
typing reflection primitives
```

These primitives are more likely to use shape ABI, effects, raises, privileged trust, and nontrivial return-flow metadata; migrate them after the schema/generator is proven by simpler modules.

## 49. Source Classification Review for Every Migrated Method

Every native/current-core method must be classified explicitly as one of:

```text
A. declaration-only @native primitive
B. reference-bodied @native primitive
C. real .ph wrapper around native floor
D. real .ph derivable/high-level algorithm
```

The migration review should reject “E. left invisible in Rust only” for language-visible required primitives.

---

# Part XI — Universe Source Migration

## 50. `scalar/number.ph`

Current source contains only:

```phalcom
class Number {}
class Int is Number {}
class Float is Number {}
```

This is the cleanest first target for native-class/source declaration work. After infrastructure lands, convert it toward:

```phalcom
/// Abstract numeric protocol implemented by Phalcom's immediate numeric values.
@native
class Number {
  /// Adds another numeric value.
  @native
  +(_ other: Number) -> Number

  // ... other primitive arithmetic/comparison declarations ...

  @class
  @native
  new() -> Number
}

@native
class Int is Number {
  @native
  &(_ other: Int) -> Int
  // ...
}

@native
class Float is Number {
  @native
  abs -> Float
  // ...
}
```

Exact return types must follow the typing design and actual numeric semantics; do not mechanically assume every operation returns its receiver class. The verifier forces the Rust and source contracts to settle those choices explicitly.

## 51. `scalar/string.ph`

Current `string.ph` is already a strong example of the desired *source-over-native-floor* design. It implements `toString`, `size`, `isEmpty`, slicing wrappers, Unicode decoding, searching, splitting, replacing, trimming, and other behavior in `.ph`, but calls undeclared internal primitives `_$byteCount`, `_$byteAt`, and `_$slice`.

Add canonical declarations near the top of `String`:

```phalcom
/// Returns the UTF-8 storage length in bytes.
@internal
@native
_$byteCount -> Int

/// Returns the byte at `index`, or `None` when out of bounds.
@internal
@native
_$byteAt(_ index: Int) -> Option<Int>

/// Returns the UTF-8 substring in the requested byte range.
@internal
@native
_$slice(_ start: Int, _ end: Int) -> String
```

Keep real source wrappers such as:

```phalcom
size -> Int { _$byteCount }
isEmpty -> Bool { _$byteCount == 0 }
slice(_ start: Int, _ end: Int) -> String { _$slice(start, end) }
```

Do not mark these wrappers native.

Also add source declarations for public native `+`, `hash`, and class-side constructors/factories according to the final source API. If a source method such as `toString` already intentionally overrides/inherits native behavior, classify it based on the actual live installer rather than assuming it should become an anchor.

## 52. `option/option.ph`

The current file contains extensive work-unit commentary describing why `Option`/`Some` are bootstrapped, why `None` is special, and historical task divisions. Convert stable semantics into Phaldoc and concise implementation comments.

Expose native primitives such as `Some` construction and `Option#match` through explicit anchors. Keep all derivable combinators—`map`, `flatMap`, `filter`, `ifSome`, `ifNone`, `orElse`, `unwrapOr`, equality/hash wrappers, and Result/Ok/Err source logic—as real `.ph`.

Once `@native class` preserves special bindings, decide whether the hidden `None` class itself should appear as a source native class anchor. If it does, prove in a regression test that the public global remains immediate `None` after universe execution.

## 53. Documentation Cleanup Policy

During each universe-file migration:

- `///` / `//!` contains durable Phaldoc and conceptual API/semantic explanation.
- `//` contains only short local implementation or bootstrap reasoning that a maintainer needs beside the code.
- task IDs, implementation-unit labels, stale line references, completed migration instructions, and “do not add until task X” archaeology move to `docs/work`, ADRs, or PDRs.
- types, native/internal status, effects, and other machine-readable facts should not be repeated as prose merely for discoverability.

The desired source should read as the implementation/reference source of the language itself.

---

# Part XII — Builtin Source Registration and Interface Cleanup

## 54. Make `phalcom-modules` the Source Corpus Authority

`BuiltinProjectSourceProvider` already owns stable builtin source IDs and bundled source loading. Runtime and LSP must call it rather than embedding their own copies.

Add provider APIs needed by bootstrap/LSP instead of exposing private constants ad hoc:

```rust
pub fn node_specs(&self) -> &'static [BuiltinNodeSpec];
pub fn module_ids(&self) -> Vec<ModuleId>;
pub fn source_location(&self, id: &ModuleId) -> Result<SourceLocation, ModuleLoadError>;
```

The exact allocation strategy can avoid the `Vec` if desired; the semantic point is one enumerator.

## 55. Validate `UNIVERSE_NODES` Against the Physical Corpus

`UNIVERSE_NODES` and `source_text()` are still manually maintained inside one crate. Add a repository test that walks `phalcom-core/core/universe/src` and ensures every relevant `.ph` file is represented by the builtin provider and every provider node loads successfully. This converts silent drift into CI failure without introducing another handwritten list.

If maintainers later want full build-time generation from `package.ph`, implement it in `phalcom-modules/build.rs`; it is not necessary to block the native implementation work because the most dangerous competing implementation registry is `NATIVE_MEMBERS`, not the source-provider mapping.

## 56. Reduce `BuiltinInterfaceBuilder` Native Overlay

`phalcom-modules/src/builtin_interface.rs` currently hard-codes module-path→native-class-name lists to overlay primordial declarations. Once canonical source files actually contain `@native class` declarations, source-derived interfaces should naturally contain those class declarations.

Remove per-module name overlays as classes migrate. Preserve only the minimal root/binding overlay required for runtime-created values or prelude/re-export semantics that cannot be expressed directly in the module source. That minimal overlay should derive names from `UNIVERSE_BINDINGS`, not another duplicated list.

---

# Part XIII — Diagnostics and Error Ownership

## 57. Error Layering

Use the layer that owns the violated invariant:

```text
parser syntax error
  declaration/body grammar malformed

compiler attribute/declaration error
  illegal target, illegal arguments, privilege, namespace assertion

native declaration macro error
  invalid Rust primitive metadata, ABI, selector, visibility

bootstrap native contract error
  valid source + valid descriptor disagree with each other

runtime invariant error
  resulting object graph/bindings violate bootstrap invariants
```

Do not report a source/native type mismatch as a generic compiler parse error, and do not defer a Rust `_$`-public mismatch to runtime if the proc macro can reject it.

## 58. Recommended New Structured Errors

In `phalcom-core/src/compiler/lib/error.rs`, add structured variants rather than only `CompilerError::Message` for high-value new rules:

```text
InternalAttributeRequired
InternalAttributeNamespaceMismatch
NativeAttributeRequiresPrivilegedCore
DeclarationBodyRequiresImplementation
NativeClassIdentityMismatch
```

In the native verifier, define a separate error enum with source span and descriptor provenance.

Stable diagnostic codes should match the attribute specs:

```text
attr.arguments_not_allowed
attr.internal_required
attr.internal_requires_implementation_namespace
attr.native_requires_privileged_core
member.visibility_conflict
member.declaration_requires_implementation
native.duplicate_anchor
native.orphan_anchor
native.missing_anchor
native.class_identity_mismatch
native.kind_mismatch
native.side_mismatch
native.visibility_mismatch
native.parameter_shape_mismatch
native.parameter_type_mismatch
native.return_type_mismatch
```

---

# Part XIV — Detailed File Change Map

## 59. New Files

### `phalcom-native-decl/Cargo.toml`

New VM-free shared crate for Rust attribute grammar/validation.

### `phalcom-native-decl/src/model.rs`

Owned primitive declaration representation used by both proc macro and scanner.

### `phalcom-native-decl/src/parse.rs`

`syn` parser for `#[primitive(...)]` arguments.

### `phalcom-native-decl/src/validate.rs`

Canonical selector, type, visibility, lifecycle, flow, ABI metadata validation.

### `phalcom-native-decl/src/emit.rs`

Shared conversion/emission helpers from owned declarations to static native-meta Rust tokens.

### `phalcom-native-surface/build.rs`

Recursively scan native primitive Rust source and generate the VM-free surface.

### `phalcom-core/src/native/source.rs`

Native class/member source anchor indexing.

### `phalcom-core/src/native/verify.rs`

Whole-universe source↔descriptor verification.

### `phalcom-semantic/src/native_contract.rs`

VM-free source/native contract type normalization and comparison.

### Optional `phalcom-core/src/native/typing.rs`

Lower verified native contracts into `phalcom-type-meta` records if this logic does not fit cleanly in the existing typing module.

### Optional `phalcom-core/src/compiler/declaration_integrity.rs`

Early `@internal`/`@native`/implementation-namespace validation if extracting it improves `class_decl.rs` maintainability.

## 60. Existing Files to Modify

### Workspace

`Cargo.toml`

- add `phalcom-native-decl` workspace member;
- add workspace dependency alias if consistent with current style.

### AST/parser

`phalcom-ast/src/ast.rs`

- add `BuiltinAttr::Internal`;
- add `MemberBody`;
- change method/getter/setter body fields;
- add helpers.

`phalcom-ast/src/parser.rs`

- parse declaration-only callable members;
- preserve block vs declaration distinction;
- parse `@internal` through builtin recognition;
- tests.

Potential `phalcom-ast/src/selector.rs`

- central source member→canonical selector projection.

### Attribute/compiler

`phalcom-core/src/compiler/attributes.rs`

- `InternalExpander`;
- attribute arg policies;
- include class target for `NativeExpander`;
- ensure early validations occur before native drop;
- adapt body-weaving helpers to `MemberBody::Block` only;
- native reference/declaration members remain dropped before weave.

`phalcom-core/src/compiler/lib/class_decl.rs`

- add `internal` to compiler-only attributes;
- call declaration-integrity validator;
- native class assertion/stub completion;
- preserve special preexisting globals for `@native class`;
- adapt all body scans to `MemberBody`;
- defensive error if declaration body survives.

`phalcom-core/src/compiler/lib/mod.rs` and other body consumers

- adapt method compilation helpers to receive executable block statements;
- keep `compiling_privileged_core()` as authority seam.

`phalcom-core/src/compiler/lib/error.rs`

- structured new compiler errors.

`phalcom-core/src/interpret.rs`

- add parsed-AST compile entry point so bootstrap can reuse verified ASTs.

### Native metadata/macro

`phalcom-native-meta/src/primitive.rs`

- `NativeAnchorPolicy`;
- add `anchor` to `PrimitiveSurfaceSpec`.

`phalcom-native-meta/src/types.rs`

- extend nested callable representation if the primitive census proves it is needed.

`phalcom-native-macros/src/lib.rs`

- delegate parsing/validation to `phalcom-native-decl`;
- retain proc-macro-specific function ABI and descriptor emission;
- enforce final internal visibility through shared validator.

### Native runtime

`phalcom-core/src/native/mod.rs`

- export source/verifier/typing modules.

`phalcom-core/src/native/registry.rs`

- keep uniqueness; optionally return structured registry validation report used by bootstrap.

`phalcom-core/src/native/install.rs`

- consume verified metadata mapping;
- return/capture installed `MethodObject` refs;
- populate `MethodSemanticIndex`.

`phalcom-core/src/native/descriptor.rs`

- likely minimal changes; preserve descriptor as runtime native implementation contract.

### Bootstrap

`phalcom-core/src/vm/bootstrap.rs`

- add fallible bootstrap path;
- split named phases;
- preflight before install;
- load source through builtin provider;
- remove hard-coded source text list;
- eventually remove `Universe::install_primitives` call.

### Legacy native table

`phalcom-core/src/universe/primitives.rs`

- shrink module-by-module during migration;
- delete language primitive registrations at completion;
- delete file entirely if no non-language bootstrap responsibility remains.

### Runtime typing

`phalcom-core/src/typing/registry.rs`

- safe pool-ID allocation/loading helper.

`phalcom-core/src/typing/side_table.rs`

- likely no representation change; add tests/helpers only if needed.

`phalcom-core/src/modules/materialize.rs`

- use non-hardcoded metadata pool registration API.

### Modules/source provider

`phalcom-modules/src/builtin.rs`

- expose canonical builtin node enumeration;
- make bootstrap/LSP consume provider rather than duplicating source text lists;
- add corpus consistency tests.

`phalcom-modules/src/builtin_interface.rs`

- retire path→native-class overlay entries as source declarations become canonical;
- derive residual binding overlay from `UNIVERSE_BINDINGS` where possible.

### Native surface

`phalcom-native-surface/src/lib.rs`

- remove handwritten `NATIVE_MEMBERS`/duplicate enums/return-shape authority;
- include generated data;
- re-export native-meta types as appropriate.

`phalcom-native-surface/Cargo.toml`

- add native-meta and build dependencies.

### LSP

`phalcom-lsp/src/semantic/core_source.rs`

- replace single `core.ph` model with universe source-set/provider model;
- remove handwritten `NATIVE_CLASSES`/`NATIVE_MEMBERS` overlay logic.

`phalcom-lsp/src/semantic/surface.rs`

- replace `native_return` with rich native attachment;
- fix `__` implementation-field classification;
- preserve source native attributes/body/source identity.

`phalcom-lsp/src/selectors.rs`

- delegate source selector projection to shared AST helper.

Hover/completion/definition files

- consume merged native metadata and actual universe source locations.

### Universe corpus

All relevant files under:

```text
phalcom-core/core/universe/src/
```

receive native class/member declarations, type annotations, Phaldoc, internal attributes, and comment cleanup.

---

# Part XV — Test Strategy

## 61. AST and Parser Tests

Add tests proving:

```text
@internal recognized as BuiltinAttr::Internal
@native foo() -> T             => MemberBody::Declaration
@native foo -> T               => getter declaration
@native foo=(put x: T) -> U    => setter declaration, if setter declaration syntax is retained
foo() {}                       => MemberBody::Block(empty)
reference body                  => MemberBody::Block(nonempty)
return annotations preserved
attribute spans preserved
```

A declaration-only ordinary member should parse successfully but fail semantic/compiler legality, proving grammar and semantics are cleanly separated.

## 62. Attribute/Compiler Tests

Add compile fixtures for:

```text
privileged _$ + @internal                  pass
privileged __field + @internal             pass
privileged _$ without @internal            fail
privileged __field without @internal       fail
@internal ordinary method                  fail
@internal _sourceField                     fail
@internal + @private                        fail
@internal + @protected                      fail
ordinary module + @native                  fail
@native(args)                               fail
@internal(args)                             fail
@class(args)                                fail
```

Retain existing `@native` drop tests, but rewrite them so the privilege requirement is explicit. A made-up user-module native method should no longer be used as the positive drop fixture.

## 63. Native Declaration Macro Tests

`phalcom-native-macros` currently has no dev-dependency test harness. Add focused parser unit tests in `phalcom-native-decl`, which can test most invalid declarations without compile-fail infrastructure. For Rust function-signature/ABI errors that require proc-macro expansion, add `trybuild` only if no existing workspace mechanism can cover proc-macro compile failures.

Required cases:

```text
canonical public primitive
canonical internal primitive
_$ + public                        fail
internal + ordinary selector       fail
duplicate options                  fail
invalid selector                   fail
selector/params mismatch           fail
label mismatch                     fail
invalid type expression            fail
callable types mismatch            fail
invalid ABI/function signature     fail
anchor default = Required
anchor = Hidden                    pass
invalid anchor value               fail
```

## 64. Generated Surface Tests

At build/unit test level:

- scanner finds every `#[primitive]` attribute under primitive source;
- generated keys are unique;
- generated visibility matches descriptor visibility;
- generated metadata preserves types/effects/flow;
- after complete migration, generated keys equal runtime `PRIMITIVES` keys;
- no handwritten `NATIVE_MEMBERS` remains.

## 65. Native Source Verifier Tests

Construct small parsed source corpora and descriptor fixtures for:

```text
exact match                         pass
source anchor, no descriptor        orphan error
descriptor required, no anchor      missing-anchor error in Complete mode
duplicate source anchor             duplicate error
side mismatch                       fail
kind mismatch                       fail
parameter label mismatch            fail
rest shape mismatch                 fail
parameter type mismatch             fail
return type mismatch                fail
_$ without @internal                source-integrity fail
_$ descriptor public                descriptor validation fail
hidden descriptor no anchor         pass
```

## 66. Bootstrap Tests

Add tests around `VM::try_new()` or the fallible bootstrap seam:

1. Native preflight occurs before primitive installation. A deliberate test fixture mismatch must fail without executing a universe body.
2. Descriptor installation occurs before real source wrappers execute.
3. Declaration-only `@native` members emit no method/bytecode and cannot overwrite primitives.
4. Reference-bodied `@native` members emit no live method.
5. Real wrappers remain executable.
6. `@native class` completes exact primordial identity.
7. `@native class` cannot allocate a new identity.
8. Special `None` global remains immediate absence after a source-native class completion test.
9. Runtime kernel invariants still pass.

## 67. Runtime Typing Tests

Extend `phalcom-core/tests/spec03_reflection.rs` or add a focused native typing test:

- retrieve an installed native method object;
- look it up through `typing_registry.method_semantics`;
- resolve its callable record;
- assert parameter/return metadata and `TrustedNative` authority;
- prove metadata pool IDs remain correct when native and program metadata pools coexist.

## 68. LSP Tests

Add/extend LSP unit/integration tests for:

```text
native source member + generated metadata merges into one member
source wrapper without @native does not get incorrectly overlaid
class-side and instance-side same selector remain distinct
__field classified as implementation storage
_$selector is Internal
internal member hidden from ordinary completion
hover includes source Phaldoc and native type/effect metadata
go-to-definition resolves actual universe module source
workspace universe source wins over bundled source when configured/discovered
bundled fallback uses phalcom://universe/... identity
```

## 69. Corpus/Invariant Tests

Final strict gates:

```text
all source @native anchors resolve
all Required descriptors have exactly one anchor
all authored _$ universe declarations carry @internal
all authored __ universe fields carry @internal
all primitive Rust language implementations use #[primitive]
no ordinary language primitive remains in Universe::install_primitives
native generated surface == runtime descriptor surface
builtin universe provider covers source corpus
```

These tests are the mechanism that prevents the architecture from drifting back into parallel handwritten registries.

## 70. Workspace Verification

Final implementation validation should include at least:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If the repository's CI uses a narrower/different clippy policy, follow the repository configuration rather than introducing unrelated lint churn.

---

# Part XVI — Implementation Sequence and Review Gates

## 71. Phase 0 — Baseline Census and Golden Invariants

Before behavior changes, add migration tooling/tests that capture current primitive keys and current universe source provider coverage. This gives reviewers a way to detect accidental API loss while the installer changes.

**Gate:** every existing installed native callable is accounted for by the census; no migration edits yet.

## 72. Phase 1 — AST Declaration Bodies + `@internal` Infrastructure

Implement `MemberBody`, parser declaration syntax, `BuiltinAttr::Internal`, argument policy, early declaration-integrity validation, and nonprivileged `@native` rejection. Adapt compiler body consumers without yet changing primitive installation.

**Gate:** existing source behavior unchanged; declaration-only syntax parses; ordinary declaration-only source cannot compile; internal namespace tests pass.

## 73. Phase 2 — Shared Primitive Declaration Schema

Create `phalcom-native-decl`, move parser/validator logic from the proc macro, add anchor policy, and harden `_$/internal` validation.

**Gate:** existing descriptor primitives compile identically; parser tests cover all attribute fields; `_$/public` is impossible.

## 74. Phase 3 — Generated Native Surface

Implement native-surface build scan and generated metadata. Keep the old handwritten table only as a temporary parity oracle, not runtime authority. Add a test comparing overlapping entries and fix all discovered mismatches.

**Gate:** generated surface covers every descriptor-backed primitive exactly; LSP can compile against generated representation.

## 75. Phase 4 — Source Index + Structural Verifier

Implement shared source selector projection, `NativeSourceIndex`, type normalization, anchor verifier, structured diagnostics, and parsed universe corpus loading through `BuiltinProjectSourceProvider`.

Initially enable `AnchorsMustResolve` mode so existing missing anchors do not break bootstrap.

**Gate:** hand-written test fixtures exercise every mismatch category.

## 76. Phase 5 — First Canonical Source Migration

Migrate descriptor-backed scalar primitives and add `@native class`/member anchors for the already descriptorized subset. Start with Number/String/Bool/Symbol and the existing descriptor-backed Option/Object pieces as practical.

Implement native-class assertion and special global preservation before adding any source class that would otherwise clobber a special binding.

**Gate:** every currently descriptor-backed `Required` primitive has an anchor for the migrated set; source wrappers continue to run.

## 77. Phase 6 — Bootstrap Preflight

Add fallible bootstrap, parse-once source corpus, preflight before native installation, and provider-driven universe source retrieval.

**Gate:** a deliberate mismatch fails before primitive installation/execution; normal VM startup remains behaviorally equivalent.

## 78. Phase 7 — Native Typing Integration

Build native semantic metadata pool, make pool IDs robust, and register installed native method objects in `MethodSemanticIndex`.

**Gate:** reflection tests return exact trusted native signatures.

## 79. Phase 8 — LSP Universe Project Migration

Replace single-core source overlay with universe project source resolution; merge source anchors with generated native metadata; fix implementation-field classification; preserve workspace source overrides.

**Gate:** hover/completion/definition tests pass using actual universe module identities and no handwritten `NATIVE_MEMBERS` dependency.

## 80. Phase 9 — Complete Primitive Descriptor Migration

Migrate every remaining legacy primitive module in waves. Each migration atomically removes corresponding legacy installer entries and adds source anchors.

**Gate:** `Universe::install_primitives` contains no ordinary language primitive registrations.

## 81. Phase 10 — Strict Bijection

Switch bootstrap/corpus validation to complete mode:

```text
source anchors == Required descriptors
```

Explicit `Hidden` descriptors are the only exception.

**Gate:** no orphan/missing anchors.

## 82. Phase 11 — Delete Legacy Sources of Truth

Delete or reduce:

```text
phalcom-native-surface handwritten NATIVE_MEMBERS
phalcom-native-surface handwritten duplicate dispatch/visibility models
phalcom-core/src/universe/primitives.rs language registration table
VM::run_universe_modules direct include_str SOURCES list
LSP BUNDLED_CORE_SOURCE/core.ph native overlay path
hardcoded per-module native class overlays that source now provides
```

Do not call the migration complete while these systems still determine runtime/tooling behavior.

## 83. Phase 12 — Universe Documentation and Corpus Cleanup

Complete class/member source declarations, Phaldoc, type annotations, and comment cleanup throughout the universe project. Move historical task rationale out of language source.

**Gate:** canonical universe source can be read as the primary language/library definition and passes strict source/native invariants.

---

# Part XVII — Performance and Build Considerations

## 84. Startup Cost

The preflight should be cheap enough to run at VM bootstrap, but it must not repeatedly parse or normalize the same data.

Use the existing parsed-unit cache in `BuiltinInterfaceBuilder`/`BuiltinProjectSourceProvider`. Build `NativeSourceIndex` once. Normalize source/native types once into verified callables. Store descriptor lookup in a `BTreeMap`/`HashMap` keyed by owned primitive identity for O(1)/O(log n) matching rather than scanning the distributed slice per source member.

Reference bodies are not executed by verification. Behavioral equivalence testing is not a startup operation.

If release startup measurements show the full verification cost matters after the architecture stabilizes, a generated build fingerprint may permit a trusted release build to validate a precomputed manifest hash while debug/development runs full structural verification. Do not introduce that optimization before profiling; correctness-first verification is inexpensive at the current primitive count.

## 85. Build Script Cost

`phalcom-native-surface/build.rs` should parse only Rust files under the primitive source tree and use `rerun-if-changed` narrowly. It should not invoke Cargo recursively, execute the compiler, or link `phalcom-core`.

The scanner's output is deterministic. Sort generated primitive entries by `(owner, side, selector)` before emission so generated diffs/debugging remain stable.

## 86. LSP Startup

The LSP remains VM-free. It reads generated native metadata and builtin/source project units, never instantiates a VM to discover core protocol. This preserves the architectural intent recorded in `phalcom-lsp/Cargo.toml` and ADR-0056.

---

# Part XVIII — Migration Hazards and Required Resolutions

## 87. Do Not Let Source Anchors Become a Binding Language

It may be tempting to write:

```phalcom
@native("string_raw_byte_at")
_$byteAt(...)
```

Do not do this. Rust names are implementation details. Structural identity is sufficient and survives Rust refactors.

## 88. Do Not Generate `.ph` from Rust

Generating canonical source from `#[primitive]` would ensure consistency by removing independence, but it would also reduce `.ph` to machine output and make parameter naming, docs, source organization, and reference bodies awkward. The intended design deliberately retains authored source and verifies it.

## 89. Do Not Compare Type Strings

`"Option<Int>" == "Option < Int >"` is not a type-system invariant. Normalize structural type nodes and compare semantics.

## 90. Do Not Treat All Native Classes Like Ordinary Global Classes

`None` demonstrates why class identity and global binding are not synonymous. Native class completion must preserve native special bindings.

## 91. Do Not Treat `@internal` as Security

A fake `@internal` attribute in user source must never authorize `_$` sends/declarations. Continue using actual core/runtime identity checks.

## 92. Do Not Add Native Metadata to Selector Dispatch

Types/effects/raises are reflective/static facts. Native migration must not accidentally make them dispatch keys.

## 93. Do Not Preserve `NATIVE_MEMBERS` as a “Backup”

A backup handwritten table becomes a future source of truth as soon as some consumer depends on it. During migration it may serve as a parity oracle; final consumers must use generated metadata.

## 94. Do Not Mechanically Convert Source Fields to `__`

`_message`, `_kind`, `_cause`, and similar source fields remain source fields unless their ownership semantics actually change. `__` means runtime/compiler implementation storage, not merely “a core field.”

## 95. Do Not Force Reference Bodies Everywhere

For irreducible VM operations, a declaration plus accurate Phaldoc/type contract is superior to fake executable-looking pseudocode. Reference bodies are for genuinely illuminating formulations.

---

# Part XIX — Final Acceptance Criteria

The feature is complete only when all of the following are true.

### Source model

- Canonical language-visible primordial classes are explicitly represented in the universe source as appropriate.
- Native methods have source `@native` anchors.
- Implementation selectors/fields visibly carry `@internal` in authored canonical source.
- Native source may be declaration-only or reference-bodied.
- Real wrappers remain executable source.
- Stable Phaldoc and type annotations are present for migrated public/native protocol.

### Rust native model

- Every ordinary language primitive is declared through `#[phalcom_native_macros::primitive(...)]`.
- `_$` primitives cannot be declared public.
- `PrimitiveSurfaceSpec` carries anchor policy.
- Descriptor registry contains unique primitive keys.

### Generation

- `phalcom-native-surface` is generated from Rust primitive attributes.
- No handwritten `NATIVE_MEMBERS` determines LSP/runtime behavior.
- Generated surface and runtime descriptor keys are equal.

### Bootstrap

- Native/source verification occurs before native installation and universe execution.
- Every source anchor resolves exactly once.
- Every `Required` descriptor has exactly one source anchor.
- Native classes complete existing primordial identities.
- Special native globals such as immediate `None` are preserved.
- Post-bootstrap kernel invariants continue to pass.

### Runtime typing

- Installed native `MethodObject`s are mapped through `MethodSemanticIndex`.
- Native callable metadata uses `PublishedTypeAuthority::TrustedNative`.
- Multiple semantic metadata pools cannot collide on hard-coded pool IDs.

### LSP

- LSP remains VM-free.
- It consumes actual universe module source through the builtin/workspace source model.
- Source `@native` anchor + generated descriptor becomes one semantic member.
- Go-to-definition lands in the real universe source module.
- Hover can show Phaldoc plus structured native/type metadata.
- `__` fields are correctly classified as implementation storage.
- internal protocol is not leaked into ordinary completion by default.

### Deletion gates

- `Universe::install_primitives` is no longer the language primitive registry.
- The legacy manual primitive registration table is deleted or has no ordinary language primitive responsibility.
- VM bootstrap does not maintain its own source text list.
- LSP no longer depends on `core/core.ph` as the canonical native-core definition.
- hard-coded per-module native class overlays are reduced to only irreducible bootstrap binding mechanics and derive from the canonical binding catalog where possible.

---

# Part XX — Example End State

## 96. `String` source

```phalcom
/// Immutable UTF-8 string value.
@native
class String {
  /// Creates a string representation of `value`.
  @class
  @native
  new(_ value: Object) -> String

  /// Concatenates two strings.
  @native
  +(_ other: String) -> String

  /// Returns the stable content hash.
  @native
  hash -> Int

  /// Returns the UTF-8 storage length in bytes.
  @internal
  @native
  _$byteCount -> Int

  /// Returns the byte at `index`, or `None` when out of bounds.
  @internal
  @native
  _$byteAt(_ index: Int) -> Option<Int>

  /// Extracts a UTF-8-aligned byte range.
  @internal
  @native
  _$slice(_ start: Int, _ end: Int) -> String

  /// Returns the UTF-8 byte length.
  size -> Int {
    _$byteCount
  }

  isEmpty -> Bool {
    _$byteCount == 0
  }

  slice(_ start: Int, _ end: Int) -> String {
    _$slice(start, end)
  }

  // Higher-level Unicode/search/trim/etc. algorithms remain real Phalcom.
}
```

## 97. Corresponding Rust primitive

```rust
#[phalcom_native_macros::primitive(
    String,
    "_$byteAt(_)",
    params = [Int],
    returns = "Option<Int>",
    types = "(Int) -> Option<Int>",
    effects = pure,
    side = instance,
    visibility = internal,
    stability = stable,
    abi = value,
    flow = value,
    trust = ordinary,
    anchor = required,
)]
pub fn string_raw_byte_at(
    vm: &mut VM,
    receiver: &Value,
    args: &[Value],
) -> PhResult<Value> {
    // native representation access
}
```

## 98. Bootstrap verification

```text
Source
  module       universe.scalar.string
  owner        String
  side         instance
  selector     _$byteAt(_)
  visibility   @internal
  params       [Int]
  returns      Option<Int>
  body         declaration

Rust
  owner        String
  side         instance
  selector     _$byteAt(_)
  visibility   Internal
  params       [Int]
  returns      Option<Int>
  implementation string_raw_byte_at

Result
  verified NativeCallable
      ↓
  native typing record
      ↓
  installed MethodObject
      ↓
  MethodSemanticIndex
```

The `.ph` declaration is the place a Phalcom developer reads and navigates. The Rust descriptor is the machine contract that installs the implementation. Bootstrap is the proof obligation connecting them.

---

# Part XXI — Recommended Commit Boundaries

The implementation is large enough that review quality will improve if it lands as a series of independently green changes. A practical sequence is:

1. `feat(ast): represent declaration-only callable members`
2. `feat(attributes): add internal declaration assertions and native privilege checks`
3. `refactor(native): extract shared primitive declaration parser`
4. `feat(native): add primitive anchor policy and strict internal visibility`
5. `build(native): generate VM-free primitive surface from primitive attributes`
6. `feat(native): index and verify universe native source anchors`
7. `feat(bootstrap): preflight native universe before installation`
8. `feat(typing): attach trusted native callable metadata to installed methods`
9. `feat(lsp): consume universe source modules and merge native metadata`
10. `refactor(native): migrate scalar primitive modules off legacy installer`
11. subsequent module-by-module primitive migration commits
12. `refactor(native): remove legacy primitive installer and handwritten native surface`
13. `docs(core): canonicalize universe native declarations and Phaldoc`
14. `test(native): enforce strict descriptor-anchor bijection and corpus invariants`

Each commit should keep `cargo test --workspace` green. Primitive migration commits should remove the matching legacy registration in the same commit that adds the descriptor to avoid temporary double installation.

---

# Part XXII — Completion Definition

The conceptual completion criterion is not “the attributes parse” and not “the universe files contain more declarations.” The implementation is complete when a native callable has one coherent lifecycle:

```text
Rust function
   ↓ declared once
#[primitive(...)]
   ↓ produces
runtime descriptor + generated tooling surface
   ↓ independently cross-checked with
canonical @native .ph declaration
   ↓ verified before execution
native semantic callable
   ↓ installed once
MethodObject
   ↓ associated through
MethodSemanticIndex
   ↓ presented by
reflection + LSP using canonical .ph source
```

At that point the competing sources of truth have been reduced to two *intentional* sources of authority that describe different dimensions and are mechanically checked against each other. The runtime no longer needs a manual primitive catalog; the LSP no longer needs a handwritten native protocol table; the universe no longer hides large portions of its actual language surface in Rust; and native typing/reflection becomes part of the same semantic architecture as ordinary source code.
