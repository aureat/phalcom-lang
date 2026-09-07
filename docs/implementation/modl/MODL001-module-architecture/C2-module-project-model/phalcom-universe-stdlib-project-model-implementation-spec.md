# Phalcom Builtin Universe, Standard Library, Reflection & Project Model — Implementation Specification

Retire the language-level core root; establish universe/std; split core.ph; formalize Project : Package : Module and stable reflection

Status: implementation-ready design specification · Baseline: phalcom-lang @ 5c64793299ab7f29dd7ec4d6a4e76c33dfa14167 · 2026-08-16

## 1. Architectural statement

Phalcom will model code organization as one subtype ladder: Module is the smallest namespace/execution unit; Package is a Module that organizes child modules/packages; Project is a Package that establishes an independent build, dependency, versioning, publication, and distribution boundary. A Project is its own root Package. There is no separate runtime root-package object beneath it.

```
Object
  └── Module
       └── Package
            └── Project
```

At the builtin boundary, “core” ceases to be a language import namespace. The language-level primordial namespace is the builtin Project universe. The standard library is a separate builtin Project std. The prelude is a small, explicit projection of universe exports, not a dynamically accumulated bootstrap namespace.

## 2. Ratified decisions

| Decision | Normative rule |
| --- | --- |
| D1 | core is not a user-visible import root; universe is the canonical builtin namespace. |
| D2 | core.ph is retired and split into real statically linked modules/packages of builtin Project universe. |
| D3 | Prelude is an explicit immutable projection of universe; bootstrap definitions never auto-enter it. |
| D4 | Standard library is a separate toolchain-owned builtin Project std composed of ordinary packages. |
| D5 | Builtin, resolved user, and synthetic project identities are structurally disjoint. |
| D6 | Lexical read-only __module__: Module, __package__: Option<Package>, __project__: Option<Project> describe definition context. |
| D7 | The complete `__name__`-shaped namespace is language-reserved and compiler-controlled; unknown dunders are forbidden, while a specifically standardized protocol hook may explicitly permit user implementation/override in defined declaration roles. `_$name` remains VM/floor internal. |
| D8 | Reflection distinguishes exports from selectors; Selector shape is first-class and __understands__ is selector-aware. |
| D9 | Ordinary Module/Package/Project methods may be shadowed by exports; non-overridable dunder reflection/context names remain guaranteed even though separately standardized protocol-hook dunders may opt into user override semantics. |
| D10 | Persistent projects explicitly declare namespace; display name and namespace are independent. |
| D11 | Current source-retaining “compiled” plans are renamed until genuinely VM-independent artifacts exist. |
| D12 | universe and std are available in standalone module/package and REPL/inline contexts without a user project. |
| D13 | Project : Package : Module; the project object is its own root package. |
| D13.1 | Only Projects declare external dependencies. |
| D13.2 | Version and publication identity belong only to Projects. |
| D13.3 | A Project namespace is its root package import name. |
| D13.4 | A standalone Package has package semantics but no Project owner/dependency graph. |
| D13.5 | Projects do not nest semantically inside another Project source tree; future workspaces compose sibling Projects. |
| D13.6 | The project root package.ph initializes the Project object itself; no duplicate root Package object is created. |
| D14 | Semantic declaration SCCs are supported in Modules v1 through predeclaration/materialization of declaration shells. This does not permit runtime module-initialization cycles or class inheritance cycles. |
| D15 | Unit-level metadata uses contiguous `@!attribute(...)` syntax. `@!` targets the current source-unit object; no competing `@module.*`/`@package.*`/`@project.*` targeting syntax is introduced. |
| D16 | Dunder reservation is a compiler policy, not a runtime identifier limitation. The compiler owns a protocol-policy table that may mark selected standardized dunders as user-implementable/overridable; unknown dunders remain reserved. |
| D17 | Logical Project/package/module path components are canonical `snake_case`; physical package directories/module file stems are canonical `kebab-case`, with deterministic reversible separator mapping. Persistent Project namespace is explicit in `project.toml`. |
| D18 | Builtin LSP/source documents have stable virtual identities such as `phalcom://universe/reflection/selector` and `phalcom://std/json`, derived from logical builtin ModuleId rather than physical installation paths. |

## 3. Terminology and semantic model

| Concept | Definition | Adds beyond parent |
| --- | --- | --- |
| Module | Smallest statically analyzable namespace and initialization unit | Declarations, imports, exports, initializer, module identity |
| Package | Module that owns/organizes child modules and packages | Hierarchy, child exposure, package path/parent |
| Project | Package that is independently buildable and dependency-bearing | Namespace root, dependencies, version, publication/build metadata |
| universe | Builtin Project defining Phalcom’s primordial language/runtime object universe | Toolchain-owned; always available |
| std | Builtin Project containing general-purpose standard library packages | Toolchain-owned; normal module/package semantics after root selection |
| prelude | Implicit fallback scope selecting a small set of universe exports | No independent runtime namespace or export authority |

## 4. Runtime class hierarchy

```
class Module { ... }
class Package is Module { ... }
class Project is Package { ... }
```

### 4.1 Runtime identity and metadata

| Class | Required immutable structural data |
| --- | --- |
| Module | ModuleId; logical name/path; export table; linked metadata; owning package Option; owning project Option |
| Package | All Module data + parent package Option + child/exposed-child metadata |
| Project | All Package data + display name + namespace + dependency metadata + version/publication/build metadata as available |

