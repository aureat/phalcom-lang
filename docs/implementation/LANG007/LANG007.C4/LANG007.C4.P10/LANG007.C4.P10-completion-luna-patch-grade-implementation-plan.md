---
id: LANG007.C4.P10
category: LANG
program: LANG007
checkpoint: LANG007.C4
plan: LANG007.C4.P10
kind: implementation-plan
title: "P10 Completion — Explicit Semantic While and Certification Closure"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
repository: aureat/phalcom-lang
branch: codex/lang007-c4-p10
prepared_against_revision: 6e5287af46e7bde7a13fb2cecd37a10f42d6ee80
prepared: 2026-10-11
continues:
  - LANG007.C4.P10-parallelized-implementation-plan.md
inherits_complete:
  - LANG007.C4.P10.T1
  - LANG007.C4.P10.T2
  - LANG007.C4.P10.T3
  - LANG007.C4.P10.T4
  - LANG007.C4.P10.T5
  - LANG007.C4.P10.S1
  - LANG007.C4.P10.T7
  - LANG007.C4.P10.T8
  - LANG007.C4.P10.T9.pack
  - LANG007.C4.P10.T9.non_loop_evaluator
remaining:
  - LANG007.C4.P10.R2
  - LANG007.C4.P10.R3
  - LANG007.C4.P10.R6
  - LANG007.C4.P10.R7
blocks:
  - LANG007.C4.P6
handoff_to:
  - LANG007.C4.P5
  - LANG007.C4.P6
  - LANG007.C4.P9
supersedes: null
---

# LANG007.C4.P10 — P10 Completion: Explicit Semantic `while` and Certification Closure

## Luna patch-grade implementation plan

---

# 0. Executor contract

This is a **completion plan**, not a replay of the original P10 foundation plan.

The original `LANG007.C4.P10-parallelized-implementation-plan.md` was prepared against `419ec6e66ce90cba6003b7627eb4c99fa2345a7a` and has already been substantially executed. Current `main` contains the P10 structural-classification foundation, `EvalControl`, static `return`, static `if`, the `break`/`continue` carrier, P1–P3 convergence work, the P10 ontology census, and the Pack/non-loop static-evaluator resource harnesses.

This plan owns only the **remaining P10 closure**:

```text
explicit semantic While identity
        +
ordinary semantic/checker/CFG convergence
        +
runtime compiler lowering
        +
bounded static While execution
        +
while-specific resource evidence
        +
current-main recertification
        +
P10 lifecycle closure
```

The implementer:

- **must execute on `main`**;
- must begin from a clean, synchronized `main` at or after the audited revision;
- must re-read every live source file named by the active task before editing it;
- may adapt helper names, private decomposition, and line locations to mechanical repository drift;
- must preserve the architectural ownership, semantic identities, and control-flow invariants defined here;
- must preserve all already-landed P10 T1–T5 behavior rather than reimplementing it;
- must preserve P3 Pack representation and fixed-call/no-Pack invariants;
- must preserve P4 `FieldKey`/Record-row ownership;
- must preserve P5 ownership of compiler-derived structural capabilities, structural reflection values, type-member result domains, and domain-neutral structural member projections;
- must preserve P6 ownership of static `match`, structural `for`, structural patterns, and proof predicates;
- must preserve the generic-conformance, associated-member, trait-dispatch, contextual-realization, and compiler call-environment architecture now present on `main`;
- must use the existing `checker::loop_analysis::solve_loop_header` rather than inventing a second loop solver;
- must use the existing `EvalControl` / `ControlBoundary` / `ControlDisposition` model rather than inventing a second static-control channel;
- must use the existing `QueryBudget` and `CancellationToken` as static-loop resource authorities;
- must preserve explicit user `whileTrue` as a method/send identity; source `while` must cease being represented semantically as such a send;
- must keep runtime compatibility behavior around `Block.whileTrue` unless a separate accepted language decision explicitly supersedes it;
- must not implement static `while` by executing runtime methods or VM dispatch;
- must not implement static `while` with unbounded Rust recursion or an unbudgeted host-language loop;
- must not opportunistically fix unrelated repository failures;
- must follow the BUILD / STABILIZE / CERTIFY verification budget below;
- must classify unrelated failures and record them rather than widening scope;
- must STOP AND CONSULT whenever a global or task-local escalation trigger fires;
- must produce a P10 walkthrough and P10 handoff before P10 is marked complete;
- must keep `LANG007.C4-CHECKPOINT.md` truthful.

Testing is evidence gathering, not a ritual. Do not run broad tests merely because source changed.

---

# 1. Goal

Complete `LANG007.C4.P10` on current `main` by replacing parser-desugared plain `while` with an explicit semantic AST/control-flow identity, integrating that identity through checking, flow, indexing, static evaluation, compiler lowering, LSP traversal, and resource verification, while preserving existing runtime behavior and all already-established P10/P3/P4/P5/P6 ownership boundaries.

---

# 2. Checkpoint acceptance objective

P10 is a **foundation/stabilization plan inside `LANG007.C4`**, not checkpoint closure.

Its remaining acceptance objective is to leave C4 with one coherent semantic-control foundation that downstream P6 can consume:

```text
ordinary source while
        ↓
explicit semantic Expr::While
        ├─ ordinary flow/checker analysis
        ├─ CFG/source-index identity
        ├─ deterministic bounded static evaluation
        └─ compiler runtime lowering

return / break / continue
        ↓
existing EvalControl
        ↓
nearest semantic control boundary
```

After this plan:

- P5 consumes the existing P10 structural classifier and P4 unordered-product facts;
- P6 consumes completed explicit `while`, `EvalControl`, static `if`, ordered/Sum reflection foundation, and later P5 structural facts;
- P9 consumes P10's ontology/tooling/resource baselines and performs final checkpoint-wide public/tooling/performance closure.

