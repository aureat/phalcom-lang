---
id: LANG006.C1.P3
category: LANG
program: LANG006
checkpoint: LANG006.C1
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
depends_on:
  - LANG006.C1.P2
  - LANG006.C1.P2-walkthrough.md
  - LANG006.C1.P2-handoff.md
follows: LANG006.C1.P2
supersedes: null
prepared: 2026-09-18
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: f7e2ca1585c755e16f7046e5c24d0858dc2c9574
---

# LANG006.C1.P3 — Compiler-Synthetic Authority and Certification Baseline Stabilization

## 0. Executor contract

This plan is designed for GPT-5.6 Luna or another capable implementation model operating under constrained architectural authority.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture, ownership boundaries, and invariants below;
- must not improvise material architecture;
- must not turn this plan into repository-wide semantic cleanup;
- must follow the testing budget rather than testing reflexively;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep `docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md` current;
- must create the required P3 walkthrough and handoff;
- must preserve unrelated dirty/staged/untracked work.

Testing is evidence gathering, not a ritual. Do not run broad suites after each edit.

This plan is intentionally narrower than the full failure inventory. It owns the regressions and false-red certification blockers identified by the post-P2 audit; it does **not** absorb independent semantic-correctness work from LANG005/SEMA006 merely because those failures appear in the same broad test run.

---

## 1. Goal

Restore a truthful `LANG006.C1` certification baseline by fixing the C1 regressions in compiler-generated constructor authority and source-visibility publication, migrating stale tests that conflict with current ratified semantics, repairing the narrowly verified `reflection.selector` generic source contract, and reclassifying the remaining broad-suite failures under their actual owners.

---

## 2. Checkpoint acceptance objective

P3 is a **corrective stabilization and certification-baseline closure** plan inside `LANG006.C1`.

It succeeds when all of the following are true:

1. generated constructor factories invoke `Class.allocate()` through compiler-owned, call-site-local authority;
2. compiler-internal authority remains transient and cannot be minted by source;
3. source visibility stored in the normalized AST survives into published semantic declaration surfaces without legacy spelling/attribute inference;
4. stale enum-behavior, cursor-iteration, raw-associated-projection, and retired private-subscript fixtures no longer encode superseded language behavior;
5. `universe:reflection.selector` no longer publishes an unsaturated `List` result type for `Selector.slots`;
6. the constructor-startup cascade is removed from the broad core suite;
7. one final broad semantic/core run produces a truthful residual failure ledger;
8. residual independent semantic/runtime defects are recorded and handed off rather than opportunistically repaired;
9. all P2-focused access/native/Foreign/reflection/tooling invariants remain green.

P3 does **not** require the entire repository to become green. Checkpoint completion and verification strength remain separate facts.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: f7e2ca1585c755e16f7046e5c24d0858dc2c9574
commit: feat(lang006): implement C1 corrective closure
relevant predecessor: LANG006.C1.P2
```

Verified repository facts at the planning revision:

- `LANG006.C1` is `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED`.
- P2 focused access-control, native-conformance, reflection, metadata, AST, LSP, compile, and Universe bootstrap checks are green.
- the full semantic suite reports `1,420 passed / 22 failed / 50 ignored`;
- the full core suite reports `537 passed / 74 failed / 28 ignored`;
- the largest core failure cluster dies before intended behavior because generated constructors reach internal `Class.allocate()` through ordinary invocation;
- `lower_constructors()` still marks the generated class-side factory with `BuiltinAttr::Constructor`;
- `class_decl.rs` still transfers that marker into compiler state;
- `scope.rs::emit_invoke(..., internal)` still emits `Bytecode::InvokeCompilerInternal`;
- the VM still scopes compiler authority transiently through `compiler_internal_dispatch_depth`;
- the LANG006.P1 declaration-modifier migration removed the old `internal_call` selection from ordinary method-call lowering and now passes `false` to `emit_invoke`;
- the semantic declaration-surface builder reads `ClassMember::visibility()` directly, so the source-visibility failure must be located along the normalized AST → declaration surface → published snapshot path rather than repaired by restoring legacy attribute/prefix inference;
- `reflection/selector.ph` declares `native slots -> List`, while `List` is now a type constructor and the runtime value is a list of `Symbol`;
- `metadata/export.rs` now deliberately exports `TypeData::AssociatedProjection`, so the historical rejection test is obsolete;
- `docs/specs/iterator.md` and LANG005.C6 own the stateful `Iterator` / `Iterable` model; `iteratorValue` no longer determines `for` element typing;
- enum behavior belongs in accepted `impl` forms rather than behavior declarations embedded directly in an enum body.

Before editing:

```sh
git rev-parse HEAD
git status --short
```

Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

If live HEAD is newer than the planning revision, inspect at minimum:

```sh
git diff f7e2ca1585c755e16f7046e5c24d0858dc2c9574..HEAD -- \
  phalcom-ast/src \
  phalcom-semantic/src \
  phalcom-semantic/tests/semantic \
  phalcom-core/src/compiler \
  phalcom-core/src/vm \
  phalcom-core/core/universe/src/reflection/selector.ph \
  phalcom-core/src/primitive/selector.rs \
  phalcom-core/tests/core \
  docs/specs \
  docs/implementation/LANG006/LANG006.C1
```

Do not re-audit unrelated repository areas.

---

## 4. Required reads before implementation

Read in this order:

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P2-corrective-access-provenance-and-implementation-kind-closure.md
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P2-walkthrough.md
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P2-handoff.md

docs/specs/README.md
docs/specs/source-declarations.md
docs/specs/vm/bytecodes.md
docs/specs/vm/vm-execution-model.md
docs/specs/objects/enums.md
docs/specs/objects/impl.md
docs/specs/iterator.md
docs/specs/associated-types.md
docs/decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md

phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/lib/class_decl.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/vm/dispatch.rs

phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/surface.rs
phalcom-semantic/src/dispatch.rs
phalcom-ast/src/ast.rs

phalcom-core/core/universe/src/reflection/selector.ph
phalcom-core/src/primitive/selector.rs

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
```

Then inspect only the exact tests named by this plan.

---

## 5. Normative authority

### 5.1 Current specifications

`docs/specs/README.md` defines specification precedence. Tests, historical plans, implementation details, and audits do not override a current owning specification.

`docs/specs/source-declarations.md` and TDR-0082 own the LANG006 declaration-modifier model:

```text
names identify declarations
visibility is explicit structured metadata
implementation kind is explicit structured metadata
legacy _$ selector semantics are retired
Universe provenance is not ordinary Internal authority
```

