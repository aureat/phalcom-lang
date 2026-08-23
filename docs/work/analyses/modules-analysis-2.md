# Analysis: what should be carried forward from the modules-feature analysis

There is substantial value in the pasted analysis. Most of its broad architectural direction is already consistent with—and in several places now explicitly ratified by—the two attached implementation specifications. The important work is therefore not to copy it wholesale, but to extract the parts that either:

1. sharpen an existing decision;
2. expose a missing implementation abstraction;
3. establish a useful invariant that the current specs imply but do not state strongly enough; or
4. reveal a place where the analysis is slightly too optimistic and needs a stronger semantic rule.

The current repair specification already establishes a statically linked module architecture, canonical cross-module declaration identity, declaration-shell SCC realization, an acyclic runtime initialization graph, provider-backed source resolution, authoritative ownership, and shared compiler/LSP identity.  The companion specification also makes `Project : Package : Module`, `universe`/`std` builtin Projects, static package exposure, builtin source providers, and semantic declaration SCCs architectural facts.

The pasted analysis fits this direction extremely well. But it contains several ideas worth promoting into the actual module model.

---

# 1. The most useful extraction: make Phalcom explicitly a graph-linking architecture

The strongest idea in the analysis is the change in mental model:

```text
old:
    import statement
        ↓
    loader
        ↓
    discover / execute

target:
    source
        ↓
    resolve a closed module universe
        ↓
    build interfaces
        ↓
    link identities and references
        ↓
    realize declarations
        ↓
    derive + validate initialization plan
        ↓
    materialize
        ↓
    initialize
```

This is not merely an implementation organization. It should be considered a fundamental modules invariant:

> **Import syntax describes relationships in a statically resolved program universe. It does not itself perform module loading or initialization.**

That rule is already strongly consistent with the attached specifications. The repair spec explicitly separates declaration planning from runtime initialization and requires runtime initialization to consume a distinct dependency DAG.  The universe spec similarly places interface discovery/linking, declaration-shell realization, and runtime initialization into distinct bootstrap phases.

I would make this even more explicit in the main modules specification.

A good authoritative architecture is:

```text
Entry
  │
  ▼
EntryClassifier
  │
  ├── EntryOwnership
  ▼
ProjectUniverse / RootResolver
  │
  ▼
SourceProvider
  │
  ▼
Parser
  │
  ▼
InterfaceBuilder
  │
  ▼
ImportResolver
  │
  ▼
SemanticLinker
  │
  ├── ModuleInterface
  ├── SymbolId
  ├── linked module references
  └── declaration references
  │
  ▼
DeclarationPlanner
  │
  ├── declaration graph
  ├── semantic SCCs
  └── declaration blueprints
  │
  ▼
InitializationPlanner
  │
  └── runtime initialization DAG
  │
  ▼
LinkedProgramPlan
  │
  ▼
Materializer
  │
  ▼
ModuleRegistry
  │
  ▼
ModuleInitCoordinator
  │
  ▼
Execution
```

This yields a particularly important architectural rule:

> **`ModuleResolver`/the linked program graph is semantic authority. `ModuleRegistry` is runtime storage, not a module resolver.**

That principle prevents a huge class of future problems in the LSP, AOT compilation, caches, package management, reflection, and testing.

---

# 2. The pasted analysis's three-edge distinction is excellent—but current Phalcom actually needs more than three graph layers

The pasted analysis proposes:

```text
Interface/reference dependency
Link/reference dependency
Initialization dependency
```

This is one of its best contributions.

However, after the later decisions reflected in the attached specs—especially canonical cross-module declaration identities and semantic declaration SCCs—the architecture should be slightly richer than that.

I would model at least these relationships:

| Relationship | Level | Cycles? | Meaning |
|---|---|---:|---|
| Project dependency | Project → Project | Preferably no | Makes another Project namespace available |
| Resolved import/reference | Module → Module | Yes | Source imports or otherwise refers to another module |
| Interface dependency | Module → Module | Yes | Compiler requires target `ModuleInterface`/symbols |
| Semantic declaration dependency | `SymbolId` → `SymbolId` | Usually yes, edge-specific | One declaration refers to another |
| Runtime reference dependency | Module → Module | Yes | Deferred executable code retains a runtime reference |
| Initialization dependency | Module → Module | **No** | Initializing A requires B to have initialized runtime values |

That gives Phalcom a much cleaner graph model.

## 2.1 Do not encode these as one `enum EdgeKind`

The same import can participate in several relationships simultaneously.

For example:

```phalcom
import .base as base

class Child is base.Shape {
    make() {
        base.factory()
    }
}
```

This import may create:

```text
module-resolution edge:
    child → base

interface edge:
    child → base

declaration edge:
    Child → base.Shape

runtime-reference edge:
    child → base
```

It does not follow that this creates an initialization edge.

So I would not model:

```rust
enum ImportEdge {
    Interface(...),
    Runtime(...),
    Initialization(...),
}
```

as if the categories were mutually exclusive.

Instead:

```rust
struct ResolvedImportEdge {
    source: ModuleId,
    target: ModuleId,
    binding: ImportBinding,
    condition: Option<BuildPredicate>,
    origin: SourceRange,
}
```

and derive separate graph projections from it.

For example:

```rust
struct LinkedModuleGraphs {
    imports: ModuleGraph<ResolvedImportEdge>,
    interface: ModuleGraph<InterfaceDependency>,
    runtime_refs: ModuleGraph<RuntimeReference>,
    initialization: Dag<InitializationDependency>,
    declarations: DeclarationGraph,
}
```

This is a better abstraction than using the syntactic import graph as every subsequent graph.

---

# 3. Ratify the central graph law

The pasted analysis expresses the core rule as:

```text
static/reference graph      cycles allowed
interface/type graph        cycles allowed
runtime initialization DAG  cycles forbidden
```

That is fundamentally correct, but the current specs add semantic declaration SCCs. I would ratify the expanded form:

```text
Project dependency graph
    → independent project-level resolution rules

Resolved module/reference graph
    → cycles allowed

Interface graph
    → cycles allowed

Semantic declaration graph
    → SCCs allowed
    → edge-specific cycle restrictions

Inheritance subgraph
    → cycles forbidden

Runtime-reference graph
    → cycles allowed

Runtime initialization graph
    → cycles forbidden
```

This matches the attached specification's explicit distinction between semantic SCCs, inheritance cycles, and runtime initialization cycles.

This is worth documenting centrally because otherwise developers will keep asking an underspecified question:

> “Does Phalcom allow circular imports?”

The correct answer should eventually be:

> That question conflates several different dependency graphs. Circular module references are permitted where their semantic relationships permit them; circular runtime initialization is not.

That is much more precise.

---

# 4. The `a`/`b` example is extremely useful—but the proposed initialization analysis needs strengthening

The pasted analysis gives this excellent example:

```phalcom
// a.ph
import .b as b

class A {
    makeB() {
        b.B.new()
    }
}
```

versus:

```phalcom
// a.ph
import .b as b

const default = b.makeDefault()
```

The intended distinction is correct.

The first clearly has a static/runtime reference to `b`, whereas the second obviously requires some runtime state from `b` while `a` is initializing.

That argues strongly against the simplistic rule:

```text
every import edge = initialization edge
```

because that rule would reject harmless mutual references.

## But there is an important problem

The pasted analysis says the compiler can largely distinguish these because method/block bodies are deferred executable code.

That is not sufficient.

Consider:

```phalcom
import .b as b

class A {
    makeDefault() {
        return b.default
    }
}

const default = A.new().makeDefault()
```

Syntactically, the read of `b.default` lives inside a method body.

Dynamically, that method is invoked during `a`'s module initialization.

So this:

```text
"inside a method" ⇒ runtime reference only
```

is unsound.

Likewise:

```phalcom
const thunk = || {
    b.value
}

const result = thunk()
```

The block body is lexically deferred but actually executes during initialization.

And dynamic dispatch makes complete reachability analysis difficult:

```phalcom
const result = factory.make()
```

where `factory.make()` eventually reaches `b`.

## 4.1 Therefore Phalcom needs an initialization-effect model

I would preserve the graph distinction but not define initialization dependencies as simply:

> cross-module reads occurring syntactically in the top-level AST.

Instead, make initialization dependency analysis operate on lowered executable semantics.

Conceptually classify code/references by phase:

```rust
enum EvaluationPhase {
    StaticSemantic,
    ModuleInitialization,
    DeferredRuntime,
}
```

And linked references:

```rust
struct LinkedRead {
    target: LinkedBindingRef,
    phase: EvaluationPhase,
    origin: SourceRange,
}
```

The difficult part is calls made during `ModuleInitialization`.

There are several possible strategies, but Phalcom should make an explicit choice before claiming the initialization DAG can always be perfectly derived.

### Conservative v1

A reasonable first implementation could conservatively propagate initialization requirements through statically known call edges originating from module initializers.

Unknown/dynamic calls would require a conservative effect such as:

```text
MayObserveExternalModuleState
```

rather than pretending there is no dependency.

Later, typing and whole-program semantic information can improve precision.

This fits Phalcom especially well because the future type checker/LSP already wants richer effect/semantic information.

### Important architectural conclusion

The analysis's separation of runtime-reference and initialization edges should absolutely be retained.

But:

> **Initialization dependency discovery is an effect-analysis problem, not merely an AST-placement problem.**

That clarification should be added before implementing the proposed three-way split.

---

# 5. Static imports should indeed be a true dependency preamble

The pasted analysis proposes making imports syntactically contiguous rather than merely declaring their execution order irrelevant.

I agree strongly.

The grammar should look approximately like:

```text
SourceUnit :=
    UnitMetadata*
    ImportDeclaration*
    ModuleItem*
```

with:

```text
UnitMetadata :=
    @!attribute(...)

ImportDeclaration :=
    import ...
    from ... import ...
```

Once the parser has accepted the first ordinary module item, subsequent static imports are invalid.

