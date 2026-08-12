# Phalcom Modules, Packages, and Projects

Status: **Final target specification**

This specification defines Phalcom's module, package, project, import, export, resolution, loading, initialization, and execution semantics. Once implemented, it supersedes the current U15 / DEC-U15 module model.

The central design is:

```text
Module < Package < Project
```

with the following principles:

* every source unit has one stable logical identity;
* filesystem layout defines the module tree, but source imports never contain filesystem paths;
* packages are explicit;
* projects define deterministic dependency universes;
* module APIs are private by default and explicitly exported;
* imports never merge namespaces implicitly;
* imports are statically resolvable but initialize modules at their source position;
* importing and directly executing the same module never create distinct module identities;
* no `sys.path`-like ambient resolution exists.

---

# 1. Conceptual Model

Phalcom has three organizational levels.

| Unit    | Meaning                                        | Runtime object? | Backing file   |
| ------- | ---------------------------------------------- | --------------: | -------------- |
| Module  | One source module                              |             Yes | `name.ph`      |
| Package | A module namespace containing modules/packages |             Yes | `package.ph`   |
| Project | Dependency/build/resolution unit               |  Not inherently | `project.toml` |

A package is semantically a specialized kind of module.

A project is not another importable namespace layered above its root package. It owns a root package and provides that package with stable identity, dependencies, source-root configuration, and executable configuration.

Conceptually:

```text
Project
└── Root Package
    ├── Module
    ├── Module
    └── Package
        ├── Module
        └── Package
```

For example:

```text
geometry/
├── project.toml
└── src/
    ├── package.ph
    ├── point.ph
    ├── vector.ph
    └── shapes/
        ├── package.ph
        ├── base.ph
        └── circle.ph
```

may define:

```text
geometry
geometry.point
geometry.vector
geometry.shapes
geometry.shapes.base
geometry.shapes.circle
```

`geometry` is the root package, not the project object.

---

# 2. Fundamental Identity Rule

Every recognized module or package has exactly one semantic identity.

Running, importing, re-exporting, or referring to that source does not create a second module.

In particular, Phalcom has no Python-style distinction between:

```text
some.module
```

and:

```text
__main__
```

when the same file is directly executed.

The same module is always the same module.

---

# 3. Packages Are Explicit

Every package directory must contain:

```text
package.ph
```

A directory does not become a package merely because it contains `.ph` files.

Valid:

```text
shapes/
├── package.ph
├── circle.ph
└── rectangle.ph
```

Invalid as a package:

```text
shapes/
├── circle.ph
└── rectangle.ph
```

This is intentional.

Filesystem organization and language namespace organization must not become accidentally equivalent.

`package.ph` is therefore the explicit package marker.

---

# 4. `package.ph`

`package.ph` is the source module backing the package itself.

For:

```text
src/shapes/package.ph
```

the logical identity is:

```text
geometry.shapes
```

not:

```text
geometry.shapes.package
```

The filename `package.ph` is reserved for this purpose.

A package module may contain ordinary top-level Phalcom code:

```phalcom
from .base import Shape
from .circle import Circle

const defaultShape = Circle(radius: 1)

export Shape, Circle, defaultShape
```

`package.ph` therefore serves two primary purposes:

1. package initialization;
2. definition of the package's public façade.

It is not implicitly the package's executable entry point.

---

# 5. Ordinary Modules

Every ordinary module is backed by one `.ph` file.

For example:

```text
src/geometry/point.ph
```

may correspond to:

```text
app.geometry.point
```

A filename contributing to a logical module path must form a valid Phalcom identifier.

Phalcom does not silently translate filesystem names such as:

```text
some-module.ph
```

into:

```text
some_module
```

Invalid filesystem names must be renamed or placed behind an explicitly named project namespace where applicable.

---

# 6. Module/Package Name Collision

A package and ordinary module may not claim the same logical name.

This layout is invalid:

```text
src/
├── network.ph
└── network/
    └── package.ph
```

because both claim:

```text
app.network
```

Phalcom must report an ambiguity/layout error.

It must never choose one according to lookup order.

The invariant is:

> One logical module identity corresponds to one source unit.

---

# 7. Directories Without `package.ph`

Directories that do not contain `package.ph` do not participate in the package namespace.

They may contain documentation, generated data, resources, or other files, but a logical module path may not traverse them.

For example:

```text
src/
├── package.ph
└── helpers/
    └── formatting.ph
```

does not define:

```text
app.helpers.formatting
```

because `helpers` is not a package.

To make it one:

```text
src/
├── package.ph
└── helpers/
    ├── package.ph
    └── formatting.ph
```

---

# 8. Projects

A project is Phalcom's highest language-level unit of source organization and dependency resolution.

A project defines:

* one root package;
* one source root;
* one stable project namespace;
* zero or more project dependencies;
* optionally, one executable entry module.

A project does not automatically correspond to a runtime `Project` object.

Runtime code interacts with the project's root `Package`.

---

# 9. `project.toml`

The module system requires the following conceptual manifest information:

```toml
[project]
name = "geometry-kit"
namespace = "geometry"
source = "src"
entry = "geometry.main"

[dependencies]
math = { package = "phalcom-math", version = "2.0" }
util = { path = "../utility" }
```

The relevant fields are:

| Field       | Meaning                                      |
| ----------- | -------------------------------------------- |
| `name`      | Distribution/project identity                |
| `namespace` | Root package name used by the project itself |
| `source`    | Source root; defaults to `src`               |
| `entry`     | Optional executable module                   |

`namespace` must be a valid Phalcom identifier.

The project name and namespace are deliberately distinct.

For example:

```toml
name = "phalcom-http-client"
namespace = "http"
```

is valid.

Phalcom does not silently rewrite a project name into an identifier.

---

# 10. Project Root Package

The source root must contain:

```text
package.ph
```

For:

```toml
namespace = "app"
source = "src"
```

this file:

```text
src/package.ph
```

backs:

```text
app
```

The physical name of the checkout directory is irrelevant.

These:

```text
/home/a/project/
/tmp/random-copy/
/Users/x/dev/foo/
```

all produce the same root package identity if their manifest declares:

```toml
namespace = "app"
```

Filesystem relocation therefore does not rename project modules.

---

# 11. There Are No Physical Imports

Phalcom source code never imports filesystem paths.

The old U15 form:

```phalcom
import "./geometry/point" as Point
```

is removed.

So are:

```phalcom
import "../shared"
import "./foo.ph"
import "/absolute/path/foo"
```

There is no quoted import target grammar.

Imports always refer to logical module/package names.

This is a foundational rule.

---

# 12. Filesystem Paths Versus Language Imports

Filesystem paths may still occur in tooling.

For example:

```text
phalcom run src/tools/demo.ph
```

is perfectly valid.

The filesystem path tells the CLI which source artifact the user selected.

The compiler then determines that source's logical identity, perhaps:

```text
app.tools.demo
```

Its source code must still use logical imports:

```phalcom
from .formatter import Formatter
```

Never:

```phalcom
from "./formatter.ph" import Formatter
```

Thus:

```text
CLI/source selection → physical paths may be used

Phalcom import semantics → physical paths never appear
```

---

# 13. Import Forms

Phalcom supports two primary import declarations.

## 13.1 Whole-module import

```phalcom
import geometry.point
```

With alias:

```phalcom
import geometry.point as pointModule
```

Relative:

```phalcom
import .point
import ..units
```

## 13.2 Selective import

```phalcom
from geometry.point import Point, distance
```

Aliases:

```phalcom
from geometry.point import Point as P, distance as dist
```

Relative:

```phalcom
from .point import Point
from ..units import Meter
```

From a package itself:

```phalcom
from . import Point
from .. import Shape
```

---

# 14. Whole-Module Default Binding

Unlike Python, this:

```phalcom
import geometry.shapes.circle
```

binds the final path component:

```text
circle
```

So it behaves conceptually as:

```phalcom
import geometry.shapes.circle as circle
```

Similarly:

```phalcom
import .point
```

binds:

```text
point
```

and:

```phalcom
import geometry
```

binds:

```text
geometry
```

There is no Python-style behavior where `import geometry.point` unexpectedly binds only `geometry`.

---

# 15. Import Aliases

Aliases affect only the importing module's local namespace.

```phalcom
import geometry.point as PointModule
```

creates:

```text
PointModule
```

as an immutable local import binding.

It does not rename the imported module itself.

Likewise:

```phalcom
from geometry.point import Point as P
```

creates local binding:

```text
P
```

while the source export remains named:

```text
Point
```

---

# 16. Absolute Imports

An absolute import begins with an explicit import root.

For a project, valid roots consist only of:

1. the current project's own namespace;
2. declared dependency aliases;
3. compiler/runtime-provided reserved roots, if any.

For example:

```toml
[project]
namespace = "app"

[dependencies]
json = { package = "phalcom-json", version = "3.0" }
math = { package = "linear-algebra", version = "2.0" }
```

can make these valid:

```phalcom
import app.model
import json.parser
import math.matrix
```

An unknown root is an error.

Phalcom never searches arbitrary directories trying to discover what the programmer meant.

---

# 17. No Implicit Relative Absolute Imports

Inside:

```text
app.geometry.circle
```

this:

```phalcom
import point
```

does not mean:

```text
app.geometry.point
```

`point` is interpreted as an absolute root name.

To access the sibling module, write:

```phalcom
import .point
```

or:

```phalcom
import app.geometry.point
```

This eliminates implicit-relative-import ambiguity.

---

# 18. Relative Imports

Relative imports operate on logical package ancestry, not filesystem directories.

The rule is:

> One leading dot selects the current package context. Every additional dot ascends one parent package.

For an ordinary module:

```text
app.geometry.circle
```

the current package context is:

```text
app.geometry
```

Therefore:

```phalcom
import .point
```

means:

```text
app.geometry.point
```

and:

```phalcom
import ..model
```

means:

```text
app.model
```

For the package module itself:

```text
app.geometry
```

the current package context is the package itself.

Therefore inside:

```text
app/geometry/package.ph
```

this:

```phalcom
import .point
```

also means:

```text
app.geometry.point
```

This gives `package.ph` intuitive access to its children.

---

# 19. Relative Import Beyond Root

A relative import may never ascend beyond the root package.

For example, from:

```text
app.foo
```

an import that attempts to climb above:

```text
app
```

is invalid.

This is a logical namespace error, not filesystem path traversal.

There is no interpretation in which relative dots can escape to an arbitrary parent directory.

---

# 20. Dependency Import Roots

The dependency key in `project.toml` defines the import root visible to the consumer.

For example:

```toml
[dependencies]
lin = { package = "linear-algebra", version = "2.0" }
```

allows:

```phalcom
import lin.matrix
```

even if the dependency project's own namespace is:

```text
linalg
```

Inside the dependency's own source, it continues using its own self namespace:

```phalcom
import linalg.matrix
```

Consumer aliases do not rewrite dependency source.

---

# 21. Canonical Project Module Identity

A consumer-visible root alias is not part of canonical identity.

Conceptually, a project module is identified by:

```text
ProjectInstanceId + ProjectRelativeModulePath
```

For example:

```text
ProjectInstanceId:
    linear-algebra@2.0 / resolved instance

Relative module:
    matrix.decomposition
```

Both:

```text
linalg.matrix.decomposition
```

inside the dependency and:

```text
lin.matrix.decomposition
```

inside a consumer may resolve to exactly the same canonical module.

This permits dependency renaming without duplicating modules.

---

# 22. Root Name Collisions

Within one project's import environment, import roots must be unambiguous.

The following may not conflict:

* the current project's namespace;
* dependency aliases;
* reserved built-in roots.

If the project's namespace is:

```text
app
```

then a dependency alias may not also be:

```text
app
```

Resolution never depends on precedence between colliding roots.

The manifest is invalid instead.

---

# 23. No Search Paths

Phalcom has no equivalent of:

```text
PYTHONPATH
sys.path
current-directory package discovery
site-packages scanning
ambient installation directories
```

Logical import resolution must be determined by explicit semantic context.

A source import cannot change meaning merely because the current working directory changed.

---

# 24. No Namespace Packages

A package cannot be assembled from multiple directories.

One package corresponds to one package source location in one resolved project/package instance.

Phalcom does not merge:

```text
directory A/foo/
directory B/foo/
```

into one logical package.

This preserves unique ownership and deterministic module identity.

---

# 25. Resolution of a Logical Path

Suppose the project namespace is:

```text
app
```

and the compiler resolves:

```phalcom
import app.geometry.shapes.circle
```

The resolver performs the semantic equivalent of:

1. resolve `app` to a root package/project instance;
2. start from that project's configured source root;
3. require `geometry/package.ph`;
4. require `geometry/shapes/package.ph`;
5. resolve final component `circle` to exactly one of:

```text
geometry/shapes/circle.ph
```

or:

```text
geometry/shapes/circle/package.ph
```

6. reject if neither exists;
7. reject if both exist;
8. canonicalize the resulting source for confinement and duplicate-source validation;
9. intern the canonical module identity.

Intermediate components must always be packages.

---

# 26. Root Package Resolution

The root itself is importable.

```phalcom
import app
```

loads the root package backed by:

```text
src/package.ph
```

A dependency root works identically:

```phalcom
import lin
```

returns that dependency project's root package object.

---

# 27. Canonical Filesystem Validation

Logical identity is primary, but physical canonicalization is still used internally for:

* root confinement;
* symlink safety;
* duplicate-source detection;
* source diagnostics.

A single source file must not accidentally acquire two distinct logical identities through symlink tricks or duplicate filesystem aliases.

Such a source-tree configuration is an error rather than two separate modules.

Physical canonical paths are implementation metadata, not import identities.

---

# 28. Project Root Confinement

A project module must resolve inside its declared source root.

After canonicalization, a logical module may not escape that root through symlinks or other filesystem indirection.

External source trees must enter the dependency graph explicitly.

The same rule applies separately to each resolved dependency project.

This gives Phalcom deterministic and sandboxable resolution semantics.

---

# 29. Imports Are Module-Scope Declarations

Imports may occur only directly in a module/package body.

Valid:

```phalcom
import app.config
from .model import User
```

Invalid:

```phalcom
method() {
    import app.config
}
```

Invalid inside:

* methods;
* blocks;
* closures;
* class bodies;
* conditionals;
* loops;
* other nested lexical scopes.

The entire module dependency graph must remain statically discoverable.

Runtime dynamic loading, if Phalcom eventually provides it, is a separate reflective API and not another form of `import`.

---

# 30. Imports Have Source-Position Scope

Although import targets are statically resolved, imported names enter lexical scope only from their source position onward.

Invalid:

```phalcom
const x = point.origin

import .point
```

Valid:

```phalcom
import .point

const x = point.origin
```

Likewise:

```phalcom
class Circle is base.Shape {
}

import .base as base
```

is invalid.

Move the import before the declaration.

---

# 31. Static Resolution Does Not Mean Hoisted Execution

The compiler may resolve and compile imports before the program starts.

That does not mean imported module initialization occurs at the beginning of the importing module.

Given:

```phalcom
System.print("before")

import app.geometry.point

System.print("after")
```

runtime initialization remains conceptually:

```text
print "before"
initialize app.geometry.point if necessary
bind point
print "after"
```

Thus:

```text
static dependency knowledge ≠ runtime initialization order
```

---

# 32. Import Bindings Are Immutable

Both whole-module and selective imports create immutable local bindings.

```phalcom
import app.geometry.point as point
from app.config import config
```

Neither:

```text
point
config
```

may subsequently be rebound in the importing module.

Imports are namespace declarations, not ordinary mutable globals.

---

# 33. Selective Imports Are Live Binding References

Selective imports do not copy the current value of an exported binding.

Suppose:

```phalcom
// settings.ph

var mode = "development"

export mode
```

Another module:

```phalcom
from .settings import mode
```

holds a read-only reference to the exported binding cell.

If `settings` itself later changes its own binding:

```phalcom
mode = "production"
```

the importer subsequently observes:

```text
"production"
```

Conceptually:

```text
settings.mode binding cell
          ↑
          └── consumer's imported `mode`
```

This deliberately differs from Python's snapshot-like `from x import y` semantics.

---

# 34. Imported Bindings Are Read-Only to Consumers

A live selective import does not grant permission to rebind the source module's global.

If:

```phalcom
from .settings import mode
```

then:

```phalcom
mode = "other"
```

is illegal because `mode` is an immutable import binding.

Likewise, obtaining a module object does not grant arbitrary external rebinding of its module globals.

If mutable behavior should be exposed, the module should export an intentional mutating API.

---

# 35. Bare Global Lookup

Bare global lookup remains:

```text
current module
    ↓
core
```

Imported module members are not implicitly searched.

Parent package exports are not implicitly searched.

Dependency exports are not implicitly searched.

For example:

```phalcom
import app.geometry.point
```

does not make this work:

```phalcom
Point(...)
```

unless `Point` was selectively imported:

```phalcom
from app.geometry.point import Point
```

Core names remain implicitly available according to Phalcom's ordinary core lookup semantics.

---

# 36. Visibility Model

Module bindings are **private by default**.

There is no Python-style rule that everything is public unless prefixed with `_`.

A module's external interface consists exclusively of its exports.

Example:

```phalcom
class Point {
}

const origin = Point(0, 0)
const implementationCache = ...

export Point, origin
```

Externally:

```phalcom
point.Point
point.origin
```

are accessible.

This is not:

```phalcom
point.implementationCache
```

because the binding was never exported.

---

# 37. There Is No `private` Keyword for Module Globals

The basic rule is sufficient:

```text
not exported = private to that module
```

Therefore Phalcom does not require:

```phalcom
private const cache = ...
```

to establish module privacy.

The absence of an export is the privacy declaration.

---

# 38. `_` Has No Visibility Semantics

These names:

```phalcom
const _cache = ...
const internalState = ...
```

have identical visibility unless one is exported.

This is legal:

```phalcom
const _special = 42
export _special
```

although style tooling may choose to discourage such APIs.

Naming convention and language visibility remain separate concepts.

---

# 39. Export Syntax

A module exports existing top-level bindings explicitly:

```phalcom
export Point, Vector, distance
```

Export aliases are supported:

```phalcom
export CartesianPoint as Point
```

The alias changes only the external name.

Example:

```phalcom
class CartesianPoint {
}

export CartesianPoint as Point
```

Inside the module, the binding remains:

```text
CartesianPoint
```

Externally, it appears as:

```text
Point
```

No extra local `Point` binding is created merely by the export.

---

# 40. Export Placement

`export` declarations are permitted only directly at module/package scope.

An export must refer to a top-level binding that has already been introduced lexically in that module.

