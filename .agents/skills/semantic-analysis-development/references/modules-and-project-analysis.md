# Modules, Packages, and Project Semantic Analysis

Phalcom's module/package implementation is evolving. Semantic analysis must avoid
baking today's source-file assumptions so deeply into identities and resolution that
future package semantics require a rewrite.

This reference defines the questions and architecture an implementer should preserve.

## 1. Separate four concepts

Do not assume these are permanently identical:

```text
source file
module
package
project/workspace
```

A first implementation may map one file to one module. The semantic model should still
make the distinction conceptually explicit.

## 2. Module identity

`ModuleId` should represent semantic module identity, not merely display path spelling.

Current LSP code uses canonicalized document URI identity. Future package work may add
logical names or package-qualified identities.

Before changing identity rules answer:

- Are symlinked paths the same module?
- Can one logical module have generated/multiple source files?
- Can two package versions expose the same logical module name?
- Does an in-memory editor document have a stable identity before it exists on disk?
- Is core a synthetic module namespace?

Migration from file URI to richer IDs must preserve snapshot/invalidation semantics.

## 3. Three import relations

An import can establish several different edges:

1. **namespace edge** — names become resolvable;
2. **semantic/compile dependency** — exported surface/facts affect analysis;
3. **runtime initialization edge** — module execution/initialization depends on another.

Do not use one graph blindly for all three if their cycle/ordering semantics diverge.

## 4. Import resolution phases

A robust pipeline is conceptually:

```text
parse import syntax
→ normalize import request
→ locate package/module candidate
→ resolve semantic ModuleId
→ expose imported namespace/surface
→ resolve member/name references
→ record dependency/invalidation edges
```

Recovery should retain unresolved import syntax and a reason rather than dropping it.

## 5. Exports and module surfaces

A module semantic surface should distinguish:

- declarations owned by the module;
- exported/public names;
- private/internal names;
- re-exports;
- imports/aliases;
- classes/protocols/types;
- top-level values/callables when supported.

An LSP completion query inside the module may see more than an external importing
module.

## 6. Name resolution across modules

Resolution order must be normative.

Possible classes of names include:

- lexical local;
- parameter;
- class/member-local semantics;
- module declaration;
- explicit import alias;
- imported/exported name;
- core/prelude/global;
- implicit-self dispatch.

Do not resolve a cross-module class by searching all modules for the same bare name.
Ambiguity should remain ambiguity until language rules select one declaration.

## 7. Class identity

Class identity must remain module-qualified at minimum.

Future package identity may make it effectively:

```text
PackageInstanceId + ModuleId + class name
```

if multiple dependency versions can coexist.

Do not use textual fully qualified names as a substitute for stable semantic identity
inside the engine unless canonicalization rules make them unambiguous.

## 8. Cycles

Different cycles have different consequences:

- namespace cycles may be legal with declaration shells;
- runtime initialization cycles may expose partially initialized state;
- type/protocol cycles may require fixed points;
- callable-summary cycles require SCC/fixed-point solving.

Do not reject or accept all cycles at the generic graph layer.

## 9. Initialization semantics

When modules execute top-level code, semantic analysis needs to know what is guaranteed
before initialization completes.

Questions:

- Are declarations hoisted/indexed before top-level execution?
- Can imported modules observe partially initialized bindings?
- Is initialization once-only?
- What happens on initialization failure?
- Are cyclic imports detected or represented as partial state?

The analyzer must not assume a top-level value is available earlier than runtime
semantics guarantee.

## 10. Package identity and versions

When a registry/package manager arrives, project semantics may need:

```text
PackageId          logical package name
PackageVersion     selected version
PackageInstanceId  concrete resolved dependency instance
ModuleId           module within that instance
```

Lockfile resolution belongs outside ordinary expression semantic analysis, but its
result is an input to semantic identity.

The same package name at two resolved versions must not share class/type identities by
accident.

## 11. Project graph

