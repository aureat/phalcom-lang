# Builds, Artifacts, and Caches

This specification defines how Archon builds Telos packages, how completed compiler authority is persisted, how incremental work is reused, and how project-local state, global cache state, registry authority, and toolchain state remain distinct.

The governing rule is:

> **Builds are reproducible realizations of resolved package inputs. Package artifacts are immutable completed compiler generations. Caches are disposable acceleration structures.**

This specification deliberately separates:

```text
package semantic generation
package compiler artifact
target executable product
project-local incremental state
global content-addressed cache
registry release authority
toolchain-installed authority
```

They may refer to one another, but they are not interchangeable.

---

# The build model

Archon owns build orchestration.

Telos owns compilation semantics.

A normal build therefore has two distinct phases:

```text
Archon resolution
    │
    ├── package
    ├── toolchain
    ├── Universe
    ├── dependencies
    ├── lock authority
    ├── target selection
    ├── profile/configuration
    └── cache lookup
    │
    ▼
ResolvedBuildPlan
    │
    ▼
Telos compilation
    │
    ├── import compatible package artifacts
    ├── parse source that must be compiled
    ├── link module worlds
    ├── perform semantic analysis
    ├── maintain compiler semantic products
    ├── lower requested executable targets
    └── freeze package compiler artifact when requested
```

A compiler layer MUST NOT independently repeat package resolution that Archon already performed.

---

# Package generation versus target product

A package semantic generation is target-independent where possible.

For a package:

```text
atlas
├── lib: atlas
├── bin: atlas
└── bin: atlas-server
```

the intended structure is:

```text
package sources + production dependencies
        │
        ▼
PackageGeneration P
        │
        ├── library surface
        ├── target lowering: atlas
        └── target lowering: atlas-server
```

Archon and Telos SHOULD share all target-independent analysis rather than compiling each binary as an isolated project.

Target products derive from `P`.

The package compiler artifact freezes `P`.

---

# Development overlays

The development target kinds:

```text
test
example
bench
```

do not redefine the canonical production package generation.

Instead:

```text
PackageGeneration P
    ├── TestOverlay T
    ├── ExampleOverlay E
    └── BenchOverlay B
```

produces development generations:

```text
PT
PE
PB
```

These may be cached, but they MUST NOT be published as the package's canonical production `.telosc`.

Running:

```bash
archon test
```

must not accidentally alter what downstream users obtain when they consume the package as a library.

---

# What `archon check` means

`archon check` is optimized for correctness feedback and incremental reuse.

Its default responsibilities are:

```text
resolve package environment
resolve/import dependencies
restore reusable local compiler state
discover changed source/configuration
invalidate affected products
recompute required semantic authority
report diagnostics
persist reusable project-local state
```

A successful `archon check` does **not** require production of a new canonical package `.telosc`.

This allows:

```text
edit
check
edit
check
```

to remain an incremental compiler workflow rather than repeatedly freezing whole-package artifacts.

Archon MAY choose to checkpoint additional reusable state after a successful check, but that state remains cache state.

---

# What `archon build` means

`archon build` requests durable build products.

For a production package, this generally means:

```text
resolve
        ↓
compile/update package generation
        ↓
successful completed generation
        ↓
freeze package compiler artifact
        ↓
validate artifact
        ↓
publish immutable artifact to cache/build store
        ↓
lower/emit requested target products
```

A build MUST NOT publish a package artifact from an incomplete or semantically failed generation.

The canonical `.telosc` must correspond to a successfully completed package generation.

---

# What `archon run` means

`archon run` is:

```text
target selection
    +
build-if-required
    +
execute selected product
```

The build machinery remains identical to ordinary build machinery.

`archon run` MUST NOT establish a second ad hoc compilation path.

For example:

```bash
archon run --bin server
```

must use the same package/dependency/toolchain authority that:

```bash
archon build --bin server
```

would use.

---

# What `archon test` means

Testing is an Archon-managed development build.

