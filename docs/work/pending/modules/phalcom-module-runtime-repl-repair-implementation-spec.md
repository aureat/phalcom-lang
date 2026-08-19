# Phalcom Module Runtime, Identity, Builtins, and REPL Convergence
## Implementation Specification — Semantic Repair Track

**Repository:** `aureat/phalcom-lang`
**Baseline commit:** `f0e51699060d31722c68b282a2d2e9a5b3260dfe` (`feat(modules): materialize universe v1 builtins`)
**Status:** implementation-ready target specification
**Companion:** `phalcom-modularity-reflection-package-interface-implementation-spec.md`

---

# 0. Purpose

This specification repairs the semantic and runtime foundations of Phalcom's module system before the new reflective/package-facing API is layered on top.

It is intentionally not a narrow patch for the observed REPL failures. The visible failures:

```text
ph> universe
// => <Project>

ph> universe.Object
MessageNotUnderstood

ph> import universe.callable
ph> callable
undefined variable

ph> import std
ph> std
undefined variable
```

are symptoms of several architectural divergences:

1. project ownership is encoded as a runtime module kind (`ProjectRoot`);
2. `Project` is currently a subtype of `Package`;
3. VM bootstrap creates a synthetic `universe` object distinct from the canonical builtin module graph;
4. builtin source and builtin interface declarations have two partially duplicated authorities;
5. the REPL bypasses `ProjectUniverse`, resolver, linker, and normal materialization;
6. import declarations are parsed into a preamble that the legacy REPL compilation path does not consume;
7. builtin source kinds are manually duplicated, and several `std` entries are classified as `Module` despite being backed by `package.ph`;
8. current dependency resolution reconstructs imported registry packages as development projects;
9. internal identity vocabulary (`ProjectIdentity`) now carries concepts that are no longer semantically Projects;
10. tests validate individual layers but insufficiently validate the full user-visible contract.

The target is one semantic module pipeline for CLI execution, project execution, standalone execution, builtins, and REPL cells.

The central rule is:

> **Project is a development context. Package is a runtime namespace/artifact. Module identity must not depend on whether a package is currently being developed, imported, builtin, or project-stripped.**

---

# 1. Read This First: Context-Minimizing Implementation Protocol

The implementation agent SHOULD NOT recursively read the repository before editing. Use the following targeted sequence.

## 1.1 Pin the baseline

Confirm the worktree still descends from:

```text
f0e51699060d31722c68b282a2d2e9a5b3260dfe
```

If not, inspect only the files in §1.2 and map moved symbols before proceeding. Do not re-derive the module architecture from scratch.

## 1.2 First-pass reads — only these files

Read these files once, in this order:

1. `phalcom-modules/src/source.rs`
   - `EntryOwnership`
   - `ModuleKind`
   - root `package.ph` classification
   - filesystem canonical-name logic
2. `phalcom-modules/src/project.rs`
   - `ResolvedProject`
   - project graph loading
   - import root construction
   - synthetic root construction
3. `phalcom-modules/src/identity.rs`
   - `BuiltinProject`
   - `ProjectIdentity`
   - `ModuleId`
   - `ModuleComponent`
4. `phalcom-modules/src/builtin.rs`
   - `UNIVERSE_NODES`
   - `STD_NODES`
   - `load_interface`
   - `source_text`
5. `phalcom-core/src/modules/compile.rs`
   - `EntrySelection`
   - project/package/module/inline branches
   - builtin discovery
   - standalone import behavior
6. `phalcom-core/src/modules/materialize.rs`
   - Phase 1 allocation
   - Phase 2 intrinsic/ownership injection
   - Phase 4 linked reads
   - Phase 5 export materialization
7. `phalcom-core/src/modules/registry.rs`
   - canonical module record/state lifecycle
8. `phalcom-core/src/heap/module.rs`
   - `ModuleObject`
   - `RuntimeExportRef`
   - current `owning_package`/`owning_project`
9. `phalcom-core/src/vm/bootstrap.rs`
   - `VM::new`
   - legacy `create_builtin_package("universe")`
   - core bootstrap compatibility
10. `phalcom-repl/src/repl.rs`
    - `ReplSession::start`
    - `eval`
    - `reload`
11. `phalcom-core/tests/modules_runtime.rs`
12. `phalcom-repl/tests/repl_phase_b.rs` and any existing REPL tests
13. `docs/spec/next/modules-next.md` only after the runtime changes are understood.

Do not read broad compiler/universe files yet. Use symbol-targeted reads only when a phase below names them.

## 1.3 Context discipline

After each implementation phase:

- run the phase-specific tests;
- summarize the changed invariant in a short scratch note;
- compact/remove stale implementation detail before opening the next subsystem;
- do not keep full `builtin.rs`, `compile.rs`, and `materialize.rs` simultaneously in active context unless the current edit crosses all three.

This work is specifically designed to avoid the failure mode where an agent consumes most of its context re-investigating architecture before writing code.

---

# 2. Baseline Source Map

All line ranges below are pinned to baseline commit `f0e516...`. Use the **symbol anchor** as authoritative if line numbers drift.

