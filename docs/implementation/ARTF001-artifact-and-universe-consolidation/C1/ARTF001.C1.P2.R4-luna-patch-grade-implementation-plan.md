---
id: ARTF001.C1.P2.R4
kind: implementation-plan
program: ARTF001
checkpoint: ARTF001.C1
phase: P2
revision: R4
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
branch: feat/artifact-universe-consolidation
planning_revision: 2d3cf9ce2ca66d5da45aae8b26b765179efbddc1
p7_merge_commit: f1d2b6dc7b70fd0bab7c3f4abe5f2fe4def3645a
p7_reference_head: 077d93d3bfcd5d3bca0127c4aac18abb55c2ed94
pre_merge_consolidation_parent: cf6de4bfff06b81994c66592cd3da8756e961e6c
supersedes: ARTF001.C1.P2.R3
target_status: COMPLETE
target_completion: IMPLEMENTED
---

# ARTF001.C1.P2.R4 — Final P2 Closure: Current-Schema P7 Recovery, Ordinary Session Publication, Consumer Consolidation, and Certification

## 0. Executive mandate

`ARTF001.C1.P2.R4` is the final implementation and certification plan for P2.

It supersedes `ARTF001.C1.P2.R3` as the active execution plan while preserving P2.R3 as implementation history and as a requirements inventory. The certified PERF003 P7 branch is merged into `feat/artifact-universe-consolidation`, and the current module, semantic, query, and canonical-image machinery is present in ancestry. QueryImage V3 was the adapter at R4 plan creation; the accepted R4.T2.6b amendment upgraded the current contract to QueryImage V4 and query fingerprint ABI V3. Current execution requirements below use those amended versions.

R4 therefore MUST NOT continue the old family-by-family transplant strategy. Its job is to:

1. make the merged artifact lifecycle internally current and truthful;
2. execute tests again and use them as the primary correctness oracle;
3. recover and recertify the complete P7 lifecycle on current V2/V4/V3 authority;
4. prove complete current native semantic reconstruction rather than staged DTO adequacy;
5. finish dependency-closed current query authority;
6. certify `CanonicalUniverseFormalImage -> SemanticWorkspaceSession::from_image` as the ordinary imported-session path;
7. prove local overlay, invalidation, rollback, and source-free operation through that ordinary session;
8. eliminate or subordinate the parallel `ArtifactImportTransaction -> ImportedCompilerImage` semantic pipeline so P2 ends with one production consumer architecture;
9. re-establish exact allocation, hostile-input, retry, source-denial, compiler-parity, package, and runtime evidence at an R4 revision;
10. freeze the consumer-side contract and prepare a narrow, explicit P3 handoff.

P2 MUST NOT be marked complete until every P2-owned gate in this plan has executed successfully or an explicitly inherited non-P2 baseline failure has been reproduced, isolated, and recorded.

The final R4 end state is:

```text
source-backed current canonical semantic generation
        |
        | existing current/P7 producer machinery
        v
ModuleWorldV2 + SemanticWorldV4 + QueryImageV4 + Metadata
        |
        v
.telosc
        |
        | hostile validation + exact allocation transaction
        v
current native SemanticSnapshot + current query authority
        |
        v
CanonicalUniverseFormalImage
        |
        v
SemanticWorkspaceSession::from_image
        |
        +--> immutable canonical query/base authority
        |
        +--> ordinary mutable local source overlay
        |
        +--> ordinary invalidation/recomputation/rollback
        |
        v
normal compiler / lowering / VM consumers
```

P3 must inherit this as a stable consumer contract. P3 must not inherit unresolved importer architecture, source-free semantic gaps, competing consumer pipelines, undefined query persistence rules, or unverified session-publication semantics.

---

# 1. Repository baseline and authoritative facts

## 1.1 Pinned branch and revisions

At planning time the consolidation branch is:

```text
branch:
    feat/artifact-universe-consolidation

HEAD:
    2d3cf9ce2ca66d5da45aae8b26b765179efbddc1
    docs(artifact): record Pack owner consultation ruling
```

The full P7 ancestry merge is:

```text
f1d2b6dc7b70fd0bab7c3f4abe5f2fe4def3645a
merge(perf): integrate certified P7 universe artifact branch

parents:
    cf6de4bfff06b81994c66592cd3da8756e961e6c
    077d93d3bfcd5d3bca0127c4aac18abb55c2ed94
```

The P7 reference branch head is therefore already in consolidation ancestry.

Required invariant:

```bash
git merge-base --is-ancestor   077d93d3bfcd5d3bca0127c4aac18abb55c2ed94   HEAD
```

MUST succeed throughout R4.

### Consequence

There is no P7 cherry-pick backlog.

```text
remaining P7 ancestry merge work = 0
```

Do not re-open the old “port all useful P7 commits” question.

---

## 1.2 Current production artifact lifecycle

Current `phalcom-core/src/modules/canonical_artifact.rs` already uses:

```text
MODULE_WORLD_SCHEMA_V2
SEMANTIC_WORLD_SCHEMA_V4
QUERY_IMAGE_SCHEMA_V4
```

and the current producer already calls:

```text
project_module_world(...)
export_semantic_world_v4(...)
export_query_image_v4(...)
```

The current loader already performs:

```text
TeloscContainerView::parse_with_allocation_budget
    ->
ModuleWorldV2::decode_with_allocation_budget
    ->
load_module_world_with_allocation_budget
    ->
SemanticWorldV4::decode_with_allocation_budget
    ->
QueryImageV4::decode_with_allocation_budget
    ->
V4 product-reference validation
    ->
V4 portable structural fingerprint validation
    ->
validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget
    ->
reconstruct_query_image_v4_with_allocation_budget
    ->
metadata decode
    ->
CanonicalUniverseFormalImage::from_loaded_artifact
    ->
LoadedUniverseArtifact
```

This is the canonical recovery baseline. Do not replace it with another loader before it is certified.

---

## 1.3 Current schema authority

Current artifact compatibility constants include:

```text
MODULE_SEMANTIC_ABI_V2   = 2
SEMANTIC_ARTIFACT_ABI_V3 = 3
QUERY_FINGERPRINT_ABI_V3 = 3
```

Current wire families include:

```text
ModuleWorldV2
SemanticWorldV4
StructuralPrimaryContractsV2
CallableAnalysisRecordV2
ModuleExecutionAuthorityV1
QueryImageV4
```

Structural V2 includes independent, project-qualified:

```text
intrinsic_static_domains
intrinsic_type_owners
```

The declaration-shell field named `intrinsic_type_owner` is intentionally retained as the three-case redundant runtime-owner witness and must equal `binding.runtime_owner`. It is not the seven-case type-member owner.

The seven current intrinsic type-member owners remain:

```text
Tuple
Record
TypePack
LabeledTypePack
CompleteTypePack
RecordRow
UnorderedShape
```

Pack remains a runtime owner and semantic application form. Under the current specification it is not an eighth direct type-member owner.

This ruling is closed for P2.

---

## 1.4 Current ordinary session bridge exists

Current `phalcom-semantic/src/session.rs` provides:

```rust
pub fn from_image(
    workspace: WorkspaceId,
    image: Arc<CanonicalUniverseFormalImage>,
) -> SemanticWorkspaceSession
```

It already installs:

```text
WorkspaceModuleSession::with_external_modules(...)
SemanticDb::with_canonical_image(...)
snapshot.store
snapshot.static_values
base declarations
base hierarchy
base dispatch
base callable signatures
base data semantics
base enum semantics
base associated surfaces
base type-member declarations
base enum requirements
last_snapshot
last_known_good
```

Therefore R4 does not need to invent a new imported-session abstraction unless current tests prove `from_image` structurally incapable of satisfying P2.

The expected direction is to certify and complete `from_image`, not replace it.

---

## 1.5 Current parallel staging pipeline still exists

`phalcom-semantic/src/artifact/mod.rs` still contains:

```text
ArtifactImportTransaction
ImportedCompilerImage
```

with a large family of modular imported sidecars, including current-main adaptations developed before the ancestry merge.

Its `commit()` explicitly returns staging data and does not publish a workspace session.

This pipeline is now a consolidation problem.

It may contain valuable current-main adapters and focused validations, but P2 MUST NOT end with two complete production semantic artifact consumers.

R4 owns final disposition.

---

## 1.6 P7 certification machinery is valuable authority

P7 previously certified:

```text
source-free artifact build/load
same-image deterministic serialization
whole-load exact allocation accounting
N success
N-1 typed failure
retry success
section-local allocation rollback
prepublication failpoints
query dependency closure
foreign/stale/cyclic query rejection
source/artifact compiler parity
negative diagnostic parity
fresh-process source denial
SlotMap source-free package/runtime behavior
hostile container/semantic/query cases
```

Historical measured allocation N was:

```text
373,069,634
```

This value is historical evidence only.

R4 MUST discover and certify the current N. The old numeric value is not an acceptance target.

---

# 2. Operating-mode amendment: the no-test period is over

## 2.1 Previous restriction

During the large P7 ancestry merge and initial API adaptation, the active handoff prohibited tests so implementation could recover a compileable integrated state without being dominated by slow or stale test execution.

That restriction is now terminated.

## 2.2 R4 rule

From the first R4 implementation commit onward:

> Tests are mandatory evidence. `cargo check` alone cannot close an R4 behavioral task.

Every implementation slice MUST use the strongest practical evidence available:

```text
cargo check
    -> compile gate only

cargo test ... --no-run
    -> test-target compile gate only

focused test
    -> local behavioral acceptance

vertical artifact test
    -> end-to-end lifecycle acceptance

fresh-process test
    -> source-independence acceptance

allocation census
    -> transaction-accounting acceptance

real package/runtime smoke
    -> user-visible compiler functionality acceptance
```

## 2.3 Required execution discipline

Do not run the slowest broad suite after every tiny edit.

Use this progression:

```text
compile affected crate
    ->
compile affected tests with --no-run
    ->
run focused discriminator
    ->
run nearest integration target
    ->
run R4 wave gate
```

A patch may be committed only when its defined focused gate passes or the commit is explicitly labeled as an intermediate mechanical integration commit in an otherwise continuously passing local stack.

Do not push knowingly broken test targets unless the branch is already blocked by a reproduced inherited failure and that exact blocker is recorded.

---

# 3. R4 scope

R4 owns all remaining P2 consumer-side work required before P3.

## 3.1 In scope

R4 includes:

- current header/schema compatibility correction;
- current test-harness compilation;
- complete V2/V4/V3 artifact lifecycle recovery;
- current declaration/alias/hierarchy equivalence;
- complete current callable semantic authority;
- witness/proof/derived-conformance authority;
- source-free static execution;
- module execution authority;
- query product reconstruction;
- query dependency closure;
- required-root completeness;
- live fingerprint recomputation and portable evidence validation;
- exact current semantic snapshot;
- immutable canonical image;
- ordinary `SemanticWorkspaceSession::from_image`;
- local overlay/invalidation/recompute/rollback;
- exact allocation accounting;
- hostile/corruption validation;
- fresh-process source denial;
- source/artifact compiler parity;
- negative diagnostic parity;
- real package compilation and runtime smoke;
- consolidation of duplicate importer architecture;
- final manifest closure;
- schema/ABI freeze;
- P2 walkthrough and P3 handoff.