Preferred:

```phalcom
class Point {
}

export Point
```

Not:

```phalcom
export Point

class Point {
}
```

The compiler may discover the eventual interface during its declaration pass, but source code itself remains ordered and readable.

---

# 41. Export Names Must Be Unique

A module may not expose two bindings under the same external name.

Invalid:

```phalcom
export CartesianPoint as Point
export PolarPoint as Point
```

Likewise, duplicate exports of the same external name are rejected rather than resolved by order.

---

# 42. Imported Names Are Not Automatically Re-Exported

This:

```phalcom
from .point import Point
```

makes `Point` available to the current module.

It does not automatically make `Point` part of the current module's public API.

Re-exporting is explicit:

```phalcom
from .point import Point

export Point
```

or:

```phalcom
from .point import Point as InternalPoint

export InternalPoint as Point
```

This prevents accidental transitive APIs.

---

# 43. No Wildcard Imports

Phalcom does not support:

```phalcom
from geometry import *
```

The reasons are semantic, not stylistic:

* imported bindings remain statically enumerable;
* collisions are explicit;
* autocomplete remains precise;
* refactoring remains reliable;
* dependency relationships remain visible.

Use selective imports or import the module/package object itself.

---

# 44. No Wildcard Exports

Likewise, Phalcom does not support:

```phalcom
export *
```

or:

```phalcom
export * from .point
```

A public API must be explicitly enumerable.

---

# 45. No `export ... from ...` Shorthand in the Base Specification

This:

```phalcom
export Point from .point
```

is not part of the base language.

Instead:

```phalcom
from .point import Point

export Point
```

expresses both dependency and re-export explicitly.

An `export ... from ...` form could later be introduced strictly as syntactic sugar without changing semantics.

---

# 46. Package Public API

Because a package is also a module, exactly the same export rules govern packages.

Example:

```text
geometry/
├── package.ph
├── point.ph
└── vector.ph
```

`point.ph`:

```phalcom
class Point {
}

export Point
```

`vector.ph`:

```phalcom
class Vector {
}

export Vector
```

`package.ph`:

```phalcom
from .point import Point
from .vector import Vector

export Point, Vector
```

Consumers can now write:

```phalcom
from geometry import Point, Vector
```

or:

```phalcom
import geometry as Geometry

const p = Geometry.Point(...)
```

`package.ph` therefore acts as an intentional façade.

---

# 47. Child Modules Are Not Automatically Package Members

Filesystem hierarchy belongs to the resolution namespace.

It does not automatically mutate the runtime public namespace.

Given:

```text
geometry/
├── package.ph
└── point.ph
```

the existence of:

```text
geometry.point
```

does not automatically imply:

```phalcom
Geometry.point
```

works after:

```phalcom
import geometry as Geometry
```

To expose the module object itself:

```phalcom
// geometry/package.ph

import .point as point

export point
```

Then:

```phalcom
Geometry.point
```

works because `point` is an explicit package export.

This establishes the important distinction:

```text
resolution namespace ≠ public runtime namespace
```

---

# 48. `from package import X` Never Searches for a Child Module

This:

```phalcom
from geometry import point
```

has exactly one meaning:

> Import the exported binding named `point` from the package `geometry`.

It never means:

> If no export exists, perhaps try importing `geometry.point`.

If the module is wanted:

```phalcom
import geometry.point
```

If the package wishes to expose that child module:

```phalcom
import .point as point

export point
```

Then selective import works for the normal reason.

No fallback behavior exists.

---

# 49. Path-Level Module Privacy

The base specification defines privacy for **module bindings**, not for logical module paths.

Therefore a consumer that knows:

```text
geometry.internals
```

may resolve that module directly if it belongs to the dependency's source tree.

However, it can access only what that module explicitly exports.

This means the base system has:

```text
private/public bindings
```

but does not yet have separate declarations such as:

```text
package-private module
project-private module
public module
```

Path-level visibility may be added independently later without changing import syntax.

---

# 50. External Module Member Access

Imported modules and packages remain ordinary runtime objects.

Their exported members are accessed using Phalcom's ordinary message/member-send machinery:

```phalcom
import geometry.point as point

point.origin
point.distance(1, 2)
```

Only exported bindings participate in the module's external member namespace.

Unexported internal globals are not externally visible merely because a runtime Module object exists.

Reflection may eventually provide explicitly privileged inspection APIs; ordinary module access does not bypass export visibility.

---

# 51. Module Interfaces Exist Before Initialization

Phalcom separates:

```text
resolution
declaration/interface discovery
compilation
runtime initialization
```

A module has a statically discoverable interface before its top-level code necessarily runs.

Conceptually:

```text
ModuleInterface
├── canonical ModuleId
├── package identity
├── imports
├── top-level declarations
├── exports
├── declaration kinds
└── available type information
```

This is central to:

* selective import validation;
* cyclic import support;
* cross-module inheritance;
* type checking;
* semantic autocomplete;
* go-to-definition;
* incremental compilation.

---

# 52. Export Binding Cells

An exported name refers to a binding cell rather than merely copying its current runtime value.

Conceptually:

```text
Internal binding
      ↓
Binding cell
      ├── module export
      ├── selective import
      └── re-export
```

Export aliases and re-exports can therefore preserve symbol identity.

This is what makes live selective imports and robust circular-import handling possible.

---

# 53. Static Import Errors Occur Before Execution

Because imports are statically resolvable, errors such as:

* unknown module;
* unknown dependency root;
* invalid relative import;
* importing an unexported/nonexistent symbol;
* module/package ambiguity;

may be diagnosed before any top-level program code executes.

Thus:

```phalcom
System.print("before")

import app.doesNotExist
```

need not print `"before"` and then fail.

Resolution/compilation can reject the program before execution begins.

What remains source-position-sensitive is successful module **initialization**, not target discovery.

---

# 54. Cross-Module Class Resolution

Imported classes may be used as superclasses.

Selective form:

```phalcom
from .base import Shape

class Circle is Shape {
}
```

Qualified form:

```phalcom
import .base as base

class Circle is base.Shape {
}
```

Both must resolve statically.

The current limitation where `is` can resolve only a bare class from the current module/core is removed.

---

# 55. Qualified Static References

Where Phalcom requires a statically resolvable class/type symbol, a qualified reference rooted in a known module import is permitted.