P10 is complete when its own remaining scope is implemented and focused-tested. It does **not** require P4/P5/P6/P7/P8/P9 to be complete.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   6e5287af46e7bde7a13fb2cecd37a10f42d6ee80
date:       2026-10-11
```

Immediately preceding source-audit revision:

```text
3ad129dce9a920362235ad071e8dbd168ba7f0b4
```

The single commit from `3ad129dc` to `6e5287af` changes only unrelated specification files, so the source audit remains valid.

Verified current repository facts:

1. `phalcom-semantic/src/structural.rs` contains the lower P10 structural classifier and shallow ordered/Sum descriptors.
2. `phalcom-semantic/src/static_eval/machine.rs` already contains `EvalControl::{Return,Break,Continue}`.
3. `ControlBoundary::Loop` already exists specifically for the remaining explicit-while work.
4. static `return` and static `if` are already implemented in the evaluator Work machine.
5. `parse_if` already produces explicit `Expr::If` with `LexicalBlock`.
6. `while let` already produces explicit `Expr::WhileLet`.
7. plain `while` still parses to synthetic `MethodCall("whileTrue")`.
8. `checker/expression.rs` still contains a specialized `"whileTrue"` control-method branch implementing ordinary while flow semantics over the parser-generated shape.
9. `checker/flow/graph.rs` likewise recognizes `"whileTrue"` as loop structure.
10. `phalcom-core/src/compiler/inliner.rs` contains mature `SacredCall::WhileTrue` runtime lowering with loop-context, guard, deopt fallback, `break`, and `continue`.
11. `phalcom-core/src/compiler/lib/expr.rs` already lowers explicit `WhileLet` directly.
12. `phalcom-semantic/src/static_eval/machine.rs` does not yet evaluate `Expr::While`.
13. static evaluator resource/cancellation tests already exist in `phalcom-semantic/tests/semantic/integration/static_definition_bodies.rs`.
14. the inherited fixed-call Pack discriminator still exists as `static_calls_and_static_tuple_emit_no_pack_machinery` in `phalcom-core/tests/core/collections/outgoing_packs_completion.rs`.
15. current explicit-control AST consumers include at least `c4_migrate.rs`, semantic checker/CFG/advisory/source-index/occurrence, core compiler/product-opt/semantic-lowering, and LSP semantic-token traversal.
16. at plan start the P10 documentation files were nested under an anomalous `.../LANG007.C4/LANG007.C4/` directory; R0 moved the plan and inherited P10 records into the canonical `LANG007.C4.P10/` directory and refreshed checkpoint links.

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as a STOP / CONSULT condition.

---

# 4. Required reads before implementation

Read these before any production edit:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-requirements-analysis.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-structural-static-upgrade-handoff.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P10/LANG007.C4.P10-parallelized-implementation-plan.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P10/LANG007.C4.P10-S1-merge-incident-2026-10-05.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P10/LANG007.C4.P10-legacy-ontology-census.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P5-implementation-plan.md

docs/specs/static-evaluation.md
docs/decisions/accepted/0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md
docs/decisions/accepted/0016-sacred-selector-inliner-and-override-guard.md
```

Read current source before implementation:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/bin/c4_migrate.rs

phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/control.rs
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/checker/loop_analysis.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/advisory/analyzer.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/error.rs

phalcom-core/src/compiler/inliner.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/jumps.rs
phalcom-core/src/compiler/lib/scope.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/modules/semantic_lowering.rs

phalcom-lsp/src/semantic_tokens.rs
```

Read focused tests before writing new tests:

```text
phalcom-ast/src/parser.rs
phalcom-semantic/tests/semantic/integration/static_definition_bodies.rs
phalcom-core/tests/core/language/corpus.rs
phalcom-core/tests/fixtures/language/control-flow/
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
```

Before production changes, perform a current-tree census:

```sh
rg -n 'Expr::WhileLet|Expr::If\(|Expr::IfLet|whileTrue|SacredCall::WhileTrue|ControlBoundary::Loop' \
  phalcom-ast phalcom-semantic phalcom-core phalcom-lsp
```

---

# 5. Normative authority

## 5.1 Static-evaluation specification

Current normative static evaluation supports `if`, `match`, `for`, `while`, evaluator-local bindings and assignment, `return`, `break`, and `continue`.

It explicitly states:

```text
while is an explicit semantic control-flow construct
```

and:

```text
a selector, method, protocol, or user declaration named whileTrue has no intrinsic or privileged meaning in static evaluation
```

Runtime lowering may independently compile semantic `while` into efficient control flow after semantic analysis.

## 5.2 TDR-0090

TDR-0090 ratifies the same control identity. P10 completion exists specifically to converge implementation with that accepted rule.

## 5.3 Existing runtime `whileTrue` behavior

TDR-0016 established guarded sacred-selector optimization for runtime `Block.whileTrue` sends, including runtime override fallback.

No currently audited decision authorizes P10 to delete that runtime behavior.

Therefore distinguish:

```text
semantic source while
    -> explicit AST/control identity

explicit source .whileTrue(...)
    -> method/send identity
    -> may retain existing guarded compiler optimization/runtime fallback
```

## 5.4 Existing flow architecture

Canonical ordinary loop analysis is owned by:

```text
phalcom-semantic/src/checker/loop_analysis.rs
    solve_loop_header(...)
```

and existing loop-frame state in `CheckingContext`.

## 5.5 Existing static-control architecture

Canonical static-control propagation is:

```text
EvalControl
    ↓
ControlBoundary
    ↓
control_disposition(...)
```

## 5.6 Existing evaluator resource authority

Canonical evaluator resources are:

```text
QueryBudget
CancellationToken
```

No separate while fuel or timeout is permitted.

---

# 6. Takeover state

## 6.1 Inherited P10 completion state

Treat these as already implemented unless current-head verification proves a regression:

| Original P10 item | Current status | Current owner |
|---|---|---|
| T1 shared structural classifier | implemented / focused-tested | `phalcom-semantic/src/structural.rs` |
| T2 ordered structural facts | implemented / focused-tested | `structural.rs` |
| T3 Sum/Variant shallow facts | implemented / focused-tested | `structural.rs` + existing reflection/enum products |
| T4 EvalControl + return + static if | implemented / focused-tested | `static_eval/machine.rs` |
| T5 break/continue carrier | implemented / focused-tested | `static_eval/machine.rs` |
| S1 P4/P10 synchronization | complete | checkpoint/history |
| T7 P1–P3 convergence audit | complete | lifecycle/docs |
| T8 tooling/ontology census | complete | P10 census |
| T9 Pack harness | implemented / focused-tested | P3/P10 tests |
| T9 non-loop evaluator harness | implemented / focused-tested | `static_definition_bodies.rs` |

Do not reopen these as design tasks.

## 6.2 Stable interface map

| Concept | Current symbol/path | Owner | P10 completion invariant |
|---|---|---|---|
| structural classification | `StructuralClassifier` / `structural.rs` | P10 | untouched except regressions |
| ordered facts | `OrderedProductStructure` | P10 | no public capability work |
| Sum facts | `SumStructure` | P10 | no P5 Variant public-value work |
| static result | `StaticEvalResult` | P2/P10 | do not redesign |
| control transfer | `EvalControl` | P10 | loop consumes Break/Continue only |
| control boundaries | `ControlBoundary` | P10 | `Loop` becomes live |
| control disposition | `control_disposition` | P10 | sole static-control boundary policy |
| evaluator work machine | `Work<'a>` / `StaticEvaluator::run` | P2/P10 | loop represented as bounded Work transitions |
| evaluator budget | `QueryBudget` | query layer | every loop work transition remains charged |
| evaluator cancellation | `CancellationToken` | query layer | cooperative cancellation retained |
| loop fixed point | `solve_loop_header` | semantic checker | ordinary while reuses it |
| runtime loop jumps | `emit_backward_loop`, compiler loop context | core compiler | reuse; no VM loop redesign |
| sacred runtime while | `SacredCall::WhileTrue` / `compile_while_true` | compiler/runtime | compatibility retained |
| P3 Pack fast path | outgoing-pack compiler/tests | P3 | must remain unchanged |
| P4 unordered facts | FieldKey/Record row machinery | P4 | not modified by P10 |
| P5 structural evidence | P5 plan | P5 | out of scope |
| P6 static match/for/proofs | P6 | P6 | out of scope |

