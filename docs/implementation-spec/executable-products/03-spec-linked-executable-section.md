# 03 — Spec: the linked executable section (schema 2)

Crates: `phalcom-compiler-artifact` (`src/executable/`) for wire and
structural validation; `phalcom-core` (`src/artifact/`) for semantic
validation and capture.

## 1. Wire

**XS-W1. Type.** Add `ExecutableImageV2` next to `ExecutableImageV1`:

```rust
/// Bytecode of one package artifact, linked by stable identity against the
/// exact dependency artifacts listed in `dependencies`.
pub struct ExecutableImageV2 {
    pub compatibility: ExecutableCompatibilityV1,     // XS-W5
    pub dependencies: Vec<ExecutableDependencyV2>,    // XS-W2
    pub imports: ExecutableImportsV2,                 // XS-W3
    pub modules: Vec<PortableModuleV1>,               // owned only
    pub prototypes: Vec<PortablePrototypeV1>,
    pub constants: Vec<ConstantRecipeV1>,
    pub semantic_pools: Vec<PortableSemanticPoolV1>,
    pub classes: Vec<PortableClassRecipeV1>,          // owned declarations only
    pub source_maps: Vec<PortableSourceMapV1>,        // XS-W6
    pub detached_methods: Vec<PortableDetachedMethodV1>,
    pub bootstrap: PortableBootstrapPlanV1,           // native_runtime_roles empty
}
```

The row types are the schema 1 types, unchanged. There are no
`native_anchors` or `primordial_roles` fields (D-XL-8). The encoding writes
`EXECUTABLE_SCHEMA_V2` first, then the fields in the order above, using the
same `Wire` and `record!`/`tagged!` machinery as schema 1.

**XS-W2. Dependencies.**

```rust
pub struct ExecutableDependencyV2 {
    pub project: StableProjectRef,
    pub artifact_digest: ArtifactDigest,
    pub role: ExecutableDependencyRoleV2,   // Universe | Package
}
```

The list holds the artifact's whole dependency closure, in the order
`plan_package_dependencies` would load it: exactly one `Universe` row,
first, then every package the artifact depends on directly or transitively.
No project appears twice.

**XS-W3. Imports.** One sorted, duplicate-free vector per stable reference
family. The families are exactly those the closed validator checks with an
`ExecutableValidationContext::has_p7_*` predicate, plus type and kind
digests:

```rust
pub struct ExecutableImportsV2 {
    pub modules: Vec<StableModuleRef>,
    pub bindings: Vec<ImportedBindingV2>,            // (module, name, Global | StaticConstant)
    pub declarations: Vec<StableDeclarationRef>,
    pub classes: Vec<StableClassRefV1>,
    pub callables: Vec<StableCallableRef>,
    pub variants: Vec<StableVariantRef>,
    pub data_components: Vec<StableDataComponentRef>,
    pub variant_fields: Vec<StableVariantFieldRef>,
    pub type_parameters: Vec<StableTypeParameterRef>,
    pub implementations: Vec<StableImplRef>,
    pub fields: Vec<StableFieldRef>,
    pub static_constants: Vec<StableStaticConstantRef>,
    pub type_digests: Vec<[u8; 32]>,
    pub kind_digests: Vec<[u8; 32]>,
}
```

"Sorted" means ascending by the entry's canonical wire encoding. If a
`has_p7_*` family is added or removed before implementation, this struct
follows it; the rule is the correspondence, not the list.

**XS-W4. What counts as external.** A reference is external when its owning
project (for type and kind digests: the pool it resolves in, XS-V7) is not
the artifact's own publishing identity. Every external reference anywhere in
the section (linked reads, constant recipes, class recipes' superclass,
metaclass and holder fields, closure-template `lexical_class` and
`super_anchor`, method origins, detached methods, semantic pools) is an
entry in `imports`.

**XS-W5. Compatibility.** `compatibility` is `ExecutableCompatibilityV1`
with `executable_schema = EXECUTABLE_SCHEMA_V2`. Every other ABI field
equals the Universe dependency's schema 1 compatibility block, including
`native_executable_abi_digest`, which is copied rather than recomputed
(there are no package anchors to hash). `validate_native_abi_digest` is
schema-dispatched: schema 1 recomputes from its own anchors, as today;
schema 2 defers the check to chain validation (XM-V3).

