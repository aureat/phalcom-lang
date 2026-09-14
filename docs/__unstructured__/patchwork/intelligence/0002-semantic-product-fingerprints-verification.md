# Step 2 Verification — Semantic Product Fingerprints

## Patch

`0002-semantic-product-fingerprints.patch`

This patch is **incremental on Step 1**. Apply `0001-semantic-query-validity-fail-closed-dependencies.patch` first.

The uploaded/current GitHub baseline inspected for this series remains:

```text
3ff22158a60db3a323a4a64e8b6ab3957de02408
```

Step 2 was authored against that tree with Step 1 applied.

## Goal

Make query fingerprints obey the architectural distinction:

```text
InputFingerprint
    = must this query refresh the stored product?

ProductFingerprint
    = did downstream-observable semantic meaning change?
```

This patch deliberately keeps source/provenance movement in structural input fingerprints for products that expose those locations, while excluding incidental movement from semantic product fingerprints used by dependency edges.

## Files changed

```text
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/session.rs
phalcom-semantic/tests/semantic_fingerprints.rs
```

No LSP, module-lifecycle, invalidation-policy, or dependency-audit files are changed in this step.

## What changed

### 1. Unlinked module interface hashing

`unlinked_interface_product_fingerprint()` now hashes the semantic interface instead of `Debug` representations:

- module identity;
- module kind;
- local declaration names and constness;
- import kind/path/alias/selective/re-export items;
- export names and targets;
- package exposed children;
- module metadata.

It excludes source ranges from the semantic product hash.

`unlinked_interface_input_fingerprint()` hashes the same structure plus source ranges. This means source movement refreshes the stored interface/provenance while a semantically unchanged interface can retain the same product fingerprint for dependents.

### 2. Linked interface hashing

`linked_interface_product_fingerprint()` now includes:

- module identity/kind;
- canonical linked exports and targets;
- module metadata.

`linked_interface_input_fingerprint()` additionally includes export/metadata ranges.

The current `LinkedModuleInterface` Rust product does **not** contain linked import/read bindings; those live in `LinkedModule` and are therefore covered by `SemanticComponent`, not fabricated into this product fingerprint. The later staged-query ownership work must decide whether the product model itself should be widened.

### 3. Declaration-surface hashing

`declaration_surface_product_fingerprint()` now hashes both dispatch sides with:

- field name/type knowledge and evidence authority;
- selector;
- callable parameter label/name/rest/type;
- return type;
- generic signature.

Type-evidence provenance ranges/descriptions are excluded from the semantic product fingerprint.

`declaration_surface_input_fingerprint()` includes that provenance so publication refreshes stored source-facing data when it moves.

The current `DeclarationSurface` Rust value does not itself contain declaration kind, direct superclass template, or the complete declaration generic record. Those are owned by other current semantic structures. Step 2 hashes the actual product rather than inventing duplicate state; staged query ownership is the place to reconcile that product boundary with the completion specification.

### 4. Callable signature hashing

`callable_signature_product_fingerprint()` now includes the full fields carried by `CallableSemanticSignature` that can affect consumers:

- callable/owner/side/selector;
- generic signature;
- parameters including index, label, rest lane, and type term;
- return type term;
- implementation kind;
- native surface ID;
- effects;
- raises;
- return-flow metadata;
- lifecycle metadata.

Source spans are excluded from product identity and included by `callable_signature_input_fingerprint()`.

### 5. Callable-analysis hashing

`callable_body_product_fingerprint()` no longer hashes binding IDs only. It now covers the current `CallableAnalysis` product:

- callable/body range;
- expression identity/range/type knowledge/denotation/status/explanation/call identity;
- binding identity/name/range/declared type/current knowledge/mutability/version/explanation;
- flow graph nodes, edges, predicates, entry and exits;
- entry/exit flow summaries;
- complete semantic diagnostics;
- reachable explanation DAG content;
- callable dependencies;
- semantic dependencies;
- callable analysis status.

`dependency_fingerprint` is intentionally excluded because it is assigned from the callable-body product fingerprint after computation; including it would make the definition recursive.

### 6. Diagnostic hashing

Module/body diagnostic hashing now includes the complete current diagnostic representation rather than only code/severity/primary range:

- code/severity/message;
- source-owned primary span;
- labels;
- notes;
- helps;
- explanation references;
- fixes/replacements;
- root cause.

