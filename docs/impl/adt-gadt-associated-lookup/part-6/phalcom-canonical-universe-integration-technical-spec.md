# Phalcom Canonical Universe Integration — Technical Specification

**Status:** Proposed implementation specification  
**Repository:** `aureat/phalcom-lang`  
**Repository baseline:** `49d74f9a7d95f695c8ff38c954eca938e6fec16f` (`main`, inspected 2026-09-01)  
**Scope:** Removal of the legacy `core` library model, removal of the separate `std` builtin package, canonicalization of the `universe` package across modules, semantics, compiler/runtime, reflection, and LSP/IDE features.

---

## 1. Purpose

Phalcom currently contains three overlapping historical models of its shipped language environment:

1. the old monolithic `core` module model;
2. the newer modular builtin `universe` project;
3. a separate builtin `std` project.

The public module system has already moved significantly toward modular `universe`, but the semantic analyzer, VM bootstrap, runtime global lookup, native-surface merge, LSP source transport, and several compiler/query paths still manufacture or consume a synthetic `ModuleId::core()`. In parallel, `std` remains a second builtin identity even though its contents are intended to ship as part of the same Phalcom distribution.

This specification defines the end state:

> **`universe` is the single toolchain-owned package containing the language/runtime substrate and everything Phalcom ships by default. Every source-visible Universe declaration has one canonical identity owned by its actual Universe source module. Prelude visibility, native implementation, primordial bootstrap status, eager initialization, and deep semantic analysis are independent properties.**

The principal correctness goal is not a vocabulary rename. It is identity convergence. A declaration such as `Int` must have one identity throughout the module system, type system, runtime, reflection, source index, hover, go-to-definition, completion, and persisted metadata.

---

## 2. Normative architectural invariants

### UNI-01 — One builtin package

Phalcom has one toolchain-owned builtin package root:

```text
universe
```

`core` and `std` are not packages, modules, forwarding aliases, semantic namespaces, runtime namespaces, or LSP virtual documents.

### UNI-02 — Actual module ownership

Every authored Universe declaration is owned by the module containing its source.

Examples:

```text
universe.object.object::Object
universe.object.class::Class
universe.scalar.number::Number
universe.scalar.number::Int
universe.scalar.number::Float
universe.collections.list::List
universe.option.option::Option
```

The implementation MUST NOT construct parallel identities such as:

```text
core::Int
core::List
universe::<root>::Int
```

merely because a declaration is prelude-visible or native.

### UNI-03 — Prelude is binding policy

Prelude membership is an implicit binding policy:

```text
"Int" -> universe.scalar.number::Int
"List" -> universe.collections.list::List
```

It is not declaration ownership and it is not a hidden global module.

### UNI-04 — Native is implementation provenance

Native metadata identifies runtime implementation and bootstrap relationships. It MUST attach to canonical source declarations. It MUST NOT manufacture semantic declarations by name.

### UNI-05 — One module authority

`phalcom-modules` is the only authority for:

- project/package/module identity;
- absolute and relative import roots;
- package `expose` traversal;
- module source identity;
- unlinked interfaces;
- linked exports and re-exports;
- module dependency graphs;
- module/package source anchors.

`phalcom-semantic`, `phalcom-core`, and `phalcom-lsp` consume those products and do not reconstruct module meaning.

### UNI-06 — One semantic authority

`phalcom-semantic` is the sole authority for:

- declaration identity;
- formal types and kinds;
- generic application;
- hierarchy;
- callable/field signatures;
- ADT/variant identities;
- source occurrences and semantic targets;
- editor semantic presentations.

The LSP does not own a competing builtin type table or resolver.

### UNI-07 — Full shallow discovery; selective deep analysis

The entire shipped Universe MUST be cheaply discoverable for imports, completion, navigation, signatures, and prelude resolution.

This MUST NOT imply eager body analysis or eager runtime initialization of every Universe module.

### UNI-08 — Runtime reachability is separate from IDE discoverability

The complete Universe catalog and the runtime-reachable `LinkedProgram` are distinct products.

Adding `universe.json` to import completion MUST NOT add it to every program's runtime initialization order.

### UNI-09 — Actual source provenance

Go-to-definition, hover documentation, source ranges, and semantic diagnostics for Universe declarations resolve to the actual module-specific source, never to a generated aggregate source.

### UNI-10 — URI identity round-trips

Canonical virtual Universe URIs round-trip exactly:

```text
ModuleId -> URI -> ModuleId
```

There is exactly one root spelling.

### UNI-11 — Runtime prelude does not use core fallback

Bare prelude names eventually lower to canonical linked bindings. `GetGlobal` MUST NOT perform “current module, then hidden core module” lookup.

### UNI-12 — Reflection agrees with semantics