## 3.2 Out of scope

R4 does not own:

- replacing `packages/universe` source location;
- final toolchain cache policy;
- universal artifact lookup across all CLI commands;
- final registry behavior;
- generalized arbitrary-package incremental artifact compilation;
- portable executable bytecode;
- P5/P6 cutover work;
- broad performance optimization unrelated to correctness;
- new direct Pack type-member semantics.

Those belong to later phases unless a P2 correctness requirement directly depends on them.

## 3.3 P3 boundary

R4 MUST leave P3 with a producer-focused responsibility.

P3 must not need to design or repair:

```text
native artifact import
semantic authority reconstruction
static execution import
witness/proof reconstruction
query persistence policy
query live fingerprint semantics
artifact-backed session creation
project identity binding
rollback
allocation accounting
source denial
```

If P3 still needs to answer one of those, P2 is not closed.

---

# 4. Architectural invariants

These invariants govern every R4 patch.

## R4-INV-01 — Current semantic authority wins

Current consolidation/main semantic structures, checker/lowering semantics, project identity, and runtime behavior are authoritative.

P7 implementation algorithms are preserved and adapted to them.

Do not regress current semantic APIs merely to make old P7 code compile.

## R4-INV-02 — One complete production consumer

At P2 completion there is one production semantic artifact import architecture.

Temporary duplication is allowed during R4 recovery only.

Final state:

```text
portable bytes
    ->
validated current DTOs
    ->
current native semantic snapshot/query image
    ->
CanonicalUniverseFormalImage
    ->
SemanticWorkspaceSession
```

There must not also be an independent complete `ImportedCompilerImage` semantic authority used by different consumers.

## R4-INV-03 — No fabricated semantics

Forbidden examples:

```text
ordinary declaration fallback for intrinsic source declaration
invented empty callable phase evidence
invented empty runtime requirements
invented query dependency lists
invented AST for imported static definitions
invented package IDs
invented owner paths
silently dropped required query roots
```

Every compiler-critical value must be:

```text
WIRE
DERIVED_FROM_VALIDATED_WIRE
TRUSTED_CURRENT_COMPILER_INTRINSIC
PROCESS_LOCAL / INTENTIONALLY_OMITTED BY EXPLICIT CONTRACT
```

## R4-INV-04 — Pack and TypePack remain distinct

`IntrinsicRuntimeOwner::Pack` does not imply `IntrinsicTypeOwner::TypePack`.

No eighth intrinsic type owner is added under P2.

Any new direct Pack projection contract requires a later semantic/specification amendment.

## R4-INV-05 — Project identity is retained exactly

Artifact import must preserve the existing project graph and stable project/module/declaration/callable identity contract.

Do not derive durable identity from:

```text
row position
display path
basename
last declaration component
newly allocated project ID
```

Use the existing `ProjectUniverse`, stable artifact refs, and exact resolver receipts.

## R4-INV-06 — Canonical image is immutable base authority

A loaded `CanonicalUniverseFormalImage` is immutable canonical semantic/query authority.

Local source updates are overlays.

Canonical roots are immutable leaves and cannot be:

```text
overwritten
retired
invalidated into local mutation
copied into local query state merely to use them
```

## R4-INV-07 — Ordinary session mechanics own local work

After `SemanticWorkspaceSession::from_image`, local compilation uses ordinary:

```text
WorkspaceModuleSession
SemanticDb
query scheduler
dependency index
update/invalidation path
snapshot publication
rollback
```

Do not create an artifact-only incremental compiler.

## R4-INV-08 — Portable evidence is not live evidence

Artifact fingerprints prove transport integrity and canonical portable evidence.

The receiving compiler recomputes current live product/input fingerprints after relocation.

Never assign a portable artifact fingerprint directly as a live DB fingerprint unless the current fingerprint function explicitly defines them as identical by contract.

## R4-INV-09 — Required canonical roots fail closed

For the final required canonical query profile:

```text
required root + incomplete dependency closure
    -> export/import failure
```

not silent omission.

Optional profile behavior, if any remains, must be explicit.

## R4-INV-10 — Exact allocation remains exact

For measured transaction stages:

```text
independently observed allocator requested bytes
==
AllocationBudget ledger requested bytes
```

No heuristic/estimated charging.

No “close enough”.

## R4-INV-11 — Transaction failure exposes nothing partial

Before commit:

```text
module world
semantic snapshot
query image
metadata
session candidate
```

remain private.

Failure must not publish a partially usable canonical image or mutate caller-owned project/session state without rollback.

## R4-INV-12 — Source denial must be fresh-process

Source-free claims require a new process that has not previously initialized the canonical Universe source baseline.

Same-process tests are useful but insufficient.

## R4-INV-13 — P7 certification code is an asset

Do not delete or substantially simplify P7 allocation census, failpoints, query closure checks, or parity helpers before their current-schema replacements pass.

---

# 5. Required final architecture

P2.R4 completion requires the following authoritative flow:

```text
CompilerArtifactPayload
    |
    +-- TeloscHeaderV2
    |       current compatibility constants
    |
    +-- ModuleWorldV2
    |
    +-- SemanticWorldV4
    |
    +-- QueryImageV4
    |
    +-- MetadataSectionV1
    |
    v
hostile validation + cumulative AllocationBudget
    |
    v
current native module world + ProjectUniverse
    |
    v
current native SemanticSnapshot
    |
    +-- TypeStore
    +-- StaticValueStore
    +-- DeclarationTypeTable
    +-- TypeAliasTable
    +-- MapTypeHierarchy
    +-- callable authority
    +-- conformance/witness/derived authority
    +-- static execution authority
    +-- module execution authority
    +-- diagnostics/proofs/explanations
    +-- all required derived indexes
    |
    v
current live query reconstruction
    |
    +-- exact owner relocation
    +-- typed product selection
    +-- portable evidence validation
    +-- current input fingerprint
    +-- current product fingerprint
    +-- exact dependency edges
    +-- cycle/staleness rejection
    |
    v
CanonicalUniverseFormalImage
    |
    +-- immutable SemanticSnapshot
    +-- immutable current query roots
    +-- dependency fingerprints
    +-- module world
    |
    v
SemanticWorkspaceSession::from_image
    |
    +-- ordinary WorkspaceModuleSession
    +-- SemanticDb::with_canonical_image
    +-- local source overlay
    +-- ordinary invalidation
    +-- ordinary recomputation
    +-- ordinary snapshot publication
    +-- rollback / last-known-good
```

No second production semantic import model survives beside this flow.

---

# 6. Task ledger

| Task | Objective | Initial status | Exit evidence |
|---|---|---|---|
| R4.T0 | Re-pin, normalize lifecycle records, terminate no-test period | NOT_STARTED | exact SHAs, clean tree, active R4 record |
| R4.T1 | Correct compatibility identity and port test/census infrastructure to current schemas | NOT_STARTED | test targets compile; ABI tests pass |
| R4.T2 | Recover complete ModuleWorldV2/SemanticWorldV4/QueryImageV4 build/load lifecycle and close current semantic authority | NOT_STARTED | canonical artifact builds/loads; native equivalence gates pass |
| R4.T3 | Re-establish exact allocation, rollback, failpoint, and retry certification | NOT_STARTED | stage deltas zero; N/N−1/retry pass |
| R4.T4 | Finish dependency-bearing current query profile and fail-closed required-root completeness | NOT_STARTED | required root census exact; query hostile matrix passes |
| R4.T5 | Certify ordinary imported `SemanticWorkspaceSession` with overlays/invalidation/rollback | NOT_STARTED | source-free session + edits/recompile pass |
| R4.T6 | Consolidate duplicate consumer/import staging architecture and stabilize APIs | NOT_STARTED | one production consumer; disposition ledger has zero unknowns |
| R4.T7 | Fresh-process source denial, compiler parity, diagnostics parity, real package/runtime functionality | IN_PROGRESS | all source-denial, parity, SlotMap/parse, and CLI gates pass |
| R4.T8 | Broad regression, schema freeze, manifest closure, cleanup | NOT_STARTED | broad checks run; P2-owned failures zero |
| R4.T9 | P2 acceptance, walkthrough, checkpoint closure, P3 handoff | NOT_STARTED | P2 COMPLETE; P3 contract explicit |

Tasks are ordered. Do not begin destructive consolidation in T6 before T2–T5 are behaviorally green.

---

# 7. R4.T0 — Re-pin, normalize records, and establish the test-first closure baseline

## Purpose

Convert the repository from “merge recovery continuation” to “P2 closure and certification”.

## T0.1 Re-pin repository state

Run:

```bash
git fetch origin
git switch feat/artifact-universe-consolidation
git status --short --branch
git rev-parse HEAD
git rev-parse origin/feat/artifact-universe-consolidation
git rev-parse origin/perf/universe-characterization
git merge-base --is-ancestor   origin/perf/universe-characterization   HEAD
```

Expected starting revision:

```text
2d3cf9ce2ca66d5da45aae8b26b765179efbddc1
```

If branch HEAD changed, stop planning assumptions, inspect all commits since `2d3cf9ce2`, and amend the R4 baseline before editing.

## T0.2 Require a clean worktree

Expected:

```text
no staged changes
no unstaged changes
no untracked implementation debris
```

Do not overwrite user changes if present.

## T0.3 Update checkpoint chronology

`CHECKPOINT.md` currently retains stale prose saying the P7 merge is still an uncommitted `MERGE_HEAD`.

Correct the active state to:

```text
P7 ancestry merged at f1d2b6dc7
current branch head <actual R4 head>
P2.R4 active
tests mandatory
Pack consultation closed
P2 remains open
P3 blocked pending R4 acceptance
```

Do not rewrite historical consultation entries. Correct only active state and task ledger.

## T0.4 Record no-test revocation

Add an explicit lifecycle statement:

```text
The merge-period test prohibition is ended.
R4 requires executed test evidence for every behavioral acceptance gate.
```

## T0.5 Preserve historical R3

Do not delete R3.

Mark:

```text
P2.R3: SUPERSEDED BY P2.R4
```

for active execution only.

## Verification

```bash
git diff --check
git status --short
```

### T0 exit gate

- exact current branch SHA recorded;
- P7 ancestry proven;
- R4 active in checkpoint;
- no-test restriction revoked;
- worktree clean after commit.

Suggested commit:

```text
docs(artifact): start P2.R4 closure and certification
```

---

# 8. R4.T1 — Correct current compatibility identity and port P7 certification code to ModuleWorldV2/SemanticWorldV4/QueryImageV4

## Purpose

Ensure the current artifact truthfully labels its schema/ABI contract and that all defining P7 tests compile against the current production path.

## T1.1 Replace stale core ABI literals

At R4 plan creation, `phalcom-core/src/modules/canonical_artifact.rs` wrote and validated old literals:

```text
module_semantic_abi = 1
semantic_artifact_abi = 1
query_fingerprint_abi = 2
```

Those values are incorrect for the amended ModuleWorldV2/SemanticWorldV4/QueryImageV4 payload.

Change both producer and reader to canonical constants:

```rust
MODULE_SEMANTIC_ABI_V2
SEMANTIC_ARTIFACT_ABI_V3
QUERY_FINGERPRINT_ABI_V3
STABLE_IDENTITY_SCHEMA_V1
```

Use named constants rather than numeric literals.

### Required affected sites

At minimum inspect:

```text
build_universe_telosc
validate_reader_header
any test artifact/header builders
compatibility identity fixtures
perf_audit_p7 artifact generation
```

### Required test

Build a canonical artifact and assert:

```text
header.module_semantic_abi   == MODULE_SEMANTIC_ABI_V2
header.semantic_artifact_abi == SEMANTIC_ARTIFACT_ABI_V3
header.query_fingerprint_abi == QUERY_FINGERPRINT_ABI_V3
header.stable_identity_schema == STABLE_IDENTITY_SCHEMA_V1
```

Rehash incompatible header mutations and assert typed rejection.

## T1.2 Port core allocation census from old DTO versions

The test-only census in `canonical_artifact.rs` still references historical versions.

Port:

```text
ModuleWorldV1      -> ModuleWorldV2
SemanticWorldV3    -> SemanticWorldV4
QueryImageV1/V2/V3 -> QueryImageV4
```

Use the same production boundaries as the real loader.

For semantic import, use:

```rust
validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget(...)
```

For query import, use:

```rust
reconstruct_query_image_v4_with_allocation_budget(...)
```

Do not keep a V3/V1 census merely because it is easier to account.

## T1.3 Port `phalcom-core/tests/artifact_allocation.rs`

Remove stale assumptions that the real artifact semantic section is `SemanticWorldV3`.

The integration allocation test must:

1. build current artifact bytes;
2. decode current `ModuleWorldV2`;
3. materialize current module world;
4. decode current `SemanticWorldV4`;
5. import with exact module world + exact loaded project universe;
6. decode/reconstruct current `QueryImageV4`;
7. measure current whole loader.

No conversion through a legacy semantic world is accepted as production-equivalent evidence.

## T1.4 Eliminate hard-coded historical byte totals in tests

Historical P7 code contains assumptions such as:

```text
failed parent ledger == 377_992
```

These numbers were useful for a fixed historical layout but are brittle after current schema expansion.

Where a test is intended to prove transaction semantics rather than a permanent byte constant, compute the expected preceding-stage debit from the same measured stage census and assert that the failed child stage did not commit.

Keep exactness:

```text
expected preceding debit == actual parent debit
```

Do not replace it with:

```text
actual <= limit
```

unless the specific invariant is only upper-bound enforcement.

## T1.5 Compile all defining test targets before expensive execution

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-compiler-artifact --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib
```

Then:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --no-run
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib --no-run
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib --no-run
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact_allocation --no-run
```

If test compilation fails, classify failures:

```text
STALE_P7_TEST_API
CURRENT_SCHEMA_BUG
UNRELATED_BASELINE
```

Do not run expensive whole-Artifact tests until these targets compile.

## T1.6 Focused ABI/schema tests

Required:

```bash
cargo test -p phalcom-compiler-artifact
```

plus focused core header incompatibility tests.

The compiler-artifact suite must include current:

```text
missing required section
duplicate required section
container digest mismatch
section digest mismatch
unsupported schema
wrong project provenance
query product reference validation
cumulative decode budget behavior
```

## T1 exit gate

All true:

- current producer advertises V2/V3/V2 ABI constants;
- current reader expects V2/V3/V2;
- defining P7 test targets compile on current schemas;
- no current certification code relies on V1/V3/V1 production assumptions;
- compiler-artifact suite passes;
- no historical hard-coded byte count is used as a substitute for a current measured transaction invariant.

Suggested commits:

```text
fix(artifact): align canonical artifact compatibility ABI
test(artifact): port P7 certification harness to V2 V4 V3
```

---

# 9. R4.T2 — Recover the complete current ModuleWorldV2/SemanticWorldV4/QueryImageV4 lifecycle and close current semantic authority

## Purpose

Prove that the merged implementation is functionally complete, not merely compileable.

This task closes the semantic-authority concerns that R3 split across T1–T5 unless tests expose a real current gap.

## T2.1 Vertical artifact build/load discriminator

Run the strongest existing P7-style lifecycle test, adapted to current names/APIs.

Before building the canonical image, propagate `SemanticProductRequest::canonical_compiler_image()` into semantic session publication. The session must honor the request at the owning generation paths before optional products are computed, while existing/default session callers continue to publish full tooling products. Compiler-image publication retains all compiler-required formal and source semantic products and omits presentation-only and advisory products. Do not compute those products and strip them afterward, weaken publication validation, or change the request to `Full`.

Focused discriminator: publish the same suitable semantic input through a default/full session and a compiler-image-configured session. Confirm the full session retains advisory and presentation products, while the compiler-image session retains required formal/source semantic products, publishes no advisory or presentation products, and avoids advisory analysis. Then run the canonical image build/publication discriminator below.

Required scenario:

```text
canonical source-backed compiler image
    ->
build_universe_telosc
    ->
nonempty deterministic V2/V4/V3 artifact
    ->
load_universe_telosc_from_bytes
    ->
source-free complete CanonicalUniverseFormalImage
    ->
compile dependent inline program
```

Required assertions:

```text
correct section schemas
correct current ABIs
artifact source digest present
artifact load succeeds
image.snapshot().is_source_free_artifact_complete()
image.reader().is_ok()
query image nonempty where profile requires roots
metadata bundle valid
dependent compilation succeeds
```

Same canonical source image serialized twice MUST produce byte-identical output and identical artifact digest unless R4 proves that current metadata intentionally introduces a nondeterministic field. Such nondeterminism is itself a blocker for P3 and should not be accepted casually.

## T2.2 Declaration native equivalence

For representative source-built current declarations, compare source native authority to artifact-imported native authority.

Required fields:

```text
declaration
form
class_object_type
kind
binding.semantic_form
binding.runtime_owner
generic_binder_header
generic_signature
supertype_template
```

Required projection families:

```text
intrinsic_static_domains
intrinsic_type_owners
```

Required cases:

```text
ordinary declaration
intrinsic Tuple
intrinsic Record
intrinsic Pack
generic nominal declaration
generic applied supertype template
shell-less static-domain owner
shell-less type-member owner
same declaration carrying both standalone projection families
```

Required negative cases:

```text
class-object mismatch
kind/form mismatch
runtime witness != binding runtime owner
duplicate shell owner
duplicate static-domain owner
duplicate type-owner owner
bad stable owner
generic supertype owner mismatch
```

## T2.3 Pack / TypePack acceptance

Tests MUST prove:

```text
Pack semantic_form == Intrinsic(Pack)
Pack runtime_owner == Pack
Pack import/export round trips
Pack runtime owner does not synthesize TypePack owner
TypePack owner exists only when explicitly carried by standalone type-owner authority
```

Do not add direct Pack type-member semantics under this task.

## T2.4 Alias native equivalence

Compare all current `TypeAliasInfo` compiler-observable fields:

```text
declaration
kind
kind_shape
generic_signature
form
structural_form
dependencies
logical source span
```

Required negative cases:

```text
duplicate alias
unknown dependency
wrong project owner
kind/form disagreement
invalid source span
forbidden dependency cycle if current semantics prohibit it
```

## T2.5 Hierarchy equivalence

Prove exact current hierarchy reconstruction:

```text
bare superclass authority
generic supertype templates
owner identity
supertype type
structural form
```

Required negative cases:

```text
missing parent
foreign unauthorized parent
template owner mismatch
cycle
```

## T2.6 Module execution authority equivalence

Build a fixture with:

```text
one whole-module import
one selective import with multiple items
one re-export with multiple items
one zero-binding/empty-item import site
```

Round trip and compare exact native:

```text
ModuleImportPhaseProduct
CallableAnalysis.observed_import_phases
RuntimeDependencyRequirement
```

For source-import binding ordinals prove exact reconstruction of:

```text
(ImportSiteLocalId, ImportBindingLocalId)
```

Required invalid cases:

```text
duplicate ordinal
missing ordinal
out-of-range ordinal
wrong importing module
missing module phase product
callable observation not represented by module product
phase disagreement
invalid coverage
bad runtime requirement importer
bad runtime requirement dependency
read-target/dependency mismatch
invalid runtime provenance
```

## T2.6a Canonical runtime-requirement identity (advisory amendment, 2026-10-20)

Runtime requirement uniqueness uses semantic obligation identity: importer,
dependency, range, reason, read target, provenance target, required product,
and required timing. The provenance consumer must equal the importer.
Producer identifies the observation mechanism and does not create another
obligation. Owning producers suppress redundant observations; transport rejects
remaining duplicate obligations without dropping records.

Distinct reads, targets, products, timing, and reasons at one source span must
round-trip. Canonical ordering covers the complete record, including consumer
and producer. The current wire carries every identity field, so this correction
requires no schema or ABI bump before R4.T8 freezes the contract.

Acceptance requires zero duplicate obligations and producer-only redundant
observations, same-span distinct-obligation native round-trip, and deterministic
runtime requirement ordering. Capture all remaining coarse-location collisions
before another producer correction. Do not add global producer precedence or a
final blanket deduplication pass.

## T2.6b Query-product family and final declaration-shell consistency (advisory amendment, 2026-10-20)

The audit ruling resolves the alias query blocker by separating logical query
identity from physical artifact row identity. This task ID is used because the
plan already assigns T2.7 to witness/derived/proof authority.

Introduce an exhaustive `PortableQueryProductFamilyV1` for every physical
product table currently addressable by a reusable query root. `QueryProductRef`
stores that family plus its row index. QueryImage advances from V3 to V4 and
`QUERY_FINGERPRINT_ABI` advances from V2 to V3 before the R4 freeze. Do not
change ModuleWorldV2, SemanticWorldV4, semantic artifact ABI V3, stable identity,
or the outer container schema absent a separate demonstrated dependency.

Use an explicit fail-closed logical-query/family compatibility function. The
current product mapping includes Module; NominalDeclarationShell; TypeAlias;
CallableDefinition; CallableSignature; DispatchSurface; TraitSurface;
DataDeclaration; EnumDeclaration; AssociatedSurface; TypeMemberDeclaration;
and ModuleStaticBindings. `DeclarationShell` permits exactly
NominalDeclarationShell or TypeAlias. Other mappings are one-to-one or share
their existing physical table as listed by the exhaustive compatibility match.
For declaration-shell roots validate family-specific bounds and exact stable
owner equality. `StructuralPrimaryContractsV2` rejects an owner represented by
both nominal shell and alias authority, whether or not a query root references
it.

Portable declaration-shell input/product fingerprints cover the selected
physical family and full selected row content. Dependency evidence uses the
same family-aware product fingerprint. A family change must change the digest.
After reconstruction, continue recomputing live SemanticDb fingerprints from
the native `TypeDeclarationShell`; never install portable digests as live DB
fingerprints.

Static-definition alias links require their exact `TypeAliasRecordV1` row,
matching record ID, and validated `StableOwnerCatalog` owner; they do not require
or synthesize a nominal shell. A conflicting nominal shell is rejected by the
semantic-world exclusivity invariant.

Package remains a source semantic repair: `reflection/package-object.ph`
explicitly imports sibling `Module`; do not widen the canonical prelude. When a
successful generation omits a blocked nominal declaration, permanently retire
its `DeclarationShell` query with dependent state. Immediately before query
freeze/publication, require every reusable declaration-shell query to have
exactly one matching final nominal or alias authority and a matching native
product variant. Reject neither/both/mismatched cases.

