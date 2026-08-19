# Phalcom Modularity, Reflection, Package Metadata, and User-Facing Interface
## Implementation Specification — Reflection and Artifact Surface Track

**Repository:** `aureat/phalcom-lang`
**Baseline commit:** `f0e51699060d31722c68b282a2d2e9a5b3260dfe`
**Status:** implementation-ready target specification
**Depends on:** `phalcom-module-runtime-repl-repair-implementation-spec.md`

---

# 0. Purpose

This specification implements the user-visible object model and reflection/package metadata surface that becomes possible after the runtime repair track separates Project from Package and converges the REPL on the normal module system.

The target model is:

```text
DEVELOPMENT ONLY

Project
├── manifest: ProjectManifest
├── dependencies: Tuple<ResolvedProjectDependency>
├── developmentEntry
└── rootPackage ────────────────────────────────┐
                                                │
                                                ▼
DURABLE PACKAGE/MODULE MODEL                Package : Module
                                           ├── package
                                           ├── parentPackage
                                           ├── rootPackage
                                           ├── packageInfo
                                           ├── children
                                           ├── exports
                                           └── descendant Modules/Packages
                                                    │
                                                    └── packageInfo
```

Publication removes the development context:

```text
Project            stripped
ProjectManifest    stripped as development configuration

Package            retained
PackageInfo        retained
module hierarchy   retained
exports            retained
durable metadata   retained
```

The central API distinction is:

> **Context dunders describe the currently executing source unit. Ordinary getters navigate arbitrary Module/Package/Project objects.**

That means:

```phalcom
__module__
__package__
__root__
__project__
```

are not the primary object navigation interface. The ordinary user-facing interface is:

```phalcom
module.package
module.rootPackage
module.packageInfo
module.exports
module.metadata
module.uri

package.parentPackage
package.rootPackage
package.packageInfo
package.children
package.isRoot

project.rootPackage
project.manifest
project.dependencies
project.developmentEntry
```

This specification also fixes remaining interface inconsistencies:

- `__path__` is replaced by a canonical logical URI surface;
- `__parent__` is not the preferred ordinary API;
- `__info__` is replaced by explicit `packageInfo`;
- manifest/development metadata is separated from durable PackageInfo;
- PackageInfo requirements are separated from Project resolved dependencies and Module dependencies;
- reflection containers are immutable, deterministic, and cached;
- export reflection describes live bindings rather than copied values;
- source `@!` metadata remains distinct from package metadata;
- builtin packages expose the same Package/PackageInfo interface as imported packages without fake Projects;
- dunder member access is implemented as standardized reflection/message protocol, not field-only syntax.

---

# 1. Context-Minimizing Implementation Protocol

Do not begin by reading all universe, primitive, reflection, manifest, and module files. The implementation order below is designed to keep context compact.

## 1.1 Prerequisite verification

Before implementing this spec, verify the companion runtime track has established:

```text
ModuleKind = Module | Package
Project !< Package
canonical universe Package
canonical std Package
REPL import convergence
context intrinsics have a real runtime/compiler seam
```

If any prerequisite is absent, stop this track and implement that prerequisite rather than adding compatibility branches here.

## 1.2 First-pass reads

Read only:

1. `phalcom-core/src/heap/module.rs`
   - ModuleObject relationships/exports/metadata
2. `phalcom-core/src/universe/core_classes.rs`
   - Module/Package/Project class creation
3. `phalcom-core/src/modules/materialize.rs`
   - module allocation and export materialization
4. `phalcom-modules/src/dunder.rs`
   - standardized dunder categories
5. `phalcom-modules/src/metadata.rs`
   - source `@!` metadata representation
6. `phalcom-modules/src/manifest.rs`
   - raw/validated manifest data
7. `phalcom-modules/src/interface.rs`
   - exports/exposed children/interface ordering
8. `phalcom-modules/src/linker.rs`
   - linked export/read descriptors
9. `phalcom-core/core/universe/src/reflection/module.ph`
10. `phalcom-core/core/universe/src/reflection/package-object.ph`
11. `phalcom-core/core/universe/src/reflection/project.ph`
12. `phalcom-native-meta/src/universe.rs`
13. the new runtime-track files created by the companion implementation.

Do not open compiler internals until the dunder protocol phase; use targeted `rg` then.

---

# 2. Baseline Source Map

Pinned to `f0e516...`; use symbol anchors if line numbers drift.

| File | Baseline range | Anchor |
|---|---:|---|
| `phalcom-modules/src/dunder.rs` | ~L1-L70 | `DunderCategory`, default standardized dunders |
| `phalcom-modules/src/metadata.rs` | ~L1-L55 | `MetadataTarget`, `ModuleMetadata::from_ast` |
| `phalcom-modules/src/manifest.rs` | ~L13-L55 | raw `ProjectSection`, `ValidatedProjectManifest` |
| `phalcom-modules/src/manifest.rs` | ~L56-L170 | semantic validation; version/authors currently dropped |
| `phalcom-modules/src/interface.rs` | ~L18-L105 | export/interface structures |
| `phalcom-modules/src/interface.rs` | ~L106-L260 | InterfaceBuilder, imports/exposure/exports |
| `phalcom-modules/src/linker.rs` | ~L15-L75 | `SymbolId`, `LinkedReadSpec`, linked module layout |
| `phalcom-modules/src/linker.rs` | anchor `LinkedExportTarget` use | binding vs module export resolution |
| `phalcom-core/src/heap/module.rs` | ~L20-L95 | `RuntimeExportRef`, ModuleObject fields |
| `phalcom-core/src/modules/materialize.rs` | ~L250-L315 | runtime export table construction |
| `phalcom-core/src/universe/core_classes.rs` | anchor `module_class`, `package_class`, `project_class` | reflection/runtime classes |
| `phalcom-core/core/universe/src/reflection/module.ph` | entire file | currently documentation shell |
| `phalcom-core/core/universe/src/reflection/package-object.ph` | entire file | currently documentation shell |
| `phalcom-core/core/universe/src/reflection/project.ph` | entire file | currently documentation shell |
| `phalcom-core/core/universe/src/package.ph` | entire file | universe exposures |
| `phalcom-core/core/std/src/package.ph` | entire file | std exposures |
| `phalcom-native-meta/src/universe.rs` | anchor `UNIVERSE_BINDINGS` | exported/prelude native universe binding authority |

---

# 3. Normative Public Model

## 3.1 Module

A runtime namespace backed by exactly one source unit.

## 3.2 Package

A specialized Module backed by `package.ph`, capable of containing/exposing child Modules and Packages.

## 3.3 Project

A live development environment. It is neither Module nor Package. It is available only for source executing in an active project development context.

## 3.4 ProjectManifest