For example:

```phalcom
class Circle is base.Shape {
}
```

and, under the type-system specification:

```phalcom
const shape: base.Shape = ...
```

The compiler resolves `base` as a module alias and `Shape` through that module's interface.

This is not equivalent to allowing an arbitrary runtime expression in a superclass position.

The reference must resolve statically to an appropriate exported declaration.

---

# 56. Package Initialization Order

Importing a nested module initializes its containing package chain from outermost to innermost before the target module.

Importing:

```text
geometry.shapes.circle
```

causes initialization conceptually in this order:

```text
geometry/package.ph
        ↓
geometry/shapes/package.ph
        ↓
geometry/shapes/circle.ph
```

Each unit initializes at most once.

If an ancestor package is already initialized, it is skipped.

If it is currently initializing because of a cycle, its existing partial module object is reused.

---

# 57. Module Initialization State

Each module/package has an explicit conceptual lifecycle:

```text
Discovered
    ↓
Declared
    ↓
Compiled
    ↓
Initializing
    ↓
Initialized
```

Failure produces:

```text
Initializing
    ↓
Failed
```

Exact internal implementation names are not normative, but the semantic distinctions are.

In particular, Phalcom must distinguish:

```text
a symbol does not exist
```

from:

```text
the symbol exists but its runtime value is not initialized yet
```

---

# 58. Early Registry Insertion

A module/package record is registered before its runtime initialization begins.

Therefore cycles terminate by finding the existing module record rather than recursively constructing another Module object.

One canonical identity implies one canonical runtime Module/Package object.

---

# 59. Circular Imports

Circular module imports are permitted.

Example:

```phalcom
// a.ph

from .b import B

class A {
}

export A
```

```phalcom
// b.ph

from .a import A

class B {
}

export B
```

The cycle itself is not automatically an error.

The declaration/interface phase knows that:

```text
a exports A
b exports B
```

before initialization completes.

Selective imports connect to those binding cells.

---

# 60. Selective Import During a Cycle Does Not Necessarily Read the Value

If an importing module encounters:

```phalcom
from .a import A
```

while `a` is already initializing, Phalcom may establish the live import binding without immediately forcing the current runtime value of `A`.

An error occurs when program initialization actually requires an uninitialized value.

This permits more cycles to exist safely than snapshot-import semantics would.

---

# 61. Uninitialized Binding Errors

If a module attempts to read an exported declaration whose cell is known but not yet initialized, Phalcom reports an initialization error rather than pretending the member does not exist.

For example:

```text
UninitializedModuleBindingError:
  `geometry.a.A` was read while `geometry.a`
  was still initializing.

Import cycle:
  geometry.a
  → geometry.b
  → geometry.a
```

This replaces misleading "missing member" behavior for known-but-not-yet-initialized exports.

---

# 62. Ordinary Missing Export Versus Uninitialized Export

These cases are semantically different.

If `foo` was never exported:

```phalcom
module.foo
```

is an inaccessible/missing public member.

If `foo` is declared and exported but its binding cell is not initialized yet during a cycle, access produces:

```text
UninitializedModuleBindingError
```

The runtime/compiler must preserve this distinction.

---

# 63. Module Initialization Is Synchronous

An import declaration synchronously ensures the target module is initialized before ordinary execution continues, except where an already-in-progress cycle returns the existing partial module record.

There is no implicit asynchronous module loading.

---

# 64. Modules Initialize Once

Within one runtime universe/process, a module's top-level initialization code runs at most once.

Repeated imports return the existing module object and/or existing exported binding cells.

This applies equally to modules first encountered through program execution and modules first encountered through imports.

---

# 65. Failed Initialization Is Sticky

If a module throws during initialization, its state becomes:

```text
Failed
```

Subsequent imports do not silently rerun its top-level code.

They propagate/report the existing module initialization failure.

Explicit development tooling may later provide reload semantics, but reload is not part of ordinary import behavior.

---

# 66. Standalone Modules

A single `.ph` file does not require `project.toml`.

Example:

```text
hello.ph
```

can be run as:

```text
phalcom run hello.ph
```

If it belongs to neither a project nor a package hierarchy, it executes as a standalone module.

It receives:

* its own module namespace;
* core visibility;
* any explicitly defined built-in import roots.

It does not receive an implicit user-defined package search path.

---

# 67. Standalone Modules Cannot Import Arbitrary Siblings

Given:

```text
scratch/
├── experiment.ph
└── helper.ph
```

there is no syntax meaning:

```text
load helper.ph because it happens to be next to experiment.ph
```

This does not exist:

```phalcom
import "./helper"
```

and this:

```phalcom
import helper
```

does not implicitly search the current directory.

If multiple files form one program, they should form a package or project.

---

# 68. Standalone Packages

A package does not require a project.

For example:

```text
demo/
├── package.ph
├── main.ph
└── helper.ph
```

is a valid standalone package.

Its root logical name is derived from the package directory name:

```text
demo
demo.main
demo.helper
```

assuming `demo` is a valid Phalcom identifier.

Within it:

```phalcom
from .helper import Helper
```

works normally.

Absolute self-imports may also use:

```phalcom
import demo.helper
```

because the standalone root package provides one explicit import root.

---

# 69. Invalid Standalone Package Name

A standalone package directory must itself be a valid Phalcom package identifier.

For example:

```text
my-tools/
    package.ph
```

cannot automatically become:

```text
my_tools
```

Attempting to use it as a standalone root package is an error.

To obtain a different stable namespace, create a project and declare:

```toml
namespace = "tools"
```

---

# 70. Nested Standalone Packages

When no project exists, a package hierarchy consists of contiguous package directories.

Example:

```text
demo/
├── package.ph
└── tools/
    ├── package.ph
    └── inspect.ph
```

Running `inspect.ph` determines its package context as:

```text
demo.tools.inspect
```

because both ancestor package directories are explicitly marked.

The outermost contiguous marked package is the standalone root.

---

# 71. Projects Supersede Standalone Package Identity

If the source is inside a recognized project's source root, project identity wins.

The project manifest determines the root namespace.

The checkout or package directory name does not.

Nested project boundaries are distinct project universes and may not be crossed by relative imports.

Another project must be referenced as a declared dependency.

---

# 72. Running a Module

Running:

```text
phalcom run src/tools/demo.ph
```

selects that exact source module as the process entry module.

