# Working on Phalcom

This guidance applies throughout the repository. Follow more specific directory
guidance when present, and follow the user's explicit task scope and instructions.

This file is intentionally model-neutral. Agents may act as principal
architects/planners, implementers, advisers, or reviewers. A task-specific
prompt determines the current role.

The repository-wide workflow is documented under `docs/workflow/`. Read only
the workflow material relevant to the current role and task; do not load every
workflow document by default.

---

## Start with the task

- Read the named plan, specification, supplied failure, handoff, and affected
  code first. Use targeted navigation such as `rg`; avoid broad repository scans
  when the relevant paths are already known.
- Check `git status --short` and the relevant diff before editing. This checkout
  may contain ongoing work from the user or other agents.
- Preserve unrelated modified, staged, and untracked files. Do not reset, clean,
  overwrite, restore, or broadly format unrelated work. Stage only the
  authorized work unit.
- A request for a plan, audit, investigation, review, or verification does not
  authorize implementation.
- Honor explicit checkpoint stops, scope limits, and delegation limits.
- Verify repository facts against the live tree. Plans and handoffs may contain
  mechanically stale paths, names, or signatures.
- Read applicable skills when useful to the task; do not load every skill or
  rebuild an existing code index for routine read-only exploration.

---

## Repository map

| Location                             | Responsibility |
|--------------------------------------| --- |
| `phalcom-ast/`                       | Lexer, parser, and syntax tree |
| `phalcom-semantic/`                  | Canonical static semantics, types, inference, semantic database, snapshots, and editor products |
| `phalcom-modules/`                   | Project and module resolution infrastructure |
| `phalcom-core/`                      | Compiler, bytecode, VM, heap, native primitives, and Universe bootstrap |
| `phalcom-repl/`                      | Command-line and REPL entry points |
| `phalcom-lsp/`                       | Language server |
| `tools/vsphalcom/`                   | VS Code extension and extension-host tests |
| `phalcom-diagnostics/`               | Shared diagnostic infrastructure |
| `phalcom-type-*`, `phalcom-native-*` | Type syntax/metadata and native declaration/surface infrastructure |
| `phalcom-test-support/`              | Shared testing support |
| `docs/specs/`                        | Normative language specification; begin with its README |
| `docs/implementation/`               | Implementation programs, checkpoints, plans, execution records, walkthroughs, and handoffs |
| `docs/workflow/`                     | Agent workflow, planning, execution, escalation, and implementation-record conventions |

---

## Specification and implementation authority

- Follow `docs/specs/README.md` for specification authority and migration rules.
  Follow topic indexes to the effective normative specification.
- A proposal, historical plan, stale test, or current implementation defect does
  not override an accepted language rule.
- Verify implementation claims against live source and relevant tests. A
  ratified specification can describe behavior that has not yet been
  implemented.
- Keep language rules, design rationale, implementation status, and future
  plans distinct. Record divergences explicitly instead of silently changing
  the rule.
- Trace source through parsing, semantic analysis, lowering, bytecode, and
  execution only as far as the task requires. Distinguish invalid fixtures from
  checker, compiler, or runtime defects.
- Keep formal semantic authority in the semantic layer. Compiler, runtime, LSP,
  and editor consumers should consume canonical products rather than
  reconstructing type, name, member, identity, or proof facts.
- Advisory observations must not strengthen formal proofs.
- Preserve stable identity and ownership boundaries. Keep inference-local
  variables out of published types and snapshots.
- Incremental changes must preserve unaffected products and diagnostics and
  agree with cold-analysis behavior.
- Prefer a focused correction at the owning layer over consumer-specific
  patches, duplicated caches, parallel sources of truth, or special-case
  workarounds.
- Avoid unrelated refactors and speculative abstractions.

---

## Workflow documents and role-specific guidance

The detailed workflow lives under `docs/workflow/`.

The canonical files include:

- `implementation-record-lifecycle-convention.md` — implementation-program,
  checkpoint, plan, walkthrough, handoff, and verification-record rules.
- `sol-principal-architect-project-doctrine.md` — principal
  architect/specifier/planner doctrine.
- `luna-patch-grade-plan-schema.md` — required shape of Luna-oriented
  patch-grade implementation plans.
- `luna-and-premium-adviser-prompts.md` — implementer and higher-end adviser
  role prompts.
- `shared-consultation-escalation-protocol.md` — escalation triggers, incident
  packets, adviser responses, and plan-amendment protocol.

Do not duplicate those documents into task-local notes. Use this file as the
repository-wide hard contract and the workflow documents for role-specific
detail.

