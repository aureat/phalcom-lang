# Step 3 Verification — Semantic Read Dependency Capture

## Patch

`0003-semantic-read-dependency-capture.patch`

This is an **ordered patch**. Apply it after:

1. `0001-semantic-query-validity-fail-closed-dependencies.patch`
2. `0002-semantic-product-fingerprints.patch`

The source baseline for the series is repository `main` at `3ff22158a60db3a323a4a64e8b6ab3957de02408`.

## Scope implemented

Step 3 is intentionally limited to callable/body semantic-read dependency capture and checker API hardening.

### Tracked type resolution

`CheckingContext` now exposes a `TrackingTypeResolver` wrapper around the underlying `TypeResolver`.

It records:

- `DeclarationSurface(declaration)` when a query-owned declaration is resolved;
- `LinkedInterface(current_module)` when resolution crosses to another query-owned module;
- `LinkedInterface(current_module)` for a failed name lookup, because adding/changing an import or linked binding can make a previously missing name resolve.

The compatibility `ModuleId::core()` universe seed is excluded because the current session bootstraps it directly and does not publish corresponding staged DB query products.

### Tracked hierarchy reads

`CheckingContext` now exposes a `TrackingTypeHierarchy` wrapper.

It records `HierarchyEdge(declaration)` for:

- direct `superclass()` reads;
- every edge traversed by `is_subclass()`;
- the terminal negative edge when traversal ends without a superclass;
- `supertype_template()` reads.

Assignability and inference helpers are passed the tracked wrapper rather than the raw hierarchy.

### Dispatch dependency capture

Dispatch resolution now records declaration-surface dependencies for every owner whose surface was inspected.

This covers the important negative-lookup case:

```text
Child has no matching method
  -> inspect Child surface
  -> inspect Base surface
  -> resolve Base.method
```

A later method added to `Child` must be capable of invalidating the previously resolved caller even if the old target signature on `Base` remains unchanged.

Known callable signatures continue to record `CallableSignature` dependencies.

Source callables with an unknown/unannotated return currently have no standalone `CallableSignature` DB product in the transitional session. For those, the checker records the declaration surface instead of creating an impossible fail-closed dependency.

### Legacy core seed handling

The compatibility core seed is excluded consistently from:

- declaration-surface dependencies;
- hierarchy-edge dependencies;
- callable-signature dependencies.

Without the callable-signature exclusion, builtin dispatch such as numeric operators could fail closed because the base core signature exists in the bootstrap tables but not as a DB query product.

### Borrowed dispatch mutation

The previous `DerefMut` implementation could panic when a checker using a borrowed workspace dispatch resolver needed to register a local/nested surface.

It has been replaced by copy-on-write behavior:

```text
Borrowed dispatch
  -> read-only fast path: zero clone
  -> first mutation: clone once into Owned
  -> subsequent mutation: mutate Owned
```

The shared workspace dispatch remains immutable.

### Raw checker bypasses removed

The audited body checker paths no longer directly use:

```text
ctx.resolver.resolve_type_name(...)
ctx.declarations.get(...)
ctx.declarations.generic_signature(...)
```

They route through `CheckingContext` helpers or the tracking wrappers.

The following checker paths were updated to use `&ctx.hierarchy` so subtype/assignability/inference reads are tracked:

- expression checking;
- statement checking;
- call argument checking;
- generic inference;
- declaration/body checking.

## Files changed

```text
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/flow/predicate.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/tests/callable_dependency_invalidation.rs
phalcom-semantic/tests/checker_dependency_tracking.rs
```

Diff size at artifact generation:

```text
9 files changed
548 insertions
90 deletions
```

## Regressions added

`checker_dependency_tracking.rs` covers:

1. non-local type resolution records both linked-interface and declaration-surface reads;
2. unresolved names record negative linked-interface reads;
3. hierarchy traversal records every consumed mutable edge, including a missing terminal edge;
4. legacy core resolver/hierarchy/dispatch reads do not create unavailable DB dependencies;
5. declaration metadata reads record the declaration-surface dependency;
6. dispatch records every owner surface inspected before finding a target;
7. borrowed dispatch detaches lazily instead of panicking;
8. checker source files do not bypass tracked resolver/declaration APIs.

`callable_dependency_invalidation.rs` adds:

- **case H:** builtin seed dispatch remains analyzable without unpublished core DB products;
- **case I:** an unannotated callable depends on its declaration surface rather than an absent `CallableSignature` product.

