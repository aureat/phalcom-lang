# Phalcom Modules v1 Implementation Specification — Part III
## Runtime module/package objects, export namespaces, graph-driven initialization, execution, CLI integration, failure semantics, migration, and performance verification

**Status:** Implementation specification
**Target:** Phalcom first-version static module/package/project system
**Repository:** `aureat/phalcom-lang`
**Repository snapshot inspected:** `ed841918546610752ec0b1d3f7b1ffa6b2056006` (`main`)
**Depends on:** Part I and Part II

---

# 1. Purpose

Parts I–II deliberately remove loading/resolution/linking responsibilities from the runtime. Part III makes the VM consume the resulting closed `CompiledProgram`.

The target runtime model is:

```text
CompiledProgram
    ↓
allocate canonical Module/Package records
    ↓
materialize static declarations/linkage
    ↓
initialize runtime dependency DAG
    ↓
execute canonical entry module
```

The runtime does **not**:

- parse import syntax;
- resolve filesystem paths;
- search package roots;
- discover dependencies;
- compile imported modules on demand;
- tolerate cyclic initialization through partial modules;
- execute `package.ph` merely because it is an ancestor;
- attach runtime decorators to modules/packages;
- run compile-time module/package expanders;
- create a second `__main__` identity.

This is the payoff of the static design: the VM receives indexes and handles, not unresolved import strings.

---

# 2. Current runtime implementation to remove

Repository anchors at the inspected snapshot:

## 2.1 `phalcom-core/src/interpret.rs`

Current functions:

- `normalize_path`
- `append_ph_extension_if_missing`
- `resolve_import_path`
- `resolve_module_path`
- `Interpreter::run_file`
- `VM::compile_closure_as`
- `VM::run_in_module`
- `VM::interpret_source`
- `VM::import_module`

`VM::import_module` currently:

1. canonicalizes a source-relative physical path;
2. probes `Universe::module_registry`;
3. allocates/registers a `ModuleObject` **before** compilation/execution;
4. compiles source on demand;
5. executes it re-entrantly;
6. returns a partial record when reached cyclically.

The new design deletes steps 1–6 from runtime import behavior.

## 2.2 `phalcom-core/src/universe/mod.rs`

Current:

```rust
pub module_registry: HashMap<String, ObjRef>
```

is keyed by canonical physical path and intentionally exposes early-inserted partially populated modules for cycles.

Remove this field from `Universe`.

`Universe` should describe the kernel/core object universe, not be the user program's module loader.

## 2.3 `phalcom-core/src/vm/mod.rs`

Current fields:

```rust
pub modules: HashMap<Symbol, ObjRef>,
pub main_module: Option<ObjRef>,
pub last_imported_module: Option<ObjRef>,
```

are insufficient or obsolete for semantic project module identity.

Replace with a dedicated runtime module registry keyed by semantic `ModuleId`.

## 2.4 `phalcom-core/src/vm/api.rs`

Current anchors:

- `VM::create_module(logical_name, abs_path)`
- `VM::register_path`
- `VM::get_module`
- `VM::get_module_from_str`
- `VM::define_global(module_sym, ...)`

Refactor APIs to operate on module handles/semantic ids rather than globally unique name symbols.

## 2.5 `phalcom-core/src/heap/module.rs`

Current `ModuleObject` contains:

- name/path;
- global slots;
- name-to-slot map;
- sources;
- closure;
- runtime attribute retention;
- binding mutability map.

Keep the useful compact slot representation, but split semantic identity/source/public-export metadata from internal globals.

## 2.6 `phalcom-core/src/primitive/module.rs`

Current `Module#doesNotUnderstand` treats **every module global** as an externally reachable member.

That conflicts with private-by-default exports and must change.

## 2.7 `phalcom-core/src/primitive/attribute.rs`

Current `Object#__attach`, `__attributes`, `__freezeAttributes` accept `Object::Module`.

That is no longer valid for module/package objects. V1 module/package attributes are inert compiler metadata only.

---

# 3. Runtime object model: `Package < Module < Object`

Add a kernel `Package` class as a subclass of `Module`.

Modify:

- `phalcom-core/src/universe/core_classes.rs`
- `phalcom-core/src/universe/primitives.rs`
- `phalcom-core/src/universe/invariants.rs`
- relevant bootstrap/table/invariant tests.

Create after `Module`:

```rust
let module_class = make_core_class(heap, "Module", object_class, metaclass_class);
let package_class = make_core_class(heap, "Package", module_class, metaclass_class);
```

Add `package_class` to `CoreClasses`.

Mark both native-representation classes.

Do **not** add another heap `Object::Package` variant. Keep one compact payload:

```rust
Object::Module(Box<ModuleObject>)
```

and add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleKind {
    Module,
    Package,
}
```

to `ModuleObject`.

Then change `Value::class` in `phalcom-core/src/value/mod.rs`:

```rust
Object::Module(module) => match module.kind {
    ModuleKind::Module => vm.universe.classes.module_class,
    ModuleKind::Package => vm.universe.classes.package_class,
}
```

This gives true runtime specialization without duplicating storage/accessors/GC tracing.

Because `Package < Module`, module class-side or inherited protocol behavior remains naturally shared.

---

# 4. Refactor `ModuleObject`

Modify `phalcom-core/src/heap/module.rs`.

Recommended shape:

```rust
pub struct ModuleObject {
    pub id: phalcom_modules::ModuleId,
    pub kind: ModuleKind,

    // Diagnostic/source data, not identity.
    pub source: SourceLocation,

    // Human-readable logical name cached for diagnostics/debug display.
    pub display_name: Box<str>,
    pub name_sym: Symbol,

    // Top-level runtime storage.
    globals: Vec<Value>,
    name_to_slot: HashMap<Symbol, u16>,
    global_bindings: HashMap<Symbol, GlobalBinding>,

    // Static linked runtime reads.
    linked_reads: Box<[RuntimeLinkedRead]>,

