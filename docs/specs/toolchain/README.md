# Telos Toolchain

This directory defines the initial architecture of the Telos development toolchain.

The toolchain is split deliberately between two principal user-facing tools:

- **`archon`** is the project, package, dependency, build, cache, registry, test, benchmark, and toolchain orchestrator.
- **`telos`** is the compiler/runtime driver. It operates on an already-resolved compilation environment and owns Telos language realization: parsing, linking, semantic analysis, compiler-artifact import/export, executable lowering, and runtime execution.

The governing boundary is:

> **Archon owns reproducible project realization. Telos owns language realization.**

This split is architectural, not cosmetic. `archon` MUST NOT duplicate compiler semantics, and `telos` MUST NOT grow into a package manager, registry client, version solver, or cache-policy engine.

---

## 1. The toolchain at a glance

A normal user workflow is package-oriented:

```text
developer
   │
   ▼
archon
   │
   ├── reads Archon.toml
   ├── reads / updates Archon.lock
   ├── resolves the active toolchain
   ├── resolves Universe
   ├── resolves path and registry dependencies
   ├── chooses package targets
   ├── consults project and global caches
   ├── constructs a resolved compilation request
   └── invokes Telos
           │
           ▼
         telos
           │
           ├── imports compatible .telosc dependencies
           ├── constructs / updates compiler semantic authority
           ├── parses and links source that must be compiled
           ├── performs semantic analysis
           ├── lowers requested executable targets
           ├── freezes package compiler artifacts
           └── executes already-resolved runnable products
```

Most developers should interact primarily with `archon`.

Typical commands are expected to look like:

```bash
archon check
archon build
archon run
archon test
archon bench
```

`telos` remains directly useful for compiler development, artifact inspection, explicit low-level compilation, runtime execution, and tooling integration.

---

## 2. Core vocabulary

The following terms are used consistently throughout the toolchain specifications.

### Package

A **package** is the persistent Telos project identity and the principal semantic compilation/artifact boundary.

A package owns:

- one package identity;
- one module world;
- one production semantic generation;
- zero or one library target;
- zero or more binary targets;
- zero or more test targets;
- zero or more example targets;
- zero or more benchmark targets;
- one canonical package compiler artifact when frozen.

A package is declared by an `Archon.toml`.

### Target

A **target** is a named product or development execution mode derived from a package.

Initial target kinds are:

```text
lib      0..1
bin      0..N
test     0..N
example  0..N
bench    0..N
```

Targets are not independent compiler projects and do not establish independent package identity.

### Library target

The package's externally consumable library surface. A package may have at most one.

### Binary target

A runnable same-package executable rooted at a canonical Telos module entry.

### Test target

An Archon-managed integration-test target. Initial Telos testing is integration-only: no unit-test declarations in production modules, no documentation tests, and no harness-free test mode.

### Example target

A runnable development target intended to demonstrate or exercise the package's public library surface.

### Benchmark target

An Archon-managed development target whose workload is compiled by Telos and measured/orchestrated by Archon.

### Package release

An immutable published package version in a registry. A release is fundamentally tied to an immutable source snapshot and registry authority, not permanently to one compiler artifact encoding.

### Compiler artifact

A `.telosc` file: an immutable, validated, portable compiler image representing completed compiler authority for one persistent package generation under a compatible Telos artifact/semantic ABI.

### Build product

Any result of a build operation. A package `.telosc`, executable product, benchmark executable, test harness product, or documentation output may all be build products.

---

## 3. Package and target architecture

The initial package model is:

```text
Package
├── lib       0..1
├── bin       0..N
├── test      0..N
├── example   0..N
└── bench     0..N
```

The package—not the individual target—is the semantic compiler ownership boundary.

For a mixed package:

```text
atlas
├── lib: atlas
├── bin: atlas          -> atlas.cli
├── bin: atlas-server   -> atlas.server
├── test: protocol      -> atlas_tests.protocol
├── example: basic      -> atlas_examples.basic
└── bench: throughput   -> atlas_benches.throughput
```

the normal production semantic world remains one package world:

```text
atlas package source
        │
        ▼
module world
        │
        ▼
semantic generation
        │
        ├── library exposure
        ├── binary target closure: atlas
        └── binary target closure: atlas-server
```

Development targets are overlays over that production package authority:

```text
production package generation
        │
        ├── test overlay
        ├── example overlay
        └── benchmark overlay
```

They do not redefine package identity.

The detailed target and manifest rules are specified in [`package-model-and-manifest.md`](./package-model-and-manifest.md).

---

## 4. `Archon.toml` and `Archon.lock`

`Archon.toml` is the authoritative package manifest.

It describes package identity, targets, dependencies, development dependencies, and package-level Archon configuration.

`Archon.lock` records the exact resolved dependency authority selected from version requirements.

Their responsibilities differ:

```text
Archon.toml
    desired package graph

Archon.lock
    exact resolved package graph
```

For example:

```toml
[dependencies]
parse = "^1.2"
```

may resolve to an exact immutable registry release such as:

```text
local:parse@1.4.1
```

The lockfile exists so a future build can recover the same dependency authority rather than resolving whatever happens to be newest at that time.

The manifest syntax and target semantics are owned by `package-model-and-manifest.md`.

Registry and lockfile resolution semantics belong to the registry specifications.

---

## 5. Archon and Telos responsibilities

The split MUST remain explicit.

| Responsibility | Owner |
|---|---|
| Parse Telos source | `telos` |
| Build module semantic authority | `telos` |
| Semantic analysis | `telos` |
| `.telosc` import/export and validation | `telos` |
| Executable lowering | `telos` |
| Runtime execution | `telos` |
| Read `Archon.toml` | `archon` |
| Read/update `Archon.lock` | `archon` |
| Workspace/package discovery | `archon` |
| Version requirement solving | `archon` |
| Registry access | `archon` |
| Path dependency realization | `archon` |
| Toolchain selection | `archon` |
| Universe artifact selection | `archon` |
| Cache lookup/promotion/GC policy | `archon` |
| Target selection | `archon` |
| Test harness orchestration | `archon` |
| Benchmark orchestration/statistics | `archon` |
| Package publication | `archon` |

A useful test is:

> If an operation requires `Archon.toml`, `Archon.lock`, registry selection, dependency solving, cache policy, workspace membership, or target selection, it belongs to Archon.

Conversely:

> Once all project and dependency choices are explicit, the remaining language/compiler operation belongs to Telos.

---

## 6. Persistence is intentionally split into four domains

The toolchain MUST NOT conflate installation state, registry authority, reusable global cache data, and mutable workspace state.

### Toolchain store

The toolchain store contains installed compiler-critical state:

```text
toolchain
├── telos
├── archon
└── universe.telosc
```

The active Universe artifact is pinned as part of the installed toolchain. It is not merely an evictable cache entry.

### Registry storage

A registry stores durable package-release authority.

The initial registry is local to one machine and contains immutable published package releases.

Deleting a compiler cache MUST NOT delete registry releases.

### Global cache

The global cache stores disposable, reconstructable, immutable objects shared across projects.

Examples include:

- compiled `.telosc` package artifacts;
- downloaded/canonical package source archives;
- other content-addressed immutable compiler outputs.

It is safe to garbage-collect these objects when they are no longer referenced.

### Project-local `.archon/`

A project's `.archon/` directory stores workspace-specific state:

```text
project/
├── Archon.toml
├── Archon.lock
├── src/
└── .archon/
    ├── state/
    ├── cache/
    └── build/
```

Typical contents include:

- source fingerprints;
- project/module topology state;
- incremental compiler reuse data;
- query/product fingerprints;
- target build indexes;
- references to globally cached immutable artifacts.

This state is disposable and MUST NOT define package authority.

---

## 7. Whole-package artifact principle

A `.telosc` compiler artifact is a freeze/thaw image of one completed package compiler generation.

The initial architecture MUST use:

> **one atomic package artifact per persistent package/project.**

It MUST NOT use a public artifact-per-source-file or artifact-per-module model.

For example:

```text
parse package
├── parse
├── parse.rule
├── parse.combinators
└── parse.internal
```

is frozen as:

```text
parse.telosc
```

not:

```text
parse.telosc
parse-rule.telosc
parse-combinators.telosc
...
```

Fine-grained incrementality remains an internal compiler/cache concern.

The artifact is the atomic portable publication boundary.

---

## 8. Normal build flow

A normal Archon build conceptually proceeds as follows:

```text
Archon.toml
   │
   ▼
resolve toolchain
   │
   ▼
resolve pinned Universe
   │
   ▼
resolve path + registry dependencies
   │
   ▼
consult Archon.lock
   │
   ▼
find compatible dependency artifacts
   │
   ├── cache hit  ───────────────┐
   │                             │
   └── cache miss                │
          │                      │
          ▼                      │
   compile immutable dependency  │
          │                      │
          ▼                      │
     publish to cache            │
          │                      │
          └──────────────────────┘
                    │
                    ▼
           construct resolved build plan
                    │
                    ▼
                  telos
                    │
                    ▼
        package semantic generation
                    │
          ┌─────────┴─────────┐
          │                   │
      freeze .telosc    emit target products
```

No lower compiler layer should independently repeat dependency resolution.

---

## 9. Check, build, run, test, and bench are different operations

The initial behavioral distinction is:

### `archon check`

Optimized for semantic feedback and incremental reuse.

It may update project-local incremental state without freezing a new canonical package artifact.

### `archon build`

Produces stable requested build outputs.

A successful build may freeze the completed package semantic generation into `.telosc` and emit executable target products.

### `archon run`

Selects one binary target—or one example target when explicitly requested—builds it if required, and executes it.

### `archon test`

Constructs and executes Archon-managed integration-test overlays.

Tests consume the package library surface and development dependencies. They are not production package declarations.

### `archon bench`

Constructs benchmark overlays and delegates benchmark execution control, measurement, and reporting to Archon.

