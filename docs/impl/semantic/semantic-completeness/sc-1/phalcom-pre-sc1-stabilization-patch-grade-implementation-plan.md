# Phalcom Pre-SC-1 Stabilization and Semantic Baseline Repair

> **For agentic workers:** execute this plan task-by-task. Use test-driven development. Do not batch unrelated tasks into one patch. Prefer one commit per task or per explicitly identified subtask.

**Repository:** `aureat/phalcom-lang`  
**Verified baseline:** `main@1c78f5d23f11865dc5e3d55e15b6f9b48a927bcc`  
**Verified:** 2026-09-01  
**Primary goal:** remove the remaining split-brain Universe/module/type authorities and correctness defects before SC-1/SC-2 proceed, while preserving already-correct canonical typing, ADT, module-linking, and runtime bootstrap work.

---

# 0. Executive implementation order

The tasks are deliberately ordered by severity first and dependency second.

| Order | Severity | Finding(s) | Task |
|---:|---|---|---|
| 1 | P0 / High | F-01 | Repair canonical Rust toolchain/CI verification |
| 2 | P0 / High | F-02 | Replace synthetic Universe-root declarations with canonical export targets |
| 3 | P0 / High | F-03 | Replace leaf-name Universe fallback with a canonical prelude map |
| 4 | P0 / High | F-04 + F-11 | Invert Universe semantic authority and factor a reusable baseline |
| 5 | P0 / High | F-05 | Remove runtime generic-arity inference by class name |
| 6 | P0 / High | F-06 | Repair unsound `Option<T>` contracts |
| 7 | P1 / Medium-High | F-07 | Enforce `expose` traversal for Universe imports |
| 8 | P1 / Medium-High | F-08 | Unify Universe dependency-path resolution with `phalcom-modules` |
| 9 | P1 / Medium | F-09 | Stop qualified type resolution from dropping path components |
| 10 | P1 / Medium | F-10 | Strengthen source/native conformance into a cross-layer identity proof |
| 11 | P2 / Medium | F-12 | Stop native source indexing from associating classes by leaf name |
| 12 | P2 / Medium-Low | F-13 | Remove stale `core`/`std` stable-metadata fixtures |
| 13 | P2 / Low | newly verified residue | Remove dead legacy-`core` semantic dependency special-casing |
| 14 | SC-1 blocker | SC1-03 | Stop ordinary open-record type tails from being erased |
| 15 | SC-1 blocker | SC1-07 | Finish type-alias publication in module interfaces/import-export |
| 16 | Certification | already-fixed SC1 findings | Lock already-completed SC-1 invariants with regression tests |

## Important re-verification correction

The earlier code review identified several SC-1 gaps. On re-reading current `main` at the same verified SHA, the following are **already implemented** in `phalcom-semantic/src/types/annotation.rs` and must **not** be reimplemented:

- explicit `TypeFormationOutcome<T>`;
- invalid kind syntax no longer becomes `KindId::TYPE`;
- missing declaration type publication no longer fabricates a nominal form;
- row-kinded generic binders use `TypeLevelBinding::RecordRow`;
- source type lambdas use scoped bound nodes;
- `TypeFormationSite` carries contextual `Self`;
- generic signature publication returns explicit formation outcomes.

Two SC-1 findings remain materially open:

1. ordinary (non-scoped) record type formation still matches `tail: _` and therefore discards an open tail;
2. semantic alias lowering exists, but `InterfaceBuilder` still does not publish `Statement::TypeAlias`, so aliases are not fully first-class module declarations for import/export/linking.

Do not overwrite the existing SC-1 implementation with older planned code.

---

# 1. Global architectural invariants

Every task in this plan must preserve these invariants.

1. **Names locate identities; identities determine semantics.** After resolution, never infer type/module meaning from a leaf string.
2. `phalcom-modules` is the authority for package/module/import/export identity.
3. `phalcom-semantic` is the authority for declaration/type/kind/ADT identity.
4. Native metadata attaches implementation/bootstrap facts to canonical semantic declarations; it does not invent ordinary language declarations.
5. Runtime reflection must map back to canonical semantic/stable identities and must never infer type parameters from a display name.
6. Universe membership, export visibility, prelude visibility, primordial status, native implementation, and eager runtime initialization are independent properties.
7. The Universe source corpus is valid Phalcom source and should be statically self-consistent.
8. `TypeId` remains store-relative; do not put process-global `TypeId`s into a shared baseline.
9. Do not introduce a second import resolver for Universe.
10. Do not force full Universe runtime reachability merely to make the Universe discoverable to semantic analysis/LSP.
11. Do not redesign ADT exact-case representation (`TypeData::ExactCase`) in this stabilization.
12. Do not redesign generic application (`TypeStore::apply_type_form`) in this stabilization.

---

# 2. Branch and baseline protocol

Before Task 1:

```bash
git switch -c semantic/pre-sc1-universe-stabilization
git rev-parse HEAD
```

Expected SHA:

```text
1c78f5d23f11865dc5e3d55e15b6f9b48a927bcc
```

If HEAD differs, inspect at minimum:

```bash
git diff 1c78f5d23f11865dc5e3d55e15b6f9b48a927bcc..HEAD -- \
  Cargo.toml rust-toolchain.toml .github/workflows/ci.yml \
  phalcom-modules/src \
  phalcom-semantic/src \
  phalcom-core/src/typing \
  phalcom-core/src/native \
  phalcom-core/core/universe/src/option \
  phalcom-core/tests/core/reflection
```

Do not blindly paste code from this plan onto a newer implementation if a symbol has materially changed.

---

# Task 1 — Repair the canonical Rust toolchain and CI verification lane

**Closes:** F-01  
**Severity:** P0 / High

## Why

The root manifest uses Cargo's unstable `codegen-backend` feature and configures Cranelift in dev/test profiles, while CI explicitly invokes stable Cargo. At the verified baseline, CI fails while parsing `Cargo.toml`, before normal workspace tests execute. SC-1 must not begin without a trustworthy regression gate.

## Architectural background

The repository already pins:

```text
nightly-2026-07-10
```

in `rust-toolchain.toml`. The simplest coherent policy is:

- repository-local/default developer commands use the pinned nightly;
- CI installs that exact pinned nightly;
- CI invokes `cargo`, not `cargo +stable`;
- rustfmt/clippy/miri use components from a compatible nightly.

Do **not** remove Cranelift merely to make CI green unless the project separately decides to abandon the profile optimization.

## Current path through the code

```text
Cargo.toml
  cargo-features = ["codegen-backend"]
  profile.dev/test.codegen-backend = "cranelift"
        ↓
.github/workflows/ci.yml
  dtolnay/rust-toolchain@stable
  cargo +stable ...
        ↓
stable Cargo parses manifest
        ↓
manifest parse error
        ↓
tests never execute
```

## Exact files

- Modify: `Cargo.toml` only if a comment is needed; no semantic profile change is required.
- Verify: `rust-toolchain.toml`
- Modify: `.github/workflows/ci.yml`

## Exact symbols / insert-replace locations

In `.github/workflows/ci.yml`, replace every Rust setup using:

```yaml
uses: dtolnay/rust-toolchain@stable
```

and every command using:

```text
cargo +stable
```

For the Miri lane, stop requesting an unconstrained latest nightly if the pinned nightly can install Miri.

## Tests to add first

This is infrastructure, so the red test is the existing CI failure.

Locally run:

```bash
cargo +stable build --workspace --all-targets
```

Expected red result before the patch: Cargo reports that `codegen-backend` requires nightly.

Then verify the intended command:

```bash
cargo build --workspace --all-targets
```

Because `rust-toolchain.toml` is present, this must select the pinned nightly.

## Paste-ready code where safe

Recommended workflow shape:

```yaml
jobs:
  test:
    name: Test (pinned nightly)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          toolchain: nightly-2026-07-10
      - uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --workspace --all-targets
      - name: Test
        run: cargo test --workspace --all-targets

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          toolchain: nightly-2026-07-10
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          toolchain: nightly-2026-07-10
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets -- -D warnings
```

Apply the same pinned toolchain to the LSP build. For Miri, first verify that `nightly-2026-07-10` has the `miri` component; if not, document one separately pinned Miri nightly rather than using floating `nightly`.

## What not to change

- Do not remove `cargo-features = ["codegen-backend"]` in this task.
- Do not change release optimization policy.
- Do not introduce multiple conflicting Rust versions across normal build/test/clippy.
- Do not make CI use “latest nightly”.
- Do not treat a green build-only lane as sufficient; tests must execute.

## Expected compiler/errors during staging

Before replacement:

```text
error: the cargo feature `codegen-backend` requires a nightly version of Cargo
```

If the pinned toolchain is not installed in CI, expect a `rustup`/toolchain-not-found error. That is an infrastructure configuration error, not a code failure.

If Miri is unavailable for the exact pin, the Miri lane may fail component installation; pin an explicitly known compatible Miri nightly and document why.

## Rust explanation

Unstable Cargo manifest features are gated by Cargo channel, not just `rustc`. Passing a stable compiler flag cannot fix a manifest parse done by stable Cargo. The command dispatcher must itself be nightly.

## Tests to add afterward

No source test is required. Add a CI smoke assertion only if the repository has a workflow-testing convention. Otherwise the workflow itself is the test.

## Verification commands

```bash
rustc --version
cargo --version
cargo build --workspace --all-targets
cargo test --workspace --all-targets
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Completion checklist

- [ ] CI installs the exact repository-pinned nightly for normal Rust lanes.
- [ ] No normal lane invokes `cargo +stable`.
- [ ] Build reaches Rust compilation.
- [ ] Test lane reaches and executes tests.
- [ ] fmt passes.
- [ ] clippy passes with `-D warnings`.
- [ ] LSP build lane passes.
- [ ] Miri uses an explicitly pinned compatible nightly.
- [ ] The SHA/toolchain used by CI is visible in logs.

---

# Task 2 — Replace synthetic Universe-root declarations with canonical export targets

**Closes:** F-02  
**Severity:** P0 / High

## Why

`BuiltinInterfaceBuilder::build_from_parsed` currently injects exported `UNIVERSE_BINDINGS` into the Universe root as *local declarations*. The linker then canonicalizes those exports as symbols owned by the root module. That creates a false identity such as:

```text
universe::Int
```

alongside the actual declaration:

```text
universe.scalar.number::Int
```

The root may expose a convenience alias, but it must never become the owner declaration.

## Architectural background

The actual root source (`phalcom-core/core/universe/src/package.ph`) owns child package imports/exports only. A convenience binding like `universe.Int` should be an alias/export whose target is the source-owned declaration. Module export identity must survive re-exporting.

Introduce a module-layer source declaration catalog. This catalog is not a semantic type table; it maps toolchain-owned Universe source identities to `phalcom_modules::DeclarationId`.

## Current path through the code

```text
Universe root source
    ↓