    // Public namespace only.
    exports: HashMap<Symbol, RuntimeExportRef>,

    // Inert static metadata.
    metadata: Arc<ModuleMetadata>,

    // Existing source/chunk diagnostic retention as required by REPL/traceback.
    pub sources: Vec<Arc<String>>,
    pub closure: Option<ObjRef>,
    pub globals_version: u64,
}
```

Retire from `ModuleObject`:

```text
attributes: Vec<Value>
attributes_frozen: bool
```

unless another non-module use specifically needs them. Class/method attributes stay untouched.

Retire the old process-global numeric `heap::module::ModuleId` alias and `next_module_id`.

Semantic module identity is `phalcom_modules::ModuleId`.

---

# 5. Runtime binding reference

Implement in `phalcom-core/src/modules/linkage.rs` or `heap/module.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingRef {
    pub module: ObjRef,
    pub slot: u16,
}
```

This is the physical implementation of a live binding cell.

Methods:

```rust
impl BindingRef {
    pub fn read(self, vm: &VM) -> Value {
        vm.heap.module(self.module).get_by_slot(self.slot as usize)
    }
}
```

Do not expose a public setter through `BindingRef`.

The defining module writes its own global slot through normal module-global assignment machinery.

Because global slots are stable after module layout construction, the pair remains valid for the life of the module object.

---

# 6. Runtime linked reads

Part II produces symbolic:

```rust
LinkedReadSpec::Module(ModuleId)
LinkedReadSpec::Binding(SymbolId)
```

Part III materializes:

```rust
pub enum RuntimeLinkedRead {
    Module(ObjRef),
    Binding(BindingRef),
}
```

Store in the importing `ModuleObject` as an indexed boxed slice.

`Bytecode::GetLinked(u16)` in `phalcom-core/src/vm/dispatch.rs`:

```text
current closure
    ↓ module ObjRef
module.linked_reads[index]
    ├── Module(target)  → push target module value
    └── Binding(ref)    → read target slot, push current value
```

This is O(1), contains no path/string lookup, and allocates nothing.

There is no runtime import opcode.

---

# 7. Runtime export references

A module export may refer to:

- one of its own global slots;
- a selective imported source binding;
- an imported module object.

Use:

```rust
#[derive(Clone, Copy, Debug)]
pub enum RuntimeExportRef {
    Binding(BindingRef),
    Module(ObjRef),
}
```

Materialization rules:

```text
export localGlobal
    → BindingRef(current module, local slot)

from M import x; export x
    → target BindingRef directly

export x from M
    → target BindingRef directly

import M as m; export m
    → RuntimeExportRef::Module(target module ObjRef)
```

Re-export does not allocate a forwarding slot unless another compiler invariant explicitly requires one.

This is both faster and semantically stronger: aliases/re-exports preserve the original live cell.

---

# 8. Export namespace is not the globals namespace

This is a hard runtime invariant.

Current `Module#doesNotUnderstand` calls:

```rust
ModuleObject::get(name_sym)
```

against all globals.

Replace external module lookup with:

```rust
ModuleObject::export(name_sym) -> Option<RuntimeExportRef>
```

Private globals remain inaccessible even though they occupy runtime slots.

There must be no reflection-free path by which a consumer can enumerate/access `name_to_slot`.

---

# 9. Solve Module-method/export-name collisions correctly

Modules are ordinary runtime values, but their public member namespace is the export namespace. A library must be free to export names such as:

```text
name
class
hash
inspect
metadata
```

without future kernel `Module` methods stealing those names.

Therefore module/package export dispatch must run **before ordinary instance-side method lookup** for user sends.

Modify the common send path in `phalcom-core/src/vm/dispatch.rs`, near `invoke_dynamic_selector`, by adding:

```rust
fn try_module_export_send(
    &mut self,
    receiver: Value,
    selector: Symbol,
    receiver_idx: usize,
    source_range: SourceRange,
) -> PhResult<Option<()>>;
```

Behavior:

1. If receiver is not `Object::Module`, return `Ok(None)`.
2. Decode selector to base name/shape.
3. Look up base name in module's **export table**.
4. If no export, return `Ok(None)` and continue ordinary class dispatch.
5. If getter/no arguments, read export and return value.
6. If call arguments exist, read export value then invoke its normal `call(...)` protocol with the incoming arguments.
7. Compiler-internal/non-user selectors may bypass this hook where required by internal authority rules.

Then normal `Module`/`Package` instance methods are fallback behavior only when no export claims that public name.

Move administrative/reflection APIs to class-side/external reflection forms, e.g. conceptually:

```phalcom
Module.identityOf(module)
Module.metadataOf(module)
```

rather than `module.identity`, so the kernel does not consume application export names.

After this change, `phalcom-core/src/primitive/module.rs::module_does_not_understand` no longer needs to be the export lookup mechanism. Simplify/remove its special global-table behavior.

---

# 10. Metadata-only module/package attributes

Part I parses metadata into `ModuleMetadata`.

Part III merely stores that immutable metadata on `ModuleObject`.

No `Attribute` instance is constructed.
No `Object#__attach` is called.
No module initializer code is generated.

Modify `phalcom-core/src/primitive/attribute.rs`:

Current accepted receiver kinds:

```text
Class, Method, or Module
```

New:

```text
Class or Method
```

Remove `Object::Module` branches from:

- `attribute_attach`
- `attribute_attributes`
- `attribute_freeze`

Update error text and tests.

If v1 exposes module metadata reflectively, prefer a class-side primitive:

```text
Module.class::metadataOf(_)
```

or an existing Reflection facility if one exists by implementation time.

Metadata conversion to surface values happens on query, not during module initialization.

---

# 11. Runtime registry

Create:

```text
phalcom-core/src/modules/registry.rs
```

```rust
pub struct ModuleRegistry {
    by_id: HashMap<ModuleId, ModuleRecord>,
}
```

`ModuleRecord`:

```rust
pub struct ModuleRecord {
    pub object: ObjRef,
    pub state: ModuleState,
    pub failure: Option<ModuleFailure>,
}
```

State:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleState {
    Prepared,
    Initializing,
    Initialized,
    Failed,
}
```

Because runtime cycles are statically rejected, `Initializing` is not an externally usable partial state. It exists to assert state transitions and diagnose internal corruption/re-entry.

Put `ModuleRegistry` on `VM`:

```rust
pub module_registry: ModuleRegistry,
```

Remove `Universe::module_registry`.

GC tracing must visit every `ModuleRecord.object`. Update `VM`/GC root walk accordingly.

Do not key registry by source path or display-name `Symbol`.

---

# 12. Program materialization

Create:

```text
phalcom-core/src/modules/materialize.rs
```

API:

```rust
impl VM {
    pub fn materialize_program(
        &mut self,
        program: &CompiledProgram,
    ) -> PhResult<RuntimeProgram>;
}
```

Phases:

## 12.1 Allocate every runtime module/package object

For every runtime-reachable compiled module:

1. intern display name;
2. construct `ModuleObject` with correct semantic `ModuleId`/kind/source;
3. allocate on heap;
4. insert `ModuleRecord { Prepared }`.

No module body runs.

Because runtime graph is already a DAG, allocation is not a cycle workaround; it is ordinary closed-program materialization.

## 12.2 Allocate global layouts

Reserve/declare all module-global slots using Part II's binding layout.

Slot indexes must be stable before exports/import linked reads are materialized.

## 12.3 Materialize declaration blueprints

Allocate static declaration objects that are designed to exist before ordinary initializer execution, especially class shells/layouts if Part II produced `ClassBlueprint`s.

Validate construct-specific dependency rules.

Do not evaluate arbitrary initializer expressions here.

## 12.4 Materialize linked reads

Translate `LinkedReadSpec`:

```text
ModuleId → target module ObjRef
SymbolId → defining module ObjRef + global slot
```

Store boxed slices on importing module.

## 12.5 Materialize export table

Translate every linked export to `RuntimeExportRef`.

## 12.6 Install initializer closures/artifacts

Associate each module with its already-compiled top-level initializer closure.

At the end:

```text
every ModuleId has one object
every linked read is indexed
every export resolves
no user top-level code has run
```

---

# 13. No ancestor package initialization

Do not add any implicit dependency edge from:

```text
geometry.shapes.circle
```

to:

```text
geometry
geometry.shapes
```

solely because those are containing packages.

Resolution may have parsed/read their static `expose` surfaces. Runtime does not execute them for containment.

Example:

```text
geometry/package.ph
    re-exports point/vector/shapes

geometry/point.ph
    small leaf
```

Running/importing `geometry.point` initializes `geometry.point` and its **actual runtime imports** only.

Importing `geometry` initializes the root package and the children it explicitly runtime-imports/re-exports.

This avoids the original spec's eager façade traversal.

---

# 14. Graph-driven initialization

Create:

```text
phalcom-core/src/modules/initialize.rs
```

The `CompiledProgram` already contains a topological initialization order.

API:

```rust
impl VM {
    pub fn initialize_program(
        &mut self,
        runtime: &RuntimeProgram,
        program: &CompiledProgram,
    ) -> PhResult<()>;
}
```

For each module in dependency-first order:

```text
Prepared
    ↓
Initializing
    ↓ success
Initialized
```

or:

```text
Initializing
    ↓ error
Failed
```

Before initializing module A, assert every `A -> dependency` is already `Initialized`.

No source import declaration is encountered during this execution.

---

# 15. Initialization order semantics

Suppose:

```text
A → B
A → C
B → D
C → D
```

Guarantees:

```text
D before B
D before C
B before A
C before A
```

No semantic guarantee:

```text
B before C
or
C before B
```

Use the precomputed deterministic order for reproducibility, but document sibling order as unspecified.

Correct modules must express real initialization dependencies through actual runtime imports.

Import text order cannot force sibling order.

---

# 16. Runtime cycle invariant

Part II rejects cycles. Part III still defends the invariant.

If initializer code attempts to access a runtime dependency whose state is not `Initialized`, do **not** expose a partial module or implement a TDZ language rule.

Return an internal invariant error such as:

```text
InternalModuleOrderViolation
```

or panic in debug-only impossible paths, according to repository error-handling conventions.

This condition must have a test that can only be triggered by constructing an invalid synthetic `CompiledProgram`, not legal source.

There is no user-facing static-import partial-module behavior.

---

# 17. Sticky initialization failure

Keep the ratified sticky failure policy.

Define:

```rust
pub struct ModuleFailure {
    pub module: ModuleId,
    pub cause: PhError,
    pub initialization_chain: Vec<ModuleId>,
}
```

When dependency B fails during A startup:

```text
B → Failed(original cause)
A → Failed(caused by B failure)
```

Do not rerun either initializer automatically.

Although normal static startup attempts initialization once, sticky state still matters for:

- embedding;
- REPL/project sessions;
- repeated entry attempts in one VM;
- future reflective loading.

Preserve original provenance.

---

# 18. Initialization and fibers/concurrency

The absence of cyclic initialization removes the hardest concurrency problem, but startup still needs a clear policy.

V1 policy:

> Static program initialization is a single scheduler-owned startup transaction. Module initializers are run serially in the precomputed dependency order. The VM does not parallelize module initialization.

This avoids:

- per-module OS mutexes;
- cross-fiber import deadlocks;
- observable partial modules;
- nondeterministic initialization races.

Scheduled fibers created during module initialization may be queued, but the ordinary scheduler pump must not begin independent scheduled work until static module initialization is complete and the entry module is ready to run, unless an explicit synchronous fiber operation is already part of ordinary language semantics.

Do not hold an OS blocking lock across a fiber suspension.

Future parallel module initialization may use the same DAG because sibling ordering is deliberately unspecified.

---

# 19. Entry module semantics

The selected entry module retains its canonical `ModuleId`.

There is no `__main__`.

Execution steps:

```text
select entry source/project/package
    ↓