The command-specific contract belongs to the future `archon-cli.md`.

---

## 10. Dependencies have two fundamentally different authorities

The initial dependency model intentionally distinguishes mutable development dependencies from immutable published dependencies.

### Path dependency

```toml
[dependencies]
parse = { path = "../parse" }
```

This means:

> Use this mutable local package checkout as development authority.

Path dependencies are expected to change as files change.

### Registry dependency

```toml
[dependencies]
parse = "^1.2"
```

This means:

> Resolve an immutable published package release from the configured registry.

A registry release cannot silently change after publication.

Archon MUST preserve this distinction.

It MUST NOT silently convert a registry dependency into an arbitrary source directory or treat a mutable checkout as the authoritative contents of an already-published package release.

---

## 11. The initial registry is local

The first package registry is intentionally local to one computer.

It exists to exercise the actual package-resolution and publication model before a network registry exists.

The local registry stores immutable releases such as:

```text
parse@1.2.0
text@1.0.0
slotmap@0.1.1
```

A release retains an immutable source snapshot and sufficient metadata for reproducible resolution.

Compatible `.telosc` files are derived compiler products and may live in the global content-addressed cache.

A future web registry MUST fit the same generic registry abstraction rather than changing package semantics.

The abstract registry model and local registry implementation are specified separately.

---

## 12. Universe is toolchain authority

Universe is a special built-in package, but it follows the same compiler-artifact philosophy.

Normal compilation should use a toolchain-pinned:

```text
universe.telosc
```

rather than reparsing/recompiling Universe source for every program.

Rebuilding Universe is a dedicated bootstrap/toolchain operation:

```text
packages/universe source
        │
        ▼
canonical Universe source compilation
        │
        ▼
candidate universe.telosc
        │
        ▼
decode / validate / parity checks
        │
        ▼
toolchain promotion
```

Normal artifact loading MUST NOT silently fall back to source when an explicitly selected Universe artifact is corrupt or incompatible.

Archon may explicitly choose a source rebuild as a higher-level recovery/build operation.

---

## 13. Determinism and authority

The toolchain should preserve several cross-cutting invariants.

### Stable package authority

Package identity must not depend on transient compiler session IDs, process IDs, memory addresses, or filesystem enumeration order.

### Deterministic artifacts

The same canonical inputs under the same compilation configuration should produce byte-identical portable artifacts.

### Fail-closed artifact loading

An incompatible or corrupt artifact must be rejected.

### Atomic publication

Artifact/cache/registry publication must not replace a known-good object with a partially written object.

### Visibility remains semantic

The fact that a private declaration is represented inside an artifact does not make it accessible to a consumer.

### No duplicate semantic ontology

Imported artifacts must reconstruct current compiler semantic products and authority. Wire DTOs are not a second semantic model.

---

## 14. Specification ownership

The toolchain specifications are intentionally divided by responsibility.

### [`package-model-and-manifest.md`](./package-model-and-manifest.md)

Owns:

- package identity;
- `Archon.toml`;
- target kinds;
- `lib`, `bin`, `test`, `example`, `bench`;
- target entry semantics;
- production vs development targets;
- dependency declaration syntax;
- manifest validation.

### `builds-artifacts-and-caches.md`

Will own:

- build/check persistence behavior;
- package artifact production;
- project cache;
- global cache;
- content-addressed objects;
- freshness and invalidation;
- deterministic and transactional artifact publication.

### `telos-cli.md`

Will own the low-level Telos compiler/runtime command surface.

### `archon-cli.md`

Will own Archon command syntax, target selection, project orchestration, testing, benchmarking, package mutation, cache commands, and toolchain commands.

### `registry.md`

Will own the generic package registry abstraction, package release identity, version requirement solving, lockfile authority, and publication semantics.

### `local-registry.md`

Will own the first concrete registry implementation: the durable single-machine local registry.

### `resolved-build-plan.md`

Will define the normalized information contract passed from Archon to Telos once project/toolchain/dependency resolution is complete.

---

## 15. Initial exclusions

The first architecture deliberately does not define:

- a network package registry;
- remote package publication;
- user accounts or authentication;
- remote build caches;
- distributed compilation;
- multiple library targets per package;
- unit-test declarations embedded in production source;
- documentation tests;
- harness-free test targets;
- arbitrary target-specific package identities;
- transparent source fallback inside artifact loading.

These may be designed later without weakening the initial package/artifact boundary.

---

## 16. Architectural summary

The intended system can be summarized as:

```text
                      Archon.toml
                          │
                          ▼
                        ARCHON
          ┌───────────────┼────────────────┐
          │               │                │
      toolchain        registry          caches
          │               │                │
          └───────────────┼────────────────┘
                          │
                  resolved build plan
                          │
                          ▼
                         TELOS
                          │
                package semantic world
                          │
             ┌────────────┴────────────┐
             │                         │
         package.telosc           target products
```

The central invariant is:

> **A Telos package is the persistent compiler-authority boundary. Archon decides which package world should exist; Telos realizes that world.**
