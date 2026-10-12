# Telos Type-System and Compiler Semantics Audit — Slice 03 Handoff

**Assessment date:** 2026-10-12  
**Repository:** `aureat/phalcom-lang` / `main`  
**Current remote revision verified:** `608c8c6cd3346556f3bbc98f0713a6df6e0a72ac` — GitHub comparison of that SHA with `main` returned `identical` (ahead 0, behind 0).  
**Original reference revision:** `5942666901b387519c5cd502bed2611cf97b53c4`  
**Authorization:** **Read-only audit.** No source/test changes, snapshot acceptance, commits, or pushes. Deliverables are Markdown files in the assistant artifact directory.  
**Execution environment:** Remote GitHub source inspection via linked repository. No executable local repository checkout was mounted. No new Cargo commands or Rust probes run in this slice; previous baseline tests are not re-certified.

## 1. Scope and completed investigation

This bounded slice followed `SLICE-02-type-system-audit-handoff.md` by investigating:

1. Canonical fixed-entry / lane projection behavior in `phalcom-semantic/src/types/store.rs`, the intrinsic projection normalizer, and type-level pack representation.
2. `SelfType`, direct and nested Dynamic, and checked versus unchecked `TypeView` specialization contracts.
3. Snapshot fallback and publication status: `SemanticWorkspaceSession::{update,update_with_delta,update_with_budget_and_cancel}`, one-shot `analyze_workspace`, snapshot completeness, compiler entry points, and production Result-returning paths.
4. Brief revisit of subtype union aggregation outcomes; full bounded tests remain pending.

## 2. Reports authored and immediate disposition

| ID | Report | Status | Severity | Evidence |
|---|---|---|---|---|
| INC-006 | `INC-006-open-product-lane-projections-erase-remainders.md` | **CLEARED / false positive** | None | Authoritative `docs/specs/structural-products.md` §4 explicitly models `OrderedShape(entries, positionalTail, labeledTail)`. `entries` is a **finite** Pack and anonymous tails are not finite collections. Fixed-entry projection should not be required to encode open tails. This discovery **withdraws** the initial INC-006 hypothesis. |
| INC-007 | `INC-007-semantic-update-failure-disguised-as-complete-snapshot.md` | **New; source-confirmed conditional defect** | P1; potential P0 not demonstrated | `session.rs:927–960` drops any failed query outcome; new synthetic snapshot uses empty diagnostics and `SnapshotStatus::Complete`; old snapshot may be returned without a failure/stale flag. `workspace.rs:123–127` and `core/modules/compile.rs:618,726` establish production consumer path. |

**Important:** INC-006 was initially reported before reading §4. It has been rewritten as a **closure** document, not left as a defect. Do not count it among unresolved incidents or recommend altering `TypePack` to represent anonymous open tails based on that hypothesis. Any future defect must show a specific consumer mistakenly interpreting fixed entries as a closed whole shape.

## 3. Cumulative register

| ID | Finding | State / priority | Remaining essential verification |
|---|---|---|---|
| INC-001 | Workspace baseline: one record-pattern parser failure and six stale AST snapshots | Previous P2 baseline blocked | Baseline occurred with floating nightly override; pinned `nightly-2026-07-10` must be recorded precisely and rerun when authorized. |
| INC-002 | Labeled Tuple/Pack relation checks labels by lookup, not ordered position | P1 source/spec confirmed | Executable API probe and source-level assignability. |
| INC-003 | Target-union subtype relation exits `Blocked` before later `Proven` | P1 source/control-flow confirmed | Blocked-first/permutation probe. Also inspect **source-union conjunction** to ensure an early Blocked does not suppress a later Refuted (same aggregation family; avoid duplicate incident without distinct root cause). |
| INC-004 | `TypeView` and checked instantiation close open Tuple when specializing | P1 source/spec confirmed | Tuple remainder identity tests, parent/checker consumer. |
| INC-005 | Record row captured bindings and anonymous typed tail specialization incomplete | P1 source/spec confirmed | Dynamic tail, row binding, key/value tail and downstream member proofs. |
| INC-006 | Fixed-entry product projection suspected to lose open tails | **CLEARED / withdrawn** | Only re-open with a concrete misinterpreting consumer, not with an invalid round-trip assumption. |
| INC-007 | Failed semantic update can masquerade as complete snapshot | P1 source/control-flow confirmed | Failure injection and end-to-end compiler source entry variants. |

**Open incidents:** 001–005 and 007 (six), with 001 a baseline evidence blocker rather than confirmed type-system defect. INC-006 is not open.

## 4. Critical new finding: snapshot failure suppression

**Relevant source paths (current `main`):**

- `phalcom-semantic/src/session.rs:922–960` — `update()` calls private `update_with_delta()`; the latter invokes a Result-bearing update, then `unwrap_or_else(|_| ...)`. If previous snapshot exists, it returns `last_known_good` without success/failure distinction. Otherwise, it manufactures a snapshot with base tables, an empty diagnostics map and empty callable-analysis map.
- `phalcom-semantic/src/snapshot.rs:295–356` — `new_with_callable_analyses` initializes `status = SnapshotStatus::Complete`; `:869–873` — `has_errors()` checks only ordinary Error diagnostics.
- `phalcom-semantic/src/workspace.rs:123–127` — one-shot `analyze_workspace` constructs a fresh session, runs `session.update(input)` and exposes only its `snapshot`.
- `phalcom-core/src/modules/compile.rs:618,726` — inline/standalone analysis routes use `analyze_workspace` and place the result in `AnalyzedProgram.semantic`.
- `phalcom-core/src/modules/compile.rs:827–831` — `compile_analyzed` rejects only `semantic.has_errors()`.
- **Excluded:** `compile.rs:797–805` uses `update_with_budget_and_cancel(...).map_err(ProgramCompileError::SemanticUpdate)?`; module update with cancellation returns `Result`. Do not assert they swallow failures.
- Actual fallible query outcomes are propagated throughout the underlying update, e.g. `session.rs:1178–1185`, `:1224–1231`, `:2328–2336`.

