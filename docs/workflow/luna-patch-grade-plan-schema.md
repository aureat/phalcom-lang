# Luna-Oriented Patch-Grade Implementation Plan Schema

> Purpose: normative schema for implementation plans intended primarily for GPT-5.6 Luna High/Extra High execution under higher-end architectural supervision.  
> Design goal: make architecture explicit, implementation efficient, testing selective, escalation objective, and repository state durable.

---

# `<PROGRAM>.C<checkpoint>.P<plan> — <Plan Name>`

```yaml
---
id: <PROGRAM>.C<checkpoint>.P<plan>
category: <CATEGORY>
program: <PROGRAM>
checkpoint: <PROGRAM>.C<checkpoint>
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - <prior plan/checkpoint>
follows: <prior plan or null>
supersedes: null
---
```

## 0. Executor contract

This plan is designed for a capable implementation model working under constrained architectural authority.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture and invariants below;
- must not improvise material architecture;
- must follow the testing budget rather than testing reflexively;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep the shared checkpoint record current;
- must produce the required walkthrough and handoff.

Testing is evidence gathering, not a ritual. Do not run tests merely because an edit occurred.

---

## 1. Goal

One sentence describing the plan's deliverable.

## 2. Checkpoint acceptance objective

Explain how this plan advances the bounded objective owned by `<PROGRAM>.C<checkpoint>`.

State whether this plan is:

- full checkpoint closure;
- one of several plans;
- corrective closure;
- stabilization;
- migration;
- performance/soundness hardening.

## 3. Repository grounding

Prepared against:

```text
repository:
branch:
revision:
relevant predecessor:
```

List repository facts verified during planning.

State:

> Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

## 4. Required reads before implementation

```text
- AGENTS.md
- docs/implementation/.../CHECKPOINT.md
- predecessor walkthrough
- predecessor handoff
- normative spec(s)
- accepted ADR/PDR(s)
- exact source paths
- exact relevant test conventions/readmes
```

Avoid broad exploration unless evidence makes it necessary.

---

## 5. Normative authority

List accepted language/spec/design authorities and user-ratified decisions.

For each, identify the behavior this plan depends on.

Explicitly separate:

- normative behavior;
- current implementation;
- future/non-normative proposals.

---

## 6. Takeover state

Summarize already-implemented architecture inherited by this plan.

Include a stable interface map:

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| ... | ... | ... | ... |

Do not repeat large predecessor documents. Include only what this plan needs.

---

## 7. Architecture

Describe the implementation architecture.

Use exact owners and flows.

For cross-layer work, state the direction of authority:

```text
semantic facts
    ↓ projected as stable lowering facts
compiler
    ↓ bytecode/runtime metadata
VM/runtime
```

Avoid ambiguous statements such as “make the compiler know X” if X should be canonical semantic knowledge.

---

## 8. Ownership boundaries

### Owns

- ...

### Does not own

- ...

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| ... | ... | ... | ... |

---

## 9. Global invariants

Number durable invariants `INV-01`, `INV-02`, ...

Example:

```text
INV-01 — source evaluation order remains exact.
INV-02 — unsupported optimization falls back to canonical materialization.
INV-03 — no per-instance generic-argument arrays.
```

Tests and tasks should reference these IDs.

---

## 10. Non-goals

Explicitly list tempting expansions that are outside this plan.

---

## 11. Expected impact map

### Expected source areas

- `path` — reason

### Expected tests

- `path` — reason

### Expected documentation/state

- checkpoint record
- walkthrough
- handoff

### Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem not anticipated here when that entry changes architecture or ownership.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

- architecture choice A;
- identity rule B;
- owner C;
- semantic behavior D;
- fallback E.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- helper names;
- private decomposition;
- equivalent APIs;
- file-local representation;
- mechanical signature drift.

### 12.3 VERIFY-FIRST

Verify these assumptions:

| Assumption | Where to verify | If false |
|---|---|---|
| ... | ... | adapt mechanically / STOP AND CONSULT |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing and build a consultation packet if any of the following occurs:

1. architecture appears impossible/materially wrong;
2. a core plan premise is false;
3. ownership/identity/lifetime/representation/semantic-authority boundary must change;
4. a downstream contract must materially change;
5. semantic expected behavior/test oracle appears wrong;
6. the same nontrivial semantic failure persists after one serious correction;
7. two plausible fixes have materially different architecture;
8. an unexpected subsystem is required;
9. a workaround/special-case architecture is emerging;
10. duplicate canonical reasoning/state would be introduced;
11. soundness-critical behavior is uncertain;
12. spec/code/test/plan authorities materially conflict;
13. an invariant must be weakened to pass tests;
14. scope becomes materially more cross-cutting than planned.

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

Before changing code, record:

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

Never rerun an unchanged failing test unless something relevant to the failure changed.

---

## 15. Testing surface analysis

Select applicable dimensions and tie them to invariants.

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | canonical success | INV-01 | ... |
| CV-02 | boundary | INV-02 | ... |
| CV-03 | fallback | INV-04 | ... |

