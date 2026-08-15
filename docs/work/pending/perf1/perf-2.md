# U-PERF work-2 — consolidating two sessions, and building the batch a quiet box will eat

**Date:** 2026-07-15. **Base SHA:** `4c6c83f`.
**Outcome: six binaries built, four A/Bs pre-registered, zero numbers taken.**

Companion doc: [perf-log/010-prereg-h16-h17-h13.md](../../perf-log/010-prereg-h16-h17-h13.md)
— the pre-registration, probe diffs, predictions, and the quiet-window commands.
This file is the session ledger: what was inherited, what was decided and why,
and what the next agent must not re-derive.

---

## 1. What this session inherited

Two prior sessions ran concurrently and both ended clean, committed,
docs-only — but neither could see the other, and a **third** result landed after
both of them stopped reporting.

| session | landed | left open |
|---|---|---|
| A | `7a8aaf3` — F22 + `ab-guarded.py`, the load guard | H13's body half (the 128 B init question), un-owned |
| B | `18a57af`/`e66af34` — cut 009 negative, F21 + H16 | H16: outline the cold arms; then re-price cut 008 |
| — (after both) | `36e19ff` — **F23 + H17** | H17: F23's `core.ph` probe counted, never timed |

**The consolidation delta: session B's parting recommendation was "H16 is the
next move, not another fusion."** That was true when written and is now
outranked. F23/H17 landed afterwards and is the only live lever F21/H16 does
**not** gate — it changes `core.ph` only, leaving the dispatch loop
byte-identical. B could not have known.

Three open holes, not two. All three are **timing** runs. All three block on one
resource: a box quiet enough for `ab-guarded.py` to consent.

---

## 2. The decision that shaped the session

**Do not sequence these as three build-and-measure units. Batch every build into
the noisy window; spend the quiet window on nothing but measurement.**

The two-build protocol already demands build-then-copy-out-then-time, so builds
cost the quiet window nothing. Load was 10.9–12.0 on 8 cores throughout — and
notably *not* `rustc`: it was desktop load (WindowServer 45%, Finder 31%, Chrome
29%). That is the user's to clear, not a concurrent session's, which is worth
knowing before anyone hunts for a rogue build.

---

## 3. The finding that invalidated the prepared pair

H17 recorded an A/B pair prebuilt at `e66af34`, explicitly "ready for a quiet
box". **It was retired unused.**