---

# 7. Architecture

## 7.1 Target control identity

Current:

```text
source while
    ↓
parser synthetic MethodCall("whileTrue")
    ↓
checker/compiler recognize source shape + selector spelling
```

Target:

```text
source while
    ↓
Expr::While { condition, body, range }
    ├─ direct checker semantics
    ├─ direct CFG/source-index identity
    ├─ direct static evaluator semantics
    └─ direct runtime compiler lowering

explicit user .whileTrue(...)
    ↓
Expr::MethodCall
    ↓
ordinary/sacred method-send behavior
```

## 7.2 Proposed AST shape

```rust
#[derive(Debug, Clone)]
pub struct WhileExpr {
    pub condition: Expr,
    pub body: LexicalBlock,
    pub range: SourceRange,
}

pub enum Expr {
    ...
    If(Box<IfExpr>),
    While(Box<WhileExpr>),
    WhileLet(Box<WhileLetExpr>),
    ...
}
```

Equivalent box placement/order is mechanically flexible.

`LexicalBlock` is preferred because plain `while` is lexical control, not Closure identity.

## 7.3 Checker authority flow

```text
Expr::While
    ↓
analyze condition as Bool
    ↓
condition truth / trusted predicate
    ↓
solve_loop_header
    ↓
true branch:
    push loop frame
    analyze lexical body
    collect continue + normal backedge
    collect break exits

false branch:
    ordinary exit
    ↓
join false exit + break exits
    ↓
Unit result
```

## 7.4 Static evaluator flow

```text
Evaluate(Expr::While)
    ↓
Evaluate(condition)
    ↓
AfterWhileCondition
    ├─ false -> Unit
    └─ true  -> EvaluateBlock(body)
                   ↓
              AfterWhileBody
                   │
          ┌────────┼────────┐
          │        │        │
       normal    Break   Continue
          │        │        │
 discard result    │        │
          └───┬────┘        │
              │             ▼
              ▼          condition
             Unit

Return(value)
    -> propagate through Loop
    -> consume at StaticApplication/Root
```

## 7.5 Runtime compiler flow

Preferred:

```text
Expr::While
    ↓
compile_while_expr
    ↓
shared lower loop emitter
    ↓
JumpIfFalse / Loop / loop-context jumps
```

Existing `SacredCall::WhileTrue` may reuse a shared lower helper, but semantic `Expr::While` must not be reconstructed as a semantic MethodCall.

---

# 8. Ownership boundaries

## Owns

- explicit plain-while AST identity;
- parser production of that identity;
- direct checker/flow semantics;
- CFG identity;
- source-index/occurrence/advisory traversal;
- static-evaluator direct while execution;
- use of `ControlBoundary::Loop`;
- while-specific evaluator budget/cancellation evidence;
- compiler lowering for explicit `Expr::While`;
- narrow AST-consumer convergence;
- runtime compatibility preservation;
- P10 recertification and lifecycle closure.

## Does not own

- P4 `FieldKey`, Record rows, Record patterns, unordered lookup;
- P5 structural capabilities/reflection/member domains/projections;
- P6 static `match`, structural `for`, patterns, proof predicates;
- P7 recursion/residual IR/memoization;
- P8 durable metadata/materialization;
- P9 final ontology/tooling/performance certification;
- generic conformance/evidence redesign;
- trait dispatch;
- associated-member lowering;
- contextual algebraic realization;
- VM dispatch redesign;
- Pack storage redesign.

## Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| source plain-while identity | `Expr::While` | checker/compiler/tools | synthetic MethodCall as semantic truth |
| loop type/flow analysis | checker + `solve_loop_header` | diagnostics/lowering | compiler-side flow solver |
| static control transfer | `EvalControl` | evaluator boundaries | second control enum |
| loop transfer policy | `control_disposition(ControlBoundary::Loop)` | evaluator | ad hoc branch policy |
| evaluator work accounting | `QueryBudget` | Work machine | hidden loop counter |
| cancellation | `CancellationToken` | Work machine | timeout subsystem |
| runtime loop emission | compiler loop/jump substrate | VM bytecode | new VM loop protocol |
| explicit `whileTrue` identity | MethodCall | checker/compiler/runtime | parser conversion to While |
| Pack fixed-call optimization | P3 compiler path | runtime calls | P10 workaround |

---

# 9. Global invariants

`P10-INV-01` — Plain source `while` has explicit semantic identity.

`P10-INV-02` — `whileTrue` spelling has no static privilege.

`P10-INV-03` — `while let` remains separate and keeps pattern semantics.

`P10-INV-04` — `while` requires canonical Bool; no truthiness.

`P10-INV-05` — Ordinary while flow uses the existing fixed-point solver.

`P10-INV-06` — `break` and `continue` bind to the nearest loop.

`P10-INV-07` — `return` propagates through loops.

`P10-INV-08` — Static iteration is represented by explicit Work scheduling, not host recursion/unbudgeted looping.

`P10-INV-09` — Every static iteration consumes ordinary query work.

`P10-INV-10` — Cancellation remains cooperative and yields `Cancelled`.

`P10-INV-11` — Per-iteration body results do not accumulate.

`P10-INV-12` — Untaken work is not evaluated.

`P10-INV-13` — Runtime source behavior remains compatibility-preserving.

`P10-INV-14` — Explicit runtime `whileTrue` remains independently identifiable.

`P10-INV-15` — No second evaluator.

`P10-INV-16` — No second structural classifier.

`P10-INV-17` — P3 fixed-call/no-Pack invariant remains intact.

`P10-INV-18` — AST tooling traverses while lexically without fake selector occurrences.

`P10-INV-19` — P5/P6 boundaries remain intact.

