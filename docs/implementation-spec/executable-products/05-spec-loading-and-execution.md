# 05 — Spec: loading an artifact chain, materializing it, and running a product

Crate: `phalcom-core` (`src/artifact/`, `src/modules/`, `src/vm/`).

## 1. Chain validation

A chain is validated as a whole before the first VM is created. The
compiler-section half of this is ARTC slice G (`compose_artifact_semantic_base_impl`
with per-package prefix authority, LD-K1); this section adds the executable
half on top of it.

**XM-V1. Order.** The chain is Universe, then packages in the order
`plan_package_dependencies` returns for the root. For every package
artifact at position `k`, every entry in its `dependencies` (XS-W2) appears
at a position before `k`.

**XM-V2. Exact digests.** For every package artifact, each
`dependencies[i].artifact_digest` equals the digest of the chain entry for
`dependencies[i].project` (D-XL-2). A mismatch is
`ChainError::DependencyDigest { artifact, dependency, expected, found }`.

**XM-V3. One native ABI.** Every package's
`compatibility.native_executable_abi_digest` equals the Universe artifact's.
All other compatibility fields are equal across the chain (XS-W5).

**XM-V4. Imports resolve in the prefix.** For package `k`, each entry of
`imports` (XS-W3) resolves in the authority of positions `0..k`:

| Family | Resolves against |
|---|---|
| `modules` | module worlds of the prefix (the `ExternalModuleAuthoritySet` built for `k` by LD-K1) |
| `bindings` | the target module's bindings in its module world; `StaticConstant` additionally in that artifact's static constants |
| `classes`, `declarations` | the declaring artifact's semantic world, and for classes its executable class recipes (or a Universe primordial role) |
| `callables`, `implementations`, `variants`, `data_components`, `variant_fields`, `type_parameters`, `fields`, `static_constants` | the owning artifact's semantic world, through the same owner catalog ARTC uses (`StableOwnerCatalog`, LD-C1…C4) |
| `type_digests`, `kind_digests` | the prefix metadata bundles, scanned in chain order; first match wins, matching the scan order of `canonical_type` and `canonical_kind` today |

An import that resolves nowhere, or that resolves to a row owned by
position `k` itself or by a later position, is rejected.

**XM-V5. Disjoint ownership.** No module, class, declaration or callable is
owned by two chain entries (XL-I3). ARTC's `from_worlds` already rejects
duplicate module ownership; this extends the check to executable rows.

**XM-V6. Executable presence.** Every package in a product's chain carries
a schema 2 section. A compiler-only package artifact in a product chain is
`ChainError::NotExecutable`.

## 2. Materialization

**XM-M0. Entry point.** Add a loaded-chain type and a VM constructor:

```rust
/// The Universe artifact and the package artifacts of one product, validated
/// together (XM-V1…V6), ready to materialize into one VM.
pub struct LoadedArtifactChain { /* universe, packages in order, composed base */ }

impl LoadedArtifactChain {
    /// Fresh VM with every artifact materialized and no package module run.
    pub fn new_vm(&self) -> Result<VM, VmBootstrapError>;
}
```

`LoadedUniverseArtifact::new_vm` stays as it is and is what `new_vm` calls
first.

**XM-M1. Universe.** `bootstrap_vm` runs unchanged: topology, natives,
metadata, linkage, executable materialization, Universe eager modules, and
Universe invariant checks.

Then, for each package in chain order:

**XM-M2. Runtime project identity.** Register the package's runtime project
from its ARTC authority receipt (`ValidatedPackageAuthorityReceipt`), so
that its modules get `ProjectIdentity::Resolved(pid)` ids. The runtime
project has no source root. Nothing in this step may touch the file system.

**XM-M3. Topology.** For each owned module of the package's `ModuleWorldV5`:
allocate the module object, register it `Prepared`, declare its globals in
slot order and check each slot against the module world's binding layout
(as `materialize_universe_artifact_topology` does for Universe), then bind
the lexical intrinsics `__module__`, `__package__`, `__root__` and
`__project__` with the same rules `materialize_program` applies to
`ProjectIdentity::Resolved`. Package modules are not privileged.
Implementation note: factor `materialize_program`'s phases 1 and 2 so they
take a source-free module description; `materialize_program` and this step
then share one implementation (XL-I7).

**XM-M4. Metadata.** Install the package's metadata bundle as a new typing
pool, after every prefix pool. Pool installation order is chain order.

**XM-M5. Linkage.** Run `materialize_linked_authority_for_module` for each
owned module. External linked reads resolve to the module objects of
earlier artifacts, which are already registered.

**XM-M6. Classes and initializers.** Generalize `ExecutableMaterializer`
over the section schema. For schema 2: install class authority for owned
class recipes (superclass and metaclass references resolve to classes
registered by earlier artifacts); skip the primordial-role path, which
schema 2 cannot express; attach every owned module's initializer closure
(the same loop that attaches Universe initializers today); install detached
methods. Then bind metadata callables for the package's classes, from the
installed metadata bundle rather than from a `CompiledProgram`
(the source-free counterpart of `bind_non_universe_metadata_callables`).

**XM-M7. No package code runs during materialization.** After XM-M6 every
package module is `Prepared` with a closure attached. Nothing has executed.
`new_vm` returns here.

**XM-M8. Invariants.** After the last package, run the checks
`bootstrap_vm` runs after Universe (`universe.verify_invariants` and the
unconsumed preallocated class check, scoped per package).

## 3. Errors

**XM-E1.** Validation failures (`§1`) are `ChainError` values reported
before `new_vm` allocates anything. Materialization failures are
`VmBootstrapError::Runtime` or `VmBootstrapError::Invariant`, as today, and
name the artifact and module.

**XM-E2.** No error path falls back to compiling source (XL-I6).

## 4. Running a product

**XR-1. Validate the order.** Before running, check the product's
`initialization` list: every module is owned by some chain package, the
last one is `target.entry`, and the order satisfies XP-C4's constraints
(artifact order, then owned topological order). A forged order is
rejected, not repaired.

**XR-2. Run.** For each module in `initialization`, run its initializer in
the module (`run_in_module`), then mark it `Initialized`, exactly as
`initialize_program` does for a source-compiled program. A module that a
running initializer reaches before its turn is initialized on demand
through the existing deferred path, as today.

**XR-3. Program result.** The program's result is the entry module's
completion. An uncaught runtime error is a runtime execution failure. A
program-requested exit status, where the runtime offers one, is preserved
(`telos-cli.md` §7.2). Load, validation and materialization failures use
different exit classes from program failures (`telos-cli.md` §15).

**XR-4. No emission.** During XR-1…XR-3 the emitter
(`compile_closure_as_with_bindings` and its callers) is never entered. A
test-only counter (`artifact_run_emission_count`, new) proves it (VER-X3).

**XR-5. Parity.** For a program that can also run from source, a product
run produces the same standard output and the same success or failure as
`LoadedUniverseArtifact::compile_user_entry` plus `VM::run_compiled` for the
same entry (XL-I7).
