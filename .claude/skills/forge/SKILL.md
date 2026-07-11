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

## Orchestrator = senior architect (you)

You — the top-level agent running `/forge` — are the senior developer and architect. You
**delegate; you do not do the heavy reading or writing yourself.** Your value is judgment,
sequencing, and coordination, not tokens spent slurping source into your own context.

**Run this session on Opus, high effort.** The orchestrator makes the widest-blast-radius
decisions (wave partitioning, sequencing, conflict re-partition, adjudicating
BLOCKED-ON-DECISION items) — never economize on capability here. High effort is cheap in
absolute tokens *because* the context stays lean (subgraphs + compact returns, never files),
so you get deep coordination judgment without paying for it. xhigh is unnecessary: the single
deepest task, plan synthesis, is already delegated to `phalcom-architect` at xhigh — only
bump to xhigh on a turn where you do that synthesis inline yourself.

Cost & context discipline (non-negotiable):
- **Never read raw source into your own context to "understand" it.** Use `graphify
  query/explain/affected` for structure and delegate all file reading to subagents. What
  enters your context is a subagent's compact structured return — never a file dump.
- **Persist, then forget.** Durable outputs live on disk, not in your context: confirmed
  findings + plan → `docs/forge/PLAN.md`; running status → `docs/forge/STATE.md`; deferred
  ideas → `docs/forge/DEFERRED.md`; cross-session "why" → memory (a one-line pointer in
  `MEMORY.md`, full rationale via claude-mem). Once written, drop the detail and reload the
  pointer on demand.
- **Hold only the working set:** the current wave's unit table (compact), open
  BLOCKED-ON-DECISION items, and pointers. Everything else is a `graphify`/doc lookup away.
- **One structured hop per delegation.** Give each subagent a tight brief + the exact spec §
  and graph scope it needs; require a schema-shaped return. Don't relay raw tool output.
- Prefer many cheap scoped subagents over one expensive wide one; batch parallel spawns in a
  single message.

## Independent parallelism (wave scheduling)

Parallelism must be *interference-free*, not merely concurrent. The architect annotates every
plan unit with a **write-set** (the exact files/modules it may modify) and its **dependency
edges**. From that you schedule **waves**:
- A wave = units whose dependencies have already landed AND whose write-sets are **pairwise
  disjoint**. Only disjoint-write units run together.
- Each parallel implementer runs in its **own git worktree** (`isolation: worktree`) so
  concurrent edits never collide on disk.
- **Integrate sequentially between waves:** merge each finished unit, re-run the green gate
  (`build && test && clippy`), update `docs/forge/STATE.md`, then launch the next wave.
- If an implementer finds it must touch a file outside its write-set, it STOPS and reports a
  conflict — you re-partition rather than let two agents fight over a file.
- **Foundational units are serialized on the critical path** (selector redesign, then
  blocks): everything depends on them, so they land alone before the wide waves fan out.

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
5. **Phase 3: wave-scheduled pipeline.** Group confirmed plan units into waves of
   dependency-satisfied, pairwise-disjoint-write-set units (see *Independent parallelism*).
   For each wave: spawn one **worktree-isolated** implementer per unit in parallel, each
   followed by an independent reviewer; gate every unit on green verify AND a reviewer
   `approve`; integrate the wave sequentially, re-run the gate, and update
   `docs/forge/STATE.md`; then the next wave. Foundational units run alone first.
6. **Keep your own context lean.** After each phase/wave, write results to disk
   (`PLAN.md`/`STATE.md`/`DEFERRED.md`) + a `MEMORY.md` pointer, then drop the detail — you
   reconstruct state from those files + `graphify`, not from a bloated transcript.
7. **Phase 4 is passive:** implementers file deferred ideas as they go; surface the register
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
