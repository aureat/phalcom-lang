# Modules, Packages, Projects, Cycles, and Initialization

## 1. Modules are semantic graph nodes

A module is not merely a file path or textual include. Semantic analysis needs canonical module identity, source/provider resolution, declared/imported surface, dependency edges, and—where runtime behavior matters—initialization state.

Separate:

```text
module identity          canonical semantic namespace
source provider          where its source/metadata comes from
import binding           local name introduced by import
analysis dependency      fact/surface relation between modules
runtime initialization   execution state/order of module body
package/project identity distribution/version/dependency context
```

Conflating these makes static analysis execute code accidentally or makes runtime cycles look like harmless graph cycles.

## 2. Current Phalcom anchor

**CURRENT:** semantic `ModuleId` is currently URI-backed; class identities are module-qualified. The LSP semantic engine maintains a `ModuleGraph`, import edges, dependent closures, provider repair on file add/remove, and module-aware class resolution. This is appropriate for source analysis but should evolve with package/project semantics rather than assuming URI string is the permanent package identity model.

The current runtime module loader is execution-oriented, while the draft optional typing architecture explicitly recommends a separate static module analysis loader. Preserve the distinction: checking a module graph must not require executing top-level Phalcom code.

## 3. Resolution graph versus initialization graph

Static name resolution may tolerate/import cycles that runtime initialization cannot execute safely without defined partial states.

Example:

```text
A imports B.x
B imports A.y
```

Questions differ:

- Can names `B.x` and `A.y` be declared/indexed without executing either module?
- Are their definitions available before top-level initialization?
- If `x` reads `A.y` during initialization, what value/error occurs?
- Does Phalcom permit cycles, reject them, or expose partially initialized modules?

A semantic module reference should therefore model declaration/interface availability separately from runtime initialized-value availability.

## 4. Module state machine

If runtime initialization is relevant, specify states rather than a boolean:

```text
Unresolved
Resolved
Indexed            declarations/surface known
Checking/Analyzing
ReadyForExecution
Initializing
Initialized
Failed(error)
```

This is a conceptual model; exact runtime states belong to module semantics. The analyzer can use `Indexed` interfaces to resolve cycles without pretending runtime field values are initialized.

## 5. Imports and local bindings

An import creates semantic edges and usually local namespace bindings. Resolve in stages:

```text
import syntax
 -> canonical provider/module resolution
 -> ModuleId
 -> imported surface/interface
 -> local import binding(s)
 -> occurrences/uses
```

A missing package/provider should produce `Blocked/MissingDependency`, distinct from “module exists but has no member `Foo`.” This distinction makes diagnostics and later incremental repair precise.

## 6. Public surfaces and body dependencies

A module should expose a compact semantic surface used by dependents: exported classes/functions/members/types/constants as Phalcom defines them. Body-only changes that do not alter the surface should ideally avoid rebuilding dependent scope/declaration indexes.

However, interprocedural inference may still create body-level dependencies across modules. Therefore maintain at least two kinds of edges:

```text
surface dependency: dependent must re-resolve/recheck interface on change
body/summary dependency: dependent summary/fact reads callee summary
```

This distinction supports precise invalidation.

## 7. SCCs in module graphs

Compute strongly connected components for cycles. For declaration indexing, an SCC can often be staged:

```text
for all modules in SCC:
    parse/recover
    extract declaration surfaces
then:
    resolve cross-module references using all SCC surfaces
then:
    analyze bodies/summaries to fixed point where needed
```

This avoids order-dependent resolution. If normative module semantics prohibit certain cycles, the SCC still helps diagnose the cycle as a unit.

## 8. Packages and projects

A future package registry means module identity cannot be naively “whatever filesystem path editor opened.” Canonical identity may include package coordinate/version/project root plus logical module path. Aliases/symlinks/workspaces should not create duplicate semantic classes for the same logical module.

A future representation might distinguish:

```rust
struct PackageId { /* registry/workspace identity */ }
struct ModuleKey { package: PackageId, path: ModulePath }
struct ModuleId(/* interned canonical key */);
```

This is a **RECOMMENDATION**, not current design. Package semantics require their own governing specification.

## 9. Core/native modules

Core declarations may be source-authored, native, generated, or hybrid. The current semantic engine reserves `phalcom://core`. Future analysis should expose a coherent semantic surface for core/native APIs independent of whether implementation resides in Rust.

Native contracts should include semantic behavior needed by analysis; source identities for native declarations can be synthetic but stable. Do not force LSP/checker to instantiate a VM just to discover native members.

## 10. Initialization facts

Static facts about module globals must account for initialization order. A global may have:

```text
DeclaredButUninitialized
Initializing(value approximation?)
DefinitelyInitialized(fact)
MaybeUninitialized due to cycle/path
```

Do not publish a post-initialization value shape as if it were valid while another module can observe the variable earlier during a cycle.

## 11. Tests

- two modules with same class name remain distinct;
- import alias resolves to canonical `ModuleId`;
- missing provider is blocked dependency, not missing member;
- file add/remove repairs import/provider edges;
- body-only cross-module callee change invalidates summary dependents but not unrelated surfaces;
- surface change invalidates import dependents;
- module SCC declaration indexing is order independent;
- runtime-initialization cycle behavior matches normative runtime tests;
- package alias/symlink resolution does not duplicate identities once package semantics exist;
- native/core surface works without VM execution.

## 12. Review questions

1. Is this a module namespace question, provider-resolution question, or runtime initialization question?
2. What is canonical module identity?
3. Does this dependency read the public surface or body summary?
4. Can a cycle be indexed before bodies are analyzed?
5. What state is visible during runtime initialization?
6. Does static checking accidentally execute top-level code?
7. How does removing/adding a provider invalidate dependents?
8. Are native/core modules represented by explicit contracts/surfaces?