| File | Baseline range | Symbol/issue |
|---|---:|---|
| `phalcom-modules/src/source.rs` | ~L11-L35 | `EntryOwnership`, `ModuleKind::{Module,Package,ProjectRoot}` |
| `phalcom-modules/src/source.rs` | ~L94-L140 | root `package.ph` returns `ProjectRoot` when `persistent_project` |
| `phalcom-modules/src/source.rs` | ~L250-L300 | logical snake_case ↔ physical kebab-case mapping |
| `phalcom-modules/src/project.rs` | ~L10-L35 | `ResolvedProject`, `persistent_project` |
| `phalcom-modules/src/project.rs` | ~L135-L225 | project graph resolution and builtin/self/dependency import roots |
| `phalcom-modules/src/project.rs` | ~L225-L285 | `load_synthetic_root` |
| `phalcom-modules/src/identity.rs` | ~L8-L125 | builtin/resolved/synthetic project identities |
| `phalcom-modules/src/identity.rs` | ~L125-L205 | `ModuleComponent`, kebab/snake conversion |
| `phalcom-modules/src/identity.rs` | ~L230-L285 | `ModuleId`, temporary `ModuleId::core()` |
| `phalcom-modules/src/builtin.rs` | ~L14-L155 | `UNIVERSE_NODES` |
| `phalcom-modules/src/builtin.rs` | ~L155-L235 | `STD_NODES`; wrong `json/fs/path` `Module` classifications |
| `phalcom-modules/src/builtin.rs` | ~L250-L335 | manually constructed builtin interfaces |
| `phalcom-modules/src/builtin.rs` | ~L335-L500 | embedded source mapping; `std/json/package.ph`, etc. |
| `phalcom-core/src/modules/materialize.rs` | ~L14-L40 | Phase 1 module allocation |
| `phalcom-core/src/modules/materialize.rs` | ~L41-L205 | Phase 2 `__module__`/`__package__`/`__project__`; root package == project |
| `phalcom-core/src/modules/materialize.rs` | ~L250-L315 | Phase 5 export table |
| `phalcom-core/src/heap/module.rs` | ~L20-L95 | `RuntimeExportRef`, `ModuleObject`, ownership fields, `HashMap` exports |
| `phalcom-core/src/universe/core_classes.rs` | anchor `let package_class` | `Package : Module`, `Project : Package` |
| `phalcom-core/src/vm/bootstrap.rs` | ~L60-L95 | synthetic bootstrap `universe`, native class globals, core global |
| `phalcom-core/src/modules/compile.rs` | ~L14-L105 | entry selection and compile errors |
| `phalcom-core/src/modules/compile.rs` | ~L105-L245 | project/package/module/inline dispatch |
| `phalcom-core/src/modules/compile.rs` | anchor `compile_standalone_module` | standalone builtin import support |
| `phalcom-repl/src/repl.rs` | ~L52-L75 | `ReplSession`, legacy `start()` |
| `phalcom-repl/src/repl.rs` | ~L76-L130 | `eval()` compiles whole source via `compile_closure_as` |
| `phalcom-repl/src/repl.rs` | ~L132-L165 | `reload()` reconstructs the same legacy module |
| `docs/spec/next/modules-next.md` | L1-L50 | contradictory `Module < Package < Project` headline vs project-owns-package prose |

---

# 3. Normative Semantic Delta

The following invariants replace all contradictory older implementation comments and specifications.

## 3.1 Namespace hierarchy

```text
Object
└── Module
    └── Package
```

`Project` is not part of this hierarchy.

## 3.2 Development ownership

```text
Project
└── rootPackage : Package
    ├── Module
    └── Package
```

A project owns exactly one root Package.

## 3.3 Runtime package invariance

A Package has the same runtime namespace semantics whether it is:

- the root package currently under development;
- a nested package;
- loaded from a path dependency;
- loaded from a published artifact;
- builtin (`universe`, `std`);
- standalone where a valid package identity exists.

Project development state must not alter `Package.class`, import semantics, export semantics, package/member semantics, or canonical module identity.

## 3.4 `package.ph`

A `package.ph` source unit materializes as exactly one `Package` object.

Never:

```text
package.ph -> Module + wrapper Package
```

Never:

```text
root package.ph -> Project
```

Always:

```text
package.ph -> Package
```

## 3.5 Project lifecycle

```text
development:
    Project -> root Package

publication:
    Project stripped
    Package + PackageInfo retained

consumption:
    Package loaded
    no synthetic Project reconstructed
```

Runtime Project reflection is addressed in the companion specification. This file establishes the foundation that makes it possible.

---

# 4. Phase 0 — Add Semantic Constitution Tests Before Editing Runtime Semantics

This is the first implementation phase.

## 4.1 Why

The current suite tests:

- builtin provider interface claims;
- linker behavior;
- ordinary module export dispatch;
- REPL cell persistence;

but it did not test the composition:

```text
REPL
 -> resolver
 -> builtin provider
 -> linker
 -> materializer
 -> runtime export send
```

The repair must start from user-visible tests.

## 4.2 New tests

Create:

```text
phalcom-core/tests/modules_semantic_contract.rs
phalcom-repl/tests/modules_semantic_contract.rs
phalcom-modules/tests/package_semantic_contract.rs
```

Because `phalcom-core/Cargo.toml` currently has `autotests = false`, explicitly register the new core test target or add it to the existing `tests/integration.rs` harness according to repository convention.

### Core contract tests

At minimum:

```rust
#[test]
fn builtin_universe_root_is_package() { ... }

#[test]
fn universe_export_object_is_runtime_readable() { ... }

#[test]
fn std_root_is_package() { ... }

#[test]
fn std_json_is_package_because_it_is_backed_by_package_ph() { ... }

#[test]
fn project_root_package_is_package_not_project() { ... }

#[test]
fn package_ph_has_one_runtime_object_identity() { ... }

#[test]
fn builtin_module_identity_is_canonical_across_import_and_member_access() { ... }
```

