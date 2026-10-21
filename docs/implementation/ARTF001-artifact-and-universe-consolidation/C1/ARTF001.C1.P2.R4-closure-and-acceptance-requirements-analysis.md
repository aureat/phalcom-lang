# ARTF001.C1.P2.R4 — Closure and Acceptance Requirements Before ARTF001.C1.P3

## Executive conclusion

`ARTF001.C1.P2.R4` is **not closed at the current branch head** and `ARTF001.C1.P3` should not be treated as the active implementation owner until R4 produces a current-head acceptance receipt.

This is no longer primarily an architecture-design problem. Most of the difficult P2 architecture is already implemented:

- current-schema `ModuleWorldV2` / `SemanticWorldV4` / `QueryImageV4` build and load;
- current-main semantic reconstruction;
- immutable `CanonicalUniverseFormalImage`;
- exact whole-load allocation accounting and rollback architecture;
- required-root query policy and dependency closure machinery;
- `SemanticWorkspaceSession::from_image`;
- local-overlay corrections preserving canonical query and derived-conformance authority;
- fresh-process source-denied **compiler** use from a loaded artifact;
- source/artifact compiler parity test infrastructure.

The remaining work is a combination of:

1. **current-head recertification** after later P8/executable work touched P2-owned semantic/import/fingerprint/allocation code;
2. **completion of ordinary image-backed session acceptance**, including the tests that were compiled or partially run but not accepted;
3. **formal closure of the query-root profile** at the current revision;
4. **consumer-architecture consolidation**, because `ArtifactImportTransaction` / `ImportedCompilerImage` still constitute a large parallel staging architecture;
5. **compiler parity and actual runtime compatibility smoke**, without stealing P3's source-free VM responsibility;
6. **broad P2 regression/schema freeze/manifest closure**;
7. **lifecycle closure and a revised P3 handoff** aligned with the new `ARTF001.C1.P3` portable-executable Universe plan.

The decisive architectural boundary is:

> **P2 must prove that a current compiler-only `.telosc` supplies complete, immutable, source-free canonical semantic/query authority to the ordinary compiler/session path. P2 does not have to prove source-free VM bootstrap. That executable/runtime responsibility belongs to P3.**

Accordingly, P2 closure must **not** wait for:

- executable-section publication;
- `ValidatedExecutableImage`;
- executable recipe materialization;
- `VM::try_new_with_universe_artifact`;
- source-free native-floor construction;
- eager/deferred executable initializers;
- source-free `run_compiled`.

Those are P3 requirements.

P2 **does** need to prove that code compiled *using the artifact-backed compiler image* is semantically equivalent to source-backed compilation and remains runnable by the existing runtime. The existing `artifact_backed_slotmap_runtime_tests_bootstrap_vm_with_generic_intrinsic_adts` test does not yet prove that because it compiles the program and constructs `VM::new()` but never calls `run_compiled`.

---

# 1. Repository and lifecycle state used for this analysis

## 1.1 Attached checkpoint authority

The attached `CHECKPOINT.md` records:

```text
ARTF001.C1.P2.R4
    IN_PROGRESS / PARTIAL / FOCUSED_TESTED

ARTF001.C1.P3
    PROPOSED / NOT_STARTED / UNVERIFIED
```

and explicitly states that P3 is blocked until P2 publishes reconstructed current-main semantic/query authority through the ordinary session.

Its active R4 ledger records:

```text
T0 COMPLETE
T1 IN_PROGRESS
T2 IN_PROGRESS
T3 COMPLETE
T4 PENDING
T5 IN_PROGRESS
T6 PENDING
T7 PENDING
T8 PENDING
T9 PENDING
```

The checkpoint's later continuation record also states that:

- the T5 canonical-query mutation incident was repaired;
- fresh-process artifact-backed compilation passed using a saved artifact;
- required roots were checked in that focused run;
- source-free VM execution was not proven;
- broad acceptance and several session regressions were not run;
- R4/P2 remained incomplete.

## 1.2 Live branch reconciliation

Live branch inspected:

```text
repository: aureat/phalcom-lang
branch: feat/artifact-universe-consolidation
HEAD: aad6a88db904de04059ee9027240ab168b136c5c
message: fix(artifact): complete P8 T3 validation groundwork
```

The attached checkpoint's active continuation state is older and still describes the P2-focused continuation around `48c47753b` plus the subsequent `22a95bbb1` / `4807ecaf2` delivery.

The live branch has since advanced through the P8 executable work. Those commits are not merely documentation-only. Between the P2 continuation and current `aad6a88db`, executable work also touched P2-owned or P2-certified files including:

```text
phalcom-core/tests/artifact_allocation.rs

phalcom-modules/src/project.rs
phalcom-modules/src/resolver.rs
phalcom-modules/src/topology.rs
phalcom-modules/tests/package_authority.rs

phalcom-semantic/src/artifact/export.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/manifest.rs
phalcom-semantic/src/artifact/query.rs
phalcom-semantic/src/artifact/query_policy.rs
phalcom-semantic/src/canonical_image.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/metadata/import.rs
phalcom-semantic/src/prelude.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/types/store.rs

multiple semantic artifact / canonical-image / fingerprint tests
```

This has an important consequence:

> R4.T3's earlier exact-allocation proof and the earlier query/session focused passes remain valid historical evidence that the architecture worked at that revision, but **they are not sufficient as the final P2 exit receipt for `aad6a88db`**.

Do not reopen T3 architecture speculatively. Re-run the current-head discriminator first. If it still passes exactly, T3 remains implementation-complete and is simply recertified at the exit revision.

---

# 2. What P2 must hand to P3

The revised P3 no longer has the old producer-only meaning assumed by the original R4 document. P3 is now the portable executable Universe/source-free VM plan.

Therefore the exact P2 → P3 contract should be frozen as:

```text
current compiler-only universe.telosc
    |
    +-- truthful current header compatibility
    +-- ModuleWorldV2
    +-- SemanticWorldV4
    +-- QueryImageV4
    +-- MetadataSectionV1
    |
    v
hostile-validated exact-accounted load
    |
    v
Arc<CanonicalUniverseFormalImage>
    |
    +-- complete current SemanticSnapshot
    +-- immutable reusable canonical query authority
    +-- current dependency fingerprints
    +-- canonical module/project identity
    |
    v
SemanticWorkspaceSession::from_image
    |
    +-- source-free canonical base
    +-- ordinary local source overlay
    +-- ordinary invalidation/recomputation
    +-- rollback / last-known-good
    |
    v
ProgramCompiler artifact-backed compilation
```

P3 may then add:

```text
EXECUTABLE section
validated executable authority
per-VM materialization
artifact-native floor
eager/deferred source-free initialization
one-artifact compiler/runtime flow
```

## 2.1 P2-owned facts that must be frozen before P3

P3 must be able to rely on all of these without redesign:

1. stable project/module/declaration/callable/variant identity;
2. project-qualified owner validation;
3. canonical source artifact compatibility tuple;
4. current native semantic reconstruction;
5. canonical `TypeStore` / `StaticValueStore`;
6. declaration, alias, hierarchy, conformance, witness, derived-rule, callable and static authority;
7. module import-phase and runtime-dependency authority;
8. required reusable query-root policy;
9. portable product-family mapping;
10. live fingerprint reconstruction;
11. query dependency closure;
12. immutable canonical query ownership in `SemanticDb`;
13. ordinary `SemanticWorkspaceSession::from_image`;
14. base-plus-local-overlay invalidation/rollback;
15. exact compiler-artifact load allocation/rollback contract;
16. native/public semantic metadata sufficient for P3 executable validation;
17. one production compiler-artifact semantic consumer architecture.

If any of these is still ambiguous, P3 will be forced to make executable decisions against unstable semantic identity and P2 is not done.

---

# 3. Critical scope correction: what R4 must not require from P3

The historical R4 plan contains an acceptance row named roughly “SlotMap artifact-backed runtime” and earlier P8 language can make it sound as though P2 must produce a source-free running VM.

That would violate the established phase boundary:

```text
P2-INV-09:
P2 does not serialize bytecode chunks, closures, or VM heap states.
```

The correct closure interpretation is:

## P2 must prove

```text
artifact-backed canonical semantic authority
    ->
artifact-backed user compilation
    ->
compiled-program equality/parity
    ->
compiled program is accepted by the existing runtime
```

## P3 must prove

```text
full artifact with executable authority
    ->
fresh VM without canonical source
    ->
run_compiled
```

### Required adjustment to the misleading existing test

Current test:

```rust
#[test]
fn artifact_backed_slotmap_runtime_tests_bootstrap_vm_with_generic_intrinsic_adts() {
    ...
    let art_tests = loaded.compile_user_entry(entry)
        .expect("artifact compilation of slotmap runtime tests");
    assert_eq!(art_tests.modules.len(), 8);

    let _vm = phalcom_core::vm::VM::new();
}
```

This is not runtime evidence.

For P2 acceptance either:

### Preferred

Change/replace the test so it actually:

```text
1. builds/loads the compiler-only artifact;
2. compiles the SlotMap runtime program through LoadedUniverseArtifact;
3. constructs the existing ordinary source-backed VM;
4. calls run_compiled(&artifact_compiled_program);
5. checks the existing SlotMap runtime success output/status.
```

This proves artifact-backed compiler output is executable by the current runtime without claiming source-free VM construction.

### Also acceptable

If an existing differential runtime helper already runs source-compiled and artifact-compiled programs under equivalent ordinary source-backed VMs, use that instead.

### Not acceptable

- merely constructing `VM::new()`;
- claiming source-free VM runtime from this P2 test;
- introducing executable artifact materialization into R4 to make the test pass.

---

# 4. Task-by-task closure analysis at the current head

## 4.1 R4.T0 — lifecycle / re-pin

### Existing state

The original T0 work was complete for the 2026-10-20 continuation, but the branch has advanced to `aad6a88db`.

### Remaining closure work

Treat T0 implementation as complete, but perform a **closure re-pin**:

```text
branch: feat/artifact-universe-consolidation
exit candidate: current HEAD
P7 ancestry: still required
P2 active: yes until acceptance
P3: proposed/blocked until exit receipt
```

Update the checkpoint before final acceptance to distinguish:

```text
historical P8 work already present in branch
vs
formal ARTF001.C1.P3 activation
```

P8 code presence does not satisfy the P3 lifecycle entry condition.

### Acceptance

```sh
git fetch origin
git status --short --branch
git rev-parse HEAD
git rev-parse origin/feat/artifact-universe-consolidation
git merge-base --is-ancestor \
  077d93d3bfcd5d3bca0127c4aac18abb55c2ed94 \
  HEAD
git diff --check
```

Required:

```text
branch/ref reconciled
P7 ancestry true
worktree state known/clean for certification
CHECKPOINT records current head
```

---

## 4.2 R4.T1 — compatibility identity and current-schema certification harness

### Assessment

**Implementation is substantially present; acceptance is not formally closed.**

Live `canonical_artifact.rs` already uses the intended named constants:

```rust
MODULE_SEMANTIC_ABI_V2
SEMANTIC_ARTIFACT_ABI_V3
QUERY_FINGERPRINT_ABI_V3
STABLE_IDENTITY_SCHEMA_V1
```

and the current wire families are:

```text
ModuleWorldV2
SemanticWorldV4
QueryImageV4
```

The old stale fixture compilation blockers were repaired and the semantic-artifact and compiler-artifact focused suites passed in the prior P2 continuation.

### Remaining implementation requirement

No new architecture should be implemented unless the current-head tests reveal drift.

The closure task is:

1. verify every producer/reader/test fixture still uses named current constants;
2. verify P8's later identity changes did not reintroduce stale literals/conversions;
3. remove/update any still-active historical comments that say V2/V4/V3 when the query image is now V4;
4. update R4/checkpoint wording to the final accepted current tuple.

### Acceptance requirement

At current head:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-compiler-artifact --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact
```

Also require the current header/schema hostile cases to reject incompatible tuples.

### Exit classification

After pass:

```text
R4.T1 = COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

No P3 executable schema needs to be involved.

---

## 4.3 R4.T2 — complete current semantic authority

### Assessment

**Most implementation exists. The main gap is acceptance completeness at current head.**

The repository now has a real source → current-schema artifact → native formal-image path. Prior evidence already includes:

- `semantic-artifact` 20/20;
- compiler-artifact round-trip 11/11;
- nominal/alias query family tests;
- blocked-shell retirement;
- source-free canonical image use;
- current canonical operational/static-program transport;
- module execution phase transport;
- current query family mapping.

The original major schema/authority blockers were resolved.

### Remaining implementation requirements

Do not reimplement semantic families preemptively.

First run the discriminators. Only implement if one of the required native-equivalence or complete-authority gates fails.

The following semantic facts must be accepted as **complete at the exit revision**, not merely present in DTOs:

#### Declarations

- nominal shell;
- form;
- class-object type;
- kind;
- semantic form;
- runtime owner;
- generic binder/signature;
- supertype template;
- standalone intrinsic static-domain projection;
- standalone intrinsic type-owner projection;
- shell runtime-owner witness consistency.

#### Pack boundary

Must remain:

```text
Pack runtime owner != TypePack type-member owner
```

No eighth direct intrinsic type-member owner.

#### Aliases / hierarchy

- complete `TypeAliasInfo`;
- dependencies;
- structural form;
- generic shape;
- current hierarchy;
- generic supertype templates;
- exact project-qualified owner.

#### Module execution

- exact source-import binding ordinal mapping;
- `ModuleImportPhaseProduct`;
- callable observed import phases;
- `RuntimeDependencyRequirement`;
- full obligation identity;
- distinct same-span obligations retained;
- duplicate semantic obligations rejected.

#### Proof/conformance

- conformance index;
- witness plans;
- derived conformance rule index;
- conditional fallback/evidence owner;
- no staging-only deferred proof authority required by compiler.

#### Static evaluation

- canonical operational program exists for compiler-image sessions;
- artifact execution of imported static definitions does not recreate source AST;
- source checked path and artifact operational path agree.

#### Callable authority

Compiler-observable callable analysis must be complete for the canonical profile.

No required callable authority may remain “deferred” in the old staged `ImportedCompilerImage` path.

### Required current-head tests

At minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic-artifact -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test artifact_query_import_test -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact --test round_trip_test -- --nocapture
```

Also execute the current exact native semantic parity tests for:

```text
declaration/runtime-owner projection
alias authority
hierarchy
module execution phase evidence
runtime requirement multiset
canonical static program
witness/derived authority
callable analysis
```

Do not accept `cargo check` as T2 closure.

### Required vertical discriminator

A **fresh current-head artifact**, not only `/tmp/artf001-universe.telosc` from the earlier continuation:

```text
current canonical source generation
    ->
build_universe_telosc
    ->
write artifact
    ->
load_universe_telosc_from_bytes/file
    ->
CanonicalUniverseFormalImage
    ->
compile novel inline user entry
```

Must prove:

```text
nonempty deterministic artifact
current section schemas
current ABI tuple
complete source-free formal image
valid metadata
required reusable query roots present/current
novel user compilation succeeds
```

### Exit classification

T2 may close when the above current native-equivalence/vertical gates pass with:

```text
missing compiler-required semantic family = 0
deferred compiler-required authority = 0
manifest unknown = 0
```

---

## 4.4 R4.T3 — exact allocation, rollback and retry

### Assessment

**Architecture and implementation were legitimately complete at the recorded P2 revision. It must be recertified at the final head.**

Previously proven:

```text
N_image = 450,922,288 bytes
independent allocator == ledger
N passes
N-1 typed BudgetExceeded
retry passes
child rollback passes
all four prepublication failpoints pass
QueryKind census = 46
QX = 0
```

The recorded session transaction boundary is also correct:

```text
Contract B:
artifact transaction ends at immutable CanonicalUniverseFormalImage
SemanticWorkspaceSession::from_image is outside N
```

### Why recertification is mandatory

Later executable work changed P2-owned import/export/fingerprint/module/allocation-test code.

Therefore:

```text
450,922,288
```

must be treated as the **last certified historical N**, not assumed final N.

### Remaining implementation requirement

None unless current-head exact accounting fails.

If it fails:

1. identify exact first stage delta;
2. fix the actual escaping allocation or changed budget owner;
3. do not "adjust" the ledger after observation;
4. preserve Contract B unless evidence proves it invalid.

### Current-head acceptance command

```sh
PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-r4-close-universe.telosc \
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core \
  --features allocation-diagnostics \
  --test artifact-allocation \
  -- --nocapture --test-threads=1
```

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib \
  loaded_artifact_prepublication_failures_are_retryable \
  -- --nocapture
```

Required final receipt:

```text
N_final = <measured>
observed == ledger == N_final
N_final succeeds
N_final - 1 typed-fails
clean retry succeeds
child rollback succeeds
prepublication failpoints all pass
QX == 0
```

### Exit classification

If the unchanged current implementation passes:

```text
R4.T3 remains COMPLETE
verification refreshed at final P2 head
```

Do not reopen its architecture merely because P8 work occurred afterward.

---

## 4.5 R4.T4 — required query profile and dependency closure

### Assessment

**Implementation appears substantially present and has focused evidence, but the plan ledger never promoted T4 and the current-head exit receipt is incomplete.**

Prior focused evidence includes:

```text
query policy: 6/6
hostile missing/stale required root: PASS
artifact query import: 4/4
required roots observed in fresh-process compile:
  DeclarationShell: 336
  DeclarationSurface: 4
  CallableSignature: 51
```

The prior run also confirmed canonical roots remained:

```text
current
validated
externally owned by the canonical image
not local session state
not invalidated/recomputed by three local revisions
```

### Remaining implementation requirements

First verify the current policy, rather than adding new query families.

Required contract:

1. all 46 current `QueryKind` values have explicit persistence/profile disposition;
2. required root profile is explicit;
3. no wildcard default admits new kinds;
4. required roots cannot be reverse-pruned silently;
5. physical product family is validated independently of logical query kind;
6. alias `DeclarationShell` roots remain valid through the TypeAlias product family;
7. open-world roots are not canonical reusable roots;
8. unsupported roots are not canonical reusable roots;
9. dependency graph is closed and acyclic;
10. portable fingerprints validate transport evidence;
11. live fingerprints are recomputed from reconstructed native products;
12. canonical `SemanticDb` roots are `Ready`/current without local state.

### Required focused acceptance

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --lib artifact::query_policy -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --lib \
  required_root_dependency_closure_rejects_missing_and_stale_live_evidence \
  -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test artifact_query_import_test -- --nocapture
```

Also run current tests proving:

```text
foreign dependency rejection
stale fingerprint rejection
cyclic dependency rejection
wrong family/row/owner rejection
alias-vs-nominal DeclarationShell behavior
required-root final native-authority equality
```

### Required full-artifact assertion

From the same fresh artifact used by closure certification:

```text
required root count expected == required root count exported == imported
required silently pruned == 0
open-world canonicalized == 0
unsupported canonicalized == 0
dependency cycles == 0
```

Record final counts in the walkthrough.

### Exit classification

After current-head focused + full-artifact census:

```text
R4.T4 = COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

---

## 4.6 R4.T5 — ordinary imported SemanticWorkspaceSession

### Assessment

**The architecture is implemented and the principal defect found during R4 was repaired, but T5 is not accepted.**

Current implementation already includes the key corrections:

- `SemanticWorkspaceSession::from_image` initializes the prelude from the image snapshot rather than canonical source;
- first local publication wraps the canonical `DerivedConformanceRuleIndex` as a parent rather than cloning it into local state;
- canonical roots remain external immutable products;
- the fresh-process source-denied test exercised three local revisions successfully.

However, the prior continuation explicitly states:

```text
ordinary_image_session test target:
    compiled
    selected 3 tests
    canceled during source bootstrap
    NOT PASS

rollback/fail-closed/source-canonical-root regressions:
    not rerun
```

That is the main direct acceptance gap.

### Required current tests already present

The live tree contains exactly the right session regressions:

```text
ordinary_image_session_keeps_universe_external_across_replacement

ordinary_image_session_local_edit_reuses_unrelated_local_queries_and_retains_universe_identity

ordinary_image_sessions_share_canonical_products_but_isolate_workspace_state

from_image_zero_canonical_entries_copied_cardinality_guard
```

### Acceptance requirements

The three ordinary-session scenarios must prove:

#### Initial authority

```text
last_snapshot Arc == image.snapshot Arc
last_known_good Arc == image.snapshot Arc
canonical query local state == None
canonical product origin == Canonical(image world)
```

#### Local overlay

```text
local module publishes normally
local query can depend on canonical root
local edit changes local state only
unrelated local query is reused
canonical root is not copied locally
canonical root is not recomputed
canonical identity is not rebased
```

#### Isolation

```text
two sessions share canonical product Arcs
workspace IDs differ
left local update does not affect right session
canonical base remains common immutable authority
```

#### Rollback

Add/run an explicit image-backed local publication failure/cancellation case:

```text
failed candidate not published
last-known-good preserved
canonical image unchanged
canonical query state unchanged
subsequent valid update succeeds
```

If an existing test already proves this, run it and record the exact name instead of adding a duplicate.

#### Source independence

With denial active before child load/session construction:

```text
canonical source_text reads = 0
canonical baseline construction = 0
canonical parse/link/semantic rebuild = 0
```

Provider construction may remain inert; source access may not occur.

### Required command group

Resolve exact current test-binary filters, but the semantic integration filter must actually execute the named tests, not only compile.

Example:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic \
  ordinary_image_session \
  -- --nocapture
```

Then run the focused rollback/fail-closed image-session tests by exact name.

### Exit classification

```text
R4.T5 = COMPLETE
```

only when ordinary local update, invalidation/reuse, rollback, multi-session isolation, and source-free canonical base behavior all pass.

---

# 5. R4.T6 — consumer architecture consolidation is the main remaining implementation task

## Assessment

**This task is genuinely still open. It is not just missing test evidence.**

The live semantic artifact module still contains two conceptually different paths.

## Canonical production path

```text
current DTOs
    ->
phalcom-semantic::artifact::import
    ->
current SemanticSnapshot
    ->
current query reconstruction
    ->
CanonicalUniverseFormalImage
    ->
SemanticWorkspaceSession::from_image
```

## Parallel staging path

`phalcom-semantic/src/artifact/mod.rs` still exposes:

```rust
pub struct ImportedCompilerImage {
    pub external_world: Arc<CanonicalExternalModuleWorld>,
    pub project_universe: Arc<ProjectUniverse>,
    pub metadata_bundle: SemanticMetadataBundle,
    pub type_graph: Arc<ImportedTypeGraph>,
    pub static_values: Arc<ImportedStaticValueGraph>,
    pub static_constant_authority: Arc<ImportedStaticConstantAuthority>,
    pub static_definition_authority: Arc<ImportedStaticDefinitionAuthority>,
    pub semantic_aggregates: Arc<ImportedSemanticAggregates>,
    ...
    pub query_products: Arc<ImportedQueryProducts>,
    pub semantic_world: SemanticWorldV4,
    pub query_image: QueryImageV4,
}