Structural relationships are materialized before any user/module initializer runs. Initialization state must never determine whether __package__ or __project__ exists.

## 5. Project is the root package

### 5.1 Filesystem interpretation

```
geometry-toolkit/
  project.toml      # namespace = "geometry_toolkit"
  src/
    package.ph      # initializes the Project object
    cli.ph
    user-model/
      package.ph
      user-record.ph
```

This produces one root object: `Project(geometry_toolkit)`. Logical children use snake_case (`geometry_toolkit.user_model.user_record`) while their physical directory/file stems use kebab-case (`user-model/user-record.ph`). Do not allocate Project → RootPackage as two runtime objects.

### 5.2 Standalone package

```
standalone/
  package.ph
  foo.ph
```

This produces a Package with __project__ == None. It has local package hierarchy and package exports, but no dependency graph, version, publication identity, or global project namespace.

### 5.3 Standalone module

A single .ph file outside any owning package/project produces a Module with __package__ == None and __project__ == None. It may not implicitly discover sibling files.

## 6. Ownership model

```
Module -> nearest Package -> owning Project

Project:
  owning_package = None   // it is the root Package
  owning_project = self

Standalone Package:
  owning_project = None

Standalone Module:
  owning_package = None
  owning_project = None
```

### 6.1 EntryOwnership mapping

| EntryOwnership | Runtime/context result |
| --- | --- |
| ProjectOwned | Module/Package/Project objects participate in one Project tree; __project__ = Some(root Project). |
| StandalonePackageOwned | Module/Package objects participate in package tree; __project__ = None. |
| StandaloneModule | One Module only; no package/project filesystem authority. |
| Inline/REPL | Synthetic Module; no package/project unless a context is explicitly attached. |

## 7. Builtin project identity and import roots

```
enum ProjectIdentity {
    Builtin(BuiltinProject),
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}

enum BuiltinProject {
    Universe,
    Std,
}
```

The module resolver treats universe and std as builtin project roots. Once the root is selected, ordinary ModulePath/package exposure/linking rules apply. There is no special “core source unit” that bypasses normal graph semantics.

### 7.1 Root availability

| Context | universe | std | own project | declared dependencies |
| --- | --- | --- | --- | --- |
| Project | Yes | Yes | Yes | Yes |
| Standalone Package | Yes | Yes | No | No |
| Standalone Module | Yes | Yes | No | No |
| Inline/REPL | Yes | Yes | No unless context attached | No unless context attached |

### 7.2 Builtin source provider

Builtin roots are backed by an explicit builtin provider/interface source, not by fake filesystem `SourceUnit`s or a reserved numeric project masquerading as an ordinary resolved Project.

```rust
struct BuiltinProjectSourceProvider {
    builtin: BuiltinProject,
    // embedded/generated source + interface catalog
}
```

The provider owns:

- builtin `ModuleId` → source/interface lookup;
- canonical builtin `SourceId`;
- logical source metadata;
- LSP virtual document identity;
- optional backing-file provenance for toolchain development.

A builtin import participates in the same closed-program discovery/linking graph as user modules. Resolver recognition without interface/discovery/link/materialization support is a failed integration.

### 7.3 Virtual builtin documents

Canonical editor/LSP document identities are logical and installation-independent:

```text
phalcom://universe/
phalcom://universe/reflection/selector
phalcom://std/json
```

Rules:

- the URI host is the builtin Project namespace (`universe` or `std`);
- path segments are logical `snake_case` components;
- no physical kebab-case spelling and no `.ph` extension appears in the canonical URI;
- the URI is generated deterministically from builtin `ModuleId`;
- builtin documents are read-only to normal editor writes;
- a physical toolchain source file may be retained as provenance/source-map metadata but does not replace the canonical URI.

## 8. What belongs in universe

universe contains identities and semantics needed for Phalcom to describe its own object/runtime universe. It is not the repository for every library shipped with the toolchain.

| Group | Representative contents | Prelude default? |
| --- | --- | --- |
| Object model | Object, Class; Behavior/Metaclass available reflectively | Object/Class yes; deeper metamodel no |
| Scalars | Number, Int, Float, String, Bool, Symbol, Unit | Mostly yes |
| Algebraic/common data | Option, Some, None-class, List, Map, Set, Tuple, Record, Range, Bytes, Iterable | Common surface yes; None-class only through universe.None |
| Callable model | Function; Closure, Method, MethodFamily, BoundMethod, Family | Function maybe; reflective implementation classes no |
| Code organization | Module, Package, Project | No by default |
| Reflection | Selector, Message, Attribute and related metadata abstractions | No by default |
| Errors | Error and language/runtime-defined errors | Error yes; specialized errors generally no |
| Runtime/concurrency floor | Fiber/Resource when they are fundamental runtime identities | No by default |

Future additions use the criterion: if the language/runtime needs the identity or semantics to explain itself, it belongs in universe; if it is a general-purpose facility implementable atop the language, it belongs in std or a third-party Project.

## 9. Standard library project std

### 9.1 Purpose

std is a toolchain-owned Project containing general-purpose library functionality. It is ordinary Phalcom code/packages after import-root selection and therefore dogfoods the same dependency preamble, exposure, linking, initialization, and reflection rules as user code.

### 9.2 Initial package taxonomy

```
std
├── io
├── fs
├── path
├── text
├── regex
├── json
├── math
├── random
├── time
├── process
├── net
├── concurrent
└── testing
```

