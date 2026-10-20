# PERF007 Semantic Import Allocation Accounting Plan

**Status:** Complete (Verified: FOCUSED_TESTED, delta == 0)

**Worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`

**Branch:** `perf/universe-characterization`

**Scope:** Close the remaining allocation-accounting gap in the successful semantic-world import path. This plan is intentionally standalone and does not change the PERF003 plan or checkpoint schema.

## Objective

Make the successful `SemanticWorldV3` import path account for every allocator-requested byte with the same `AllocationBudget` transaction that owns the semantic import. Preserve semantic identity, source/frozen ownership, transactional rollback, canonical prefix behavior, and existing wire formats.

The immediate discriminator is:

```text
Type graph import: observed 765,031 bytes = ledger 765,031 bytes
Semantic import:   observed 265,533,280 bytes
                   ledger   264,389,786 bytes
                   delta      1,143,494 bytes
```

The plan is complete only when the independent semantic-import discriminator reports `observed == ledger`, exact budget `N` succeeds, budget `N-1` returns typed allocation exhaustion, and failed imports commit no parent allocation or snapshot.

## Rules

- Work only in the requested `perf-universe-characterization` worktree.
- Do not reset, clean, checkout, overwrite, or broadly format the dirty worktree.
- Preserve all existing dirty source and evidence edits, especially the current changes in `phalcom-semantic/src/artifact/import.rs` and `phalcom-semantic/src/artifact/operational.rs`.
- Do not modify PERF003 records as part of this plan. Add or update only files under `docs/implementation/PERF007` unless implementation work later requires a source/test edit.
- Do not use allocator backtraces to classify budget ownership. Budget helpers precharge before the allocator call, so a stack frame is not a reliable discriminator.
- Do not estimate B-tree, HAMT, `im` node, or allocator-internal bookkeeping bytes.
- Do not construct a native map/set and convert it into frozen storage. Use a precharged contiguous buffer, sort in place, reject duplicates, and publish the frozen owner.
- Do not remove `require_semantic_import_allocation_accounting()` or any other fail-closed guard until successful-path equality is proven.
- Do not weaken strict equality, accept any `N-1` error, or count a malformed-input error as allocation exhaustion.
- Use typed `ArtifactError::Allocation(AllocationError::...)` failures on budget exhaustion paths. Do not format an owned error string after the budget may be exhausted.

## Current Source of Truth

Read these files before changing code:

- `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7-R1/reports/PERF003.C2.P7.PATCH5-R-continuation-handoff.md`
- `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7-R1/patches/PERF003.C2.P7.PATCH5-R-Luna-Patch-Grade-Implementation-Plan.md`, especially the semantic owner closure section
- `phalcom-core/tests/artifact_allocation.rs`
- `phalcom-semantic/src/artifact/import.rs`
- `phalcom-semantic/src/artifact/operational.rs`
- `phalcom-semantic/src/static_value/artifact.rs`
- `phalcom-compiler-artifact/src/semantic/world.rs`
- `phalcom-compiler-artifact/src/semantic/static_value.rs`
- `phalcom-type-meta/src/allocation.rs`

The current semantic-import root is:

```text
validate_and_import_semantic_world_with_allocation_budget
  -> validate_and_import_semantic_world_inner
     -> world.validate_with_allocation_budget
     -> TypeStore::import_type_graph_with_allocation_budget
     -> import_structural_primary_contracts
     -> import_static_value_graph_with_allocation_budget
     -> callable / trait / dispatch / proof / explanation import
     -> static definition / static constant / module binding import
     -> formal projection
     -> SemanticSnapshot construction
