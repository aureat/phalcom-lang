# Review of Phalcom’s Future Module Specification

I would not implement this specification exactly as written.

The overall architecture is unusually strong: logical rather than physical imports, deterministic project universes, explicit package boundaries, private-by-default module bindings, explicit façades, stable execution/import identity, live selective-import bindings, and a compiler-visible module graph all point in the right direction. Several of these choices directly avoid long-lived problems in Python and other mature ecosystems. The specification is also internally much more coherent than most language module systems at their first serious design stage.

But there are a handful of decisions that I would change **before they become observable language semantics**. The most important are not syntax. They concern module-path privacy, initialization timing, package initialization, concurrent initialization, module-object dispatch, dependency-instance identity, and the distinction between interface dependencies and runtime dependencies. Those are precisely the kinds of decisions that become extraordinarily expensive to reverse after packages exist in the wild.

My high-level assessment is:

| Area | Assessment | Change before implementation? | Cost of changing later |
|---|---|---:|---:|
| Logical imports / no filesystem imports | Excellent | No | Very high |
| No ambient search path | Excellent | No | Very high |
| Stable module identity / no `__main__` duplicate | Excellent | No | Extreme |
| Explicit packages | Strong | Mostly no | High |
| Private-by-default bindings | Excellent | No | High |
| Explicit package façades | Excellent | No | High |
| No implicit child-module lookup | Excellent | No | High |
| Live selective imports | Strong | Clarify | High |
| Path-level module privacy | **Missing** | **Yes** | **Extreme** |
| Source-position initialization | **Problematic** | **Yes** | **Extreme** |
| Ancestor `package.ph` execution | **Problematic** | **Yes** | **Extreme** |
| Concurrent initialization semantics | **Missing** | **Yes** | **Very high** |
| Module-object/export namespace collision | **Missing** | **Yes** | **Very high** |
| `ProjectInstanceId` semantics | Underspecified | **Yes** | **Very high** |
| Type/interface-only dependencies | **Missing** | **Yes architecturally** | High |
| Conditional import graph | Missing | Design hook now | Medium/high |
| Sticky failed initialization | Reasonable | Clarify | Medium |
| Standalone-package identity | Mostly good | Clarify | Medium |
| Filesystem canonicalization | Good goal, too physical | Refactor abstraction | Medium |
| `export ... from` shorthand | Fine to defer | No | Low |
| Multiple executables | Fine to defer | No | Low |
| Wildcards | Correctly rejected | No | Low |
| Namespace packages | Correctly rejected | No | Very high |

The rest of this review explains why.

---

# 1. The strongest parts should remain essentially untouched

Before criticizing the dangerous parts, several decisions deserve to be locked down.

## 1.1 Logical import identity is exactly the right foundation

The distinction:

```text
CLI/source selection → physical path
language import       → logical identity
```

is excellent. So is making the project namespace independent of the checkout directory. The specification correctly refuses things like:

```phalcom
import "./foo.ph"
import "../foo"
```

and instead resolves:

```phalcom
import app.foo
import .foo
```

through a semantic project/package universe.

Do not weaken this later with an escape hatch like Python's mutable import paths. Python's import machinery explicitly searches an import path and permits extensible finders/loaders; `sys.modules` and `sys.path` therefore participate materially in runtime import semantics.  Phalcom's explicit dependency universe is much cleaner for compilation, LSP analysis, reproducibility, sandboxing, package management, and future AOT compilation.

This is one of the decisions that should be treated as architectural bedrock.

---

## 1.2 Eliminating the `__main__` dual identity is an exceptionally good decision

The invariant:

```text
entry module identity = imported module identity
```

should remain absolute.

Python has accumulated multiple PEPs around the awkward interaction between executable modules, package-relative imports, and module qualified names. PEP 366 was specifically introduced so explicit relative imports could work from executable modules, while later proposals continued discussing qualified-name problems around main modules.

Phalcom's model is much cleaner:

```text
app.tools.demo
```

is `app.tools.demo` whether selected as the process entry point or reached through an import.

Do not introduce a magic second module namespace later.

---

## 1.3 `resolution namespace ≠ public runtime namespace` is excellent

Sections 46–48 are among the best pieces of the design.

The filesystem may contain:

```text
geometry/
├── package.ph
├── point.ph
└── vector.ph
```

without causing:

```phalcom
geometry.point
geometry.vector
```

to magically become runtime members of the `geometry` package object.

That separation avoids a Python behavior where loading a submodule creates a corresponding binding on its parent package object; Python documents this as an import-system invariant.

Phalcom instead says:

```phalcom
import .point as point
export point
```

if the package actually wants `point` in its public runtime namespace.

Keep that.

---

## 1.4 Private-by-default bindings plus explicit re-export is the right API model

This:

```phalcom
class Point {}
const origin = Point(0, 0)
const cache = ...

export Point, origin
```

is substantially better than convention-based privacy.

Likewise:

```phalcom
from .point import Point
```

not implicitly re-exporting `Point` is the right choice.

Rust's `pub use` model is valuable precisely because the public API can be decoupled from the internal module organization. The Rust documentation explicitly recommends re-exports as a way of presenting an API structure different from internal code structure.

Phalcom's `package.ph` façade model has the same valuable property.

There is, however, one missing half of that story.

---

# 2. Critical issue: do not defer module-path privacy

This is the strongest recommendation in this review.

Section 49 currently says that binding privacy exists, but module-path privacy does not. Consequently, given:

```text
geometry/
├── package.ph
└── internals/
    ├── package.ph
    └── cache.ph
```

a consumer may still write:

```phalcom
import geometry.internals.cache
```

provided it knows the path.

That makes the package façade only a **convenience API**, not an actual encapsulation boundary.

That will become a compatibility trap.

Suppose version 1 ships:

```text
geometry.internal.matrix_cache
```

and someone discovers:

```phalcom
from geometry.internal.matrix_cache import FastMatrixCache
```

Even if the library author never documented it, users will depend on it. Package search engines, autocomplete, AI tooling, code search, examples, and Stack Overflow equivalents will propagate it.

Then version 2 wants:

```text
geometry._engine.cache
```

The library author has three choices:

1. break downstream source;
2. preserve the old module forever;
3. create compatibility forwarding modules indefinitely.

This is exactly the kind of ecosystem debt you can prevent now.

Node's modern package `"exports"` mechanism explicitly encapsulates package subpaths; Node's documentation points out that this gives stronger guarantees about public interfaces and semver-compatible package evolution.

Go added `internal` packages because exported-versus-unexported identifiers were insufficient to prevent external clients from depending on implementation components. Its own release notes explain that this was the motivating problem.

Java modules distinguish exported and non-exported packages.  Rust's visibility system similarly applies to module paths themselves, not merely to leaves inside them.

### What I would adopt

Make module-path accessibility a separate dimension from binding accessibility:

```text
Module path visibility
    ≠
Module binding visibility
```

Within the same project, internal module paths can remain freely addressable.

Across project boundaries, a dependency should expose only:

- its root package;
- explicitly public submodule paths;
- explicitly public subpackage paths.

Conceptually:

```phalcom
// geometry/package.ph

public module .point
public package .shapes
```

The exact syntax is negotiable. The semantics are not.

You could instead use something like:

```phalcom
export module .point
export package .shapes
```

although overloading `export` between binding exports and path exports may become visually confusing.

The important invariant should be:

```text
External project:
    can resolve dependency root
    can resolve explicitly exposed paths
    cannot resolve arbitrary implementation paths

Same project:
    can resolve its own internal paths
```

Then:

```phalcom
from geometry import Point
```

can expose `Point` even if its defining module:

```text
geometry.internal.cartesian
```

is itself inaccessible externally.

That gives Phalcom real façade encapsulation.

### New diagnostic

Add:

```text
ModulePathNotExportedError
```

with a diagnostic such as:

```text
ModulePathNotExportedError:
  `geometry.internal.cache` exists but is not part of
  project `geometry-kit`'s public module surface.

  Public import roots include:
    geometry
    geometry.point
    geometry.shapes
```

This needs to exist before the first third-party package ecosystem.

---

# 3. Critical issue: source-position initialization is fighting the static module model

Sections 29–31 define a curious hybrid:

```text
imports are static module-scope declarations
+
the complete graph is discoverable before execution
+
imports initialize at their textual source position
```

For example:

```phalcom
System.print("before")

import app.geometry.point

System.print("after")
```

means:

```text
print before
initialize point
print after
```

even though the compiler already knows about `point` before execution.

I think this is the wrong compromise.

It combines the implementation burdens of a static module system with many of the semantic hazards of executable Python imports.

## 3.1 Moving an import changes program behavior

Consider:

```phalcom
System.configureLogging()

import app.server
```

versus:

```phalcom
import app.server

System.configureLogging()
```

If `app.server` performs initialization that observes logging configuration, these programs behave differently.

That means imports are not really declarations. They are effectful initialization statements.

This has consequences for:

- formatter import sorting;
- automatic import organization;
- refactoring;
- unused-import removal;
- incremental compilation;
- module-cycle diagnostics;
- dependency visualization;
- code motion;
- top-level optimization;
- future dead-code elimination.

An IDE cannot freely move an import to the conventional top-of-file import section because it might alter initialization semantics.

An "unused" import might actually be:

```phalcom
import app.telemetry
```

solely for initialization effects, so the compiler cannot remove it.

Textual import placement becomes an invisible control-flow mechanism.

That is undesirable in a module system explicitly designed to be statically analyzable.

---

# 4. I would make static imports a module preamble

Instead, use the static nature of the model fully.

Conceptually:

```phalcom
from .config import Config
import .logging as logging
import json.parser as parser

// imports end here

const config = Config.load(...)
...
```

Static imports should form a module dependency preamble.

The semantics become:

```text
resolve
    ↓
discover interfaces
    ↓
link imports/exports
    ↓
initialize required runtime dependencies
    ↓
execute module body
```

For acyclic dependencies:

```text
A imports B
B imports C
```

initialization becomes:

```text
C
B
A
```

independent of whether `A` wrote its `import B` on line 1 or line 200.

Go follows dependency-directed package initialization: imported packages initialize before the importer, and it prohibits import cycles entirely.  ECMAScript modules explicitly separate module loading/linking/environment initialization from execution and have dedicated cyclic-module evaluation machinery.

Phalcom can keep its richer cycle semantics while adopting the same separation.

### What about delayed loading?

Make delayed loading a different operation.

Future Phalcom can have:

```phalcom
Module.load(...)
```

or some other reflective API.

That operation means:

> perform runtime module acquisition now.

Static:

```phalcom
import .foo
```

should mean:

> this module has a static dependency on `.foo`.

Those are different concepts and should not use the same mechanism.

Interestingly, even Python continues to feel pressure around import-time startup costs; Python 3.15 is introducing explicit lazy-import functionality rather than changing ordinary import semantics retroactively.  Phalcom can leave a clean semantic slot for laziness from the beginning.

---

# 5. Critical issue: ancestor `package.ph` initialization creates an eager-loading bomb

Section 56 says importing:

```text
geometry.shapes.circle
```

initializes:

```text
geometry/package.ph
geometry/shapes/package.ph
geometry/shapes/circle.ph
```

in that order.

This is Python-like. Python likewise executes a package's `__init__.py` when importing a submodule.

But in Phalcom this conflicts badly with `package.ph`'s second purpose: being the package façade.

The specification's own complete example demonstrates the problem.

Root `package.ph` is:

```phalcom
from .point import Point, origin
from .vector import Vector
from .shapes import Shape, Circle

export Point, origin, Vector, Shape, Circle
```

Now a user writes:

```phalcom
import geometry.point
```

Under §56, Phalcom first initializes `geometry/package.ph`.

But initializing that package imports:

```text
geometry.point
geometry.vector
geometry.shapes
```

and `geometry.shapes/package.ph` then imports:

```text
geometry.shapes.base
geometry.shapes.circle
```

and `circle.ph` imports `geometry.point`.

So requesting one leaf:

```text
geometry.point
```

can initialize effectively the entire library.

That is not theoretical. It falls directly out of the proposed semantics.

## 5.1 It also manufactures import cycles

