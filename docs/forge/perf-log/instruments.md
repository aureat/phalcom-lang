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
| Opcode histogram | `phalcom-core/src/opcode_stats.rs` | instructions retired, per-opcode mix (§3bb) | **per-opcode price** (**H13**) |
| **Opcode pair counts** | same, `PAIRS` | superinstruction candidates (F16 r2) | whether a fusion pays |
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

`for.ph`'s ranked candidates:

| fusion | count | share of all instructions |
|---|---|---|
| `GetLocal -> Invoke` | 6,000,000 | **8.8%** |
| `Constant -> Invoke` | 3,000,001 | 4.4% |
| `GetSelf -> GetLocal` | 3,000,000 | 4.4% |
| `GetGlobal -> Invoke` | 2,000,050 | 2.9% |

**The verdict did not change: still defer.** F16 reason 1 is load-bearing (S1 may
delete the motive). What changed is that the ceiling is now *measured* — the best
single fusion removes 8.8% of dispatches on `for` — rather than guessed.

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
- **A hypothesis fitted to one noisy run will appear to confirm.** Reproduce the
  *baseline observation* before explaining it (002; F13's first table).
