# REQUIREMENTS — `docs/learn/vm/frame-identity.md` (VM track, Doc 6)

Phase 2 per [AUTHORING.md](../AUTHORING.md). Grounded on [recon.md](recon.md), not on assumption.

---

## 1. The obligation

After reading, the reader can **re-derive** the token. Given only:

- frames live by value in a `Vec` that `truncate`s and reuses slots (Doc 3), and
- a block can outlive the activation it names (Doc 2 / upvalues), and
- the failure must be *recoverable*, not undefined,

they should reconstruct `(recyclable location, globally-unique serial)` and the
check-before-mutate ordering, without being told.

## 2. The reader

Knows PL design; not fluent in implementation. Has read Docs 1–5 of this track and `upvalues.md`.
**Therefore already knows**: the `is_live` compare, the `DeadFrameError` output line, and the
Smalltalk `BlockCannotReturn` name. Re-teaching those is a failure, not a recap (recon §6).

Specific weakness: cannot hold moving state in their head. The trace must be notated, not narrated.

## 3. Doc kind: **knot**

Genuine circularity, and it is why this doc is last. The token is *minted* by a frame (Doc 3),
*stamped* onto a compiled artifact (Doc 2), *carried in* through a send (Doc 4), and its unwind
*terminates* by arranging state for Doc 1's drain check rather than by returning. You cannot
explain the mechanism without all four; each of the four deferred it here. Structure = show the
cycle, then cut it at the one place it is cuttable.

It also closes a two-sided tie: `upvalues.md` introduced `FrameToken`/`DeadFrameError` from the
**closure** side (what a captured `return` needs). Doc 6 closes it from the **frame** side (what a
recycled slot owes).

## 4. The grip

> A `FrameToken` is a pointer split in two: `frame_index` is *where to look*, `generation` is
> *who it was*. The first is fast, fiber-local, and recycled; the second is globally unique and
> never reused. Every non-local return dereferences with the cheap half and is only ever *trusted*
> because of the expensive one.

Stated early. **Earned** by the cross-fiber trace, where the cheap half is not merely stale but
belongs to a different address space of meaning entirely — and the design survives it unchanged.

## 5. The design space

The question: **how do you name an activation that may already be gone, such that using the name
after it is gone is detectable?**

| Branch | Occupant | Buys | Costs / forecloses |
|---|---|---|---|
| **(a) Raw index / raw frame pointer** | naïve VMs; C `setjmp` idioms | one word, zero check | ABA: a recycled slot silently accepts a stale name. **The ADR's named rejected alternative.** |
| **(b) Location + generation serial** | generational arenas, ECS entity ids, slotmap | `Copy`, one word pair, O(1) validate, no GC edge | serial exhaustion; validity is *checked*, never *guaranteed* by the type |
| **(c) Strong/weak reference to a heap activation** | Smalltalk-80 `BlockContext`; Python frame objects | liveness is a real query; contexts are first-class and reifiable | activations must be heap objects — surrenders Doc 3's `Vec`-of-`Copy` entirely; refcount/GC traffic per call |
| **(d) Side liveness table** | — | frames stay dumb | a second structure to keep in sync with `truncate`; a lookup, not a compare |
| **(e) Static escape prevention** | linear types, regions, Rust's own lifetimes | no runtime check at all | needs a type system Phalcom does not have; forecloses first-class blocks |
| **(f) Unforgeable capability / nonce** | capability systems | tamper-proof naming | solves forgery, not *liveness* — orthogonal problem |

