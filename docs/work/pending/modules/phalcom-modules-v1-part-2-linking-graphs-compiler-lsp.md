# Phalcom Modules v1 Implementation Specification — Part II
## Static linking, dependency graphs, semantic cycles, live binding references, compiler integration, and LSP integration

**Status:** Implementation specification
**Target:** Phalcom first-version static module/package/project system
**Repository:** `aureat/phalcom-lang`
**Repository snapshot inspected:** `ed841918546610752ec0b1d3f7b1ffa6b2056006` (`main`)
**Depends on:** Part I — syntax/projects/resolution/interfaces
**Followed by:** Part III — runtime materialization/initialization/execution

---

# 1. Purpose

Part I makes modules statically identifiable and resolvable. Part II turns that resolved source universe into a linked semantic program.

This part replaces the current model:

```text
parse one file
    ↓
compile Import bytecode
    ↓
runtime reaches import instruction
    ↓
resolve filesystem path
    ↓
compile target
    ↓
execute target
```

with:

```text
resolve complete reachable module graph
    ↓
build interfaces
    ↓
link symbols/imports/re-exports
    ↓
validate semantic graph
    ↓
validate runtime dependency DAG
    ↓
compile already-linked modules
    ↓
hand a closed ProgramImage to the runtime
```

The runtime in Part III receives no unresolved source import string.

---

# 2. Ratified cycle model

Phalcom must distinguish two fundamentally different forms of dependency.

## 2.1 Semantic/interface dependency

A semantic dependency means:

> To understand/check declaration A, the compiler needs the semantic identity or interface of declaration/module B.

Examples:

- type references;
- mutually referring API signatures;
- protocols that mention one another;
- generic constraints;
- callback signatures;
- recursive ADTs;
- mutually recursive type declarations;
- cross-module declaration references;
- future type-level computation that remains compile-time and non-effectful.

These dependencies may form cycles.

A cycle means the declarations must be solved/checked as a strongly connected component. It does **not** mean any runtime value must be produced first.

## 2.2 Runtime module dependency

A runtime module dependency means:

> This module has a runtime import/re-export binding whose target module must be initialized before the importing module's ordinary initialization may execute.

In v1, ordinary value-level:

```phalcom
import M
from M import x
export x from M
```

creates a runtime module dependency unless the semantic/type subsystem proves the dependency is interface-only and erases it from the runtime binding set.

The runtime dependency graph must be acyclic.

## 2.3 Why this is not contradictory

A semantic cycle:

```text
Person interface → Company interface
       ↑                 │
       └─────────────────┘
```

can be resolved by creating declaration identities first and solving/checking references as one SCC.

A runtime initialization cycle:

```text
A initializer → B initialized state
      ↑               │
      └───────────────┘
```

has no valid first execution and would require partial modules, temporal dead zones, arbitrary SCC ordering, or runtime cycle semantics. Those are explicitly rejected.

Normative rule:

> Cycles are allowed where the compiler is solving relationships; cycles are rejected where the runtime would need to invent an execution order.

---

# 3. No “partial module” fallback

The old `modules-next.md` model used early registry insertion and partially initialized `ModuleObject`s to terminate circular imports. That behavior must be removed.

The compiler/linker must guarantee before runtime:

```text
runtime dependency graph is a DAG
```

Part III may still allocate every module record before initialization, but allocation is internal preparation. User execution cannot observe an uninitialized dependency.

There is no normal-language:

```text
UninitializedModuleBindingError
```

for static imports.

If a linked-read runtime invariant sees an uninitialized target, that indicates a compiler/linker/runtime bug, not a user-visible circular-import feature.

---

# 4. Graph model

Extend `phalcom-modules/src/graph.rs` with explicit graph types.

Do not use one `ImportEdge` structure for everything.

Recommended model:

```rust
pub struct ModuleGraphs {
    pub references: ReferenceGraph,
    pub semantics: SemanticGraph,
    pub runtime: RuntimeDependencyGraph,
}
```

## 4.1 Reference graph

Records statically resolved source references.

```rust
pub struct ReferenceEdge {
    pub from: ModuleId,
    pub to: ModuleId,
    pub kind: ReferenceKind,
    pub range: SourceRange,
}
```

Kinds:

```rust
pub enum ReferenceKind {
    WholeModuleImport,
    SelectiveImport,
    ReExport,
    InterfaceOnly,
}
```

The reference graph may contain cycles.

Its purpose:

- reachability;
- diagnostics;
- LSP dependency navigation;
- invalidation;
- deriving semantic/runtime edges.

## 4.2 Semantic graph

```rust
pub struct SemanticEdge {
    pub from: SemanticNodeId,
    pub to: SemanticNodeId,
    pub kind: SemanticEdgeKind,
    pub range: SourceRange,
}
```

At minimum module-level semantic edges are required now. Declaration-level nodes can be added progressively as typing/protocol/ADT systems land.

Kinds should leave room for:

```rust
pub enum SemanticEdgeKind {
    ModuleInterface,
    TypeReference,
    Superclass,
    ProtocolReference,
    ConstraintReference,
    CallbackSignature,
    AdtReference,
}
```

Unknown future kinds do not require changing cycle infrastructure.