For a numbered implementation task, the implementer should normally read the
implementation-record convention and the active plan/handoff. A principal
architect or adviser should read the role-specific workflow document when the
task actually requires that role.

---

# Numbered implementation work

A checkpoint is a bounded acceptance objective. Multiple numbered plans may
serve the same checkpoint.

Use identifiers consistently:

```text
<PROGRAM>                 program
<PROGRAM>.C<n>            checkpoint
<PROGRAM>.C<n>.P<n>       numbered plan
T<n>                      task inside a plan
G<n>                      verification or decision gate inside a plan
```

Do not use internal `C0/C1/...` labels for plan-local phases.

When executing a numbered plan such as `LANG005.C1.P3`, read:

1. this `../../AGENTS-old.md`;
2. `docs/workflow/implementation-record-lifecycle-convention.md`;
3. the active numbered plan;
4. the owning checkpoint record;
5. the current plan handoff when continuing partially completed work;
6. predecessor walkthrough/handoff when the active plan depends on them;
7. normative specs/decisions explicitly named by the plan;
8. live source and tests named by the plan.

Do not re-audit already-established work merely because a fresh session began.
If a handoff says an invariant or interface is established and the live tree
agrees, treat it as established.

The checkpoint record is the shared durable state for all plans in the
checkpoint. Do not create a competing plan-specific state document when
checkpoint-wide state belongs in `CHECKPOINT.md`.

---

## Plan authority and repository drift

A patch-grade plan is an implementation contract, not a frozen snapshot of the
repository.

Plans should distinguish, explicitly or by clear intent:

- **FIXED** — architecture, semantics, ownership, identity, lifetime, and other
  decisions the implementer must preserve.
- **MECHANICALLY FLEXIBLE** — helper names, private decomposition, equivalent
  APIs, imports, or other local details that may adapt to the live tree.
- **VERIFY-FIRST** — repository assumptions that must be checked before use.

If a plan path, helper, or private signature drifted mechanically, adapt
locally.

If repository evidence invalidates the architecture, ownership model,
observable semantics, or a downstream contract, do not silently adapt the
plan. Escalate under the rules below.

---

## Implementation authority

An implementer may decide local mechanical details, including:

- private helper names;
- equivalent internal APIs;
- imports;
- mechanical signature drift;
- local file decomposition;
- ordinary borrow/type/compiler cleanup;
- test fixture and plumbing changes;
- straightforward algorithmic details where architecture is already fixed.

An implementer must not silently redesign:

- ownership;
- lifetime or rooting;
- canonical identity or canonicalization;
- representation semantics;
- semantic authority;
- compiler/runtime contracts;
- GC policy;
- concurrency or liveness semantics;
- stable IDs;
- generic reification or runtime typing;
- optimizer soundness or fallback behavior;
- accepted language behavior or semantic test oracle.

When a material change in one of these areas appears necessary, stop and use
the escalation protocol.

---

# Build, stabilize, certify

Do not treat every edit as a release candidate.

## BUILD MODE

This is the default during implementation.

- Implement coherent planned slices.
- Use sparse, discriminating compiler/test feedback.
- Continue through intentionally incomplete intermediate states.
- Do not run broad suites unless the plan or evidence requires them.
- Do not repair unrelated repository failures.
- Prefer implementation progress over premature stabilization.

## STABILIZE MODE

Enter at explicit coherent gates.

- Run focused regressions.
- Run directly affected suites.
- Diagnose failures plausibly caused by the patch.
- Classify and record unrelated failures.
- Establish the plan's focused acceptance evidence.

## CERTIFY MODE

Enter only at an explicit checkpoint/release gate or user request.

- Run the named broader crate/workspace/lint/release gates.
- Promote verification status only when completed evidence supports it.

Most numbered implementation work should spend most of its time in BUILD MODE.

---

# Testing doctrine

Testing is an information-gathering operation, not a ritual.

Run the smallest test that answers the current correctness question.

Do not:

- run all tests after every edit;
- run a broad affected suite after every mechanical step;
- rerun an unchanged failure when nothing relevant changed;
- broaden testing merely because a focused test passed;
- use broad testing as a substitute for causal reasoning;
- run workspace tests or clippy repeatedly during BUILD;
- test an intentionally incomplete intermediate state unless the result is
  useful.

A focused PASS means the evidence was acquired. Continue implementation unless
the plan names a broader gate.

A plan may explicitly say `DO NOT RUN` a suite during BUILD. Honor that.

Confirm that a test filter selected actual tests. Zero selected tests are not
evidence.

Run Cargo validation commands serially when competing builds would waste
resources.

