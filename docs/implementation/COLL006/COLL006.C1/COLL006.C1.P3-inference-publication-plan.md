---
id: COLL006.C1.P3
program: COLL006
checkpoint: COLL006.C1
kind: implementation-plan-supplement
scope: COLL006.C1.P3.T4
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
parent_plan: COLL006.C1.P3-implementation-plan.md
---

# P3 T4 supplement — Inference-owned sequence publication

## 1. Deliverable and authority

Make executable sequence construction facts visible only after every inference owner on which those facts depend has accepted its final, materializable result. Failed owners discard dependent candidates; independently established products remain available.

This is the active implementation supplement to P3 T4, authorized by the user's implementation request. Completion and verification are recorded in the checkpoint and P3 walkthrough. It does not close P3 or create a separate checkpoint/state record.

Fixed authorities: repository AGENTS.md; the C1 checkpoint; P3's solved-only publication contract; `docs/specs/type-level-domains.md` canonical terms; `docs/specs/generic-constraints.md` §13 (all instantiated constraints must be satisfied, solver order is not source semantics); existing causal-invalidity and proof-integrity behavior. This changes checker publication lifetime, not source-language inference or runtime semantics.

Required implementation reads: this supplement, current P3 plan/handoff, checkpoint consultation records, implementation-record convention, the exact sources/tests below, and semantic test placement README. Preserve all existing dirty work, including the unrelated Bytes edit.

## Current implementation checkpoint — 2026-10-08

This supplement is `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. The late-failure discriminator, checker-local registry, simple owners, and substantial application/row integration are present. Ten focused publication tests completed successfully before the latest static caller change; three registry unit tests passed earlier. T5 static fail-closed handling and incremental parity, several T4 coverage obligations, and T6 gates are unfinished. The latest `session.rs` caller correction has not been compiled or tested. See the [continuation report](COLL006.C1.P3-inference-publication-report.md) for exact source/evidence boundaries; the checkpoint remains the state authority.

## 2. Verified repository grounding

Prepared on `feat/telos-collection-upgrade` in `.worktrees/telos-collections-upgrade`, HEAD `e3dfe492` plus the uncommitted continuation described in the P3 handoff. Re-read named symbols before editing; line numbers below locate the planning snapshot, not frozen signatures.

| Current path/symbol | Observed behavior | Consequence |
|---|---|---|
| `checker/sequence_literal.rs::synthesize`, around 398 | Inserts a locally valid resolution into `ctx.sequence_literals` | Contextual inference can still fail later |
| `checker/expression.rs::analyze_expression`, around 76 | Adds the outer subtype relation after synthesis | Publication can precede the decisive constraint |
| `checker/context.rs::finish_inference_context`, around 1356 | Removes only the solver graph | No product commit/discard lifecycle exists |
| `checker/statement.rs`, around 94–110 | Solves annotation, attempts materialization, then disposes context | Final owner evidence exists but is not communicated to publication |
| `checker/call.rs`, around 1737/2332/2463 | Solves during seeding, arguments, and expected-result checking | An intermediate Solved outcome cannot commit products |
| `checker/inference.rs::finish_frame`, around 2700 | Classifies one frame, not every ancestor/descendant | Child success is not root acceptance |
| `checker/call.rs::publish_generic_return_with_rows` | Ordinary and row inference both affect accepted results | Scalar solver success alone is insufficient |
| `checker/call.rs`, around 1973/4196 | A generic parameter expectation can materialize into ExpectedType::Proper before the application finishes | Preserve application provenance separately from the expected-type variant |
| `checker/row_inference.rs::GenericApplicationSession` | Query row equations use the coordinator, but parameter bindings and row-premise proofs remain application-local | Deferred readiness must retain these local facts before the application view is dropped |
| `checker/call.rs::terminal_generic_return_with_fallback` | A failing call can retain a canonical fixed return type | Known TypeId alone is not an acceptance receipt |
| `checker/context.rs::run_flow_probe`, around 1088 | Fresh product maps, shared Rc solver handles | Publication state cannot live in the shared coordinator |
| `checker/context.rs::with_resolver`, around 893 | Copies context products and solver handles | Pending/terminal publication state needs explicit isolated copy semantics |
| `checker/static_body.rs`, around 293/330 | Copies sequence index before final expected inference solve | Successful static artifacts could miss newly committed candidates |
| `checker/context.rs::finalize_with_normal_returns`, around 3686 | Exports sequence index directly | Only committed products may be present there |
| `checker/expression.rs`, around 115 | Explanation reads the sequence index | Private candidate lookup is needed while publication is deferred |
| `modules/semantic_lowering.rs` in core, around 1423 | Projects all published sequence resolutions | Consumers must not reconstruct acceptance or see pending entries |

The previous fixed-slot corrections are useful but do not provide general inference ownership. Retain them as immediate diagnostics; do not use them as the lifetime mechanism.

### Discriminating source scenario

The following uses source forms already present in the generic proof-integrity tests. Its exact diagnostic and product assertions must be established by T1; it has not been executed during planning.

```telos
class Consumer {
  @class
  reject<T>(_ values: Vector<T, 1>) -> Int where T == String { 0 }

