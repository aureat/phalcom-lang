# PERF003.C2.P7 continuation handoff

Status: IN_PROGRESS; completion: PARTIAL; verification: UNVERIFIED.
Worktree: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization
Branch: perf/universe-characterization; start HEAD: 3ca30fca.

## Established code boundary

CanonicalExternalModuleWorld owns completed LinkedProgram, interfaces and
ModuleTopology; from_parts accepts completed authority and validates matching
module sets without linking. CanonicalModuleProducts wraps that world and
source-build units/fingerprint. CanonicalUniverseFormalImage holds the external
world separately from build sources and optional presentation provider.

WorkspaceModuleSession::with_external_modules and ModuleResolver::with_external_modules
require only completed external authority. External source loads fail explicitly;
interfaces and logical source locations come from external authority. Linker
consumers now accept CanonicalExternalModuleWorld. Image-backed semantic type
resolution uses image.linked(), rather than reacquiring source-built linking.
Optional source presentation remains available on source-built images.

ArtifactHasher is BLAKE3-256 with existing domain framing. Reader rejects
unsupported section schemas/flags/compression and missing executable capability
payload. Stable identity schema participates in compatibility checks.

## Remaining implementation

T4/T5 code is present; preserve P5 immutable-parent and P6 local-index boundaries.
Start with the shared deterministic export-context/census refinement in
[the P7 execution companion](PERF003.C2.P7-remaining-execution-plan.md), then
load structural primary contracts before T6 static ADT values. T6–T15 and
P8 implementation/acceptance remain open; the P8 detailed plan is now prepared.
The telosc specification records the accepted intended contract, not completion.
There is no generated universe.telosc or source-free compiler/VM proof yet.

## Check policy

User explicitly requested compiler-driven checks only: do not run tests or
broader verification unless that instruction changes. Core library compilation
passed during the continuation; the final consumer check is recorded in the
walkthrough. Do not infer behavioral correctness from compiler success.

Preserve the inherited dirty T1/T2 changes and documentation moves. No commit
or push was requested. The shared CHECKPOINT.md owns durable state.


## Latest continuation: T4/T5 code

Use phalcom_modules::compiler_artifact::{project_module_world, load_module_world}
and CanonicalExternalModuleWorld::{encode_artifact, decode_artifact} for the
source-free module payload. Wire definitions live in the artifact crate's
module_world.rs; ModuleTopology::validate_completed checks persisted fingerprints.

TypeStore::{project_artifact, decode_artifact} are implemented in
phalcom-semantic/src/types/artifact.rs, a child module of the store ownership
boundary. Decode returns `(TypeStore, TypeRelocation)` with a fresh store ID.
Its wire graph lives in the artifact crate's semantic/type_store.rs. All graph
edges use explicit per-arena maps. Loader counters/acceptance are not yet integrated.

Retain current parameter slot selection independently of historical parameter
versions. Scoped union order/repetition must not be normalized as global unions.
Do not serialize optional parameter/lambda source provenance into compiler authority.

Latest core/LSP compiler-only check passed (8.31 seconds); the unchanged LSP
unused import warning remains. Tests are forbidden by the current user instruction.
T4/T5 are code-implemented, not behaviorally accepted. Continue with T6; then
complete primary semantic-world materialization, query/metadata transport, writer,
loader, source-free compiler proof, and P8. Cross-process determinism has not run.

For T6, inspect ordinary intern_data/intern_variant in static_value/store.rs.
They require DataInfo/VariantInfo, not only TypeStore. The T7 ordering is explicitly
recommended rather than fixed; arrange type-only declaration/constructor products
so existing validation is available before publishing static values. If a real
cycle exists, use the plan's validated two-phase construction contract.

Final compiler-only check after codec error classification: flags cleared,
cargo check -p phalcom-core -p phalcom-lsp --lib passed in 5.30 seconds.
Only the unchanged LSP unused TypeHierarchy import warning remains.


## Requested review corrections — 2026-10-14

Inherit restored recursive MaterializationDescriptor::{Tuple,Record,Pack}; the
compiler routes them through VM::ensure_static_constant_materialized, and the VM
uses existing product builders under its temporary-root/owner-cache protocol.
StaticConstantProduct::residuals retains evaluator-owned symbolic nodes in both
publication paths. Artifact validate_type now enforces parameter kind equality
and keeps RecordRow parameters out of the TypeId domain.

