---
id: LANG007.C4.P7.T7
category: LANG
program: LANG007
checkpoint: LANG007.C4
plan: LANG007.C4.P7
task: T7
kind: implementation-plan
title: "Residual Specialization Engine & Canonical Environment Architecture"
status: APPROVED
completion: COMPLETE
verification: FOCUSED_TESTED
depends_on:
  - LANG007.C4.P7.T6
follows: LANG007.C4.P7-INC-003-residual-specialization-environment.md
supersedes: null
prepared_against_revision: 5942666901b387519c5cd502bed2611cf97b53c4
wip_revision: 792961f9e44d81bf79d7e4e98af6c8205a9a69c6
last_updated: 2026-10-12
---

# LANG007.C4.P7.T7 — Residual Specialization Engine & Canonical Environment Architecture

## 0. Executor contract

This patch-grade implementation plan is prepared for execution under the `AGENTS.md` and Luna patch-grade lifecycle doctrine, incorporating the ratified architect ruling on follow-up consultation `INC-003`.

The implementer:
- may adapt local mechanical details, private helper names, and signature drift to the live tree;
- must preserve the three-way environment decomposition, the store-lineage-local fingerprint boundary, and all architectural invariants (A1–A17) defined herein;
- must not improvise architectural changes or bypass normalization;
- must follow the testing budget (BUILD MODE: discriminating checks only; STABILIZE MODE: focused gates);
- must classify and defer unrelated baseline failures (Category D);
- must STOP AND CONSULT when an escalation condition fires;
- must update `LANG007.C4-CHECKPOINT.md` and `LANG007.C4.P7-INC-003-residual-specialization-environment.md` upon completion.

---

## 1. Goal

Implement the session-local static residual specialization engine for `Parameter`, `Apply`, `Proof`, and `MemberProjection` residuals in `phalcom-semantic`, replacing flat-history environment capture with current-frame-anchored activation capture, strict substitution-aware supplied composition, and result-relevant application-key identity, ensuring exact parity with cold evaluation.

---

## 2. Checkpoint acceptance objective

This plan implements Task 7 (`P7.T7`) and Gate 7 (`P7.G7`) of `LANG007.C4.P7` ("Recursive generic static computation, residualization, and memoization") within Checkpoint `LANG007.C4` ("Structural Static Upgrade").

Completing T7 establishes the core operational capability of P7: taking an unreduced symbolic residual (produced during open static computation) and evaluating it to a terminal result once concrete arguments or ambient proof facts are supplied. It unblocks conditional and pattern residual specialization (T8, T9) and structural iteration specialization (T10).

---

## 3. Repository grounding

- **Repository:** `aureat/phalcom-lang`
- **Branch:** `main`
- **Reviewed commit:** `5942666901b387519c5cd502bed2611cf97b53c4`
- **WIP commit:** `792961f9e44d81bf79d7e4e98af6c8205a9a69c6`

### Verified repository facts
1. `phalcom-semantic/src/static_eval/application.rs` defines `StaticSpecializationEnvironmentKey`, `StaticApplicationKey`, `StaticSpecializationArgument`, and `CanonicalStaticProofFact`. It contains a public constructor `new` that bypasses normalization.
2. `phalcom-semantic/src/static_eval/machine.rs` contains `effective_specialization_environment` which incorrectly folds all `active_applications` using strict `merge`, causing recursive calls with different formal arguments (e.g. `Grow<T = Int>` -> `Grow<T = Next<Int>>`) to fail.
3. `StaticEvaluator` maintains `active_branch_proofs: Vec<&'a BranchProofEnvironment>` and `saved_frames: Vec<SavedFrame<'a>>`. On `load_frame`, `active_branch_proofs` is reset to empty.
4. `active_generic_evidence` and `resolve_type_term_bounded` in `machine.rs` consume canonical parameter bindings, `Equal`, and `Subtype` constraints, but ignore `local_bindings`, `local_equalities`, and `HasMember`.
5. `StaticResidualStore::content_fingerprint` in `residual.rs` hashes raw `TypeId` and `StaticValueRef`, establishing store-lineage-local identity, not durable cross-store identity.
6. `phalcom-semantic/src/db/fingerprint.rs` hashes `residual_ref.store_id().raw()` and `residual_ref.residual_id().raw()` for `StaticEvalOutcome::Symbolic`, which is an arena-identity placeholder governed by T12/P8.