The conditional failure is internally provable, but whether a source program or default-budget request exercises that failure path remains **unproven**. Do not claim emitted bytecode is incorrect or classify P0 until an executable witness is obtained.

## 5. Investigations not promoted to incidents

### 5.1 `SelfType` divergence

- `types/environment.rs:134–142` resolves `TypeData::SelfType` using `env.self_binding`.
- `types/instantiation.rs:337–343` returns `SelfType` unchanged; `TypeEnvironment::to_generic_instantiation` only exports type and row bindings.
- `TypeView::materialize_checked` (environment.rs:122–125) therefore cannot resolve the `self_binding` of its TypeEnvironment.
- **Disposition:** Candidate API mismatch only. Indexed search found no production `materialize_checked` call beyond its definition; no normative assertion that this method is expected to apply `Self` was established. Needs direct API contract/consumer trace, not a P1 defect report yet.

### 5.2 Dynamic escapes and nested structural types

- `environment.rs:109–119` represents root dynamic parameter as `TypeKnowledge::Dynamic(ExplicitEscape)`.
- `environment.rs:185–202` handles immediate generic `Applied` argument `StaticArgument::Term(Parameter)` with a dynamic binding by forming `StaticArgument::DynamicType`.
- `tests/semantic/foundations/type_annotations.rs:2793–2840` explicitly tests bare parameter / nested `List<T>` / unaffected fixed type cases (CV12–CV14).
- `environment.rs:213–256` recurses through Tuple/Callable fixed elements but `materialize_view(Parameter)` returns a parameter unchanged when binding is *only* dynamic; `Tuple` and `Callable` have no `DynamicType` argument slots.
- **Disposition:** Representation/contract gap requiring specification and expected-outcome analysis. May be intentional to reject dynamic escape in proper structural elements; current `materialize_knowledge` can still return assumed knowledge on such a result. Construct positive and negative API tests before declaring an incident. `docs/specs/generic-constraints.md` states Dynamic is not a proof of conformance or static equivalence; it does not directly specify nested structural escape materialization.

### 5.3 Record row passing through nested static arguments

`environment.rs:165–171`, `:189–199`, `:344–351` forwards raw `RecordRowId` through type-member projection / `Applied` / `StaticData`. Checked instantiation traverses row arguments (`instantiation.rs:78–103`, `:126–155`, `:309–334`). This likely belongs to the existing INC-005 root cause; do not double-count until a distinct semantic effect is reproduced.

### 5.4 Fixed entries vs whole structural shape

`docs/specs/structural-products.md` §4 explicitly separates `entries` from tails. `docs/specs/type-level-domains.md` §4.1 defines kinded `TypePack` and `LabeledTypePack` as ordered sequences. The source `store.project_positionals/project_labeled` maps to fixed entries. **Rebuilding a closed Tuple from these entries is not a required identity law for an open Tuple.** This is the reason for INC-006 clearance.

## 6. Next bounded slice — recommended order

1. **Behavioral probes, if a local checkout becomes available:** verify remote `main` SHA and local status, use pinned `nightly-2026-07-10` explicitly, write test harnesses **inside the audit folder only**; do not modify source or checked-in tests. Prioritize INC-002/003/004/005 and forced-failure INC-007. Preserve full command, output, toolchain, and failure status.
2. **INC-007 failure injection and consumer call graph:** identify an approved test seam to cause `QueryOutcome::BudgetExceeded`, `Blocked`, or `Failed` in one-shot update; compare `analyze_workspace` result with strict `update_with_budget_and_cancel`. Trace both compiler routes at `compile.rs:618,726` through `compile_analyzed/project_analyzed` and classify any guards that block incomplete products. Audit `update_module_workspace` users for stale generation and formal/editor authority.
3. **Self/dynamic materialization:** test exact root and nested forms, including `Tuple(T)`, `Callable(T)->T`, `Record` typed tail, type-member projection with row binding, static data with row argument; require honest `Known/Unknown/Dynamic` and no incorrect proof strengthening. Scope all findings to supported surface expressions or a public API contract.
4. **Additional subtype aggregation checks:** source-union conjunction with early `Blocked` and later `Refuted`, target union with budget/cancellation, applied invariance relation and proof-state ordering. Link any findings to INC-003 if shared root cause.
5. **If still no checkout:** continue independent read-only traversal of inference variable lifecycle (`checker/inference.rs`) and row solver (`types/row_solver.rs`) or compiler lowering identity; produce new incidents only with concrete source/spec contradiction.

## 7. Audit evidence policy

- Normative `docs/specs/` supersedes speculative assumptions and historical plans.
- Keep **source-confirmed transformation**, **API-executed reproduction**, **source-program reproduction**, and **end-to-end runtime impact** as separate stages.
- Preserve query failure and uncertainty distinctions; a synthetic `Complete` snapshot is not an acceptable substitute for unsuccessful publication.
- Never assume local repository/head/test state from GitHub results. Existing baseline was on macOS, with `rust-toolchain.toml` pin `nightly-2026-07-10` but an active floating nightly directory override (`rustc 1.99.0-nightly (1a98b1e13 2026-08-07)` as documented). Pinned rustc previously identified as `1.99.0-nightly (af3d95584 2026-07-09)`.
- User work remains outside implementation authorization. Audit Markdown file creation here does not modify the repository.