`P10-INV-20` — Lifecycle truth is preserved.

---

# 10. Non-goals

Do not redesign `if`, merge While/WhileLet, implement static match/for/proofs, add a static `whileTrue` intrinsic, redesign the VM, redesign non-local return, redesign generic evidence, alter P4/P5 architecture, alter Pack storage, add residualization, add final metadata materialization, or perform P9 release certification.

---

# 11. Expected impact map

## Source

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/bin/c4_migrate.rs

phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/advisory/analyzer.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/static_eval/machine.rs

phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/inliner.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/modules/semantic_lowering.rs

phalcom-lsp/src/semantic_tokens.rs
```

## Tests

```text
phalcom-ast/src/parser.rs
phalcom-semantic/tests/semantic/integration/static_definition_bodies.rs
phalcom-core/tests/fixtures/language/control-flow/
phalcom-core/tests/core/language/corpus.rs
```

Verification-only Pack test:

```text
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
```

## Lifecycle

Canonical target:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P10/
    LANG007.C4.P10-parallelized-implementation-plan.md
    LANG007.C4.P10-S1-merge-incident-2026-10-05.md
    LANG007.C4.P10-legacy-ontology-census.md
    LANG007.C4.P10-completion-luna-patch-grade-implementation-plan.md
    LANG007.C4.P10-walkthrough.md
    LANG007.C4.P10-handoff.md
```

Unexpected architectural entry into `generic_evidence.rs`, `types/type_member.rs`, VM dispatch, Block primitives, or type-meta is a consultation condition unless purely mechanical.

---

# 12. Implementer decision authority

## FIXED

The implementer must not change:

1. explicit `Expr::While` semantic identity;
2. distinct `Expr::WhileLet` identity;
3. Bool-only condition semantics;
4. static `whileTrue` non-privilege;
5. `solve_loop_header` ownership;
6. `EvalControl` ownership;
7. `ControlBoundary::Loop` ownership;
8. `QueryBudget`/`CancellationToken` ownership;
9. Work-machine iteration;
10. return propagation through loops;
11. nearest-loop break/continue;
12. P3 Pack invariant;
13. P5/P6 ownership boundaries;
14. runtime compatibility unless superseded by accepted authority.

## MECHANICALLY FLEXIBLE

May adapt:

- exact `WhileExpr` box layout;
- helper names;
- Work variant names;
- private compiler helper factorization;
- parser test names;
- exact fixture split;
- narrow visitor helper decomposition;
- doc move mechanics.

## VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| parser still desugars plain while | `parse_while` | consume current partial migration |
| `Expr` lacks While | `ast.rs` | skip duplicate AST work |
| Loop boundary unused | `static_eval/machine.rs` | integrate with current usage; consult if semantics materially differ |
| checker donor still under whileTrue | `checker/expression.rs` | port current equivalent |
| CFG donor still under whileTrue | `flow/graph.rs` | port current equivalent |
| sacred runtime lowering still exists | `inliner.rs` | preserve current equivalent |
| loop solver still canonical | `loop_analysis.rs` | use current canonical solver |
| Pack discriminator still exists | outgoing pack test | locate equivalent if renamed |
| P10 docs still nested | docs tree | do not move again if fixed |

---

# 13. Global STOP / CONSULT triggers

Stop if:

1. explicit While cannot preserve accepted runtime behavior without a language decision;
2. accepted static/runtime authorities conflict materially;
3. implementation needs static invocation of `whileTrue`;
4. implementation needs a second evaluator;
5. implementation needs a second resource budget;
6. correct loop analysis cannot use the canonical loop solver;
7. existing EvalControl cannot express correct loop transfer;
8. lexical while body would require Closure semantics;
9. VM-dispatch architecture must change;
10. generic conformance/evidence must change;
11. P4 row/FieldKey must change;
12. P5 structural/member architecture must change;
13. P6 match/for/proof scope must be pulled forward;
14. Pack verification failure suggests representation change;
15. the same semantic failure persists after one serious correction;
16. two plausible fixes differ materially in compatibility semantics;
17. an invariant must be weakened to pass tests;
18. main moves materially and invalidates the plan premise.

---

# 14. Debugging budget

Mechanical failures: up to three coherent inspect/fix/retest cycles while evidence shows progress.

Semantic failure protocol:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

One serious corrective attempt per underlying semantic failure; if it persists, consult.

Architectural failures: zero speculative fixes.

---

# 15. Testing surface analysis

| Coverage | Dimension | Invariants | Case |
|---|---|---|---|
| CV-01 | parser identity | 01,03 | plain while -> While; while-let unchanged |
| CV-02 | method discriminator | 02,14 | explicit whileTrue stays MethodCall |
| CV-03 | Bool boundary | 04 | non-Bool rejected / ExpectedBool |
| CV-04 | zero iteration | 08,11,12 | false -> Unit, body skipped |
| CV-05 | finite iteration | 08,09,11 | mutable local reaches expected result |
| CV-06 | break | 06,11,12 | nearest loop exits |
| CV-07 | continue | 06,11,12 | condition retested |
| CV-08 | return | 07,12 | crosses loop to call/root |
| CV-09 | nesting | 06,07 | inner control binds correctly |
| CV-10 | fixed point | 05 | loop-carried mutation/refinement sound |
| CV-11 | CFG | 01,18 | direct LoopHeader/backedge/exit |
| CV-12 | source index | 18 | lexical body + outer writes |
| CV-13 | selector absence | 01,18 | no fake whileTrue occurrence |
| CV-14 | runtime parity | 13 | ordinary source while runs |
| CV-15 | explicit runtime send | 14 | explicit whileTrue behavior retained |
| CV-16 | budget | 09 | infinite loop -> BudgetExceeded |
| CV-17 | cancellation | 10 | active loop -> Cancelled |
| CV-18 | work scaling | 09 | more iterations -> more steps |
| CV-19 | stack discipline | 11 | no per-iteration accumulation |
| CV-20 | structural regression | 16 | classifier tests pass |
| CV-21 | static-control regression | 15 | static return/resource tests pass |
| CV-22 | Pack regression | 17 | fixed calls emit no Pack machinery |
| CV-23 | affected crates | 18,19 | AST/semantic/core/LSP compile |
| CV-24 | lifecycle | 20 | checkpoint and records agree |

---

# 16. Verification execution budget

## BUILD

Use exact new test selectors only.

Examples:

```sh
cargo test -p phalcom-ast while -- --nocapture
```

and exact new semantic while tests.

## STABILIZE

```sh
cargo test -p phalcom-ast while -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  integration::static_definition_bodies \
  -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  foundations::structural_classification \
  -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  adts::structural_reflection \
  -- --nocapture
```

