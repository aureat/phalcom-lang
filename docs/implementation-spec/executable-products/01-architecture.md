# 01 — Architecture of executable products and cross-artifact linking

## 1. Goal

A program built from several packages must run from artifacts alone:

- each package's bytecode is frozen once, into that package's own `.telosc`,
  next to its compiler sections (one atomic artifact per package generation,
  as `docs/specs/toolchain/README.md` §7 requires);
- that bytecode refers to code and data in its dependencies (Universe and
  other packages) by stable identity, and those references are resolved
  when the chain is loaded into one VM;
- a small product file names the entry to run and the exact artifacts it
  was built against;
- running the product reads no source, runs no semantic analysis, and emits
  no bytecode.

Compiler-only package artifacts (ARTC) keep working unchanged. An
executable section is an addition to a package artifact, not a different
kind of artifact.

## 2. What exists today

| Concern | Today | Anchor (greppable) |
|---|---|---|
| Executable wire | One image per artifact: modules, prototypes, constants, semantic pools, classes, native anchors, primordial roles, source maps, detached methods, bootstrap plan | `ExecutableImageV1` in `phalcom-compiler-artifact/src/executable/image.rs` |
| Schema gate | Decoder accepts exactly schema 1 | `EXECUTABLE_SCHEMA_V1`; the `executable_schema != EXECUTABLE_SCHEMA_V1` check in `ExecutableImageV1` |
| Operand relocation | Operands index tables inside the image. Three operand domains relocate across tables: constant recipes, linked module reads, semantic-pool entries | `RelocationRequirementV1`, `relocation_requirement_v1` |
| Cross-module data access | Symbolic, per module: an ordered list of `(module)` / `(module, name)` reads that the VM resolves to slots | `PortableLinkedReadV1`, `materialize_linked_authority_for_module` |
| Class, module and declaration references | Stable identities, not indices | `ConstantRecipeV1::Class`, `ConstantRecipeV1::Module`, `StableClassRefV1`, `StableDeclarationRef` |
| Method calls | Selector text plus dynamic dispatch. Super sends are name-baked. No call-target addresses exist in the wire | `MethodSignatureV1`, `SelectorPatternV1`, `OperandDomainV1::SymbolConstant` |
| Type references from bytecode | Structural digests resolved by scanning every installed typing pool | `PortableTypeRecipeV1::Canonical`, `canonical_type` in `phalcom-core/src/artifact/semantic_materialize.rs` |
| Capture | Hooked into the canonical source bootstrap; captures each module's initializer closure, class layouts and detached methods as they are compiled | `ExecutableImageCapture`, `capture_module`, `VM::build_canonical_executable_payload` |
| Validation | Closed world: every module of the Universe linked program must appear, and nothing else may | `validate_executable_image`, `validate_module_coverage`, `validate_bootstrap_dependencies` |
| Materialization | One fresh VM from one Universe artifact: topology, natives, metadata, linkage, then classes and initializers, then eager modules | `bootstrap_vm` in `phalcom-core/src/artifact/materialize.rs`, `ExecutableMaterializer` |
| User programs today | Compiled from source against the loaded Universe, then materialized and emitted inside the VM | `LoadedUniverseArtifact::compile_user_entry`, `VM::run_compiled`, `compile_program_module_closure` |
| Package executable sections | Rejected by package staging | `stage_package_semantic_generation*`, ARTC 01 §6 |
| Capability bits | Unknown bits are silently dropped on decode | `TeloscCapabilities::from_bits_truncate` in `phalcom-compiler-artifact/src/header.rs` |

## 3. The key observation

The executable wire is already almost entirely identity-based. The only
dense indices in it point *inside* the image: `PrototypeId`, `ConstantId`,
`SourceMapId`, `SemanticPoolId`, and local frame, upvalue and field slots.
Everything that names something outside a prototype names it by stable
identity: modules, classes, declarations, callables, implementations,
variants, selectors and types.

Q1 in ARTC worried that "call targets and global slots" need a
cross-artifact link format. They do not need *new addressing*:

- **Call targets** are selectors. Dispatch finds the method at run time.
  There is nothing to relocate.
- **Global slots in other modules** are reached through the module's
  linked-read list, which is symbolic (`PortableLinkedReadV1`). The VM turns
  each entry into a slot when it materializes the module. A read of
  `universe.list::List` from a package module is the same kind of row as a
  read between two Universe modules.
- **Own globals** are slot indices into the module's own global table, whose
  layout comes from the module world's bindings. Those are owned.

What *is* missing is narrower:

1. **A place to say what the bytecode imports.** The closed validator proves
   every stable reference resolves inside one image. A linked section needs
   an explicit, exact list of the external identities it uses, so the
   validator can prove they resolve in the dependency chain before any heap
   object is built (`03` §2).
2. **A binding to the exact dependency bytes.** Field slot numbers of a
   subclass depend on the superclass's layout, and module global slots
   depend on the dependency's binding table. The bytecode is only valid
   against the exact dependency it was compiled against (D-XL-2).
3. **Ownership rules.** A package section must not carry natives,
   primordial roles, or rows owned by another artifact (`03` §3).