If it belongs to a project, its project logical identity is used.

If it belongs to a standalone package, its standalone package identity is used.

Otherwise it is a standalone module.

Its parent packages, if any, initialize before it.

No duplicate `__main__` module is created.

---

# 73. Running and Importing the Same Module

Suppose:

```text
app.tools.demo
```

is selected as the entry module.

If some code subsequently imports:

```phalcom
import app.tools.demo
```

the loader finds the same registered module.

The source is not executed twice.

The invariant is:

```text
entry module identity = imported module identity
```

---

# 74. Entry Status Does Not Change Module Semantics

Being selected as the program entry point does not:

* rename the module;
* change its globals;
* automatically export anything;
* transform it into a function;
* alter relative import rules;
* bypass package initialization;
* create a special second namespace.

The process merely records that this module was selected as its entry.

An explicit reflection API may expose this fact later without introducing magic names such as `__main__`.

---

# 75. Running a Package

Running a package directory means running its conventional child entry module:

```text
<package>.main
```

Specifically, the package must contain:

```text
main.ph
```

Example:

```text
tools/
├── package.ph
├── main.ph
└── commands.ph
```

Then:

```text
phalcom run tools/
```

runs:

```text
tools.main
```

after initializing `tools/package.ph`.

---

# 76. `package.ph` Is Not the Package Executable Entry

This is deliberately not equivalent:

```text
phalcom run tools/
```

to:

```text
execute tools/package.ph as the application
```

`package.ph` is package initialization/API code and may execute whenever the package is imported.

Application startup code therefore belongs in:

```text
main.ph
```

or another explicitly selected project entry module.

This avoids package imports accidentally launching applications.

---

# 77. Direct `main.ph` Execution Equals Package Execution

Given:

```text
tools/
├── package.ph
└── main.ph
```

these select the same entry module:

```text
phalcom run tools/
```

and:

```text
phalcom run tools/main.ph
```

Both resolve to:

```text
tools.main
```

They receive the same package initialization and module identity.

---

# 78. Package Without `main.ph`

A package without a direct child `main.ph` is not executable through package-directory execution.

For example:

```text
geometry/
├── package.ph
└── point.ph
```

causes:

```text
phalcom run geometry/
```

to fail with a package-not-executable diagnostic.

Phalcom does not silently execute `package.ph`.

A child package:

```text
geometry/main/package.ph
```

does not satisfy the convention.

The executable package entry is specifically:

```text
geometry/main.ph
```

---

# 79. Explicitly Running `package.ph`

If a user explicitly selects:

```text
phalcom run geometry/package.ph
```

the selected source is the package module itself.

This means:

> initialize that package module as the explicitly selected entry source.

It does **not** redefine package-directory execution.

This explicit file-target behavior is mainly useful for direct diagnostics/testing; normal executable packages should use `main.ph`.

---

# 80. Running a Project

Running a project rather than an individual module/package uses the manifest entry.

For example:

```toml
[project]
namespace = "app"
entry = "app.cli"
```

then:

```text
phalcom run
```

from that project runs:

```text
app.cli
```

The entry must resolve to a regular module belonging to the current project.

It may not resolve to a dependency module.

---

# 81. Library Projects

A project is not required to be executable.

For example:

```toml
[project]
name = "linear-algebra"
namespace = "linalg"
source = "src"
```

with no `entry` is a valid library project.

Running the project itself reports:

```text
ProjectNotExecutableError
```

Individual modules may still be explicitly selected by tooling when useful.

---

# 82. Project Execution and Root-Package Execution Are Distinct

Suppose:

```toml
namespace = "app"
entry = "app.server"
```

and:

```text
src/main.ph
```

also exists.

Then:

```text
phalcom run
```

runs:

```text
app.server
```

while explicitly running the root package source directory:

```text
phalcom run src/
```

runs:

```text
app.main
```

This distinction is intentional:

```text
run project → manifest entry

run package → package.main
```

---

# 83. Directory Target Classification

A CLI directory target follows these semantic categories:

1. if selected as a project directory containing `project.toml`, run the project entry;
2. otherwise, if it is a package directory containing `package.ph`, run its `main.ph`;
3. otherwise, it is not a directly runnable Phalcom program unit.

If a project's source root is also the project directory, explicit project selection takes precedence over package convention.

---

# 84. Entry Modules Need Not Be Exported

Being executable is unrelated to being public.

For example:

```text
app.main
```

does not need to be exported through:

```text
app/package.ph
```

The resolver can select it directly by project/package execution rules.

Export visibility governs external module member access, not whether the compiler can resolve a module identity.

---

# 85. Entry Arguments

Running a module does not synthesize:

```phalcom
main(args)
```

or transform top-level source into a hidden callable.

The entry module initializes normally.

Process arguments belong to process/runtime context, for example through whatever standard process API Phalcom defines.

Conceptually:

```phalcom
const app = CLI(System.arguments)
app.run
```

The module system itself does not define a special `main` function signature.

---

# 86. Dependency Projects

Only projects are dependency-resolution units in the base specification.

A standalone package has no manifest dependency graph.

If reusable code requires dependencies and external consumers, it should normally become a project.

The manifest/package manager resolves dependency project instances before module resolution begins.

---

# 87. Project Dependency Graph

Project dependency resolution and version solving belong partly to package-manager specification, but the module system requires the result to be deterministic.

A resolved dependency root maps to one project instance.

Project dependency cycles should be rejected at project-resolution level.

Module import cycles inside the resulting resolved project graph remain governed by the module-cycle rules described above.

---

# 88. Package Imports Across Projects

Relative imports never cross project boundaries.

A dependency must be accessed through its explicit dependency root:

```phalcom
import math.matrix
```

not:

```phalcom
import ....someOtherProject
```

The same remains true for path dependencies configured in `project.toml`: the physical path belongs to manifest resolution, never to source import syntax.

---

# 89. Core Visibility

Phalcom's existing core visibility principle is retained.

Bare lookup is:

```text
current module bindings
then core
```

Core classes do not need explicit imports merely because the source is in a package/project.

Core is conceptually distinct from arbitrary package resolution.

If Phalcom defines a conventional standard-library root such as `std`, it participates as a reserved explicit import root under the same deterministic resolution rules.

---

# 90. No Namespace Pollution

Whole-module imports create exactly one explicit local binding.

