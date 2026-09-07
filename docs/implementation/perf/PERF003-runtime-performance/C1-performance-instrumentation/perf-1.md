# U-PERF work-1 — session ledger, 2026-07-15

> **What this file is.** Every action of one perf session, in order, successful or not,
> with the *expectation* held before each and the *reason* it succeeded or failed. It
> exists so the next agent does not re-run a refuted idea, and does not become
> needlessly cautious about one that was never actually tried.
>
> **What it is not.** Not a scoreboard ([`SCOREBOARD.md`](../../perf-log/SCOREBOARD.md)
> holds the numbers), not a findings file ([`findings.md`](../../perf-log/findings.md)
> holds what was learned). This holds the **route and the reasoning** — the part those
> two files deliberately drop.
>
> Session HEAD: started `e66af34`, ended `9fe1d39` (concurrent session committed
> throughout). Landed: `8eae379`, `36e19ff`.

---

## 0. Read this first — the meta-mistake that shaped the session

- **I answered a perf question from memory and was wrong by ~7×.** Asked "how fast could
  Phalcom theoretically be", I answered from a stored memory holding the U-BENCH Tier-0
  baseline: *Skynet ~19–20× Wren, bare_send ~329 ns, allocation is the #1 lever, do
  U-PRIM-ABI before U-IC.* **Every one of those was superseded.** Real state at
  `5254586`: Skynet **2.7×**, bare_send **~144 ns**, U-PRIM-ABI **and** U-IC both landed,
  object-heap arena insert **2.2%** of Skynet ticks.
- **The specific wrong recommendation:** "generational nursery + bump allocation should
  outrank NaN-boxing, because allocation is 28.2%." **Premise-falsified before I said
  it.** That 28.2% was *Rust-host* `Vec`/`Box` malloc, which cut 001 already took. The
  *object-heap* allocation a GC redesign would target is the `slotmap::try_insert_with_key`
  row: **2.2%** (BASELINE §3b). A nursery targets 2.2%, not 28%. Also: adding a collector
  *costs* CPU and buys RSS — Skynet has no old generation (every fiber dies), which is the
  workload generational GC helps *least*.
- **Lesson, now enforced:** the `perf-baseline-measured` memory was rewritten to say
  *never quote perf from memory; read SCOREBOARD.md first*. Cuts land fast enough that any
  number held in context is stale within a session. **`UNITS-TRACKER.md` §11 is stale in
  the same direction** — it lists U-IC and U-HOTPATH as "not started" when cuts filed
  under both have landed. **Trust the perf-log cut trail over the tracker's checkboxes.**
- **What caught it:** the `/language-design` operating procedure, specifically step 4
  (reconcile with the committed position) and step 2 (scan for hazards before evaluating
  on merits). The error was not a reasoning failure, it was a **grounding** failure — and
  only a forced re-read of the source of truth caught it.

---

## 1. Actions, in order

### A. Spawned a subagent for a perf-log resume — **succeeded, high value**

- **Why:** the perf-log is ~2000 lines across 14 files. Reading it inline would have
  burned the context I needed for analysis.
- **Config:** `general-purpose`, model `sonnet`, background, with an explicit demand for
  `file:line` citations and an instruction to *flag contradictions between sources rather
  than reconcile them silently*.
- **Result:** returned the full cut table, the current standing, every negative result,
  and **three drift findings I had not asked for** (H15's silent pass; `phalcom-perf`
  cited in docs but not registered as a `[[bin]]`; the tracker's stale U-IC/U-HOTPATH
  rows).
- **Transferable:** the "report contradictions as findings, not nuisances" instruction is
  what produced the most valuable half of that output. Reuse it.

### B. Clean worktree before measuring — **necessary, do this every time**

- **Why:** the main tree was dirty with a concurrent session's U-CTOR work (`dispatch.rs`,
  `expr.rs`, `value/mod.rs`, `class_decl.rs`). **`dispatch.rs` is the hot loop** —
  benchmarking that tree would have measured their in-flight edits.
- **Did:** `git worktree add --detach <scratch> e66af34`, built there, removed after.
- **Confirmed live hazard:** HEAD moved `e66af34` → `7a8aaf3` → `5ed495e` → `9fe1d39`
  *during* the session. Any A/B against "the tree" would have drifted mid-run.

### C. Two-build protocol for the histogram — **worked, do not shortcut**

- `cargo build -r --bin phalcom` → `/tmp/ph-default`; then
  `--features opcode-histogram` → `/tmp/ph-hist`; **then rebuild default**, because the
  histogram build overwrites `target/release/phalcom`. Counts from the hist build, wall
  from the default. Never time a counting build.

### D. Ran the opcode histogram — **succeeded; reproduced §3bb exactly**