> *Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.*

---

## 4. Required reads before implementation

- `AGENTS.md`
- `docs/workflow/implementation-record-lifecycle-convention.md`
- `docs/workflow/shared-consultation-escalation-protocol.md`
- `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md`
- `docs/implementation/LANG007/LANG007.C4/P7/LANG007.C4.P7-INC-001-composite-static-arguments.md`
- `docs/implementation/LANG007/LANG007.C4/P7/LANG007.C4.P7-INC-002-member-projection-bridge.md`
- `docs/implementation/LANG007/LANG007.C4/P7/LANG007.C4.P7-INC-003-residual-specialization-environment.md`
- `docs/implementation/LANG007/LANG007.C4/P7/LANG007.C4.P7-luna-patch-grade-implementation-plan.md` (specifically §§6.3, 18.1, 21.3, T7)
- Live source: `phalcom-semantic/src/static_eval/application.rs`, `machine.rs`, `residual.rs`
- Live tests: `phalcom-semantic/tests/semantic/foundations/static_residuals.rs`, `phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs`

---

## 5. Normative authority

- **Specification:** `docs/specs/` (Normative Static Evaluation and Proof Predicates)
- **Architectural Rulings:**
  - `INC-001`: Composite static arguments interned in active session `TypeStore` (`Arc::make_mut(&mut self.store)`).
  - `INC-002`: Member projection bridges through `StructuralReflectionProjector`, preserving domain-neutral outcomes.
  - `INC-003`: Refined Level-2 amendment: Three-way environment decomposition, current-frame anchored capture, nearest-active branch precedence, strict post-substitution composition, store-lineage-local fingerprints.

---

## 6. Takeover state

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Environment Carrier | `StaticSpecializationEnvironmentKey` (`static_eval/application.rs`) | `phalcom-semantic` | Must be normalized by construction; holds argument bindings, type bindings, constraints. |
| Application Key | `StaticApplicationKey` (`static_eval/application.rs`) | `phalcom-semantic` | Encodes definition, canonical explicit arguments, and result-relevant ambient environment. |
| Residual Payloads | `StaticResidualData::Proof`, `StaticResidualData::MemberProjection` (`static_eval/residual.rs`) | `phalcom-semantic` | Carry normalized `StaticSpecializationEnvironmentKey` for self-sufficient escape. |
| Evaluator Machine | `StaticEvaluator` (`static_eval/machine.rs`) | `phalcom-semantic` | Manages frame stack, local bindings, cycle detection, and memoization. |
| Structural Projector | `StructuralReflectionProjector` (`member_projection/`) | `phalcom-semantic` | Authoritative evaluator for `MemberProjection`. |

---

## 7. Architecture

The core architecture addresses the defect where historical execution history was treated as lexical/specialization scope.

```text
                  inherited semantic environment
                              │
                              ▼
                     current activation
                    /                 \
                   /                   \
          exact current args       branch proofs
                   \                   /
                    \                 /
                     ▼               ▼
                  residual-capture environment
                           │
                           ├──────────────► Proof residual
                           │
                           └──────────────► MemberProjection residual


caller semantic context
        │
        ▼
application-identity projection
        +
specialized explicit callee arguments
        │
        ▼
StaticApplicationKey
        │
        ├── cycle detection
        └── evaluator-local memo


escaped residual
        +
supplied specialization
        │
        ▼
substitution closure
        │
        ▼
strict normalized composition
        │
        ▼
existing semantic authority
        │
        ├── exact reduced result
        └── normalized residual
```

### 7.1 Three-Way Environment Decomposition

