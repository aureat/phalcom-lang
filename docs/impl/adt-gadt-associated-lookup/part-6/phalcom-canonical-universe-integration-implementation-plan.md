# Phalcom Canonical Universe Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the legacy `core` library/runtime/semantic/LSP model and the separate `std` builtin package, make `universe` the sole toolchain-owned shipped package, and make actual Universe modules/declarations authoritative for typing, navigation, hover, completion, runtime identity, and reflection.

**Architecture:** `phalcom-modules` owns one explicit Universe module graph and source provider; `phalcom-semantic` builds a reusable shallow `UniverseSemanticBaseline` from the actual modules and exposes canonical editor queries; `phalcom-core` materializes/executes canonical Universe modules instead of flattening them into core; `phalcom-lsp` becomes a protocol adapter over immutable module/semantic products. Full Universe topology is always available for discovery, while deep semantic analysis and runtime initialization remain dependency-driven.

**Tech Stack:** Rust workspace; `phalcom-modules`; `phalcom-semantic`; `phalcom-core`; `phalcom-lsp`; `phalcom-native-meta`; `phalcom-native-surface`; `phalcom-type-meta`; Phalcom `.ph` builtin sources; `cargo test`; existing semantic/LSP integration harnesses.

**Spec:** `phalcom-canonical-universe-integration-technical-spec.md`

**Repository baseline:** `49d74f9a7d95f695c8ff38c954eca938e6fec16f` (`main`, inspected 2026-09-01).

## Global Constraints

- [ ] Do not implement this as a textual `core -> universe` rename.
- [ ] Do not create a hidden module named `universe` and continue flattening all source into it.
- [ ] Do not preserve `std` as a forwarding builtin package.
- [ ] Do not make the entire Universe runtime-reachable/eager merely to support editor completion.
- [ ] Do not make the LSP parse, resolve, or infer builtin semantics independently.
- [ ] Do not allow native metadata to manufacture semantic declaration identity from a leaf name.
- [ ] Do not regress package `expose` semantics when resolving Universe paths.
- [ ] Keep local/import shadowing precedence over prelude.
- [ ] Keep ordinary user edits from rebuilding the immutable Universe baseline.
- [ ] Preserve actual source-module ownership across declarations, variants, callables, fields, diagnostics, reflection, and persisted keys.
- [ ] Implement each task test-first where practical and run the focused gate before committing.
- [ ] Before claiming completion, run the final repository-wide deletion searches and full verification gates.

---

## Task 0 — Freeze baseline and add migration guard tests

**Files:**

- Modify: `phalcom-modules/tests/builtin_provider.rs`
- Modify: `phalcom-modules/tests/universe_project_model.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/imported_resolution.rs`
- Modify: `phalcom-lsp/tests/module_navigation.rs`
- Modify: `phalcom-lsp/tests/source_text.rs`
- Add: `phalcom-semantic/tests/semantic/integration/universe_resolution.rs`

### Steps

- [ ] Record the current baseline in the branch/task notes:

```bash
git rev-parse HEAD
# Expected starting baseline or a descendant:
# 49d74f9a7d95f695c8ff38c954eca938e6fec16f
```

- [ ] Add failing tests describing the final identity rules before changing production code.

In `phalcom-semantic/tests/semantic/integration/universe_resolution.rs`, add tests named approximately:

```rust
#[test]
fn prelude_int_resolves_to_actual_universe_number_module();

#[test]
fn explicit_universe_list_import_and_prelude_use_same_declaration_identity();

#[test]
fn local_declaration_shadows_universe_prelude();

#[test]
fn universe_imported_class_participates_in_formal_inference();

#[test]
fn canonical_snapshot_contains_no_synthetic_core_declaration();
```

- [ ] In `phalcom-lsp/tests/module_navigation.rs`, replace the weak future expectation “virtual or physical core location” with a new ignored/failing target test asserting exact Universe URI and range for `Object`/`Int`.

- [ ] In `phalcom-modules/tests/builtin_provider.rs`, add future-target tests asserting there is one builtin root and former std facilities are Universe children.

- [ ] Keep these tests red until their owning tasks land; if the repository convention disallows committed red tests, create them within each task immediately before implementation rather than committing Task 0 separately.

### Focused gate

```bash
cargo test -p phalcom-modules --test builtin_provider
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-lsp --test module_navigation
```

### Commit

```bash
git commit -m "test(universe): define canonical builtin integration contract"
```

---

## Task 1 — Replace generic builtin-project identity with explicit Universe identity

**Files:**

- Modify: `phalcom-modules/src/identity.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Modify: every compile error produced by exhaustive matches on `ProjectIdentity` / `ImportRootTarget`
- Modify tests under: `phalcom-modules/tests/`

**Current code reference:** `phalcom-modules/src/identity.rs` defines `BuiltinPackage::{Universe, Std}`, `ProjectIdentity::Builtin(BuiltinPackage)`, `ImportRootTarget::Builtin(BuiltinPackage)`, `ModuleId::builtin`, and `ModuleId::core`.

### Steps

- [ ] Add the explicit owner variant first:

```rust
pub enum ProjectIdentity {
    Universe,
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}
```

- [ ] Replace:

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

- [ ] Add:

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

- [ ] Remove `BuiltinPackage` only after downstream matches compile against `ProjectIdentity::Universe`. Do not leave an enum with one member unless a short staged commit is necessary.

- [ ] Delete `ModuleId::core()`.

- [ ] Delete `ModuleId::builtin(...)` once all callers have migrated.

- [ ] Update `ProjectIdentity::{as_builtin}` users. Replace them with `is_universe()` or direct pattern matching:

```rust
pub const fn is_universe(self) -> bool {
    matches!(self, Self::Universe)
}
```

- [ ] Update `fmt::Display for ProjectIdentity` so Universe renders explicitly, e.g. `universe` or `builtin:universe`, consistently with diagnostics.

- [ ] In `phalcom-modules/src/lib.rs`, remove re-exports of `BuiltinPackage` and `builtin_module_uri`; add the new Universe URI functions in Task 3.

### Tests

Add/adjust:

```rust
#[test]
fn universe_identity_is_disjoint_from_resolved_and_synthetic();

#[test]
fn module_id_has_no_legacy_core_constructor();
```

The second is enforced primarily by deletion search rather than Rust test.

### Focused gate

```bash
cargo check -p phalcom-modules
cargo test -p phalcom-modules
```

### Commit

```bash
git commit -m "refactor(modules): make Universe the explicit builtin owner"
```

---

## Task 2 — Merge `std` topology and source mapping into Universe

**Files:**

- Modify: `phalcom-modules/src/builtin.rs`
- Modify: `phalcom-modules/src/builtin_interface.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Modify: `phalcom-core/core/universe/src/package.ph`
- Move later or now: `phalcom-core/core/std/src/**` into the Universe tree
- Modify tests: `phalcom-modules/tests/builtin_provider.rs`
- Modify tests: `phalcom-modules/tests/package_semantic_contract.rs`
- Modify tests: `phalcom-modules/tests/universe_project_model.rs`

### Steps

- [ ] Rename:

```rust
BuiltinNodeSpec
    -> UniverseNodeSpec

BuiltinProjectSourceProvider
    -> UniverseSourceProvider
```

- [ ] Replace the provider field/constructor:

```rust
pub struct UniverseSourceProvider;

impl UniverseSourceProvider {
    pub const fn new() -> Self {
        Self
    }
}
```

There is no builtin-project discriminator.

- [ ] Merge the contents of `STD_NODES` into `UNIVERSE_NODES`.

The root node's children must include both current Universe roots and former std roots:

```rust
children: &[
    "object",
    "scalar",
    "errors",
    "callable",
    "option",
    "concurrency",
    "collections",
    "reflection",
    "io",
    "fs",
    "path",
    "text",
    "regex",
    "json",
    "math",
    "random",
    "time",
    "process",
    "net",
    "concurrent",
    "testing",
],
```

Adjust kinds to match each actual `package.ph`/module layout rather than blindly preserving the old table.

- [ ] Remove `STD_NODES`.

- [ ] Rewrite `source_text` match arms so former std sources resolve from `ModuleId::universe(...)`.

If physical source movement is deferred to Task 27, it is acceptable for this task's `include_str!` to point temporarily at `phalcom-core/core/std/src/...`, but logical identity MUST already be Universe.

- [ ] Modify `phalcom-core/core/universe/src/package.ph` to `expose` the former std top-level children that are public.