The package façade wants to re-export children:

```text
package → child
```

while child resolution requires ancestor package initialization:

```text
child → package
```

So façade construction itself creates cycles.

Your binding-cell cycle machinery can make many of those cycles survivable, but the module system should not gratuitously manufacture them.

---

# 6. Separate package existence from package execution

I recommend this instead:

> Resolving a child module requires its package chain to be discovered, declared, and registered, but does not inherently execute the ancestor package modules.

Then:

```phalcom
import geometry.shapes.circle
```

does approximately:

```text
discover geometry
discover geometry.shapes
discover geometry.shapes.circle

initialize dependencies of geometry.shapes.circle
initialize geometry.shapes.circle
```

It does **not** automatically execute:

```text
geometry/package.ph
geometry/shapes/package.ph
```

merely because they are ancestors.

If code actually imports the package:

```phalcom
import geometry
```

then `geometry/package.ph` initializes normally.

If a child explicitly needs something exported by its parent:

```phalcom
from . import Config
```

then that is a real runtime dependency on the package module and initializes it.

This distinction is powerful:

```text
package hierarchy
    = resolution structure

package.ph execution
    = runtime dependency
```

The package remains semantically a module. The only thing being removed is the assumption that lexical containment implies an initialization dependency.

I would change §56 before implementation.

---

# 7. The cycle model is conceptually strong, but one word is dangerously underspecified

Sections 51–62 contain a very good idea: interfaces and export cells exist before runtime initialization.

This:

```text
Declared export
      ↓
BindingCell(uninitialized)
      ↓
eventually initialized value
```

is substantially cleaner than trying to infer whether a partially initialized module "has" a member.

ECMAScript uses immutable indirect import bindings and explicit module-environment/link/evaluation machinery, which is conceptually close to Phalcom's live cell model.

But §60 says:

> Phalcom **may** establish the live import binding without immediately forcing the current runtime value.

"May" cannot remain in final semantics.

That needs to become normative.

I recommend:

```text
Selective import linkage MUST NOT read the exported value.

It resolves an exported SymbolId and establishes an immutable
reference to that symbol's BindingCell.

A runtime read of that binding reads the cell.

If the cell is Uninitialized:
    UninitializedModuleBindingError
```

Thus:

```phalcom
from .a import A
```

during a cycle is itself harmless.

This is what forces:

```phalcom
const x = A.new()
```

and this is where an uninitialized read can fail.

Likewise:

```phalcom
export InternalPoint as Point
```

links the same cell; it does not read the cell.

That gives Phalcom extremely crisp cycle semantics.

---

# 8. Critical missing piece: concurrent module initialization

The specification defines:

```text
Initializing
Initialized
Failed
```

but not what happens when multiple fibers/tasks/threads concurrently encounter the same not-yet-initialized module.

That omission cannot survive Phalcom's concurrency model.

Suppose:

```text
Fiber 1: import app.a
Fiber 2: import app.a
```

or worse:

```text
Fiber 1: initialize A → import B
Fiber 2: initialize B → import A
```

A naive "mutex per module" design can deadlock.

Python itself continues to have to address module-lock interactions; its development history includes fixes around deadlocks caused by concurrent imports of parents and submodules.

Phalcom should specify this at the semantic level now.

## 8.1 Recommended v1 rule

Do not parallelize first-time module initialization.

Use a runtime-wide scheduler-aware initialization coordinator.

Conceptually:

```text
ModuleInitCoordinator
    ↓
one initialization transaction at a time
```

The executing fiber may recursively initialize dependencies.

Other fibers requesting a module that belongs to the active transaction wait through a scheduler-aware future/condition rather than blocking an OS thread.

Conceptually:

```text
Fiber A
    import X
        owns initialization transaction
        X → Y → Z

Fiber B
    import X
        awaits X.initialization
```

A reentrant import from the **same initialization transaction** may see the registered partial module/cells for cycle resolution.

An unrelated concurrent fiber should not receive a half-initialized module.

That gives this important invariant:

```text
partial modules are visible to cycle resolution
inside an initialization transaction,

not generally observable by unrelated concurrent execution
```

Later, if module initialization proves worth parallelizing, the coordinator can evolve into SCC-aware graph scheduling.

Do not start with that complexity.

---

# 9. Define whether module initialization may suspend

This becomes particularly important with fibers.

Suppose top-level code calls something which suspends:

```phalcom
const config = Network.fetch(...)
```

Can module initialization yield its execution fiber?

I recommend yes, if ordinary Phalcom calls may suspend.

Then "synchronous import" should mean:

```text
the importing execution cannot proceed past the import dependency
until initialization completes
```

not:

```text
the operating-system thread may never yield
```

This is exactly the distinction a fiber runtime needs.

The module initialization coordinator therefore needs to be scheduler-aware.

---

# 10. Critical missing piece: `Module` methods can collide with exported names

This is particularly important for Phalcom because modules are ordinary runtime objects and member access uses the normal message/member-send mechanism.

The specification says:

```phalcom
import geometry.point as point

point.origin
point.distance(...)
```

is ordinary object/member dispatch.

But what happens if `Module` itself responds to:

```text
name
id
path
exports
inspect
hash
class
```

and the imported module exports:

```phalcom
const name = "geometry"
export name
```

What does:

```phalcom
point.name
```

mean?

The export?

Or `Module#name`?

This needs an answer before module objects become public runtime values.

It is especially dangerous because some collisions involve universal `Object` protocol selectors, not just `Module`-specific APIs.

## 10.1 Do not reserve arbitrary export names to solve this

I would not say:

```text
modules may not export `name`, `hash`, `class`, ...
```

That leaks implementation protocol into application namespace design and will grow over time.

Instead, distinguish the export namespace from module introspection.

Conceptually:

```text
Module value
   │
   ├── export namespace
   │      Point
   │      origin
   │      name
   │
   └── metaobject/reflection
          Module identity
          source
          package
          initialization state
```

Ordinary module-qualified member access should primarily mean **export lookup**.

Reflection should use an external API, for example conceptually:

```phalcom
Reflection.module(point).identity
Reflection.module(point).source
```

or class-side operations:

```phalcom
Module.identity(of: point)
```

rather than consuming instance selectors such as:

```phalcom
point.moduleId
```

ECMAScript went so far as to define module namespace objects as a special kind of object whose string-keyed properties correspond exactly to exported bindings, with specialized lookup/set/delete behavior.

Phalcom does not need to copy JS's object model, but it needs an equally explicit answer to this namespace collision problem.

This is a **hard-to-reverse runtime-object-model decision**.

---

# 11. `ProjectInstanceId` is the right abstraction but is not sufficiently defined

Section 21 says canonical project module identity is:

```text
ProjectInstanceId + ProjectRelativeModulePath
```

Excellent.

But:

```text
linear-algebra@2.0 / resolved instance
```

is only an illustrative description.

Before implementation, define what makes two project instances the same instance.

Consider:

```text
linear-algebra 2.0 from registry
linear-algebra 2.0 from Git revision X
linear-algebra 2.0 from ../local-copy
linear-algebra 2.0 patched to another source
```

Are these one instance or four?

Now consider:

```toml
[dependencies]
linA = { package = "linear-algebra", version = "2" }
linB = { package = "linear-algebra", version = "2" }
```

If the resolver deduplicates them to one dependency graph node, they should clearly refer to the same modules.

But if the resolver deliberately resolves two independent project instances, their class/type identities should presumably be different.

Cargo provides a useful caution here: when multiple versions of the same crate appear in a dependency graph, otherwise identically named types are different compiler identities, which can create incompatibilities when those types cross library boundaries.

Phalcom needs to make this explicit.

## 11.1 Recommended model

Internally:

```text
ResolvedProjectId   // opaque dependency-graph-node identity
ModuleId {
    project: ResolvedProjectId,
    path: ProjectRelativeModulePath
}
```

Do not define `ResolvedProjectId` as a user-visible string.

Its package-manager metadata can contain:

```text
distribution name
resolved version
source kind
source identity
content/revision identity
lockfile graph node
```

but identity itself should be opaque.

Reflection could expose:

```text
module.identity       // opaque stable runtime identity
module.project.name   // diagnostic metadata
module.project.version
module.path
```

without pretending:

```text
"linear-algebra@2.0.matrix"
```

is the semantic identity.

Most importantly, specify:

> Two dependency aliases resolving to the same resolved project node produce the same ModuleIds.

and:

> Two distinct resolved project nodes produce distinct ModuleIds, even if their project names, namespaces, versions, and relative paths happen to match.

This matters for:

- classes;
- generic specializations;
- protocols;
- exception classes;
- reflection;
- serialization;
- `is`/identity checks;
- type checking;
- native extension ABI boundaries.

---

# 12. Add a distinction between runtime dependencies and interface dependencies now

This is especially important given Phalcom's future typing plans.

The specification already anticipates:

```phalcom
import .base as base

const shape: base.Shape = ...
```

and says module interfaces are available before initialization.

But currently the only module edge is effectively:

```text
import → runtime initialization dependency
```

That will become problematic.

Suppose:

```phalcom
import .models as models

method(user: models.User) -> models.Profile {
    ...
}
```

If this import exists purely so the compiler can resolve type declarations, should loading the current module execute all of `models.ph`?

Probably not.

Python typing has accumulated substantial machinery around forward references and circular imports because runtime import relationships and annotation dependencies are not the same graph; the Python typing specification explicitly discusses cases where direct imports needed only for annotations become impossible because of circular imports.

Phalcom already has the architectural ingredients to do better.

## 12.1 Internally distinguish graph-edge kinds

At minimum:

```text
ImportEdge
├── InterfaceEdge
└── RuntimeEdge
```

Potentially later:

```text
ImportEdge
├── InterfaceEdge
├── RuntimeEdge
├── ReExportEdge
└── DynamicEdge
```

An interface edge says:

```text
I need this ModuleInterface/SymbolId during compilation.
I do not inherently require its top-level code to execute.
```

A runtime edge says:

```text
my initialization depends on this module's initialized values.
```

A superclass may eventually be both:

```phalcom
class Circle is base.Shape {}
```

because declaration resolution needs the interface and runtime class construction may require `Shape`'s initialized class object.

A pure annotation may be interface-only.

Whether the eventual syntax is:

```phalcom
import type .models as models
```

or something more Phalcom-specific can be deferred.

The **compiler graph representation cannot**.

---

# 13. Add a conditional-dependency hook before freezing `ImportEdge`

Another consequence of "all imports are statically known" is platform-dependent code.

You currently reject:

```phalcom
if System.windows {
    import .windows
}
```

correctly, because imports cannot appear in conditionals.

But then how does a cross-platform module say:

```text
Windows → implementation A
Linux   → implementation B
```

without resolving and compiling both modules for every target?

Rust's module system integrates conditional compilation through `cfg`.  Go similarly has build constraints for target-specific source selection.

You do not need to choose Phalcom syntax now, but the dependency graph should eventually support:

```text
ImportEdge {
    target: ModuleId,
    kind: Runtime,
    condition: BuildPredicate?
}
```

This will also matter for:

- optional dependencies;
- feature-selected implementations;
- native versus pure-Phalcom implementations;
- test-only dependencies;
- development-only tooling;
- architecture-specific modules.

Do not make the initial compiler graph representation assume every discovered edge is universally active.

---

# 14. Standalone package identity is intentionally less stable than project identity — say so

The specification opens with the principle:

> every source unit has one stable logical identity.

That is completely true inside a project.

It is only conditionally true for standalone packages.

Given:

```text
demo/
├── package.ph
└── tools/
    ├── package.ph
    └── inspect.ph
```

you derive:

```text
demo.tools.inspect
```

from filesystem names.

Rename:

```text
demo → demo2
```

and the module identity changes.

Worse, if a parent directory becomes a contiguous package later:

```text
workspace/
├── package.ph
└── demo/
    ├── package.ph
    └── ...
```

the outermost standalone package root changes.

That is acceptable for lightweight standalone use, but it should not be described with the same stability guarantees as project-backed identity.

I would document:

```text
Project-backed module identity:
    stable under physical relocation and checkout-directory rename.

Standalone-package identity:
    derived from package hierarchy and therefore intentionally
    sensitive to package-directory names and boundaries.

Standalone-module identity:
    execution-local unless promoted into a package/project.
```

This gives users a clear rule:

> If identity stability matters, create `project.toml`.

---

# 15. Precisely define nested project ownership

Section 71 says projects supersede standalone identity and nested project boundaries are distinct.

Good—but the resolver needs an exact rule.

Suppose:

```text
outer/
├── project.toml
└── src/
    ├── package.ph
    └── vendor/
        └── inner/
            ├── project.toml
            └── src/
                └── ...
```

The outer physical source root contains the nested project physically.

Can:

```text
outer
```

resolve modules by traversing into:

```text
vendor/inner/src
```

if package markers happen to line up?

It should not.

Define:

> A discovered nested project root is an ownership boundary. Its files are excluded from the enclosing project's source namespace even if physically located under the enclosing source root.

And when executing a source file directly:

> The nearest owning project boundary wins.

This is necessary to preserve the invariant that one source cannot silently obtain two project identities.

---

# 16. Filesystem canonicalization needs a `SourceId` abstraction

Sections 27–28 correctly require:

- confinement;
- symlink safety;
- duplicate-source detection.

But the specification phrases this primarily in terms of canonical physical paths.

That is slightly too concrete for the future design promised in §100, where a logical module might come from:

- source;
- bytecode cache;
- AOT artifact.

You may eventually also have:

- package archives;
- virtual/generated sources;
- embedded standard-library modules.

I recommend internal layering like:

```text
ModuleId
    semantic identity

SourceId
    loader/source-provider identity

SourceLocation
    diagnostic location

PhysicalPath?
    optional filesystem metadata
```

Then the normal filesystem source provider implements:

```text
canonicalize(path) -> SourceId
```

and enforces confinement.

A future archive provider can implement the same contract without pretending an archive entry has a meaningful `realpath()`.

This makes §27's intended invariant more future-proof:

> One source provider identity may not accidentally satisfy two distinct ModuleIds unless explicitly supported by that provider.

For the filesystem provider, symlink aliases are an error.

---

# 17. Specify path spelling, Unicode, and case semantics

Filesystem-defined logical names expose a cross-platform issue that the specification currently does not mention.

If identifiers permit Unicode, questions include:

```text
é.ph
```

versus differently normalized Unicode spellings.

If identifiers are case-sensitive:

```text
Foo.ph
foo.ph
```

may coexist on one filesystem and collide on another.

The language should not derive semantic equality from host filesystem quirks.

At minimum define:

```text
logical identifier comparison
    = Phalcom identifier semantics

filesystem lookup
    = source-provider operation
```

and give project validation permission to reject non-portable collisions.

I would also make package publication tooling stricter than local compilation and detect:

- case-fold collisions;
- Unicode-normalization collisions;
- reserved filenames;
- module/package collisions.

This is far easier than debugging packages that work on Linux but cannot be checked out consistently elsewhere.

---

# 18. Core needs an explicit qualified escape hatch

The specification retains:

```text
bare lookup:
    current module
    then core
```

That is reasonable.

But what happens when a module deliberately shadows a core name?

```phalcom
class System {}
```

or:

```phalcom
from .testing import System
```

Now how do you refer to the actual core `System`?

A language with an implicit prelude benefits from also having an explicit canonical path. Rust, for example, has implicitly available prelude names while still having explicit roots such as `std`.

Phalcom should have something analogous:

```phalcom
core.System
```

or:

```phalcom
import core
core.System
```

The exact spelling is less important than having a non-magical escape hatch.

I recommend treating `core` as a reserved root even if most programs never import it.

---

# 19. Improve visibility diagnostics: nonexistent and private are different errors

The specification currently groups a selective import of an unknown binding and a non-exported binding under `ImportNameError`.

But the compiler has enough information to distinguish:

```phalcom
// target.ph
class InternalParser {}
```

from:

```phalcom
// target.ph
// no InternalParser at all
```

So:

```phalcom
from .target import InternalParser
```

should not produce the same diagnostic in both cases.

Use something like:

```text
UnknownImportNameError
NonExportedImportError
```

For the second:

```text
NonExportedImportError:
  `.target` declares `InternalParser`, but the binding is private.

  Defined at target.ph:17.
```

Likewise add after path privacy:

```text
ModulePathNotExportedError
```

This is a small implementation cost because the `ModuleInterface` already knows declarations and exports.

It substantially improves developer experience.

---

# 20. Keep sticky initialization failure, but define exactly what is cached

Section 65 says failed initialization is sticky.

I mostly agree.

Python takes a different approach: if ordinary module loading fails, the failing module is removed from `sys.modules`, while modules successfully initialized as side effects remain cached.

That allows a later Python import to retry the failed module, but it also means partially performed external side effects may have happened already.

Phalcom's:

```text
Failed
```

state is more deterministic.

I would retain it for ordinary static imports.

But specify what gets cached.

For example:

```text
ModuleFailure {
    moduleId
    originalError
    initializationTrace
    cycleTrace?
}
```

Then first failure:

```text
DatabaseConnectionError
  while initializing app.config
```

and later import:

```text
ModuleInitializationError:
  app.config previously failed to initialize
Caused by:
  DatabaseConnectionError ...
```

should preserve the original failure provenance rather than manufacturing a new unrelated error each time.

Also specify ancestor behavior:

```text
A initialization imports B
B fails
```

Then:

```text
B → Failed
A → Failed
```

with `A`'s failure caused by `B`'s cached initialization failure.

An unrelated module that merely had its interface discovered should remain `Compiled`, not `Failed`.

---

# 21. Add memory-publication semantics to initialization completion

If Phalcom eventually runs execution in parallel, this becomes part of correctness.

When:

```text
module state transitions Initializing → Initialized
```

all initialized binding-cell values must be safely published to waiters.

In implementation terms this is a release/acquire style synchronization concern.

Language-level wording can remain abstract:

> Completion of module initialization establishes a synchronization boundary. Execution that observes the module as `Initialized` observes all writes performed by that module's initialization before the transition.