Run only the relevant core control-flow group/current equivalent.

Pack discriminator:

```sh
cargo test -p phalcom-core --test core \
  static_calls_and_static_tuple_emit_no_pack_machinery \
  -- --nocapture
```

## CERTIFY

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-core \
  -p phalcom-lsp \
  --message-format=short
```

Optionally compile semantic test binary:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  --no-run \
  -p phalcom-semantic \
  --test semantic \
  --message-format=short
```

Do not reflexively run workspace tests/clippy/release certification during BUILD.

---

# 17. Baseline failure policy

Classify:

```text
A — caused by patch
B — probably caused by patch
C — unclear
D — unrelated/baseline
```

A/B are active responsibility. C gets one bounded classification pass. D is recorded and deferred immediately.

---

# 18. Task dependency graph

```text
R0  current-main takeover + lifecycle topology
 │
 ▼
R1  explicit While AST/parser + exhaustive consumers
 │
 ├──────────────┐
 ▼              ▼
R2 checker/CFG  R3 runtime compiler lowering
 │              │
 └──────┬───────┘
        ▼
R4 static evaluator while
        │
        ▼
R5 resource/cancellation/stack evidence
        │
        ▼
R6 current-main recertification
        │
        ▼
R7 lifecycle closure + handoff
```

---

# 19. R0 — Current-main takeover and lifecycle topology normalization

## Purpose

Establish a clean execution baseline and repair durable P10 record layout.

## Required commands

```sh
git fetch origin
git switch main
git pull --ff-only
git status --short
git rev-parse HEAD
git rev-parse origin/main
```

Required:

```text
branch = main
worktree clean
HEAD = origin/main
HEAD >= audited revision
```

If dirty, inspect and preserve user work. Do not stash/reset/discard as a convenience.

Normalize the nested P10 documentation path if still present. Preserve historical contents and commit history. Do not rewrite old plan metadata as though it were newly prepared.

### Tests now

No runtime tests.

### Acceptance

- exact baseline recorded;
- coherent P10 directory;
- checkpoint links resolve;
- historical state preserved;
- no source behavior changed.

### STOP

Dirty ownership ambiguity, branch divergence, doc-path collision, or competing canonical state docs.

---

# 20. R1 — Explicit semantic `While` AST/parser identity and AST-consumer convergence

## Purpose

Replace parser-generated `MethodCall("whileTrue")` for plain source `while` with first-class semantic AST identity.

## Files

Read/modify:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/bin/c4_migrate.rs
```

Then compile/census downstream exhaustive matches.

## Required implementation

Add conceptually:

```rust
#[derive(Debug, Clone)]
pub struct WhileExpr {
    pub condition: Expr,
    pub body: LexicalBlock,
    pub range: SourceRange,
}
```

Add `Expr::While` and range handling.

Change ordinary `parse_while` branch from synthetic block + `whileTrue` MethodCall to:

```rust
let condition = self.parse_expr_without_trailing_closures()?;
let body = self.parse_lexical_block()?;
Expr::While(...)
```

Preserve `while let`.

Add parser discriminator proving explicit `.whileTrue` remains `Expr::MethodCall`.

Update `c4_migrate.rs` traversal, including known binding and field-write visitors.

## Forbidden

No hidden MethodCall inside While, no wildcard WhileLet encoding, no Closure body, no parser recognition of arbitrary `whileTrue` spelling as control.

## Tests

```sh
cargo test -p phalcom-ast while -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast --message-format=short
```

## Acceptance

CV-01/CV-02 satisfied and AST crate passes.

---

# 21. R2 — Ordinary checker, CFG, source-index, occurrence, and advisory convergence

## Purpose

Move ordinary while semantics from synthetic `whileTrue` shape recognition to direct `Expr::While` analysis.

## Files

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/loop_analysis.rs        # read; change only if helper extraction is truly needed
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/advisory/analyzer.rs
```

## Checker implementation donor

Use the current `synthesize_control_method_call` branch:

```rust
"whileTrue" if receiver_is_literal_block => { ... }
```

as the semantic donor.

Prefer a direct helper:

```rust
fn synthesize_while_expr(
    ctx: &mut CheckingContext<'_>,
    while_expr: &WhileExpr,
    expected: &ExpectedType,
) -> TypedExpression
```

Port behavior, not synthetic source shape.

Algorithm:

```text
before = current flow

evaluate_step(header):
    flow = header
    analyze condition against Bool
    derive true/false flow
    apply trusted predicates
    if true reachable:
        push loop frame
        analyze lexical body
        pop loop frame
        normal completion + continue -> backedge
        break -> exits
    return backedge, false exit, breaks, causal invalidity

solve_loop_header(...)
final real pass at stable header
join false exit + breaks
return Unit
```

Preserve causal invalidity and trusted-predicate refinement.

### Explicit `whileTrue` checker behavior

Do not automatically delete all explicit-send compatibility behavior. Source while no longer depends on it, but ordinary runtime method-send semantics may still require it. Keep or narrow it based on current accepted runtime behavior; do not let it remain the authority for source while.

## CFG

Add direct `Expr::While` graph with LoopHeader, condition, body, backedge, false exit, and loop-stack targets.

## Source index / occurrences

Condition uses enclosing lexical environment. Body gets normal lexical loop-body scope. No synthetic method occurrence.

## Advisory

Traverse condition/body only; do not create a second advisory loop solver.

## Forbidden

No second loop solver, no fake MethodCall insertion, no one-pass mutation approximation, no compiler-side Bool checking.

## Tests

Cover CV-03, CV-06, CV-07, CV-09, CV-10, CV-11, CV-12, CV-13 with the narrowest existing semantic test infrastructure.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --message-format=short
```

## Acceptance

Direct checked/CFG/index identity with fixed-point semantics intact.

---

# 22. R3 — Runtime compiler lowering and explicit-send compatibility

## Purpose

Compile explicit semantic `Expr::While` efficiently while preserving observable runtime behavior.

## Files

```text
phalcom-core/src/compiler/inliner.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/jumps.rs       # read
phalcom-core/src/compiler/lib/scope.rs       # read
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/modules/semantic_lowering.rs
```

## Required implementation

Add direct compiler arm:

```rust
Expr::While(while_expr) => self.compile_while_expr(*while_expr)?
```

Reuse mature `compile_while_true` loop skeleton:

```text
loop start
loop context
condition
JumpIfFalse
body
Pop body result
backward Loop
break patch
continue patch
Unit/Nil result
```

Factor a small private shared loop emitter if useful. Do not broadly refactor sacred selectors.

### Runtime compatibility

Current runtime behavior historically includes guarded `Block.whileTrue` override fallback. Preserve that unless a newer accepted decision explicitly supersedes it.

Preferred shape:

```text
semantic Expr::While
    -> direct fast loop
    -> compiler-local compatibility fallback if required