## 4.3 Runtime dependency graph

```rust
pub struct RuntimeDependencyEdge {
    pub importer: ModuleId,
    pub dependency: ModuleId,
    pub range: SourceRange,
    pub reason: RuntimeDependencyReason,
}
```

`importer → dependency` means:

```text
dependency must be Initialized before importer initialization starts
```

Reasons:

```rust
pub enum RuntimeDependencyReason {
    WholeModuleImport,
    SelectiveValueImport,
    ReExport,
    RuntimeDeclarationReference,
}
```

The runtime graph must be a DAG.

---

# 5. Interface-only edges and future typing

The module architecture must support cyclic typing/declaration graphs **without requiring the runtime module system to become cyclic**.

Do not prematurely freeze a user-facing `import type` syntax if the typing/protocol/ADT surface is not yet ratified.

Instead, implement an internal phase classification now:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyPhase {
    InterfaceOnly,
    Runtime,
}
```

Every linked import/reference records the strongest required phase.

Rules:

- runtime use upgrades the edge to `Runtime`;
- pure semantic/type/declaration use may remain `InterfaceOnly`;
- direct re-export is runtime because it contributes a runtime public binding;
- whole-module import is runtime because it creates a runtime `Module`/`Package` binding;
- current cross-module superclass resolution is a semantic reference and, until class declaration materialization is separated completely, may conservatively be runtime as well;
- future type signatures/protocol/constraint references should contribute interface-only edges when no value-level binding is required.

When optional typing lands, the type system plugs into this classification rather than inventing a second module resolver.

This is the hook that permits:

```text
A type interface ↔ B type interface
```

while still rejecting:

```text
A runtime import ↔ B runtime import
```

when both edges require eager initialized runtime modules.

---

# 6. SCC processing

Implement deterministic SCC computation in `phalcom-modules/src/graph.rs`.

Avoid a new graph dependency unless there is a compelling benchmarked reason; Tarjan's algorithm is small and O(V+E).

API:

```rust
pub fn strongly_connected_components<N>(
    nodes: impl IntoIterator<Item = N>,
    successors: impl Fn(&N) -> &[N],
) -> Vec<Vec<N>>
where
    N: Clone + Eq + Hash + Ord;
```

In practice use dedicated graph methods rather than an overly abstract API if that reduces allocation.

Requirements:

- O(V+E);
- stable/deterministic component output for diagnostics;
- no recursion deep enough to overflow Rust stack on a large generated graph; prefer iterative Tarjan/Kosaraju if practical;
- component members sorted by `ModuleId` only for deterministic output, not semantic ordering.

Semantic SCCs are legal by default. Construct-specific validators inspect them.

Runtime SCCs:

- singleton with no self-edge: legal;
- singleton with self-edge: error;
- more than one node: error.

---

# 7. Construct-specific semantic-cycle validation

“Semantic cycles allowed” does not mean every recursive relation is valid.

Examples:

## 7.1 Cyclic type references

Potentially legal:

```text
A field type → B
B callback signature → A
```

The type checker resolves/checks the SCC as a unit.

## 7.2 Recursive ADT

Legal if the future type system's own recursive-type rules accept it.

## 7.3 Protocol/constraint recursion

The protocol/constraint solver decides coherence/productivity/satisfiability.

## 7.4 Inheritance cycle

Always invalid:

```phalcom
class A is b.B { }
class B is a.A { }
```

Report:

```text
CyclicInheritanceError:
  A → B → A
```

The module system should not misreport this as a generic module cycle. The module/reference SCC may be legal; the inheritance relation is not.

This separation is important for diagnostics and future type-system growth.

---

# 8. Runtime dependency-cycle diagnostics

Before any bytecode runs, call:

```rust
RuntimeDependencyGraph::validate_acyclic()
```

On failure emit a compile/link diagnostic with:

- minimal or clear representative cycle;
- every module logical name;
- source span of each edge;
- reason for each edge.

Example:

```text
CyclicModuleInitializationError:
  runtime module initialization contains a cycle:

    app.a
      -- selective import `B` --> app.b
      -- whole-module import --> app.a

  Runtime module dependencies must be acyclic.
  Semantic/type-only declaration cycles are allowed; initialized runtime values
  cannot depend cyclically.
```

Do not mention partial-module workarounds.

---

# 9. Runtime topological order

Implement Kahn topological sorting in `graph.rs`:

```rust
pub fn initialization_order(&self) -> Result<Vec<ModuleId>, ModuleGraphError>;
```

Semantics:

```text
A imports runtime B
=> B appears before A
```

For independent modules:

```text
B and C both required by A
but B and C do not depend on each other
```

the language defines no relative order between B and C.

Implementation should nevertheless use a deterministic stable tie-break (e.g. `BTreeSet<ModuleId>`) for reproducibility.

Normative distinction:

```text
implementation order is deterministic
≠
program may rely on sibling ordering
```

This leaves future room for parallel initialization without changing correct program behavior.

---

# 10. Symbol identity and interface linking

Add `phalcom-modules/src/linker.rs` in Part II.

Extend crate layout:

```text
phalcom-modules/src/
    linker.rs
    graph.rs
    interface.rs