Without something equivalent, "initialize once" is insufficient for a parallel runtime.

---

# 22. Keep live imports — they fit the architecture very well

This:

```phalcom
// settings.ph
var mode = "development"
export mode
```

with:

```phalcom
from .settings import mode
```

remaining live as `settings` later rebinds `mode` is a strong design.

It aligns naturally with:

```text
SymbolId → BindingCell
```

rather than:

```text
import statement → copied Value
```

ECMAScript likewise implements imported bindings as indirect module bindings rather than value copies.

This has several benefits:

- re-export identity is natural;
- aliases preserve symbol identity;
- cycles can link before values exist;
- LSP references can point to one symbol;
- debugger watch expressions have coherent ownership;
- mutable exported globals behave consistently whether accessed qualified or selectively imported.

I would ratify this.

The VM should simply resist the temptation to optimize `from M import x` into a copied local value unless it proves the source binding immutable.

---

# 23. The mandatory `package.ph` is defensible, but invest in tooling

Rust's 2018 module-system changes are worth examining here. Rust's own Edition Guide says its module system had been a major source of confusion, and Rust changed path rules and removed the previous `mod.rs` requirement for a common filesystem organization.

That is not an argument that Phalcom must remove `package.ph`. The two mechanisms have different semantics: Phalcom's file is an actual package module and façade.

I would keep the explicit marker.

But acknowledge the DX cost in:

```text
a/
    package.ph
    b/
        package.ph
        c/
            package.ph
```

Tooling needs to make this essentially free.

For example:

```text
phalcom new package foo.bar.baz
```

should create the markers.

The LSP should diagnose:

```text
app.helpers.formatting cannot exist because helpers/
is not a package
```

and offer:

```text
Create helpers/package.ph
```

as a one-click code action.

That converts explicitness from friction into useful intentionality.

---

# 24. Do not add namespace packages

Despite the previous point, I would retain the absolute rejection of split namespace packages.

Python namespace packages can aggregate multiple physical "portions" and may dynamically reconsider package search locations as the parent search path changes.

That is useful in Python's packaging environment, but it cuts directly against Phalcom's:

```text
one ModuleId
one owner
one resolved project instance
```

model.

For plugin ecosystems, provide explicit plugin registration, service discovery, manifests, traits/protocols, or dependency metadata.

Do not represent extensibility as:

```text
four unrelated projects all contribute directories to the same package
```

That would infect the resolver, compiler cache keys, LSP project model, module-path privacy, and package manager.

---

# 25. No wildcard imports or exports is correct

Keep:

```phalcom
from geometry import *
```

illegal.

Keep:

```phalcom
export *
```

illegal.

Especially with static interfaces and API stability, there is no compelling upside.

Explicit façade APIs should remain enumerable.

---

# 26. But `export ... from ...` is worth adding eventually

The specification correctly says it can be added later as syntax sugar.

I agree it is safe to defer.

For package façades:

```phalcom
from .point import Point
from .vector import Vector

export Point, Vector
```

will become repetitive.

Eventually:

```phalcom
export Point from .point
export Vector from .vector
```

could lower exactly to the existing import-cell plus re-export-cell semantics.

No need to complicate v1 for this.

The same applies to grouped imports:

```phalcom
from geometry import (
    Point,
    Vector,
    Shape,
    Circle,
)
```

Pure grammar ergonomics can be added without altering semantic foundations.

---

# 27. One important lesson from Rust: keep all path grammars unified

Rust's 2018 redesign specifically simplified inconsistencies between `use` paths and other paths. Its documentation describes the earlier consequences as counterintuitive and confusing.

Phalcom is currently doing well here:

```phalcom
import .base as base
class Circle is base.Shape {}
```

and:

```phalcom
const x: base.Shape = ...
```

resolve through the same module alias.

Preserve that principle.

Do not later invent:

```text
import paths
type paths
superclass paths
reflection paths
macro paths
```

with subtly different roots.

They should all ultimately resolve through the same:

```text
Name/Path → SymbolId
```

machinery, subject only to context-specific "must resolve to a class/type/module/etc." validation.

---

# 28. The current complete example exposes the package-initialization flaw very clearly

The specification's example is useful enough to trace explicitly.

Given:

```phalcom
// geometry/package.ph

from .point import Point, origin
from .vector import Vector
from .shapes import Shape, Circle

export Point, origin, Vector, Shape, Circle
```

and:

```phalcom
// geometry/shapes/package.ph

from .base import Shape
from .circle import Circle

export Shape, Circle
```

and:

```phalcom
// geometry/shapes/circle.ph

from .base import Shape
from ..point import Point

class Circle is Shape {}
export Circle
```

under current §56:

```text
request geometry.point
        │
        ▼
initialize geometry/package.ph
        │
        ├── request geometry.point
        ├── request geometry.vector
        └── request geometry.shapes
                      │
                      ▼
             initialize shapes/package.ph
                      │
                      ├── base
                      └── circle
                           │
                           ├── base
                           └── point
```

A small direct import has become an eager package-facade traversal.

Under the model I recommend:

```text
request geometry.point
        │
        ├── discover package geometry
        └── initialize geometry.point
```

while:

```text
request geometry
        │
        ▼
initialize geometry/package.ph
        │
        ├── geometry.point
        ├── geometry.vector
        └── geometry.shapes
```

The second behavior is correct because the user explicitly requested the façade.

That distinction will materially improve startup, cycle behavior, modularity, and reasoning.

---

# 29. Proposed revised lifecycle

I would revise the conceptual pipeline from:

```text
Discovered
Declared
Compiled
Initializing
Initialized
```

to distinguish static graph state from runtime state more strongly.

Conceptually:

```text
                    ┌──────────────┐
                    │  Resolved    │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │  Declared    │
                    │ interface IDs│
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │    Linked    │
                    │ import cells │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │   Compiled   │
                    └──────┬───────┘
                           │
        runtime begins     │
                           ▼
                    ┌──────────────┐
                    │Initializing  │
                    └──────┬───────┘
                     success│ failure
             ┌─────────────┴──────────────┐
             ▼                            ▼
      ┌──────────────┐             ┌──────────────┐
      │ Initialized  │             │    Failed    │
      └──────────────┘             └──────────────┘
```