```

The fallback may synthesize compiler-local block/send material only for code generation. It must not restore MethodCall as semantic AST identity.

Explicit source `.whileTrue` remains MethodCall and follows current ordinary/sacred send behavior.

### Product optimizer

Traverse direct While and maintain current loop-depth accounting.

### Semantic-lowering visitor

Traverse condition and body only. Do not touch current generic evidence/call-environment logic except mechanical exhaustive-match integration.

## Tests

Use the narrow language control-flow corpus/current equivalent. Prove CV-14/CV-15 and runtime break/continue/non-Bool behavior.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --message-format=short
```

## STOP

Any need for VM redesign, selector semantic redesign, or unresolved conflict between accepted runtime and static authorities.

---

# 23. R4 — Static evaluator `while` using existing Work/EvalControl architecture

## Purpose

Implement deterministic direct static evaluation of `Expr::While` without runtime method execution, a second evaluator, host recursion, or unbudgeted host looping.

## File

```text
phalcom-semantic/src/static_eval/machine.rs
```

Read `static_eval/error.rs` for existing outcomes/errors.

## Required implementation shape

Add Work states conceptually:

```rust
AfterWhileCondition(&'a WhileExpr),
AfterWhileBody(&'a WhileExpr),
```

Equivalent explicit loop-frame Work data is acceptable.

### Evaluate While

```rust
Expr::While(w) => {
    work.push(Work::AfterWhileCondition(w));
    work.push(Work::Evaluate(&w.condition));
}
```

### After condition

Pop exactly one static value and require `StaticValueData::Bool`.

False:

```text
push Unit
skip body
```

True:

```text
schedule loop body boundary
EvaluateBlock(body)
```

Non-Bool:

```text
ExpectedBool
```

### Pending-control allowlist — critical

Current evaluator skips ordinary queued work while `pending_control` exists. The loop-boundary Work item **must remain executable while control is pending**.

Conceptually include it with:

```text
PopTrace
FinishRoot
FinishStaticApplication
AbortStaticApplication
AfterWhileBody / HandleLoopControl
```

If omitted, Break/Continue escape incorrectly.

### Loop boundary

Use only:

```rust
control_disposition(ControlBoundary::Loop, control)
```

Required mapping:

```text
normal body completion:
    discard exactly one body result
    schedule condition again

Break:
    consume
    push Unit

Continue:
    consume
    schedule condition again

Return:
    propagate unchanged
    do not schedule another iteration

Invalid:
    existing canonical invalid-control path
```

Inspect pending control before popping a normal body value. Abrupt bodies may not leave one.

### Nested loops

Rely on Work-stack nesting. Do not add global loop-depth semantic authority.

### Resource behavior

Each scheduled Work item is already budget-charged and cancellation-checked. Keep iteration as recurring Work scheduling so this remains true.

No `MAX_STATIC_WHILE`, wall-clock timeout, or hidden loop fuel.

### Trace/value-stack discipline

Iteration must not retain prior body results or trace frames. Host stack depth must be independent of iteration count.

## Tests

Primary file:

```text
phalcom-semantic/tests/semantic/integration/static_definition_bodies.rs
```

Add focused functional static-while coverage using only already-supported static operations:

```text
zero iteration
finite counter
break
continue
return from loop
nested loop
```

Add explicit static `whileTrue` non-privilege discriminator at the appropriate checker/evaluator layer.

Run exact new test, then existing:

```text
static_return_control_propagates_to_root_and_stops_at_call_boundaries
```

## Acceptance

CV-04..CV-09 and CV-19 pass; existing return-control regression remains green.

## STOP

Any need for second evaluator/control channel/global loop state or inability to preserve bounded Work semantics.

---

# 24. R5 — While-specific resource, cancellation, and untaken-work evidence

## Purpose

Complete P10 T9's while-dependent resource coverage.

## Test file

```text
phalcom-semantic/tests/semantic/integration/static_definition_bodies.rs
```

Prefer a new focused test near the existing evaluator resource test.

## Required evidence

### Work scaling

```text
LoopLong steps > LoopShort steps
```

Both reduce.

Do not assert exact step counts.

### Budget exhaustion

A static infinite loop under small `QueryBudget` must produce `BudgetExceeded`.

### Pre-cancelled

Pre-cancel token -> `Cancelled`.

### Active cancellation

Reuse the existing threaded cancellation pattern; assert `Cancelled` and `steps_taken > 0`.

### Untaken work

False condition / Break / Return skips later expensive or unsupported work.

### Stack discipline

Use a reasonably long finite loop to prove no practical iteration-proportional host/value-stack growth. Do not turn it into a stress benchmark.

## Forbidden

No exact timing assertions, no exact step goldens, no new benchmark framework.

## Acceptance

CV-16/CV-17/CV-18/CV-19 pass.

---

# 25. R6 — Current-main recertification

## Purpose

Prove new while convergence without regressing inherited P10 structural work or P3 Pack fast paths.

## Pre-test review

```sh
git diff --check
git status --short
git diff --stat
```

Review final diff and explicitly confirm:

```text
no P4 unordered redesign
no P5 structural capability/member-domain implementation
no P6 match/for/proof implementation
no VM control redesign
no Pack representation change
no static whileTrue intrinsic
no second loop solver
no second evaluator
```

## G1 — Parser identity

```sh
cargo test -p phalcom-ast while -- --nocapture
```

## G2 — Static control/evaluator

```sh
cargo test -p phalcom-semantic --test semantic \
  integration::static_definition_bodies \
  -- --nocapture
```

If too broad, run exact named P10 tests first, then the module group.

## G3 — Structural foundation regression

```sh
cargo test -p phalcom-semantic --test semantic \
  foundations::structural_classification \
  -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  adts::structural_reflection \
  -- --nocapture
```

## G4 — Runtime while/control compatibility

Run the narrow core control-flow fixture group/current equivalent. Include ordinary while, break/continue, explicit whileTrue compatibility, and while-let regression if shared code changed.

## G5 — P3 fixed-call/no-Pack discriminator

```sh
cargo test -p phalcom-core --test core \
  static_calls_and_static_tuple_emit_no_pack_machinery \
  -- --nocapture
```

If filter requires full module path, use it.

On failure, diagnose causally. Do not alter Pack architecture without consultation.

## G6 — Affected crates compile

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-core \
  -p phalcom-lsp \
  --message-format=short