### 7. Semantic-component product identity

`query_semantic_component()` no longer publishes:

```rust
ProductFingerprint::new(input_fingerprint.raw())
```

as the linked result identity.

`semantic_component_product_fingerprint()` now hashes the successful `LinkedProgram` result:

- semantic project topology;
- entry module;
- linked modules/interfaces;
- local/import binding layouts;
- linked reads;
- runtime dependencies;
- reference graph topology/kinds;
- semantic graph topology/kinds;
- runtime graph topology/reasons;
- initialization order.

Graph/source ranges are excluded from this semantic product identity.

`semantic_component_input_fingerprint()` includes the complete structural inputs needed to refresh the linked product:

- entry;
- project topology;
- project physical/source/manifest state observable on the returned universe;
- full unlinked interfaces including ranges;
- exact resolved-import target map.

This fixes both the old under-hash (the resolved map was absent from direct input identity) and the incorrect product=input aliasing.

### 8. Wrapper-query and transitional-session input identity

The following now use dedicated structural input fingerprints instead of wrapping a semantic product fingerprint as an `InputFingerprint`:

```text
ResolvedImports          <- UnlinkedInterface structural input
SemanticComponent        <- full linking structural input
LinkedInterface          <- linked structural input
DeclarationSurface       <- provenance-sensitive surface input
CallableSignature        <- source-sensitive signature input
```

The same distinction is applied to the current manual publication paths in `SemanticWorkspaceSession`. Those paths are transitional and are scheduled for replacement by actual staged-query ownership, but they must remain correct until deleted.

## Regression coverage added

`phalcom-semantic/tests/semantic_fingerprints.rs` contains 16 focused tests covering:

1. unlinked local declaration changes;
2. unlinked metadata changes;
3. unlinked range-only movement: product stable, input changed;
4. linked range-only movement: product stable, input changed;
5. linked metadata changes;
6. declaration provenance movement: product stable, input changed;
7. declaration callable generic contract changes;
8. callable generics/effects/lifecycle changes;
9. callable source movement: product stable, input changed;
10. callable binding-state changes;
11. expression denotation/status changes;
12. flow/exit/callable-status changes;
13. referenced explanation-content changes;
14. callable-body diagnostic-detail changes;
15. module diagnostic-detail changes;
16. semantic-component linked-target changes.

## Static verification performed in this sandbox

### Current repository baseline

A fresh GitHub commit query showed `main` still points at:

```text
3ff22158a60db3a323a4a64e8b6ab3957de02408
```

There were no newer commits to rebase onto during Step 2.

### Scope check

A byte comparison of the Step-1 baseline and Step-2 working tree found exactly four changed files:

```text
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/session.rs
phalcom-semantic/tests/semantic_fingerprints.rs
```

### Diff hygiene

Run:

```bash
git diff --check
```

Result:

```text
git diff --check: OK
```

### Delimiter/syntax-structure sanity check

A source scanner that ignores comments/string literals checked balanced `()`, `[]`, and `{}` in all four changed Rust files.

Result:

```text
fingerprint.rs: balanced
query.rs: balanced
session.rs: balanced
semantic_fingerprints.rs: balanced
```

This is **not** a substitute for `rustc`.

### Public API documentation check

Every newly introduced public fingerprint function has an immediately associated Rustdoc comment.

Checked functions:

```text
parsed_module_input_fingerprint
unlinked_interface_input_fingerprint
unlinked_interface_product_fingerprint
linked_interface_input_fingerprint
linked_interface_product_fingerprint
declaration_surface_input_fingerprint
declaration_surface_product_fingerprint
callable_signature_input_fingerprint
callable_signature_product_fingerprint
hierarchy_edge_product_fingerprint
callable_body_input_fingerprint
callable_body_product_fingerprint
resolved_imports_product_fingerprint
module_diagnostics_product_fingerprint
semantic_component_input_fingerprint
semantic_component_product_fingerprint
```

### Debug-string hashing check

Search of `phalcom-semantic/src/db/fingerprint.rs` found one remaining `format!("{...:?}")` fingerprint use:

```rust
format!("{statement:?}")
```

It is confined to `callable_body_input_fingerprint()`, where the fingerprint is deliberately exact/syntax-sensitive input identity. No semantic **product** fingerprint uses `Debug` representation.

