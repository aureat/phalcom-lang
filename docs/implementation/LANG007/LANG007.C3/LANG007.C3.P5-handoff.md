Continue LANG007.C3.P5 from the checkpoint below. The active plan remains authoritative.

## Mission and Scope
- **Deliver now:** Complete the remaining tasks of `LANG007.C3.P5`: finish T8/T9 iterator integration closure, execute T10 (metadata, incremental parity, and LSP persistence), T11 (rest/module-export evidence transport), T12 (focused regression matrix), T13 (residual failure classification), T14 (broad certification), and T15 (walkthrough, handoff, and checkpoint finalization).
- **In scope:**
  - Tasks T8 through T15 under adopted P5 Rulings P5-A through P5-J and Invariants INV-01 through INV-28.
  - Final verification of the natural `(0..).iter.take(10).toList` pipeline (CV-22) and `toMap` single-advance (CV-21).
  - Preserving the exact structural model (INV-01..07) and canonical Dynamic generic argument envelope (INV-08..13).
- **Out of scope:**
  - Re-auditing established T1–T7 deliverables.
  - Redesigning nominal class inheritance or turning Dynamic into a `TypeId`.
  - Running full workspace certification before Gate G12 / T14.
- **Authority:** `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-luna-patch-grade-implementation-plan.md` and `docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md`.

---

## Current State & Accomplished Work

### Tasks T1–T6: Completed and Verified
- **T1 & T2 (Baseline & Ratification):** Ratified in `docs/specs/structural-products.md` and recorded in `LANG007.C3-CHECKPOINT.md`.
- **T3 (AST & Canonical Representation):** `StructuralMemberPresence::Optional`, `RecordRowTail::Anonymous`, `AnonymousRecordRemainder`, and `AnonymousTupleRemainder` implemented across `phalcom-ast` (all 282 parser tests pass).
- **T4 (Dynamic Generic Argument Slots):** `StaticArgument::DynamicType` integrated in semantic environment, associated types, calls, expressions, inference, GADT proofs, and row inference (61/61 type annotation tests pass).
- **T5 (Structural Subtyping Relations):** Implemented in `phalcom-semantic/src/types/relation.rs` (exact closed records, open records with remainders, optional fields, tuple lane independence, rejection of pack subtyping). 100% verified across relation and record row suites (CV-01 through CV-09 proven).
- **T6 (Dynamic Consumers & Persistence):** Lowering, inspect, environment, reflection, and runtime representation in `phalcom-core` updated.

### Tasks T7 & T8 (Witness Specialization & Universe Bootstrap Restoration):
- **Witness Specialization (`impls.rs`):** `specialize_witness_signature` updated to map `inferred_return` via `TypeView`, ensuring inherent methods (`iterator { self }`) specialize to the receiver.
- **Applied Dynamic Normalization (`projection.rs`):** Normalization of applied types holding Dynamic arguments preserves the applied outer head (`Option<Dynamic>`), matching `Option<Self::Item>`.
- **Universe Bootstrap Restoration:**
  - **100% of Universe bootstrap diagnostics eliminated!** All 20 `ImplConformanceIncomplete` and 3 `GenericInferenceUnderconstrained` errors are resolved.
  - Added explicit `where I |= Iterator` bounds across iterator adapters in `iterator_laws.ph`.
  - Re-exported `DuplicateKeyError` in `prelude.rs`, `errors/argument.ph`, and `package.ph`.
- **Generic Evidence Conformance Query Fix (`generic_evidence.rs`):**
  - Removed redundant recursive trait contract re-validation inside `evaluate_conformance_goal` for derived rules to prevent false `RecursiveFixpoint` blocks during `TakeIterator |= Iterable` evaluation.
- **Core Iteration Test Suite:**
  - In `phalcom-core/tests/core/language/iteration.rs`, **21 of 24 tests are currently passing**.

---

## Code and Artifact Map
- `docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md` — Authoritative checkpoint ledger.
- `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P5-luna-patch-grade-implementation-plan.md` — Active P5 implementation plan.
- `docs/specs/structural-products.md` — Normative language specification for structural tuples and records.
- `phalcom-semantic/src/generic_evidence.rs` — Generic conformance proving and evidence construction.
- `phalcom-semantic/src/impls.rs` — Witness specialization and trait default signature projection.
- `phalcom-semantic/src/types/relation.rs` — Structural subtyping relation engine.
- `phalcom-core/core/universe/src/collections/iterator_laws.ph` — Universe iterator and iterable protocols.
- `phalcom-core/core/universe/src/collections/list.ph` — List collection implementation and inherent methods.
- `phalcom-core/core/universe/src/collections/tuple.ph` — Tuple implementation and `TupleIterator`.

---

## Resume Plan (Step-by-Step for Next Agent)

1. **Close T8 & T9 (Iteration Suite Finalization):**
   - **`TupleIterator.next` (`tuple.ph`):** Ensure `_source.at(_index)` return value is cleanly unwrapped without double-`Option` or private field violations.
   - **`List` Forwarding Methods (`list.ph`):** Ensure `toList`, `toSet`, and `toMap` are accessible on `List` for dynamic method sends in eager operation tests.
   - **Un-ignore and verify CV-22:** Verify `unbounded_take_to_list_keeps_guarded_iterable_dispatch_evidence` in `phalcom-core/tests/core/language/iteration.rs` passes cleanly.
   - Run `cargo test -p phalcom-core --test core language::iteration` to verify 24/24 PASS.
2. **Execute T10 (Metadata, Incremental Parity, and Tooling):**
   - Verify serialization/deserialization round-trip of `StaticArgument::DynamicType` and new structural AST/type representations (CV-15, CV-26, CV-27).
   - Run incremental parity tests: `cargo test -p phalcom-semantic --test semantic incremental`.
3. **Execute T11 (Rest / Module-Export Evidence Transport):**
   - Verify conformance evidence transport across module exports and rest parameters (CV-24, CV-25).
4. **Execute T12 & T13 (STABILIZE Mode):**
   - Run the focused P5 law/regression matrix across `phalcom-semantic` and `phalcom-core`.
   - Classify residual test failures according to causality A/B/C/D (CV-28).
5. **Execute T14 & T15 (CERTIFY & Documentation):**
   - Run workspace validation gates (Gate G12).
   - Generate `LANG007.C3.P5-walkthrough.md` and update `LANG007.C3-CHECKPOINT.md`.

---

## Do Not Re-Explore
- Do NOT re-audit T1–T6; their types, AST, parser, and subtyping laws are verified.
- Do NOT convert `Dynamic` to a `TypeId` or add speculative compiler hacks for Range/TakeIterator.
- Do NOT run full workspace clippy or broad release tests during intermediate task edits (follow BUILD/STABILIZE discipline).