`docs/specs/vm/bytecodes.md` and `docs/specs/vm/vm-execution-model.md` own the VM realization of compiler-only invocation authority. `InvokeCompilerInternal` is a trusted compiler-emitted form; its authority is temporary and must not become ambient callee authority.

`docs/specs/objects/enums.md` and `docs/specs/objects/impl.md` own enum behavior placement. A stale test does not authorize reintroducing enum-body behavior syntax.

`docs/specs/iterator.md` owns the canonical stateful iteration model. Legacy `iterate` / `iteratorValue` cursor-style typing authority must not be restored to satisfy historical fixtures.

`docs/specs/associated-types.md` owns associated projection identity/normalization. The current metadata representation is allowed to carry an associated projection; old branch-stage rejection behavior is not normative.

### 5.2 Current implementation authority

The P2 plan/walkthrough/handoff own the accepted implementation architecture for:

- exact Module/Package/Project access provenance;
- lookup-then-authorize dispatch;
- source-derived runtime origin;
- native/source conformance;
- language `ImplementationKind` vs native execution provenance;
- bounded `Foreign`;
- compiler-internal transient authority as a synthetic seam.

P3 must not redesign these.

### 5.3 Historical evidence

The broad failure report and pre-P2/P1 comparisons are evidence for regression classification, not semantic authority.

---

## 6. Takeover state

P3 inherits the following stable architecture:

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Source visibility | `phalcom_common::modifier::Visibility`, AST member `visibility` | AST/source declaration model | no selector/attribute reconstruction |
| Semantic member access | `phalcom-semantic/src/access.rs` | semantic layer | exact identity-based access |
| Runtime source origin | `RuntimeAccessOrigin` on method objects | compiler/runtime projection | source declaration origin, not guessed caller |
| Native origin | source native anchor + module reconciliation | native bridge | source declaration authority |
| Language implementation kind | `phalcom_common::ImplementationKind` | language semantic model | `Phalcom/Native/Foreign` only |
| Native execution provenance | `phalcom_native_meta::ImplementationProvenance` | provider/runtime | never a second language category |
| Compiler-only invocation | `InvokeCompilerInternal` + `compiler_internal_dispatch_depth` | compiler/VM | authority lasts for dispatch only |
| Constructor expansion | `compiler/attributes.rs::lower_constructors` | compiler | public factory allocates then calls hidden initializer |
| Declared class surface | `checker/declaration.rs::build_declared_class_surface` | semantic checker | publishes normalized AST facts |
| Durable associated projection | `metadata/export.rs::TypeNode::AssociatedProjection` | semantic metadata | projection is exportable |

Stable flow for compiler-only authority:

```text
compiler-generated call-site provenance
        ↓
compiler lowering
        ↓
Bytecode::InvokeCompilerInternal
        ↓
VM authorization for this dispatch
        ↓
authority removed before callee/user code executes
```

Stable semantic authority direction:

```text
source syntax
    ↓
normalized AST modifiers
    ↓
canonical semantic declaration surface
    ↓
compiler/runtime/editor projection
```

No consumer may infer visibility from names or retired attributes.

---

## 7. Architecture

P3 has four implementation lanes.

### Lane A — call-site-local synthetic authority

The generated constructor factory is compiler-authored code. Only the allocator send requires compiler-internal authority.

The required architecture is:

```text
source constructor
    ↓ compiler expansion
generated class-side factory
    ├── self.allocate()
    │      call provenance = CompilerInternal
    │      ↓
    │   InvokeCompilerInternal
    │
    ├── instance.<hidden initializer>(...)
    │      call provenance = Ordinary
    │
    └── return instance
```

The authority fact belongs to the **generated call site**, not to:

- selector spelling;
- the whole generated method body;
- the constructed class;
- the Universe project;
- the callee after dispatch.

Preferred implementation: attach a non-source-authored invocation provenance/kind to the synthesized method-call expression or equivalent compiler-local lowering node. The parser always creates `Ordinary`; `lower_constructors()` explicitly creates `CompilerInternal` only for the generated allocator send.

Equivalent local representation is mechanically flexible if and only if it preserves the exact call-site invariant. A range-only side table is not sufficient because synthetic expressions reuse source ranges and range identity is not a stable unique call-site identity.

### Lane B — visibility publication continuity

Visibility must remain one fact through the whole path:

```text
source modifier
    ↓
AST ClassMember.visibility
    ↓
semantic Callable/Field signature + DeclarationSurface
    ↓
snapshot/advisory/compiler consumers
```

P3 must find where `Private` becomes `Public`. It must not reconstruct visibility from:

```text
starts_with("_$")
starts_with("__")
@private / @protected
field-name conventions
native-ness
Universe provenance
```

### Lane C — stale-oracle migration

Tests that encode superseded semantics are changed to test the current language, not used to force production code backward.

The migration set is bounded:

```text
4 enum-body behavior fixtures
2 iteratorValue/cursor-typing fixtures
1 raw associated-projection rejection fixture
1 retired private-subscript syntax fixture
```

Every migrated test must preserve the original semantic intent where that intent is still meaningful.

### Lane D — narrow Universe generic contract correction

`Selector.slots` returns selector slot symbols. Its source contract must be an applied list type, not a bare type constructor:

```phalcom
native slots -> List<Symbol>
```

Native descriptor/type metadata must agree to the maximum precision supported by the current native contract machinery.

This task is included because the defect is fully localized, objectively diagnosed, and directly blocks two core module-linking tests. It does not authorize broader Universe typing cleanup.

---

## 8. Ownership boundaries

### Owns

P3 owns:

- compiler-generated constructor allocator authority;
- exact non-escape of compiler-only authority;
- LANG006 visibility publication continuity;
- migration of the explicitly identified stale test oracles;
- `Selector.slots` applied generic source contract;
- broad-suite reclassification after these corrections;
- checkpoint bookkeeping needed to distinguish C1 correctness from unrelated baselines.

### Does not own

P3 does not own implementation of:

- canonical List/Map subscript semantic application defects;
- flow-sensitive field invalidation after opaque calls;
- general unknown-cause composition;
- impl-query/incremental invalidation architecture;
- semantic product-reuse/recomputation policies;
- stateful spread/iterator error shaping;
- general typing-integration readiness (`Functor.map`);
- new enum grammar;
- legacy cursor iteration restoration;
- new FFI/native loading;
- changes to access-domain semantics;
- changes to VM authorization policy.

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| compiler-only synthetic call | synthesized call-site provenance | compiler emitter | method-wide ambient privilege |
| visibility | normalized AST / semantic declaration | checker/compiler/runtime/LSP | name/attribute inference |
| enum behavior placement | current enum/impl specs | parser/tests | test-defined alternate grammar |
| iteration authority | canonical Iterable/Iterator semantics | checker/compiler/tests | `iteratorValue` fallback |
| associated projection metadata | semantic type metadata | serializer/importer | branch-stage rejection oracle |
| selector slots type | Universe source declaration | semantic/native verification | bare unsaturated `List` |