The semantic project graph may eventually include:

```text
package dependency edges
module import edges
re-export edges
callable dependency edges
inheritance edges
type/protocol dependency edges
```

Do not collapse all into one untyped adjacency map. Edge kind matters for invalidation.

## 12. Incremental invalidation

An edit to a module should invalidate dependents according to changed semantic surface,
not always the entire workspace.

Possible future optimization:

- private body edit with unchanged callable summary → local rebuild only;
- exported selector/signature change → importing modules;
- class inheritance change → subclasses and dispatch dependents;
- type metadata change → checker/type dependents;
- runtime-only implementation edit → perhaps no LSP type surface rebuild beyond
  affected summaries.

Correctness comes before granularity. Start conservative, then narrow with tests.

## 13. Core/prelude semantics

Bundled core source is currently a synthetic semantic module. Preserve the principle
that core/native semantic facts use the same identity/query machinery as user code
where possible.

Avoid magic bare-name fallbacks that cannot explain whether a symbol came from:

- current module;
- explicit import;
- prelude/core;
- runtime global.

## 14. Generated/native modules

Future packages may expose modules backed partly or entirely by Rust/native metadata.
Treat them as semantic surfaces with stable identities and explicit contracts, not as
special cases hardcoded into completion.

They should specify:

- exported declarations;
- selectors;
- parameter/result type metadata when available;
- runtime shape contracts where sound;
- effects;
- documentation/source/native location;
- platform availability.

## 15. Platform-conditional modules

OS/process/filesystem/network packages may expose platform-specific APIs.

Project semantic analysis needs an explicit target environment/configuration. Do not
silently union every platform's members and present impossible completions as if they
were available.

A future semantic configuration identity may include:

```text
target OS
architecture
feature flags
language version
package features
```

Changing configuration invalidates configuration-dependent facts.

## 16. Module diagnostics

Retain enough provenance for:

- unresolved module;
- ambiguous import;
- cyclic initialization issue;
- duplicate export;
- inaccessible/private symbol;
- package version conflict;
- platform-unavailable API.

Diagnostics should identify both the import site and candidate/export sites when
useful.

## 17. LSP project behavior

Queries must operate on a coherent semantic snapshot of the project graph.

For a file edit:

- publish the edited module and all required semantic dependent recomputation together;
- avoid answers that combine new source with stale imported surfaces;
- cancellation may discard work, but a published generation must be coherent.

Workspace symbol/search can tolerate broader indexing approximations only where it does
not claim identity/dispatch facts that are false.

## 18. Testing modules

Minimum fixture dimensions:

- direct import;
- alias import;
- same class name in two modules;
- re-export;
- private/inaccessible export;
- diamond dependency;
- cycle;
- unresolved import then file creation;
- file removal;
- module rename/path change;
- symlink/canonical-path case if supported;
- core/prelude shadowing;
- two package versions if package resolver supports it;
- platform-conditional module;
- deterministic project rebuild.

Incremental tests should assert the affected frontier, not only final hover text.

## 19. Migration discipline

When module/package semantics change:

1. ratify identity and import rules;
2. change module graph/identity types;
3. migrate source surfaces and scopes;
4. migrate occurrence/reference identity;
5. migrate dispatch/type/proof dependencies;
6. update invalidation;
7. update LSP queries;
8. update compiler/runtime loader semantics as necessary;
9. add cross-layer conformance tests.

Do not patch completion/definition paths first and leave the semantic database with old
identity assumptions.

## 20. Review questions

1. Which concept is this: file, module, package, or project?
2. Is the identity semantic or merely a path string?
3. What namespace edge is created?
4. What runtime initialization edge is created?
5. What invalidation edge is created?
6. Are cycles modeled at the correct layer?
7. Can duplicate bare names coexist safely?
8. Does core/native code use the same identity system?
9. What happens under unresolved/incomplete imports?
10. Does the published project snapshot remain coherent after edits/removals?
