---
id: ARTF001.C1.P2.R4.fast-closure-handoff
program: ARTF001
checkpoint: ARTF001.C1
plan: ARTF001.C1.P2.R4
kind: advanced-agent-handoff
branch: feat/artifact-universe-consolidation
pinned_revision: aad6a88db904de04059ee9027240ab168b136c5c
time_budget: 5-10 minutes
verification_policy: defer-long-running
---

# ARTF001.C1.P2.R4 — Advanced Agent Fast-Closure Handoff

## 0. Mission

You have approximately **5–10 minutes of advanced-agent usage**.

Use that window for the work that benefits most from deep reasoning and repository synthesis. **Do not spend it waiting on long Rust compilation, canonical Universe bootstrap, allocation census, workspace tests, or broad verification.**

Your job is to make the largest possible reduction in the remaining `ARTF001.C1.P2.R4` closure surface, with the following ordering:

```text
1. T6 — eliminate / subordinate the duplicate semantic artifact consumer architecture
2. T7 — make the existing "artifact-backed runtime" test prove actual runtime compatibility
3. T5 — close any obvious remaining ordinary-image-session implementation gap
4. T1/T2/T4/T8/T9 — only make code/doc corrections that are statically obvious
5. leave long acceptance execution for the next agent/session
```

The branch is currently:

```text
repo:   aureat/phalcom-lang
branch: feat/artifact-universe-consolidation
HEAD:   aad6a88db904de04059ee9027240ab168b136c5c
commit: fix(artifact): complete P8 T3 validation groundwork
```

Do **not** switch to `main`.

Do **not** start implementing P3 executable VM materialization.

---

# 1. Non-negotiable time discipline

## 1.1 Do not run these during the short window

Unless a command is already warm and demonstrably completes in seconds, **do not run**:

```bash
cargo test --workspace --no-run
cargo test -p phalcom-semantic
cargo test -p phalcom-core --lib
cargo test -p phalcom-core --test artifact-allocation
cargo run ... packages/slotmap/tests/runtime
```

Also avoid anything that invokes canonical Universe source bootstrap and sits for 30–150 seconds.

Do not start a long command hoping it finishes before reset.

## 1.2 Allowed cheap commands

Use these aggressively:

```bash
git status --short --branch
git rev-parse HEAD
git diff --check

rg -n 'ImportedCompilerImage|ArtifactImportTransaction'
rg -n 'publish_query_products_into_empty_db'
rg -n 'stage_payload|stage_payload_with_digest'
rg -n 'TrustedPackageResolver|stage_artifact_project_authorities'
rg -n 'artifact_backed_slotmap_runtime_tests_bootstrap_vm_with_generic_intrinsic_adts'
rg -n 'run_compiled\(' phalcom-core
rg -n 'ordinary_image_session_' phalcom-semantic/tests
rg -n 'from_image\(' phalcom-semantic/src phalcom-semantic/tests
```

Use direct source inspection and static reasoning.

If you make edits, `cargo fmt` or a tiny focused `cargo check` is acceptable **only if you know it is incremental and likely sub-20s**. Otherwise leave explicit verification commands in your handoff.

---

# 2. P2/P3 boundary — do not cross it

P2 closes the **compiler/session** artifact contract.

P3 owns the **portable executable/source-free VM** contract.

## P2 must finish

```text
compiler-only universe.telosc
    ->
current native semantic/query authority
    ->
CanonicalUniverseFormalImage
    ->
SemanticWorkspaceSession::from_image
    ->
artifact-backed user compilation
```

P2 must prove that artifact-compiled user code is runtime-compatible with the existing runtime.

## P3 owns

```text
EXECUTABLE section
ValidatedExecutableImage
per-VM recipe relocation
artifact-native VM floor
eager/deferred source-free Universe initialization
VM::try_new_with_universe_artifact
source-free run_compiled
```

Do **not** solve a P2 test failure by implementing P3 runtime materialization.

---

# 3. Known closure state — do not rediscover it

The hard architecture work for T1–T5 is mostly already present.

## Already implemented / previously focused-proven