```

## 10.1 `SymbolId`

Use semantic symbol identity independent of runtime interner symbols:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolId {
    pub module: ModuleId,
    pub name: Box<str>,
}
```

If a later declaration-id layer distinguishes same-name namespaces, evolve `name` into `DeclarationId`. Module globals currently have one name namespace, so this is sufficient for v1.

## 10.2 Export target

Before link:

```rust
pub enum UnlinkedExportTarget {
    Local(String),
    ReExport {
        module: ModuleId,
        remote: String,
    },
}
```

After link:

```rust
pub struct LinkedExport {
    pub public_name: Box<str>,
    pub symbol: SymbolId,
    pub range: SourceRange,
}
```

A re-export resolves to the **original canonical `SymbolId`**, not a new copied symbol identity.

Aliases affect only public/local names.

This gives:

```text
point.Point SymbolId
      ↑
package export Point
      ↑
consumer selective import Point
```

one semantic symbol.

---

# 11. Live selective imports without heap-cell allocation

The original design's “BindingCell” idea is correct, but the current runtime already provides a cheaper physical representation.

`phalcom-core/src/heap/module.rs::ModuleObject` stores globals in stable append-only slots:

```rust
globals: Vec<Value>
name_to_slot: HashMap<Symbol, usize>
```

A runtime binding reference can therefore be:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingRef {
    pub module: ObjRef,
    pub slot: u16,
}
```

Semantics:

```text
BindingRef ≡ immutable reference to one mutable module slot
```

This is the runtime representation of a logical binding cell.

Benefits:

- no heap object per exported binding;
- no extra tracing object;
- no `Rc`/`RefCell`;
- two machine-sized fields;
- re-export copies the reference;
- selective import copies the reference;
- source rebinding updates the original slot;
- every importer observes the new value;
- symbol identity is preserved separately by linker metadata.

Do **not** implement exports by copying current `Value`s.

---

# 12. Link plan structures

Part II should produce a VM-independent linked program plan.

```rust
pub struct LinkedProgram {
    pub universe: Arc<ProjectUniverse>,
    pub modules: BTreeMap<ModuleId, LinkedModule>,
    pub graphs: ModuleGraphs,
    pub entry: ModuleId,
}
```

`LinkedModule`:

```rust
pub struct LinkedModule {
    pub interface: LinkedModuleInterface,
    pub bindings: ModuleBindingLayout,
    pub linked_reads: Vec<LinkedReadSpec>,
    pub runtime_dependencies: Vec<ModuleId>,
}
```

`ModuleBindingLayout`:

```rust
pub struct ModuleBindingLayout {
    pub local_globals: BTreeMap<Box<str>, GlobalBindingId>,
    pub imports: BTreeMap<Box<str>, ImportBindingId>,
}
```

`LinkedReadSpec` is symbolic:

```rust
pub enum LinkedReadSpec {
    Module(ModuleId),
    Binding(SymbolId),
}
```

The runtime materializer in Part III converts these to `ObjRef`/`BindingRef`.

---

# 13. Import binding semantics

Local imports are immutable namespace bindings.

Compiler binding resolution must distinguish:

```rust
pub enum ModuleBinding {
    Global {
        name: Symbol,
        mutable: bool,
    },
    Import {
        name: Symbol,
        linked_read: u16,
    },
}
```

There is no assignment opcode/path for `Import`.

Attempting:

```phalcom
from .settings import mode
mode = "other"
```

is a compile-time immutable-binding error.

If `settings` later assigns its own mutable `mode` global, the import read still returns the updated source slot.

---

# 14. Add `Bytecode::GetLinked`

The current `Bytecode::Import(u16)` must be deleted. Static imports do not execute.

Add:

```rust
Bytecode::GetLinked(u16)
```

Meaning:

> Push the value represented by the current module's already-materialized linked-read entry.

Runtime linked-read representation in Part III:

```rust
pub enum RuntimeLinkedRead {
    Module(ObjRef),
    Binding(BindingRef),
}
```

`GetLinked`:

- `Module(id)` => push `Value::Obj(id)`;
- `Binding(ref)` => read source module slot and push current value.

There is no `SetLinked`.

Update:

- `BYTECODE_NAMES`;
- `Bytecode::VARIANTS`;
- `Bytecode::index`;
- disassembler formatting;
- opcode histogram indices/tests.

The bytecode carries no module path string and performs no filesystem or compilation work.

---

# 15. Remove compiler-emitted import execution

File: `phalcom-core/src/compiler/lib/mod.rs`

Delete/replace:

- `Compiler::compile_import`
- `Statement::Import` compile arm
- old `known_globals` import collection based on body statements
- old `import_bindings` assumptions that imports appear as runtime statements.

Instead, `Compiler::new` receives a linked module context:

```rust
pub(crate) struct Compiler<'vm, 'link> {
    pub(crate) vm: &'vm mut VM,
    module: ObjRef,
    linkage: &'link CompiledModuleLinkage,
    ...
}
```

or, before runtime `ObjRef` allocation is available, split compilation so VM-independent linking precedes VM-specific constant materialization.

Prefer a narrow context:

```rust
pub struct CompileBindings {
    pub globals: HashMap<Symbol, GlobalBindingInfo>,
    pub imports: HashMap<Symbol, LinkedImportInfo>,
}
```

Binding resolution:

```text
local/upvalue
    ↓