```

The type graph path is already exact. Do not rework it unless a later compiler error proves a direct dependency requires a mechanical adaptation.

## Files To Change

Expected source/test files:

- `phalcom-type-meta/src/scalar.rs`
  - Add or correct exact budget-aware construction for large `StaticInt` values if the existing helper cannot account for temporary `BigInt` backing storage.
- `phalcom-semantic/src/types/artifact.rs`
  - Reuse the existing exact decimal integer parser and make its temporary limb and packed-digit allocations obey the supplied budget.
- `phalcom-semantic/src/static_value/artifact.rs`
  - Replace direct `BigInt::parse_bytes` calls for static values and record keys with the budget-aware parser.
- `phalcom-semantic/src/artifact/import.rs`
  - Replace remaining unbudgeted recursive boxes, collected vectors, identity clones, `BindingState` clones, and normal-return scratch construction.
  - Add narrowly scoped budget-aware clone helpers only for owners reached by this import path.
- `phalcom-semantic/src/artifact/operational.rs`
  - Validate and complete the existing dirty budget-threading edits. Do not discard them or replace them with an alternate design.
- `phalcom-semantic/src/checker/analysis.rs`
  - Budget cloned module identities in frozen callable-analysis indexing if required by the compiler after the first source slices.
- `phalcom-semantic/src/presentation.rs`
  - Budget transitive `BindingConsistency`/`BlockReason`/`QueryKey` ownership in frozen formal projection if required by the discriminator.
- `phalcom-semantic/src/checker/analysis_map.rs`, `analysis_set.rs`, `source_index/interval.rs`, or existing allocation helpers
  - Touch only if a helper needs an exact fallible constructor for an already identified import-path allocation.
- `phalcom-type-meta/tests/allocation_helpers.rs`
  - Extend only for a reusable scalar allocation helper or exact layout rule.
- `phalcom-semantic/tests/semantic/artifact_allocation.rs`
  - Add focused unit/integration coverage for exact integer parsing and rollback only if existing tests do not cover the corrected owner.
- `phalcom-core/tests/artifact_allocation.rs`
  - Use the existing semantic discriminator. Do not make strict equality unconditional in this plan; that belongs to the later whole-load gate after semantic equality is closed.

## Allocation Accounting Model

Use the existing helpers in `phalcom-type-meta/src/allocation.rs`:

- `vector(count)` charges the initial exact vector layout.
- `grow_vector_exact` charges replacement layouts for growth.
- `boxed(value)` charges `Layout::new::<T>()` before `Box::try_new`.
- `boxed_slice(values)` charges a shrink request when capacity exceeds length.
- `arc(value)` charges the `ArcInner` layout.
- `arc_slice(values)` charges shrink plus the `Arc<[T]>` layout.
- `string`, `boxed_str`, and `arc_str` charge owned string bytes.
- `child` plus `commit_child` gives nested import rollback.

Every retained owner must be one of:

1. Source-only and unreachable from charged import.
2. Allocation-free, with a concrete proof that it does not allocate.
3. Scratch allocated by an explicit budget helper.
4. Retained frozen storage allocated by an explicit budget helper.
5. `Arc::clone` or borrowed reuse with no new heap allocation.

If a path cannot be classified, stop that slice and inspect the owner rather than adding an estimate.

## Step 0: Establish the Exact Baseline

Do not repeat the PERF003 R.T0 baseline suite. Capture only the current worktree anchor and the discriminator needed for this plan.

### Commands

Run from the worktree:

```bash
git branch --show-current
git rev-parse HEAD
git status --short
git diff --check
```

Expected:

- branch is `perf/universe-characterization`;
- existing dirty files are listed and preserved;
- `git diff --check` passes.

Run the focused library check:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
```

Run the exact semantic discriminator using the already generated artifact:

```bash
PHALCOM_UNIVERSE_TELOSC_FOR_TEST=/tmp/perf003-patch5r-universe.telosc \
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test artifact-allocation \
  full_universe_telosc_load_allocation_equality_and_n_minus_one \
  -- --nocapture
```

Record the current observed bytes, ledger bytes, and delta in the implementation session notes. The expected current result is approximately:

```text
TYPE GRAPH IMPORT: ledger=765031 observed=765031 delta=0
semantic import: observed=265533280 ledger=264389786 delta=1143494
```

