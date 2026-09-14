# Step 4 Verification — DB-Owned Formal Semantic Queries

## Baseline

- Upstream repository: `aureat/phalcom-lang`
- Upstream baseline commit: `3ff22158a60db3a323a4a64e8b6ab3957de02408`
- Required prior local patches, in order:
  1. `0001-semantic-query-validity-fail-closed-dependencies.patch`
  2. `0002-semantic-product-fingerprints.patch`
  3. `0003-semantic-read-dependency-capture.patch`
- Step 4 patch: `0004-db-owned-formal-semantic-queries.patch`
- Patch SHA-256: `deb28207841158cdbbede37abc81486892756a38386c35252e36019104a1389c`

A fresh GitHub commit query during packaging still reported `3ff22158` as current `main`.

## Scope

Step 4 transfers ownership of three formal products from `SemanticWorkspaceSession` manual publication into first-class `SemanticDb` queries:

- `HierarchyEdge(DeclarationId)`
- `DeclarationSurface(DeclarationId)`
- `CallableSignature(CallableId)`

The externally built `LinkedProgram` remains a transitional session input. This patch intentionally does **not** perform compiler-owned module lifecycle inversion, broad invalidation removal, or LSP migration.

## Files changed

1. `phalcom-semantic/src/checker/context.rs`
2. `phalcom-semantic/src/db/fingerprint.rs`
3. `phalcom-semantic/src/db/mod.rs`
4. `phalcom-semantic/src/db/query.rs`
5. `phalcom-semantic/src/dispatch.rs`
6. `phalcom-semantic/src/session.rs`
7. `phalcom-semantic/tests/formal_query_ownership.rs` (new)

Patch statistics from `git apply --stat`: 7 files, 554 insertions, 148 deletions.

## Implemented ownership changes

### HierarchyEdge

`query_hierarchy_edge` now:

1. validates/publishes the parsed-module and linked-interface prerequisites;
2. locates the source class declaration;
3. resolves the direct superclass itself;
4. computes its direct input fingerprint;
5. validates cached reuse through `SemanticDb`;
6. records `ParsedModule(owner_module)` and `LinkedInterface(owner_module)` dependencies;
7. computes and publishes `HierarchyEdgeProduct`.

`SemanticWorkspaceSession` no longer constructs or manually publishes `HierarchyEdge` products. It materializes the compatibility `MapTypeHierarchy` from returned query products.

A transitional `hierarchy_edge_input_fingerprint` hashes declaration identity, exact superclass spelling, and the current resolved superclass identity. The resolved target is included because the current linked-interface product does not yet expose every binding consumed by the ambient resolver. This is conservative and prevents stale edge reuse until module/query ownership is completed later.

### DeclarationSurface

`query_declaration_surface` now:

1. validates/publishes parsed-module and linked-interface prerequisites;
2. finds the source class;
3. constructs the class surface itself using `register_class_surface` inside a fresh `CheckingContext`;
4. computes input/product fingerprints;
5. records parsed-module and linked-interface prerequisites;
6. publishes the typed `DeclarationSurface` product.

The session now consumes that product to materialize its compatibility `SurfaceDispatchResolver`; it does not construct a temporary declaration surface and push it into the DB.

Surface construction currently occurs before reuse validation because resolved `TypeId` values are part of the transitional direct-input identity. This favors correctness over optimal recomputation until Step 5/Step 7 give formal queries fully canonical syntax/import inputs.

### CallableSignature

`query_callable_signature` now projects the canonical `CallableSemanticSignature` from the current DB-owned declaration surface and records `DeclarationSurface(owner)` as its prerequisite.

It preserves:

- callable identity/owner/side/selector;
- generic signature;
- parameter order, labels, rest semantics, and canonical type terms;
- canonical return type.

The source/native metadata defaults match the existing source-signature reconstruction path.

## Partial source contracts

The old session reconstruction could publish a misleading canonical signature when the return was known but one or more parameters were unknown, because unknown parameters were dropped during projection.

Step 4 adds `CallableSignature::has_complete_types()`. A source signature is projected only when **every parameter and the return value have a canonical known type**.

For a partial signature:

- `query_callable_signature` publishes no canonical signature product;
- it becomes `Blocked(UnknownType(UnannotatedDeclaration))`;
- body analysis continues to depend fail-closed on `DeclarationSurface(owner)` rather than a nonexistent `CallableSignature` product.

This preserves the Step-1/Step-3 dependency semantics and avoids fabricating a shorter callable contract.

## Session ownership checks

Fresh structural searches after implementation found:

- no `QueryKey::HierarchyEdge`, `QueryKey::DeclarationSurface`, or `QueryKey::CallableSignature` construction in `session.rs`;
- no `SemanticProduct::HierarchyEdge`, `SemanticProduct::DeclarationSurface`, or `SemanticProduct::CallableSignature` publication in `session.rs`;
- production calls to the three formal queries occur from the session, while their computation/publication lives in `db/query.rs`.