### 9.3 Universe/std classification examples

| Facility | Home | Reason |
| --- | --- | --- |
| Object, Class, Option, Module, Package, Project | universe | Defines the language object/code model |
| Selector | universe.reflection | Required to describe Phalcom dispatch precisely |
| Fiber primitive | universe | Runtime execution identity if fibers remain language-level |
| Future abstraction | std.concurrent by default | Can be library-level atop runtime scheduling/fibers |
| Path, File, Directory | std.path/std.fs | Environment/library facilities |
| JSON | std.json | General-purpose codec |
| HTTP | std.net.http | General-purpose protocol |
| Testing framework | std.testing | Tool/library facility |

## 10. Retire core as a language namespace

### 10.1 Internal versus language-level terminology

The word core may remain internally for kernel/core classes, primitive floor code, and bootstrap invariants. It must not remain a public import root or ordinary ModuleId category.

### 10.2 Rust naming

Rename the internal Rust Universe struct to KernelUniverse or RuntimeUniverse to avoid colliding conceptually with the language-level universe Project. The exact internal name is implementation discretion; the requirement is that VM kernel state and the runtime universe package/project are not presented as the same abstraction.

### 10.3 Temporary compatibility

If needed during a staged migration, ModuleId::core()/ImportRootTarget::Core may be temporary aliases to BuiltinProject::Universe. They must not survive the final cleanup phase.

## 11. Split core.ph into the universe module graph

### 11.1 Required end state

```
universe/
  project.toml                # builtin/toolchain descriptor
  src/
    package.ph                # root Project surface
    object/
      package.ph
      object.ph
      behavior.ph
      class.ph
      metaclass.ph
    scalar/
      package.ph
      number.ph
      string.ph
      bool.ph
      symbol.ph
    callable/
      package.ph
      function.ph
      closure.ph
      method.ph
      family.ph
    option/
      package.ph
    collections/
      package.ph
      iterable.ph
      list.ph
      map.ph
      set.ph
      tuple.ph
      record.ph
      range.ph
      bytes.ph
    errors/
      package.ph
      error.ph
      argument.ph
      indexing.ph
      contracts.ph
    reflection/
      package.ph
      module.ph
      package-object.ph
      project.ph
      selector.ph
      message.ph
      attribute.ph
    concurrency/
      package.ph
      fiber.ph
```

The exact file grouping may be adjusted during inventory, but the semantic requirement is fixed: each resulting unit is a real Module/Package in the builtin universe Project and participates in ordinary static linking.

### 11.2 Migration inventory algorithm

0.  Do not split a class/declaration relationship across modules until the repair specification's canonical static declaration reference, declaration-shell materialization, and semantic-SCC gates are green.

1.  Inventory every class reopen, class declaration, helper, and top-level initializer currently in `core.ph`.

2.  Classify each item as kernel-floor wrapper, universe abstraction, or std/library abstraction.

3.  Move std/library abstractions (for example Path/Future if judged library-level) out of universe rather than blindly preserving current core.ph membership.

4.  Construct explicit import/static-reference dependencies between new universe modules. Every cross-module class/superclass/reopen relation must retain canonical linked declaration identity.

5.  Create package.ph surfaces that re-export only intentional public names.

6.  Build the root universe Project surface as curated re-exports, not wildcard/bootstrap-global accumulation.

7.  Delete run_core_module/include_str!(core.ph) after the builtin graph initializes successfully.

## 12. Bootstrap phases

```
Phase 0  Allocate kernel class/metaclass object graph
Phase 1  Install native floor primitives
Phase 2  Materialize builtin universe Project/Package/Module objects
Phase 3  Discover/link universe interfaces and canonical static declaration identities
Phase 4  Predeclare/materialize declaration shells, including semantic SCCs
Phase 5  Resolve declaration edges; attach/reopen canonical kernel classes; compile/install declaration bodies and module initializers
Phase 6  Initialize universe modules in validated runtime topological order
Phase 7  Freeze/commit universe public interfaces and prelude projection
Phase 8  Materialize std on demand or as a separately linked builtin graph
Phase 9  Resolve/link/materialize user program
```

Kernel class objects must exist before universe modules that reopen/extend them execute. A split source unit that reopens or references a kernel class must resolve the canonical kernel/universe declaration identity; same-leaf-name allocation is an error. Conversely, Phalcom-defined behavior currently in `core.ph` must be attached through normal universe declaration realization and module initialization rather than an exceptional one-file execution path.

## 13. Prelude semantics

### 13.1 Prelude is a projection

The prelude is a declaratively listed subset of universe exports. VM startup must not scan all globals created during universe initialization and add them to prelude_names.

```
local lexical binding
    > explicit module declaration/import
    > prelude fallback
    > unresolved
```

### 13.2 Shadowing and export behavior

- A module may declare a local name that equals a prelude name; the explicit local binding shadows the prelude fallback.

- Prelude bindings do not occupy the explicit module namespace and therefore do not create duplicate-declaration errors by themselves.

- export Name does not implicitly re-export a prelude binding. Re-export requires an explicit import/binding.

- Prelude membership is versioned, curated, and testable as data.

### 13.3 Initial prelude recommendation

Start practical but deliberately smaller than the current bootstrap-global set: Object, Class, Number, Int, Float, String, Bool, Symbol, Unit, Option, Some, Iterable, List, Map, Set, Tuple, Record, Range, Bytes, Error, and possibly Function. Keep Behavior, Metaclass, Method-family classes, Module/Package/Project, Message, System, Fiber, Resource, and other reflective/runtime implementation identities universe-only unless concrete ergonomic evidence justifies promotion.