Immutable validated development configuration derived from `project.toml`.

## 3.5 PackageInfo

Immutable durable metadata describing the root package artifact. It exists independently of Project and survives publication.

## 3.6 Three dependency layers

```text
Module.dependencies
    runtime/source module dependencies

PackageInfo.requirements
    durable package requirements

Project.dependencies
    current development environment's resolved dependencies
```

Never collapse these into one generic `dependencies` representation.

---

# 4. Context Intrinsics Versus Object Navigation

This distinction is normative.

## 4.1 Context intrinsics

```phalcom
__module__  -> Module
__package__ -> Option<Package>
__root__    -> Option<Package>
__project__ -> Project
```

`__project__` is available only in a real Project development context.

Recommended standalone semantics:

```text
__module__   available
__package__  available, returns None
__root__     available, returns None
__project__  unavailable/context diagnostic
```

## 4.2 Ordinary getters

```phalcom
module.package
module.rootPackage
module.packageInfo
module.uri

package.parentPackage
package.rootPackage
package.packageInfo

project.rootPackage
```

A user should not need dunders for ordinary navigation.

## 4.3 Why

This prevents ambiguity such as:

```phalcom
somePackage.__package__
```

being mistaken for "parent package." The contextual `__package__` answers:

> What Package context does the currently executing source use for relative imports?

The arbitrary-object API answers with explicit names.

---

# 5. New/Changed Runtime Classes

The following classes must become real universe-visible classes.

## 5.1 Existing classes to repair

```text
Module
Package
Project
```

`Project` must already be independent from Package due to the runtime track.

## 5.2 New classes

Create runtime/universe classes:

```text
ProjectManifest
PackageInfo
PackageAuthor
PackageRequirement
ResolvedProjectDependency
ModuleDependency
ExportTable
Export
ExportKind
ChildModuleTable
ModuleIdentity
PackageIdentity
ProjectIdentity
Uri
```

Some may reuse existing general-purpose URI/identity types if Phalcom already has suitable equivalents. Do not duplicate a mature existing `Uri` class.

## 5.3 Recommended implementation strategy

Use native-backed reflection descriptors where values are runtime/toolchain-created and immutable.

Why:

- descriptor values are not normally user-constructed;
- canonical caching is required;
- they frequently wrap compiler/runtime identities;
- they need safe live access to module slots without exposing slots;
- generic mutable InstanceObjects would need defensive copying and identity management.

Provide `.ph` files for Phaldoc/protocol declarations and any methods that can be expressed safely in Phalcom.

### Alternative

Ordinary immutable instances with private native constructor hooks are possible. Choose this only if the runtime already has a robust immutable-instance pattern with canonical interning.

---

# 6. Proposed Universe Source Layout

Under:

```text
phalcom-core/core/universe/src/reflection/
```

retain/update:

```text
module.ph
package-object.ph
project.ph
```

Create:

```text
project-manifest.ph
package-info.ph
package-author.ph
package-requirement.ph
resolved-project-dependency.ph
module-dependency.ph
export-table.ph
export.ph
export-kind.ph
child-module-table.ph
module-identity.ph
package-identity.ph
project-identity.ph
uri.ph              # only if no canonical Uri already exists
```

Update `reflection/package.ph` to expose these children.

If a class is intended to be prelude-visible, declare that through native universe metadata rather than accidental core-global scanning.

Most reflection descriptor classes SHOULD NOT be prelude names. They should be discoverable through `universe.reflection` and through returned values.

---

# 7. Module Public Interface

The preferred user-facing interface should be approximately:

```phalcom
/// A loaded Phalcom source module.
///
/// A Module is one canonical runtime namespace backed by one semantic source
/// unit. Importing, re-exporting, or directly executing the same source does not
/// create another Module identity.
class Module {

    /// The package directly containing this module.
    ///
    /// Returns None for a genuinely standalone module.
    package -> Option<Package>

    /// The root package of this module's namespace hierarchy.
    ///
    /// Returns None for a genuinely standalone module.
    rootPackage -> Option<Package>

    /// Durable package information for the root package containing this module.
    ///
    /// Returns None for a genuinely standalone module that belongs to no Package.
    packageInfo -> Option<PackageInfo>

    /// Canonical immutable public export view.
    exports -> ExportTable

    /// Source-level retained @! metadata.
    metadata -> ModuleMetadata

    /// Module-level runtime/reference dependencies.
    dependencies -> Tuple<ModuleDependency>

    /// Canonical logical source URI.
    uri -> Uri

    /// Opaque semantic identity.
    identity -> ModuleIdentity
}
```

## 7.1 Reflection protocol aliases

Where standardized reflection is useful, support:

```phalcom
module.__name__
module.__id__
module.__uri__
module.__exports__
module.__export__(#name)
module.__metadata__
module.__dependencies__
module.__understands__(#name)
```

Do not require users to prefer these over ordinary getters.

## 7.2 `__path__`

Current dunder registry contains `__path__`.

Recommendation:

- deprecate/remove `__path__` as canonical identity;
- replace with `__uri__`;
- optionally expose `sourceLocation -> Option<SourceLocation>` for tooling/debugging;
- do not promise an absolute filesystem path for builtins, packaged binaries, generated modules, or future native artifacts.

---

# 8. Package Public Interface

```phalcom
/// A Module namespace backed by package.ph.
///
/// Package is a specialized Module. The Package object is the one runtime
/// object corresponding to its package.ph source unit.
class Package : Module {

    /// This package's own package context.
    ///
    /// Package package modules use themselves as the current package context.
    package -> Package

    /// Enclosing Package, or None for a root Package.
    parentPackage -> Option<Package>

    /// Root Package of this namespace hierarchy.
    ///
    /// A root Package returns itself.
    rootPackage -> Package

    /// Durable information for the root package artifact.
    ///
    /// Nested packages return the same canonical PackageInfo as their root.
    packageInfo -> PackageInfo

    /// Immutable exposed-child view.
    ///
    /// Exposure controls import-path traversal and is distinct from exports.
    children -> ChildModuleTable

    /// Whether this is the root Package.
    isRoot -> Bool
}
```

## 8.1 Required invariants

```phalcom
root.package === root
root.rootPackage === root
root.parentPackage == None
root.isRoot == true

nested.package === nested
nested.parentPackage == Some(root)
nested.rootPackage === root
nested.packageInfo === root.packageInfo
nested.isRoot == false
```

## 8.2 Module descendant invariant

```phalcom
circle.package == Some(shapes)
circle.rootPackage == Some(geometry)
circle.packageInfo == Some(geometry.packageInfo)
```

---

# 9. Project Public Interface

Project is a live development object only.