compile closed static program
    ↓
materialize all runtime-reachable modules
    ↓
initialize dependencies
    ↓
initialize/execute entry module
```

If another module references/imports the entry module in a legal acyclic graph, the linker/runtime must still resolve one canonical object. In practice a runtime edge back to an importer may form a cycle and be rejected; identity is nevertheless singular.

Entry status does not:

- rename globals;
- alter exports;
- change relative import rules;
- create a function;
- bypass package semantics.

---

# 20. Module initialization versus application startup

A module's top-level executable code is its initializer.

For an entry module, that same top-level code is the application entry execution.

There is no hidden:

```phalcom
main(args)
```

call synthesized by the module system.

Process arguments remain a standard-runtime concern.

---

# 21. CLI entry classification

Refactor `phalcom-core/bin/phalcom/cli.rs::cmd_run`.

Current path:

```text
read one source
create Module("main", abs path)
interpret_source
```

must be replaced by `EntrySelection`.

Recommended:

```rust
pub enum EntrySelection {
    Project(ProjectRoot),
    Package(PackageSelection),
    Module(SourceSelection),
    Inline(Arc<str>),
}
```

## 21.1 No explicit path

If command is run with no path/source:

1. if current directory contains/owns a project, run manifest entry;
2. otherwise report usage/project-not-found according to CLI policy.

CWD is used only to select a project target, not for imports.

## 21.2 Project directory target

Directory containing `project.toml`:

- parse project;
- require `[project].entry`;
- resolve entry in current project;
- compile/run it.

## 21.3 Package directory target

Directory containing `package.ph` but not selected as project:

- require direct child `main.ph`;
- entry = `<package>.main`.

Do not execute `package.ph` as package-directory entry.

## 21.4 File target

Determine nearest ownership:

```text
project-backed?
else standalone package?
else standalone module
```

Compute canonical logical identity before compilation.

## 21.5 Explicit `package.ph`

Selecting `package.ph` explicitly selects the package module itself as entry. This is distinct from running the package directory.

---

# 22. Project execution

For:

```toml
[project]
namespace = "app"
entry = "app.cli"
```

`phalcom run` from project (or equivalent current CLI invocation) runs `app.cli`.

Validation before runtime:

- entry resolves;
- entry belongs to current project;
- entry is ordinary module or explicitly allowed package module according to chosen rule;
- dependency entry is rejected;
- project with no entry => `ProjectNotExecutableError`.

A library project is still valid without entry.

---

# 23. Package execution

Given:

```text
tools/
├── package.ph
├── main.ph
└── commands.ph
```

running package directory selects:

```text
tools.main
```

`tools/package.ph` is **not** automatically initialized merely because it is the containing package.

If `tools.main` needs façade exports, it explicitly imports:

```phalcom
from . import SomeBinding
```

which creates a real runtime dependency on package `tools`.

This is the direct consequence of the ratified containment ≠ initialization rule.

---

# 24. Standalone module execution

Inline/standalone source receives an execution-local identity and no sibling user import root.

For an inline `--source` program:

- core is available;
- user project imports are rejected unless an explicit project context was supplied through a future CLI option;
- there is no fallback to process CWD.

Remove the old behavior where `"<main>"` caused relative path imports to resolve against CWD.

---

# 25. Runtime source locations and traceback display

Semantic identity is no longer a physical path, but tracebacks still need source locations.

Keep `SourceLocation` on `ModuleObject`.

Current `VM::capture_frames` accesses:

```text
closure.module
module.source_at(...)
module.name_sym
```

Preserve this path.

Display should use logical module name plus source path where appropriate:

```text
app.geometry.point (/home/x/project/src/geometry/point.ph)
```

but never use the path as registry identity.

Relocating a project changes diagnostic paths, not class/module semantic structure.

---

# 26. Core module handling

Current `VM.modules` provides name-based access to `core`/`main`.

Replace ad hoc main/core lookup with explicit handles:

```rust
pub struct RuntimeRoots {
    pub core: ObjRef,
    pub entry: Option<ObjRef>,
}
```

or equivalent fields on `VM`.

Core bootstrap remains special in that it exists before user program materialization, but user program imports/resolution should refer to an explicit reserved core semantic identity.

Do not put user modules back into a `HashMap<Symbol, ObjRef>` merely to preserve old helper APIs.

---

# 27. Global definition API refactor

Current:

```rust
VM::define_global(module_sym, name_sym, value)
```

looks module up through `VM.modules`.

Change compiler/runtime call sites to operate directly on the current frame/module handle:

```rust
pub fn define_global(
    &mut self,
    module: ObjRef,
    name: Symbol,
    value: Value,
) -> PhResult<usize>
```

This avoids an unnecessary global symbol lookup and removes the false assumption that module names are VM-wide unique simple symbols.

Likewise refactor:

- `get_module`;
- `get_module_from_str`;
- `register_path`.

Delete helpers that no longer have a semantic use.

---

# 28. `ModuleObject` global layout and export slots

During materialization, create global slots from the compiler's known layout once.

Avoid repeated `HashMap` insert/resize during initializer execution where practical.

Recommended builder:

```rust
pub struct ModuleObjectBuilder {
    ...
}
```

or:

```rust
ModuleObject::with_layout(
    id,
    kind,
    source,
    global_names,
    metadata,
)
```

`name_to_slot` may remain for dynamic internal/global operations, but compiled code should prefer numeric slot bytecodes when the compiler already has them.

Exports resolve to numeric slots once.

This is an opportunity to move more top-level global access away from name hash lookups if consistent with the current compiler architecture.

---

# 29. Module/Package creation is not public construction

Keep:

```text
Module.class::new() → NotAllowed
```

and ensure `Package.class::new()` inherits/receives the same restriction.

Only the program materializer creates these runtime values.

The class hierarchy exists for type/reflection/dispatch semantics, not user allocation.

---

# 30. Public module member send semantics

Examples:

```phalcom
import geometry as Geometry