### Language-level acceptance examples

The harness must exercise the equivalent of:

```phalcom
universe.class == Package
universe.Object == Object

import universe.callable
callable.class == Package

import std
std.class == Package

import std.json
json.class == Package
```

Whether `universe.callable` / `std.json` are also member sends is a separate façade decision in the companion spec; imports are mandatory here.

### REPL contract tests

Execute separate cells:

```text
cell 1: import std
cell 2: std

cell 3: import std.json
cell 4: json

cell 5: import universe.callable
cell 6: callable
```

Assert both values and persistent bindings.

## 4.3 Replace stale tests

The existing tests that explicitly require builtin or filesystem roots to be `ModuleKind::ProjectRoot` must be rewritten to require `ModuleKind::Package`.

Do not retain a compatibility assertion for `ProjectRoot`.

---

# 5. Phase 1 — Remove `ModuleKind::ProjectRoot`

## 5.1 Current problem

At `phalcom-modules/src/source.rs` around L20-L35:

```rust
pub enum ModuleKind {
    Module,
    Package,
    ProjectRoot,
}
```

and root source resolution around L94-L140 branches on `persistent_project`.

This encodes two unrelated dimensions in one enum:

```text
namespace kind: Module | Package
development ownership: project-backed | standalone | builtin | ...
```

## 5.2 Required replacement

Replace the enum with:

```rust
pub enum ModuleKind {
    Module,
    Package,
}
```

Replace:

```rust
is_package_like()
```

with either:

```rust
pub const fn is_package(self) -> bool {
    matches!(self, Self::Package)
}
```

or remove the helper and use `kind == ModuleKind::Package`.

Every `package.ph`, including a persistent project root, resolves as `Package`.

### `source.rs`

Replace:

```rust
kind: if project.persistent_project {
    ModuleKind::ProjectRoot
} else {
    ModuleKind::Package
},
```

with:

```rust
kind: ModuleKind::Package,
```

## 5.3 Move ownership to an orthogonal representation

`EntryOwnership` already demonstrates the right idea.

Preserve/extend ownership separately:

```rust
pub enum EntryOwnership {
    ProjectOwned { project: ResolvedProjectId },
    StandalonePackageOwned { package_root: PathBuf },
    StandaloneModule { file: PathBuf },
    Inline { synthetic: SyntheticExecutionId },
}
```

Do not put ownership back into `ModuleKind`.

## 5.4 Metadata consequence

`ModuleMetadata::from_ast` currently maps `ProjectRoot` to `MetadataTarget::Project`.

After this phase:

```text
package.ph metadata -> Package metadata
module.ph metadata  -> Module metadata
```

Project metadata must originate from project/manifest context, not from root-package source kind.

The companion spec defines the public Project metadata surface.

## 5.5 Search-and-replace checklist

Targeted command:

```bash
rg -n "ProjectRoot|is_package_like|persistent_project" \
  phalcom-modules phalcom-core phalcom-repl docs
```

Review each hit. Do not mechanical-replace `persistent_project`; it still has value as a property of resolution context until that field is refactored.

### Acceptance gate

```text
rg "ModuleKind::ProjectRoot" -> zero production hits
```

Tests/documentation may mention the removed spelling only in migration commentary.

---

# 6. Phase 2 — Repair the Kernel Class Hierarchy

## 6.1 Current state

In `phalcom-core/src/universe/core_classes.rs`, anchor:

```rust
let package_class = make_core_class(heap, "Package", module_class, metaclass_class);
// Project is the root package object, not a wrapper around one.
let project_class = make_core_class(heap, "Project", package_class, metaclass_class);
```

## 6.2 Target

```text
Object
├── Module
│   └── Package
└── Project
```

Change Project superclass to `object_class`:

```rust
let package_class = make_core_class(heap, "Package", module_class, metaclass_class);
let project_class = make_core_class(heap, "Project", object_class, metaclass_class);
```

Project remains native-representation if the runtime will construct native-backed Project values. The companion spec defines its fields/interface.

## 6.3 Class lookup mapping

Find every dispatch/classification branch using `ModuleKind`.

Targeted read:

```bash
rg -n "ModuleKind::|project_class|package_class|module_class" phalcom-core/src
```

Any logic equivalent to:

```rust
ProjectRoot => project_class
Package     => package_class
Module      => module_class
```

must become:

```rust
Package => package_class
Module  => module_class
```

Project objects are no longer created by module-kind classification.

## 6.4 No compatibility subclass

Do not temporarily keep `Project < Package` "until reflection lands." That would preserve the exact semantic ambiguity this migration exists to remove.

### Acceptance gate

```phalcom
project.rootPackage.class == Package
project.class == Project
project is Module   // false
project is Package  // false
```

The exact type-test syntax can use the repository's available mechanisms.

---

# 7. Phase 3 — Refactor Identity Vocabulary Away From “Everything Is a Project”

## 7.1 Current state

`phalcom-modules/src/identity.rs` models module ownership as:

```rust
pub enum ProjectIdentity {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}

pub struct ModuleId {
    pub project: ProjectIdentity,
    pub path: ModulePath,
}
```

This was reasonable under the old architecture but is semantically misleading after project stripping.

## 7.2 Recommended approach: staged rename

Do not combine a deep identity rewrite with all runtime repairs in one commit.

### Stage A — semantic alias / new vocabulary

Introduce:

```rust
pub enum ModuleOwnerId {
    Builtin(BuiltinPackage),
    ResolvedPackage(ResolvedPackageId),
    Synthetic(SyntheticExecutionId),
}

pub struct ModuleId {
    pub owner: ModuleOwnerId,
    pub path: ModulePath,
}
```

