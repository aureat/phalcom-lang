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