pub struct ArtifactImportTransaction<'a> {
    ...
}
```

The file comments still explicitly describe this as an in-progress current-main importer retained as an adaptation source.

The latest handoff found no workspace production consumer outside its semantic wrappers/re-exports, but also found one important capability not yet demonstrably subsumed:

```text
generic project/package trusted-receipt staging
```

Therefore the correct closure is not "delete the whole directory".

## Required implementation strategy

### Step 1 — produce a final disposition ledger

Every parallel importer family gets one disposition:

```text
CANONICAL
RETAIN_HELPER
VALIDATION_ONLY
MERGED_INTO_CANONICAL
SUPERSEDED_REMOVE
HISTORICAL_TEST_ONLY
```

Required:

```text
UNKNOWN = 0
```

### Step 2 — preserve trusted generic project authority

Do not lose:

```text
ProjectUniverse
TrustedPackageResolver
stage_artifact_project_authorities
exact package/version/fingerprint receipt
validated artifact digest binding
persistent target identity
```

If the Universe-specific canonical loader does not need package receipts, the generic transport API may remain — but it must not retain a competing semantic snapshot/query authority.

### Step 3 — decide `ArtifactImportTransaction`

Preferred:

```text
ArtifactImportTransaction
    ->
transport/project authority staging
    ->
canonical current native importer
    ->
CanonicalUniverseFormalImage or canonical native import result
```

Acceptable:

- retain it only as generic transport/validation API;
- remove its independent semantic publication role.

Forbidden:

```text
ArtifactImportTransaction -> complete ImportedCompilerImage semantics
AND
canonical artifact loader -> complete CanonicalUniverseFormalImage semantics
```

as parallel production authority.

### Step 4 — decide `ImportedCompilerImage`

Allowed outcomes:

#### Thin wrapper

It becomes a wrapper/reference to the same canonical native imported authority.

#### Validation/test object

It remains only for focused validation/backward test fixtures and is not exposed as a production compiler semantic source.

#### Removed

If callers are fully migrated.

Forbidden: a second current semantic/query reconstruction model.

### Step 5 — preserve useful helpers

Do not throw away current-main adapters merely because their staging aggregate is superseded.

Candidate reusable helper classes include:

```text
stable owner validation
project/package receipt validation
individual product validators
type/static relocation helpers
hostile fixture constructors
```

### Acceptance measurements

At P2 exit:

```text
complete production semantic artifact consumer count = 1
complete production query reconstruction path count = 1
type relocation authority count = 1
static relocation authority count = 1
proof/evidence native reconstruction authority count = 1

unknown disposition count = 0
```

### Regression requirement after destructive consolidation

Re-run:

```text
T2 semantic artifact gates
T3 allocation gate
T4 query closure
T5 ordinary session gates
```

before continuing to T7.

This ordering is mandatory: do not consolidate the consumer architecture while T2–T5 still have unclassified failures.

---

# 6. R4.T7 — fresh-process source denial, parity, diagnostics, and real functionality

## Assessment

There is strong focused groundwork, but T7 is not accepted.

Already demonstrated at the prior P2 revision:

```text
fresh process
loaded saved artifact
source denial active
three local revisions
packages/slotmap compiled (4 modules)
packages/slotmap/tests/runtime compiled (8 modules)
canonical query local state remained zero
required roots current/validated
canonical rule Arc retained
```

Limitations explicitly recorded:

```text
saved artifact reused
not a fresh current-head artifact
programs compiled but not executed
later P8 changes occurred afterward
ordinary source/session broad gates incomplete
```

## 6.1 Fresh-process closure test must use a fresh current-head artifact

Parent process:

```text
build current compiler-only universe.telosc
atomically write it
```

Child process starts without canonical Universe initialization:

```text
set PHALCOM_DENY_SOURCE_BOOTSTRAP=1
load that exact artifact
construct SemanticWorkspaceSession::from_image
compile a novel inline program
perform at least one local edit/recompile
compile packages/slotmap
compile packages/slotmap/tests/runtime
```

Required:

```text
canonical source read = 0
canonical semantic bootstrap = 0
canonical query copied local = 0
canonical query recomputation = 0
```

Do not allow the environment variable to point at an older saved artifact for the final acceptance receipt.

## 6.2 Positive compiler parity

Current live test:

```text
source_backed_and_artifact_backed_compiler_program_parity
```

already compares at least:

```text
packages/slotmap
packages/slotmap/tests/runtime
```

through the source formal image and loaded artifact.

Run it at final head.

Acceptance is compiled-program semantic/normalized parity, not just both sides succeeding.

## 6.3 Negative diagnostic parity

Current live test:

```text
source_backed_and_artifact_backed_negative_diagnostic_parity
```

plus focused cases:

```text
negative_parity_constructor_resolution_failure
negative_parity_inaccessible_declaration
negative_parity_invalid_pattern_matching
negative_parity_trait_resolution_failure
```

Run all current negative parity cases.

Require normalized diagnostics/error parity.

## 6.4 Actual artifact-compiled runtime smoke

Current test:

```text
artifact_backed_slotmap_runtime_tests_bootstrap_vm_with_generic_intrinsic_adts
```

does not call `run_compiled`.

Before P2 closure, fix this evidence.

Required semantics:

```text
artifact compiler image
    ->
compile SlotMap runtime project
    ->
ordinary current source-backed VM
    ->
run_compiled(artifact_compiled_program)
    ->
existing expected SlotMap runtime result
```

This belongs in P2 because it proves the artifact-backed **compiler** produced runtime-compatible program output.

It does **not** need:

```text
artifact VM
portable executable Universe
source denial during VM::new
```

Those are P3.

If the test name continues to imply stronger source-free behavior, rename it to make the boundary truthful.

## 6.5 Ordinary source-mode regressions

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test cli-smoke

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime
```