---

## 9. Global invariants

```text
INV-01 — ordinary source cannot mint compiler-internal invocation authority.

INV-02 — compiler-internal authority is attached to an exact compiler-generated
         dispatch site, not to an entire generated method body.

INV-03 — compiler-internal dispatch authority is removed before callee/user code
         executes and cannot leak through nested calls.

INV-04 — Class.allocate() remains Internal; P3 does not make it public or weaken
         runtime authorization.

INV-05 — source visibility is read from normalized structured declaration data;
         no consumer restores prefix/attribute inference.

INV-06 — inaccessible and missing members remain distinct; P3 does not change
         P2 lookup-then-authorize behavior.

INV-07 — stale tests conform to current specifications; production syntax is not
         expanded solely to satisfy historical fixtures.

INV-08 — enum behavior remains impl-owned.

INV-09 — for/iteration typing remains canonical Iterator/Iterable evidence-driven;
         no legacy cursor fallback returns.

INV-10 — associated projections remain durable metadata forms when supported by
         the current metadata schema.

INV-11 — type constructors are not used as value types where applied arguments
         are semantically required.

INV-12 — P3 does not repair unrelated incremental/type-analysis defects merely
         because broad certification observes them.

INV-13 — P2 access/native/Foreign architecture remains unchanged and green.

INV-14 — broad-suite failure counts are evidence, not acceptance semantics;
         residual failures are classified by cause/owner.
```

All tasks/tests below reference these invariants.

---

## 10. Non-goals

P3 must not:

- make `Class.allocate()` public;
- classify all calls inside a generated constructor factory as compiler-internal;
- restore `_$/__` semantic namespaces;
- restore `@private`, `@protected`, `@internal`, or `@native` as semantic modifiers;
- add enum-body methods;
- reintroduce cursor-based iteration semantics;
- reject associated projection metadata to satisfy an obsolete branch-stage test;
- redesign List/Map canonical call analysis;
- redesign incremental dependency capture;
- alter generic conformance/trait execution;
- “fix” broad suite counts by ignoring/skipping valid failing tests;
- run release-wide verification after every task;
- claim `RELEASE_COMPLETE`.

---

## 11. Expected impact map

### Expected source areas

- `phalcom-core/src/compiler/attributes.rs` — generated constructor call provenance.
- `phalcom-core/src/compiler/lib/expr.rs` — consume exact invocation provenance.
- `phalcom-core/src/compiler/lib/scope.rs` — verify existing `emit_invoke` seam; modify only if a small API refinement is mechanically required.
- `phalcom-ast/src/ast.rs` — only if the selected exact call-site representation is stored on AST call nodes.
- `phalcom-semantic/src/checker/declaration.rs` — visibility publication trace/fix if the defect is here.
- semantic snapshot/query projection path — only the exact overwrite site if visibility is correct in `build_declared_class_surface`.
- `phalcom-core/core/universe/src/reflection/selector.ph` — `slots -> List<Symbol>`.
- `phalcom-core/src/primitive/selector.rs` — native contract string/metadata parity if supported/required.

### Expected tests

- `phalcom-core/tests/core/execution/compiler_authority.rs`
- `phalcom-core/tests/core/compiler/declaration_dispatch.rs`
- `phalcom-core/tests/core/collections/outgoing_packs_completion.rs` only for authority non-mint regression if needed
- `phalcom-semantic/tests/semantic/integration/advisory_analysis.rs`
- `phalcom-semantic/tests/semantic/integration/access_control.rs`
- `phalcom-semantic/tests/semantic/adts/behavior.rs`
- `phalcom-semantic/tests/semantic/capabilities/getters.rs`
- `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`
- the live semantic metadata integration test file containing `metadata_branch_a_rejects_raw_associated_projection`
- the live core module-linking tests containing `comp_03...` and `mat_06...`

### Expected documentation/state

- `docs/implementation/LANG006/LANG006.C1/LANG006.C1-CHECKPOINT.md`
- `docs/implementation/LANG006/LANG006.C1/LANG006.C1.P3-walkthrough.md`
- `docs/implementation/LANG006/LANG006.C1/LANG006.C1.P3-handoff.md`

No normative specification edit is expected. If a current spec is found to conflict with the architecture above, STOP AND CONSULT.

### Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem if the entry changes architecture, ownership, or semantic authority.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. compiler-only authority is call-site-local;
2. `Class.allocate()` remains Internal;
3. runtime transient-authority semantics remain unchanged;
4. visibility originates from normalized declaration metadata;
5. enum behavior remains `impl`-owned;
6. legacy iterator cursor authority remains retired;
7. associated projection export support remains;
8. residual independent semantic defects are deferred, not absorbed;
9. P2 access/native/Foreign architecture remains intact.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- helper/type names for synthetic invocation provenance;
- whether exact provenance is carried by a small AST enum/field or an equivalent compiler-local synthesized node;
- private function decomposition;
- exact test module placement following live test README conventions;
- mechanically changed test filters;
- native descriptor syntax required to express `List<Symbol>`;
- imports and local Rust ownership/borrowing structure.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| generated factory still contains exactly one allocator send | `lower_constructors` | adapt mechanics; STOP if constructor architecture changed |
| `emit_invoke(..., true)` still maps to `InvokeCompilerInternal` | `scope.rs` | STOP if authority mechanism changed |
| VM authority remains transient | `dispatch.rs`, `send.rs` | STOP if semantics changed |
| `ClassMember::visibility()` carries source modifier | parser/AST tests | fix parser storage if mechanical; STOP if modifier model changed |
| declaration surface receives AST visibility | `checker/declaration.rs` | trace next projection/overwrite |
| enum behavior spec still requires `impl` | current specs | STOP if spec changed |
| iterator spec still rejects legacy cursor authority | `docs/specs/iterator.md` | STOP if spec changed |
| metadata schema supports associated projection | `metadata/export.rs` | STOP if live schema regressed |
| `Selector.slots` values are Symbols | primitive implementation | adapt type only if representation changed |

---

## 13. Global STOP / CONSULT triggers

Stop editing and build a consultation packet if any of the following occurs:

1. exact call-site provenance cannot be represented without materially redesigning the AST/compiler IR;
2. the compiler-internal VM authority mechanism no longer has dispatch-only lifetime;
3. generated constructor allocation now depends on more than one privileged send or on ambient privilege;
4. `allocate()` must be made less restrictive to make constructors work;
5. the visibility failure requires resurrecting name/attribute inference;
6. the AST correctly stores visibility and the surface builder correctly publishes it, but a second canonical surface owner overwrites it;
7. a stale fixture cannot be expressed using current accepted semantics;
8. current specs conflict materially with the stale-test migrations;
9. `List<Symbol>` cannot be represented by the current semantic/native contract machinery without a broader generic-native redesign;
10. any P2-focused access/native/Foreign test starts failing for architectural reasons;
11. the same nontrivial semantic failure persists after one serious corrective attempt;
12. an independent incremental/type-analysis failure appears necessary to fix in order to implement T1–T4;
13. an invariant must be weakened to pass tests;
14. scope becomes materially larger than this plan.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

A cycle is:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

### Semantic failures

Before changing production code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

### Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless something relevant changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| P3-CV-01 | canonical success | INV-01..04 | generated constructor allocates and runs initializer |
| P3-CV-02 | authority identity | INV-02 | generated allocator dispatch emits compiler-internal form |
| P3-CV-03 | negative/source | INV-01,04 | ordinary source cannot invoke internal allocator with compiler authority |
| P3-CV-04 | non-escape | INV-03 | nested ordinary call from generated factory does not inherit authority |
| P3-CV-05 | visibility continuity | INV-05,06 | private/protected/public source members retain exact surface visibility |
| P3-CV-06 | field visibility | INV-05 | explicit private and default-public fields publish correctly |
| P3-CV-07 | enum migration | INV-07,08 | same behavior semantics expressed through accepted impl syntax |
| P3-CV-08 | iteration migration | INV-07,09 | fixtures use canonical Iterator/Iterable evidence, not iteratorValue |
| P3-CV-09 | metadata migration | INV-07,10 | associated projection exports/round-trips/fingerprints rather than rejects |
| P3-CV-10 | modifier syntax migration | INV-05,07 | private subscript fixture uses current modifier syntax |
| P3-CV-11 | generic source contract | INV-11 | `Selector.slots` is `List<Symbol>` |
| P3-CV-12 | module integration | INV-11 | two previously blocked module-linking tests compile/run |
| P3-CV-13 | cascade collapse | INV-01..04 | constructor-dependent core clusters no longer die at allocate |
| P3-CV-14 | predecessor parity | INV-13 | P2 focused access/native/Foreign gates remain green |
| P3-CV-15 | residual classification | INV-12,14 | every remaining broad failure has owner/classification |

Selected dimensions: canonical success, negative behavior, authority lifetime, identity, diagnostics, stale-oracle migration, cross-layer integration, and baseline classification.

No performance benchmark, GC stress, or workspace release matrix is required by P3.

---

## 16. Verification execution budget

### Modes

#### BUILD MODE

Sparse, discriminating feedback only.

#### STABILIZE MODE

Focused regressions and directly affected suites.

#### CERTIFY MODE

One broad semantic/core classification pass, plus P2 focused parity. No automatic workspace certification.

### Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

### Mandatory during BUILD

- exact generated-constructor authority regression;
- exact visibility publication regression;
- exact stale fixture being migrated;
- exact selector/module-linking regression after source-contract edit;
- `cargo check` for a crate only when changed Rust code makes it informative.

### Mandatory during STABILIZE

- constructor authority focused group;
- semantic visibility/access focused group;
- migrated stale-fixture filters;
- module-linking two-test filter;
- P2 focused access/native/Foreign parity.

### Mandatory during CERTIFY

Exactly once after T1–T4 are accepted:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
git diff --check
```

### Only if evidence demands

- `cargo test -p phalcom-lsp --lib`
- native package full suites
- `cargo check --workspace --all-targets`

### Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
all language corpus tests
full semantic suite after each semantic test migration
full core suite after constructor authority alone
```

Reason: these runs do not answer the local correctness question and would recreate the P2 over-testing problem.

---

## 17. Baseline / unrelated failure policy

Classify:

```text
A — definitely caused by LANG006/P1/P2 transition
B — probably caused by LANG006 transition
C — unclear
D — clearly unrelated/baseline to LANG006
S — stale oracle/fixture conflicting with current authority
```

Policy:

- A/B: active P3 responsibility.
- S: migrate the test to current authority; do not change semantics backward.
- C: one bounded classification pass; if nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks P3 acceptance.
- Do not weaken/skip assertions to make broad gates green.

### Initial audited classification

| Cluster | Initial class | P3 action |
|---|---|---|
| generated `Class.allocate()` authority | A | fix T1 |
| source visibility publication | A | fix T2 |
| enum body behavior fixtures | S | migrate T3 |
| legacy `iteratorValue` fixtures | S | migrate T3 |
| raw projection rejection fixture | S | migrate T3 |
| retired private-subscript syntax | S | migrate T3 |
| `Selector.slots -> List` unsaturated type | D but bounded blocker | fix T4 under explicit P3 scope |
| flow refinement after opaque call | D | hand off |
| List/Map canonical subscript semantics | D | hand off |
| unknown-cause preservation | D | hand off |
| impl-query invalidation | D | hand off |
| seven incremental/product-reuse failures | D | hand off |
| spread/iterator DNU shaping | D | hand off |
| typing integration/Functor.map readiness | D | hand off |

Record final classification in the checkpoint ledger after G5.

---

# 18. Tasks

## T0 — Freeze the P3 baseline and activate the checkpoint plan

### Purpose

Prevent implementation from operating against a vague or moving failure baseline.

### Preconditions

P2 walkthrough/handoff consumed.

### Consumes

- audited revision and failure report;
- P2 checkpoint state;
- current git state.

### Produces

- current revision record;
- exact initial P3 focused baseline;
- updated checkpoint `active_plan`.

### Files and symbols

**Read:**
- `LANG006.C1-CHECKPOINT.md`
- P2 plan/walkthrough/handoff
- exact test files named below

**Modify:**
- `LANG006.C1-CHECKPOINT.md` only for active-plan/current-state bookkeeping.

### Required implementation shape

1. Record `git rev-parse HEAD` and `git status --short`.
2. If HEAD differs, perform the bounded diff from §3.
3. Confirm the exact failing tests still exist.
4. Reconcile P2 lifecycle truth before activating P3. The P2 walkthrough already establishes that the corrective vertical is implemented and focused-tested while broad certification is baseline-blocked. Update the P2/checkpoint lifecycle metadata to the truthful completed-implementation state:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
```

Use `FOCUSED_TESTED` instead of `BASELINE_BLOCKED` only if the current repository lifecycle convention explicitly permits a completed focused implementation to retain that verification value despite the named broad gate being blocked. Do not claim `RELEASE_COMPLETE`, and do not rewrite P2's historical walkthrough content.

5. Then update the checkpoint plan pointers:

```yaml
active_plan: LANG006.C1.P3
latest_completed_plan: LANG006.C1.P2
next_plan: null
```

P3 inherits the remaining corrective/certification-baseline work; it does not retroactively redefine P2.

### Tests to run now

Run only:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  test_subscript_getter_and_setter_do_not_trigger_duplicate_selector -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  compiler_surfaces_publish_source_member_visibility -- --nocapture
```

