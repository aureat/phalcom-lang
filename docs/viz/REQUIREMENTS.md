# Execution visualizer — requirements

The operating manual. Every design decision in [`UI-SPEC.md`](UI-SPEC.md) resolves against this file,
and every future change to that spec must be re-checked against it. Requirements are numbered so the
spec can cite them.

**Scope of the artifact.** A React/JS player that replays a Phalcom execution trace so a viewer can
see fibers, scheduling, frames, the stack, locals and upvalues — and how they collide. First cut is
**hand-authored traces, three examples, no VM connection**.

---

## 1 · Purpose, stated testably

Not "display VM state." The bar, inherited from the `docs/learn` method: **the viewer can re-derive
the design from the picture.**

The design is done when a viewer who has read nothing can:

| | Test |
|---|---|
| **T1** | Predict where the tape is after a switch, *before* stepping — and be right |
| **T2** | Point at the screen and say why E3's second program is illegal |
| **T3** | State what a `FiberObject` contains without having read Rust |
| **T4** | Spot the dangling upvalue in the abort trace unprompted |
| **T5** | Explain, unprompted, why the switch has a hole in the middle |

A change that makes any of T1–T5 harder loses, regardless of how it looks.

## 2 · Three audiences, and they conflict

| Audience | Context | Needs |
|---|---|---|
| **Owner** (primary) | long sessions, deep dives | density, exactness, every slot value |
| **Interviewer** (secondary) | 2–5 min over a shoulder, zero Phalcom knowledge | legible in ~30 s, no narration |
| **`docs/learn` reader** (tertiary) | static embedded stills | must survive as a screenshot |

The conflict is itself a requirement.

- **R-ZOOM** — the screen reads at two levels: a **coarse read** (*what moved*) in ~5 s, and a
  **fine read** (*exact slots, values, offsets*) on inspection. Serving only one fails two audiences.
- **R-STATIC** — every teaching moment is legible as a **still frame**. Motion may reinforce; it may
  never be load-bearing. This constrains the switch rendering hardest: the hole must read frozen.
- **R-SILENT** — if it needs someone talking over it, it failed.

## 3 · Co-visibility — the real layout driver

Panels are not the constraint. *Relationships* are. The lesson is always a relationship, so both ends
must be on screen at once.

| Must be co-visible | Because the lesson is |
|---|---|
| tape ↔ fiber lockers | the move — invisible if either is offscreen |
| upvalue cell ↔ tape ↔ locker | the connector *stretching* into the locker |
| host gutter ↔ tape | yield legality |
| source ↔ bytecode ↔ tape | three descriptions of one instant |
| resumer chain ↔ fiber status | where an unwind stops (`call` vs `try`) |

Anything **not** in that table may be a tab, toggle or hover. Anything in it may not.

- **R-SPINE** — the tape is the spine. Everything else is positioned by its relationship to it.
  Nothing else may claim the visual center.

## 4 · Truth constraints — what the picture must never imply

Hard prohibitions. Violating one teaches a wrong model, which is worse than teaching nothing.

| | The picture must never imply | Because |
|---|---|---|
| **N1** | per-fiber *machine* stacks | ADR-0030 rejected stackful coroutines; guest `Vec` and host stack must never share a visual vocabulary |
| **N2** | frames own separate stacks | one array, brackets over regions (`stack_offset` indexes the shared `Vec`) |
| **N3** | a round-robin scheduler | `ready_queue` holds only never-started fibers; control transfer happens on the **resumer chain** |
| **N4** | the switch is atomic | Lua/Wren swap a pointer and have no hole; Phalcom moves contents and has one |
| **N5** | upvalues hold addresses | they name `(fiber, slot)` — `heap/upvalue.rs:34-36` |
| **N6** | `ip` stays behind | it is a `CallFrame` field (`frame.rs:72`) and parks with the fiber |
| **N7** | any state the VM never holds | no frames invented for animation smoothness |

N7 generalizes to the sharpest rule in the manual:

> **Animation may interpolate position. Never state.**
> An object may *travel*. It may never be shown in a configuration the VM never holds.

## 5 · Interaction requirements