Also run the current focused package-parse compile regression.

Artifact consolidation must not regress the normal source compiler path.

## T7 exit receipt

```text
fresh current-head source-denied artifact compilation = PASS
positive compiler parity = PASS
negative diagnostics parity = PASS
artifact-compiled SlotMap runtime under ordinary VM = PASS
SlotMap ordinary source mode = PASS
parse/package regression = PASS
CLI smoke = PASS
```

Explicit non-requirement:

```text
source-free artifact VM runtime = NOT A P2 GATE
```

---

# 7. R4.T8 — broad regression, schema freeze, manifest closure

## Assessment

This has not been performed for the P2 exit revision.

The checkpoint explicitly records that broad workspace checks were not run.

P8 subsequently changed several P2-owned semantic/fingerprint/import files, which makes T8 even more important before freezing the semantic contract consumed by P3's executable verifier.

## 7.1 Workspace compile acceptance

Required:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run
```

This must be **executed**.

If it fails:

```text
A/B caused by current branch → fix
C unclear → one bounded classification pass
D inherited/unrelated → reproduce/record, does not block if no P2-owned target
```

Do not reuse an old baseline failure classification without current evidence.

## 7.2 Relevant crate suites

Required:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib
```

If a full semantic/core suite is operationally prohibitive, the closure record may separate unrelated corpus timeouts from P2-owned partitions, but all P2-owned artifact/canonical/session tests must run.

Do not use `--no-run` as behavioral acceptance.

## 7.3 Schema/ABI freeze

Record exact final compiler-artifact contract:

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

P3 executable additions may add their own executable compatibility, but must not silently mutate P2's compiler-image contract.

## 7.4 Manifest closure

At current head:

```text
unclassified snapshot fields = 0
unclassified session base families = 0
unclassified query kinds = 0
unclassified compiler attachments = 0
unclassified imported artifact families = 0
```

Later P8 semantic edits make this current-head check mandatory.

## 7.5 Determinism

Build the compiler-only canonical artifact twice in fresh equivalent construction contexts.

Require:

```text
bytes identical
artifact digest identical
section digests identical
required query-root ordering identical
```

P3 will extend deterministic output with EXECUTABLE; it must inherit deterministic compiler sections.

## T8 exit

```text
P2-owned regression failures = 0
workspace --no-run executed
remaining failures classified
schema tuple frozen
manifest unknown = 0
compiler-only artifact deterministic
```

---

# 8. R4.T9 — formal acceptance and P3 handoff

R4 does not close automatically when the tests become green.

A durable exit record is required.

## 8.1 Required walkthrough

Create/update an R4 walkthrough containing:

```text
start SHA
final SHA
P7 ancestry confirmation
implementation commits
consumer-consolidation result
schema/ABI tuple
artifact size
artifact digest
final allocation N
required root counts by family
source-denial command/result
session tests/results
positive parity result
negative parity result
artifact-compiled runtime smoke result
source-mode SlotMap result
CLI smoke result
workspace --no-run result
broad crate-suite results
baseline/unrelated failures
```

## 8.2 Required lifecycle transition

Only after every P2-owned gate:

```text
ARTF001.C1.P2.R4
    status: COMPLETE
    completion: IMPLEMENTED
    verification: strongest truthful classification

ARTF001.C1.P3
    status: PROPOSED or READY
    completion: NOT_STARTED
```

Then, when P3 implementation actually begins:

```text
P3 -> IN_PROGRESS
```

Do not retroactively classify existing historical P8 work as having satisfied the new P3 entry gate.

## 8.3 Revised P3 handoff contract

The old R4 text assumed a producer-focused P3. That is obsolete relative to the current `ARTF001.C1.P3` plan.

The new handoff must state that P3 inherits:

```text
Validated compiler-only artifact format
    ModuleWorldV2
    SemanticWorldV4
    QueryImageV4
    MetadataSectionV1

LoadedUniverseArtifact
    formal_image: Arc<CanonicalUniverseFormalImage>
    metadata: Arc<SemanticMetadataBundle>

SemanticWorkspaceSession::from_image
    proven source-free canonical compiler base

LoadedUniverseArtifact::compile_user_entry*
    proven artifact-backed compilation API

current stable semantic/callable/variant/module identity
    authoritative for executable validation

exact compiler-only load allocation contract
    frozen P2 baseline
```

P3 then owns:

```text
EXECUTABLE section
executable capability
cross-section executable proof
full-artifact allocation extension
VM-local recipe materialization
artifact-native VM floor
eager/deferred source-free initialization
same-artifact compiler/runtime workflow
```

The handoff must explicitly state:

> P3 must not reopen P2 semantic/query reconstruction, project identity, query persistence, session overlay, or compiler-only load accounting unless a concrete executable integration regression demonstrates a P2 defect.

---

# 9. Current-head closure matrix

The following is the recommended truthful status at `aad6a88db` before re-running closure gates.

| R4 task | Implementation assessment | Acceptance assessment | Required before P3 |
|---|---|---|---|
| T0 lifecycle | Previously implemented | stale current-head record | re-pin `aad6...`/final head and update checkpoint |
| T1 compatibility/harness | materially implemented | not formally closed at current head | current ABI/schema + compiler-artifact certification |
| T2 current semantic authority | materially implemented | strong focused evidence, incomplete full exit proof | fresh vertical build/load + native-equivalence gates |
| T3 exact allocation | implementation complete | certified at earlier P2 revision, not final head | rerun exact N/N−1/retry/failpoint at exit head |
| T4 query profile | substantially implemented | focused evidence, task never promoted | exhaustive current-head profile/root census/hostile closure |
| T5 ordinary session | key defects fixed | incomplete; named session tests/rollback not accepted | run full ordinary-image session gate + source-denied fresh artifact |
| T6 consumer consolidation | **not complete** | not accepted | one production consumer + disposition ledger |
| T7 parity/functionality | partial | compile parity groundwork; runtime test is non-proof | fresh source-denied compile, parity, diagnostics, actual artifact-compiled runtime smoke |
| T8 regression/freeze | not done | not done | broad compile/suites, schema freeze, manifest/determinism |
| T9 lifecycle | not done | not done | walkthrough, checkpoint closure, new P3 handoff |