1. **Activation / Residual Capture Environment:**
   - Answers: *What exact semantic context must an unresolved residual retain if it escapes this application now?*
   - Model: `inherited_environment` + current frame's exact formal argument bindings + current active branch parameter bindings (nearest-first) + union of current canonical proof facts (`Equal`, `Subtype`).
   - Shadowing: Nearest active frame shadows older bindings for the same formal.

2. **Application Identity Environment (`StaticApplicationKey`):**
   - Answers: *Which ambient semantic facts, in addition to explicit arguments, can change this application's result?*
   - Model: Definition + explicit canonical arguments + result-relevant ambient facts.
   - Separation: Historical caller argument bindings must **not** pollute application keys.

3. **Supplied Specialization Composition:**
   - Answers: *Given an escaped residual with captured context, what is its context after supplying new exact information?*
   - Model: Strict composition after canonical substitution. Form substitution closure, substitute through captured kinded bindings and constraints, check for genuine concrete conflicts, and coalesce identical bindings.

---

## 8. Ownership boundaries

### Owns
- Creation, normalization, and composition of specialization and application environments.
- Residual specialization engine in `phalcom-semantic/src/static_eval/specialize.rs` for `Parameter`, `Apply`, `Proof`, and `MemberProjection`.
- Alignment of evaluator proof consumption with application-key proof facts.
- Store-lineage-local residual fingerprinting and documentation.

### Does not own
- General `Residual::If`, `Residual::Match`, or loop residuals (owned by T8, T9, T10).
- Durable cross-session or cross-module residual serialization (owned by P8).
- Incremental database symbolic constant fingerprinting (owned by T12/P8).
- VM or bytecode residual evaluation (prohibited).
- New proof solvers or type inference engines (must reuse existing semantic authorities).

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Specialization Environment | `StaticSpecializationEnvironmentKey` | Residual capture, Specializer | Un-normalized arrays or raw map |
| Application Identity | `StaticApplicationKey` | Cycle detection, Memoization | Raw stack fold |
| Proof Satisfaction | `generic_evidence`, `types::operations` | Proof specialization, Evaluator | Ad-hoc proof predicate solver |
| Member Projection | `StructuralReflectionProjector` | Evaluator, Specializer | Re-implemented projection logic |
| Type Interning | Session-owned `TypeStore` | Composition, Specializer | Isolated temporary `TypeStore` |

---

## 9. Global invariants

- **A1:** Active recursion history is not semantic environment authority.
- **A2:** Current activation bindings shadow older bindings only during activation/residual environment capture.
- **A3:** Supplied specialization composition is strict after canonical substitution.
- **A4:** Application keys contain explicit arguments once, plus only result-relevant ambient semantic facts.
- **A5:** Residual capture contains every exact binding/proof fact required to specialize an escaped residual independently.
- **A6:** Branch-proof projection mirrors actual evaluator proof consumption.
- **A7:** Proof facts participating in application result participate in application identity.
- **A8:** No checker-local or query-local rigid identity enters P7 residual identity unless canonicalized first.
- **A9:** Every specialization environment is normalized by construction.
- **A10:** Residual arguments are never represented as exact bindings.
- **A11:** Composite type specialization uses the active session-owned `TypeStore` only.
- **A12:** Proof specialization uses the existing proof authority.
- **A13:** `MemberProjection` specialization uses the existing T6 projector.
- **A14:** Cold exact evaluation and open-then-specialize agree semantically.
- **A15:** Cancelled, blocked, invalid, or budget-exceeded work is never cached as semantic success.
- **A16:** P7 fingerprints remain store-lineage-local.
- **A17:** Durable/cross-lineage residual identity remains P8/T12 work.

---

## 10. Non-goals

- Implementing `Residual::If` or `Residual::Match` specialization (deferred to T8/T9).
- Modifying T6's checked member projection bridge or reflection projector.
- Changing incremental database hashing of symbolic outcomes in `src/db/fingerprint.rs`.
- Inventing a new proof system, subtype checker, or unification engine.
- Exposing unchecked public constructors for specialization environments.

---

## 11. Expected impact map