Do not interpret a changed baseline as a fix until the source diff and command output identify the cause.

## Step 1: Audit The Existing Dirty Diff Before Editing

Inspect only the relevant dirty diff:

```bash
git diff -- \
  phalcom-semantic/src/artifact/import.rs \
  phalcom-semantic/src/artifact/operational.rs \
  phalcom-semantic/src/static_value/artifact.rs \
  phalcom-compiler-artifact/src/semantic/static_value.rs \
  phalcom-compiler-artifact/src/semantic/world.rs \
  phalcom-semantic/src/checker/analysis.rs \
  phalcom-semantic/src/presentation.rs
```

Do not use a whole-worktree `git diff` for this audit. The worktree contains
preserved edits outside this owner slice.

Classify each existing edit as:

- already budgeted and semantically correct;
- compiling but incomplete;
- unrelated to this plan;
- suspicious because it duplicates, clones, or reconstructs an owner.

Do not revert unrelated edits. Do not rewrite a correct dirty slice merely to improve style. The immediate expected finding is that operational static-program import is budget-threaded but equality remains nonzero, so it is not the only remaining owner.

## Step 2: Fix Large Static Integer Parsing First

This is the strongest confirmed allocation boundary.

### Existing defect

The following paths parse a `BigInt` directly:

- `phalcom-semantic/src/static_value/artifact.rs:919`
- `phalcom-semantic/src/static_value/artifact.rs:1006`

They call `BigInt::parse_bytes`, then call `StaticInt::new_with_allocation_budget`. The latter charges only the `Arc<BigInt>` owner for a large integer. The parser has already allocated backing storage, so the independent allocator observes bytes not present in the ledger.

### Required implementation

1. Inspect `phalcom-semantic/src/types/artifact.rs:2563` (`integer_in_with_allocation_budget`).
2. Change its visibility from private to `pub(crate)` and treat that parser as the canonical implementation for normalized decimal integers. Keep it in the semantic owner that already understands `ValidationLimits` and `StaticInt`; do not duplicate the limb algorithm in `static_value/artifact.rs`.
3. Confirm its accounting for:
   - temporary `Vec<u32>` limb storage;
   - packed `Vec<u64>` storage used by `num_bigint` on 64-bit hosts;
   - `Arc<BigInt>` allocation for `StaticInt::Large`.
4. Call it from static-value import rather than duplicating parsing logic. The required call shape is:

```rust
let value = crate::types::artifact::integer_in_with_allocation_budget(
    decimal,
    limits,
    allocations,
)?;
```

5. If the current module boundary prevents that call without a cycle, move only the reusable scalar parser into the owning scalar/type-meta layer and expose the same three inputs (`decimal`, string limit, `&mut AllocationBudget`). Keep the wire validation rules unchanged:
   - decimal only;
   - normalized sign;
   - no leading zeroes except `0`;
   - bounded by `ValidationLimits::max_string_bytes`.
6. Replace the direct parser at both static-value sites with the budgeted parser.
7. Add a `limits: &ValidationLimits` parameter to `key_from_wire_budgeted`, then thread it from `import_static_value_graph_with_allocation_budget` at the record construction call site. Also pass `limits` into the value loop so both integer forms use the same parser.
8. Do not parse into an unbudgeted `BigInt` and then attempt to charge afterward.

### Tests

Add or extend focused tests with:

- an `i64`-fit decimal, proving no large-integer `Arc` allocation is charged;
- a just-outside-`i64` positive integer;
- a just-outside-`i64` negative integer;
- a large record key;
- exact budget success;
- one-byte-short typed exhaustion with caller ledger unchanged on nested failure.

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib static_value -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_values -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
```

If the exact test name differs, list tests first:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --list | rg 'static|record|artifact'
```

Then rerun the exact semantic discriminator. Record the new delta. A material reduction confirms the owner; equality is still required.

## Step 3: Close Unbudgeted Recursive Boxes And Temporary Vectors

Search the charged importer only. Do not change source-side constructors.