Geometry.Point
Geometry.distance(p, q)
```

Runtime export dispatch:

```text
receiver Object::Module
    ↓
export base-name lookup
    ↓
RuntimeExportRef
    ↓
read Value
    ├── getter: return value
    └── call-shape: send call(...) to value
```

This preserves the current useful “module member function is a callable exported value” behavior while enforcing exports.

A missing export falls through to genuine Module/Object behavior and eventually ordinary `doesNotUnderstand`.

An unexported private global is indistinguishable from a nonexistent public member through ordinary module member access. Compiler/LSP selective-import diagnostics can still distinguish private from absent because they possess the interface.

---

# 31. Static fast path versus dynamic module values

Two access routes must be semantically equal.

Static known alias:

```phalcom
import .base as base
base.Shape
```

Part II may compile direct `GetLinked` for `Shape`.

Dynamic module value:

```phalcom
const m = chooseModule()
m.Shape
```

uses export dispatch.

Both read the same `RuntimeExportRef`.

Tests must prove equality for:

- local export;
- selective re-export;
- re-export alias;
- mutable exported global;
- exported module object.

---

# 32. Module metadata reflection

If implemented in v1, add class-side primitive(s) rather than instance getter names.

Example conceptual surface:

```phalcom
Module.metadataOf(m)
```

returns immutable data describing file-header metadata.

Requirements:

- query performs no source execution;
- metadata cannot be mutated to change compiler/runtime behavior;
- Package works because `Package < Module`;
- metadata lookup never changes export dispatch.

If no public reflection API is ready, storing `ModuleMetadata` is sufficient for v1; LSP/docs/compiler can still consume it. Do not invent an instance `module.metadata` selector that competes with exports.

---

# 33. Failure diagnostics

Required runtime errors:

```text
ModuleInitializationError
ProjectNotExecutableError
PackageNotExecutableError
InvalidProjectEntryError
```

Static resolution/link errors come from Parts I–II.

`ModuleInitializationError` should report:

- logical ModuleId/display name;
- source location;
- original error;
- dependency initialization chain if failure propagated.

Example:

```text
ModuleInitializationError:
  failed to initialize `app.server`

Caused by initialization of dependency `app.config`:
  FileReadError: ...

Dependency chain:
  app.server
  → app.config
```

Do not turn the original error into a generic string if structured cause retention is possible.

---

# 34. Sticky failure tests

Test:

```text
B initializer increments host-side probe then fails
A depends on B
```

First startup:

```text
probe == 1
B Failed
A Failed
```

Second attempt in same VM:

```text
probe remains 1
same cached B failure reused
```

No retry.

Development reload is future explicit tooling.

---

# 35. Package façade example under final semantics

Tree:

```text
geometry/
├── project.toml
└── src/
    ├── package.ph
    ├── point.ph
    ├── vector.ph
    └── internal/
        ├── package.ph
        └── cache.ph
```

Root `package.ph`:

```phalcom
expose .point
expose .vector

export Point from .point
export Vector from .vector
```

`point.ph`:

```phalcom
class Point {
}

export Point
```

External:

```phalcom
import geometry
from geometry import Point
import geometry.point
```

works.

External:

```phalcom
import geometry.internal.cache
```

fails at static path visibility.

Direct leaf import:

```phalcom
import geometry.point
```

does **not** execute root `geometry/package.ph`.

Root façade import:

```phalcom
import geometry
```

initializes runtime dependencies required by the façade re-exports before the root package initializer.

---

# 36. Live selective import example

`settings.ph`:

```phalcom
let mode = "development"
export mode

class SettingsControl {
    static production() {
        mode = "production"
    }
}

export SettingsControl
```

consumer:

```phalcom
from .settings import mode, SettingsControl

System.print(mode)
SettingsControl.production()
System.print(mode)
```

Expected:

```text
development
production
```

Runtime mechanism:

```text
consumer linked-read `mode`
    = BindingRef(settings module ObjRef, mode slot)
```

No copied snapshot.

Attempt:

```phalcom
mode = "test"
```

is compile-time illegal because import binding is immutable.

---

# 37. Runtime initialization cycle example

`a.ph`:

```phalcom
from .b import B

class A {
}

export A
```

`b.ph`:

```phalcom
from .a import A

class B {
}

export B
```

If both selective imports are runtime dependencies, Part II rejects:

```text
a → b → a
```

No module object is partially exposed and no initializer begins.

If future typing makes these references interface-only because they occur solely in semantic signatures, the semantic SCC is legal and no runtime cycle exists.

This distinction must remain visible in compiler diagnostics.

---

# 38. No import-time source position effects

Old:

```phalcom
System.print("before")
import .point
System.print("after")
```

is no longer legal because import is outside the preamble.

Valid:

```phalcom
import .point

System.print("before")
System.print("after")
```

`point` initialization is scheduled from the runtime dependency graph, not from a program counter reaching an import statement.

Formatter/import organizer may reorder static import declarations without behavior change.

---

# 39. Disassembler behavior

After the change, source imports should not appear as `IMPORT` bytecodes.

For:

```phalcom
from .point import Point

