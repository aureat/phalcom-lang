# Spec 03.5 Canonical Core Surface Implementation Plan

> For agentic workers: REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Finish Spec 03.5 so one rich #[primitive] declaration drives runtime installation, VM-free semantic/LSP surfaces, implementation provenance, documentation, and deterministic presentation without handwritten native-member authorities.

**Architecture:** Keep Rust primitive attributes as authored truth and produce two projections: executable PRIMITIVES descriptors and a checked-in, VM-free phalcom-native-surface catalog. Import that catalog into the compiler-owned semantic core surface, merge it with real .ph declarations by (owner, side, selector), and let runtime reflection/LSP consume explicit provenance and presentation records. Retire compatibility tables/installers only after census and differential tests prove parity.

**Tech Stack:** Rust 2024 workspace, syn/quote procedural-macro support, criterion bench-only instrumentation, phalcom-native-meta, phalcom-native-surface, phalcom-semantic, phalcom-core, phalcom-lsp, Cargo integration tests, graphify.

**Spec:** docs/work/analyses/typing/03.5-canonical-core-surface-native-implementation-provenance-and-semantic-presentation.md

## Global Constraints

- One native semantic declaration: #[primitive] metadata is authored once; generated catalogs and runtime descriptors are projections.
- Keep phalcom-native-surface, phalcom-semantic, and phalcom-lsp VM-free; no dependency from those crates to phalcom-core.
- Parse primitive attributes and Rust doc attributes only. Never inspect Rust function bodies to infer Phalcom semantics.
- Do not synthesize executable .ph bodies. Conceptual sketches are presentation-only and explicitly non-executable.
- Keep MethodObject compact. Store implementation metadata in immutable catalogs and ObjRef-keyed side tables.
- Native type-lowering failures are toolchain defects: report them structurally; never silently replace concrete metadata with Object, Dynamic, or Unknown.
- Merge source and native members by canonical (owner declaration, dispatch side, selector) identity. Do not merge by name or arity.
- Source declarations and source Phaldoc remain authoritative when a source/native key is explicitly paired.
- Catalog and presentation ordering/fingerprints are deterministic and exclude unstable absolute Rust paths.
- Preserve runtime behavior and Spec 04 syntax work. Do not revert or bypass current parser/type-syntax changes.
- Use @phalcom-semantic-model, @programming-language-semantics, @rust-router, and @rust-testing during implementation; use @superpowers:subagent-driven-development or @superpowers:executing-plans to execute this plan.

## Current Baseline and Audit Findings

At audit start, the interrupted task-owned diff comprised only:

- phalcom-lsp/src/semantic/core_source.rs: rich-catalog ingestion plus legacy NATIVE_MEMBERS fallback;
- phalcom-lsp/src/semantic/surface.rs: source-only callable_index() and an INVALID AST marker;
- phalcom-lsp/src/semantic/mod.rs: incremental core replacement regression assertion.