If filters select zero tests, inspect `-- --list` and correct the filter.

### Tests explicitly deferred

Everything else.

### Acceptance

P2 lifecycle metadata is reconciled truthfully, P3 is the active plan, and both root regressions are reproduced or repository drift is concretely classified.

### Local STOP / CONSULT triggers

Stop if either root regression already disappeared due to newer architectural changes and the audit assumptions no longer apply.

### Checkpoint update

Record P3 activation and current baseline revision.

---

## T1 — Restore exact compiler-generated allocator authority

### Purpose

Repair the constructor-startup regression without reintroducing selector-spelling privilege or ambient method-wide compiler authority.

### Preconditions

T0 complete.

### Consumes

- `lower_constructors()` synthetic factory;
- `emit_invoke(..., internal)`;
- `InvokeCompilerInternal`;
- transient VM authorization.

### Produces

An exact compiler-generated allocator call marked for compiler-internal dispatch.

### Files and symbols

**Read:**
- `phalcom-core/src/compiler/attributes.rs::lower_constructors`
- `phalcom-core/src/compiler/lib/class_decl.rs`
- `phalcom-core/src/compiler/lib/expr.rs`
- `phalcom-core/src/compiler/lib/scope.rs::emit_invoke`
- `phalcom-core/src/vm/dispatch.rs::invoke_compiler_internal_at`
- `phalcom-core/src/vm/send.rs::authorize_method_access_as`
- `phalcom-core/tests/core/execution/compiler_authority.rs`

**Modify:**
- `attributes.rs`
- `expr.rs`
- exact call-provenance representation owner
- tests

### Required implementation shape

Introduce an explicit non-source-authored invocation provenance equivalent to:

```rust
enum InvocationAuthority {
    Ordinary,
    CompilerInternal,
}
```

The exact name/location is flexible.

Requirements:

1. parser-created calls are always `Ordinary`;
2. `lower_constructors()` marks **only** the synthesized `self.allocate()` call as `CompilerInternal`;
3. generated hidden-initializer calls remain `Ordinary`;
4. ordinary method-call lowering passes the exact provenance to `emit_invoke`;
5. `emit_invoke` continues to select `InvokeCompilerInternal` only for exact marked calls;
6. no call is classified by selector prefix;
7. compiler authority remains VM-transient.

If the AST is the least disruptive carrier, a small call-node field is acceptable. If a compiler-local synthesized-call wrapper already exists or can be used without creating a parallel IR, prefer it.

Do **not** use source range as sole call identity.

### Forbidden approaches

- `self.emit_invoke(..., self.compiler_internal)` for every call in the generated factory;
- `method == "allocate"` as the sole authority proof;
- making `allocate()` public/module/package/internal-to-project for constructors;
- skipping runtime authorization for `Class.allocate`;
- restoring `_$/starts_with` privilege;
- widening `compiler_internal_dispatch_depth` across callee execution;
- giving every compiler-generated method blanket Internal access.

### Test changes required

Add/retain tests for P3-CV-01..04.

At minimum prove:

```text
generated constructor factory succeeds
generated allocator dispatch has compiler-internal bytecode/provenance
ordinary source cannot mint that authority
authority does not leak into nested ordinary calls
```

Prefer one compiler-level bytecode/disassembly assertion plus runtime behavior tests.

A useful negative fixture is a generated constructor factory whose subsequent ordinary call targets an Internal member that must still be denied unless ordinary lexical/project authority independently permits it.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  compiler_internal_dispatch_authority_is_transient -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  ordinary_source_cannot_mint_compiler_internal_pack_authority -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  test_subscript_getter_and_setter_do_not_trigger_duplicate_selector -- --nocapture
```

Also run the new exact bytecode/provenance regression.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

### Tests explicitly deferred

Do not run all 74 former core failures here.

### Acceptance

- constructor fixture reaches its intended body;
- bytecode/provenance proves allocator is the privileged site;
- source cannot mint the capability;
- transient-authority test remains green.

### Local STOP / CONSULT triggers

Stop if exact call-site provenance requires a new compiler IR architecture or if more than the allocator dispatch requires synthetic Internal authority.

### Checkpoint update

Record the durable call-site-local compiler-authority invariant.

---

## T2 — Repair normalized source visibility publication

### Purpose

Restore the C1 invariant that source modifier metadata survives unchanged into semantic declaration surfaces.

### Preconditions

T0 complete. T1 may proceed independently, but do not run broad suites until both are done.

### Consumes

- parser modifier result;
- AST member `visibility`;
- declared semantic surface publication.

### Produces

Exact Private/Protected/Public/Module/Package/Internal values in compiler/semantic surfaces.

### Files and symbols

**Read first:**
- `phalcom-ast/src/ast.rs::ClassMember::visibility`
- parser member/modifier construction
- `phalcom-semantic/src/checker/declaration.rs::member_visibility`
- `build_declared_class_surface`
- `DeclarationSurface::add_*_with_visibility`
- snapshot/query publication that reconstructs surfaces
- existing access-control and advisory tests

**Modify:**
Only the exact ownership point where visibility is lost.

### Required implementation shape

Use a three-discriminator trace before editing:

1. parser/AST assertion:
   ```text
   private hide()      => Visibility::Private
   protected guarded() => Visibility::Protected
   open()              => Visibility::Public
   private _secret     => Visibility::Private
   _field              => Visibility::Public
   ```
2. direct `build_declared_class_surface` assertion;
3. final `SemanticWorkspaceSession` snapshot surface assertion.

The first stage that differs from expected owns the bug.

If AST is wrong, repair parser normalization.
If AST is right and direct surface is wrong, repair declaration publication.
If both are right and snapshot is wrong, repair only the later canonical projection/overwrite.

### Forbidden approaches

- reintroducing `@private`/`@protected` interpretation;
- treating all fields as private by default;
- selector/field prefix inference;
- patching only advisory output while canonical surface remains wrong;
- adding a second visibility cache/source of truth.

### Test changes required

P3-CV-05 and P3-CV-06.

Retain and strengthen:

```text
compiler_surfaces_publish_source_member_visibility
integration::access_control field/callable visibility assertions
```

If missing, add one direct AST/parser modifier-storage test so future parser drift is isolated from semantic publication.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  compiler_surfaces_publish_source_member_visibility -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  integration::access_control -- --nocapture
```

