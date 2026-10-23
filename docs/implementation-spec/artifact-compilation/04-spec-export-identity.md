# 04 — Spec: stable identity of package-owned rows in export (slice F1)

## 1. Problem (verified at the anchor)

The artifact export path converts native identities to stable identities
with the **context-free** functions in
`phalcom-semantic/src/metadata/stable_identity.rs` (`to_stable_module`,
`to_stable_declaration`, `to_stable_callable`, …). Those functions call
`to_stable_project`, which does this:

```rust
ProjectIdentity::Resolved(_) => panic!("resolved project requires StableIdentityContext"),
```

Context-free call counts at the anchor (`rg -c "to_stable_[a-z_]+\("`):
`artifact/export.rs` 199, `artifact/roots.rs` 24,
`artifact/export/explanation.rs` 13, `artifact/export/proof.rs` 9,
`artifact/operational.rs` 8, `types/artifact.rs` 6,
`static_value/artifact.rs` 4. Slices C and D are scoped correctly, but they
have only run with the Universe scope, where every module is
`ProjectIdentity::Universe`. **The first package-owned row exported would
panic.** This blocks F3, independently of slice E.

A second, quieter problem: when a package project *does* go through a
context, it exports as `StableProjectRef::SourceArtifact { logical_uri,
source_fingerprint }` unless a resolver receipt is registered. Here
`logical_uri` is the source directory path, so artifact bytes would depend on
where the package was checked out (violates ARCH-I7). A package being built
has no receipt yet: receipts bind the artifact digest, which does not exist
until the build finishes.

## 2. Publishing identity

**EXP-P1. Package self-identity.** The builder computes the package's stable
identity **before** export:

```
StableProjectRef::Package {
    package:  <project.toml name>,
    version:  <project.toml version>,
    artifact_fingerprint: package_identity_fingerprint(...),
}
```

**EXP-P2. `package_identity_fingerprint`.** Take the first 16 bytes of
`ArtifactDigest::digest_domain(DOMAIN_PACKAGE_IDENTITY, bytes)`, where
`DOMAIN_PACKAGE_IDENTITY = "phalcom.telosc.package-identity.v1"` and `bytes`
is the canonical binary encoding (existing `BinaryWriter` conventions) of:

1. package name, version;
2. the package `source_digest` (EXP-P4);
3. the sorted `external_projects` list (each dependency's full
   `StableProjectRef`, including the dependency's own fingerprint);
4. the header ABI fields that the artifact will carry
   (`semantic_model_version`, `stable_identity_schema`,
   `module_semantic_abi`, `semantic_artifact_abi`, `query_fingerprint_abi`,
   `native_surface_schema_version`, `native_catalog_fingerprint`,
   `type_metadata_schema_version`).

The fingerprint MUST NOT depend on the artifact bytes or digest (that would be
circular), or on any filesystem path. Two builds from identical sources and
dependencies yield the same identity. A changed dependency yields a new
identity, so downstream artifacts that recorded the old identity fail closed
at receipt lookup instead of silently binding to different products.

**EXP-P3. Path freedom.** No `StableProjectRef::SourceArtifact` may appear
in any section of a package artifact. Validate this in the builder after
export by walking the produced sections' project references (cheapest: check
`external_projects`, module identities and `external_modules` in the module
world; the semantic and query sections only see refs produced through the
context of EXP-C1, which never emits `SourceArtifact` for the owned or
external packages). A violation is a build error.

**EXP-P4. Package `source_digest`.** Domain-separated digest
(`"phalcom.telosc.package-source.v1"`) over the owned modules in
`ModuleId` order. For each module: module path components, then the exact
source text bytes. No paths, timestamps or manifest bytes. Place it in
`TeloscHeaderV2.source_digest`.

## 3. Context threading

**EXP-C1. `StableIdentityContext` gains a publishing override.**

```rust
pub struct StableIdentityContext<'a> {
    pub projects: &'a ProjectUniverse,
    query_fingerprint: bool,
    publishing: Option<(ResolvedProjectId, &'a StableProjectRef)>, // new
}
impl<'a> StableIdentityContext<'a> {
    /// Exports `project` as `identity` regardless of receipts or source
    /// provenance. Used only while building that package's artifact.
    pub fn publishing(projects: &'a ProjectUniverse, project: ResolvedProjectId, identity: &'a StableProjectRef) -> Self;
}
```

`to_stable_project_with_context` checks `publishing` first, then
`stable_package_owner`, then the existing `SourceArtifact` fallback. The
override applies to **both** the artifact context and the `for_query`
context. Query fingerprints computed during package export MUST use the
publishing identity too; otherwise query fingerprints embed the source path
(see `09` Q3).

**EXP-C2. Export takes a context.** `ArtifactSemanticProjection` gains an
identity context:

```rust
pub struct ArtifactSemanticProjection<'a> {
    snapshot: &'a SemanticSnapshot,
    scope: &'a ArtifactProjectionScope,
    identity: &'a StableIdentityContext<'a>,   // new
}
```

Every stable conversion in `artifact/export.rs`, `artifact/export/*.rs`,
`artifact/roots.rs`, `artifact/operational.rs`, `types/artifact.rs`
(`TypeExportContext`) and `static_value/artifact.rs` (`StaticExportContext`)
switches to the `*_with_context` variant. Context variants already exist for
module, declaration, callable, field, impl, static constant, variant,
variant field and data component. Add any missing ones (dispatch side needs
none). Then make the context-free variants `pub(crate)` and
`#[deprecated]` inside the artifact module, or remove the artifact module's
imports of them. The final state is that `rg "to_stable_[a-z_]+\(" phalcom-semantic/src/artifact`
matches only `_with_context` calls.

**EXP-C3. Universe equivalence.** For the Universe build, the context is
`StableIdentityContext::new(projects)` with no publishing override. Every
Universe identity is `ProjectIdentity::Universe`, for which the context
variants return exactly what the context-free ones return. Gate: VER-R1
(byte-identical Universe artifact).

**EXP-C4. Import side is unchanged.** Import resolves stable refs through
`StableOwnerCatalog`. No change in this slice beyond `06` LD-C2.

## 4. Tests

| ID | Test |
|---|---|
| EXP-T1 | Unit: `to_stable_project_with_context` with a publishing override returns the override for that project and is unchanged for others (Universe, a receipt-bound package, a source project). |
| EXP-T2 | Unit: `package_identity_fingerprint` is stable across two calls; changes when any input in EXP-P2 changes; does not change when the package directory is moved. |
| EXP-T3 | Integration: scoped semantic + query export of a fixture snapshot holding a `Resolved` package module completes without panic and emits only `Package` / `Builtin` project refs. |
| EXP-T4 | Regression: the existing Universe artifact build is byte-identical (VER-R1). |