The immediate None value remains a special prelude/global value. universe.None denotes the None class object, preserving the existing semantic distinction.

## 14. Project manifests, naming, physical layout, and unit metadata

### 14.1 Persistent project manifest

```
[project]
name = "Geometry Toolkit"     # display/distribution metadata
namespace = "geometry_toolkit" # required logical import-root component
version = "..."               # optional/required according to publish policy

[dependencies]
json = { package = "...", version = "..." }
```

`name` and `namespace` are independent. Renaming the project display name must not silently rewrite source import paths. Persistent Project namespace is never inferred from the physical project-directory name.

### 14.2 Project-only capabilities

| Capability | Module | Package | Project |
| --- | --- | --- | --- |
| Imports/exports/initializer | Yes | Yes | Yes |
| Child module/package organization | No | Yes | Yes |
| External dependency declarations | No | No | Yes |
| Version/publication identity | No | No | Yes |
| Global import namespace root | No | No | Yes |
| Build/toolchain project metadata | No | No | Yes |

### 14.3 Nested projects

A Project source tree may not semantically contain another Project. If a nested `project.toml` is discovered inside an owning project source root, report a diagnostic unless the outer context is a future workspace that explicitly declares separate member roots. Workspaces compose Projects as siblings in a workspace graph; they do not make one Project the package child of another.

### 14.4 Logical `snake_case` versus physical `kebab-case`

Phalcom deliberately distinguishes programming identity from filesystem spelling.

```text
physical:
    ~/path/to/geometry-toolkit/package-a/module-b.ph

project.toml:
    namespace = "geometry_toolkit"

logical:
    geometry_toolkit.package_a.module_b
```

Normative rules:

- Project `namespace` is explicit and canonical `snake_case`.
- Every logical package/module path component, dependency alias, and other logical import-root alias is canonical `snake_case`.
- Physical package-directory names and module file stems are canonical `kebab-case`.
- Resolver mapping is reversible separator conversion after validation: physical `package-a` ↔ logical `package_a`.
- Logical `-` and physical `_` are rejected rather than treated as aliases.
- Case/mixed-style alternate spellings are rejected; portability/normalization diagnostics remain authoritative where filesystem behavior can still create aliases.
- The physical Project root directory is non-semantic. Its name may conventionally correspond to the namespace (`geometry-toolkit` ↔ `geometry_toolkit`) but `project.toml` is the sole namespace authority.
- Reserved structural files such as `package.ph` and `project.toml` are not transformed path components.
- `package.ph` establishes Package identity. `main.ph`, if used as a default executable entry convention, does not establish package identity.

Compiler/LSP displays should prefer logical dotted paths. Filesystem diagnostics may additionally show the physical kebab-case path.

### 14.5 Unit-level `@!attribute(...)` metadata

The canonical source syntax for metadata attached to a Module/Package/Project unit is:

```phalcom
@!documentation("...")
@!some_metadata(...)
```

The `@!` marker is written contiguously with the attribute name. No whitespace, newline, or comment trivia may separate `!` from the identifier. `@! documentation(...)` is invalid. This design does not introduce `@module.documentation`, `@package.documentation`, or `@project.documentation`.

Conceptual grammar:

```text
unit_attribute := "@!" IDENT attribute_arguments?
```

A unit attribute is a declarative top-level unit-header item, not an executable statement and not an ordinary attribute attached to the next declaration. Its placement/order among other unit metadata does not create runtime initialization effects.

Target semantics:

- ordinary source module → current `Module`;
- standalone/nested `package.ph` → current `Package`;
- Project root `package.ph` → current `Project` object.

Because `Project : Package : Module`, the Project root still has one target object; metadata is not duplicated among separate “facets.”

Metadata is inert by default. Unknown valid unit metadata remains attached as data. A metadata spelling acquires compiler/runtime semantics only through an explicitly standardized attribute definition. Standardized unit attributes declare the unit kinds on which they are valid; applicability may respect the `Project : Package : Module` subtype relation where the attribute definition says so. Hard-coded name tables that silently make arbitrary strings “package-only” are prohibited.

## 15. Context intrinsics: __module__, __package__, __project__

```
__module__  : Module
__package__ : Option<Package>
__project__ : Option<Project>
```

### 15.1 Semantics

- Lexical/definition-context semantics: a method or closure refers to the module/package/project in which it was defined, not the caller’s context.

- __module__ always exists because standalone/inline execution still materializes a Module object.

- __package__ is None only for genuine module contexts without package ownership.

- __project__ is None for standalone packages/modules/inline contexts without project ownership.

- Option expresses structural absence, never “not initialized yet.” Structural objects are created before initialization.

### 15.2 Root project package.ph

```
__module__  == project_object
__package__ == Some(project_object)
__project__ == Some(project_object)
```

### 15.3 Standalone package package.ph

```
__module__  == package_object
__package__ == Some(package_object)
__project__ == None
```

## 16. Reserved dunder namespace and compiler-controlled protocol hooks

The complete `__name__`-shaped namespace is owned by the language. This reservation is enforced by the compiler/source validator; the runtime symbol interner, selector representation, dispatch machinery, and reflective model remain capable of representing such names.

