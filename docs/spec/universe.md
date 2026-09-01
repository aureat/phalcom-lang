# Phalcom Universe Package — Architecture and Semantic Contract

**Status:** Authoritative design specification  
**Scope:** Builtin package identity, source ownership, module/package semantics, prelude policy, semantic integration, runtime bootstrap, reflection, and tooling  
**Canonical package:** `universe`

---

## 1. Purpose

Phalcom has one canonical builtin package:

```phalcom
universe
```

`universe` contains the language/runtime substrate and everything distributed as part of the default Phalcom platform. It is not a compatibility namespace, a synthetic compiler module, or a presentation-only facade. It is a real package in the module graph, composed of real packages and modules with real source units, declarations, exports, source locations, runtime module objects, and reflection identities.

The governing invariant is:

> Every builtin declaration, module, package, runtime behavior, reflection product, semantic identity, and tooling location must refer to the same canonical Universe module graph.

Different compiler/runtime layers may project different representations of one entity, but they must not create competing builtin ownership models.

This document defines that contract. It intentionally does not enumerate the full API of the Universe library, define every native primitive ABI, or prescribe patch-by-patch implementation work.

---

## 2. One builtin package, multiple policies

There is no independent `core` package and no independent `std` package. Facilities historically thought of as language core, runtime library, or standard library all belong to `universe`.

Conceptually:

```text
Universe
├── object and class model
├── scalar types
├── callable model
├── ADTs and errors
├── collections
├── reflection
├── concurrency
├── text / regex
├── IO / filesystem / path
├── JSON
├── math / random / time
├── process / networking
├── testing
└── other facilities shipped by Phalcom
```

Universe membership is independent of other policies.

| Property | Meaning |
|---|---|
| **Universe** | Distributed as part of the canonical builtin package |
| **Primordial** | Needed to establish the runtime substrate itself |
| **Native** | Has VM/compiler/runtime-assisted implementation |
| **Prelude** | May be referred to without explicit import |
| **Eager** | Must be initialized during ordinary bootstrap |
| **Source-owned** | Has an authored canonical `.ph` declaration |
| **Runtime-support** | Exists for runtime behavior but is not necessarily a source-visible declaration |

For example:

```text
Object
    Universe: yes
    primordial/native/prelude/eager: yes

Int
    Universe: yes
    primordial/native/prelude/eager: yes

Option
    Universe: yes
    source-owned: yes
    native representation: yes
    prelude: yes
    primordial: not necessarily

json
    Universe: yes
    prelude: no
    primordial: no
    eager: no
```

The implementation must not infer one policy from another. In particular:

```text
Universe ≠ prelude
Universe ≠ primordial
Universe ≠ eager
Universe ≠ native
```

---

## 3. Canonical identity model

### 3.1 Project identity

Universe has a distinct project identity, conceptually:

```rust
enum ProjectIdentity {
    Universe,
    Resolved(ResolvedProjectId),
    Synthetic(SyntheticProjectId),
}
```

Universe is not encoded as project number zero, as a synthetic project, or as a filesystem project.

### 3.2 Module identity

A Universe module is identified by:

```text
ProjectIdentity::Universe + ModulePath
```

Examples:

```text
universe:<root>
universe:scalar.number
universe:collections.list
universe:errors.result
universe:reflection.typing
universe:json
```

The root package is the empty `ModulePath`.

Canonical helpers may be expressed as:

```rust
ModuleId::universe(path)
ModuleId::universe_root()
```

The exact API is implementation-specific; the semantics are not.

### 3.3 Declaration identity

Every source-visible builtin declaration is owned by the exact module in which it is authored:

```text
universe.scalar.number::Number
universe.scalar.number::Int
universe.scalar.number::Float

universe.option.option::Option
universe.errors.result::Result
universe.object.ordering::Ordering

universe.collections.list::List
universe.collections.map::Map
```

The compiler must not flatten these declarations into the Universe root and must not reconstruct them under a synthetic builtin module merely because they have native implementations.

The following are different declarations even though their short names match:

```text
universe.errors.result::Result
my_project.domain.result::Result
third_party.validation.result::Result
```

Short names are lookup/presentation syntax, not canonical identity.

### 3.4 Native keys

The runtime may maintain compact keys such as:

```text
UniverseKey::Object
UniverseKey::Int
UniverseKey::Option
UniverseKey::Result
UniverseKey::Ordering
```

These keys are runtime/native associations, not a second declaration database. Each key maps to the canonical source declaration or to an explicitly internal runtime-support identity:

```text
UniverseKey::Int
    -> universe.scalar.number::Int

UniverseKey::Option
    -> universe.option.option::Option

UniverseKey::Result
    -> universe.errors.result::Result
```

Native metadata may enrich canonical declarations with implementation information; it must not create competing semantic ownership.

---

## 4. Source graph and canonical documents

Universe is composed of authored packages/modules. A representative topology is:

```text
universe/
├── package.ph
├── object/
│   ├── package.ph
│   ├── object.ph
│   ├── behavior.ph
│   ├── class.ph
│   ├── metaclass.ph
│   └── ordering.ph
├── scalar/
│   ├── package.ph
│   ├── number.ph
│   ├── string.ph
│   ├── bool.ph
│   └── symbol.ph
├── callable/
├── option/
├── errors/
├── collections/
├── concurrency/
├── reflection/
├── io/
├── fs/
├── path/
├── text/
├── regex/
├── json/
├── math/
├── random/
├── time/
├── process/
├── net/
└── testing/
```

The precise set may evolve. The invariant is that each node has one canonical `ModuleId` and one canonical authored source unit.

Canonical virtual URIs use the module graph directly:

```text
phalcom://universe/
phalcom://universe/object/object
phalcom://universe/scalar/number
phalcom://universe/collections/list
phalcom://universe/errors/result
phalcom://universe/json
```

The module system owns the mapping:

```text
ModuleId <-> canonical Universe URI
```

LSP/editor code must consume that codec rather than reimplementing it.

A physical sysroot override may substitute physical source files for bundled source, but only the source location changes. Semantic identity remains the same.

There must be no aggregate synthetic source document equivalent to `phalcom://core` that concatenates all builtins into one pseudo-module. Such a document destroys exact module ownership, package structure, import provenance, source navigation, and reflection fidelity.

---

## 5. Package, import, export, and `expose` semantics

Universe participates in the ordinary Phalcom module/package model.

### 5.1 Imports

Examples:

```phalcom
import universe.json
import universe.collections.list
import universe.reflection.typing
```

Selective import:

```phalcom
from universe.collections.list import List
```

Alias:

```phalcom
import universe.json as json
```

The semantic target is always canonical. For example, selective `List` binds to:

```text
universe.collections.list::List
```

not to a synthetic root declaration and not to a local import-site declaration.

### 5.2 Relative imports

Universe modules belong to a real package hierarchy and therefore use the same relative import semantics as ordinary packages. `ProjectIdentity::Universe` must be sufficient context for canonical relative resolution.

A separate bootstrap dependency mechanism may cache or precompute dependencies, but it must not define different language semantics.

### 5.3 `expose`

Existence in the builtin source catalog does not imply external importability.

These are separate facts:

```text
module exists
module is a child
module is exposed
module is imported
module is prelude-visible
```

External imports must respect package exposure at every relevant boundary.

Compiler resolution and editor completion must consume the same exposure model; it is invalid for completion to hide a child that compilation accepts.

### 5.4 Exports

Universe module public surfaces derive from ordinary source/interface semantics. Native metadata may augment a module only where runtime-provided declarations require it.

The model is:

```text
source/interface semantics
+
explicit native augmentation
```

not:

```text
all builtin declarations are public
```

Root-level convenience re-exports do not change the defining declaration's identity.

---

## 6. Prelude semantics

The prelude is a visibility policy over canonical declarations. It is not an alternate declaration namespace.

Conceptually:

```text
PreludeBindings {
    "Object" -> universe.object.object::Object
    "Int" -> universe.scalar.number::Int
    "String" -> universe.scalar.string::String
    "Option" -> universe.option.option::Option
    "Result" -> universe.errors.result::Result
    ...
}
```

Thus:

```phalcom
const x: Int = 42
```

resolves `Int` directly to:

```text
universe.scalar.number::Int
```

### 6.1 Lookup precedence

At minimum:

```text
1. lexical/local declaration
2. explicit imported/re-exported binding
3. prelude binding
```

Therefore local shadowing remains valid:

```phalcom
class Int {
}

const x: Int = Int()
```

Here `Int` is the local declaration.

### 6.2 No broad builtin-name fallback

Prelude lookup must not be implemented as:

```text
if UniverseKey has this spelling, accept it
```

because native/runtime metadata contains names that are not necessarily prelude-visible or source-visible.

### 6.3 Runtime-support identities

The VM may require behavior classes corresponding to implementation concepts such as `Some` and `None`. That does not imply that bare `Some` or `None` are ordinary nominal source types.

The semantic model must distinguish:

```text
source-visible declaration
runtime-support behavior
exact variant type
```

For example, an exact case such as `ExactCase<Option<Int>, Some>` must not be conflated with a VM support class merely because the runtime uses such a class for dispatch or `.class` behavior.

---

## 7. Semantic Universe baseline

The semantic analyzer must know enough about Universe to analyze user code without rebuilding the complete platform on every edit.

The intended model is a reusable semantic baseline, whether represented by one structure or several immutable shared products.

Conceptually it contains:

```text
UniverseSemanticBaseline
├── parsed module/source catalog
├── source locations
├── module interfaces
├── linked imports/exports
├── canonical declaration catalog
├── hierarchy
├── generic declaration information
├── callable/field signatures
├── ADT metadata
├── native/source associations
├── source semantic index
└── prelude bindings
```

Required properties:

1. declarations use their real Universe module identities;
2. native/bootstrap data enriches those declarations rather than duplicating them;
3. user/project semantics overlay the baseline;
4. the complete Universe graph is cheaply discoverable;
5. deep body analysis is performed only when required.

This deliberately separates:

```text
catalog/interface/declaration knowledge
```

from:

```text
deep method/body semantic analysis
```

A compiler/LSP must be able to discover `universe.json` or navigate to `List<T>` without eagerly analyzing every Universe body.

### 7.1 Identity convergence

The compiler should obtain the same declaration no matter which canonical route reaches it:

```text
UniverseKey::Result
    ↓
universe.errors.result::Result

Prelude "Result"
    ↓
universe.errors.result::Result

Authored source declaration
    ↓
universe.errors.result::Result
```

These are different lookup mechanisms over one identity, not three declarations.

---

## 8. Typing, generics, and source validity

Universe declarations use the same formal type system as user declarations.

For example:

```phalcom
enum Option<T> { ... }
enum Result<T, E> { ... }
class List<T> { ... }
```

Their semantic forms retain the same information as user declarations:

- generic binder identity;
- generic parameter kinds;
- bounds and `where` clauses;
- variance when supported;
- generic method binders;
- callable signatures;
- substitution semantics;
- exact-case and ADT relationships.

Native implementation does not justify weakening public signatures to `Dynamic` when the formal type system can express the real contract.

A native body exemption is not a signature-soundness exemption. Source-owned Universe declarations must themselves be semantically valid.

For example, a method equivalent to:

```phalcom
unwrapOr<U>(_ default: U) -> U
```

is invalid if one branch can return unrelated `T` without a constraint relating `T` and `U`.

The Universe therefore acts as a consumer of Phalcom's own type system, not as a privileged escape hatch from it.

---

## 9. ADTs and variant identity

Builtin ADTs such as:

```text
Option<T>
Result<T, E>
Ordering
```

are canonical source-owned enums.

Their source declarations determine:

- nominal enum identity;
- generic binders;
- variant identities;
- constructor selectors;
- method signatures;
- source locations;
- documentation/navigation surfaces.

Physical runtime storage is separate.

A variant identity is conceptually:

```text
VariantId(
    owner = universe.errors.result::Result,
    selector = Ok(_)
)
```

It is distinct from:

```text
runtime variant ID
case discriminant
runtime behavior class
source spelling
```

Therefore compiler/runtime lowering must not recognize variants by strings such as `"Ok"`, `"Some"`, or `"Equal"`. A user enum may legally use the same names.

Semantic resolution must determine the `VariantId`, and executable lowering must carry that identity forward.

### 9.1 Representation independence

The VM may represent an ADT using:

```text
general heap cases
immediate unary wrappers
singletons
other specialized storage
```

without changing semantic identity.

For example, `Option<Int>` remains an application of canonical `universe.option.option::Option` regardless of how `Some(42)` is encoded in `Value`.