- current `ModuleWorldV2`;
- current `SemanticWorldV4`;
- current `QueryImageV4`;
- current ABI constants;
- current native semantic import;
- query product-family separation;
- required-root policy;
- exact load accounting architecture;
- historical exact `N_image = 450,922,288`;
- N−1 typed failure;
- retry;
- child rollback;
- four prepublication failpoints;
- `SemanticWorkspaceSession::from_image`;
- image-backed prelude initialization;
- immutable canonical query authority;
- canonical `DerivedConformanceRuleIndex` parent overlay;
- source-denied artifact-backed compilation;
- source/artifact compiler parity infrastructure.

Do not rewrite these systems because the final exit tests have not yet been rerun.

## Still materially open

The main implementation gap is **T6**.

`phalcom-semantic/src/artifact/mod.rs` still says, in effect:

```text
Certified-P7 portable producer/importer pipeline = canonical candidate
current-main importer below = in-progress adaptation source
```

and still exposes the large parallel types:

```rust
ImportedCompilerImage
ArtifactImportTransaction
```

with independently staged:

```text
type graph
static values
static constants
static definitions
semantic aggregates
associated authority
callable authority
binding authority
expression authority
conditional dispatch
conformance evidence
specialization
module static bindings
constructors
dispatch
conditional inherent authority
conformance
traits
witness preflight
derived conformance
enum requirements
query products
SemanticWorldV4
QueryImageV4
```

P2 may not close with this as a second complete production semantic authority.

---

# 4. Priority 0 — T6 singular-consumer consolidation

This is where the advanced reasoning time should go first.

## 4.1 First 60–90 seconds: establish the real caller graph

Run:

```bash
rg -n 'ImportedCompilerImage|ArtifactImportTransaction' \
  --glob '*.rs' \
  --glob '!target/**'

rg -n 'publish_query_products_into_empty_db|stage_payload_with_digest|stage_payload\(' \
  --glob '*.rs' \
  --glob '!target/**'

rg -n 'Imported[A-Z][A-Za-z]+Authority' phalcom-semantic phalcom-core phalcom-modules
```

Classify every caller as:

```text
PRODUCTION
TEST
PUBLIC_WRAPPER_ONLY
UNUSED_INTERNAL
GENERIC_PACKAGE_AUTHORITY
```

Do **not** infer usage from exports alone.

The previous audit found no ordinary workspace production consumer outside semantic wrappers/re-exports, but it also found one capability that must not be lost:

```text
generic ProjectUniverse + TrustedPackageResolver package-receipt staging
```

Confirm that quickly.

---

## 4.2 Target architecture

P2 ends with exactly one complete semantic path:

```text
portable bytes
    ->
current validation
    ->
current native SemanticSnapshot
    ->
current query reconstruction
    ->
CanonicalUniverseFormalImage
    ->
SemanticWorkspaceSession::from_image
```

The canonical implementation is the existing P7/current path under:

```text
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/query.rs
phalcom-semantic/src/canonical_image.rs
phalcom-core/src/modules/canonical_artifact.rs
```

`ArtifactImportTransaction` may survive only as:

```text
transport / project-authority staging
validation helper
thin adapter to canonical import
```

It must not remain a second complete semantic publication architecture.

---

## 4.3 Preserve the unique generic package authority behavior

Do not delete or duplicate:

```rust
ProjectUniverse
TrustedPackageResolver
stage_artifact_project_authorities(...)
```

and the exact receipt contract:

```text
StableProjectRef::Package
package name
version
artifact fingerprint
validated ArtifactDigest
persistent ResolvedProjectId
```

The likely good endpoint is:

```text
generic project/package authority staging
        |
        v
canonical native semantic importer
```

not:

```text
generic project authority staging
        |
        v
second independently reconstructed ImportedCompilerImage
```

---

## 4.4 Fastest safe consolidation strategy

Prefer **routing/demotion** over a mass deletion.

### Option A — best if caller graph permits

Make `ArtifactImportTransaction` terminate in the canonical native import result rather than assembling independent semantic authority.

Conceptually:

```rust
ArtifactImportTransaction
    -> validate/stage project authority
    -> call canonical native importer
    -> return canonical imported image/result
```

Do not invent a new semantic DTO.

### Option B — if full routing is too large for the timebox

Narrow its public semantic role now:

1. separate the project/package authority staging from semantic reconstruction;
2. mark the large staged semantic aggregate as non-production / legacy adaptation;
3. make production callers use the canonical loader/importer;
4. reduce visibility of the legacy semantic result if the caller graph proves it is internal-only;
5. leave mechanical deletion to a later patch.

This is still high-value because it establishes **one authoritative production route**.

### Option C — if external API compatibility blocks visibility reduction

Retain the public type as a compatibility wrapper, but have it contain/reference canonical imported authority instead of independently owning all semantic sidecars.

The invariant is more important than the type name.

---

## 4.5 Do not do this

Do not spend the short window:

- deleting 30 importer files blindly;
- reimplementing current native import;
- creating another abstraction layer;
- changing project identity;
- changing wire schemas;
- changing query persistence;
- replacing P7 relocation algorithms;
- "simplifying" exact allocation code.

---

## 4.6 Required T6 disposition ledger

Before leaving T6, record each family as exactly one of:

```text
CANONICAL
RETAIN_HELPER
VALIDATION_ONLY
MERGED_INTO_CANONICAL
SUPERSEDED_REMOVE
HISTORICAL_TEST_ONLY
```

Required final metric:

```text
UNKNOWN = 0
```

If you cannot finish all deletions, put the ledger in the implementation handoff so the next agent can finish mechanically.

---

# 5. Priority 1 — fix the misleading P2 runtime test

This should be a very small patch and gives substantial closure value.

Current live test:

```text
phalcom-core/tests/core/modules/canonical_artifact.rs

artifact_backed_slotmap_runtime_tests_bootstrap_vm_with_generic_intrinsic_adts
```

currently does approximately:

```rust
let art_tests = loaded.compile_user_entry(entry)?;
assert_eq!(art_tests.modules.len(), 8);

let _vm = phalcom_core::vm::VM::new();
```

That does **not** prove runtime compatibility.

## Required P2 semantics

Change the test so it actually executes the program compiled from the artifact-backed compiler image:

```rust
let art_tests = loaded
    .compile_user_entry(entry)
    .expect("artifact compilation of slotmap runtime tests");

assert_eq!(art_tests.modules.len(), 8);

let mut vm = phalcom_core::vm::VM::new();
vm.run_compiled(&art_tests)
    .expect("ordinary VM must execute program compiled from artifact-backed canonical authority");
```

Use the real current `VM::run_compiled(&CompiledProgram)` signature in `phalcom-core/src/interpret.rs`.

If existing test support captures stdout/result, reuse it and assert the normal SlotMap runtime completion output.

If doing exact-output capture would turn into a large harness refactor, do **not** burn the timebox. At minimum make the test call `run_compiled` and rename it so the scope is truthful.

Recommended name:

```text
artifact_backed_compiler_output_runs_on_ordinary_vm
```

or a similarly explicit name.

## Important

This test may use:

```rust
VM::new()
```

and therefore source-backed Universe VM bootstrap.

That is intentional for P2.

The test proves:

```text
artifact compiler authority
    ->
valid executable user CompiledProgram
```

It does **not** prove:

```text
source-free artifact VM
```

That belongs to P3.

---

# 6. Priority 2 — T5 ordinary session implementation gaps

Do not spend time running the expensive semantic integration suite.

Inspect the current implementation and existing tests statically.

Key files:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/canonical_image.rs
phalcom-semantic/tests/semantic/integration/canonical_image.rs
```

Existing high-value tests already include:

```text
ordinary_image_session_keeps_universe_external_across_replacement

ordinary_image_session_local_edit_reuses_unrelated_local_queries_and_retains_universe_identity

ordinary_image_sessions_share_canonical_products_but_isolate_workspace_state