The shared worktree also contains unrelated concurrent changes in phalcom-type-meta/src/validate.rs and docs/spec/attributes/*.md. Preserve those paths and do not stage them as part of this plan.

Focused gates currently pass:

    cargo test -p phalcom-lsp --lib semantic::tests::live_core_replacement_updates_semantic_surface --no-fail-fast  # 1 passed
    cargo test -p phalcom-lsp --lib hover::tests::render_selector_hover_for_builtin_has_no_phaldoc_section --no-fail-fast # 1 passed
    cargo test -p phalcom-lsp --test integration semantic --no-fail-fast                                      # 26 passed
    cargo test -p phalcom-native-surface --no-fail-fast                                                        # 1 passed
    cargo test -p phalcom-semantic --test core_surface_conformance --no-fail-fast                              # 2 passed
    cargo test -p phalcom-core --test spec03_5_conformance --no-fail-fast                                      # 2 passed

The earlier hover failure was a stale expectation: native docs now add one --- section separator. The current assertion accepts that intended behavior. Existing semantic warnings about unused imports/dead helpers remain unrelated baseline warnings.

Remaining implementation gaps confirmed from live source and the attached interrupted run:

1. phalcom-native-surface/src/generated.rs contains 73 surface_record rows, while the handwritten NATIVE_MEMBERS projection still contains roughly 315 macro rows. No deterministic scanner/generator or stale-output check exists.
2. phalcom-semantic/src/types/native.rs has generic registration, but register_standard_surfaces remains exported and called from workspace.rs and checker/context.rs.
3. LSP now marks native members as MemberOrigin::Native but still stores MemberAstRef::INVALID; the spec requires a native surface identity, not a sentinel.
4. LSP still uses advisory NativeReturnShape for core inference and stores native members with empty parameter surfaces instead of consuming canonical callable signatures.
5. NATIVE_CLASSES, semantic hierarchy setup, and runtime bootstrap still carry overlapping class-relation authority.
6. Runtime provenance side-table binding and reflection exist, but parity, replacement/mutation behavior, and descriptor-only cutover are not proven by the required census.
7. Native docs are captured by the macro for runtime descriptors, but generated rich records contain hand-maintained docs; no shared normalized parser, generated-vs-runtime equality test, or catalog fingerprint exists.
8. phalcom-lsp/src/completion.rs and phalcom-lsp/src/backend.rs still consume NATIVE_MEMBERS directly outside core_source.rs.
9. Required per-owner census and semantic differential tests are absent.

## File and Responsibility Map

- phalcom-native-meta/src/primitive.rs and src/universe.rs: shared stable metadata and bootstrap identities.
- phalcom-native-decl/{Cargo.toml,src/lib.rs,src/normalized.rs,src/parse.rs,src/validate.rs}: one non-proc-macro primitive declaration parser/validator.
- phalcom-native-macros/src/lib.rs: proc-macro adapter; emits descriptors and captured docs using the shared parser.
- phalcom-native-surface/src/lib.rs and src/generated.rs: VM-free generated catalog, compatibility projections, indexed queries, fingerprint.
- phalcom-native-surface-gen/{Cargo.toml,src/main.rs}: deterministic attribute/doc scanner with --check.
- phalcom-semantic/src/types/native.rs, src/workspace.rs, src/checker/context.rs: canonical native type lowering/import; no per-class handwritten registration.
- phalcom-semantic/src/core_surface/{mod.rs,native.rs,source.rs,merge.rs,conformance.rs,presentation.rs}: merged surface, conformance, and presentation IR.
- phalcom-core/src/universe/primitives.rs, src/vm/bootstrap.rs, src/native/{registry.rs,descriptor.rs,install.rs}, src/typing/side_table.rs, src/primitive/method.rs: runtime census, installer cutover, provenance, reflection.
- phalcom-lsp/src/semantic/{core_source.rs,surface.rs,source.rs,invalidation.rs,mod.rs,analyzer.rs}, src/{completion.rs,backend.rs,inlay_hints.rs,hover.rs}: canonical native surface adapter, origin-safe source consumers, and presentation.
- phalcom-core/tests/spec03_5_{census,conformance}.rs, phalcom-semantic/tests/core_surface_conformance.rs, phalcom-lsp/tests/integration.rs: cross-layer acceptance tests.

---

### Task 0: Freeze Live Baseline and Spec 04 Boundaries

Spec anchors: §23 P0, §27 coordination, §31 implementation handoff.

Files:
- Read: AGENTS.md
- Read: docs/work/analyses/typing/03.5-canonical-core-surface-native-implementation-provenance-and-semantic-presentation.md
- Read: docs/work/analyses/typing/04-user-facing-type-syntax-and-lowering-REVISED.md
- Read: docs/superpowers/plans/2026-08-22-spec-01-compiler-owned-typing.md
- Inspect only: git status, current diff, current branch, graphify-out/graph.json

- [ ] Step 1: Record current branch, HEAD, dirty paths, untracked paths, and the exact task-owned paths. Do not stage or reset concurrent files.
- [ ] Step 2: Re-read the Spec 04 syntax/lowering handoff and mark every 03.5 file that overlaps active type-syntax work. The implementation must adapt around those changes.
- [ ] Step 3: Run the six focused baseline commands listed above and preserve their pass counts and warnings in the execution handoff.
- [ ] Step 4: Run graphify query "Spec 03.5 canonical native surface runtime provenance semantic LSP" and validate returned paths against live source before broad edits.
- [ ] Step 5: Stop this task if the current branch or dirty ownership changes materially; otherwise proceed to Task 1 with this plan as the conflict freeze.

### Task 1: Establish Four-Way Census and Red Acceptance Tests

Spec anchors: §22.1-22.3, §25.2-25.4, §30 Checkpoints 1-2.

Files:
- Create: phalcom-core/tests/spec03_5_census.rs
- Modify: phalcom-core/src/universe/primitives.rs
- Modify: phalcom-core/src/native/registry.rs
- Modify: phalcom-semantic/src/types/native.rs
- Modify: phalcom-semantic/src/core_surface/conformance.rs
- Modify: phalcom-semantic/tests/core_surface_conformance.rs

Interfaces:
- Produce sorted key sets for legacy installer A, generated surface B, distributed descriptors C, and semantic imported callables D.
- Use canonical key (UniverseKey, NativeDispatch, selector); no source ranges or Rust symbols.
- Produce deterministic per-owner/per-side counts and a single NativeSurfaceConformance report consumed by runtime and semantic tests.

- [ ] Step 1: Add a test-only legacy installer census hook. Record each key at the existing Universe::install_primitives registration point while preserving installation order and behavior.
- [ ] Step 2: Add runtime-registry key extraction. Expose a test-only iterator over PRIMITIVES keys and assert duplicate keys fail deterministically.
- [ ] Step 3: Add semantic-import key extraction. Return imported canonical keys from native registration so the test compares D without scraping debug output.
- [ ] Step 4: Write the census test. Assert every intentional key is classified into A ∩ C, A - C, C - A, B △ C, and D △ B; print sorted owner/side/selector rows for every mismatch.
- [ ] Step 4a: Add per-owner/per-side snapshots and route the same report through validate_native_surface_conformance; a global count alone is not an acceptance test.
- [ ] Step 5: Run the red baseline census.

    cargo test -p phalcom-core --test spec03_5_census -- --nocapture

Expected: the test reports current legacy-only, rich-catalog-only, and semantic/catalog omissions without deleting compatibility paths. Convert every reported row into a named migration item before proceeding.

- [ ] Step 6: Commit the baseline harness.

    git add phalcom-core/tests/spec03_5_census.rs phalcom-core/src/universe/primitives.rs phalcom-core/src/native/registry.rs phalcom-semantic/src/types/native.rs phalcom-semantic/tests/core_surface_conformance.rs
    git commit -m "test(spec03.5): establish native surface census"

### Task 2: Share Primitive Declaration Parsing and Generate the VM-Free Catalog

Spec anchors: §6.1-6.5, §10.2-10.3, §20.5, §25.1-25.2.

Files:
- Create: phalcom-native-decl/Cargo.toml
- Create: phalcom-native-decl/src/lib.rs
- Create: phalcom-native-decl/src/normalized.rs
- Create: phalcom-native-decl/src/parse.rs
- Create: phalcom-native-decl/src/validate.rs
- Create: phalcom-native-surface-gen/Cargo.toml
- Create: phalcom-native-surface-gen/src/main.rs
- Modify: Cargo.toml
- Modify: phalcom-native-macros/Cargo.toml
- Modify: phalcom-native-macros/src/lib.rs

Interfaces:
- phalcom_native_decl::parse_primitive_attribute(&syn::Attribute) -> Result<NormalizedPrimitiveDecl, DeclError>.
- phalcom_native_decl::validate_decl(&NormalizedPrimitiveDecl) -> Result<(), DeclError>.
- phalcom-native-surface-gen --check [--root <repo>] exits non-zero when generated output is stale.
- Generator reads #[primitive(...)] and #[doc = "..."] only; it never parses function bodies.

- [ ] Step 1: Add parser tests for one getter, one method with labels/rest, docs, conceptual metadata, intrinsic, visibility, and invalid selector.
- [ ] Step 2: Implement normalized declaration structs and validation. Preserve PrimitiveKey, callable type, effects, raises, flow, ABI, lifecycle, intrinsic, trust, docs, and conceptual fields without runtime function pointers.
- [ ] Step 2a: If the current metadata schema still lacks lifecycle fields, add NativeLifecycleSpec in phalcom-native-meta/src/primitive.rs and carry it through descriptor, generated-surface, semantic, runtime, and presentation projections without inventing lifecycle semantics for records that declare Unknown.
- [ ] Step 3: Refactor the proc macro to consume the shared parser. Keep compile errors attached to original attribute spans and keep emitted PrimitiveSurfaceSpec/PrimitiveDescriptor output unchanged.
- [ ] Step 4: Add generator tests using a temporary primitive source fixture. Assert generated records match normalized macro metadata and source traversal order does not affect output order.
- [ ] Step 5: Implement generator scanning. Scan phalcom-core/src/primitive/**/*.rs, collect primitive attributes and attached docs, reject duplicate keys, validate selector/type arity, and emit deterministic Rust into phalcom-native-surface/src/generated.rs.
- [ ] Step 6: Run parser, macro, and generator checks.

    cargo test -p phalcom-native-decl --no-fail-fast
    cargo test -p phalcom-native-macros --no-fail-fast
    cargo run -p phalcom-native-surface-gen -- --root .
    cargo run -p phalcom-native-surface-gen -- --root . --check

