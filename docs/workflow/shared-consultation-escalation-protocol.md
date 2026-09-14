# Phalcom Shared Escalation & Consultation Protocol

> Purpose: stable interchange protocol between an implementation agent and a scarce higher-end architectural adviser.  
> Goal: make consultations rare, mandatory when genuinely needed, compact, evidence-rich, and decision-oriented.

---

## 1. Core principle

Consultation exists to resolve high-leverage uncertainty.

It is not:

- a replacement for ordinary implementation effort;
- a routine code review after every task;
- an excuse to send the adviser the entire repository;
- a retry mechanism for every compiler error;
- a way to avoid local diagnosis.

The implementer performs cheap evidence collection.

The adviser performs expensive judgment.

---

## 2. Consultation states

Implementation operates in three states.

### GREEN — autonomous execution

Conditions:

- architecture matches the plan;
- failures are mechanical/local;
- expected files/interfaces exist or have obvious mechanical drift;
- focused tests behave predictably.

Action:

> Continue without adviser.

### AMBER — bounded diagnosis

Conditions:

- behavior is unexpectedly wrong;
- more than one local causal explanation is plausible;
- a focused test fails for a non-mechanical reason;
- repository drift is significant but architecture may still be intact.

Action:

> Diagnose before editing. Form a causal hypothesis. Make at most the allowed serious correction. If the same underlying semantic problem remains, enter RED.

### RED — mandatory consultation

Conditions include the global triggers below.

Action:

> Stop editing. Build a consultation packet. Consult. Record the resulting decision before resuming.

---

## 3. Global RED triggers

Consultation is mandatory when:

1. the planned architecture appears impossible or materially incorrect;
2. a core plan premise is disproved by live repository evidence;
3. ownership must change;
4. canonical identity/canonicalization must change;
5. lifetime/rooting must change;
6. representation semantics must change;
7. semantic authority moves between layers;
8. a compiler/runtime/semantic contract relied on by later work must change;
9. the expected semantic test oracle appears wrong;
10. the same nontrivial semantic failure survives one serious corrective attempt;
11. two plausible fixes have different architectural consequences;
12. implementation requires a materially unexpected subsystem;
13. a workaround/special-case architecture is about to be introduced;
14. a second source of canonical truth/inference/resolution/identity would be created;
15. GC, concurrency, stable IDs, generic reification, optimizer soundness, incremental invalidation, or VM control behavior is uncertain and nonlocal;
16. accepted spec, plan, current code, or tests materially conflict;
17. passing tests appears to require weakening a stated invariant;
18. patch scope becomes materially more cross-cutting than planned.

A RED condition cannot be self-waived by the implementer.

---

## 4. Non-triggers

Do **not** consult merely because:

- a Rust import is missing;
- the borrow checker rejects a local implementation;
- a helper was renamed;
- a private signature mechanically changed;
- a test filter selected zero tests;
- formatting fails;
- a fixture path moved;
- an obviously unrelated test fails;
- a broad test suite exposes a pre-existing failure;
- implementation is tedious;
- the implementer wants reassurance.

Use local reasoning first.

---

## 5. Debugging budgets before RED

### Mechanical problems

Maximum: roughly three coherent correction cycles while each cycle has a specific cause and shows progress.

### Semantic problem

Maximum: one serious correction for the same underlying causal hypothesis.

Before editing:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

If the prediction fails and the same issue persists, RED.

### Architectural problem

Maximum speculative architecture attempts: zero.

RED immediately.

---

## 6. Implementation Incident Packet

Use this exact structure unless a field is genuinely inapplicable.

```markdown
# IMPLEMENTATION INCIDENT

## 1. Identity

- Program:
- Checkpoint:
- Plan:
- Task/Gate:
- Repository:
- Branch:
- Starting revision:
- Current revision:

## 2. Escalation trigger

Which mandatory trigger fired?

State it explicitly.

## 3. Required invariant

What must remain true?

Reference plan invariant IDs where possible.

## 4. Expected architecture

What does the active plan require?

Keep this to the relevant architecture only.

## 5. Observed behavior

What actually happened?

Distinguish symptom from inference.

## 6. Minimal reproduction

Exact command/test/input:

```sh
...
```

Observed output:

```text
...
```

Expected output/behavior:

```text
...
```

## 7. Relevant code path

List only the important files/symbols and the flow between them.

```text
A::foo
  -> B::bar
  -> C state