Where practical, type-alias the old names temporarily inside `phalcom-modules` while migrating call sites.

Suggested mappings:

```text
BuiltinProject        -> BuiltinPackage
ResolvedProjectId     -> ResolvedPackageId   (or ResolutionUnitId)
ProjectIdentity       -> ModuleOwnerId
SyntheticProjectId    -> SyntheticExecutionId
```

### Stage B — rename `ProjectUniverse`

A larger rename to `ResolutionUniverse` or `PackageUniverse` is semantically attractive but optional for this implementation if it causes excessive churn.

Recommended:

```text
keep ProjectUniverse for one migration
document it as resolution graph
do NOT expose its name in user-facing interfaces
```

Then perform the rename in a follow-up mechanical cleanup.

## 7.3 Alternative approach: preserve internal names

It is technically possible to retain `ProjectIdentity` as an implementation term meaning "resolution owner" while changing only public semantics.

This is lower churn but not recommended: the same old assumption will continue to leak into comments, APIs, and future code.

## 7.4 `ModuleId::core()`

Current `ModuleId::core()` is explicitly temporary compatibility under builtin Universe.

Target:

- `core` is internal bootstrap machinery;
- it is not a language import root;
- no public resolver policy should reserve `core` merely because bootstrap uses that spelling.

Remove `core` from manifest import-root collision policy once no public root exists.

The internal core module may retain a synthetic/private identity until the core/universe split is fully eliminated, but it must not be represented as `universe.core` if `universe.core` is not a real builtin node.

---

# 8. Phase 4 — Fix Builtin Graph Source-Kind Inconsistencies

## 8.1 Concrete bug

`phalcom-modules/src/builtin.rs` `STD_NODES` marks at least:

```text
std.json
std.fs
std.path
```

as `ModuleKind::Module`.

Yet `source_text()` maps them to:

```text
phalcom-core/core/std/src/json/package.ph
phalcom-core/core/std/src/fs/package.ph
phalcom-core/core/std/src/path/package.ph
```

Under the explicit package rule, these are Packages.

## 8.2 Immediate repair

All builtin node kinds must agree with their source-unit shape:

```text
.../package.ph -> Package
.../<name>.ph  -> Module
```

At minimum fix all `STD_NODES`, not only `json`.

Add a test that enumerates every builtin node and checks that the embedded source mapping and node kind agree.

## 8.3 Better structural repair

Manual `kind` duplication is fragile.

Recommended representation:

```rust
enum BuiltinSourceKind {
    Package,
    Module,
}

struct BuiltinNodeSpec {
    path: &'static [&'static str],
    source: BuiltinSourceSpec,
    children: &'static [&'static str],
}
```

or derive kind from the source map:

```rust
BuiltinSourceSpec::Package(include_str!(...))
BuiltinSourceSpec::Module(include_str!(...))
```

Then:

```rust
fn kind(&self) -> ModuleKind
```

is derived rather than separately declared.

This prevents "source says package, metadata says module" regressions.

---

# 9. Phase 5 — Establish One Authoritative Builtin Interface Pipeline

## 9.1 Current duplication

`BuiltinProjectSourceProvider::load_interface()` manually creates:

- declarations;
- exports;
- exposed children;
- empty metadata.

`source_text()` separately contains the actual source declarations:

```phalcom
@!documentation(...)
expose .child
...
```

As a result, builtin source metadata can be dropped even though the source contains it.

## 9.2 Target architecture

Use:

```text
embedded builtin source
        │
        ├── parse
        ├── InterfaceBuilder
        │
        ▼
source-derived interface
        │
        + native interface overlay
        │
        ▼
authoritative builtin interface
```

### Source-derived facts

These should come from source:

- `@!` metadata;
- `expose`;
- ordinary imports;
- ordinary declarations;
- ordinary exports.

### Native overlay facts

These may come from native metadata:

- primordial universe bindings that are implemented as native kernel classes;
- their export names;
- prelude membership;
- native provenance;
- primitive/native implementation metadata where applicable.

## 9.3 New abstraction

Create:

```text
phalcom-modules/src/builtin_interface.rs
```

Suggested API:

```rust
pub struct BuiltinInterfaceBuilder;

impl BuiltinInterfaceBuilder {
    pub fn build(
        provider: &BuiltinProjectSourceProvider,
        id: &ModuleId,
    ) -> Result<UnlinkedModuleInterface, ModuleLoadError>;
}
```

Algorithm:

1. get canonical source text;
2. parse;
3. `InterfaceBuilder::build(id, provider.kind(...), &program)`;
4. overlay native declarations/exports;
5. reject conflicts deterministically;
6. return one interface.

## 9.4 Overlay conflict rule

If source declares an ordinary binding with the same name as a toolchain-owned native overlay:

```text
error: builtin interface collision
```

Never silently prefer Rust metadata over source or vice versa.

## 9.5 Performance

Builtin source parsing should be cached per toolchain process/generation. Do not reparsed embedded source on every import.

---

# 10. Phase 6 — Canonicalize Builtin Runtime Materialization

## 10.1 Current problem

`VM::new()` currently:

1. creates bootstrap core;
2. creates a special builtin package called `universe`;
3. defines exported kernel classes directly into its global slots;
4. freezes it;
5. exposes that object from core.

Normal module compilation/materialization has a separate canonical builtin identity graph.

Therefore there are two conceptual universe objects.

## 10.2 Target invariant