- [ ] Do not automatically add former std facilities to prelude metadata.

- [ ] Update `phalcom-modules/src/lib.rs` exports:

```rust
pub use builtin::{
    UniverseNodeSpec,
    UniverseSourceProvider,
    UNIVERSE_NODES,
};
```

### Tests

Replace disjoint-builtin-project tests with:

```rust
#[test]
fn universe_provider_contains_platform_and_language_modules();

#[test]
fn universe_root_exposes_shipped_public_children();

#[test]
fn json_has_universe_identity();

#[test]
fn no_std_builtin_provider_exists();
```

### Focused gate

```bash
cargo test -p phalcom-modules --test builtin_provider
cargo test -p phalcom-modules --test package_semantic_contract
cargo test -p phalcom-modules --test universe_project_model
```

### Commit

```bash
git commit -m "refactor(universe): absorb std into the shipped Universe"
```

---

## Task 3 — Centralize canonical Universe URI encoding and decoding

**Files:**

- Modify: `phalcom-modules/src/identity.rs`
- Modify: `phalcom-modules/src/builtin.rs`
- Modify: `phalcom-modules/src/stabilization.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Add tests: `phalcom-modules/tests/universe_uri.rs`

### Steps

- [ ] Replace `builtin_module_uri` with:

```rust
pub fn universe_module_uri(
    id: &ModuleId,
) -> Option<String> {
    if !matches!(id.project, ProjectIdentity::Universe) {
        return None;
    }

    let path = id.path
        .components()
        .iter()
        .map(ModuleComponent::as_str)
        .collect::<Vec<_>>()
        .join("/");

    if path.is_empty() {
        Some("phalcom://universe/".to_string())
    } else {
        Some(format!("phalcom://universe/{path}"))
    }
}
```

- [ ] Add inverse parsing:

```rust
pub fn universe_module_from_uri(
    raw: &str,
) -> Option<ModuleId>
```

It must:

1. require scheme `phalcom`;
2. require host `universe`;
3. reject query/fragment/userinfo/ports;
4. normalize only the single canonical root form;
5. decode every path segment through `ModuleComponent::from_identifier`;
6. return `ModuleId::universe(path)`.

- [ ] Make `UniverseSourceProvider::source_id()` call the same encoder rather than formatting independently.

- [ ] Update `phalcom-modules/src/stabilization.rs` so it does not branch over former builtin package IDs.

### Tests

Add table-driven round-trip tests over every `UNIVERSE_NODES` entry:

```rust
for id in UniverseSourceProvider::new().module_ids() {
    let uri = universe_module_uri(&id).unwrap();
    assert_eq!(universe_module_from_uri(&uri), Some(id));
}
```

Add negative tests for `phalcom://core`, `phalcom://std`, malformed path segments, query/fragment variants.

### Focused gate

```bash
cargo test -p phalcom-modules --test universe_uri
cargo test -p phalcom-modules
```

### Commit

```bash
git commit -m "feat(modules): add canonical Universe URI codec"
```

---

## Task 4 — Remove `std` root injection and centralize root resolution

**Files:**

- Modify: `phalcom-modules/src/project.rs`
- Modify: `phalcom-modules/src/resolver.rs`
- Modify: `phalcom-modules/src/query.rs`
- Modify: `phalcom-modules/src/error.rs`
- Modify tests: `phalcom-modules/tests/query.rs`
- Modify tests: import/resolution suites under `phalcom-modules/tests/`

### Steps

- [ ] In both `ProjectUniverse::resolve_project_recursive` and `ProjectUniverse::load_synthetic_root`, remove creation/insertion of `std_comp`.

- [ ] Insert only:

```rust
import_roots.insert(
    ModuleComponent::from_identifier("universe")
        .expect("canonical universe root"),
    (ImportRootTarget::Universe, false),
);
```

- [ ] Update `ResolvedProject::import_roots` documentation; remove “+ core”/“+ std” wording.

- [ ] In `ModuleResolver::resolve_import_with_trace`, remove the `"std"` builtin branch.

- [ ] Keep an explicit removed-root error if desired:

```rust
if root_seg.name == "std" {
    return Err(
        ModuleResolutionError::LegacyStdImportRemoved
    );
}
```

Add the error to `phalcom-modules/src/error.rs` with actionable wording:

```text
the `std` builtin package was removed; use `universe.<path>`
```

Do not return a forwarding target.

- [ ] Replace the `"universe"` branch with `ImportRootTarget::Universe`.

- [ ] Extract root selection into one helper so `ModuleQueryFacade` and compiler code do not duplicate the string mapping.

A reasonable module-only API is:

```rust
pub fn canonical_import_root(
    universe: &ProjectUniverse,
    importer: &ModuleId,
    root: &ModuleComponent,
) -> Option<ImportRootQueryTarget>;
```

If this fits query ownership better, place it behind `ModuleQueryFacade`.

### Focused gate

```bash
cargo test -p phalcom-modules
```

### Commit

```bash
git commit -m "refactor(modules): remove std import-root identity"
```

---

## Task 5 — Make Universe package exposure validation use ordinary package interfaces

**Files:**

- Modify: `phalcom-modules/src/resolver.rs`
- Modify: `phalcom-modules/src/query.rs`
- Modify tests: `phalcom-modules/tests/universe_project_model.rs`
- Add tests as needed: `phalcom-modules/tests/universe_exposure.rs`

### Current seam

`ModuleResolver::resolve_import_with_trace` returns directly from the builtin branch after locating a node. It does not traverse package `exposed_children` using the same generic external path validation used for resolved projects.

### Steps

- [ ] Generalize:

```rust
validate_external_path_with_trace(
    target_project_id: ResolvedProjectId,
    ...
)
```

to accept owner identity:

```rust
validate_external_path_with_trace(
    target_owner: ProjectIdentity,
    path: &ModulePath,
    package_interfaces: &mut BTreeSet<ModuleId>,
)
```

- [ ] Replace `load_package_surface(ResolvedProjectId, ...)` with owner-aware loading:

```rust
fn load_package_surface(
    &mut self,
    owner: ProjectIdentity,
    path: &ModulePath,
) -> Result<PackagePathSurface, ModuleResolutionError>
```

For `ProjectIdentity::Universe`, load the interface through `UniverseSourceProvider`.

- [ ] Ensure an external import of Universe calls this validation before returning the target `SourceUnit`.

- [ ] Extend relative-import resolution so an importer whose owner is Universe derives package depth from the Universe module/interface instead of requiring `ResolvedProjectId`.

- [ ] Make `ModuleQueryFacade::external_import_children` use the same source-owned interface semantics. Do not special-case former std nodes.

### Tests

Create a fixture where a Universe child exists in `UNIVERSE_NODES` but is not exposed by the parent package and assert:

- direct provider lookup can see it internally;
- external import rejects it;
- external completion excludes it;
- relative/internal completion can see it where appropriate.

### Focused gate

```bash
cargo test -p phalcom-modules --test universe_exposure
cargo test -p phalcom-modules
```

### Commit

```bash
git commit -m "fix(modules): enforce Universe package exposure canonically"
```

---

## Task 6 — Build a complete immutable Universe source/interface catalog

**Files:**