module import?
    → GetLinked
    ↓
module global?
    → GetGlobal
    ↓
core
```

Exact order depends on existing local/global resolution rules, but import/global collisions are rejected during interface construction so there is no ambiguous fallback.

---

# 16. Predeclare imported names before body compilation

Because imports have module-wide scope, the compiler must seed import bindings from `Program.preamble` / linker results before compiling `Program.statements`.

No bytecode is emitted for this seed.

Current `Compiler::known_globals` should become a more explicit module namespace table rather than a `HashSet<String>` plus separate import map.

Recommended:

```rust
pub enum TopLevelBindingKind {
    MutableGlobal,
    ImmutableGlobal,
    Class,
    Import(LinkedReadId),
}
```

```rust
pub struct TopLevelBindingInfo {
    pub kind: TopLevelBindingKind,
    pub declared_at: SourceRange,
}
```

Use one table for collision diagnostics:

```rust
HashMap<Symbol, TopLevelBindingInfo>
```

This can replace overlapping logic spread across:

- `global_bindings`;
- `known_globals`;
- `import_bindings`.

Preserve existing specialized class diagnostics where required, but source truth should be one binding table.

---

# 17. Qualified static references

The current `Compiler::resolve_superclass_key` in `phalcom-core/src/compiler/lib/mod.rs` resolves only:

```text
current module
then core
```

and the current LSP `build_module_surface` in `phalcom-lsp/src/semantic/surface.rs` assumes superclass `ClassId` is in the same module.

The target spec explicitly permits:

```phalcom
import .base as base

class Circle is base.Shape {
}
```

Implement a static qualified reference AST.

In `phalcom-ast/src/ast.rs` replace/extend `SuperclassRef` with:

```rust
pub struct StaticSymbolRef {
    pub root: String,
    pub root_range: SourceRange,
    pub members: Vec<PathSegment>,
    pub range: SourceRange,
}
```

For superclass position v1, require exactly:

```text
bare identifier
or
known module import alias + exported declaration
```

Do not accept arbitrary runtime expressions.

Linker API:

```rust
pub fn resolve_static_symbol(
    &self,
    module: &ModuleId,
    reference: &StaticSymbolRef,
) -> Result<SymbolId, LinkError>;
```

Then class semantic resolution consumes `SymbolId`, not a runtime module send.

This same abstraction should later serve type references.

---

# 18. Class identity migration

Current runtime class identity:

```rust
pub struct ClassKey {
    pub module: ObjRef,
    pub name: Symbol,
}
```

is runtime-correct but unavailable to VM-free semantic analysis.

Introduce a compile/link identity:

```rust
pub struct SemanticClassId {
    pub symbol: SymbolId,
}
```

LSP `ClassId` should become an alias/wrapper around this semantic identity instead of URI-backed module + String.

Runtime materialization maps:

```text
SemanticClassId
    ↓
ClassId/ObjRef
```

in Part III.

Do not force `ObjRef` into linker/LSP layers.

---

# 19. Declaration materialization seam

To support interface discovery before ordinary initialization, compiled output must distinguish declaration metadata from initializer bytecode.

Introduce:

```rust
pub struct ModuleArtifact {
    pub id: ModuleId,
    pub declarations: Vec<RuntimeDeclarationBlueprint>,
    pub initializer: ObjRef, // or VM-independent compiled closure handle at the appropriate layer
    pub linkage: CompiledModuleLinkage,
}
```

At minimum classes should have a blueprint:

```rust
pub struct ClassBlueprint {
    pub symbol: SymbolId,
    pub superclass: Option<SymbolId>,
    pub field_layout: ...,
    pub methods: ...,
}
```

Part III may materialize class objects before ordinary module initialization.

This does **not** expose a partial module. It is internal declaration instantiation after all interfaces are linked.

Any initializer expression with arbitrary user computation remains in the module initializer and therefore respects runtime dependency ordering.

---

# 20. Compile pipeline

Refactor current `VM::compile_closure_as` single-file path into a program compiler pipeline.

Recommended high-level API:

```rust
pub struct ProgramCompiler<'a, P: SourceProvider> {
    resolver: ModuleResolver<'a, P>,
    ...
}