from_image_zero_canonical_entries_copied_cardinality_guard
```

The prior defects were already fixed:

```text
from_image seeds PreludeTypeMap from imported snapshot
canonical DerivedConformanceRuleIndex becomes parent authority
canonical queries remain external
```

## What to look for

Search for an explicit **image-backed rollback/failure** regression.

If none exists and adding one is trivial, add a test that:

```text
1. creates SemanticWorkspaceSession::from_image
2. publishes valid local overlay
3. attempts an invalid/failing local revision
4. asserts failed candidate is not published
5. asserts last_known_good remains prior valid snapshot
6. asserts canonical image/query ownership unchanged
7. performs a valid retry
```

Do not redesign rollback.

If constructing the failure fixture takes more than ~2 minutes, stop and leave the exact missing test in the handoff.

---

# 7. Do not spend advanced-agent time on T1/T2/T3/T4 unless code is obviously stale

These are mainly **acceptance execution** now.

## T1

Live core already uses:

```rust
MODULE_SEMANTIC_ABI_V2
SEMANTIC_ARTIFACT_ABI_V3
QUERY_FINGERPRINT_ABI_V3
STABLE_IDENTITY_SCHEMA_V1
```

Do not revisit unless you find hard-coded conflicting values.

## T2

Do not rewrite semantic authority unless static inspection finds a real TODO/deferred production path.

## T3

The architecture is already accepted:

```text
exact accounting
N/N-1
retry
child rollback
failpoints
Contract B: load ends at CanonicalUniverseFormalImage
```

Later P8 changes mean it needs **rerunning**, not redesign.

Do not spend this short window on `artifact-allocation`.

## T4

Required-root/query policy already has strong focused evidence.

Do not add new persistence semantics.

Only fix a statically obvious manifest/profile inconsistency.

---

# 8. Priority 3 — cheap closure hygiene if time remains

## 8.1 Update stale comments

The R4 plan/checkpoint still contains historical wording such as:

```text
V2/V4/V3
```

where the accepted current query schema is `QueryImageV4`.

Fix active-state comments only.

Do not rewrite historical chronology.

## 8.2 Make the P2/P3 boundary explicit

Where tests/docs still imply:

```text
P2 must source-free run the VM
```

correct them to:

```text
P2 = compiler/session source independence + runtime compatibility
P3 = source-free executable VM
```

## 8.3 Leave an exact verification queue

At the end of your work, write the next commands in the handoff rather than running slow ones.

---

# 9. Timeboxed execution schedule

Use this aggressively.

## Minute 0–1

```text
git status
confirm HEAD
rg caller graph for ImportedCompilerImage / ArtifactImportTransaction
```

Do not read the whole historical plan.

## Minute 1–5

Work only on T6.

Goal:

```text
one authoritative production semantic consumer
or
a concrete routing/demotion patch that makes that outcome explicit
```

If T6 reveals a genuine architecture dependency after ~3 minutes, record it and move on instead of getting stuck.

## Minute 5–7

Patch the runtime-compatibility test to call:

```rust
vm.run_compiled(&art_tests)
```

Do not build it.

## Minute 7–9

Inspect T5 rollback coverage.

If a small test is obvious, add it.

Otherwise document the exact existing symbol to run later.

## Minute 9–10

```bash
git diff --check
git status --short
git diff --stat
```

Write a concise continuation note:

```text
what changed
what remains
what was intentionally not tested
exact slow verification commands
```

Do not start a long build in the last minute.

---

# 10. Files with highest expected value

## T6

```text
phalcom-semantic/src/artifact/mod.rs
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/artifact/query.rs
phalcom-semantic/src/artifact/identity.rs
phalcom-semantic/src/artifact/query_import.rs
phalcom-semantic/src/artifact/query_product_import.rs
phalcom-core/src/modules/canonical_artifact.rs
phalcom-modules/src/project.rs
phalcom-modules/src/compiler_artifact.rs
```

## T5

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/canonical_image.rs
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/tests/semantic/integration/canonical_image.rs
```

## T7 runtime compatibility

```text
phalcom-core/tests/core/modules/canonical_artifact.rs
phalcom-core/src/interpret.rs
phalcom-core/src/modules/compile.rs
phalcom-core/tests/support/vm.rs
```

---

# 11. Hard invariants

Do not violate these to make progress faster.

## Semantic authority

```text
one current TypeStore
one current StaticValueStore
one current SemanticSnapshot model
one current SemanticDb
```

No second artifact-only semantic ontology.

## Identity

Wire references use stable identities.

Do not derive durable identity from:

```text
row index
display name
basename
raw arena ID
VM pointer
```

## Canonical image

Canonical image/query products remain immutable.

Local updates are overlays.

Never mutate canonical query state because a local query fails.