  @class
  run() {
    let independent = <Int, 1>[9]
    let rejected = Consumer.reject(<Int, 1>[1])
  }
}
```

The explicit Vector argument can form closed local construction facts while the generic application's T is unsettled. Its fixed extent agrees, so the existing fixed-slot guard does not decide the generic constraint. The call's `T == String` obligation conflicts with the argument's Int evidence. The dependent argument construction must not survive in the final executable sequence index; the independent sibling must survive. The fixed Int return type must not turn the failed application into a publication success.

## 3. Architecture decision

Use checker-owned staged publication with explicit owner acceptance. Keep `SequenceLiteralResolution` and the committed sequence index canonical and unchanged. Introduce private query-local bookkeeping in `CheckingContext` (a private helper module is mechanically flexible).

Conceptual state:

```text
OwnerGate = (InferenceContextId, InferenceFrameId)
OwnerState = Open | WaitingForAncestor | Accepted | Rejected(reason)
PendingSequence = closed SequenceLiteralResolution + set of required OwnerGates
CommittedSequences = existing SequenceLiteralResolutionIndex
```

These states describe authorization to publish executable construction facts, not a second type/proof solver. They are exact lifecycle facts. Accepted/Rejected are terminal. Waiting is not failure. Multiple required gates combine conjunctively: every gate must accept. Any rejection discards the candidate. No widening, heuristic confidence, or runtime fallback is involved.

Keep a single payload owner: a resolution is pending or committed, never duplicated between indexes. Owner dependencies and terminal receipts may be indexed separately. Deterministic BTree collections are suitable. Context/frame IDs, terms retained for deferred readiness, and receipts remain query-local and never enter CallableAnalysis, CheckedStaticExpression, metadata, fingerprints, reflection, or core lowering.

### Dependency capture — actual premises, not lexical containment

1. Capture the original outer ExpectedType's inference owner and explicit contextual publication provenance at expression entry, before literal-head elaboration can push another context.
2. A product checked against an unresolved contextual inference term depends on that owner. A generic argument also depends on its application owner when provisional parameter context has already materialized into ExpectedType::Proper. Canonicality of the expected type does not erase that application dependency. An independently canonical, nonprovisional expectation does not acquire a dependency merely because a frame is active nearby.
3. Preserve the actual ancestor dependency of borrowed child frames in the same inference graph. A locally accepted child does not remove its unsettled ancestor gate.
4. Explicit literal-head inference can add a separate local owner. The literal result can require both local and outer owners. Independent children inferred solely within the local head are not automatically dependent on an unrelated outer failure.
5. Propagate unresolved publication dependencies from required operand facts when those facts are used to establish element type/cardinality. Reuse existing required-operand composition; do not walk source ranges or tag every active frame.
6. A private expression dependency index in CheckingContext may accompany the pending registry. It tracks premise ownership only; it must not reconstruct types or duplicate resolution payloads.
7. When a candidate is registered after an owner has settled, normalize against the retained terminal receipt: Accepted clears that gate; Rejected discards; unknown/disposed owner without acceptance fails closed. Do not turn a missing solver handle into success.

At generic and union argument entry, pass a checker-private contextual publication input alongside ExpectedType. It contains the application gate even when `materialize_for_expected` produced a proper type. The root argument's candidate consumes this input; derived element/member expectations carry it only when they use that provisional context. Do not install an ambient gate on every expression visited while analyzing the argument. Independent nested declarations/bodies with their own established expectations retain their independence. Non-generic argument checking has no provisional application gate unless one arrives through an actual enclosing contextual premise. API decomposition is flexible; preserving provenance through canonicalization is fixed.

### Publication operations

- `stage_sequence(resolution, dependencies)`: canonical local facts only. Commit immediately if there are no outstanding dependencies and the expression's local construction checks succeeded; otherwise keep private.
- `accept_owner(gate, readiness)`: only the owning semantic path may provide final acceptance. Clear the gate and commit candidates whose complete dependency set is now accepted.
- `defer_owner(gate, ancestor, requirements)`: child context is reusable but not independently accepted. Preserve the gate and its actual ancestor dependency; keep necessary query-local materialization requirements until final parent solving.
- `reject_owner(gate, reason)`: discard only dependent candidates and propagate rejection through actual owner dependencies. Preserve unrelated committed/pending products and normal diagnostics/explanations.
- `dispose_context(context)`: reject unresolved candidates, then remove solver state. Disposal never implies acceptance. All current disposal/error paths must use this centralized defense.

Exact private API names are flexible. A Boolean named success without evidence is not an adequate interface.

### Acceptance receipt

An owner accepts only after its final constraints have been processed, required scalar/static/row terms can be materialized canonically, and the owner has no invalid/blocked/ambiguous/cancelled/budget/internal-failure result relevant to the candidate.

For a generic call, require final application success, mandatory where-constraint proof, applicable row completion, canonical materialization, and the final causal/status decision. Preserve existing Established/Assumed evidence distinctions; this lifecycle must not strengthen advisory or assumed facts into formal proof. A known fallback return type does not satisfy these conditions. Do not modify existing epistemic fallback behavior just to simplify product gating.

For a deferred child, after the parent finishes its final existing solve, the owning inference path must revalidate the child's saved readiness requirements using existing frame completion, residual-obligation, row, and materialization facilities. The registry does not solve constraints. Do not accept a child merely because the root frame is solved: `finish_frame(root)` does not classify all child frames. Do not add a broad all-variable solve to every expression as a substitute.

### Deferred readiness ownership and settlement order

Before returning from an ancestor-dependent generic application, extract an owned, checker-private `DeferredApplicationRequirements` record. Retain its context/frame and required ancestor gates; active parameter-to-term bindings; required result/argument materialization terms; row parameter bindings, row roots, and application-local row-premise proof states; references to its registered mandatory residual obligations; and the local causal/status evidence needed to distinguish ancestor-blocked readiness from terminal failure. Retain canonical signature/result inputs only as needed to invoke existing materialization rules. An already rejected child cannot become deferred merely because an ancestor is open.

Extract row requirements before the local `GenericApplicationSession` is dropped. Its query-backed constraints already belong to the existing coordinator, while its `parameter_bindings` and `row_proofs` do not. Introduce narrow extraction/revalidation helpers in `row_inference.rs` that preserve those facts by value and use the same `row_solution_for_roots`, proof-state meet, and instantiation/materialization rules as ordinary application checking. Do not retain a stale row outcome as final evidence, recreate inference variables, or duplicate the coordinator's equations/solver. Pending records are plain query-local owned data, so context copies can copy them by value without sharing publication mutation through Rc. Verify the complete set of needed local fields during T4; a missing proof premise must cause escalation rather than inferred acceptance.

The owning semantic path settles in this order:

```text
final parent constraint solve
  -> revalidate required deferred children and mandatory residuals
  -> materialize required child and parent results; aggregate final status
  -> mark accepted owner gates and commit eligible sequence products
  -> dispose inference context
