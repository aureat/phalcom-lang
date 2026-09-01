# Phalcom Post-Universe Integration Correctness Remediation — Detailed Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Correct the concrete semantic, runtime, module-system, metadata, bootstrap, and build-quality defects identified by the post-Universe integration review, while preserving the new canonical Universe identity architecture.

**Architecture:** Keep `ProjectIdentity::Universe`, real source-owned `DeclarationId`s, semantic lowering, and compiler-owned immutable semantic products as the authorities. Remove remaining shortcuts that infer meaning by spelling, bypass package visibility, create duplicate runtime identities, or treat runtime-support metadata as source-visible semantics. Each workstream establishes one cross-layer invariant and then adds regression tests proving all consumers agree.

**Tech Stack:** Rust, `phalcom-modules`, `phalcom-semantic`, `phalcom-core`, `phalcom-native-meta`, `phalcom-type-meta`, Phalcom Universe source, Cargo stable CI, existing integration-test harnesses.

**Repository baseline:** `main @ 1c78f5d23f11865dc5e3d55e15b6f9b48a927bcc` (`chore: remove stale ADT debug artifacts`). Rebase this plan if `main` moves before execution; do not blindly apply exact replacement blocks to a different revision.

**Companion plan:** `phalcom-native-result-lightweight-representation-plan.md` handles the separate native/immediate `Result` representation gap after the correctness work in this document is complete.

---

## 0. How to use this plan

This is intentionally more detailed than a normal maintainer checklist. It assumes the implementer knows Rust syntax but does not yet know Phalcom's compiler architecture.

For every task:

1. Read the **Why this layer owns the fix** section before editing.
2. Make the test change first when the task supplies a red-green test.
3. Run the narrow test command immediately; do not accumulate multiple unrelated edits before compiling.
4. Apply the code edit exactly at the named symbol or block.
5. Run the narrow test again.
6. Run the crate-level gate.
7. Commit that task independently unless it is explicitly marked as part of the same atomic change as the next task.

Do not “simplify” a task by reintroducing name-based builtin recognition. The core invariant throughout this remediation is:

> **Source spelling is not semantic identity.** Resolve to `ModuleId`, `DeclarationId`, `VariantId`, or a linked target first, then make runtime/tooling decisions from that identity.

---

# 1. Review findings mapped to tasks

| Review ID | Severity | Finding | Tasks |
|---|---:|---|---|
| B-01 | Blocker | current `main` CI does not reach tests | 1, 2, 18 |
| M-05 | Medium | repository Cargo config is host/nightly-specific | 1 |
| C-01 | Critical | canonical `Result`/`Ordering` can receive duplicate runtime root classes | 3, 4 |
| C-02 | Critical | type resolver ignores explicit prelude policy and leaks runtime-support types | 5, 6 |
| C-03 | Critical | `Option<T>` public signatures are unsound/overly Dynamic | 7, 8 |
| H-01 | High | Universe import resolution bypasses ordinary exposure semantics and relative resolution | 9, 10 |
| H-03 | High | fallback match lowering guesses builtin variants by name | 11 |
| H-05 | High | stable metadata uses session-local `proj#N` plus zero fingerprint | 12, 13 |
| M-03 | Medium | builtin interface post-processing exports every non-root declaration | 14 |
| M-02 | Medium | Universe `__package__` semantics differ from ordinary packages | 15 |
| M-04 | Medium | canonical `Error` variant coexists with compiler/test `Err` assumptions | 11, 16 |
| H-02 | High | bootstrap topologically executes the entire Universe corpus | 17 |
| M-01 | Medium | Universe semantic bootstrap is rebuilt inline per workspace session | 19 |
| H-04 | Feature gap | lightweight native `Result` is not implemented | companion plan |

The ordering is deliberate. Do not start with the semantic baseline extraction: first make the current semantics correct, then extract the stable baseline without changing behavior.

---

# 2. Architectural orientation for a new contributor

Before editing, understand the main ownership boundaries.

```text
phalcom-native-meta
    stable catalog keys and native/builtin policy metadata
    (UniverseKey, prelude/export/native relationships)

phalcom-modules
    canonical project/module/package identity
    source discovery
    package exposure
    imports/exports/re-exports/expose
    linked module products

phalcom-semantic
    DeclarationId / VariantId meaning
    type forms, generics, constraints, hierarchy
    ADT semantics and exact-case proof products
    source indexes and editor semantic targets

phalcom-core/src/modules/semantic_lowering.rs
    projection from formal semantic products to backend-facing specs

phalcom-core/src/vm/*
    runtime class/value/ADT realization

phalcom-core/src/modules/materialize.rs
    maps compiled linked modules and semantic metadata into VM objects

phalcom-lsp
    protocol adapter over semantic/module query products
    (should not independently infer module/type meaning)
```

The most important distinction for the ADT fixes is:

```text
DeclarationId      semantic nominal identity
VariantId          semantic exact variant identity
ClassId            VM-local runtime behavior identity
RuntimeEnumId      VM-local ADT registry identity
RuntimeVariantId   VM-local runtime variant descriptor identity
```

Do not collapse them. The fix for C-01 is not “make `DeclarationId == ClassId`”; it is “one `DeclarationId` must map to one authoritative root `ClassId`.”

---

# Phase A — Restore a trustworthy build/verification baseline

## Task 1: Make checked-in Cargo configuration portable and stable-compatible

**Review coverage:** B-01, M-05

**Why this layer owns the fix:** `.cargo/config.toml` applies before any crate code is compiled. Host-specific `sccache`, `target-cpu=native`, and nightly `-Z` flags therefore contaminate every workspace consumer, including CI. Local performance tuning is a developer-machine concern, not a repository semantic requirement.

**Files:**
- Modify: `.cargo/config.toml`
- Create: `docs/development/local-cargo-tuning.md`

**Current block to remove:** the reviewed baseline contains `[unstable]`, `[resolver] feature-unification`, `rustflags = ["-Zunstable-options", "-Zthreads=6", "-Ctarget-cpu=native"]`, `rustc-wrapper = "sccache"`, and `jobs = 8`.

### Steps

- [ ] **Step 1: Replace `.cargo/config.toml` with a portable minimum.**

Open `.cargo/config.toml` and replace the entire file with:

```toml
[env]
# Some compiler/runtime integration tests still require a larger worker stack.
# Keep this repository-level compatibility setting until the specific deep
# recursion sites are removed; do not combine it with machine-specific build
# acceleration settings.
RUST_MIN_STACK = "33554432"
```

Do **not** keep any of these in checked-in Cargo configuration:

```text
-Zunstable-options
-Zthreads
-Ctarget-cpu=native
rustc-wrapper = "sccache"
jobs = <machine-specific value>
```

- [ ] **Step 2: Document optional local tuning instead of enforcing it.**

Create `docs/development/local-cargo-tuning.md` with:

```markdown
# Optional local Cargo tuning

The checked-in `.cargo/config.toml` is intentionally portable and must work
with the stable toolchain used by CI.

Developers may configure local-only settings outside the repository, for
example in `$CARGO_HOME/config.toml`:

```toml
[build]
rustc-wrapper = "sccache"
jobs = 8
```

Nightly experiments such as `-Zthreads` must be invoked explicitly with a
nightly toolchain and must not be committed as workspace defaults.

Likewise, `-Ctarget-cpu=native` is suitable only for local benchmarking. It
must not be a repository default because it produces host-specific build
artifacts and interacts poorly with shared caches.
```

- [ ] **Step 3: Verify stable Cargo can parse the workspace configuration.**

Run:

```bash
cargo +stable metadata --no-deps --format-version 1 >/dev/null
```

Expected: exit code `0`.

- [ ] **Step 4: Compile the smallest affected workspace surface.**

Run:

```bash
cargo +stable check -p phalcom-ast
```

Expected: exit code `0`. If it does not, stop: that failure is now a real crate/toolchain failure rather than a missing local wrapper or nightly flag.

- [ ] **Step 5: Commit this infrastructure-only change.**

```bash
git add .cargo/config.toml docs/development/local-cargo-tuning.md
git commit -m "fix(build): make workspace Cargo config portable"
```

**Do not** mix compiler semantic code into this commit.

---

## Task 2: Restore formatting and establish the red/green verification commands

**Review coverage:** B-01

**Files:**
- Modify: every Rust file touched only by rustfmt as reported by the command
- No semantic edits in this task

### Steps

- [ ] **Step 1: Record the current formatting failure before changing files.**

```bash
cargo +stable fmt --all -- --check
```

Expected at the reviewed SHA: failure.

- [ ] **Step 2: Apply rustfmt.**

```bash
cargo +stable fmt --all
```

- [ ] **Step 3: Verify format is clean.**

```bash
cargo +stable fmt --all -- --check
```

Expected: exit code `0`.

- [ ] **Step 4: Run the workspace build before semantic remediation begins.**

```bash
cargo +stable build --workspace --all-targets
```

This command is a **baseline diagnostic gate**. If it still fails after Task 1, save the compiler output with the task/commit that owns it. Do not claim the semantic tests pass until this build succeeds.

- [ ] **Step 5: Once build succeeds, run the workspace tests once and save the failure list.**

```bash
cargo +stable test --workspace --all-targets
```

Any failures at this point are the pre-remediation baseline. Later tasks must not silently delete or ignore them.

- [ ] **Step 6: Commit only formatting changes.**

```bash
git add -u
git commit -m "style: apply workspace rustfmt"
```

---

# Phase B — Canonical runtime enum root identity

## Task 3: Reuse primordial root classes for canonical Universe enums

**Review coverage:** C-01

**Invariant after this task:**

```text
Universe Option DeclarationId   -> exactly one Option root ClassId
Universe Result DeclarationId   -> exactly one Result root ClassId
Universe Ordering DeclarationId -> exactly one Ordering root ClassId
```