```

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  --no-run \
  -p phalcom-semantic \
  --test semantic \
  --message-format=short
```

## Acceptance

All P10-required gates pass or unrelated failures are classified and recorded.

P10 may then become:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

subject to R7 record closure.

---

# 26. R7 — Lifecycle closure, walkthrough, handoff, checkpoint truth

## Purpose

Make the implemented P10 state durable for P5/P6/P9.

## Create

```text
LANG007.C4.P10-walkthrough.md
LANG007.C4.P10-handoff.md
```

under the canonical P10 directory.

## Update

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
```

## Walkthrough must record

```text
final branch/revision
explicit While AST shape
parser migration
checker/fixed-point implementation
CFG/index/tooling convergence
runtime compiler lowering choice
whileTrue compatibility decision
static Work/EvalControl loop
pending-control loop-boundary handling
body-result stack discipline
budget/cancellation behavior
exact tests added
exact commands run
pass/fail evidence
deferred/unrelated failures
deviations
consultations
residual risks
```

## Handoff must state

### P5 inherits

```text
StructuralClassifier
OrderedProductStructure
SumStructure
SpecializedVariantStructure
```

P10 completion does not own P5 public reflection/evidence.

### P6 inherits

```text
Expr::While
ordinary while checker/CFG identity
EvalControl
ControlBoundary::Loop
bounded direct static while
return/break/continue behavior
```

P6 must not recreate `whileTrue` static control semantics.

### P9 inherits

```text
P10 ontology census
Pack/evaluator resource baseline
completed while control foundation
```

## Checkpoint ledger

Update P10 row to COMPLETE / IMPLEMENTED / FOCUSED_TESTED only if actual R6 evidence supports it.

Do not mark C4 complete.

---

# 27. Static evaluator implementation notes for Luna

This section is intentionally mechanical because R4 is the highest-risk part.

## 27.1 Current pending-control filter

Current `run()` conceptually skips most work while `pending_control` exists. The new loop-boundary item must be allowed through.

Predictable bug if omitted:

```text
break
 -> pending_control = Break
 -> loop boundary skipped
 -> Break reaches root/application
 -> Invalid
```

Test Break immediately after implementing the allowlist.

## 27.2 Preserve control while propagating

Do not destroy the control before deciding disposition.

Conceptually:

```rust
if let Some(control) = pending_control.take() {
    match control_disposition(ControlBoundary::Loop, control) {
        ControlDisposition::ConsumeBreak => { ... }
        ControlDisposition::ConsumeContinue => { ... }
        ControlDisposition::Propagate(_) => {
            pending_control = Some(control);
        }
        ...
    }
}
```

`EvalControl` is currently Copy, so this is straightforward.

## 27.3 Body-result handling

Normal lexical block evaluation leaves its last result on the stack.

Therefore:

```text
normal -> pop one result -> next condition
abrupt -> inspect control first; do not assume a result exists
```

## 27.4 Condition-result handling

Condition must be ordinary `StaticValueData::Bool`. No kinded truthiness path.

## 27.5 Unit result

Condition false or Break pushes Unit exactly once. Continue and Return do not produce loop result.

## 27.6 Nested loops

Correct LIFO Work ordering should naturally make inner loop boundary receive control first. Do not introduce global current-loop state.

## 27.7 Resource properties

N iterations should produce O(N) Work transitions and O(1) retained loop-control state beyond live locals.

---

# 28. Runtime lowering notes for Luna

Understand `compile_while_true` before editing. It already owns:

```text
GuardBlock
loop start
loop context
condition
JumpIfFalse
body
body Pop
Loop backedge
break patch
continue patch
Unit/Nil result
deopt fallback
whileTrue send
```

Prefer factoring only the lower loop-emission mechanics needed by direct `Expr::While`.

Do not refactor all sacred selectors.

Semantic `While.condition` is an ordinary expression and `body` is lexical. Compile them directly.

If compatibility fallback needs Closure/block objects, synthesize them only inside compiler code generation.

Continue target = condition reevaluation. Break target = loop exit.

---

# 29. Checker notes for Luna

The current `whileTrue` checker branch is the donor. Port the algorithm, not the synthetic block receiver shape.

With direct While:

```rust
&while_expr.condition
```

is available directly; no need to recover the last expression from a synthetic condition block.

Preserve:

```text
AlwaysFalse / AlwaysTrue / Unknown condition truth
positive/negative trusted predicates
loop-header fixpoint
continue as backedge
break as exit
false condition as exit
Unit result
causal invalidity
```

---

# 30. Parser/tooling notes for Luna

Update stale comments describing plain while as parser desugaring.

Search `wrap_expr_as_block` uses before deleting anything; do not remove it reflexively.

LSP/source index must not invent a selector token/occurrence absent from source.

Body lexical scope must not leak declarations after the loop.

---

# 31. Test source design notes

Use only currently supported static operations. Preferred static tests rely on:

```text
let mut
assignment
Int literals
supported arithmetic/comparison
if
return
break
continue
```

If one operator is unsupported by static intrinsics, choose another already-supported construct rather than expanding evaluator semantics.

For cancellation, reuse the existing threaded cancellation pattern in `static_definition_bodies.rs`.

For infinite-loop budget, use a small deterministic QueryBudget rather than wall-clock timing.

---

# 32. Verification gates summary

| Gate | Scope | Required |
|---|---|---|
| G1 | AST/parser identity | PASS |
| G2 | semantic checker/flow | PASS |
| G3 | runtime compiler compatibility | PASS |
| G4 | static while control | PASS |
| G5 | while resource/cancellation | PASS |
| G6 | inherited static return/resource | PASS |
| G7 | inherited structural classifier | PASS |
| G8 | P3 fixed-call/no-Pack | PASS |
| G9 | affected crates compile | PASS |
| G10 | lifecycle consistency | PASS |

No workspace-wide release gate is required for P10 completion.

---

# 33. Final focused acceptance matrix

The walkthrough fills actual results:

| Coverage | Evidence | Result |
|---|---|---|
| CV-01 | plain while / while-let parser identity | TBD |
| CV-02 | explicit whileTrue parser discriminator | TBD |
| CV-03 | Bool-only condition | TBD |
| CV-04 | zero iteration | TBD |
| CV-05 | finite iteration | TBD |
| CV-06 | break | TBD |
| CV-07 | continue | TBD |
| CV-08 | return | TBD |
| CV-09 | nested control | TBD |
| CV-10 | loop fixed point | TBD |
| CV-11 | CFG direct While | TBD |
| CV-12 | source-index lexical body | TBD |
| CV-13 | no fake whileTrue occurrence | TBD |
| CV-14 | runtime source while | TBD |
| CV-15 | explicit whileTrue runtime behavior | TBD |
| CV-16 | BudgetExceeded | TBD |
| CV-17 | cancellation | TBD |
| CV-18 | work scaling | TBD |
| CV-19 | stack/body-result discipline | TBD |
| CV-20 | structural classifier regression | TBD |
| CV-21 | static return/resource regression | TBD |
| CV-22 | fixed-call Pack discriminator | TBD |
| CV-23 | affected crates compile | TBD |
| CV-24 | lifecycle closure | TBD |

Do not prefill PASS.

---

# 34. Performance/resource evidence

Required properties:

```text
evaluator host-stack depth:
    O(control nesting), independent of iteration count