**Weight (steers Agent A, not the doc's final shape):** (a) and (b) deep — they are the real fork
and (a) is what the ADR actually killed. (c) deep, because it is the *branch Doc 3 already
foreclosed*, so it teaches the coupling between the two docs. (d) a paragraph. (e), (f) one
sentence each; (f) exists only to be dismissed as a different question.

**Honesty requirement (§5.2):** the ADR deliberated exactly **(a) vs (b)**. The rest of this table
is pedagogical reconstruction and the doc must say so in those words.

## 6. Comparison filter

A language enters only if it (1) took the other branch with the bill, (2) has a scar, (3) names
something Phalcom does anonymously, or (4) is an ancestor.

| Language | Test | Why it earns its place |
|---|---|---|
| **Smalltalk-80** | 4 + 3 | Ancestor of the whole feature; **names** the error (`BlockCannotReturn`). Took branch (c): contexts are real objects. |
| **Generational arenas / ECS** | 3 | **Names** the pattern (`generation`, ABA) that Phalcom applies without naming. Highest-value entry. |
| **Rust (`ObjRef`, and lifetimes)** | 3 + 1 | The host language solves the *same* problem twice — once in the type system it cannot apply here, once in the slotmap it does apply. |
| **Ruby / JS** | 1 | `return` from a lambda/arrow vs a proc/function: the semantics Phalcom's `DeadFrameError` is the price of. Bill attached. |

**Cut, and say why in the doc:** Java (no non-local return from lambdas — nothing to compare),
Go (its scar is loop-variable capture, already spent in `upvalues.md`), Lua (its upvalue design is
spent; its `error` unwind is not this mechanism), C# (spent in `upvalues.md`).
Expected survivors: 4. Do not exceed 5.

## 7. Tensions to surface

1. **Speed vs. trust.** The index is the only reason the return is O(1) — no search for the home
   frame. The generation is the only reason the index is believable. Neither half is optional and
   they pull opposite ways.
2. **Fiber-local index vs. VM-global serial.** The load-bearing asymmetry (recon §1). Do not
   present it as elegance; ask whether it is *deliberate* or *emergent* and answer from source.
3. **Detection vs. prevention.** The token cannot stop the escape; a `DeadFrameError` is a
   *runtime* answer to what (e) would answer statically. State what Phalcom bought and gave up.
4. **The token is not a GC edge.** `heap/trace.rs`: a token does not keep its frame alive — the
   opposite of what a `Weak` would do, and the reason the design costs the collector nothing.
5. **Failure atomicity.** Check before mutate, or a *caught* `DeadFrameError` leaves a torn stack.
   The comment says this; the doc must make the reader feel the alternative.

## 8. Structural rules

- Grip stated within the first screen; earned at the cross-fiber trace, not before.
- **≥1 predict-then-check.** Mandated site: *"the block escaped to another fiber. Its
  `frame_index` now indexes a completely different array — one that may well have a live frame at
  that index. What stops it?"* Let the reader answer before revealing the counter is not swapped.
- **Hard trace, not the easy one.** The easy case is `runtime_non_local_return_dead_frame.ph`
  (already shown twice in this track — do not lead with it). Trace **`blocks_non_local_return_two_deep.ph`**:
  the *successful* unwind past two block frames and a native `each` frame, ending in the
  hand-off to Doc 1's drain check. Then the cross-fiber failure as the counterpart. Both from real
  output.
- Anchors symbol-first: `file.rs::Type::method` (~Lxxx).
- Destroy `frames.md`'s **Lie #1** explicitly, by name, with a link.
- Close the loop on Docs 1–5 in the closing section — that is what makes it the capstone.
- Any diagram must draw the *two-scope split*; do not draw a pointer arrow for a design whose
  thesis is that it is not a pointer.

## 9. Checklist (gate, §6 of AUTHORING)

- [ ] Re-derive test + ≥1 predict-then-check.
- [ ] Grip grounded in a read type, stated early, earned late.
- [ ] Reconciliation table built (A's theory vs Phalcom's representation).
- [ ] Honesty: design space labelled a reconstruction; ADR deliberated only (a) vs (b); the
      `interpret.rs` duplicated stamp reported as fact, not as a flaw; the ADR's "unifies with
      `throw`/`abort`" claim either verified or marked aspirational.
- [ ] Claims ledger: every perf/comparative/forward claim cited, labelled unverified, or cut.
      **No unmeasured cost claims** ("one compare is free") without a number or the word
      *unmeasured*.
- [ ] Hard trace is `_two_deep` + cross-fiber, from real output.
- [ ] Nothing from `upvalues.md` §"Non-local return" restated as new.
- [ ] Lie #1 destroyed by name; all links resolve (ADR filename is
      `0013-closure-upvalues-and-frame-token-return.md`).
- [ ] Weighted, ~30% cut from the design-space walk.

## 10. Build sequence

1. Phase 3: Agents A (theory, no source) and B (source map) in parallel.
2. Reconciliation table → honesty pass → claims ledger → insert predict-then-check → trace the
   hard case → cut.
3. Ship to `docs/learn/vm/frame-identity.md`. Scratch stays here.
4. Update the track index / forward pointers in `frames.md` if they name Doc 6 as pending.

## 11. Open risks

| Risk | If wrong, the doc… |
|---|---|
| **Overlap with `upvalues.md`.** Highest risk by far. That doc already spent the compare, the output, and the Smalltalk name. | …becomes a restatement wearing a new title. Mitigation: §8's forbidden-list; B is told to find what upvalues.md did *not* cover. |
| **The two-scope claim** (fiber-local index, global serial) is the doc's spine and rests on `mem::take` in `fiber.rs` plus the absence of a swap for `next_frame_generation`. Recon read both, but *absence* is the weaker evidence. | …spine collapses. B must confirm by exhaustive grep that nothing else writes `next_frame_generation`. |
| **`wrapping_add` / ABA at u64 wrap.** Unexamined at HEAD. | …either flatters the design (claiming soundness it has not argued) or invents a bug. Report what is there; do not editorialize. |
| **The ADR's `throw`/`abort` unification** may be aspirational. | …repeats an unshipped claim as fact. B must check. |
| **Doc 1's drain check** — the hand-off is read from a *comment* asserting it, plus the absence of `return Ok`. | …mis-describes termination. B must trace `run_until`'s halt condition against the post-`ReturnNonLocal` state. |
| **`DeadFrameError` through a native window** — three sites mention it; behavior under user `on(_)` unverified. | …claims catchability it has not observed. B must run a program. |