Reflection reports the same actual Universe module/declaration ownership used by semantic analysis.

---

## 3. Definition of Universe

`universe` is the complete toolchain-owned library environment distributed with Phalcom by default.

Membership means:

> “This module/library is owned, versioned, and shipped with the Phalcom toolchain.”

Membership does **not** mean:

- prelude-visible;
- primitive;
- `@native`;
- VM-primordial;
- eagerly initialized;
- deeply analyzed during editor startup.

The package can therefore contain several dependency/bootstrap strata without exposing separate top-level package identities:

```text
Tier 0 — primordial/runtime substrate
Tier 1 — foundational language library
Tier 2 — platform library
Tier 3 — development/testing facilities
```

These tiers are implementation/dependency classifications, not import roots.

---

## 4. Current-state diagnosis at the pinned baseline

### 4.1 Identity still models two builtin packages and a fake core module

[`phalcom-modules/src/identity.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-modules/src/identity.rs) currently defines:

```rust
pub enum BuiltinPackage {
    Universe,
    Std,
}
```

and:

```rust
pub fn core() -> Self {
    Self::builtin(
        BuiltinPackage::Universe,
        ModulePath::from_components(vec![
            ModuleComponent::from_identifier("core")
                .expect("valid identifier")
        ]),
    )
}
```

`ModuleId::core()` creates a logical `builtin:universe:core` identity that has no corresponding canonical source module.

### 4.2 Builtin source topology is split

[`phalcom-modules/src/builtin.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-modules/src/builtin.rs) has `UNIVERSE_NODES` and a separate `STD_NODES`.

`STD_NODES` currently owns facilities including:

```text
io
fs
path
text
regex
json
math
random
time
process
net
concurrent
testing
```

These are to become children of Universe.

### 4.3 Project root tables inject both builtin roots

[`phalcom-modules/src/project.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-modules/src/project.rs) inserts both `universe` and `std` into every resolved project and standalone package import-root table.

### 4.4 Resolver special-cases both names

[`phalcom-modules/src/resolver.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-modules/src/resolver.rs), `ModuleResolver::resolve_import_with_trace`, recognizes `universe` and `std` with bespoke branches and rejects `core`.

The builtin branch also bypasses the ordinary resolved-project external exposure traversal. Universe needs the same package-interface semantics as other external packages.

### 4.5 Semantic bootstrap invents core-owned declarations

[`phalcom-semantic/src/session.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-semantic/src/session.rs), `SemanticWorkspaceSession::with_workspace`, currently calls:

```rust
bootstrap_universe_declarations(
    &mut store,
    &|key| DeclarationId::new(
        ModuleId::core(),
        key.name().into(),
    ),
)
```

and builds hierarchy edges and native surfaces with `ModuleId::core()`.

It also constructs a dummy `LinkedProgram` whose entry and initialization order are `ModuleId::core()`.

### 4.6 Type resolution still contains a core fallback

[`phalcom-semantic/src/resolver.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-semantic/src/resolver.rs) stores a `prelude_module: ModuleId`, builds a declaration under that module, and then separately retries:

```rust
let core_decl =
    DeclarationId::new(ModuleId::core(), root.into());
```

This must be replaced by a name-to-canonical-target prelude map.

### 4.7 Native/source surface merge manufactures core identities

[`phalcom-semantic/src/core_surface/merge.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-semantic/src/core_surface/merge.rs) turns a native owner name into:

```rust
DeclarationId::new(
    crate::identity::ModuleId::core(),
    owner_name.into(),
)
```

while source declarations already correctly derive `DeclarationId` from their actual module in [`core_surface/source.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-semantic/src/core_surface/source.rs).

### 4.8 A generated aggregate core presentation source remains

`SemanticWorkspaceSession` imports `render_canonical_core_source`, parses the generated document, builds a source-index shard for `ModuleId::core()`, and inserts it into presentation sources when a core source is absent.

This produces editor provenance for declarations that already have authored modular source.

### 4.9 VM bootstrap still reconstructs the old monolith

[`phalcom-core/src/vm/bootstrap.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-core/src/vm/bootstrap.rs) performs:

```rust
vm.install_core();
...
vm.run_universe_modules(&source_index)?;
```

`install_core()` creates a hidden core `ModuleObject`.

`run_universe_modules()` then compiles every parsed Universe source unit with that same module handle:

```rust
let module = self.core_module().expect(...);

for parsed in &source_index.units {
    let closure = self.compile_ast_as(
        module,
        source_id,
        (*parsed.program).clone(),
        UnitKind::File,
    )?;
    self.run_in_module(module, closure)?;
}
```

Thus modular source files are flattened back into the old execution namespace.

### 4.10 Runtime materialization already contains the better model

[`phalcom-core/src/modules/materialize.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-core/src/modules/materialize.rs) already allocates one `ModuleObject` per `CompiledModule`, establishes package/root ownership, materializes `LinkedReadSpec`, and builds export tables.