For every builtin `ModuleId`:

```text
one ModuleId
 -> one ModuleRegistry record
 -> one ModuleObject
```

This includes:

```text
phalcom://universe/
phalcom://std/
all builtin descendants
```

## 10.3 Required materialization path

Recommended:

```text
VM kernel class bootstrap
    ↓
create module registry
    ↓
materialize canonical universe root object
    ↓
install native universe slots into that object
    ↓
materialize its RuntimeExportRef table
    ↓
register global implicit binding `universe`
    ↓
materialize std lazily/on demand through same builtin graph
```

Do not call `create_builtin_package("universe")` to allocate a second identity.

## 10.4 Native binding installation

This is load-bearing.

The builtin interface can correctly say:

```text
Object is an exported declaration
```

but source `package.ph` does not define the kernel class value.

Add an explicit native-binding installer, for example:

```text
phalcom-core/src/modules/builtin_materialize.rs
```

with:

```rust
pub fn install_universe_native_bindings(
    vm: &mut VM,
    universe_root: ObjRef,
) -> PhResult<()>;
```

For each `UNIVERSE_BINDINGS` entry:

1. locate/declare canonical slot;
2. assign the kernel class/native value;
3. leave export table creation to canonical interface materialization;
4. preserve prelude flags separately;
5. freeze namespace only after all expected bootstrap writes.

## 10.5 Export table

Do not equate globals with exports.

`universe.Object` must work because Phase 5 materialized an export reference:

```rust
RuntimeExportRef::Binding(...)
```

not because module send dispatch falls back to arbitrary globals.

## 10.6 `std`

Do not implicitly bind `std` globally.

Normative rule:

```text
universe = implicit builtin binding
std      = reserved builtin import root, explicit import required
```

`import std` must return the canonical std root Package.

---

# 11. Phase 7 — Prelude Canonicalization

## 11.1 Authority

`phalcom_native_meta::UNIVERSE_BINDINGS[*].prelude` remains the authority unless a later spec deliberately moves it.

## 11.2 Identity invariant

For every prelude class/binding:

```phalcom
Object === universe.Object
List === universe.List
```

where the right-hand side is actually exported by the root facade.

Prelude lookup must not create copies or parallel globals with different values.

## 11.3 Lookup semantics

Specify and test:

1. local declarations shadow prelude;
2. explicit imports can shadow prelude subject to ordinary binding rules;
3. prelude does not count as an import declaration;
4. LSP should attribute the resolved name to canonical `universe`;
5. not every universe export is prelude-visible.

---

# 12. Phase 8 — Make Project Ownership Runtime-Orthogonal

## 12.1 Remove `owning_project: Option<ObjRef>` as “root package object”

`ModuleObject` currently contains:

```rust
pub owning_package: Option<ObjRef>,
pub owning_project: Option<ObjRef>,
```

and materialization may set `owning_project` to the root package.

That becomes invalid.

## 12.2 Recommended runtime structure

Keep namespace relationships on ModuleObject:

```rust
pub package: Option<ObjRef>,
pub root_package: Option<ObjRef>,
```

For `Package`, `package` may be `self`.

Project development context should be separate:

```rust
pub development_project: Option<ProjectContextRef>,
```

or maintained in the program/module execution context rather than every Package object.

The companion spec determines the exact Project runtime object.

## 12.3 Relationship table

```text
ordinary module:
    package      = nearest Package
    rootPackage  = root Package

nested Package:
    package      = self
    parentPackage= containing Package
    rootPackage  = root Package

root Package:
    package      = self
    parentPackage= None
    rootPackage  = self

standalone Module:
    package      = None
    rootPackage  = None
```

Do not derive `rootPackage` from “project root object.”

---

# 13. Phase 9 — Context Intrinsics Must Stop Being User Globals

This phase establishes only runtime/compiler semantics; public documentation is in the companion spec.

## 13.1 Current implementation

`materialize.rs` currently defines:

```text
__module__
__package__
__project__
```

as global bindings in every materialized module.

This causes reserved-dunder and field/member interactions to become accidental.

## 13.2 Target intrinsics

```phalcom
__module__  -> Module
__package__ -> Option<Package>
__root__    -> Option<Package>
__project__ -> Project   // only in active development context
```

They are contextual compiler/runtime intrinsics, not mutable global variables.

## 13.3 Recommended implementation: dedicated bytecodes

Introduce bytecodes equivalent to:

```rust
GetCurrentModule
GetCurrentPackage
GetCurrentRootPackage
GetCurrentProject
```

Advantages:

- no global-slot collision;
- no user redeclaration path;
- semantics follow the active module frame;
- no materialization-time `Option` object allocation merely to seed hidden globals;
- REPL and file compilation can share semantics;
- unavailable `__project__` can be diagnosed by compile context.

Alternative: compiler-injected immutable hidden slots. This is acceptable only if they cannot be observed/mutated as ordinary globals and cannot collide with reflection dispatch. Dedicated bytecodes are preferred.

## 13.4 Compiler work

Targeted search only after runtime fields are ready:

```bash
rg -n "GetGlobal|compile_variable|resolve.*global|DunderPolicy|__module__" \
  phalcom-core/src/compiler phalcom-core/src
```

Recognize reads of the four intrinsic spellings before ordinary global resolution.

Declarations remain rejected through `DunderPolicy`.

---

# 14. Phase 10 — Introduce a Shared Semantic Execution Context

The current largest modularity problem is that the REPL manually creates a VM module while normal execution constructs a complete linked program.

Create one reusable execution-context layer.