const p = Point.new()
```

disassembly should show a linked read, e.g.:

```text
GET_LINKED 0   ; Point -> geometry.point::Point
...
```

Disassembler may use compiler metadata to annotate the target logically.

It must not print a physical source path as the semantic operand.

---

# 40. GC integration

Moving registry ownership from `Universe` to `VM` changes roots.

Update:

- `phalcom-core/src/vm/gc.rs`
- `phalcom-core/src/heap/trace.rs`
- `Universe::each_handle`
- any root census docs/tests.

Requirements:

1. every runtime module object in `ModuleRegistry` is rooted;
2. module globals continue tracing `Value` children;
3. `RuntimeExportRef::Module(ObjRef)` must be reachable either through registry or module trace;
4. `BindingRef` contains an `ObjRef`; if export/import linkage could outlive registry ownership, trace it explicitly;
5. `RuntimeLinkedRead::Module`/`Binding` references are traced from `ModuleObject` or guaranteed by registry roots;
6. metadata containing no heap `Value`s needs no GC tracing.

Prefer explicit tracing of module linkage references even if registry roots make it redundant; this preserves correctness if registry lifetime changes later.

---

# 41. REPL implications

The existing REPL uses a persistent module and repeated `compile_closure`/`run_cell`.

Do not force project static-graph semantics onto every individual REPL cell immediately.

Define a compatibility boundary:

- the REPL session module remains one canonical synthetic module;
- ordinary module-system imports in a REPL cell require the session to possess a project/module resolver context;
- once a static dependency is added to the session, it is linked through the module subsystem, not runtime `Bytecode::Import`;
- previously linked imports remain immutable bindings;
- no per-cell dynamic import statement is created.

If full REPL import support would balloon this implementation, initially reject user module import declarations in context-free REPL sessions with a targeted diagnostic and add project-aware REPL support separately.

Do not retain `VM::import_module` solely for the REPL.

---

# 42. `Interpreter` refactor

`phalcom-core/src/interpret.rs` should stop owning import path resolution.

Recommended responsibilities after refactor:

```text
Interpreter
    owns VM
    owns/configures ProgramCompiler
    selects/compiles program
    materializes program
    initializes/executes program
```

New high-level methods:

```rust
impl Interpreter {
    pub fn compile_entry(
        &mut self,
        selection: EntrySelection,
    ) -> Result<CompiledProgram, CompileProgramError>;

    pub fn run_compiled(
        &mut self,
        program: &CompiledProgram,
    ) -> PhResult<()>;
}
```

`VM::interpret_source` may remain for isolated bootstrap/tests/REPL, but it is not the general multi-module project entry path.

---

# 43. Core bootstrap isolation

`VM::new()` currently bootstraps core using its own source/compiler path.

Do not make user project resolution a prerequisite for kernel bootstrap.

Recommended:

```text
VM::new()
  → bootstrap core synthetic ModuleId
  → install kernel classes/primitives/core.ph
  → user CompiledProgram materialization later
```

Core's semantic identity is reserved and stable.

After bootstrap, mark existing performance guards pristine exactly as today.

The module refactor should avoid perturbing unrelated kernel bootstrap behavior.

---

# 44. Module source compilation cache hook

Because imports are static logical identities, `CompiledProgram` can later source artifacts from:

- parsed source;
- bytecode cache;
- AOT object.

Do not expose artifact kind in source syntax.

Part III runtime accepts a `ModuleArtifact`; it should not care whether that artifact originated from source this process compiled or a validated cache.

This is why physical source canonicalization remains outside `ModuleId`.

---

# 45. CLI diagnostics and migration

Physical U15 import:

```phalcom
import "./geometry/point" as Point
```

must fail parsing or a migration diagnostic.

If implementing targeted migration diagnostics, suggest:

```phalcom
import .geometry.point as Point
```

or absolute logical path based on known project context.

Do not silently accept/translate old imports in the compiler proper.

Other migration diagnostics:

```text
import after body start
    → move to dependency preamble

external deep import of private path
    → import package façade or request path exposure

unknown export
    → name exists but is private / truly missing distinction

package directory without main.ph
    → PackageNotExecutableError
```

---

# 46. Remove ancestor package execution assumptions from CLI

Current original specification expected running nested modules to initialize package ancestors. The new CLI must not synthesize these steps.

Selecting:

```text
src/tools/demo.ph
```

resolves `app.tools.demo`.

Runtime dependency list comes solely from linked imports/re-exports.

The fact that `app` and `app.tools` package sources exist is resolution metadata, not startup code.

---

# 47. Project/package/main identity tests

Required:

1. running project manifest entry `app.cli` and importing `app.cli` refer to same semantic `ModuleId`;
2. direct `tools/main.ph` and package-directory execution select same `tools.main`;
3. explicit `tools/package.ph` selects `tools` package module, not `tools.main`;
4. moving project checkout preserves logical identities;
5. renaming standalone package directory changes standalone identity;
6. consumer dependency alias change does not change dependency `ModuleId`;
7. two distinct resolved dependency instances produce distinct module/class identities.

---

# 48. Module object identity tests

Within one VM:

```text
ModuleId -> exactly one ObjRef
```

Test:

- two local imports of same target;
- dependency alias/self-name references that resolve same project node;
- direct re-export chain;
- entry module reference;
- package root import.

All must reach one canonical object.

No registry key should depend on alias spelling.

---

# 49. Module export collision tests

Create a module exporting names that collide with core protocol selectors:

```phalcom
const name = "exported-name"
const hash = 42
export name, hash
```

Verify:

```phalcom
m.name
m.hash
```

read exports.

Verify reflective/administrative operations remain available through class-side/external APIs.

This test prevents later additions to `Module` from silently breaking package APIs.

---

# 50. Privacy runtime tests

Compile-time should catch most direct private access, but dynamic module values require runtime public-namespace enforcement.

Given:

```phalcom
const hidden = 1
const visible = 2
export visible
```

dynamic module send:

```text
module.visible -> 2
module.hidden  -> MessageNotUnderstood / missing public member
```

No direct `globals` fallback.

---

# 51. No partial visibility tests

Construct synthetic invalid runtime program (test-only API) where A initializer tries to read B while B is `Prepared`.

Expected:

- internal invariant failure;
- no value from B is returned;
- no partially initialized module is exposed;
- no `UninitializedModuleBindingError` public semantics are added.

Legal source must be rejected earlier by graph validation.

---

# 52. Initialization ordering tests

Fixtures:

## Linear

```text
C
↑
B
↑
A
```

log must be:

```text
C B A
```

## Diamond

```text
    A
   / \
  B   C
   \ /
    D