This distinction is intentional: reserving the source namespace protects developer experience and future language evolution without baking an artificial string restriction into runtime semantics.

### 16.1 Dunder policy categories

The compiler maintains one authoritative policy table for standardized dunder names and permitted declaration roles.

Conceptually:

```rust
enum DunderPolicy {
    IntrinsicOnly,
    RuntimeGuaranteed,
    UserImplementable(AllowedDeclarationRoles),
    UserOverridable(AllowedDeclarationRoles),
}
```

The exact enum is implementation-defined.

Categories:

1. **Intrinsic/context-only** — for example `__module__`, `__package__`, `__project__`. User source cannot declare, shadow, import-alias, export, parameter-name, field-name, or method-define these names.
2. **Guaranteed reflection/runtime protocol** — for example `__name__`, `__exports__`, `__selectors__`. These remain collision-free and non-overridable unless a later language decision explicitly changes the policy.
3. **Standardized user-implementable/overridable hooks** — a future hook such as `__intercept__` may be added here if message interception is ratified. The name remains language-reserved, but the compiler specifically permits a method implementation/override in the declared contexts. Permission does not automatically allow using the same dunder as a local variable, import alias, export name, field, etc.
4. **Unknown dunder** — rejected by the compiler. This reserves future protocol space.

A dunder becomes user-definable only through an explicit language/compiler policy entry. There is no general escape hatch such as “all dunders may be defined if quoted.”

Keep `_$name` distinct: `_$` is the compiler/VM floor/internal primitive lane, while dunder is the language-owned context/reflection/protocol lane. Do not copy Python’s broad operator-overload dunder convention; Phalcom already has selectors/protocol dispatch for ordinary behavior.

## 17. Reflection protocol

### 17.1 First-class Selector

Introduce a language/runtime Selector value capable of representing complete Phalcom selector shape. Reflection must never collapse selector identity to a base method name.

```
class Selector {
    name
    kind
    positionalCount
    labels
    // canonical representation / rest-family metadata as required
}
```

Selector belongs in universe.reflection and is not initially prelude-visible.

### 17.2 Module protocol

```
module.__name__       -> String/Symbol (ratify exact scalar during API implementation)
module.__id__         -> stable reflective module identity value
module.__path__       -> logical module path
module.__exports__    -> Set<Symbol>
module.__export__(name: Symbol) -> Option<Object>
module.__package__    -> Option<Package>
module.__project__    -> Option<Project>
module.__metadata__   -> metadata view
module.__understands__(selector: Selector) -> Bool
```

__path__ is logical, not a physical filesystem path. Source-location reflection may be added separately later for tooling/debug builds.

### 17.3 Package protocol

```
package.__parent__   -> Option<Package>
package.__children__ -> Set<Symbol>   // exposed/logical children, not raw filesystem listing
```

Package inherits all Module reflection. __children__ must respect package exposure rules and must not reveal private/internal filesystem structure.

### 17.4 Project protocol

```
project.__namespace__    -> Symbol/String
project.__dependencies__ -> reflective read-only dependency view
project.__version__      -> Option<Version> or equivalent metadata type
project.__metadata__     -> read-only project metadata view
```

Avoid exposing arbitrary source-root filesystem paths as a guaranteed runtime API. Such data may be unavailable in packaged/embedded environments.

## 18. Exports versus selectors

Exports and class methods are different reflective dimensions. module.__exports__ enumerates public export names. Class selector reflection enumerates dispatchable method selectors. A module export may contain a callable object without becoming a Method installed on Module.

### 18.1 Class/Behavior reflection

```
SomeClass.__selectors__             // all instance selectors understood, including inherited
SomeClass.__definedSelectors__      // directly defined instance selectors
SomeClass.__classSelectors__        // all class-side selectors understood
SomeClass.__definedClassSelectors__ // directly defined class-side selectors
obj.__understands__(selector)       // ordinary dispatch capability
```

### 18.2 Module dispatch capability

module.__understands__(selector) follows the actual export-first Module dispatch algorithm. Therefore it can be true because an export handles the selector or because the Module/Package/Project class hierarchy does. It must preserve Getter versus Method(0) and labeled/rest shape exactly.

## 19. Ordinary names and collision policy

Keep exports first for ordinary names. This protects existing/user export surfaces from future additions to Module/Package/Project ordinary APIs. Framework/tooling code uses dunder reflection for guaranteed access.

```
config.name       // may be an exported binding named "name"
config.__name__   // always language-defined reflection
```

Ordinary convenience aliases such as module.name may exist, but they are explicitly non-guaranteed under export shadowing. The dunder form is canonical.

## 20. Import and export semantics for Projects

Because Project is Package, no separate project export system exists. The project root package.ph declares the root public surface using ordinary package/module exports. A dependency import root is simply the dependency Project object/package surface.

```
// dependency Project `json`
import json
import json.parser

// root project package.ph may re-export:
export Parser
export encoder
```

Publication exposes the Project’s package surface; it does not manufacture a second distribution namespace.

## 21. Static package/project graph rules

- Every Project has exactly one root Package identity: itself.

- Nested Packages have exactly one parent Package and at most one owning Project.

- A Module has at most one owning Package and at most one owning Project.

- External dependency edges originate only from Project nodes.

- Packages organize source/API surface; Projects distribute/version/build.

- Project dependency graphs and module dependency graphs remain conceptually distinct: project dependencies make import roots available; module imports create static module dependency edges within the resolved universe.