```phalcom
/// A live Phalcom project development environment.
///
/// Project is not an import namespace and is not retained as part of a
/// published package artifact.
class Project {

    /// Project/distribution development name.
    name -> String

    /// Root package namespace.
    namespace -> Symbol

    /// Validated development manifest.
    manifest -> ProjectManifest

    /// The Package owned as this Project's source root.
    rootPackage -> Package

    /// Resolved development dependencies.
    dependencies -> Tuple<ResolvedProjectDependency>

    /// Optional development execution entry.
    developmentEntry -> Option<ModuleIdentity>

    /// Opaque identity for this live development context.
    identity -> ProjectIdentity
}
```

## 9.1 Explicitly absent Project protocol

Do not add:

```text
Project.package
Project.parentPackage
Project.children
Project.exports
Project.__package__
Project.__root__
```

These are namespace concepts.

The fact that Project owns `rootPackage` does not make Project part of that hierarchy.

## 9.2 Project construction

Project object materialization should occur only for the **active development project context**.

Dependency packages do not need user-visible Project wrappers.

A `ResolvedProjectDependency` may expose its package/root Package/PackageInfo without exposing a dependency Project.

---

# 10. ProjectManifest Interface

`ProjectManifest` represents validated semantic configuration, not the raw TOML syntax tree.

```phalcom
/// Immutable validated project-development configuration.
class ProjectManifest {
    name -> String
    namespace -> Symbol

    version -> Option<Version>
    authors -> Tuple<PackageAuthor>
    description -> Option<String>
    license -> Option<String>
    homepage -> Option<Uri>
    repository -> Option<Uri>

    /// Declared project-relative source configuration, not an absolute host path.
    source -> ProjectPath

    /// Optional development execution entry declaration.
    entry -> Option<ModulePath>

    /// Declared dependency requirements before/currently independent of resolution.
    dependencyDeclarations -> Tuple<ProjectDependencyDeclaration>
}
```

## 10.1 Current manifest repair

At baseline, raw `ProjectSection` retains `version` and `authors`, but `ValidatedProjectManifest` drops them.

Change validated representation to retain all durable/project fields required by PackageInfo construction.

Add manifest parsing for:

```toml
description = "..."
license = "..."
homepage = "..."
repository = "..."
```

## 10.2 Structured authors

Do not make the semantic API permanently `Tuple<String>`.

Introduce:

```phalcom
class PackageAuthor {
    name -> String
    email -> Option<String>
    url -> Option<Uri>
}
```

Manifest syntax can initially accept strings and normalize them:

```toml
authors = ["Ada Example"]
```

Later it can accept structured forms without changing runtime API.

## 10.3 Manifest source path

Do not expose absolute machine-specific source roots as the manifest's durable value.

Separate:

```text
ProjectManifest.source
    declared semantic/project-relative path

Project.sourceRoot/sourceLocation
    resolved development location, if a public getter is later needed
```

---

# 11. PackageInfo Interface

PackageInfo is the durable metadata boundary.

```phalcom
/// Immutable descriptive information associated with a root package artifact.
///
/// PackageInfo survives independently of the Project used to develop the package.
class PackageInfo {

    /// Distribution/package artifact name.
    name -> String

    /// Root language namespace.
    namespace -> Symbol

    version -> Option<Version>
    authors -> Tuple<PackageAuthor>
    description -> Option<String>
    license -> Option<String>
    homepage -> Option<Uri>
    repository -> Option<Uri>

    /// Durable dependency requirements.
    requirements -> Tuple<PackageRequirement>

    /// Optional durable default executable entry.
    defaultEntry -> Option<PackageEntry>

    /// Opaque package artifact identity.
    identity -> PackageIdentity
}
```

## 11.1 Package name vs namespace

This distinction must be documented and tested.

Example:

```toml
name = "phalcom-http-client"
namespace = "http"
```

Then:

```phalcom
http.name
// if Module/Package ordinary name getter exists: #http

http.packageInfo.name
// "phalcom-http-client"

http.packageInfo.namespace
// #http
```

Never make `Package.name` ambiguously mean distribution name.

## 11.2 Sources

PackageInfo can be constructed from:

```text
development Project:
    validated manifest + validated package facts

published artifact:
    artifact metadata

builtin:
    toolchain builtin metadata
```

All three produce the same runtime class.

## 11.3 No fake manifest

Builtin PackageInfo must not manufacture a `ProjectManifest`.

For:

```phalcom
universe.packageInfo
std.packageInfo
```

the info exists, but no Project/manifest exists.

---

# 12. PackageRequirement Interface

Package requirements are durable and unresolved.

```phalcom
/// A dependency requirement embedded in package metadata.
class PackageRequirement {
    /// Local import root used by this package's source.
    alias -> Symbol

    /// Registry/distribution package name.
    package -> String

    /// Version constraint.
    versionRequirement -> VersionRequirement

    /// Optionality, if/when optional dependencies are supported.
    optional -> Bool
}
```

## 12.1 Alias is mandatory semantic information

For:

```toml
[dependencies]
lin = { package = "linear-algebra", version = "^2" }
```

the package source may contain:

```phalcom
import lin.matrix
```

Therefore publication metadata must preserve `lin`.

## 12.2 Path dependency publication

A development-only path dependency:

```toml
util = { path = "../util" }
```

must either:

1. resolve to a publishable package identity/version and normalize to `PackageRequirement`; or
2. cause publication validation failure.

Never serialize `"../util"` into durable package metadata.

---

# 13. ResolvedProjectDependency Interface

This represents current development resolution, not durable artifact requirements.

```phalcom
/// A dependency as resolved inside the active development Project.
class ResolvedProjectDependency {
    alias -> Symbol

    /// The requirement that led to the resolution, if applicable.
    requirement -> Option<PackageRequirement>

    /// Durable metadata of the resolved package.
    packageInfo -> PackageInfo

    /// Canonical runtime root Package when loaded/materialized.
    rootPackage -> Package

    /// How this development environment obtained the package.
    origin -> PackageOrigin
}
```

Do not expose internal `ResolvedProjectId` numeric graph IDs.

---

# 14. PackageOrigin

Package origin is environment-specific provenance and therefore does not belong inside portable PackageInfo identity.

Recommended:

```phalcom
enum PackageOrigin {
    builtin
    workspace
    path
    registry
    vendored
    embedded
}
```

The exact set may begin smaller.

Important distinction:

```text
PackageInfo:
    what package artifact is this?

PackageOrigin:
    where/how did this loaded instance come from?
```

The same package artifact can be obtained from a cache or vendored store without becoming a different package identity.

---

# 15. ModuleDependency Interface

Keep module dependencies distinct:

```phalcom
/// One semantic/runtime dependency from a Module to another Module/Package.
class ModuleDependency {
    module -> Module
    phase -> DependencyPhase
    reason -> DependencyReason
}
```