Expected: all tests pass; the second generator command reports the checked-in artifact is current.

- [ ] Step 7: Commit shared parsing and generator.

    git add Cargo.toml phalcom-native-decl phalcom-native-surface-gen phalcom-native-macros/Cargo.toml phalcom-native-macros/src/lib.rs phalcom-native-surface/src/generated.rs
    git commit -m "feat(native): generate VM-free surface from primitive metadata"

### Task 3: Make phalcom-native-surface Complete, Indexed, and Fingerprinted

Spec anchors: §6.5-6.7, §20.1-20.5, §21.1-21.4, §25.2.

Files:
- Modify: phalcom-native-meta/src/primitive.rs
- Modify: phalcom-native-surface/src/lib.rs
- Replace generated output: phalcom-native-surface/src/generated.rs
- Modify: phalcom-native-surface/Cargo.toml
- Modify: phalcom-native-surface/src/lib.rs tests
- Modify: phalcom-core/tests/spec03_5_census.rs

Interfaces:
- NativeSurfaceId is the stable PrimitiveKey identity carried by LSP/semantic presentation.
- NativeSurfaceCatalog indexes PrimitiveKey -> &'static NativeSurfaceRecord once.
- catalog_fingerprint() -> NativeCatalogFingerprint hashes structural metadata only, excluding NativeSourceSpec file/line.
- NativeIntrinsicExpectation records legal owner/side/selector, callable shape, guard conditions, and required fallback behavior for each intrinsic ID.
- NATIVE_MEMBERS becomes a generated compatibility projection from the complete rich catalog; it is not an authored list.