User-defined enums with the same source spelling must still allocate distinct root classes.

**Files:**
- Modify: `phalcom-core/src/vm/adt.rs`
- Test: `phalcom-core/tests/native_adt_runtime.rs`

**Interfaces consumed:**
- `phalcom_semantic::core_surface::CoreDeclarationIds`
- `phalcom_native_meta::UniverseKey`
- `VM::universe.classes.resolve(UniverseKey)`
- existing `RuntimeEnumClassBinding`

**Interfaces produced:**
- helper `canonical_universe_enum_root(&self, owner: &DeclarationId) -> Option<ClassId>`
- helper `allocate_general_variant_classes(&mut self, spec: &EnumLoweringSpec, root_class_id: ClassId) -> Result<BTreeMap<VariantId, ClassId>, RuntimeError>`

### Step 3.1 — Add imports needed for identity-based root selection

- [ ] Open `phalcom-core/src/vm/adt.rs`.
- [ ] Find the existing imports:

```rust
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::VariantId;
```

- [ ] Replace the second line with a grouped import and add the core-ID helper:

```rust
use phalcom_semantic::core_surface::CoreDeclarationIds;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{DeclarationId, VariantId};
```

If `DeclarationId` is not publicly re-exported from `phalcom_semantic::identity` at execution time, import it from `phalcom_modules::DeclarationId` instead. Use the same concrete `DeclarationId` type already carried by `EnumLoweringSpec::owner`; do not introduce a new wrapper.

### Step 3.2 — Add a full-identity root lookup helper

- [ ] In `impl VM`, immediately before `bind_native_option_classes`, insert:

```rust
fn canonical_universe_enum_root(&self, owner: &DeclarationId) -> Option<ClassId> {
    let ids = CoreDeclarationIds::default();

    let key = if ids.is_option(owner) {
        phalcom_native_meta::UniverseKey::Option
    } else if ids.is_result(owner) {
        phalcom_native_meta::UniverseKey::Result
    } else if ids.is_ordering(owner) {
        phalcom_native_meta::UniverseKey::Ordering
    } else {
        return None;
    };

    Some(self.universe.classes.resolve(key))
}
```

**Why:** this compares the entire canonical `DeclarationId`, including owning `ModuleId`. A user declaration named `Result` in `app.models` therefore cannot match `universe.errors.result::Result`.

**Do not write:**

```rust
if owner.name.as_ref() == "Result" { ... }
```

That recreates the identity bug in a new location.

### Step 3.3 — Split variant-class allocation from root-class allocation

- [ ] Find `fn allocate_general_enum_classes`.
- [ ] Extract the variant loop into a new helper placed immediately before it:

```rust
fn allocate_general_variant_classes(
    &mut self,
    spec: &EnumLoweringSpec,
    root_class_id: ClassId,
) -> Result<BTreeMap<VariantId, ClassId>, RuntimeError> {
    let mut variants = BTreeMap::new();

    for var_spec in spec.variants.iter() {
        let case_class_name = format!("{}::{}", spec.owner.name, var_spec.id.selector);
        let mut case_class = ClassObject::bare(&case_class_name);
        case_class.class = self.universe.classes.class_class;
        case_class.superclass = Some(root_class_id);
        let case_class_id = self.heap.alloc_class(case_class);
        variants.insert(var_spec.id.clone(), case_class_id);
    }

    Ok(variants)
}
```

- [ ] Replace the body of `allocate_general_enum_classes` with:

```rust
fn allocate_general_enum_classes(
    &mut self,
    spec: &EnumLoweringSpec,
) -> Result<RuntimeEnumClassBinding, RuntimeError> {
    let mut root_class = ClassObject::bare(&spec.owner.name);
    root_class.class = self.universe.classes.class_class;
    root_class.superclass = Some(self.universe.classes.object_class);
    let root_class_id = self.heap.alloc_class(root_class);

    let variants = self.allocate_general_variant_classes(spec, root_class_id)?;

    Ok(RuntimeEnumClassBinding {
        root: root_class_id,
        variants,
    })
}
```

The behavior for ordinary user enums must remain byte-for-byte equivalent at the conceptual level: a fresh root plus fresh hidden case behavior classes.

### Step 3.4 — Add canonical Universe binding for General enums

- [ ] Immediately after `allocate_general_enum_classes`, insert:

```rust
fn bind_canonical_universe_enum_classes(
    &mut self,
    spec: &EnumLoweringSpec,
) -> Result<Option<RuntimeEnumClassBinding>, RuntimeError> {
    let Some(root) = self.canonical_universe_enum_root(&spec.owner) else {
        return Ok(None);
    };

    let ids = CoreDeclarationIds::default();
    if ids.is_option(&spec.owner) {
        return self.bind_native_option_classes(spec).map(Some);
    }

    let variants = self.allocate_general_variant_classes(spec, root)?;
    Ok(Some(RuntimeEnumClassBinding { root, variants }))
}
```

This deliberately keeps Result/Ordering **value representation** as `General` for now. The only thing reused here is the nominal root class.

### Step 3.5 — Change `class_binding_for_enum`

- [ ] Replace the existing function:

```rust
fn class_binding_for_enum(
    &mut self,
    spec: &EnumLoweringSpec,
) -> Result<RuntimeEnumClassBinding, RuntimeError> {
    match spec.representation {
        crate::adt::RuntimeAdtRepresentation::NativeOption => self.bind_native_option_classes(spec),
        crate::adt::RuntimeAdtRepresentation::General => self.allocate_general_enum_classes(spec),
    }
}
```

with:

```rust
fn class_binding_for_enum(
    &mut self,
    spec: &EnumLoweringSpec,
) -> Result<RuntimeEnumClassBinding, RuntimeError> {
    if let Some(binding) = self.bind_canonical_universe_enum_classes(spec)? {
        return Ok(binding);
    }

    match spec.representation {
        crate::adt::RuntimeAdtRepresentation::NativeOption => Err(RuntimeError::Internal(
            "NativeOption representation is reserved for canonical universe Option".into(),
        )),
        crate::adt::RuntimeAdtRepresentation::General => self.allocate_general_enum_classes(spec),
    }
}
```

This additionally prevents a user enum from accidentally receiving Option's physical representation merely because a malformed lowering spec says `NativeOption`.

### Step 3.6 — Add canonical Result/Ordering regression tests

The existing `phalcom-core/tests/native_adt_runtime.rs` incorrectly constructs `Option` as `ModuleId::universe_root()` even though canonical Option now lives in `universe.option.option`. Do not copy that mistake into new tests.

- [ ] Add these imports to the test file:

```rust
use phalcom_native_meta::UniverseKey;
use phalcom_semantic::core_surface::universe_declaration;
```

- [ ] Add a small helper:

```rust
fn empty_general_enum(owner: DeclarationId) -> EnumLoweringSpec {
    EnumLoweringSpec {
        owner,
        representation: phalcom_core::adt::RuntimeAdtRepresentation::General,
        variants: Box::new([]),
    }
}
```

This helper is only for root-identity tests. Variant behavior is covered by existing ADT tests.

- [ ] Add:

```rust
#[test]
fn canonical_result_reuses_primordial_runtime_root() {
    let mut vm = VM::new();
    let result_decl = universe_declaration(UniverseKey::Result);
    let expected = vm.universe.classes.resolve(UniverseKey::Result);

    let actual = vm
        .register_enum_from_spec(&empty_general_enum(result_decl))
        .expect("register canonical Result");

    assert_eq!(actual, expected);
}

#[test]
fn canonical_ordering_reuses_primordial_runtime_root() {
    let mut vm = VM::new();
    let ordering_decl = universe_declaration(UniverseKey::Ordering);
    let expected = vm.universe.classes.resolve(UniverseKey::Ordering);

    let actual = vm
        .register_enum_from_spec(&empty_general_enum(ordering_decl))
        .expect("register canonical Ordering");

    assert_eq!(actual, expected);
}
```

- [ ] Add an anti-regression test proving full identity is used:

```rust
#[test]
fn user_result_does_not_reuse_universe_result_root() {
    let mut vm = VM::new();
    let mut ids = phalcom_modules::SyntheticProjectIdAllocator::default();
    let owner = DeclarationId::new(
        ModuleId::synthetic(ids.allocate(), phalcom_modules::ModulePath::root()),
        "Result".into(),
    );

    let universe_result = vm.universe.classes.resolve(UniverseKey::Result);
    let user_root = vm
        .register_enum_from_spec(&empty_general_enum(owner))
        .expect("register user Result");

    assert_ne!(user_root, universe_result);
}
```

### Step 3.7 — Run narrow gates

```bash
cargo +stable test -p phalcom-core --test native_adt_runtime
cargo +stable check -p phalcom-core
```

Expected: all pass.

### Step 3.8 — Commit

```bash
git add phalcom-core/src/vm/adt.rs phalcom-core/tests/native_adt_runtime.rs
git commit -m "fix(adt): reuse canonical Universe enum roots"
```

---

## Task 4: Add cross-registry runtime identity assertions

**Review coverage:** C-01

Task 3 prevents duplicate root allocation. This task ensures another registry cannot silently keep using the wrong root.

**Files:**
- Modify/Test: `phalcom-core/tests/native_adt_runtime.rs`
- Inspect while implementing: `phalcom-core/src/modules/materialize.rs`, especially Phase 8 semantic metadata registration

### Steps

- [ ] **Step 1: Add an ADT-registry accessor assertion to the Result test.**

After `register_enum_from_spec`, add:

```rust
let enum_id = vm
    .adt_registry
    .enum_by_declaration(&result_decl)
    .expect("Result ADT descriptor");
let descriptor = vm
    .adt_registry
    .enum_descriptor(enum_id)
    .expect("Result enum descriptor");

assert_eq!(descriptor.root_class, expected);
```

