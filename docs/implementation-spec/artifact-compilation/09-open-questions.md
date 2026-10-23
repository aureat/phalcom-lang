# 09 — Open questions and risks

These items are outside this program, or not yet ruled. None of them blocks
slices 0–I unless marked **BLOCKING-IF**.

## Q1. Executable package artifacts

Package staging rejects `PORTABLE_EXECUTABLE`. To execute a downstream
program source-free, the package's bytecode must be in its artifact, and the
executable section must link across artifacts (call targets and global slots
in Universe and other packages). D-MW5-4 fixes the ordering contract (the
dependency artifact initializes before the importer), but no executable
cross-artifact link format exists. Next program after I. VER-H6 tracks the
gap.

*Update 2026-10-23:* specified by
[`../executable-products/`](../executable-products/README.md) (XLNK,
PROPOSED). VER-H6 closes through XLNK VER-X6.

## Q2. Package-to-package availability

LD-W2 rejects external witnesses into non-Universe packages, so a package
depending on another package must take eager runtime edges. If deferred
initialization across packages turns out to matter, packages will need their
own environment contract (a V4-style `bootstrap_availability` per package).
Not ruled.

## Q3. Query fingerprints and `for_query` identity

`StableIdentityContext::for_query` fingerprints a `Resolved` project by its
source path, by design for in-process queries. EXP-C1 applies the publishing
override in both contexts during package export. It is **unverified**
whether any persisted query fingerprint is computed outside the export
context, for example cached inside `SemanticDb` before export starts. In
that case the cached value carries the path. VER-D2 detects this.
**BLOCKING-IF** VER-D2 fails for that reason: then query fingerprints must be
recomputed at export under the publishing context. That is an ABI-level
change (`QUERY_FINGERPRINT_ABI_*`) and needs a ruling.

## Q4. Moving the Universe writer to V5

D-MW5-3 keeps Universe on V4. Moving it later removes the dual writer, but it
changes Universe artifact digests and every cached artifact. Decide when
ARTF001 P5 (`packages/universe` cutover) rewrites the Universe build anyway.

## Q5. Package identity and versions in the resolver

EXP-P2 changes the package identity whenever a dependency changes, so
downstream artifacts must be rebuilt. This is intended, but it means a
resolver/cache design (ARTF001 P6) must key by full `StableProjectRef`. It
must not key by name and version. This holds only once amendment A1 to
EXP-P2 is implemented (Universe artifact digest as an identity input).
Before A1, two builds over different Universe artifacts share one
`StableProjectRef`.

## Q6. Metadata section scoping

PB-I5 assumes the metadata bundle can be restricted to owned modules. The
anchor has no scoped metadata builder. If `SemanticMetadataBundle` cannot be
filtered without re-deriving it, F3 needs a scoped metadata export:
**BLOCKING-IF** found in F2. Write an incident note; do not ship a bundle
that contains Universe rows.

## Q7. Ownerless queries

`TraitDispatchBucket` and `GenericConformance` keys have no owning module,
so no package freezes them (slice D, PB-I3). A downstream session therefore
recomputes them over the composed base. This is correct, but it costs some
performance. Revisit if profiling shows it matters.

## Q8. Pre-existing test-target failures

Slice 0 determines whether the 3 lib-test compile errors predate `48435bb71`.
If A–D introduced them, fix them in slice 0 and note it in
`baseline-failures.md`. Do not leave them for later slices.