All affected library and test targets compile. Regression coverage was added but
not executed; the compiler-only instruction remains in force. Correction code is
IMPLEMENTED / UNVERIFIED; P7 is still IN_PROGRESS / PARTIAL / UNVERIFIED. Continue
the existing T6 next action without treating these checks as behavioral acceptance.
Changes remain uncommitted; unrelated inherited edits and documentation moves were
preserved.


## Prepared execution route

Use [shared research/context](../Program-Architecture-and-Planning/PERF003.C2.P7-P8-execution-research.md)
and [P8 detailed execution](../P8/PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md).
No source implementation or compiler/test command ran during this planning pass.
P8 BUILD follows completed P7 interfaces; deferred acceptance still blocks P7/P8
COMPLETE. Checkpoint state remains the shared authority.


## T7.A/T6 transport continuation — 2026-10-14

Commit `5196d002` (user-authored checkpoint) carries the current artifact and
semantic source; one uncommitted correction fixes the `TypeRelocation` import
path (`crate::types::store::artifact`) in `phalcom-semantic/src/static_value/artifact.rs`.

### Established interfaces

- `phalcom-compiler-artifact::semantic::static_value::StaticValueGraphV1`:
  bounded fail-closed codec; `validate()` also returns dependency-first value
  order and rejects cycles; `validate_type_references()` cross-checks type edges.
- `phalcom-compiler-artifact::semantic::aggregates::StructuralPrimaryContractsV1`:
  versioned structural-contract DTO codec (schema 1). Owner refs are restricted
  to builtin Universe identity; component indexes preserve declaration order.
- `phalcom_semantic::static_value::{StaticRelocation, import_static_value_graph}`:
  fresh-lineage import through ordinary validated factories, including
  DataInfo/VariantInfo validation and shared pack backing preservation.
  Requires a loaded `TypeStore` + verified `TypeRelocation`, data/enum tables,
  and stable-ref resolvers for declarations/variants.

### Not established — do not assume

- T6 export projection and `StaticExportContext` do not exist. The static graph
  has an importer only; nothing projects a live store into the wire form.
- T7.A semantic-owned projection/loading is not implemented. The wire DTO covers
  a structural subset; live `DataInfo`/`EnumInfo`/`VariantInfo` additionally
  require shape/constructor templates, case environments, `DeclaredTypeFact`
  dynamic/unknown states, and visibility. Do not fill these with defaults.
- T7.B, T8–T12, writer/loader: not implemented.

### Escalation packet — T7.B compiler attachments (RESOLVED 2026-10-14)

- Task: P7.T7.B checked operational templates.
- Invariant: loader must execute checked definitions without parsing, rebinding,
  type-checking, or semantically reinterpreting dependency syntax (guide §7;
  INV-P7-14).
- Live evidence: `StaticDefinitionProduct.checked_body` contains checked
  operations and facts but its evaluator root reaches syntax; `EvaluatorBody::root_work`
  and `StaticEvalMachine::run` consume those roots; `StaticDefinitionSignature`
  retains a syntactic where-clause.
- Decision (user, this session): APPROVED — implement a source-independent
  operational-template wire design (relocated operands, stable targets,
  retained evaluation order) and the corresponding evaluator contract change.
  Empty attachment tables remain forbidden.

### Next actions, in order

1. T7.A semantic-owned projection/loading with exhaustive DataInfo/
   EnumInfo/VariantInfo DTO coverage; keep factory-based validation.
2. Add T6 export projection (`StaticExportContext`) mirroring the importer's
   family coverage; compile the coherent seam; determinism evidence stays deferred
   to the named gate.
3. T7.B (approved): source-independent operational-template wire plus evaluator
   contract change; relocated operands, stable targets, retained evaluation
   order; no empty attachment tables.

Compiler-only checks that passed after the latest edits: artifact crate (clean),
semantic crate (14.19 s), core+lsp libs (8.22 s, inherited warning only).
Behavioral acceptance remains deferred; tests are still forbidden by user policy.


## T7.A exhaustive DTO continuation — 2026-10-14 (second session)

Status: IN_PROGRESS; the T7.B escalation above was resolved by user decision
this session. Working tree adds an **uncommitted, uncompiled** exhaustive
rewrite of `phalcom-compiler-artifact/src/semantic/aggregates.rs` on top of
commit `5196d002` plus doc updates and the one-line `TypeRelocation` import
fix. Nothing in the rewrite has passed `cargo check` yet; the passing checks
listed above predate it.

### What the rewrite contains (implemented, NOT verified)