If `adt_registry` is not public to integration tests, expose a read-only VM test/query method rather than making registry internals public wholesale:

```rust
pub fn runtime_enum_root_for_declaration(
    &self,
    declaration: &DeclarationId,
) -> Option<ClassId> {
    let enum_id = self.adt_registry.enum_by_declaration(declaration)?;
    self.adt_registry.enum_descriptor(enum_id).map(|desc| desc.root_class)
}
```

Place that method in `phalcom-core/src/vm/adt.rs` near `case_behavior_class`.

- [ ] **Step 2: Add the equivalent Ordering assertion.**

- [ ] **Step 3: Add a comment in `phalcom-core/src/modules/materialize.rs` Phase 8 directly above the `UNIVERSE_BINDINGS` loop:**

```rust
// IMPORTANT: canonical Universe ADTs must already have registered the same
// root ClassId in RuntimeAdtRegistry. This loop binds semantic metadata to the
// primordial class table; it must never manufacture or substitute another
// enum root.
```

Do not change runtime behavior in `materialize.rs` if Task 3 makes the IDs agree.

- [ ] **Step 4: Run:**

```bash
cargo +stable test -p phalcom-core --test native_adt_runtime
cargo +stable test -p phalcom-core adt
```

- [ ] **Step 5: Commit.**

```bash
git add phalcom-core/src/vm/adt.rs phalcom-core/src/modules/materialize.rs phalcom-core/tests/native_adt_runtime.rs
git commit -m "test(adt): enforce canonical runtime root identity"
```

---

# Phase C — Make prelude visibility explicit instead of inferred from UniverseKey

## Task 5: Introduce a canonical semantic prelude type map

**Review coverage:** C-02

**Why this layer owns the fix:** `phalcom-native-meta` owns policy flags (`prelude`, `exported`, runtime-support classification), but `phalcom-semantic` owns whether an unqualified source type name resolves. The semantic layer should therefore lower native policy into a canonical read-only map of source-visible prelude type names.

**Files:**
- Create: `phalcom-semantic/src/prelude.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-semantic/src/resolver.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Create: `phalcom-semantic/tests/semantic/integration/prelude.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/mod.rs`

**New interface:**

```rust
#[derive(Clone, Debug, Default)]
pub struct PreludeTypeMap { ... }

impl PreludeTypeMap {
    pub fn canonical_universe() -> Self;
    pub fn get(&self, name: &str) -> Option<&DeclarationId>;
    pub fn contains_name(&self, name: &str) -> bool;
}
```

### Step 5.1 — Create `prelude.rs`

- [ ] Create `phalcom-semantic/src/prelude.rs` with:

```rust
use crate::core_surface::universe_declaration;
use crate::identity::DeclarationId;
use phalcom_native_meta::{UNIVERSE_BINDINGS, UniverseBindingKind};
use std::collections::HashMap;

/// Canonical source-visible type names supplied implicitly by the Phalcom
/// prelude. Entries point directly at their real Universe source declarations;
/// the prelude does not create alias declarations or a synthetic module.
#[derive(Clone, Debug, Default)]
pub struct PreludeTypeMap {
    entries: HashMap<Box<str>, DeclarationId>,
}

impl PreludeTypeMap {
    pub fn canonical_universe() -> Self {
        let mut entries = HashMap::new();

        for binding in UNIVERSE_BINDINGS {
            if !binding.prelude {
                continue;
            }
            if binding.kind == UniverseBindingKind::RuntimeSupportClass {
                continue;
            }

            entries.insert(binding.name.into(), universe_declaration(binding.key));
        }

        Self { entries }
    }

    pub fn get(&self, name: &str) -> Option<&DeclarationId> {
        self.entries.get(name)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &DeclarationId)> {
        self.entries.iter().map(|(name, declaration)| (name.as_ref(), declaration))
    }
}
```

If `UniverseBindingKind` is not re-exported from the crate root, import it from `phalcom_native_meta::universe::UniverseBindingKind` instead. Do not change the policy source; the point is to consume existing flags correctly.

### Step 5.2 — Export the module

- [ ] Open `phalcom-semantic/src/lib.rs`.
- [ ] Add:

```rust
pub mod prelude;
```

near other semantic infrastructure modules.

### Step 5.3 — Replace `LinkedTypeResolver`'s fake prelude module with the explicit map

Open `phalcom-semantic/src/resolver.rs`.

- [ ] Remove these imports:

```rust
use crate::core_surface::universe_declaration;
use phalcom_native_meta::UniverseKey;
```

- [ ] Add:

```rust
use crate::prelude::PreludeTypeMap;
```

- [ ] Change the struct from:

```rust
pub struct LinkedTypeResolver {
    linked: Arc<LinkedProgram>,
    known_declarations: HashSet<DeclarationId>,
    prelude_module: ModuleId,
}
```

into:

```rust
pub struct LinkedTypeResolver {
    linked: Arc<LinkedProgram>,
    known_declarations: HashSet<DeclarationId>,
    prelude_types: Arc<PreludeTypeMap>,
}
```

- [ ] Replace the constructor with:

```rust
pub fn new(
    linked: Arc<LinkedProgram>,
    known_declarations: HashSet<DeclarationId>,
    prelude_types: Arc<PreludeTypeMap>,
) -> Self {
    Self {
        linked,
        known_declarations,
        prelude_types,
    }
}
```

- [ ] In `resolve_type_name`, delete both old fallback blocks:

```rust
let prelude_decl = DeclarationId::new(self.prelude_module.clone(), root.into());
...
if let Some(key) = UniverseKey::from_name(root) {
    ...
}
```

- [ ] Replace them with exactly one policy lookup:

```rust
// 4. Canonical prelude declaration. The map contains only names explicitly
// marked as prelude-visible and points at their real source-owned DeclarationId.
if let Some(decl) = self.prelude_types.get(root) {
    if self.known_declarations.contains(decl) {
        return Some(decl.clone());
    }
}
```

The lookup precedence remains:

```text
local declaration
selective import
re-export/current linked namespace
prelude
```

### Step 5.4 — Update `SemanticWorkspaceSession::with_workspace`

Open `phalcom-semantic/src/session.rs`.

- [ ] Add:

```rust
use crate::prelude::PreludeTypeMap;
```

- [ ] Immediately after building `known_declarations`, construct:

```rust
let prelude_types = Arc::new(PreludeTypeMap::canonical_universe());
```

- [ ] Replace:

```rust
let resolver = LinkedTypeResolver::new(
    dummy_linked,
    known_declarations,
    ModuleId::universe_root(),
);
```

with:

```rust
let resolver = LinkedTypeResolver::new(
    dummy_linked,
    known_declarations,
    prelude_types,
);
```

- [ ] Search the same file for every later `LinkedTypeResolver::new(...)`. Pass `Arc::new(PreludeTypeMap::canonical_universe())` only if the resolver is genuinely short-lived and no shared session field exists yet. Prefer adding a session field in Task 6 rather than rebuilding repeatedly.

### Step 5.5 — Register the new tests

- [ ] Add `mod prelude;` to `phalcom-semantic/tests/semantic/integration/mod.rs`.

- [ ] Create `phalcom-semantic/tests/semantic/integration/prelude.rs`.

Use the existing `SemanticWorkspaceSession`/workspace fixture style from `workspace.rs` and `imported_resolution.rs`. Add tests with these source snippets:

```phalcom
const x: Int = 1
const y: Option<Int> = Option::None
const z: Result<Int, Error> = Result::Ok(1)
```

Assert no `AnnotationUnresolved` diagnostic for `Int`, `Option`, `Result`, or `Error`.

Add negative snippets:

```phalcom
const x: Nil = 1
```

and, separately:

```phalcom
const x: Some = 1
const y: None = 1
```

For each non-prelude/runtime-support name, assert the annotation receives `DiagnosticCode::AnnotationUnresolved` unless an explicit import intentionally makes that declaration visible.

### Step 5.6 — Run

```bash
cargo +stable test -p phalcom-semantic prelude
cargo +stable test -p phalcom-semantic imported_resolution
cargo +stable check -p phalcom-semantic
```

### Step 5.7 — Commit

```bash
git add phalcom-semantic/src/prelude.rs \
        phalcom-semantic/src/lib.rs \
        phalcom-semantic/src/resolver.rs \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/integration/mod.rs \
        phalcom-semantic/tests/semantic/integration/prelude.rs