If the existing graph already has richer types (`DependencyPhase`, runtime reasons), map them rather than inventing a second graph.

Do not expose compiler-only graph nodes directly.

---

# 16. Export Reflection

## 16.1 Problem with returning a Map of values

Exports can point to mutable bindings.

Example:

```phalcom
var counter = 0
export counter
```

A cached value Map would become stale.

## 16.2 ExportTable

Create a canonical immutable descriptor:

```phalcom
/// Immutable reflective view of a Module's public export surface.
///
/// Descriptors are stable; binding values are resolved live.
class ExportTable {
    names -> Tuple<Symbol>
    size -> Int

    contains(name: Symbol) -> Bool
    descriptor(name: Symbol) -> Option<Export>
    get(name: Symbol) -> Option<Object>
}
```

## 16.3 Export

```phalcom
class Export {
    name -> Symbol
    kind -> ExportKind
    module -> Module

    /// Resolve the current exported value.
    value -> Object
}
```

## 16.4 ExportKind

At minimum:

```phalcom
enum ExportKind {
    binding
    module
}
```

This maps directly to existing runtime:

```rust
RuntimeExportRef::Binding(...)
RuntimeExportRef::Module(...)
```

without exposing slot indices.

## 16.5 Stable descriptor/live value invariant

```phalcom
module.exports === module.exports

let descriptor = module.exports.descriptor(#counter).unwrap
counter = 5

descriptor === module.exports.descriptor(#counter).unwrap
descriptor.value == 5
```

---

# 17. ChildModuleTable

Package child exposure is separate from export.

```phalcom
/// Immutable exposed-child view of a Package.
///
/// A child can be legal in an import path without being exported as a member.
class ChildModuleTable {
    names -> Tuple<Symbol>
    size -> Int
    contains(name: Symbol) -> Bool
    get(name: Symbol) -> Option<Module>
}
```

## 17.1 Normative distinction

```phalcom
expose .point
```

means:

```phalcom
import geometry.point
```

is permitted.

It does **not** alone guarantee:

```phalcom
geometry.point
```

as a member send.

To make member access work, the package facade must also export a module binding.

This distinction must appear in Phaldoc on both `children` and `exports`.

---

# 18. Builtin Facade Policy

General language semantics must not special-case dotted traversal, but builtins can choose a richer facade through ordinary exports.

## 18.1 Recommended builtin facade

For ergonomic reflection and discovery, root builtin packages should export their principal exposed children in addition to exposing them.

Then:

```phalcom
universe.reflection
universe.callable

import std
std.json
std.io
```

can work.

Implementation should construct ordinary module exports:

```rust
LinkedExportTarget::Module(...)
```

rather than add a VM branch saying "if receiver is builtin package, traverse child graph."

## 18.2 Alternative

Keep builtins path-import-only:

```phalcom
import std.json
```

while `std.json` member access is invalid unless explicitly exported.

This is semantically purer but less discoverable. The recommended facade is better for builtins and reflection.

## 18.3 Tests

Test both distinctions:

```text
generic package with expose only:
    import child works
    package.child member lookup fails

builtin root with expose + export:
    import child works
    package.child works
```

---

# 19. Dunder Registry Redesign

## 19.1 Current state

`phalcom-modules/src/dunder.rs` currently classifies:

Intrinsic:

```text
__module__
__package__
__project__
```

Guaranteed reflection:

```text
__name__
__id__
__path__
__exports__
__export__
__metadata__
__understands__
__parent__
__children__
__namespace__
__dependencies__
__version__
...
```

## 19.2 Target categories

Context intrinsics:

```text
__module__
__package__
__root__
__project__
```

Guaranteed reflection protocol:

```text
__name__
__id__
__uri__
__exports__
__export__
__metadata__
__understands__
__children__       # Package where meaningful
__namespace__
__dependencies__
__version__        # only on semantically relevant receiver
...
```

Do not interpret "guaranteed reflection protocol" as "every class answers every dunder." It means the language owns the spelling and semantics where applicable.

## 19.3 Ordinary API replacements

Prefer:

```text
__parent__       -> parentPackage
__path__         -> uri / __uri__
__info__         -> packageInfo (do not add generic __info__)
```

It is reasonable to keep `__parent__` as a standardized reflection protocol for Package tooling, but documentation should steer ordinary code to `parentPackage`.

## 19.4 Future dunder hooks

Preserve the existing `Hook { roles }` policy. This is important for future selectively overridable dunders such as a message-send interception protocol.

Do not weaken the full dunder namespace reservation.

---

# 20. Fix Dunder Member Dispatch

The observed baseline behavior for:

```phalcom
universe.__dependencies__
```

produced a field-oriented error rather than invoking reflection semantics.

That means reservation policy and member lowering are not enough.

## 20.1 Required semantics

A reflection dunder getter is a zero-argument standardized message/property on a compatible receiver.

It is not a hidden mutable field.

```phalcom
module.__exports__
package.__children__
```

must work on arbitrary compatible module/package objects, not only `self`.

## 20.2 Targeted compiler/runtime read

Only during this phase:

```bash
rg -n "__dependencies__|__exports__|field.*self|Expected a field|GetField|member" \
  phalcom-core/src phalcom-ast/src
```

Identify the branch interpreting dunder-like member syntax as self-field access.

Route guaranteed reflection selectors through normal send/property dispatch or an explicit intrinsic protocol dispatch.

## 20.3 Do not expose mutable fields

Even if native implementations use slots internally, the surface remains read-only getters.

---

# 21. Reflection Cache Architecture

## 21.1 Principle

> Reflection over immutable semantic structure returns canonical immutable descriptor objects.

Required identity invariants:

```phalcom
module.exports === module.exports
package.children === package.children
package.packageInfo === package.packageInfo
project.manifest === project.manifest
project.dependencies === project.dependencies
```

## 21.2 New cache

Recommended:

```text
phalcom-core/src/modules/reflection_cache.rs
```

Either attach cache handles to Module/Package/Project native objects or maintain a VM-owned cache keyed by canonical identity.

Suggested keys:

```rust
enum ReflectionCacheKey {
    ModuleExports(ModuleId),
    PackageChildren(ModuleId),
    PackageInfo(PackageArtifactIdentity),
    ProjectManifest(ProjectRuntimeId),
    ProjectDependencies(ProjectRuntimeId),
}
```

Do not key public reflection by raw object address if canonical semantic identity exists.

## 21.3 GC

If cached descriptors are heap objects, they must be reachable/marked.

Prefer ownership from the Module/Package/Project object or VM roots so descriptor lifetime naturally follows its owner.

## 21.4 Invalidation

For ordinary linked program modules:

- export structure is immutable after linking;
- child exposure is immutable;
- PackageInfo is immutable;
- ProjectManifest is immutable.

Therefore no invalidation is needed for these descriptor identities.

REPL-generated module export structure may evolve if REPL supports `export` in later cells. Decide explicitly:

### Recommended

Treat REPL module export reflection as generation-aware. If export surface changes, replace the cached ExportTable descriptor and document that stable identity holds until semantic interface generation changes.

Do not silently mutate an immutable ExportTable.

---

# 22. Deterministic Reflection Ordering

Runtime `ModuleObject.exports` is currently a `HashMap`.

Do not expose its iteration order.

Normative ordering:

1. declaration/interface order when retained;
2. otherwise canonical lexical symbol order.

Recommended implementation:

- retain an ordered export-name vector from linked interface (`BTreeMap` currently provides canonical lexical order);
- build ExportTable from linked interface ordering;
- do not iterate the runtime `HashMap` to determine API order.

Apply the same rule to:

```text
children
requirements
resolved dependencies
metadata attributes
```

where the source model does not already define order.

---

# 23. Identity Objects

Do not expose:

```text
proj#2
synthetic#17
ObjRef(...)
slot 42
```

as stable language identity.

## 23.1 ModuleIdentity

Opaque value representing canonical module semantic identity.

Methods may include:

```phalcom
uri -> Uri
toString -> String
```

Equality is semantic identity equality.

## 23.2 PackageIdentity

Represents a durable package artifact identity.

For development packages without a version/published artifact, it may represent an ephemeral development package identity distinct from Project identity.

Do not force `name@version` to be globally sufficient if future registries/sources require stronger identity.

## 23.3 ProjectIdentity

Represents one live development Project context.

It does not become a Module owner identity in the public object model.

---

# 24. URI Model

## 24.1 Canonical language URI

Continue builtin identities such as:

```text
phalcom://universe/
phalcom://universe/reflection/selector
phalcom://std/json
```

Every Module should have a stable logical URI.

## 24.2 Project modules

Define a logical URI independent of checkout path. Example direction:

```text
phalcom://project/<project-runtime-id>/geometry/shapes/circle
```

or a package-oriented URI.

Do not freeze exact string format until PackageIdentity design is stable, but do guarantee:

- relocation does not change semantic identity;
- cwd does not change URI;
- dependency import aliases do not change URI;
- REPL session modules have synthetic but stable session URIs.

## 24.3 Filesystem location

If exposed:

```phalcom
module.sourceLocation -> Option<SourceLocation>
```

This is tooling/debug information, not identity.

---

# 25. PackageInfo for Development, Published, Builtin, and Standalone Packages

The Package API should remain the same.

## 25.1 Development root

Derived from validated Project + package facts.

```phalcom
__project__.rootPackage.packageInfo
```

exists.

## 25.2 Published/imported

Loaded from package artifact metadata.

No Project is reconstructed.

## 25.3 Builtin

Toolchain constructs PackageInfo directly.

Recommended minimum:

```text
universe:
    name        = "universe"
    namespace   = #universe
    version     = runtime/toolchain version
    description = "Primordial language and object universe for Phalcom."

std:
    name        = "std"
    namespace   = #std
    version     = runtime/toolchain version
    description = "Phalcom standard library."
```

Authors/license/repository may be populated from toolchain build metadata if authoritative; otherwise use absence rather than invented values.

## 25.4 Standalone Package

Make PackageInfo total for Package:

```phalcom
Package.packageInfo -> PackageInfo
```

A standalone Package with no manifest receives minimal info:

```text
name         = canonical standalone package name
namespace    = canonical namespace
version      = None
authors      = ()
description  = None
requirements = ()
defaultEntry = None
```

This requires the runtime track to define a canonical standalone package name/namespace rather than silently guessing from arbitrary filesystem spelling.

---

# 26. Default Entry Versus Development Entry

Keep these distinct.

```text
Project.developmentEntry
    local/project run selection

PackageInfo.defaultEntry
    durable package artifact entry
```

Do not automatically copy developmentEntry to defaultEntry for every package.

Publication configuration can explicitly choose/default it.

Use `defaultEntry`, not merely `entry`, to leave room for future named entry points:

```text
entries["cli"]
entries["migrate"]
...
```

without breaking the initial surface.

---

# 27. Source Metadata Versus Package Metadata

Never collapse these:

```text
module.metadata / __metadata__
    source-level @! metadata

package.packageInfo
    durable package/artifact metadata

project.manifest
    development configuration
```

Example:

```phalcom
@!documentation("Parsing helpers")
```

belongs to Module/Package source metadata.

It does not implicitly become `PackageInfo.description`.

A publisher may deliberately derive PackageInfo fields from explicit manifest data, but Phaldoc/source docs and package registry descriptions remain conceptually separate.

---

# 28. Phaldoc Interface Files

The `.ph` reflection files should become authoritative public documentation surfaces.

Use Phaldoc comments in the style below.

## 28.1 `reflection/module.ph`

```phalcom
/// A loaded Phalcom source module.
///
/// Every recognized module has one semantic identity. Import aliases,
/// re-exports, direct execution, and reflection do not duplicate the module.
class Module {
    /// The Package directly containing this Module.
    ///
    /// Returns None for a standalone module with no package context.
    package -> Option<Package>

    /// The root Package for this Module's namespace.
    rootPackage -> Option<Package>

    /// Durable metadata for the containing root package artifact.
    packageInfo -> Option<PackageInfo>

    /// Canonical immutable public export view.
    exports -> ExportTable

    /// Retained source @! metadata.
    metadata -> ModuleMetadata

    /// Canonical logical URI.
    uri -> Uri
}
```

Whether these are class reopenings, documentation shells, or native declarations must follow existing universe conventions; do not accidentally define duplicate runtime classes.

## 28.2 `reflection/package-object.ph`

Document package context explicitly:

```phalcom
/// This Package's own package context.
package -> Package

/// Parent Package, or None for a root Package.
parentPackage -> Option<Package>

/// Namespace root Package; root Packages return self.
rootPackage -> Package
```

## 28.3 `reflection/project.ph`

Emphasize:

```text
development-only
not importable
not Package
does not survive publication
```

This is important user documentation, not an implementation footnote.

---

# 29. Native Method Installation

Reflection getters can be implemented as native primitives on `Module`, `Package`, `Project`, and descriptor classes.

## 29.1 Recommended split

Rust/native:

- identity access;
- object relationship access;
- cache lookup;
- live export value resolution;
- project context access;
- URI/source location access.

Phalcom `.ph`:

- convenience derived methods;
- formatting where safe;
- Phaldoc.

## 29.2 Avoid field leakage