That path should become the model for Universe source execution as well.

### 4.11 LSP still reconstructs a synthetic core document

[`phalcom-lsp/src/core_documents.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-lsp/src/core_documents.rs) declares:

```rust
pub const CORE_MODULE_URI: &str = "phalcom://core";
```

and `canonical_universe_source()` concatenates all Universe modules into one string.

This file should be removed rather than renamed.

### 4.12 Completion is close, but incomplete

[`phalcom-lsp/src/import_completion.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-lsp/src/import_completion.rs) already queries `ModuleQueryFacade`, distinguishes external and relative child completion, and uses linked public exports.

However:

- `std` remains an import root;
- `expose` is folded into generic relative-child syntax;
- there is no `export ...` completion context;
- every exported binding is presented as `CompletionItemKind::CLASS`;
- full Universe completion is only possible if all Universe interfaces are present in query products.

---

## 5. Target identity model

Because Universe is the only builtin package, the preferred final representation is explicit rather than an enum whose sole member is Universe.

### 5.1 `ProjectIdentity`

Replace:

```rust
pub enum ProjectIdentity {
    Builtin(BuiltinPackage),
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}
```

with:

```rust
pub enum ProjectIdentity {
    Universe,
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}
```

### 5.2 `ImportRootTarget`

Replace:

```rust
pub enum ImportRootTarget {
    Builtin(BuiltinPackage),
    Resolved(ResolvedProjectId),
}
```

with:

```rust
pub enum ImportRootTarget {
    Universe,
    Resolved(ResolvedProjectId),
}
```

### 5.3 `ModuleId`

Add explicit constructors:

```rust
impl ModuleId {
    pub fn universe(path: ModulePath) -> Self {
        Self {
            project: ProjectIdentity::Universe,
            path,
        }
    }

    pub fn universe_root() -> Self {
        Self::universe(ModulePath::root())
    }
}
```

Delete:

```rust
ModuleId::core()
ModuleId::builtin(...)
```

from production API once callers are migrated.

### 5.4 Stable identities

Stable module/declaration metadata MUST preserve the Universe owner explicitly.

If persisted metadata currently serializes builtin project names or paths, bump the relevant metadata/cache schema. Old `core`/`std` identities MUST not silently alias current canonical IDs.

---

## 6. Canonical Universe source provider

Rename the builtin-specific provider to reflect its sole responsibility:

```rust
pub struct UniverseSourceProvider;
```

Suggested public API:

```rust
impl UniverseSourceProvider {
    pub const fn new() -> Self;

    pub fn nodes(&self)
        -> &'static [UniverseNodeSpec];

    pub fn module_ids(&self)
        -> impl Iterator<Item = ModuleId>;

    pub fn kind(&self, path: &ModulePath)
        -> Option<ModuleKind>;

    pub fn source_id(&self, id: &ModuleId)
        -> Result<SourceId, ModuleLoadError>;

    pub fn source_text(&self, id: &ModuleId)
        -> Result<Arc<str>, ModuleLoadError>;

    pub fn load_parsed(&self, id: &ModuleId)
        -> Result<Arc<ParsedModuleUnit>, ModuleLoadError>;

    pub fn load_interface(&self, id: &ModuleId)
        -> Result<UnlinkedModuleInterface, ModuleLoadError>;
}
```

`UNIVERSE_NODES` becomes the complete package topology, including former `std` children.

The provider is the source of canonical module IDs and virtual source IDs. No downstream subsystem should construct URI strings by hand.

---

## 7. Canonical Universe URI codec

Create one codec in `phalcom-modules`.

Preferred URI spelling:

```text
phalcom://universe/
phalcom://universe/object
phalcom://universe/object/object
phalcom://universe/scalar/number
```

The root trailing slash is canonical because the root is a package document and it already matches provider `SourceId` behavior.

Required APIs:

```rust
pub fn universe_module_uri(id: &ModuleId) -> Option<String>;

pub fn universe_module_from_uri(
    uri: &str,
) -> Option<ModuleId>;
```

Required invariant:

```rust
let id2 = universe_module_from_uri(
    &universe_module_uri(&id).unwrap()
).unwrap();

assert_eq!(id, id2);
```

Only module infrastructure parses or formats these URIs.

---

## 8. Universe topology and exposure

Universe package interfaces use the same `InterfaceBuilder` / builtin interface model as other packages.

### 8.1 Absolute external import

For:

```phalcom
import universe.collections.list
```

the resolver MUST validate every package boundary:

```text
universe package exposes collections
universe.collections exposes list
```

