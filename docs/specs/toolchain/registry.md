# Telos Package Registry Specification

This specification defines the registry model used by Archon to resolve and publish Telos packages.

It defines **what a registry means** independently of transport or storage. The first implementation is the single-machine local registry defined in [`local-registry.md`](./local-registry.md). A future network registry must implement the same authority model rather than introducing a second kind of package identity.

The central rule is:

> **A registry publishes immutable package releases. A compiled `.telosc` is a derived compiler artifact of a release, not the release itself.**

That distinction is foundational to package identity, lockfiles, artifact compatibility, compiler upgrades, and reproducible dependency resolution.

---

## 1. What a registry owns

A registry is an authority over published package releases.

It answers questions such as:

```text
Does package `parse` exist?
Which published versions of `parse` exist?
What immutable source release is `parse@1.4.1`?
What dependency requirements were published with that release?
What source digest identifies that release?
```

A registry does **not** own:

```text
workspace source edits
path dependencies
the active Archon workspace
project-local incremental state
SemanticWorkspaceSession state
target selection
compiler semantics
the global compiler cache
toolchain installation
```

A registry may advertise or associate compatible compiler artifacts with a release, but those artifacts remain derived build products.

---

## 2. Registry identity

Every registry has a stable logical identity:

```text
RegistryId
```

The first registry is:

```text
local
```

Future examples might be:

```text
official
company
experimental
```

A registry identifier participates in dependency authority.

These two releases are not automatically the same authority:

```text
local:parse@1.2.0
official:parse@1.2.0
```

even if their source bytes happen to be identical.

Content equality may be proven separately through source digests, but registry origin remains part of resolution provenance.

Archon MUST NOT silently substitute a release from one registry for a release selected from another registry.

---

# Package releases

A registry release is identified conceptually by:

```text
RegistryId
PackageName
PackageVersion
SourceDigest
```

The user-facing shorthand:

```text
parse@1.2.0
```

is therefore not sufficient by itself to establish exact immutable authority.

Once published, a registry record binds:

```text
(local, parse, 1.2.0)
```

to exactly one immutable source release.

The registry MUST reject an attempt to publish different source authority under that same registry/package/version coordinate.

## Why source release is primary

Compiler artifacts evolve.

For example:

```text
parse@1.2.0
source digest S
```

may originally be compiled under:

```text
semantic ABI 7
artifact ABI 4
```

and later need to be rebuilt under:

```text
semantic ABI 8
artifact ABI 5
```

The package release has not changed.

Its source authority is still:

```text
parse@1.2.0 / S
```

but its compatible derived artifact has changed.

The registry model therefore MUST NOT define:

```text
package version == one forever-valid .telosc digest
```

Instead:

```text
PackageRelease R
    │
    ├── immutable source S
    │
    ├── derived artifact A for compatibility/build context X
    │
    └── derived artifact B for compatibility/build context Y
```

is valid.

---

# Package names and versions

Package naming rules are defined by the package/manifest specification.

A registry MUST store canonical validated package names and semantic versions.

The registry MUST NOT silently normalize two distinct invalid spellings into one published identity after publication.

Package versions use semantic-version precedence.

At minimum, the initial resolver model must support:

```text
1.2
^1.2
~1.2.3
=1.2.4
```

with the following intended meanings:

```text
1.2
^1.2
    >= 1.2.0 and < 2.0.0

~1.2.3
    >= 1.2.3 and < 1.3.0

=1.2.4
    exactly 1.2.4
```

For major version zero, caret requirements follow semantic-version compatibility rules rather than treating all `0.x` releases as mutually compatible.

For example:

```text
^0.3.2
    >= 0.3.2 and < 0.4.0

^0.0.5
    >= 0.0.5 and < 0.0.6
```

Pre-release versions MUST NOT satisfy an ordinary stable requirement unless the requirement explicitly admits the relevant pre-release according to the chosen semantic-version rules.

The implementation SHOULD use one well-defined SemVer requirement implementation rather than creating a Telos-specific approximation.

---

# Dependency authority

The package model defines two initial dependency classes:

```text
PathDependency
RegistryDependency
```