## 14.1 New file

Recommended:

```text
phalcom-core/src/modules/context.rs
```

with an API conceptually similar to:

```rust
pub enum ExecutionOwnership {
    Project { root: ResolvedPackageId, project: DevelopmentProjectDescriptor },
    StandalonePackage { root: ResolvedPackageId },
    StandaloneModule { id: ModuleId },
    ReplStandalone { id: ModuleId },
}

pub struct ModuleExecutionContext {
    pub universe: Arc<ProjectUniverse>, // rename later if desired
    pub entry: ModuleId,
    pub ownership: ExecutionOwnership,
    pub package_context: Option<ModuleId>,
    pub root_package: Option<ModuleId>,
}
```

Do not expose these Rust names directly to user reflection.

## 14.2 Factory functions

Centralize:

```rust
from_project_path(...)
from_module_path(...)
from_package_path(...)
for_project_repl(...)
for_standalone_repl(...)
```

These factories should be used by both `ProgramCompiler` and the REPL.

## 14.3 Benefit

This removes duplicated decisions for:

- project discovery;
- builtin root availability;
- package context;
- root package identity;
- relative import legality;
- current Project availability;
- standalone behavior.

---

# 15. Phase 11 — REPL Semantic Convergence

This is a major feature track, not incidental cleanup.

## 15.1 Current failure

`phalcom-repl/src/repl.rs` `ReplSession::start` does:

```rust
let mut vm = VM::new();
let module = vm.create_module("main", cwd);
```

and `eval` does:

```rust
vm.compile_closure_as(module, source, UnitKind::Repl)
```

The parser puts imports into `program.preamble.dependencies`, but the normal closure compiler does not perform project resolution/linking for that preamble. An import-only cell can therefore succeed while binding nothing.

## 15.2 New REPL state

Refactor `ReplSession` to carry:

```rust
pub struct ReplSession {
    pub vm: VM,
    pub repl_module: ObjRef,
    pub semantic: ReplSemanticContext,
    pub next_cell: usize,
    pub history: Vec<String>,
    ...
}
```

New:

```text
phalcom-repl/src/semantic.rs
```

or, preferably, core-owned reusable context types with a thin REPL wrapper.

Conceptually:

```rust
pub struct ReplSemanticContext {
    pub execution: ModuleExecutionContext,
    pub repl_module_id: ModuleId,
    pub import_bindings: PersistentImportBindings,
    pub generation: u64,
}
```

## 15.3 Startup modes

### Project-backed REPL

`phalcom repl` under an owning project:

1. discover nearest `project.toml`;
2. load the same project resolution graph as normal execution;
3. identify root Package;
4. create active development Project context;
5. make `universe` and `std` roots resolvable;
6. create one stable synthetic REPL Module owned by the session but package-contextualized to the project root;
7. expose `__project__`.

### Standalone REPL

No owning project:

1. create one stable synthetic REPL Module;
2. no Package context;
3. no Project context;
4. builtin absolute roots `universe` and `std` remain resolvable;
5. relative imports fail with a clear context diagnostic.

**Standalone REPL must not mean “no module system.”**

## 15.4 Cell compilation pipeline

Replace top-level cell compilation with:

```text
source cell
  ↓
parse once
  ↓
classify echo behavior
  ↓
build/extend cell interface
  ↓
resolve preamble dependencies
  ↓
link new immutable import bindings
  ↓
compile body with persistent linked bindings
  ↓
execute in persistent REPL Module
  ↓
merge permitted session declarations/import state
```

The source should not be parsed once for echo classification and independently parsed again through an unaware compiler if the parsed `Program` can be passed forward.

## 15.5 Persistent imports

An import in cell N must remain resolvable in cell N+1.

Example:

```text
ph> import std.json
ph> json
// => <Package std.json>

ph> json.parse("{}")
...
```

Persist semantic imported binding descriptors, not just VM globals.

## 15.6 Cross-cell shadowing

Recommended interactive rule:

```text
within one cell:
    import binding immutable

later cell:
    a new declaration or import may shadow prior REPL session binding
```

This matches existing REPL shadowing behavior without making imports mutable.

Closures compiled before shadowing preserve their linked/captured behavior according to ordinary closure semantics.

## 15.7 Relative import context

Initial rule:

```text
project-backed REPL:
    current package context = project root Package

standalone REPL:
    current package context = None
```

Do not infer package context from arbitrary cwd subdirectories.

A future explicit `phalcom repl path/to/module.ph` may establish a narrower package context, but is not required here.

## 15.8 Stable REPL Module identity

The REPL module is not `main.ph`, not the root Package, and not a new identity per cell.

Give one identity per session.

Suggested URI model:

```text
phalcom://repl/<session-id>
```

or project-qualified equivalent:

```text
phalcom://repl/<session-id>?root=app
```

The exact URI spelling can be finalized in the companion spec. The invariant is stable identity across cells and distinct identity across independent sessions.

## 15.9 `:reload`

Current `reload()` reconstructs the legacy VM/module and replays history.

After refactor, reload must recreate the same semantic mode:

```text
project discovery
resolution graph
canonical builtins
active Project context
root Package
REPL module
```

then replay history.

Do not preserve old imported-binding objects across the fresh VM; rebuild them deterministically from history.

## 15.10 Error behavior

The following must fail, not silently produce `Unit`:

```phalcom
import nonexistent
import std.nonexistent
from universe import NotAnExport
import .foo  // standalone REPL
```

The error must come from resolver/linker context with source range.

---