The existing Step-1 constructor regression remains in place and continues to require constructor bodies to depend on the canonical class-side constructor signature.

## Static verification performed

### Diff hygiene

```bash
git diff --check
```

Result: success.

### Raw semantic-read bypass audit

```bash
rg -n -U \
  'ctx\s*\.\s*resolver\s*\.\s*resolve_type_name|ctx\s*\.\s*declarations\s*\.\s*(get|generic_signature)' \
  phalcom-semantic/src/checker
```

Result: no matches.

### Panic-based dispatch mutation audit

```bash
rg -n \
  'DerefMut|Attempted to mutate borrowed|register_surface is only valid for Owned dispatch' \
  phalcom-semantic/src/checker/context.rs
```

Result: no matches.

### Dependency-write audit

No checker source outside `CheckingContext` directly mutates the internal semantic dependency set after this patch.

### Delimiter smoke check

All nine changed Rust files were checked for balanced `{}`, `()`, and `[]` delimiters. Result: balanced.

This is only a syntax smoke check and is not a substitute for Rust compilation.

## Patch round-trip verification

A fresh tree was reconstructed from `phalcom-main.zip`, then Steps 1 and 2 were applied.

Against that Step-2 baseline:

```bash
git apply --check 0003-semantic-read-dependency-capture.patch
patch --dry-run -p1 < 0003-semantic-read-dependency-capture.patch
```

Both succeeded.

The patch was then applied and all nine changed files were compared byte-for-byte with the authored Step-3 tree.

Result:

```text
byte-match: phalcom-semantic/src/checker/body.rs
byte-match: phalcom-semantic/src/checker/call.rs
byte-match: phalcom-semantic/src/checker/context.rs
byte-match: phalcom-semantic/src/checker/declaration.rs
byte-match: phalcom-semantic/src/checker/expression.rs
byte-match: phalcom-semantic/src/checker/flow/predicate.rs
byte-match: phalcom-semantic/src/checker/statement.rs
byte-match: phalcom-semantic/tests/callable_dependency_invalidation.rs
byte-match: phalcom-semantic/tests/checker_dependency_tracking.rs
```

The patch was also dry-run against the raw pre-Step-1 archive and was rejected as expected. This verifies that `0003` is genuinely ordered after the earlier patches.

## Rust verification unavailable in this sandbox

A fresh attempt to run:

```bash
cargo +nightly-2026-07-10 test -p phalcom-semantic --test checker_dependency_tracking -- --nocapture
```

returned:

```text
bash: cargo: command not found
exit status: 127
```

`cargo`, `rustc`, and `rustfmt` are absent from this environment. Therefore this report does **not** claim that the patch compiles or that the Rust regressions pass.

## Required verification with the Phalcom toolchain

Run at minimum:

```bash
cargo +nightly-2026-07-10 test -p phalcom-semantic --test checker_dependency_tracking -- --nocapture
cargo +nightly-2026-07-10 test -p phalcom-semantic --test callable_dependency_invalidation -- --nocapture
cargo +nightly-2026-07-10 test -p phalcom-semantic --test semantic_db_incremental -- --nocapture
cargo +nightly-2026-07-10 test -p phalcom-semantic --test db -- --nocapture
cargo +nightly-2026-07-10 fmt --all --check
cargo +nightly-2026-07-10 clippy -p phalcom-semantic --all-targets -- -D warnings
```

Before merging the series:

```bash
cargo +nightly-2026-07-10 check --workspace
cargo +nightly-2026-07-10 test --workspace
cargo +nightly-2026-07-10 clippy --workspace --all-targets -- -D warnings
```

## Important transitional limitation

The checker can now *capture* declaration-metadata reads as `DeclarationSurface` dependencies, but the current manually published `DeclarationSurface` product does not yet own every declaration-semantic datum consumed from `DeclarationTypeTable` (for example the complete generic declaration metadata).

Therefore Step 3 closes the **read-capture/bypass** problem but cannot by itself guarantee that every declaration-metadata edit changes the dependency product fingerprint.

That is one of the reasons Step 4 must make hierarchy/declaration/signature products authoritative query producers rather than manually published projections from the whole-workspace algorithm.

This limitation is intentional at the Step-3 boundary and should not be worked around with ad-hoc extra invalidation.

## Out of scope by design

Step 3 does not implement:

- staged declaration/hierarchy/signature query ownership (Step 4);
- fine-grained product-stability invalidation (Step 5);
- persistent workspace/module lifecycle ownership;
- LSP lifecycle deletion gates;
- cold-vs-incremental equivalence proof.