### Expected source areas
- `phalcom-semantic/src/static_eval/application.rs`: Environment normalization, private constructors, application-identity projection, error variants.
- `phalcom-semantic/src/static_eval/machine.rs`: Explicit inherited environment state, current-frame anchored capture, branch proof projection, proof consumption alignment.
- `phalcom-semantic/src/static_eval/residual.rs`: Residual payload documentation, fingerprint wording correction.
- `phalcom-semantic/src/static_eval/specialize.rs` (new): Specialization engine for `Parameter`, `Apply`, `Proof`, and `MemberProjection`.
- `phalcom-semantic/src/static_eval/mod.rs`: Module exports.

### Expected tests
- `phalcom-semantic/tests/semantic/foundations/static_residuals.rs`: Normalization laws, fingerprint lineage tests, direct escape tests.
- `phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs`: Activation shadowing, substitution-before-conflict, cold-exact parity, cancellation/budget.

### Expected documentation/state
- `docs/implementation/LANG007/LANG007.C4/P7/LANG007.C4.P7-INC-003-residual-specialization-environment.md`
- `docs/implementation/LANG007/LANG007.C4/P7/LANG007.C4.P7-luna-patch-grade-implementation-plan.md`
- `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md`

---

## 12. Implementer decision authority

### 12.1 FIXED
- Three-way environment decomposition (Activation Capture vs Application Identity vs Supplied Composition).
- Nearest-first branch proof precedence.
- Substitution closure before conflict checking in supplied composition.
- Reuse of existing `StructuralReflectionProjector` and proof systems.
- Store-lineage-local fingerprint boundary.

### 12.2 MECHANICALLY FLEXIBLE
- Helper names and private decomposition in `specialize.rs` and `application.rs`.
- Choosing between a dedicated newtype `StaticApplicationEnvironmentKey` vs an explicit projection `StaticSpecializationEnvironmentKey::application_identity_projection()` that validates empty argument bindings.
- Internal error variant names under `StaticSpecializationEnvironmentError`.

### 12.3 VERIFY-FIRST
| Assumption | Where to verify | If false |
|---|---|---|
| `resolve_type_term_bounded` depth limit | `machine.rs:2993` | Preserve depth/recursion bounds |
| Proof consumers ignore `HasMember` | `machine.rs:2956-3025` | Follow Future Consumer Rule (§12) |
| Session `TypeStore` availability | `machine.rs:254` | Use `Arc::make_mut` via existing session hooks |

---

## 13. Global STOP / CONSULT triggers

1. If specialization requires creating a secondary temporary `TypeStore` instead of the session store.
2. If an evaluator proof consumer needs to read checker-local or query-local rigid identities that cannot be canonicalized.
3. If cold exact evaluation and open-then-specialize cannot be reconciled without weakening typing or proof invariants.
4. If exact cycle detection or memo hits fail due to an irreconcilable key collision.

---

## 14. Debugging budget

- **Mechanical failures:** Maximum 3 coherent cycles (e.g. borrow checker, import, or signature updates).
- **Semantic failures:** Form explicit hypothesis; maximum 1 serious correction. If unresolved, STOP AND CONSULT.
- **Architectural failures:** 0 attempts. Stop immediately.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-ENV-01 | Normalization | A9 | Input order independence, duplicate coalescing, equality symmetry, subtype directionality |
| CV-ENV-02 | Activation Shadowing | A1, A2 | `Grow<T = Int>` -> `Grow<T = Next<Int>>` -> `Grow<T = Next<Next<Int>>>` captures deepest binding without conflict |
| CV-ENV-03 | Unrelated Inherited Bindings | A2, A4 | Outer bindings preserved across inner calls with distinct formal owners |
| CV-ENV-04 | Branch Proof Precedence | A6 | Nearest active branch binding wins in nested proofs |
| CV-ENV-05 | Cycle Identity | A1, A4 | Repeating `F<A> -> F<A>` cycles promptly without pollution from caller argument history |
| CV-ENV-06 | Memo Identity | A4, A7 | Same args + same ambient proof = memo hit; different proof = separate evaluation |
| CV-SUB-01 | Substitution Before Conflict | A3 | Captured `T = Box<U>` + Supplied `U = Int, T = Box<Int>` succeeds |
| CV-SUB-02 | Conflict Rejection | A3 | Captured `T = Box<U>` + Supplied `U = Int, T = Box<String>` fails closed |
| CV-ESC-01 | Direct Proof Escape | A5, A12, A14 | Child `Proof` escapes, specializes after frame teardown, matches cold exact |
| CV-ESC-02 | Direct MemberProjection Escape| A5, A13, A14 | Child `MemberProjection` escapes, specializes to `ReducedValue`, matches cold exact |
| CV-ID-01 | No-New-Facts Identity | A5, A14 | `specialize(R, empty) == normalize(R)` reuses interned ref |
| CV-ID-02 | Idempotence | A3, A14 | `specialize(specialize(R, E), E) == specialize(R, E)` |
| CV-RES-01 | Resource & Cancellation | A15 | Cancellation / step budget halts specialization cleanly without caching invalid results |

