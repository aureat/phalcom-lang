# Instruments — what measures what, and where it lives

Companion to [README §Method](README.md#method--instrument-selection-learned-the-hard-way-in-002)
(*which* instrument to reach for) and [SCOREBOARD](SCOREBOARD.md) (the numbers). This
file is the **symbol-level map**: what exists, where, and what each one refuses to
answer.

| Instrument | Lives in | Answers | Cannot answer |
|---|---|---|---|
| Criterion micro-benches | `phalcom-core/benches/vm_bench.rs` | per-send / per-spawn ns (§3a) | anything `tracing`-sensitive (003); anything RSS (002) |
| Wren suite compare | `benchmarks/vm/compare-wren.py` | ×-vs-Wren, **output-verified** (§1) | RSS, `sys` (**H5**) |
| Whole-process time+RSS | `/usr/bin/time -l`, `benchmarks/vm/run.sh` | `user`/`sys`/`real`/peak RSS (§2) | attribution |
| **Bootstrap tripwire** | `benchmarks/vm/run.sh:65-90` | *did `VM::new` blow up* (**H7**) | why |
| Opcode histogram | `phalcom-core/src/opcode_stats.rs` | instructions retired, per-opcode mix (§3bb) | per-opcode **body** price (**H13**, narrowed) |
| **Opcode pair counts** | same, `PAIRS` | superinstruction candidates (F16 r2) | ~~whether a fusion pays~~ — **it does now**, combined with F19's ~3.3 ns dispatch price: `pairs_removed × 3.3 ns ÷ wall` |
| **Dispatch-price differential** | ad hoc; recipe in [F19](findings.md#f19--a-dispatch-costs-33-ns-and-that-is-what-a-fusion-buys-h13) | **what one dispatch costs (~3.3 ns)** | an opcode *body*'s cost |
| **Load-guarded A/B** | `benchmarks/vm/ab-guarded.py` | alternating same-session A/B; **exits 3 rather than time a busy box** (F22) | anything, while another session builds — *by design*, that is the point |
| `sample <pid>` | macOS, ad hoc | leaf-tick attribution (§4) | anything inside the dispatch `match` (F17) |

---

## Bootstrap tripwire (H7) — `benchmarks/vm/run.sh`

**Why it exists.** [F13](findings.md#f13--bootstrap-went-5-ms--180-ms-the-iftrue-inliner-is-exponential-in-nest-depth):
bootstrap regressed **5 ms → 180 ms (35×)** and **passed all three gates the harness
had** — `run.sh`'s micro-program loop only asked "did it run"; the criterion benches
amortize bootstrap inside a ~0.9 s program; the wren-suite table was single-run.
`benchmarks/vm/bootstrap.ph` existed since `8ba87ec` with **nothing asserting on it**.

**Shape** (`run.sh:65-90`): times `bootstrap.ph` whole-process, **best-of-3**, fails
over a ceiling.

```
==> bootstrap tripwire (whole-process ceiling 20 ms)
PASS     bootstrap.ph  7.7 ms  (ceiling 20 ms)
```

| knob | value | why |
|---|---|---|
| ceiling | **20 ms** (`BOOTSTRAP_CEILING_MS` overrides) | ~2.6× above HEAD's 7.7 ms, ~9× under F13's 180 ms — catches a blowup, ignores a blip |
| reps | best-of-3 | must fail on a 35× regression, not on scheduling noise |
| timer | `python3` `time.perf_counter` | `run.sh` is `bash`; no `bc`/`%N` portability bet |

**Verified in both directions** — a gate that has only ever passed is not known to be
a gate. At the default ceiling the suite passes (7.7 ms); at
`BOOTSTRAP_CEILING_MS=1` it prints `FAIL` and **`run.sh` exits 1**.

**The failure text points at the compiler, not the VM**, because that is what
bootstrap prices: `bootstrap.ph` retires ~660 instructions against ~7.7 ms of wall
(F17), so its ns/instr is ~1000× every other row. It measures `core.ph`'s *compile*.
A future blowup here is a compiler regression — profile the compiler first.

---

## Opcode histogram + pair counts — `phalcom-core/src/opcode_stats.rs`

Feature-gated `opcode-histogram`, **off by default**, zero-cost when off (verified: no
`opcode_stats` symbols in the default release binary).

| symbol | line | role |
|---|---|---|
| `COUNTS` | `:68` | `[u64; Bytecode::VARIANTS]`, per-opcode execution count |
| `PAIRS` | `:74` | `[[u64; VARIANTS]; VARIANTS]`, **statically-adjacent** `[prev][cur]` |
| `PREV` | `:79` | `Cell<Option<(ObjRef, usize, usize)>>` — last `(closure, ip, opcode)` |
| `record(opcode, closure, ip)` | `:91` | the hook; called from `dispatch.rs` under `#[cfg]` |
| `snapshot()` / `pair_snapshot()` | `:108` / `:113` | readers |
| `dump()` / `dump_pairs(total)` | `:123` / `:149` | stderr output |

Thread-local, not atomic: the VM is single-threaded (fibers are cooperative, ADR-0030),
so an atomic would cost a lock-prefixed instruction on the hottest path and distort the
very mix being counted.

`dump()` writes to **stderr** deliberately — every `tests/lang/` fixture asserts exact
stdout and `compare-wren.py` diffs stdout byte-for-byte; printing there would fail all
of them and make the feature unusable on the corpus it most needs.

### The two-build protocol — not optional

Counting costs an increment per instruction: **the same per-opcode work `vm-trace`'s
span cost 18.2% of arith wall** (003). So a timing read from a counting build is wrong.
Counts are **deterministic**, which is what makes the split sound:

1. counts from a `--features opcode-histogram` build,
2. wall-clock from a **default** build,
3. divide.

`benchmarks/vm/opcode-cost.py` mechanizes it. **Never collapse it to one run. Never
time a histogram build.**

### Why pairs are *statically* adjacent (H13's sibling, F16 reason 2)

[F16](findings.md#f16--superinstructions-are-premature-no-opcode-histogram-and-the-inliner-already-covers-the-classic-win)
deferred superinstructions partly because "the pairs cannot be chosen" — the histogram
says `Invoke` is 13–25% of every hot mix but not what it *follows*.

A pair is counted **only when `cur` is `prev`'s static successor**: same closure,
`ip == prev_ip + 1` (`record`, `:91-105`). A fusion is a compile-time rewrite of two
opcodes **adjacent in one chunk's code array**, so "the previous instruction executed"
is the wrong predicate:

- the opcode dynamically preceding a callee's first instruction is the caller's `Invoke`
- the one before a loop body's first is the bottom `Loop`

Both are execution-adjacent and **unfusible**. A naive `(prev, cur)` counter would rank
exactly those non-candidates at the top — a histogram that looks like an answer.

**Self-check.** `for.ph`: 53.0M pairs over 68.0M instructions ⇒ **15,000,011
control-flow transfers**, which the single-opcode table independently derives (5M
`Return` ⇒ 5M calls ⇒ 10M transfers, + 2M `Loop` + 2M `Jump` + ~1M taken conditionals
≈ 15M). A wrong adjacency predicate would not land on its own derivation. The deficit
(`sum(pairs) < total − 1`) is **not slack** — it is the transfer count, which is why
pair shares are reported against `total`, not against `paired`.

`for.ph`'s ranked candidates **at `5516504`** (pre-cut-008; the top two were then fused
and no longer appear — see the current table below):

| fusion | count | share of all instructions |
|---|---|---|
| `GetLocal -> Invoke` | 6,000,000 | **8.8%** |
| `Constant -> Invoke` | 3,000,001 | 4.4% |
| `GetSelf -> GetLocal` | 3,000,000 | 4.4% |
| `GetGlobal -> Invoke` | 2,000,050 | 2.9% |

### Remaining candidates at `1d2baea` (post cut 008) — measured, do not re-derive

> **⚠ GATED by [H16](SCOREBOARD.md#6-open-holes--what-is-empty-and-how-to-fill-it) — do
> not spend this table until H16 is answered.** [Cut 009](009-fuse-getself-getfield.md)
> took the `GetSelf -> GetField` row below, removed exactly the 6,333,335 dispatches it
> predicted, and **still measured net negative**: +5.9% `string_equals`, +4.8% `fib` —
> rows that execute **zero** of the new opcode. The ~5% cost is **the new arm's code in
> the dispatch loop**, and it is **larger than every ceiling in this table**
> ([F21](findings.md#f21--an-arms-code-is-paid-by-every-program-not-the-ones-that-execute-it)).
> **The counts below are still correct. The ceiling column is still an upper bound on
> the *gain*.** What the column never included is the *cost of adding the arm* — so
> read it as a ceiling, never as a net.

Ranked by count summed across the suite. **Ceiling for any row is
`count × ~3.3 ns ÷ that row's wall`** ([F19](findings.md#f19--a-dispatch-costs-33-ns-and-that-is-what-a-fusion-buys-h13)).

| fusion | suite total | fattest single row | that row's ceiling |
|---|---|---|---|
| `GuardBool -> JumpIfFalse` | 15,142,285 | `string_equals` 10,000,000 (11.9%) | **~3.9%** |
| `GetGlobal -> InvokeConst` | 12,400,004 | `map_numeric` 12,000,003 (11.1%) | ~1.1% — heavy row, will not pay |
| **`GetSelf -> GetField`** | 10,295,567 | **`method_call` 6,333,335 (13.2%)** | **~4.8%** |
| **`GetLocal -> InvokeConst`** | 10,284,565 | **`fib` 10,284,565 (17.4%)** | **~5.1%** |
| `GetGlobal -> GetGlobal` | 10,000,001 | `map_numeric` 10,000,001 (9.3%) | ~0.9% — heavy row |
| `Constant -> InvokeConst` | 10,000,000 | `string_equals` 10,000,000 (11.9%) | **~3.9%** |
| `Pop -> Constant` | 9,000,030 | `string_equals` 9,000,030 (10.7%) | ~3.5% |
| `SetGlobal -> Pop` | 8,400,002 | `map_numeric` 8,000,002 (7.4%) | ~0.8% — heavy row |

**Cut 008 created new candidates by chaining.** `GetLocal -> InvokeConst` and
`GetSelf -> InvokeLocal` did not exist before 008 — a fused opcode is itself a fusible
head, so a second round buys **3-instruction** fusions (`GetLocal, Constant, Invoke`
→ one dispatch). `fib`'s 10.3M `GetLocal -> InvokeConst` is its `fib(n-1) + fib(n-2)`
shape and is the single fattest remaining candidate in the suite.

**Read the ceiling column, not the count column.** The three fattest `map_numeric` rows
are worth ~1% each because its instructions cost 32 ns; `method_call` and `fib` are
worth ~5% because theirs cost 9–11. This is the whole of cut 008's lesson in one table.

~~**The verdict did not change: still defer.**~~ **The verdict changed — [cut 008](008-fuse-invoke-pairs.md)
landed the top two pair shapes** (`string_equals` −8.1%, `for` −5.1%). S1a did **not**
delete the motive: it removed the per-opcode *re-derivation*, leaving the ~3.3 ns
*fixed* dispatch cost a fusion deletes ([F19](findings.md#f19--a-dispatch-costs-33-ns-and-that-is-what-a-fusion-buys-h13)).

**This table's counts predicted the result exactly.** `for` retired
68,000,706 → 59,000,705 instructions, **−9,000,001** = the 6,000,000 `GetLocal ->
Invoke` + 3,000,001 `Constant -> Invoke` counted above, to the digit. **A share is
still not a price**, though — 8.8% of instructions bought 5.1% of wall, because a
fusion removes a *dispatch*, not an opcode's *work*. Multiply by ~3.3 ns, never by the
§3bb mean.

### What it still cannot do (H13)

`wall / total` is a true mean over each program's **executed mix**, not a per-opcode
price: a `Loop` and an `Invoke` land in the same average. **Do not quote a §3bb row as
"the cost of `Invoke`."** Pricing one opcode needs a *differential* — two programs
differing by a known count of exactly one opcode. The histogram makes the "known
count" verifiable, which is what would make the subtraction sound; it does not perform
it. Least-squares over the 11 execution-bound rows is underdetermined at 35 opcodes —
do not fit it and report coefficients as prices.

---

## Standing traps (each cost real time)

- **`$?` after a pipe is the last command's status, not the interesting one.** The
  logged form is `cargo test --workspace | tail` (exit code is `tail`'s), but it
  generalizes: `cargo build … | grep -E "^error"` + `$?` reports **`grep`/`head`'s**
  status and a broken build reads green. Capture the exit code of the command itself,
  then grep the log.
- **Verify each commit from a clean checkout** (`git worktree add -d <tmp> <sha>`), not
  the in-tree gate — it hides partially-staged commits.
- **Criterion certifies noise at `p = 0.00` on this hardware** (twice). Under ~10%, use
  alternating same-session A/B and read the **sign across pairs**. Its `change:` line is
  only usable when the stored baseline's provenance is known — see §3a's note, where it
  happened to be.
- **The Bash tool runs `fish`**, which does not word-split `for n in $LIST`. Use
  `python3` for any loop over a computed list. (`run.sh` itself is `bash` — unaffected.)
- **Never `cargo build` inside a measurement loop** — it contends with the bench.
- **This box is shared, and a *concurrent session* building contends exactly the same
  way** (F22, and it cost a full measurement round). The trap above says "not inside
  *your* loop"; that is not enough. Live `.claude/worktrees/agent-*` sessions and other
  Claude windows run `cargo build` whenever they like, and `git worktree list` showed
  **8** other checkouts. A probe timed at **load 7.1–10.4 on 8 cores** produced: the
  **baseline** binary drifting **~4%** between passes, 7-rep signs evaporating at 15
  reps, and `min` improving while `median` did not (the tell: contention is one-sided,
  so `min` catches the rare quiet window and `median` reports the truth).
  **`getloadavg()` lags ~1 min and will not see a short `rustc` burst** — scan for the
  process. Use `benchmarks/vm/ab-guarded.py`, which refuses (exit 3) rather than
  degrade; check `uptime` before believing any A/B, and re-run anything measured under
  load. **Alternation does not repair a contended run** — it defends against slow drift,
  not against a burst landing on one arm.
- **A hypothesis fitted to one noisy run will appear to confirm.** Reproduce the
  *baseline observation* before explaining it (002; F13's first table).