---

## 10. Runtime bootstrap and nominal identity

Some runtime objects must exist before authored Universe source can execute. Primordial allocation is therefore legitimate for the object/class/metaclass substrate and other runtime-critical identities.

The crucial rule is that primordial runtime objects implement canonical declarations; they do not replace them.

A representative bootstrap sequence is:

```text
1. allocate primordial runtime identities;
2. materialize canonical Universe package/module objects;
3. bind primordial/native objects into their owning modules;
4. install native implementations;
5. compile/execute required Universe source in canonical modules;
6. establish root exports and prelude policy;
7. publish completed runtime/reflection products.
```

Universe source executes in the runtime module object corresponding to its canonical `ModuleId`. For example, `scalar/number.ph` executes in `universe:scalar.number`, not in the root package and not in a hidden `core` module.

### 10.1 One nominal runtime root

Within one VM, one canonical nominal declaration corresponds to one canonical root runtime class identity.

For `Result`:

```text
semantic declaration
    universe.errors.result::Result

ADT registry root class
    same canonical Result ClassId

runtime typing association
    same canonical Result ClassId

module binding/export
    same canonical Result ClassId

reflection
    same nominal Result identity
```

The runtime must not allocate another enum root merely because generic ADT registration also needs a class.

Variant behavior classes may be distinct runtime objects, but they must attach to the same canonical enum/variant semantics.

---

## 11. Materialization, discovery, and initialization

Three concepts must remain distinct:

```text
Discoverable Universe graph
    ≠
Deeply analyzed Universe graph
    ≠
Runtime initialization graph
```

### Materialization

The VM may preallocate module/package objects for the complete Universe graph. This is useful for stable identities, parent/root links, circular dependencies, reflection, and import targets.

### Initialization

Top-level execution is separate. Only modules required by the runtime's initialization/reachability policy should execute.

The presence of a module in the Universe catalog does not imply it must run during VM startup.

This separation becomes essential as Universe includes platform facilities such as filesystem, networking, regex, JSON, or testing.

### Runtime linked reads

Prelude/import access should resolve to canonical module bindings, conceptually:

```text
LinkedReadSpec::Binding(
    universe.scalar.number::Int
)
```

rather than relying on:

```text
GetGlobal("Int")
if missing -> search hidden builtin globals
```

A hidden fallback creates a second namespace semantics and undermines canonical ownership.

---

## 12. Module context intrinsics

Universe modules participate in the same lexical context model as ordinary modules.

### `__module__`

The current module object.

### `__package__`

The package context defined by Phalcom's ordinary module/package semantics. Equivalent Universe and user package/module shapes must behave consistently.

### `__root__`

The Universe root package for Universe modules.

### `__project__`

Universe is toolchain-owned rather than a user's development project. Unless Phalcom explicitly defines a reflective project object for Universe, ordinary development-project semantics must not be fabricated.

Builtin materialization and ordinary program materialization must agree on these language-visible meanings.

---

## 13. Reflection and reification

Reflection must expose canonical Universe ownership.

For example, reflecting on `Int` must identify:

```text
declaration: universe.scalar.number::Int
module:      universe.scalar.number
source:      phalcom://universe/scalar/number
```

Reflecting on `Result<T,E>` preserves its enum identity, generic parameters, exact module, variants, and relevant native/runtime implementation metadata.

Reflection must not expose obsolete semantic ownership such as a `core` module or separate `builtin_std` project.

### Generic specialization

Reified types such as:

```text
List<Int>
Option<String>
Result<Int, Error>
```

are specializations of canonical Universe declarations.

Runtime type descriptors may be lightweight and selectively reified, but their constructor/declaration identity remains canonical. Reification must never cause specializations to appear owned by a synthetic builtin namespace.

---

## 14. Editor and LSP contract

Tooling consumes module and semantic products; it must not create its own builtin interpretation.

### Go-to-definition

For:

```phalcom
const x: Int = 42
```

definition navigation targets the authored `Int` declaration in:

```text
phalcom://universe/scalar/number
```

For:

```phalcom
from universe.collections.list import List
```

`List` navigates to the canonical `class List<T>` declaration in:

```text
phalcom://universe/collections/list
```

Module aliases have module targets:

```phalcom
import universe.json as json
```