**XS-W6. Source maps.** `logical_uri` for a package module MUST NOT
contain a host path. Add `package_module_uri(identity, module)` next to
`universe_module_uri` in `phalcom-modules/src/identity.rs`, producing
`phalcom://package/<package>/<version>/<module path>`. Line tables are
copied from the package source at build time; the source text itself is
not stored.

**XS-W7. Container.** A package artifact with a linked section sets
`PORTABLE_EXECUTABLE` in its header and writes the section with
`SectionKind::Executable`, using the same section digest rule as the
Universe executable section in `build_full_universe_telosc`. The four ARTC
sections are unchanged.

**XS-W8. Decode.** `ExecutableSection::decode` (new, in
`phalcom-compiler-artifact/src/executable/`) reads the schema word and
dispatches: 1 → `ExecutableImageV1::decode`, 2 → `ExecutableImageV2::decode`,
anything else → `CodecError`. Both decoders charge the existing
`AllocationBudget`. `ExecutableImageLimitsV1` gains caps for each import
vector and for `dependencies`.

## 2. Validation

Structural checks (codec crate, no runtime authority) are `XS-V1`…`XS-V3`.
Semantic checks (`phalcom-core`, `validate_linked_executable_image`, new
sibling of `validate_executable_image`) are `XS-V4`…`XS-V12`. All run before
any heap allocation for the artifact (XL-I6).

**XS-V1. Header agreement.** Schema 2 appears only in an artifact whose
header identity is `TeloscPackageIdentity::Package`. Schema 1 appears only
in the Universe artifact. Any other pairing is rejected.

**XS-V2. Table shape.** Every import vector is sorted and duplicate-free.
`dependencies` has exactly one `Universe` row and it is first, and no
project repeats. `bootstrap.native_runtime_roles` is empty.

**XS-V3. Index domains.** All intra-image indices (`PrototypeId`,
`ConstantId`, `SourceMapId`, `SemanticPoolId`) are in range, exactly as
schema 1's `validate_structure` checks them.

**XS-V4. Ownership.** Every `modules[i].identity`, every
`PrototypeOwnerV1::ModuleInitializer`/`Callable` owner, every class recipe's
declaration, every detached method's callable, and every source map's
module belongs to the artifact's publishing identity (XL-I3).

**XS-V5. Module coverage against the module world.** The set of
`modules[*].identity` equals the set of owned rows in the artifact's
`ModuleWorldV5`. For each module, `linked_reads` matches the module world's
linked reads in order, with the same per-variant comparison
`validate_module_coverage` makes. Every module in `imports.modules` is a
target of the module world's external table (`external_modules`). In
short: the executable section and the compiler section of one artifact
describe the same modules and the same reads.

**XS-V6. Closure in both directions.** Every external reference (XS-W4) is
in `imports`; every entry of `imports` is referenced at least once
(D-XL-3). Owned references must resolve inside the artifact: owned modules
in `modules`, owned classes in `classes`, owned declarations, callables and
others in the artifact's own semantic world and metadata.

**XS-V7. Type and kind digests.** Owned `PortableTypeRecipeV1::Canonical`
digests must resolve in the artifact's own metadata bundle; imported ones
must not, and are resolved against the chain (XM-V4).

**XS-V8. Foreign mutation is rejected.** A `MethodRecipeV1` whose `holder`
or `access_owner` is an imported class, and a `PortableClassRecipeV1` for
an imported declaration, are rejected (XL-I4). If the census in slice X0
finds that the current lowering installs package-owned trait
implementations into foreign class tables, this rule is **BLOCKING** for
that construct: stop and write an incident (see `09` Q2).

**XS-V9. No artifact-level eagerness.** `bootstrap.eager_modules` is
empty and `bootstrap.deferred_modules` lists every module of `modules`
(D-XL-9). Which package modules run, and in what order, is decided per
product (`04` XP-W8). `bootstrap.required_declarations` lists only owned
declarations.

**XS-V10. Instruction validation.** Operand domains, stack shape and
control flow run unchanged on every prototype
(`validate_prototype_operands`, `validate_prototype_stack`,
`validate_prototype_control_flow`). Where an operand's validator requirement
needs a referent (`ModuleReference`, `MethodReference`,
`SemanticPoolReference`, ...), it consults owned tables first, then
`imports`.