- Modify: `phalcom-modules/src/builtin.rs`
- Add: `phalcom-modules/src/universe_catalog.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Add tests: `phalcom-modules/tests/universe_catalog.rs`

### Steps

- [ ] Add a module-only immutable catalog:

```rust
#[derive(Clone, Debug)]
pub struct UniverseModuleCatalog {
    pub sources:
        BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub locations:
        BTreeMap<ModuleId, SourceLocation>,
    pub unlinked:
        BTreeMap<ModuleId, UnlinkedModuleInterface>,
}
```

- [ ] Add constructor:

```rust
impl UniverseModuleCatalog {
    pub fn build(
        provider: &UniverseSourceProvider,
    ) -> Result<Self, ModuleLoadError>;
}
```

- [ ] Build it by enumerating all `UNIVERSE_NODES`, preserving exact module ID, kind, source location, text, and interface.

- [ ] Do not link everything into the executable `LinkedProgram` here. This is the complete discovery catalog.

- [ ] Provide direct immutable query helpers by `ModuleId`.

- [ ] Ensure source location uses the canonical URI codec for bundled sources.

### Tests

- [ ] Number of catalog entries equals number of unique `UNIVERSE_NODES`.
- [ ] Every node has parseable source and interface.
- [ ] Every source's `ParsedModuleUnit.id` equals the catalog key.
- [ ] Former std modules are present under Universe IDs.
- [ ] No key has a path component `core` or owner `std`.

### Focused gate

```bash
cargo test -p phalcom-modules --test universe_catalog
```

### Commit

```bash
git commit -m "feat(modules): publish complete immutable Universe catalog"
```

---

## Task 7 — Add canonical native-key-to-source-declaration ownership

**Files:**

- Modify: `phalcom-native-meta/src/universe.rs`
- Add/modify: `phalcom-semantic/src/universe/catalog.rs`
- Modify tests: native metadata/conformance tests in `phalcom-semantic`

### Steps

- [ ] Keep `UniverseKey`; do not make it a semantic declaration ID.

- [ ] If source ownership can be declared statically without duplication drift, extend `UniverseBindingSpec` with a compact source anchor:

```rust
pub struct UniverseSourceAnchor {
    pub module_path: &'static [&'static str],
    pub declaration: &'static str,
}
```

and:

```rust
pub source: Option<UniverseSourceAnchor>
```

Use `None` for runtime-support-only classes that intentionally have no semantic declaration.

- [ ] Alternatively, if source-derived discovery is preferred, construct the mapping by scanning declaration shells and validate a small authoritative `UniverseKey -> expected module path` table. Do not resolve by leaf name globally.

- [ ] In `phalcom-semantic/src/universe/catalog.rs`, add:

```rust
pub struct UniverseDeclarationCatalog {
    by_native_key:
        BTreeMap<UniverseKey, DeclarationId>,
    ...
}
```

- [ ] Validate exact source declaration kind/name for every semantic `UniverseBindingKind::Class`.

- [ ] Validate `RuntimeSupportClass` does not create a declaration.

### Tests

```rust
#[test]
fn every_semantic_universe_key_maps_to_one_actual_source_declaration();

#[test]
fn runtime_support_classes_do_not_create_semantic_declarations();
```

### Focused gate

```bash
cargo test -p phalcom-native-meta
cargo test -p phalcom-semantic native
```

### Commit

```bash
git commit -m "feat(semantic): bind native Universe keys to source declarations"
```

---

## Task 8 — Create `UniverseSemanticBaseline`

**Files:**

- Add: `phalcom-semantic/src/universe/mod.rs`
- Add: `phalcom-semantic/src/universe/baseline.rs`
- Add: `phalcom-semantic/src/universe/catalog.rs`
- Add: `phalcom-semantic/src/universe/prelude.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-semantic/src/declarations.rs`
- Modify tests: add `phalcom-semantic/tests/semantic/universe_baseline.rs`

### Steps

- [ ] Define `UniverseSemanticBaseline` as described in the technical spec.

- [ ] Implement build phases explicitly; do not hide the cycle inside one giant function.

Recommended internal functions:

```rust
fn load_universe_sources(...)
    -> UniverseModuleCatalog;

fn build_universe_declaration_shells(...)
    -> DeclarationShellTable;

fn build_universe_declaration_catalog(...)
    -> UniverseDeclarationCatalog;

fn build_universe_nominal_types(...)
    -> DeclarationTypeTable;

fn build_universe_hierarchy(...)
    -> MapTypeHierarchy;

fn build_universe_signatures_and_surfaces(...)
    -> ...;

fn build_universe_source_index(...)
    -> SourceSemanticIndex;

fn build_universe_prelude(...)
    -> PreludeBindings;
```

- [ ] Change `bootstrap_universe_declarations` so it accepts exact declaration IDs/catalog entries, or retire it in favor of source-shell-driven nominal allocation.

It MUST NOT accept a closure that manufactures IDs from `UniverseKey::name()` plus one module.

- [ ] Store parsed source units and source locations in the baseline.

- [ ] Do not perform deep callable-body analysis across all modules in this constructor.

### Tests

Assert exact owners for a representative cross-section:

```text
Object      -> universe.object.object
Int         -> universe.scalar.number
List        -> universe.collections.list
Option      -> universe.option.option
Error       -> universe.errors.error
```

Use provider/source catalog lookup to avoid duplicating paths in production code.

### Focused gate

```bash
cargo test -p phalcom-semantic --test universe_baseline
```

### Commit

```bash
git commit -m "feat(semantic): build shallow semantics from real Universe source"
```

---

## Task 9 — Replace `CoreDeclarationIds` with source-derived `UniverseDeclarationIds`

**Files:**

- Rename directory: `phalcom-semantic/src/core_surface/` -> `phalcom-semantic/src/universe_surface/`
- Modify: `phalcom-semantic/src/universe_surface/identity.rs`
- Modify: `phalcom-semantic/src/universe_surface/source.rs`
- Modify: `phalcom-semantic/src/universe_surface/merge.rs`
- Modify: `phalcom-semantic/src/universe_surface/conformance.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Update every import of `core_surface`

### Steps

- [ ] Rename `CoreDeclarationIds` -> `UniverseDeclarationIds`.

- [ ] Delete `impl Default` that constructs all IDs under `ModuleId::core()`.

- [ ] Add construction from canonical catalog:

```rust
impl UniverseDeclarationIds {
    pub fn from_catalog(
        catalog: &UniverseDeclarationCatalog,
    ) -> Result<Self, UniverseCatalogError>;
}
```

- [ ] Preserve helpers:

```rust
is_object
is_callable_supertype
is_option
is_result
is_ordering
```

but compare real IDs.

- [ ] Rename `is_core_adt` -> `is_universe_adt` or a more semantic name such as `is_builtin_native_adt`.

- [ ] Update module docs/comments from “core surface” to “Universe surface”.

### Focused gate

```bash
cargo check -p phalcom-semantic
cargo test -p phalcom-semantic core_surface
cargo test -p phalcom-semantic universe
```

### Commit

```bash
git commit -m "refactor(semantic): rename core surface to canonical Universe surface"
```

---

## Task 10 — Make native/source merge consume exact canonical owners and support enums

**Files:**

- Modify: `phalcom-semantic/src/universe_surface/merge.rs`
- Modify: `phalcom-semantic/src/universe_surface/source.rs`
- Modify: `phalcom-semantic/src/universe_surface/conformance.rs`
- Modify native conformance tests

### Steps

- [ ] Replace the current native owner construction:

```rust
DeclarationId::new(
    ModuleId::core(),
    owner_name.into(),
)
```

with lookup by `UniverseKey` through `UniverseDeclarationCatalog`.

- [ ] Change `merge_surfaces` signature to accept owner catalog.

- [ ] Generalize `MergedClassSurface` to a declaration-kind-neutral representation or add `MergedEnumSurface`.

- [ ] Ensure `SourceDeclarationRecord::Enum` flows into native merge and conformance.

- [ ] Native-only records whose owner has no semantic declaration MUST be explicitly classified as runtime-support/generated implementation; they must not silently create a new source-semantic class.

### Tests

- [ ] Native `Int` method merges into actual `universe.scalar.number::Int`.
- [ ] Native enum variant/method surface merges into actual canonical enum where applicable.
- [ ] Unknown native owner is a deterministic conformance error.
- [ ] Runtime-support class does not appear as a source declaration.

### Focused gate

```bash
cargo test -p phalcom-semantic native_conformance
cargo test -p phalcom-semantic adt
```

### Commit

```bash
git commit -m "fix(semantic): merge native surfaces by canonical Universe identity"
```

---

## Task 11 — Replace `LinkedTypeResolver` prelude-module fallback with canonical prelude bindings

**Files:**

- Modify: `phalcom-semantic/src/resolver.rs`
- Modify: `phalcom-semantic/src/universe/prelude.rs`
- Update all `LinkedTypeResolver::new(...)` call sites
- Add tests: `phalcom-semantic/tests/semantic/integration/universe_resolution.rs`

### Steps

- [ ] Change struct field:

```rust
prelude_module: ModuleId
```

to:

```rust
prelude: Arc<PreludeBindings>
```

- [ ] Change constructor signature accordingly.

- [ ] Keep resolution order:

1. current module declaration;
2. explicit selective import binding;
3. current module re-export if relevant;
4. prelude.

- [ ] Replace prelude branch with direct target lookup.

- [ ] Delete:

```rust
let core_decl =
    DeclarationId::new(ModuleId::core(), root.into());
```

and its fallback.

- [ ] Add tests proving:

```text
local Int wins over prelude Int
imported Int alias wins according to normal binding rules
unqualified Int targets actual Universe Int otherwise
```

### Focused gate

```bash
cargo test -p phalcom-semantic universe_resolution
cargo test -p phalcom-semantic imported_resolution
```

### Commit

```bash
git commit -m "fix(semantic): resolve prelude names to canonical Universe declarations"
```

---

## Task 12 — Integrate the Universe baseline into `SemanticWorkspaceSession`

**Files:**

- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-semantic/src/workspace.rs` if input composition needs extension
- Modify: `phalcom-semantic/src/db/**` only where query inputs require a stable baseline identity
- Add tests: `phalcom-semantic/tests/semantic/incremental_universe_baseline.rs`

### Steps

- [ ] Add session field:

```rust
universe:
    Arc<UniverseSemanticBaseline>,
```

- [ ] Remove synthetic initialization in `with_workspace()`:

```rust
bootstrap_universe_declarations(... ModuleId::core ...)
UNIVERSE_CLASS_RELATIONS -> ModuleId::core()
dummy LinkedProgram entry ModuleId::core()
register_native_surfaces(... ModuleId::core() ...)
```

- [ ] Construct/reuse `UniverseSemanticBaseline` once.

For testability, consider:

```rust
pub fn with_workspace_and_universe(
    workspace: WorkspaceId,
    universe: Arc<UniverseSemanticBaseline>,
) -> Self
```

while `with_workspace` uses the canonical process baseline.

- [ ] Replace fields:

```text
base_declarations
base_hierarchy
base_dispatch
base_callable_signatures
```

with either baseline references or clearly named projected clones.

- [ ] During `update`, start workspace semantic products from the baseline's real declaration/hierarchy/signature products.

- [ ] Make the combined `known_declarations` include Universe declarations and project declarations.

- [ ] Ensure project edits invalidate only project-dependent query keys, not the immutable Universe baseline.

- [ ] Extend `SemanticUpdateStats` if useful with an invariant/test counter that Universe baseline recomputation is zero for normal user edits.

### Tests

Run two user revisions and assert:

```rust
assert_eq!(
    snapshot1.declarations.form(&universe_int),
    snapshot2.declarations.form(&universe_int)
);
assert_eq!(
    snapshot1.store.id(),
    snapshot2.store.id()
);
```

and baseline source fingerprints remain unchanged/not recomputed.

### Focused gate

```bash
cargo test -p phalcom-semantic incremental_universe_baseline
cargo test -p phalcom-semantic
```

### Commit

```bash
git commit -m "refactor(semantic): compose workspaces over immutable Universe baseline"
```

---

## Task 13 — Delete generated aggregate core presentation source

**Files:**

- Delete: `phalcom-semantic/src/universe_surface/presentation.rs` if it exists only for aggregate core source
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/snapshot.rs`
- Update tests that reference `render_canonical_core_source`

### Steps

- [ ] Delete import/use of:

```rust
render_canonical_core_source
```

- [ ] Delete the branch that:

1. creates `let core = ModuleId::core()`;
2. renders generated source;
3. parses it;
4. builds `SourceSemanticIndex` for the synthetic module;
5. inserts it into `presentation_sources`.

- [ ] Retain `presentation_sources` only if other genuinely generated semantic documents use it. If no production caller remains, remove the field/API completely.

- [ ] Ensure the baseline's actual Universe source shards are in the source index.

### Tests

- [ ] `editor.definition_sites(Int)` returns the actual source declaration.
- [ ] Snapshot source/index module keys contain real Universe modules.
- [ ] No source/index key represents `universe.core`.
- [ ] No `Canonical Core Surface` generated text exists.

### Focused gate

```bash
cargo test -p phalcom-semantic builtin_declaration_has_canonical_definition_site
cargo test -p phalcom-semantic universe
```

### Commit

```bash
git commit -m "refactor(semantic): remove synthetic core presentation document"
```

---

## Task 14 — Index import/re-export/export/expose path segments as semantic targets

**Files:**

- Modify: `phalcom-semantic/src/source_index/builder.rs`
- Modify: `phalcom-semantic/src/source_index/occurrence.rs`
- Modify: `phalcom-semantic/src/source_index/site.rs`
- Modify: `phalcom-semantic/src/source_index/mod.rs`
- Modify source-index context construction in `phalcom-semantic/src/session.rs`
- Add tests: `phalcom-semantic/tests/semantic/integration/module_syntax_targets.rs`

### Steps

- [ ] Add source-site kinds for module path segments where the existing `SourceSiteKind::Module` is insufficient to distinguish occurrence versus module source definition.

A possible representation:

```rust
pub enum SourceSiteKind {
    ...
    ModulePathSegment {
        target: ModuleId,
    },
    ImportBinding,
    ExportReference,
    ExposeChild,
}
```

Prefer target storage in the existing target map rather than duplicating it in enum payload if that matches current source-index design.

- [ ] Extend `SourceIndexContext` with resolved path-segment information:

```rust
resolved_path_segments:
    BTreeMap<SourceSiteId, ModuleId>
```

or an importer/range-based structure populated by canonical module resolution.

- [ ] While indexing:

```phalcom
import universe.collections.list
```

emit one occurrence per segment and target the prefix module identity.

- [ ] For selective imports, resolve exported item through linked export target to `SemanticTargetId::Declaration`/other canonical target.

- [ ] For aliases, index alias declaration/use as references to the external target while preserving local binding metadata.

- [ ] For re-export/export items, target canonical exported binding/declaration.

- [ ] For `expose`, target direct child `ModuleId`.

### Tests

Assert exact `target_at` values at each token offset for:

```phalcom
import universe.collections.list
from universe.collections.list import List as L
export L
expose .parser
```

### Focused gate

```bash
cargo test -p phalcom-semantic module_syntax_targets
cargo test -p phalcom-semantic imported_resolution
```

### Commit

```bash
git commit -m "feat(semantic): index canonical module syntax targets"
```

---

## Task 15 — Extend `ModuleQueryProducts` with full Universe topology and source anchors

**Files:**

- Modify: `phalcom-semantic/src/snapshot.rs`
- Modify: `phalcom-modules/src/query.rs`
- Modify: session publication construction in `phalcom-semantic/src/session.rs`
- Add tests: `phalcom-semantic/tests/semantic/integration/module_queries.rs`

### Steps

- [ ] When publishing `ModuleQueryProducts`, merge immutable Universe catalog maps with workspace module products.

- [ ] Do not limit `unlinked`/source topology to runtime reachable linked modules.

- [ ] In `ModuleQueryFacade::import_root_entries`, remove hardcoded `std`; return Universe plus project/dependency roots.

- [ ] Add module-only query helpers as needed:

```rust
pub fn module_kind(
    &self,
    module: &ModuleId,
) -> Option<ModuleKind>;

pub fn module_source(
    &self,
    module: &ModuleId,
) -> Option<&SourceLocation>;

pub fn export_target(
    &self,
    module: &ModuleId,
    name: &str,
) -> Option<&LinkedExportTarget>;
```

- [ ] Ensure `module_children(ProjectIdentity::Universe, ...)` sees all Universe nodes even if no user module imports them.

- [ ] Ensure `external_import_children` follows `exposed_children`.

### Tests

- [ ] `import universe.j|` query can discover JSON with a user program that imports no Universe modules explicitly.
- [ ] A non-exposed child does not appear externally.
- [ ] Relative/internal child query still sees legal internal child.

### Focused gate

```bash
cargo test -p phalcom-semantic module_queries
cargo test -p phalcom-modules query
```

### Commit

```bash
git commit -m "feat(semantic): publish complete Universe module query products"
```

---

## Task 16 — Add declaration/module presentations and definition anchors to `EditorSemanticQuery`

**Files:**

- Modify: `phalcom-semantic/src/editor.rs`
- Modify: `phalcom-semantic/src/presentation.rs` or presentation module files
- Modify: `phalcom-semantic/src/snapshot.rs`
- Add tests: `phalcom-semantic/tests/semantic/editor_universe.rs`

### Steps

- [ ] Add a declaration presentation API using existing `TypePresenter`, declaration table, generic signatures, hierarchy, and source index.

- [ ] Add a module/package presentation API using `ModuleQueryFacade` products.

- [ ] Add:

```rust
pub fn definition_anchors(
    &self,
    target: &SemanticTargetId,
) -> Vec<SourceAnchor>
```

that:

1. uses exact declaration/callable/field/variant definition sites when available;
2. returns module/package source anchor for `SemanticTargetId::Module`;
3. never synthesizes a core location.

- [ ] Keep `definition_sites` if useful internally, but make LSP consume the richer source-anchor query.

- [ ] Add a target-presentation enum so hover can dispatch by semantic target without AST-specific logic.

### Tests

- [ ] `Int` presentation includes actual module and formal type.
- [ ] `List<T>` presentation includes generic shape.
- [ ] `universe.collections` presentation is Package.
- [ ] `universe.collections.list` presentation is Module.
- [ ] definition anchor range for `Int` is exact declaration range.

### Focused gate

```bash
cargo test -p phalcom-semantic editor_universe
```

### Commit

```bash
git commit -m "feat(semantic): expose Universe-aware editor presentations"
```

---

## Task 17 — Include prelude bindings in visible-symbol completion with correct shadowing

**Files:**

- Modify: `phalcom-semantic/src/editor.rs`
- Modify: `phalcom-semantic/src/universe/prelude.rs`
- Modify tests: `phalcom-semantic/tests/semantic/editor_universe.rs`
- Modify LSP completion only for protocol mapping if necessary

### Steps

- [ ] In `EditorSemanticQuery::visible_symbols_at`, after collecting lexical/module/import symbols, append prelude candidates whose names are not already bound.

- [ ] Each prelude candidate targets the actual canonical declaration.

- [ ] Give prelude candidates a source site or optional declaration-site representation that points to the Universe definition, not a fake local binding.

If `VisibleSymbol` currently requires a local `declaration_site`, change it to:

```rust
pub struct VisibleSymbol {
    pub name: Box<str>,
    pub target: SemanticTargetId,
    pub local_declaration_site: Option<SourceSiteId>,
}
```

or introduce a dedicated visibility-origin enum.

- [ ] Add origin metadata if useful:

```rust
pub enum VisibleSymbolOrigin {
    Lexical,
    Module,
    Import,
    Prelude,
}
```

### Tests

- [ ] `Int` appears in user-module completion.
- [ ] local `Int` suppresses prelude `Int`.
- [ ] explicit imported alias suppresses same-name prelude.
- [ ] target is canonical Universe declaration.

### Focused gate

```bash
cargo test -p phalcom-semantic editor_universe
cargo test -p phalcom-lsp completion
```

### Commit

```bash
git commit -m "feat(semantic): expose canonical prelude symbols to editor queries"
```

---

## Task 18 — Remove LSP aggregate core source transport

**Files:**

- Delete: `phalcom-lsp/src/core_documents.rs`
- Modify: `phalcom-lsp/src/lib.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/request_context.rs`
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/tests/source_text.rs`
- Modify VS Code extension source provider only if it special-cases `phalcom://core`

### Steps

- [ ] Delete `CORE_MODULE_URI`, `CoreSource`, `canonical_universe_source()`.

- [ ] Remove `compiler_module_for_uri` special case:

```rust
if uri.as_str() == CORE_MODULE_URI {
    ...
}
```

- [ ] Remove `compiler_uri_for_module` special case for `ModuleId::core()`.

- [ ] Replace `builtin_module_from_uri` in `analysis_service.rs` with a thin call to:

```rust
phalcom_modules::universe_module_from_uri(uri.as_str())
```

Rename the helper to `universe_module_from_uri` or delete the local wrapper.

- [ ] Rename/remove `WorkspaceScanRequest.core_source_path`. If physical override remains, use:

```rust
pub universe_source_root: Option<PathBuf>
```

and define it as a tree root, not one source file.

- [ ] Rename/remove `AnalysisEvent::CoreSourceSelected`.

- [ ] Implement source-text requests for exact Universe URIs by reading the immutable semantic snapshot source or the canonical provider by module ID.

- [ ] No LSP request concatenates sources.

### Tests

Replace:

```text
phalcom://core
```

source-text tests with exact module requests such as:

```text
phalcom://universe/scalar/number
phalcom://universe/collections/list
```

and compare returned source to provider module text.

### Focused gate

```bash
cargo test -p phalcom-lsp --test source_text
cargo check -p phalcom-lsp
```

### Commit

```bash
git commit -m "refactor(lsp): remove synthetic core virtual document"
```

---

## Task 19 — Make LSP go-to-definition entirely semantic-target driven

**Files:**

- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/request_context.rs`
- Modify: `phalcom-lsp/tests/module_navigation.rs`
- Modify: `phalcom-lsp/tests/stage2_index.rs`

### Steps

- [ ] Delete `compiler_import_definition_location()` after Task 14 path-segment targets are available.

- [ ] In `goto_definition`:

1. resolve request URI to canonical module;
2. convert position to offset;
3. call `snapshot.editor().target_at(module, offset)`;
4. call `snapshot.editor().definition_anchors(&target)`;
5. convert each anchor's module to a URI through snapshot source provenance / Universe URI codec;
6. convert exact source range with target document line index.

- [ ] Do not use `Range::default()` for module path navigation when a canonical source anchor exists.

- [ ] Do not parse the import AST again in LSP to determine target.

### Tests

Add exact assertions:

```text
Object -> phalcom://universe/object/object
Int -> phalcom://universe/scalar/number
List import -> phalcom://universe/collections/list
```

Also test each path segment of `universe.collections.list`.

### Focused gate

```bash
cargo test -p phalcom-lsp --test module_navigation
cargo test -p phalcom-lsp --test stage2_index
```

### Commit

```bash
git commit -m "fix(lsp): navigate Universe targets through semantic identity"
```

---

## Task 20 — Make hover target-driven for declarations, modules, and prelude names

**Files:**

- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/hover.rs`
- Modify tests: hover integration suites
- Add: `phalcom-lsp/tests/universe_hover.rs`

### Steps

- [ ] At hover position, resolve `SemanticTargetId` first.

- [ ] Dispatch to `EditorSemanticQuery::target_presentation`.

- [ ] Map protocol-neutral presentation to Markdown.

- [ ] Preserve existing callable formal signature rendering and native documentation supplementation.

- [ ] Add declaration rendering for:

```text
class/enum declaration
formal type
kind/generic signature
supertype
module
native/source provenance
Phaldoc
```

- [ ] Add module/package rendering.

- [ ] Delete any legacy “builtin core class” name-based hover path made redundant by semantic targets.

### Tests

- [ ] Prelude `Int` hover identifies `universe.scalar.number`.
- [ ] Explicitly imported `Int` hover matches prelude hover semantically.
- [ ] `List<T>` shows generic formal information.
- [ ] `universe.collections` path segment hover identifies Package.
- [ ] `universe.collections.list` identifies Module.

### Focused gate

```bash
cargo test -p phalcom-lsp --test universe_hover
cargo test -p phalcom-lsp hover
```

### Commit

```bash
git commit -m "feat(lsp): hover canonical Universe declarations and modules"
```

---

## Task 21 — Complete import, export, expose, and package-path completion

**Files:**

- Modify: `phalcom-lsp/src/import_completion.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Prefer adding compiler-owned candidate APIs in: `phalcom-semantic/src/editor.rs`
- Add/modify LSP import completion tests

### Steps

- [ ] Replace/extend `ImportContext`:

```rust
pub enum ModuleSyntaxCompletionContext {
    ImportRoot { partial: String },
    ImportChild { ... },
    RelativeImportChild { ... },
    SelectiveImport { ... },
    ExportBinding { partial: String },
    ExposeChild { ... },
}
```

- [ ] Stop representing `expose` as generic `RelativeChild`.

- [ ] Prefer parser/source-index structural context over line-prefix string splitting where recovered AST data can identify the syntax reliably. If text-prefix detection remains for incomplete syntax, it must only classify context; semantic candidate selection remains compiler-owned.

- [ ] `ImportRoot`: use canonical query roots; `std` and `core` absent.

- [ ] `ImportChild`: use `external_import_children` for external Universe/dependency roots and `module_children` for self.

- [ ] `SelectiveImport`: use linked public exports and semantic target kinds.

Replace:

```rust
LinkedExportTarget::Binding(_) =>
    CompletionItemKind::CLASS
```

with a semantic kind mapping returned by the compiler.

- [ ] `ExportBinding`: query legal current module namespace targets. Exclude already-exported names.

- [ ] `ExposeChild`: query direct child modules/packages, only if the current source unit is a package; optionally exclude already-exposed children.

- [ ] Map compiler candidate kinds to LSP `CompletionItemKind` only in LSP.

### Tests

Cover:

```phalcom
import |
import universe.|
import universe.collections.|
from universe.collections.list import |
export |
expose .|
```

Also assert:

```text
std absent
core absent
json appears under universe
non-exposed child absent externally
completion kinds are correct
```

### Focused gate

```bash
cargo test -p phalcom-lsp import_completion
cargo test -p phalcom-lsp --test integration
```

### Commit

```bash
git commit -m "feat(lsp): complete canonical module syntax completion"
```

---

## Task 22 — Fix semantic diagnostic ownership while core IDs disappear

**Files:**

- Modify: `phalcom-semantic/src/diagnostic.rs`
- Modify checker call sites under: `phalcom-semantic/src/checker/**`
- Modify: `phalcom-lsp/src/diagnostics.rs`
- Add/modify tests: semantic diagnostic ownership suites

### Steps

- [ ] Find every constructor that defaults a diagnostic module:

```bash
rg 'SemanticDiagnostic::error|ModuleId::core\(\)' phalcom-semantic/src
```

- [ ] Change constructors so source owner is required:

```rust
pub fn error(
    module: ModuleId,
    range: SourceRange,
    code: DiagnosticCode,
    message: impl Into<String>,
) -> Self
```

or use `SemanticSourceSpan` directly.

- [ ] Thread `CheckingContext.current_module` / source owner to each checker diagnostic creation site.

- [ ] Remove `ModuleId::core()` from LSP diagnostic tests; use actual fixture modules.

### Tests

- [ ] User source mismatch reports user module.
- [ ] Universe source validation error reports exact Universe module in validation-mode tests.
- [ ] Related information across modules maps exact URIs.

### Focused gate

```bash
cargo test -p phalcom-semantic diagnostic
cargo test -p phalcom-lsp diagnostics
```

### Commit

```bash
git commit -m "fix(semantic): require exact module ownership for diagnostics"
```

---

## Task 23 — Replace `install_core()` with primordial allocation plus canonical Universe materialization

**Files:**

- Modify: `phalcom-core/src/vm/bootstrap.rs`
- Modify: `phalcom-core/src/vm/mod.rs`
- Modify: `phalcom-core/src/modules/builtin_materialize.rs`
- Modify: `phalcom-core/src/modules/materialize.rs`
- Modify: `phalcom-core/src/universe/core_classes.rs`
- Modify: `phalcom-core/src/heap/mod.rs` and/or module containing `CORE_MODULE_NAME`
- Add runtime tests under `phalcom-core/tests/`

### Steps

- [ ] Split current `install_core()` responsibilities into explicit phases.

Suggested functions:

```rust
fn allocate_primordial_classes(
    &mut self,
) -> PrimordialUniverseState;

fn materialize_universe_modules(
    &mut self,
    catalog: &UniverseModuleCatalog,
) -> PhResult<UniverseRuntimeModules>;

fn bind_primordial_declarations(
    &mut self,
    modules: &UniverseRuntimeModules,
) -> PhResult<()>;

fn execute_required_universe_initializers(
    &mut self,
    modules: &UniverseRuntimeModules,
    ...
) -> PhResult<()>;
```

- [ ] Delete creation of:

```rust
ModuleId::core()
CORE_MODULE_NAME
"<internal core module>"
```

- [ ] `RuntimeRoots` becomes:

```rust
pub struct RuntimeRoots {
    pub universe: ObjRef,
    pub entry: Option<ObjRef>,
}
```

or remove `universe` if `module_registry[ModuleId::universe_root()]` is sufficient everywhere.

- [ ] Materialize one `ModuleObject` for every required Universe module using actual `ModuleId`.

Prefer reusing/generalizing `materialize_program` phases instead of creating a second module object implementation.

- [ ] Mark Universe module objects privileged by owner identity:

```rust
matches!(id.project, ProjectIdentity::Universe)
```

- [ ] Bind primordial ClassIds into the slot of their canonical declaration-owning module, using the semantic/native source ownership map.

- [ ] Root Universe exports should point to those canonical slots/modules rather than storing duplicate class-owned identities.

### Tests

- [ ] No runtime module registry entry uses a `core` path.
- [ ] `Int` class `ClassKey.module` is the runtime module for `universe.scalar.number`.
- [ ] `List` class owner is `universe.collections.list`.
- [ ] Universe root package exists.
- [ ] Kernel invariants remain green.

### Focused gate

```bash
cargo test -p phalcom-core universe
cargo test -p phalcom-core invariants
```

### Commit

```bash
git commit -m "refactor(vm): materialize primordial classes into canonical Universe modules"
```

---

## Task 24 — Execute Universe source in its actual module instead of flattening it

**Files:**

- Modify: `phalcom-core/src/vm/bootstrap.rs`
- Modify: `phalcom-core/src/modules/builtin_materialize.rs`
- Modify: `phalcom-core/src/modules/registry.rs` if module state transitions need reuse
- Add tests: runtime module ownership/source provenance tests

### Steps

- [ ] Delete `run_universe_modules(&NativeSourceIndex)` implementation that uses one `core_module`.

- [ ] Replace it with a loop over canonical module IDs in dependency/topological order.

For each parsed source:

```rust
let record = self.module_registry
    .get(&parsed.id)
    .expect("Universe module materialized");

let module = record.object;

let source_id = self.heap
    .module_mut(module)
    .push_source(parsed.text.clone());

let closure = self.compile_ast_as(
    module,
    source_id,
    (*parsed.program).clone(),
    UnitKind::File,
)?;

self.run_in_module(module, closure)?;
```

- [ ] Derive order from linked/runtime dependencies rather than provider enumeration when source initialization order matters.

- [ ] Ensure package files execute in their own package objects.

- [ ] Ensure source-authored classes bind into their own module namespaces and do not collide merely by leaf name.

### Tests

Use two Universe modules with same test-only declaration name where feasible, or unit-test module/class registry ownership directly.

### Focused gate

```bash
cargo test -p phalcom-core modules
cargo test -p phalcom-core universe
```

### Commit

```bash
git commit -m "fix(vm): execute Universe source in canonical modules"
```

---

## Task 25 — Rehome semantic roots and `None` invariants away from core globals

**Files:**

- Modify: `phalcom-core/src/vm/bootstrap.rs`
- Modify: `phalcom-core/src/vm/mod.rs`
- Modify: runtime ADT/Option bootstrap files as needed
- Modify runtime invariant tests

### Steps

- [ ] Replace all reads such as:

```rust
let core = vm.core_module()?;
vm.heap.module(core).get(...)
```

for:

```text
unsupported
ellipsis
Ordering
None
```

with canonical module/declaration slot lookup.

- [ ] Add helper:

```rust
fn universe_binding_value(
    &self,
    declaration: &Stable/RuntimeDeclarationKey,
) -> Option<Value>
```

or use exact module + symbol from the native/source catalog.

- [ ] Keep `SemanticRoots` as a cache only; populate it from canonical Universe modules.

- [ ] Update the `None` global invariant to inspect the canonical owner/export binding and verify immediate `Value::none()`.

### Focused gate

```bash
cargo test -p phalcom-core absence
cargo test -p phalcom-core invariants
```

### Commit

```bash
git commit -m "fix(vm): source semantic roots from canonical Universe bindings"
```

---

## Task 26 — Lower prelude reads to canonical linked bindings and delete runtime core fallback

**Files:**

- Modify compiler lowering files under `phalcom-core/src/compiler/`
- Modify: `phalcom-core/src/modules/compile.rs`
- Modify: `phalcom-core/src/modules/materialize.rs`
- Modify: `phalcom-core/src/vm/dispatch.rs`
- Modify: `phalcom-core/src/vm/mod.rs`
- Modify global-cache tests and relevant language fixtures

### Steps

- [ ] First locate every legacy runtime fallback:

```bash
rg 'core_module\(|prelude_names|core module|core-module fallback' \
  phalcom-core/src
```

- [ ] Add an explicit prelude-linked-read input to compilation.

The analyzer/lowering should provide name -> canonical `SymbolId` for every prelude binding referenced by a module.

- [ ] When compiling an unqualified global read that resolves semantically to a prelude declaration, emit/use a linked binding read rather than `Bytecode::GetGlobal` name fallback.

If the existing `GetLinked` opcode is the established lowering path, reuse it.

- [ ] Thread prelude linked reads into `CompiledModule.linked_reads`.

- [ ] In `modules/materialize.rs`, materialize these reads exactly like explicit imported `LinkedReadSpec::Binding`.

- [ ] Delete VM field:

```rust
pub prelude_names: HashSet<Symbol>
```

- [ ] Delete bootstrap population of `prelude_names`.

- [ ] In `vm/dispatch.rs`, remove `GetGlobal` fallback to core.

After migration, `GetGlobal` is module-local global lookup only.

- [ ] Remove the special global-cache semantics that existed solely to cache a core fallback; retain ordinary local-global cache semantics.

### Negative control

Temporarily disable prelude-linked-read emission. Tests using bare `Int`, `List`, `Object`, etc. from a user module must fail. Restore implementation and verify green.

### Focused gate

```bash
cargo test -p phalcom-core prelude
cargo test -p phalcom-core ic_global_cache
cargo test -p phalcom-core --test lang
```

### Commit

```bash
git commit -m "refactor(compiler): lower prelude names as canonical linked reads"
```

---

## Task 27 — Remove superclass and `SuperSend` core-name fallback

**Files:**

- Modify: `phalcom-core/src/compiler/class_decl.rs`
- Modify: `phalcom-core/src/compiler/attributes.rs`
- Modify: compiler scope/lowering files that encode `SuperSend`
- Modify: `phalcom-core/src/vm/dispatch.rs`
- Modify: `phalcom-core/src/vm/mod.rs`
- Add/update inheritance tests

### Steps

- [ ] Search exact remaining fallback sites:

```bash
rg 'core_module\(|ClassKey.*core|SuperSend|class_parents' \
  phalcom-core/src/compiler phalcom-core/src/vm
```

- [ ] Superclass resolution should consume canonical semantic/module binding identity instead of “own module then core by name.”

- [ ] Default superclass `Object` must be an explicit canonical Universe declaration/class target.

- [ ] Change the compile-time representation carried by `SuperSend` if necessary so runtime dispatch can identify the lexically defining class without name lookup across modules.

Possible representations:

1. compiled module + class symbol, where the module is exact and no fallback occurs;
2. a runtime class descriptor/index resolved during materialization.

- [ ] Delete VM fallback:

```text
ClassKey { current_module, name }
or
ClassKey { core_module, name }
```

for `SuperSend`.

- [ ] Preserve correct lexical-super semantics for inherited methods.

### Tests

- [ ] User class with same leaf name as Universe class does not acquire Universe superclass/privilege.
- [ ] Three-level inherited `super` case remains correct.
- [ ] Explicit Universe superclass resolves.
- [ ] Default `Object` superclass is canonical Universe Object.

### Focused gate

```bash
cargo test -p phalcom-core inheritance
cargo test -p phalcom-core super
```

### Commit

```bash
git commit -m "fix(compiler): resolve superclass and super by canonical identity"
```

---

## Task 28 — Update standalone/module compiler paths to Universe-only imports

**Files:**

- Modify: `phalcom-core/src/modules/compile.rs`
- Modify compiler/module tests

### Steps

- [ ] Remove imports of `BuiltinPackage` and `BuiltinProjectSourceProvider`.

- [ ] Replace with `UniverseSourceProvider` and explicit Universe identity.

- [ ] Change `ProgramCompileError::StandaloneImportRequiresPackageContext` text from:

```text
'universe' and 'std'
```

to only Universe.

- [ ] In `analyze_standalone_module`, remove:

```rust
root.name == "std" => ...
```

- [ ] Prefer central `ModuleResolver`/Universe catalog APIs rather than manually repeating builtin root resolution.

- [ ] Ensure inline/standalone semantic analysis receives the canonical Universe semantic baseline even when the user's `LinkedProgram` contains only the synthetic entry module.

### Tests

- [ ] standalone source can explicitly import Universe;
- [ ] standalone source cannot import `std`;
- [ ] bare prelude type inference works with no explicit Universe import.

### Focused gate

```bash
cargo test -p phalcom-core modules
cargo test -p phalcom-core standalone
```

### Commit

```bash
git commit -m "refactor(core): make standalone analysis consume canonical Universe"
```

---

## Task 29 — Collapse reflection to one shipped Universe identity

**Files:**

- Modify: `phalcom-core/src/primitive/reflection.rs`
- Modify: `phalcom-modules/src/package_info.rs`
- Modify: reflection cache/helpers under `phalcom-core/src/modules/`
- Modify reflection tests

### Steps

- [ ] Delete `builtin_std` package descriptor branch.

- [ ] Ensure former std modules report `ProjectIdentity::Universe`.

- [ ] Ensure reflected declaration/module paths use actual Universe module path.

- [ ] Replace any leaf-name-based builtin declaration lookup with canonical declaration metadata.

- [ ] Verify ADT reflection continues using exact canonical enum/variant identities.

### Tests

- [ ] `universe.json` reflects as Universe.
- [ ] `Int` reflects declaring module `universe.scalar.number`.
- [ ] no reflection API produces `core` or `std` builtin identity.

### Focused gate

```bash
cargo test -p phalcom-core reflection
```

### Commit

```bash
git commit -m "refactor(reflection): report one canonical Universe"
```

---

## Task 30 — Move physical shipped source into one explicit Universe tree

**Files:**

- Move: `phalcom-core/core/universe/**`
- Move: `phalcom-core/core/std/**`
- Modify: every `include_str!` in `phalcom-modules/src/builtin.rs`
- Modify: LSP/source configuration documentation
- Modify: repository scripts/docs that discover the builtin source root

### Recommended destination

```text
phalcom-core/builtins/universe/src/
```

### Steps

- [ ] Move existing Universe sources preserving logical paths.

- [ ] Move former std sources under the chosen corresponding Universe child paths.

- [ ] Delete empty historical directories:

```text
phalcom-core/core/universe
phalcom-core/core/std
phalcom-core/core
```

if nothing unrelated remains.

- [ ] Update provider `include_str!` paths.

- [ ] Search for physical-path assumptions:

```bash
rg 'core/universe|core/std|phalcom-core/core' .
```

Update production code and current documentation. Preserve historical docs only when they intentionally describe old states.

### Focused gate

```bash
cargo check --workspace
cargo test -p phalcom-modules
cargo test -p phalcom-core universe
```

### Commit

```bash
git commit -m "refactor(repo): consolidate shipped sources under Universe"
```

---

## Task 31 — Version persisted metadata/cache identities

**Files:**

- Inspect/modify: `phalcom-type-meta/src/**`
- Inspect/modify: `phalcom-core/src/modules/reflection_metadata.rs`
- Inspect/modify: typing metadata loader/validator
- Modify metadata tests

### Steps

- [ ] Find serialized stable project/module/declaration forms:

```bash
rg 'StableProject|StableModule|StableDeclaration|Builtin' \
  phalcom-type-meta phalcom-core/src/modules
```

- [ ] Change builtin stable owner representation from generic builtin name/core/std assumptions to explicit Universe.

- [ ] Bump metadata schema/version if old serialized IDs could otherwise deserialize successfully.

- [ ] Reject old persisted:

```text
universe/core/...
std/...
```

unless an explicit migration tool is intentionally provided.

- [ ] Ensure current metadata stores actual declaring Universe path.

### Tests

- [ ] new metadata round-trips canonical Universe declaration;
- [ ] legacy core/std fixture is rejected with version/identity error;
- [ ] runtime metadata lookup finds `Int`/`List` under actual module paths.

### Focused gate

```bash
cargo test -p phalcom-type-meta
cargo test -p phalcom-core typing
```

### Commit

```bash
git commit -m "fix(type-meta): persist canonical Universe declaration ownership"
```

---

## Task 32 — Strengthen end-to-end semantic/LSP acceptance coverage

**Files:**

- Expand: `phalcom-semantic/tests/semantic/integration/universe_resolution.rs`
- Expand: `phalcom-semantic/tests/semantic/integration/module_syntax_targets.rs`
- Expand: `phalcom-lsp/tests/module_navigation.rs`
- Expand: `phalcom-lsp/tests/source_text.rs`
- Add/expand: `phalcom-lsp/tests/universe_hover.rs`
- Expand import-completion tests

### Required end-to-end scenarios

- [ ] Prelude `Int`:
  - exact semantic target;
  - correct formal type;
  - exact hover;
  - exact definition.

- [ ] `List<Int>`:
  - `List` and `Int` actual declaration IDs;
  - generic type application uses those IDs.

- [ ] Local shadowing:
  - local `Int` wins.

- [ ] Explicit import:
  - same declaration identity as prelude.

- [ ] Alias:
  - local spelling but external canonical target.

- [ ] Module alias:
  - module target, hover, definition, export/member completion.

- [ ] Re-export:
  - original declaration identity retained.

- [ ] `expose`:
  - module target;
  - legal completion;
  - external path visibility behavior.

- [ ] Former std:
  - `universe.json` works;
  - `std.json` errors;
  - `json` appears under Universe completion.

- [ ] Exact module source text:
  - no aggregate core document.

- [ ] Universe declaration source method:
  - source-authored method definition/hover works even if native floor exists.

### Focused gate

```bash
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-lsp
```

### Commit

```bash
git commit -m "test(universe): cover canonical semantics and IDE behavior end to end"
```

---

## Task 33 — Add performance/incremental regression tests

**Files:**

- Modify/add semantic incremental tests
- Modify LSP perf counters/harness only if necessary
- Add a benchmark or deterministic counter-based test rather than timing-sensitive CI where possible

### Steps

- [ ] Add a semantic test:

1. construct session;
2. publish user module;
3. record Universe source/query fingerprints;
4. edit one user callable body;
5. republish;
6. assert Universe shallow products were reused.

- [ ] Add a query-path test/counter ensuring hover/definition/completion perform no provider I/O.

If provider calls are difficult to observe, inject a test provider that panics when called after snapshot publication.

- [ ] Add a completion test proving unimported Universe modules are discoverable without being runtime reachable.

- [ ] Add runtime test proving importing nothing does not initialize every Tier 2/3 Universe module merely because it exists in the catalog.

### Focused gate

```bash
cargo test -p phalcom-semantic incremental
cargo test -p phalcom-lsp perf
cargo test -p phalcom-core modules
```

### Commit

```bash
git commit -m "test(universe): lock incremental and lazy-discovery behavior"
```

---

## Task 34 — Remove all remaining legacy `core` and `std` production machinery

**Files:**

- Repository-wide production source
- Do not indiscriminately rewrite historical documents

### Steps

- [ ] Run:

```bash
rg 'ModuleId::core\(\)' \
  --glob '*.rs' \
  --glob '!docs/**' \
  --glob '!patchwork/**'
```

Expected: zero production hits.

- [ ] Run:

```bash
rg 'BuiltinPackage::Std|STD_NODES|builtin_std' \
  --glob '*.rs' \
  --glob '!docs/**' \
  --glob '!patchwork/**'
```

Expected: zero.

- [ ] Run:

```bash
rg 'CORE_MODULE_NAME|CORE_MODULE_URI|CoreSource|render_canonical_core_source|install_core|core_module\(' \
  --glob '*.rs' \
  --glob '!docs/**' \
  --glob '!patchwork/**'
```

Expected: zero architectural hits.

- [ ] Run:

```bash
rg 'phalcom://core|phalcom://std' \
  --glob '!docs/**' \
  --glob '!patchwork/**'
```

Expected: only explicit compatibility/negative-test strings if still intentionally tested.

- [ ] Audit remaining identifiers containing `core` manually. Keep legitimate meanings such as:
  - Rust crate name `phalcom-core`;
  - “core language semantics” prose if not referring to the removed module;
  - historical decision documents.

- [ ] Rename `finalize_all_core_base_names`, comments such as “kernel core classes,” and similar vocabulary when they actually mean Universe/primordial classes.

### Commit

```bash
git commit -m "refactor(universe): delete legacy core and std machinery"
```

---

## Task 35 — Update authoritative documentation and compatibility diagnostics

**Files:**

- Modify language/module specification docs under `docs/spec/`
- Modify current architecture docs under `docs/impl/`
- Modify error documentation/tests
- Do not rewrite historical documents as though old decisions never existed

### Steps

- [ ] Define Universe exactly:

> The complete toolchain-owned library environment distributed with Phalcom by default.

- [ ] Document independent properties:
  - Universe membership;
  - prelude visibility;
  - native implementation;
  - primordial status;
  - eager runtime initialization.

- [ ] Document only `universe` as builtin absolute root.

- [ ] Document `std` removal and transition diagnostic.

- [ ] Document actual module ownership for prelude declarations.

- [ ] Document virtual URI format.

- [ ] Mark older core/std architecture documents historical/superseded where appropriate rather than silently editing their historical claims.

### Commit

```bash
git commit -m "docs(universe): specify canonical shipped package model"
```

---

## Task 36 — Final verification

### Build and focused suites

- [ ] Run:

```bash
cargo fmt --all -- --check
cargo check --workspace
```

- [ ] Run module tests:

```bash
cargo test -p phalcom-modules
```

- [ ] Run semantic tests:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
```

- [ ] Run runtime/compiler tests:

```bash
cargo test -p phalcom-core
```

- [ ] Run LSP tests:

```bash
cargo test -p phalcom-lsp
```

- [ ] Run metadata/native tests:

```bash
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-surface
cargo test -p phalcom-type-meta
```

- [ ] Run full workspace:

```bash
RUST_MIN_STACK=8388608 cargo test --workspace
```

### Architectural deletion gates

- [ ] Verify zero production `ModuleId::core()`:

```bash
rg 'ModuleId::core\(\)' \
  --glob '*.rs' \
  --glob '!docs/**' \
  --glob '!patchwork/**'
```

- [ ] Verify zero production std builtin machinery:

```bash
rg 'BuiltinPackage::Std|STD_NODES|builtin_std' \
  --glob '*.rs' \
  --glob '!docs/**' \
  --glob '!patchwork/**'
```

- [ ] Verify zero aggregate core document/runtime module machinery:

```bash
rg 'CORE_MODULE_NAME|CORE_MODULE_URI|CoreSource|render_canonical_core_source|install_core|core_module\(' \
  --glob '*.rs' \
  --glob '!docs/**' \
  --glob '!patchwork/**'
```

### Identity assertions

- [ ] Run a targeted test/debug dump proving:

```text
Object -> universe.object.object
Int -> universe.scalar.number
List -> universe.collections.list
```

- [ ] Confirm no duplicate declaration ID exists for those names under another Universe path.

### IDE assertions

- [ ] Manually or integration-test:
  - hover `Int`;
  - go-to-definition `Int`;
  - `import universe.|`;
  - `from universe.collections.list import |`;
  - `export |`;
  - `expose .|`;
  - open `phalcom://universe/scalar/number`.

### Runtime assertions

- [ ] Dump module registry in test mode and confirm no `core` module.
- [ ] Confirm primordial class owner modules are canonical.
- [ ] Confirm bare prelude reads work with hidden core fallback removed.
- [ ] Confirm `None`, `Ordering`, `unsupported`, and `ellipsis` invariants.

### Final commit

```bash
git commit -m "feat(universe): complete canonical builtin integration"
```

---

## Implementation sequencing rationale

Tasks 1–7 establish identity and source/module authority before semantic/runtime consumers change. Tasks 8–17 move semantic truth to actual Universe declarations and give the editor a complete read model. Tasks 18–22 then delete LSP-specific compatibility machinery because the semantic products it needs exist. Tasks 23–28 migrate VM/compiler execution only after canonical ownership is available. Tasks 29–31 clean reflection, physical source, and persistence. Tasks 32–36 prove the migration behaviorally and delete leftovers.

Do **not** invert that sequence by deleting `ModuleId::core()` first and replacing each compile error with an ad hoc `ModuleId::universe_root()`. That would simply relocate the identity bug from a fake child module to the root package. Canonical declaration ownership must be established before downstream removal is complete.

---

## Critical review checkpoints

After Task 8:
- confirm actual source declaration ownership exists before touching LSP;
- confirm no semantic declaration must be manufactured solely from `UniverseKey::name()`.

After Task 15:
- confirm full Universe topology is queryable without entering runtime reachability.

After Task 17:
- confirm local/import shadowing over prelude.

After Task 24:
- inspect module registry and `ClassKey` owners before removing runtime fallbacks.

After Task 27:
- explicitly test `super` and default superclass semantics; this area historically depended on own-module-then-core lookup.

After Task 31:
- ensure persisted identity changes are versioned; do not rely on accidental cache invalidation.

Before Task 36 completion:
- use `superpowers:verification-before-completion`;
- do not claim success from deletion searches alone;
- require behavioral identity/navigation/inference/runtime tests.