Conceptually:

```text
production package generation P
        │
        ├── normal dependencies
        └── library surface
                │
                ▼
        test development environment
                │
        + dev dependencies
        + selected test targets
                │
                ▼
        integration-test generation
                │
                ▼
          Archon test harness
```

The base production generation should be reused when valid.

Test-specific state belongs to development build/cache state.

It does not become part of the package's published semantic authority.

---

# What `archon bench` means

Benchmarking follows the same overlay principle:

```text
production generation
    +
dev dependencies
    +
benchmark target
        ↓
benchmark executable/workload
        ↓
Archon measurement harness
```

Archon owns:

```text
warm-up
iteration policy
measurement
statistics
baseline comparison
reporting
```

Telos owns compilation and execution semantics.

Benchmark result databases, if introduced, are neither package artifacts nor compiler cache authority.

---

# Package compiler artifacts

A `.telosc` is an immutable, validated compiler image representing one completed package generation.

The public artifact boundary is:

> **one persistent package → one package compiler artifact per completed compatible generation**

A package containing:

```text
parse
parse.rule
parse.combinators
parse.internal
```

is persisted as:

```text
parse.telosc
```

not as independently consumable module artifacts.

Module/query-level reuse may exist internally in caches.

That internal granularity MUST NOT weaken whole-package atomic artifact publication.

---

# Artifact atomicity

Consumers must never observe a partially updated package artifact.

Artifact production must follow a transactional sequence such as:

```text
completed semantic generation
        ↓
encode candidate
        ↓
write temporary object
        ↓
validate container/header/sections
        ↓
decode/self-validate if required
        ↓
compute final artifact digest
        ↓
publish immutable content-addressed object
        ↓
atomically update build/index reference
```

Archon MUST NOT truncate or overwrite the currently usable artifact in place.

If production of a new artifact fails, the prior known-good immutable object remains valid.

---

# Artifact determinism

Portable artifacts SHOULD be byte-deterministic for identical canonical inputs and compilation configuration.

The serialized result MUST NOT depend on incidental process state such as:

```text
PID
wall-clock timestamp
temporary directory
absolute checkout path
random session ID
memory address
hash-table iteration order
thread scheduling
filesystem enumeration order
```

If non-semantic metadata such as build timestamp is ever included, it MUST be kept outside the deterministic artifact identity or normalized so that it does not invalidate reproducibility.

---

# Build identity is not artifact identity

Two identities must remain distinct.

## Build key

A `BuildKey` answers:

> Do we already have a valid result for this exact set of build inputs and compatibility requirements?

It is an input identity.

## Artifact digest

An `ArtifactDigest` answers:

> Which exact immutable artifact bytes/result is this?

It is an output identity.

They MUST NOT be conflated.

Conceptually:

```text
BuildKey
    ↓ lookup
ArtifactDigest
    ↓
immutable artifact bytes
```

Different implementations may produce the same artifact bytes for equivalent build requests, and a build system may evolve how it indexes build requests without changing artifact content identity.

---

# BuildKey inputs

The precise representation belongs to implementation, but the package build key must account for every input capable of changing the relevant compiler result.

Conceptually:

```text
package release/development identity
source authority/fingerprint
manifest semantic configuration
toolchain/compiler compatibility
Universe authority
resolved dependency authorities
dependency interface/semantic fingerprints as appropriate
compile profile
target-independent language configuration
feature/configuration state when introduced
```

Target-specific products extend package-generation identity with:

```text
target kind
target name
target entry
runtime/lowering configuration
```

Development targets additionally include:

```text
dev dependencies
development overlay source
harness/configuration
```

---

# Compatibility identity

A cached `.telosc` must not be reused solely because:

```text
package name matches
package version matches
```

Reuse also depends on compiler compatibility.

Relevant compatibility dimensions include the compiler-artifact architecture's versioned contracts, such as:

```text
container schema
minimum reader schema
semantic model version
stable identity schema
module semantic ABI
semantic artifact ABI
query fingerprint ABI
native surface schema/fingerprint
type metadata schema
capabilities
```

Archon may summarize these into a normalized compiler compatibility key for cache lookup.

Telos remains the final authority that validates whether an artifact is actually import-compatible.

---

# Source digest, interface digest, semantic digest, and artifact digest

These identities answer different questions.

## Source digest

> What canonical source/provenance snapshot is this?

Useful for release integrity and freshness.

## Interface digest

> What externally relevant package interface authority changed?

Useful for downstream invalidation where implementation-only changes do not matter.

## Semantic digest

> What completed package semantic authority changed?

Useful where deeper semantic compatibility matters.

## Query/product fingerprints

> Can one specific compiler product be reused against its exact dependencies?

Useful for fine-grained incremental compilation.

## Artifact digest

> What exact immutable encoded artifact is this?

Useful for exact object identity and trusted artifact authority.

Build logic SHOULD use the narrowest correct dependency identity rather than invalidating everything on every source byte change.

---

# Dependency invalidation

A package depending on another package should not necessarily rebuild every product when an implementation detail changes.

The build architecture should permit dependency edges to use relevant authority.

Examples:

| Consumer dependency | Relevant authority |
|---|---|
| import/name resolution | interface/module topology |
| public type surface | interface/type metadata |
| deep semantic product | semantic/product fingerprint |
| cached query reuse | query input/product/dependency fingerprints |
| docs/presentation | metadata digest |
| exact artifact provenance | artifact digest |
| registry release integrity | source digest |

The initial implementation may invalidate more broadly while correctness is established, but it MUST NOT define over-invalidation as the permanent semantic contract.

---

# Project-local Archon state

Every mutable workspace may have:

```text
.archon/
```

This is disposable build state owned by Archon.

A conceptual initial organization is:

```text
.archon/
├── state/
│   ├── workspace
│   ├── resolution
│   └── build-index
│
├── cache/
│   ├── source/
│   ├── modules/
│   ├── semantic/
│   └── queries/
│
└── build/
    ├── debug/
    └── release/
```

The exact filenames and encoding are implementation details unless explicitly surfaced by Archon.

---

# What belongs in project-local state

Project-local state is appropriate for data valuable because it corresponds to the current mutable workspace.

Examples:

```text
source path -> source fingerprint
source revisions
module discovery state
workspace project topology
path dependency topology
incremental semantic products
query fingerprints/dependency evidence
diagnostic reuse state
target build indexes
references to global immutable objects
```

This state may include host paths because it is workspace-local and disposable.

It MUST NOT be used as portable package authority.

---

# What should not initially be globalized

The global cache SHOULD NOT initially contain arbitrary mutable-workspace compiler query state such as:

```text
/home/user/dev/app/src/main...
session-specific QueryKey products
editor revisions
WorkspaceId authority
temporary module IDs
uncommitted path-dependent semantic snapshots
```

Globalizing such state prematurely creates problems with:

```text
absolute paths
privacy
session identity
workspace provenance
unbounded object growth
cross-workspace invalidation
```

Fine-grained persistent query caching should begin project-local.

It may be generalized later if measurement justifies a portable cache format.

---

# Global cache

The global Archon cache stores immutable reusable objects shared between projects.

Its logical shape is:

```text
ArchonCacheDir/
├── objects/
├── sources/
├── downloads/
└── indexes/
```

A content-addressed object store is preferred:

```text
objects/
└── <digest algorithm>/
    └── <digest-prefix>/
        └── <full digest>
```

Exact sharding is implementation detail.

---

# Global cache properties

Global cache objects are:

- immutable after publication;
- addressed by strong content identity;
- reusable across projects;
- disposable;
- reconstructable from authoritative source/toolchain/registry inputs;
- eligible for garbage collection.

A cache entry MUST NOT become the only authoritative copy of a registry release.