# 16. Phase 12 — Unify Builtin Imports Across Standalone Program and REPL

`ProgramCompiler::compile_standalone_module` already contains special support for absolute builtin `universe` and `std` roots, while `EntrySelection::Inline` currently rejects all preamble dependencies with `ReplImportRequiresProjectContext`.

This is inconsistent terminology and behavior.

## 16.1 Decouple Inline from REPL

`EntrySelection::Inline` is not the REPL implementation after this migration.

Rename/reframe errors such as:

```text
ReplImportRequiresProjectContext
```

to an inline-context-specific diagnostic if still necessary.

## 16.2 Shared builtin root resolver

Extract the duplicated:

```text
root == universe -> Builtin
root == std      -> Builtin
```

logic to the shared resolution/execution context.

Both standalone module execution and standalone REPL should use it.

---

# 17. Phase 13 — Published Package Artifact Boundary

This phase does not implement a registry client. It fixes an architectural semantic gap.

## 17.1 Current behavior

`DependencyProvider::resolve_package` returns:

```rust
ResolvedDependencySource {
    manifest_path: PathBuf
}
```

Then `ProjectUniverse` recursively loads the dependency's `project.toml` and treats it as another development project.

This contradicts the decided lifecycle:

```text
published package = Project stripped
```

## 17.2 Introduce artifact abstraction

Create:

```text
phalcom-modules/src/artifact.rs
```

Suggested:

```rust
pub struct ResolvedPackageArtifact {
    pub identity: PackageArtifactIdentity,
    pub info: PackageInfoDescriptor,
    pub root_namespace: ModuleComponent,
    pub requirements: Vec<PackageRequirementDescriptor>,
    pub module_source: Arc<dyn PackageSourceProvider>,
}

pub trait PackageArtifactProvider {
    fn resolve(
        &self,
        package: &str,
        version_requirement: &str,
    ) -> Result<ResolvedPackageArtifact, ProjectError>;
}
```

Names can change; the semantic split may not.

## 17.3 Path dependencies

During development, a path dependency may point at another project checkout. Resolution can derive a package artifact view from that project's validated root package.

Do not expose a dependency's development `Project` to the consuming runtime merely because the source came from a checkout.

## 17.4 Registry dependencies

Future registry resolution returns package artifact metadata/source directly. It must not require reconstructing the publisher's source-development `Project`.

## 17.5 Transitional compatibility

If implementing the full provider abstraction would excessively expand this patch, introduce an adapter:

```text
ManifestBackedPackageArtifactProvider
```

around the current provider, and make the rest of the compiler consume the artifact abstraction. This creates the correct boundary without implementing remote resolution.

---

# 18. Phase 14 — Naming and Physical Layout Consistency

There is current tension between:

- specification language saying filesystem names are not silently translated;
- `ModuleComponent::to_kebab()` and filesystem provider enforcing logical snake_case → physical kebab-case;
- `load_synthetic_root` using `name.replace('-', "_")`.

This must be settled.

## 18.1 Recommended rule

Preserve the deliberate canonical convention already implemented:

```text
logical module component: snake_case Phalcom identifier
physical source spelling: kebab-case
```

Examples:

```text
logical: http_client
physical: http-client.ph

logical: data_tools
physical: data-tools/package.ph
```

Then update `modules-next.md` to explicitly specify this convention instead of saying no translation occurs.

Why this recommendation:

- code already has canonical validation;
- it prevents platform/case ambiguity;
- it supports package/distribution names independently;
- it is deterministic rather than Python-style fuzzy lookup.

## 18.2 Alternative

Remove all kebab/snake conversion and require exact identifier-compatible physical names.

This is simpler conceptually but is a larger behavior change and discards existing canonicalization machinery.

Whichever choice is made, `load_synthetic_root`, filesystem provider, docs, diagnostics, and tests must agree. Do not keep the current mixed model.

---

# 19. Phase 15 — Initialization and Canonical Registry Invariants

The existing registry/materializer already provides a useful foundation. Preserve these properties while refactoring.

## 19.1 One registry record

Every reachable module/package identity must have one record.

## 19.2 Builtins share the same lifecycle

Builtin modules should participate in prepared/initialized/sticky-failure semantics where initialization applies.

Native bootstrap writes may happen in a special preparation phase but may not create a parallel object graph.

## 19.3 No duplicate direct-execution identity

A module selected directly by CLI and the same module reached through import must resolve to the same `ModuleId`.

Keep/add tests for:

```text
direct execution
import
re-export
REPL import
```

converging on one object where they refer to the same semantic package/module.

---

# 20. User-Visible Behavior After This Spec

The following behavior is mandatory even before the full reflection API from the companion spec lands.

## 20.1 Builtins

```text
ph> universe
// => <Package universe>

ph> universe.Object
// => Object

ph> import universe.callable
ph> callable
// => <Package universe.callable>

ph> import std
ph> std
// => <Package std>

ph> import std.json
ph> json
// => <Package std.json>
```

## 20.2 Project

The root namespace imported or referenced at runtime is a Package:

```text
<Project geometry-kit>  // development context, not namespace
       |
       +-- rootPackage --> <Package geometry>
```

## 20.3 Standalone

A truly standalone Module remains a Module with no fabricated Package/Project.

---

# 21. Diagnostics

Add structured errors rather than generic internal errors for:

- unavailable relative import context;
- unavailable Project intrinsic;
- builtin interface collision;
- builtin source-kind mismatch (should normally be assertion/test failure);
- package artifact resolution failure;
- package requirement not publishable from path-only identity;
- duplicate source identity;
- canonical package/module object mismatch.

