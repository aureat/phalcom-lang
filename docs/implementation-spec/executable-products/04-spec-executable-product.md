# 04 — Spec: the executable product (`.telosx`)

Crate: `phalcom-compiler-artifact` (new module `src/product.rs`) for the
wire; `phalcom-core` for building and validating products against loaded
artifacts.

A product answers three questions for `telos run`: what to run, which exact
artifacts it needs, and which runtime ABI it was built for. It contains no
bytecode (D-XL-5).

## 1. Wire

**XP-W1. File layout.**

```text
magic           8 bytes   "TELOSX\0\1"
product_schema  u32       PRODUCT_SCHEMA_V1 = 1
body_length     u64
body            ExecutableProductV1, encoded with BinaryWriter
digest          32 bytes  ArtifactDigest over magic‖schema‖body_length‖body,
                          domain DOMAIN_PRODUCT = "telos.telosx.v1.product"
```

Trailing bytes after the digest are an error. The digest reuses the
artifact digest algorithm in `phalcom-compiler-artifact/src/digest.rs`
with the new domain constant.

**XP-W2. Body.**

```rust
pub struct ExecutableProductV1 {
    pub target: ProductTargetV1,
    pub root: StableProjectRef,                 // the package that owns the entry
    pub chain: Vec<ProductArtifactV1>,          // XP-W4
    pub initialization: Vec<StableModuleRef>,   // XP-W8
    pub compatibility: ExecutableCompatibilityV1, // equals the Universe artifact's
}

pub struct ProductTargetV1 {
    pub kind: ProductTargetKindV1,   // Bin | Example | Test | Bench
    pub name: String,                // toolchain target name
    pub entry: StableModuleRef,      // module whose initializer is the program
}

pub struct ProductArtifactV1 {
    pub project: StableProjectRef,
    pub role: ExecutableDependencyRoleV2,   // Universe | Package
    pub artifact_digest: ArtifactDigest,
}
```

**XP-W3. Target kinds.** All four kinds are representable so that the
format does not change when Archon's overlays arrive. In this program only
`Bin` MUST be produced. `Example`, `Test` and `Bench` products MAY be
produced when the target's entry is a module of an ordinary root package
(the pre-Archon arrangement used by `packages/slotmap/tests/runtime`).

**XP-W4. Chain.** `chain` is the complete artifact chain in load order:
Universe first, then packages in dependency order, with `root` last. It is
exactly the root artifact's executable `dependencies` (XS-W2) followed by
the root itself.

**XP-W8. Initialization order.** `initialization` lists the package
modules (never Universe modules) that the program initializes eagerly, in
the order they run, ending with `target.entry`. It is computed at
construction (XP-C4) and re-validated at run time (XR-1). Universe modules
are initialized by Universe's own bootstrap plan, or on demand through the
existing deferred-module path when a package module first reads one.

**XP-W5. No paths, no host data.** The body contains no file paths, host
names, timestamps, user names or environment values (D-XL-6, XL-I8).

**XP-W6. Limits.** `ValidationLimits` gains `max_product_bytes` and
`max_product_chain_len`. Decoding charges an `AllocationBudget`, as
artifact decoding does.

**XP-W7. Reserved extension.** `product_schema` 2 is reserved for an
optional embedded-artifact form (self-contained bundle). A schema 1 reader
rejects it. Its content is not specified here.

## 2. Construction

**XP-C1. Inputs.** A product is built from the loaded root artifact, its
loaded chain, and an explicit target `(kind, name, entry)`. Telos never
chooses the target (`telos-cli.md` §5.4). No source is read.

**XP-C2. Checks before writing.**

1. The root artifact has a linked executable section (schema 2).
2. `entry` is owned by `root` and is a module of that section.
3. The chain validates as a whole (`05` §1).
4. `initialization` satisfies XP-C4.

**XP-C3. Determinism.** The same root artifact, chain and target produce a
byte-identical product.

**XP-C4. Computing the initialization order.** (Amended 2026-10-23.)

1. **Closure.** Start from `target.entry`. Follow the `runtime_dependencies`
   of each `ModuleWorldV5` row, including external references into earlier
   artifacts. Stop at Universe modules.
2. **Grouping.** Group the closure by owning artifact, in chain order.
   Every module of artifact `j` precedes every module of a later artifact
   `k` (D-MW5-4 as refined by D-XL-9).
3. **Within one artifact.** Run Kahn's algorithm over the owned edges
   restricted to the closure, always taking the smallest ready module by
   `StableModuleRef` order. This is the algorithm
   `RuntimeGraph::initialization_order` (`phalcom-modules/src/graph.rs`)
   uses, with a stable key in place of the session-assigned `ModuleId`.
4. **Entry last.** Every other module in the closure is a transitive
   dependency of the entry, so the entry comes out last in the root
   artifact's group.

*Cross-check.* When `target.entry` is the root artifact's
`ModuleWorldV5::entry`, the root group MUST equal that module world's
`initialization_order`. Within one project, `ModuleId` and
`StableModuleRef` order modules by path, so the two algorithms agree. A
mismatch is an incident.

*Not required.* The order is **not** required to equal the source path's
`CompiledProgram::initialization_order`. That order interleaves projects,
because it always takes the globally smallest ready `ModuleId`, and it
breaks ties by `ResolvedProjectId`, which is assigned per compilation
session. The two paths must agree on observable output (XR-5), not on
order. `09` Q9 tracks aligning the source path with this rule.

## 3. Validation at run time

**XP-V1. Decode.** Magic, schema and digest are checked before the body is
decoded. A digest mismatch is `ProductError::Corrupt`.

**XP-V2. Artifact binding.** The caller supplies one file per chain entry
(D-XL-6). For each entry, the file's artifact digest MUST equal
`artifact_digest`, and the artifact header's identity MUST match `project`.
A missing entry, an extra file, or a mismatch is an error; nothing is
searched for.

**XP-V3. Root and entry.** `root` is the last chain entry; `entry` is owned
by it (re-checked, because the product may be forged).

**XP-V4. Compatibility.** `compatibility` equals the Universe artifact's
schema 1 compatibility block and the running VM's ABI constants.

## 4. Inspection

**XP-I1.** `telos inspect` on a product prints the target, the root, and
each chain entry with its digest, without loading the artifacts. With the
artifacts supplied, it also reports which entries resolve.