evaluator work:
    O(number of semantic Work transitions)

loop iterations:
    each charge positive QueryBudget work

value-stack retained loop state:
    O(1) across iterations beyond live locals

trace:
    bounded by semantic nesting, not iteration history

runtime compiler:
    no first-class Pack allocation introduced by ordinary while

runtime loop:
    existing jump/loop machinery
```

Do not add microbenchmarks. P9 owns final performance certification.

---

# 35. Checkpoint bookkeeping

At start, record only continuation identity and remaining scope.

During execution, update only for durable new invariant/interface, consultation/amendment, deferred failure, or coherent verification gate.

At completion, record final revision, tests actually run, deferred failures, runtime compatibility decision, and downstream handoff.

---

# 36. Walkthrough deliverable

Create `LANG007.C4.P10-walkthrough.md` with:

```text
Identity
Final result
Starting repository state
Explicit While AST/parser
Checker/fixed-point flow
CFG/index/tooling
Runtime compiler lowering
Static evaluator Work/EvalControl loop
Budget/cancellation
whileTrue compatibility
Tests added
Verification actually run
Failures/deferred items
Deviations
Consultations
Final invariants
Residual risks
```

---

# 37. Handoff deliverable

Create `LANG007.C4.P10-handoff.md` with:

```text
repository / branch / revision
stable inherited interfaces
must-preserve invariants
P5 consumption
P6 first action
P9 later action
deferred failures
first recommended commands
things not to redesign
```

---

# 38. Completion truth table

| Fact | Meaning |
|---|---|
| `Expr::While` added | source written |
| tests added | coverage authored |
| new while tests pass | focused behavior proven |
| structural/Pack regressions pass | inherited seams recertified |
| affected crates check | compile integration proven |
| P10 marked COMPLETE | bounded P10 accepted |
| C4 complete | **not implied** |
| release certified | **not implied** |

Expected P10 final state:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

---

# 39. Plan self-review checklist

Before implementation:

- [ ] branch is `codex/lang007-c4-p10`, created from prepared main revision `6e5287af46e7bde7a13fb2cecd37a10f42d6ee80`;
- [ ] worktree clean;
- [ ] HEAD synchronized;
- [ ] P10 docs normalized or normalization task active;
- [ ] plain while still desugars to whileTrue;
- [ ] `Expr::While` not already present;
- [ ] `ControlBoundary::Loop` still needs evaluator consumer;
- [ ] checker donor still exists;
- [ ] runtime donor still exists;
- [ ] loop solver remains canonical;
- [ ] static return/resource tests exist;
- [ ] fixed-call Pack discriminator exists.

Before marking complete:

- [ ] P10-INV-01..20 reviewed against final diff;
- [ ] all important CV obligations have evidence;
- [ ] no duplicate solver/evaluator;
- [ ] no static whileTrue privilege;
- [ ] no P4/P5/P6 architecture absorbed;
- [ ] runtime compatibility decision recorded;
- [ ] Pack discriminator passed;
- [ ] affected crates compiled;
- [ ] walkthrough exists;
- [ ] handoff exists;
- [ ] checkpoint truthful;
- [ ] no placeholder verification claims remain.

---

# 40. Compact executor sequence

```text
1. Rebaseline main and normalize P10 docs.
2. Add WhileExpr + Expr::While.
3. Parse plain while directly; preserve WhileLet and explicit whileTrue.
4. Fix exhaustive AST visitors.
5. Move checker loop semantics to Expr::While.
6. Add direct CFG/source-index/occurrence/advisory traversal.
7. Add direct compiler Expr::While lowering using existing loop machinery.
8. Preserve explicit whileTrue runtime send compatibility.
9. Add Work-machine static While.
10. Make loop boundary runnable under pending_control.
11. Consume Break/Continue at Loop; propagate Return.
12. Discard normal body result each iteration.
13. Add finite/break/continue/return/nested static tests.
14. Add budget/cancellation/work-scaling tests.
15. Run focused structural regression.
16. Run fixed-call/no-Pack discriminator.
17. Run affected-crate compile gate.
18. Write walkthrough/handoff.
19. Update checkpoint P10 -> COMPLETE / IMPLEMENTED / FOCUSED_TESTED.
```

If any step requires material architecture outside this sequence, stop and consult.

---

# 41. Architectural end state

```text
                         SOURCE
                           │
             ┌─────────────┴─────────────┐
             │                           │
       while condition             block.whileTrue(...)
             │                           │
             ▼                           ▼
       Expr::While                 Expr::MethodCall
             │                           │
     ┌───────┼────────┐                  │
     │       │        │                  │
     ▼       ▼        ▼                  ▼
 checker    CFG    source index     ordinary/send
     │                tooling       compiler path
     │
     ▼
loop fixed point
     │
     ├──────────────────────────┐
     │                          │
     ▼                          ▼
static evaluator           runtime compiler
     │                          │
     ▼                          ▼
Work + EvalControl       jump/loop lowering
     │                          │
     ├─ Break -> consume        ├─ break patch
     ├─ Continue -> consume     ├─ continue patch
     └─ Return -> propagate     └─ compatibility fallback
     │
     ▼
QueryBudget + CancellationToken
```

C4 ownership remains:

```text
P4 unordered FieldKey/Record facts ───┐
                                      │
P10 ordered/Sum classifier ───────────┼──► P5 structural capabilities/reflection
                                      │
P10 static control + explicit while ──┘
                    │
                    ▼
                  P6
         match / structural for /
        patterns / proof predicates
                    │
                    ▼
                  P7
          recursion / residuals
                    │
                    ▼
                  P8
        durable metadata/materialization
                    │
                    ▼
                  P9
       ontology/tooling/performance closure
```

The defining P10 completion rule is:

> **Plain `while` is a real semantic control construct. `whileTrue` is not its static semantic identity. Static execution uses the existing bounded Work/EvalControl machinery, while runtime lowering reuses the existing compiler loop substrate without redesigning the VM or downstream structural plans.**