Consider where applicable:

- canonical success;
- boundary cardinalities;
- negative behavior;
- lifecycle/persistence;
- identity/canonicalization;
- repetition;
- recovery after failure;
- ordering;
- aliasing;
- composition;
- fallback;
- differential oracle;
- cold/incremental equivalence;
- GC/lifetime;
- stale IDs/generations;
- reflection/type observation;
- diagnostics;
- optimizer soundness;
- concurrency/liveness.

Do not mechanically include every dimension.

---

## 16. Verification execution budget

### Modes

#### BUILD MODE

Sparse, discriminating feedback only.

#### STABILIZE MODE

Focused regressions and directly affected suites.

#### CERTIFY MODE

Broader checkpoint/release gates only when required.

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

- ...

### Mandatory at named gates

- ...

### Mandatory during STABILIZE

- ...

### Only if evidence demands

- ...

### Explicitly deferred / DO NOT RUN during BUILD

- workspace test
- workspace clippy
- broad unrelated language corpora
- other suite(s) ...

State why.

---

## 17. Baseline / unrelated failure policy

Classify:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not consume adviser compute on D unless it blocks acceptance.
- Do not weaken/skip assertions to make broad gates green.

Record deferred failures in the checkpoint ledger.

---

# 18. Tasks

Use coherent tasks rather than edit-sized microsteps.

## T1 — `<Task Name>`

### Purpose

...

### Preconditions

- ...

### Consumes

- exact interfaces/facts from prior tasks

### Produces

- exact interface/data/API that later tasks depend on

### Files and symbols

**Read:**
- `...`

**Modify:**
- `...`

**Create:**
- `...`

**Tests:**
- `...`

### Required implementation shape

Describe the architecture in enough detail for Luna to implement without inventing design.

Use code/pseudocode where precision matters.

### Forbidden approaches

- no duplicated type inference;
- no alternate identity reconstruction;
- no eager whole-suite verification;
- task-specific prohibitions.

### Test changes required

Tie each to coverage IDs.

### Tests to run now

Only discriminating commands.

If no test is useful yet, state:

> **Do not run runtime/integration tests after this task. The intermediate state is intentionally incomplete.**

### Tests explicitly deferred

- ...

### Acceptance

Concrete observable condition.

### Local STOP / CONSULT triggers

Task-specific triggers beyond the global set.

### Checkpoint update

State whether T1 establishes a durable interface/invariant worth recording now.

---

## T2 — `<Task Name>`

Repeat the complete task contract.

---

# 19. Verification gates

## G1 — `<Coherent Behavior Gate>`

Purpose: prove ...

Run:

```sh
<exact focused command>
```

Expected:

```text
...
```

Do not broaden after PASS unless G1 explicitly instructs it.

On failure, classify before editing.

---

## G2 — `<Integration Gate>`

...

---

## 20. Final focused acceptance

At plan completion, prove every required coverage obligation.

Provide a mapping:

| Coverage ID | Test | Result |
|---|---|---|
| CV-01 | ... | PASS |
| ... | ... | ... |

Run only the plan's required stabilization gates.

Do not automatically run workspace certification.

---

## 21. Performance/resource evidence

Only when the plan has performance or allocation goals.

State the specific metric and acceptable evidence.

Do not add benchmarking ceremony to non-performance plans.

---

## 22. Checkpoint bookkeeping

### At plan start

- mark plan active;
- record starting state if useful.

### During plan

Update only for:

- durable new invariant/interface;
- consultation/amendment;
- deferred failure;
- coherent verification gate.

### At plan completion

- update plans ledger;
- record completion/verification;
- record final invariants;
- record deferred failures;
- record consultations;
- set next action.

---

## 23. Walkthrough deliverable

Create:

```text
<PLAN>-walkthrough.md
```

Must contain:

- final result;
- architecture implemented;
- important code changes;
- deviations;
- consultation decisions;
- tests added;
- tests actually run;
- tests deferred;
- verification classification;
- residual risks.

---

## 24. Handoff deliverable

Create:

```text
<PLAN>-handoff.md
```

Must contain:

- current state/revision;
- inherited stable interfaces;
- invariants;
- next objective;
- must-read files;
- deferred failures;
- first recommended commands;
- things not to redesign/re-explore.

---

## 25. Completion truth table

Do not conflate:

```text
source written
tests added
targeted tests passed
affected suite passed
checkpoint accepted
release certified
```

Use repository metadata truthfully.

---

## 26. Plan self-review

Before delivery, the architect must check:

- every invariant has implementation ownership;
- every important semantic risk has coverage;
- no duplicate source of truth is planned;
- Luna has objective escalation triggers;
- debugging loops are bounded;
- testing budget is intentionally narrow;
- broad tests are not accidentally mandatory after every task;
- checkpoint/walkthrough/handoff obligations exist;
- no placeholders remain;
- signatures/names are internally consistent;
- tasks form a valid dependency order.