The required-root profile depends on logical `QueryKind`, not physical product
family. A retained reusable alias `DeclarationShell` root must not be reverse-
pruned solely because its product family is TypeAlias.

Focused acceptance covers nominal and alias round trips; both wrong-family
pairs; wrong row/index/owner and missing authority; nominal/alias owner
ambiguity; mutations to nominal and alias input/product fingerprints and alias
dependency evidence; alias-only static-definition authority; blocked-query
retirement; Package final nominal authority; and exact equality between every
final shell query variant and snapshot authority. Run the compiler-artifact
suite and narrow semantic query/static-import/session regressions before the
single expensive allocation gate.

## T2.7 Witness / derived / proof authority

The merged P7 importer should be treated as the candidate complete implementation.

Do not reimplement these families unless a focused test fails.

Required proof:

```text
source witness plan
    ==
compiler-observable imported witness plan

source derived conformance rule index
    ==
compiler-observable imported derived rule index
```

Test representative:

```text
selected requirement callables
associated-type plan
conditional fallback
evidence sharing
nested evidence
evidence owner validation
evidence cycle rejection
```

## T2.8 Source-free static execution

Prove that current imported static definitions execute through one evaluator semantics.

Required:

```text
source checked static execution
vs
artifact canonical operational execution
```

Compare:

```text
result value
value fingerprint
dependencies
nested static applications
associated resolution
failure/diagnostic behavior
```

Add/retain a discriminator proving the artifact path does not consult source AST.

Forbidden:

```text
fabricating StaticDefinitionBodyRepr AST
second artifact-only evaluator
source fallback after artifact load
```

## T2.9 Callable semantic completeness

Use compiler-observable parity as the primary proof.

Required representative callable analyses must cover:

```text
call specialization
conditional dispatch
conformance evidence
trait dispatch
associated resolution
callable reference resolution
family applications
flow graph
iteration lanes
positional expansion
sequence literals
match lowering
pattern resolution
local proof graph
return/throw flow
return validation
diagnostics
explanation graph
observed import phases
```

The imported canonical profile must have:

```text
deferred required callable authority count = 0
```

for publishable canonical analyses.

Do not keep staged “deferred” sidecars as the final compiler authority.

## T2.10 Manifest closure

Use `artifact/manifest.rs` as an executable completeness oracle.

Required:

```text
SNAPSHOT_FIELD_MANIFEST entries = all current SemanticSnapshot fields
SESSION_BASE_FAMILY_MANIFEST entries = all current imported base families
QUERY_KIND_MANIFEST entries = all current QueryKind variants
COMPILER_ATTACHMENT_MANIFEST entries = all compiler attachment families
IMPORTED_ARTIFACT_FAMILY_MANIFEST entries = all imported artifact families
```

Existing current counts are a planning baseline only; use current enum/struct definitions at execution time.

Unknown classifications:

```text
0
```

Any new field added during R4 must update the manifest in the same patch.

## T2 exit gate

All true:

```text
canonical V2/V4/V3 artifact builds
canonical V2/V4/V3 artifact loads
source-free snapshot complete
current declaration authority equivalent
current alias authority equivalent
current hierarchy authority equivalent
module execution authority equivalent
witness/derived/proof authority usable
static execution source-free
callable required authority complete
manifest unknowns = 0
```

If any family is genuinely absent, implement the smallest current-schema adapter in the canonical importer. Do not resurrect the old transplant sequence wholesale.

Suggested commit grouping:

```text
test(artifact): certify current declaration and execution authority
fix(artifact): close current semantic import gaps
test(artifact): prove current native semantic equivalence
```

---

# 10. R4.T3 — Re-establish exact allocation, rollback, failpoint, and retry certification

## Purpose

Recover the P7 exact transaction guarantee against the final current semantic path.

## T3.1 Whole-load stage census

Preserve and update the P7 stage model.

### T3.1a Imported module graph representation amendment

The user selected **frozen base with mutable overlays** after the module
materialization stage showed an unaccounted allocation delta. Implement imported
module indexes and graph storage with budget-aware frozen containers. Preserve
mutable source-backed graph behavior. Workspace edits must be represented as
mutable overlays over the imported base; do not mutate or implicitly thaw the
canonical imported graph.

This is a representation clarification for R4.T3, not an artifact wire-schema
change. Keep project/module identity canonical and singular across base and
overlay. Verify the owning lookup/update path for base-only reads, overlay
replacement, and overlay hiding/deletion. Then rerun the saved-artifact module
materialization discriminator. Continue to the later stage, N/N-1, rollback,
and retry gates only if the measured discrepancy is resolved.

### T3.1b Allocation-exact borrowed semantic validation amendment

`SemanticWorldV4` validation must operate on borrowed current-schema authority.
Production decode, validation, and import paths must not materialize a complete
`SemanticWorldV3` or another owned semantic-world copy. V3 and V4 share common
validation over borrowed family views; structural V1/V2 differences and V4-only
module execution and observed-import-phase rules remain explicit. The V1/V2
callable payload is exposed through a borrowed callable view, with V1 supplying
an empty observed-phase slice and V2 borrowing its phase rows.

The former V4-to-V3 projection may remain temporarily as a test-only
differential oracle, but must be removed from production encode/decode/import
paths. Preserve all existing common and V4-specific validation rules. No
semantic-world schema, semantic artifact ABI, stable identity, or container
schema change is authorized by this amendment.

Known unaccounted successful-path allocations must remain fail-closed at public
budgeted validation/import APIs. Any diagnostic bypass must be test-only and
must invoke the inner implementation without weakening production guards.
Measure V4 validation and V4 decode-plus-validation independently against
allocator observations before rerunning semantic import. If a residual delta
remains after removing the projection, isolate its allocation owner before
changing the ledger or advancing to N/N-1, rollback, or retry gates.

The same V4 validation path is used by semantic-world decode and semantic
import. Therefore T3 must establish exact accounting for both boundaries, not
only stage E.

At minimum measure:

```text
A   container/root + header/provenance
B   ModuleWorldV2 DTO decode
C   module native materialization
D   SemanticWorldV4 DTO decode
E   semantic native import
F   QueryImageV4 DTO decode
G   metadata DTO decode
H0  canonical semantic-world identity
H-K current query reconstruction
M   metadata bundle materialization
N-P CanonicalUniverseFormalImage publication
O   LoadedUniverseArtifact publication
```

If R4 changes actual stage boundaries, update labels but preserve equivalent accounting visibility.

For each measured stage:

```text
observed_bytes == ledger_bytes
```

## T3.2 Query reconstruction census

Preserve:

```text
Q0 closure/root validation
Q1 owner decode + QueryKey reconstruction
Q2 product + fingerprint reconstruction
Q3 dependency/entry/fingerprint assembly
Q4 loaded query-image validation
Q5 query image Arc publication
QX unclassified
```

Required:

```text
QX observed = 0
QX ledger   = 0
QX signed   = 0
```

If current V3 reconstruction legitimately adds a new phase, classify it as a named phase rather than allowing QX to absorb it.

## T3.3 Discover current exact N

Use:

```text
load_universe_telosc_from_bytes_with_allocation_summary
```

or the final equivalent production entry point.

Discover current:

```text
N_image
```

Do not encode historical N.

## T3.4 Exact N / N-1 / retry

Required:

```text
load(limit = N_image)
    -> PASS

independent observed allocator requested
    == N_image

load(limit = N_image - 1)
    -> typed cumulative allocation failure

clean retry(limit = N_image)
    -> PASS
```

Required failure type must remain stable and typed.

## T3.5 Section-local child rollback

Force a child-stage allocation failure.

Assert:

```text
failed child debit not committed to parent
parent requested == sum(successful preceding stages)
clean retry succeeds
```

Do not assert a stale historical absolute number unless that number is deliberately derived by the current test.

## T3.6 Prepublication failpoints

Retain/adapt the existing four boundaries:

```text
AfterModuleImport
AfterSemanticImport
AfterQueryImport
AfterMetadataValidation
```

For every boundary:

1. control load succeeds;
2. injected load fails;
3. no `LoadedUniverseArtifact` escapes;
4. retry succeeds;
5. retry image identity matches control.

If R4 adds a final session transaction boundary, add a session-level failpoint rather than overloading core artifact failpoints.

## T3.7 Session construction allocation boundary

R4 must explicitly define one of two contracts.

### Contract A — session construction inside artifact transaction

Then discover:

```text
N_session
```

and require N/N−1/retry through usable `SemanticWorkspaceSession`.

### Contract B — artifact load transaction ends at immutable image

Then prove:

```text
CanonicalUniverseFormalImage is complete atomic product
SemanticWorkspaceSession::from_image is separately infallible
or has its own fully tested failure/rollback contract
```

Do not leave the transaction boundary undocumented.

Prefer Contract B if `from_image` remains a deterministic in-memory view/construction over already-owned current native products and can be made effectively infallible without source access.

## T3 exit gate

Required:

```text
all named stage signed deltas = 0
QX = 0
N exact = PASS
N-1 = typed FAIL
retry = PASS
section-local rollback = PASS
prepublication retry = PASS
final image/session transaction boundary explicitly documented and tested
```

Suggested commit:

```text
test(artifact): recertify exact current universe load accounting
```

---

# 11. R4.T4 — Finish query closure, required-root completeness, and current live fingerprint semantics

## Purpose

Ensure P2 hands P3 a closed query contract rather than an exporter that silently loses required canonical roots.

## T4.1 Re-audit all current query kinds

`QUERY_KIND_MANIFEST` currently covers 46 kinds.

At execution time enumerate `QueryKind` exhaustively and ensure every kind has:

```text
portable mapping status
CanonicalQueryPersistence
artifact profile support
root/product location
required/optional status for canonical Universe profile
```

No wildcard fallback for new kinds.

## T4.2 Preserve P7 persistence dispositions

Current base policy:

```text
EligibleStructural
OpenWorld
Unsupported
```

Examples:

```text
ParsedModule                 -> source-dependent / not reusable
HierarchyEdge                -> dependency evidence, not normal root
GenericConformance           -> OpenWorld
ConformanceDispatch          -> OpenWorld
TraitDispatchBucket          -> OpenWorld
CallableBody                 -> OpenWorld
CheckedStaticBody            -> OpenWorld
AttributeArgument            -> Unsupported unless separately proven
```

Do not promote open-world products merely to increase hit rate.

## T4.3 Define canonical required-root profile

Add a single explicit current profile describing roots required for a valid canonical Universe compiler image.

At minimum evaluate these existing reusable families:

```text
UnlinkedInterface
ResolvedImports
LinkedInterface
DeclarationShell
CallableDefinition
CallableSignature
FieldSignature
DeclarationSurface
TraitHeader
TraitSurface
DataDeclaration
EnumDeclaration
AssociatedSurface
TypeMemberDeclaration
ModuleStaticBindingCatalog
ModuleStaticBindings
```

Do not mechanically mark every eligible structural query required.

The profile must be based on actual compiler/bootstrap requirements.

## T4.4 Remove silent loss of required roots

Current `export_query_image_v4` reverse-prunes candidates until closure is stable.

For a root classified as REQUIRED:

```text
missing portable dependency
missing live dependency fingerprint
unrepresentable owner
missing product row
```

MUST fail export.

It MUST NOT silently disappear.