This matrix should replace the simpler interpretation that “T3 complete, therefore P2 is almost automatically closed.”

The likely implementation-heavy remaining slice is **T6**, plus any defects exposed by T5/T7 current-head tests. Most other remaining work should be certification and lifecycle work if the implementation is as complete as the focused evidence suggests.

---

# 10. Recommended closure execution order

Do not resume P3 executable implementation until this sequence completes.

## Wave A — re-pin and inexpensive current-head truth

```text
A1. current HEAD / clean tree / P7 ancestry
A2. check five owning crates
A3. compiler-artifact suite
A4. semantic-artifact + artifact-query-import
A5. query policy / hostile required-root closure
```

Purpose: determine whether P8 semantic identity changes broke P2 before doing expensive tests.

## Wave B — vertical compiler artifact and exact load

```text
B1. build fresh current-head compiler-only universe.telosc
B2. current build/load/novel-inline compile
B3. exact allocation N/N−1/retry at current head
B4. prepublication failpoints
```

Save this artifact only as evidence for the same revision.

Do not reuse the older `/tmp/artf001-universe.telosc` as final evidence.

## Wave C — ordinary session

Run:

```text
ordinary_image_session_keeps_universe_external_across_replacement
ordinary_image_session_local_edit_reuses_unrelated_local_queries_and_retains_universe_identity
ordinary_image_sessions_share_canonical_products_but_isolate_workspace_state
from_image_zero_canonical_entries_copied_cardinality_guard
image-backed rollback/fail-closed tests
```

Fix only concrete failures.

## Wave D — T6 consolidation

Only after A–C are green:

```text
inventory parallel importer families
assign disposition
route/demote/remove competing production authority
retain trusted project/package receipt staging
prove one production semantic/query path
rerun T2–T5 focused gates
```

## Wave E — compiler parity / functionality

```text
fresh current-head source-denied compiler artifact child
source/artifact compiled-program parity
negative diagnostics parity
artifact-compiled SlotMap runtime on ordinary VM
source-mode SlotMap
parse regression
CLI smoke
```

## Wave F — broad/freeze

```text
workspace --no-run
relevant crate suites
manifest zero-unknown
schema/ABI freeze
determinism
```

## Wave G — acceptance

```text
R4 walkthrough
final checkpoint
P2 COMPLETE
revised P3 handoff
P3 entry receipt
```

---

# 11. Exact minimum acceptance gates

P2 should not be closed with an informal "all focused tests look good." Record a concrete exit matrix.

## G-P2.1 — current compiler image transport

Required:

```text
current header compatibility PASS
compiler-artifact suite PASS
fresh deterministic compiler-only artifact build PASS
fresh load PASS
complete source-free CanonicalUniverseFormalImage PASS
```

## G-P2.2 — native semantic authority

Required:

```text
declarations PASS
aliases PASS
hierarchy PASS
intrinsic projections PASS
Pack/TypePack distinction PASS
module phase authority PASS
runtime requirements PASS
witness/derived/conformance PASS
static operational execution PASS
callable compiler-required authority PASS
manifest unknowns = 0
```

## G-P2.3 — query authority

Required:

```text
46/46 query kinds classified
required root profile explicit
required missing = 0
required silently pruned = 0
open-world reusable = 0
unsupported reusable = 0
dependency closure PASS
cycle rejection PASS
portable evidence PASS
live fingerprint recomputation PASS
canonical roots current/validated PASS
```

## G-P2.4 — exact immutable load

Required:

```text
N_final known
independent allocator == ledger == N_final
N_final PASS
N_final - 1 typed FAIL
clean retry PASS
child rollback PASS
four prepublication failpoints PASS
QX = 0
```

## G-P2.5 — ordinary image session

Required:

```text
from_image initial Arc identity PASS
canonical query local state = none
local overlay PASS
local edit/invalidation PASS
unrelated local reuse PASS
canonical non-recompute PASS
rollback/last-known-good PASS
multi-session isolation PASS
source access under denied child = 0
```

## G-P2.6 — singular consumer

Required:

```text
production semantic consumer count = 1
production query reconstruction count = 1
unknown importer dispositions = 0
generic trusted package/project receipt capability preserved
```

## G-P2.7 — compiler behavior

Required:

```text
fresh-process source-denied artifact-backed compilation PASS
source/artifact positive compiled-program parity PASS
negative diagnostics parity PASS
artifact-compiled real runtime program under ordinary VM PASS
ordinary source SlotMap runtime PASS
parse/package regression PASS
CLI smoke PASS
```

## G-P2.8 — broad/freeze

Required:

```text
workspace --no-run EXECUTED
P2-owned broad failures = 0
relevant crate suites PASS or explicitly classified unrelated
schema/ABI tuple frozen
compiler-only artifact deterministic
```

## G-P2.9 — lifecycle

Required:

```text
R4 walkthrough present
checkpoint truthful
P2 COMPLETE
new P3 handoff present
P3 entry contract frozen
```

---

# 12. Explicit non-gates for P2

Do **not** keep P2 open for these:

```text
full EXECUTABLE section
executable bytecode identity validator
ValidatedExecutableImage publication
full-artifact N including executable
VM-local constant/prototype materialization
source-free native floor
source-free VM::try_new_with_universe_artifact
eager executable initializer execution
deferred executable initializer execution
two artifact-VM isolation
one-artifact runtime CLI
source-free run_compiled
```

These are the revised P3 plan.