No getter should require user knowledge of:

```text
owning_package
root_package ObjRef
RuntimeExportRef
BindingRef
ModuleId
```

These are implementation details.

---

# 30. `__understands__` Semantics for Modules/Packages

Define precisely.

Recommended:

```phalcom
module.__understands__(#name)
```

returns whether `name` is in the public export/member surface, not whether a private global slot exists.

For Package, if facade child modules are exported, they count through exports.

Exposure-only children do not automatically count.

This aligns message-send semantics with reflection.

---

# 31. Namespace/Name Getters

Specify meanings to avoid distribution-name confusion.

## Module

```text
name
    local/canonical logical module name

namespace
    root Package namespace if package-backed
```

## Package

```text
name
    package namespace component or canonical logical package name

packageInfo.name
    distribution/artifact name

packageInfo.namespace
    root namespace
```

## Project

```text
name
    project/distribution development name

namespace
    root Package namespace
```

Do not overload a generic `name` to change meaning based on publication context.

---

# 32. Version Getter

Current dunder registry reserves `__version__`.

Do not put a meaningless version getter on every Module.

Recommended:

```text
PackageInfo.version
```

is canonical.

If `Package.__version__` is retained as convenience reflection, it delegates to:

```phalcom
package.packageInfo.version
```

Nested Packages return the root package artifact version.

Ordinary Module `__version__` should either be unsupported or clearly defined as containing package version. Prefer not to add it unless there is strong compatibility value.

---

# 33. Builtin Documentation and Metadata

Because builtin interfaces must now derive source metadata correctly, verify:

```phalcom
universe.metadata
std.metadata
universe.callable.metadata
std.json.metadata
```

reflect their `@!documentation(...)` source metadata.

PackageInfo descriptions for builtin roots may be populated from a dedicated builtin PackageInfo table, optionally using the same descriptive text as source documentation if the implementation explicitly chooses that mapping.

Do not implicitly map all `@!documentation` to PackageInfo description.

---

# 34. New Rust Modules Recommended

After the runtime track, create or consolidate:

```text
phalcom-modules/src/package_info.rs
    VM-independent PackageInfoDescriptor
    PackageAuthorDescriptor
    PackageRequirementDescriptor
    PackageArtifactIdentity

phalcom-modules/src/project_manifest_semantics.rs
    only if manifest.rs becomes too large; otherwise keep validated forms there

phalcom-core/src/modules/reflection.rs
    construction/access helpers

phalcom-core/src/modules/reflection_cache.rs
    canonical descriptor cache

phalcom-core/src/heap/reflection.rs
    native heap object structs if repository organization prefers heap types together
```

Do not create files solely to mirror every user-facing class; keep Rust modules cohesive.

---

# 35. Heap Representation Options

## Approach A — dedicated native heap variants (recommended for tables/identity)

Examples:

```rust
Object::ExportTable(...)
Object::Export(...)
Object::PackageInfo(...)
Object::Project(...)
```

Advantages:

- canonical immutable representation;
- no user field mutation;
- direct internal handles;
- cheaper access.

Disadvantages:

- expands `Object` match surfaces;
- requires GC/accessor/class mapping updates.

## Approach B — ordinary InstanceObject with sealed native constructors

Advantages:

- smaller heap enum;
- more behavior can live in `.ph`.

Disadvantages:

- harder canonical caching;
- must protect internal fields;
- live export bindings need indirect native handles anyway.

### Recommendation

Use dedicated native representation for:

```text
Project
PackageInfo
ExportTable
Export
ChildModuleTable
identity objects
```

`PackageAuthor` and small pure-value descriptors can be ordinary immutable records/instances if Phalcom's immutable record model is appropriate.

---

# 36. GC and Equality

## 36.1 Identity-based descriptors

For canonical objects:

```text
PackageInfo
ExportTable
ChildModuleTable
ProjectManifest runtime wrapper
```

ordinary object identity (`===`) should be stable because the runtime caches them.

## 36.2 Semantic identity objects

`ModuleIdentity`, `PackageIdentity`, `ProjectIdentity` should support semantic equality even if object interning strategy later changes.

If canonicalized, both `==` and `===` may often coincide, but do not make semantic correctness depend on pointer identity.

## 36.3 Value descriptors

PackageAuthor/PackageRequirement can use value equality if appropriate.

---

# 37. LSP/Tooling Benefits

The implementation should expose VM-independent descriptor types in `phalcom-modules` so the LSP and docs generator need not instantiate a VM to answer:

- package name/version/authors;
- module URI;
- exports;
- children/exposure;
- requirements;
- project manifest metadata.

Runtime objects should wrap the same semantic descriptors rather than maintain a second incompatible schema.

This is a major modularity objective.

---

# 38. Acceptance Tests — Public Interface

Create:

```text
phalcom-core/tests/module_reflection_contract.rs
phalcom-core/tests/package_info_contract.rs
phalcom-core/tests/project_reflection_contract.rs
phalcom-modules/tests/package_info_semantics.rs
```

Register according to the core test harness rules.

## 38.1 Navigation

```phalcom
__module__.package == Some(__package__)
__module__.rootPackage == __root__

__project__.rootPackage === __root__
```

where Project exists.

## 38.2 Root Package

```phalcom
root.package === root
root.rootPackage === root
root.parentPackage == None
root.isRoot == true
```

## 38.3 Nested Package

```phalcom
nested.package === nested
nested.parentPackage == Some(root)
nested.rootPackage === root
nested.packageInfo === root.packageInfo
```

## 38.4 Module

```phalcom
module.package == Some(nested)
module.rootPackage == Some(root)
module.packageInfo == Some(root.packageInfo)
```

## 38.5 Builtin PackageInfo

```phalcom
universe.packageInfo.namespace == #universe
std.packageInfo.namespace == #std
```

No Project object is fabricated for them.

## 38.6 Caching

```phalcom
module.exports === module.exports
package.children === package.children
package.packageInfo === package.packageInfo
__project__.manifest === __project__.manifest
__project__.dependencies === __project__.dependencies
```

## 38.7 Live export

Mutate an exported variable and assert ExportTable/Export descriptor identity remains stable while `.value` changes.

## 38.8 Exposure/export distinction

A package with:

```phalcom
expose .child
```

allows import traversal but does not answer `package.child` unless separately exported.

## 38.9 Dunder dispatch

Assert arbitrary-object reflection works:

```phalcom
someModule.__exports__
somePackage.__children__
```

and does not trigger a "field on self" diagnostic.

---

# 39. REPL Acceptance Tests for Public Reflection

Once runtime and REPL convergence is complete:

```text
ph> __module__
<Module ...repl...>

ph> __package__
Some(<Package app>)      # project REPL

ph> __root__
Some(<Package app>)

ph> __project__
<Project app>
```

