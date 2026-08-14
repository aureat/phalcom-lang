# Modules, Projects, Incrementality, and Semantic Generations

## Module graph is semantic infrastructure

Module support is not merely import syntax. Semantic analysis needs a graph whose nodes and
edges have well-defined identity and meaning.

Represent unresolved edges too. A missing module is still a dependency that may become
resolvable after an edit/package change.

## Separate edge meanings when needed

One `imports` edge may eventually be insufficient. Languages commonly need to distinguish:

- namespace/name-resolution dependency;
- type/signature dependency;
- runtime initialization dependency;
- compile/build dependency;
- optional/test-only dependency;
- native/FFI dependency.

Do not over-model before Phalcom's module spec is settled, but avoid APIs that assume every
edge always means the same thing.

## Dependency closure

When module `A` changes, recomputation candidates include:

- `A` itself;
- modules whose resolved names/surfaces depend on `A`;
- callables whose summaries depend on changed callables in `A`;
- consumers of field/parameter/type facts changed by `A`;
- future modules whose proof/type obligations import declarations from `A`.

Compute affected frontiers from semantic dependencies, not timestamps alone.

## Whole-file replacement is acceptable

The current front end does not require sub-file incremental parsing for semantic correctness.
A robust architecture can replace one file contribution wholesale, then recompute only
semantic dependents.

Avoid premature complexity that makes correctness harder than reparsing a small file.

## Immutable publication

The current model of:

```text
mutable worker SemanticEngine
  -> rebuild affected state
  -> clone/publish immutable SemanticSnapshot
  -> LSP queries read snapshot lock-free/cheaply
```

is a strong architecture for editor consistency.

Maintain these properties:

- one published generation is internally coherent;
- queries never observe half-updated module/class/summary maps;
- mutation is isolated to worker state;
- stale request results can be identified by generation/stamp if needed.

## Generation and file revision

Keep separate concepts:

- file revision: version of one source document;
- semantic generation: coherent project-wide published state.

Several files may be updated in one semantic batch and should produce one generation.

## Staleness

Never cache a derived fact without knowing what invalidates it.

A cache entry should conceptually depend on stable keys such as:

```text
(module revision, semantic generation, declaration IDs, dependency summary versions)
```

Rather than cloning foreign module data, store references/IDs or recomputable query results.

## Cycles

Module cycles and callable cycles are different problems.

Module cycles may affect:

- name visibility;
- initialization order;
- partial module state;
- type declaration resolution;
- package loading.

Callable cycles affect inference/proof fixed points.

Do not treat both as "just graph cycles" without their semantic rules.

## Package/project future

Reserve conceptual room for:

```text
Workspace
  Package instance (name + version/source identity)
    Module(s)
      declarations
```

Questions that must be specified before package-aware semantic IDs are permanent:

- Are two copies of the same package version semantically identical?
- Does lockfile resolution affect module identity?
- Can generated/native modules share a logical namespace?
- What does visibility mean across package boundaries?
- How is core/std versioned?

## Core source and native floor

Phalcom's semantic engine already analyzes bundled core source. This is valuable because it
lets tooling consume the same visible declarations users do.

Native primitives need one of:

- source declarations with trusted native implementation metadata;
- generated semantic stubs;
- explicit native semantic signatures.

Do not hardcode editor-only behavior that has no visible/spec/runtime counterpart.

## Invalidation diagnostics

For difficult incremental bugs, expose test/debug traces showing:

- modules recomputed;
- callables recomputed;
- changed summaries;
- why a dependent was enqueued.

This is preferable to fixing stale-state bugs by invalidating the whole world forever.