**XS-V11. Profile and ABI.** `compilation_profile` is `Debug` (as
`validate_executable_image` requires today). `portable_bytecode_abi`,
`emitter_abi`, `vm_operand_frame_abi` and `native_runtime_abi` equal the
running VM's constants.

**XS-V12. Dependency agreement.** `dependencies` (minus Universe) equals
the set of projects in the artifact's `ModuleWorldV5` external project
table, extended transitively by each dependency's own `dependencies`.
Digests are checked against the actual chain at load (XM-V2), not here.

## 3. Builder changes

**XS-B1.** `build_package_telosc` builds the canonical artifact, with the
executable section, by default (D-XL-1). It requires the loaded dependency
chain, including every dependency's executable section, and fails with a
typed error if any dependency is compiler-only. A separate, explicitly
named entry point (for example `build_package_compiler_only_telosc`) keeps
today's compiler-only output for compiler development and test fixtures.
Its rustdoc MUST state that its output is not a canonical package artifact.

**XS-B2.** The executable section is captured from the same
`PackageCompilerImage` (same `ExecutableLinkedProgram`, same publishing
identity) as the compiler sections, in the same call. The identity retry
loop in `build_package_telosc` covers it: if capture discovers an external
project not already in the identity inputs, the retry includes it.

**XS-B3.** The builder's load self-check (ARTC PB-H5) is extended: after
writing, it decodes the artifact and runs `XS-V1`…`XS-V12` and the chain
checks of `05` §1 against the same dependency chain.

## 4. Capture

**XS-C1. Capture subject.** Generalize `ExecutableImageCapture::new` and
`capture_module` from `&CanonicalUniverseProgram` to a capture subject that
supplies: the owned module set, each module's linked reads, its source unit
(for line tables only), eager or deferred, and the identity context for
`to_stable_module`. Universe implements it from `CanonicalUniverseProgram`;
packages implement it from `PackageCompilerImage`. The Universe output MUST
stay byte-identical (ARTC VER-R1 gate).

**XS-C2. Capture VM.** Bootstrap a VM from the dependency chain as a run
would (`05` §2, XM-M1…XM-M6 for every dependency), but initialize no
package module of any dependency: only Universe's own bootstrap runs. If
emission turns out to read an initialized value from a dependency package
module, the census (slice X0) records it and the implementer writes an
incident before going further. Then:

1. materialize the package's linked program into it (`materialize_program`
   phases for topology, lexical intrinsics and linkage), without
   initializing any module;
2. for each owned module in the linked program's initialization order,
   compile its closure with `compile_program_module_closure` and call the
   capture hooks (`capture_module`, `capture_class_layouts`,
   `capture_detached_methods`) as the Universe bootstrap does;
3. assert that every owned module is still in state `Prepared` (D-XL-7).

**XS-C3. Classifying reached objects.** When capture reaches a heap object:

- an owned closure, method, class layout or constant is captured as a
  recipe, as today;
- a class or module owned by a dependency becomes a reference recipe
  (`ConstantRecipeV1::Class` / `Module`) and an import;
- a method object whose holder is a foreign class is an error
  (`ExecutableCaptureError::ForeignMethod`, new);
- any other foreign heap object is an error
  (`ExecutableCaptureError::ForeignObject`, new). Universe singletons that
  the emitter embeds as constants (for example `None`) are reached through
  an existing immediate or reference recipe and are not foreign objects;
  the census (slice X0) lists every such case.

**XS-C4. Import assembly.** Imports are collected while capturing, then
sorted (XS-W3). Capture then checks XS-V6 itself and fails on any unused or
missing entry, so a capture bug is caught at build time.

**XS-C5. Determinism.** Prototype, constant and source-map ids follow
capture order, and capture order follows the linked program's
initialization order, which is deterministic. Constant deduplication is the
existing `constant_by_object` map. Two builds in fresh processes MUST
produce byte-identical sections (VER-X1).

**XS-C6. Atomicity.** A capture failure fails the whole build. The builder
MUST NOT fall back to writing a compiler-only artifact when an executable
one was requested.