Standalone:

```text
ph> __module__
<Module ...repl...>

ph> __package__
None

ph> __root__
None

ph> __project__
context diagnostic
```

After importing:

```text
ph> import std
ph> std.packageInfo
<PackageInfo std ...>

ph> std.__project__
MessageNotUnderstood / selector absent
```

The last assertion protects the rule that Project is not package provenance.

---

# 40. Project-Stripping Fixture

Create a fixture demonstrating the lifecycle.

Example:

```text
tests/fixtures/package_lifecycle/
├── producer/
│   ├── project.toml
│   └── src/
│       ├── package.ph
│       └── api.ph
└── consumer/
    ├── project.toml
    └── src/
        ├── package.ph
        └── main.ph
```

The test should derive a package artifact view from producer, then load it into consumer without exposing producer Project.

Assert:

```text
producer development:
    __project__ available
    rootPackage.packageInfo exists

consumer:
    imported package.packageInfo matches durable fields
    no importedPackage.project getter
```

Do not require an actual registry.

---

# 41. PackageInfo Construction Pipeline

Recommended layered design:

```text
Raw ProjectManifest
        ↓ parse
ValidatedProjectManifest
        ↓
Development Project Descriptor
        + root package/interface facts
        ↓
PackageInfoDescriptor
        ↓
runtime cached PackageInfo
```

Published artifact loading starts at:

```text
PackageInfoDescriptor
```

not `ProjectManifest`.

Builtin construction also starts at:

```text
PackageInfoDescriptor
```

This keeps PackageInfo independent of development syntax.

---

# 42. Project Dependency Construction

`Project.dependencies` should wrap resolved dependency entries using semantic package data.

Algorithm:

1. iterate resolved dependency aliases in deterministic order;
2. obtain resolved package artifact identity;
3. obtain PackageInfoDescriptor;
4. when runtime root Package is materialized, associate its canonical object;
5. construct/canonicalize `ResolvedProjectDependency`.

Do not materialize a Project object for each dependency merely to answer this API.

---

# 43. Error Semantics

Add structured errors for:

- PackageInfo construction missing required root namespace;
- invalid author metadata;
- non-publishable path dependency;
- reflection lookup of invalid export;
- unavailable `__project__` context;
- unsupported dunder on receiver;
- duplicate native/source builtin metadata;
- inconsistent package artifact identity.

Reflection `get` methods should return Option for normal absence, not throw for missing names:

```phalcom
module.exports.get(#missing) -> None
```

---

# 44. Performance Requirements

This reflection surface should be cheap enough for LSP/debugger/REPL inspection.

Requirements:

- PackageInfo cached;
- ExportTable cached;
- ChildModuleTable cached;
- ProjectManifest runtime wrapper cached;
- Project dependency tuple cached;
- names/ordering precomputed;
- export value lookup follows one descriptor -> binding reference without reconstructing maps;
- no parsing of `project.toml` or builtin `.ph` source on each getter call.

---

# 45. Documentation Updates

Update:

```text
docs/spec/next/modules-next.md
```

with:

1. Project development-only runtime context;
2. Package lifecycle and stripping;
3. contextual intrinsic table;
4. object navigation table;
5. PackageInfo vs ProjectManifest;
6. dependency-layer distinction;
7. URI vs filesystem source location;
8. exposure vs export reflection;
9. builtin roots as Packages;
10. standalone PackageInfo behavior.

Create/update Phaldoc in all reflection source files.

Remove stale documentation claiming:

```text
Module < Package < Project
```

Replace with:

```text
Package < Module inheritance direction in prose:
Package is a specialized Module.

Project owns a root Package but is not a Module.
```

Avoid mathematical `<` notation if it can be misread as ownership versus subtype.

---

# 46. Suggested Phaldoc — Context Intrinsics

Document outside the class navigation API:

```phalcom
/// The Module currently executing.
///
/// This is a contextual intrinsic, not an ordinary mutable global.
intrinsic __module__ -> Module

/// The Package context used for relative imports by the current source unit.
///
/// A standalone Module has no Package context and returns None.
intrinsic __package__ -> Option<Package>

/// The root Package of the current Module namespace.
///
/// Returns None for a standalone Module.
intrinsic __root__ -> Option<Package>

/// The active Project development environment.
///
/// This intrinsic is available only when source executes inside a real Project.
/// Published packages, builtin packages, and standalone execution do not receive
/// synthetic Projects.
intrinsic __project__ -> Project
```

---

# 47. Suggested Phaldoc — PackageInfo

```phalcom
/// Immutable descriptive information for a root Package artifact.
///
/// PackageInfo is durable package metadata. It is distinct from ProjectManifest:
/// a ProjectManifest describes how a package is being developed, whereas
/// PackageInfo describes the package artifact itself.
///
/// Development, imported, standalone, and builtin Packages expose the same
/// PackageInfo interface.
class PackageInfo {
    /// Distribution/artifact name. This can differ from `namespace`.
    name -> String

    /// Root Phalcom package namespace.
    namespace -> Symbol

    /// Package version where meaningful.
    version -> Option<Version>

    /// Structured authorship metadata.
    authors -> Tuple<PackageAuthor>

    description -> Option<String>
    license -> Option<String>
    homepage -> Option<Uri>
    repository -> Option<Uri>

    /// Durable unresolved dependency requirements.
    requirements -> Tuple<PackageRequirement>

    /// Optional durable default executable entry.
    defaultEntry -> Option<PackageEntry>

    identity -> PackageIdentity
}
```

---

# 48. Suggested Phaldoc — ExportTable

```phalcom
/// Immutable reflective view of a Module's public exports.
///
/// ExportTable describes export bindings rather than copying their current
/// values. Mutable exported bindings therefore remain live through a stable
/// Export descriptor.
class ExportTable {
    names -> Tuple<Symbol>
    size -> Int

    contains(name: Symbol) -> Bool
    descriptor(name: Symbol) -> Option<Export>
    get(name: Symbol) -> Option<Object>
}
```

---

# 49. Alternative Design Decisions and Rejections

## 49.1 `Package.__info__`

Rejected as primary API.

Reason: on nested Package it sounds like information about that nested package rather than the root package artifact.

Use:

```phalcom
package.packageInfo
```

## 49.2 `Project.__root__`

Rejected.

Reason: it makes Project appear to participate in the Module/Package namespace hierarchy.

Use:

```phalcom
project.rootPackage
```

## 49.3 `Package.__project__ -> Option<Project>`

Rejected as general provenance API.

Reason: Project exists only as development context and is stripped on publication.

Use contextual:

```phalcom
__project__
```

inside active Project source, and PackageInfo/PackageOrigin for package provenance.