InterfaceBuilder::build
    ↓
BuiltinInterfaceBuilder::build_from_parsed
    ↓ synthetic declarations["Int"]
    ↓ exports["Int"] = Local("Int")
    ↓
LinkContext::resolve_export
    ↓
SymbolId { module: universe_root, name: "Int" }   ← wrong owner
```

## Exact files

- Create: `phalcom-modules/src/universe_catalog.rs`
- Modify: `phalcom-modules/src/lib.rs`
- Modify: `phalcom-modules/src/interface.rs`
- Modify: `phalcom-modules/src/builtin_interface.rs`
- Modify: `phalcom-modules/src/linker.rs`
- Modify: `phalcom-modules/tests/builtin_catalog.rs`
- Add or modify a linker integration test under `phalcom-modules/tests/` (prefer `linker.rs`/existing linker test file if present).

## Exact symbols / insert-replace locations

### `phalcom-modules/src/interface.rs`

Extend `UnlinkedExportTarget`.

Current conceptual shape:

```rust
pub enum UnlinkedExportTarget {
    Local(String),
    ReExport { path: ImportPath, remote: String },
}
```

Add a canonical external declaration target:

```rust
CanonicalDeclaration(crate::declaration::DeclarationId),
```

Insert this next to `Local`, before `ReExport`.

### `phalcom-modules/src/universe_catalog.rs`

Create `UniverseSourceDeclarationCatalog`.

The catalog should:

- use `UniverseKey::source_path()` to select the expected source module;
- load the parsed module from `UniverseSourceProvider`;
- run ordinary `InterfaceBuilder::build` on that parsed module;
- verify that the expected source declaration exists;
- return its `DeclarationId`;
- distinguish keys that intentionally have no independent top-level source declaration (`Some`, `None`) from missing declarations.

### `phalcom-modules/src/builtin_interface.rs`

Inside `BuiltinInterfaceBuilder::build_from_parsed`, replace the entire inner root branch:

```rust
if parsed.id.path.is_root() {
    for binding in UNIVERSE_BINDINGS ... {
        iface.declarations.insert(...)
        iface.exports.insert(... Local(name))
    }
}
```

with root **export alias construction only**. Do not mutate `iface.declarations` for these aliases.

### `phalcom-modules/src/linker.rs`

Inside `LinkContext::resolve_export`, extend the match on `surface.target`.

For `CanonicalDeclaration(declaration)`, emit:

```rust
LinkedExportTarget::Binding(SymbolId {
    module: declaration.module.clone(),
    name: declaration.name.clone(),
})
```

Before returning, verify the target interface/declaration exists when it is part of the interface universe. For bootstrapped Universe declarations that are deliberately outside the program's reachable interface set, use the project’s existing policy rather than inventing a fake owner.

## Tests to add first

### Red test 2.1 — root has no synthetic `Int`

In `phalcom-modules/tests/builtin_catalog.rs`, replace the semantics of `bcat_06_load_interface_root_exports_universe_bindings`.

Add:

```rust
#[test]
fn universe_root_does_not_own_native_declaration_aliases() {
    let provider = UniverseSourceProvider::new();
    let root = make_universe_id(&[]);
    let iface = provider.load_interface(&root).expect("root interface");

    assert!(!iface.declarations.contains_key("Int"));
    assert!(!iface.declarations.contains_key("List"));
    assert!(!iface.declarations.contains_key("Object"));
}
```

This must fail before the patch.

### Red test 2.2 — root export target is canonical

Assert that root `Int` export target is `CanonicalDeclaration` pointing to:

```text
ModuleId::universe(["scalar", "number"]) :: Int
```

### Red test 2.3 — linker preserves canonical owner

Construct/link a consumer selecting `Int` through Universe root and assert the resulting `LinkedReadSpec::Binding` contains the child-module `SymbolId`, not `universe_root`.

## Paste-ready code where safe

Safe target enum addition:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum UnlinkedExportTarget {
    /// A declaration or imported local name in the current module.
    Local(String),

    /// A canonical declaration owned by another module. Used for toolchain
    /// aliases such as Universe-root convenience bindings without forging a
    /// declaration in the aliasing module.
    CanonicalDeclaration(crate::declaration::DeclarationId),

    /// A name selected from another module path.
    ReExport { path: ImportPath, remote: String },
}
```

Safe linker arm:

```rust
UnlinkedExportTarget::CanonicalDeclaration(declaration) => (
    crate::interface::LinkedExportTarget::Binding(SymbolId {
        module: declaration.module.clone(),
        name: declaration.name.clone(),
    }),
    surface.range,
),
```

Do not add a runtime dependency in this arm merely because an alias exists. Runtime reachability must remain a separate decision.

## What not to change

- Do not move `Int`, `List`, `Object`, etc. source declarations into root source.
- Do not change their `DeclarationId`.
- Do not delete runtime root/prelude aliases in `vm/bootstrap.rs`.
- Do not make every Universe source module reachable/initialized merely because root has a discoverable alias.
- Do not model the alias as a second declaration.
- Do not change ordinary source `export` or `re-export` semantics beyond adding the canonical target variant.

## Expected compiler/errors during staging

After adding the enum variant, expect Rust `E0004` non-exhaustive match errors in every `UnlinkedExportTarget` match. Fix each explicitly; do not add `_ =>`.

After changing the root builder, old `bcat_06` will fail until rewritten because it expects the former synthetic model.

If `DeclarationId` is imported from the wrong module, expect `E0432`/`E0412`; use `crate::declaration::DeclarationId` or the crate re-export.

## Rust explanation

A re-export is a reference to a declaration, not declaration duplication. The enum variant makes that distinction explicit in the type system so the linker cannot accidentally reconstruct ownership from the exporting module.

## Tests to add afterward

- root `Object`, `Int`, `String`, `List`, `Map`, `Option`, `Result`, `Ordering` all resolve to expected owner modules;
- no root alias appears in `ModuleBindingLayout.local_globals`;
- a direct import from the owner module and a root convenience import yield equal `SymbolId`;
- root package child exports still behave unchanged.

## Verification commands

```bash
cargo test -p phalcom-modules --test builtin_catalog
cargo test -p phalcom-modules
cargo check -p phalcom-semantic
cargo check -p phalcom-core
```

## Completion checklist

- [ ] `universe_root` owns no synthetic `Int/List/Object/...` declarations.
- [ ] Root convenience exports target canonical child declarations.
- [ ] Linker preserves child-module `SymbolId`.
- [ ] No new eager runtime dependencies are introduced.
- [ ] Existing child-package exports still work.
- [ ] All `UnlinkedExportTarget` matches are exhaustive.

---

# Task 3 — Replace semantic leaf-name Universe fallback with a canonical prelude map

**Closes:** F-03  
**Severity:** P0 / High  
**Depends on:** Task 2's canonical Universe source declaration catalog.

## Why

`LinkedTypeResolver::resolve_type_name` currently falls back to:

```rust
UniverseKey::from_name(root)
```

and then resolves any known Universe declaration, regardless of whether the binding is actually prelude-visible. This makes non-prelude declarations such as `Behavior`, `Metaclass`, `Method`, or `Family` globally available in type position.

## Architectural background

Prelude visibility is policy attached to canonical declarations. It is not equivalent to “has a `UniverseKey`”.

Build one explicit mapping:

```text
bare prelude name -> canonical DeclarationId
```

and pass it into the resolver.

## Current path through the code

```text
annotation "Behavior"
   ↓
LinkedTypeResolver::resolve_type_name
   ↓ locals/imports/reexports miss
   ↓ prelude_module synthetic lookup misses
   ↓ UniverseKey::from_name("Behavior")
   ↓ universe_declaration(Behavior)
   ↓ known_declarations contains it
   ↓ resolves bare    ← visibility leak
```

## Exact files

- Modify: `phalcom-modules/src/universe_catalog.rs`
- Modify: `phalcom-semantic/src/resolver.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Modify/add: `phalcom-semantic/tests/semantic/integration/imported_resolution.rs`
- Prefer adding a focused file `phalcom-semantic/tests/semantic/integration/universe_resolution.rs`, and register it in `integration/mod.rs` if that test tree uses explicit modules.

## Exact symbols / insert-replace locations

### `UniverseSourceDeclarationCatalog`

Add:

```rust
pub fn prelude_declarations(&self) -> &BTreeMap<Box<str>, DeclarationId>
```

Populate only entries whose `UNIVERSE_BINDINGS` row has `prelude == true` and whose canonical source declaration is valid.

Do not include `Some`/`None` merely because runtime has special prelude handling unless the language specification explicitly says they are bare type names. They are variant surfaces, not ordinary top-level type declarations.

### `phalcom-semantic/src/resolver.rs`

Replace fields:

```rust
prelude_module: ModuleId,
```

with:

```rust
prelude: Arc<BTreeMap<Box<str>, DeclarationId>>,
```

Remove imports:

```rust
crate::core_surface::universe_declaration
phalcom_native_meta::UniverseKey
```

Replace constructor parameter `prelude_module` with the map.

In `resolve_type_name`, delete both:

```rust
let prelude_decl = DeclarationId::new(self.prelude_module.clone(), root.into());
...
if let Some(key) = UniverseKey::from_name(root) { ... }
```

and replace with one map lookup.

## Tests to add first

Create a resolver integration test using a real `SemanticWorkspaceSession`.

Assert prelude positives:

```text
Object
Int
Float
String
Bool
Option
Result
List
Map
Set
```

resolve without imports.

Assert prelude negatives:

```text
Behavior
Metaclass
Method
MethodFamily
BoundMethodFamily
Family
Nil
```

do not resolve bare.

Then explicitly import one negative declaration from its owning Universe module and assert it resolves.

## Paste-ready code where safe

Resolver field/lookup:

```rust
#[derive(Clone, Debug)]
pub struct LinkedTypeResolver {
    linked: Arc<LinkedProgram>,
    known_declarations: HashSet<DeclarationId>,
    prelude: Arc<BTreeMap<Box<str>, DeclarationId>>,
    alias_forms: RefCell<BTreeMap<DeclarationId, crate::types::id::TypeId>>,
}
```

Bare fallback:

```rust
if let Some(declaration) = self.prelude.get(root) {
    if self.known_declarations.contains(declaration) {
        return Some(declaration.clone());
    }
}

