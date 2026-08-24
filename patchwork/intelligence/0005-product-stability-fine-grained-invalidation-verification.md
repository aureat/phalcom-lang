# Step 5 Verification — Product-Stability Propagation and Fine-Grained Invalidation

## Patch

`0005-product-stability-fine-grained-invalidation.patch`

SHA-256:

```text
0f739d0a6deae4adcb6f8b6278d5fcfe21592e0f4e4d4ebf7188b74575558626
```

This patch is **incremental on Steps 1–4**. Apply these first, in order:

```text
0001-semantic-query-validity-fail-closed-dependencies.patch
0002-semantic-product-fingerprints.patch
0003-semantic-read-dependency-capture.patch
0004-db-owned-formal-semantic-queries.patch
```

A fresh GitHub repository check made while packaging Step 5 still showed `aureat/phalcom-lang` `main` at:

```text
3ff22158a60db3a323a4a64e8b6ab3957de02408
```

The repository toolchain remains:

```text
nightly-2026-07-10
```

## Goal

Step 5 changes the incremental model from eager reverse-closure deletion on ordinary recomputation to product-stability propagation:

```text
input changed
    -> recompute only that query
    -> preserve cached dependents
    -> republish semantic product
    -> unchanged product fingerprint lets dependents revalidate
    -> changed product fingerprint forces dependents to recompute when queried
```

Hard reverse invalidation is retained for disappearance/removal, where no replacement product can prove semantic stability.

This patch also completes callable-body query ownership sufficiently to remove the duplicate legacy class-body checking pass, while preserving field-initializer and declaration-annotation diagnostics.

## Files changed

```text
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/mod.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/tests/formal_query_ownership.rs
phalcom-semantic/tests/product_stability_invalidation.rs
phalcom-semantic/tests/semantic_db_incremental.rs
phalcom-semantic/tests/type_store_revisions.rs
```

Patch size:

```text
12 files changed
931 insertions
121 deletions
```

No LSP files or module-lifecycle implementation files are changed.

## Implementation changes

### 1. Query-local recomputation preserves incoming dependents

`SemanticDb::discard_for_recompute()` removes the current query state/product and its outgoing dependency edges, but deliberately preserves reverse edges from queries that consumed its previous product.

This is different from `SemanticDb::invalidate()`, which remains the destructive reverse-closure operation used when a product disappears entirely.

The distinction is required for product-stability propagation. A dependent is not eagerly deleted merely because one dependency must recompute.

### 2. Staged query refreshes use local discard rather than reverse invalidation

All ordinary recomputation paths in `phalcom-semantic/src/db/query.rs` now use `discard_for_recompute()`.

A changed query may therefore publish the same `ProductFingerprint` in a newer revision. Step 1's current-revision validation then permits the old dependent product to be revalidated without pretending it recomputed.

### 3. Source edits no longer seed four module products for destruction

`SemanticWorkspaceSession` no longer handles every source-text edit as:

```text
ParsedModule
UnlinkedInterface
LinkedInterface
ModuleDiagnostics
    -> reverse-closure invalidate all
```

Instead, every source is run through `query_unlinked_interface()`, which first brings `ParsedModule` current. A body-only edit therefore recomputes the parse and unlinked-interface query, but an unchanged unlinked semantic product can stop propagation.

Whole-module removal still uses hard reverse invalidation because there is no replacement product whose fingerprint can establish stability.

### 4. Formal queries no longer depend on the whole parsed-module product

`HierarchyEdge` and `DeclarationSurface` retain declaration-specific direct input identity and `LinkedInterface` dependencies, but no longer record `ParsedModule(module)` as a semantic product dependency.

This prevents any implementation-body parse change from automatically invalidating every hierarchy edge and declaration surface in the module.

### 5. Callable-body products own tail-return checking

The DB-owned callable analyzer now type-checks a final expression statement against the callable's expected return type.

This closes behavior previously supplied by the later legacy `check_class_bodies()` pass and allows that duplicate callable pass to be removed from the session.

### 6. Duplicate callable-body checking is removed

After `CallableBody` queries execute, the compatibility diagnostic pass now checks only:

- class field initializer expressions; and
- non-class top-level statements.