- [ ] Step 1: Regenerate the catalog from every primitive attribute. Preserve supported metadata and add explicit records for legacy-only runtime methods identified by Task 1.
- [ ] Step 2: Keep NATIVE_CLASSES only as a transitional class-owner projection. Ensure no member row is authored beside it.
- [ ] Step 3: Implement the indexed catalog. Build the owner/side/selector map once and make find_native_surface use the index rather than scanning the full slice on each hover/query.
- [ ] Step 4: Implement stable fingerprinting. Feed sorted owner, side, selector, visibility, ABI, callable types, effects, raises, flow, intrinsic, trust, docs, and conceptual identifiers into a fixed deterministic digest; omit raw source paths.
- [ ] Step 5: Add intrinsic expectation validation. Assert BoolNot, BoolAnd, and BoolOr claims match legal owner, side, selector, callable shape, guard conditions, and ordinary primitive fallback.
- [ ] Step 6: Add catalog tests. Cover unique keys, valid owners, canonical selectors, stable ordering, fingerprint equality across reordered inputs, docs capture, lifecycle consistency, intrinsic expectations, and NATIVE_MEMBERS projection parity.
- [ ] Step 7: Re-run census and package tests.

    cargo run -p phalcom-native-surface-gen -- --root . --check
    cargo test -p phalcom-native-surface --no-fail-fast
    cargo test -p phalcom-core --test spec03_5_census -- --nocapture

Expected: generated and distributed key sets agree for migrated primitives; any remaining A - C rows are explicit migration failures, not hidden fallback rows.

- [ ] Step 8: Commit the complete catalog.

    git add phalcom-native-meta/src/primitive.rs phalcom-native-surface/Cargo.toml phalcom-native-surface/src/lib.rs phalcom-native-surface/src/generated.rs phalcom-core/tests/spec03_5_census.rs
    git commit -m "feat(native): complete and fingerprint canonical surface"

### Task 4: Converge Bootstrap Class Relations

Spec anchors: §6.8 and §30 Checkpoint 3.

Files:
- Modify: phalcom-native-meta/src/universe.rs
- Modify: phalcom-native-surface/src/lib.rs
- Modify: phalcom-semantic/src/workspace.rs
- Modify: phalcom-core/src/vm/bootstrap.rs
- Modify: phalcom-lsp/src/semantic/core_source.rs
- Modify: phalcom-core/tests/spec03_5_census.rs

Interfaces:
- UNIVERSE_CLASS_RELATIONS is the VM-free authority for genuinely Rust-bootstrapped classes.
- Source-only helper classes remain source-owned and are not forced into UniverseKey.

- [ ] Step 1: Add a census comparison for NATIVE_CLASSES, semantic hierarchy rows, source core declarations, and runtime bootstrap relations.
- [ ] Step 2: Define UniverseClassRelationSpec { class: UniverseKey, superclass: Option<UniverseKey> } beside existing universe identity catalog.
- [ ] Step 3: Derive semantic, runtime, and LSP bootstrap relations from the catalog. Keep real .ph source declarations authoritative when present.
- [ ] Step 4: Add exact parity tests for the metaclass/object tower and every bootstrapped class relation.
- [ ] Step 5: Run relation census and focused tests.

    cargo test -p phalcom-core --test spec03_5_census -- --nocapture
    cargo test -p phalcom-semantic --test workspace --no-fail-fast
    cargo test -p phalcom-lsp --lib semantic::core_source --no-fail-fast

Expected: no relation discrepancy remains; source-only helper classes are explicitly excluded from the canonical bootstrap set.

- [ ] Step 6: Commit relation convergence.

    git add phalcom-native-meta/src/universe.rs phalcom-native-surface/src/lib.rs phalcom-semantic/src/workspace.rs phalcom-core/src/vm/bootstrap.rs phalcom-lsp/src/semantic/core_source.rs phalcom-core/tests/spec03_5_census.rs
    git commit -m "refactor(core): centralize bootstrap class relations"

### Task 5: Replace Semantic Handwritten Registration with Canonical Import

Spec anchors: §7.1-7.6, §18.1-18.4, §22.3, §30 Checkpoints 2-3.

Files:
- Modify: phalcom-semantic/Cargo.toml
- Create: phalcom-semantic/src/core_surface/native.rs
- Modify: phalcom-semantic/src/types/native.rs
- Modify: phalcom-semantic/src/types/mod.rs
- Modify: phalcom-semantic/src/lib.rs
- Modify: phalcom-semantic/src/signature.rs
- Modify: phalcom-semantic/src/db/mod.rs
- Modify: phalcom-semantic/src/workspace.rs
- Modify: phalcom-semantic/src/checker/context.rs
- Modify: phalcom-semantic/src/dispatch.rs
- Modify: phalcom-semantic/tests/core_surface_conformance.rs