Selective imports create exactly the listed local bindings.

Nothing else enters the importing module's namespace.

For example:

```phalcom
import geometry.point as point
```

does not inject:

```text
Point
origin
distance
geometry
```

unless explicitly requested.

---

# 91. No Parent-Package Implicit Globals

A module does not inherit globals or exports from its parent package.

Given:

```text
app/package.ph exports Config
```

inside:

```text
app/server.ph
```

this does not automatically exist:

```phalcom
Config
```

The module must explicitly import it:

```phalcom
from . import Config
```

This keeps every module's dependencies visible in its own source.

---

# 92. No Global Dependency Namespace

Declaring:

```toml
[dependencies]
json = ...
```

does not create a global runtime variable named `json`.

It merely establishes an absolute import root.

The module must still write:

```phalcom
import json
```

to create a local runtime binding.

---

# 93. Module Interface and LSP Semantics

Because module graphs and export tables are statically knowable, language tooling can determine:

* module existence;
* package hierarchy;
* exported declarations;
* aliases;
* re-exports;
* declaration kinds;
* definitions;
* cross-module types;
* superclass relationships.

For:

```phalcom
import .models as models

const user = models.User(...)
```

the LSP can resolve `models` to a `ModuleId`, read its `ModuleInterface`, discover exported `User`, and provide navigation/completion without executing module initialization.

This architecture is intentionally designed to support strong semantic tooling.

---

# 94. Module Object Access and Static Analysis

Qualified member access through a statically known module alias should receive special semantic knowledge from the compiler/LSP.

Given:

```phalcom
import .base as base
```

then:

```phalcom
base.
```

can offer only appropriate exported members of `base`.

This does not change runtime semantics: the runtime operation remains ordinary module object/member dispatch.

Static knowledge simply describes that runtime operation precisely.

---

# 95. Error Model

The module system should provide dedicated diagnostics rather than reducing every problem to a generic missing-member/file error.

Core categories include:

| Error                             | Meaning                                               |
| --------------------------------- | ----------------------------------------------------- |
| `ModuleNotFoundError`             | Logical module cannot be resolved                     |
| `PackageNotFoundError`            | Required package component does not exist             |
| `InvalidModuleLayoutError`        | Source tree violates package/module rules             |
| `AmbiguousModuleError`            | Both `x.ph` and `x/package.ph` claim one identity     |
| `InvalidModuleNameError`          | Filesystem component cannot form a logical identifier |
| `UnknownImportRootError`          | Absolute import root is unknown                       |
| `ImportRootCollisionError`        | Project/dependency/builtin roots collide              |
| `RelativeImportBeyondRootError`   | Relative import climbs above root package             |
| `ImportNameError`                 | Selective import names a non-exported/unknown member  |
| `DuplicateImportBindingError`     | Import creates an already-bound local name            |
| `UnknownExportError`              | `export` refers to invalid local binding              |
| `DuplicateExportError`            | Multiple exports claim same public name               |
| `UninitializedModuleBindingError` | Known export read during incomplete initialization    |
| `ModuleInitializationError`       | Module initialization failed                          |
| `PackageNotExecutableError`       | Package has no `main.ph`                              |
| `ProjectNotExecutableError`       | Project declares no entry                             |
| `InvalidProjectEntryError`        | Manifest entry cannot resolve to valid project module |
| `ImportOutsideSourceRootError`    | Canonical source escapes authorized root              |

Exact type names may follow Phalcom's normal error naming conventions, but the semantic distinctions are normative.

---

# 96. Complete Example

Project:

```text
geometry-kit/
├── project.toml
└── src/
    ├── package.ph
    ├── main.ph
    ├── point.ph
    ├── vector.ph
    └── shapes/
        ├── package.ph
        ├── base.ph
        └── circle.ph
```

Manifest:

```toml
[project]
name = "geometry-kit"
namespace = "geometry"
source = "src"
entry = "geometry.main"
```

`point.ph`:

```phalcom
class Point {
    // ...
}

const origin = Point(0, 0)

const cache = ...

export Point, origin
```

`vector.ph`:

```phalcom
class Vector {
    // ...
}

export Vector
```

`shapes/base.ph`:

```phalcom
class Shape {
    // ...
}

export Shape
```

`shapes/circle.ph`:

```phalcom
from .base import Shape
from ..point import Point

class Circle is Shape {
    // ...
}

export Circle
```

`shapes/package.ph`:

```phalcom
from .base import Shape
from .circle import Circle

export Shape, Circle
```

root `package.ph`:

```phalcom
from .point import Point, origin
from .vector import Vector
from .shapes import Shape, Circle

export Point, origin, Vector, Shape, Circle
```

`main.ph`:

```phalcom
from . import Point, Circle

const center = Point(0, 0)
const circle = Circle(center: center, radius: 10)

System.print(circle)
```

---

# 97. Valid Consumer Forms

Whole root package:

```phalcom
import geometry

const p = geometry.Point(1, 2)
```

Aliased root package:

```phalcom
import geometry as Geometry

const p = Geometry.Point(1, 2)
```

Direct module:

```phalcom
import geometry.point as point

const p = point.Point(1, 2)
```

Selective:

```phalcom
from geometry.point import Point, origin
```

Package façade:

```phalcom
from geometry import Point, Circle
```

Nested module:

```phalcom
from geometry.shapes.circle import Circle
```

Relative sibling:

```phalcom
from .point import Point
```

Relative parent:

```phalcom
from ..point import Point
```

Qualified superclass:

```phalcom
import .base as base

class Circle is base.Shape {
}
```

---

# 98. Invalid or Intentionally Unsupported Forms

Physical imports:

```phalcom
import "./point"
```

Invalid.

Filesystem extension imports:

```phalcom
import "point.ph"
```

Invalid.

Wildcard import:

```phalcom
from geometry import *
```

Invalid.

Wildcard export:

```phalcom
export *
```

Invalid.

Import inside method:

```phalcom
run() {
    import geometry
}
```

Invalid.

Implicit sibling import:

```phalcom
import point
```

when `point` merely happens to be a sibling module.

Invalid unless `point` is an explicit absolute import root.

Implicit re-export:

```phalcom
from .point import Point
```

does not export `Point`.

Automatic child-module package member:

```phalcom
import geometry as Geometry
Geometry.point
```

does not work merely because `geometry.point` exists.