`4c6c83f` (`wip(U-CTOR)`, a concurrent session's constructor work) edited
`dispatch.rs:887` — **inside `run_until_inner`**, which starts at line 477. It
rewrote the `SuperSend` super-construct fallback and **shrank the dispatch loop
by 248 B**.

Under F21 that is not neutral: F21's claim is that the loop's code footprint is
paid by every program regardless of what it executes. A base five commits stale,
on the wrong side of a loop-footprint change, cannot anchor a measurement whose
effect size is ~5%.

**Everything was rebuilt at one common SHA (`4c6c83f`).** This is the generalised
form of F22's lesson: F22 says a concurrent session's *rustc* voids your timing;
this says a concurrent session's *merged code* can void your base. Neither is
about your own loop.

**Then it was turned into an asset.** `c82c8bf` (HEAD's parent) was built too, so
`prev` vs `base` is a **natural experiment**: an unintentional 248 B perturbation
of F21's exact variable, made by someone with no performance intent. If unrelated
work moves the suite by percent through layout alone, every cut in this log
carries an uncontrolled layout term — and that would be the round's biggest
finding, free, from binaries that already exist.

---

## 4. What was built, and what was verified without timing

Six timing binaries at `~/phalcom-perf-phaseA/` (durable — `/tmp` is purged at
boot, and rebooting is a plausible way to quiet the box), plus two counting
binaries. Full manifest and shas in
[010](../../perf-log/010-prereg-h16-h17-h13.md#6-binary-manifest).

Load-independent facts established this session — **all measured, none timed:**

**H16's probe does exactly what it claims.** Outlining the five cold arms
(`Class`, `Import`, `MakeFamily`, `FinalizeClass`, `Method`) behind
`#[inline(never)]`:

| symbol | base | h16 | Δ |
|---|---|---|---|
| `run_until_inner` | 16,296 | 12,088 | **−4,208 B (−25.8%)** |
| `call_method` / `invoke_at` | 1,136 / 1,584 | 1,136 / 1,584 | **+0 / +0** |

A clean single-variable cut of exactly the quantity F21 indicted. Cut 004/007's
hoisted `Rc<Callable>` is what made it free of lifetime fights — the arms take
`&Rc<Callable>` from a local, not a borrow of `self.heap`, so an outlined method
keeps `&mut self` with no `unsafe`.

**F23's "immune to F21" claim is now verified rather than argued.** `ph-A-h17`
leaves `run_until_inner` at **16,296 B — identical to base**. H16 does not gate
H17: that is a measurement now.

**F23's counts reproduce at HEAD.** F23 counted at `e66af34`, *before* `c82c8bf`
renamed `raw*` → trailing `_`. Re-derived at `4c6c83f` with fresh counting
builds: **−6,000,001 instructions (−10.2%), −2,000,001 `Return`s**, stdout
identical. Same to the digit across the rename — and −2,000,001 is exactly 2
frames/element over 1M.

**H13's obvious probe was confounded, and was replaced.** The first attempt
(`MaybeUninit` + elementwise `for i in 0..arity` write loop) **grew `call_method`
by 96 B**, because it silently swapped a bulk `copy_from_slice` memcpy for a
loop. It measures *init-removal ⊕ memcpy→loop*. `ph-A-h13b` keeps the bulk copy
and removes only the init: `call_method` **−24 B**, which is what deleting dead
stores should look like. h13b is primary; h13 demoted to an optional second
question.

**Invariance:** all 8 `full` rows × all 5 binaries produce **byte-identical
stdout**; `--test lang` is 46 passed / 0 failed on every probe. With one caveat
recorded loudly in 010 §3: for H17 the green corpus is **not** evidence of
invariance — it is evidence the corpus does not reopen `List`.

---

## 5. Why predictions were written down before any number exists

010 §2–§5 record, per probe, the expected sign, the falsifier, and what a null
would mean — **before** the quiet window opens.

This log has been burned twice in ways that pre-registration addresses directly.
**F18** was an estimate with the wrong sign. **F21** was worse and more
instructive: a ceiling that was *right* and still insufficient, because
`pairs × 3.3 ns` priced what a fusion removes and was silent on what adding the
arm costs. A prediction written after seeing the number cannot fail. One written
before can, which is the only reason it is worth anything.

The specific trap this round: **H16's falsifier is a split sign, not a small
number.** If rows disagree in direction, the mechanism is not "footprint" but
something narrower (I-cache set conflicts, alignment of one hot arm), and H16's
framing is wrong *even if the magnitudes are impressive*. That distinction
decides whether any future cut can control this variable on purpose — which
matters more than the percentage.

---

## 6. Method notes worth keeping

- **Worktrees, not the main tree.** Concurrent U-CTOR/U-BINDINGS sessions are
  live in `dispatch.rs` and `send.rs` — the exact files every probe touches.
  All five probes were built in detached worktrees under the session scratchpad;
  `phalcom-core/src` on `main` was never touched. Sessions A and B both had to
  dodge this by hand; worktrees make it structural. Never `git checkout -b` here.
- **Symbol sizes are a free pre-flight.** `nm -n` + next-symbol subtraction gives
  each function's code extent. It caught H13's +96 B confound and verified H16's
  −25.8% *before* the scarce quiet window was spent on either. Any probe claiming
  a footprint effect should be checked this way first. (BSD `awk` has no
  `strtonum` — use Python.)
- **Counting builds are load-independent.** Instruction counts were re-derived at
  load ~12 and are exact. Keep counting binaries at separate paths
  (`/tmp/ph-count-*`) so a counting build can never be timed by accident.
- **`timeout(1)` does not exist on macOS** — it silently broke an invariance
  sweep before it was rewritten in Python.

---

## 7. State, and the next move

**Foreground: nothing. Phase A is complete and the ball is with the user**, who
is preparing the quiet window.

**When the box is quiet**, run the four A/Bs in
[010 §7](../../perf-log/010-prereg-h16-h17-h13.md#7-the-quiet-window--one-run-six-arms)
in order: H16 (the gate), H17 (ungated, worst row), H13b (smallest, likely null),
then the natural experiment (canary, droppable). Exit 3 means the box was not
quiet — **it is not a null result and there is nothing to salvage from it.**

**Serialize the U-CTOR/U-BINDINGS track against the quiet window.** Those units
run `rustc`; that is F22's exact hazard, and `ab-guarded.py` will abort mid-run.
The two tracks cannot be interleaved — which is a scheduling constraint on the
project, not a preference.

**Still owed, out of scope this round:** H16's second half — re-A/B cut 008's two
arms against a layout-matched base to split its −8.1% into fusion-part and
layout-part. It is only worth starting once H16 has a number.

**Do not resurrect** anything from F22's void round in either direction, and do
not read a refusal as a null.