None
```

## What not to change

- Do not infer prelude membership from `UniverseKey::from_name`.
- Do not use `ModuleId::universe_root()` as a pretend owner for prelude declarations.
- Do not change value-name runtime prelude behavior in this task.
- Do not make non-prelude declarations unreachable through explicit imports.

## Expected compiler/errors during staging

Changing `LinkedTypeResolver::new` will produce `E0061` at all constructor call sites, especially `session.rs` bootstrap and workspace analysis. Update every call explicitly.

Removing `prelude_module` may produce unused import warnings promoted to errors under clippy.

## Rust explanation

Passing the map as `Arc<BTreeMap<...>>` makes the visibility policy immutable and explicit. The resolver no longer needs to reconstruct canonical identity from a string, and callers can share the same policy product.

## Tests to add afterward

- local declarations shadow prelude names;
- selective imports shadow/resolve before prelude;
- a source class named `Behavior` in the current module resolves locally even though Universe `Behavior` is non-prelude;
- diagnostics for unimported `Behavior` say unresolved rather than silently selecting Universe.

## Verification commands

```bash
cargo test -p phalcom-semantic --test semantic universe_resolution
cargo test -p phalcom-semantic --test semantic imported_resolution
cargo test -p phalcom-semantic
```

## Completion checklist

- [ ] No `UniverseKey::from_name` remains in `LinkedTypeResolver`.
- [ ] Resolver has one canonical prelude map.
- [ ] Non-prelude Universe types fail bare resolution.
- [ ] Explicit imports still work.
- [ ] Local/import precedence is unchanged.

---

# Task 4 — Make source-derived Universe declarations authoritative and factor the semantic baseline

**Closes:** F-04 and F-11  
**Severity:** P0 / High  
**Depends on:** Tasks 2–3.

## Why

`SemanticWorkspaceSession::with_workspace` currently starts from `bootstrap_universe_declarations`, which derives ordinary declaration kinds/generic arity from `UNIVERSE_BINDINGS` / `UNIVERSE_TYPE_FORMS`, then separately parses Universe source and augments enum semantics. This leaves two possible authorities.

It also embeds a long baseline-construction procedure directly inside every new workspace session.

## Architectural background

Use two layers:

```text
process-wide / source-stable:
UniverseSourceBaseline
  parsed modules
  source declaration catalog
  source generic syntax
  source hierarchy anchors
  prelude target map
  source provenance
          ↓ instantiate into a TypeStore
workspace/store-local:
UniverseSemanticBaseline
  DeclarationTypeTable
  MapTypeHierarchy
  SurfaceDispatchResolver
  CallableSignatureTable
  EnumSemanticTable/products
  associated surfaces/requirements
```

Do **not** share raw `TypeId` across independent `TypeStore`s.

Native metadata remains necessary for:

- native implementation descriptors;
- primordial runtime allocation;
- runtime-only support classes;
- explicit native contracts.

But it should be validated against source-derived ordinary declarations rather than used to silently define them.

## Current path through the code

```text
SemanticWorkspaceSession::with_workspace
  ↓ TypeStore::new
  ↓ bootstrap_universe_declarations(native metadata)
  ↓ manual Some/None support declarations
  ↓ UNIVERSE_CLASS_RELATIONS hierarchy
  ↓ register_native_surfaces
  ↓ parse Universe nodes
  ↓ source enum predeclaration/behavior
  ↓ construct base_* fields
```

## Exact files

- Create: `phalcom-semantic/src/universe_baseline.rs`
- Modify: `phalcom-semantic/src/lib.rs` or module declaration root
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/declarations.rs`
- Modify: `phalcom-semantic/src/core_surface/conformance.rs` minimally now; fully in Task 10
- Reuse: `phalcom-modules/src/universe_catalog.rs`
- Modify/add tests:
  - `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`
  - `phalcom-semantic/tests/semantic/integration/native_conformance.rs`
  - new `universe_baseline.rs` integration test if cleaner.

## Exact symbols / insert-replace locations

### New `UniverseSourceBaseline`

Place in `universe_baseline.rs`.

It must contain only stable/source identities and parsed/source products—no store-local IDs.

Suggested fields:

```rust
pub struct UniverseSourceBaseline {
    pub declarations: Arc<[UniverseSourceDeclaration]>,
    pub prelude: Arc<BTreeMap<Box<str>, DeclarationId>>,
    pub modules: Arc<[Arc<ParsedModuleUnit>]>,
}
```

`UniverseSourceDeclaration` should retain:

- `DeclarationId`;
- declaration kind (class/enum; runtime support presentation separately);
- generic parameter syntax/kinds;
- superclass syntax;
- source span/module;
- optional `UniverseKey` only as implementation correspondence.

### New `UniverseSemanticBaseline`

Store-local:

```rust
pub struct UniverseSemanticBaseline {
    pub declarations: DeclarationTypeTable,
    pub hierarchy: MapTypeHierarchy,
    pub dispatch: SurfaceDispatchResolver,
    pub callable_signatures: CallableSignatureTable,
    pub enum_semantics: EnumSemanticTable,
    // existing products...
    pub prelude: Arc<BTreeMap<Box<str>, DeclarationId>>,
}
```

Add:

```rust
pub fn instantiate(
    source: &UniverseSourceBaseline,
    store: &mut TypeStore,
) -> Result<Self, UniverseBaselineError>
```

### `SemanticWorkspaceSession::with_workspace`

Replace the large block beginning with:

```rust
let mut base_declarations =
    bootstrap_universe_declarations(...)
```

through the construction of all `base_*` values with a call to the baseline builder/instantiator.

Then assign session fields from that baseline.

### `bootstrap_universe_declarations`

After production call sites are removed:

- either delete it if only obsolete tests use it;
- or rename it to a clearly test/native helper and mark `#[cfg(test)]` / `pub(crate)` as appropriate.

It must not remain the production declaration authority.

## Tests to add first

### Red test 4.1 — source generic arity wins

For canonical `List<T>`, `Map<K,V>`, `Option<T>`, `Result<T,E>`, inspect source-derived baseline records and assert arity/kinds match source.

Before refactor, this test should demonstrate that the baseline is still derived from native metadata rather than source.

### Red test 4.2 — source-only Universe declaration works without `UniverseKey`

Use a test fixture/helper that constructs a Universe parsed module containing:

```phalcom
class SourceOnly<T> {}
```

Assert the source baseline gives it a canonical `DeclarationId` and the semantic instantiator gives it a constructor kind.

Do not add a fake `UniverseKey`.

### Red test 4.3 — native/source mismatch is rejected

Inject a test catalog row saying a one-parameter source declaration has two native parameters and assert baseline/conformance construction fails rather than choosing one silently.

## Paste-ready code where safe

Safe structural split:

```rust
#[derive(Clone, Debug)]
pub struct UniverseSemanticBaseline {
    pub declarations: DeclarationTypeTable,
    pub hierarchy: MapTypeHierarchy,
    pub dispatch: SurfaceDispatchResolver,
    pub callable_signatures: CallableSignatureTable,
    pub prelude: Arc<BTreeMap<Box<str>, DeclarationId>>,
}
```

The full field set must include the existing enum/associated products currently stored on `SemanticWorkspaceSession`; do not drop them merely to fit this abbreviated snippet.

## Source-declaration formation algorithm

Use explicit passes:

### Pass A — discover all source declarations

For every Universe parsed module:

- class -> canonical `DeclarationId(module, name)`;
- enum -> canonical `DeclarationId(module, name)`;
- type alias when SC-1 alias completion is active;
- do not derive owner from leaf name.

### Pass B — predeclare forms/kinds

For ordinary language declarations:

- lower generic parameter *kind syntax* from source;
- allocate nominal constructor form with the correct arrow kind;
- do not yet resolve `where` constraints/supertypes.

### Pass C — resolve complete generic signatures

With all declarations known:

- call existing `resolve_generic_signature`;
- publish constraints atomically;
- attach variance/source provenance.

### Pass D — hierarchy

Resolve source superclass references through canonical resolver.

For native metadata `UNIVERSE_CLASS_RELATIONS`, compare against source; do not silently overwrite a contradictory source relation.

### Pass E — native attachments

Call `register_native_surfaces` against the source-derived declaration table.

### Runtime support exception

`Some`/`None` support classes are implementation identities tied to the `Option` variants. They may remain bootstrap-only declaration/type records if required by existing sealed-inheritance/runtime code, but:

- they must not enter the ordinary source declaration catalog;
- they must not enter the type prelude map;
- their identity must be explicitly marked runtime-support;
- no source class declaration should be fabricated for them.

`Unit` remains a special proper type (`TypeData::Unit`) even though a runtime/source class presentation exists; preserve current special-type semantics unless a separate language ruling changes it.

## What not to change

- Do not make `TypeStore` global.
- Do not share `TypeId`s across stores.
- Do not remove native metadata required for VM primordial allocation.
- Do not redesign exact enum cases.
- Do not make `Some` or `None` top-level ordinary declarations.
- Do not analyze every Universe body eagerly just to build a shallow baseline.
- Do not move this source identity authority into `phalcom-core`.

## Expected compiler/errors during staging

Moving fields out of `with_workspace` will cause ownership/borrow errors (`E0382`, `E0502`) if baseline parts borrow the store while also being moved. Make the baseline own its store-relative tables.

Changing resolver construction will continue to surface `E0061` until all calls pass the canonical prelude map.

Deleting production `bootstrap_universe_declarations` calls may reveal tests importing it; update tests to construct the actual source baseline unless they are specifically low-level native-metadata unit tests.

## Rust explanation