`json` denotes `ModuleId(universe.json)`, not a fabricated local declaration.

### Hover

Hover must use compiler-owned semantic presentation. Prelude and explicit-import references to the same declaration must present equivalent type/declaration identity.

### Completion

Completion uses the same module/interface products as compilation:

```phalcom
import |
```

includes `universe` plus valid project/dependency roots.

```phalcom
import universe.|
```

shows externally available children.

```phalcom
import universe.collections.|
```

shows valid exposed children.

`core` and `std` are not builtin roots.

### Source semantic index

Import/export syntax should retain semantic targets at useful granularity. For:

```phalcom
import universe.collections.list
```

the path components correspond to:

```text
universe     -> Universe root
collections  -> universe.collections
list         -> universe.collections.list
```

For selective imports, aliases, re-exports, and `expose`, tooling should consume the canonical targets already established by module/semantic analysis.

---

## 15. Stable identity, persistence, and diagnostics

Universe has a durable builtin identity suitable for metadata and reflection. A stable reference may conceptually contain:

```text
namespace: universe
toolchain/library version
module path
declaration path
```

Resolved user projects require their own durable identity. Session-local graph IDs such as `proj#1` must not be serialized as stable project identity.

Historical persisted references to removed `core` or `std` ownership must be versioned, invalidated, or explicitly migrated. They must not be silently reinterpreted.

Diagnostics likewise use exact source ownership:

```text
Universe declaration diagnostic
    -> actual Universe module/source

User declaration diagnostic
    -> actual user module/source
```

No diagnostic path should fabricate a generic `core` owner.

---

## 16. Bootstrap tiers

Universe may be divided into conceptual bootstrap/dependency tiers while remaining one package.

### Tier 0 — primordial substrate

Object/class/metaclass foundations, scalar foundations, and runtime-critical callable infrastructure.

### Tier 1 — foundational language library

Option, Result, errors, collections, and common protocols.

### Tier 2 — platform library

Text/regex, filesystem/path, JSON, networking, process, time, and random facilities.

### Tier 3 — development/tooling library

Testing and tooling-oriented facilities.

The exact membership may evolve. The rule is:

> Lower bootstrap tiers must not acquire dependencies on higher tiers that require those higher facilities before the substrate needed to implement them exists.

Tiering controls dependency and initialization policy; it never creates separate builtin package identities.

---

## 17. Normative invariants

A conforming implementation preserves all of the following.

| ID | Invariant |
|---|---|
| **U-1** | Phalcom has exactly one builtin package identity: Universe. |
| **U-2** | Every source-visible builtin declaration is owned by its authored Universe module. |
| **U-3** | No production semantic/runtime/tooling path uses a hidden `core` module as declaration authority. |
| **U-4** | Former standard-library facilities have Universe ownership; there is no independent `std` identity. |
| **U-5** | Native/runtime keys associate implementation with canonical declarations instead of creating parallel declarations. |
| **U-6** | Prelude availability is policy over canonical declarations, not a second namespace. |
| **U-7** | Runtime-support behaviors do not automatically become source-visible nominal types. |
| **U-8** | Universe obeys ordinary import, relative-import, export, and `expose` semantics. |
| **U-9** | Compiler, semantic queries, reflection, and LSP consume the same canonical module/declaration graph. |
| **U-10** | One canonical nominal declaration corresponds to one canonical root runtime class identity within a VM. |
| **U-11** | Optimized physical value representation does not change nominal declaration or variant identity. |
| **U-12** | Definition/hover navigation points to real authored Universe source modules. |
| **U-13** | Full Universe discovery does not require eager execution or deep analysis of every module. |
| **U-14** | Persisted identity never depends on session-local project numbering. |
| **U-15** | Native implementation does not exempt source-owned Universe declarations from formal type-signature correctness. |

---

## 18. Prohibited shortcuts

The following are non-conforming unless the architecture is explicitly amended.

### Hidden-core preservation

Renaming a synthetic `core` module to `universe` while preserving declaration flattening is not sufficient.

### Root flattening

All builtin declarations must not be defined as if owned by the Universe root.

### All-Universe-as-prelude

Universe membership must not imply implicit visibility.

### All-Universe-as-eager

Universe membership must not imply startup execution.

### Name-based builtin identity