This establishes the intended Step-4 ownership boundary: the session orchestrates/materializes; the DB queries compute and publish.

## Tests added

`phalcom-semantic/tests/formal_query_ownership.rs` adds two focused regressions.

### `formal_products_are_query_owned_and_record_prerequisites`

For a small `Base -> Child` program with a fully annotated class method, it asserts:

- a ready typed `HierarchyEdge(Child)` product exists and resolves `Base`;
- hierarchy-edge dependencies include the parsed module and linked interface;
- a ready typed `DeclarationSurface(Child)` exists and owns the class-side method surface;
- declaration-surface dependencies include the parsed module and linked interface;
- a ready typed `CallableSignature(Child.identity)` exists with the expected parameter count;
- the callable-signature dependency is the owning declaration surface.

### `partial_source_signature_stays_surface_backed_without_truncated_callable_product`

For a method whose parameter is unannotated but whose return type is known, it asserts:

- no canonical `CallableSignature` product is published;
- the callable-body query records `DeclarationSurface(Api)`;
- the callable body does not record a dependency on the absent callable-signature product.

This is a direct regression for the previous unknown-parameter truncation behavior.

## Static verification executed

The following checks were executed successfully in this environment:

- `git diff --check` on all seven Step-4 files: clean.
- Patch generation contains exactly seven `diff --git` entries.
- `git apply --check 0004-db-owned-formal-semantic-queries.patch` against a fresh Step-3 baseline: success.
- Actual `git apply` against that fresh baseline: success.
- Byte-for-byte `cmp` between every applied file and the authored Step-4 tree: **7/7 matched**.
- `git diff --check` after round-trip application: clean.
- Delimiter count sanity check on `db/query.rs`: parentheses, braces, and brackets balanced.
- Structural search confirmed no direct session ownership/publication of the three formal products.
- Structural search confirmed formal query callsites are the three session orchestration sites plus the query definitions.

## Rust execution status

Rust compilation and test execution remain unavailable in this sandbox.

Fresh command attempted:

```bash
cargo +nightly-2026-07-10 test -p phalcom-semantic --test formal_query_ownership
```

Result:

- exit status: **127**
- error: `cargo: command not found`

The environment also has no `rustc` or `rustfmt`. Earlier provisioning attempts could not reach the Rust distribution host because DNS/network access was unavailable.

Therefore this report makes **no claim that the patch compiles or that Rust tests pass**. Static and patch-integrity verification are complete; executable verification must be performed in a Rust-enabled checkout.

## Required executable verification after applying

Run with the repository-pinned toolchain:

```bash
cargo +nightly-2026-07-10 fmt --all -- --check
cargo +nightly-2026-07-10 test -p phalcom-semantic --test formal_query_ownership
cargo +nightly-2026-07-10 test -p phalcom-semantic --test callable_dependency_invalidation
cargo +nightly-2026-07-10 test -p phalcom-semantic --test constructor_self_type
cargo +nightly-2026-07-10 test -p phalcom-semantic --test semantic_db_incremental
cargo +nightly-2026-07-10 check --workspace
cargo +nightly-2026-07-10 clippy --workspace --all-targets -- -D warnings
cargo +nightly-2026-07-10 test --workspace
```

## Deliberately deferred findings

These are not hidden by this patch and should remain visible in later steps:

1. **Exact dynamic formal dependency topology is still transitional.** The original Task-4 specification calls for referenced declaration-surface dependencies during annotation/generic-bound resolution and a more precise hierarchy dependency on the resolved superclass declaration product. The current ownership transfer uses parsed/linked prerequisites plus conservative direct-input identity. This avoids inventing incomplete scheduler/SCC behavior before the staged graph owns all prerequisite products. Step 5/Step 7 should refine this topology.

2. **Method-generic annotation scoping predates Step 4.** In `register_class_surface`, method parameter/return annotations are resolved before the method's own generic resolver is installed. This should be repaired separately rather than mixed into an ownership patch.

3. **Persistent type-parameter interning issue discovered in Step 2 remains.** `TypeStore::intern_type_parameter` can reuse an `(owner,index)` identity without refreshing changed binder kind/name/source metadata across revisions. This needs its own correctness repair.

4. **Linked-interface ownership is still transitional.** Step 4 invokes `query_linked_interface` for prerequisites, but the externally linked workspace remains the source. Compiler-owned project/import/link lifecycle is intentionally reserved for the later module-lifecycle step.

## Acceptance statement

At the source/patch level, Step 4 establishes the requested ownership transfer: hierarchy edges, declaration surfaces, and complete callable signatures are computed and published by first-class semantic DB queries, while the workspace session consumes those products to materialize compatibility structures.

Executable acceptance remains pending until the Rust commands above are run successfully.