```

Children waiting on the parent may become locally ready during revalidation, but their products still require parent acceptance. Required child failure prevents parent acceptance; rejection discards the dependent candidates. Keep deferred validation separate from terminal receipt publication so cyclic authorization is not introduced between parent acceptance and child readiness. Budget/cancellation during revalidation follows existing blocked/control reporting and never commits. No Accepted receipt is issued before this readiness pass completes.

If a valid existing deferred application cannot provide this receipt without changing canonical inference semantics, STOP AND CONSULT with that reproducer. Do not publish on trust or silently drop a product required by an otherwise valid program.

## 4. Owner integration inventory

| Owner/path | Required terminal integration |
|---|---|
| Declaration partial annotation (`statement.rs`) | Retain ownership until solve + annotation materialization + initializer contract reconciliation are known. Accept only the valid result; reject all other terminal outcomes. Avoid the current early disposal before reconciliation. |
| Explicit literal head (`sequence_literal.rs`) | Preserve outer owner before elaboration. Settle local head only after local construction, solved head materialization, element/cardinality checks, and required dependencies are valid. Dispose on every exit. |
| Annotation elaboration error (`annotation_elaboration.rs:113`) | Reject/dispose; never accept. |
| Owned generic application (`call.rs:1652`) | Return final readiness alongside the internal result. Apply acceptance only after final call causal/status aggregation; move disposal late enough to provide evidence. |
| Borrowed generic child (`call.rs:1647–1650`) | Child readiness can become locally ready but cannot clear parent dependency. Underconstrained ancestor-dependent children defer; actual terminal failure rejects. Apply terminal receipts only through the settlement ordering in §3. Pop/close is cleanup only. |
| Union shared scope (`UnionGenericInferenceScope::finish`) | Typed arguments are reused by actual arms. Preparation's probe-frame close is not rejection. Accept only after required arms, joined result, and final status/causal aggregation succeed; failure paths reject. Current finish runs before final knowledge/status assembly and must move. |
| Union preparation failures (`call.rs:4151/4158/4173/4190`) | Explicit rejection and centralized disposal, preserving existing diagnostics/control outcomes. |
| Static expected owner (`static_body.rs:330`) | Resolve/settle the owner before extracting committed indexes. Do not copy the index and then accept candidates afterward. |
| Callable/static final artifact | Export committed index only; unresolved candidates are never serialized. Detect unresolved ownership at finalization through existing incomplete/control/internal-invariant reporting; do not silently certify a ready artifact that lacks required construction facts. |

No change is required in the solver's intermediate `solve_inference_in_frame` or `close_frame` semantics. No compiler/VM workaround is authorized.

## 5. Nested contexts, probes, and explanations

- Keep registry and terminal receipts outside the shared `Rc<RefCell<InferenceCoordinator>>`.
- `run_flow_probe` starts with fresh pending-product and expression-dependency maps, matching its fresh executable product maps. Copy existing owner metadata/receipts by value only as needed to interpret inherited ExpectedType/frame references. Probe accept/reject/dispose cannot mutate parent candidates or receipts, and no probe pending products merge back. Existing solver sharing is inherited; this supplement does not redesign speculative solver behavior.
- `with_resolver` copies publication state coherently with the existing product maps. It must not share mutable publication containers or merge child state into the parent implicitly. The current method's definition has no discovered call sites; retain that copy contract rather than inventing a merge mechanism.
- Union argument preparation is different from a disposable flow probe: its argument products are consumed by real arms and stay pending until union acceptance.
- Provide a private read-only construction lookup for explanation building that can see locally checked pending facts. It must not expose them through the committed executable index. Such explanations describe local formation and contextual requirements, not final inference acceptance; keep existing cause/derivation links. No new durable provisional TypeId or inference gate is permitted in ExplanationArena.
- Invalidity discovered after expression synthesis must be reflected by owner rejection; retaining a useful type/explanation is not permission to retain an executable construction capability.

## 6. Invariants and non-goals

- INV-01: committed sequence resolutions have closed canonical terms and accepted required owners.
- INV-02: intermediate solver success, known fallback types, frame close, and context disposal never authorize publication.
- INV-03: ancestor rejection prevents dependent child publication; independent siblings/children survive.
- INV-04: no inference identity or pending lifecycle state escapes checker-local state.
- INV-05: probe publication effects are isolated; reanalysis does not inherit terminal receipts from another query/revision.
- INV-06: final callable/static/compiler artifacts contain only committed products; successful artifact extraction occurs after owner settlement.
- INV-07: established diagnostics, useful epistemic types, exact extent proofs, and concrete/symbolic runtime lowering retain their semantics.

Non-goals: negative-extent formation, Array API redesign, changing inference selection/constraint solving, global transactionalization of every semantic index, changing runtime construction, P4 materialization/default cutover, or repairing unrelated baseline failures.

The initial implementation covers all sequence families through one path. Do not create Vector-only transaction bookkeeping. Other executable indexes may have analogous issues; record them if found, but expand this plan only if they are necessary to prove the sequence contract.

## 7. Execution tasks and dependency ordering

T1 -> T2 -> T3 -> T4 -> T5 -> T6. Tests are added with the owning task; no broad test ladder after every pass.

### T1 — Establish the late-failure discriminator

Read: generic proof-integrity fixtures and existing P3 semantic cases. Modify tests only: add `foundations/coll006_c1_p3_publication.rs`, registered in `foundations/mod.rs`.

Add the generic-constraint scenario in §2 and its successful twin. Assert diagnostic/status, absence of the dependent sequence resolution, preservation of the independent sibling, and retained fixed return knowledge on the invalid generic call. Add successful canonical/partial annotation cases so a blanket suppression implementation cannot pass.

Run only the new publication filter. Expected baseline is a failing nonpublication assertion, not a parse/selector fixture error. If the old source no longer reproduces, use one bounded source inspection to identify the actual delayed constraint path before coding. Do not weaken fixed-slot guards to manufacture a failure.

Acceptance: an executable reproducer distinguishes a late owner failure from early literal-local rejection; valid controls select actual tests.

### T2 — Add checker-local publication lifecycle

Modify: `checker/context.rs`; optionally private `checker/sequence_publication.rs` and its module registration. Add registry unit tests next to the owner/helper.

Implement staged/committed payload ownership, conjunctive owner dependencies, retained terminal receipts, child deferral, centralized unresolved-context rejection, and isolated clone/probe behavior. No coordinator-owned product state. Keep existing committed index shape.

Law tests: no publish before acceptance; two required gates; accept then ancestor reject; terminal/missing owner; cleanup with cancellation/materialization rejection; independent survivor; probe does not mutate parent; no unresolved entry exports.

Intermediate owner integration is intentionally incomplete. Run only helper/context unit tests, not core runtime suites. Record a durable interface in the checkpoint only once it is coherent.

### T3 — Route literals and simple owners

Modify: `expression.rs`, `sequence_literal.rs`, `annotation_elaboration.rs`, `statement.rs`, and required dependency composition plumbing only as necessary.

Capture original expected owner and explicit contextual provenance at expression entry; stage closed sequence facts; propagate actual premise dependencies; provide private explanation lookup. Integrate declaration and explicit-head final readiness/disposal after materialization and final checks. Keep early fixed-slot diagnostics.

Tests: valid partial annotations commit; outer late conflict discards; explicit-head local success plus outer failure remains pending/rejected; independent nested closed literals survive; failed materialization and cancellation discard; family explanation remains correct before/after settlement.

Run focused publication tests for these paths. Do not run workspace/core suites here.

### T4 — Integrate generic, deferred-child, and union owners

Modify: `call.rs`, private extraction/revalidation helpers in `row_inference.rs`; narrowly extend existing coordinator readiness inspection only if required for saved child scalar/row materialization. Publication logic remains in CheckingContext.

Return explicit final readiness from owned/borrowed application paths rather than infer it from TypeKnowledge. Do not settle at seeding/argument-phase solves. Preserve where constraints, row completion, causal-invalidity capture, and known invalid-call return knowledge. Finish union scope after final aggregate decision. All early exits reject or defer explicitly.

Carry the application publication gate into argument analysis even when the expectation is already Proper. Extract deferred requirements before dropping application-local row evidence; revalidate required children before parent acceptance using the ordering in §3. Exercise both unresolved and provisionally materialized parameter expectations.

Tests: T1 late generic conflict; fixed-return failure; child solved then parent conflict; deferred child eventually accepted; deferred child still incomplete; all required union arms accepted versus one invalid arm; union argument analyzed once; independent products retained; row-dependent valid and failing cases.

Run publication filter plus `foundations::generic_inference_proof_integrity` and `foundations::record_row_inference` only when these boundaries are changed. If child readiness requires new inference semantics, STOP AND CONSULT rather than fabricating a receipt.

### T5 — Close extraction and revision boundaries

Modify: `static_body.rs`, context finalization helpers, and targeted incremental test module. Keep snapshot schemas and core lowering unchanged.

Settle static expected owner before extracting products. T5 Level 1 clarification (2026-10-08): repair the static session caller so partial annotation elaboration and initializer checking share the configured initializer CheckingContext. Reject missing inference owners at the static boundary. This caller correction does not extend static collection evaluation/materialization. Export committed maps through a single checked accessor at callable/static boundaries. Ensure incomplete/control outcomes do not become Ready merely because the pending map was discarded. Do not serialize registry state. Ensure contextual explanations do not claim final acceptance for rejected candidates.

Tests: static successful/invalid owner publication; no pending products exported; invalid -> valid -> invalid edits; cold/incremental comparison of canonical formatted family/type/extent and diagnostics; unchanged independent callable products survive. Compare semantic meaning across sessions, not raw TypeId equality.

Use existing `incremental/type_store_revisions.rs` session pattern or a registered focused incremental module. Run only the new parity filter and publication filter.

### T6 — Stabilize and reconcile P3 records

Run G1/G2 below; review scoped diff and formatting. Update the checkpoint with actual architecture, tests, deferred failures, and next action. Update P3 walkthrough/handoff. This closes only inference-owned sequence publication, not all P3 acceptance work.

## 8. Coverage obligations

| ID | Required evidence | Invariants |
|---|---|---|
| CV-01 | Valid canonical/partial/explicit-hole List/Array/Vector products publish | 01,07 |
| CV-02 | Late generic where conflict removes dependent argument product, preserves fixed return knowledge and independent sibling | 01,02,03,07 |
| CV-03 | Child local success is not enough; ancestor rejection wins | 02,03 |
| CV-04 | Underconstrained/ambiguous/blocked/cancelled/budget/internal/materialization failure never commits | 01,02 |
| CV-05 | Deferred child succeeds only after final scalar/row readiness; unresolved child cannot be silently accepted | 01,02,04 |
| CV-06 | Probe isolation and union real-argument retention/aggregation | 03,05 |
| CV-07 | Static extraction after settlement; callable/static artifacts contain no pending products | 04,06 |
| CV-08 | Explanation local facts remain useful without implying final acceptance | 04,07 |
| CV-09 | Independent nested/sibling products remain unchanged by unrelated rejection | 03,07 |
| CV-10 | Invalid/valid revision transitions match cold analysis, with unaffected product reuse | 04,05,06 |

Failure/control mechanics may use registry/owner unit tests to reach states that are difficult to force with small source fixtures. Do not create production-only failure injection or tests that merely assert map implementation details. Assert the externally meaningful publication contract.

## 9. Verification gates and budget

Commands use the pinned toolchain and run serially. Confirm nonzero selected tests.

G1 — publication and directly affected semantic behavior:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generic_inference_proof_integrity
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_row_inference
```