Optional eligible roots may be pruned only if the profile explicitly permits it and the artifact remains complete without them.

## T4.5 Recompute live fingerprints

During V3 import:

```text
portable fingerprint
    -> validate against artifact structural product

live product
    -> reconstruct in current native snapshot

live fingerprint
    -> recompute with current db::fingerprint functions
```

Validate portable claims and current live equality expectations separately.

Never make current fingerprint functions match serialized bytes merely to pass artifact tests.

## T4.6 Dependency closure validation

For every retained root prove:

```text
dependency owner exists
dependency evidence unique
observed dependency fingerprint matches current reconstructed product
dependency graph closed
dependency graph acyclic
root input fingerprint current
root product fingerprint current
```

Required hostile tests:

```text
duplicate relocated key
missing dependency
stale dependency fingerprint
wrong owner
foreign project owner
cycle
unsupported root
open-world root presented as canonical reusable
bad product family reference
bad stable aggregate owner
```

## T4.7 Query publication parity

For each required root family, compare:

```text
source canonical DB product
artifact image product
```

using current compiler-observable equality/fingerprint.

Also prove `SemanticDb::with_canonical_image` reports:

```text
is_current(root) == true
validate_ready(root) == true
product_origin(root) == Canonical(...)
local_query_state(root) == None
```

## T4 exit gate

Required:

```text
all current QueryKind values classified
required canonical root set explicit
required root count matches exported/imported root count
required root silently pruned = 0
open-world root published canonical = 0
unsupported root published canonical = 0
dependency closure complete
dependency cycles = 0
portable/live fingerprint checks pass
```

Suggested commits:

```text
feat(artifact): make canonical query profile fail closed
test(artifact): certify current query closure and fingerprints
```

---

# 12. R4.T5 — Certify ordinary imported SemanticWorkspaceSession behavior

## Purpose

Close the final compiler-integration boundary: a loaded artifact must become an ordinary semantic workspace base, not a special frozen compiler side path.

## T5.1 Use `SemanticWorkspaceSession::from_image`

Default architecture:

```rust
SemanticWorkspaceSession::from_image(workspace, image)
```

is the production bridge.

Do not add another imported-session type unless tests demonstrate a hard architectural deficiency.

## T5.2 Prove zero canonical source baseline construction

Instrument or use existing Universe source provider denial hooks so an artifact-backed session proves it does not call:

```text
canonical_universe_semantic_baseline()
SemanticWorkspaceSession::with_workspace()
canonical source parse/link bootstrap
```

while constructing the artifact-backed base.

`UniverseSourceProvider::new()` as an inert object is acceptable only if no canonical source read occurs.

## T5.3 Initial session equivalence

Immediately after `from_image`:

```text
session last snapshot == imported canonical snapshot
session last-known-good == imported canonical snapshot
session store == imported canonical store authority
session static store == imported canonical static store authority
module session uses imported external modules
db canonical image == loaded image
local query table does not duplicate canonical roots
```

Use semantic/pointer identity where the architecture guarantees sharing.

## T5.4 Immutable canonical roots

Retain and extend existing canonical image DB tests.

Prove:

```text
canonical root read succeeds
canonical root has no local query state
canonical root cannot be overwritten
canonical root cannot be retired
canonical root invalidation does not mutate the image
canonical root survives local DB rollback
```

## T5.5 Local source overlay

Create a local user module/package above the imported image.

Run ordinary session update APIs.

Prove:

```text
local source parsed/linked/checked normally
local query state published normally
local query may depend on canonical roots
canonical dependency fingerprint recorded correctly
canonical roots remain immutable
```

## T5.6 Local edit and invalidation

Modify the local source.

Prove:

```text
correct local query seeds invalidated
transitive local dependents recomputed/revalidated
unrelated local queries reused
canonical roots not recomputed
canonical roots not invalidated into local state
```

Record update statistics where useful.

## T5.7 Rollback

Inject/cause a local semantic publication failure or cancellation.

Prove:

```text
last-known-good local snapshot retained
failed candidate not published
canonical image unchanged
canonical DB roots unchanged
subsequent valid update succeeds
```

## T5.8 Multiple sessions over one image

Create two workspace sessions over the same `Arc<CanonicalUniverseFormalImage>`.

Prove:

```text
same immutable canonical image may be shared
local session A updates do not affect local session B
canonical base remains common and immutable
workspace IDs remain distinct
```

## T5.9 Package/project identity retention

If local package graphs or trusted package receipts are involved, prove that the imported `ProjectUniverse` identity is retained and local overlays extend rather than replace that identity model.

No ID rebasing.

## T5 exit gate

Required end-to-end scenario:

```text
artifact bytes
    ->
loaded image
    ->
SemanticWorkspaceSession::from_image
    ->
compile local program
    ->
edit local source
    ->
ordinary invalidation
    ->
recompile
```

with:

```text
canonical source reads = 0
canonical source compilation = 0
canonical query mutation = 0
```

Suggested commits:

```text
test(semantic): certify artifact-backed workspace session overlays
fix(semantic): close imported session publication gaps
```

---

# 13. R4.T6 — Consolidate the duplicate artifact-consumer architecture

## Purpose

P2 must end with one production consumer path before P3 begins.

This task is intentionally after lifecycle/session certification so working P7 behavior remains the oracle during consolidation.

## T6.1 Create a salvage/disposition ledger

Classify every relevant module under `phalcom-semantic/src/artifact/`, especially current pre-merge modular importers.

Use exactly one final disposition:

```text
CANONICAL
    production path owns this behavior

RETAIN_HELPER
    used by canonical importer; no independent authority

VALIDATION_ONLY
    useful validation logic but no competing semantic state

MERGED_INTO_CANONICAL
    logic moved into canonical path

SUPERSEDED_REMOVE
    redundant after parity proof

HISTORICAL_TEST_ONLY
    retained only as historical/test fixture
```

Unknown:

```text
0
```

## T6.2 Reconcile `ArtifactImportTransaction`

Choose one architecture.

### Preferred outcome

Refactor the transaction to become a transport/front-end transaction that ultimately invokes/returns the canonical imported image/session path, rather than independently reconstructing the same semantic world.

Conceptually:

```text
ArtifactImportTransaction
    -> validate bytes / identity / trusted receipts
    -> canonical native importer
    -> CanonicalUniverseFormalImage
    -> optional session construction
```

### Acceptable alternative

Retire the transaction from production if the canonical core/semantic loader fully subsumes it and public callers can migrate without architecture loss.

### Forbidden outcome

Keeping:

```text
canonical P7 importer
AND
complete independent ImportedCompilerImage semantic importer
```

as two long-term production sources of truth.

## T6.3 Dispose `ImportedCompilerImage`

If retained, it must become a thin transport/result wrapper over the same canonical imported native authority.

It must not own a second set of staged semantic sidecars used by different compiler paths.

If it is not necessary, remove it after migrating callers/tests.

## T6.4 Preserve valuable current adapters

Do not delete current modular importers blindly.

For each, identify logic not present in the merged P7 path, such as:

```text
current project-qualified identity
post-P7 module execution authority
callable observed import phases
current dispatch/specialization API shape
current structural product validation
```

Move or reuse those adaptations in the canonical importer before deleting the redundant path.

## T6.5 Consolidate query import paths

Final production query reconstruction must be one path.

Do not retain:

```text
ImportedQueryProducts publication path
AND
CanonicalSemanticQueryImage V3 reconstruction path
```

as independent semantics.

Focused validation helpers may be shared.

## T6.6 Consolidate type/static relocation

P7 `TypeExportContext`, `TypeStore` artifact relocation, and static-value relocation are expensive certified assets.

Do not replace them with a second relocation algorithm merely for modular aesthetics.

Refactor only after parity tests cover the behavior.

## T6.7 Re-run exactness after each destructive slice

After removing or merging an importer family, run the nearest:

```text
semantic equivalence test
artifact vertical test
query closure test
allocation exactness test
```

Do not batch-delete the entire staged architecture and debug globally afterward.

## T6 exit gate

Required:

```text
complete production artifact consumer paths = 1
complete production query reconstruction paths = 1
complete production type relocation paths = 1
complete production static relocation paths = 1
complete production proof/evidence reconstruction paths = 1
unknown importer dispositions = 0
```

All T2–T5 behavioral gates still pass after consolidation.

Suggested commit series:

```text
refactor(artifact): route transaction through canonical importer
refactor(artifact): consolidate current semantic import adapters
refactor(artifact): remove superseded staged consumer paths
```

---

# 14. R4.T7 — Fresh-process source denial, compiler parity, diagnostics parity, and real functionality

## Purpose

Prove that the final consolidated consumer is genuinely usable as the compiler's canonical base.

## T7.1 Fresh-process source-denial test

Strong required scenario:

### Parent process

```text
build deterministic universe.telosc
write artifact
```

### Child process

Before any canonical Universe source baseline is initialized:

```text
deny canonical Universe source access
load artifact
construct SemanticWorkspaceSession::from_image
compile local user program/package
edit local source
recompile
exit successfully
```

Required evidence:

```text
canonical Universe source parse count = 0
canonical Universe source link/bootstrap count = 0
canonical Universe semantic rebuild count = 0
```

Use explicit instrumentation/denial rather than inference from timing.

## T7.2 Positive compiler parity

Use existing `phalcom-core/src/modules/parity.rs`.

Compare source-backed vs artifact-backed:

```text
normalized compiled program
module layout
callable/lowering products
runtime dependency behavior
selected entry
```

At minimum:

```text
inline user program
multi-module fixture
representative package
SlotMap
```

A test where both modes fail is not positive parity.

### T7.2 amendment — preserve semantic imports and finalized runtime layout

The formal image's module products come from
`SemanticLinkedProgram::to_unfinalized_program()`. That compatibility view
retains every source import as a provisional runtime read and edge. It is not
the executable runtime layout: phase-aware finalization can classify an import
as interface-only, and legal semantic import cycles may therefore be absent
from the finalized runtime graph.

Artifact projection must keep the complete source import and semantic graph
authority, but take runtime read slots, import bindings, dependencies, graph,
and initialization order together from the finalized executable link. Record a
phase disposition for every source import. An interface-only import remains
available to semantic consumers without a runtime slot. A runtime read whose
eager edge is removed by availability evidence retains its runtime slot and
the exact witness for its target, product, authority, and availability time.
The loader must reject missing, mismatched, or incomplete disposition and
availability evidence. Keep strict runtime DAG validation.

Add focused checks that round-trip the `reflection.module` /
`reflection.package_object` semantic import cycle while preserving an acyclic
runtime graph; reject a version where both directions are runtime dependencies;
and reject a runtime read whose eager edge is omitted without an exact
availability witness. Then rerun the unchanged strict source/artifact parity
comparator and T7 gates. Do not normalize away module, read, dependency, or
constructor products to obtain parity.

### T7.2 amendment — persist compiler-derived availability requirements

The finalized graph also receives `MetadataTypeOwner` requirements from
`runtime_metadata_type_requirements`. These requirements are derived from
canonical semantic products during executable finalization, but the existing
semantic execution authority only persisted requirements produced by semantic
analysis. Consequently an exact finalized witness for a metadata type owner
could not be matched by the source-free loader. Do not exempt that producer:
the witness currently has no independent catalog proof, and such an exemption
could let a forged witness remove an eager runtime edge.