A process-global source blueprint can safely be `Arc`/`OnceLock` because its identities are stable Rust values (`ModuleId`, `DeclarationId`, AST/source records). `TypeId`, by contrast, indexes a particular `TypeStore`, so store-local semantic instantiation is necessary unless the entire session architecture is changed to share one store.

## Tests to add afterward

- two independent workspace sessions produce semantically equivalent Universe declaration shapes while having distinct `TypeStoreId`s;
- source provenance for `Int` points to `universe.scalar.number`;
- source-only helper declarations do not require native metadata;
- native records with no source counterpart are classified explicitly as runtime support or rejected;
- baseline construction is deterministic.

## Verification commands

```bash
cargo test -p phalcom-semantic --test semantic authority_boundaries
cargo test -p phalcom-semantic --test semantic native_conformance
cargo test -p phalcom-semantic
cargo test -p phalcom-modules
```

## Completion checklist

- [ ] Production semantic bootstrap no longer creates ordinary declarations from `UNIVERSE_TYPE_FORMS`.
- [ ] Source declarations determine owner/kind/generic arity.
- [ ] Native metadata attaches to or validates canonical source declarations.
- [ ] Runtime support identities are explicit exceptions, not ordinary source declarations.
- [ ] Baseline construction is factored out of `with_workspace`.
- [ ] No shared object contains cross-store `TypeId`s.
- [ ] Prelude map comes from the same source baseline/catalog.

---

# Task 5 — Remove runtime generic constructor arity inference by class name

**Closes:** F-05  
**Severity:** P0 / High

## Why

`phalcom-core/src/typing/inspect.rs::class_constructor_arity` compares `heap.class(class).name` against native Universe type-form names. A user class named `List`, `Map`, `Option`, `Result`, etc. can therefore acquire builtin generic arity in runtime reflection.

## Architectural background

Display names are not identity. Runtime reflection needs:

```text
ClassId -> canonical runtime/semantic declaration identity -> generic signature
```

For primordial Universe classes, a `ClassId -> UniverseKey` identity registry is sufficient as a bootstrap fallback. For user classes, use loaded semantic metadata.

## Current path through the code

```text
RuntimeOverlayTypeNode::Nominal { class }
  ↓ remaining_parameter_count
  ↓ class_constructor_arity(heap, class)
  ↓ heap.class(class).name
  ↓ compare to UNIVERSE_TYPE_FORMS.owner.name()
  ↓ fake builtin arity for user shadow class
```

## Exact files

- Modify: `phalcom-core/src/typing/inspect.rs`
- Modify: runtime class/typing registry location that can map `ClassId` to stable declaration metadata. Inspect and choose the narrowest existing owner; likely:
  - `phalcom-core/src/typing/registry.rs`
  - and/or VM class registry in `phalcom-core/src/vm/*`
- Modify: bootstrap registration in `phalcom-core/src/vm/bootstrap.rs`
- Modify tests: `phalcom-core/tests/core/reflection/reflection.rs`
- Possibly add an internal unit test near `typing/inspect.rs`.

## Exact symbols / insert-replace locations

Replace:

```rust
fn class_constructor_arity(heap: &Heap, class: ClassId) -> usize
```

with an identity-aware function. Do not leave any fallback that compares a class display name to a builtin name.

Preferred shape:

```rust
fn class_constructor_arity(
    context: &TypingContextData,
    registry: &RuntimeTypingRegistry,
    heap: &Heap,
    class: ClassId,
) -> usize
```

Algorithm:

1. if runtime typing registry has a semantic declaration record bound to this `ClassId`, return that declaration's generic signature arity;
2. else if the class is a primordial Universe class and the VM has an explicit `ClassId -> UniverseKey` mapping, use `UNIVERSE_TYPE_FORMS` keyed by `UniverseKey`;
3. otherwise return `0`.

Update all `remaining_parameter_count` calls to pass the extra context/registry.

## Tests to add first

In `phalcom-core/tests/core/reflection/reflection.rs`, add end-to-end programs:

```phalcom
class List {}
class Map {}
class Option {}
class Result {}
```

Reflect each class and assert `remainingParameterCount == 0`.

Add:

```phalcom
class List<A, B, C> {}
```

and, once user declaration metadata is loaded into runtime reflection, assert count `3`.

The first four must fail before the bug fix.

## Paste-ready code where safe

Safe rule, not full implementation:

```rust
// Forbidden:
let name = heap.class(class).name.as_str();
UNIVERSE_TYPE_FORMS
    .iter()
    .find(|spec| spec.owner.name() == name)

// Required:
let key = runtime_identity.universe_key_for_class(class);
```

Do not paste a new parallel map if an existing VM class-key/metadata binding can provide the identity; reuse the narrowest existing registry.

## What not to change

- Do not rename user classes to avoid collisions.
- Do not special-case strings `"List"`, `"Map"`, etc.
- Do not make all user classes generic arity zero if semantic metadata is available.
- Do not change `base_constructor_arity`, which already uses stable declaration metadata, except to share helpers if useful.
- Do not modify type application semantics.

## Expected compiler/errors during staging

Changing `class_constructor_arity` signature will yield `E0061` at its call sites.

If you add a class-identity mapping to a registry struct, constructors/default impls may fail with `E0063` missing field. Prefer `Default`-initialized map or update every explicit initializer.

## Rust explanation

`ClassId` is a handle identity. A `String` is presentation data. Mapping from handle to stable declaration is an identity-preserving operation; mapping from display name back to semantic type is not.

## Tests to add afterward

- user `List<T,U,V>` reports 3;
- canonical Universe `List<T>` still reports 1;
- canonical `Map<K,V>` reports 2;
- unrelated class named `Some` does not inherit Option support-class generic semantics;
- a metaclass/class-object reflection path preserves the same declaration arity.

## Verification commands

```bash
cargo test -p phalcom-core --test core reflection
cargo test -p phalcom-core --test core object_model
cargo test -p phalcom-core --test core
```

## Completion checklist

- [ ] `class_constructor_arity` contains no name comparison.
- [ ] Primordial arity uses identity.
- [ ] User generic arity uses semantic metadata where available.
- [ ] Shadow-name regression tests pass.
- [ ] Existing Universe reflection tests pass.

---

# Task 6 — Repair unsound `Option<T>` generic contracts

**Closes:** F-06  
**Severity:** P0 / High

## Why

Current canonical source contains:

```phalcom
unwrapOr<U>(_ default: U) -> U {
  some: |v| v
  none: || default
}
```

The `Some` branch returns `T`, so `U` is not a sound result without `T <: U`.

`okOr<E>(_ err)` also omits the parameter annotation that ties `err` to `E`.

## Architectural background

Universe source must be typed as ordinary authoritative source. Do not encode an unsound library API merely because current inference cannot yet reject it.

For the current feature set, use same-type fallback semantics.

## Current path through the code

```text
Option<T>.unwrapOr<U>
  Some branch => T
  None branch => U
  declared result => U
  no T <: U proof
```

## Exact files

- Modify: `phalcom-core/core/universe/src/option/option.ph`
- Add/modify Option tests under `phalcom-core/tests/core/` (use existing Option module if present).
- Add semantic Universe-source contract test in `phalcom-semantic/tests/semantic/integration/native_conformance.rs` or a dedicated Universe source test.

## Exact symbols / insert-replace locations

In `option.ph`, replace exactly:

```phalcom
unwrapOr<U>(_ default: U) -> U
```

with:

```phalcom
unwrapOr(_ default: T) -> T
```

Replace:

```phalcom
okOr<E>(_ err) -> Result<T, E>
```

with:

```phalcom
okOr<E>(_ err: E) -> Result<T, E>
```

Update documentation text for `unwrapOr`: remove `@typeparam U`.

## Tests to add first

Add a semantic test proving heterogeneous fallback cannot masquerade as the fallback type.

Conceptual source:

```phalcom
const x: Option<Int> = Option::Some(1)
const y: String = x.unwrapOr("none")
```

Expected after the patch: type mismatch at the call/assignment boundary (depending on current SC-2 status). If generic call inference cannot yet fully exercise this, at minimum assert the published method signature is `(T) -> T`.

Add runtime tests:

- `Some(1).unwrapOr(2) == 1`;
- `None<Int>.unwrapOr(2) == 2` using the language's current way of specializing/typing None.

## Paste-ready code where safe

```phalcom
unwrapOr(_ default: T) -> T {
  match(
    some: |v| v,
    none: || default
  )
}

okOr<E>(_ err: E) -> Result<T, E> {
  match(
    some: |v| Result::Ok(v),
    none: || Result::Error(err)
  )
}
```

## What not to change

- Do not add a `where T <: U` design in this stabilization unless that source constraint is already fully enforced by SC-1/SC-2.
- Do not weaken the return type to `Dynamic`.
- Do not change `map`/`flatMap` APIs as part of this patch.

## Expected compiler/errors during staging

If source-native surface metadata still declares the old signature, native conformance tests should fail. Update the corresponding native surface declaration only if one exists for these methods; source-only methods should require no native change.

If existing tests assume heterogeneous defaults, they will fail and should be updated to the sound API.

## Rust explanation

No Rust implementation change is required for the Phalcom-level type rule, but the compiler's semantic metadata tests should treat the `.ph` signature as the authority.

## Tests to add afterward

- signature reflection reports `unwrapOr(_ default: T) -> T`;
- `okOr` exposes `E` on its argument;
- Option source corpus passes semantic checking.

## Verification commands

```bash
cargo test -p phalcom-core --test core option
cargo test -p phalcom-semantic --test semantic native_conformance
cargo test -p phalcom-core --test core
```

## Completion checklist

- [ ] `unwrapOr` is sound for every `T`.
- [ ] `okOr` parameter is annotated `E`.
- [ ] docs match signature.
- [ ] reflection/native conformance agrees.
- [ ] runtime behavior unchanged.

---

# Task 7 — Enforce package `expose` traversal for Universe imports

**Closes:** F-07  
**Severity:** P1 / Medium-High

## Why

`ModuleResolver::resolve_import_with_trace` returns directly from the `ImportRootTarget::Universe` branch after checking provider node existence. It therefore bypasses the package exposure walk used for ordinary external projects.

## Architectural background

Universe has a special source provider, not special package visibility semantics.

Factor exposure traversal over a package-surface loader so both resolved projects and Universe use the same algorithm.

## Current path through the code