They are intentionally different authorities.

## Path dependency

```toml
[dependencies]
parse = { path = "../parse" }
```

means:

> Use the mutable package checkout at this path.

A path dependency is development authority.

Its contents may change without changing a registry version.

It is outside registry resolution.

## Registry dependency

```toml
[dependencies]
parse = "^1.2"
```

means:

> Resolve package `parse` from the selected registry using version requirement `^1.2`.

A registry dependency resolves to an immutable published release.

Archon MUST NOT satisfy a registry dependency by finding an arbitrary mutable directory with a matching manifest.

Likewise, Archon MUST NOT silently reinterpret a path dependency as a published release.

---

# Dependency aliases

The local import alias may differ from the registry package name.

For example:

```toml
[dependencies]
parser = { package = "parse", version = "^1.2" }
```

means:

```text
dependency alias: parser
registry package: parse
version requirement: ^1.2
```

The alias participates in the consuming package's module/import environment.

The registry resolves the package identity `parse`; it does not own the consuming package's alias.

---

# Registry selection

The initial implementation has one default registry:

```text
local
```

The internal model MUST nevertheless preserve registry identity from the beginning.

A future manifest may support an explicit registry:

```toml
[dependencies]
parse = { version = "^1.2", registry = "official" }
```

but multiple registry configuration is not required for the first implementation.

If no registry is explicitly selected, Archon resolves the dependency through the configured default registry.

The selected registry MUST be recorded in resolved dependency authority.

---

# Resolution model

Registry resolution happens in Archon, not in Telos and not recursively inside a registry backend.

The logical process is:

```text
Archon.toml requirements
        │
        ▼
determine dependency source class
        │
        ├── path dependency
        │      └── resolve mutable local package
        │
        └── registry dependency
               │
               ▼
          choose RegistryId
               │
               ▼
        enumerate candidate releases
               │
               ▼
           version solving
               │
               ▼
       exact package releases
               │
               ▼
          Archon.lock
```

The registry backend provides package-release information.

Archon owns graph-wide version solving.

A registry backend MUST NOT independently make hidden recursive dependency choices that Archon cannot observe or reproduce.

---

# Candidate selection

For a registry dependency:

```text
parse ^1.2
```

Archon asks the registry for published versions of `parse`, filters them by the version requirement, and participates in graph-wide solving.

Candidate selection MUST be deterministic for a given:

```text
registry state
root requirements
existing lock constraints
resolver algorithm/version
```

Implementations SHOULD prefer already-locked compatible releases during ordinary builds so that a newly published compatible version does not silently rewrite an existing lockfile.

An explicit update operation may request a new solution.

---

# `Archon.lock`

`Archon.toml` contains requirements.

`Archon.lock` records exact resolved package-release authority.

The lockfile MUST be sufficient to identify the exact immutable registry release selected for every registry dependency.

A conceptual record is:

```toml
[[package]]
name = "parse"
version = "1.4.1"
registry = "local"
source-digest = "blake3:..."

dependencies = [
    "text 1.3.0 (local)",
]
```

The final serialization may differ, but the semantic information is required.

## What the lockfile pins

For registry dependencies, the lockfile MUST pin:

```text
registry identity
package name
exact package version
immutable source digest
resolved dependency edges
```

It MAY also record additional integrity/provenance metadata.

## What the lockfile should not fundamentally pin

A package release may have multiple valid `.telosc` artifacts over time because compiler/artifact compatibility changes.

Therefore the canonical dependency lock SHOULD NOT define the package release itself by one `.telosc` digest.

The selected compiler artifact is better treated as derived build state:

```text
LockedRelease
    +
CompilerCompatibility
    +
ResolvedDependencyArtifactAuthority
        ↓
CompatibleArtifact
```

Archon may cache the resulting artifact digest in project/global build indexes.

If a future reproducibility mode needs to pin exact compiler artifacts as well as releases, that must be modeled as an additional build/toolchain lock, not by conflating package release identity with artifact bytes.

---

# Source digest

Every registry release has an immutable `SourceDigest`.

The digest identifies the canonical source snapshot accepted by the registry.

The exact canonical source-tree hashing algorithm is registry-format-specific but MUST satisfy:

- deterministic file ordering;
- deterministic path representation;
- no host absolute paths;
- no modification-time contribution;
- no process/session-specific metadata;
- content sensitivity;
- manifest sensitivity where the manifest participates in the published source snapshot.

The source digest is release integrity/provenance.

It is not automatically equivalent to:

```text
interface digest
semantic digest
artifact digest
```

These represent different compiler/build concepts.

---

# Published dependency metadata

A package release records the dependency requirements declared by its published manifest.

For example, a release may state:

```text
parse@1.4.1 requires text ^1.0
```

It does not necessarily fix one universal exact `text` version for every consuming graph.

A root build solves the full dependency graph and records its exact selections in `Archon.lock`.

This distinction allows:

```text
package release requirements
```

to remain reusable while:

```text
application/workspace lock
```

records one concrete dependency solution.

A compiled package artifact MUST nevertheless refer to the exact dependency authorities against which it was built. Artifact reuse is valid only when those dependency/compatibility requirements are satisfied.

---

# Registry release contents

A registry release MUST retain enough immutable information to reconstruct the package release after disposable caches are deleted.

At minimum:

```text
release metadata
canonical published Archon.toml
canonical source snapshot
source digest
declared dependency requirements
package identity
```

A registry MAY additionally retain:

```text
README/license metadata
publication metadata
artifact availability records
signatures/receipts
provenance
```

Development sources such as tests/examples/benchmarks may be included in the published source snapshot. Their presence does not make them part of the package's dependency surface.

The production dependency surface remains determined by the package model.

---

# Publishing a release

Publishing creates a new immutable registry release.

Conceptually:

```text
mutable package checkout
        │
        ▼
validate manifest/package
        │
        ▼
construct canonical source snapshot
        │
        ▼
compute SourceDigest
        │
        ▼
construct release metadata
        │
        ▼
registry atomic publication
```

Archon may also build and validate a compiler artifact before or during `archon publish`, but artifact production is not what defines the release identity.

## Immutability rule

If:

```text
local:parse@1.2.0
```

already exists, publishing different source under:

```text
local:parse@1.2.0
```

MUST fail.

The normal correction is to publish a new version:

```text
parse@1.2.1
```

The registry MAY recognize an exact idempotent re-publication attempt whose complete immutable release identity is already present, but it MUST NOT mutate the existing release.

---

# Canonical source snapshot

Publication operates on a snapshot, not on a live mutable directory reference.

After publication:

```text
registry release
```

MUST NOT depend on the original checkout continuing to exist.

Deleting or editing:

```text
../parse
```

must not alter an already-published:

```text
local:parse@1.2.0
```

The first local registry's snapshot rules are defined in `local-registry.md`.

A future network registry may use a different transfer representation while preserving the same canonical release authority.

---

# Compiler artifacts associated with releases

A compatible `.telosc` can be associated with a package release.

However, artifact reuse requires more than matching:

```text
package name
package version
```

A usable artifact must also satisfy compiler compatibility and exact dependency authority requirements.

A conceptual artifact record therefore includes:

```text
package release
compiler/artifact compatibility identity
resolved dependency authority
artifact digest
artifact fingerprint/identity evidence
```

The precise compatibility fields are owned by the compiler-artifact specification.

A registry MUST NOT advertise an artifact as reusable merely because it was once built from the same package version.

---

# Artifact bytes and registry authority

A registry need not physically store every artifact byte.

The architecture permits:

```text
registry:
    release metadata
    source authority
    artifact references

global CAS:
    artifact digest -> immutable bytes
```

This separation is preferred for the initial local registry.

The registry is durable authority.

The global cache is disposable storage.

Deleting the global cache may force recompilation, but MUST NOT erase the published package release.

---

# Trusted package resolution

A compiler artifact must not authenticate its own package authority merely by containing a claim such as:

```text
"I am parse@1.2.0"
```

Archon resolves package authority independently.

When Telos imports a package artifact, Archon must be able to supply trusted evidence binding the artifact to the package release it resolved.

Conceptually:

```text
RegistryResolutionReceipt
    registry
    package
    version
    source digest

ArtifactResolutionReceipt
    resolved release
    compiler compatibility
    full artifact digest
    artifact fingerprint
```

The exact runtime type names may differ.

The invariant is:

> **Artifact package identity is externally authorized by the resolver; it is not self-authenticated by artifact contents.**

This aligns registry resolution with fail-closed compiler-artifact import.

---

# Missing compatible artifact

A registry release remains usable even when no compatible `.telosc` is available.

Archon may explicitly perform:

```text
select immutable release
        │
        ▼
materialize canonical source
        │
        ▼
compile with current toolchain
        │
        ▼
validate .telosc
        │
        ▼
store in global cache
```

This is an Archon build decision.

It is not silent source fallback inside Telos artifact import.

If Archon explicitly selects an artifact and Telos reports it corrupt or incompatible, that import attempt fails.

Archon may then make a separate, visible decision to rebuild from registry source authority.

---

# Registry errors

The registry abstraction must distinguish at least:

```text
package not found
no version satisfies requirement
selected release missing
release metadata corrupt
source snapshot missing
source digest mismatch
registry schema unsupported
publication conflict
registry unavailable
artifact reference corrupt/unavailable
```

These errors must not be flattened into "package not found."

Resolution diagnostics should identify:

```text
registry
package
requested requirement
relevant locked version if any
```

where applicable.

---

# Yanking and deletion

The first registry model does not require yanking or administrative deletion.

If yanking is introduced later, it SHOULD mean:

```text
do not select this release for new resolution
```

while allowing an existing lockfile to continue identifying the immutable release where policy permits.

Yanking MUST NOT mutate the source authority of the release.

Hard deletion is an administrative storage concern and is intentionally outside the initial portable registry contract.

---

# Future network registries

A future remote registry may add:

```text
HTTP/API transport
authentication
accounts
namespaces/ownership
signatures
mirrors
replication
upload sessions
server-side policy
rate limits
```

but it MUST preserve the same core package semantics:

- registry IDs are explicit authority;
- published release versions are immutable;
- source releases and compiler artifacts are distinct;
- version requirements are solved by Archon;
- lockfiles pin exact release authority;
- artifact compatibility is validated independently;
- Telos itself does not become a registry client.

The local registry is therefore not a temporary semantic shortcut. It is the first transport/storage implementation of this model.

---

# Registry invariants

The following are normative:

1. **Published release immutability**  
   A registry/package/version coordinate cannot silently acquire different source authority.

2. **Source/artifact separation**  
   A package release is not permanently identified by one `.telosc`.

3. **Resolver authority**  
   Archon chooses registry releases. Telos consumes already-resolved authority.

4. **No path substitution**  
   A registry dependency cannot be silently satisfied by an arbitrary mutable checkout.

5. **No self-authenticating artifacts**  
   Artifact claims are validated against independently resolved package authority.

6. **Lock exact releases, not vague requirements**  
   `Archon.lock` identifies immutable selected releases.

7. **Cache independence**  
   Registry authority survives deletion of the global cache.

8. **Transport independence**  
   Local and future remote registries share package-release semantics.

9. **Deterministic resolution inputs**  
   A given registry state, root requirements, lock constraints, and resolver version must not depend on filesystem enumeration order or other nondeterministic host behavior.

10. **Fail closed on corruption**  
    Missing or mismatched release integrity data is an error, not an invitation to reinterpret some other source as the package.

---

# Relationship to other toolchain specifications

`package-model-and-manifest.md` owns:

```text
dependency declaration syntax
package identity syntax
target semantics
dev-dependencies
```

This specification owns:

```text
registry release authority
version matching/resolution model
lockfile release identity
publication semantics
source/artifact distinction
trusted registry resolution
```

`local-registry.md` owns:

```text
local storage
filesystem layout
local publication transaction
source snapshot representation
local corruption handling
```

The future build/cache specification owns:

```text
global artifact CAS
BuildKey
artifact lookup
artifact promotion
cache GC
```

The future Archon CLI specification owns:

```text
archon publish
archon add
archon update
archon fetch
archon registry ...
```

Those command implementations must obey the authority model defined here.