Interfaces:
- NativeSurfaceId is NativeSurfaceId(pub PrimitiveKey).
- NativeCatalogFingerprint is NativeCatalogFingerprint(pub [u8; 16]).
- NativeSurfaceCatalog owns the static records plus a BTreeMap<PrimitiveKey, usize> index.
- NativeSurfaceImportError is an enum with InvalidSelector, OwnerMissing, SelectorArityMismatch, TypeLowering, and UnsupportedMetadata variants, each carrying PrimitiveKey and the source metadata error where applicable.
- NativeSurfaceImportReport contains imported canonical keys, Vec<(CallableId, CallableSemanticSignature)>, and structured failures.
- register_native_surfaces(...) -> Result<NativeSurfaceImportReport, NativeSurfaceImportError> consumes only phalcom_native_surface::NATIVE_SURFACES.
- The report is inserted into CallableSignatureTable and the compiler-owned SemanticDb/core-surface product; LSP consumes that projection and does not lower PrimitiveSurfaceSpec itself.
- register_standard_surfaces is removed from exports and all callers.

- [ ] Step 1: Add failing differential tests. For every catalog record, assert semantic lookup returns the same owner, side, selector, parameter labels/arity, return form, effects, flow, and TrustedNative authority.
- [ ] Step 2: Update native registration to use canonical PrimitiveSurfaceSpec callable metadata. Resolve owner forms, parameters, return types, external labels, side, visibility, and flow in stable key order.
- [ ] Step 3: Reject malformed records at the import boundary. Replace Selector::try_decode_exact(...); continue with a structured NativeSurfaceImportError; never silently skip a catalog row.
- [ ] Step 4: Thread the result through workspace/checking setup, CallableSignatureTable, and the SemanticDb publication product. Internal built-in metadata failures become toolchain diagnostics; ordinary user analysis remains separate.
- [ ] Step 5: Delete per-class method construction and the register_standard_surfaces wrapper. Retain resolve_native_type_form and normalization helpers.
- [ ] Step 5a: Add tests that EffectSpec::Unknown, RaisesSpec::Unknown, and ReturnFlowSpec::Unknown remain unknown in semantic signatures and presentation; absence of a claim must not become pure, no-raises, or value flow.
- [ ] Step 6: Run semantic differential and regression gates.

    cargo test -p phalcom-semantic --test core_surface_conformance --no-fail-fast
    cargo test -p phalcom-semantic --test workspace --no-fail-fast
    cargo test -p phalcom-semantic --lib --no-fail-fast
    cargo test -p phalcom-core --test spec03_5_census -- --nocapture

Expected: every supported generated native record has one canonical semantic callable; unsupported concrete metadata fails explicitly rather than widening silently.

- [ ] Step 7: Commit semantic convergence.

    git add phalcom-semantic/Cargo.toml phalcom-semantic/src/core_surface/native.rs phalcom-semantic/src/types/native.rs phalcom-semantic/src/types/mod.rs phalcom-semantic/src/lib.rs phalcom-semantic/src/signature.rs phalcom-semantic/src/db/mod.rs phalcom-semantic/src/workspace.rs phalcom-semantic/src/checker/context.rs phalcom-semantic/src/dispatch.rs phalcom-semantic/tests/core_surface_conformance.rs
    git commit -m "feat(semantic): import canonical native surfaces"

### Task 6: Complete Source/Native Merge and Presentation IR

Spec anchors: §8.1-8.8, §11.3-11.7, §12.1-12.4, §25.5-25.6.

Files:
- Modify: phalcom-semantic/src/core_surface/mod.rs
- Modify: phalcom-semantic/src/core_surface/source.rs
- Modify: phalcom-semantic/src/core_surface/merge.rs
- Modify: phalcom-semantic/src/core_surface/conformance.rs
- Modify: phalcom-semantic/src/core_surface/presentation.rs
- Modify: phalcom-semantic/tests/core_surface_conformance.rs

Interfaces:
- SurfaceMergeOutcome covers SourceOnly, NativeOnly, SourceDeclarationNativeImplementation, SourceWrapperOverNative, Generated, and Conflict.
- ClassPresentation and MethodPresentation carry canonical callable identity, implementation kind, intrinsic/effect/raises/flow summaries, docs, and optional conceptual text.
- render_markdown() and render_virtual_source() are pure presentation functions; neither returns parser AST or executable source.

- [ ] Step 1: Add source surface projection tests for source-only methods, source docs, source ranges, class-side members, and source identity.
- [ ] Step 2: Merge source and native records by (DeclarationId, DispatchSide, selector). Preserve native-only methods even when the source shell has no body.
- [ ] Step 3: Add SourceNativeBindingRole::{None, DeclarationImplementation, WrapperOverNative} to source records and diagnose unclassified collisions. Do not silently prefer source or native when both claim an executable key without an explicit binding role.
- [ ] Step 4: Make presentation ordering deterministic by class, dispatch side, and selector. Mark conceptual text as read-only/non-executable.
- [ ] Step 5: Add presentation goldens for Bool, Int, String, TypingContext, and a source-only class; assert native/intrinsic/pure badges and source-doc precedence.
- [ ] Step 6: Run presentation tests.

    cargo test -p phalcom-semantic --test core_surface_conformance --no-fail-fast
    cargo test -p phalcom-semantic --test core_surface_conformance --no-fail-fast

Expected: native-only methods are present, source-only methods retain source identity, collisions are classified, and conceptual output cannot be fed to parser/codegen APIs.