If one of them appears in an R4 test name, rewrite the test expectation so it proves only the P2 compiler/session contract, or move the test to P3.

---

# 13. Acceptance failure policy

P2 closure should be efficient. Do not use the remaining gate list as permission for broad redesign.

## If T1/T2/T4 focused tests fail

First classify whether the failure was introduced by P8's later identity/determinism changes.

Fix the smallest current-semantic regression.

Do not change wire schema unless a current compiler-required fact truly cannot be represented or derived.

## If T3 exact allocation fails

The earlier architecture is accepted.

Locate the first exact observed-vs-ledger delta.

Do not:

```text
change to approximate accounting
raise the budget until it passes
exclude the new path from the transaction
```

## If T5 fails

Treat the immutable canonical image as fixed authority.

Local fixes should occur in:

```text
session bootstrap
local overlay ownership
local invalidation/failure handling
```

Do not make canonical queries mutable to satisfy local update logic.

## If T6 discovers a real consumer of `ImportedCompilerImage`

Determine whether it consumes unique generic project/package staging or actually depends on the parallel semantic authority.

Preserve unique transport/receipt logic.

Route semantic authority through the canonical importer.

Do not delete the caller or keep two complete semantics just to avoid migration.

## If T7 runtime compatibility fails

First determine whether:

```text
artifact compiler produced different program
or
test VM/runtime harness is wrong
```

Do not implement P3 executable materialization as the P2 fix.

## Architectural STOP / CONSULT

Stop if closure reveals:

- a current compiler-critical semantic fact absent from all P2 wire/native authority;
- a required canonical query root cannot be represented without a new persistence contract;
- `SemanticWorkspaceSession::from_image` fundamentally requires mutable canonical authority;
- exact allocation ownership must be redesigned;
- singular-consumer consolidation would lose unique semantic authority;
- runtime compatibility requires source-free executable Universe state rather than ordinary VM compatibility;
- a P2 schema/ABI change would alter P3's assumed stable identities.

---

# 14. The P2 exit receipt P3 should require

P3 T0 should not simply check:

```text
CHECKPOINT says P2 complete
```

It should require a concrete receipt in the R4 walkthrough/handoff containing:

```yaml
p2_exit:
  branch: feat/artifact-universe-consolidation
  revision: <final-r4-sha>

  compiler_artifact:
    deterministic: true
    artifact_size_bytes: <value>
    artifact_digest: <digest>

  schema:
    container: <value>
    minimum_reader: <value>
    stable_identity: <value>
    module_semantic_abi: <value>
    semantic_artifact_abi: <value>
    query_fingerprint_abi: <value>
    module_world: 2
    semantic_world: 4
    query_image: 4
    metadata: 1
    native_surface: <value>
    type_metadata: <value>

  allocation:
    exact_n_bytes: <N_final>
    observed_equals_ledger: true
    n_minus_one_typed_failure: true
    retry: true
    child_rollback: true
    prepublication_failpoints: true

  query:
    kind_count: 46
    required_root_counts:
      <family>: <count>
    missing_required: 0
    silently_pruned_required: 0
    open_world_ready: 0
    unsupported_ready: 0
    dependency_cycles: 0

  session:
    ordinary_from_image: true
    source_denied_compile: true
    local_overlay: true
    invalidation: true
    rollback: true
    multiple_session_isolation: true

  consumers:
    production_semantic_importers: 1
    unknown_dispositions: 0

  parity:
    positive_compiler: true
    negative_diagnostics: true
    artifact_compiled_runtime_smoke: true

  broad:
    workspace_no_run_executed: true
    p2_owned_failures: 0

  executable_runtime:
    intentionally_deferred_to_p3: true
```

This is the cleanest way to stop future P3 implementation from re-litigating P2.

---

# 15. Recommended immediate implementation slice

The fastest architecturally correct next action is **not** T6 code immediately.

First establish whether current `aad6a88db` still satisfies the already-implemented P2 contract.

## Slice 1 — current-head discriminators

Run, in order:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-compiler-artifact

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic-artifact -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test artifact_query_import_test -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --lib artifact::query_policy -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --lib \
required_root_dependency_closure_rejects_missing_and_stale_live_evidence \
-- --nocapture
```

If green, run the fresh exact allocation gate and save the newly built artifact.

## Slice 2 — T5 ordinary session

Run the named current tests.

Fix any real defect.

Do not touch T6 yet if T5 is red.

## Slice 3 — T6 singular-consumer consolidation

Once T2/T3/T4/T5 are green, implement the final disposition/routing.

Then rerun only the discriminating P2 gates.

## Slice 4 — T7/T8/T9 certification

Finish parity/runtime smoke, broad checks, freeze, and lifecycle.

At that point P3 may formally begin.

---

# 16. Final acceptance statement

`ARTF001.C1.P2.R4` is ready to close when the following statement is true at one recorded final branch revision:

> A fresh current-schema compiler-only `universe.telosc` deterministically builds and hostile-validates; imports with exact transactional allocation accounting into complete current native semantic/query authority; publishes one immutable `CanonicalUniverseFormalImage`; initializes the ordinary `SemanticWorkspaceSession` without canonical Universe source access; supports isolated local overlays, invalidation, recomputation and rollback without mutating canonical roots; has an explicit dependency-closed required query profile; compiles representative novel and real-package programs equivalently to the source-backed compiler with diagnostic parity; produces runtime-compatible user programs; and has one production semantic artifact-consumer architecture with zero unclassified artifact families. All of those claims are re-certified at the final R4 revision, the compiler-artifact schema/ABI tuple is frozen, and the checkpoint/walkthrough/handoff truthfully record the evidence.

Only then should `ARTF001.C1.P3` become the active implementation plan.

The P3 entry statement should then be:

> P2 compiler/session authority is closed. P3 may extend the frozen compiler artifact with a validated portable executable image and source-free VM bootstrap without reopening P2 semantic/query/session architecture.