Run parser/AST exact filter if T2 modifies parser/AST.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Tests explicitly deferred

Full semantic suite.

### Acceptance

Published source surfaces match normalized modifiers exactly and no legacy inference returns.

### Local STOP / CONSULT triggers

Stop if two independently produced declaration surfaces compete as canonical owners or if fixing visibility requires changing P2 access semantics.

### Checkpoint update

Record the exact corrected publication seam.

---

## T3 — Migrate obsolete certification fixtures to current semantics

### Purpose

Remove false-red tests without expanding production semantics backward.

### Preconditions

Current specs re-read.

### Consumes

- enum/impl specs;
- iterator spec;
- associated-projection metadata support;
- current LANG006 modifier syntax.

### Produces

Current-language tests preserving valid original semantic intent.

### Files and symbols

**Tests to migrate:**

```text
semantic::adts::behavior::
  adt_beh_01_enum_wide_method_is_published_as_root_default_and_requirement
  adt_beh_02_variant_override_has_exact_case_dispatch_target
  adt_beh_03_case_only_method_is_not_promised_by_root_type

semantic::capabilities::getters::
  enum_generic_getter_uses_canonical_application_path

semantic::foundations::canonical_call_application::
  zero_argument_iteration_getter_uses_callable_application
  parameterized_iteration_protocol_fails_closed_without_cursor

semantic::integration::metadata::
  metadata_branch_a_rejects_raw_associated_projection

compiler_declaration_dispatch::
  private_subscript_enforces_visibility
```

### Required implementation shape

#### T3.A — enum behavior

Rewrite enum behavior fixtures using the current accepted inherent/case `impl` syntax.

Preserve the tested semantic questions:

- root/default behavior publication;
- exact-case override dispatch target;
- case-only behavior not promised by the root type;
- generic getter/callable application identity.

Do not add parser support for enum-body methods.

If the old test name materially encodes the old syntax rather than the semantic assertion, rename it to describe current behavior.

#### T3.B — iteration

The two `iteratorValue` fixtures are not valid authorities after LANG005.C6.P5.

Replace them with canonical `Iterator` / `Iterable` scenarios.

For the positive case, prove that canonical iteration evidence determines the element type through associated `Item` and selected iterator/next application.

For the negative case, prove fail-closed behavior when the receiver lacks required canonical iterable evidence. Do not test “parameterized iteratorValue without cursor”.

If these assertions now belong under `semantic::capabilities::iteration`, move them rather than keeping misleading `canonical_call_application` names solely for historical continuity.

#### T3.C — associated projection metadata

Replace rejection expectation with durable metadata behavior.

At minimum assert:

```text
export succeeds
associated projection node survives
subject / trait application / requirement identity survive
structural fingerprint is deterministic
```

Add import/round-trip coverage if the current metadata test helpers already support it.

Do not weaken projection representation back to the old branch-stage rule.

#### T3.D — private subscript

Rewrite the fixture using current explicit visibility syntax around the subscript declaration. Preserve the original access-control assertion.

Do not restore retired attribute/private syntax.

### Forbidden approaches

- parser changes to accept stale syntax;
- deleting tests without equivalent current-semantic coverage;
- converting stale tests to `#[ignore]`;
- weakening assertions because the new architecture is harder to fixture;
- testing legacy cursor semantics under a new name.

### Tests to run now

Run each migrated filter individually first.

Then one coherent semantic group:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  adts::behavior -- --nocapture
```

Run the exact getter/iteration/metadata filters, and:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  private_subscript_enforces_visibility -- --nocapture
```

### Tests explicitly deferred

Do not run full semantic/core suites until G5.

### Acceptance

All eight stale fixtures either:
- pass with current syntax/semantics and preserve their valid intent; or
- are replaced by clearly named current-semantic equivalents.

No production semantic change is made solely for them.

### Local STOP / CONSULT triggers

Stop if current accepted specs cannot express the original valid semantic assertion or if a purported stale test actually exposes a current-spec implementation defect.

### Checkpoint update

Record only the stale-oracle reclassification, not every test edit.

---

## T4 — Correct `Selector.slots` to an applied generic result contract

### Purpose

Remove the localized Universe semantic error that blocks two module-linking tests.

### Preconditions

List is still a generic type constructor and selector slots still produce Symbols.

### Consumes

- reflection selector source shell;
- native selector primitive descriptor;
- current native/source conformance machinery.

### Produces

An applied result type for selector slots.

### Files and symbols

**Modify:**
- `phalcom-core/core/universe/src/reflection/selector.ph`
- `phalcom-core/src/primitive/selector.rs` only as required for descriptor/type-contract parity
- semantic diagnostics baseline only if the corresponding error legitimately disappears and baseline files are meant to track current expected diagnostics

### Required implementation shape

Source:

```phalcom
native slots -> List<Symbol>
```

Native contract should express the same semantic result to the precision supported by the current descriptor system.

Preferred type string where supported:

```text
() -> List<Symbol>
```

The runtime implementation remains unchanged: it returns a List of Symbol values.

Do not generalize this task into a sweep of every bare generic Universe annotation. If another annotation fails as a consequence, classify it separately.

### Forbidden approaches

- reverting List to a non-generic semantic type;
- replacing result with `Dynamic` to suppress diagnostics;
- skipping semantic checking for native source;
- weakening native/source conformance;
- editing module-linking logic to ignore dependency diagnostics.

### Test changes required

P3-CV-11 and P3-CV-12.

Run:

```text
modules_linking::comp_03_module_import_uses_last_segment
modules_linking::mat_06_module_import_binding_resolves_to_module
```

Also run the focused native conformance test that covers selector source descriptors if available.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  comp_03_module_import_uses_last_segment -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  mat_06_module_import_binding_resolves_to_module -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  integration::native_conformance -- --nocapture
```

### Tests explicitly deferred

No full Universe/core suite yet.

### Acceptance

The unsaturated-constructor diagnostic disappears for `reflection.selector`, module-linking tests progress past semantic dependency compilation, and native conformance remains exact.

### Local STOP / CONSULT triggers

Stop if the native descriptor system cannot express the applied generic result without a material redesign.

### Checkpoint update

Record this as a bounded baseline-correction consumed by P3 certification, not as a new LANG006 language rule.

---

## T5 — Run focused stabilization and predecessor-parity gates

### Purpose

Prove P3 corrections coherently before any broad suite.

### Preconditions

T1–T4 accepted.

### Consumes

All changed production/test paths.

### Produces

Focused P3 evidence and P2 non-regression evidence.

### Required implementation shape

Run G1–G4 below serially.

Failures:

- A/B: fix under current task ownership;
- S: revisit stale migration;
- D: do not repair here;
- architectural conflict: consult.

### Acceptance

All P3 focused coverage passes and P2 architecture remains green.

### Checkpoint update

Record coherent gate results, not individual command chatter.

---

## T6 — Perform one broad reclassification pass and build owner handoffs

### Purpose

Measure the post-fix repository truth and separate constructor cascades from independent defects.

### Preconditions

G1–G4 pass.

### Consumes

The original 22 semantic / 74 core baseline and all P3 corrections.

### Produces

- post-P3 suite totals;
- per-failure A/B/C/D/S classification;
- owner handoff ledger.

### Required implementation shape

Run exactly once:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
```