```bash
rg -n 'Box::new|collect::<Result<Vec|into_boxed_slice|Vec::new|Vec::with_capacity' \
  phalcom-semantic/src/artifact/import.rs \
  phalcom-semantic/src/artifact/import/explanation.rs \
  phalcom-semantic/src/artifact/import/proof.rs \
  phalcom-semantic/src/artifact/operational.rs
```

### 3A. Witness selection fallback

At `phalcom-semantic/src/artifact/import.rs` around the recursive fallback in `import_callable_requirement_selection`, replace:

```rust
import_callable_requirement_selection(...).map(Box::new)
```

with a closure that calls the budget helper:

```rust
import_callable_requirement_selection(...)
    .map(|selection| importer.allocations.boxed(selection).map_err(allocation_error))
```

Keep recursion-depth validation unchanged. The box must be charged before allocation and must remain inside the importer child ledger.

### 3B. Diagnostic vectors

At the diagnostics paths around `import_callable_associated_type_plan` and the witness-plan import:

1. Allocate a vector with `allocations.vector(record.diagnostics.len())`.
2. Push each imported diagnostic into that vector.
3. Convert with `allocations.boxed_slice(...)` or `allocations.arc_slice(...)` according to the destination field.
4. Do not use `.collect::<Result<Vec<_>, _>>()?` in a charged path unless the vector allocation itself is explicitly charged before collection.

Apply the same rule to any remaining proof/explanation helper that maps into a boxed slice. Preserve the existing `map_boxed` helper pattern where it already exists.

### 3C. Normal-return synthetic status

At `phalcom-semantic/src/artifact/import.rs` around the normal-return loop:

1. Stop cloning `exit.knowledge`, `exit.status`, or other wire rows solely to call `import_callable_expression_statuses`.
2. Add a small private conversion helper that converts `CallableExpressionStatusV1` and its causal invalidity directly into the native `AnalysisStatus` using the current child budget.
3. Use that helper for the synthetic normal-return status.
4. Keep `import_type_knowledge_with_allocation_budget` for the actual `exit.knowledge` import.

The goal is to remove unneeded temporary wire-owned allocations, not to remove any retained semantic fact.

The direct conversion helper should have this shape near
`import_callable_expression_statuses`:

```rust
fn import_callable_status(
    status: &CallableExpressionStatusV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<AnalysisStatus, ArtifactError> {
    // Copy the existing match arms from import_callable_expression_statuses.
    // Only arms containing owned reasons call allocation-aware helpers.
}
```

Use this helper for the synthetic status row. Do not allocate a temporary wire
row merely to reuse a helper that expects a full expression record.

### Tests and discriminator

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib artifact:: -- --nocapture
PHALCOM_UNIVERSE_TELOSC_FOR_TEST=/tmp/perf003-patch5r-universe.telosc \
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation \
  full_universe_telosc_load_allocation_equality_and_n_minus_one -- --nocapture