- `method_call` **48,134,098** and `for` **59,000,705** — matching SCOREBOARD §3bb *to the
  digit*.
- **Why that mattered more than it looks:** it validated (a) the instrument, (b) my
  worktree really was cut 008, and (c) that counts are load-independent. This is the only
  cross-check available on a contended box, and it is why the F23 numbers below are
  trustworthy while no timing in this session is.

---

## 2. Ideas tried, in order

### IDEA 1 — delete `GetSelf` by making `GetField` read `stack[stack_offset]` — **REFUTED before building. Do not retry.**

- **Observation that motivated it:** `GetSelf` is **24.2% of all instructions** in
  `method_call` (11,666,675 of 48.1M), and the pair is a pure round-trip:

  ```rust
  Bytecode::GetSelf   => { let receiver = self.stack[stack_offset]; self.stack.push(receiver); }
  Bytecode::GetField(slot) => { let receiver = self.stack.pop()...;  // pops what GetSelf just pushed
  ```

  A 16-byte push/pop through a `Vec` to move a value already addressable at a known offset.
- **Expectation:** ~9.0M instructions deletable (every `GetField` 6.33M + `SetField`
  2.67M is preceded by a `GetSelf`) = **−18.7% on `method_call`**, with **zero new
  opcodes** — therefore structurally immune to
  [F21](../../perf-log/findings.md#f21--an-arms-code-is-paid-by-every-program-not-the-ones-that-execute-it)'s
  ~5% arm tax that killed cut 009. It looked strictly better than cut 009: same win, none
  of the cost, and it *shrinks* the arm (drops the pop + underflow check).
- **Why it is wrong:** `GetField`'s receiver is **not** always the frame receiver. Read
  [`compiler/lib/scope.rs:31`](../../../../phalcom-core/src/compiler/lib/scope.rs)
  (`emit_self`) and [`compiler/lib/expr.rs:259-271`](../../../../phalcom-core/src/compiler/lib/expr.rs):

  | context | emitted | receiver is |
  |---|---|---|
  | method body, instance field | `GetSelf; GetField` | `stack[stack_offset]` ✅ |
  | **inside a block** | `GetUpvalue(idx); GetField` | a **captured upvalue** ❌ |
  | **static field, instance context** | `GetSelf; Invoke(class); GetField` | a **send result** ❌ |

  Inside a block, `self` is not the block object — it is the enclosing method's receiver,
  captured as an ordinary upvalue (ADR-0013, functions.md §2). And static-field access
  sends `class` to self first (ADR-0017), so the receiver is a class object off the stack.
- **Cost to refute: two file reads.** No build, no benchmark.
- **What survives:** the *observation* is real and still unexploited — the receiver is
  stored **three** times (`stack[stack_offset]`, `CallFrame.context`, and pushed again by
  `GetSelf`). [F15](../../perf-log/findings.md#f15--value-is-2-wrens-and-objref-blocks-nan-boxing)'s
  **S4** row (`CallFrame` 96 → ~32 B) attacks the same redundancy from the sound side:
  "`context` duplicates the receiver already at `stack[stack_offset]`, which is where Wren
  reads it." **If you want this win, do S4, not this.**
- **If someone insists on retrying:** it would require the *compiler* to know which of the
  three shapes it is emitting, and encode that — which means a distinct opcode, which
  re-enters F21's tax, which is exactly what made cut 009 lose. The idea's entire appeal
  was "no new opcode", and soundness removes that appeal. **It is closed.**

### IDEA 2 — H15: `compare-wren.py` resolved a dead path and passed silently — **SUCCEEDED, landed `8eae379`**

- **How found:** the subagent flagged H15 from SCOREBOARD; I checked the filesystem and
  found the binary *exists* at `resources/wren/bin/wren_test` (Wren 0.4.0, working). Only
  the **default path** was stale (`ec3b6af` moved Wren in-tree; the script still pointed at
  `~/dev/repos/wren/bin/wren_test`).
- **The bug was two bugs.** The dead path was the visible one. The real one is line 79:
  ```python
  ok = ph_rc == 0 and wren_rc == 0 and (matches or wren_out is None)
  ```
  `wren_out is None` (no comparator) ⇒ `ok = True` ⇒ **every row prints `ok`, exit 0**.
  A harness that degrades instead of failing emits numbers people believe.
- **Blast radius:** SCOREBOARD §1's ×-vs-Wren column froze at `5254586`. **Cut 008
  landed** and cut 009 was judged with **no working comparator on the machine**. Cut 009's
  rejection is still self-consistent (it was decided on Phalcom-only A/B), but cut 008's
  Wren column is unverified.
- **Fix:** resolve `resources/wren/bin/wren_test` first, then the pre-`ec3b6af` external
  checkout (so an older working copy still resolves); **exit 1** when no comparator is
  found; `ALLOW_NO_WREN=1` opts into Phalcom-only and says loudly that the run cannot
  refresh §1.
- **Verified both halves** — resolves the in-repo binary; `WREN_TEST=/nonexistent` exits 1.
  H15's own "how to fill" prescribed exactly this ("default to the in-repo path so it
  cannot rot again… exit non-zero rather than degrading").
- **Still owed:** §1 is *still frozen*. The instrument is repaired; the re-measurement was
  not taken (see §3 — the box is never quiet).

### IDEA 3 — F23: `for` is a `.ph` call chain — **SUCCEEDED on counts, NOT timed, NOT landable as-is**

- **Route to it:** `for` is the suite's worst row (**10.7×**). Rather than look at opcodes
  (as every cut 001–009 did), I read what the row actually *executes*: `core.ph`.
- **What I found** — per element of `for (x in list)`:

  | step | frames |
  |---|---|
  | `Iterable.iterate(cursor)` (`core.ph:649`) — inherited generic | 1, **plus `self.size` → `List.size => self.length_` (`core.ph:780`)** = 2 |
  | `List.iteratorValue(cursor) => self.at(cursor)` (`core.ph:801`) | 1, **plus `List.at(i) { return self.at_(i) }` (`core.ph:782`)** = 2 |

  **Four `.ph` frames per element**, two of them pure forwarding wrappers over a floor
  primitive. Wren's `List.iterate`/`iteratorValue` are **both primitives** — zero frames.
- **Expectation before the probe:** deleting the two wrappers removes exactly **2 frames ×
  1M elements = 2,000,001 `Return`s**.
- **Probe:** give `List` its own `iterate` (using `self.length_`) and
  `iteratorValue(cursor) => self.at_(cursor)`. **`core.ph` only — no Rust touched.**

  | | base `e66af34` | probe | Δ |
  |---|---|---|---|
  | `for.ph` instructions | 59,000,705 | **53,000,704** | **−6,000,001 (−10.2%)** |
  | `Return` (`.ph` frames) | 5,000,004 | **3,000,003** | **−2,000,001** |
  | stdout | `499999500000` | identical | — |
  | `--test lang` | — | 46 passed, 0 failed | green |

  **Prediction met to the digit.**
- **Why it succeeded where cut 009 failed** — and this is the transferable part:
  **F21's tax is paid for adding code to `run_until_inner`. A `core.ph` change leaves the
  dispatch loop byte-identical, so the tax cannot apply.** Cut 009's own probe 2 proves
  the mechanism is *arm code in the loop*, not the win. A `core.ph` cut has no arm.
- **`List` is the outlier, not the pattern** — every other collection already calls its
  primitive directly in `iteratorValue`: `Map:928` `keyAt_`, `Set:978` `at_`,
  `Tuple:1019` `at_`, `Range:1110` `start_ + cursor`. **Only `List:801` goes through a
  `.ph` hop.** So that half is a consistency fix with four precedents in the same file.
- **The wider half:** `Iterable.iterate` calls `self.size` **per element**, and `size` is a
  `.ph` wrapper over a native on *every* collection — that frame is paid by **all**
  iteration in the language, not just `List`.
- **Why it is NOT landable as written:** methods are open (ADR-0026), so reopening `List`
  to override `size`/`at` would no longer affect `for`. Under **P2** that is a spec change,
  not a perf edit. **The green golden corpus is NOT evidence of invariance** — it only
  proves the corpus never reopens `List`. Resolution already in tree: `VM::sealed_classes`
  sealed `Option`/`Some`/`None` at `8d401f4`; **sealing the kernel collections converts
  this whole family of wrapper collapses into behavior-invariant ones.**
- **ADR-0019's floor rule does not block this.** That rule bars moving `.ph` → native
  *for speed*. Collapsing a `.ph` wrapper into a call on an **existing** floor primitive is
  above the floor and does not engage it. Do not cite ADR-0019 to reject this.

---

## 3. Timing — attempted 3×, refused 3×, **no number taken**

- `benchmarks/vm/ab-guarded.py` (landed `7a8aaf3` by the concurrent session) refuses above
  `LOAD_MAX=1.5`. Measured `load1` at **5.30**, then **3.85**, on **8 cores**. Exit 3, no
  output.
- **Top consumers were WindowServer (~50%) and Claude Helper (~40%)** — i.e. the user's own
  UI. **This box does not go quiet while it is being used.** Plan around that; do not wait
  for it.
- **I did not override.** The harness docstring: *"Override only to smoke-test the harness:
  `LOAD_MAX=99` (never for a number)."*
  [F18](../../perf-log/findings.md#f18--presizing-the-fiber-vecs-is-negative-and-f3h9s-memmove-lever-is-spent-206--30)
  was a **sign** error from a contended run; F22 voided a whole `INLINE_ARGS` round the same
  way. A contended A/B cannot be repaired by more reps.
- **Do not derive F23's wall-clock from its instruction count.** The deleted instructions
  are `CallFrame` pushes (96 B), not dispatches, so
  [F19](../../perf-log/findings.md#f19--a-dispatch-costs-33-ns-and-that-is-what-a-fusion-buys-h13)'s
  3.3 ns dispatch price **does not size it**. −6M × ~9.5 ns ≈ −10% is *arithmetic, not a
  measurement*, and writing it down as an estimate is precisely how F18 happened.
- **Ready for whoever gets a quiet box** — binaries pre-built and copied out per the
  two-build protocol, both default builds at `e66af34`:
  ```
  benchmarks/vm/ab-guarded.py 15 full --bin base=/tmp/ph-base --bin probe=/tmp/ph-probe
  ```
  (`/tmp` may be cleared; the probe is 2 edits to `core.ph`, reproducible from IDEA 3.)

---

## 4. Unverified claims I made — **do not cite these as facts**

Recorded so they are not laundered into the record by repetition:

- **"Wren costs ~3 ns/instruction."** *Never measured.* I inferred it from Wren's
  47 ns/dispatch and a guessed instruction count, to argue the gap is per-instruction cost
  rather than instruction count. **Both halves of that decomposition are unmeasured.**
  Phalcom's 9.1–9.5 ns/instr *is* measured (§3bb); Wren's is not. If you need it, count
  Wren's instructions — do not inherit my number.
- **"Phalcom retires ~24 instructions per method call vs Wren's ~5."** Same problem — the
  Phalcom side is real (48.1M ÷ dispatches), the Wren side is a guess.
- **"~4.6× decomposes as ~1.5× count × ~3× per-instruction cost."** Arithmetic over two
  guesses. It is a *hypothesis worth testing*, not a finding.

---

## 5. Tooling notes for the next agent

- **`graphify query` was low-value here** (333 nodes, mostly docs, for a question about two
  opcode arms). The hook mandates it; it oriented me to `bytecode.rs:48` and the cut-009
  node, but **the decisive reads were `dispatch.rs:940-990` and `scope.rs:31` directly.**
  Budget accordingly — use graphify to *find* the file, then read the file.
- **Counts are load-independent; timings are not.** On a contended box you can still do
  real work: histograms, instruction deltas, `Return`-count deltas, correctness gates. The
  entire F23 result was produced on a box too loud to time on.
- **`Return` count is a frame counter.** It was the cleanest signal in this session —
  `.ph` frames pushed, exactly. Under-used.
- **The pair histogram** (`statically-adjacent opcode pairs`) prints at the bottom of the
  `opcode-histogram` stderr dump. For `for` at `e66af34`: `GetSelf -> InvokeLocal` 3,000,000
  (5.1%), `GetGlobal -> Invoke` 2,000,050, `Pop -> GetGlobal` 2,000,016. **These are fusion
  candidates and are therefore gated on H16 — do not size them until H16 resolves.**

---

## 6. State at end of session

**Landed:**
- `8eae379` — fix(bench): close H15.
- `36e19ff` — docs(perf-log): F23 + close H15 + open H17.

**Not landed, deliberately:** the F23 probe (blocked on behavior-invariance, §2 IDEA 3).

**Open, in priority order:**
1. **Sealing the kernel collections** — a ruling, not code. Unblocks F23 *and* the whole
   wrapper-collapse family. This is the gate.
2. **H17** — F23's timing, on any quiet box.
3. **H15's residue** — §1 is still frozen at `5254586`; instrument repaired,
   re-measurement owed. Cut 008's Wren column remains unverified.
4. **H16** — still the fattest number on the board, still gates every fusion. Note cut
   009's own probes are evidence *against* its proposed remedy: outlining was tried twice
   (+5.4% both), and what was free was `unreachable!()` (LLVM **deleting** the case). So
   the mechanism looks like match-lowering/jump-table shape, not code volume. **Re-scope
   H16 to "read the asm and find where LLVM changes switch strategy" before outlining
   anything.**
5. **F15 S4** (`CallFrame` 96 → ~32 B) — the sound version of IDEA 1's observation.

**The structural claim worth carrying forward:** cuts 001–009 are **every one of them
Rust/VM cuts**. No cut has touched `core.ph`'s call chains. That seam is unexplored *and*
exempt from H16. The work list was declared "blocked on H16" — true of fusions,
over-generalised to perf work in general.
