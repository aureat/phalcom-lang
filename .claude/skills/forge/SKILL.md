---
name: forge
description: >
  Lean, no-idle operating method for the Phalcom language, invoked as /forge [command].
  Base /forge (alias /forge senior) asserts an escalation-ladder, no-idle,
  minimal-overthinking working discipline. Subcommands: `orchestrate` runs the spec-anchored, adversarially-verified
  implementation pipeline (stabilize/audit/verify/plan/implement/review) as a continuous
  no-idle parallel fleet; `handoff` emits a context-only continuation prompt with no
  survey; `compact` writes a lossless continuation seed and trims; `offload "<task>"`
  formulates and spawns a well-specified background subagent without waiting; `status`
  prints the work ledger; `caveman` stacks caveman terse-output mode on top of the senior
  discipline for max read/work/output minimization. Use when the user types /forge, or asks
  to review/plan/implement Phalcom, hand off, smart-compact, offload to a subagent, run
  lean/parallel without overthinking, or combine forge with caveman mode.
---

# /forge

Dispatch on the text after `/forge`:

| Argument | Section |
|---|---|
| empty · `senior` · `mode` · `on` | [§Senior](#senior) |
| `orchestrate` | [§Orchestrate](#orchestrate) |
| `handoff` | [§Handoff](#handoff) |
| `compact` | [§Compact](#compact) |
| `offload <task>` | [§Offload](#offload) |
| `status` | [§Status](#status) |
| `caveman` | [§Caveman](#caveman) |

Each subcommand is also a direct slash command (`.claude/commands/forge/`): `/forge:senior`, `/forge:orchestrate`, `/forge:handoff`, `/forge:compact`, `/forge:offload <task>`, `/forge:status`, `/forge:caveman`. Those thin commands load this skill and jump straight to the matching section — this file stays the single source of truth.

Do not narrate the dispatch. Read only the matching section, act, done.

---

## Global rule: think on demand, not by default

These are powerful models; the failure mode is *over*-thinking, not under.

- Deliberate only when it **changes your next action** — real ambiguity, an expensive-to-reverse step, or a genuine design fork. For known/mechanical steps, act.
- Spend reasoning effort on the **design/write** step, not on locating files or restating what you already know.
- A fact established earlier this session is settled. Do not re-derive, re-verify, or re-explain it.
- Never re-survey to reach a conclusion you already reached. Catch yourself opening a file to confirm something you already know → stop.

Holds across every subcommand.

---

<a id="senior"></a>
## §Senior — the operating discipline

Adopt for the rest of the session. Announce nothing; just work this way.

### 1. Escalation ladder — climb only when the rung below can't answer

1. **What I already know** + memory recall (`MEMORY.md`, claude-mem / mem-search).
2. **graphify** — `graphify query "<q>"` · `explain "<node>"` · `path "<A>" "<B>"` · `affected "<sym>"` (when `graphify-out/` exists; else `Grep`/`Glob`).
3. **Targeted `Read`** of a *named* span you will act on — never a whole-file survey to "understand."
4. **A subagent** — the most expensive move (a whole fresh context). Reserved for **parallel implementation** or an **investigation whose conclusion is compact but whose process would bloat me**.

A full-file read or a spawned subagent needs a reason the rung below couldn't serve. No reflexive sweeps.

### 2. Work ledger — make idleness visible

Two short lists in your session scratchpad, updated each turn:

```
foreground:  <my own queued tasks, in order>
outstanding: <subagent-id → deliverable / write-set>
```

Few lines, not prose. The instrument for the stop-rule.

### 3. Never-idle stop-rule

End your turn **only** when `foreground` is empty **and** everything left is blocked on `outstanding` **and** there is no preparatory work. Otherwise keep working. A running subagent is **not** a reason to stop — the harness re-invokes you on completion. Ending a turn *is* waiting; don't wait unless truly blocked or you need the user.

### 4. Pipeline rule — the anti-idle engine

When you offload, name what **you** do meanwhile. Offload the leaf for step N+1; do step N yourself now. Can't name foreground work after a spawn → you decomposed wrong: either the task is sequential (do it inline, don't spawn) or you offloaded the part you should have kept (synthesis/writing stays with you).

### 5. Subagent contract — every spawned task carries this

Front-load your known coordinates so the subagent starts at rung 3, not rung 0:

> Deliverable: `<one line>`.
> Entry points I already know: `<graphify nodes / file:line / spec § / ADR>`.
> Locate via graphify (`graphify-out/` exists — query before reading source); `Read` only the span you'll act on; **do not survey**.
> Return **exactly**: `<structured shape — a decision / diff / finding, not an exploration dump>`.
> If the graph is insufficient, say so and name what you read and why.

The subagent's final message *is* the result and is not shown to the user — demand a conclusion, not a file dump. Heavy exploration stays in the subagent's context; yours gets only the answer.

### 6. Re-entry as interrupt, not await

A completion notification **interrupts** foreground work → integrate the compact result → update the ledger → resume. The result *is* the exploration; never re-explore what a subagent found.

### Caveat — don't force fan-out

Parallelism has coordination cost. Over-parallelizing *dependent* work causes rework. Offload only genuinely-independent leaves; a cheap rung 2–3 lookup, do yourself.

---

<a id="orchestrate"></a>
## §Orchestrate — the Phalcom implementation pipeline

Spec-anchored, adversarially-verified pipeline that keeps the implementation **correct and clean** by separating *deciding*, *writing*, and *approving* into different agents, each grounded in an external source of truth (`docs/spec/current/`, `docs/adr/`) rather than its own judgment. A single agent that reviews and implements its own work compounds errors; each phase checks the last.

**This is [§Senior](#senior) applied to the pipeline.** Every rule above holds — ladder, ledger, stop-rule, subagent contract, re-entry. This section adds only what is phalcom- and pipeline-specific. Do not restate the mode; obey it.

### Ground rules (every phase, every agent)
- **Spec is source of truth.** Every finding, plan unit, diff cites a `docs/spec/current/` § or an ADR. No spec coverage → a *new* ADR (`documentation-and-adrs`).
- **Writer ≠ approver.** The implementer never approves its own diff.
- **No merge on red.** `cargo build && cargo test && cargo clippy --workspace` is the gate at every step. Verify substrate (invariant harness + golden `.ph` corpus + snapshot/fuzz/miri) stood up in Phase 0, gates everything after.
- **Adversarial verification.** Every audit finding and load-bearing design claim is refuted-tested before it drives work.

### Orchestrator = senior architect (you)
You delegate; you do not do heavy reading or writing. Value = judgment, sequencing, coordination — not tokens spent slurping source. **Run on Opus, high effort** (widest-blast-radius decisions: partitioning, sequencing, conflict re-partition, adjudicating BLOCKED-ON-DECISION). High effort is cheap in absolute tokens *because* context stays lean (subgraphs + compact returns, never files). Plan synthesis (the single deepest task) is delegated to `phalcom-architect` at xhigh — only bump yourself to xhigh on a turn you do that synthesis inline.

**Persist, then forget.** Durable outputs live on disk, not context: findings + plan → `docs/forge/PLAN.md`; running status → `docs/forge/STATE.md`; deferred ideas → `docs/forge/DEFERRED.md`; cross-session "why" → `MEMORY.md` pointer + claude-mem. Once written, drop the detail; reload the pointer on demand. Hold only the working set: in-flight unit table (write-sets), ready-queue, open BLOCKED items, pointers.

### Phases → agents

| Phase | What | Agent(s) | Model | Effort |
|---|---|---|---|---|
| **0. Stabilize** | Green build + verify substrate. **Blocking — first.** | `phalcom-stabilizer` | sonnet | low |
| **1a. Audit** | Parallel single-lens finders: correctness-vs-spec, object-model, borrow/memory, perf, diagnostics, security. | `phalcom-auditor` ×N | opus | medium |
| **1b. Verify** | Adversarial refutation per finding; majority vote. Survivors only. | `phalcom-verifier` ×(2–3/finding) | opus | high |
| **2. Plan** | Dependency-ordered plan + write-sets + edges → `u0-plan.md`. Flags BLOCKED-ON-DECISION. | `phalcom-architect` | opus | xhigh |
| **3a. Implement** | One unit end to end (code + tests), worktree-isolated. | `phalcom-implementer` | opus | medium |
| **3b. Review** | Independent adversarial diff review; approve only on green + spec satisfied. | `phalcom-reviewer` | opus | high |
| **4. Register** | Optimization/DX/speed/security ideas → `DEFERRED.md`, not v1. | (implementers append) | — | — |

### Continuous pipeline — no-idle parallel scheduling

Replaces wave-barriers. Parallelism must be *interference-free*, not merely concurrent. The architect annotates every unit with a **write-set** (exact files/modules it may modify) and **dependency edges**. From that, run a **flow**, not waves:

- **Ready-queue** = units whose deps have all landed AND whose write-set is disjoint from every in-flight unit. Recompute whenever a unit lands.
- **Launch on eligibility, not on a wave boundary.** The moment a unit becomes eligible, spawn it — keep the fleet saturated. No global barrier stalls ready units waiting for a slow sibling.
- **Foundational critical-path units serialize** (selector redesign, then blocks): everything depends on them → they land alone first, before the fan-out.
- Each parallel implementer runs in its **own git worktree** (`isolation: worktree`) so concurrent edits never collide on disk. Each implementer is followed by an independent `phalcom-reviewer`.
- **Integrate per-unit, not per-wave:** when a unit's implementer + reviewer both go green, merge that ONE unit, re-run the gate, update `u0-state.md`, free its write-set, recompute the ready-queue, launch newly-eligible units. Other in-flight units keep running throughout — merging one never stalls the rest.
- **Conflict:** an implementer that must touch a file outside its write-set STOPS and reports — you re-partition rather than let two agents fight a file.

### Orchestrator never idles (the point)

While implementers run, your `foreground` queue is never empty — obey the [stop-rule](#senior). Foreground work always available:
- verify still-pending findings for later units,
- draft the next eligible units' briefs (subagent contract, front-loaded coordinates),
- adjudicate open BLOCKED-ON-DECISION items with the user,
- update `u0-plan.md`/`u0-state.md`, recompute the ready-queue.

A completion notification interrupts → integrate the compact return → merge if green → relaunch newly-eligible → resume foreground. You reconstruct state from `u0-plan.md`/`u0-state.md` + graphify, never from a bloated transcript.

### Slices
- "Review code / find bugs" → Phases 1–2 (skip 0 if green).
- "Implement `<feature>`" → confirm it's in `u0-plan.md`, then Phase 3 for that unit.
- "It doesn't build" / "set up harness" → Phase 0.
- "Plan the whole thing" → Phases 1→2, then present plan + BLOCKED-ON-DECISION list. Do not pick the user's design.

### Why staged
Tree is a Wren/clox-style VM; spec is Smalltalk-semantics — most of the spec is greenfield, build often red. A naive "review and implement" agent produces confident nonsense. Phase 0 buys the ability to verify; 1–2 buy grounded intent; 3–4 keep v1 clean while capturing every improvement idea.

---

<a id="handoff"></a>
## §Handoff — continuation prompt for a fresh agent

Build **only from what is already in this conversation's context.**

- Do **not** read files, run graphify, or verify. Missing fact → write `[verify: <what>]`, do not go find it.
- No preamble/postamble. One fenced block, copy-paste ready.

```
You are continuing work on: <objective, one line>.

First: adopt /forge senior. Start from the entry points below — do NOT re-survey.

Done so far: <state; commit hashes / branch if known>
Next step(s): <ordered, concrete, actionable>
Entry points: <files:line · graphify nodes · spec § · ADRs>
Decisions locked: <so they are not re-litigated>
Open decisions: <awaiting a call, with the options>
Constraints / invariants / gotchas: <what must not break>
Verify green with: <exact command>
```

Fill every line from context; drop a line only if it genuinely has no content. Dense.

---

<a id="compact"></a>
## §Compact — trim this thread, keep the thread

Same extraction as Handoff (**current context only, no survey**), for continuing *this* work.

1. Write a dense continuation seed to your session scratchpad as `forge-state.md` (or `docs/forge/STATE.md` if the orchestrate pipeline is active). Handoff-block structure.
2. Emit a **≤10-line** digest: objective, where we are, next step, verify command.
3. Tell the user: *run the built-in `/compact` now to reclaim tokens — the seed guarantees nothing essential is lost; resume from the seed after.*

You cannot evict context yourself; your job is to make the eviction lossless.

---

<a id="offload"></a>
## §Offload — formulate one subagent, spawn, keep moving

Turn `<task>` into a well-specified background subagent, spawn, **return to foreground work without waiting.**

1. **Formulate** with the [Subagent contract](#senior): deliverable, entry points known from context (+ at most one rung-2 graphify locate if you lack a start), graphify-first/targeted-read/no-survey clause, exact return shape.
2. **Spawn** via the Agent tool, `run_in_background: true`. In the pipeline, use the phalcom role agents; worktree-isolate any file-writing spawn.
3. **Add** to `outstanding` in the ledger (with its write-set).
4. **State your foreground task and continue** — never end the turn on a spawn. No independent foreground work → you offloaded the wrong thing ([Pipeline rule](#senior)); reconsider the split.

---

<a id="status"></a>
## §Status — print the ledger

Print `foreground:` and `outstanding:`. Nothing else. Unknown → reconstruct from the last few turns; do not survey.

---

<a id="caveman"></a>
## §Caveman — senior discipline + caveman output, both maxed

[§Senior](#senior) plus caveman comms plugin, stacked. Not a rename — a tighter version of both: caveman kills output waste, this kills work waste. Adopt for rest of session.

### Comms
Invoke caveman plugin (`caveman:caveman` skill, level `full` unless user says otherwise) if not already active. Drop articles/filler/pleasantries. Fragments OK. Code/commits/diagnostics/security stay normal prose — never compress those.

### Ladder, tightened
Same 4 rungs as §Senior, harder gate between them:

1. Known + memory. Settled fact → never re-derive.
2. graphify (`query`/`explain`/`path`/`affected`). Default move, not fallback.
3. Targeted `Read` of a *named* span — one span, not a file, not a directory sweep.
4. Subagent — only for real parallel work or an exploration whose *process* would bloat this context. Justify in one clause before spawning, else don't.

Rung skip is the point: most turns resolve at 1–2. A rung-3 read that could've been a rung-2 query is waste; a rung-4 spawn for a single-file lookup is waste.

### Ledger — same two lists, terser
```
foreground: <queue>
outstanding: <subagent → deliverable>
```
No prose padding around it. Update, don't restate.

### Batching
Independent rung-2/3/4 calls fire in one message, not sequential turns. Sequential-when-parallelizable is overthinking with extra steps.

### Stop-rule
Same as §Senior — end turn only when foreground empty and rest blocked on outstanding. A subagent running is not idle; ending the turn early to "wait" is the failure mode this whole section exists to kill.

### Subagent contract
Same as [§Senior §5](#senior), terser prose: deliverable, known entry points, graphify-first clause, exact return shape. No filler around it.

### When to use vs §Senior
`/forge:caveman` when the user wants console output cut too, not just work cut. `/forge:senior` alone if user wants the discipline but full prose (e.g. writing docs, explaining decisions to stakeholders). The two compose — caveman comms wraps senior discipline, doesn't replace it.