For example:

```phalcom
@!documentation("Geometry primitives")

import .point as point
from .vector import Vector

const origin = point.Point.new(...)
```

valid.

But:

```phalcom
const origin = ...

import .point as point
```

invalid.

## 5.1 Why this is more than cosmetic

It creates the compiler invariant:

```text
Before semantic analysis of the module body begins,
the complete explicit static import set is known.
```

That improves:

- interface building;
- dependency discovery;
- incremental compilation;
- source tooling;
- diagnostics;
- formatter behavior;
- human readability;
- future build conditions;
- AOT linking.

It also makes the semantic rule extremely obvious:

```phalcom
import .a
import .b
```

and:

```phalcom
import .b
import .a
```

describe the same module dependencies.

Import source order should not be a hidden initialization mechanism.

The universe specification already assumes ordinary module dependency preambles for `std`, so turning that concept into an explicit grammar invariant is a natural strengthening.

---

# 6. Ratify: source order among imports has zero runtime semantics

This should be stated independently of the grammar.

I recommend a normative invariant such as:

> **I-IMP-1:** Reordering static import declarations without changing their bindings or targets cannot change program semantics.

That means:

```phalcom
import .database
import .logging
```

and:

```phalcom
import .logging
import .database
```

cannot specify initialization order.

Any required ordering arises from the initialization graph.

This matters because otherwise the syntactic preamble could still accidentally become an ordered initializer list.

---

# 7. Initialization should expose a partial order, not a sibling order

The pasted analysis makes another very useful point.

Suppose:

```text
      A
     / \
    B   C
```

where `A` requires both `B` and `C`, but neither depends on the other.

The language should guarantee only:

```text
B before A
C before A
```

not:

```text
B before C
```

or:

```text
C before B
```

## Recommended semantic rule

> **The initialization graph establishes only dependency ordering. Relative initialization order between incomparable modules is unspecified.**

The implementation may nevertheless use a stable deterministic topological sort today.

For example:

```text
implementation:
    sort equal-ready nodes by ModuleId

language contract:
    no ordering guarantee between equal-ready nodes
```

That gives reproducible behavior now without freezing it into the language.

Why this matters:

```text
Today:
    sequential topological initialization

Future:
    initialize independent B and C concurrently
```

If the language accidentally promises source-order or traversal-order initialization now, parallel initialization becomes an observable breaking change.

This is an excellent piece of future-proofing from the pasted analysis.

---

# 8. `containment ≠ initialization` should become a named Phalcom invariant

This is probably the second-most important semantic addition from the pasted analysis.

For:

```text
geometry.shapes.circle
```

these facts exist:

```text
Project/Package containment:
geometry
└── shapes
    └── circle
```

But that must not imply:

```text
initialize geometry
initialize geometry.shapes
initialize geometry.shapes.circle
```

The package hierarchy and runtime initialization graph are different graphs.

The attached universe specification already requires structural ownership information to exist before user/module initialization.  That makes this separation particularly natural.

I would add:

> **I-PKG-2:** Package ancestry establishes structural ownership and resolution scope only. It does not itself establish runtime initialization dependencies.

---

# 9. Package surfaces should have a static half and a runtime half

The pasted analysis's conceptual split of `package.ph` is very useful:

```text
package.ph
├── statically inspectable package interface
└── ordinary runtime initializer/body
```

I would not necessarily represent those as two source objects. They are two semantic products of one source file.

For example:

```rust
struct PackageInterface {
    module: ModuleInterface,
    children: ChildExposureTable,
    metadata: StaticUnitMetadata,
}

struct ModuleInitPlan {
    module: ModuleId,
    executable: InitializerArtifact,
    dependencies: Vec<ModuleId>,
}
```

Then resolving:

```phalcom
import geometry.shapes.circle
```

may require the compiler to inspect:

```text
geometry/package.ph
geometry/shapes/package.ph
```

to establish whether `shapes` and `circle` are externally addressable.

But it does not execute those package initializers merely to answer the resolution question.

That is exactly the kind of separation necessary for a statically analyzable package model.

---

# 10. Path visibility and binding visibility should be explicitly independent

The pasted analysis provides a very clean distinction that deserves to be elevated.

These are different questions:

```text
Can another Project address geometry.internal.matrix_impl?
```

versus:

```text
Can another Project access binding Matrix through geometry?
```

Phalcom should never conflate them.

Conceptually:

```text
path exposure:
    whether ModuleId is externally addressable

binding export:
    whether a symbol is externally visible through a Module/Package object
```

For example:

```text
geometry._internal.matrix_impl
    └── defines Matrix

geometry
    └── exports Matrix
```

Consumers may legitimately use:

```phalcom
import geometry
geometry.Matrix
```

without acquiring permission to deep-import:

```phalcom
import geometry._internal.matrix_impl
```

This gives library maintainers freedom to move implementation modules later.

That is a major ecosystem-compatibility benefit.

---

# 11. Hierarchical immediate-child exposure is a good model

The pasted analysis proposes:

```phalcom
// geometry/package.ph
expose .shapes

// geometry/shapes/package.ph
expose .circle
```

for:

```text
geometry.shapes.circle
```

to become externally addressable.

The conceptual rule is excellent:

> **Each Package controls exposure of its immediate children.**

That gives:

```text
geometry
  owns exposure decision for:
    geometry.point
    geometry.shapes

geometry.shapes
  owns exposure decision for:
    geometry.shapes.circle
    geometry.shapes.rectangle
```

rather than letting the root publish arbitrary deep paths.

Benefits include:

- local encapsulation;
- understandable refactoring boundaries;
- no gigantic root package exposure registry;
- natural LSP completion;
- clean reflection through `Package.__children__`;
- compatibility with the attached requirement that `__children__` expose logical/exposed children rather than raw filesystem structure.

I would preserve this.

---

# 12. Do not make package exposure depend on executing `package.ph`

This follows directly from the preceding two points and deserves an explicit prohibition.

Bad architecture:

```text
resolver asks:
"may I resolve geometry.shapes?"

runtime:
execute geometry/package.ph

package initializer:
mutates exposure table

resolver:
continues
```

That makes static dependency discovery impossible.

Instead:

```text
parse package.ph
        ↓
build PackageInterface
        ↓
read exposed-child metadata
        ↓
resolve path
```

Then later:

```text
ModuleInitPlan(package)
        ↓
normal runtime initialization
```

This also preserves the current requirement that the compiler and LSP resolve exposure/private access identically.

---

# 13. Retain binding cells—but not as a circular-import mechanism

This is another strong idea.

The older circular-import model apparently motivated cells partly because imports could observe incompletely initialized modules.

With initialization cycles forbidden, that justification disappears.

But cells can still be useful.

Conceptually:

```text
Defining binding
      │
      ▼
 BindingCell
   │     │
   │     ├── exported view
   │     ├── selective import
   │     └── re-export
   │
   ▼
current value
```

This gives Phalcom a stable identity for a binding independently of its current value.

Potential advantages:

```text
SymbolId / BindingCell
├── live mutable exports
├── re-export aliasing
├── debugger identity
├── reflection
├── incremental recompilation
└── future hot reload
```

The important simplification is:

> A cell should no longer be an ordinary user-visible “possibly uninitialized because of a legal cycle” state.

That removes one of the ugliest properties of circular module initialization while retaining useful binding identity.

## Suggested separation

```rust
struct SymbolId(...);

struct BindingCell {
    symbol: SymbolId,
    value: Value,
}
```

Static code refers to `SymbolId`.

Materialization maps that identity to a runtime `BindingCell`.

Imports/re-exports refer to the same underlying cell when semantics require a live binding rather than copying the value.

This would fit particularly well with the current canonical declaration-identity architecture.

---

# 14. The runtime state machine can indeed stay much simpler now

The pasted analysis correctly observes that rejecting runtime initialization cycles eliminates the need for machinery such as:

```text
transaction-local partially initialized modules
same-cycle reentrant access
SCC runtime initialization
early exposure of half-populated exports
cycle-specific uninitialized binding semantics
```

The current repair specification has already moved strongly in this direction:

```text
Allocated
    ↓
Prepared
    ↓
Initializing
   ↙       ↘
Failed   Initialized
```

with the topological driver as the sole normal initialization authority.

That is the right design.

I would keep this architecture and explicitly delete any older implementation machinery whose only purpose was making runtime initialization SCCs work.

---

# 15. If concurrent initialization arrives, use single-flight completion—not cyclic transactions

One useful residual idea from the pasted analysis is:

```text
Initializing(completion)
```

Even without initialization cycles, concurrency can produce this situation:

```text
Fiber 1:
    begins initializing M

Fiber 2:
    asks for M
```

The desired answer is not:

```text
initialize M again
```

and not:

```text
observe M halfway through
```

Instead:

```text
Fiber 2 awaits M's same completion
```

So a future scheduler-aware record might be:

```rust
enum ModuleInitState {
    Allocated,
    Prepared,
    Initializing(InitCompletion),
    Initialized,
    Failed(StoredModuleError),
}
```

This is a much simpler use of `Initializing` than the old circular-import transaction model.

It is not necessary to introduce scheduler machinery before parallel/on-demand module initialization exists, but the state model should not preclude it.

---

# 16. The import preamble and unit metadata fit together cleanly

The new `@!attribute(...)` design in the attached specs actually makes the pasted preamble proposal cleaner.

I would define a source unit approximately as:

```text
SourceUnit :=
    UnitAttribute*
    ImportDeclaration*
    UnitBodyItem*
```

Thus:

```phalcom
@!documentation("JSON utilities")
@!stability(#experimental)

import std.text as text
import .parser as parser

export Decoder

class Decoder {
    ...
}
```

This gives a very clear front-end sequence:

```text
1. Parse unit identity metadata
2. Parse dependency preamble
3. Parse declarations / exports / runtime body
```

The attached specification already requires `@!` to be declarative unit-header metadata rather than an executable statement.