## 22. Builtin universe graph, semantic SCCs, and runtime-cycle constraints

Phalcom supports semantic declaration SCCs in Modules v1. The semantic graph and runtime initialization graph therefore have deliberately different cycle rules.

### 22.1 Semantic declaration SCC realization

For a semantic SCC containing mutually referential declarations:

```text
Phase A  predeclare/materialize all declaration shells in the SCC
Phase B  resolve canonical static references among those shells
Phase C  validate edge-specific constraints and realize declaration bodies
```

Qualified references preserve linked declaration identity throughout; they may not fall back to leaf-name/current-module lookup.

Semantic SCC support is especially important before splitting `core.ph`, because class identities/reopens and other declaration relationships that currently coexist in one file will cross real `universe` module boundaries.

### 22.2 Cycles that remain illegal

Semantic SCC support does **not** imply general cyclic execution.

- The runtime module-initialization dependency graph remains acyclic and is executed in topological order.
- Class superclass/inheritance relationships remain acyclic. `class A is B` plus `class B is A` is invalid even if both declarations occur in one semantic SCC.
- Kernel object-model cycles remain Phase 0/1 bootstrap facts, not module-import cycles.

New universe source modules should still be partitioned so their **runtime initializer graph** can be topologically initialized. If an irreducible runtime cycle appears, move the primitive identity/floor into kernel bootstrap rather than adding a privileged cyclic module evaluator.

## 23. Migration from current runtime bootstrap

| Current behavior | Target behavior |
| --- | --- |
| VM::install_core creates ModuleId::core and globals for kernel classes | VM creates builtin Project universe and its root package/module graph identity |
| Runtime creates a builtin package named universe and copies class bindings into it | universe is the actual builtin Project root; exports are linked interfaces, not a copied convenience namespace |
| UNIVERSE_BINDINGS has exported/prelude flags | Retain/expand declarative catalog as seed metadata, but map entries to universe module/package exports and curated prelude |
| run_core_module include_str!(core.ph) | Link/materialize/initialize universe module DAG |
| After core.ph, every core global is inserted into prelude_names | Delete this scan; prelude derives solely from explicit curated data |
| ImportRootTarget::Core / ModuleId::core | BuiltinProject::Universe |

## 24. Implementation phases

| Phase | Work | Exit condition |
| --- | --- | --- |
| A — Identity foundation | Land ProjectIdentity/BuiltinProject/SyntheticProjectId; fix resolver root model | universe/std/user/synthetic roots cannot alias |
| B — Ownership/naming/provider foundation | Enforce Project/Package/Module ownership; snake_case↔kebab-case mapping; BuiltinProjectSourceProvider; virtual builtin IDs | Source authority and builtin identities are canonical |
| C — Static declaration gate | Populate declaration blueprints; canonical qualified references; declaration shells; semantic SCC realization | Cross-module class/declaration identity works without leaf-name fallback |
| D — Runtime hierarchy | Add/complete Project class; make Package < Module and Project < Package metadata/constructors authoritative | One object is both project and root package |
| E — Context/reflection floor | Compiler-controlled dunder policies; lexical context intrinsics; minimal Module/Package/Project dunder reflection | Context works before initializer execution and future hook policy is extensible |
| F — Universe bootstrap shell | Create builtin universe Project graph/materialization path while still using legacy `core.ph` content temporarily | No resolver special-case core source needed; full-pipeline builtin root test green |
| G — Split core.ph | Inventory/move source into universe modules and std where appropriate; add package surfaces/import/static-reference graph | Legacy core.ph empty/deleted; cross-module kernel reopens/classes preserve identity |
| H — Prelude cleanup | Explicit projection only; remove bootstrap-global scan | Prelude membership deterministic and tested |
| I — std project | Create builtin std root and initial packages; make available in all entry contexts | std resolves through ordinary module graph |
| J — Selector reflection | First-class Selector and class/module selector APIs | Reflection preserves exact dispatch shape |
| K — Project manifests + unit metadata | Require snake_case namespace; project-only dependencies/version/publication; `@!attribute(...)`; nested-project diagnostics | Project/source-unit surface is explicit and stable |
| L — Cleanup | Rename internal Universe, remove core aliases/dead bootstrap code, update docs/LSP/tests | No public “core” namespace or duplicate root package abstraction remains |

## 25. Detailed test plan