Checks such as:

```text
name == "Result"
name == "Option"
name == "Int"
```

must not replace canonical identity when semantic identity matters.

### Parallel LSP semantics

LSP/editor tooling must not independently reinterpret Universe paths, imports, exports, or declaration ownership.

### Native metadata as declaration authority

Native metadata must not become an alternate source of semantic declaration ownership.

### Physical path as semantic identity

Moving bundled sources or supplying a physical sysroot override must not change semantic identity.

### Silent compatibility namespaces

`core` or `std` must not survive as transparent aliases unless compatibility behavior is explicitly specified as part of the language.

---

## 19. Conformance examples

### Prelude type

```phalcom
const x: Int = 42
```

Required:

```text
Declaration: universe.scalar.number::Int
Source:      phalcom://universe/scalar/number
Runtime:     canonical Int ClassId
```

### Explicit import

```phalcom
from universe.collections.list import List
const xs: List<Int> = ...
```

Required:

```text
List -> universe.collections.list::List
Int  -> universe.scalar.number::Int
```

If `List` is also prelude-visible, both routes denote the same declaration.

### Local shadowing

```phalcom
class Int {
}

const x: Int = Int()
```

Required: `Int` denotes the local declaration.

### Module alias

```phalcom
import universe.json as json
```

Required: `json` has a module semantic target for `universe.json`.

### User homonym

```phalcom
enum Result {
    @variant Ok(_ value: Int)
    @variant Failure(_ message: String)
}
```

Required:

```text
user Result      != universe.errors.result::Result
user Result::Ok  != canonical Universe Result::Ok
```

Matching/construction use resolved `VariantId`, never spelling alone.

### Native ADT storage

```phalcom
const x: Option<Int> = Option::Some(42)
```

The VM may encode the value immediately.

Required regardless of representation:

```text
nominal declaration -> universe.option.option::Option
variant identity     -> canonical Option::Some VariantId
reflection           -> Option<Int>
.class/dispatch       -> behavior associated with canonical Option/Some semantics
```

---

## 20. Implementation ownership and evolution

The exact repository organization may change, but responsibility should remain approximately:

| Concern | Authoritative layer |
|---|---|
| Universe project/module identity | module system |
| builtin source catalog | module/source provider |
| import/export/`expose` legality | module resolver/linker |
| declaration identity | module + semantic identity |
| prelude mapping | semantic/bootstrap policy |
| nominal/generic type meaning | semantic analyzer |
| enum/variant semantic identity | semantic analyzer |
| executable ADT identity lowering | semantic lowering |
| runtime enum/class association | VM/runtime ADT registry |
| native implementation association | native metadata/runtime |
| source definitions/occurrences | semantic source index |
| canonical Universe URI codec | module system |
| hover/definition presentation | semantic/editor products |
| LSP transport/adaptation | LSP |
| durable serialized identity | metadata/stable project identity |

A lower-level consumer must not become the source of truth for a higher-level semantic fact.

When adding a new Universe facility:

1. choose its canonical package/module location;
2. give it canonical Universe module identity;
3. provide source declarations where language-visible;
4. define ordinary export/`expose` behavior;
5. specify prelude, native, primordial, and eager policies independently;
6. map native keys to canonical source identities where required;
7. preserve authored source navigation and reflection;
8. add initialization dependencies only where necessary;
9. avoid introducing a new builtin namespace category.

Moving a declaration between Universe modules is identity-affecting unless an explicit migration/aliasing scheme says otherwise.

---

## 21. Summary

Phalcom's Universe is not merely "the standard library under one name." It is the canonical builtin module graph shared by every layer of the implementation.

```text
one builtin package
        |
        v
real packages and modules
        |
        v
real source-owned declarations
        |
        +-------------------+
        |                   |
        v                   v
semantic identities     native/runtime associations
        |                   |
        +---------+---------+
                  |
                  v
         runtime / reflection / tooling
```

The central rule is:

> Canonical semantic identity is singular; implementation representations may be plural.

A builtin declaration may simultaneously have authored source, a native implementation, a primordial runtime class, an optimized value representation, a reified type descriptor, a prelude binding, reflection metadata, and LSP presentation. These are projections of one canonical declaration and one canonical Universe module graph.

The implementation is correct only when those projections agree.