---

# 17. Semantic `@!` attributes must be resolvable without running the unit

The pasted analysis's attribute point should also be carried forward.

Unknown metadata remains inert:

```phalcom
@!documentation("...")
```

But if Phalcom eventually defines:

```phalcom
@!some_build_attribute(...)
```

whose semantics affect:

- source selection;
- dependency edges;
- exposure;
- compilation mode;
- project/module graph construction;

then that attribute must be interpretable during static graph construction.

It cannot require running the module.

A useful attribute-definition classification might eventually be:

```rust
enum UnitAttributeSemantics {
    MetadataOnly,
    ResolverSemantic,
    CompilerSemantic,
    ReflectedMetadata,
}
```

with combinations allowed.

The important invariant is:

> **Any attribute whose result can change which program is being linked must be statically interpretable.**

Runtime execution cannot define the graph that runtime execution itself depends upon.

---

# 18. Conditional import support is worth reserving in the edge representation now

The pasted analysis suggests:

```rust
ImportEdge {
    target,
    kind,
    condition: BuildPredicate?,
}
```

Even if conditional-import syntax is not implemented yet, this is a sensible schema decision.

Future use cases are obvious:

```text
platform
architecture
feature
test-only dependencies
native vs pure implementation
optional subsystem
debug/release configuration
```

The key is not necessarily to put `condition` on every derived graph edge.

Better:

```rust
struct ResolvedImportEdge {
    target: ModuleId,
    binding: ImportBinding,
    condition: Option<BuildPredicate>,
    origin: SourceRange,
}
```

Then graph construction evaluates the predicate under a specific immutable:

```rust
BuildConfiguration
```

and derives active interface/declaration/runtime edges.

## Cache consequence

`BuildConfiguration` must participate in the resolver/semantic generation key.

Otherwise:

```text
feature A enabled
```

and:

```text
feature A disabled
```

could accidentally reuse the same import graph cache.

That is the kind of forward-compatible representation decision worth making now.

---

# 19. `SourceProvider` is more important than it first appears

The pasted analysis emphasizes keeping physical storage behind:

```rust
trait SourceProvider
```

This is absolutely worth retaining.

The current repair spec already moves in this direction with distinct Project, standalone Package, standalone Module, builtin, and inline providers.

But the architectural payoff is broader:

```text
ModuleId
   ↓
SourceProvider
   ↓
SourceUnit
```

means the compiler is independent of where source comes from.

Potential providers:

```text
filesystem
builtin embedded source
package-manager cache
archive
editor overlay
generated source
remote build cache
test fixture
REPL/inline buffer
```

The compiler should not contain logic like:

```rust
PathBuf::from(module_id)
```

outside provider implementations.

That one boundary will pay for itself repeatedly.

---

# 20. Make `SourceId` the provenance identity and `ModuleId` the semantic identity

The pasted analysis supports an important conceptual split already present in the attached repair spec.

```text
ModuleId
    = "what module is this?"

SourceId
    = "what source input produced this?"
```

Those are not interchangeable.

For example, an editor buffer could replace the physical backing source of:

```text
geometry.point
```

while its semantic identity remains:

```text
ModuleId(geometry.point)
```

Likewise a builtin module has:

```text
ModuleId(universe.reflection.selector)
```

and a builtin `SourceId`, but no meaningful user-facing filesystem identity.

This distinction is also why the canonical builtin URI model works:

```text
phalcom://universe/reflection/selector
```

rather than a fake `<core>` path. The current specs explicitly make the LSP consume shared `ModuleId`/`SourceId` identity rather than reconstructing semantic identity from URIs.

---

# 21. The incremental-compilation pipeline in the pasted analysis is worth formalizing

The analysis proposes roughly:

```text
SourceId + source hash
        ↓
AST
        ↓
ModuleInterface
        ↓
resolved edges
        ↓
semantic result
        ↓
compiled artifact
```

That is a very good incremental-compilation architecture.

I would refine it slightly:

```text
SourceSnapshot
    { SourceId, content_hash }
            │
            ▼
ParsedUnit
            │
            ▼
ModuleInterface
    { interface_fingerprint }
            │
            ▼
ResolvedModule
    { resolved_imports }
            │
            ▼
SemanticModule
    { declaration_fingerprint }
            │
            ▼
LinkedModulePlan
    { runtime_ref_fingerprint,
      init_dependency_fingerprint }
            │
            ▼
Executable/Materialization input
```

Why multiple fingerprints matter:

A change such as:

```phalcom
method body implementation changes
```

may leave:

```text
ModuleInterface
```

identical.

Therefore downstream modules that only need exported declarations may not need complete semantic re-analysis.

But an implementation-only change could still change:

```text
runtime initialization dependencies
```

or runtime executable code.

So “public interface unchanged” should not mean “nothing whatsoever invalidates.”

A layered fingerprint system gives much finer invalidation.

---

# 22. This architecture is particularly valuable for the LSP

The attached repair specification already establishes:

```rust
struct ResolvedDocument {
    uri: Url,
    source: SourceId,
    module: ModuleId,
    program: ParsedProgram,
    imports: Vec<ResolvedImportEdge>,
    generation: SourceGeneration,
}
```

and requires the semantic engine not to perform a second independent resolver pass.

The pasted analysis explains why that decision is so valuable.

The compiler and LSP can share:

```text
ModuleId
SourceId
ModuleInterface
ResolvedImportEdge
SymbolId
DeclarationGraph
```

rather than the LSP maintaining a fake shadow version of the language.

That leads naturally to:

```text
Compiler:
    SourceProvider → resolver → semantic graph

LSP:
    editor SourceProvider overlay
              ↓
    same resolver
              ↓
    same semantic graph
```

Only the source provider differs.

That is an excellent long-term architecture.

---

# 23. Project identity as resolved graph identity is already correct—and should not be weakened

The pasted analysis argues:

```rust
ModuleId {
    projectInstance: ResolvedProjectId,
    relativePath: ModulePath,
}
```

rather than using package names as semantic identity.

The current specs improve this further:

```rust
enum ProjectIdentity {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}
```

with builtin, user-resolved, and synthetic identity spaces structurally disjoint.

The useful principle from the pasted text is:

> aliases are resolution conveniences, not semantic identities.

So:

```text
alias foo ─┐
           ├──► resolved project node P
alias bar ─┘
```

means types from:

```text
foo.Model
bar.Model
```

refer to the same semantic declaration if both aliases resolve to `P`.

Conversely two independently resolved project nodes with coincidentally equal metadata are not necessarily the same identity.

This becomes increasingly important once Phalcom has:

- nominal types;
- protocols;
- generic specializations;
- exceptions;
- reflection;
- native ABI integration;
- serialized type identities.

The attached tagged identity design is the correct implementation of that earlier insight.

---

# 24. The current declaration-SCC design is a major improvement over the pasted analysis

One area where the attached specs have overtaken the pasted analysis is static declaration realization.

The current specification explicitly requires:

```text
1. collect/link declaration identities
2. build blueprints
3. materialize shells
4. resolve static edges
5. validate constraints
6. realize bodies
7. execute runtime initialization separately
```

and explicitly supports semantic SCCs.

This is stronger than treating all compile-time relationships merely as “interface edges.”

I would therefore update the analysis's terminology.

Instead of:

```text
interface/type graph
```

use:

```text
module interface graph
declaration semantic graph
```

because the latter operates at `SymbolId` granularity.

For example:

```phalcom
// a.ph
import .b as b

class A {
    foo(x: b.B) { ... }
}
```

the interesting relationship is not only:

```text
a → b
```

but:

```text
A.foo parameter type → SymbolId(b.B)
```

That distinction becomes crucial for:

- SCCs;
- inheritance;
- generic constraints;
- protocols;
- future type checking;
- incremental invalidation.

---

# 25. Do not reintroduce runtime SCC machinery just because semantic SCCs exist

The pasted analysis anticipated this distinction, and the current specs now make it explicit.

These are entirely different:

```text
A's declaration refers to B's declaration
B's declaration refers to A's declaration
```

versus:

```text
A's initializer needs initialized B
B's initializer needs initialized A
```

The former may be legal.

The latter is not.

Likewise:

```text
class A is B
class B is A
```

remains illegal even though both classes can be placed into a semantic SCC for predeclaration.

The current specs are very strong here, and that separation should remain one of Phalcom's central explanatory concepts.

---

# 26. The loader must never discover semantic identity at runtime

The pasted analysis's final architectural statement deserves to become a design law.

Avoid APIs shaped conceptually like:

```rust
vm.import("geometry.point")
```

where the VM:

1. parses the string;
2. finds files;
3. determines a project;
4. reads `package.ph`;
5. resolves dependencies;
6. creates identities;
7. compiles;
8. executes.

Instead:

```rust
let program: LinkedProgramPlan = compiler.link(entry)?;
vm.run(program)?;
```

By the time runtime receives it, things such as:

```text
ModuleId
ownership
resolved import targets
SymbolIds
package ancestry
declaration identities
initialization dependencies
```

should already be settled.

Runtime still materializes objects, but it should not be deciding what source code “means.”

---

# 27. A concrete representation I would adopt

Putting the useful pieces together, I would aim for a core representation approximately like this.

```rust
struct ResolvedModule {
    id: ModuleId,
    source: SourceId,
    kind: UnitKind,
    interface: ModuleInterface,
    imports: Vec<ResolvedImportEdge>,
    metadata: UnitMetadata,
}
```

```rust
struct ModuleInterface {
    namespace: Namespace,
    exports: ExportTable,
    child_exposure: Option<ChildExposureTable>,
}
```

```rust
struct ResolvedImportEdge {
    target: ModuleId,
    binding: ImportBinding,
    condition: Option<BuildPredicate>,
    origin: SourceRange,
}
```

```rust
struct LinkedSymbolRef {
    symbol: SymbolId,
    defining_module: ModuleId,
}
```