```

assert only dependency constraints, not B/C semantic order.

Implementation deterministic order can be snapshot-tested separately as a reproducibility property.

## No ancestor package init

Import leaf and record package initializer side effect; package side effect must not occur.

## Façade

Import package façade that re-exports children; child dependencies initialize before package.

---

# 53. Failure-order tests

Diamond where D fails:

```text
D fails
B not initialized successfully
C not initialized successfully
A not initialized successfully
```

Exact policy:

- modules not yet attempted remain `Prepared` or are marked blocked/failed according to chosen internal representation;
- repeated program startup must not rerun D if VM retains program session;
- reported chain should identify D as root cause.

Avoid executing independent modules after a fatal startup failure unless there is a deliberate reason; simplest v1 behavior is fail-fast.

---

# 54. Performance implementation notes

The new runtime should be materially cheaper than U15 imports.

Removed from runtime hot path:

- `PathBuf` join;
- `.ph` extension logic;
- filesystem `canonicalize`;
- `fs::read_to_string`;
- parser invocation;
- compiler invocation;
- module registry string hash by canonical path;
- re-entrant `run_until` per import opcode;
- import-path constant decode;
- partial-cycle registry behavior.

Added:

- one `GetLinked(u16)` indexed read;
- one topological startup walk;
- export hash lookup only for dynamic module member sends;
- direct linked-read fast path for statically known module exports.

---

# 55. Memory requirements

Keep module metadata/linkage compact.

Suggested representations:

```text
RuntimeLinkedRead: 16 bytes-ish, boxed slice
BindingRef: two compact handles/index
RuntimeExportRef: small enum
global slots: existing Vec<Value>
export map: only public names, not all globals
```

Do not allocate an object for:

- each import declaration;
- each export binding;
- each re-export;
- each binding cell.

Static compiler structures can be dropped after program materialization if not required for reflection/debugging/LSP.

---

# 56. Startup complexity

For V runtime modules and E runtime dependency edges:

```text
materialization: O(V + bindings + exports)
initialization planning: already O(V+E) in compiler
runtime startup walk: O(V+E assertions) + initializer execution
```

Runtime should not recompute SCCs; that belongs to Part II.

Optionally retain graph edges in debug builds for invariant checks/diagnostics, but the release runtime can use compact dependency/index arrays from `CompiledProgram`.

---

# 57. Module initialization instrumentation

Add tracing spans only on cold startup paths, not per `GetLinked`.

Useful events:

```text
module.materialize
module.initialize.begin
module.initialize.end
module.initialize.fail
```

fields:

```text
module_id
logical_name
kind
source
```

Do not add a tracing span around every imported binding read.

Repository performance history already shows per-opcode tracing can be expensive; preserve that discipline.

---

# 58. Disallow runtime module/package attribute attachment

Besides removing `Object::Module` from native attribute primitives, ensure compiler attribute expansion cannot synthesize:

```text
module.__attach(...)
```

for file-header metadata.

Search:

```bash
rg '__attach|freezeAttributes|attributes_frozen' phalcom-core
```

Module/package targets must not enter existing class/method runtime attachment paths.

A future compile-time module/package expander will be a compiler feature, not retrofitted runtime attachment.

---

# 59. Future dynamic/lazy loading seam

Do not implement now.

Preserve a clean future API boundary such as:

```text
ModuleLoader / ProgramExtension
```

that would:

1. resolve through an explicit project universe;
2. compile/link a new closed subgraph;
3. validate its runtime DAG against already initialized modules;
4. materialize/initialize it explicitly.

It must not make local-scope `import` suddenly legal or resurrect path-string source imports.

Ordinary static import semantics remain unchanged.

---

# 60. Future parallel initialization seam

Because v1 specifies no semantic sibling order, later runtime can initialize independent DAG branches concurrently.

To preserve that option now:

- do not document deterministic lexical/module-name tie-break as semantic;
- do not expose “current sibling init position” reflection;
- module completion must establish a synchronization boundary before dependents start;
- mutable global writes performed by initializer must be visible to dependents after state becomes `Initialized`.

V1 serial execution naturally satisfies memory publication; future parallel implementation must make it explicit.

---

# 61. TDD implementation sequence

## Task 1 — Package kernel class

Files:

- `phalcom-core/src/universe/core_classes.rs`
- `phalcom-core/src/universe/primitives.rs`
- `phalcom-core/src/universe/invariants.rs`
- `phalcom-core/src/value/mod.rs`
- invariant tests.

Red test:

```text
package runtime object class == Package
Package.superclass == Module
```

## Task 2 — ModuleObject shape

Files:

- `phalcom-core/src/heap/module.rs`
- `phalcom-core/src/heap/mod.rs`
- tracing/accessors as necessary.

Add semantic id/kind/export/linkage/metadata; remove module runtime attribute store.

## Task 3 — Registry

Files:

- create `phalcom-core/src/modules/registry.rs`
- modify `phalcom-core/src/modules/mod.rs`
- `phalcom-core/src/vm/mod.rs`
- `phalcom-core/src/universe/mod.rs`
- `phalcom-core/src/vm/gc.rs`.

Move module roots out of Universe.

## Task 4 — Program materialization

Files:

- create `phalcom-core/src/modules/materialize.rs`
- module tests.

Allocate all module objects/layouts/linked refs before execution.

## Task 5 — `GetLinked`

Files:

- `phalcom-core/src/bytecode.rs`
- `phalcom-core/src/vm/dispatch.rs`
- compiler/disasm tests.

Complete Part II's opcode migration.

## Task 6 — export dispatch

Files:

- `phalcom-core/src/primitive/module.rs`
- `phalcom-core/src/vm/dispatch.rs`
- possibly `phalcom-core/src/vm/send.rs`.

Tests for public/private exports and collision priority.

## Task 7 — attribute restriction

Files:

- `phalcom-core/src/primitive/attribute.rs`
- module/class/method attribute tests.

Module target must be rejected by runtime `__attach`.

## Task 8 — DAG initializer

Files:

- create `phalcom-core/src/modules/initialize.rs`
- `registry.rs`
- runtime errors.

Tests linear/diamond/failure/no partial.

## Task 9 — interpreter/CLI

Files:

- `phalcom-core/src/interpret.rs`
- `phalcom-core/bin/phalcom/cli.rs`
- binary/integration tests.

Replace single-source “main” creation with `EntrySelection` + `CompiledProgram`.

## Task 10 — delete U15 loader

Delete:

- `resolve_import_path`;
- `append_ph_extension_if_missing` if unused;
- `VM::import_module`;
- `Universe::module_registry`;
- `Bytecode::Import`;
- old runtime import docs/comments;
- source-position import assumptions.

## Task 11 — execution modes

Tests:

- project run;
- package run;
- file run;
- explicit package.ph;
- standalone package;
- standalone module;
- inline source restrictions.

## Task 12 — perf/GC regression

Run GC stress, module stress, VM benchmarks.

---

# 62. Verification commands

Core:

```bash
cargo fmt --all -- --check
cargo clippy -p phalcom-core --all-targets -- -D warnings
cargo test -p phalcom-core --test integration
cargo test -p phalcom-core --test lang
cargo test -p phalcom-core --test invariants
cargo test --workspace
```

Targeted source audit:

```bash
rg 'Bytecode::Import|resolve_import_path|VM::import_module|module_registry' \
  phalcom-core phalcom-lsp phalcom-ast phalcom-modules