```text
normal external import
  → validate_external_path_with_trace
  → parent package exposed_children checks

Universe import
  → provider.kind(path)
  → provider.source_id
  → return
  → no expose check
```

## Exact files

- Modify: `phalcom-modules/src/resolver.rs`
- Add/modify tests under `phalcom-modules/tests/`
- Prefer a new `universe_exposure.rs` if not already present.

## Exact symbols / insert-replace locations

Inside `ModuleResolver::resolve_import_with_trace`, in:

```rust
ImportRootTarget::Universe => { ... }
```

insert exposure validation before accepting the target.

Factor a helper near `validate_external_path_with_trace`:

```rust
fn validate_path_exposure(
    path: &ModulePath,
    mut load_surface: impl FnMut(&ModulePath) -> Result<PackagePathSurface, ModuleResolutionError>,
    package_interfaces: &mut BTreeSet<ModuleId>,
) -> Result<(), ModuleResolutionError>
```

Then:

- resolved projects call it with `load_package_surface`;
- Universe calls it with a Universe package surface loader.

## Tests to add first

Create a unit-level exposure-walk fixture that has:

```text
root exposes public
root does not expose hidden
```

but both nodes exist.

Assert hidden is rejected.

Then run an integration test for a currently exposed real path, e.g.:

```text
universe.reflection.selector
```

and verify success.

## Paste-ready code where safe

The algorithm must check **the parent package before stepping into each component**:

```rust
let mut current = ModulePath::root();
for component in path.components() {
    let surface = load_surface(&current)?;
    if !surface.exposed_children.contains(component) {
        return Err(ModuleResolutionError::ModulePathNotExposed { ... });
    }
    current = current.join(component.clone());
}
```

Do not check exposure on the child after entering it; that answers the wrong question.

## What not to change

- Do not use `UNIVERSE_NODES.children` as the visibility authority when authored `package.ph` `expose` syntax exists.
- Do not add hidden nodes to package source just to satisfy a test.
- Do not make Universe imports resolved-project identities.

## Expected compiler/errors during staging

Factoring closures that borrow `self` mutably can trigger `E0500`/`E0501` due to nested mutable borrows. If so, use separate helper methods for resolved and Universe surface loading around a pure exposure walker, rather than a closure that captures `&mut self`.

## Rust explanation

Separating pure traversal from provider-specific I/O avoids borrow complexity and prevents semantic duplication.

## Tests to add afterward

- nested three-level exposure traversal;
- failure reports the first hidden boundary;
- root package remains addressable;
- completion/query code still discovers all explicitly exposed Universe children.

## Verification commands

```bash
cargo test -p phalcom-modules universe_exposure
cargo test -p phalcom-modules resolver
cargo test -p phalcom-modules
```

## Completion checklist

- [ ] Universe imports obey authored `expose`.
- [ ] Resolved-project behavior unchanged.
- [ ] One traversal algorithm defines visibility.
- [ ] Hidden existing Universe nodes are not importable.
- [ ] No eager runtime reachability change.

---

# Task 8 — Unify Universe dependency-path resolution with `phalcom-modules`

**Closes:** F-08  
**Severity:** P1 / Medium-High

## Why

`phalcom-core/src/native/source.rs` contains a private `universe_dependency_target` that manually interprets import roots and relative dots. This duplicates module semantics and even clamps excessive parent traversal to root rather than reporting the resolver's `RelativeImportBeyondRoot` error.

## Architectural background

Import syntax-to-`ModuleId` resolution belongs in `phalcom-modules`. Runtime/native source indexing may consume the result but must not reimplement it.

## Current path through the code

```text
NativeSourceIndex::initialization_order
  ↓ each preamble import
  ↓ universe_dependency_target
  ↓ manual relative/absolute path calculation
  ↓ custom graph
```

## Exact files

- Modify: `phalcom-modules/src/resolver.rs`
- Modify: `phalcom-modules/src/lib.rs` if a public helper is exported
- Modify: `phalcom-core/src/native/source.rs`
- Add tests:
  - `phalcom-modules/tests/` for pure path semantics;
  - internal tests in `phalcom-core/src/native/source.rs` for initialization order.

## Exact symbols / insert-replace locations

In `phalcom-modules/src/resolver.rs`, extract a pure helper from the relative branch:

```rust
pub fn resolve_logical_import_target(
    importer: &ModuleId,
    importer_kind: ModuleKind,
    syntax: &ImportPath,
) -> Result<ModuleId, ModuleResolutionError>
```

For an absolute root:

- accept `universe` only when constructing a Universe target in this pure helper;
- ordinary project roots remain resolved by `ProjectUniverse` in `ModuleResolver`.

For relative:

- derive importer package path exactly as current resolver does;
- reject `dots == 0`;
- reject traversal beyond root;
- append validated segments;
- preserve importer project identity.

Use this helper from `ModuleResolver` where applicable.

In `phalcom-core/src/native/source.rs`, delete:

```rust
fn universe_dependency_target(...)
```

and call the module-layer helper.

## Tests to add first

- module at `universe.a.b.file` with one dot resolves sibling inside `a.b`;
- two dots ascends exactly one package;
- too many dots returns `RelativeImportBeyondRoot`;
- absolute `universe.x.y` resolves correctly;
- absolute unrelated root is rejected by the Universe-only use.

## Paste-ready code where safe

The root-bound check must follow current resolver semantics:

```rust
let ascend_count = dots - 1;
if ascend_count > package_components.len() {
    return Err(ModuleResolutionError::RelativeImportBeyondRoot {
        dots,
        depth: package_components.len(),
    });
}
```

Do not use:

```rust
parent().unwrap_or_else(ModulePath::root)
```

in a loop; that silently changes invalid source into a different valid path.

## What not to change

- Do not move runtime initialization ordering into `phalcom-modules` unless the existing graph abstraction is directly reusable.
- Do not let `phalcom-core` parse import path semantics.
- Do not alter valid relative import meaning.

## Expected compiler/errors during staging

Deleting the private helper produces `E0425` until all call sites are switched.

If the extracted helper is public but its argument types are not re-exported, expect import-path visibility errors; export only the narrow helper and existing public types.

## Rust explanation

A pure path resolver is deterministic and easily unit tested. Source providers can then perform existence/exposure checks separately without duplicating syntax semantics.

## Tests to add afterward

- `NativeSourceIndex::initialization_order` remains deterministic;
- the same Universe import yields equal `ModuleId` through compiler resolver and native source index;
- invalid relative imports fail identically.

## Verification commands

```bash
cargo test -p phalcom-modules
cargo test -p phalcom-core --lib native::source
cargo test -p phalcom-core --test core
```

## Completion checklist

- [ ] `universe_dependency_target` is deleted.
- [ ] One module-layer helper defines relative path semantics.
- [ ] Beyond-root traversal errors rather than clamps.
- [ ] Universe initialization order tests pass.

---

# Task 9 — Stop qualified semantic type resolution from dropping path components

**Closes:** F-09  
**Severity:** P1 / Medium

## Why

The qualified branch of `LinkedTypeResolver::resolve_type_name` uses `members.last()` and constructs the declaration directly in the imported module. Intermediate components are silently ignored.

The module linker already rejects static symbol references with `members.len() != 1`. The semantic type resolver should obey the same current language contract until true nested namespace traversal is designed.

## Architectural background

Correct rejection is better than assigning unintended meaning.

## Current path through the code

```text
alias.A.B
  ↓ target module for alias
  ↓ leaf = "B"
  ↓ DeclarationId(target module, "B")
  ↓ "A" discarded
```

## Exact files

- Modify: `phalcom-semantic/src/resolver.rs`
- Modify/add: `phalcom-semantic/tests/semantic/integration/imported_resolution.rs`

## Exact symbols / insert-replace locations

Inside the `else` qualified branch of:

```rust
impl TypeResolver for LinkedTypeResolver {
    fn resolve_type_name(...)
```

insert before module lookup:

```rust
if members.len() != 1 {
    return None;
}
```

Then replace `members.last().unwrap()` with `&members[0]`.

## Tests to add first

Add a focused resolver test:

- module alias `S` targeting module with declaration `Point`;
- `S.Point` resolves;
- `S.Nested.Point` returns `None` even if `Point` exists directly in `S`'s target module.

This must fail before the patch because the latter currently resolves to `Point`.

## Paste-ready code where safe

```rust
if members.len() != 1 {
    return None;
}

let leaf_name = &members[0];
```

## What not to change

- Do not invent nested module/type-member traversal.
- Do not reinterpret `A.B.C` as `A.C`.
- Do not change `SimpleTypeResolver` test behavior unless its explicit full-string lookup is intentionally different.

## Expected compiler/errors during staging

None expected beyond a possible clippy complaint if an old `unwrap()` becomes unused.

## Rust explanation

The explicit length guard encodes the current resolver contract and removes a silent semantic lossy transformation.

## Tests to add afterward

- diagnostics for unsupported deeper qualification are unresolved rather than incorrectly resolved;
- normal `moduleAlias.Type` remains valid;
- imported selective types are unaffected.

## Verification commands

```bash
cargo test -p phalcom-semantic --test semantic imported_resolution
cargo test -p phalcom-semantic
```

## Completion checklist

- [ ] Intermediate components are never ignored.
- [ ] One-member qualification still works.
- [ ] Deeper qualification fails safely.

---

# Task 10 — Turn native/source conformance into a cross-layer identity proof

**Closes:** F-10  
**Severity:** P1 / Medium  
**Depends on:** Tasks 2–4.

## Why

`validate_native_surface_conformance` currently ignores its resolver/current-module arguments and checks native type expressions largely against native-derived declaration metadata. It can prove internal consistency without proving agreement with source ownership.

## Architectural background

Conformance should verify a commuting diagram:

```text
UniverseKey
   ↓ expected source path
source declaration
   ↓
DeclarationId
   ↓
source kind/generic signature/hierarchy
   ↔ native metadata contract
   ↓
native surface owner
```

Any disagreement is a bootstrap error.

## Current path through the code

```text
validate_native_surface_conformance
  ↓ universe_declaration(key)
  ↓ resolve native type specs
  ↓ success even if source/module identity is stale
```

## Exact files

