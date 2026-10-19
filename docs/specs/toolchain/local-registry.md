# Local Package Registry

This specification defines the first concrete Telos package registry: a durable, single-machine registry managed by Archon.

Its registry identity is:

```text
local
```

It implements the generic package-release model defined in [`registry.md`](./registry.md).

The local registry is intentionally simple:

- no web service;
- no authentication;
- no user accounts;
- no network transport;
- no remote synchronization;
- no mutable package releases.

It is nevertheless a **real registry**, not a directory search convention.

The central rule is:

> **The local registry stores immutable published package releases. Mutable development checkouts use path dependencies instead.**

---

# Why a local registry exists

Before a public network registry exists, Telos still needs to exercise the real package lifecycle:

```text
develop mutable package
        │
        ▼
publish version
        │
        ▼
immutable package release
        │
        ▼
version resolution
        │
        ▼
Archon.lock
        │
        ▼
artifact reuse / source rebuild
```

Using only path dependencies would avoid the most important registry semantics:

- immutable versions;
- source provenance;
- release integrity;
- lockfile authority;
- resolver behavior;
- source snapshots;
- publication conflicts;
- package-artifact authorization.

The local registry exists so these semantics are implemented correctly before transport becomes remote.

---

# Registry identity

The built-in registry has the logical identity:

```text
RegistryId("local")
```

Archon MUST preserve this identity in resolved package authority and lockfiles.

A future package:

```text
official:parse@1.2.0
```

is not automatically the same registry authority as:

```text
local:parse@1.2.0
```

even if source digests happen to match.

---

# Storage domain

The local registry is durable Archon **data**, not cache.

Its logical root is:

```text
ArchonDataDir/registries/local/
```

The exact platform-native `ArchonDataDir` is a toolchain/platform concern.

Typical examples may be:

```text
Linux:
    $XDG_DATA_HOME/archon/registries/local/

macOS:
    ~/Library/Application Support/Archon/registries/local/

Windows:
    <platform Archon application-data directory>\registries\local\
```

A development/test installation may redirect the Archon data root through a documented Archon environment/configuration override.

The registry MUST NOT be placed inside:

```text
project/.archon/
```

and MUST NOT be treated as part of the disposable global cache.

Therefore:

```bash
archon clean
archon cache clean
archon cache gc
```

MUST NOT delete published local registry releases.

---

# On-disk format version

The local registry storage schema MUST be explicitly versioned from its first implementation.

The root contains a format marker such as:

```text
VERSION
```

or equivalent registry metadata containing:

```text
schema = 1
```

Archon MUST validate the registry schema before mutating the registry.

If the schema is newer than the running Archon understands, Archon MUST fail rather than attempting best-effort writes.

Read-compatible migration may be added later, but mutation always requires a supported schema.

---

# Initial logical layout

The first implementation should use a simple inspectable layout.

Conceptually:

```text
<local-registry>/
├── VERSION
├── registry.toml
├── index/
│   ├── parse.toml
│   ├── slotmap.toml
│   └── text.toml
│
├── releases/
│   ├── parse/
│   │   ├── 1.2.0/
│   │   │   ├── release.toml
│   │   │   └── source.tar
│   │   └── 1.4.1/
│   │       ├── release.toml
│   │       └── source.tar
│   │
│   └── text/
│       └── 1.0.0/
│           ├── release.toml
│           └── source.tar
│
└── staging/
```

The exact index sharding strategy may change without changing registry semantics.

For example, a future large local registry may shard package index files by hash or name prefix.

What is normative is the authority represented by the records, not the cosmetic directory layout.

---

# `registry.toml`

The local registry root should identify itself.

For example:

```toml
schema = 1
id = "local"
```

It may later contain administrative metadata, but package-release authority MUST live in per-package/per-release records rather than in mutable ad hoc directory naming.

---

# Package index

The index is an acceleration structure for resolution.

For one package, an index record conceptually contains:

```toml
name = "parse"

[[release]]
version = "1.2.0"
source-digest = "blake3:..."

[[release]]
version = "1.4.1"
source-digest = "blake3:..."
```