```

No live runtime-loader occurrence should remain.

Attribute audit:

```bash
rg 'Object::Module.*attach|attributes_frozen|__attach' phalcom-core/src
```

Verify module/package runtime targets are absent.

GC stress on module-heavy fixtures:

```bash
PHALCOM_GC_STRESS=1 cargo test -p phalcom-core --test integration
```

Use the repository's existing performance harness for before/after whole-process startup measurements.

---

# 63. Recommended module-system integration tests

Add fixtures under the existing integration test convention, for example:

```text
phalcom-core/tests/fixtures/modules_v1/
├── project_basic/
├── package_facade/
├── private_paths/
├── live_import/
├── runtime_cycle/
├── semantic_cycle/
├── no_ancestor_init/
├── reexport_chain/
├── duplicate_project_instance/
└── standalone_package/
```

Each fixture should include expected stdout/diagnostic snapshots.

Do not rely only on unit tests of graph structures; end-to-end tests must prove parser → resolver → linker → compiler → runtime behavior.

---

# 64. Compatibility-sensitive invariants

These require explicit ADR/PDR documentation because changing them after package ecosystem growth is expensive:

```text
R1. ModuleId is semantic, not physical.
R2. One ModuleId maps to one runtime Module/Package object.
R3. Entry execution does not create another identity.
R4. Public module member namespace is exports only.
R5. Export names take precedence over Module instance protocol names.
R6. Selective imports are live immutable references.
R7. Module path visibility is separate from binding export.
R8. Child paths are project-private by default.
R9. Static imports are declarations, not source-position effects.
R10. Package containment does not initialize ancestors.
R11. Runtime dependency cycles are statically forbidden.
R12. Semantic/interface cycles are allowed and validated by their own systems.
R13. Initialization sibling order is not language semantics.
R14. Static modules initialize at most once per runtime program/session.
R15. Initialization failure is sticky.
R16. Module/package attributes are metadata-only in v1.
```

---

# 65. Final removal checklist

Before merging the final module-system implementation:

- [ ] no physical import string grammar;
- [ ] no runtime import opcode;
- [ ] no VM filesystem import resolver;
- [ ] no runtime compile-on-import;
- [ ] no early registry insertion for cycle tolerance;
- [ ] no partially visible modules;
- [ ] no ancestor package execution rule;
- [ ] no URI/path semantic ModuleId;
- [ ] no external deep import of unexposed dependency paths;
- [ ] no module globals exposed merely through `doesNotUnderstand`;
- [ ] no module runtime attribute attachment;
- [ ] no source-position import scope;
- [ ] no duplicate entry-module identity;
- [ ] runtime graph cycle checked before execution;
- [ ] semantic SCC tests accepted;
- [ ] live re-export/import binding test passes;
- [ ] project/package/file execution identities pass;
- [ ] LSP/compiler use shared resolver semantics;
- [ ] GC roots include runtime module registry;
- [ ] startup benchmark shows no regression from path/loader machinery because that machinery is gone.

---

# 66. Deferred features

Explicitly defer:

- dynamic module loading;
- lazy module loading;
- reload;
- runtime import declarations;
- module initialization parallelism;
- compile-time module/package attribute expansion;
- runtime module/package decorators;
- namespace packages;
- arbitrary user import finders/search paths;
- workspaces;
- multiple manifest binaries;
- package-manager registry acquisition;
- conditional/platform module edges unless separately ratified;
- compiled artifact import syntax.

The architecture is intentionally prepared for some of these, but none should weaken the static v1 semantics.

---

# 67. End-state architecture

After all three parts, the module system should read conceptually as:

```text
              project.toml
                   │
                   ▼
          Resolved ProjectUniverse
                   │
          logical ownership/roots
                   ▼
             SourceProvider
                   │
                   ▼
                ModuleId
                   │
        ┌──────────┴──────────┐
        ▼                     ▼
 ModuleInterface        source/artifact
        │
        ▼
    static linker
        │
        ├── semantic graph ── SCCs allowed
        │
        └── runtime graph  ── DAG required
                   │
                   ▼
            CompiledProgram
                   │
                   ▼
         runtime materializer
                   │
        one ModuleId → one ObjRef
                   │
                   ▼
       dependency-order initializer
                   │
                   ▼
          canonical entry module
```

The runtime is no longer a module **loader** in the Python sense. It is the executor of a statically resolved, linked, cycle-validated program image.

That is the intended first-version Phalcom architecture: explicit source ergonomics, deterministic ownership, private package topology, strong static tooling, live binding identity, and minimal runtime burden.