It no longer re-runs every method/getter/setter/index body after the DB query already analyzed it.

This is important for performance semantics: a cached callable now avoids the actual second body analysis rather than merely being counted as reused in DB statistics.

### 7. Field initializer diagnostics remain explicit

`check_class_field_initializers()` consumes the already-published field surface and checks default expressions against it.

This preserves field-default mismatch diagnostics while avoiding duplicate annotation resolution for the field contract.

### 8. Declaration-surface queries retain their annotation diagnostics

`SemanticProduct::DeclarationSurface` now wraps a `DeclarationSurfaceProduct` containing:

```text
semantic DeclarationSurface
declaration/member annotation diagnostics
```

Diagnostics participate in the declaration-surface **input** fingerprint so source/range/detail changes refresh the stored query payload.

They do not participate in the semantic declaration-surface **product** fingerprint, so diagnostic-only/provenance changes do not force callable consumers to recompute.

### 9. Persistent generic parameters are revision-safe

The persistent `TypeStore` exposed a separate correctness hole during Step-5 review.

Previously:

```text
(owner, parameter index) -> one eternal TypeParameterId
```

Changing a binder's name/kind/variance returned the old ID and old metadata. A kind edit was worse: `TypeData::Parameter(old_id)` and nominal declaration forms could retain their old `KindId` in a persistent session.

Step 5 now treats `(owner, index)` as the lookup for the **current binder version**:

- if name/kind/variance are unchanged, the existing `TypeParameterId` is reused;
- source-only provenance movement refreshes the live parameter metadata without changing semantic identity;
- if semantic binder data changes, a new `TypeParameterId` is allocated and becomes the current mapping;
- old IDs/data remain intact for cached products and retained snapshots.

### 10. Kind is part of TypeStore interning identity

`TypeStore::type_to_id` is now keyed by:

```text
(TypeData, KindId)
```

rather than `TypeData` alone.

This permits a declaration/parameter payload to acquire a different kind in a newer revision without mutating or aliasing the old canonical `TypeId`.

Old snapshots and old cached products therefore keep their original denotation.

## Regression coverage

### `semantic_db_incremental.rs`

Added direct DB laws:

- local recomputation does not eagerly delete dependents;
- unchanged replacement product fingerprint lets dependent revalidate;
- changed replacement product fingerprint leaves dependent cached but non-reusable until recomputed.

### `product_stability_invalidation.rs`

Added workspace-level regressions for:

1. body-only edit:
   - `ParsedModule` recomputes;
   - `UnlinkedInterface` reevaluates;
   - edited body recomputes;
   - linked interface, hierarchy, surface, signature, and unaffected caller remain old products validated in the new revision;
   - unaffected callable `Arc` identity is retained;
2. callable-body-owned tail-return diagnostics;
3. field initializer diagnostics after removing duplicate class-body analysis;
4. declaration-surface-owned unresolved annotation diagnostics;
5. signature edit:
   - owning surface/signature recompute;
   - unchanged exact caller body recomputes through recorded callable dependency;
   - unrelated callable remains reused;
6. superclass edit:
   - exact `HierarchyEdge` recomputes;
   - unchanged dispatch consumer recomputes through hierarchy dependency;
   - unrelated callable remains reused.

### `type_store_revisions.rs`

Added regressions proving:

- semantic generic binder changes allocate new `TypeParameterId` versions;
- old parameter forms retain their previous kind in both retained snapshots and the live persistent store;
- source-only binder movement reuses semantic identity while refreshing current provenance;
- a workspace edit from `F: Type -> Type` to `F: Type` versions both parameter and nominal declaration forms instead of aliasing stale kinds.

### `formal_query_ownership.rs`

Updated ownership assertions to enforce that hierarchy/surface queries use declaration-specific direct inputs rather than whole-module `ParsedModule` product dependencies.

## Static verification performed

### Patch hygiene

Run on the authored Step-5 tree:

```bash
git diff --check
```

Result:

```text
PASS
```

### Invalidation audit

Search of production semantic code after Step 5 found:

```text
phalcom-semantic/src/session.rs
```

as the only remaining call site of destructive `db.invalidate(...)` in the Step-5 path, and that call is inside the **removed-module** branch.