Compute and canonicalize the complete generated requirement set once, pass the
same set to executable finalization and semantic-world export, and persist it in
`ModuleExecutionAuthorityV1.runtime_requirements`. Keep exact witness-to-
requirement matching and strict DAG validation. The focused discriminator must
round-trip a `MetadataTypeOwner` witness with its exact persisted requirement,
reject a missing or altered requirement, and preserve the existing missing-
witness and runtime-cycle rejections. Then rerun the unchanged T7 parity gates.

### T7 semantic identity amendment — callable-local binders and trait projections

The focused iterator compile exposed a canonical type-formation defect in
`resolve_generic_signature`: an empty associated-type trait context was being
filled with the current callable's binders. For `Iterator.chain<J>`, this
changed `<J as Iterator>::Item` from the owning `Iterator<>` application to
`Iterator<J>`. Trait application is part of associated-projection identity,
so equality must retain the exact `TraitRef` rather than ignore its arguments.

The fallback is now restricted to `GenericBinderSite::Trait`, where the
binders form the trait's own generic header. Callable-local binders leave the
provided trait context unchanged. This is the narrow semantic correction
required by the T7 iterator gate; it does not alter equality orientation or
infer projection subjects backwards.

Focused proof obligations are: the non-generic `Iterator.chain<I, J>`
projections carry no trait arguments; method binders preserve a generic
`Converter<Target>` application; matching associated items permit constructor
specialization; and a caller without the equality is rejected at the
constructor call. The existing iterator-adapter regression also verifies the
valid chain and precise negative diagnostic. Run these semantic gates before
retrying the core iterator runtime gate. The core runtime and remaining T7
gates are still required for P2.R4 completion.

### T7 semantic completion amendment — discharge only proved call equalities

After the projection-identity correction, the exact `Iterator.chain` runtime
gate receives argument evidence for both constructor parameters but ends as
underconstrained. The associated-projection equality is intentionally deferred
by inference rather than solved by reversing the projection subject. Advisory
review confirms that symbolic `Self` is valid value-argument evidence here; the
remaining question is whether call completion can validate an already-proved
declaration equality after materializing the argument bindings.

Before changing completion, capture the failing call's argument knowledge and
inference terms, parameter bindings, unsolved variables, residual relation and
origin, and whether the exact lexical generic evidence proves the materialized
equality. If and only if every relevant parameter is materialized and the only
remaining blocker is a `GenericWhere` associated equality proven by the current
lexical evidence, completion may discharge that residual and proceed through
the existing specialization path. Any unsolved variable, unrelated residual,
or absent proof must remain fail-closed. Do not infer projection subjects
backwards, weaken trait-application identity, or treat an unproved equality as
selection evidence.

Focused acceptance: a positive chain from distinct iterator types with equal
items retains distinct `I`/`J` bindings; the existing missing-equality case
still reports `GenericConstraintUnsatisfied`; and a result-only generic without
value evidence remains underconstrained. Re-run the exact iterator CLI gate
after the semantic regressions. The broader iterator corpus failures for
missing array `map` and `iter` remain independently classified.

The lexical-equality completion amendment is implemented. It discharges a
residual only when the call has no unsolved variables, every generic binding
materializes, every residual is an associated equality from the same
callable's `GenericWhere`, and the existing lexical evidence proves each exact
materialized relation. The inference session retains that proof marker so a
later propagation pass cannot recreate the residual. The positive generic
chain regression preserves distinct constructor bindings; the missing-equality
regression still reports `GenericConstraintUnsatisfied`; and the result-only
generic remains underconstrained.

The exact CLI now compiles and reaches execution, where it fails because the
active runtime conformance environment has no target for the abstract
`Iterable.iterator` requirement inside the `Iterable.iter` default method.
This is a separate runtime slot/propagation incident. Do not change that
contract until the active requirement target and slot path are identified.

## T7.3 Negative diagnostic parity

Exercise representative invalid programs:

```text
constructor resolution failure
inaccessible declaration
invalid/non-exhaustive pattern
trait resolution failure
invalid static operation where relevant
```

Use existing normalization helpers:

```text
assert_semantic_diagnostics_parity
assert_compile_error_parity
```

Do not compare unstable file-system presentation fields if parity helpers deliberately normalize them.

## T7.4 SlotMap

Required source-mode regression:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime
```

Required artifact-backed equivalent must compile and execute the same runtime workload through the artifact-loaded canonical base.

Expected semantic result includes existing successful SlotMap/DenseSlotMap assertions.

## T7.5 Parse package

Run at least the known focused parse package compile regression used during prior P2 work.

Also run artifact-backed compilation if the current test harness permits the same package to select an artifact canonical base.

## T7.6 CLI smoke

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke
```

This protects ordinary source-mode behavior from artifact consolidation regressions.

If slow runtime remains, record duration, but do not weaken correctness assertions.

## T7 exit gate

Required:

```text
fresh-process source denial = PASS
source/artifact positive compiler parity = PASS
negative diagnostic parity = PASS
SlotMap source-mode = PASS
SlotMap artifact-backed = PASS
Parse focused regression = PASS
CLI smoke = PASS
```

Suggested commits:

```text
test(artifact): prove fresh-process source-free session compilation
test(artifact): recertify source and artifact compiler parity
```

---

# 15. R4.T8 — Broad regression, schema freeze, manifest closure, and cleanup

## Purpose

Prepare the repository for P3 rather than leaving it in a fragile “focused tests only” state.

## T8.1 Run broad compile gate

Required:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run
```

Unlike the earlier P7 certification, this command MUST be executed for R4 closure.

If it fails:

1. capture exact failures;
2. determine whether failures are caused by R4;
3. if claiming inherited baseline, reproduce against the appropriate clean predecessor/current stable reference;
4. record the exact count/families;
5. ensure no P2-owned test target is among the failures.

Do not simply reuse the historical “412 errors” classification.

The repository has changed materially since that evidence.

## T8.2 Run broad relevant crate suites

At minimum:

```bash
cargo test -p phalcom-type-meta
cargo test -p phalcom-compiler-artifact
cargo test -p phalcom-modules
cargo test -p phalcom-semantic
cargo test -p phalcom-core --lib
```

If full `phalcom-semantic` or `phalcom-core` suites are prohibitively slow, R4 still requires their relevant complete artifact/canonical/session partitions plus a recorded reason for any deferred unrelated corpus suite.

## T8.3 Freeze current schema/ABI contract

Record the final P2 accepted values:

```text
container schema
minimum reader schema
stable identity schema
module semantic ABI
semantic artifact ABI
query fingerprint ABI
ModuleWorld schema
SemanticWorld schema
StructuralPrimaryContracts schema
QueryImage schema
Metadata schema
native surface schema
type metadata schema
```

Add/retain incompatibility tests proving old incompatible required forms reject cleanly.

After R4 closure, P3 must treat these as frozen unless it opens an explicit versioning amendment.

## T8.4 Final manifest audit

Assert:

```text
unclassified snapshot fields = 0
unclassified session base families = 0
unclassified query kinds = 0
unclassified compiler attachments = 0
unclassified imported artifact families = 0
```

Ensure every `Wire` and `DerivedOnLoad` disposition corresponds to an actual final canonical implementation.

Delete or reclassify manifest rows that described the superseded staged pipeline only.

## T8.5 Warning and dead-code audit

R4 does not require warning-free workspace unless that is repository policy.

It does require:

```text
no dead production artifact path left only because consolidation forgot it
no stale V1/V3/V1 active imports in canonical current path
no obsolete TODO claiming P2 work that is actually closed
no comments saying tests are prohibited
```

Do not perform unrelated stylistic cleanup.

## T8.6 Determinism recheck

Build canonical artifact twice from the same current image.

Assert:

```text
bytes identical
artifact digest identical
section digests identical
required root ordering identical
```

If deterministic ordering depends on sorted maps/vectors, keep explicit canonical sorting.

## T8 exit gate

Required:

```text
all P2-owned crate/test suites green
workspace --no-run executed
any remaining broad blocker proven unrelated/inherited
schema/ABI contract recorded
manifest unknowns = 0
deterministic artifact reproduction = PASS
```

Suggested commit:

```text
test(artifact): complete P2 regression and compatibility matrix
```

---

# 16. R4.T9 — P2 acceptance, walkthrough, checkpoint closure, and P3 handoff

## Purpose

Close P2 formally and leave an implementation-grade producer contract for P3.

## T9.1 Create R4 walkthrough

Add a walkthrough document recording:

```text
starting SHA
ending SHA
commits
architecture changes
files changed
test commands
test outputs/results
current exact allocation N
source-denial evidence
query required-root census
consolidation disposition summary
remaining unrelated baseline failures
```

Do not state historical P7 numbers as current evidence.

## T9.2 Update checkpoint

Set P2:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: strongest supported verified classification
```

Only if every P2-owned gate passed.

Set R4:

```text
COMPLETE
```

Record R3 as superseded historical work.

## T9.3 Explicit P3 handoff contract

P3 receives:

```text
current source-built CanonicalCompilerImageView / equivalent
    |
    | P3: production root collection + deterministic artifact generation integration
    v
current frozen artifact schema
    |
    | R4-certified consumer
    v
CanonicalUniverseFormalImage
    |
    v
SemanticWorkspaceSession::from_image
```

P3 is allowed to focus on:

```text
producer API cleanup
canonical root collection productionization
deterministic generation command/build integration
artifact output location
packages/universe production path
cache/toolchain integration preparation
```

P3 is not allowed to reopen P2 semantic import design without a concrete R4 regression.

## T9.4 Reassess old P3 scope

Do not assume the original proposed P3 task list is still correct.

Because P7 already brought a complete producer into the branch, the P3 planner MUST first audit:

```text
what production exporter already works
what root discovery already works
what deterministic serialization already works
what only needs command/build integration
```

P3 should not reimplement `export_semantic_world_v4` or `export_query_image_v4` from scratch.

## T9.5 Final P2 completion report

Report:

```text
final branch SHA
P7 ancestry confirmation
current ABI/schema tuple
artifact byte size
artifact digest
current exact allocation N
required query root counts by family
source denial result
parity result
SlotMap result
Parse result
workspace broad result
consumer architecture count
importer disposition unknown count
manifest unknown count
```

---

# 17. Verification command matrix

The implementer should keep this matrix current and execute commands at the specified gates.

## Compile gates

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-compiler-artifact --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib
```

## Test-target compile gates

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --no-run
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib --no-run
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib --no-run
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact_allocation --no-run
```

## Artifact transport

```bash
cargo test -p phalcom-compiler-artifact
```

## Canonical image/query

Use the current exact test filters after compilation, including equivalent current tests for:

```text
canonical_image_database_reads_are_immutable_leaves_across_local_rollback
canonical_image_rejects_foreign_stale_and_cyclic_dependencies
canonical_image_reader_validates_all_final_arena_and_static_roots
canonical image freeze / dependency closure
```

## Core allocation

```bash
cargo test -p phalcom-core --test artifact_allocation -- --nocapture --test-threads=1
```

## Core failpoint retry

```bash
cargo test -p phalcom-core --lib loaded_artifact_prepublication_failures_are_retryable -- --nocapture
```

## CLI smoke

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke
```

## SlotMap source regression

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime
```