---

## 16. Verification execution budget

- **BUILD MODE:** `RUSTFLAGS='' cargo check -p phalcom-semantic --tests`
- **STABILIZE MODE:**
  - `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::static_residuals`
  - `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::static_recursive_evaluation`
- **DO NOT RUN DURING BUILD:** Full workspace tests, full compiler/VM tests (`cargo test --workspace`).

---

## 17. Baseline / unrelated failure policy

- Pre-existing failures outside `static_eval` are Category D.
- Record any unexpected failures in `LANG007.C4-CHECKPOINT.md`. Do not attempt out-of-scope fixes.

---

# 18. Tasks

## T7-A — Lock Environment Invariants & Normalization

### Purpose
Make `StaticSpecializationEnvironmentKey` normalized by construction, eliminate the un-normalized `new` constructor bypass, make fields private or provide strict builders, and implement strict same-layer composition.

### Files
- **Modify:** `phalcom-semantic/src/static_eval/application.rs`
- **Tests:** `phalcom-semantic/tests/semantic/foundations/static_residuals.rs`

### Required shape
- Remove or privatize `StaticSpecializationEnvironmentKey::new`.
- Provide `normalized(...) -> Result<Self, StaticSpecializationEnvironmentError>`.
- Enforce idempotence: $N(N(E)) == N(E)$.
- Canonicalize `Equal(A, B)` symmetrically (`min(A, B), max(A, B)`), preserve `Subtype` direction.
- Add error variants: `ConflictingArgumentBinding`, `ConflictingTypeBinding`, `ArgumentDomainMismatch`, `MalformedEnvironment`.

### Acceptance
`static_residuals.rs` verifies order independence, duplicate coalescing, symmetric equality, and fail-closed conflict for same-layer bindings.

---

## T7-B — Split Activation Capture from Application Identity

### Purpose
Anchored environment capture on the current frame. Stop folding `active_applications`. Decouple application-key identity from activation-history argument bindings.

### Files
- **Modify:** `phalcom-semantic/src/static_eval/application.rs`, `phalcom-semantic/src/static_eval/machine.rs`
- **Tests:** `phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs`

### Required shape
- Add `inherited_specialization_environment: StaticSpecializationEnvironmentKey` to `StaticEvaluator`.
- In `save_current_frame` / `load_frame`, maintain inherited environment cleanly.
- Implement `capture_residual_environment(&self) -> Result<StaticSpecializationEnvironmentKey, Stop>`:
  - Start with `self.inherited_specialization_environment.clone()`.
  - Overlay current frame's exact arguments from `current_application_key.arguments` and `self.body`.
  - Overlay current `active_branch_proofs` (nearest-first).
  - Union canonical proof facts.
- Define application key construction using `application_identity_projection()` (ensuring no exact argument bindings contaminate memo/cycle identity).

### Acceptance
Recursive definitions with changing arguments (`Grow<T = Int>` -> `Grow<T = Next<Int>>`) capture environments without colliding with historical activations.

---

## T7-C — Correct Branch-Proof Projection

### Purpose
Ensure branch parameter binding capture mirrors `resolve_type_term_bounded` (nearest active branch proof wins). Exclude unconsumed checker-local proof facts.