- Modify: `phalcom-semantic/src/core_surface/conformance.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/native_conformance.rs`
- Reuse: `phalcom-modules/src/universe_catalog.rs`
- Reuse: `phalcom-semantic/src/universe_baseline.rs`
- Possibly modify `phalcom-semantic/src/core_surface/mod.rs` exports.

## Exact symbols / insert-replace locations

Replace `ConformanceReport.failures: Vec<String>` with a structured enum if practical:

```rust
pub enum ConformanceFailure {
    MissingSourceDeclaration { key: UniverseKey },
    SourceOwnerMismatch { key: UniverseKey, expected: ModuleId, actual: ModuleId },
    GenericArityMismatch { ... },
    GenericKindMismatch { ... },
    HierarchyMismatch { ... },
    NativeSurfaceTypeFailure { ... },
}
```

Change:

```rust
validate_native_surface_conformance(
    store,
    declarations,
    _resolver,
    _current_module,
)
```

to receive either:

```rust
catalog: &UniverseSourceDeclarationCatalog,
baseline: &UniverseSemanticBaseline,
```

or a narrower conformance view.

Delete unused parameters rather than retaining underscores.

## Tests to add first

Replace current test setup that uses:

```rust
bootstrap_universe_declarations(...)
core_mod = ModuleId::universe_root()
```

with the actual source baseline.

Add injected mismatch tests:

1. expected source module differs;
2. native generic arity differs;
3. source declaration missing;
4. source superclass differs from native relation.

Each must return a typed failure.

## Paste-ready code where safe

Safe assertion style:

```rust
let source_decl = catalog
    .declaration_for_key(record.owner())
    .ok_or(ConformanceFailure::MissingSourceDeclaration {
        key: record.owner(),
    })?;
```

Then compare `source_decl` to the declaration used by the semantic baseline. Do not call `universe_declaration(key)` and assume that proves source existence.

## What not to change

- Do not make conformance mutate semantic state.
- Do not “repair” mismatches by choosing native or source data.
- Do not downgrade identity mismatches to warnings.
- Do not compare only display names.

## Expected compiler/errors during staging

Changing `ConformanceReport.failures` type will break tests expecting `Vec<String>`. Update assertions to pattern-match failures.

Changing function signature yields `E0061` at call sites.

## Rust explanation

A typed failure enum makes invariants executable and prevents tests from depending on fragile error strings. It also lets CI distinguish path identity failure from type-expression failure.

## Tests to add afterward

- every native ordinary declaration maps to exactly one source declaration;
- support-class exceptions are explicitly classified;
- all native surfaces resolve parameter/return type specs against the source-derived baseline;
- report count is deterministic.

## Verification commands

```bash
cargo test -p phalcom-semantic --test semantic native_conformance -- --nocapture
cargo test -p phalcom-semantic
cargo test -p phalcom-core --test core reflection
```

## Completion checklist

- [ ] Conformance consumes source-derived identity.
- [ ] No ignored `_resolver`/`_current_module` parameters.
- [ ] Source path, arity, kinds, hierarchy are verified.
- [ ] Mismatch is fatal/structured.
- [ ] Native type expression checks remain.

---

# Task 11 — Stop native source indexing from associating classes by leaf name

**Closes:** F-12  
**Severity:** P2 / Medium

## Why

`NativeSourceIndex::index_class` and `index_enum` call `UniverseKey::from_name(&name)` without first proving the current module is the key's canonical `source_path`. A source-only helper class with a colliding leaf name can be mistaken for a native presentation.

## Architectural background

The source index already knows `module: &ModuleId`. Use `(module, name)` identity.

## Current path through the code

```text
parse class List in any Universe module
  ↓ UniverseKey::from_name("List")
  ↓ Some(UniverseKey::List)
  ↓ row marked as native catalog presentation
```

## Exact files

- Modify: `phalcom-core/src/native/source.rs`
- Modify internal tests in the same file.

## Exact symbols / insert-replace locations

Add a helper near `universe_dependency_target`'s former location or near indexing helpers:

```rust
fn universe_key_for_source_declaration(
    module: &ModuleId,
    name: &str,
) -> Option<UniverseKey>
```

Algorithm:

1. `let key = UniverseKey::from_name(name)?;`
2. construct canonical module from `key.source_path()`;
3. return `Some(key)` only if module equals canonical owner.

Use it in both:

- `NativeSourceIndex::index_enum`
- `NativeSourceIndex::index_class`

For `@native` source declarations whose name corresponds to a key but module is wrong, prefer a hard source-contract error rather than silently treating them source-only.

## Tests to add first

Inside the existing `#[cfg(test)] mod tests`:

- index a non-native `class List {}` in a fake Universe module other than `collections.list`; assert `universe_key == None`;
- index `@native class List {}` in the wrong module; assert an error mentioning owner/path mismatch;
- index canonical List path; assert key is `UniverseKey::List`.

## Paste-ready code where safe

```rust
fn universe_key_for_source_declaration(module: &ModuleId, name: &str) -> Option<UniverseKey> {
    let key = UniverseKey::from_name(name)?;
    let expected = ModuleId::universe(ModulePath::from_components(
        key.source_path()
            .iter()
            .map(|component| {
                phalcom_modules::ModuleComponent::from_identifier(component)
                    .expect("canonical Universe component")
            })
            .collect::<Vec<_>>(),
    ));
    (module == &expected).then_some(key)
}
```

## What not to change

- Do not remove `UniverseKey` from native implementation metadata.
- Do not make source-only helpers illegal merely because their leaf name collides, unless they claim `@native`.
- Do not use display path strings when `ModuleId` is available.

## Expected compiler/errors during staging

None expected except unused imports if `UniverseKey::from_name` moves exclusively into the helper.

## Rust explanation

The helper converts a potentially ambiguous human-readable name into an identity only after checking the owning module. This is the same “resolve once, then use identity” rule applied to native source indexing.

## Tests to add afterward

- all bundled native source presentations still index successfully;
- source-only collisions remain source-only;
- Option variant support handling remains unchanged.

## Verification commands

```bash
cargo test -p phalcom-core --lib native::source
cargo test -p phalcom-core --test core
```

## Completion checklist

- [ ] `index_class` does not directly call name-only association.
- [ ] `index_enum` does not directly call name-only association.
- [ ] Wrong-path `@native` declarations fail.
- [ ] Source-only collisions are safe.

---

# Task 12 — Remove stale `core` / `std` stable-metadata test fixtures

**Closes:** F-13  
**Severity:** P2 / Medium-Low

## Why

Production stable identity currently represents Universe as:

```rust
StableProjectRef::Builtin {
    namespace: "universe",
    version: "0.1.0",
}
```

but reflection tests still fabricate `"core"` and `"std"` builtin project identities. These tests no longer certify the actual production identity model.

## Architectural background

`StableProjectRef::Builtin` itself can remain generic for schema compatibility. The cleanup is about canonical Phalcom fixture data, not removing the enum variant.

## Current path through the code

Stale fixtures:

- `phalcom-core/tests/core/reflection/reflection.rs::test_module` → `"core"`
- `phalcom-core/tests/core/reflection/type_metadata.rs` → `"std"`

Production conversion:

- `phalcom-semantic/src/metadata/stable_identity.rs::to_stable_project` → `"universe"`
- `phalcom-core/src/modules/materialize.rs` uses `"universe"`.

## Exact files

- Modify: `phalcom-core/tests/core/reflection/reflection.rs`
- Modify: `phalcom-core/tests/core/reflection/type_metadata.rs`
- Verify only: `phalcom-type-meta/tests/schema_compat.rs`
- Verify only: `phalcom-semantic/src/metadata/stable_identity.rs`

## Exact symbols / insert-replace locations

In `reflection.rs`, replace `test_module()` fixture's `"core"` namespace with `"universe"` and use a plausible canonical path for the synthetic test declaration.

In `type_metadata.rs`, replace Phalcom-specific `"std"` fixtures with `"universe"`.

Do not alter schema-compat tests that intentionally use arbitrary `"test"` builtin namespaces.

## Tests to add first

A grep-based review is sufficient before patch:

```bash
rg 'namespace:\s*"core"|namespace:\s*"std"' \
  phalcom-core/tests/core/reflection \
  phalcom-semantic \
  phalcom-core/src
```

The command should find the stale tests before patch.

## Paste-ready code where safe

```rust
project: StableProjectRef::Builtin {
    namespace: "universe".into(),
    version: "0.1.0".into(),
},
```

## What not to change

- Do not remove `StableProjectRef::Builtin`.
- Do not alter arbitrary schema test namespaces used to test serialization generically.
- Do not add backwards compatibility for `core`/`std` unless explicitly required.

## Expected compiler/errors during staging

None expected. Snapshot/serialized fixture assertions may change if tests compare exact JSON; update expected canonical values.

## Rust explanation

This is fixture correctness, not runtime semantics. Stable identity tests should encode the production namespace they claim to model.

## Tests to add afterward

If there is a stable-identity round-trip test, assert:

```text
ProjectIdentity::Universe -> Builtin("universe", ...)
```

and never `"core"`/`"std"`.

## Verification commands

```bash
cargo test -p phalcom-core --test core reflection
cargo test -p phalcom-type-meta
rg 'namespace:\s*"core"|namespace:\s*"std"' \
  phalcom-core/tests/core/reflection \
  phalcom-semantic/src \
  phalcom-core/src
```

## Completion checklist

- [ ] Phalcom reflection fixtures use `universe`.
- [ ] Generic schema fixtures remain generic.
- [ ] No production path emits `core`/`std` stable project identity.

---

# Task 13 — Remove the dead legacy-`core` semantic dependency special case

**Closes:** newly verified migration residue  
**Severity:** P2 / Low

## Why

Current `phalcom-semantic/src/checker/context.rs` still contains:

```rust
fn is_query_owned_module(module: &ModuleId) -> bool {
    let components = module.path.components();
    !(module.project == Universe
      && components.len() == 1
      && components[0] == "core")
}
```

There is no canonical `universe.core` module in the new identity model. The comment describes a compatibility core surface that the migration has retired.

## Architectural background

Real Universe modules are now ordinary canonical module identities for semantic dependency purposes. Dead exceptions are dangerous because they can become accidentally reachable through a synthetic test/module and suppress invalidation tracking.

## Current path through the code

`TrackingTypeResolver` and hierarchy dependency recorders call `is_query_owned_module` before recording semantic dependencies.

## Exact files