impl ProgramCompiler<'_, _> {
    pub fn compile_entry(
        &mut self,
        entry: EntrySelection,
    ) -> Result<CompiledProgram, CompileProgramError>;
}
```

Phases:

```text
1. Determine owning project/standalone universe
2. Resolve entry ModuleId
3. Parse entry source
4. Traverse static dependency preambles
5. Parse each reachable target once
6. Build UnlinkedModuleInterfaces
7. Resolve paths
8. Build reference graph
9. Link imports/re-exports to SymbolIds
10. Build semantic graph
11. Process semantic SCCs / construct validation
12. Derive runtime dependency graph
13. Reject runtime SCCs
14. Topologically order runtime modules
15. Compile each module using linked bindings
16. Produce CompiledProgram
```

No step 15 may discover a new import target. If it does, the interface/link phases are incomplete.

---

# 21. Direct re-export linking

Given:

```phalcom
export Point as P from .point
```

Part I parsed this as a preamble re-export and gave it import-equivalent local binding semantics.

Link steps:

1. resolve `.point` -> `ModuleId`;
2. require target module path visibility as appropriate;
3. inspect target `ModuleInterface`;
4. require exported `Point`;
5. resolve canonical target `SymbolId`;
6. create immutable local import binding `Point` (or alias if syntax defines one);
7. create current module export `P -> canonical SymbolId`;
8. add reference/runtime dependency edge;
9. emit no runtime re-export bytecode.

At runtime, Part III maps both import/export to the same `BindingRef`.

---

# 22. Package façade efficiency

This design removes the eager-loading pathology from the original spec.

Given root `package.ph`:

```phalcom
export Point from .point
export Vector from .vector
```

importing:

```phalcom
import geometry.point
```

does not even compile/initialize `geometry/package.ph` merely because it is an ancestor, except to the limited extent the resolver reads the static package exposure surface required for external path validation.

Importing:

```phalcom
import geometry
```

does link/initialize the package's actual runtime dependencies (`point`, `vector`) because the façade explicitly re-exports them.

That is intentional demand caused by requesting the façade itself.

Static path-surface discovery must not execute package code.

---

# 23. Module public member namespace

Part II determines public members from `LinkedModuleInterface.exports`.

The runtime must never use the current `ModuleObject::globals` table as the public member table.

Prepare:

```rust
pub struct LinkedModuleInterface {
    pub exports: BTreeMap<Box<str>, LinkedExport>,
    ...
}
```

Part III materializes a runtime export table containing only those names.

This is how:

```phalcom
module.privateGlobal
```

becomes inaccessible even though the module object internally owns that global slot.

---

# 24. Static qualified module-access optimization

For an immutable known module alias:

```phalcom
import .models as models

const x = models.User
```

the compiler/LSP knows:

```text
models -> ModuleId
User   -> exported SymbolId
```

Do not require the hot runtime path to hash the export name and enter `doesNotUnderstand`.

During expression compilation, when a getter/member send receiver is provably a module import binding and the selected member is statically exported, the compiler may allocate another `LinkedReadSpec::Binding(SymbolId)` and emit:

```rust
Bytecode::GetLinked(index)
```

directly.

Semantics remain identical to module export access.

For:

- a dynamically computed `Module` value;
- an unknown selector shape;
- reflective sends;

fall back to normal module-object export dispatch in Part III.

This optimization is safe because import bindings are immutable and module export interfaces are closed for the program image.

---

# 25. Runtime dependency derivation

In v1, derive runtime dependencies conservatively and explicitly.

A dependency is runtime when any of these is true:

1. whole-module import creates a runtime module value;
2. selective imported binding is used in value/runtime namespace;
3. a selective binding is re-exported;
4. direct re-export exists;
5. runtime declaration materialization requires the target runtime declaration object;
6. future semantic analysis explicitly upgrades an interface-only edge.

A dependency may remain interface-only only when the semantic subsystem can prove no runtime binding/value is required.

Do **not** attempt speculative lazy import elimination here. Dynamic/lazy loading is deferred.

This conservative rule intentionally favors:

```text
sound static DAG
```

over complex effect inference.

---

# 26. Cyclic semantic examples

The architecture must pass tests built from synthetic interface edges even before all future type constructs exist.

## 26.1 Mutually referring signatures

```text
module person:
  Person.employer -> Company

module company:
  Company.employees -> Sequence<Person>
