# PERF003.C2.P7 continuation walkthrough

Date: 2026-10-14. Status IN_PROGRESS, completion PARTIAL, verification UNVERIFIED.
Starting HEAD 3ca30fca. Inherited T1/T2 edits and documentation moves preserved.

Implemented the P7.T3 source-free external module boundary across modules,
semantic image sessions, and linker/compiler consumers. Preserved source-built
tooling as an optional capability. Corrected artifact integrity to BLAKE3-256,
stable identity compatibility, and fail-closed section encoding/capability checks.
Published the accepted telosc container and architecture specification and
registered it in the normative specification index.

Compiler check: RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib
passed before the final presentation-capability adjustment. Final consumer check passed: RUSTFLAGS='' RUSTC_WRAPPER='' cargo check
-p phalcom-core -p phalcom-lsp --lib (10.41 seconds). The LSP reports one
unused TypeHierarchy import in unchanged backend.rs; it was left untouched. Tests and broader verification deliberately
omitted on the user's explicit instruction. No source-free artifact acceptance
or P7/P8 completion claimed. Next implementation unit is P7.T4.


## T4/T5 continuation

Added explicit portable module-world DTOs and source-free module-owner adapters.
They retain compiled interfaces, read/binding layouts, graphs, inert metadata,
topology, and ABI fingerprints. Bounded decoding precedes materialization;
runtime DAG validation is iterative to avoid recursive graph walks on input.

Added explicit portable type/kind/scoped/lambda/row/family/parameter/domain/
constructor/variant DTOs and exhaustive source-owned projection. Fresh-store
relocation reconstructs ordinary interners, preserves historical parameter
versions, restores current slots, and checks scoped binder addressability.
The import path does not invoke semantic analysis or source construction.

Compiler feedback resolved mechanical import/type-annotation issues. Source
inspection established that scoped unions preserve repeated/source-ordered
contributions, unlike normalized global unions; import now preserves that fact.

Final cross-layer compiler check: RUSTFLAGS='' RUSTC_WRAPPER='' cargo check
-p phalcom-core -p phalcom-lsp --lib passed in 8.31 seconds. No tests, runtime
experiments, parity checks, determinism checks, or broad verification performed.
Completion remains PARTIAL and verification remains UNVERIFIED. T6 is next.

Final compiler-only check after codec error classification: flags cleared,
cargo check -p phalcom-core -p phalcom-lsp --lib passed in 5.30 seconds.
Only the unchanged LSP unused TypeHierarchy import warning remains.


## Requested review corrections — 2026-10-14

Restored Tuple/Record/Pack semantic descriptors and their compiler/VM owner-cache
materialization handlers. Restored residual-store ownership on static constant
products in both publication paths. Added checked parameter-kind rejection in
TypeStore artifact relocation before interning.

Added regression coverage for nested structural descriptors, VM materialization
and owner reuse, symbolic product ownership after evaluator teardown, and artifact
parameter acceptance/rejection. Replaced the stale compiler test expecting Tuple
runtime use to fail with an assertion that the read uses its owner's value.

The library consumer check, semantic/core integration-test compilation, and core
test-target compilation passed with RUSTFLAGS and RUSTC_WRAPPER cleared. No tests
were executed under the inherited compiler-only policy. Existing unused
TypeHierarchy import warnings remain. Corrections: IMPLEMENTED, behavioral
verification UNVERIFIED. P7 overall remains PARTIAL / UNVERIFIED. No consultation,
plan amendment, commit, or push.