Compare against the original baseline.

The constructor-dependent clusters are expected to collapse substantially. If a constructor-dependent test still reports `allocate()` access denial, P3 has failed and must not proceed to completion.

### Residual failures that are **not** P3 implementation work

Unless new evidence connects them to P3, carry these to their owners:

#### Semantic flow/evidence

```text
opaque_call_drops_refined_field_current_without_rewriting_contract
unknown_expansions_do_not_disappear
```

Owner: semantic evidence/flow stabilization (SEMA).

#### Canonical subscript application

```text
assumed_list_receiver_caps_structural_index_result
list_subscript_checks_index_contract
list_subscript_set_checks_index_and_value_and_returns_unit
map_subscript_checks_key_contract
```

Owner: canonical semantic callable/subscript application (SEMA).

#### Incremental/reuse

```text
adding_impl_member_does_not_invalidate_another_target_or_its_caller
dependency_edit_remove_readd_recomputes_affected_summary_deterministically
canonical_universe_reads_track_canonical_query_dependencies
legacy_universe_core_path_is_not_a_dependency_sentinel
signature_edit_recomputes_exact_callers_and_reuses_unrelated_bodies
superclass_edit_recomputes_hierarchy_consumers_without_touching_unrelated_bodies
a6_high_fanout_unrelated_declaration_and_hierarchy_retain_existing_products
unused_public_export_does_not_recompute_unrelated_consumer_body
```

Owner: semantic incremental database/product-stability work (SEMA006 or current canonical successor).

#### Runtime spread error shaping

```text
positional_spread_of_non_iterable_has_structured_boundary
test_labeled_spread_traceback_includes_spread_site
test_list_spread_traceback_includes_spread_site
```

Only those still failing for iterator/DNU boundary reasons after constructor repair.

Owner: argument expansion / canonical iteration integration, not LANG006.

#### Typing integration

```text
traverse_body_resolves_symbolic_inherited_capabilities
```

Owner: typing integration/capability analysis.

Do not create implementation fixes for these inside P3.

### Tests to run now

The two broad suite commands only.

Then:

```sh
git diff --check
```

### Tests explicitly deferred

```text
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
```

### Acceptance

- no constructor authority cascade remains;
- residual failures are classified with concrete owner and cause;
- no residual A/B failure remains unresolved.

### Local STOP / CONSULT triggers

Stop if post-P3 broad results show a new regression in P2 access/native/Foreign behavior or if constructor authority failure persists in any ordinary generated constructor.

### Checkpoint update

Record final residual-failure ledger and revised totals.

---

## T7 — Finalize checkpoint records, walkthrough, and handoff

### Purpose

Make the durable repository state match the actual implementation and evidence.

### Preconditions

T1–T6 accepted; no unresolved A/B failure.

### Consumes

Focused and broad evidence, final diff, residual ledger.

### Produces

- updated C1 checkpoint;
- P3 walkthrough;
- P3 handoff.

### Required implementation shape

Update `LANG006.C1-CHECKPOINT.md` with:

- P3 plan ledger entry;
- durable call-site synthetic-authority invariant;
- corrected source-visibility publication seam;
- stale-test migrations;
- selector generic-contract correction;
- post-P3 broad suite totals;
- deferred D failures and owners;
- final verification state.

Completion truth should be selected honestly.

If all LANG006 requirements and focused P2/P3 gates pass but unrelated broad failures remain:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
active_plan: null
latest_completed_plan: LANG006.C1.P3
next_plan: null
```

If repository convention instead requires checkpoint `status: IN_PROGRESS` while verification is baseline-blocked, preserve that convention and explain why in the walkthrough. Do not fabricate `FOCUSED_TESTED` or `RELEASE_COMPLETE`.

### Forbidden approaches

- claiming all broad failures are fixed;
- changing D failures to ignored;
- rewriting historical P1/P2 walkthroughs;
- declaring release certification without release gates.

### Acceptance

Durable records make it possible for a new session to understand:
- what P3 fixed;
- which original failures were stale;
- which constructor cascades disappeared;
- which independent defects remain and who owns them.

---

# 19. Verification gates

## G1 — Compiler-synthetic authority gate

Purpose: prove P3's most important architectural correction.

Run exact live equivalents:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  compiler_internal_dispatch_authority_is_transient -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  ordinary_source_cannot_mint_compiler_internal_pack_authority -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  test_subscript_getter_and_setter_do_not_trigger_duplicate_selector -- --nocapture
```

Plus the new exact generated-bytecode/provenance regression.

Expected:

```text
generated allocator is privileged
ordinary source is not
nested user call does not inherit privilege
constructor reaches initializer
```

Do not broaden after PASS.

---

## G2 — Source visibility gate

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  compiler_surfaces_publish_source_member_visibility -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  integration::access_control -- --nocapture
```

Expected exact Private/Protected/Public/Module/Package/Internal presentation and unchanged access semantics.

---

## G3 — Stale-oracle and generic-contract gate

Run all migrated stale filters individually, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  private_subscript_enforces_visibility -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  comp_03_module_import_uses_last_segment -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  mat_06_module_import_binding_resolves_to_module -- --nocapture
```

Expected:
- no parser support added for obsolete enum/private syntax;
- no legacy cursor authority;
- projection metadata test validates support rather than rejection;
- selector module compiles with applied List type.

---

## G4 — P2 predecessor parity

Run the focused P2 surfaces most likely to catch accidental architecture regression:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  integration::access_control -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  integration::native_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  reflection_conformance -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  compiler_internal_dispatch_authority_is_transient -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Use live equivalent filters for Foreign/presentation only if T1–T4 touched those paths or evidence creates a concern.

Expected: all predecessor invariants remain green.

---

## G5 — Broad certification-baseline reclassification