- Modify: `phalcom-semantic/src/checker/context.rs`
- Add/modify: incremental dependency tests under `phalcom-semantic/tests/semantic/incremental/` or `foundations/authority_boundaries.rs`.

## Exact symbols / insert-replace locations

Delete the old compatibility comment and special-case implementation.

If every current module is query-owned, replace the helper with direct recording and remove it.

If immutable bootstrap products are intentionally excluded, encode that by an explicit semantic product category—not a magic module path string.

## Tests to add first

Construct a synthetic `ModuleId::universe(["core"])` only at the unit level and show the old helper suppresses dependency tracking. Then delete the test or rewrite it to assert no path-based suppression remains.

More importantly, add a real Universe declaration dependency test and assert tracking records it.

## Paste-ready code where safe

If no exclusion remains:

```rust
fn is_query_owned_module(_module: &ModuleId) -> bool {
    true
}
```

Use this only as an intermediate compile step; final code should preferably delete the helper and simplify callers.

## What not to change

- Do not change `ModuleId` constructors to support legacy core.
- Do not add a compatibility module.
- Do not suppress Universe semantic dependencies globally.

## Expected compiler/errors during staging

Deleting the helper yields `E0425` until callers are simplified. This is expected.

## Rust explanation

Path-string sentinel logic is not type-safe authority. If a product category is immutable, encode that in the product/query layer.

## Tests to add afterward

- incremental edits to a real Universe semantic dependency invalidate/recompute the expected product when supported by the test harness;
- no `"core"` module sentinel remains in `checker/context.rs`.

## Verification commands

```bash
cargo test -p phalcom-semantic --test semantic incremental
cargo test -p phalcom-semantic
rg 'universe.*core|components\[0\].*core' phalcom-semantic/src
```

## Completion checklist

- [ ] Dead path sentinel removed.
- [ ] Dependency tracking uses real product ownership.
- [ ] No new legacy core compatibility introduced.

---

# Task 14 — Reject ordinary open-record tails instead of erasing them

**Closes:** SC1-03 remaining current defect  
**Severity:** SC-1 blocker

## Why

The scoped/type-lambda path already rejects open record tails, but ordinary `resolve_type_form` still matches:

```rust
TypeAnnotationExpr::Record { fields, tail: _, ... }
```

and constructs a closed record. This silently changes the user's type.

## Architectural background

SC-3 owns open-row solving. Before SC-3, the only sound SC-1 behavior is explicit unsupported/invalid formation—not tail erasure.

## Current path through the code

```text
#{ name: String, | R }
  ↓ resolve_type_form ordinary path
  ↓ tail ignored
  ↓ store.record(fields)
  ↓ published as closed #{ name: String }   ← unsound
```

## Exact files

- Modify: `phalcom-semantic/src/types/annotation.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`

## Exact symbols / insert-replace locations

Inside `resolve_type_form`, replace:

```rust
TypeAnnotationExpr::Record { fields, tail: _, range: _ } => {
```

with:

```rust
TypeAnnotationExpr::Record { fields, tail, range: _ } => {
    if tail.is_some() {
        diagnostics.push(...);
        return TypeFormResolution::Invalid(
            TypeFormationInvalid::UnsupportedOpenRecordTail
        );
    }
    ...
}
```

Mirror the diagnostic/category already used by `lower_scoped_type_form`; do not invent a second error classification.

## Tests to add first

In `type_annotations.rs`, parse or construct:

```phalcom
#{ name: String, | R }
```

with `R` bound as `RecordRow`.

Assert:

- result is `TypeFormationOutcome::Invalid(UnsupportedOpenRecordTail)`;
- diagnostic code is `AnnotationUnsupported`;
- no closed `TypeData::Record` is published.

## Paste-ready code where safe

```rust
TypeAnnotationExpr::Record { fields, tail, range: _ } => {
    if tail.is_some() {
        diagnostics.push(SemanticDiagnostic::error_in(
            current_module.clone(),
            DiagnosticCode::AnnotationUnsupported,
            "open record type tails are not available before SC-3 row solving",
            annotation.range,
        ));
        return TypeFormResolution::Invalid(
            TypeFormationInvalid::UnsupportedOpenRecordTail,
        );
    }

    // existing closed-record lowering continues here
```

## What not to change

- Do not implement row solving here.
- Do not convert the tail to `Dynamic`.
- Do not drop the tail.
- Do not alter closed record canonicalization.

## Expected compiler/errors during staging

No compiler errors expected. Existing tests that incorrectly expected tail erasure should fail semantically and must be corrected.

## Rust explanation

The AST carries information the semantic layer cannot yet represent in this path. Returning an explicit non-success preserves soundness and allows SC-3 to later replace the outcome without changing source meaning.

## Tests to add afterward

- closed records still lower normally;
- duplicate field diagnostics remain unchanged;
- scoped and ordinary paths return the same unsupported category for open tails.

## Verification commands

```bash
cargo test -p phalcom-semantic --test semantic type_annotations
cargo test -p phalcom-semantic
```

## Completion checklist

- [ ] No `tail: _` remains in ordinary record type lowering.
- [ ] Open tails fail explicitly.
- [ ] Closed records unchanged.
- [ ] Scoped/ordinary error categories agree.

---

# Task 15 — Finish type-alias publication in module interfaces and import/export linking

**Closes:** SC1-07 remaining integration defect  
**Severity:** SC-1 blocker

## Why

Current semantic session code already:

- predeclares `Statement::TypeAlias`;
- creates `DeclarationKind::Alias` shells;
- lowers generic aliases through scoped type lambdas;
- stores alias forms;
- publishes alias declaration shells.

However, `phalcom-modules/src/interface.rs::InterfaceBuilder::build` Pass 1 still handles `Class`, `Enum`, and `Let`, not `Statement::TypeAlias`.

Therefore an alias can exist semantically inside a module but fail to participate correctly in:

- module interface declarations;
- `export Alias`;
- selective imports;
- cross-module resolution;
- canonical linked identity.

## Architectural background

Transparent alias identity and alias denotation are separate:

```text
DeclarationId(module, Alias)   // navigation/import identity
        ↓
transparent semantic form      // underlying type/type constructor
```

Do not give aliases nominal runtime class identity.

## Current path through the code

```text
parser -> Statement::TypeAlias
   ↓
semantic session sees alias and lowers it
   BUT
InterfaceBuilder Pass 1 ignores alias
   ↓
export/import linker cannot see declaration
   ↓
cross-module alias is incomplete
```

## Exact files