### Files
- **Modify:** `phalcom-semantic/src/static_eval/application.rs`, `phalcom-semantic/src/static_eval/machine.rs`
- **Tests:** `phalcom-semantic/tests/semantic/foundations/static_residuals.rs`

### Required shape
- Update `from_branch_proofs` to iterate in reverse for parameter bindings (nearest-first wins).
- Canonicalize and union `Equal` and `Subtype` constraints.
- Explicitly omit `HasMember`, `local_bindings`, and `local_equalities`.

### Acceptance
Nested branch proofs with shadowed type parameters project the nearest binding into the captured environment.

---

## T7-D — Align Application Identity & Proof Consumption

### Purpose
Ensure evaluator proof consumers consult the canonical inherited environment so that application identity and runtime proof evaluation agree.

### Files
- **Modify:** `phalcom-semantic/src/static_eval/machine.rs`
- **Tests:** `phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs`

### Required shape
- Teach `resolve_type_term_bounded` and `active_generic_evidence` to consult `inherited_specialization_environment` type bindings and proof facts.
- Guarantee: Facts participating in application identity participate in evaluation, and vice-versa.

### Acceptance
Memoization test passes: same arguments with different inherited proof facts evaluate separately; identical facts hit memo cache.

---

## T7-E — Implement Substitution-Aware Composition

### Purpose
Implement composition of captured environment with supplied specialization, computing substitution closure before conflict checking.

### Files
- **Modify:** `phalcom-semantic/src/static_eval/application.rs`, `phalcom-semantic/src/static_eval/specialize.rs`
- **Tests:** `phalcom-semantic/tests/semantic/foundations/static_residuals.rs`

### Required shape
- Given captured context $E_{\text{cap}}$ and supplied context $E_{\text{sup}}$:
  1. Build substitutions from available exact bindings.
  2. Substitute through captured kinded bindings using `TypeSubstitution` and the session `TypeStore`.
  3. Substitute through proof facts.
  4. Compare canonicalized assignments: coalesce identical, reject contradictory.
  5. Form canonical union.

### Acceptance
Transitive substitution cases (`T = Box<U>` with supplied `U = Int, T = Box<Int>`) succeed without false conflicts; conflicting substitutions fail closed.

---

## T7-F — Parameter and Apply Specialization

### Purpose
Implement specialization for `StaticResidualData::Parameter` and `StaticResidualData::Apply`.

### Files
- **Create:** `phalcom-semantic/src/static_eval/specialize.rs`
- **Modify:** `phalcom-semantic/src/static_eval/mod.rs`, `phalcom-semantic/src/static_eval/machine.rs`
- **Tests:** `phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs`

### Required shape
- `Parameter(P)`: Lookup exact binding in composed environment. Return `StaticEvalTerm::Kinded` or `Value`. If unmapped, retain normalized parameter residual.
- `Apply`: Specialize arguments recursively. If all arguments become concrete, invoke `StaticEvaluator` using existing cycle/memo machinery. If any remain residual, re-intern normalized `Apply`.

### Acceptance
Open applications specialized with concrete arguments reduce to exact values matching cold execution.

---

## T7-G — Proof Specialization

### Purpose
Implement specialization for `StaticResidualData::Proof`.

### Files
- **Modify:** `phalcom-semantic/src/static_eval/specialize.rs`
- **Tests:** `phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs`

### Required shape
- Specialize goal propositions via composed environment.
- Query existing proof authorities (`GenericEvidenceEnvironment`, `TypeHierarchy`).
- Map outcomes: `Proven -> Bool(true)`, `Refuted -> Bool(false)`, `Unknown -> Proof residual`, preserving `Blocked` / `Dynamic` / `Cancelled`.

### Acceptance
Directly escaped `Proof` residual specializes cleanly after caller frame is popped; matches cold exact proof result.

---

## T7-H — MemberProjection Specialization

### Purpose
Implement specialization for `StaticResidualData::MemberProjection`.

### Files
- **Modify:** `phalcom-semantic/src/static_eval/specialize.rs`
- **Tests:** `phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs`