It may additionally record:

```text
yank state
release metadata digest
artifact availability hints
```

if those features are later introduced.

The index is not the sole durable authority.

The corresponding immutable release record must also exist.

If the index and release record disagree, the registry is corrupt.

---

# Immutable release directory

A published release has an immutable directory:

```text
releases/<package>/<version>/
```

containing at minimum:

```text
release.toml
source.tar
```

The directory MUST NOT be edited in place after successful publication.

Updating package metadata for an already-published version by rewriting `release.toml` is forbidden if that metadata contributes to release authority.

A new release requires a new version.

---

# Release metadata

`release.toml` conceptually contains:

```toml
schema = 1

[package]
name = "parse"
version = "1.4.1"
namespace = "parse"

[source]
digest = "blake3:..."
archive = "source.tar"

[manifest]
digest = "blake3:..."
```

and normalized published dependency requirements, for example:

```toml
[[dependencies]]
alias = "text"
package = "text"
requirement = "^1.0"
registry = "local"
```

Development dependencies may be retained as package-source metadata because tests/examples/benchmarks can be present in the source release, but they do not participate in the normal dependency surface of consumers.

The metadata format SHOULD remain human-inspectable initially.

---

# Canonical source snapshot

Publication MUST copy package source into an immutable registry-owned snapshot.

A release MUST NOT retain a live pointer to the publishing checkout.

This must remain true after:

```text
publisher edits source
publisher deletes checkout
publisher switches git branch
publisher moves project directory
```

The registry release remains unchanged.

---

# What is included in the source snapshot

The initial publication snapshot should include the package files needed to reconstruct and inspect the published release.

At minimum:

```text
Archon.toml
production Telos source
declared target source required by the package
```

It SHOULD also include package-level documentation/license files that Archon designates publishable.

Development target source may be included:

```text
tests/
examples/
benches/
```

because it belongs to the package source release even though it is not part of the external library dependency surface.

The snapshot MUST exclude transient or host-local state such as:

```text
.archon/
.git/
other VCS administrative directories
compiler build outputs
temporary files
editor state
global-cache links
absolute-host-path state
```

`Archon.lock` SHOULD NOT be part of the published library source authority by default. Registry releases publish dependency requirements; the consuming root graph produces its own exact lock solution.

A future package-level publish include/exclude facility may refine these rules.

---

# Symlinks and special files

The initial local registry SHOULD reject symbolic links and filesystem special files in the published source snapshot.

The first canonical snapshot format should accept only ordinary files and directories.

This avoids early ambiguity involving:

```text
symlink escape outside package root
platform-specific link semantics
device files
sockets/FIFOs
non-portable permissions
```

Symlink publication may be designed later with explicit canonical semantics.

---

# Canonical source digest

The `SourceDigest` MUST identify canonical source contents, not incidental archive encoding.

The recommended model is a canonical source-tree digest over ordered file records.

Conceptually, for every included regular file:

```text
record:
    canonical relative path
    byte length
    file bytes
```

Records are sorted lexicographically by canonical relative path.

The digest MUST NOT depend on:

```text
absolute source path
filesystem enumeration order
mtime
ctime
inode
UID/GID
host-specific directory entry order
temporary extraction location
```

Canonical paths use package-relative normalized separators.

Path traversal components such as:

```text
..
```

are forbidden.

The source archive is a transport/storage representation of this source tree.

Its compression or container format MUST NOT silently redefine release identity.

---

# Source archive representation

The initial registry may use an uncompressed deterministic tar-like archive:

```text
source.tar
```

or another simple deterministic archive representation.

If tar is used:

- entry order MUST be deterministic;
- stored paths MUST be relative and normalized;
- timestamps MUST be normalized;
- user/group ownership MUST be normalized;
- unsupported special entries MUST be rejected;
- extraction MUST prevent path traversal;
- archive metadata that does not belong to semantic source identity MUST not affect `SourceDigest`.

The registry SHOULD verify the canonical source-tree digest after materialization.

---

# Publication preparation

`archon publish --registry local` begins from a mutable package checkout but does not publish that directory directly.

