# Resolved Build Plan

This specification defines the information contract Archon supplies to Telos after package, dependency, toolchain, cache, and target resolution are complete.

It exists to preserve the central toolchain boundary:

> **Archon decides what exact world should be compiled. Telos compiles that exact world.**

Without an explicit resolved-plan boundary, Archon and Telos would gradually duplicate responsibility: both would discover manifests, choose package versions, locate Universe, interpret lockfiles, and infer targets.

The `ResolvedBuildPlan` prevents that drift.

It is an internal architectural contract first. This specification does not require the first implementation to serialize it as a public JSON/TOML protocol or execute Telos as a separate process.

---

# The state before resolution

A user's request begins with unresolved intent.

For example:

```bash
archon build --bin server
```

plus:

```toml
[package]
name = "atlas"
namespace = "atlas"
version = "1.3.0"

[lib]
root = "atlas"

[[bin]]
name = "server"
entry = "atlas.server"

[dependencies]
http = "^2.0"
parse = "^1.4"
```

This information is insufficient for Telos compilation.

Questions remain:

```text
Which exact http release?
Which exact parse release?
Which registry?
Which compatible dependency artifacts?
Which toolchain?
Which exact Universe artifact?
Which package source authority?
Which compile profile?
Which binary target definition?
Which dev dependencies, if any?
Which cached artifacts are trusted?
```

Those are Archon questions.

---

# The state after resolution

Archon transforms unresolved user intent into a complete normalized compiler request.

Conceptually:

```text
Archon.toml
Archon.lock
CLI selection
toolchain configuration
registry state
project/workspace state
cache indexes
        │
        ▼
      ARCHON
        │
        ▼
ResolvedBuildPlan
        │
        ▼
       TELOS
```

By the time Telos receives the plan:

> **No package version requirement remains unresolved.**

Telos must not need to ask a registry which dependency version to use.

---

# What the plan represents

A `ResolvedBuildPlan` represents one requested compilation realization.

It identifies:

```text
root package authority
resolved dependency graph
toolchain/compiler authority
Universe authority
source authorities
artifact authorities
target request
compilation configuration
output request
```

It does not contain mutable compiler semantic products.

Those are created or imported by Telos.

The plan is therefore:

```text
resolved input authority
```

not:

```text
compiler database snapshot
```

---

# Root package authority

The plan must identify the package being compiled.

For a mutable workspace package, this conceptually includes:

```text
package identity
manifest authority
canonical package root/source provider
workspace/project identity
source revision/fingerprint authority
```

For example:

```text
Package:
    name: atlas
    namespace: atlas
    version: 1.3.0
    authority: WorkspacePath(...)
```

The exact host path may be required to read mutable source, but the plan must distinguish:

```text
host source location
```

from:

```text
persistent package identity
```

A temporary extraction or checkout path must not silently redefine package semantic identity.

---

# Resolved dependency graph

Every dependency edge is exact.

The unresolved manifest might say:

```toml
parse = "^1.4"
```

The resolved plan says, conceptually:

```text
alias: parse
package: parse
registry: local
version: 1.4.1
source digest: S
resolved project/package authority: P
```

If a compatible artifact is selected:

```text
artifact digest: A
artifact fingerprint: F
artifact compatibility: C
artifact location/object handle: ...
```

The compiler receives exact dependency authority, not a version range.

---

# Path dependencies

A path dependency is represented differently because its authority is mutable development source.

Conceptually:

```text
alias: util
kind: Path
canonical source root: /workspace/util
package identity: util
source authority/fingerprint: U
```

It is not represented as:

```text
registry = local
version = ...
```

unless it has independently been published and the manifest actually selected that registry release.

The plan must preserve the path-vs-registry distinction established by `Archon.toml`.

---

# Registry dependencies

A registry dependency contains exact release authority.

Conceptually:

```text
kind: Registry

registry: local
package: parse
version: 1.4.1
source digest: S
```

The plan may additionally contain a trusted registry-resolution receipt binding those values.

If Archon selected a compiled artifact, the plan must provide enough external evidence for Telos to verify that the artifact is authorized for this resolved package release.

---

# Dependency aliases

A dependency edge has two names where aliasing is used:

```text
import alias
registry package name
```

For:

```toml
[dependencies]
parser = { package = "parse", version = "^1.4" }
```

the resolved plan preserves:

```text
alias = parser
package = parse
```

Telos uses the alias when constructing the root package's import environment.

Registry/package authority remains attached to `parse`.

---

# Dependency graph, not flat list

The plan must represent the resolved graph rather than merely direct dependencies.

For example:

```text
app
├── parse@1.4.1
│   └── text@1.2.0
└── http@2.3.0
    └── text@1.2.0
```

The exact `text@1.2.0` node should be represented once as one resolved package authority and referenced by both parents.

This supports:

```text
shared imported generation
stable dependency identity
deduplicated artifact loading
consistent transitive resolution
```

The compiler should not independently discover a different transitive package graph.

---

# Toolchain authority

The plan identifies the active toolchain/compiler compatibility authority.

Conceptually:

```text
toolchain id
compiler version/build identity
semantic model version
artifact reader/writer compatibility
native surface schema/fingerprint
type metadata schema
language/configuration version
```

Not every field must be duplicated if Telos can obtain it intrinsically from its own binary/runtime.

However, Archon and Telos must agree on the toolchain identity against which package/artifact selection was performed.

Archon must not select dependency artifacts under one compatibility model and invoke an unrelated Telos compiler under another.

---

# Universe authority

Universe is explicit resolved input authority.

The normal plan contains an exact toolchain-pinned Universe artifact.

Conceptually:

```text
UniverseAuthority::Artifact {
    package: universe,
    version: ...,
    artifact_digest: U,
    artifact_location: ...,
    compatibility: ...
}
```

Normal package compilation MUST NOT ask Telos to search for Universe.

Archon has already selected it.

---

# Universe bootstrap plans

A Universe rebuild is a distinct plan kind or explicit authority mode.

Conceptually:

```text
UniverseAuthority::BootstrapSource {
    canonical_source: packages/universe,
    project_identity: Universe,
    bootstrap_mode: ...
}
```

This prevents accidental behavior such as:

```text
rebuild Universe
while also importing installed Universe as the same canonical authority
```

The bootstrap path must be explicit.

It is not ordinary fallback.

---

# Source authorities

Every package that Telos may compile from source requires an explicit source authority.

Possible kinds include conceptually:

```text
WorkspaceSource
PathDependencySource
RegistrySnapshotSource
UniverseBootstrapSource
Standalone/explicit source, where supported
```

A source authority includes:

```text
persistent semantic owner
source provider/materialization handle
source fingerprint/provenance where available
```

It must not force Telos to infer ownership from whichever filesystem path happens to be supplied.

---

# Artifact authorities

Every precompiled package input selected by Archon is represented as an explicit artifact authority.

Conceptually:

```text
ResolvedArtifact {
    package authority
    full artifact digest
    artifact fingerprint
    compiler compatibility evidence
    object/path handle
    resolver trust receipt
}
```

Telos may then:

```text
open
decode
validate
relocate
publish imported generation
```

using its compiler-artifact subsystem.

If validation fails, the artifact input fails.

The plan does not authorize Telos to silently reinterpret a source directory as replacement authority.

---

# Source or artifact is an explicit decision

For every dependency, Archon chooses a realization mode:

```text
UseArtifact
or
CompileSource
```

This is important.

A plan MUST NOT mean:

```text
try artifact, and if anything goes wrong, use source
```

because that hides authority and correctness failures.

Instead:

```text
Plan 1:
    UseArtifact(A)
```

may fail.

Archon can then construct a new explicit build action:

```text
Plan 2:
    CompileSource(S)
```

if policy and authoritative source permit it.

This separation keeps artifact validation fail-closed.

---

# Target selection

The plan contains exact target selection.

For a binary:

```text
kind: bin
name: server
entry: atlas.server
```

Telos should not read `default-run` to decide what to compile.

`default-run` has already served its purpose inside Archon.

For an example:

```text
kind: example
name: basic
entry: atlas_examples.basic
```

For a test:

```text
kind: test
name: parser
entry: atlas_tests.parser
harness: ArchonManaged
```

For a benchmark:

```text
kind: bench
name: throughput
entry: atlas_benches.throughput
harness: ArchonManagedBenchmark
```

The plan may also represent package-only checking/building without a runnable target.

---

# Production target versus development overlay

The target selection must tell Telos whether it is operating on:

```text
production package world
```

or:

```text
development overlay
```

For:

```text
lib
bin
```

the requested target belongs to the production package world.

For:

```text
test
example
bench
```

the plan identifies:

```text
base production package generation
+
development source overlay
+
dev dependency graph
```

This allows Telos to reuse the base package generation while preserving the rule that development declarations do not pollute the canonical production artifact.

---

# Development dependencies

A normal production plan excludes dev dependencies unless required by a selected development target.

For:

```bash
archon build --lib
```

the dependency graph contains normal dependencies only.

For:

```bash
archon test
```

or:

```bash
archon run --example basic
```

the plan may extend the graph with `[dev-dependencies]`.

The plan must make this explicit rather than leaving Telos to reread `Archon.toml` and decide.

---

# Compilation mode and profile

The plan includes normalized compilation configuration.

Conceptually:

```text
profile:
    debug
    release

compile mode:
    checked
    release
    unchecked
    ...

optimization/lowering configuration
debug metadata policy
contract/assertion projection
```

Only settings relevant to Telos are included.

Archon-only presentation settings should not be part of the compiler plan.

The plan should normalize user-facing aliases/options into compiler-level configuration once.

---

# Future target/configuration dimensions

The plan is the right place to carry future resolved configuration such as:

```text
target architecture
target OS/ABI
feature set
conditional-compilation values
language edition
native capability set
FFI configuration
```

Telos should not query host/project configuration independently after Archon has resolved these choices.

Where a configuration dimension changes semantic output, it must participate in relevant BuildKeys and artifact compatibility/fingerprints.

---

# Requested outputs

The plan tells Telos what outputs Archon actually needs.

Examples:

```text
CheckOnly
FreezePackageArtifact
EmitExecutable(target)
FreezeArtifactAndEmitExecutable
CompileDevelopmentOverlay
Inspect/verify-only workflows where appropriate
```

This avoids requiring Telos to emit every possible product for every invocation.

For:

```bash
archon check
```

the plan may request:

```text
semantic completion
diagnostics
incremental state update
no mandatory package artifact
```

For:

```bash
archon build --bin server
```

it may request:

```text
completed package generation
package artifact freeze
server executable product
```

depending on build policy.

---

# Output destinations versus identities

Archon owns output placement policy.

Telos may receive destination handles/paths for requested outputs, but an output path is not output identity.

For example:

```text
.archon/build/debug/server
```

is a project build location/reference.

The actual immutable product may be globally addressed by:

```text
ExecutableDigest E
```

Similarly:

```text
some/path/parse.telosc
```

does not establish package identity merely by filename.

---

# Cache decisions belong to Archon

The resolved plan may include already-selected cached artifacts, but Telos should not search the global Archon cache itself.

The relationship is:

```text
Archon cache lookup
        ↓
selected object A
        ↓
ResolvedBuildPlan references A
        ↓
Telos validates/uses A
```

not:

```text
Telos receives package name
        ↓
Telos scans ~/.cache/archon
```

Cache directory layout must remain outside compiler semantics.

---

# Project-local incremental state

The plan may identify a persistent compiler-session/cache context for the current workspace.

However, Archon should not serialize compiler-internal query state into the plan itself.

A useful conceptual separation is:

```text
ResolvedBuildPlan
    exact external inputs and requested outputs

SemanticWorkspaceSession
    live mutable compiler state
```

An Archon daemon/LSP/build server may retain a live `SemanticWorkspaceSession` across multiple plans for the same workspace.

A command-line invocation may reconstruct one from artifact/source/cache state.

The plan does not dictate lifecycle strategy.

---

# Imported generations and one compiler world

When dependencies are supplied as `.telosc` artifacts, Telos must reconstruct them into the current compiler semantic model.

The desired live structure is conceptually:

```text
Immutable imported generation: Universe
Immutable imported generation: parse
Immutable imported generation: http
            │
            └──────────────┐
                           ▼
                 one SemanticWorkspaceSession
                           │
                 local package mutable overlay
```

