# LANG005.C1.P1 — Diagnosis, fixes, and implementer guidance

Date: 2026-09-13. Base HEAD: `7b5046e5ab5db195128788cf938d7f10ed3d4792`.
Status: STOPPED at the user's explicit request. Resume from this handoff; do not restart exploration.

Scope: concrete defect repair and architectural guidance. Checkpoint implementation and release certification remain with the implementers. Existing unrelated checkout work is preserved; nothing committed or pushed.

## Decision: CONSULT FURTHER before declaring P1 complete

The original handoff percentages overstate the evidence. Six source-to-runtime tests passed initially, but omitted exact generic identity, hash consistency, source-order evaluation, and arbitrary-precision integers. C4 storage edits were started before the user clarified this task's advisory/repair role; those edits remain in the checkout and have focused validation. They are not a declaration of checkpoint completion.

## Fixed defects

| Defect | Correction | Evidence |
| --- | --- | --- |
| `Int` annotations selected `Int64`, rejecting legal heap-backed large integers. | Semantic projection uses `Value` for unrestricted `Int`. `Int64` remains available only to a future producer with an actual range proof. | Data and enum regressions using `9223372036854775808`. |
| Direct data construction used declaration-only descriptors, collapsing distinct applications. | Construction carries canonical exported semantic metadata; the VM loads/interns it through the existing typing registry and registers exact applied descriptors. Monomorphic direct/first-class construction retains the declaration descriptor identity. | `Box(1)` and `Box(1.0)` differ; repeated `Box(1)` agrees; both applications share one behavior class. |
| Record fields evaluated in declaration order, changing side effects. | Evaluate expressions in source order and apply the executable argument-to-component mapping at construction. Missing components no longer synthesize `Nil`. | A counter evaluates `second` before `first`, while both properties retain the correct values. |
| Data hashing used raw storage/handles, including nested data and signed-zero float bits. | Decode logical components and combine their ordinary hash results. | Equal nested data with `+0.0`/`-0.0` have equal hashes. |
| Construction could publish missing/partially initialized components. | `ProductStorage::from_values` validates arity and encodes before heap publication; data construction checks arity even for nullary values. | Existing data/runtime suite; source constructor regressions. |
| General enum constructor routes allocated payloads directly, bypassing NativeOption handling. | Route static and dynamic family construction through `construct_variant_value`. General cases consume shared product storage; NativeOption retains its immediate branch. | ADT suite and native Option through a first-class constructor. |
| Dynamic associated-family packs discarded all arguments and selected the first candidate. | Decode the pack, validate its actual operation shape, and use the existing dynamic invocation router. | `Pair` receives both spread arguments; `Option<Int>::Some` remains immediate. |

Data behavior classes are marked `native_repr`; data instances continue to use `DataObject` or immediate singleton values. The low-level data constructor no longer clones the layout per value.

## Instructions to implementers, in priority order