```

Include compact code excerpts only when needed.

## 8. Scoped implementation diff

Summarize current changes.

If attaching/providing a diff, scope it to the active work.

## 9. Evidence

Facts observed directly from:

- code;
- debugger/logging;
- tests;
- semantic products;
- bytecode;
- runtime state.

Do not mix guesses into this section.

## 10. Implementer's causal hypothesis

What currently seems to explain the failure?

## 11. Alternative hypotheses

What else plausibly explains it?

## 12. Attempts already made

For each serious attempt:

- change;
- prediction;
- observed result.

Do not list trivial typo/import fixes.

## 13. Why local authority is insufficient

Explain which architecture/semantic boundary requires adviser judgment.

## 14. Decision requested

Ask the narrowest useful question.

Good:

> Should persistent import binding ownership live in the workspace session or per-evaluation environment given facts A/B/C, and what state must remain ephemeral?

Bad:

> Why doesn't this work?
```

---

## 7. Packet quality rules

A good packet:

- is small;
- is reproducible;
- separates evidence from inference;
- identifies the violated invariant;
- includes the relevant diff;
- contains the implementer's best hypothesis;
- asks one architectural/root-cause decision.

A bad packet:

- dumps thousands of lines without prioritization;
- asks the adviser to rediscover the repository;
- omits the current diff;
- omits the expected architecture;
- gives only a failing test name;
- asks for code before diagnosis;
- mixes unrelated failures.

---

## 8. Adviser Response Contract

The adviser should return:

```markdown
# ADVISORY DECISION

## 1. Root cause

...

## 2. Evidence / confidence

...

## 3. Architecture verdict

One of:

- PLAN ARCHITECTURE SOUND
- PLAN NEEDS CLARIFICATION
- PLAN NEEDS MATERIAL AMENDMENT
- IMPLEMENTATION BUG ONLY
- TEST/ORACLE BUG
- NORMATIVE CONFLICT REQUIRES HUMAN/ARCHITECT DECISION

## 4. Required correction

Dependency-ordered implementation direction.

## 5. Invalidated assumptions

...

## 6. Invariants to preserve

...

## 7. Forbidden shortcuts

...

## 8. Focused verification required

Smallest discriminating tests.

## 9. Plan amendment

Required: YES | NO

If YES:
- original assumption;
- why false;
- revised architecture;
- tasks affected;
- tests affected.

## 10. Resume decision

- IMPLEMENTER MAY RESUME
or
- IMPLEMENTER SHOULD REMAIN STOPPED
```

---

## 9. Adviser follow-up requests

The adviser may request additional evidence only when necessary for the decision.

Prefer:

```text
show symbol X and its call sites
show owner/lifetime Y
run one discriminator Z
show generated bytecode for case Q
```

Avoid broad “send more repository” requests.

---

## 10. After advisory decision

The implementer must:

1. stop treating its pre-consultation hypothesis as authoritative;
2. extract the adopted decision;
3. update the checkpoint consultation/amendment ledger;
4. amend the active plan if required by policy;
5. implement the correction;
6. run the focused verification named by the adviser/plan;
7. resume normal debugging budgets.

Do not repeatedly consult on mechanical consequences of an already-clear advisory decision.

---

## 11. Plan amendment levels

### Level 0 — no amendment

Implementation bug only.

Record consultation in checkpoint/walkthrough.

### Level 1 — clarification

Architecture remains intact but an ambiguity was resolved.

Record a concise amendment note and continue.

### Level 2 — material amendment

Ownership, identity, semantics, task dependency, or acceptance changes.

Principal architect should amend the plan before broad continuation.

### Level 3 — new corrective plan

The new work is coherent, material, and too large to append safely.

Create a new `P<n+1>` in the same checkpoint if it still serves the same acceptance objective.

---

## 12. Premium-compute optimization rules

1. Prefer one high-quality consultation over repeated weak ones.
2. Do not consult before evidence collection.
3. Do not send entire unchanged files when a relevant excerpt/symbol suffices.
4. Ask for architecture/diagnosis, not mechanical implementation.
5. Reuse the advisory decision through checkpoint/handoff records.
6. Do not ask a second higher-end model to independently re-solve ordinary incidents.
7. Use a second premium opinion only for unusually consequential, unresolved, or contested decisions.
8. Never spend adviser compute on clearly unrelated baseline failures.

---

## 13. Human escalation

The adviser should explicitly defer to the user/principal architect when:

- two semantics are both plausible language-design choices;
- accepted normative documents conflict;
- a proposed fix changes user-visible language behavior not already ratified;
- scope expansion materially changes the checkpoint objective;
- a high-impact architecture decision lacks enough evidence to choose safely.

The implementer remains stopped on that architectural fork while unrelated independent planned work may continue only if it does not depend on the unresolved decision.

---

## 14. Repository record

Checkpoint consultation entry should be concise:

```text
INC-<n>
Plan/task:
Trigger:
Observed:
Decision:
Architecture impact:
Plan amendment:
Verification required:
Status:
```

The walkthrough should mention the incident when it materially changed implementation.

The handoff should include only decisions still relevant to future work.