A cache entry MUST NOT become the only authoritative installed copy of a toolchain component that the toolchain manager claims is installed.

---

# Global cache contents

Appropriate global objects include:

```text
package .telosc artifacts
canonical materialized registry source trees
downloaded package archives
other immutable compiler outputs
possibly executable target objects when portable/reusable
```

For example:

```text
ArtifactDigest A
    -> immutable parse.telosc bytes
```

Multiple projects depending on the same compatible `parse` artifact should reuse one global object.

---

# Build indexes

Content addressing answers:

> Where are bytes for digest A?

It does not answer:

> Which digest should this build request use?

Archon therefore needs build indexes.

Conceptually:

```text
BuildKey(package-generation request) -> ArtifactDigest
BuildKey(bin/server)                 -> ExecutableDigest
```

A project-local index may point to globally stored objects.

This permits:

```text
project/.archon/build/debug/server
    -> global object digest E
```

without physically duplicating large immutable output.

The representation may be metadata rather than filesystem symlinks.

---

# Registry storage is not cache

The local registry stores immutable published package release authority.

It must remain separate from:

```text
ArchonCacheDir
```

Deleting:

```bash
archon cache clean
```

MUST NOT remove:

```text
local:parse@1.4.1
```

from the registry.

After cache deletion, Archon may rebuild a compatible `parse.telosc` from the registry's immutable source release.

This is a required architectural proof.

---

# Toolchain storage is not cache

Installed toolchains are likewise not ordinary cache state.

Conceptually:

```text
ArchonDataDir/
└── toolchains/
    └── <toolchain>/
        ├── bin/
        │   ├── archon
        │   └── telos
        └── lib/
            └── universe.telosc
```

The exact installation layout may vary.

The active Universe is pinned toolchain authority.

A generic cache GC MUST NOT remove it from an installed toolchain.

The same bytes may also be deduplicated through a global object store internally, but the toolchain must hold a strong durable reference.

---

# Universe rebuild behavior

Normal package compilation uses the toolchain-pinned Universe artifact.

Rebuilding Universe is a dedicated bootstrap operation.

The normal path:

```text
toolchain universe.telosc
        ↓
Telos artifact import
        ↓
package compilation
```

The Universe rebuild path:

```text
compiler bootstrap/native substrate
        ↓
packages/universe source
        ↓
canonical Universe source compilation
        ↓
candidate universe.telosc
        ↓
decode/validate
        ↓
source-vs-artifact parity proof
        ↓
source-free downstream canaries
        ↓
determinism check
        ↓
atomic toolchain promotion
```

The previous known-good Universe artifact remains installed until the candidate passes all promotion gates.

---

# Cache lookup precedence

The normal artifact lookup strategy should be explicit and deterministic.

For dependencies, the conceptual precedence is:

```text
1. exact artifact required by explicit invocation, if any
2. exact project/lockfile build reference
3. project-local build index
4. global content-addressed/build cache
5. registry release source -> explicit rebuild
```

For Universe:

```text
1. explicit development/bootstrap override, when allowed
2. active toolchain-pinned Universe artifact
```

Archon MUST NOT scan for the "newest" `.telosc` file by filesystem timestamp.

Artifact selection is identity-driven.

---

# Source fallback is an Archon decision

Artifact import itself is fail-closed.

If Telos is asked to import:

```text
parse.telosc
```

and reports:

```text
corrupt
incompatible
authority mismatch
```

that import has failed.

Telos MUST NOT silently find `parse` source and continue.

Archon may then make an explicit higher-level decision:

```text
selected registry release still has valid immutable source authority
        ↓
rebuild package with current toolchain
```

That is a new build action, not hidden fallback.

---

# Cache publication

Publishing to the global cache is transactional.

The general pattern is:

```text
produce bytes
    ↓
compute content digest
    ↓
write temporary object
    ↓
validate
    ↓
atomic publish at digest path
```

If the same digest already exists, Archon may verify and reuse the existing object.

A content-addressed object MUST NOT be overwritten with different bytes.