1. **Close exact-type propagation for every constructor route.** Direct construction now has a `DataConstructionLoweringSpec`, but `ExecutableFamilyTarget::DataConstructor`, `MakeDataConstructorThunk`, and dynamic family invocation still retain declaration-only constructor identity. Carry specialization metadata through those capabilities instead of rediscovering it from runtime argument classes. Test phantom applications and equality between direct and first-class construction. Never allocate an unspecialized generic descriptor as a concrete value.
2. **Handle runtime generic environments explicitly.** Exporting a `TypeNode::Parameter` is not substituting a concrete runtime type. Audit construction inside generic functions. Use the existing runtime typing environment to instantiate it, or report an explicit unsupported boundary; do not silently merge specializations. The existing metadata exporter also erases `ExactCase` to its enum type: audit exact-case type arguments before claiming exact data identity for every type form.
3. **Move normalization fully into canonical products.** The record compiler currently derives source-entry mapping from logical component names supplied by semantic lowering. Publish that mapping at the semantic call-resolution boundary so imported records, spreads, and mixed labels use the same authority. Keep source evaluation order independent of storage order.
4. **Correct the plan's native-slot rule.** Nominal `Int` includes arbitrary-precision heap values (`value/mod.rs::from_bigint`, `Value::class`, and integer literal lowering). The P1 instruction “proven Int → Int64” needs a range-proof qualification. Retain `Value` until that proof exists; do not narrow the language's integer semantics to meet a compactness target.
5. **Finish descriptor binding/caching and reflection.** Reuse exact construction descriptors and layouts before entering repeated execution. The repair currently scans metadata pools and builds/interns the projected layout at construction. Bind once per executable site/VM, retain semantic identity across shared layouts, and integrate nominal metadata with ordinary declaration-class/reification bindings. Do not confuse zero arena-object allocations for singletons with zero host allocations.
6. **Strengthen incremental and tooling evidence.** `data_declaration_cold_incremental_equivalence` currently compares component counts on an unchanged initial input. Add actual label/type/generic edits, cold/edit semantic-product and diagnostics equality, unaffected-product retention, imported-data construction, and source-index/LSP navigation. Run the named P1 gates only after these contracts are covered.
7. **Qualify representation compatibility.** Retain mixed native/Value GC tests, native Option, nullary enum constructor freshness, case behavior, and GADT matching. Existing ignored ADT tests remain unverified. Do not mark C4/C5 complete from the focused passing filter alone.

## Owning seams

- `phalcom-core/src/modules/semantic_lowering.rs`: physical slot projection and concrete construction metadata.
- `phalcom-core/src/compiler/lib/expr.rs`, `associated.rs`: construction emission and source-order evaluation.
- `phalcom-core/src/vm/data.rs`, `data.rs`: exact descriptor construction and identity.
- `phalcom-core/src/primitive/object.rs::object_hash`: logical component hashing.
- `phalcom-core/src/vm/adt.rs`, `dispatch.rs`, `send.rs`: shared constructor boundary and argument-pack routing.
- `phalcom-core/src/product/storage.rs`, `heap/adt.rs`, `heap/trace.rs`: fully encoded storage and precise tracing.

## Evidence ledger

All Cargo commands use the pinned toolchain with `RUSTFLAGS='' RUSTC_WRAPPER=''`.

- Initial `cargo test -p phalcom-core --test core language::data_e2e`: 6 passed.
- Repaired `cargo test -p phalcom-core --test core language::data_`: 15 passed, zero ignored. A later rerun was launched after the final small changes; its completion was not inspected before the user requested an immediate stop.
- `cargo test -p phalcom-core --test core language::algebraic_data`: 42 passed, 19 pre-existing ignored. Includes mixed Float/Bool/reference storage, layout sharing without case-identity sharing, GC retention, and large integers.
- `cargo test -p phalcom-core --test core associated_dynamic_pack_preserves_arguments_and_native_option`: 1 passed.
- `cargo check -p phalcom-core --all-targets`: passed.
- Scoped `git diff --check`: passed. Modified Rust files formatted with child traversal disabled.
- Native ADT target: not run; `cargo check --all-targets` compiled it, but did not execute its tests.
- Workspace build/test/Clippy and complete C0–C5 certification: not run by this focused repair task.

No checkpoint is newly marked COMPLETE by this document.

## Immediate resumption checklist

1. Read this document and the scoped diff. Preserve all staged, unstaged, and untracked work: this was a heavily dirty shared checkout before the session started. No commits, pushes, resets, or worktree migration were performed.
2. Inspect `/tmp/lang005-data-verified.log` for the final data rerun. It was launched as tool session `42634`; its result is **unconfirmed**, not a reported pass. Do not launch another Cargo process until any existing run has finished.
3. Execute `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test native_adt_runtime`. The target's fixture literals were updated for the layout field and compile, but runtime execution remains outstanding.
4. Review the concrete changes, especially exact descriptor propagation, pack routing, and conservative Int slots. Then implement the prioritized remaining contracts above. Do not undertake another blanket migration or claim release completion on the basis of this repair task.
5. Run focused regressions for each new change. The full C5 gates belong to the implementers after the remaining P1 contracts are closed.