- [ ] Step 7: Commit merge and presentation IR.

    git add phalcom-semantic/src/core_surface phalcom-semantic/tests/core_surface_conformance.rs
    git commit -m "feat(semantic): add canonical source-native presentation"

### Task 7: Finish Runtime Descriptor Cutover and Provenance Conformance

Spec anchors: §9.1-9.5, §13.1-13.4, §19.1-19.3, §21.1-21.4, §25.4.

Files:
- Modify: phalcom-core/src/universe/primitives.rs
- Modify: phalcom-core/src/vm/bootstrap.rs
- Modify: phalcom-core/src/native/registry.rs
- Modify: phalcom-core/src/native/descriptor.rs
- Modify: phalcom-core/src/native/install.rs
- Modify: phalcom-core/src/typing/side_table.rs
- Modify: phalcom-core/src/typing/capability.rs
- Modify: phalcom-core/src/primitive/method.rs
- Modify: phalcom-core/src/primitive/reflection.rs
- Create: phalcom-core/core/universe/src/reflection/implementation.ph
- Modify: phalcom-core/tests/spec03_5_census.rs
- Modify: phalcom-core/tests/spec03_5_conformance.rs

Interfaces:
- MethodImplementationIndex remains keyed by live ObjRef and stores RuntimeImplementationRef without changing MethodObject layout.
- Runtime native install binds ObjRef -> PrimitiveKey/ImplementationKind/intrinsic/ABI/source at the exact allocation point.
- Reflection returns high-level #native/#source; detailed source paths remain authority-gated tooling data.
- Detailed implementationOf(_) returns a logical module/symbol record only with OBSERVE_IMPLEMENTATION_PROVENANCE; RuntimePublic does not imply this capability.
- NativeInstallMode::{Dual, DescriptorOnly} is selected by VM::new_with_native_install_mode for deterministic pre-cutover comparison; VM::new uses the shipping default.

- [ ] Step 1: Extend conformance tests to compare every installed method owner/side/selector, visibility, ABI, intrinsic claim, and side-table key against generated metadata.
- [ ] Step 2: Add method replacement tests. Install/replace a method and assert the replacement object does not inherit old native provenance.
- [ ] Step 3: Add OBSERVE_IMPLEMENTATION_PROVENANCE to phalcom-core/src/typing/capability.rs and implement capability-checked logical module/symbol reflection in primitive/reflection.rs and the implementation.ph source shell.
- [ ] Step 4: Make implementationKind consult side-table provenance when available, while preserving source-method behavior. Keep isNative and isIntrinsic consistent with the same metadata.
- [ ] Step 5: Add authority-forging tests: user/source declarations cannot create NativePrimitive, Privileged, or intrinsic provenance; only compiler/runtime installation can do so.
- [ ] Step 6: Run VM startup in NativeInstallMode::Dual and NativeInstallMode::DescriptorOnly. Compare installed keys, visibility, reflection, and representative behavior deterministically.
- [ ] Step 7: Delete only the install_primitives method from phalcom-core/src/universe/primitives.rs and delete legacy install macros from phalcom-core/src/primitive/mod.rs after the four-way census has no intentional mismatch. Preserve validate_native_surface and source wrappers.
- [ ] Step 8: Run runtime gates.

    cargo test -p phalcom-core --test spec03_5_census -- --nocapture
    cargo test -p phalcom-core --test spec03_5_conformance --no-fail-fast
    cargo test -p phalcom-core --test spec03_reflection --no-fail-fast
    cargo test -p phalcom-core --test invariants --no-fail-fast

Expected: descriptor-only installation has the same runtime floor and reflection results; no MethodObject size/layout change occurs.

- [ ] Step 9: Commit runtime cutover.

    git add phalcom-core/src/universe/primitives.rs phalcom-core/src/vm/bootstrap.rs phalcom-core/src/native/registry.rs phalcom-core/src/native/descriptor.rs phalcom-core/src/native/install.rs phalcom-core/src/typing/side_table.rs phalcom-core/src/typing/capability.rs phalcom-core/src/primitive/method.rs phalcom-core/src/primitive/reflection.rs phalcom-core/core/universe/src/reflection/implementation.ph phalcom-core/tests/spec03_5_census.rs phalcom-core/tests/spec03_5_conformance.rs
    git commit -m "refactor(core): cut over to descriptor native installation"

### Task 8: Remove LSP Sentinels and Consume Canonical Signatures/Presentation

Spec anchors: §14.1-14.6, §15.1-15.4, §25.7.

Files:
- Modify: phalcom-lsp/src/semantic/surface.rs
- Modify: phalcom-lsp/src/semantic/core_source.rs
- Modify: phalcom-lsp/src/semantic/source.rs
- Modify: phalcom-lsp/src/semantic/invalidation.rs
- Modify: phalcom-lsp/src/semantic/mod.rs
- Modify: phalcom-lsp/src/semantic/analyzer.rs
- Modify: phalcom-lsp/src/completion.rs
- Modify: phalcom-lsp/src/backend.rs
- Modify: phalcom-lsp/src/inlay_hints.rs
- Modify: phalcom-lsp/src/hover.rs
- Modify: phalcom-lsp/tests/integration.rs