## Project/package authority

Do not invent package identities.

Preserve trusted resolver receipt validation.

## Query

Do not silently publish open-world/unsupported roots as canonical reusable roots.

Do not weaken dependency closure.

## Allocation

Do not approximate P2's exact allocation model.

If later verification fails, fix the escaping allocation.

## P3 boundary

Do not implement executable VM materialization in this R4 patch.

---

# 12. Slow verification queue — leave for the next session

After implementation is complete, the next agent/session should run these in order.

## Cheap/current semantic truth

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic-artifact -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test artifact_query_import_test -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --lib artifact::query_policy -- --nocapture
```

## T5 ordinary session

Run the actual current filters for:

```text
ordinary_image_session_keeps_universe_external_across_replacement
ordinary_image_session_local_edit_reuses_unrelated_local_queries_and_retains_universe_identity
ordinary_image_sessions_share_canonical_products_but_isolate_workspace_state
from_image_zero_canonical_entries_copied_cardinality_guard
```

plus image-backed rollback/fail-closed.

## T3 current-head exact accounting

```bash
PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-r4-close-universe.telosc \
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core \
  --features allocation-diagnostics \
  --test artifact-allocation \
  -- --nocapture --test-threads=1
```

Then:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --lib \
  loaded_artifact_prepublication_failures_are_retryable \
  -- --nocapture
```

## T7

Run exact current tests:

```text
fresh_process_source_denied_artifact_compilation
source_backed_and_artifact_backed_compiler_program_parity
source_backed_and_artifact_backed_negative_diagnostic_parity
negative_parity_constructor_resolution_failure
negative_parity_inaccessible_declaration
negative_parity_invalid_pattern_matching
negative_parity_trait_resolution_failure
artifact_backed_compiler_output_runs_on_ordinary_vm
```

Then source regressions:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test cli-smoke

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime
```

## T8

Only after the above:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib
```

---

# 13. Required final P2 metrics

Do not mark P2 complete without a final receipt containing:

```text
final branch SHA
compiler-only artifact size
compiler-only artifact digest
schema/ABI tuple

N_final
observed == ledger
N-1 typed failure
retry
child rollback
prepublication failpoints

QueryKind count = 46
required root counts by family
missing required = 0
silently pruned required = 0
dependency cycles = 0

ordinary from_image session = PASS
local edit/invalidation = PASS
rollback = PASS
two-session isolation = PASS
source-denied compilation = PASS

production semantic consumers = 1
unknown importer dispositions = 0

positive compiler parity = PASS
negative diagnostic parity = PASS
artifact-compiled runtime compatibility = PASS

workspace --no-run = EXECUTED
P2-owned broad failures = 0
```

Explicitly record:

```text
source-free artifact VM runtime = deferred to ARTF001.C1.P3
```

---

# 14. Success definition for this short advanced-agent window

You are **not** expected to run all R4 certification in 5–10 minutes.

The short-window success hierarchy is:

## Excellent

```text
T6 production-consumer architecture consolidated
+
runtime compatibility test fixed to run artifact-compiled code
+
clean targeted diff
```

## Very good

```text
T6 routing/demotion patch implemented
+
complete disposition ledger
+
runtime compatibility patch
```

## Good

```text
T6 caller graph resolved definitively
+
small safe architectural patch landed
+
exact remaining mechanical work recorded
+
runtime test patch landed
```

## Poor use of the window

```text
waiting 90 seconds for cargo test
rerunning exact allocation
reading historical plans end-to-end
starting P3 VM materializer
rewriting semantic import from scratch
```

Use the expensive reasoning where it matters: **decide and collapse the duplicate consumer architecture without losing generic package authority**. Everything else is comparatively mechanical.

---

# 15. Final instruction

Start here:

```bash
git status --short --branch
git rev-parse HEAD

rg -n 'ImportedCompilerImage|ArtifactImportTransaction' \
  --glob '*.rs' \
  --glob '!target/**'
```

Then attack T6 immediately.

Do not return after reconnaissance if a safe implementation path is evident. Make the code change.

After T6, patch the runtime compatibility test so it actually calls `run_compiled`.

Leave slow verification to the next session with exact commands and a clean, precise handoff.