Digest collision or mismatching pre-existing content is a fatal integrity error.

---

# Cache corruption

Because the global cache is disposable, corruption can often be repaired by removing the corrupt object and reconstructing it from authority.

However, repair policy depends on invocation.

## Implicit cache hit

If Archon discovers a corrupt cache object while satisfying an ordinary build and authoritative source remains available, it may:

```text
mark/remove corrupt cache object
rebuild/refetch explicitly
```

with suitable diagnostics.

## Explicit artifact input

If the user explicitly supplied an artifact path, corruption MUST fail.

Archon/Telos must not silently substitute a different artifact.

## Registry authority corruption

Registry corruption is not cache corruption and MUST fail closed according to the registry specification.

---

# `archon clean`

`archon clean` removes project-local build state for the selected project/workspace.

It may remove:

```text
.archon/state
.archon/cache
.archon/build
```

subject to future selective flags.

It MUST NOT by default remove:

```text
global cache
local registry
installed toolchains
```

This operation should be safe for resolving persistent local build inconsistencies without destroying globally reusable data.

---

# Global cache maintenance

Archon should provide separate cache operations.

Conceptually:

```bash
archon cache status
archon cache path
archon cache gc
archon cache clean
```

## `cache gc`

Garbage collection removes globally cached objects that are not protected by active references/policy.

Possible roots include:

```text
recent project build references
installed toolchains
registry artifact references if treated as strong
explicitly pinned objects
```

The exact retention policy is implementation-defined initially.

## `cache clean`

An explicit destructive cache clean may remove all reconstructable global cache objects.

It still MUST NOT delete:

```text
registry releases
toolchain installations
project source
```

---

# Profiles and compilation configuration

Build profiles such as:

```text
debug
release
```

are Archon build configuration.

They may affect:

```text
runtime lowering
assertion/contract projection
optimization
debug metadata
executable output
```

If a profile affects semantic compiler output, that effect must be represented in the package-generation `BuildKey`.

If it affects only executable lowering, it should not unnecessarily invalidate target-independent semantic artifacts.

The build system SHOULD maintain this distinction rather than hashing the complete profile object into every possible cache identity.

---

# Future compilation configuration fingerprint

As Telos grows, semantics may become dependent on configuration such as:

```text
target architecture
target OS/ABI
feature selection
conditional declarations
pointer width
FFI/native availability
language edition
```

When such configuration materially affects semantic output, it should be represented through an explicit normalized compilation-configuration fingerprint.

The artifact format SHOULD NOT accumulate arbitrary one-off header fields for every future build option unless the option is itself part of artifact compatibility.

---

# Memory behavior during artifact import

Artifact import should not permanently retain redundant representations without need.

The preferred eager-load lifecycle is:

```text
encoded artifact bytes
        ↓
validate/decode portable DTOs
        ↓
reconstruct/relocate current compiler products
        ↓
publish immutable imported generation
        ↓
drop temporary decoded wire DTOs where no longer required
```

Steady state should avoid retaining all of:

```text
encoded artifact bytes
portable SemanticWorld DTOs
portable QueryImage DTOs
live TypeStore products
live StaticValueStore products
live SemanticDb products
```

simultaneously unless a measured feature requires them.

---

# Relocation efficiency

Portable dense artifact IDs should be relocated with dense tables where possible.

Examples:

```text
Vec<PortableTypeId -> live TypeId>
Vec<PortableKindId -> live KindId>
Vec<PortableModuleId -> live ModuleId>
```

Stable symbolic maps should be used where symbolic lookup is semantically required, not as a default for every relocation edge.

This matters because package artifact import is expected to become part of normal compiler startup.

---

# Incremental compiler persistence

Project-local incremental persistence may eventually store reusable query/product state such as:

```text
QueryKey
InputFingerprint
ProductFingerprint
DependencyFingerprint set
portable reusable product payload
```

However, this feature is subordinate to the package artifact architecture.