Archon first prepares a `PublishCandidate` containing conceptually:

```text
validated package identity
validated Archon.toml
canonical source snapshot
SourceDigest
published dependency requirements
optional compatible artifact evidence
```

The local registry receives this prepared candidate.

This keeps responsibilities clear:

```text
Archon:
    package validation
    build/test policy
    source selection
    candidate preparation

Local registry:
    conflict detection
    immutable storage
    index update
    atomic publication
```

The registry backend does not independently run Telos compilation.

---

# Publication transaction

Publishing must be transactional.

For:

```text
parse@1.4.1
```

the logical operation is:

```text
1. validate registry schema
2. acquire publication coordination
3. verify package/version does not conflict
4. write candidate into staging/
5. verify release metadata
6. verify canonical source digest
7. sync/finish candidate files as required
8. atomically publish release directory
9. atomically update package index
10. release publication coordination
```

At no point may readers observe:

```text
index says release exists
but release is only partially written
```

or:

```text
release directory has been overwritten with half of another publication
```

---

# Concurrent publication

Two processes may attempt:

```text
archon publish
```

for the same package/version concurrently.

The implementation MUST serialize the conflicting publication boundary.

Possible implementation strategies include:

```text
registry-wide write lock
package-scoped lock
atomic create/rename with conflict detection
```

The exact mechanism is implementation-specific.

The semantic result is not:

```text
last writer wins
```

Instead:

- if neither has published yet, at most one transaction establishes the release;
- if both candidates are byte-for-byte/authority-identical, the loser MAY report the release as already present/idempotent;
- if their source authority differs, the second publication MUST fail with a version conflict.

---

# Idempotent re-publication

The local registry MAY support this convenience:

```text
candidate identity exactly equals existing immutable release
        ↓
report "already published"
```

without rewriting the release.

Equality must be based on immutable release authority, not merely:

```text
same package name and version
```

At minimum, the source digest and immutable release metadata must agree.

---

# Registry immutability

After publication, this is forbidden:

```text
local:parse@1.4.1 -> source S1

later

local:parse@1.4.1 -> source S2
```

The valid operation is:

```text
parse@1.4.2 -> source S2
```

The local registry MUST NOT provide an ordinary "force overwrite version" feature.

Such a feature would invalidate `Archon.lock`, trusted artifact receipts, and reproducibility.

---

# Resolution API

The local registry is a data provider to Archon's resolver.

Conceptually, it needs operations equivalent to:

```text
list_versions(package)
load_release(package, exact_version)
materialize_source(exact_release)
publish(candidate)
```

Potential interfaces may resemble:

```rust
trait PackageRegistry {
    fn versions(&self, package: &PackageName) -> Result<Vec<ReleaseSummary>, RegistryError>;

    fn release(
        &self,
        package: &PackageName,
        version: &PackageVersion,
    ) -> Result<ReleaseMetadata, RegistryError>;

    fn materialize_source(
        &self,
        release: &ExactRelease,
    ) -> Result<SourceSnapshot, RegistryError>;

    fn publish(
        &self,
        candidate: PublishCandidate,
    ) -> Result<PublishedRelease, RegistryError>;
}
```

These names are illustrative.

The important boundary is that the local registry exposes release data; it does not perform graph-wide dependency solving.

---

# Version resolution

Given:

```toml
[dependencies]
parse = "^1.2"
```

Archon asks the local registry for versions of `parse`.

Suppose the registry contains:

```text
1.1.9
1.2.0
1.4.1
2.0.0
```

The requirement admits:

```text
1.2.0
1.4.1
```

Archon's resolver chooses an exact release according to the generic resolver and current lock constraints.

The registry backend itself MUST NOT silently decide:

```text
pick 1.4.1
```

without Archon's resolver owning that choice.

---

# Lockfile authority

Once selected, the root build may record:

```toml
[[package]]
name = "parse"
version = "1.4.1"
registry = "local"
source-digest = "blake3:..."
```

A subsequent locked build resolves that exact release.

If:

```text
releases/parse/1.4.1
```