### Required shape
- Compose environment, substitute projection subject and arguments.
- Call `StructuralReflectionProjector::project_member`.
- Preserve `ReducedValue(StaticValueRef)` with `ValueType(ProductShape)`. Do not route through Kind-only path.

### Acceptance
Directly escaped `Product::Shape<T>` residual specializes to exact shape value matching cold exact projection.

---

## T7-I — Fingerprint Boundary Cleanup

### Purpose
Refine documentation and tests for residual fingerprints to accurately state store-lineage-local semantics. Record DB fingerprinting as a T12 boundary.

### Files
- **Modify:** `phalcom-semantic/src/static_eval/residual.rs`
- **Tests:** `phalcom-semantic/tests/semantic/foundations/static_residuals.rs`

### Required shape
- Update `test_hash_independent_of_allocation_order` comments and assertion messages: prove allocation order independence for identical IDs within compatible lineages, not cross-lineage semantic equality.
- Document that symbolic outcome hashing in `db/fingerprint.rs` is an arena-identity placeholder to be addressed in T12/P8.

### Acceptance
Tests pass with accurate assertions reflecting store-lineage-local identity.

---

## T7-J — G7 Acceptance Verification

### Purpose
Certify full T7 acceptance against the 14 mandatory test areas.

### Files
- **Tests:** `phalcom-semantic/tests/semantic/foundations/static_residuals.rs`, `phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs`

### Required shape
- Run discriminating test suite covering all CV items (CV-ENV-01 through CV-RES-01).
- Confirm zero compiler warnings in modified files.

### Acceptance
All focused tests pass. Gate G7 is promoted to `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

---

# 19. Verification gates

## G7.A — Environment Normalization Gate
Run:
```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::static_residuals::specialization_environment_normalization
```
Status: PASS (1 passed; 0 failed). Order independence, symmetry, duplicate coalescing, and idempotence confirmed.

## G7.B — Activation Shadowing & Recursion Gate
Run:
```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::static_recursive_evaluation::recursive_activation_shadowing
```
Status: PASS (1 passed; 0 failed). Deep recursion captures correct frame bindings without collision; unrelated bindings preserved; cycle and memo identity verified.

## G7.C — Substitution Before Conflict Gate
Run:
```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::static_residuals::substitution_before_conflict
```
Status: PASS (1 passed; 0 failed). Transitive substitutions compose without false conflicts, and genuine contradictions fail closed.

## G7.D — Direct Escape & Parity Gate
Run:
```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::static_recursive_evaluation::direct_residual_escape_and_parity
```
Status: PASS (1 passed; 0 failed). Escaped Proof and MemberProjection residuals specialize to identical cold-evaluation values; no-new-facts identity and idempotence confirmed.

## G7.E — Cancellation & Budget Gate
Run:
```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::static_recursive_evaluation::specialization_cancellation_and_budget
```
Status: PASS (1 passed; 0 failed). Pre-cancelled tokens and exhausted step budgets halt specialization cleanly without corrupted caching.


---

# 20. Acceptance scenarios and exit criteria

### Scenario 1: Exact Changing-Argument Recursion
`Grow<T = Int>` calling `Grow<T = Next<Int>>` captures `Grow.T = Next<Int>` in the inner frame and terminates at base case without flat-frame conflict.

### Scenario 2: Direct Child Escape
An application returns an unreduced `Proof` or `MemberProjection` child residual directly. The caller frame is torn down. Supplying exact arguments later reduces the residual to the exact terminal value matching cold evaluation.

### Scenario 3: Substitution Closure
Supplying `U = Int` and `T = Box<Int>` to captured context `T = Box<U>` resolves compatibly without conflict. Supplying `T = Box<String>` fails closed with `ConflictingTypeBinding`.

### Exit Criteria
1. Tasks T7-A through T7-J are fully implemented.
2. Gates G7.A through G7.E pass with verified non-zero test counts.
3. Invariants A1–A17 are preserved.
4. `LANG007.C4-CHECKPOINT.md` and `INC-003` are updated to record G7 promotion.