| Test | Behavior |
| --- | --- |
| MODEL-01 | Project object is instance/subclass relationship Package → Module as specified. |
| MODEL-02 | Project root package.ph has __module__, __package__, __project__ all referring to the Project object as specified. |
| MODEL-03 | Standalone package has __project__ == None. |
| MODEL-04 | Standalone module has __package__ == None and __project__ == None. |
| MODEL-05 | Method/closure captures lexical definition module/project, not caller context. |
| ROOT-01 | universe resolves in project, standalone package, standalone module, and REPL. |
| ROOT-02 | std resolves in all four contexts. |
| ROOT-03 | User project dependency does not resolve in standalone context. |
| ROOT-04 | `universe` import completes full resolve/discover/link/materialize/initialize/execute pipeline. |
| ROOT-05 | Builtin module `universe.reflection.selector` maps to `phalcom://universe/reflection/selector`; `std.json` maps to `phalcom://std/json`. |
| CORE-01 | No public import core resolves after compatibility removal. |
| CORE-02 | `universe.Object` and every required builtin identity resolves from actual universe Project exports. |
| CORE-03 | Transitional `core` (if present) is a full alias or deliberate early error; `core.foo` never collapses to the root. |
| BOOT-01 | Universe source graph initializes topologically without privileged module cycles. |
| BOOT-02 | Kernel class reopens/wrappers from split source attach to original kernel class identities. |
| BOOT-03 | Cross-universe-module superclass/reference uses canonical linked declaration identity, never leaf-only lookup. |
| SCC-01 | Legal semantic declaration SCC is predeclared and realized successfully. |
| SCC-02 | Inheritance cycle remains illegal even when declarations belong to one semantic SCC. |
| SCC-03 | Semantic SCC does not permit runtime module-initialization cycle. |
| PRE-01 | Only curated universe bindings appear through prelude fallback. |
| PRE-02 | A universe export not in prelude requires explicit universe access/import. |
| PRE-03 | Local declaration shadows prelude binding. |
| PRE-04 | export Name does not re-export an unimported prelude binding. |
| NONE-01 | Prelude None is immediate absence; universe.None is None class object. |
| PROJ-01 | Manifest name and namespace remain independent. |
| PROJ-02 | Persistent project missing namespace receives validation diagnostic. |
| PROJ-03 | Nested project manifest under project source root rejected outside workspace semantics. |
| PROJ-04 | Only Project manifests can declare external dependencies/version/publication fields. |
| NAME-01 | `namespace = "geometry_toolkit"` and dependency/import-root aliases accepted when canonical snake_case; logical kebab/mixed-case forms rejected. |
| NAME-02 | Physical `package-a/module-b.ph` maps to logical `package_a.module_b`; physical snake_case/mixed-case component rejected. |
| NAME-03 | Project directory name cannot override explicit manifest namespace. |
| PKG-01 | `main.ph` without `package.ph` does not create Package identity. |
| EXP-01 | Project root exports are ordinary package exports; dependency root observes same surface. |
| REF-01 | Unknown/non-overridable dunder declarations/import aliases/exports are rejected by compiler policy. |
| REF-01A | Runtime selector/symbol machinery can represent a reserved dunder without imposing a string-level runtime ban. |
| REF-01B | A fixture-only standardized overridable dunder hook is accepted only in its authorized method declaration/override role; the same name remains forbidden as unrelated local/import/export/field syntax. |
| REF-02 | _$ internal selector lane remains separate and inaccessible per existing internal-access rules. |
| REF-03 | module.__exports__ reports only public exports. |
| REF-04 | module.__export__(symbol) returns Some(value) or None without exposing private bindings. |
| REF-05 | `module.__understands__` agrees with actual export-first send behavior. |
| META-01 | `@!documentation(...)` parses as declarative unit-header metadata and attaches to the current Module/Package/Project unit; separated `@!`/identifier forms are rejected. |
| META-02 | Project-root `@!` metadata appears once on the Project object, not duplicate Package/Module objects. |
| META-03 | Unknown valid unit metadata is inert; standardized semantic metadata is validated through explicit attribute definition, not magic spelling. |
| SEL-01 | Selector distinguishes Getter(size) from Method(size,0). |
| SEL-02 | Selector preserves labels and rest-family shape. |
| SEL-03 | __selectors__/__definedSelectors__ inherited/direct distinction correct. |
| SHADOW-01 | Export named `name` shadows ordinary module.name alias; `module.__name__` remains stable. |
| SHADOW-02 | Package ordinary-name export collision does not affect guaranteed non-overridable dunder reflection. |
| SHADOW-03 | Project ordinary-name export collision does not affect guaranteed non-overridable dunder reflection. |
| STD-01 | std package imports use ordinary module dependency preamble and exposure rules. |

### 25.1 Required migration fixtures and integration routes

Use stable repository fixtures that exercise the architecture through public paths, not only unit helpers. Recommended layout:

```text
fixtures/universe-v1/
  project-hierarchy/
    project.toml                 # name + namespace = "geometry_toolkit"
    src/
      package.ph
      package-a/
        package.ph
        module-b.ph

  unit-metadata/
    project.toml
    src/
      package.ph                 # @!documentation(...) on Project
      child/
        package.ph               # @!documentation(...) on Package
      module-a.ph                # @!documentation(...) on Module

  dunder-policy/
    forbidden-unknown.ph
    forbidden-reflection-override.ph
    allowed-hook-override.ph     # compiler test fixture with an explicitly registered test-only hook policy

  semantic-scc/
    project.toml
    src/
      package.ph
      a.ph
      b.ph                       # legal mutual semantic references

  inheritance-cycle/
    project.toml
    src/
      package.ph
      a.ph
      b.ph                       # explicit illegal superclass cycle

  builtin-client/
    project.toml
    src/
      package.ph
      main.ph                    # imports universe.reflection.selector / std as needed
```

Builtin-project integration additionally uses the actual toolchain `universe`/`std` source trees as fixtures. Do not create a second simplified “test universe” that bypasses the real builtin provider.

Required integration routes:

1. **Bootstrap integration**: construct a VM from scratch, resolve/link/materialize the real builtin `universe`, realize declaration SCCs/shells, reopen canonical kernel classes, initialize the runtime DAG, and observe known exports.
2. **Project model integration**: execute `project-hierarchy` and assert the same root object satisfies `Project`, `Package`, and `Module` relationships plus exact `__module__`/`__package__`/`__project__` values.
3. **Naming integration**: resolve physical `package-a/module-b.ph` through the production source provider and assert logical `geometry_toolkit.package_a.module_b`; invalid alternate spellings must fail before fallback probing.
4. **Metadata integration**: parse/compile/runtime-reflect `@!documentation(...)` on Module, Package, and Project units; assert one attachment to the current unit object and inert handling of unknown metadata.
5. **Dunder compiler-policy integration**: prove runtime representation accepts dunder selectors while source compilation rejects unknown/non-overridable dunders and admits only a test-registered allowed hook in the exact authorized declaration role. This does not ratify `__intercept__`; it tests the policy mechanism.
6. **LSP virtual-document integration**: open/navigate definitions in the real builtin graph and assert canonical URIs such as `phalcom://universe/reflection/selector`; no host installation path may leak into semantic document identity.
7. **Compatibility integration**: while `core` compatibility exists, verify full-pipeline aliasing or deliberate early rejection, then delete the compatibility fixtures when `core` is removed.

Every test that claims dispatch, initialization, identity, or reflection semantics must assert the resulting value/object/selector/counter, not merely successful execution.

## 26. Documentation and tooling migration

- Update module specification terminology so “project root package” means the Project object itself.

- Update LSP semantic model and hover/navigation to show Module/Package/Project runtime/static relationships and open builtin definitions through canonical `phalcom://...` virtual documents.

- Update phaldoc design later to treat Project as the published root Package and document package/module exports through one surface model.

- Update tutorials to teach `universe` versus prelude versus `std`, logical snake_case module paths versus physical kebab-case source paths, and `@!attribute(...)` unit metadata explicitly.

- Update internal docs that use core module/core globals where they actually mean the language-level universe Project.

- Preserve “core class/kernel core” terminology only where it refers to VM bootstrap internals.

## 27. Explicit future work, not part of v1

- Workspaces: multi-Project coordination, shared lockfile/build graph, explicit member roots.

- Independent standard-library versioning or alternate std implementations.

- Rich Export descriptor objects beyond Set<Symbol> + __export__.

- Runtime physical source-path reflection for packaged/embedded programs.

- Full VM-independent compiled ModuleArtifact format.

- Package-level independent versioning/dependencies (intentionally excluded unless future evidence overturns the Project boundary).

## 28. Acceptance criteria

- No user-visible core import root remains.

- universe and std are represented as builtin Projects and resolve through the shared module resolver.

- The Project runtime/static model is a specialization of Package, itself a specialization of Module.

- No duplicate root Package object exists for a Project.

- Only Projects own external dependencies, version/publication identity, and global namespace roots.

- core.ph has been deleted or reduced to no executable language surface; its contents are partitioned into universe/std modules.

- Prelude membership is declarative and no bootstrap-global scan expands it.

- __module__/__package__/__project__ are lexical, immutable, structurally available before initialization, and Option-valued only for actual absence.

- The dunder namespace is language-reserved and compiler-controlled: non-overridable context/reflection names remain collision-free; only explicitly standardized hook names may permit user implementation/override in defined declaration roles.

- Module exports and class selectors are reflectively distinct; Selector preserves complete dispatch shape.

- Standalone/REPL contexts can access universe/std without receiving accidental project dependency/filesystem authority.

- Semantic declaration SCCs are supported by declaration-shell predeclaration/materialization, while runtime initialization and inheritance cycles remain independently prohibited.

- Cross-module static declaration/class references preserve canonical linked identity; splitting `core.ph` never relies on same-leaf-name lookup.

- `package.ph` establishes Package identity; `main.ph` alone does not.

- Logical Project/package/module paths use canonical snake_case, physical package/module source components use canonical kebab-case, and the resolver mapping is reversible and alias-free.

- Unit metadata uses contiguous `@!attribute(...)` syntax and targets the current source-unit object; unknown metadata is inert.

- Builtin documents use stable logical `phalcom://universe/...` and `phalcom://std/...` URIs.

- Compiler/runtime/LSP/docs use the same ownership and identity vocabulary.

## 29. Files/subsystems expected to change

| Subsystem | Likely implementation areas |
| --- | --- |
| Builtin catalog/kernel | phalcom-native-meta/src/universe.rs; phalcom-core/src/universe/*; native install/bootstrap code |
| VM bootstrap | phalcom-core/src/vm/bootstrap.rs; remove run_core_module path and bootstrap-global prelude scan |
| Module identity/resolver | phalcom-modules/src/{identity,project,resolver,source}.rs |
| Runtime objects | ModuleObject/Package support plus new Project object/class representation; heap/object model and GC rooting |
| Compiler | Lexical context intrinsics; compiler-controlled dunder protocol policy; `@!` unit metadata parsing/lowering; snake↔kebab validation; project/package unit-kind lowering; canonical static declaration references/SCC realization |
| Dispatch/reflection | Selector representation; Module export send; class selector enumeration; __understands__ |
| Source tree | Replace phalcom-core/core/core.ph with builtin universe project/module graph; create std project tree |
| Manifest | Require namespace; project-only dependency/version/publication validation |
| CLI/REPL | Builtin root availability independent of user project context |
| LSP | Shared project/package/module hierarchy, resolved-document ingestion, canonical `phalcom://` builtin-root navigation/virtual documents |
| Tests/docs | Bootstrap, semantic-SCC/cross-module-class, reflection/dunder-hook, unit-metadata, naming/layout, project ownership, prelude, std, standalone, virtual-builtin-document, migration fixtures |