`Linked` is useful because it makes explicit the point where:

```phalcom
from .a import A
```

has already been transformed into something like:

```text
LocalImportBinding {
    sourceSymbol: SymbolId(a::A)
    cell: BindingCell*
}
```

without requiring `A`'s runtime class object to exist yet.

---

# 30. Recommended internal abstractions

I would avoid implementing this as a single "module loader."

The architecture naturally decomposes into at least these components.

```text
ProjectResolver
    resolves manifest dependency graph
    dependency aliases → ResolvedProjectId

ModuleResolver
    logical path + importing project
    → ModuleId

SourceProvider
    ModuleId
    → SourceId / source contents / diagnostics location

InterfaceBuilder
    source
    → ModuleInterface

ModuleLinker
    ImportDecl / ExportDecl
    → SymbolId / BindingCell relationships

ModuleCompiler
    linked module
    → executable artifact

ModuleRegistry
    ModuleId
    → canonical ModuleRecord

ModuleInitCoordinator
    scheduler-aware initialization ownership/waiting

ModuleRuntime
    executes top-level module artifact

ModuleReflection
    runtime object → module metadata
```

And the central records could look conceptually like:

```text
ModuleId {
    projectInstance: ResolvedProjectId
    relativePath: ModulePath
}
```

```text
ModuleInterface {
    id: ModuleId
    declarations: Map<Name, Symbol>
    exports: Map<PublicName, SymbolId>
    imports: List<ImportEdge>
}
```

```text
BindingCell {
    state:
        Uninitialized
        Initialized(Value)
}
```

```text
ModuleRecord {
    id: ModuleId
    interface: ModuleInterface
    namespace: ExportNamespace
    state: ModuleState
    runtimeObject: Package | Module
}
```

```text
ModuleState =
    Compiled
    | Initializing(InitTransactionId)
    | Initialized
    | Failed(ModuleFailure)
```

This decomposition directly reflects the abstractions the specification already gestures toward.

---

# 31. Language-history comparison

The useful comparison is not "which language should Phalcom imitate?" but "which mistakes have already demonstrated their cost?"

| Language/system | Historical pressure | Lesson for Phalcom |
|---|---|---|
| Python | Implicit-relative behavior was replaced with explicit relative imports; executable package modules required special handling around `__main__`.  | Phalcom's explicit relative syntax and stable entry-module identity are correct. |
| Python | Import paths, mutable import caches, package initialization, and partial modules make import identity/loading highly dynamic.  | Keep deterministic project universes and avoid ambient path mutation. |
| Python typing | Annotation dependencies and runtime imports can collide badly, especially with circular references.  | Distinguish interface/type edges from runtime initialization edges. |
| Rust | Module/path rules were confusing enough that the 2018 edition deliberately simplified them and removed common `mod.rs` requirements.  | Keep one path-resolution model; make explicit package markers tooling-cheap. |
| Rust | `pub use` deliberately decouples public API topology from implementation topology.  | Keep package façades and explicit re-exports. |
| Rust/Cargo | Multiple resolved dependency versions produce distinct type identities and can leak compatibility hazards across APIs.  | Define `ProjectInstanceId` and duplicate-instance type identity before package management ships. |
| Go | Identifier visibility alone proved insufficient for implementation encapsulation, motivating `internal` package boundaries.  | Do not defer module-path privacy. |
| Go | Imports define package dependencies and package cycles are prohibited.  | Phalcom can support cycles, but must accept that it therefore needs substantially stronger linker/init machinery. |
| ECMAScript | Imports are linked as indirect bindings; cyclic module records have explicit link/evaluation phases.  | Phalcom's binding cells/interface-before-init design is sound. |
| ECMAScript | Module namespace objects have specialized semantics rather than behaving like arbitrary ordinary objects.  | Phalcom must explicitly solve export-name vs `Module`-protocol selector collisions. |
| Node packages | Package `"exports"` was added to encapsulate subpaths and provide stronger API/semver guarantees.  | Public module paths need declaration now, before deep imports become ecosystem contracts. |

---

# 32. What I would actually change in the normative decisions

The following are the revisions I would make before calling the document final.

### Change §26: import initialization timing

Current:

```text
Successful module initialization occurs synchronously
at the import's source position.
```

Recommended:

```text
Static import declarations establish module dependencies.

Before a module begins ordinary top-level execution, all of its
required runtime import dependencies are initialized according
to the module initialization graph.

Source position affects lexical diagnostics and source organization,
but does not schedule dependency initialization.

Delayed runtime loading is a separate reflective facility.
```

I would go further and require static imports to occur in the module preamble.

---

### Change §45 / current package-init rule

Current:

```text
Parent packages initialize outermost-to-innermost
before a nested target module.
```

Recommended:

```text
Containing packages are resolved, declared, and registered
outermost-to-innermost before a nested target module.

A containing package's package.ph executes only when that package
is itself a runtime dependency.

Logical containment alone does not imply initialization dependency.
```

This is a major improvement.

---

### Replace §39 path-privacy decision

Current:

```text
Module-path privacy is not a separate visibility dimension.
```

Recommended:

```text
Module-path visibility is separate from binding visibility.

Within a project, project modules may resolve project-private paths.

Across project boundaries, only public module/package paths may
be resolved.

The dependency root package is public by default.
Submodule/subpackage path visibility is explicitly declared.
```

---

### Clarify cycle import semantics

Current:

```text
Phalcom may establish a live import without reading the value.
```

Recommended:

```text
Selective-import linkage never reads the target value.

It links to the target exported BindingCell.

Only evaluation requiring the binding's runtime value reads the cell.
```

---

### Add concurrent initialization semantics

Normative rule:

```text
One canonical ModuleId has one canonical initialization operation.

Concurrent requests for the same initializing module await the
existing operation.

Only reentrant resolution participating in the active initialization
transaction may observe a partial module record for cycle handling.

Unrelated concurrent execution does not observe partially initialized
modules.
```

---

### Add project-instance identity

Normative rule:

```text
ResolvedProjectId is an opaque identity assigned to a node in the
resolved dependency graph.

Dependency aliases are not part of it.

Two aliases mapping to the same resolved node share module/type identity.

Distinct resolved nodes have distinct module/type identities even when
human-readable package metadata matches.
```

---

### Add dependency-edge kinds

At least architecturally:

```text
Interface dependency
Runtime initialization dependency
```

with room for build conditions.

---

### Add module export/protocol precedence

Normative rule:

```text
A module value's public member namespace is its export namespace.

Module metadata and loader/runtime administration are accessed through
the reflection/metaobject API and do not consume ordinary export names.
```

That is the cleanest model for Phalcom.

---

# 33. Implementation difficulty versus reversibility

This is the part I would use to prioritize engineering.

| Decision | Implement now | Complexity now | Complexity if changed after ecosystem adoption |
|---|---:|---:|---:|
| Logical `ModuleId` | Yes | Medium | Extreme |
| Dependency alias → instance resolution | Yes | Medium | Extreme |
| Opaque `ResolvedProjectId` | Yes | Medium | Extreme |
| Binding cells | Yes | High | Extreme |
| Interface discovery before init | Yes | High | Extreme |
| Module-path privacy | **Yes** | Low/medium | **Extreme** |
| Do not execute ancestor packages automatically | **Yes** | Actually simplifies loader | **Extreme** |
| Graph-level rather than textual init | **Yes** | Medium/high | **Extreme** |
| Concurrent initialization coordinator | **Yes** | High | Very high |
| Module export namespace semantics | **Yes** | Medium | Very high |
| Interface/runtime edge distinction | Structural hook now | Medium | High |
| Conditional-edge representation | Structural hook now | Low/medium | Medium/high |
| Source-provider abstraction | Yes | Medium | Medium |
| Core qualified escape hatch | Prefer now | Low | Medium |
| `export X from M` | Later | Low | Low |
| grouped import grammar | Later | Low | Low |
| multiple executable targets | Later | Medium | Low |
| reload semantics | Later | High | Low if separate API |
| dynamic loading | Later | High | Low if separate API |
| workspace model | Later | High | Low/medium |
| compiled artifact syntax | Never | — | — |

The interesting observation is that two of my strongest proposed changes actually make implementation easier:

1. **do not execute ancestor package modules merely for child lookup**;
2. **do not let textual import position schedule initialization**.

Both remove state-machine complexity instead of adding it.

---

# 34. A cleaner final mental model

I think Phalcom should ultimately be described like this:

```text
Project graph
    determines ownership and dependency universes
                    │
                    ▼
Module graph
    determines logical ModuleIds
                    │
                    ▼
Interface graph
    determines SymbolIds, exports, types, inheritance
                    │
                    ▼
Runtime dependency graph
    determines initialization requirements
                    │
                    ▼
Module initialization
    materializes BindingCell values
                    │
                    ▼
Module/Package objects
    expose only explicit export namespaces
```

That gives five deliberately distinct concepts:

```text
filesystem containment
package containment
module identity
API visibility
initialization dependency
```

The current specification occasionally collapses the last two or three together—particularly when ancestor package containment implies runtime initialization.

They should remain independent.

---

# 35. Revised example

With these changes, suppose:

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

Root package:

```python
from .point import Point
from .vector import Vector

from .deep_space.point.vector import (Point, Vector)  // error: path not exported

export Point, Vector

public module .point
public module .vector
```

`cache.ph`:

```phalcom
class Cache {}

export Cache
```

The last export says:

> if someone legitimately acquires the `geometry.internal.cache` module object, `Cache` is its public binding.

It does **not** say:

> arbitrary dependency consumers may resolve the path `geometry.internal.cache`.

So externally:

```phalcom
from geometry import Point
```

works.

```phalcom
import geometry.point
```

works.

```phalcom
import geometry.internal.cache
```

fails:

```text
ModulePathNotExportedError
```

Internally, another module in the same project can still write:

```phalcom
from .internal.cache import Cache
```

That is a much more complete definition of encapsulation.

---

# 36. Final verdict

I would characterize the attached specification as **architecturally excellent but not yet safe to freeze as the final semantic target**.

The parts most worth preserving are the ones where Phalcom is deliberately more explicit than Python: one canonical module identity, logical imports, no ambient search paths, no implicit relative imports, no automatic child-module package members, no wildcard namespace merging, private-by-default bindings, explicit re-export, package façades, live binding cells, and interface discovery before runtime execution. Those collectively give the compiler and LSP an unusually strong semantic substrate.

The decisions I would reopen now are much narrower but much more consequential:

1. **Add module-path privacy now.** This is the single biggest compatibility hazard in the current text.
2. **Remove source-position scheduling of static import initialization.** Static imports should behave like dependency declarations, not disguised top-level effect statements.
3. **Do not execute ancestor `package.ph` modules simply because a descendant is imported.** Register/link the package chain; execute package modules only when they are actual runtime dependencies.
4. **Specify scheduler-aware concurrent initialization before fibers and modules meet in production.**
5. **Separate the module export namespace from the `Module` object's reflective/runtime protocol**, or export names will collide with ordinary selectors.
6. **Make `ResolvedProjectId`/`ProjectInstanceId` genuinely normative and opaque**, including duplicate dependency-instance semantics.
7. **Represent interface/type dependencies separately from runtime initialization dependencies**, even if type-only import syntax comes later.
8. **Leave a build-condition field in the module graph**, because optional/platform-specific imports will otherwise force a later graph redesign.
9. **Make standalone identity's weaker stability explicit**, and precisely define nested-project ownership.
10. **Abstract physical canonicalization behind a `SourceId`/source-provider layer** rather than baking real filesystem paths into the future compiled-module architecture.

With those revisions, I would be comfortable treating the rest of this model as the long-term Phalcom module architecture. More importantly, it would become a remarkably coherent foundation for the other things Phalcom is moving toward: optional correctness-relevant typing, static superclass/type resolution, incremental LSP analysis, compiled caches, package distribution, fibers/concurrency, and eventually native/AOT compilation. The module system would cease being merely a loader and become what it should be in Phalcom: the semantic ownership and identity layer underneath the compiler, runtime, type system, and tooling.