Run once:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core
git diff --check
```

Expected:

- constructor `allocate()` authorization error absent from all ordinary constructor-based fixtures;
- four enum syntax false-reds gone/replaced;
- two legacy iterator semantic false-reds gone/replaced;
- raw associated-projection rejection false-red gone/replaced;
- source visibility regression green;
- selector unsaturated-constructor module-linking failures green;
- remaining failures map to the D-owner ledger.

G5 does not require zero failures to establish P3 correctness. It requires zero unresolved A/B failures and a credible owner classification for the rest.

---

## 20. Final focused acceptance

At completion, the walkthrough must map every coverage obligation:

| Coverage ID | Test/evidence | Required result |
|---|---|---|
| P3-CV-01 | generated constructor execution | PASS |
| P3-CV-02 | bytecode/provenance regression | PASS |
| P3-CV-03 | source non-mint test | PASS |
| P3-CV-04 | nested non-escape test | PASS |
| P3-CV-05 | compiler surface visibility | PASS |
| P3-CV-06 | access-control visibility fixtures | PASS |
| P3-CV-07 | migrated enum behavior | PASS |
| P3-CV-08 | migrated iteration semantics | PASS |
| P3-CV-09 | projection metadata support | PASS |
| P3-CV-10 | current private subscript syntax | PASS |
| P3-CV-11 | Selector slots type contract | PASS |
| P3-CV-12 | two module-linking tests | PASS |
| P3-CV-13 | broad core constructor cascade | ABSENT |
| P3-CV-14 | P2 focused parity | PASS |
| P3-CV-15 | residual owner ledger | COMPLETE |

Run only the plan's named stabilization gates.

Do not automatically run workspace certification.

---

## 21. Performance/resource evidence

P3 has no performance objective.

Negative constraints:

- no new per-call heap allocation solely for authority;
- no runtime selector-string inspection to discover compiler privilege;
- no runtime widening of authority scope;
- no new per-instance visibility metadata;
- no repeated semantic reconstruction during dispatch.

A compile-time enum/flag on a synthesized call site is acceptable.

No benchmark is required.

---

## 22. Checkpoint bookkeeping

### At plan start

- reconcile P2 to a truthful completed-implementation state while preserving its blocked broad-verification status;
- then mark P3 active and `latest_completed_plan: LANG006.C1.P2`;
- record starting revision;
- record the audited 22/74 broad baseline.

### During plan

Update only for:

- durable synthetic-authority invariant;
- corrected visibility publication owner;
- consultation/amendment;
- stale-oracle reclassification;
- coherent G1–G4 gate;
- materially changed deferred failure classification.

### At plan completion

- update plan ledger;
- record P3 completion;
- record final verification strength;
- record post-P3 semantic/core totals;
- record deferred owner ledger;
- set next action;
- create walkthrough/handoff.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P3-walkthrough.md
```

Must contain:

- final result;
- starting revision and failure counts;
- exact generated-constructor root cause;
- exact call-site provenance architecture implemented;
- proof that authority does not leak;
- source visibility root cause and corrected owner;
- stale tests migrated and why each was stale;
- `Selector.slots` generic contract correction;
- important code changes;
- deviations;
- consultations;
- P3 coverage table with exact commands/results;
- final broad semantic/core totals;
- remaining D failures by owner;
- verification classification;
- residual risks.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG006/LANG006.C1/LANG006.C1.P3-handoff.md
```

Must contain:

- current revision/state;
- inherited P2 stable interfaces;
- P3 synthetic call-provenance seam;
- P3 visibility publication seam;
- stale-oracle migrations;
- selector generic contract;
- invariants;
- remaining SEMA/LANG005/COLL failures;
- first recommended commands for the next owner;
- things not to redesign/re-explore;
- whether LANG006.C1 has implementation work remaining.

The handoff should explicitly direct independent semantic failures to the current canonical SEMA stabilization plan/checkpoint rather than inventing a new owner.

---

## 25. Completion truth table

Do not conflate:

```text
source written
tests migrated
targeted tests passed
P2 parity passed
broad suite reclassified
checkpoint implemented
release certified
```

Truth examples:

```text
T1/T2 implemented, exact tests green
    => PARTIAL / FOCUSED evidence only

T1–T4 implemented + G1–G4 green
    => IMPLEMENTED locally, broad baseline not yet reclassified

G5 run + no A/B residual + D ledger complete
    => P3 COMPLETE

LANG006 behavior complete but broad unrelated failures remain
    => checkpoint may be COMPLETE / IMPLEMENTED / BASELINE_BLOCKED

workspace/release gates not run
    => never RELEASE_COMPLETE
```

---

## 26. Plan self-review

### Architecture

- [x] compiler authority has one exact owner: synthesized call provenance;
- [x] VM transient authority is consumed, not redesigned;
- [x] source visibility has one owner: normalized declaration metadata;
- [x] stale tests cannot override current specs;
- [x] selector generic result is source-contract-owned;
- [x] unrelated semantic defects remain with their owning program.

### Luna executability

- [x] dependency-ordered tasks;
- [x] FIXED/FLEXIBLE/VERIFY-FIRST explicit;
- [x] objective STOP/CONSULT triggers;
- [x] semantic debugging bounded;
- [x] exact test intent;
- [x] broad tests explicitly deferred until G5.

### Testing

- [x] constructor success/negative/lifetime covered;
- [x] visibility continuity covered at multiple layers;
- [x] stale syntax/semantics migrations covered;
- [x] module-linking generic contract covered;
- [x] P2 parity gate included;
- [x] one broad post-fix reclassification pass included;
- [x] residual owner ledger required.

### Documentation lifecycle

- [x] checkpoint start/update/completion rules included;
- [x] walkthrough required;
- [x] handoff required;
- [x] P1/P2 records remain historical.

### Premium compute

- [x] no adviser needed for routine patching;
- [x] call-site-provenance architectural fork pre-decided;
- [x] visibility competing-owner condition triggers consultation;
- [x] generic native contract redesign triggers consultation;
- [x] unrelated SEMA work does not consume LANG006 implementation budget.

---

# Final implementation order

```text
T0  freeze baseline + activate P3
 ↓
T1  exact compiler-generated allocator authority
 ↓
T2  source visibility publication continuity
 ↓
T3  stale-oracle migration
 ↓
T4  Selector.slots applied generic contract
 ↓
T5 / G1–G4 focused stabilization + P2 parity
 ↓
T6 / G5 one broad reclassification pass
 ↓
T7 records + owner handoffs + checkpoint truth
```

T1 and T2 may be implemented in either order after T0, but do not run G5 until T1–T4 are complete.

The plan is intentionally complete without implementing the independent SEMA/incremental/spread/typing defects. Those defects remain important; they are simply not `LANG006.C1` authority and must not be silently absorbed into this checkpoint.