The plan MUST NOT imply:

```text
one independent semantic database per dependency
```

or:

```text
portable artifact DTOs remain semantic authority
```

The compiler's current semantic products remain authoritative.

---

# Normalized project graph

By the time Telos is invoked, Archon has normalized package resolution into stable graph nodes.

The plan should avoid making Telos recursively parse dependency manifests merely to reconstruct the same package graph.

For each resolved node, Telos needs enough information to establish:

```text
stable project/package identity
dependency aliases/import roots
source or artifact authority
module-world authority
```

but not unresolved package-management syntax.

---

# Manifest data in the plan

Not every `Archon.toml` field belongs in the resolved plan.

For example:

```toml
default-run = "server"
```

does not need to be passed after Archon has already selected:

```text
bin/server
```

Similarly, registry version requirement text:

```toml
parse = "^1.4"
```

may be useful as provenance/diagnostic metadata but is no longer a compiler decision once Archon resolved:

```text
parse@1.4.1
```

The plan should prefer normalized compiler-relevant facts over raw manifest duplication.

---

# Proposed conceptual data model

The first implementation may use Rust structures resembling the following shape:

```rust
struct ResolvedBuildPlan {
    toolchain: ResolvedToolchain,
    universe: ResolvedUniverseAuthority,

    root: ResolvedRootPackage,
    projects: Vec<ResolvedProjectNode>,

    request: BuildRequest,
    configuration: ResolvedCompilationConfiguration,
    outputs: OutputRequest,
}
```

A project node might conceptually be:

```rust
struct ResolvedProjectNode {
    id: PlanProjectId,
    package: ResolvedPackageIdentity,
    source: ProjectInputAuthority,
    dependencies: Vec<ResolvedDependencyEdge>,
}
```

With:

```rust
enum ProjectInputAuthority {
    Source(ResolvedSourceAuthority),
    Artifact(ResolvedArtifactAuthority),
}
```

And an edge:

```rust
struct ResolvedDependencyEdge {
    alias: ModuleComponent,
    target: PlanProjectId,
}
```

These types are illustrative, not mandatory API names.

The architectural properties are mandatory.

---

# Plan-local IDs

A serialized or in-memory plan may use dense plan-local identifiers:

```text
PlanProjectId(0)
PlanProjectId(1)
...
```

These are references within the plan only.

They MUST NOT leak into persistent `.telosc` identity.

Persistent package/artifact authority uses stable package/project identities.

This permits efficient graph representation without confusing transient planning indexes with semantic identity.

---

# Root request model

A build request should distinguish package operations from target operations.

Conceptually:

```rust
enum RequestedTarget {
    Package,
    Library,
    Binary(TargetName),
    Test(TargetName),
    Example(TargetName),
    Bench(TargetName),
}
```

And action:

```rust
enum BuildAction {
    Check,
    Build,
    Run,
    Test,
    Bench,
}
```

The exact division may differ in implementation.

The important property is that by the time Telos acts, selection ambiguity has disappeared.

---

# Test harness contract

For a test plan, Archon owns harness policy.

The plan tells Telos that the selected development target participates in an Archon-managed test harness.

The initial test architecture does not support:

```text
unit tests in production modules
documentation tests
harness-free test target
```

Therefore no plan variant should initially encode those modes.

The harness may require Telos to emit structured test discovery/execution hooks, but Archon remains the owner of scheduling and aggregate result reporting.

---

# Benchmark harness contract

Similarly, a benchmark plan identifies an Archon-managed benchmark target.

Telos may expose compiled benchmark cases or an executable benchmark workload.

Archon owns:

```text
warm-up
iterations
timing
statistics
baseline comparison
reporting
```

The plan should carry compiler-relevant benchmark target information, not Archon's entire reporting configuration unless execution architecture actually requires it.

---

# Diagnostics ownership

Telos produces language/compiler diagnostics.

Archon adds orchestration diagnostics.

For example:

```text
"method does not exist"
    → Telos diagnostic

"no version of parse satisfies ^3"
    → Archon/registry diagnostic

"multiple binary targets; select --bin"
    → Archon diagnostic

"artifact semantic ABI incompatible"
    → Telos artifact diagnostic, surfaced by Archon
```

