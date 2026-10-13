# PERF003.C2.P5 Handoff — Canonical Publication and Persistent Image Boundary

**Program:** `PERF003`
**Checkpoint:** `PERF003.C2`
**Plan:** `PERF003.C2.P5`
**Status:** `IN_PROGRESS`
**Completion:** `PARTIAL`
**Verification:** `FOCUSED_TESTED`
**Date:** 2026-10-13
**Source revision:** `eeca76f3` (continuation over `e71057d9`)
**Working Directory:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
**Branch:** `perf/universe-characterization`

---

Current continuation is committed as `eeca76f3`. See
`PERF003.C2.P5-baseline-failure-fix-handoff.md` for takeover details and
CHECKPOINT.md for current evidence. The baseline failures below were observed
in the earlier Slice 1 session and are now resolved by focused regressions.
The original Slice 1 implementation history is retained below.

## 1. Executive Summary

`PERF003.C2.P5` makes the **canonical in-memory compiler world explicit, consumer-aware, immutable, compositional, and serialization-ready** before persisting `universe.telosc` in `PERF003.C2.P7`.

During this session:
1. **Verification & Canonical Provider Isolation Fix:**
   - Diagnosed and repaired a test failure in [`canonical_formal_image_source_parity_and_provider_isolation`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/tests/semantic/integration/canonical_image.rs#L66): [`bootstrap_universe_declarations`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/declarations.rs#L488) had previously hardcoded `UniverseSourceProvider::new()`, ignoring test provider overrides.
   - Introduced [`bootstrap_universe_declarations_with_provider`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/declarations.rs#L492) and wired it into [`CanonicalUniverseSemanticBaseline::build_with_provider_inner`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/canonical_baseline.rs#L132).
   - All 4 tests in [`tests/semantic/integration/canonical_image.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/tests/semantic/integration/canonical_image.rs) now pass cleanly (135.74s).
   - All 10 unit tests in [`phalcom-semantic/src/canonical_image.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/canonical_image.rs) pass cleanly (39.27s).

2. **P5.T5 / P5.T6 Slice 1 (Parent-Aware Table Architecture - Implemented):**
   - Converted [`DeclarationTypeTable`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/declarations.rs#L180) to support an immutable canonical parent overlay:
     - `parent: Option<Arc<DeclarationTypeTable>>`
     - [`DeclarationTypeTable::with_parent(parent: Arc<Self>)`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/declarations.rs#L192)
     - `local_len()`, `len()`, `is_empty()`, `parent()`
     - Delegation lookups (`get`, `contains`, `form`, `class_object_type`, `kind`, `generic_signature`, `generic_binder_header`, `supertype_template`, `intrinsic_runtime_owner`, `intrinsic_type_owner`, `intrinsic_static_domain_kind`, etc.)
     - Non-recursive parent-chain iterator [`iter()`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/declarations.rs#L436) and local iterator [`iter_local()`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/declarations.rs#L431)
     - Local-only mutation safety: `remove()` removes from local maps only; `store_authoritative()` asserts `!parent.contains(&info.declaration)` to prevent local modification of canonical declarations.
   - Converted `CallableSignatureTable` and `FieldSignatureTable` to immutable parent overlays with delegated lookups, local-only mutation, and parent-aware iteration.
   - Converted `MapTypeHierarchy` to an immutable parent overlay with delegated superclass/template lookup and local-only mutation.
   - Wired all four overlays into image-backed `SemanticWorkspaceSession` creation and retained the canonical image. Previous-snapshot declaration iteration now visits local entries only, and field signatures seed from the canonical parent.

---

## 2. Plan Progress Census

| Task | Description | Status |
|---|---|---|
| **P5.T0** | Entry gate, lifecycle normalization, measurement census | **COMPLETED** (in commit `84227536`) |
| **P5.T1** | Explicit canonical compiler-image product contract ([`SemanticProductRequest::canonical_compiler_image()`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/request.rs#L55)) | **COMPLETED** (in commit `84227536`) |
| **P5.T2** | Product-family classification & request-model cleanup | **COMPLETED** |
| **P5.T3** | Canonical bootstrap cutover to explicit request ([`canonical_image.rs`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/canonical_image.rs#L425)) | **COMPLETED** (in commit `84227536`) |
| **P5.T4** | Consumer-specific publication semantics ([`SourceSemanticLevel`](file:///Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/request.rs#L9)) | **COMPLETED** (in commit `84227536`) |
| **P5.T5 / P5.T6 Slice 1** | Parent-aware core tables (`DeclarationTypeTable`, `CallableSignatureTable`, `FieldSignatureTable`, `MapTypeHierarchy`) | **IMPLEMENTED** in `aaac8106`; focused unit and canonical-image verification passes; formerly blocked workspace/recomputation filters now pass |
| **P5.T6 Slices 2–3** | Dispatch/surface, data/enum/associated/enum-requirement/type-member tables | **IMPLEMENTED / FOCUSED_TESTED** in `e71057d9` |
| **P5.T6 Slice 4** | Alias/static/accepted-callable parent maps established; static-case/associated/module binding maps remain | **PARTIAL / FOCUSED_TESTED** |
| **P5.T7** | Snapshot/read API hardening for parented canonical aggregates | **PENDING** |
| **P5.T8** | Formal canonical persistence inventory and capability descriptor | **PENDING** |
| **P5.T9** | Exhaustive query-family persistence classification used by freeze | **IMPLEMENTED / FOCUSED_TESTED** |
| **P5.T10** | Bind P4 canonical metadata authority into compiler-image boundary | **BLOCKED ON P4 PR-B** |
| **P5.T11** | Observability & cardinality verification (`CanonicalAggregateEntriesCopied == 0`) | **PENDING** |
| **P5.T12** | Semantic/tooling/compiler parity & performance verification | **PENDING** |
| **P5.T13** | Lifecycle closure & handoff to P6/P7 | **PENDING** |

---

## 3. Historical Slice 1 Evidence (superseded where noted)

- `cargo check -p phalcom-semantic`: **PASS** (clean, 0 warnings, 0 errors).
- `RUSTFLAGS='' cargo check -p phalcom-semantic`: **PASS** (clean, 0 warnings, 0 errors) after Slice 1 wiring.
- `RUSTFLAGS='' cargo test -p phalcom-semantic --lib signature`: **PASS** (7 passed).
- `RUSTFLAGS='' cargo test -p phalcom-semantic --lib types::relation`: **PASS** (13 passed).
- `cargo test -p phalcom-semantic --lib canonical_image`: **PASS** (10 passed, 0 failed, 39.27s).
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::canonical_image`: **PASS** (4 passed, 0 failed, 131.79s):
  - `canonical_formal_image_is_send_and_sync`: ok
  - `canonical_formal_image_is_complete_and_process_shared`: ok
  - `ordinary_image_session_keeps_universe_external_across_replacement`: ok
  - `canonical_formal_image_source_parity_and_provider_isolation`: ok
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::workspace`: **BASELINE-BLOCKED** (14 passed, 3 failed); the same three failures reproduce on clean `84227536`.
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::recomputation_metrics`: **BASELINE-BLOCKED** (9 passed, 4 failed); failures are existing alias relowering/readiness/parity cases outside this overlay change.
- `cargo fmt --all -- --check`: **BASELINE-BLOCKED** by pre-existing formatting drift in unrelated files; `git diff --check` is clean.

---

## 4. Immediate Next Steps for Next Agent

### Next: Continue residual Slice 4 and T7–T8

Dispatch/data/enum/associated/type-member overlays are established. Alias
forms, static definitions/constants and accepted callable definitions are
parent-shared at `eeca76f3`; query freeze consumes the extracted exhaustive
family classification. Continue remaining static-case/associated static/module
binding copies, then snapshot/read hardening and persistence inventory. P4
metadata composition still gates T10 and final P5 acceptance. Do not redo
established overlay work.

---

## 5. Architectural Invariants to Maintain

- **INV-P5-05 (No Copied Canonical Aggregates):** Ordinary sessions must own `Arc<CanonicalUniverseFormalImage> + local mutable deltas`, not cloned Universe maps.
- **INV-P5-06 (Parent Immutability):** Parent tables are strictly immutable. Local remove/mutation must never touch or clone parent tables.
- **Deterministic Iteration:** Iterators must deterministically visit parent entries followed by local entries without duplicate entries.
- **Zero-cost Overlay Creation:** Cloning an empty or small local overlay is $O(\text{local})$, never $O(\text{Universe})$.