---

# 99. Deferred Features

The following are deliberately outside the base specification:

* wildcard imports;
* wildcard exports;
* `export X from module` shorthand;
* package-private/project-private module paths;
* runtime import declarations inside local scopes;
* Python-style namespace packages;
* user-defined import search paths;
* environment-driven module resolution;
* automatic directory packages;
* runtime project objects;
* explicit module reload semantics;
* multiple executable `[[bin]]` manifest targets;
* workspace-level semantics;
* direct import syntax for compiled bytecode artifacts.

These can be added independently without weakening the base model.

---

# 100. Compiled Modules and Caches

Import syntax names logical modules, never source artifacts.

Therefore a future compiler may satisfy:

```phalcom
import geometry.point
```

using:

* source compilation;
* cached bytecode;
* ahead-of-time compiled artifacts;

without changing language semantics.

The artifact is an implementation detail associated with the same canonical `ModuleId`.

There will never be syntax such as:

```phalcom
import "./point.phc"
```

---

# 101. Migration from U15

Current U15:

```phalcom
import "./geometry/point" as Point
```

must migrate to logical imports.

For a sibling/package-relative target:

```phalcom
import .geometry.point as Point
```

or absolute:

```phalcom
import app.geometry.point as Point
```

Selective importing becomes:

```phalcom
from .geometry.point import Point
```

Top-level declarations that were previously externally visible through Module member access must now be explicitly exported:

```phalcom
class Point {
}

export Point
```

`as` is no longer mandatory for whole-module imports.

Physical import strings are not retained as legacy semantics.

A migration-era compiler may issue a targeted diagnostic suggesting the logical replacement, but quoted physical imports are no longer part of the language.

---

# 102. Final Normative Decisions

The Phalcom module system is therefore defined by the following final decisions:

1. Every ordinary `.ph` source unit is a module.
2. Every package is backed by a mandatory `package.ph`.
3. `package.ph` represents the package itself, not a child named `package`.
4. Packages are explicit; ordinary directories do not create namespaces.
5. A project owns exactly one root package.
6. `project.toml` establishes project namespace, source root, dependencies, and optional entry module.
7. The project itself is not an additional importable runtime namespace above its root package.
8. Source imports are always logical.
9. Physical/path-string imports do not exist.
10. Logical imports never contain `.ph`.
11. CLI tooling may accept filesystem paths to select source targets.
12. Absolute imports begin from an explicit root namespace.
13. Valid roots are the current project/package root, declared dependency aliases, and reserved built-in roots.
14. There is no ambient module search path.
15. Relative imports operate on package ancestry, never filesystem ancestry.
16. One leading dot means the current package context; each additional dot ascends one package.
17. Relative imports may never climb above the root package.
18. `import a.b.c` binds `c` by default.
19. `as` aliases are optional.
20. `from M import A, B as C` provides selective imports.
21. Whole-module and selective import bindings are immutable.
22. Selective imports are live read-only references to exported binding cells.
23. Imports are permitted only directly at module/package scope.
24. Import names enter lexical scope only at their source position.
25. Import targets and interfaces are statically resolved before runtime initialization.
26. Successful module initialization still occurs synchronously at the import's source position.
27. Bare lookup searches the current module and then core only.
28. Parent package bindings are never implicitly injected into child modules.
29. Module bindings are private by default.
30. `export` explicitly defines the module's public interface.
31. `_` has no privacy semantics.
32. There is no separate module-global `private` declaration requirement.
33. Export aliases are supported.
34. Imported bindings are not automatically re-exported.
35. Wildcard imports and exports are not supported.
36. Package APIs are explicitly constructed through `package.ph` exports.
37. Child modules do not automatically become members of their parent package objects.
38. `from package import X` never falls back to searching for child module `package.X`.
39. Module-path privacy is not a separate visibility dimension in the base specification.
40. External access to a Module/Package object sees only its exports.
41. External code cannot arbitrarily rebind another module's globals.
42. Module interfaces exist independently of runtime initialization.
43. Imported classes may be resolved as cross-module superclasses.
44. Qualified static references such as `base.Shape` are supported when rooted in known module aliases.
45. Parent packages initialize outermost-to-innermost before a nested target module.
46. Module objects are registered before initialization begins.
47. Import cycles terminate through stable module identity and early registry insertion.
48. Declared/exported-but-uninitialized bindings are distinguishable from nonexistent bindings.
49. Circular imports fail only when initialization actually requires a value that is not yet initialized.
50. Modules initialize at most once.
51. Failed initialization is sticky for the runtime session.
52. A standalone `.ph` module may execute without a project.
53. A standalone module does not gain implicit sibling imports.
54. A directory containing `package.ph` may form a standalone package without `project.toml`.
55. Standalone package identity derives from its valid directory name.
56. Standalone packages may use logical relative and self-root absolute imports.
57. A project gives its source modules project-defined stable identities.
58. Project identity takes precedence when a source belongs to a project's source root.
59. Running a module executes that canonical module as the process entry module.
60. Direct execution does not create a `__main__` duplicate.
61. Importing the entry module returns the same module object.
62. Running a package directory runs its direct `main.ph`.
63. `package.ph` is package initialization/API code, not the package-directory executable entry.
64. Running `package/main.ph` directly and running the package directory select the same module.
65. A package without `main.ph` is not directly executable as a package.
66. Running a project executes its manifest-declared entry module.
67. A project without an entry remains a valid library project.
68. Project execution and root-package execution are intentionally distinct.
69. Dependency aliases are source-local resolver roots, not canonical module identities.
70. Canonical project module identity is project-instance identity plus project-relative module path.
71. Physical source canonicalization enforces confinement and detects duplicate source identities.
72. Dependency source trees are reached only through declared project dependencies.
73. Relative imports can never cross project boundaries.
74. No current-working-directory, environment variable, or arbitrary installation-directory search participates in source module resolution.
75. The resulting module graph is deterministic, statically inspectable, tooling-friendly, and independent of checkout location.

The resulting design is intentionally **Python-like in source-tree ergonomics, Rust-like in project/dependency determinism, and more explicit than either in namespace/API semantics**. It preserves Phalcom's runtime-object model for modules while giving the compiler and LSP a stable semantic graph suitable for inheritance resolution, type analysis, completion, navigation, incremental compilation, and future compiled-module caching.