- **R-REV** — bidirectional stepping is mandatory, not a nicety. *"Wait — what just happened?"* is the
  core learning moment and it requires going back. Time-travel is the entire reason this beats a
  debugger.
- **R-BOUND** — the cursor lands only on event boundaries. No interpolated positions (corollary of N7).
- **R-SCRUB** — scrubbing stays responsive at any cursor. Keyframe snapshots, replay from nearest; a
  naive O(n²) fold makes the primary interaction unusable.
- **R-MANUAL** — manual control is primary; autoplay is optional and secondary.
- **R-POINT** — every element answers *"what is this"* on hover/click. The interview audience points.
- **R-KEYS** — keyboard driven. Demoing with a trackpad reads badly.
- **R-GATE** — predict-then-check gates at authored teaching moments: pause, offer 2–3 answers, reveal.
  This is what makes it a teaching tool rather than a debugger.

## 6 · Attention and delta legibility

At most cursor positions exactly one thing changed. Unmarked, scrubbing degrades into
spot-the-difference — exhausting, and it defeats the learning.

- **R-DELTA** — what changed at this cursor is marked, always. First-class, not polish. And per
  R-STATIC it must be visible in a *frozen* frame, so: persistent marking, never a flash.
- **R-ONEMOVER** — the eye tracks one moving object. During a switch the tape moves and nothing else does.
- **R-NODECOR** — no decorative motion. Motion means "this is what changed."

## 7 · Design envelope

Stated limits. Degrading outside them is fine; pretending otherwise is not.

- ~4 fibers · ~8 live frames · **9 tape slots** · ~150 events per trace

  *Amended after the build:* the slot budget was ~40 at 28 px. Values are the reason the tape exists
  and `<Counter>` does not fit in 28 px, so the slot sizes to the value and the grid dropped to 9.
  No teaching trace has exceeded 7 slots; E1 peaks at 5.
- Desktop, landscape, ≥ 1280 px. Not mobile.
- Values render as short strings (`3`, `"hi"`, `<Counter>`); no nested object inspection.

## 8 · Failure modes to design against

1. **The confident lie** — hand-authored trace is wrong, picture is plausible → player-side invariant
   checks on by default, red banner on violation, plus `tools/viz/check.mjs` as a commit gate.

   *Sharpened after the build.* Structural validity turned out not to be the real risk. A trace can be
   perfectly well-formed and have **silently stopped teaching its lesson** — when E1 rendered one full
   card at the hole instead of two, nothing was malformed, nothing looked broken, and there was no
   reason to look. So checks come in two kinds, and the second matters more:
   **assert the lesson, not just the well-formedness.** Anything a reviewer would catch only by
   remembering to look, a check should catch instead (UI-SPEC §10).
2. **Spot-the-difference fatigue** → R-DELTA.
3. **Panel soup** — everything visible, nothing readable → §3 promotes only relationship-critical pairs.
4. **Atomicity implied by a smooth tween** → the hole is a stoppable cursor position, not a transition.
5. **The Lua picture creeping back** — boxes sitting still, a pointer arrow moving between them. This is
   the default every reference visualizer nudges toward. Constant vigilance.
6. **Needs narration** → R-SILENT.
7. **Too pretty to verify** — if a trace cannot be checked against `primitive/fiber.rs` by eye, it drifts.

## 9 · Explicit non-requirements

Out of scope, stated so they do not leak in: live VM attachment · arbitrary user code · dispatch,
inline caches, sacred inlining, GC, metaclass tower, AST pane · breakpoints, watch expressions,
conditional stops · performance profiling · mobile · screen-reader parity (keyboard nav yes; full a11y
honestly deferred).

## 10 · Tiebreakers — ranked

When two designs conflict:

1. **Truthfulness** > clarity. Never simplify into a lie.
2. **Co-visibility of the relationship** > per-panel beauty.
3. **Delta legibility** > information density.
4. **Still-frame legibility** > animation quality.
5. **Reversibility** > polish.
6. **5-second coarse read** > completeness (completeness lives in the fine layer).
7. **Needs a new panel?** → it is a later tenant, not a v1 compromise.