Interfaces:
- MemberOrigin::Source(MemberAstRef), MemberOrigin::Native(NativeSurfaceId), and MemberOrigin::Generated(GeneratedMemberOrigin) are the only origin forms.
- GeneratedMemberOrigin is a stable-key wrapper: GeneratedMemberOrigin { stable_key: Box<str> }.
- MemberSurface has no usize::MAX native AST sentinel. Source-only helpers accept Option<MemberAstRef> or a MemberOrigin::Source value.
- Native LSP callable signatures come from the compiler-owned semantic projection; NativeReturnShape remains only as an explicitly advisory fallback during incomplete snapshots and is removed when the shared signature path is live.

- [ ] Step 1: Add failing origin tests. Assert every native core member carries its PrimitiveKey, callable_index() contains only source members, and no native member stores MemberAstRef::INVALID.
- [ ] Step 2: Change MemberOrigin::Native to carry NativeSurfaceId. Remove MemberAstRef::INVALID, its is_invalid() API, and native ast field usage.
- [ ] Step 3: Update all source-AST consumers. inlay_hints.rs, semantic/invalidation.rs, and semantic/source.rs must pattern-match MemberOrigin::Source before indexing a Program; native members receive no fake source range/body.
- [ ] Step 4: Build one LSP adapter over phalcom-semantic::core_surface presentation/signature records. Populate parameter labels, return summaries, effects, docs, and origin from the shared catalog rather than manually constructing empty native params.
- [ ] Step 5: Replace per-hover linear catalog lookup with the indexed catalog. Render owner/signature/native/intrinsic/effect/docs badges; omit raw Rust paths by default and respect internal visibility.
- [ ] Step 5a: Replace the remaining direct NATIVE_MEMBERS consumers in completion.rs and backend.rs with the same indexed canonical catalog and semantic presentation adapter.
- [ ] Step 6: Preserve source behavior tests. Confirm source hovers, definitions, inlay hints, invalidation fingerprints, completion, and semantic inference remain unchanged for real source members.
- [ ] Step 7: Run LSP gates.

    cargo test -p phalcom-lsp --lib semantic::surface --no-fail-fast
    cargo test -p phalcom-lsp --lib semantic::tests::live_core_replacement_updates_semantic_surface --no-fail-fast
    cargo test -p phalcom-lsp --lib hover::tests::render_selector_hover_for_builtin_has_no_phaldoc_section --no-fail-fast
    cargo test -p phalcom-lsp --test integration semantic --no-fail-fast

Expected: native members are discoverable and render canonical signatures/docs; source-only indexing and incremental invalidation never dereference a native member as source AST.

- [ ] Step 8: Commit LSP convergence.

    git add phalcom-lsp/src/semantic/surface.rs phalcom-lsp/src/semantic/core_source.rs phalcom-lsp/src/semantic/source.rs phalcom-lsp/src/semantic/invalidation.rs phalcom-lsp/src/semantic/mod.rs phalcom-lsp/src/semantic/analyzer.rs phalcom-lsp/src/completion.rs phalcom-lsp/src/backend.rs phalcom-lsp/src/inlay_hints.rs phalcom-lsp/src/hover.rs phalcom-lsp/tests/integration.rs
    git commit -m "feat(lsp): consume explicit native origins and presentation"

### Task 9: Add Native Documentation, Conceptual Presentation, and Optional Durable Metadata

Spec anchors: §10.1-10.5, §11.3-11.7, §15.1-15.4, §16.1-16.4.

Files:
- Modify: phalcom-native-meta/src/primitive.rs
- Modify: phalcom-native-macros/src/lib.rs
- Modify: phalcom-semantic/src/core_surface/presentation.rs
- Modify: phalcom-lsp/src/hover.rs
- Modify: phalcom-semantic/src/metadata/export.rs
- Modify: phalcom-type-meta/src/bundle.rs
- Modify: phalcom-type-meta/src/header.rs
- Add: native docs/presentation golden tests beside the owning package tests

Interfaces:
- Native docs are one concatenated static NativeDocumentationSpec/raw Phaldoc-compatible string; source docs win for source declarations.
- Conceptual presentation is non-executable metadata and never enters parser/codegen/runtime installation.
- Optional serialized extension key is core.implementation-presentation, keyed by stable callable identity and omitting absolute paths.

- [ ] Step 1: Add macro tests for deterministic /// capture, blank-line preservation, conceptual metadata, and unchanged executable expansion.
- [ ] Step 2: Route native and source docs through existing Phaldoc normalization behavior. Do not duplicate tag aliases in the macro.
- [ ] Step 3: Add hover/presentation goldens proving native docs appear, source docs take precedence, conceptual text is labeled read-only, and Rust paths are absent by default.
- [ ] Step 4: Add durable core.implementation-presentation only if packaged metadata consumers are in scope. Serialize logical owner/selector/provenance/docs/conceptual data and add compatibility tests; otherwise record this exact extension as deferred after in-process acceptance.
- [ ] Step 5: Run docs/metadata tests.

    cargo test -p phalcom-native-macros --no-fail-fast
    cargo test -p phalcom-semantic --test core_surface_conformance --no-fail-fast
    cargo test -p phalcom-lsp --lib hover --no-fail-fast
    cargo test -p phalcom-semantic --test metadata_export --no-fail-fast

