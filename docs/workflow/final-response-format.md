# Mandatory Final Response Format

At the end of the task, respond using exactly this structure.

## Result

```text
Status: COMPLETE | PARTIAL | BLOCKED
Completion: NOT_STARTED | PARTIAL | IMPLEMENTED
Verification: UNVERIFIED | FOCUSED_TESTED | BASELINE_BLOCKED | RELEASE_COMPLETE
```

One short paragraph stating what was accomplished and whether the requested scope is fully complete.

## Implemented

List only material implementation changes.

```text
- <component/file>: <what changed and why>
- <component/file>: <what changed and why>
```

Do not narrate routine edits, imports, formatting, or mechanical churn.

If nothing was implemented:

```text
- None.
```

## Architecture / Invariants

List only durable architectural facts established, changed, or confirmed by this work.

```text
- <invariant or architectural decision>
- <ownership/interface/lifetime/identity rule established>
```

If none changed:

```text
- No architectural changes; implementation followed the existing plan.
```

## Verification

Report only verification actually executed.

```text
PASS
- `<command or test>` — <what it proves>

FAIL / INCOMPLETE
- `<command or test>` — <result and relevance>
```

Do not claim tests that were not run.

Do not treat a zero-test filter as a PASS.

## Deliberately Deferred Verification

List meaningful tests or gates intentionally not run under the testing budget.

```text
- `<suite/gate>` — deferred because <reason>
```

If nothing relevant was deferred:

```text
- None.
```

## Deferred / Baseline Issues

List failures discovered but intentionally left outside the active scope.

```text
- <issue> — classification: C | D — <why it does not block this task>
```

If none:

```text
- None.
```

## Consultation / Plan Amendments

If consultation occurred:

```text
- Trigger: <why escalation was required>
- Decision: <architectural/root-cause decision received>
- Amendment: <none | concise description>
```

If none:

```text
- No consultation or material plan amendment required.
```

## Implementation Records Updated

Report the durable records actually updated.

```text
- `[PROGRAM].[CHECKPOINT].[PLAN]-CHECKPOINT.md` — <state recorded>
- `<PLAN>-walkthrough.md` — <created/updated>
- `<PLAN>-handoff.md` — <created/updated>
```

Omit walkthrough/handoff entries when the task did not reach a plan-completion boundary.

## Remaining Work

If the requested scope is complete:

```text
- None within this task's scope.
```

Otherwise list the exact remaining work:

```text
- <next unfinished task>
- <blocker or dependency>
```

## Next Action

Give exactly one recommended next action.

Examples:

```text
Continue with `<next task/plan>`.
```

or:

```text
Obtain architectural guidance for the reported escalation before implementation continues.
```

or:

```text
Proceed to checkpoint certification when broader verification is desired.
```

## Final-response rules

- Be concise.
- Report facts, not a chronological work log.
- Do not repeat the plan.
- Do not include routine debugging history.
- Do not claim verification that did not occur.
- Do not conceal deferred or baseline failures.
- Do not inflate `FOCUSED_TESTED` into `RELEASE_COMPLETE`.
- If blocked by a mandatory consultation trigger, include the consultation packet before this final summary.
- The checkpoint/documentation state must already be updated before reporting completion.