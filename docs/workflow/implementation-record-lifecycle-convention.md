# Phalcom Implementation Record & Work-Lifecycle Convention

> Status: proposed canonical workflow contract  
> Intended location: `docs/implementation/` (either as an expansion of `README.md` or as a normative companion referenced by it)  
> Exemplars: `CONC002` checkpoints C1–C3  
> Scope: implementation-program organization, checkpoint state, numbered plans, execution records, walkthroughs, handoffs, verification state, and corrective follow-up work

---

## 1. Purpose

`docs/implementation/` is the durable execution memory for Phalcom implementation work.

Its records must allow a fresh architect, implementer, adviser, or reviewer to answer—without reconstructing prior sessions:

1. What bounded objective is currently being implemented?
2. What architecture and invariants have already been established?
3. Which numbered plans contributed to the checkpoint?
4. What is implemented, what is merely planned, and what is actually verified?
5. What tests were run, and what tests were deliberately not run?
6. What unresolved failures were deferred rather than repaired?
7. What architectural consultations occurred and what decisions resulted?
8. What must the next plan inherit?
9. Where is the single current source of truth for checkpoint state?

This convention is intentionally optimized for agentic implementation. It must prevent rediscovery, contradictory state files, architectural drift, repeated testing, and session-to-session loss of context.

---

## 2. Canonical hierarchy

The intended hierarchy is:

```text
docs/implementation/
  <category>/
    <PROGRAM>-<program-name>/
      PROGRAM.md
      STATUS.md
      C1-<checkpoint-name>/
        CHECKPOINT.md
        STATUS.md                    # optional if useful; CHECKPOINT remains authoritative
        <PROGRAM>.C1.P1-<plan-name>.md
        <PROGRAM>.C1.P1-walkthrough.md
        <PROGRAM>.C1.P1-handoff.md
        <topic>-spec.md              # only genuine technical/formal companion specs
        <PROGRAM>.C1.P2-<plan-name>.md
        <PROGRAM>.C1.P2-walkthrough.md
        <PROGRAM>.C1.P2-handoff.md
```

Existing legacy program layouts do not need opportunistic reorganization during unrelated implementation work. Apply this convention to new work and normalize older families when that work is explicitly in scope.

### 2.1 Stable identifiers

Identifiers are dot-qualified:

```text
Program:     LANG005
Checkpoint:  LANG005.C1
Plan:        LANG005.C1.P3
Task:        LANG005.C1.P3.T4
Gate:        LANG005.C1.P3.G2
```

Use:

- `C<n>` only for repository checkpoints.
- `P<n>` only for numbered implementation plans inside a checkpoint.
- `T<n>` for implementation tasks inside a plan.
- `G<n>` for verification or decision gates inside a plan.

Do **not** use internal `C0`, `C1`, `C2` phase labels inside a plan. That collides with checkpoint identifiers and makes durable records ambiguous.

---

## 3. Conceptual ownership

### 3.1 Program

A program owns a coherent implementation concern over time.

A program record answers:

- Why does this program exist?
- What subsystem or semantic concern does it own?
- Which checkpoints compose it?
- What is its current lifecycle state?

A program is not a session, release, or temporary working directory.

### 3.2 Checkpoint

A checkpoint is a **bounded acceptance objective**.

This is the most important unit in the implementation hierarchy.

Multiple plans may belong to one checkpoint when:

- the checkpoint is too large for one safe execution plan;
- later corrective work is discovered;
- repository drift invalidates part of an earlier plan;
- implementation reveals a bounded architectural closure that still belongs to the same acceptance objective;
- testing exposes missing work required to honestly accept the checkpoint.

Do not create a new checkpoint merely because a first implementation attempt was incomplete.

### 3.3 Plan

A plan is a bounded executable work package that advances one checkpoint.

A plan is not the durable source of truth for checkpoint state. It describes intended work.

### 3.4 Walkthrough

A walkthrough records what the plan **actually implemented and proved**.

It is retrospective and evidence-oriented.

### 3.5 Handoff

A handoff gives the next plan/agent the smallest complete operational context needed to continue without rediscovery.

It is prospective.

### 3.6 Checkpoint record

The checkpoint record is the **single shared durable state document** for all plans in the checkpoint.

A checkpoint must not accumulate multiple independently-maintained “implementation-state” files that compete with it.

---

## 4. Required lifecycle metadata

Every canonical program, checkpoint, and plan should use the repository lifecycle vocabulary:

```yaml
status: PROPOSED | IN_PROGRESS | BLOCKED | DEFERRED | COMPLETE | SUPERSEDED | ABANDONED
completion: NOT_STARTED | PARTIAL | IMPLEMENTED
verification: UNVERIFIED | FOCUSED_TESTED | BASELINE_BLOCKED | RELEASE_COMPLETE
```

These axes are intentionally separate.

### 4.1 `status`

Describes work lifecycle.

### 4.2 `completion`

Describes whether the intended implementation exists.

### 4.3 `verification`

Describes the strength of evidence.

The following distinctions are mandatory:

```text
IMPLEMENTED      != FOCUSED_TESTED
FOCUSED_TESTED   != RELEASE_COMPLETE
BASELINE_BLOCKED != implementation failure
COMPLETE         must not be inferred from a clean Git tree
```

A plan may legitimately finish as:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

when its required feature behavior is proven but full-repository certification is intentionally deferred.

---

## 5. `CHECKPOINT.md` — canonical shared state

Every active checkpoint must have exactly one canonical checkpoint record.

Recommended metadata:

```yaml
---
id: LANG005.C1
category: LANG
program: LANG005
checkpoint: LANG005.C1
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
requires:
  - <prerequisite>
---
```

The body should contain the following sections.

### 5.1 Objective and ownership boundary

State:

- the checkpoint's bounded acceptance objective;
- what it owns;
- what it explicitly does not own.

### 5.2 Plans ledger