```

Semantic graph:

```text
person ↔ company
```

Runtime graph:

```text
(no edge if references are interface-only)
```

Legal.

## 26.2 Callback signatures

```text
A accepts callback(B) -> A
B accepts callback(A) -> B
```

Legal interface SCC.

## 26.3 Recursive ADT across modules

If future type rules permit:

```text
A = End | Next(B)
B = End | Next(A)
```

the type checker receives one SCC.

## 26.4 Cyclic inheritance

```text
A is B
B is A
```

semantic SCC exists, but inheritance validator rejects.

## 26.5 Runtime value cycle

```text
A runtime-imports B
B runtime-imports A
```

runtime SCC rejected before bytecode execution.

---

# 27. LSP module graph replacement

Current file:

`phalcom-lsp/src/semantic/module_graph.rs`

currently:

- stores file-relative `path: String`;
- resolves imports with `Url` + filesystem canonicalization;
- maintains its own candidate resolver.

Retain the **incremental reverse-edge/candidate-index idea**, but replace target semantics.

Recommended LSP graph:

```rust
pub struct LspModuleGraph {
    forward: BTreeMap<ModuleId, Vec<ReferenceEdge>>,
    reverse: BTreeMap<ModuleId, BTreeSet<ModuleId>>,
    unresolved: BTreeMap<UnresolvedLogicalTarget, BTreeSet<ModuleId>>,
}
```

Resolution comes from `phalcom-modules::ModuleResolver`.

When a source file appears/disappears:

1. map URI -> owning project + possible `ModuleId`;
2. update source-provider/project index;
3. ask resolver which retained unresolved logical candidates may now change;
4. update only those importers;
5. invalidate transitive semantic dependents through reverse edges.

Do not canonicalize each import independently inside the LSP.

---

# 28. LSP module surfaces

Current file:

`phalcom-lsp/src/semantic/surface.rs`

Extend `ModuleSurface` beyond classes:

```rust
pub struct ModuleSurface {
    pub module: ModuleId,
    pub exports: BTreeMap<String, ExportSurface>,
    pub imports: BTreeMap<String, ImportSurface>,
    pub classes: BTreeMap<ClassId, ClassSurface>,
    pub metadata: ModuleMetadata,
}
```

The LSP can then support:

- import completion from actual exports;
- private-binding diagnostics;
- package-path exposure diagnostics;
- re-export go-to-definition to original declaration;
- rename across re-export/import aliases;
- module alias completion;
- public API documentation;
- path completion restricted to exposed dependency paths.

---

# 29. LSP superclass resolution

Current `build_module_surface` constructs:

```text
ClassId(module.clone(), parent.name)
```

for every superclass, which hardcodes same-module resolution.

Replace this with unresolved static symbol surface:

```rust
pub struct UnresolvedClassSurface {
    pub superclass: Option<StaticSymbolRef>,
    ...
}
```

Then semantic linking resolves:

```phalcom
class Circle is base.Shape
```

through the same import/interface tables as the compiler.

Compiler and LSP must not implement separate superclass path logic.

---

# 30. LSP invalidation by edge kind

Different dependency changes require different invalidation.

Maintain reverse indexes at least by:

```rust
pub enum ReverseDependencyKind {
    Interface,
    Runtime,
    ReExport,
}
```

Examples:

- body-only change with unchanged interface: do not invalidate unrelated importers' completion/type surfaces;
- export change: invalidate selective importers/re-exporters;
- `expose` change: invalidate external path-resolution candidates;
- metadata-only change: invalidate docs/reflection metadata consumers, not body inference;
- runtime initializer body change: compiler runtime artifact invalidation, but LSP interface may remain reusable.

This follows the performance direction already present in the LSP's incremental module graph.

---

# 31. Compiler error migration

Current `CompilerError::ImportNotAtTopLevel` becomes obsolete for parsed files because the parser structurally owns preamble legality.

Add/route link diagnostics through a top-level compile-program error:

```rust
pub enum CompileProgramError {
    Parse { module: ModuleId, error: SyntaxError },
    Project(ProjectError),
    Resolve(ModuleResolutionError),
    Link(LinkError),
    Semantic(SemanticError),
    RuntimeDependency(ModuleGraphError),
    Bytecode { module: ModuleId, error: CompilerError },
}
```

Do not force module-system diagnostics into `CompilerError::Message(String)`.

The CLI/LSP can render structured source ranges from the originating module.

---

# 32. Compiler/global lookup changes

The current compiler's bare global resolution ultimately relies on runtime module/global tables plus core.

For linked imports, resolve at compile time.

Recommended compile binding lookup:

```text
lexical local?
    yes → local/upvalue bytecode
no
module import binding?
    yes → GetLinked
no
module global declaration?
    yes → GetGlobal
no
core-known binding?
    yes → GetGlobal/core path or existing optimized core mechanism
no
undefined
```

Because preamble/import collisions are rejected, there is no ambiguous “import versus global” tie.

The LSP should mirror the same namespace order.

---

# 33. Core qualification

Keep existing implicit core fallback for ordinary bare globals.

Also reserve an explicit resolver root:

```phalcom
import core
```

or equivalent core module access once its public interface is modeled.

A local `System` binding can shadow bare `System`; explicit core qualification remains possible.

This should be implemented through the same `ModuleId`/interface path, not a filesystem exception.

---

# 34. Linking data ownership and allocation

The linker is not a hot runtime component. Optimize for correctness and reusable immutable results.

Recommended:

- `BTreeMap` where deterministic iteration is valuable for diagnostics/build artifacts;
- `HashMap` for caches/lookups where order is irrelevant;
- `Box<str>`/`Arc<str>` for retained names rather than repeated `String` clones;
- immutable `Arc<LinkedModuleInterface>` in caches;
- avoid cloning whole ASTs; retain `Arc<Program>` or source-index references;
- do not put `ObjRef` in `phalcom-modules`.

At the core boundary, convert semantic names into the existing `Interner::Symbol` once per module compilation.

---

# 35. `CompiledProgram`

Create a core-side artifact, preferably in a new subsystem:

```text
phalcom-core/src/modules/
├── mod.rs
├── compile.rs
├── artifact.rs
└── linkage.rs
```

Types:

```rust
pub struct CompiledProgram {
    pub project_universe: Arc<ProjectUniverse>,
    pub entry: ModuleId,
    pub modules: BTreeMap<ModuleId, CompiledModule>,
    pub initialization_order: Vec<ModuleId>,
}