## Broad compile

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run
```

Do not assume historical test filter names not present after R4 refactors. Resolve actual names from the current test targets and record them in the walkthrough.

---

# 18. Required hostile-input matrix

R4 final certification must cover at least:

## Container/header

```text
missing MODULE_WORLD
missing SEMANTIC_WORLD
missing QUERY_IMAGE
missing METADATA
duplicate required section
container digest mismatch
section digest mismatch
unsupported container schema
wrong minimum reader schema
wrong module semantic ABI
wrong semantic artifact ABI
wrong query fingerprint ABI
wrong native surface schema
wrong native catalog fingerprint
wrong type metadata schema
wrong Universe package identity
```

## Module/project identity

```text
duplicate module identity
invalid module path
duplicate source import binding
bad external project receipt
wrong artifact digest receipt
wrong project target
conflicting persistent package identity
```

## Structural semantic authority

```text
duplicate declaration shell
wrong class-object identity
kind/form disagreement
runtime-owner witness mismatch
duplicate intrinsic static-domain row
duplicate intrinsic type-owner row
invalid shell-less owner
generic supertype owner mismatch
bad alias dependency
hierarchy cycle
```

## Module execution authority

```text
missing module phase product
duplicate module phase product
non-dense binding ordinals
out-of-range callable observed phase
wrong-module callable observation
phase disagreement
invalid runtime requirement importer/dependency
bad read target
bad provenance
```

## Type/static graphs

```text
duplicate relocated type identity
type cycle where prohibited
invalid type reference
duplicate static identity
static cycle
malformed pack/record labels
```

## Callable/proof authority

```text
missing callable owner
missing dispatch owner
missing proof/evidence owner
evidence cycle
invalid local expression owner
invalid explanation/cause ref
invalid flow edge
invalid pattern/match owner
```

## Query

```text
unsupported reusable root
open-world root marked canonical Ready
duplicate relocated query key
missing dependency evidence
stale dependency evidence
cyclic dependency
wrong product row
foreign semantic world
incompatible type prefix
incompatible static store
```

## Allocation/publication

```text
encoded size limit
decoded family limit
section-local allocation exhaustion
cumulative N-1 allocation exhaustion
failure after module import
failure after semantic import
failure after query import
failure after metadata validation
retry after each failure
session publication/update rollback
```

---

# 19. Source/native equivalence requirements

DTO equality is not the acceptance criterion.

The target is compiler-observable native equivalence.

Required comparison families:

## Structural

```text
DeclarationTypeTable
TypeAliasTable
MapTypeHierarchy
DataSemanticTable
EnumSemanticTable
AssociatedFamilyTable
TypeMember declarations
Enum requirements
```

## Callable

```text
CallableSignatureTable
CallableAnalysis
dispatch
specialization
conformance evidence
flow
match/pattern/proof
diagnostics/explanations
module import phase observations
```

## Conformance

```text
ConformanceIndex
ConformanceWitnessPlans
DerivedConformanceRuleIndex
Trait surfaces
conditional inherent authority
```

## Static

```text
StaticValueStore
StaticConstantTable / current equivalent
static execution products
module static bindings
source-free evaluation behavior
```

## Module execution

```text
ModuleImportPhaseProduct
RuntimeDependencyRequirement
```

## Query

```text
required root key set
input fingerprints
product fingerprints
dependency edges
product origin
```

---

# 20. File-level implementation map

This is a starting map, not permission for broad edits.

## `phalcom-compiler-artifact`

Likely focused files:

```text
src/header.rs
src/module_world.rs
src/semantic/aggregates.rs
src/semantic/world.rs
src/query.rs
src/payload.rs
tests/round_trip_test.rs
tests/hostile_matrix_test.rs
```

Rule:

```text
transport/validation only
no current semantic decision logic
```

## `phalcom-semantic/src/artifact`

Canonical P7/current path:

```text
export.rs
import.rs
query.rs
roots.rs
operational.rs
manifest.rs
validate.rs
```

Current modular staging path to reconcile:

```text
associated_import.rs
callable_binding_import.rs
callable_dispatch_import.rs
callable_evidence_import.rs
callable_expression_import.rs
callable_import.rs
callable_specialization_import.rs
conformance_evidence_import.rs
conformance_import.rs
constructor_import.rs
denotation_import.rs
derived_import.rs
diagnostic_import.rs
dispatch_import.rs
enum_requirement_import.rs
identity.rs
inherent_import.rs
module_static_import.rs
query_import.rs
query_policy.rs
query_product_import.rs
semantic_import.rs
static_case_import.rs
static_constant_import.rs
static_definition_import.rs
static_dependency_import.rs
static_import.rs
static_program_import.rs
trait_import.rs
type_import.rs
witness_import.rs
```

Do not mechanically preserve or delete based on file age. Classify by final authority.

## `phalcom-semantic`

Key integration:

```text
canonical_image.rs
session.rs
snapshot.rs
db/mod.rs
db/query.rs
db/fingerprint.rs
declarations.rs
type_alias.rs
static_definition.rs
static_eval/*
derived_conformance.rs
impls/*
checker/analysis/*
```

Avoid unrelated semantic rewrites.

## `phalcom-modules`

Likely:

```text
compiler_artifact.rs
project identity / ProjectUniverse APIs
WorkspaceModuleSession external-module construction
source import ordinal helpers
```

Do not create artifact-owned project identity.

## `phalcom-core`

Key files:

```text
src/modules/canonical_artifact.rs
src/modules/parity.rs
src/modules/canonical_universe.rs
examples/perf_audit_p7.rs
tests/artifact_allocation.rs
tests/core/*
```

Do not fork a second core artifact loader for R4.

---

# 21. Patch sequencing and commit discipline

Recommended commit slices:

```text
R4.0 docs(artifact): start P2.R4 closure and certification
R4.1 fix(artifact): align canonical artifact compatibility ABI
R4.2 test(artifact): port P7 certification harness to current schemas
R4.3 fix/test(artifact): close current native semantic authority gaps
R4.4 test(artifact): recertify exact load allocation and retry
R4.5 feat/test(artifact): make required canonical query profile fail closed
R4.6 fix/test(semantic): certify artifact-backed ordinary workspace session
R4.7 refactor(artifact): consolidate duplicate consumer architecture
R4.8 test(artifact): certify fresh-process parity and package runtime
R4.9 test/docs(artifact): close P2 and hand off P3
```

Exact count may change.

Rules:

- each commit should be buildable;
- each semantic behavior commit should have focused test evidence;
- do not mix unrelated formatting;
- do not globally rewrite merged P7 files for style;
- preserve blame/history where practical.

---

# 22. Escalation rules

Most R4 failures are implementation failures, not architecture consultations.

Escalate only if evidence proves one of these:

## RED-01

A current compiler-critical semantic field has no current wire representation and cannot be uniquely derived.

## RED-02

A current static operation cannot be represented by the existing canonical operational program without semantic change.

## RED-03

A source-free current compiler reader genuinely requires AST/source data that is not presentation-only.

## RED-04

Current query reuse semantics materially diverge from the P7 closed-world persistence model.

## RED-05

Exact allocation accounting cannot represent a required current allocation without changing the transaction ownership model.

## RED-06

`SemanticWorkspaceSession::from_image` cannot support ordinary local overlays without introducing competing semantic authority.

## RED-07

Consolidating the staged importer would delete current semantics not present in the canonical importer.

## RED-08

Current package/project identity cannot be preserved without replacing durable IDs.

## RED-09

A normative specification now requires direct Pack type-member operations.

Do not escalate for:

```text
ordinary compile errors
renamed types
additional fields
test fixture drift
changed allocation N
changed struct layout
warning count
stale historical test imports
```

---

# 23. Explicit prohibitions

R4 must not:

- cherry-pick P7 commits individually;
- reimplement the P7 importer/exporter from scratch;
- restore P7-era semantic types over current-main types;
- add global `ours`/`theirs` merge choices;
- default current V4 authority empty merely to compile;
- infer `TypePack` from Pack runtime ownership;
- add an eighth intrinsic type owner under P2;
- fabricate AST for imported static definitions;
- create a second static evaluator;
- create an artifact-only query database;
- create an artifact-only type system;
- create artifact-owned project IDs;
- silently prune required query roots;
- accept portable fingerprint as live fingerprint without current recomputation;
- weaken N/N−1;
- replace exact allocation accounting with estimates;
- remove P7 failpoints before current replacements pass;
- claim source-free behavior from a process that already initialized the source baseline;
- delete modular importers before adaptation salvage;
- keep two complete production importers at P2 closure;
- start P3 while any R4 P2-owned gate is open.

---

# 24. P2.R4 measurable success matrix

| Gate | Required result |
|---|---|
| Branch ancestry | P7 head is ancestor of final R4 HEAD |
| Current producer ABI | module 2 / semantic 3 / query fingerprint 2 |
| Test-target compilation | PASS |
| Compiler-artifact transport suite | PASS |
| Canonical artifact deterministic build | PASS |
| Current artifact load | PASS |
| Source-free snapshot completeness | PASS |
| Declaration equivalence | PASS |
| Alias equivalence | PASS |
| Hierarchy equivalence | PASS |
| Module execution authority | PASS |
| Callable phase evidence | PASS |
| Witness/proof/derived authority | PASS |
| Source-free static execution | PASS |
| Callable compiler-observable parity | PASS |
| Manifest unknowns | 0 |
| Required query roots missing | 0 |
| Required roots silently pruned | 0 |
| Open-world roots canonicalized | 0 |
| Query dependency closure | PASS |
| Query cycles | 0 |
| Query QX allocation | 0 |
| Whole-load allocation observed vs ledger | exact equality |
| Exact N | PASS |
| N−1 | typed failure |
| Retry after N−1 | PASS |
| Section-local rollback | PASS |
| Prepublication failpoint rollback | PASS |
| Ordinary `SemanticWorkspaceSession::from_image` | PASS |
| Local overlay compile | PASS |
| Local edit/invalidation/recompile | PASS |
| Local rollback | PASS |
| Canonical source reads in artifact session | 0 |
| Complete production consumer paths | 1 |
| Unknown importer dispositions | 0 |
| Fresh-process source denial | PASS |
| Positive compiler parity | PASS |
| Negative diagnostics parity | PASS |
| SlotMap source-mode runtime | PASS |
| SlotMap artifact-backed runtime | PASS |
| Parse focused regression | PASS |
| CLI smoke | PASS |
| Workspace `--no-run` | EXECUTED; P2-owned failures 0 |
| Schema/ABI freeze documented | YES |
| R4 walkthrough | PRESENT |
| P3 handoff | PRESENT |

---

# 25. P2 completion checklist

## Authority

- [ ] Current artifact ABIs are truthful.
- [ ] Complete current native declaration table exists after artifact load.
- [ ] Complete current alias table exists.
- [ ] Complete current hierarchy exists.
- [ ] Intrinsic projection rows survive independently.
- [ ] Pack and TypePack remain distinct.
- [ ] Complete witness plans exist.
- [ ] Complete derived conformance authority exists.
- [ ] Source-free static execution works through one evaluator.
- [ ] Complete current callable authority exists.
- [ ] Callable observed import phases survive.
- [ ] Module import phase products survive.
- [ ] Runtime dependency requirements survive.
- [ ] Required derived snapshot indexes exist.
- [ ] Manifest unknowns are zero.

## Query

- [ ] Every current query kind is explicitly classified.
- [ ] Required canonical root profile is explicit.
- [ ] Required roots are never silently pruned.
- [ ] Every required root has a current native product.
- [ ] Portable fingerprints validate.
- [ ] Live fingerprints recompute.
- [ ] Dependency evidence is unique.
- [ ] Dependency graph is closed.
- [ ] Dependency graph is acyclic.
- [ ] Open-world/unsupported roots are not canonical Ready.

## Session

- [ ] `SemanticWorkspaceSession::from_image` is the ordinary imported-session bridge.
- [ ] Canonical source baseline is not initialized.
- [ ] Canonical roots are immutable.
- [ ] Local source overlays work.
- [ ] Local invalidation works.
- [ ] Local recomputation works.
- [ ] Local rollback preserves canonical base.
- [ ] Multiple sessions can share the image safely.
- [ ] Project identity remains exact.

## Consolidation

- [ ] One production semantic artifact consumer remains.
- [ ] One production query reconstruction path remains.
- [ ] One type/static relocation architecture remains.
- [ ] One proof/evidence reconstruction architecture remains.
- [ ] `ArtifactImportTransaction` has a final non-competing disposition.
- [ ] `ImportedCompilerImage` has a final non-competing disposition.
- [ ] Modular importer ledger has zero unknowns.

## Certification

- [ ] Test prohibition removed.
- [ ] Current artifact test targets compile.
- [ ] Exact whole-load N discovered.
- [ ] Observed allocation equals ledger.
- [ ] N passes.
- [ ] N−1 fails typed.
- [ ] Retry passes.
- [ ] Section-local rollback passes.
- [ ] Prepublication retry passes.
- [ ] Fresh-process source denial passes.
- [ ] Source/artifact compiler parity passes.
- [ ] Negative diagnostic parity passes.
- [ ] SlotMap source path passes.
- [ ] SlotMap artifact path passes.
- [ ] Parse focused regression passes.
- [ ] CLI smoke passes.
- [ ] Workspace `--no-run` executed.
- [ ] Any remaining broad blocker is proven unrelated/inherited.
- [ ] Deterministic artifact rebuild passes.

## Lifecycle

- [ ] `CHECKPOINT.md` reports actual final state.
- [ ] R4 walkthrough exists.
- [ ] R3 is marked superseded, not erased.
- [ ] P2 is COMPLETE.
- [ ] P3 is unblocked.
- [ ] P3 handoff explicitly forbids re-solving P2 importer architecture.

---

# 26. Definition of P2 done

P2 is done only when the following statement is true:

> A current-schema `.telosc` artifact can be hostile-validated and imported with exact transactional allocation accounting into complete current native compiler authority; its dependency-closed reusable query products form an immutable `CanonicalUniverseFormalImage`; that image can initialize an ordinary `SemanticWorkspaceSession` without canonical Universe source access; ordinary local source overlays, invalidation, recomputation, and rollback function correctly above the immutable base; source-backed and artifact-backed compiler behavior is equivalent for representative positive and negative workloads; real package/runtime tests pass; and there is one production artifact-consumer architecture with zero unknown semantic-family dispositions.

Anything weaker is still P2.

---

# 27. Definition of P3-ready

P3 may begin when and only when P2 is complete and the following producer contract is frozen:

```text
INPUT TO P3:
    source-built current canonical semantic generation
    current canonical module world
    explicit required query-root profile
    current metadata

P3 RESPONSIBILITY:
    production root collection
    deterministic projection
    deterministic .telosc generation integration
    build/toolchain-facing production path

R4-CERTIFIED OUTPUT CONTRACT:
    ModuleWorldV2
    SemanticWorldV4
    QueryImageV4
    MetadataSectionV1
    frozen current header ABI/schema tuple

R4-CERTIFIED CONSUMER:
    bytes
      -> CanonicalUniverseFormalImage
      -> SemanticWorkspaceSession::from_image
```

P3 must not need to determine:

```text
how to reconstruct declarations
how to reconstruct aliases
how to reconstruct hierarchy
how to rebuild witnesses/proofs
how to run static programs
how to restore callable analysis
how module phase evidence is encoded
how runtime requirements are encoded
which query roots are safe
how live query fingerprints are computed
how imported sessions work
how project identities bind
how allocation rollback works
how source denial is proven
```

Those are P2.R4 responsibilities.

---

# 28. Final implementer handoff

Start from the exact consolidation branch, not `main`.

Do the work in this order:

```text
1. re-pin and activate R4;
2. correct ABI identity;
3. port tests/census to V2/V4/V3;
4. get source -> artifact -> load vertical green;
5. use failing tests to close only real semantic gaps;
6. recertify exact allocation and retry;
7. make required query profile exact/fail-closed;
8. certify SemanticWorkspaceSession::from_image + local overlays;
9. consolidate duplicate importer architecture under green tests;
10. run fresh-process source denial + parity + real packages;
11. run broad regressions and freeze schemas;
12. close P2 and write the P3 handoff.
```

The governing principle is:

```text
RECOVER AND PROVE
    before
CONSOLIDATE
    before
HAND OFF
```

Do not return to speculative family-by-family reimplementation unless an R4 discriminator proves the merged canonical importer lacks actual current compiler authority.

The final branch should make P3 boring in the best sense: P3 should consume a stable current semantic generation, invoke already-proven deterministic artifact projection machinery, and integrate production artifact generation without reopening the artifact-consumer architecture.

## Current correction slice — 2026-10-22

The `fix/universe-semantic-source` continuation completed the bounded
Universe-source diagnostic portion of R4. It is recorded separately from the
larger artifact/session acceptance ledger because R4 itself remains open.

The slice started at `5a68a1b6edfbc22d94ec27b9a3ae0f3a4eef0539`; its semantic
implementation commit is `a21aeb9d33b5f613eafd48e21457ac438b9502c3`. The
previous delivery at `0206f2f6785f814252fc95a39f471f2caf35f02f` passed
focused workflow run `39693098845`. Its semantic target passed 8/8, the
standalone `Module.packageInfo` unit test passed 1/1, and a fresh canonical
Universe census passed with 48 classified residual rows. The current
continuation passes 11/11 Universe semantic regression tests locally and
passes the standalone reflection regression 1/1. Proof commit
`bd4736743277da2dfd155e50120a48ab374b8e87` passed focused workflow run
`39713887081`: canonical parse/link 1/1, mutual type cycle 1/1, enum native
root 1/1, Option 9/9, Result callback 2/2, Universe diagnostics 11/11, and
standalone `Module.packageInfo` 1/1. The source proof is complete; P2.R4
artifact/session acceptance remains open.

The detailed evidence is in
`ARTF001.C1.P2.R4-universe-semantic-source-walkthrough.md`; the takeover
boundary is in
`ARTF001.C1.P2.R4-universe-semantic-source-handoff.md`. This slice does not
promote R4/T9, close P2, or establish source-free VM execution.

### T7 iterator default-body evidence layout amendment — 2026-10-22

The failure-only trace establishes that canonical semantic selection is
complete: the selected `Iterable.iter` default carries the concrete
`Array.iterator` conformance target. The compiled default body's abstract
`Iterable.iterator` invocation reads slot 55 (`41` callee-body evidence slots
plus raw requirement slot 14). The selected caller projection places that
same target at slot 29, the lexical `Self |= Iterable` evidence entry. The
runtime creates a 41-slot environment containing slot 29 and passes that same
environment to the `Iterable.iter` frame; slot 55 is outside the vector.

The architectural ruling assigns this mismatch to compiler lowering/runtime
layout projection. Preserve two distinct layout roles: the canonical generic
evidence slot keyed by the exact `GenericEvidenceSlotKey` for the default
body's implicit receiver conformance, and the direct abstract requirement
slot keyed by the exact `TraitRequirementId`. Project the same selected
`requirement_targets` entry into both mapped slots. Do not replace slot 29 with
slot 55, infer a target from the receiver/name, or weaken fail-closed runtime
reads. Use the canonical callee-body generic-evidence layout, including owner,
implicit receiver, and callable constraints, and the trait requirement table
to derive both maps. Keep the semantic target authoritative.

The focused discriminator must inspect the lowered default-body requirement
slot (55 for this fixture), prove that runtime materialization places the
exact selected `Array.iterator` target at both slot 29 and slot 55, preserve a
generic forwarded call through slot 29, and show that a missing target still
fails closed. Then run the exact `iterator_skip_take_takewhile.ph` CLI gate.
This is a compiler/runtime layout contract amendment within T7; do not close
T7 or P2.R4 until its focused runtime and artifact-backed acceptance passes.

### Universe diagnostic continuation — 2026-10-22

The active worktree continues from `4f0861428d82b962a094f588450e9d0577960e68`.
The correction removes 21 unused inherent associated-type declarations and
the two unsupported enum `Monoid` conformances, while preserving direct
`Chain.empty` and `Journal.empty`. Both source-data declaration-shell passes
now refresh without discarding registered intrinsic projections. A fresh
canonical regression verifies that `UnorderedShape` retains its owner and all
three intrinsic type-member products.

The canonical source snapshot has two remaining errors: the unsaturated
`Map` annotations on `Module.metadata` and `Module.__metadata__` in
`reflection.module` lines 21 and 28. The earlier 43 inherent-associated,
two enum class-side, and one intrinsic-owner diagnostics are gone. The exact
two errors are asserted by
`canonical_universe_has_only_deferred_module_metadata_errors_and_retains_unordered_shape_intrinsics`.

Do not add a public `ModuleMetadata` surface in this plan. The live normative
specification corpus does not define one, and the runtime's current
documentation-only `Option<String>` behavior does not settle the source API.
Keep these two diagnostics classified as a deferred contract blocker until
the owning specification adopts a public representation. The source cleanup
is focused-tested; P2.R4 remains open for T7, artifact/session publication,
parity, allocation, and certification gates.

### T7 `takeWhile(...).toList` inference continuation — 2026-10-22

After projecting the exact selected `Iterable.iterator` target into both the
callee generic-evidence slot and direct trait-requirement slot, the original
iterator failure disappeared. The exact source fixture then failed later at
`TakeWhileIterator.toList`. A focused semantic trace showed that the
`Iterator.takeWhile` default body accepted an untyped `pred`; the call to
`TakeWhileIterator.new(self, pred)` was `Blocked(UnknownType(NoTypeEvidence))`,
and the method had no inferred return. The following `.toList` therefore had
no canonical `Iterable::toList` selection.

The Universe method now declares `pred: (Self::Item) -> Bool`, matching the
existing constructor's contract. No trait evidence or dispatch rule changed.
The new regression
`language::iteration::take_while_to_list_keeps_guarded_iterable_dispatch_evidence`
proves the constructor result and inferred method return, the derived
`Iterable::toList` target, the lowering projection, and runtime output. It
passed 1/1 in 157.66 seconds. The exact fixture CLI then passed and printed
`4`, `5`, `4`, `[1, 2, 3]`, `[1, 2]`. A fresh canonical Universe census passed
1/1 in 99.13 seconds and retained exactly the two deferred metadata
diagnostics listed above.

This closes the exact source iterator runtime discriminator. It does not close
T7's artifact-backed runtime and positive/negative parity gates or R4.T8/T9;
keep P2.R4 `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` until those gates pass.