There are no ordinary staged-query recomputation calls to `db.invalidate(...)` in `db/query.rs`.

### Duplicate-body-pass audit

Search of `session.rs` found no remaining `check_class_bodies` call.

### TypeStore audit

Static checks confirmed:

```text
type_to_id: HashMap<(TypeData, KindId), TypeId>
```

and semantic parameter versioning is present before the current `(owner,index)` mapping is replaced.

### Exact Step-4 patch round trip

A fresh tree was reset to the exact committed Step-4 content used when `0004` was packaged.

Run:

```bash
git apply --check 0005-product-stability-fine-grained-invalidation.patch
git apply 0005-product-stability-fine-grained-invalidation.patch
git diff --check
```

Result:

```text
PASS
```

Every one of the 12 patched files was then byte-compared with the authored Step-5 working tree:

```text
roundtrip-byte-compare: OK
changed-files: 12
```

### Full patch-series reconstruction

A completely fresh directory was extracted from the original uploaded `phalcom-main.zip` baseline.

The five patches were checked and applied sequentially:

```text
CHECK 0001-semantic-query-validity-fail-closed-dependencies.patch
CHECK 0002-semantic-product-fingerprints.patch
CHECK 0003-semantic-read-dependency-capture.patch
CHECK 0004-db-owned-formal-semantic-queries.patch
CHECK 0005-product-stability-fine-grained-invalidation.patch
```

All five `git apply --check` operations succeeded, all five patches applied, and every Step-5 file byte-matched the authored tree:

```text
series-apply-and-byte-compare: OK
```

## Rust execution status

This sandbox still contains no Rust toolchain.

Fresh environment check:

```text
cargo: command not found
rustc: command not found
```

Therefore this report **does not claim**:

- Rust compilation succeeds;
- focused tests pass;
- Clippy passes;
- workspace tests pass.

Static patch/invariant verification is complete, but executable verification must be run in a Rust-enabled checkout.

## Required executable verification after applying Step 5

Use the repository-pinned `nightly-2026-07-10` toolchain.

Run focused Step-5 gates first:

```bash
cargo fmt --check

RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic_db_incremental -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test product_stability_invalidation -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test type_store_revisions -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test formal_query_ownership -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test callable_dependency_invalidation -- --nocapture
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test checker_dependency_tracking -- --nocapture
```

Then run the semantic crate gate:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
```

Then workspace verification:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
RUST_MIN_STACK=8388608 cargo test --workspace
```

If any focused Step-5 test fails, stop before applying later patches and preserve the full compiler output.

## Deliberate remaining boundaries

### Import/export propagation remains transitional

The compiler-owned module lifecycle has not yet moved into `SemanticWorkspaceSession`. `LinkedInterface` is still supplied from the externally assembled `LinkedProgram`, and its current DB query does not derive itself from `UnlinkedInterface`/resolved-import products.

For that reason Step 5 does **not** pretend to prove the final import/export invalidation law from the architectural completion spec. Exact import/export propagation belongs to the later compiler-owned module-lifecycle/query-ownership step, where those dependency edges can become real rather than synthetic.

### Linked-interface dependencies are still coarse

Declaration/body reads currently depend on module-level linked-interface products in several places. Step 3 made those reads explicit and sound; later query-topology work can narrow them to exact resolved bindings without changing the product-stability mechanism added here.

### Declaration-surface reuse still computes a candidate input

The declaration-surface query currently resolves a candidate surface/diagnostics before deciding whether the stored semantic product can be reused. Step 5 prevents downstream recomputation when the semantic product is stable, but this is not yet the minimum-cost syntax-keyed surface query. That optimization should be performed when declaration/snapshot projection ownership is tightened, rather than by introducing another temporary authority.

## Step-5 acceptance assessment

At source level, this patch establishes the central fine-grained invalidation law:

```text
new revision != semantic change
```

Recomputation revision and semantic product identity are distinct. A stable recomputed product can stop propagation; a changed product forces only queries that recorded a dependency on it to recompute when demanded.

The patch also removes the major hidden duplicate-body-analysis cost and makes persistent type/kind identities safe across generic semantic edits.

Executable acceptance remains pending the Rust commands above.