---

## Verification breadth

Use the narrowest useful level and broaden only when justified:

1. exact reproducer or new regression;
2. directly affected feature tests;
3. owning subsystem/module suite;
4. adjacent cross-layer integration;
5. broad crate/package verification;
6. workspace/release verification.

Do not automatically climb this ladder after a PASS.

Broaden only when:

- the plan names a gate;
- changed code crosses the relevant boundary;
- a failure creates a real unresolved concern;
- checkpoint/release certification requires it;
- the user explicitly asks for broader verification.

---

## Test placement

- Consult `phalcom-semantic/tests/semantic/README.md` and
  `phalcom-core/tests/README.md` for subsystem test placement and
  responsibility.
- Semantic integration tests share `tests/semantic.rs`; use the existing module
  tree. For exact test selection, inspect the full name with `-- --list` when
  necessary.
- Cargo accepts one positional filter; output capture is disabled with
  `-- --nocapture`.
- Test formal identities and proofs in semantic tests.
- Test compiler projection/lowering in compiler tests.
- Test runtime behavior in VM/core tests.
- Choose the lowest VM bootstrap tier that actually supplies the tested
  behavior; source-language tests generally need the full Universe.
- Add regression coverage for behavior changes.
- Do not add tests that merely mirror implementation details.
- Do not run compilation/test machinery for prose-only edits.
- For extension changes, use the scripts in `tools/vsphalcom/package.json` and
  the extension CI lane; extension-host tests require a working language
  server.

---

# Debugging budget

The goal is to prevent edit/test thrashing.

## Mechanical failures

Examples include missing imports, obvious local type mismatches, borrow-checker
cleanup, renamed helpers, and incorrect test selectors.

Permit a small number of coherent correction cycles while each cycle has a
concrete cause and makes progress.

A correction cycle is:

```text
inspect the failure
identify the concrete cause
make one coherent correction
rerun the smallest useful discriminator
```

Do not make speculative random patches.

As a default, roughly three coherent mechanical cycles are enough before
reassessing whether the problem is still mechanical.

## Semantic or behavioral failures

Before a nontrivial corrective edit, establish:

```text
Observed:
Causal hypothesis:
Evidence:
Predicted effect of the correction:
Discriminating test:
```

Make at most one serious corrective attempt for the same underlying semantic
problem unless the task-specific plan explicitly grants more.

If substantially the same problem remains, stop and escalate.

## Architectural failures

Do not make speculative architecture changes.

Escalate immediately.

---

# Mandatory escalation conditions

When an architectural adviser is available, an implementer must stop editing
the affected issue and request guidance when any of the following occurs:

1. the planned architecture appears impossible or materially wrong;
2. a core plan premise is disproved by live repository evidence;
3. ownership, identity, lifetime, representation, semantic authority, or a
   cross-layer contract must change;
4. an interface relied upon by later tasks must materially change;
5. expected semantic test behavior appears incorrect;
6. the same nontrivial semantic failure survives the allowed serious
   correction;
7. two plausible fixes have materially different architectural consequences;
8. implementation requires entering a materially unexpected subsystem for an
   architectural reason;
9. a workaround or special-case architecture is becoming necessary;
10. the patch would introduce a second source of canonical
    inference/resolution/identity/state;
11. correctness in GC, concurrency, stable IDs, generic reification, optimizer
    soundness, incremental semantics, or VM control flow is nonlocal or
    uncertain;
12. accepted specification, plan, checkpoint state, live code, and/or tests
    materially conflict;
13. an invariant would need to be weakened to make tests pass;
14. scope becomes materially larger or more cross-cutting than planned.

A triggered escalation condition is not self-waivable by the implementer.

Use `docs/workflow/shared-consultation-escalation-protocol.md`.

---

## Consultation efficiency

Before escalating, collect cheap evidence yourself:

- plan/task;
- required invariant;
- expected architecture;
- minimal reproducer;
- relevant symbols and call path;
- scoped diff;
- observed facts;
- implementer root-cause hypothesis;
- alternative hypotheses when material;
- serious attempt already made;
- exact escalation trigger;
- precise decision requested.

Do not ask a higher-end adviser to perform routine implementation or rediscover
the whole repository when focused evidence is available.

The adviser should normally resolve architecture/root cause and return execution
to the implementer rather than writing the routine patch.

After an advisory decision:

- record the durable decision in the checkpoint;
- amend the plan when required;
- resume implementation;
- do not re-consult for mechanical consequences of the same clear decision.

---

# Baseline and unrelated failures

A failing test outside the active focused surface is not automatically active
work.

