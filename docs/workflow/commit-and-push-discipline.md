# Commit and Push Discipline

When the user has authorized commits/pushes for the task, treat Git commits as **durable implementation checkpoints**, not as a ceremony after every small edit.

## When to commit

Commit at coherent milestones such as:

- a planned task or meaningful implementation slice is complete;
- a stable interface/invariant has been established;
- a coherent focused verification gate has passed;
- an adviser-directed correction has been implemented and verified;
- the numbered plan is complete.

Do **not** create commits for:

- trivial intermediate edits;
- known-broken/incomplete mechanical states unless explicitly useful for handoff;
- formatting-only churn mixed into implementation;
- unrelated repository cleanup.

Prefer a few coherent commits over many tiny commits.

## Before committing

Before each commit:

1. inspect `git status --short`;
2. inspect the scoped diff;
3. ensure unrelated user/agent changes are excluded;
4. ensure the relevant checkpoint record is current;
5. run only the focused verification appropriate to that milestone;
6. stage only files belonging to the coherent work unit.

Do not run broad verification merely because a commit is about to be created.

## Commit contents

Keep logically coupled work together:

```text
implementation
+ directly associated tests
+ required fixtures/expected outputs
+ checkpoint-state update
```

When a walkthrough/handoff is part of plan completion, include those completion records with the final plan commit or another clearly coherent documentation commit.

Do not stage files merely because they share an extension or directory.

## Commit messages

Use concise messages describing the implemented behavior or architectural milestone.

Prefer:

```text
lang005: lower exact product runtime metadata
lang005: complete C2 product reification integration
semantic: preserve applied-type identity through lowering
```

Avoid vague messages such as:

```text
fix stuff
updates
more changes
wip
```

Follow an existing repository commit-message convention when one is clearly established.

## Push behavior

If the user has authorized pushing, push after a coherent successful commit rather than after every edit.

If several small milestones are being completed rapidly, it is acceptable to accumulate a small sequence of coherent commits and push them together.

Push promptly when:

- a meaningful plan milestone is complete;
- the user needs the state available to another agent;
- an architectural escalation/handoff requires the current work to be durable;
- the numbered plan completes.

Do not push a known-broken state unless explicitly requested or needed as an intentional handoff point.

## Safety rules

Do not:

- commit unrelated existing modifications;
- reset, clean, restore, or discard user/agent work;
- amend, squash, rebase, force-push, or rewrite history unless explicitly authorized;
- change branches unexpectedly;
- create a new branch unless requested or repository workflow requires it;
- use `--no-verify` to bypass repository checks unless explicitly authorized.

If a commit or push fails for a mechanical reason, fix the mechanical issue and retry.

If Git state is ambiguous or performing the requested operation risks overwriting/diverging from existing work, stop and report the exact repository state rather than guessing.

## Documentation synchronization

The checkpoint record must describe the state represented by the commit.

Do not create a commit that materially advances implementation while leaving `[PROGRAM].[CHECKPOINT].[PLAN]-CHECKPOINT.md` stale.

At plan completion, ensure the final committed state includes the required checkpoint update, walkthrough, and handoff before reporting the plan complete.

## Final response

When commits/pushes were authorized, include:

```text
Git
- Commit: `<sha> <subject>`
- Push: `<remote>/<branch>` — succeeded | not requested | failed
```

If multiple commits were produced, list only the coherent commits created for the task.