A comment in the implementation records that a parser-owned syntax fingerprint can replace this later without altering dependency semantics.

### Product=input alias check

The only remaining:

```rust
ProductFingerprint::new(input_fingerprint.raw())
```

is `ParsedModule`, whose product identity is intentionally the exact parsed-module source input in the current model.

`SemanticComponent` no longer aliases its product fingerprint to its input fingerprint.

### Product field-coverage audit

A static field audit confirmed explicit hashing paths for the current product fields in:

```text
UnlinkedModuleInterface
LinkedModuleInterface
CallableSemanticSignature
CallableAnalysis
SemanticDiagnostic
LinkedProgram
```

The audit also checked the full linked-program binding/read/graph/init-order paths.

### Patch round trip

Against the Step-1 baseline:

```bash
git apply --check 0002-semantic-product-fingerprints.patch
patch --dry-run -p1 < 0002-semantic-product-fingerprints.patch
```

Both succeeded.

The patch was then applied to a fresh copy of the Step-1 baseline and all four changed files were compared byte-for-byte with the authored working tree.

Result:

```text
byte-match: phalcom-semantic/src/db/fingerprint.rs
byte-match: phalcom-semantic/src/db/query.rs
byte-match: phalcom-semantic/src/session.rs
byte-match: phalcom-semantic/tests/semantic_fingerprints.rs
```

The patch was also checked against the raw pre-Step-1 tree and correctly **failed** at the Step-1-modified query context. This confirms that `0002` is an ordered patch and must follow `0001`.

## Rust verification unavailable in this sandbox

A fresh attempt to run:

```bash
cargo test -p phalcom-semantic --test semantic_fingerprints -- --nocapture
```

returned:

```text
bash: cargo: command not found
exit status: 127
```

`cargo`, `rustc`, `rustup`, and `rustfmt` are absent from this environment, and earlier network provisioning attempts could not resolve the Rust distribution host. Therefore this report does **not** claim that the patch compiles or that the Rust tests pass.

## Required verification with the Phalcom toolchain

Use the repository-pinned toolchain (`nightly-2026-07-10`) and run at minimum:

```bash
cargo +nightly-2026-07-10 test -p phalcom-semantic --test semantic_fingerprints -- --nocapture
cargo +nightly-2026-07-10 test -p phalcom-semantic --test semantic_db_incremental -- --nocapture
cargo +nightly-2026-07-10 test -p phalcom-semantic --test db -- --nocapture
cargo +nightly-2026-07-10 fmt --check
cargo +nightly-2026-07-10 clippy -p phalcom-semantic --all-targets -- -D warnings
```

Before merging this series, also run the workspace gates required by the project:

```bash
cargo +nightly-2026-07-10 check --workspace
cargo +nightly-2026-07-10 test --workspace
cargo +nightly-2026-07-10 clippy --workspace --all-targets -- -D warnings
```

## Discovered follow-on correctness issue: persistent generic parameter interning

While auditing generic-signature fingerprinting, Step 2 exposed a separate persistent-store issue in:

```text
phalcom-semantic/src/types/store.rs
TypeStore::intern_type_parameter
```

The current key is `(TypeParameterOwner, index)`. If the same owner/index is interned again in a later revision with a changed binder name, kind, or source, the existing `TypeParameterId` is returned without replacing/allocating updated `TypeParameterData`.

This is not only a fingerprint concern. A changed generic binder kind can therefore retain stale type-parameter data; `parameter_form()` can also encounter an existing `TypeData::Parameter(id)` whose stored kind reflects an older revision.

This patch intentionally does **not** redesign TypeStore interning. The safer likely direction is append-only parameter identities for changed binder metadata while updating the current `(owner,index)` lookup, preserving old snapshot denotations. That needs its own focused tests and ownership review rather than being smuggled into fingerprint work.

Until that is fixed, the new generic-signature fingerprints accurately hash the current `GenericSignature` representation, but the representation itself can fail to change for some incremental binder-metadata edits.

## Remaining architectural work

Step 2 does not attempt to solve:

- full semantic-read/dependency capture audit (Step 3);
- conversion of hierarchy/declaration/signature products into authoritative staged query producers (Step 4);
- fine-grained product-stability invalidation (Step 5);
- overlay/workspace lifecycle inversion;
- LSP deletion gates;
- cold-vs-incremental equivalence.

Those remain separate patch boundaries by design.
