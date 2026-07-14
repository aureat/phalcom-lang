# Handoff — performance investigation (2026-07-14)

Session: benchmark harness repair, U-HOTPATH attribution, two landed cuts, one
open decision. Read [`perf-log/README.md`](perf-log/README.md) +
[`perf-log/findings.md`](perf-log/findings.md) first — everything below is
recorded there in full. This file is only the "where I was standing" note.

## Landed this session

| Commit | What |
|---|---|
| `8ba87ec` | Benchmarks verify their **answers**, not just wall-clock; `compare-wren.py`; `fiber_churn.ph`; wren-suite table re-measured |
| `9207fac` | perf-log: corrected Change 2 verdict, fresh profiles, re-ranked levers, F10–F13 |
| `0274f10` | **Inliner fix (F13)**: bootstrap 0.18s → 0.005s; nest depth 26: 70.9s → 0.022s; `--test lang` 122s → 2.8s |
| `4f2eed8` | **Yield-adaptive GC (F11)**: skynet −11.7% user / −8% RSS; fiber_churn −7.4% / −15% RSS. `cargo test --workspace` now **fully green** (was red on main since ≥`bd3f492`) |

## INTERRUPTED HERE — the shadowing ruling (owner asked me to investigate + recommend)

**Question.** F12's slot-resolution of module globals needs a ruling: may a later
`define` shadow a name an earlier callsite already resolved (esp. a core name)?

**What I established before the interrupt — all measured, not assumed:**

1. **Late shadowing IS observable today.** `class C { static get { return List } }`
   → `C.get` returns core `List`; after `var List = 42`, the *same callsite*
   returns `42`. Scratch: `shadow1.ph`.
2. **My F12 prototype gets this wrong** — it prints `List` / `List`. The
   per-callsite `(module, slot)` cache is **not** correct as written. It is a
   measurement of the ceiling (−15% bare_send), **not a landable patch**. Do not
   land it as-is.
3. **Forward references to not-yet-defined globals work and are ordinary.**
   `class C { static get { return later } }` before `var later = "…"` resolves
   fine. So late binding is not an exotic corner — it is how the language
   currently reads a global defined further down the file.
4. **Spec Q4 ([`core/decisions.md`](../spec/v0.2/core/decisions.md:121)) leans
   against making shadowing illegal**: the ruling models kernel names as "the core
   module's exports, auto-imported", explicitly so the import system "can re-scope
   or **shadow** it without a breaking change" (forward-compat §3, ADR-0027).

**The one probe I did not get to run:** whether the *core-fallback* resolution
specifically (main-module miss → core hit) is reachable-then-shadowed in any real
program, vs. only in a synthetic one. That distinction is what separates the two
candidate designs below.

**My recommendation as it stood (finish verifying, then decide):**

> **Keep late-binding; cache with invalidation.** Option "shadowing illegal" is
> the fastest but (3) shows plain forward references share the same machinery, and
> (4) shows the spec wants shadowing to stay expressible. Emit a per-callsite
> `(module, slot)` cache guarded by a **module-globals version** — bump it in
> `ModuleObject::define` (and only there; `declare` is append-only so existing
> slots never move). That is the exact shape of the IC's `world_version` guard, so
> it is a known-good pattern in this codebase. Expected to retain most of the
> −15%, since the guard is one integer compare against a hash+probe.
>
> Compile-time `GetGlobalSlot(u16)` stays available later *if* Q4's import unit
> ever declares module scope closed — do not pre-empt that ruling to buy a few %.

## Answers to the three questions asked (all measured, prototypes in scratch)

1. **1M-fiber benchmark** — was GC-bound: `trace_object` ~20% of ticks, GC family
   ~30%, collections freeing ~nothing. **Landed** (`4f2eed8`), −11.7%. Remaining:
   object density (F7 — 1.4 KB/fiber vs Wren's ~0.6 KB), untouched since Win A.
2. **Message sends** — the top non-loop cost is **reading a variable**, not
   dispatch (U-IC landed in `f5e41f1`; method lookup is cached). SipHash probe per
   global access = ~13% of `bare_send` ticks → F12, blocked on the ruling above.
   Then: U-HOTPATH Change 1 (repays cut 004's measured +5–7% send regression),
   variadic-IC refill, DEC-PRIM-B arithmetic fast path (`call_method` ~14%).
3. **`for` loop** — same global-access bug, **four probes per iteration**; F12
   gives −5%. Was 144× vs Wren, now **13.2×**; the band is 4–14× across the suite.

## Owner rulings this session (do not re-litigate)

- **`fiber-pool` flag: keep, keep OFF, do not use.** It is measured **negative**
  (+72–86% RSS, linear ~450 B/fiber; +37% user at 1M fibers) — see F10. Reviving
  needs a *new mechanism*, not a re-run.
- **Change 2 (selector memo) stays** — it is a **−28% win** on `variadic_send.ph`.
  My earlier "wash" verdict was a harness hole (no benchmark reached the probe);
  corrected in `9207fac`.
- Worktrees: 3 stale ones removed; `.claude/worktrees/agent-af9723bf346e6a782`
  **kept at owner's request** (its uncommitted LSP edits are superseded by main —
  196/198 lines already there, other 2 reformatted — but left alone deliberately).

## Traps worth knowing

- **`cargo test --workspace | tail` lies** — the pipe's exit code is `tail`'s. Grep
  for `FAILED` explicitly. A "green" run hid 12 failures earlier.
- **The Bash tool runs fish, not bash.** `set -- $x` silently produced empty args
  and a table of `0.00s` timings that looked real.
- **Percentages hid a constant.** F13's +175 ms bootstrap read as a uniform "+20%
  throughput regression" across unrelated benchmarks. `bare_send` touching no
  strings is what exposed it. Suspect a constant when a regression is suspiciously
  uniform across unrelated workloads.
- **A green test can be green by luck.** `automatic_safepoint_fires` flipped red→
  green from the GC tuning alone (live 3857 → 2073) while testing nothing about
  safepoints. It now asserts the invariant and passes with the adaptive factor both
  on and off — that two-way check is the point.
- **Benchmark A/B needs matched commits.** My first variadic A/B compared
  `1531070` against the *working tree* and reported +11% (i.e. the wrong sign),
  because two U-STRING commits had landed in between.

## Scratch (throwaway, not in repo)

`/private/tmp/claude-501/-Users-altunhasanli-dev-phalcom-phalcom/8add6f4e-7530-490e-83a1-4f6d6ec28480/scratchpad/`
— binaries `ph-{base,c4,main,inline,final,glob,gcadapt,nocache,cache2,str}`,
worktrees `wt-{c4,head,str,gc,glob}`, `shadow1.ph`/`shadow2.ph`, `bare5m.ph`,
`arith5m.ph`. **`wt-glob` holds the F12 prototype diff** (`chunk.rs` `gcaches` +
`dispatch.rs` Get/SetGlobal) — the only copy; it is measurement-grade, and
incorrect per (2) above. Worktrees are removable with `git worktree remove -f`.