Classify unexpected failures:

```text
A — definitely caused by the current change
B — probably caused by the current change
C — unclear
D — clearly unrelated or baseline
```

Policy:

- **A/B:** active responsibility; investigate and fix.
- **C:** perform one bounded classification pass. If still unclear, nonblocking,
  and outside active acceptance, record it and continue.
- **D:** record and continue immediately.

Do not spend large amounts of compute fixing D during an unrelated plan.

Do not consult a premium adviser about D unless it blocks proof of the active
implementation.

Do not weaken assertions, skip failing tests, or change expected behavior merely
to make a broad gate green.

When a suspected baseline must be confirmed, use a clean predecessor or other
non-destructive comparison without disturbing the current checkout.

---

# Implementation records and lifecycle

Follow
`docs/workflow/implementation-record-lifecycle-convention.md`
and the active `docs/implementation/` hierarchy.

A checkpoint is a bounded acceptance objective. Multiple plans may belong to
one checkpoint when follow-up or corrective work is required to satisfy the
same acceptance objective.

The checkpoint record should own:

- plan ledger;
- established architecture and invariants;
- stable takeover interfaces;
- meaningful verification evidence;
- deferred/baseline failures;
- consultations and amendments;
- active plan and next action.

Do not turn the checkpoint into a chronological log of every edit or test.

## At plan start

- confirm the checkpoint record exists;
- mark or confirm the active plan;
- record the starting state/revision when useful;
- establish the intended focused baseline;
- consume the current handoff rather than rediscovering established work.

## During a plan

Update checkpoint state only for meaningful durable events:

- a stable interface or invariant is established;
- a consultation or amendment occurs;
- a meaningful deferred failure is discovered;
- a coherent verification gate completes.

## At plan completion

Update:

- plan status;
- completion state;
- verification state;
- final durable invariants;
- residual/deferred failures;
- consultations/amendments;
- next action.

Then create the normal per-plan completion records:

```text
<PLAN>-walkthrough.md
<PLAN>-handoff.md
```

The walkthrough records what actually happened and what was actually verified.

The handoff records what the next implementer needs to inherit without
rediscovery.

---

## Implementation-state vocabulary

Use the repository lifecycle vocabulary truthfully.

Typical verification values are:

```text
UNVERIFIED
FOCUSED_TESTED
BASELINE_BLOCKED
RELEASE_COMPLETE
```

Keep implementation completion separate from verification strength.

```text
IMPLEMENTED != FOCUSED_TESTED
FOCUSED_TESTED != RELEASE_COMPLETE
BASELINE_BLOCKED != implementation failure
```

A plan may be correctly completed as `IMPLEMENTED` + `FOCUSED_TESTED`.

A broad unrelated failure may justify `BASELINE_BLOCKED` without invalidating
the implementation.

Do not claim a gate passed unless its completed result supports that claim.

---

# Standard validation guidance

Use the toolchain pinned in `rust-toolchain.toml`.

Before changing build settings, inspect `.cargo/config.toml` and
`.github/workflows/ci.yml`.

CI clears `RUSTFLAGS` and `RUSTC_WRAPPER`; the examples below clear local
compiler flags while retaining the pinned toolchain.

Focused examples:

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::db
RUSTFLAGS='' cargo test -p phalcom-core --test core language::algebraic_data
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus booleans
RUSTFLAGS='' cargo test -p phalcom-core --test cli-smoke
```

When a task explicitly calls for workspace or release certification, the
standard format/build/test/lint gates are:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

Do not run the release set by reflex during ordinary BUILD work.

Report canceled or timed-out checks as incomplete.

Broaden or repeat validation only when required by the plan/task or justified by
changed code, failures, or unresolved concerns.

---

# Delivery

Before finishing implementation work:

- review the scoped diff and check for accidental unrelated or whitespace-only
  changes;
- report what changed and why;
- report focused verification actually performed;
- report tests deliberately deferred;
- report consultations and plan amendments;
- report residual/baseline failures;
- report the exact completion and verification classification;
- update checkpoint state;
- create required walkthrough/handoff records.

A clean or pushed tree is delivery evidence, not behavioral certification.

Commit and push when requested. Keep related source, fixtures, and expected
outputs in one coherent work unit; do not stage files merely because they share
an extension.

Use `feat/`, `fix/`, `docs/`, `test/`, etc. for new branches unless instructed otherwise.

Keep progress updates brief and concrete. Do not claim a gate passed unless its
completed result supports that claim.

The repository objective is trustworthy forward progress, not maximum
verification ceremony per edit.