A physically present but non-exposed child is not externally importable and must not appear in external path completion.

### 8.2 Relative imports inside Universe

Universe source itself can use relative imports. The resolver therefore needs package-owner-aware relative traversal that works for:

```text
ProjectIdentity::Universe
ProjectIdentity::Resolved(...)
```

rather than requiring `ResolvedProjectId`.

### 8.3 Full topology versus runtime reachability

All Universe interfaces belong in the immutable module query catalog.

Only modules actually required by the executable graph belong in program initialization unless bootstrap tier rules require them.

---

## 9. Canonical Universe declaration catalog

Introduce a semantic/source-derived catalog mapping canonical declaration identities.

Suggested type:

```rust
#[derive(Clone, Debug)]
pub struct UniverseDeclarationCatalog {
    pub by_name: BTreeMap<Box<str>, DeclarationId>,
    pub by_native_key:
        BTreeMap<phalcom_native_meta::UniverseKey, DeclarationId>,
    pub source_sites:
        BTreeMap<DeclarationId, SourceSiteId>,
}
```

The name map is useful for validation and prelude construction, but source module ownership is authoritative.

### 9.1 Native key mapping

`UniverseKey` remains a VM-free runtime/native catalog key.

It becomes:

```text
UniverseKey::Int
    ->
universe.scalar.number::Int
```

rather than:

```text
UniverseKey::Int
    ->
DeclarationId(ModuleId::core(), "Int")
```

For runtime-support classes that intentionally have no independent semantic declaration, the mapping is absent or represented explicitly as runtime-support-only.

### 9.2 Validation

During Universe baseline construction:

- every `UniverseBindingSpec` of semantic kind `Class` MUST resolve to exactly one authored declaration;
- no two keys may resolve to the same declaration unless explicitly specified;
- every resolved source declaration must have the expected name/kind;
- runtime-support-only entries MUST NOT manufacture declarations.

---

## 10. `UniverseSemanticBaseline`

Add a reusable compiler-owned shallow semantic baseline.

Recommended location:

```text
phalcom-semantic/src/universe/
    mod.rs
    catalog.rs
    baseline.rs
    prelude.rs
```

Suggested aggregate product:

```rust
#[derive(Clone, Debug)]
pub struct UniverseSemanticBaseline {
    pub sources:
        Arc<BTreeMap<ModuleId, Arc<ParsedModuleUnit>>>,

    pub source_locations:
        Arc<BTreeMap<ModuleId, SourceLocation>>,

    pub unlinked:
        Arc<BTreeMap<ModuleId, UnlinkedModuleInterface>>,

    pub linked:
        Arc<BTreeMap<ModuleId, LinkedModuleInterface>>,

    pub resolved_imports:
        Arc<BTreeMap<(ModuleId, String), ModuleId>>,

    pub declarations:
        Arc<DeclarationTypeTable>,

    pub hierarchy:
        Arc<MapTypeHierarchy>,

    pub dispatch:
        Arc<SurfaceDispatchResolver>,

    pub callable_signatures:
        Arc<CallableSignatureTable>,

    pub field_signatures:
        Arc<FieldSignatureTable>,

    pub source_index:
        Arc<SourceSemanticIndex>,

    pub declaration_catalog:
        Arc<UniverseDeclarationCatalog>,

    pub prelude:
        Arc<PreludeBindings>,
}
```

This product is immutable over ordinary workspace edits.

It is created from actual Universe source modules, not generated core declarations.

---

## 11. Staged semantic bootstrap

Universe analysis has a legitimate bootstrap cycle: primitive/natively implemented declarations are needed to type Universe source, but their canonical identities must come from that source.

Solve the cycle by staging products, not by inventing a module.

### Phase U0 — source topology

Load every Universe package/module identity and parse source.

Produce:

```text
ModuleId
ModuleKind
SourceLocation
ParsedModuleUnit
UnlinkedModuleInterface
```

### Phase U1 — declaration shells

Enumerate top-level declarations from every parsed unit.

Assign canonical IDs immediately:

```text
DeclarationId(actual_module, actual_name)
```

At this phase, no method body needs analysis.

### Phase U2 — native/runtime association

Resolve `UniverseKey`/native surface owners to declaration shells.

Allocate/register intrinsic type forms needed for annotation resolution.

### Phase U3 — hierarchy and signatures

Resolve:

- generic parameter lists;
- explicit superclass references;
- explicit field types;
- explicit callable parameter/return types;
- enum/ADT declarations and variants;
- native-surface associations.

### Phase U4 — package linking/source index

Build:

- linked exports and re-exports;
- prelude bindings;
- import path semantic targets;
- declaration/member/source sites;
- package/module source anchors.

### Phase U5 — deep analysis on demand