```rust
struct DeclarationEdge {
    from: SymbolId,
    to: SymbolId,
    kind: DeclarationEdgeKind,
    origin: SourceRange,
}
```

```rust
enum DeclarationEdgeKind {
    Superclass,
    TypeReference,
    Reopen,
    ProtocolReference,
    StaticValueReference,
    // later...
}
```

```rust
struct RuntimeReferenceEdge {
    from: ModuleId,
    to: ModuleId,
    origin: SourceRange,
}
```

```rust
struct InitializationDependency {
    prerequisite: ModuleId,
    dependent: ModuleId,
    reason: InitDependencyReason,
    origin: SourceRange,
}
```

```rust
enum InitDependencyReason {
    DirectInitializerRead,
    InitializerCallEffect,
    RequiredRuntimeValue,
}
```

And finally:

```rust
struct LinkedProgramPlan {
    modules: IndexMap<ModuleId, LinkedModulePlan>,

    declaration_graph: DeclarationGraph,
    declaration_sccs: Vec<SemanticScc>,

    initialization_dag: InitializationDag,

    entry: ModuleId,
}
```

That makes the various authorities explicit.

---

# 28. New normative invariants I would add to the current module spec

The current repair spec already has a strong invariant table. I would add approximately these.

| Proposed invariant | Rule |
|---|---|
| `I-IMP-1` | Static import declarations are dependency declarations, not executable statements. |
| `I-IMP-2` | Static imports form a contiguous dependency preamble after unit metadata. |
| `I-IMP-3` | Reordering equivalent imports cannot change program semantics. |
| `I-GRAPH-1` | The resolved import/reference graph and runtime initialization graph are distinct graph projections. |
| `I-GRAPH-2` | A runtime reference does not by itself imply an initialization dependency. |
| `I-GRAPH-3` | Initialization dependencies are derived from initialization-time runtime requirements, not from import syntax alone. |
| `I-GRAPH-4` | Semantic declaration edges are represented independently from module initialization edges. |
| `I-PKG-2` | Package containment never implicitly establishes runtime initialization order. |
| `I-PKG-3` | Package child exposure is statically inspectable without executing package initialization. |
| `I-VIS-1` | Path addressability and binding export visibility are independent capabilities. |
| `I-INIT-1` | The runtime initialization graph specifies only dependency ordering; order among incomparable nodes is not language-defined. |
| `I-RUN-4` | Runtime materialization consumes already resolved semantic identities and may not perform source/module resolution. |
| `I-CACHE-2` | Interface, semantic, and executable invalidation may use distinct fingerprints; source equality is not the only cache boundary. |

I think these would materially strengthen the current specifications.

---

# 29. Additional regression tests suggested by the pasted analysis

Several tests are worth adding beyond the already extensive matrix.

## Import preamble

```text
IMP-01
Import after an ordinary module item is rejected.

IMP-02
Unit metadata followed by imports followed by declarations succeeds.

IMP-03
Reversing two independent imports produces the same linked graph.

IMP-04
Import order does not create initialization edges.
```

## Graph separation

```text
GRAPH-01
a method in A references B, but A has no initializer-time use of B;
A→B exists in runtime-reference graph but not initialization DAG.

GRAPH-02
A initializer directly reads B export;
A has an initialization dependency on B.

GRAPH-03
Mutually referring deferred declarations/modules can form a static/reference cycle
without creating a runtime initialization cycle.
```

## Critical initialization-effect test

```text
GRAPH-04
A top-level initializer invokes a locally defined method that reads B.
The initialization planner must not incorrectly classify the B read as
"deferred merely because it appears in a method body".
```

This test is especially important because it catches the weakness in the pasted analysis's first approximation.

## Package containment

```text
PKG-03
Resolving a nested child inspects ancestor package interfaces
but does not execute ancestor package initializers.

PKG-04
Importing pkg.child does not automatically create pkg initializer dependency.

PKG-05
If child actually requires an initialized export from pkg,
the dependency is created explicitly by runtime requirements.
```

## Visibility

```text
VIS-01
Public binding re-export from a private implementation module succeeds.

VIS-02
Consumer can access façade export but cannot deep-import private defining module.

VIS-03
Exposing parent child does not automatically expose grandchildren.

VIS-04
Grandchild becomes addressable only when each package boundary exposes the next component.
```

## Ordering

```text
INIT-ORDER-01
Graph records B→A and C→A but no B↔C relationship.

INIT-ORDER-02
No semantic test assumes deterministic sibling initialization ordering.
```

## Caches

```text
CACHE-02
Implementation-only change preserving ModuleInterface does not invalidate
interface-only dependent analysis.

CACHE-03
Changing initializer dependencies invalidates initialization planning even when
the exported interface fingerprint remains unchanged.

CACHE-04
Changing BuildConfiguration invalidates conditional-import graph state.
```

---

# 30. What from the pasted analysis is already superseded by the attached specs

Several parts are good observations but no longer need new decisions.

## Project identity

Already superseded by the stronger tagged algebra:

```rust
ProjectIdentity::Builtin
ProjectIdentity::Resolved
ProjectIdentity::Synthetic
```

That is better than the pasted `ResolvedProjectId`-only formulation.

## Semantic declaration cycles

The attached specs already go significantly further than the analysis by explicitly supporting declaration SCCs and declaration-shell materialization.

## Runtime initialization

The current repair specification already has the simplified nonrecursive topological state machine and sticky failure semantics.

## Source authority

The attached repair spec already has authoritative `EntryOwnership`, restricted providers, canonical source confinement, and bidirectional `SourceId ↔ ModuleId` mappings.

## LSP authority

The current spec already makes `phalcom-modules` the sole authority and prohibits a second LSP resolver.

## Builtins

The current universe spec already makes `universe` and `std` complete provider-backed Project graphs rather than resolver aliases.

So these parts of the pasted analysis should be viewed as architectural justification for decisions that are now already ratified.

---

# 31. The one point I would explicitly *not* copy verbatim

I would not adopt this reasoning literally:

> “Top-level executable initialization is a finite compile-time region; method and block bodies can be linked without executing them.”

The first half is useful.

The second half is insufficient to infer initialization dependencies because those bodies can be called from that finite initialization region.

The correct formulation is closer to:

> **Module initialization begins from a statically identifiable root region, but its runtime dependency effects may flow through code invoked from that region. Initialization dependency analysis therefore operates on initialization effects/reachability, not lexical nesting alone.**

That preserves the useful insight without introducing a subtle correctness hole.

This deserves attention before implementing the more precise initialization graph; otherwise Phalcom risks trading overly restrictive import cycles for unsound under-approximation of actual initialization dependencies.

---

# 32. Recommended final architecture

Putting the current specifications and the useful parts of this analysis together, I would describe Phalcom modules like this:

```text
                         PROJECT GRAPH
                  ┌────────────────────────┐
                  │ available import roots │
                  └────────────┬───────────┘
                               │
                               ▼
                        SOURCE UNIVERSE
              ModuleId ⇄ SourceId / SourceProvider
                               │
                               ▼
                     RESOLVED IMPORT GRAPH
                        cycles allowed
                               │
                 ┌─────────────┼──────────────┐
                 ▼             ▼              ▼
          INTERFACE GRAPH  DECLARATION     RUNTIME
          cycles allowed      GRAPH         REFS
                              │              │
                              │ SCC          │ cycles
                              │ allowed      │ allowed
                              ▼              │
                     declaration shells      │
                     + semantic realization  │
                              │              │
                              └──────┬───────┘
                                     ▼
                       INITIALIZATION EFFECT
                              ANALYSIS
                                     │
                                     ▼
                        INITIALIZATION DAG
                          cycles forbidden
                                     │
                                     ▼
                         LINKED PROGRAM PLAN
                                     │
                                     ▼
                           MATERIALIZATION
                                     │
                                     ▼
                         MODULE REGISTRY
                                     │
                                     ▼
                         TOPOLOGICAL INIT
                                     │
                                     ▼
                              EXECUTION
```

And package structure is orthogonal:

```text
containment graph
Project / Package / Module
          │
          ├── ownership
          ├── path exposure
          ├── reflection hierarchy
          └── structural context

NOT:

          └── implicit initialization ordering
```

This is a very strong module architecture.

---

# 33. Priority: what I would actually add to the specs now

If the objective is to extract only the highest-value additions rather than expand the specification indefinitely, I would add these in this order:

1. **Explicit graph taxonomy.** Distinguish resolved import, interface, declaration, runtime-reference, and initialization relationships.

2. **Static import preamble.** Imports are contiguous declarative header items and source order has no semantics.

3. **Initialization partial-order rule.** Incomparable module initialization order is unspecified.

4. **`containment ≠ initialization`.** Resolving or structurally materializing package ancestry cannot implicitly execute ancestor package initializers.

5. **Static package surfaces.** Exposure/path-addressability is determined from `PackageInterface` without runtime execution.

6. **Path visibility ≠ binding visibility.** Deep-import permission and façade export permission are separate.

7. **Initialization-effect qualification.** Do not infer initialization dependencies solely from lexical top-level/module-body placement.

8. **Retain binding identity/cells independently of cycles.** If Phalcom's import/re-export semantics remain live, use canonical cells for that purpose rather than circular-import escape hatches.

9. **Reserve conditional dependency representation.** `ResolvedImportEdge.condition: Option<BuildPredicate>` or equivalent, even if syntax comes later.

10. **Layered compiler-cache fingerprints.** Separate source, interface, semantic/declaration, and initialization/executable invalidation.

The first seven are language/module architecture decisions. The last three are implementation architecture decisions that are cheap to prepare for now and costly to retrofit poorly later.

The resulting design is significantly stronger than a conventional “module loader”: Phalcom gets a statically resolved semantic universe with distinct compile-time, declaration, reference, and initialization relationships. That is exactly the substrate its future typing, LSP, `universe` split, package manager, AOT compilation, reflection, and eventually parallel initialization need.