Example:

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG005.C1.P1` | ... | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | ... |
| `LANG005.C1.P2` | ... | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | ... |
| `LANG005.C1.P3` | ... | IN_PROGRESS | PARTIAL | UNVERIFIED | active |

This is the checkpoint's plan index and progress ledger.

### 5.3 Established invariants

Record only durable architectural facts already accepted or implemented.

Examples:

- canonical identity owner;
- lifetime owner;
- representation rule;
- semantic authority;
- fallback behavior;
- execution ordering rule;
- optimizer soundness rule;
- threading rule.

Do not record speculative design as an established invariant.

### 5.4 Stable interface / takeover map

Record symbols, IDs, data structures, bytecodes, APIs, or files that subsequent plans inherit.

### 5.5 Verification ledger

Record evidence at coherent gates.

Do **not** turn this into a chronological log of every test invocation.

Capture:

- gate;
- relevant scope;
- command/target;
- result;
- what the result proves.

### 5.6 Deferred / baseline issue ledger

Record failures intentionally left unresolved.

For each:

```text
ID:
Observed:
Classification: unrelated | baseline | unclear-but-nonblocking
Why it does not block active acceptance:
Reproduction:
When to revisit:
```

Do not silently repair unrelated failures during a numbered plan.

### 5.7 Consultation / amendment ledger

For any architectural escalation, record:

```text
Incident:
Plan/task:
Trigger:
Decision:
Architectural consequence:
Plan amendment:
Additional tests:
```

Do not paste entire model conversations. Preserve the decision and evidence needed by future agents.

### 5.8 Active plan and next action

The checkpoint should always identify:

- active plan;
- latest completed plan;
- next planned action;
- blockers if any.

---

## 6. Checkpoint update obligations

Updating the checkpoint is part of implementation, not optional documentation cleanup.

### 6.1 At plan start

The implementer must:

- locate the owning checkpoint record;
- confirm the plan is in its ledger;
- mark the plan active/in progress;
- record the starting revision when useful;
- record inherited baseline facts if they changed;
- confirm predecessor walkthrough/handoff exists when applicable.

If no checkpoint record exists for an active numbered checkpoint, create or normalize it before substantial implementation proceeds, unless the user explicitly excludes documentation work.

### 6.2 During implementation

Update the checkpoint only at meaningful durable events:

- a task establishes an interface later tasks depend on;
- an architectural invariant becomes implemented;
- a consultation changes the plan;
- a significant deferred failure is discovered;
- a coherent verification gate completes.

Do not write checkpoint state after every edit or test.

### 6.3 At plan completion

The implementer must:

- update the plan ledger;
- record final implementation/completion/verification state;
- record significant verification evidence;
- record residual failures;
- record adopted consultations/amendments;
- set the next action.

A plan is not administratively complete until its checkpoint state is current.

---

## 7. Plan completion artifacts

Every completed numbered implementation plan should normally produce:

```text
<PLAN>-walkthrough.md
<PLAN>-handoff.md
```

Exceptions require an explicit reason (for example, terminal program closure with no successor may make a separate handoff redundant).

### 7.1 Walkthrough contract

The walkthrough answers: **What actually happened?**

Required content:

1. Plan identity and final state.
2. Objective and outcome.
3. Architecture implemented.
4. Important files/components changed.
5. Durable interfaces/invariants established.
6. Deliberate deviations from the plan.
7. Consultations and adopted architectural decisions.
8. Tests added.
9. Tests actually executed.
10. Tests deliberately deferred.
11. Performance evidence if relevant.
12. Residual/baseline failures.
13. Verification classification.
14. Any follow-up work required.

Never claim a test or verification gate passed without completed evidence.

### 7.2 Handoff contract

The handoff answers: **What does the next implementer need to know?**

Required content:

1. Current repository revision or reliable state anchor.
2. Completed prerequisites.
3. Stable interface/takeover map.
4. Inherited architecture.
5. Invariants that must not be redesigned.
6. Known repository drift.
7. Deferred failures.
8. Must-read files.
9. Next plan objective.
10. Known high-risk areas.
11. Recommended first commands/tests.
12. Explicit “do not re-explore / do not redesign” guidance.

The handoff must be compact enough to save context yet complete enough to prevent rediscovery.

---

## 8. Plan records are intent, not implementation truth

A numbered plan may contain:

- expected file paths;
- pseudocode names;
- anticipated interfaces;
- architectural assumptions.

The live source, accepted normative specifications, checkpoint amendments, and adopted consultation decisions may reveal mechanical drift.

Rules:

1. Mechanical drift may be adapted locally and recorded.
2. Architectural drift may not be silently absorbed.
3. A plan may be amended.
4. A completed walkthrough records the actual result.
5. The checkpoint record reflects the durable accepted state.

---

## 9. Corrective work

Corrective work belongs to the checkpoint whose acceptance objective it serves.

Prefer:

```text
LANG005.C1.P3
LANG005.C1.P4-corrective-closure
```

over inventing an unrelated program/checkpoint when P4 is required to make C1 truthful.

Create a new checkpoint only when the acceptance objective genuinely changes.

---

## 10. Testing records: designed breadth, executed selectivity

Implementation documentation must distinguish:

### 10.1 Coverage obligations

What correctness dimensions the plan intends to prove.

### 10.2 Executed verification

What tests were actually run during this plan.

A rich coverage design does **not** imply full-suite execution after every task.

Verification records should preserve the principle:

> Run the smallest test that answers the current correctness question.

Broad gates belong at deliberately chosen integration, checkpoint, or release boundaries.

---

## 11. Unrelated failures

Do not convert every observed failure into active scope.

Classify failures:

```text
A — definitely caused by current change
B — probably caused by current change
C — unclear
D — clearly pre-existing/unrelated
```

Policy:

- A/B: active implementation responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record and defer.
- D: record and defer immediately.
- Do not spend architectural-adviser compute on D unless it blocks proving the active work.

The checkpoint's deferred/baseline ledger is the durable home for such failures.

---

## 12. Repository hygiene

Implementation agents must:

- inspect `git status --short` before editing;
- preserve unrelated modified/staged/untracked work;
- avoid resets, cleans, broad reformatting, or broad staging;
- stage only authorized work units;
- distinguish current physical files from historical/migration artifacts;
- avoid opportunistic documentation-tree migrations during feature work.

---

## 13. Canonical record responsibilities summary

| Record | Owns | Must not become |
|---|---|---|
| `PROGRAM.md` | long-lived program purpose and checkpoint map | session log |
| `CHECKPOINT.md` | shared bounded-objective state | plan-specific scratchpad |
| numbered plan | intended implementation work | durable state authority |
| walkthrough | what actually shipped/proved | future-plan instructions |
| handoff | next-agent operational context | full implementation history |
| spec/ADR/PDR | normative/design authority as applicable | implementation progress tracker |

---

## 14. Acceptance rules

A checkpoint may be marked `COMPLETE` only when:

1. its bounded acceptance objective is implemented;
2. all required plans are complete or explicitly superseded;
3. its checkpoint record reflects actual architecture;
4. residual failures are classified;
5. verification state is truthful;
6. the next action is explicit.

`RELEASE_COMPLETE` is a stronger certification than checkpoint implementation completion and should only be used when the named broad gates actually ran successfully.

---

## 15. Exemplar doctrine

`CONC002` C1–C3 are accepted as strong exemplars for:

- shared checkpoint records;
- plans ledgers;
- invariant ledgers;
- verification evidence;
- per-plan walkthroughs;
- per-plan handoffs.

This convention hardens those practices by adding:

- strict task/gate terminology;
- mandatory checkpoint update points;
- consultation/amendment ledgers;
- deferred/baseline failure ledgers;
- sparse-testing doctrine;
- model-neutral execution-role boundaries;
- explicit separation of plan intent from checkpoint truth.