Analyze source bodies only when:

- a public signature requires inference;
- a user semantic dependency requires the result;
- explicit validation mode requests whole-Universe checking;
- the Universe source itself is being developed.

Normal user editing does not deep-analyze the complete Universe.

---

## 12. Prelude model

Add:

```rust
#[derive(Clone, Debug, Default)]
pub struct PreludeBindings {
    by_name: BTreeMap<Box<str>, DeclarationId>,
}
```

with:

```rust
pub fn get(&self, name: &str)
    -> Option<&DeclarationId>;

pub fn iter(&self)
    -> impl Iterator<Item = (&str, &DeclarationId)>;
```

Construction resolves configured prelude names to canonical source declarations.

Lookup precedence is:

```text
local lexical/module declaration
explicit import/re-export binding
prelude binding
```

A local declaration named `Int` shadows the prelude `Int`.

No synthetic local import site is created for a prelude occurrence. The occurrence directly targets the Universe declaration.

---

## 13. `LinkedTypeResolver`

Change [`phalcom-semantic/src/resolver.rs`](https://github.com/aureat/phalcom-lang/blob/49d74f9a7d95f695c8ff38c954eca938e6fec16f/phalcom-semantic/src/resolver.rs):

From:

```rust
prelude_module: ModuleId
```

to:

```rust
prelude: Arc<PreludeBindings>
```

Constructor:

```rust
pub fn new(
    linked: Arc<LinkedProgram>,
    known_declarations: HashSet<DeclarationId>,
    prelude: Arc<PreludeBindings>,
) -> Self
```

The final bare-name branch is:

```rust
self.prelude
    .get(root)
    .filter(|decl| self.known_declarations.contains(*decl))
    .cloned()
```

Delete all `ModuleId::core()` fallback construction.

---

## 14. Universe surface/native conformance

Rename `core_surface` vocabulary to `universe_surface`.

`extract_source_declarations(module_id, program)` already has the correct identity rule and supports classes and enums.

Change `merge_surfaces` so native records receive an exact owner map:

```rust
pub fn merge_surfaces<'a>(
    source_declarations: &'a [SourceDeclarationRecord],
    native_records: &'a [NativeSurfaceRecord],
    native_owners:
        &BTreeMap<UniverseKey, DeclarationId>,
) -> Result<Vec<MergedDeclarationSurface<'a>>,
          SurfaceMergeError>
```

No native path may form a `DeclarationId` from `(name, ModuleId::core())`.

The merge must support canonical enums as well as classes so `Option`, `Result`, and `Ordering` migration does not restore class-only assumptions.

---

## 15. Semantic workspace integration

`SemanticWorkspaceSession` should own:

```rust
universe: Arc<UniverseSemanticBaseline>
```

instead of independent synthetic `base_declarations`, `base_hierarchy`, etc. created under core IDs.

During each workspace revision, project source products are composed with the immutable Universe baseline.

The semantic snapshot should expose enough Universe products for:

- type inference;
- dispatch;
- definition sites;
- hover;
- module/path completion;
- references to Universe declarations.

A normal user-source edit MUST NOT reparse/rebuild the baseline.

---

## 16. Snapshot module products

`ModuleQueryProducts` must contain the complete shallow Universe module/interface/source catalog in addition to current workspace products.

This is required because:

```phalcom
import universe.j|
```

must suggest `json` even when JSON is not in the current program's reachable import graph.

Do not force every Universe module into `LinkedProgram.initialization_order` merely to make it queryable.

---

## 17. Source semantic indexing of module syntax

The source index must give semantic identity to import/export/expose syntax.

### 17.1 Import path segments

For:

```phalcom
import universe.collections.list
```

index exact ranges:

```text
universe
  -> SemanticTargetId::Module(universe root)

collections
  -> SemanticTargetId::Module(universe.collections)

list
  -> SemanticTargetId::Module(universe.collections.list)
```

### 17.2 Selective import item

For:

```phalcom
from universe.collections.list import List
```

`List` targets the canonical exported declaration, not its local import binding.

The local import binding retains declaration metadata but is not a second definition.

### 17.3 Aliases

For:

```phalcom
from universe.collections.list import List as Vector
```

uses of `Vector` target `universe.collections.list::List`.

### 17.4 Re-exports

Export/re-export occurrences retain provenance while the semantic target stays the original canonical target.

### 17.5 `expose`

For:

```phalcom
expose .collections
```

the child token targets the actual child package/module.

### 17.6 `export`

For:

```phalcom
export Foo
```

the occurrence targets whatever local/imported canonical binding is being exported.

---

## 18. Module/package source anchors

Define a protocol-neutral source anchor:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceAnchor {
    pub module: ModuleId,
    pub range: SourceRange,
}
```

For declarations, the anchor is the declaration's exact source range.

For modules/packages without explicit declaration syntax:

- module anchor: start/preamble range of the module source;
- package anchor: its `package.ph` source.

`phalcom-modules` supplies module source identity; `phalcom-semantic` can supply semantic range anchors.

---

## 19. Editor semantic query API

Expand `EditorSemanticQuery` with target-driven presentations.

Suggested protocol-neutral products:

```rust
pub enum EditorTargetPresentation {
    Declaration(DeclarationPresentation),
    Callable(CallablePresentation),
    Field(FieldPresentation),
    Variant(VariantPresentation),
    Module(ModulePresentation),
}

pub struct DeclarationPresentation {
    pub id: DeclarationId,
    pub kind: DeclarationKind,
    pub name: Box<str>,
    pub formal_type: FormalPresentation,
    pub kind_presentation: FormalPresentation,
    pub generic_signature: Option<...>,
    pub superclass: Option<DeclarationId>,
    pub documentation: Option<Arc<str>>,
    pub native: bool,
}

pub struct ModulePresentation {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
    pub documentation: Option<Arc<str>>,
}
```

Required queries:

```rust
pub fn target_at(
    &self,
    module: &ModuleId,
    offset: usize,
) -> Option<SemanticTargetId>;

pub fn definition_anchors(
    &self,
    target: &SemanticTargetId,
) -> Vec<SourceAnchor>;

pub fn target_presentation(
    &self,
    target: &SemanticTargetId,
) -> Option<EditorTargetPresentation>;

pub fn visible_symbols_at(...);

pub fn import_candidates(...);

pub fn export_candidates(...);

pub fn expose_candidates(...);
```

The exact type names may be adjusted to existing presentation conventions, but the semantic authority boundary is normative.

---

## 20. Go-to-definition

The LSP implementation becomes target-driven:

```text
URI + position
    ->
canonical ModuleId + offset
    ->
editor.target_at(...)
    ->
editor.definition_anchors(...)
    ->
LSP Location
```

### Required behavior

#### Prelude

```phalcom
const x: Int = 42
```

`Int` -> exact `Int` declaration in `phalcom://universe/scalar/number`.

#### Explicit import

```phalcom
from universe.collections.list import List
```

`List` -> actual `class List<T>` source declaration.

#### Alias

Uses of `Vector` in:

```phalcom
from universe.collections.list import List as Vector
```

-> actual `List` declaration.

#### Module alias

```phalcom
import universe.json as json
```

`json` -> canonical JSON module/package source.

#### Path segments

Each segment navigates to the corresponding package/module.

#### `expose`

Child token -> child package/module.

#### Source-authored native member

A source `@native` method navigates to its Phalcom source declaration. Rust implementation source is implementation provenance, not language definition.

---

## 21. Hover

Hover must use `phalcom-semantic` formal products.

Examples:

```text
class Int is Number
type: Int
module: universe.scalar.number
implementation: native
```

and:

```text
class List<T> is Iterable
type: List<T>
kind: Type -> Type
module: universe.collections.list
```

Callable hover continues to use canonical callable signatures.

Module/package hover displays:

- package/module identity;
- kind;
- documentation;
- source provenance.

Prelude hover and explicit-import hover for the same declaration MUST be semantically identical.

---

## 22. Completion

### 22.1 Import roots

`import |` offers:

```text
universe
current project namespace
dependency aliases
```

It does not offer `core` or `std`.

### 22.2 Universe children

`import universe.|` uses the full Universe topology and `expose` policy.

Former std facilities appear here.

### 22.3 Nested children

`import universe.collections.|` lists only externally exposed children.

### 22.4 Relative imports

Relative completion uses internal package traversal, not external exposure filtering.

### 22.5 Selective import

`from universe.collections.list import |` lists linked public exports only.

Completion kind derives from semantic target kind, not `Binding -> CLASS`.

### 22.6 `export`

`export |` lists legal current-module namespace bindings that can be exported and excludes already exported names where practical.

### 22.7 `expose`

`expose .|` lists direct child package/module candidates and only in package context.

### 22.8 Prelude normal completion

Normal lexical completion includes prelude bindings after local/imported symbols and respects shadowing.

---

## 23. LSP source transport

Delete `phalcom-lsp/src/core_documents.rs`.

Do not create `universe_documents.rs` that concatenates sources.

`phalcom/sourceText` or equivalent virtual-source requests resolve an exact `phalcom://universe/...` URI to an exact canonical `ModuleId`, then return that module's source from:

1. the semantic snapshot/source catalog; or
2. the canonical Universe source provider.

No request path returns a monolithic aggregate document.

---

## 24. Physical sysroot override

If a configurable source override remains, configure a **Universe source root**, not one `core.ph` or `package.ph`.

Bundled and physical Universe sources map to the same module IDs.

For example:

```text
bundled:
  phalcom://universe/collections/list

development checkout:
  .../universe/src/collections/list.ph
```

both denote the same `ModuleId::universe(...)`.

Source location can differ; semantic identity cannot.

---

## 25. Runtime module materialization

The VM must stop creating a hidden core module.

### 25.1 Primordial allocation

It is still valid to allocate primordial `ClassId`s before Universe source execution.

This is runtime initialization order, not semantic ownership.

### 25.2 Canonical module allocation

Materialize every required Universe package/module with its actual `ModuleId`.

Reuse the existing `ModuleObject`/`ModuleRegistry`/package ownership approach in `modules/materialize.rs`.

### 25.3 Bind primordial classes to actual owners

When the owning Universe module is prepared, bind the already-allocated `ClassId` into that module's declaration slot.

For `Int`:

```text
ClassId(Int)
    <->
universe.scalar.number::Int
```

`ClassKey.module` is the canonical `ModuleObject` for `universe.scalar.number`.

### 25.4 Execute source per module

Replace the flattening `run_universe_modules` loop with execution in each parsed module object.

### 25.5 Root exports

The Universe root/package export table can re-export canonical bindings from child modules. Re-exporting does not change declaration ownership.

---

## 26. Runtime prelude lowering

The long-term runtime representation of prelude reads should reuse symbolic linked bindings.

A source-level bare prelude name should resolve semantically to a `DeclarationId`/`SymbolId` and lower to a canonical linked binding read.

The runtime MUST NOT rely on:

```text
GetGlobal current-module miss
    ->
hidden core module lookup
```

Delete `prelude_names` once all compile paths emit explicit prelude reads.

This also removes the corresponding global-cache shadowing special case.

---

## 27. `super` and superclass lookup

Compiler/runtime superclass resolution must use canonical declaration/class identities.

Current historical “own module then core by name” fallback is invalid after the migration.

For a user class implicitly deriving from `Object`, the compiler resolves the canonical prelude/Universe `Object` target first and stores/lowers that target.

`SuperSend` should not rediscover a defining class by leaf name in a core fallback table. It should use the canonical defining class/module identity already known at compile/lowering time, or a stable runtime equivalent.

---

## 28. Semantic roots

`unsupported`, `ellipsis`, `Ordering`, `None`, and other special runtime values must be resolved from canonical Universe declarations/exports.

Do not search a hidden core globals map.

A dedicated runtime `SemanticRoots` struct may remain, but its fields are late-bound from canonical Universe module slots/descriptors.

---

## 29. Reflection

Remove `builtin_std` reflection identity.

Former std modules are reflected as ordinary Universe children.

Declaration reflection reports actual source ownership:

```text
Int.declaringModule
    = universe.scalar.number
```

ADT/variant reflection uses the same canonical IDs used by semantic analysis.

---

## 30. Diagnostics

No semantic diagnostic constructor may silently default source ownership to core.

Required invariant:

```text
diagnostic.primary.module
    ==
the module whose source produced the diagnostic
```

Cross-module labels/related information use each exact module's source URI.

This migration should include a production audit for `SemanticDiagnostic::error(...)` or any constructor that does not require explicit source ownership.

---

## 31. Incremental behavior and performance

### 31.1 Baseline reuse

A normal workspace edit MUST reuse:

- parsed Universe source;
- Universe unlinked interfaces;
- Universe declaration shells;
- native owner mapping;
- hierarchy/signatures whose source is unchanged;
- Universe source index;
- Universe topology.

### 31.2 Request path

Hover, definition, completion, references, and signature help MUST perform immutable snapshot reads only.

They MUST NOT:

- read files;
- invoke source providers;
- parse;
- link;
- run deep analysis;
- instantiate VM objects.

### 31.3 Deep Universe analysis

Whole-Universe deep analysis is a validation/development mode, not an editor cold-start prerequisite.

---

## 32. Physical source layout

The historical path:

```text
phalcom-core/core/universe/
phalcom-core/core/std/
```

should be removed.

Recommended final layout:

```text
phalcom-core/
    builtins/
        universe/
            src/
                package.ph
                object/
                scalar/
                collections/
                io/
                fs/
                json/
                ...
```

The Rust crate name `phalcom-core` is unrelated to the legacy source-library module and does not need renaming.

---

## 33. Compatibility policy

### `core`

`import core...` may continue to produce a dedicated diagnostic such as `LegacyCoreImportRemoved`, but there is no forwarding module and no semantic/runtime core identity.

### `std`

During transition, `import std...` may produce a dedicated `LegacyStdImportRemoved` diagnostic with a suggestion:

```text
use `universe.<path>` instead
```

It MUST NOT resolve through an alias package.

After a compatibility window, even the dedicated diagnostic can be removed in favor of ordinary unknown-root behavior.

---

## 34. Required test matrix

### Modules

- only `universe` builtin root;
- former std nodes exist beneath Universe;
- external package exposure enforced;
- relative Universe imports work;
- URI round-trip exact;
- `core` rejected;
- `std` rejected.

### Semantics

- prelude `Int` target is actual `universe.scalar.number::Int`;
- prelude `List` target is actual `universe.collections.list::List`;
- local shadowing wins;
- explicit Universe import and prelude target the same declaration;
- imported alias keeps external identity;
- imported Universe class participates in inference with real module owner;
- no `ModuleId::core()` declaration exists;
- native owner map resolves exact declarations;
- runtime-support classes do not create semantic declarations.

### Source index

- path segments have module targets;
- selective imports have declaration targets;
- export/re-export occurrences retain canonical targets;
- expose child has module target;
- definition sites include actual Universe declarations;
- no generated aggregate core shard.

### LSP

- exact prelude go-to-definition;
- exact hover with type information;
- explicit Universe import navigation;
- module alias navigation;
- path-segment navigation;
- import root/child completion;
- former std facility completion under Universe;
- selective import completion kinds;
- `export` completion;
- `expose` completion;
- source text for exact Universe URI;
- no `phalcom://core`.

### Runtime

- no hidden core module;
- each Universe source executes in its own module;
- primordial class `ClassKey` owner is actual Universe module;
- prelude reads remain correct;
- late local shadowing remains correct;
- semantic roots and `None` invariants remain correct;
- superclass/`super` behavior remains correct.

### Reflection/persistence

- former std modules reflect as Universe;
- declaration owner is actual source module;
- old metadata IDs rejected or migrated explicitly.

---

## 35. Deletion gates

At completion, production source searches should return zero for architectural legacy uses:

```text
ModuleId::core()
BuiltinPackage::Std
CORE_MODULE_NAME
CORE_MODULE_URI
CoreSource
render_canonical_core_source
install_core
core_module(
prelude_names
phalcom://core
phalcom://std
builtin_std
STD_NODES
```

Appropriate `core_surface` and `CoreDeclarationIds` names should also be gone where they mean the builtin language environment.

Historical documentation may retain those strings when describing history.

---

## 36. Acceptance examples

### 36.1 Prelude

```phalcom
const age: Int = 42
```

Expected:

```text
Int occurrence
 -> Declaration(universe.scalar.number::Int)
 -> TypeStore form for that declaration
 -> hover from that declaration
 -> definition at exact source range
```

### 36.2 Explicit import

```phalcom
from universe.collections.list import List
const values: List<Int> = []
```

Expected:

```text
List -> universe.collections.list::List
Int  -> universe.scalar.number::Int
```

No core ID participates.

### 36.3 Former std module

```phalcom
import universe.json
```

works.

```phalcom
import std.json
```

does not resolve.

### 36.4 Expose

```phalcom
expose .parser
```

`parser` targets the actual child package/module; definition opens its source; completion suggests legal direct children only.

---

## 37. Decision register

| ID | Decision |
|---|---|
| UNI-D01 | `universe` is the only toolchain-owned builtin package. |
| UNI-D02 | `std` is removed; its shipped libraries move under Universe. |
| UNI-D03 | `core` is removed as semantic/runtime/module/LSP machinery, not renamed. |
| UNI-D04 | Canonical declaration ownership is the actual source module. |
| UNI-D05 | Prelude is a name-to-canonical-target map. |
| UNI-D06 | Universe has full shallow indexing and demand-driven deep analysis. |
| UNI-D07 | Module/IDE discovery does not enlarge runtime initialization reachability. |
| UNI-D08 | Native metadata maps to source declarations; it does not own semantic identity. |
| UNI-D09 | Actual Universe source replaces generated aggregate core presentation source. |
| UNI-D10 | `phalcom-modules` owns URI/path/import/export/expose resolution. |
| UNI-D11 | `phalcom-semantic` owns target/type/presentation queries consumed by LSP. |
| UNI-D12 | Runtime prelude ultimately uses explicit canonical linked reads, not core fallback. |
| UNI-D13 | Physical sysroot and bundled source share identical module identities. |
| UNI-D14 | Old core/std persisted identities require explicit invalidation/migration. |

---

## 38. Completion criterion

The feature is complete when the following statement is mechanically true:

> **For every Universe declaration referenced by user source, the parser/module system, linker, semantic analyzer, source index, editor query layer, compiler lowering, runtime registry, reflection layer, and LSP all converge on the same canonical source module/declaration identity, and neither `core` nor `std` is required to express or recover that identity.**