```

If the delta does not change, retain the edits if they are proven correct and continue the owner audit. A zero change does not prove that the path is unreachable; use the actual call path and fixture contents.

## Step 4: Budget Module-Binding Owner Clones

The module-binding importer retains complete binding facts, so shallow container charging is insufficient.

Inspect:

- `phalcom-semantic/src/artifact/import.rs:5661-5670`
- `phalcom-semantic/src/module_bindings.rs:163-172`
- `phalcom-semantic/src/checker/analysis.rs:335-349`
- `phalcom-semantic/src/types/evidence.rs`
- `phalcom-semantic/src/types/outcome.rs`
- `phalcom-semantic/src/types/denotation.rs`

### Required implementation

1. Add a budget-aware clone method for `BindingState`, either beside its existing constructors in `checker/analysis.rs` or in a small owner module helper.
2. The method must copy and charge:
   - `name` via `allocations.string` or `boxed_str` matching the destination type;
   - `parameter` through a budgeted `CallableParameterId` clone;
   - `contract.source` as scalars and `contract.ty` as a relocated ID;
   - `current` through the existing budget-aware `TypeKnowledge` clone/import path;
   - `denotation` through a budget-aware semantic-denotation clone or relocation helper;
   - `consistency`, including nested `UnknownReason`, `BlockReason`, `StableModuleKey`, `QueryKey`, strings, and boxed messages;
   - scalar flags and IDs without allocation.
3. Add budget-aware helpers for any nested enum that currently derives `Clone` but owns heap data.
4. In `import_module_static_binding_indexes`, replace `state: row.state.clone()` with the budget-aware clone.
5. Avoid cloning the complete `SymbolId` twice. Build the key and fact symbol intentionally:
   - either charge two real owned symbols if both are retained independently;
   - or construct one owned symbol and borrow/reuse it where the destination type allows.
6. Preserve the fact’s canonical symbol, body-qualified binding identity, source ranges, phase, and visible aliases.

The minimum helper decomposition should be:

```rust
impl BindingState {
    pub(crate) fn clone_with_allocation_budget(
        &self,
        relocation: &TypeRelocation,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, AllocationError>;
}

fn clone_binding_consistency_with_allocation_budget(
    consistency: &BindingConsistency,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<BindingConsistency, ArtifactError>;

fn clone_block_reason_with_allocation_budget(
    reason: &BlockReason,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<BlockReason, ArtifactError>;

fn clone_query_key_with_allocation_budget(
    key: &QueryKey,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<QueryKey, ArtifactError>;
```

Adapt visibility and error types to the owning modules. The contract is that
every owned string, stable identity, nested denotation, and boxed reason is
charged exactly once before its allocation. Use `relocation` for type IDs; do
not copy process-local IDs from the wire.

### Validation

Add a focused unit test that imports one module-binding row containing:

- an owned binding name;
- a known type;
- an owned unresolved-name or blocked reason;
- a module-value or denotation payload;
- a visible-name alias.

Assert exact budget success and N−1 typed exhaustion with no caller debit on failure.

Run the semantic discriminator after the focused test.

## Step 5: Budget Callable-Analysis Index And Formal Projection Clones

Only perform this step after the previous steps compile. These are retained owner/index paths, not scratch estimates.

### 5A. Callable-analysis module index

Inspect `phalcom-semantic/src/checker/analysis.rs:788-814`.

The current frozen constructor clones each module identity with `let module = rows[start].0.owner.module().clone();`. Replace it with a budget-aware module clone before pushing `module_ranges`:

```rust
let module = rows[start]
    .0
    .owner
    .module()
    .clone_with_allocation_budget(allocations)
    .map_err(CallableAnalysisTableBuildError::Allocation)?;
```

Keep the existing sorted rows and module range semantics. Do not charge `Arc::clone` unless a new allocator request occurs.

### 5B. Formal projection consistency payload

Inspect `phalcom-semantic/src/presentation.rs:776-840` and the `BindingConsistency` ownership tree.

1. Add budget-aware cloning for `BindingConsistency` and nested owned `BlockReason`/`QueryKey` values.
2. In the formal projection loop, replace the direct `binding.consistency.clone()` with the budget-aware clone.
3. Keep `FormalContractRelation` semantics unchanged.
4. Ensure the frozen projection’s `FormalFactSite` owns exactly the data it previously owned; do not remove status, causal invalidity, contract origin, or source attachment.

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib presentation -- --nocapture
```

Then rerun the semantic discriminator.

## Step 6: Complete The Operational Static Import Audit

The dirty operational edits must be validated, not assumed complete.

Inspect:

- `phalcom-semantic/src/artifact/operational.rs:756-877`
- `phalcom-semantic/src/artifact/operational.rs:949-975`
- `phalcom-semantic/src/artifact/operational.rs:1045-1106`
- `phalcom-semantic/src/artifact/import.rs:5435-5505`

For every node in `OperationalNode`, verify:

- nested `Box` uses `allocations.boxed`;
- child vectors use `allocations.vector` with exact source lengths;
- boxed slices use `allocations.boxed_slice`;
- `Arc` slices use `allocations.arc_slice`;
- stable modules, declarations, selectors, symbols, and parameter names use budgeted identity/string helpers;
- dependencies passed into the canonical program are already budgeted and are not cloned again without charge;
- the canonical program itself is wrapped with `allocations.arc` exactly once at the retained owner boundary.

Remove no semantics and do not change the operational wire schema. Fix only missing budget ownership or mechanical compiler errors.

Run:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib artifact::operational -- --nocapture
```

Then rerun the semantic discriminator.

## Step 7: Audit Validator Scratch That Runs Inside Semantic Import

The semantic import calls `world.validate_with_allocation_budget` before owner materialization. Inspect every helper called by `SemanticWorldV3::validate_inner`.

Start with these known calls:

- `self.type_graph.validate(limits)`
- `validate_callable_signature`
- `validate_static_constants`
- `validate_callable_definition`
- `validate_trait_surface`
- `validate_declaration_surface`
- `validate_conditional_inherent_set`
- `validate_callable_block_reason`
- `validate_callable_denotation`
- `validate_callable_flow_state`
- `validate_static_case_owners`
- `validate_module_static_bindings`
- `proof::validate_callable_proofs`
- `CallableAnalysisOrdinalContext::validate`

For each helper:

1. Search for `Vec`, `BTreeSet`, `BTreeMap`, `collect`, and temporary owned strings.
2. If it runs on the measured semantic path and allocates, add an explicit `&mut AllocationBudget` parameter.
3. Precharge the exact scratch layout before allocation.
4. Keep validation-only scratch in a local child budget where failure must roll back.
5. Preserve validation order and rejection behavior.
6. Do not convert validation error strings to owned formatting after exhaustion.

The type graph validator currently performs only scalar length checks and is already exact; do not add a second type graph scratch ledger.

Run the semantic discriminator after each validator family, not after speculative broad edits.

## Step 8: Inspect Remaining Direct Allocation Boundaries

Use targeted searches after the previous corrections:

```bash
rg -n 'Arc::new|Arc::from|Box::new|into_boxed_slice|Vec::new|Vec::with_capacity|collect::<|format!|to_string|to_owned|clone\(\)' \
  phalcom-semantic/src/artifact/import.rs \
  phalcom-semantic/src/artifact/import \
  phalcom-semantic/src/artifact/operational.rs \
  phalcom-semantic/src/static_value/artifact.rs \
  phalcom-semantic/src/checker/analysis.rs \
  phalcom-semantic/src/presentation.rs
```

For every match reachable from `validate_and_import_semantic_world_inner`:

- mark it source-only, allocation-free, shared/no-new-allocation, or budgeted;
- if unresolved, trace the caller and destination type;
- fix the owning helper, not the caller with a compensating estimate.

Pay special attention to:

- `SemanticSnapshot` empty `BTreeMap`/`Arc` fields;
- `SemanticGraph::default()` and its authority backing;
- source/index/advisory empty owners;
- frozen table wrapper `Arc`s;
- `TypeStore` and `StaticValueStore` clones;
- stable identity clones used as map keys and values;
- `Arc<[T]>` conversion shrink reallocations.

An empty retained owner still needs an explicit layout decision. If its standard-library construction is allocation-free, prove it with a focused allocator test. If it allocates, charge the exact layout through an existing helper.

## Step 9: Rerun The Discriminator After Each Coherent Owner Slice

Use this exact command:

```bash
PHALCOM_UNIVERSE_TELOSC_FOR_TEST=/tmp/perf003-patch5r-universe.telosc \
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test artifact-allocation \
  full_universe_telosc_load_allocation_equality_and_n_minus_one \
  -- --nocapture
```

For each run record:

```text
slice name:
source files:
observed bytes:
ledger bytes:
delta:
semantic check result:
interpretation:
```

Interpretation rules:

- delta decreases: the slice owned at least part of the gap;
- delta unchanged: the slice may be unreachable on this fixture or its charge is already duplicated elsewhere; inspect before removing it;
- delta increases: stop and inspect for double charging or a changed measured path;
- equality zero: do not immediately remove guards; proceed to exact-N/N−1 and rollback proofs.

## Step 10: Prove Semantic Exact N/N−1 And Rollback

Once successful semantic equality is zero:

1. Run the semantic import with its discovered exact ledger `N`.
2. Assert successful import and `observed == ledger == N`.
3. Run with `N-1`.
4. Assert `Err(ArtifactError::Allocation(AllocationError::Exhausted | Overflow | Failed))` as appropriate. Do not accept arbitrary errors.
5. Assert the parent transaction’s `requested()` remains zero after failed child import.
6. Trigger a malformed semantic-world failure after at least one allocating owner family and assert no snapshot is returned.
7. Retry with sufficient budget and assert success.

If the existing core test only reaches whole-load behavior, add a focused semantic test in `phalcom-semantic/tests/semantic/artifact_allocation.rs` that invokes `validate_and_import_semantic_world_with_allocation_budget` directly with a decoded `SemanticWorldV3`.

## Step 11: Prove Canonical Prefix And Overlay Behavior

After allocation equality is exact, add or run focused tests for:

- canonical type prefix lookup after import;
- canonical static-value prefix lookup after import;
- fork A local insertion not visible in fork B;
- canonical values/types remain visible in both forks;
- structurally equal local insertion resolves to the canonical ID where required;
- foreign store relocation is rejected;
- `extends_canonical_prefix` succeeds for valid forks and fails for altered prefixes;
- loaded frozen maps reject mutation or cannot shadow canonical keys.

Use existing APIs and tests before adding new helpers. Keep local source overlays mutable and loaded canonical backing immutable.

## Step 12: Final Focused Verification For PERF007

Run serially:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta --test allocation_helpers
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib artifact:: -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation \
  full_universe_telosc_load_allocation_equality_and_n_minus_one -- --nocapture
git diff --check
```

If a command fails:

- classify it as caused by the PERF007 source changes, a mechanical test/API issue, or a baseline failure;
- fix caused failures before declaring the plan complete;
- do not broaden into workspace tests until this focused set is green;
- do not claim R.G1 or whole-load equality based on a compile-only pass.

## Completion Criteria

PERF007 is complete only when all of these are true:

- `cargo check -p phalcom-semantic --lib` passes.
- Focused scalar/static-value allocation tests pass.
- Successful semantic import has `observed == ledger` with zero delta.
- Exact semantic budget `N` succeeds.
- Semantic budget `N-1` returns typed allocation exhaustion.
- Failed child import does not commit its requested bytes to the parent.
- Malformed semantic input publishes no snapshot.
- Operational static definitions retain equivalent semantics and fingerprints.
- Module-binding, callable-analysis, formal-projection, proof, explanation, and diagnostic owners preserve all fields while charging owned allocations.
- Canonical type/static prefixes and local overlays pass identity/fork tests.
- No allocation is covered by a guessed byte estimate.
- `git diff --check` passes.

Do not mark the broader PERF003 PATCH5-R gates complete from this plan alone. Query reconstruction, formal-image publication, whole-load unconditional equality, package parity, harness repairs, and later release gates remain outside PERF007 unless explicitly executed and verified separately.

## Stop Conditions

Stop and record an architectural incident instead of continuing if:

- a consumer requires mutable access to loaded frozen authority;
- a required allocation cannot be represented by an explicit fallible layout;
- preserving identity requires re-interning or a second canonical source;
- a source/frozen representation changes iteration order or fingerprints;
- two plausible fixes have different ownership or semantic-authority consequences;
- equality remains nonzero after all confirmed direct boundaries are closed and one bounded classification pass cannot identify the next owner.

An incident record must contain the current revision, exact command/output, call path, invariant at risk, candidate fixes, and the specific architectural decision needed.
