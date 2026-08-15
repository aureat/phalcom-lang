# Modules, Imports, and Initialization

Modules combine namespace identity, dependency resolution, execution, caching, visibility, and package/project structure. Treating import as textual inclusion collapses distinctions that later become correctness bugs.

## 1. Separate the concepts

```text
module identity        Which module is this?
namespace              Which bindings/classes does it own?
dependency edge        Which other module does it reference?
loading                How source/metadata is located and prepared?
initialization         When top-level executable effects run?
```

A package adds distribution/version/build concerns but does not erase these distinctions.

## 2. Module identity

A module ID should be canonical within a project/runtime universe. File path can participate, but path spelling is not necessarily identity because of symlinks, normalization, package roots, virtual/core modules, registry packages, and multiple package versions.

```text
ModuleId = canonical project/package module identity
```

Current URI-based LSP identities are useful tooling identities; future runtime/package semantics may require stronger qualification.

## 3. Namespace ownership

Top-level declaration:

```phalcom
class Point { ... }
```

creates/binds `Point` in module `M`. Class identity is therefore module-qualified:

```text
ClassId(M, Point)
```

Same spelling in module `N` is another class.

## 4. Import binding

An import can create a local alias:

```text
BindingId localAlias -> ModuleId/entity target
```

This preserves distinction between alias rename and target identity.

## 5. Loading state machine

A robust semantic model:

```text
Unloaded
  -> Loading(partial state)
  -> Initialized(namespace)
  -> Failed(error)
```

Cycles make `Loading` observable unless cycles are rejected before execution.

Settle:

- Are cycles legal?
- Are declarations indexed before initializers execute?
- Can a cycle read an uninitialized binding?
- What error results?
- Is failed initialization cached?
- Can module be reloaded explicitly?

## 6. Initialization-once

If import executes module at most once per runtime module identity:

```text
initialize(M):
    Initialized -> reuse namespace
    Loading     -> cycle rule
    Unloaded    -> mark Loading; execute top-level; mark Initialized
```

Textual inclusion would repeat effects and duplicate identities.

## 7. Declaration availability versus execution

Recursive classes/protocols/types may require phases:

```text
A. create declaration identities/shells
B. resolve metadata/type references
C. execute ordinary top-level initialization
```

Do not assume this exact design is ratified; the semantic lesson is that declaration identity and runtime initializer execution are independent.

## 8. Top-level evaluation order

If top-level executable initializers run in source order, say so. If declarations are hoisted/indexed before execution, specify which operations are legal before associated initializer has run.

A module cycle is the stress test for this distinction.

## 9. Compile-time versus runtime dependency

Distinguish:

```text
compile-time dependency
runtime initialization dependency
namespace lookup dependency
package build dependency
type-metadata dependency
```

They often overlap but need not be identical. Invalidation algorithms should use the dependency kind they actually require.

## 10. Exports and re-exports

Export controls accessibility, not identity. Re-exporting a class should ordinarily expose the same underlying identity, unless language explicitly creates wrapper/alias objects.

Decide whether imported/re-exported bindings are live views or copied binding values. Dynamic mutable module bindings make this observable.

## 11. Relative/absolute resolution

Resolve relative imports from semantic package/module roots, not process current working directory unless cwd is explicitly language semantics.

Canonical resolution affects module singleton behavior, class identity, cache keys, LSP navigation, and reproducible packages.

## 12. Package versions

With a registry, module identity may need package-instance context:

```text
PackageInstanceId(name, resolvedVersion/source, dependency instance)
ModuleId(packageInstance, modulePath)
```

Two package versions loaded simultaneously may need distinct classes/modules even with identical textual names.

## 13. Native/mixed modules

Rust + Phalcom package requires defined ordering among:

- native registration;
- Phalcom declaration shells;
- type/protocol metadata;
- native method attachment;
- top-level code.

Native symbols must attach to same logical module/class identities used by source/reflection.

## 14. Failure semantics

If module initialization throws:

```text
Loading -> Failed(error)
```

decide subsequent import behavior:

- rethrow cached error;
- retry initialization;
- expose failed namespace;
- prohibit future use.

Do not let loader implementation choose accidentally.

## 15. Reload / REPL

Reload is separate feature. Define:

- whether module identity is preserved;
- old class objects/instances;
- method replacement;
- module globals;
- dependent modules;
- reflection identity;
- cache/proof invalidation.

Development-time file change is not automatically runtime reload semantics.

## 16. Static analysis

Analyzer should preserve unresolved imports as uncertainty, not fabricate local identities. Cross-module summaries should be tied to module/package revision/dependency facts.

## 17. Conformance scenarios

Test:

- same-named classes in different modules remain distinct;
- repeated import initializes once if policy says so;
- re-export preserves target identity;
- cycle behavior matches chosen rule;
- canonical aliases do not duplicate modules;
- failed initialization follow-up matches policy;
- native/source portions share one identity.

## 18. Competency checks

1. Why is file-path spelling insufficient module identity?
2. How can declaration be visible before initializer has run?
3. Which semantic guarantees are broken by textual inclusion?
4. Why are compilation and initialization graphs different concepts?
5. What identity component becomes important with multiple package versions?