pub struct CompiledModule {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub source: SourceLocation,
    pub interface: Arc<LinkedModuleInterface>,
    pub artifact: ModuleArtifact,
    pub linked_reads: Vec<LinkedReadSpec>,
}
```

Part III consumes this to create runtime module records.

The compiler must be able to compile all modules without executing any one of them.

---

# 36. Remove path strings from bytecode constants

After migration, no import target path belongs in a `Chunk` constant pool.

Search/verify:

```bash
rg 'Bytecode::Import|import_module|resolve_import_path' phalcom-core
```

Expected after Part II/III completion:

- no `Bytecode::Import`;
- no runtime module resolution from bytecode;
- no import path symbol constant;
- import/re-export targets live in `CompiledProgram` linkage.

This is a measurable simplification of the interpreter dispatch loop.

---

# 37. Re-export/live-binding invariant

For:

```phalcom
// settings.ph
let mode = "development"
export mode
```

```phalcom
// facade/package.ph
export mode from .settings
```

```phalcom
// app.ph
from facade import mode
```

link identity must be:

```text
settings::mode SymbolId
      ↓
settings global slot
      ↑
facade export BindingRef
      ↑
app linked read BindingRef
```

When `settings` writes:

```phalcom
mode = "production"
```

`app` subsequently reads `"production"`.

No module in the chain copies the `Value`.

---

# 38. Direct module object export

For:

```phalcom
import .point as point
export point
```

the exported value is the canonical target module object.

Link surface should preserve:

```text
local `point` -> LinkedReadSpec::Module(point ModuleId)
export `point` -> that same import binding
```

Part III materializes it as the target `ObjRef`.

The package does not synthesize a child-module object member merely because the child exists; this happens only due explicit import+export.

---

# 39. Testing strategy

Add a dedicated crate test suite:

```text
phalcom-modules/tests/
├── identity.rs
├── resolver.rs
├── interface.rs
├── linker.rs
├── graph.rs
└── fixtures/
```

Add core tests:

```text
phalcom-core/tests/modules_compile.rs
```

Add LSP tests:

```text
phalcom-lsp/tests/modules.rs
```

## 39.1 Graph tests

Required:

- empty graph;
- simple DAG;
- diamond DAG;
- runtime self-cycle rejected;
- runtime two-node cycle rejected;
- runtime three-node cycle rejected;
- semantic two-node SCC accepted;
- semantic SCC + cyclic inheritance rejected by inheritance validator;
- deterministic diagnostic cycle rendering.

## 39.2 Linking tests

- whole import resolves ModuleId;
- selective import resolves exported SymbolId;
- private binding rejected;
- missing export rejected;
- direct re-export canonicalizes original SymbolId;
- re-export alias;
- import collision;
- export collision;
- private path cannot be deep-imported externally;
- private path can be re-exported through façade;
- import order does not change linked plan.

## 39.3 Compiler tests

- no `Bytecode::Import` emitted;
- imported bare binding emits `GetLinked`;
- whole module import emits `GetLinked`;
- assignment to import rejected;
- class/import name collision preserved;
- qualified superclass resolves through linker;
- moving/reordering imports leaves disassembly of body semantically identical except source spans/constant ordering if unavoidable.

## 39.4 Live-binding tests

Runtime-facing tests can land fully in Part III, but Part II should at least assert linked exports/imports share canonical `SymbolId`.

## 39.5 LSP tests

- dependency alias completion;
- relative import go-to-definition;
- export-only completion;
- private binding diagnostic;
- private child path omitted from external completion;
- re-export definition jumps to origin;
- source URI rename inside same project updates location without changing semantic rules;
- same file under a different resolved project instance has different ModuleId.

---

# 40. Migration of existing LSP performance machinery

Do not throw away the current reverse/candidate-index architecture in `phalcom-lsp/src/semantic/module_graph.rs`.

Preserve the useful idea:

```text
changed provider
→ only candidate importers repaired
→ reverse dependents invalidated
```

Replace only the identity/resolution substrate.

Add targeted indexes:

```text
logical candidate path -> importers
exported symbol -> selective/re-export consumers
package exposure child -> external importers
```

This makes the new richer module system cheaper to update than a full workspace rescan.

---

# 41. Performance requirements

Part II must make the static design cheaper at runtime, not simply move work into an unbounded compiler pass.

Required:

1. Parse each reachable source at most once per compilation generation.
2. Build each interface once.
3. Resolve each logical import once and cache it.
4. SCC computation is O(V+E).
5. Runtime DAG validation is O(V+E).
6. Topological sorting is O(V+E) plus deterministic ready-set cost.
7. Re-exports preserve canonical symbol references; no copied export values.
8. Selective imports compile to indexed linked reads.
9. Static known module member getter may compile to direct linked read.
10. No filesystem/import path lookup occurs in VM dispatch.
11. No import-time compilation occurs after execution starts.
12. LSP reuses resolver/interface results and reverse edges.
13. Interface-only cycles do not force runtime module initialization.
14. No heap object is allocated solely to represent an export binding cell.

Benchmark targets should include:

```text
- 1 module / no imports
- 100-module linear graph
- 1000-module sparse graph
- wide package façade
- many selective imports from one module
- semantic SCC of 100 modules
```

Measure compiler/link wall-clock separately from execution.

---

# 42. Static safety invariants

Assert/document:

```text
L1. Every linked import target has a resolved ModuleId.
L2. Every selective runtime import resolves to an exported SymbolId.
L3. Every direct re-export resolves to an exported SymbolId.
L4. Import aliases never change target SymbolId/ModuleId.
L5. Re-export aliases never create a new declaration identity.
L6. Semantic SCCs may exist.
L7. Runtime dependency SCCs may not exist.
L8. No unresolved import reaches bytecode generation.
L9. No import declaration emits executable import bytecode.
L10. Every imported local binding is immutable.
L11. Every runtime selective import is represented as a reference, never a copied value.
L12. Package ancestry alone contributes no runtime dependency edge.
L13. Cross-project path accessibility has already been checked before linking symbols.
L14. Compiler and LSP resolve qualified static symbols through the same linked interfaces.
```

---

# 43. TDD implementation sequence

## Task 1 — graph primitives

Files:

- `phalcom-modules/src/graph.rs`
- `phalcom-modules/tests/graph.rs`

Write SCC/DAG tests first.

## Task 2 — linker and SymbolId

Files:

- create `phalcom-modules/src/linker.rs`
- extend `interface.rs`
- tests `linker.rs`

Start with import/export/re-export identity.

## Task 3 — runtime-vs-interface phase classification

Files:

- `interface.rs`
- `linker.rs`
- `graph.rs`

Tests must demonstrate a semantic cycle accepted while runtime cycle rejected.

## Task 4 — static qualified reference

Files:

- `phalcom-ast/src/ast.rs`
- `phalcom-ast/src/parser.rs`
- `phalcom-modules/src/linker.rs`
- AST/parser tests.

Then migrate superclass semantic resolution.

## Task 5 — core compile artifact subsystem

Files:

- create `phalcom-core/src/modules/mod.rs`
- create `phalcom-core/src/modules/artifact.rs`
- create `phalcom-core/src/modules/linkage.rs`
- create `phalcom-core/src/modules/compile.rs`
- modify `phalcom-core/src/lib.rs`

Produce `CompiledProgram` without changing runtime yet.

## Task 6 — compiler namespace binding refactor

Files:

- `phalcom-core/src/compiler/lib/mod.rs`
- `phalcom-core/src/compiler/lib/state.rs` if binding resolution belongs there
- `phalcom-core/src/compiler/lib/error.rs`

Unify top-level binding metadata and seed imports before body compile.

## Task 7 — bytecode change

Files:

- `phalcom-core/src/bytecode.rs`
- disassembler/opcode tests
- `phalcom-core/src/vm/dispatch.rs` stub/handler for `GetLinked` completed in Part III.

Delete compiler emission of `Import`.

## Task 8 — superclass resolution

Files:

- `phalcom-core/src/compiler/lib/mod.rs`
- `phalcom-core/src/compiler/lib/class_decl.rs`
- `phalcom-lsp/src/semantic/surface.rs`
- linker.

Remove same-module-only LSP assumption.

## Task 9 — LSP shared resolver migration

Files:

- `phalcom-lsp/Cargo.toml`
- `phalcom-lsp/src/semantic/ids.rs`
- `phalcom-lsp/src/semantic/module_graph.rs`
- `phalcom-lsp/src/semantic/engine.rs`
- `phalcom-lsp/src/semantic/snapshot.rs`
- `phalcom-lsp/src/semantic/mod.rs`
- `phalcom-lsp/src/backend.rs`
- `phalcom-lsp/src/completion.rs`
- `phalcom-lsp/src/inlay_hints.rs`
- tests.

Replace every `ModuleId::from_uri` with document->module mapping.

---

# 44. Verification commands

At milestone boundaries:

```bash
cargo fmt --all -- --check
cargo clippy -p phalcom-modules --all-targets -- -D warnings
cargo test -p phalcom-modules
cargo test -p phalcom-ast
cargo test -p phalcom-lsp
cargo test -p phalcom-core --test integration
cargo test -p phalcom-core --test lang
cargo test --workspace
```

Repository instructions also require maintaining relevant graph/documentation artifacts if `graphify` is available in the implementation environment.

Before claiming the removal complete:

```bash
rg 'Bytecode::Import|compile_import|resolve_import_path|import_module' \
  phalcom-core phalcom-lsp phalcom-ast
```

Any remaining occurrence must be documentation/migration text or intentionally transitional code called out in the implementation PR.

---

# 45. Completion criteria for Part II

Part II is complete when:

- the compiler receives a closed linked module graph;
- runtime and interface dependency edges are distinct;
- semantic/interface SCCs are supported;
- cyclic inheritance has its own semantic error;
- runtime dependency cycles are compile/link errors;
- initialization order is precomputed;
- selective imports link to canonical source symbols;
- re-exports preserve canonical symbol identity;
- no runtime import path appears in compiled bytecode;
- `Bytecode::Import` is removed/replaced by indexed linked reads;
- import bindings are module-wide and immutable;
- qualified superclass references resolve through imported module interfaces;
- LSP and compiler use the same `ModuleId` and resolver semantics;
- LSP no longer equates URI with semantic module identity;
- package ancestry creates no runtime edge;
- static import order does not affect linked semantics.

At that point Part III can materialize the already-proven program without becoming a loader/resolver/compiler.