`StructuralPrimaryContractsV1` now mirrors live shapes exhaustively instead of
the structural subset:

- `GenericSignatureV1`: owner (`ParameterOwnerV1`), parameters, parameter
  kinds/kind shapes/variances/modes, constraint shapes, and real
  `GenericConstraintV1` terms (`Subtype`/`Equivalent`/`Conforms` with
  `TraitRefV1`), replacing the old normalized-constraint sketch.
- `DeclaredTypeFactV1` with `Known(TypeTermV1)`/`Dynamic(DynamicReasonV1)`/
  `Unknown(UnknownReasonV1)` states and all nine bases; `TypeTermV1` is
  `Canonical` or `Self` (inference-local variables stay off the wire).
- Full `DataInfoV1`/`EnumInfoV1`/`VariantInfoV1`: shape templates with
  expansion contributions, constructor signatures/parameters with visibility,
  variant case environments (bindings sorted by parameter + equalities),
  variant visibility axes, payload kinds, static-domain registrations
  (`ArtifactStaticDomainId`/`ArtifactStaticConstructorId` relocations).
- `variant_contracts` is now a top-level family with cross-validated enum
  membership (every variant row belongs to exactly one enum and vice versa).
- Source spans are omitted per the DTO design rule; reconstruction sets
  `source: None`.

Removed wire-only inventions with no live counterpart: `PendingStaticValueRefV1`,
`ConstraintKindV1`, `NormalizedConstraintV1`, component `default_static`, variant
`discriminant_static`. Live `DataInfo`/`VariantInfo` carry no such fields; static
payloads arrive through the T6 static-value graph, not these contracts. Re-add a
typed pending reference only if a live field requires it — never default-fill.

Identity fields are structurally derivable and intentionally not on the wire:
`DataComponentId` = owner+index, `DataConstructorId` = owner,
`VariantConstructorId` = variant, `VariantFieldId` = variant+index,
`VariantInfo.family` and `EnumInfo.variant_families` derive from variant
selectors in declaration order (`VariantId::family()`), and
`DataInfo.constructor_template` exists iff shape contributions contain an
expansion (checker `algebraic_declaration.rs:275`). Deriving these from
persisted declaration order is canonical reconstruction, not default-filling.

### Repository facts established this session (verified by reading live source)

- Construction sites: `phalcom-semantic/src/checker/algebraic_declaration.rs`
  (DataInfo ~:287, VariantInfo ~:649, EnumInfo ~:668). Shape contributions and
  the components list are pushed in lockstep; `validate_contributions` in the
  rewritten wire enforces component contributions cover components in order.
- Table insertion APIs: `DataSemanticTable::insert_data(Arc<DataInfo>)`,
  `EnumSemanticTable::{insert_enum, insert_variant}`; iteration via
  `ParentMap::iter()`/`iter_local()`.
- `GenericSignature::validate_publishable(&TypeStore)` is the reusable
  publication validator (owner/contiguity/mode/kind/variance checks, rejects
  inference variables) — run it on import-reconstructed signatures.
- Stable-identity round-trips exist in
  `phalcom-semantic/src/metadata/stable_identity.rs`
  (`from_stable_declaration/module/callable/impl/dispatch_side`).
- No other file references the old subset DTO names — the rewrite is currently
  self-contained; first `cargo check -p phalcom-compiler-artifact --lib` must
  confirm macro/import fit (`record!`/`tagged!` from `module_world`).

### Next actions, in order

1. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-compiler-artifact --lib`
   for the rewritten aggregates codec; fix mechanical issues only.
2. T7.A semantic side (plan file list: `phalcom-semantic/src/artifact/import.rs`,
   `export.rs`; register under `artifact/mod.rs`): project live
   DataSemanticTable/EnumSemanticTable/VariantInfo into the wire through a
   closed `TypeExportContext` (tags `DataContract=5`/`EnumContract=6`/
   `VariantContract=7` exist for root registration), and reconstruct through
   the ordinary constructors with `validate_publishable`, exact-case store
   cross-checks, and derived identity reconstruction; insert via the table APIs
   above. Do not run formation or solving.
3. T6 export projection (`StaticExportContext`) mirroring the importer's family
   coverage; needs `pub(crate)` arena iteration accessors on
   `StaticValueStore` (store fields are private to `static_value::store`).
4. T7.B per the approved decision.

Check policy is unchanged: compiler-only, serial, flags/wrapper cleared.
Preserve unrelated inherited work; no commit/push was requested.