The initial system MUST first establish:

```text
correct package artifact freeze/thaw
correct imported semantic authority
correct local mutable overlay
```

before introducing a broad persistent global query cache.

A cache design MUST NOT become a second semantic database model.

---

# Successful build publication

A production package build is complete only when:

```text
1. dependency authority is resolved;
2. package semantic analysis succeeds;
3. requested production targets are valid;
4. the completed package generation is internally consistent;
5. any requested package artifact encodes successfully;
6. the encoded artifact validates;
7. the immutable artifact is published transactionally;
8. requested executable products are emitted successfully or failure is reported;
9. build indexes are updated only to valid completed products.
```

No build index may point at a partially emitted object.

---

# Deterministic rebuild proof

For a package whose canonical inputs have not changed:

```text
Build #1 -> artifact digest A
Build #2 -> artifact digest A
```

should hold when using the same compiler/toolchain/configuration.

A determinism regression should be treated as a compiler/build-system defect, not ordinary cache churn.

---

# Source-free dependency proof

An important package-artifact acceptance test is:

```text
1. compile dependency A from source;
2. freeze A.telosc;
3. start a fresh compiler process;
4. make A source unavailable;
5. compile downstream package B using A.telosc;
6. prove A is not parsed, linked from source, or semantically recompiled;
7. produce the same relevant downstream semantic/runtime behavior.
```

This distinguishes genuine artifact reuse from a loader that still depends on source.

---

# Local-registry rebuild proof

The corresponding cache/registry proof is:

```text
1. publish A@1.0.0 to local registry;
2. build B against A;
3. delete the global cache;
4. preserve local registry;
5. rebuild B;
6. Archon materializes immutable A@1.0.0 source;
7. Telos rebuilds a compatible A.telosc;
8. B compiles successfully against the rebuilt artifact.
```

This proves:

```text
registry = durable authority
cache = acceleration
```

rather than the reverse.

---

# Build-system invariants

The implementation MUST preserve the following.

### One package artifact boundary

Modules and targets do not become independently publishable semantic artifacts by accident.

### Immutable global objects

A digest-addressed object is never mutated in place.

### Disposable caches

Deleting caches changes performance, not package meaning.

### Durable registry authority

Published package releases survive cache deletion.

### Durable toolchain authority

Installed Universe/compiler state survives generic cache GC.

### No hidden source fallback

Artifact failure and source compilation are distinct operations.

### Deterministic selection

Artifact choice is identity-driven, never "newest file wins."

### Fine-grained invalidation without semantic weakening

Optimization may reduce work but must preserve the same compiler authority as a clean build.

### Transactional publication

Failed builds do not poison current build indexes or known-good artifacts.

### Current semantic model remains authoritative

Persistent wire/cache products reconstruct current compiler products; they do not become a parallel ontology.

---

# Initial implementation priorities

The build/cache architecture should be introduced in this order:

```text
1. package-level .telosc freeze/thaw correctness
2. source-free dependency artifact import
3. project-local build index
4. global immutable artifact CAS
5. local-registry source rebuild integration
6. target-product caching
7. persistent project-local query/module reuse
8. profiling-driven refinement
```

Do not begin by designing a highly granular global incremental database.

The package artifact path is the architectural spine.

---

# Summary

The build system should be understandable through four layers:

```text
AUTHORITATIVE
─────────────
toolchain store
local/remote registries
project source

DERIVED + DURABLE UNTIL CLEANED
───────────────────────────────
global immutable cache objects

MUTABLE + WORKSPACE-LOCAL
─────────────────────────
project .archon/ state

LIVE
────
SemanticWorkspaceSession
```

And through one central flow:

```text
resolved inputs
    ↓
incremental compilation
    ↓
completed package generation
    ↓
freeze immutable package artifact
    ↓
cache/share/reload
```

A `.telosc` is therefore not the mutable incremental compiler database.

It is the portable immutable freeze of a completed package compiler generation.