Diagnostics should use logical module identity and URI first; filesystem source location may be supplementary.

---

# 22. Test Matrix

Every row that applies must be exercised in at least core runtime tests and REPL tests.

| Behavior | Project program | Direct module in project | Standalone module | Project REPL | Standalone REPL | Builtin |
|---|---:|---:|---:|---:|---:|---:|
| `universe` implicit binding | ✓ | ✓ | ✓ | ✓ | ✓ | n/a |
| `universe.Object` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `import std` | ✓ | ✓ | ✓ | ✓ | ✓ | n/a |
| `import std.json` | ✓ | ✓ | ✓ | ✓ | ✓ | n/a |
| root `package.ph` is Package | ✓ | ✓ | n/a | ✓ | n/a | ✓ |
| relative import | ✓ | ✓ | ✗ | ✓ root-context | ✗ | ✓ internal |
| import persists across REPL cells | n/a | n/a | n/a | ✓ | ✓ | n/a |
| canonical module registry identity | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Project context exists | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| package identity survives project stripping | fixture | fixture | n/a | n/a | n/a | conceptually ✓ |

---

# 23. Implementation Sequence and Commit Boundaries

Recommended sequence:

1. **tests(modules): add semantic contract failures**
2. **refactor(modules): remove ProjectRoot**
3. **refactor(runtime): separate Project from Package class hierarchy**
4. **fix(builtins): correct package kinds**
5. **refactor(builtins): unify source-derived interface + native overlay**
6. **fix(runtime): canonical builtin materialization**
7. **refactor(modules): orthogonal namespace/package ownership**
8. **feat(runtime): contextual intrinsic lowering**
9. **refactor(modules): shared execution context**
10. **feat(repl): resolver/linker-aware persistent imports**
11. **fix(repl): semantic reload**
12. **refactor(packages): artifact/provider boundary**
13. **docs(modules): reconcile semantic model**

Do not combine all phases into one unreviewable commit if the repository workflow permits incremental commits.

---

# 24. Non-Goals

Do not implement in this track:

- registry network client;
- package publishing command;
- lockfile format;
- package signing;
- workspace/multi-package project model;
- optional feature resolution;
- final user-facing reflection classes;
- full PackageInfo public protocol;
- multi-entry package UX;
- remote cache policy.

The artifact boundary must support those later without requiring semantic reversal.

---

# 25. Completion Checklist

The track is complete only when all are true:

- [ ] production code contains no `ModuleKind::ProjectRoot`;
- [ ] `Project` no longer subclasses `Package`;
- [ ] root project `package.ph` materializes as `Package`;
- [ ] builtin roots materialize as `Package`;
- [ ] `std.json`, `std.fs`, `std.path`, and every other `package.ph` builtin are classified as `Package`;
- [ ] builtin source metadata is not dropped by a manual interface shortcut;
- [ ] canonical `universe` is the same object used by resolver/import/runtime member access;
- [ ] `universe.Object` reads through the runtime export table;
- [ ] no second bootstrap universe package is created;
- [ ] `std` is resolvable through imports in program and REPL;
- [ ] REPL imports no longer silently no-op;
- [ ] REPL imports persist across cells;
- [ ] `:reload` reconstructs semantic context before replay;
- [ ] standalone REPL can import `universe`/`std` but cannot perform relative imports;
- [ ] Project ownership is not encoded in ModuleKind;
- [ ] published-package resolution has an artifact boundary distinct from Project development context;
- [ ] tests cover user-visible semantic composition, not only isolated layers;
- [ ] module specification no longer claims `Project` is above Package in runtime inheritance.

---

# Appendix A — Targeted Symbol Searches

Use these only when the named phase is active:

```bash
# ProjectRoot removal
rg -n "ProjectRoot|is_package_like" phalcom-modules phalcom-core phalcom-repl

# Project-as-package runtime coupling
rg -n "project_class|owning_project|__project__" phalcom-core

# builtin duplication
rg -n "create_builtin_package|UNIVERSE_BINDINGS|BuiltinProjectSourceProvider" \
  phalcom-core phalcom-modules

# REPL bypass
rg -n "compile_closure_as|create_module\\(\"main\"|ReplSession" \
  phalcom-repl phalcom-core

# import preamble
rg -n "preamble\\.dependencies|ImportSurface|LinkedReadSpec|GetLinked" \
  phalcom-ast phalcom-modules phalcom-core

# core compatibility
rg -n "ModuleId::core|CORE_MODULE_NAME|\"core\" \\| \"universe\" \\| \"std\"" \
  phalcom-modules phalcom-core

# package artifact provider
rg -n "DependencyProvider|ResolvedDependencySource|resolve_package" phalcom-modules
```

---

# Appendix B — Architectural Decisions Ratified by This Spec

1. **Project is not a namespace.**
2. **Project does not survive publication.**
3. **Package is the runtime namespace/artifact root.**
4. **Every `package.ph` creates a Package.**
5. **Builtin roots are Packages.**
6. **`universe` is implicitly bound; `std` is explicitly imported.**
7. **Prelude values are canonical universe values, not duplicates.**
8. **REPL uses the module system rather than a parallel compiler model.**
9. **Standalone REPL still has builtin import roots.**
10. **Context intrinsics are compiler/runtime semantics, not ordinary user globals.**
11. **Exposure and export remain distinct.**
12. **Registry package resolution must target package artifacts, not recreate development Projects.**
13. **Reflection/public API is layered only after these identity/runtime invariants hold.**