## Additional identified boundaries requiring explicit qualification

- **Monomorphic/direct versus first-class construction:** the repair preserves the declaration descriptor for nominal metadata through `RuntimeDataRegistry::bind_nominal_type`. Add an end-to-end equivalence regression; the existing generic direct-construction regression does not exercise first-class data capabilities.
- **Phantom/nullary generic data:** runtime unit tests manufacture distinct descriptor IDs, which does not prove source `Signal<Int>()` versus `Signal<String>()`. Add source-level explicit-type-argument tests, repeated construction, `.class`, and allocation assertions. This session did not establish that these source routes work.
- **Cross-module record construction:** the old compiler looked only in local `data_decls`; construction now carries component names and mapping. Imported-record ordering still needs a dedicated regression. Move semantic mapping ownership to an explicit product when completing the larger lowering work; do not reintroduce source-order fallback or missing-field `Nil`.
- **Unknown/open type states:** `DataConstructionLoweringSpec` now transports the existing exported metadata bundle, but successful export alone does not prove a concrete materializable application. Validate type parameters, open rows, self types, and exact-case arguments at the appropriate owning layer. No runtime-generic-environment solution was implemented here.
- **Exact reflection:** direct construction retains a `RuntimeTypeRef`, but this session did not establish the full nominal-binding/reification/public-reflection path. One shared behavior class and exact applied type are separate identities.
- **Construction invariants:** add hostile low-level arity/mapping tests, including nullary constructors receiving arguments. `from_values` validates before publishing; the public low-level `new`/store/load APIs still rely on callers supplying a matching valid layout. Do not treat their existence as proof of hostile-input robustness.
- **Allocation/representation claims:** only focused arena/GC/value-size evidence was run. Metadata-pool scans, layout interning, argument vectors, and temporary mapping vectors remain on the direct construction path. No allocator benchmark, object-arena-size regression, or full first-class zero-`InstanceObject` qualification was completed.
- **Equality coverage:** nested data hashing and signed-zero consistency have regression support. This is not exhaustive qualification of arbitrary component categories, user-defined equality/hash protocols, options containing data, or recursive container semantics.
- **Tooling/documentation:** no source-index/LSP smoke run, canonical syntax-document synchronization, PDR STATUS delivery update, or complete negative-gate audit was performed. Keep ratified semantics separate from implementation status.
- **Ignored ADT cases:** 19 existing tests in the affected filter were ignored for pre-existing RED/GATED reasons. They were neither enabled nor repaired. Their source annotations are the next evidence boundary, not passing coverage.

## Exact validation artifacts retained

- `/tmp/lang005-data-e2e.log`: original six-test baseline, passed.
- `/tmp/lang005-data-final.log`: fifteen focused data tests, passed.
- `/tmp/lang005-data-verified.log`: last rerun, completion uninspected at stop.
- `/tmp/lang005-adt-final.log`: 42 passed, 19 ignored.
- `/tmp/lang005-pack2.log`: corrected dynamic-pack/NativeOption regression, passed.
- `/tmp/lang005-check.log`: core all-targets check, passed before the final removal of an unnecessary layout clone.

Earlier temporary logs include invalid test-fixture attempts that were corrected (`let mut`, bare closure syntax, and an underconstrained Option constructor reference). They are not current product failures. Avoid printing entire semantic-error debug snapshots: one failed fixture emitted a very large snapshot; inspect bounded excerpts instead.

## Delivery boundary

The source changes and this handoff remain uncommitted. Main C0–C5 implementation, architectural closure, and release gates belong to the implementer agents. This task's owner requested diagnosis, concrete fixes, and instructions, then explicitly ordered an immediate handoff and no further work. No additional work was performed after writing this stop record.