git commit -m "fix(semantic): enforce explicit Universe prelude policy"
```

---

## Task 6: Store the prelude map once per semantic session and use it for editor visibility

**Review coverage:** C-02, part of M-01

Task 5 fixes type lookup. This task prevents later editor/completion code from constructing a different implicit builtin set.

**Files:**
- Modify: `phalcom-semantic/src/session.rs`
- Inspect/modify as needed: source-index visible-symbol construction in `phalcom-semantic/src/source_index/*`
- Test: `phalcom-semantic/tests/semantic/integration/editor.rs`
- Test: `phalcom-semantic/tests/semantic/integration/prelude.rs`

### Steps

- [ ] **Step 1: Add a field to `SemanticWorkspaceSession`:**

```rust
prelude_types: Arc<PreludeTypeMap>,
```

Place it next to `base_declarations`, because it is part of the immutable Universe-derived baseline rather than user workspace state.

- [ ] **Step 2: In `with_workspace`, construct it exactly once:**

```rust
let prelude_types = Arc::new(PreludeTypeMap::canonical_universe());
```

Use `prelude_types.clone()` for every `LinkedTypeResolver` created during session setup or updates.

- [ ] **Step 3: Add the field to the final `Self { ... }` initializer.**

- [ ] **Step 4: Search production semantic code for `UNIVERSE_BINDINGS`, `UniverseKey::from_name`, or manual lists used only to decide unqualified editor visibility.**

Any such editor-visibility path should consume `self.prelude_types.iter()` instead of inventing another list. Native conformance/runtime bootstrap code may still use `UNIVERSE_BINDINGS`; do not mechanically replace every occurrence.

- [ ] **Step 5: Add editor regression coverage.**

At a normal expression position, `visible_symbols_at` should include prelude names such as `Int`, `Option`, and `Result`, and exclude non-prelude `Nil` and runtime-support `Some`/`None` class names. `Option::Some`/`Option::None` remain discoverable through associated/variant completion, not as top-level nominal type names.

- [ ] **Step 6: Run:**

```bash
cargo +stable test -p phalcom-semantic editor
cargo +stable test -p phalcom-semantic prelude
```

- [ ] **Step 7: Commit.**

```bash
git add phalcom-semantic/src/session.rs phalcom-semantic/src/source_index phalcom-semantic/tests/semantic/integration
git commit -m "fix(editor): derive implicit symbols from semantic prelude map"
```

---

# Phase D — Make `Option<T>` formally sound

## Task 7: Replace unsound/Dynamic Option signatures with generic contracts

**Review coverage:** C-03

**Why this layer owns the fix:** The `.ph` declaration is the language contract. Native implementation does not justify `Dynamic` when the generic relationship is statically expressible.

**Files:**
- Modify: `phalcom-core/core/universe/src/option/option.ph`
- Inspect: native method descriptor/conformance output for Option; do not weaken the source declaration to satisfy stale metadata

### Ratified target signatures for this remediation

Use the conservative, sound APIs below:

```phalcom
@native
match<R>(
  some: (value: T) -> R,
  none: () -> R
) -> R

ifNone(_ f: () -> Unit) -> Option<T>

orElse(_ f: () -> Option<T>) -> Option<T>

map<U>(_ f: (value: T) -> U) -> Option<U>

flatMap<U>(_ f: (value: T) -> Option<U>) -> Option<U>

filter(_ pred: (value: T) -> Bool) -> Option<T>

ifSome(_ f: (value: T) -> Unit) -> Option<T>

unwrapOr(_ default: T) -> T

okOr<E>(_ err: E) -> Result<T, E>
```

Do not use the previously unsound `unwrapOr<U>(_ default: U) -> U` unless a future language/API decision introduces a constraint such as `T <: U` and the inference engine intentionally supports that more general contract.

### Step 7.1 — Replace `match`

Find:

```phalcom
@native
match(some: Dynamic, none: Dynamic) -> Dynamic
```

Replace with:

```phalcom
@native
match<R>(
  some: (value: T) -> R,
  none: () -> R
) -> R
```

### Step 7.2 — Type observation helpers

Replace:

```phalcom
ifNone(_ f) -> Self
```

with:

```phalcom
ifNone(_ f: () -> Unit) -> Option<T>
```

Replace:

```phalcom
ifSome(_ f) -> Self
```

with:

```phalcom
ifSome(_ f: (value: T) -> Unit) -> Option<T>
```

Using `Option<T>` rather than `Self` keeps the public generic relationship explicit while the semantics of `Self` are still being expanded elsewhere. If the repository's current semantic tests prove `Self` preserves the applied generic receiver exactly, retaining `Self` is acceptable; do not accept `Dynamic`.

### Step 7.3 — Replace `orElse`

Find:

```phalcom
orElse(_ f) -> Self | Dynamic
```

Replace with:

```phalcom
orElse(_ f: () -> Option<T>) -> Option<T>
```

The existing body remains valid:

```phalcom
match(
  some: |v| self,
  none: || f.call()
)
```

### Step 7.4 — Replace `map`

Find the full declaration beginning:

```phalcom
map(_ f) -> Self | Option<Dynamic>
```

Change only the signature to:

```phalcom
map<U>(_ f: (value: T) -> U) -> Option<U>
```

Keep the body:

```phalcom
match(
  some: |v| Option::Some(f.call(v)),
  none: || self
)
```

The `None` branch is valid for every `Option<U>` because the nullary `None` case carries no `T` payload and its exact case refines into any applied `Option<U>` through the enum's generic case semantics.

### Step 7.5 — Replace `flatMap`

Change:

```phalcom
flatMap(_ f) -> Self | Option<Dynamic>
```

into:

```phalcom
flatMap<U>(_ f: (value: T) -> Option<U>) -> Option<U>
```

Keep the body unchanged.

### Step 7.6 — Replace `filter`

Change:

```phalcom
filter(_ pred) -> Self | Option<Dynamic>
```

into:

```phalcom
filter(_ pred: (value: T) -> Bool) -> Option<T>
```

### Step 7.7 — Fix `unwrapOr`

Replace the entire declaration header:

```phalcom
unwrapOr<U>(_ default: U) -> U
```

with:

```phalcom
unwrapOr(_ default: T) -> T
```

Update the documentation immediately above it:

- remove `@typeparam U`;
- state that `default` must have the option's payload type `T`;
- state that the result type is `T`.

This is a soundness fix, not merely type precision.

### Step 7.8 — Fix `okOr`

Replace:

```phalcom
okOr<E>(_ err) -> Result<T, E>
```

with:

```phalcom
okOr<E>(_ err: E) -> Result<T, E>
```

### Step 7.9 — Format/check the Universe source through its actual parser tests

Run the parser/native-source tests used for Universe source conformance. At minimum:

```bash
cargo +stable test -p phalcom-ast
cargo +stable test -p phalcom-semantic native_conformance
cargo +stable test -p phalcom-semantic generic_adts
```

If native conformance reports a descriptor mismatch, update the native descriptor to the same generic signature. Do **not** revert the source to `Dynamic` merely to match old metadata.

### Step 7.10 — Commit

```bash
git add phalcom-core/core/universe/src/option/option.ph
git add phalcom-native-meta phalcom-semantic  # only if conformance metadata actually required matching edits
git commit -m "fix(option): make generic API type-safe"
```

---

## Task 8: Add negative soundness tests and require Universe declarations to self-check

**Review coverage:** C-03

**Files:**
- Create: `phalcom-semantic/tests/semantic/integration/option_typing.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/mod.rs`
- Modify: `phalcom-semantic/tests/semantic/integration/native_conformance.rs` or add a dedicated Universe-source conformance test there

### Steps

- [ ] Add `mod option_typing;` to integration `mod.rs`.

- [ ] Create tests that analyze user source using the real canonical Option declaration.

**Positive map test:**

```phalcom
const x: Option<Int> = Option::Some(1)
const y: Option<String> = x.map(|value| value.toString)
```

Assert no type error and that the final expression/declaration type is `Option<String>`.

**Positive flatMap test:**

```phalcom
const x: Option<Int> = Option::Some(1)
const y: Option<String> = x.flatMap(|value| Option::Some(value.toString))
```

**Negative unwrapOr test:**

```phalcom
const x: Option<Int> = Option::Some(1)
const y: String = x.unwrapOr("missing")
```

Expected: semantic type error at the call/assignment; it must never infer `String` while returning an `Int` from the Some branch.

**Negative okOr test:**

Use an explicit contextual result type with incompatible error value and assert the generic call is rejected rather than producing a falsely typed `Result`.

- [ ] Add a Universe-source signature conformance test that analyzes `universe.option.option` through the same semantic signature builder used for user code and asserts the declarations are free of semantic errors. Native bodies may remain implementation-special, but their source signatures must be valid.

- [ ] Run:

```bash
cargo +stable test -p phalcom-semantic option_typing
cargo +stable test -p phalcom-semantic native_conformance
```

- [ ] Commit.

---

# Phase E — Make Universe use the canonical module/package resolver

## Task 9: Generalize package-surface validation from `ResolvedProjectId` to `ProjectIdentity`

**Review coverage:** H-01

**Why this layer owns the fix:** `phalcom-modules::ModuleResolver` is the canonical import authority. LSP completion already queries exposure-aware module products; the compiler must not use weaker semantics for Universe.

**Files:**
- Modify: `phalcom-modules/src/resolver.rs`
- Test: `phalcom-modules/tests/integration.rs`
- Test: `phalcom-core/tests/core/modules/universe.rs`

### Step 9.1 — Add provider-neutral helpers

Inside `impl ModuleResolver`, add:

```rust
fn locate_project_module(
    &self,
    project: ProjectIdentity,
    path: &ModulePath,
) -> Result<SourceUnit, ModuleResolutionError> {
    match project {
        ProjectIdentity::Universe => {
            let provider = UniverseSourceProvider::new();
            let id = ModuleId::universe(path.clone());
            let kind = provider
                .kind(path)
                .ok_or_else(|| ModuleResolutionError::ModuleNotFound(format!(
                    "Universe module universe.{path} not found"
                )))?;
            let source_id = provider.source_id(&id).map_err(|error| match error {
                ModuleLoadError::Resolution(error) => error,
                other => ModuleResolutionError::ModuleNotFound(other.to_string()),
            })?;
            let uri_path = path
                .components()
                .iter()
                .map(|component| component.as_str())
                .collect::<Vec<_>>()
                .join("/");
            Ok(SourceUnit {
                id,
                kind,
                source: SourceLocation {
                    source_id,
                    display_path: PathBuf::from(format!("<universe>/{uri_path}")),
                },
            })
        }
        ProjectIdentity::Resolved(project_id) => {
            let project = self.universe.get_project(project_id).ok_or_else(|| {
                ModuleResolutionError::ModuleNotFound(format!(
                    "Target project {project_id:?} not found"
                ))
            })?;
            self.source.locate(project, path)
        }
        ProjectIdentity::Synthetic(_) => Err(ModuleResolutionError::ModuleNotFound(
            "synthetic execution contexts do not have a package source provider".into(),
        )),
    }
}
```

If `SourceProvider::locate` returns `ModuleLoadError` rather than `ModuleResolutionError` at execution time, keep the existing `?` conversion used elsewhere in this file; do not introduce stringly conversion unnecessarily.

Add:

```rust
fn package_surface_for(
    &mut self,
    project: ProjectIdentity,
    package_path: &ModulePath,
) -> Result<PackagePathSurface, ModuleResolutionError> {
    let module_id = ModuleId {
        project,
        path: package_path.clone(),
    };

    let interface = self
        .load_interface(&module_id)
        .map_err(|error| ModuleResolutionError::PackageSurface(Box::new(error)))?;

    if !interface.kind.is_package_like() {
        return Err(ModuleResolutionError::PackageNotFoundError(module_id.to_string()));
    }

    Ok(PackagePathSurface {
        exposed_children: interface.exposed_children.clone(),
    })
}
```

### Step 9.2 — Generalize external path validation

Replace:

```rust
pub fn validate_external_path(
    &mut self,
    target_project_id: ResolvedProjectId,
    path: &ModulePath,
) -> ...
```

with:

```rust
pub fn validate_external_path(
    &mut self,
    target_project: ProjectIdentity,
    path: &ModulePath,
) -> Result<(), ModuleResolutionError> {
    let mut trace = BTreeSet::new();
    self.validate_external_path_with_trace(target_project, path, &mut trace)
}
```

Change `validate_external_path_with_trace` the same way.

Inside its loop, construct the package ID as:

```rust
let pkg_mod_id = ModuleId {
    project: target_project,
    path: current_pkg_path.clone(),
};
```

and load:

```rust
let surface = self.package_surface_for(target_project, &current_pkg_path)?;
```

For the diagnostic `project` label:

```rust
let project_name = match target_project {
    ProjectIdentity::Universe => "universe".to_string(),
    ProjectIdentity::Resolved(id) => self
        .universe
        .get_project(id)
        .map(|project| project.name.clone())
        .unwrap_or_else(|| id.to_string()),
    ProjectIdentity::Synthetic(id) => id.to_string(),
};
```

Use this in `ModulePathNotExposed`.

Delete the old `load_package_surface(ResolvedProjectId, ...)` method once all call sites use `package_surface_for(ProjectIdentity, ...)`.

### Step 9.3 — Rewrite the absolute Universe branch to go through validation

In `resolve_import_with_trace`, remove the early `return Ok(...)` from `ImportRootTarget::Universe`.

Replace the `target_project_id` block with:

```rust
let target_project = match target_root {
    ImportRootTarget::Universe => ProjectIdentity::Universe,
    ImportRootTarget::Resolved(id) => ProjectIdentity::Resolved(id),
};

if !is_self {
    self.validate_external_path_with_trace(
        target_project,
        &target_path,
        &mut package_interfaces,
    )?;
}

let target = self.locate_project_module(target_project, &target_path)?;
Ok(ImportResolutionTrace {
    target,
    package_interfaces,
})
```

This is the key correction: Universe existence and Universe visibility are separate questions.

### Step 9.4 — Tests

Add a module-system test using a small custom/bundled interface fixture where a known Universe child is present in the provider but not exposed through its parent package. Assert:

```text
resolver.resolve_import(...) -> ModulePathNotExposed
ModuleQueryFacade::external_import_children(...) -> child absent
```

The two authorities must agree.

Run:

```bash
cargo +stable test -p phalcom-modules
cargo +stable test -p phalcom-core universe
```

Commit:

```bash
git add phalcom-modules/src/resolver.rs phalcom-modules/tests phalcom-core/tests/core/modules/universe.rs
git commit -m "fix(modules): enforce Universe package exposure"
```

---

## Task 10: Support relative imports inside Universe through the same resolver

**Review coverage:** H-01

**Files:**
- Modify: `phalcom-modules/src/resolver.rs`
- Test: `phalcom-modules/tests/integration.rs`
- Modify later consumers only if compile errors expose duplicated special paths: `phalcom-core/src/modules/compile.rs`, `phalcom-modules/src/session.rs`

### Steps

- [ ] **Step 1: Stop deriving relative-import capability solely from `ResolvedProject`.**

At the top of `resolve_import_with_trace`, keep `importer_project` only for resolved-project import-root lookup. Do not use it to decide whether relative imports are legal.

- [ ] **Step 2: Replace the relative branch's current `importer_project.ok_or_else(...)` block.**

Use the canonical `ModuleId` and source interface instead:

```rust
let importer_unit = self.locate_project_module(importer.project, &importer.path)?;
let package_path = match importer_unit.kind {
    ModuleKind::Package => importer.path.clone(),
    ModuleKind::Module => importer.path.parent().unwrap_or_else(ModulePath::root),
};
```

Keep the existing dot arithmetic unchanged.

At the end replace:

```rust
let target = self.source.locate(importer_project, &target_path)?;
```

with:

```rust
let target = self.locate_project_module(importer.project, &target_path)?;
```

Synthetic projects will receive the explicit unsupported-provider error from the helper unless a higher-level standalone resolver handles them.

- [ ] **Step 3: Add direct tests.**

Construct `importer = ModuleId::universe(<known nested module>)` and parse a relative import actually used by Universe source. Assert the result equals the canonical target `ModuleId::universe(...)`.

Also test `..` beyond root produces `RelativeImportBeyondRoot` for Universe exactly as for resolved projects.

- [ ] **Step 4: Run:**

```bash
cargo +stable test -p phalcom-modules relative
cargo +stable test -p phalcom-modules universe
cargo +stable check -p phalcom-core
```

- [ ] **Step 5: Inspect duplicated resolver consumers.**

`phalcom-core/src/modules/compile.rs` and `phalcom-modules/src/session.rs` should continue calling `resolver.resolve_import(...)`; do not add a new Universe-specific branch there. If one becomes unnecessary because the canonical resolver now works, delete that special branch rather than retaining two algorithms.

- [ ] **Step 6: Commit.**

---

# Phase F — Remove string-derived ADT semantics from match compilation

## Task 11: Delete builtin-name guessing from fallback match lowering

**Review coverage:** H-03, part of M-04

**Files:**
- Modify: `phalcom-core/src/compiler/lib/match_expr.rs`
- Modify: `phalcom-core/src/compiler/lib/error.rs`
- Test: existing compiler/match integration tests; create `phalcom-core/tests/match_identity.rs` if no suitable file exists

**Current unsafe code:** `synthesize_fallback_pattern` recognizes `Some`, `None`, `Ok`, `Error`, `Err`, `Less`, `Equal`, `Greater`, `Unordered` by spelling and manufactures canonical Universe owners.

### Design decision for this remediation

The fallback compiler is allowed to compile structural patterns that need no semantic declaration identity. It is **not** allowed to compile variant patterns without semantic lowering.

This is deliberately stricter than guessing. Variant patterns need a proven `VariantId`, so standalone/unlinked compilation must obtain a semantic snapshot first or return a structured compiler error.

### Step 11.1 — Add a dedicated error

Open `phalcom-core/src/compiler/lib/error.rs` and add an error variant following the file's existing `thiserror` style:

```rust
#[error("variant match pattern at {0:?} requires semantic lowering")]
VariantPatternRequiresSemanticLowering(SourceRange),
```

Import `SourceRange` if needed.

### Step 11.2 — Remove builtin imports that existed only for guessing

In `match_expr.rs`, after the change you should no longer need:

```rust
use phalcom_common::selector::Selector;
use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::identity::{VariantFieldId, VariantId};
```

Retain only imports still used by structural fallback patterns.

### Step 11.3 — Simplify the fallback helper signature

Change:

```rust
fn synthesize_fallback_pattern(
    pat: &Pattern,
    module_id: &ModuleId,
    binding_counter: &mut u32,
    bindings: &mut Vec<ExecutableBindingSpec>,
) -> Result<ExecutablePattern, CompilerError>
```

into:

```rust
fn synthesize_fallback_pattern(
    pat: &Pattern,
    binding_counter: &mut u32,
    bindings: &mut Vec<ExecutableBindingSpec>,
) -> Result<ExecutablePattern, CompilerError>
```

Update every recursive call to remove `module_id`.

In `compile_match_expr`, remove:

```rust
let module_id = self.vm.heap.module(self.module).id.clone();
```

and call the helper with only pattern/counter/bindings.

### Step 11.4 — Replace the entire `Pattern::Variant(v)` branch

Delete all owner-name guessing, selector construction, `VariantId`, and field projection synthesis.

Replace the branch with:

```rust
Pattern::Variant(v) => Err(
    CompilerError::VariantPatternRequiresSemanticLowering(v.range)
),
```

This is the essential correctness change.

### Step 11.5 — Add regression tests

Add a compiler-level test where a local enum shadows familiar builtin spelling:

```phalcom
enum Result {
    @variant Ok(_ value: Int)
    @variant Error(_ message: String)
}
```

Compile through the normal analyzed/semantic-lowering pipeline and assert the match uses the local `VariantId`, not `universe.errors.result::Result`.

Add equivalent tests for local:

```text
Option::Some
Ordering::Equal
```

Add a fallback-only test that intentionally bypasses semantic lowering and contains a variant pattern. Assert the compiler returns `VariantPatternRequiresSemanticLowering`, not a guessed builtin pattern.

### Step 11.6 — Remove `Err` semantic aliasing

Search production Rust code for logic where the string `"Err"` is treated as a `Result` variant. Remove it unless there is a separately ratified source-level alias feature. Method names such as `mapErr`, `unwrapErr`, `isErr` are API names and are not affected by this cleanup.

Run:

```bash
cargo +stable test -p phalcom-core match
cargo +stable test -p phalcom-semantic matching
```

Commit:

```bash
git add phalcom-core/src/compiler/lib/match_expr.rs phalcom-core/src/compiler/lib/error.rs phalcom-core/tests
git commit -m "fix(match): require semantic identity for variant lowering"
```

---

# Phase G — Make metadata identity genuinely stable

## Task 12: Add stable project lookup to `ProjectUniverse`

**Review coverage:** H-05

**Why this layer owns the fix:** `ResolvedProjectId` is explicitly documented as an opaque graph-node identity. `ResolvedProject` already stores `ProjectSourceIdentity`, which is the correct starting point for durable identity. Conversion to stable metadata must therefore be context-aware: it needs the resolved `ProjectUniverse`, not just the numeric ID.

**Files:**
- Modify: `phalcom-modules/src/project.rs`
- Modify: `phalcom-modules/src/identity.rs` only if helper conversion belongs there
- Test: `phalcom-modules/tests/integration.rs`

### Step 12.1 — Add a stable-key query

In `impl ProjectUniverse`, immediately after `get_project`, add:

```rust
pub fn stable_project_key(
    &self,
    id: ResolvedProjectId,
) -> Option<crate::identity::StableProjectKey> {
    let project = self.get_project(id)?;
    Some(crate::identity::StableProjectKey::from_source(
        project.source_identity.clone(),
    ))
}
```

Add another query returning source identity directly if useful to callers:

```rust
pub fn project_source_identity(
    &self,
    id: ResolvedProjectId,
) -> Option<&ProjectSourceIdentity> {
    self.get_project(id).map(|project| &project.source_identity)
}
```

### Step 12.2 — Add revision fingerprint derivation

The current `ProjectRevisionFingerprint` type exists but is not populated by stable metadata conversion. Add a deterministic helper in `phalcom-modules` that hashes the project identity inputs available at resolution time.

For this remediation, use a stable artifact fingerprint based on:

```text
canonical project source identity path
validated manifest semantic content, when present
```

Do not use `DefaultHasher`, whose algorithm is not a persistence contract. Use the same deterministic 128-bit fingerprint primitive already used by `phalcom-type-meta::Fingerprint128`, or expose a small conversion helper if crate dependency direction forbids importing type-meta into modules.

If `phalcom-modules` must remain independent of `phalcom-type-meta`, define `ProjectRevisionFingerprint([u8;16])` there and compute it with an already-workspace-supported stable hash dependency; convert the bytes in semantic metadata code.

Add:

```rust
pub fn project_revision_fingerprint(
    &self,
    id: ResolvedProjectId,
) -> Option<ProjectRevisionFingerprint>
```

The exact hash input must be documented and tested. It must not include `ResolvedProjectId`.

### Step 12.3 — Order-independence test

Build equivalent project graphs in two different discovery/dependency traversal orders and assert:

```text
stable_project_key(project A in graph 1)
== stable_project_key(project A in graph 2)
```

and same for the revision fingerprint.

Run:

```bash
cargo +stable test -p phalcom-modules stable_project
```

Commit before changing semantic metadata callers.

---

## Task 13: Make stable metadata conversion require project context

**Review coverage:** H-05

**Files:**
- Modify: `phalcom-semantic/src/metadata/stable_identity.rs`
- Modify all call sites discovered by `rg "to_stable_(project|module|declaration|callable|field)" phalcom-semantic`
- Test: `phalcom-semantic/tests/semantic/integration/metadata.rs`

### Step 13.1 — Introduce conversion context

Replace free conversion functions that cannot resolve `ResolvedProjectId` with a context object:

```rust
pub struct StableIdentityContext<'a> {
    projects: &'a phalcom_modules::ProjectUniverse,
}

impl<'a> StableIdentityContext<'a> {
    pub fn new(projects: &'a phalcom_modules::ProjectUniverse) -> Self {
        Self { projects }
    }

    pub fn project(
        &self,
        project: &ProjectIdentity,
    ) -> Option<StableProjectRef> {
        match project {
            ProjectIdentity::Universe => Some(StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            }),
            ProjectIdentity::Resolved(id) => {
                let resolved = self.projects.get_project(*id)?;
                let revision = self.projects.project_revision_fingerprint(*id)?;
                Some(StableProjectRef::SourceArtifact {
                    logical_uri: resolved
                        .source_identity
                        .0
                        .to_string_lossy()
                        .into_owned()
                        .into_boxed_str(),
                    source_fingerprint: Fingerprint128::from_bytes(*revision.as_bytes()),
                })
            }
            ProjectIdentity::Synthetic(id) => Some(StableProjectRef::Session {
                session_fingerprint: Fingerprint128::from_u128(id.raw() as u128),
            }),
        }
    }

    pub fn module(&self, module: &ModuleId) -> Option<StableModuleRef> { ... }
    pub fn declaration(&self, declaration: &DeclarationId) -> Option<StableDeclarationRef> { ... }
    pub fn callable(&self, callable: &CallableId) -> Option<StableCallableRef> { ... }
    pub fn field(&self, field: &FieldId) -> Option<StableFieldRef> { ... }
}
```

Implement `module`, `declaration`, `callable`, and `field` by moving the existing code into methods and propagating `Option` with `?` when a resolved project cannot be found.

### Step 13.2 — Delete the unsafe conversion

Remove this behavior entirely:

```rust
logical_uri: res_id.to_string()
source_fingerprint: Fingerprint128::ZERO
```

There must be no durable metadata path that serializes `proj#N` as source identity.

### Step 13.3 — Update metadata export call sites

Where semantic export already has `snapshot.linked.universe` or another `Arc<ProjectUniverse>`, construct once:

```rust
let stable_ids = StableIdentityContext::new(&snapshot.linked.universe);
```

Pass/reference that context through exporter helpers rather than rebuilding it for every field.

Do not change semantic `DeclarationId`; this task changes only durable serialization identity.

### Step 13.4 — Tests

In `metadata.rs`, add:

1. same physical project loaded with different graph-node IDs -> identical stable declaration refs;
2. two distinct physical projects with identical module/declaration names -> different stable refs;
3. Universe declarations retain `Builtin { namespace: "universe", ... }`;
4. synthetic sessions remain explicitly session-local.

Run:

```bash
cargo +stable test -p phalcom-semantic metadata
cargo +stable check -p phalcom-semantic
```

Commit.

---

# Phase H — Make Universe source interfaces obey ordinary visibility semantics

## Task 14: Stop exporting every declaration from non-root Universe modules

**Review coverage:** M-03

**Files:**
- Modify: `phalcom-modules/src/builtin_interface.rs`
- Test: `phalcom-modules/tests/integration.rs` or create `phalcom-modules/tests/builtin_interface.rs`

### Step 14.1 — Preserve source authority

Open `BuiltinInterfaceBuilder::build_from_parsed`.

Keep:

```rust
let mut iface = InterfaceBuilder::build(...)?;
```

That result is the source-owned interface.

Keep the root-only native overlay for policy bindings that genuinely do not have root source declarations.

Delete the entire non-root `else` branch that currently loops over `iface.declarations` and inserts an export for every declaration.

The final shape should be:

```rust
if parsed.id.project == ProjectIdentity::Universe && parsed.id.path.is_root() {
    for binding in phalcom_native_meta::UNIVERSE_BINDINGS
        .iter()
        .filter(|binding| binding.exported)
    {
        // existing root overlay logic
    }
}
```

Do not add a replacement loop for non-root modules.

### Step 14.2 — If Universe source requires public-by-default behavior, fix `InterfaceBuilder`, not the builtin wrapper

After removing the branch, run module tests. If intended public declarations disappear, inspect the language's ordinary `InterfaceBuilder` visibility rules. The invariant is:

> The same source syntax has the same export semantics whether the owning project is Universe or user code.

If ordinary public-by-default declaration semantics are missing, implement that once in `InterfaceBuilder` with tests for both user and Universe modules. Do not restore a Universe-only override.

### Step 14.3 — Add a private/non-exported fixture test

Use a parsed Universe module fixture containing at least one declaration that ordinary `InterfaceBuilder` does not export. Assert `BuiltinInterfaceBuilder::build_from_parsed` also does not export it.

Run:

```bash
cargo +stable test -p phalcom-modules builtin_interface
cargo +stable test -p phalcom-modules
```

Commit.

---

# Phase I — Unify lexical package intrinsics

## Task 15: Make Universe `__package__` match ordinary module/package semantics

**Review coverage:** M-02

**Files:**
- Modify: `phalcom-core/src/modules/builtin_materialize.rs`
- Test: `phalcom-core/tests/core/modules/universe.rs`
- Inspect: `phalcom-core/src/modules/materialize.rs` Phase 2 as the canonical ordinary behavior

### Target semantics

```text
package module:
    ModuleObject.package = enclosing package if one exists, otherwise self at root
    __package__ = Some(self)

ordinary module:
    ModuleObject.package = nearest containing package
    __package__ = Some(nearest containing package), or None if none exists
```

The key distinction is that `ModuleObject.package` is an ownership relation; the language-visible `__package__` value for a package is the package object itself.

### Step 15.1 — Simplify parent assignment

In `initialize_canonical_universe`, this code:

```rust
vm.heap.module_mut(object).package = Some(if node.kind == ModuleKind::Package { parent } else { parent });
```

is redundant. Replace it with:

```rust
vm.heap.module_mut(object).package = Some(parent);
```

Do not try to fix `__package__` by changing the ownership field to self for all nested packages; ordinary materialization keeps ownership and lexical value as separate concepts.

### Step 15.2 — Compute lexical `__package__` by kind

In `install_universe_native_bindings`, replace:

```rust
let pkg_val = vm
    .heap
    .module(module)
    .package
    .map(Value::obj)
    .map(|v| v.wrap_some())
    .transpose()?
    .unwrap_or(Value::none());
```

with:

```rust
let pkg_target = if vm.heap.module(module).kind == ModuleKind::Package {
    Some(module)
} else {
    vm.heap.module(module).package
};

let pkg_val = pkg_target
    .map(Value::obj)
    .map(|value| value.wrap_some())
    .transpose()?
    .unwrap_or(Value::none());
```

Use the actual accessor/field name for `ModuleKind` exposed by `ModuleObject` if it differs; the semantic rule above is the important part.

### Step 15.3 — Add tests for all three Universe cases

In `phalcom-core/tests/core/modules/universe.rs`, assert:

1. Universe root package: `__package__ == Some(root)`;
2. nested Universe package: `__package__ == Some(that nested package)`;
3. ordinary Universe module under a package: `__package__ == Some(parent package)`.

Compare object identity, not display names.

Also keep/extend ordinary user-package tests so both materializers enforce the same semantics.

Run:

```bash
cargo +stable test -p phalcom-core universe
cargo +stable test -p phalcom-core modules
```

Commit.

---

# Phase J — Canonicalize Result variant terminology

## Task 16: Remove `Err` as a variant identity from fixtures and compiler semantics

**Review coverage:** M-04

Task 11 removes production fallback guessing. This task cleans semantic fixtures so future contributors do not reintroduce `Err` under the assumption it is canonical.

**Files:**
- Modify: `phalcom-semantic/tests/semantic/adts/matching/conformance.rs`
- Search/modify other test fixtures where a type intentionally named `Result` uses `@variant Err`
- Do not rename public methods `isErr`, `mapErr`, `inspectErr`, `unwrapErr`, `expectErr` merely because their method spelling contains Err

### Steps

- [ ] Run:

```bash
rg -n '\bErr\b|@variant\s+Err' phalcom-semantic phalcom-core phalcom-modules --glob '!docs/**'
```

Classify every hit:

```text
variant spelling / compiler semantic alias -> change to Error
method API name such as mapErr           -> keep
local test enum intentionally unrelated   -> rename only if it claims to model canonical Result
```

- [ ] In canonical Result-model fixtures change:

```phalcom
@variant Err(_ error: E)
```

into:

```phalcom
@variant Error(_ error: E)
```

and change match arms accordingly.

- [ ] Add one parser/semantic regression asserting `Result::Err` is unresolved unless the user has explicitly declared such a member in their own type.

- [ ] Run:

```bash
cargo +stable test -p phalcom-semantic matching
cargo +stable test -p phalcom-semantic adt
```

- [ ] Commit.

---

# Phase K — Separate complete Universe discovery from runtime initialization reachability

## Task 17: Execute only runtime-reachable Universe modules

**Review coverage:** H-02

**Files:**
- Modify: `phalcom-core/src/native/source.rs`
- Modify: `phalcom-core/src/vm/bootstrap.rs`
- Test: native source-index/bootstrap tests; create a focused test module if needed

**Current problem:** `NativeSourceIndex::initialization_order()` topologically sorts all `self.units`; `run_universe_modules()` executes all of them. The full catalog is useful for discovery/tooling, but should not imply eager execution.

### Design

Keep:

```rust
pub units: Vec<Arc<ParsedModuleUnit>>
```

as the complete source catalog.

Add a separate API:

```rust
pub fn initialization_order_from_roots(
    &self,
    roots: &[ModuleId],
) -> Result<Vec<Arc<ParsedModuleUnit>>, String>
```

It computes the transitive dependency closure of `roots`, then topologically sorts only that closure.

### Step 17.1 — Extract dependency edges once

Inside `NativeSourceIndex`, add:

```rust
fn dependency_indices(
    &self,
) -> Result<Vec<std::collections::BTreeSet<usize>>, String> {
    let by_id = self
        .units
        .iter()
        .enumerate()
        .map(|(index, unit)| (unit.id.clone(), index))
        .collect::<HashMap<_, _>>();

    let mut dependencies = vec![std::collections::BTreeSet::new(); self.units.len()];

    for (importer_index, unit) in self.units.iter().enumerate() {
        for dependency in &unit.program.preamble.dependencies {
            let path = match dependency {
                DependencyDecl::Import(ImportDecl::Module(decl)) => Some(&decl.path),
                DependencyDecl::Import(ImportDecl::Selective(decl)) => Some(&decl.path),
                DependencyDecl::ReExport(decl) => Some(&decl.path),
                DependencyDecl::Expose(_) => None,
            };
            let Some(path) = path else { continue };
            let Some(target) = universe_dependency_target(unit, path) else { continue };
            let Some(&target_index) = by_id.get(&target) else {
                return Err(format!(
                    "Universe dependency {target} referenced by {} is not materialized",
                    unit.id
                ));
            };
            dependencies[importer_index].insert(target_index);
        }
    }

    Ok(dependencies)
}
```

### Step 17.2 — Add dependency closure

```rust
fn reachable_from_roots(
    &self,
    roots: &[ModuleId],
    dependencies: &[std::collections::BTreeSet<usize>],
) -> Result<std::collections::BTreeSet<usize>, String> {
    let by_id = self
        .units
        .iter()
        .enumerate()
        .map(|(index, unit)| (unit.id.clone(), index))
        .collect::<HashMap<_, _>>();

    let mut reachable = std::collections::BTreeSet::new();
    let mut stack = Vec::new();

    for root in roots {
        let Some(&index) = by_id.get(root) else {
            return Err(format!("Universe initialization root {root} is not in source index"));
        };
        stack.push(index);
    }

    while let Some(index) = stack.pop() {
        if !reachable.insert(index) {
            continue;
        }
        stack.extend(dependencies[index].iter().copied());
    }

    Ok(reachable)
}
```

### Step 17.3 — Implement topological sort over the closure

Move the existing indegree/dependents algorithm into `initialization_order_from_roots`, but only create indegree edges when both importer and dependency are in `reachable`.

Do not use provider enumeration order as a tie breaker; keep deterministic `ModuleId` ordering using the existing `BTreeSet<(ModuleId, usize)>` ready queue.

Retain `initialization_order()` only if tests/tools use it, and redefine it as an explicit all-units helper:

```rust
pub fn initialization_order(&self) -> Result<Vec<Arc<ParsedModuleUnit>>, String> {
    let roots = self.units.iter().map(|unit| unit.id.clone()).collect::<Vec<_>>();
    self.initialization_order_from_roots(&roots)
}
```

Mark/document it as census/testing behavior, not VM bootstrap policy.

### Step 17.4 — Define bootstrap roots explicitly

Open `phalcom-core/src/vm/bootstrap.rs` and find `run_universe_modules`.

Replace its call to all-unit `initialization_order()` with an explicit root set containing only modules required for primordial/prelude/runtime startup. Derive this list from native bindings and true eager policy, not from every provider node.

Prefer a helper in native metadata such as:

```rust
fn canonical_universe_bootstrap_roots() -> Vec<ModuleId>
```

constructed from the source owners of primordial/native/prelude declarations that need source bodies at startup plus the root package if its initializer is semantically required.

Do **not** make `json`, `fs`, `net`, `testing`, etc. roots merely because they ship with Universe.

### Step 17.5 — Regression test

Create an index fixture with:

```text
root A -> dependency B
unrelated C
```

Call `initialization_order_from_roots([A])` and assert:

```text
contains B before A
contains A
not contains C
```

Also test a dependency cycle inside the reachable set still errors.

Run:

```bash
cargo +stable test -p phalcom-core native_source
cargo +stable test -p phalcom-core universe
```

Commit.

---

# Phase L — Final correctness gates before refactoring the semantic baseline

## Task 18: Run the complete correctness matrix and add zero-hit architecture searches

**Review coverage:** B-01 and all correctness fixes

No new architecture in this task.

### Commands

- [ ] Format:

```bash
cargo +stable fmt --all -- --check
```

- [ ] Build:

```bash
cargo +stable build --workspace --all-targets
```

- [ ] Tests:

```bash
cargo +stable test --workspace --all-targets
```

- [ ] Clippy:

```bash
cargo +stable clippy --workspace --all-targets -- -D warnings
```

- [ ] Miri AST lane, matching CI:

```bash
MIRIFLAGS=-Zmiri-disable-isolation cargo +nightly miri test -p phalcom-ast --lib
```

- [ ] LSP build:

```bash
cargo +stable build -p phalcom-lsp
```

- [ ] If Node dependencies are available, extension tests matching CI:

```bash
npm ci --prefix tools/vsphalcom
xvfb-run -a npm --prefix tools/vsphalcom test
```

### Architecture searches

The following searches must produce no production-code hits matching the forbidden semantic patterns:

```bash
rg -n 'UniverseKey::from_name\(root\)' phalcom-semantic/src
rg -n 'v\.base == "Err"|"Err"\s*=>.*Result' phalcom-core/src
rg -n 'res_id\.to_string\(\).*Stable|source_fingerprint:\s*Fingerprint128::ZERO' phalcom-semantic/src
```

Search the compiler for direct builtin variant spelling recognition:

```bash
rg -n '"Some"|"None"|"Ok"|"Error"|"Less"|"Equal"|"Greater"|"Unordered"' phalcom-core/src/compiler
```

Every remaining hit must be syntax/documentation/error text, not owner/variant identity construction.

Search the module resolver for Universe early-return bypasses:

```bash
rg -n 'ImportRootTarget::Universe' phalcom-modules/src/resolver.rs
```

Review each hit and verify it routes into common validation/locate helpers rather than returning before exposure validation.

Do not proceed to Task 19 until all gates are green.

---

# Phase M — Extract the immutable Universe semantic baseline after correctness is proven

## Task 19: Move inline Universe semantic bootstrap into `UniverseSemanticBaseline`

**Review coverage:** M-01

**Important:** This is deliberately last. It is a behavior-preserving extraction. If earlier tasks are not green, extracting them into a baseline makes bugs harder to locate.

**Files:**
- Create: `phalcom-semantic/src/universe_baseline.rs`
- Modify: `phalcom-semantic/src/lib.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Test: `phalcom-semantic/tests/semantic/incremental/*`
- Test: `phalcom-semantic/tests/semantic/integration/workspace.rs`

### Target structure

Create:

```rust
#[derive(Clone, Debug)]
pub struct UniverseSemanticBaseline {
    pub declarations: DeclarationTypeTable,
    pub hierarchy: MapTypeHierarchy,
    pub dispatch: SurfaceDispatchResolver,
    pub callable_signatures: CallableSignatureTable,
    pub enum_semantics: EnumSemanticTable,
    pub enum_products: Vec<Arc<EnumDeclarationProduct>>,
    pub associated_surfaces: AssociatedFamilyTable,
    pub associated_surface_products: Vec<Arc<AssociatedSurface>>,
    pub enum_requirements: EnumRequirementTable,
    pub enum_requirement_products: Vec<(DeclarationId, Arc<EnumRequirementsProduct>)>,
    pub prelude_types: Arc<PreludeTypeMap>,
}
```

Do not include mutable workspace sources, diagnostics, callable-body analyses, or user-project module products.

### Step 19.1 — Create a builder that owns the current bootstrap code

Add:

```rust
impl UniverseSemanticBaseline {
    pub fn build(store: &mut TypeStore) -> Self {
        // move, do not redesign, the validated bootstrap sequence from
        // SemanticWorkspaceSession::with_workspace here
    }
}
```

Move the following blocks from `with_workspace` in their existing order:

1. `bootstrap_universe_declarations`;
2. runtime-support Some/None declaration forms needed internally;
3. `UNIVERSE_CLASS_RELATIONS` hierarchy;
4. canonical prelude map construction;
5. dummy linked resolver for native signature import;
6. `register_native_surfaces`;
7. core Object test surfaces;
8. native callable signature population;
9. canonical `Class.new` signature;
10. real Universe source enum declaration-form discovery;
11. enum semantic construction;
12. enum behavior/default/case signatures;
13. associated surfaces;
14. enum requirements.

Do not change any algorithms during the move.

### Step 19.2 — Replace many session fields with one baseline

In `SemanticWorkspaceSession`, replace:

```text
base_declarations
base_hierarchy
base_dispatch
base_callable_signatures
base_enum_semantics
base_enum_products
base_associated_surfaces
base_associated_surface_products
base_enum_requirements
base_enum_requirement_products
prelude_types
```

with:

```rust
baseline: Arc<UniverseSemanticBaseline>,
```

During a first migration commit it is acceptable to add forwarding methods or local bindings such as:

```rust
let base_declarations = &self.baseline.declarations;
```

so the rest of the session logic changes mechanically rather than simultaneously being rewritten.

### Step 19.3 — Reuse the baseline across sessions

Because `TypeId`/interned structures belong to a `TypeStore`, do **not** put a global `OnceLock<UniverseSemanticBaseline>` around store-specific IDs unless the store itself is shared/frozen accordingly.

Choose one of these safe models based on the current TypeStore ownership:

**Preferred:** create a frozen Universe `TypeStore`/baseline pair and clone/share immutable arenas if the store supports stable shared IDs.

**Conservative first extraction:** keep `UniverseSemanticBaseline::build(&mut store)` per session, but isolate the code and prove semantic equivalence. Then add structural sharing in a separate performance change.

The first extraction closes the architecture issue without inventing invalid cross-store `TypeId` sharing.

### Step 19.4 — Snapshot equivalence test

Analyze the same workspace before/after extraction and assert stable equality/fingerprints for:

```text
canonical declaration index
hierarchy edges
Option/Result/Ordering enum semantics
callable signatures
prelude targets
source definition targets
```

Incremental tests must show user edits do not mutate/reconstruct baseline identities within a session.

### Step 19.5 — Run

```bash
cargo +stable test -p phalcom-semantic workspace
cargo +stable test -p phalcom-semantic incremental
cargo +stable test -p phalcom-semantic native_conformance
```

Commit:

```bash
git add phalcom-semantic/src/universe_baseline.rs phalcom-semantic/src/lib.rs phalcom-semantic/src/session.rs phalcom-semantic/tests
git commit -m "refactor(semantic): isolate canonical Universe baseline"
```

---

# 3. Final implementation order

Execute tasks in exactly this order unless a compile dependency forces a tiny adjacent adjustment:

```text
1  portable Cargo config
2  format/baseline build
3  canonical enum root reuse
4  cross-registry identity tests
5  explicit prelude map
6  editor/prelude reuse
7  Option type contracts
8  Option soundness + Universe self-check tests
9  Universe external exposure validation
10 Universe relative imports
11 semantic-only variant match lowering
12 stable project keys/revision identity
13 stable metadata conversion context
14 source-authoritative Universe exports
15 __package__ parity
16 Error/Err cleanup
17 reachable Universe initialization
18 full correctness verification
19 Universe semantic baseline extraction
```

Then execute the companion native-Result plan.

---

# 4. Commit strategy

Keep these changes independently reviewable. Recommended commit series:

```text
fix(build): make workspace Cargo config portable
style: apply workspace rustfmt
fix(adt): reuse canonical Universe enum roots
test(adt): enforce canonical runtime root identity
fix(semantic): enforce explicit Universe prelude policy
fix(editor): derive implicit symbols from semantic prelude map
fix(option): make generic API type-safe
test(option): enforce generic soundness
fix(modules): enforce Universe package exposure
fix(modules): resolve Universe relative imports canonically
fix(match): require semantic identity for variant lowering
feat(modules): expose durable project identity
fix(metadata): serialize stable project identities
fix(modules): preserve source-owned Universe exports
fix(runtime): align Universe package intrinsics
refactor(result): canonicalize Error variant fixtures
fix(universe): initialize only runtime-reachable modules
refactor(semantic): isolate canonical Universe baseline
```

Do not squash all of these into one change while debugging. The sequence itself is useful evidence about where a regression entered.

---

# 5. Reviewer checklist after implementation

A reviewer unfamiliar with the implementation should be able to validate the resulting architecture with the following questions.

## Runtime identity

- [ ] Does `universe.errors.result::Result` map to one `ClassId` everywhere?
- [ ] Does `universe.object.ordering::Ordering` map to one `ClassId` everywhere?
- [ ] Can a user declaration named `Result` receive its own unrelated class?
- [ ] Does physical ADT representation remain separate from nominal root ownership?

## Prelude/type resolution

- [ ] Is implicit type visibility derived from `binding.prelude`, not `UniverseKey::from_name`?
- [ ] Are runtime-support classes excluded from ordinary type lookup?
- [ ] Do local/imported declarations still shadow prelude names?
- [ ] Does editor completion consume the same prelude policy?

## Option

- [ ] Can `Option<Int>.unwrapOr("missing")` no longer type-check as `String`?
- [ ] Is `okOr<E>`'s error value actually typed `E`?
- [ ] Do `map`/`flatMap` preserve generic result types without `Dynamic`?
- [ ] Are canonical Universe source signatures semantically checked?

## Modules/packages

- [ ] Does `universe.*` import traversal honor `expose` at every package boundary?
- [ ] Do relative imports inside Universe use `ModuleResolver`?
- [ ] Can LSP completion and compiler resolution disagree about an exposed child? They should not.
- [ ] Does builtin interface generation preserve source visibility rules?

## Match/ADTs

- [ ] Does compiler production code contain no mapping from string `"Ok"` to Universe Result identity?
- [ ] Does fallback compilation reject unresolved variant patterns instead of guessing?
- [ ] Is `Error` the sole canonical Result variant identity?

## Metadata

- [ ] Is `ResolvedProjectId` absent from durable identity encoding?
- [ ] Are stable refs invariant under project graph discovery order?
- [ ] Does revision identity use a deterministic nonzero source fingerprint?

## Bootstrap

- [ ] Is the complete Universe still discoverable/materialized for tooling?
- [ ] Are only explicit runtime roots and their dependency closure executed?
- [ ] Does an unrelated shipped library remain uninitialized until needed?

## Verification

- [ ] `cargo +stable fmt --all -- --check`
- [ ] `cargo +stable build --workspace --all-targets`
- [ ] `cargo +stable test --workspace --all-targets`
- [ ] `cargo +stable clippy --workspace --all-targets -- -D warnings`
- [ ] Miri AST lane passes
- [ ] LSP builds and VS Code E2E lane passes where environment supports it

---

# 6. Explicit non-goals of this correctness plan

The following are intentionally not mixed into the remediation above:

1. rank-N polymorphism or other generic-system expansion;
2. changing the language's `Result::Error` naming decision;
3. redesigning `Self` beyond what is necessary for sound Option signatures;
4. changing user-facing package syntax;
5. implementing the compact native Result storage representation;
6. redesigning `Value` metadata allocation for DispatchKey or other future metadata;
7. broad performance refactors beyond separating initialization reachability and isolating the semantic baseline.

The compact Result feature is handled by the companion implementation plan because it changes physical `Value` representation, GC/spill behavior, and runtime case extraction, whereas the tasks above are semantic/runtime identity correctness fixes.

---

# 7. Definition of done

This remediation is complete only when all of the following are supported by fresh command output and regression tests:

```text
one canonical Universe declaration -> one semantic identity
one canonical native enum declaration -> one runtime root class
explicit prelude policy -> semantic + editor visibility
source interface -> compiler + LSP import/export visibility
semantic VariantId -> compiler match lowering
stable source/project identity -> durable metadata
complete Universe catalog != eager Universe execution set
```

A green build alone is not sufficient. The cross-layer identity tests and negative soundness tests are required because the defects reviewed here are mostly cases where code can compile while two subsystems silently disagree.