4. **Capture outside the Universe bootstrap.** Today capture runs only from
   `build_canonical_executable_payload` (`03` §4).
5. **Materializing several images into one VM.** `bootstrap_vm` takes one
   artifact (`05`).
6. **A product file and a run path** (`04`, `05` §4, `06`).

## 4. Target pipeline

```text
          ┌──────────────── build (per package) ─────────────────┐
 source ─►│ compile over composed artifact base (ARTC PB-B1)     │
          │   → PackageCompilerImage (ARTC F2)                   │
          │ VM from the artifact chain (no package code yet)     │
          │ emit owned module closures in that VM (XS-C2)        │
          │   → capture owned prototypes/constants/classes       │
          │   → import table of external identities (XS-W3)      │
          │ ExecutableImageV2 section (schema 2)                 │
          │ + ARTC compiler sections → slotmap.telosc            │
          └──────────────────────────────────────────────────────┘
                              │
          ┌──────────── product (per runnable target) ───────────┐
          │ entry module + artifact chain by digest → .telosx    │
          └──────────────────────────────────────────────────────┘
                              │
          ┌──────────── run (per process) ───────────────────────┐
          │ caller maps each digest to a path (no search)        │
          │ load + validate compiler sections (ARTC G)           │
          │ validate each linked section against its prefix (XM) │
          │ bootstrap_vm(Universe)                               │
          │ for each package in chain order:                     │
          │   topology → metadata pool → linkage → classes →     │
          │   initializers → eager modules                       │
          │ initialize entry module (XR)                         │
          └──────────────────────────────────────────────────────┘
```

## 5. Invariants

**XL-I1. No source at run time.** Running a product MUST NOT read any
source file, MUST NOT call the parser, checker or bytecode emitter, and MUST
NOT build the canonical Universe formal image from source. Tests assert this
with `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`,
`canonical_universe_formal_image_build_count() == 0`, and a new emission
counter (`07`).

**XL-I2. Exact binding.** A linked executable section is valid only against
the exact dependency artifacts it was captured against, identified by
artifact digest (D-XL-2). There is no ABI-range compatibility in this
program.

**XL-I3. Ownership is total and disjoint.** Every executable row (module,
prototype owner, class recipe, detached method) belongs to exactly one
artifact in a chain, read from `StableModuleRef.project`. An artifact never
carries a row owned by another.

**XL-I4. Closed classes stay closed.** Materializing a package MUST NOT add
methods, fields or conformances to a class owned by another artifact.
PDR-0001 ("classes are closed") already forbids reopening; this program
does not weaken it.

**XL-I5. Dependencies initialize first.** Universe's bootstrap plan runs
first, unchanged. Then the product's initialization order runs: every
module of artifact `j` in that order runs before any module of a later
artifact `k` (D-MW5-4 as refined by D-XL-9). Modules outside the order
initialize on demand through the existing deferred path.

**XL-I6. Fail closed.** Any mismatch (digest, import, schema, profile,
native ABI, ownership, order) is an error before the first heap object of
the offending artifact is allocated. There is no fallback to source.

**XL-I7. One semantic model.** Linked sections reconstruct the same runtime
objects that `run_compiled` produces from source for the same program. Wire
rows are construction recipes, not a second object model. Output parity
with the source path is a gate (`07`).

**XL-I8. Determinism.** The same inputs produce byte-identical linked
sections and products, independent of process, host path and filesystem
order.

## 6. Crate responsibilities

| Crate | New responsibility |
|---|---|
| `phalcom-compiler-artifact` | `ExecutableImageV2` wire with the import table; schema-dispatching decode; structural validation; the product container (`04`). |
| `phalcom-core` (`artifact/`) | Package capture; linked validation against a dependency prefix; generalized `ExecutableMaterializer`; chain bootstrap; product build and run API. |
| `phalcom-core` (`modules/`) | `build_package_telosc` gains the executable section; a loaded artifact chain type that the VM bootstraps from. |
| `phalcom-modules` | Source-free runtime project identities for packages (from ARTC receipts), so materialized package modules get `ProjectIdentity::Resolved` ids without a source directory. |
| `telos` driver (first appears here, see `06`) | `compile --emit artifact|product`, `run`. |

## 7. Non-goals

- **Development overlays** (test, example and bench targets compiled as
  overlays over a production package, toolchain `builds-artifacts-and-caches.md`).
  Until Archon exists, a test suite is an ordinary root package that
  depends on the package under test, as `packages/slotmap/tests/runtime`
  already is.
- **Self-contained bundles** that embed the artifacts inside the product.
  The product format leaves room for it (`04` XP-W7); it is not specified.
- **ABI-compatible dependency upgrades** without rebuilding dependents
  (XL-I2). Rebuild is the only path.
- **Native code in packages** (`@native` methods, primordial roles). Only
  Universe binds natives.
- **Release and unchecked profiles.** `validate_executable_image` accepts
  only `CompilationProfileV1::Debug` today; this program keeps that.
- **Archon itself.** `06` states what Archon must do to run a product; it
  does not specify Archon.