Expected: docs and conceptual presentations are deterministic, authority-safe, and absent from ordinary VM execution paths.

- [ ] Step 6: Commit documentation/presentation metadata.

    git add phalcom-native-meta/src/primitive.rs phalcom-native-macros/src/lib.rs phalcom-semantic/src/core_surface/presentation.rs phalcom-lsp/src/hover.rs phalcom-semantic/src/metadata/export.rs phalcom-type-meta/src/bundle.rs phalcom-type-meta/src/header.rs
    git commit -m "feat(docs): expose native implementation presentation safely"

### Task 10: Final Deletion, Drift Gates, and Full Verification

Spec anchors: §20, §22, §23 P8, §25.8, §26, §30 Checkpoints 4-6.

Files:
- Modify: scripts/verify.sh
- Modify: scripts/bench.sh
- Create: phalcom-semantic/benches/spec03_5_surface.rs
- Modify: phalcom-semantic/Cargo.toml
- Modify: docs/work/analyses/typing/03.5-canonical-core-surface-native-implementation-provenance-and-semantic-presentation.md only for completed status/checkpoint notes
- Delete after parity: the install_primitives method in phalcom-core/src/universe/primitives.rs; legacy install macros in phalcom-core/src/primitive/mod.rs; handwritten NATIVE_MEMBERS rows in phalcom-native-surface/src/lib.rs; register_standard_surfaces wrapper in phalcom-semantic/src/types/native.rs; native AST sentinel helpers and formal NativeReturnShape field paths in phalcom-lsp/src/semantic/surface.rs and its consumers. Preserve phalcom-core/src/universe/primitives.rs validate_native_surface.
- Modify: Cargo.toml and affected crate Cargo.toml files to remove obsolete dependencies/features

- [ ] Step 1: Add repository verification gates. scripts/verify.sh must invoke cargo run -p phalcom-native-surface-gen -- --root . --check, cargo test -p phalcom-core --test spec03_5_census, cargo test -p phalcom-semantic --test core_surface_conformance, and a zero-match assertion for MemberAstRef::INVALID and native usize::MAX sentinel construction.
- [ ] Step 2: Run format and whitespace checks.

    cargo fmt --all -- --check
    git diff --check

Expected: both commands pass.

- [ ] Step 3: Run all focused acceptance lanes.

    cargo run -p phalcom-native-surface-gen -- --root . --check
    cargo test -p phalcom-native-meta --no-fail-fast
    cargo test -p phalcom-native-macros --no-fail-fast
    cargo test -p phalcom-native-surface --no-fail-fast
    cargo test -p phalcom-semantic --test core_surface_conformance --no-fail-fast
    cargo test -p phalcom-core --test spec03_5_census --no-fail-fast
    cargo test -p phalcom-core --test spec03_5_conformance --no-fail-fast
    cargo test -p phalcom-core --test spec03_reflection --no-fail-fast
    cargo test -p phalcom-lsp --test integration --no-fail-fast

Expected: all targeted lanes pass with zero intentional census/differential mismatches.

- [ ] Step 4: Run workspace verification.

    cargo test --workspace --no-fail-fast
    ./scripts/verify.sh

Expected: workspace tests and repository verification pass. Classify any unrelated baseline failure separately; do not hide a Spec 03.5 regression under a broad green result.

- [ ] Step 5: Run required performance checks. cargo bench -p phalcom-semantic --bench spec03_5_surface must cover cold/warm catalog import, core member lookup, and presentation rendering; scripts/bench.sh must cover native LSP hover and VM initialization. Record allocation counts for ordinary VM startup versus detailed reflection and confirm no docs/conceptual allocations on ordinary execution.
- [ ] Step 6: Update the knowledge graph after code changes.

    graphify update .

- [ ] Step 7: Commit deletion/drift prevention.

    git add scripts/verify.sh scripts/bench.sh Cargo.toml phalcom-semantic/Cargo.toml phalcom-semantic/benches/spec03_5_surface.rs docs/work/analyses/typing/03.5-canonical-core-surface-native-implementation-provenance-and-semantic-presentation.md
    git commit -m "test(spec03.5): enforce canonical surface drift gates"

## Final Acceptance Report

Report four buckets separately:

- Passing: exact commands and counts for generator, catalog, semantic differential, runtime conformance, LSP, workspace, and formatting gates.
- Baseline/unrelated: pre-existing warnings or failures with file/test evidence and no causal relation to Spec 03.5.
- Deferred: only the optional durable metadata extension, virtual-document transport, or P1b bootstrap relation convergence when the execution handoff records the exact mismatch and names its follow-on plan.
- Unverified: manual editor validation, packaged-toolchain loading, or benchmarks not actually run.

Do not claim Spec 03.5 complete from focused tests alone. Completion requires generated-catalog parity, four-way census closure, semantic differential equality, descriptor-only runtime behavior, explicit LSP provenance, deterministic presentation, and deletion/drift gates.
