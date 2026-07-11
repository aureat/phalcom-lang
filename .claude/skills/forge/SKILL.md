---
name: forge
description: >
  Spec-anchored, adversarially-verified pipeline for planning and implementing the
  Phalcom language. Use when the task is to review the current implementation, find
  bugs/weaknesses/spec-deviations, plan the next slice of work, or implement a spec
  feature cleanly. Orchestrates six role agents (stabilizer, auditor, verifier,
  architect, implementer, reviewer) with a green-build gate at every phase. Trigger:
  /forge. Invoke this skill before doing forge-style review/implementation work.
---

# /forge — the Phalcom implementation method

A pipeline that keeps the implementation **correct and clean** by separating *deciding*,
*writing*, and *approving* into different agents, each grounded in an external source of
truth (`docs/spec/`, `docs/adr/`) rather than its own judgment. A single agent that
reviews and implements its own work compounds errors; this design makes each phase check
the last.

## Ground rules (apply to every phase and every agent)
- **Spec is source of truth.** Every finding, plan unit, and diff cites a `docs/spec/` §
  or an ADR. A change with no spec coverage requires a *new* ADR (use `documentation-and-adrs`).
- **Orient before reading.** `graphify-out/graph.json` exists → agents run `graphify
  query/explain/path/affected` before raw file reads. Intent/"why" → `mem-search`.
- **Writer ≠ approver.** The implementer never approves its own diff.
- **No merge on red.** `cargo build && cargo test && cargo clippy --workspace` is the gate
  at every step. The verification substrate (invariant harness + golden `.ph` corpus +
  snapshots + fuzz) is stood up in Phase 0 and gates everything after.
- **Adversarial verification.** Every audit finding and every load-bearing design claim is
  refuted-tested before it drives work.

## Phases → agents

| Phase | What | Agent(s) | Model | Effort |
|---|---|---|---|---|
| **0. Stabilize** | Green build + verify substrate (invariants, golden corpus, snapshot/fuzz/miri lanes). **Blocking — do first.** | `phalcom-stabilizer` | sonnet | low |
| **1a. Audit** | Parallel single-lens finders: correctness-vs-spec, object-model, borrow/memory, perf/representation, diagnostics, security. | `phalcom-auditor` ×N | opus | medium |
| **1b. Verify** | Adversarial refutation of each finding; majority vote. Only survivors proceed. | `phalcom-verifier` ×(2–3/finding) | opus | high |
| **2. Plan** | Dependency-ordered, forward-looking plan → `docs/forge/PLAN.md`. Flags BLOCKED-ON-DECISION items. | `phalcom-architect` | opus | xhigh |
| **3a. Implement** | One planned unit end to end (code + tests), worktree-isolated if parallel. | `phalcom-implementer` | opus | medium |
| **3b. Review** | Independent adversarial diff review; approve only on green + spec satisfied. | `phalcom-reviewer` | opus | high |
| **4. Register** | Optimization / DX / speed / security ideas → `docs/forge/DEFERRED.md`, not into v1. | (implementers append) | — | — |

## Orchestration

1. **Always run Phase 0 first if the build is red.** Nothing downstream can verify on a
   red tree. Confirm `cargo build` before auditing or implementing.
2. **Phase 1: fan out.** Spawn one auditor per lens *in parallel* (one message, multiple
   Agent calls). Each is scoped to one spec doc + a graphify subgraph. Collect findings.
3. **Verify before trusting.** For each non-trivial finding, spawn 2–3 verifiers; keep only
   `confirmed`. Bias toward dropping uncertain findings — they resurface cheaply.
4. **Phase 2: one architect** synthesizes confirmed findings + the spec's own *Recommended
   implementation order* (already in `implementation-status.md`) into the plan. Surface every
   BLOCKED-ON-DECISION item to the user before implementing — do not pick their design.
5. **Phase 3: pipeline per unit** — implement → review. Each unit gates on green verify AND a
   reviewer `approve`. Use `git` worktree isolation when running units concurrently so they
   don't clobber each other.
6. **Phase 4 is passive:** implementers file deferred ideas as they go; surface the register
   to the user at the end as a ranked backlog — the "detailed suggestions" deliverable.

## When to use which slice
- "Review the code / find bugs" → Phases 1–2 (skip 0 if already green).
- "Implement <spec feature>" → confirm it's in the plan, then Phase 3 for that unit.
- "It doesn't build" / "set up the harness" → Phase 0.
- "Plan the whole thing" → Phases 1→2, then present the plan and BLOCKED-ON-DECISION list.

## Why staged
The tree is a Wren/clox-style VM; the spec is Smalltalk-semantics — most of the spec is
greenfield, and the build is currently red. A naive "review and implement" agent on that
substrate produces confident nonsense. Phase 0 buys the ability to verify; 1–2 buy grounded
intent; 3–4 keep v1 clean while capturing every improvement idea.