- Modify: `phalcom-modules/src/interface.rs`
- Modify: `phalcom-modules/tests/` interface tests
- Verify/modify: `phalcom-semantic/src/session.rs`
- Verify/modify: `phalcom-semantic/src/resolver.rs`
- Modify/add:
  - `phalcom-semantic/tests/semantic/integration/imported_resolution.rs`
  - `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
- Verify DB products already used by `TypeDeclarationShell::Alias`; only add missing query plumbing if a test exposes it.

## Exact symbols / insert-replace locations

### `InterfaceBuilder::build`, Pass 1

Immediately after `Statement::Enum(enum_def)` handling, add `Statement::TypeAlias(alias)`.

Use the same declaration namespace collision path as classes/enums.

### `DeclarationSurface`

If no declaration-kind field is needed by linker, do not expand it just for aliases. `is_const: true` is acceptable as a value-namespace immutability fact if that field is the existing interface abstraction.

The declaration's semantic kind (`Alias`) already belongs in `DeclarationBlueprint`/semantic shell, not necessarily the module linker.

### Semantic session

The verified current session already has alias lowering. Do not duplicate it. Only adjust if the linked interface prerequisite now allows the existing code to work cross-module.

## Tests to add first

### Red test 15.1 — module interface contains alias

Parse:

```phalcom
type UserId = Int
export UserId
```

Assert `InterfaceBuilder::build(...).declarations.contains_key("UserId")` and export validation succeeds.

### Red test 15.2 — alias collides with import/declaration

Test:

```phalcom
class UserId {}
type UserId = Int
```

and import/alias collisions. They must use the existing unified namespace duplicate error.

### Red test 15.3 — cross-module transparent alias

Module A:

```phalcom
type UserId = Int
export UserId
```

Module B:

```phalcom
from .a import UserId
const x: UserId = 1
```

Assert:

- import target is alias `DeclarationId` in module A;
- type form resolves transparently to `Int`;
- go-to-definition/navigation target remains the alias declaration, not the `Int` source declaration.

### Red test 15.4 — generic alias

```phalcom
type Boxed<T> = List<T>
export Boxed
```

Consumer uses `Boxed<Int>`; canonical form should beta-reduce/expand to `List<Int>` while preserving alias declaration identity for source navigation.

## Paste-ready code where safe

Inside `InterfaceBuilder::build` Pass 1:

```rust
Statement::TypeAlias(alias) => {
    let range = (alias.range.start..alias.name_range.end).into();
    Self::validate_dunder(&alias.name, DunderRole::Binding, range)?;
    Self::collect_declaration(
        &alias.name,
        true,
        range,
        &mut namespace,
        &mut declarations,
    )?;
}
```

This is safe if the current `TypeAliasDef` still exposes `name_range` as verified by parser/LSP code.

## What not to change

- Do not create `TypeData::Nominal` for aliases.
- Do not create a class object type for aliases.
- Do not make aliases runtime globals unless the language explicitly reifies type aliases as values.
- Do not bypass module export visibility.
- Do not collapse navigation identity to the underlying target declaration.

## Expected compiler/errors during staging

The interface patch itself should compile directly.

Cross-module semantic tests may initially fail with unresolved alias form if resolver alias-form registration is only workspace-local/current-module. Follow the existing `LinkedTypeResolver::alias_forms` path; do not add another alias table.

If query products omit imported alias shells, expect “missing declaration metadata” failures in session publication. Extend the existing alias `TypeDeclarationShell`, not a parallel representation.

## Rust explanation

A transparent alias has two identities:

- stable declaration identity for source/module graph purposes;
- canonical denotation for type equivalence/application.

Keeping these separate prevents nominality leakage while supporting tooling.

## Tests to add afterward

- exported generic alias through a re-export;
- alias cycle diagnostics if current SC-1 spec defines them;
- alias to constructor-kinded form;
- alias import in LSP go-to-definition/hover.

## Verification commands

```bash
cargo test -p phalcom-modules interface
cargo test -p phalcom-semantic --test semantic imported_resolution
cargo test -p phalcom-semantic --test semantic type_annotations
cargo test -p phalcom-semantic
```

## Completion checklist

- [ ] `InterfaceBuilder` publishes aliases.
- [ ] Alias export/import links canonically.
- [ ] Alias namespace collisions are rejected.
- [ ] Imported aliases expand transparently.
- [ ] Generic aliases work.
- [ ] Navigation keeps alias declaration identity.
- [ ] No nominal/runtime class is fabricated.

---

# Task 16 — Lock the SC-1 fixes already present on current `main`

**Closes:** re-verification of SC1-01, SC1-02, SC1-04, SC1-05, SC1-06, SC1-08  
**Severity:** Certification / regression prevention

## Why

The current repository already contains implementation that the earlier review identified as missing. The danger now is accidentally reintroducing those bugs while performing the Universe refactor.

## Architectural background

Do not rewrite working machinery. Add focused tests that make its invariants non-regressible.

## Current path through the code

Verified current implementation in `phalcom-semantic/src/types/annotation.rs` includes:

- `TypeFormationOutcome<T>`;
- `TypeFormationInvalid::InvalidKindSyntax`;
- `TypeFormationMissing::DeclarationProduct`;
- `TypeLevelBinding::RecordRow`;
- scoped `Bound { depth, index }` lambda lowering;
- `TypeFormationSite::member`;
- explicit generic signature outcome propagation.

`checker/declaration_signature.rs` constructs `TypeFormationSite::member` using the actual callable side.

## Exact files

- Modify: `phalcom-semantic/tests/semantic/foundations/type_annotations.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`
- Possibly modify: generic tests in `generics_core.rs`.

## Exact symbols / insert-replace locations

No production code should be changed unless one of the tests exposes a regression.

Add tests adjacent to existing type-formation tests.

## Tests to add first

Add explicit regression tests for:

1. invalid `KindSyntax` returns `InvalidKindSyntax`, never `Type`;
2. resolved declaration without type publication returns `Missing(DeclarationProduct)`;
3. `<T> =>> List<T>` contains a scoped bound node and beta-reduces correctly;
4. instance `Self` and class-side `Self` carry distinct `DispatchSide`;
5. `RecordRow` binder does not call `TypeStore::parameter_form`/panic;
6. malformed generic constraint prevents signature publication rather than being dropped;
7. `Dynamic` remains distinct from canonical `TypeId`.

These tests should largely pass immediately; their purpose is to certify current state.

## Paste-ready code where safe

No production paste is intended. Use existing public test APIs and pattern-match explicit outcome variants.

## What not to change

- Do not downgrade explicit outcome variants back to `UnknownReason`.
- Do not simplify scoped type lambdas into free canonical types.
- Do not make `Self` infer side from declaration type alone.
- Do not call `parameter_form` on row binders.

## Expected compiler/errors during staging

Tests may need updated imports because `TypeFormationOutcome` replaced the older `Known/Unknown` names. That is test adaptation, not a production regression.

## Rust explanation

Regression tests are especially important here because exhaustive enums and scoped arena types already encode the correct design. A future “simplification” can otherwise silently erase those guarantees.

## Tests to add afterward

No additional production behavior beyond the listed certification matrix is required.

## Verification commands

```bash
cargo test -p phalcom-semantic --test semantic type_annotations
cargo test -p phalcom-semantic --test semantic authority_boundaries
cargo test -p phalcom-semantic
```

## Completion checklist

- [ ] Every already-fixed SC-1 invariant has a focused regression test.
- [ ] No old `Known/Unknown` formation model is reintroduced.
- [ ] No production rewrite was done merely because an old plan said to do it.

---

# 17. Cross-task integration tests

After Tasks 1–16, add a certification module, preferably:

```text
phalcom-semantic/tests/semantic/integration/universe_identity.rs
```

and, for module-only rules:

```text
phalcom-modules/tests/universe_identity.rs
```

The certification suite should prove these end-to-end equalities.

## 17.1 Canonical `Int`

All of the following must converge on:

```text
DeclarationId(
  module = universe.scalar.number,
  name = "Int"
)
```

Paths:

- bare type annotation `Int`;
- `from universe import Int` / current equivalent selective syntax;
- direct owner-module import;
- root export target;
- semantic hover target;
- go-to-definition target;
- stable metadata declaration;
- runtime prelude binding target.

## 17.2 Prelude visibility

For each `UNIVERSE_BINDINGS` row:

```text
prelude=true  => bare semantic type lookup succeeds (when it denotes a type)
prelude=false => bare lookup does not select that Universe declaration
```

Explicit import remains available according to export/package policy.

## 17.3 No root declaration duplication

Assert there is no canonical declaration:

```text
universe_root::Int
universe_root::List
universe_root::Object
universe_root::Result
```

Root aliases are exports only.

## 17.4 Runtime identity

User classes named:

```text
List Map Option Result Some
```

must not inherit builtin generic arity or native identity.

## 17.5 Source/native agreement

For every ordinary native Universe declaration:

- source owner module exists;
- source declaration exists;
- native source path agrees;
- generic arity/kinds agree;
- hierarchy agrees;
- native surface owner maps to same declaration.

---

# 18. Expected final code ownership

After implementation, the intended ownership should be:

```text
phalcom-modules
  UniverseSourceProvider
  UniverseSourceDeclarationCatalog
  InterfaceBuilder
  ModuleResolver
  ModuleLinker
  package expose semantics
  canonical root alias targets

phalcom-semantic
  UniverseSourceBaseline consumption
  UniverseSemanticBaseline (store-local)
  canonical prelude type map
  DeclarationId / TypeId / KindId
  generics / aliases / ADTs / hierarchy
  native/source conformance

phalcom-core
  primordial physical allocation
  runtime ModuleObject/ClassId
  native implementation execution
  stable semantic metadata loading
  runtime reflection by identity
```

No layer should reconstruct declaration identity from a display name.

---

# 19. Full verification matrix

Run in this order so failures are localizable.

## 19.1 Module subsystem

```bash
cargo check -p phalcom-modules
cargo test -p phalcom-modules --test builtin_catalog
cargo test -p phalcom-modules
```

## 19.2 Semantic subsystem

```bash
cargo check -p phalcom-semantic
cargo test -p phalcom-semantic --test semantic type_annotations
cargo test -p phalcom-semantic --test semantic imported_resolution
cargo test -p phalcom-semantic --test semantic native_conformance
cargo test -p phalcom-semantic --test semantic authority_boundaries
cargo test -p phalcom-semantic
```

## 19.3 Runtime/reflection

```bash
cargo check -p phalcom-core
cargo test -p phalcom-core --test core reflection
cargo test -p phalcom-core --test core modules
cargo test -p phalcom-core --test core
```

## 19.4 Whole workspace

```bash
cargo build --workspace --all-targets
cargo test --workspace --all-targets
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## 19.5 Legacy/name-reconstruction audits

```bash
rg 'UniverseKey::from_name' \
  phalcom-semantic/src \
  phalcom-core/src \
  phalcom-modules/src

rg 'owner\.name\(\)\s*==|\.name\(\)\s*==' \
  phalcom-core/src/typing \
  phalcom-semantic/src

rg 'namespace:\s*"core"|namespace:\s*"std"' \
  phalcom-core/tests/core/reflection \
  phalcom-semantic/src \
  phalcom-core/src

rg 'ModuleId::core|BuiltinPackage::Std|BuiltinPackage::Universe' .
```

Every remaining name-based lookup must be manually classified:

- source parsing/discovery: potentially legitimate;
- post-resolution semantic/runtime identity: suspicious and should normally be removed.

---

# 20. Commit sequence

Recommended commits:

```text
1. ci: align workspace verification on pinned nightly
2. modules: add canonical Universe source declaration catalog
3. modules: make Universe root aliases preserve declaration ownership
4. semantic: consume canonical Universe prelude bindings
5. semantic: factor source-derived Universe baseline
6. runtime: derive generic arity from class identity
7. universe: repair Option generic contracts
8. modules: apply expose policy to Universe imports
9. modules: centralize logical import target resolution
10. semantic: reject unsupported deep qualified type references
11. semantic: strengthen native/source conformance
12. runtime: make native source association module-aware
13. tests: retire core/std stable identity fixtures
14. semantic: remove dead legacy-core dependency exclusion
15. semantic: reject open record tails before SC-3
16. modules/semantic: finish imported/exported type aliases
17. tests: certify already-completed SC-1 formation invariants
18. tests: add cross-layer Universe identity certification
```

Do not squash these until review is complete; the staged history makes regression localization substantially easier.

---

# 21. Final completion gate

Do not declare the baseline ready for SC-1/SC-2 until all are true:

- [ ] CI is green on the canonical pinned toolchain.
- [ ] `phalcom-modules` never creates synthetic root declarations for canonical Universe classes/types.
- [ ] Root convenience exports preserve actual source-owner `SymbolId`.
- [ ] semantic bare-type lookup uses an explicit prelude map.
- [ ] non-prelude Universe declarations do not leak into bare type scope.
- [ ] ordinary Universe declaration kinds/generic arity come from source.
- [ ] native metadata mismatches source fail conformance.
- [ ] runtime generic arity never depends on a class display name.
- [ ] `Option.unwrapOr` is sound.
- [ ] Universe imports obey package `expose`.
- [ ] runtime Universe dependency calculation uses module-layer path semantics.
- [ ] qualified type resolution never discards components.
- [ ] native source indexing validates owner module as well as name.
- [ ] reflection fixtures no longer model retired `core`/`std`.
- [ ] dead legacy-core dependency special-casing is removed.
- [ ] ordinary open record tails are rejected rather than erased.
- [ ] type aliases are module declarations and import/export correctly.
- [ ] already-fixed SC-1 formation invariants have regression tests.
- [ ] whole-workspace build/test/fmt/clippy pass.

At that point the baseline has the intended architecture:

```text
source
  ↓
phalcom-modules canonical module/export identity
  ↓
phalcom-semantic canonical declaration/type identity
  ↓
native implementation attachment + conformance
  ↓
runtime materialization/reflection
```

and SC-1/SC-2 can proceed without inheriting a second hidden Universe identity system.