The plan should carry enough package/source context for Telos diagnostics to map back to stable package/module/source authority.

---

# Failure semantics

A plan execution can fail because:

```text
selected artifact is corrupt
selected artifact is incompatible
source package fails to compile
target entry is semantically invalid
requested output cannot be emitted
runtime lowering fails
```

These failures do not authorize Telos to alter the plan.

For example, if:

```text
Dependency A = Artifact(authority X)
```

fails validation, Telos reports failure.

It does not mutate the plan into:

```text
Dependency A = Source(...)
```

Archon may create a subsequent plan or build step after applying explicit rebuild policy.

---

# Plan determinism

For the same:

```text
Archon.toml
Archon.lock
toolchain
registry authority
workspace source state
CLI request
configuration
```

Archon should construct an equivalent normalized plan.

The plan must not depend on:

```text
filesystem enumeration order
hash-map iteration order
random IDs
PID
temporary extraction paths as persistent identity
```

Temporary paths may appear as access handles, but they must not alter normalized package authority.

---

# Plan fingerprint

Archon may compute a `ResolvedBuildPlanFingerprint` for caching/debugging.

Such a fingerprint should cover normalized build-relevant authority, not raw serialization bytes if those contain incidental fields.

It may be useful for:

```text
build index lookup
diagnostic tracing
"why did this rebuild?" tooling
reproducibility audits
```

A plan fingerprint is not the same as a package artifact digest.

---

# Plan inspection

Archon SHOULD eventually offer a way to inspect the resolved plan.

Conceptually:

```bash
archon build --plan
```

or:

```bash
archon explain build
```

could show:

```text
root: atlas
target: bin/server
toolchain: ...
Universe: digest ...
dependencies:
    http -> local:http@2.3.0 -> artifact ...
    parse -> local:parse@1.4.1 -> artifact ...
profile: debug
outputs:
    package artifact
    server executable
```

This is especially valuable while the artifact/cache architecture is being developed.

The exact CLI is deferred to the Archon CLI specification.

---

# No registry access inside Telos

A critical invariant is:

```text
ResolvedBuildPlan contains:
    exact package version

Telos does not receive:
    version range requiring solution
```

Telos MUST NOT perform:

```text
registry search
package version selection
lockfile mutation
package publication
```

These belong to Archon.

If the compiler needs external package authority during artifact import, that authority is supplied through the plan/resolver receipts.

---

# No cache policy inside Telos

Similarly, Telos MUST NOT decide:

```text
which global cache directory to use
which old objects to delete
which BuildKey index to consult
whether an artifact should be promoted globally
```

Telos can produce deterministic artifact bytes and validate supplied artifacts.

Archon manages storage policy.

---

# No target ambiguity inside Telos

Telos should not need to interpret:

```text
default-run
```

or choose from:

```text
server
client
admin
```

The plan contains one exact selected target or an explicit package-wide operation.

This keeps target UX out of compiler semantics.

---

# Build-plan versus compiler API

The resolved plan is an architectural contract, not necessarily one giant public function argument.

Implementation may decompose it into:

```text
ProjectUniverse input
ArtifactResolver authority
source provider
SemanticWorkspaceInput
target lowering request
output sink
```

as long as:

> No lower layer silently reintroduces an unresolved Archon decision.

The design should favor typed decomposition where ownership is clear.

---

# Build-plan versus process protocol

The first implementation may invoke Telos compiler libraries in-process.

Later, Archon may launch a standalone:

```text
telos
```

process.

This specification does not force that choice.

If a stable process protocol is introduced later, the protocol should serialize the same resolved semantics defined here.

Do not shape the architecture around today's process boundary.

---

# Minimal information required by Telos

A useful test for plan completeness is:

> If `Archon.toml`, `Archon.lock`, registry directories, and Archon's cache indexes became unavailable immediately after the plan was constructed, could Telos execute the requested compilation using only the plan's supplied authorities and referenced input objects?

For an already-resolved build, the answer should be yes.

Telos may of course need to read source/artifact bytes through paths/handles supplied by the plan.

It should not need to perform new package-management decisions.

---

# Example: normal binary build

Input:

```bash
archon build --bin server
```

Manifest:

```toml
[package]
name = "atlas"
namespace = "atlas"
version = "1.0.0"

[[bin]]
name = "server"
entry = "atlas.server"

[dependencies]
parse = "^1.4"
```

Resolved plan conceptually becomes:

```text
Toolchain:
    compiler = C
    Universe = artifact U

Root:
    atlas
    source = workspace /.../atlas

Dependencies:
    parse alias -> PlanProjectId(1)

Project 1:
    package = local:parse@1.4.1
    source digest = S
    realization = artifact A
    trusted artifact authority = ...

Request:
    Build
    Binary("server")
    entry = atlas.server

Configuration:
    profile = debug

Outputs:
    freeze package artifact
    emit server executable
```

Telos does not need the text:

```text
^1.4
```

to compile this plan.

---

# Example: test build

Input:

```bash
archon test --test parser
```

Resolved plan conceptually becomes:

```text
Base package:
    parse workspace generation

Production dependencies:
    text@1.2.0

Development dependencies:
    test-support@0.3.2

Development target:
    kind = test
    name = parser
    entry = parse_tests.parser

Harness:
    ArchonManagedIntegrationTest

Outputs:
    test executable/discovery product
    no replacement of canonical parse.telosc with test overlay
```

The compiler sees an explicit development overlay.

It does not infer test semantics from the directory name alone.

---

# Example: local registry cache miss

A consumer selects:

```text
local:parse@1.4.1
source digest S
```

No compatible artifact exists in the global cache.

Archon performs a source-compilation build step using the registry snapshot.

The plan for that step contains:

```text
Root:
    parse@1.4.1
    authority = registry release
    source = immutable materialized snapshot S

Dependencies:
    exact locked releases

Universe:
    U

Outputs:
    FreezePackageArtifact
```

After Telos succeeds:

```text
artifact A
```

is validated and promoted by Archon to the global cache.

A downstream plan may then reference:

```text
parse -> Artifact A
```

This demonstrates why source compilation and artifact consumption are explicit separate realizations.

---

# Implementation invariants

The first implementation must preserve:

### Fully resolved dependency authority

No version requirement is left for Telos to solve.

### Explicit Universe authority

Normal builds do not discover Universe implicitly.

### Explicit source/artifact realization

Every package input says whether Telos compiles source or imports an artifact.

### Trusted artifact authority

Artifacts are tied to independently resolved package authority.

### Target selection complete before compilation

No `default-run` or ambiguous target choice remains.

### Dev dependency separation

Development dependencies only enter plans that require development overlays.

### One package semantic world

Multiple targets do not become multiple package identities.

### No cache-directory semantics in compiler

Archon selects objects; Telos consumes them.

### No registry I/O in compiler

Archon resolves packages; Telos compiles.

### Fail-closed plan execution

Telos reports invalid supplied authority instead of silently inventing a replacement.

---

# Initial implementation sequence

A practical sequence is:

```text
1. Define typed resolved package/project graph.
2. Represent exact Universe authority.
3. Represent source-vs-artifact authority per resolved project.
4. Carry trusted package/artifact receipts.
5. Normalize target selection into one selected target request.
6. Normalize profile/compile mode.
7. Integrate with SemanticWorkspaceSession construction.
8. Route package artifact freeze through explicit output requests.
9. Add development overlay representation for test/example/bench.
10. Add optional plan fingerprint/inspection tooling.
```

The plan should remain small enough to reason about and rich enough that Telos never needs to become Archon.

---

# Summary boundary

The entire toolchain should preserve this transformation:

```text
USER INTENT
───────────
Archon.toml
Archon.lock
CLI options
registry configuration
toolchain configuration

        │
        ▼

ARCHON RESOLUTION
─────────────────
exact package graph
exact Universe
exact artifact/source authorities
exact target
exact profile
exact requested outputs

        │
        ▼

RESOLVED BUILD PLAN
───────────────────
no unresolved package-management decisions

        │
        ▼

TELOS
─────
language/compiler realization only
```

The key test is simple:

> **Once `ResolvedBuildPlan` exists, Telos should never need to ask, “Which package/version/target/toolchain did the user mean?”**