## 49.4 `__manifest__` on Package

Rejected.

Manifest is development configuration.

Use:

```phalcom
__project__.manifest
package.packageInfo
```

## 49.5 Generic metadata dictionary

Rejected.

Do not merge:

```text
@! metadata
ProjectManifest
PackageInfo
native metadata
```

into one untyped Map.

## 49.6 `HashMap<String, Value>` exports

Rejected.

It cannot be both cached and live for mutable bindings.

Use ExportTable descriptors.

---

# 50. Implementation Order

This track should land in this order:

1. **validated manifest retention**
   - version/authors/new fields
   - structured semantic descriptors
2. **VM-independent PackageInfo/requirement/identity descriptors**
3. **native/core reflection classes**
4. **Module/Package ordinary navigation getters**
5. **Project + ProjectManifest runtime wrapper**
6. **PackageInfo construction for development roots**
7. **PackageInfo construction for builtins**
8. **Module.packageInfo and nested Package sharing**
9. **ExportTable/Export live reflection**
10. **ChildModuleTable**
11. **reflection cache**
12. **deterministic ordering**
13. **dunder registry update**
14. **dunder member-dispatch repair**
15. **builtin facade child exports**
16. **standalone PackageInfo**
17. **project-stripping lifecycle fixture**
18. **Phaldoc/spec documentation**
19. **LSP/tooling adaptation to VM-independent descriptors**

Do not implement the reflection cache before the descriptor semantics are correct, and do not implement user-facing Project getters before the runtime track has separated Project from Package.

---

# 51. Suggested Commit Boundaries

```text
feat(manifest): retain durable package metadata
feat(packages): add PackageInfo semantic descriptors
feat(reflection): add module/package navigation surface
feat(project): add development Project reflection
feat(reflection): add canonical ExportTable
feat(reflection): add ChildModuleTable
perf(reflection): cache immutable descriptor objects
fix(dunder): separate contextual intrinsics and reflection protocol
fix(reflection): route arbitrary receiver dunder getters through protocol
feat(builtins): export builtin child facades
docs(phaldoc): document module/package/project/package-info surface
```

---

# 52. Completion Checklist

- [ ] `Module.package` implemented and documented.
- [ ] `Module.rootPackage` implemented and documented.
- [ ] `Module.packageInfo` implemented and returns None only for genuinely package-less Module.
- [ ] `Package.package` returns self.
- [ ] `Package.parentPackage` distinguishes namespace ancestry from package context.
- [ ] `Package.rootPackage` returns canonical root.
- [ ] `Package.packageInfo` is total and canonical.
- [ ] `Package.children` represents exposure, not exports.
- [ ] `Project` is development-only and not Module/Package.
- [ ] `Project.manifest` returns canonical immutable ProjectManifest wrapper.
- [ ] `Project.rootPackage` returns the actual Package.
- [ ] `Project.dependencies` returns semantic resolved dependency descriptors.
- [ ] `PackageInfo` survives project stripping.
- [ ] `PackageInfo.name` and namespace are distinct.
- [ ] PackageInfo requirements preserve dependency aliases.
- [ ] path dependency publication cannot leak filesystem paths.
- [ ] builtin universe/std PackageInfo exists without fake ProjectManifest.
- [ ] `ExportTable` is stable/cached and values are live.
- [ ] `ChildModuleTable` is stable/cached.
- [ ] reflection ordering is deterministic.
- [ ] raw internal IDs/slots/ObjRefs are not exposed.
- [ ] `__root__` and `__uri__` are standardized.
- [ ] `__path__` is removed/deprecated as canonical identity.
- [ ] arbitrary-object dunder reflection no longer triggers self-field errors.
- [ ] context intrinsics are documented separately from object getters.
- [ ] source `@!` metadata, ProjectManifest, and PackageInfo remain distinct.
- [ ] builtins use ordinary exports for any facade child member access.
- [ ] Phaldoc source files describe all new public classes and semantic invariants.
- [ ] `modules-next.md` matches the implemented model.

---

# Appendix A — Targeted Reads by Phase

## Manifest phase

```bash
rg -n "ProjectSection|ValidatedProjectManifest|authors|version|entry|dependencies" \
  phalcom-modules/src/manifest.rs
```

Read only the matched validation functions.

## Module/Package reflection

```bash
rg -n "ModuleObject|owning_package|root_package|RuntimeExportRef|exports" \
  phalcom-core/src/heap phalcom-core/src/modules
```

## Native class registration

```bash
rg -n "module_class|package_class|project_class|CoreClasses" \
  phalcom-core/src/universe
```

## Dunder

```bash
rg -n "DunderPolicy|GuaranteedReflection|__path__|__parent__|__project__" \
  phalcom-modules phalcom-core
```

## Reflection member dispatch

```bash
rg -n "Expected a field|GetField|field.*self|send_dynamic|ModuleObject.*export" \
  phalcom-core/src
```

## PackageInfo builtins

```bash
rg -n "UNIVERSE_BINDINGS|BuiltinProject|source_text|documentation" \
  phalcom-native-meta phalcom-modules/src/builtin.rs phalcom-core/core
```

---

# Appendix B — Final Conceptual Surface

```text
CONTEXT

__module__  -> Module
__package__ -> Option<Package>
__root__    -> Option<Package>
__project__ -> Project       [development context only]


MODULE

Module
├── package        -> Option<Package>
├── rootPackage    -> Option<Package>
├── packageInfo    -> Option<PackageInfo>
├── exports        -> ExportTable
├── metadata       -> ModuleMetadata
├── dependencies   -> Tuple<ModuleDependency>
├── uri            -> Uri
└── identity       -> ModuleIdentity


PACKAGE

Package : Module
├── package        -> self
├── parentPackage  -> Option<Package>
├── rootPackage    -> Package
├── packageInfo    -> PackageInfo
├── children       -> ChildModuleTable
└── isRoot         -> Bool


PROJECT

Project
├── name
├── namespace
├── manifest       -> ProjectManifest
├── rootPackage    -> Package
├── dependencies   -> Tuple<ResolvedProjectDependency>
├── developmentEntry
└── identity


DURABLE PACKAGE METADATA

PackageInfo
├── name
├── namespace
├── version
├── authors
├── description
├── license
├── homepage
├── repository
├── requirements
├── defaultEntry
└── identity
```

The resulting model has one clear rule for each concern:

```text
execution context       -> contextual intrinsics
namespace navigation    -> Module/Package getters
development config      -> Project + ProjectManifest
durable artifact info   -> PackageInfo
module public API       -> ExportTable
import-path exposure    -> ChildModuleTable
semantic identity       -> opaque identity objects + URI
```

That separation is the primary modularity enhancement delivered by this specification.