is missing, Archon MUST report missing/corrupt registry authority.

It MUST NOT silently select:

```text
parse@1.4.2
```

merely because that version exists.

---

# Materialization

When source is required for compilation, Archon materializes the immutable registry source into compiler-accessible temporary/build storage.

The extraction location is not package identity.

For example:

```text
/tmp/archon/materialized/...
```

or:

```text
<global-cache>/sources/<digest>/
```

may be used internally.

The compiler-facing project identity remains derived from the resolved package release, not from the transient extraction path.

This is critical: two extractions of the same release at different host paths must not become different persistent package authorities.

---

# Interaction with the global cache

The local registry and global cache have different lifetimes.

```text
LOCAL REGISTRY
    durable authority
    immutable source releases
    package/version index

GLOBAL CACHE
    disposable
    compiled .telosc bytes
    materialized source objects
    other reusable immutable build products
```

A release may have a compatible artifact cached as:

```text
ArtifactDigest A -> bytes
```

The local registry may record or derive an association between:

```text
release R
compatibility/build context C
artifact digest A
```

but it does not need to duplicate the artifact bytes under the release directory.

---

# Artifact lookup

After Archon selects an exact local release, it may perform:

```text
release R
   │
   ▼
derive artifact BuildKey/compatibility requirement
   │
   ▼
global cache lookup
```

If a compatible artifact exists:

```text
validate authority
→ import through Telos
```

If it does not exist:

```text
materialize immutable registry source
→ compile with Telos
→ validate artifact
→ store in global CAS
→ create trusted artifact-resolution receipt
```

The absence of a cached artifact does not mean the package release is absent.

---

# Artifact authority receipt

When Archon loads a `.telosc` derived from a local-registry package, the compiler must receive external authority tying the artifact to the selected release.

Conceptually:

```text
local registry release:
    local
    parse
    1.4.1
    source digest S

selected artifact:
    artifact digest A
    artifact fingerprint F
    compatibility C
```

The trusted resolver binding supplied to Telos must cover the exact artifact authority required by the compiler-artifact importer.

The artifact's own claim to be:

```text
parse@1.4.1
```

is not sufficient.

---

# Corruption handling

The local registry is durable authority and must fail closed on corruption.

Examples:

## Missing release

Index contains:

```text
parse 1.4.1
```

but the release directory is absent.

Result:

```text
registry corruption
```

not:

```text
package not found, try another version
```

for an exact locked release.

## Digest mismatch

`release.toml` claims source digest `S`, but canonical source contents hash to `T`.

Result:

```text
registry corruption
```

The source MUST NOT be compiled as if it were the locked release.

## Malformed release metadata

Invalid package identity, invalid dependency requirements, unsupported release schema, or path escape is an error.

## Unexpected mutable files

Extra transient files that are outside the defined release format MUST NOT silently influence release identity or resolver results.

---

# Verification

The local registry SHOULD expose a verification operation through Archon.

Conceptually:

```bash
archon registry verify local
```

or equivalent future command.

Verification can check:

```text
root schema
index/release agreement
release metadata validity
source archive availability
source digests
duplicate/conflicting coordinates
artifact reference integrity where locally available
```

Verification is diagnostic; ordinary resolution should still validate the specific release authority it consumes.

---

# Cache deletion and rebuild proof

A core acceptance property for the local registry is:

```text
1. publish package release
2. resolve/use package successfully
3. delete global compiler cache
4. keep local registry intact
5. resolve same locked release
6. rebuild compatible artifact from registry source
7. obtain semantically equivalent package authority
```

If deleting the cache destroys the only usable copy of a published release, the implementation has incorrectly conflated registry and cache.

---

# Local development workflow

The registry is not a replacement for path dependencies.

Two intended workflows coexist.

## Co-development

```toml
[dependencies]
parse = { path = "../parse" }
```

Changes in `../parse` immediately participate in the build.

## Stable local publication

From the package checkout:

```bash
archon publish
```

Then consumers use:

```toml
[dependencies]
parse = "^1.4"
```

The consumer receives an immutable local registry release.