Also run the exact new incremental parity selector and registry unit selector established by T2/T5; record full names/counts in the walkthrough. Do not represent uncreated selectors as executed evidence.

G2 — unchanged consumer integration, once after semantic stabilization:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic -p phalcom-core --all-targets
```

Scoped formatting and diff whitespace review are mandatory. Do not run workspace tests/clippy, release benchmarks, broad corpus, native generation, or metadata suites reflexively: this supplement changes checker publication, not native surfaces or durable schemas. Existing P3-wide gates remain obligations of final P3 closure.

## 10. Failure budget and implementer authority

FIXED: checker owns publication; only accepted owner receipts commit; true dependencies rather than lexical containment; pending state stays out of shared solver handles and durable products; all owner paths in §4 covered; unchanged canonical proof/diagnostic/runtime semantics.

MECHANICALLY FLEXIBLE: private helper/module names, compact container organization, equivalent internal APIs, and test grouping. No new numbered plan/checkpoint is required.

VERIFY-FIRST: T1 reproducer; exact frame/materialization/row APIs; contextual dependency capture through operand composition; all context disposal and static/callable extraction sites; current clone/probe behavior. Follow live code rather than stale line numbers.

Mechanical correction budget: roughly three coherent progress-making cycles. Semantic failure: record Observed/Hypothesis/Evidence/Prediction/Discriminator and permit one serious correction; persistent failure escalates. Architecture/identity/solver-semantics conflict escalates immediately under the shared protocol.

Classify unrelated failures A/B/C/D using repository policy. Do not change test oracles, suppress diagnostics, or broaden into unrelated repairs to obtain a pass.

Final acceptance: every CV obligation has completed evidence; no active owner path silently disposes pending products on valid source; no candidate or gate enters a durable artifact; source/compiler behavior of valid collection programs is preserved. Record this supplement's implementation outcome in the existing P3 checkpoint/walkthrough/handoff; P3 itself remains partial until its other obligations close.