This creates a deliberate promotion boundary:

```text
mutable checkout
    ↓
publish
    ↓
immutable local release
```

---

# Package-listing and inspection behavior

The local registry should support enough introspection to make early package development practical.

Expected Archon-level operations include:

```bash
archon registry list
archon registry show local
archon registry path local

archon package list
archon package show parse
```

A package inspection should be able to show:

```text
available versions
source digests
dependency requirements
whether compatible artifacts are currently cached
```

The exact CLI presentation belongs to the Archon CLI specification.

---

# No arbitrary filesystem scanning

The local registry MUST NOT mean:

> Search a user-configured directory tree for any `Archon.toml` whose package name happens to match.

Resolution comes from the registry index and immutable release records.

This prevents:

```text
directory traversal order
checkout branch
uncommitted source
duplicate manifests
filesystem naming accidents
```

from becoming package-resolution authority.

A mutable directory is used only when explicitly declared as a path dependency.

---

# No implicit auto-publication

A registry dependency MUST NOT cause Archon to silently publish a nearby mutable package when the requested registry release is absent.

For example:

```toml
parse = "^1.4"
```

must not trigger:

```text
"Oh, ../parse says version 1.4.1; use/publish that."
```

Publication is an explicit operation.

This keeps release creation observable and intentional.

---

# Registry migration

The local registry format will evolve.

A schema upgrade must distinguish:

```text
read old schema
migrate old schema
write new schema
```

A newer Archon MAY provide an explicit migration operation.

It MUST NOT partially upgrade a registry during ordinary package resolution.

Migration must preserve:

```text
registry identity
package/version coordinates
source digests
immutable source authority
```

If those cannot be preserved, automatic migration is invalid.

---

# Initial administrative limits

The first implementation does not require:

```text
yank
delete
mirror
export/import registry
signing
access control
remote sync
garbage collection of releases
```

The absence of deletion is deliberate: registry releases are authority, not cache garbage.

Administrative lifecycle can be added once its semantics are designed.

---

# Local-registry invariants

The implementation must preserve these properties:

### Release immutability

```text
(local, package, version)
```

cannot be rebound to different source authority.

### Registry/cache separation

Deleting global cache state cannot delete or invalidate a correctly stored release.

### Snapshot independence

Published releases do not depend on the publishing checkout remaining present.

### Canonical integrity

Every release source snapshot has a deterministic source digest.

### Resolver ownership

The registry enumerates/releases metadata; Archon owns graph-wide version solving.

### Artifact/source separation

A missing or incompatible artifact does not erase the package release. Archon may rebuild from immutable registry source.

### Atomic publication

Readers never observe a half-published release.

### Concurrent safety

Concurrent publication cannot produce last-writer-wins mutation of an immutable coordinate.

### Fail-closed corruption

Digest/index/schema inconsistencies are explicit errors.

### No hidden path substitution

Registry lookup never silently falls through to arbitrary mutable source directories.

---

# First implementation success criteria

The first local registry implementation is functionally complete when all of the following can be demonstrated:

```text
1. Package A is authored as a mutable local project.

2. `archon publish` creates:
       local:A@1.0.0
   with immutable source authority.

3. Package B declares:
       A = "^1.0"

4. Archon resolves exactly A@1.0.0 from registry `local`.

5. Archon.lock records the exact local release and source digest.

6. B can build against a compatible cached A.telosc.

7. After the global artifact cache is deleted, B can rebuild a compatible
   A.telosc from the immutable local-registry source snapshot.

8. Editing the original source checkout of A does not change A@1.0.0.

9. Attempting to publish changed source again as A@1.0.0 fails.

10. Publishing the changed source as A@1.0.1 succeeds.

11. A locked build of B continues selecting A@1.0.0 until an explicit
    dependency update changes the lock solution.

12. Corrupting A@1.0.0's source archive or metadata causes a fail-closed
    registry-integrity error rather than fallback to another source.
```

Those properties matter more than the exact v1 directory names.

They establish the local registry as a genuine package authority suitable for later replacement or supplementation by a web registry without changing Telos package semantics.
