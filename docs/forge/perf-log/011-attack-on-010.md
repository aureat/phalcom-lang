# 011 — an attack on 010, before the window is spent

**Status: ANALYSIS. Nothing timed, nothing landed.** Every number below comes from
`sysctl`, `nm`, `otool`, `shasum`, `cargo test`, or a `SCOREBOARD.md` row, and the
command is shown. The machine was at `load1 = 10.24` throughout, which is why this
document contains no wall-clock.

This document attacks [010](010-prereg-h16-h17-h13.md). It concludes that **010 should
not be run as written**, that **two of its six arms are refuted statically**, that its
headline arm **H16 is unfalsifiable as specified**, and that its runner **will probably
abort itself** partway through the window for reasons unrelated to contention.

The verdict is not that 010 is bad work. It pre-registered against itself, listed nine
self-doubts, and refused to predict a number it could not derive. That discipline is
what made the defects findable without spending the window. Three of the four decisive
refutations below come from evidence 010 or F21 already contained.

---

## 0. Outcome first

| # | Finding | Verdict | Basis |
|---|---|---|---|
| **1** | H16's I-cache mechanism is **physically impossible** on this machine | **REFUTED** | `sysctl`: P-core L1i = **196,608 B**; `run_until_inner` = 16,296 B = **8.3%** of it |
| **2** | F21 **already ran H16's experiment** and it did not recover the cost | **REFUTED** | F21 verbatim: *"outlining the whole arm body (**+5.4%**…), and reordering the functions in the file … (**+5.2%**)"* |
| **3** | H16 does **not** change the dispatch branch's target set | **CONFIRMED** | jump table decoded from both binaries: **37 slots / 36 distinct targets, base and h16, delta 0** |
| **4** | H13's "128 B init" is **8 bytes** | **REFUTED** | `otool`: the init is **8 × `strb wzr`** at stride 16, offsets `0x68…0xd8` |
| **5** | The runner's quiet-wait threshold **guarantees** the guard can trip mid-run | **CONFIRMED** | wait needs `load1<1.5` at idle; harness's own child adds **~1.0** to `load1` within ~5 min; guard then needs ambient `<0.5` |
| **6** | Position is **not** balanced at 15 reps (010 claims it is) | **CONFIRMED** | mean position: base **3.333** … prev **3.667**, monotone in arm order |
| **7** | h16 is **not** a single-variable cut | **CONFIRMED** | `sub sp,sp,#0x400` → `#0x350` (**−176 B frame**); `x23`→`x21` regalloc change; `invoke_at` relocated **−1,404 B** |
| **8** | h17 **is** layout-clean — the only uncontaminated arm in the batch | **CONFIRMED** | `__text` same size; all diffs inside `run_until_inner` are at instruction-word byte-offsets **1–2** (immediate fields), 106 of 16,296 B |
| **9** | The repo has **no miri lane**; miri was **not installed** | **CONFIRMED** | `rustup component list --installed` — no miri; nightly present; installed during this session |

**The single most important finding is #2.** F21's own text records that outlining the
arm body was tried and left the cost at +5.4%. H16's probe is outlining. H16 is built
on the one framing F21's own counter-probes refute.

---

## 1. Ground truth (Phase 0)

| ID | Question | Answer | Command |
|---|---|---|---|
| **G1** | CPU / caches | **Apple M1 Pro**, 8 cores = 6 P + 2 E. **P-core L1i = 196,608 B (192 KiB)**, L1d 131,072 B, L2 12,582,912 B. E-core L1i = 131,072 B. Line 128 B | `sysctl -n machdep.cpu.brand_string hw.perflevel0.l1icachesize …` |
| **G2** | P/E placement pinned? | **No.** Nothing in `ab-guarded.py` or `run-phase-b.sh` sets QoS or calls `taskpolicy`. Core cluster is unrecorded and uncontrolled | read of both files |
| **G3** | Thermal | Not sampled by the harness. A 12–20 min run has a thermal ramp inside it and nothing records it | read of both files |
| **G4** | Release profile | **No `[profile.*]` section exists anywhere in the workspace.** No `.cargo/config.toml`. `RUSTFLAGS` empty. So: `opt-level=3`, **`lto=false`**, **`codegen-units=16`**, `panic=unwind`, no `target-cpu` | `grep -rn profile Cargo.toml`; `cat .cargo/config.toml` |
| **G5** | `Value` / `CallFrame` size | **`CallFrame` = 96 B — read straight out of the binary** (`mov w11, #0x60; madd x8, x8, x11, x10`). **`Value` = 16 B** — the arg-buffer init writes 8 tags at stride **0x10** | `otool -tvV` on `run_until_inner` / `call_method` |
| **G6** | Manifest + arms intact | **All 8 binaries OK** | `shasum -a 256 -c MANIFEST.sha256` |
| **G7** | miri / workspace tests | **miri was NOT INSTALLED** (nightly toolchain present; installed this session; run in progress). `cargo test --workspace`: **h16 358 passed / 0 failed / 16 ignored, exit 0**; **h13b 358 / 0 / 16, exit 0** | `rustup component list`; `cargo test --workspace` |
| **G8** | Symbol sizes | **010's size table is arithmetically correct.** Independently reproduced via `nm -n` + next-symbol subtraction in Python, symbol names verified unique (1 match each), `call_method`'s mangled hash identical across base/h16. **But see §3 — the table is true and misleading** | `nm -n` + Python |

**G4 is a standing result, not a detail.** `codegen-units=16` with `lto=false` means
function placement is a *global* function of the source. Any edit anywhere can move
anything. "Layout is an uncontrolled term" is therefore true **by construction**, before
any measurement — which is half of 010's meta-question answered for free.

---

## 2. The mechanism, re-derived (F21 and H16)

### 2.1 What F21 actually measured

| build | 38th arm present in the compiled loop? | result |
|---|---|---|
| base | — | reference |
| `unreachable!()` body (**LLVM erases it**) | **no** | **−0.6%** |
| real body, inline | yes | **+6.1%** |
| real body, **outlined** | yes (as a call) | **+5.4%** |
| shared field read outlined | yes | **+5.4%** |
| functions reordered in the file | yes | **+5.2%** |

Read the middle column. **Every build in which the arm exists is slow. The one build in
which it does not exist is fast — and that build is base.** LLVM erased the arm, so its
`run_until_inner` is base's `run_until_inner`.

Two consequences 010 does not draw:

1. **The `unreachable!()` row is a null probe.** It compares base to base. It cannot
   discriminate any mechanism. What it *does* supply is the only noise-floor estimate
   in the log: **~0.6% for one build pair, one session**. That is the number every
   effect below ~1% must be read against, and it is the only such number that exists.

2. **F21 has no negative control.** Every slow build is a *perturbation* of base. F21
   concludes "systematic, not a per-build coin flip" from four perturbations agreeing in
   sign. But a model in which **base happens to sit at a favourable layout point and
   essentially any perturbation lands ~5% worse** predicts exactly that data, including
   the sign agreement. F21 never built the control that separates the two: a
   semantically-neutral perturbation of comparable size. **This is the null hypothesis
   for the entire log, and it is untested.**

### 2.2 Mechanism ranking, after G1

| mechanism | verdict | why |
|---|---|---|
| **M-icache** (L1i capacity/conflict) | **REFUTED, twice** | (a) P-core L1i is **196,608 B**; the loop is **16,296 B = 8.3%**, and the *arm-entry span* is only **5,172 B = 2.6%**. A contiguous 16 KB region cannot capacity- or conflict-miss a 192 KiB cache. (b) F21 outlined the body — which restores footprint — and the cost **stayed at +5.4%**. If footprint were the mechanism, outlining recovers it. It did not |
| **M-align / layout lottery** | **PLAUSIBLE — leading, and untested** | The null hypothesis of §2.1. `codegen-units=16`, `lto=false` (G4) makes placement a global function of source. Mytkowicz et al., ASPLOS 2009, "Producing Wrong Data Without Doing Anything Obviously Wrong!": link order and environment size shift alignment enough to produce effects **larger than the optimization under study, with a consistent sign** |
| **M-btb** (38th live indirect target) | **PLAUSIBLE — the only mechanism that fits all five F21 rows** | The loop has **exactly one `br x`** (measured, §2.3). Erased arm ⇒ 37 targets ⇒ fast. Real arm, inline **or outlined** ⇒ 38 targets ⇒ slow. Outlining does not change target count, which is precisely why F21's outlining probe recovered nothing |
| **M-regalloc** | **PLAUSIBLE — newly evidenced here** | h16 changes the loop's frame `0x400`→`0x350` and reassigns registers (§3). A real 38th arm adds live values across a 4,074-instruction function |
| **M-uop** | **PLAUSIBLE, weak** | Apple Silicon has no documented x86-style µop cache; unsupported speculation |

### 2.3 What the dispatch loop actually is (measured, `otool -tvV`)

```
ldr  x8, [x23, #0x40]        ; frames.len
cbz  x8, <panic>             ; .unwrap()
ldr  x9, [x20, #0x18]        ; chunk.code ptr
add  x9, x9, x27, lsl #3     ; + ip*8      <- Bytecode is 8 B
ldrh w5,  [x9, #0x2]         ; operands, free with the opcode
ldrsw x21,[x9, #0x4]
ldrb w28, [x9, #0x1]
ldr  x10, [x23, #0x38]       ; frames.ptr
ldrb w9,  [x9]               ; THE OPCODE
mov  w11, #0x60              ; 96 = size_of::<CallFrame>()   <- rematerialized every dispatch
madd x8, x8, x11, x10        ; frames.ptr + len*96   <- a MULTIPLY, every dispatch
ldur x10, [x8, #-0x18]       ; frame.ip
add  x10, x10, #0x1          ;   += 1
stur x10, [x8, #-0x18]       ;   store back          <- store->load dependency, every dispatch
adrp x11, …; add x11, x11, #0x8a0
adr  x8, #16
ldrh w10, [x11, x9, lsl #1]  ; jump table, u16 entries
add  x8, x8, x10, lsl #2
br   x8                      ; the ONLY indirect branch in the function
```

Source: `dispatch.rs:568` — `self.frames.last_mut().unwrap().ip += 1;`

**One `br x`. 37 jump-table slots. 36 distinct targets.** (One pair of opcodes shares a
target.) The arm entries span **5,172 B**.

**Nothing in 010 measures any of this.** The `frames.last_mut()` recompute — an
emptiness check, a rematerialized constant, an integer multiply, and a
load-add-store round trip through memory — happens on **every single opcode**, and it is
not on any hole, any finding, or any arm.

### 2.4 The decisive fact about h16

The jump table was decoded from both binaries and the targets constrained to lie inside
`run_until_inner`:

```
base   37 slots -> 36 DISTINCT targets   arm-entry span 5,172 B
h16    37 slots -> 36 DISTINCT targets   arm-entry span 4,152 B
delta  0
```

**h16 does not remove a target. It replaces five arm bodies with five calls.** The
indirect branch sees the same 37 slots and the same 36 targets it saw before.

So:

- Under **M-btb**, h16 predicts **exactly zero**.
- Under **M-icache**, h16 predicts multi-percent — but M-icache is refuted (§2.2).
- Under **M-align**, h16 predicts a **random draw**, and h16 happens to be the only arm
  whose `run_until_inner` is **64-byte aligned** (§3) — so a positive result is
  attributable to the lottery, not to outlining.

**Under every surviving mechanism, H16 either predicts null or cannot be read.**

### 2.5 H16's prediction block is unfalsifiable (D3)

010 §2 offers three outcomes:

- uniform same-sign win ⇒ "footprint-proportional and monotone"
- split sign ⇒ "I-cache set conflicts, alignment, branch-predictor pressure"
- null <1% ⇒ "the tax is asymmetric… strange but survivable"

The block encodes **only M-icache**, which G1 refutes. And the three outcomes do not
partition the mechanisms: a null is consistent with M-btb *and* M-align *and*
"asymmetric footprint" — three explanations, one observation, no discriminator. A
positive is consistent with M-align (h16 drew 64-byte alignment) as readily as with
H16's own story.

**There is no result H16 can return that changes what anyone should do next.** That is
the bug D3 says to hunt for. It is the headline arm.

---

## 3. h16 is not a single-variable perturbation

010 §2 claims: *"The two neighbouring hot functions are byte-identical in size, so this
is a clean single-variable perturbation of exactly the quantity F21 indicted."*

Identical **size** was verified (G8). Identical **placement** was not checked, and does
not hold:

```
                    addr base -> h16              size
run_until_inner     0x100052ce8 -> 0x1000532c0    (+1,496 B)   16,296 -> 12,088
call_method         0x100051db4 -> 0x100051db4    (   +0 B)     1,136 ->  1,136
invoke_at           0x10005701c -> 0x100056aa0    (-1,404 B)     1,584 ->  1,584
```

`invoke_at` is byte-identical and **moved 1,404 B**. `op_make_family` now sits at
`0x100052ce8` — base's `run_until_inner` address.

And the loop itself was **recompiled**, not merely shortened:

```
base : sub sp, sp, #0x400     mov x23, x1     str x0, [sp, #0x88]
h16  : sub sp, sp, #0x350     mov x21, x1     str x0, [sp, #0x68]
```

**−176 B of stack frame (−17.2%) and a different register assignment for the same
value.** Outlining five arms changed regalloc across the whole function.

Loop entry alignment, all six arms:

| arm | `run_until_inner` | mod 64 |
|---|---|---|
| base | `0x100052ce8` | 40 |
| **h16** | `0x1000532c0` | **0** |
| h17 | `0x100052ce8` | 40 |
| h13b | `0x100052cd0` | 16 |
| h13 | `0x100052d48` | 8 |
| prev | `0x10004ad74` | 52 |

**h16 is the only arm that drew 64-byte alignment.** h16 measures *outlining ⊕ frame
shrink ⊕ regalloc ⊕ relocation ⊕ alignment*. Five variables.

`prev` is worse: every symbol moved **~−32.6 KB**. `prev` vs `base` is not a 248 B
footprint perturbation — it is a **wholesale relocation**. It cannot test F21's variable.
It is a **layout draw**, and that is the only thing it can honestly be read as.

---

## 4. H13 is refuted by disassembly — no window needed

010 §4: *"`let mut args = [Value::Nil; INLINE_ARGS];` — 8 × 16 B = **128 B initialized
on every primitive send**"*.

What `base`'s `call_method` actually contains:

```
0000000100051e54  cmp   x23, #0x9            ; arity <= 8 ?
0000000100051e58  b.hs  0x100051f58
0000000100051e5c  strb  wzr, [sp, #0x68]     ; \
0000000100051e60  strb  wzr, [sp, #0x78]     ;  |  stride 0x10 = 16 B
0000000100051e64  strb  wzr, [sp, #0x88]     ;  |  => Value is 16 B (G5)
0000000100051e68  strb  wzr, [sp, #0x98]     ;  |
0000000100051e6c  strb  wzr, [sp, #0xa8]     ;  |  8 SINGLE-BYTE stores.
0000000100051e70  strb  wzr, [sp, #0xb8]     ;  |  8 bytes written. Not 128.
0000000100051e74  strb  wzr, [sp, #0xc8]     ;  |
0000000100051e78  strb  wzr, [sp, #0xd8]     ; /
0000000100051e7c  subs  x0, x1, x23
0000000100051e84  lsl   x2, x23, #4          ; memcpy len = arity*16
0000000100051e8c  add   x0, sp, #0x68
0000000100051e90  bl    _memcpy              ; <- symbol-stub call
```

**LLVM already elided the payload initialization.** `Value::Nil` carries no payload, so
`[Value::Nil; 8]` compiles to **eight discriminant-byte stores** — 8 bytes, 8
instructions. The init is real; it is **16× smaller than 010 claims**.

The `base → h13b` instruction diff confirms the probe removes exactly these and nothing
else:

```
< strb wzr, [sp, #ADDR]   x8
< subs x0, x1, x23
---
> cmp  x1, x23
```

Net **−6 instructions / −24 B**, matching the symbol table.

**Sizing it.** 8 stores at ~2/cycle on Firestorm ≈ 4 cycles ≈ **~1.25 ns**, to two
already-hot stack lines, on no dependency chain, fully reorderable. Against `bare_send`
at **~144 ns/send** (SCOREBOARD §3a: 28.83 ms / 200,000 sends) that is a ceiling of
**~0.87%** — **at or under the only noise-floor estimate in the log (F21's −0.6%)**.

> **H13's body half is CLOSED by disassembly, not by timing.** The hole asks what the
> 128 B init costs per primitive send. There is no 128 B init. There are 8 bytes, with a
> ceiling below the noise floor. `h13b` and `h13` are **2/6 of the window buying a
> number that cannot clear the noise floor**, to answer a question whose premise is
> false.

Pull both arms.

**What the disassembly found instead.** `copy_from_slice` lowers to a **symbol-stub call
to `_memcpy`** with length `arity*16` — for the common `arity == 1`, an indirect call
through the stub to move **16 bytes**. There are three such calls in `call_method`. That
is plausibly an order of magnitude more than the init the probe targets, and it is
un-held by any hole. See T4 in §8. This is a *new static observation*, not a
resurrection of F22's void round.

---

## 5. h17 is the only clean arm in the batch

Verified here, and stronger than 010 claims:

- `run_until_inner`: **same size (16,296 B) and same address (`0x100052ce8`)** as base.
- `call_method`, `invoke_at`: same size, same address.
- `__TEXT,__text`: **same size** (`0x146180`) as base.
- Inside `run_until_inner`, **106 of 16,296 bytes differ**, and **every differing byte
  is at offset 1 or 2 of a 4-byte instruction word** — immediate fields, not opcodes.
  These are `adrp`/`add` immediates chasing `__cstring` addresses that shifted because
  the embedded `core.ph` changed length.

**h17 runs base's exact machine code, at base's exact addresses, in base's exact
register allocation.** It has no layout confound at all. It is the only arm in the batch
of which that is true, and 010 undersells it: §3(b) claims "+0 B", which is the weaker
statement.

This matters beyond h17. **h17's non-`for` rows are a within-run, layout-matched,
base-vs-base noise measurement** — seven rows of it, free, in the same session. 010 lists
"other rows flat" as a *falsifier*; it is also the **noise floor estimator** the log has
never had, and it is worth more than that.

---

## 6. Ruling on the nine self-declared weaknesses

**#1 — positional bias. CONFIRMED, and worse than stated. Fix is free.**

010 says *"Mean position is 3.5 for all — balanced — but the spread is not."* **The mean
claim is false at 15 reps.** `order = arms if r % 2 == 0 else reversed(arms)` gives 8
forward and 7 reversed passes at `reps=15`:

| arm | base | h16 | h17 | h13b | h13 | prev |
|---|---|---|---|---|---|---|
| mean position | **3.333** | 3.400 | 3.467 | 3.533 | 3.600 | **3.667** |

Monotone in arm-list order. Any **linear** position effect — thermal ramp within a rep,
DVFS, cache state — biases the arms in exactly the order they are listed, and `base`,
the thing every `d%` is measured against, sits at the extreme. The imbalance vanishes at
`reps=16`.

**Do not accept the "6-arm vs 5-pairwise" dilemma. There is a third option: randomize.**
A random permutation per rep, or a balanced Latin square (6 arms ⇒ 12 or 18 reps),
balances position at **every moment**, not just the mean. It is ~3 lines in the runner,
costs **zero window time**, and preserves 010 §7's real win (one base, identical machine
conditions). Pairwise runs cost 960 runs vs 600 and give drift a pair to land on.

**Ruling: keep the 6-arm design; randomize the order; use an even rep count.** Weakness
#1 dissolves. 010's §7 justification survives — it was right for the wrong rep count.

**#2 — `min` as the statistic. CONFIRMED as a hazard; the fix is not to choose.**
`min` estimates "the best machine state we happened to sample", which is a different
estimand per arm when arms sample states unequally (#1). F22 records the fingerprint of
it lying. But the argument is unnecessary: **the harness already computes the full
per-rep vector `t[label]` and then throws it away.** Emitting it costs **zero window
time** and makes `min`, median, IQR, and any future statistic recomputable post hoc,
forever, without re-running. Do that instead of arguing.

**#3 — all-or-nothing. CONFIRMED; fix is checkpointing, not shrinking.** `sys.exit(3)`
at minute 18 discards minutes 1–17, which are perfectly good data taken on a quiet box.
Append each rep's raw vector to disk as it completes and make the runner resumable. Then
a burst costs **one rep**, not the window. Same shape as #1: the weakness is in the
runner, not the design. Do not shrink the batch to work around a missing `flush()`.

**#4 — h16 changes bootstrap. REFUTED as a concern — bound it, don't hand-wave.** 010
asks for arithmetic rather than judgement, so: `op_class`/`op_method` run once per class
and per method definition. SCOREBOARD §3bb records bootstrap at **661 instructions**
(`Constant 31%, Method 26%, GetGlobal 9%`), and bootstrap wall is **7.7 ms** (H7). An
added non-inlined call is ~1–2 ns. Even at 10³ definitions that is **~microseconds
against 0.5 s rows** — six orders of magnitude down. Immaterial, **confirmed by
arithmetic**. This was 010's best-audited weakness and it holds.

**#5 — h13b's unsafe. Partially cleared; the arm is moot anyway (§4).**
- **The handoff's claim that "the repo has a miri lane" is FALSE.** `rustup component
  list --installed` shows **no miri**. There is no miri lane in CI, `Makefile`, or any
  script; the only hits are skill documentation. Nightly *is* installed. Miri was
  installed during this session and run.
- **By inspection the unsafe is sound**: `MaybeUninit<Value>` is guaranteed same
  size/align/ABI, so `&[Value] → &[MaybeUninit<Value>]` only forgets information;
  `copy_from_slice` initializes exactly `arity` elements; `Value: Copy` with no `Drop`;
  `src`'s borrow of `self.stack` ends at `copy_from_slice` (NLL) before `native_fn` takes
  `&mut self`; `init` borrows a local, not `self`.
- **State plainly what miri clears and what it does not**: miri finds UB on the paths it
  executes. It does **not** clear miscompilation risk, and a green miri on `invariants`
  is evidence about `invariants`, not about every arity. Result recorded in §11.

**#6 — only `--test lang` was run. CLOSED.** `cargo test --workspace`, true exit codes
captured (the first attempt piped through `tail`, so its exit code was `tail`'s and was
discarded):
- **h16: 358 passed / 0 failed / 16 ignored, exit 0.**
- **h13b: 358 passed / 0 failed / 16 ignored, exit 0.**

**#7 — no RSS. CONFIRMED; fix is free.** `ab-guarded.py` calls `subprocess.run` and
never reads rusage. `os.wait4()` or `resource.getrusage(RUSAGE_CHILDREN)` gives peak RSS
with no extra process launches and **zero window cost**. SCOREBOARD's own law is "a row
without RSS is half a row" (H5).

**#8 — `prev` is confounded. AGREE it is confounded; DISAGREE about what it is.** 010
calls it "a natural experiment… an *unintentional* perturbation of exactly F21's
variable". It is not. **Every symbol in `prev` moved ~−32.6 KB** (§3). That is not a
248 B footprint perturbation; it is a wholesale relocation with a 17-file selector
rename and changed interned string lengths on top. It cannot test F21's variable.
**Keep the arm — but re-label it.** It is a **layout draw**: two builds of near-identical
semantics at very different addresses. Read that way it is evidence about M-align, which
is the leading mechanism, and it is the *only* such evidence in the batch. Read 010's
way — "if F21 generalises, `4c6c83f` is faster on rows that never execute SuperSend" —
it is uninterpretable.

**#9 — `LOAD_MAX=1.5` achievable? NO. CONFIRMED, and this is the finding that will
actually cost the user their window.**

The harness runs one child at a time, back to back, ~100% duty cycle. One continuously
runnable process asymptotes `load1` to **1.0**. `load1` is a ~1-min EMA:

| elapsed | harness's OWN contribution to `load1` | ambient headroom under `LOAD_MAX=1.5` |
|---|---|---|
| 0 s | 0.000 | 1.500 |
| 60 s | 0.632 | 0.868 |
| 300 s | 0.993 | **0.507** |
| 720 s | 1.000 | **0.500** |

`quiet_check("preflight")` runs at **t=0**, when the harness contributes **0.0** — so it
passes at ambient up to 1.5. `run-phase-b.sh`'s wait loop uses the **same** threshold:
`os.getloadavg()[0] < 1.5`, also at idle. So the script happily starts at ambient = 1.4,
and by minute 5 `load1` = 1.4 + 1.0 = **2.4 > 1.5 → exit 3**, discarding the whole run.

**The wait loop's success condition guarantees the guard can fire.** The two thresholds
must differ, because the harness's own load is present in one and absent from the other:

- **wait / preflight** (harness idle): needs `load1 < ~0.5`
- **per-rep** (harness running): `load1 < 1.5` is correct as-is

`LOAD_MAX=1.5` is the *right* steady-state number. **The bug is that `run-phase-b.sh`
waits for 1.5 instead of 0.5.** The user's box read `load1 = 10.24 / 6.81 / 7.08`
(`uptime`, this session); they must know the target is **0.5**, not 1.5, or they will
close their apps, wait for 1.4, start, and lose the window at minute 5 having done
everything right.

This is not a reason to bypass the guard. It is one number in one script.

---

## 7. Verdict and the modified plan

**Verdict: DO NOT RUN 010 as written. RUN MODIFIED.**

### Must happen before the window opens (all zero window cost)

| # | Change | Why |
|---|---|---|
| **B1** | `run-phase-b.sh`: wait for **`load1 < 0.5`**, not 1.5 | §6 #9. Otherwise the run aborts itself mid-window |
| **B2** | Emit the **raw per-rep vector** per row per arm to disk | §6 #2. Already computed and discarded. Makes every future statistic free |
| **B3** | **Checkpoint per rep**, resumable | §6 #3. A burst costs one rep, not the window |
| **B4** | **Randomize arm order per rep** (seeded, recorded); rep count **even** | §6 #1. Kills positional bias at every moment, not just the mean |
| **B5** | **Record RSS** via `os.wait4` | §6 #7. Free |
| **B6** | Record per rep: `load1`, elapsed, arm order, **executing core cluster** | G2/G3. Makes the confounds covariates instead of lurking terms |
| **B7** | **Drop `h13b` and `h13`** | §4. Premise refuted statically; ceiling ~0.87% is under the noise floor |
| **B8** | Rewrite **§2's prediction block**; re-label **`prev`** as a layout draw | §2.5, §6 #8 |

### The arm list

| slot | arm | what it now is |
|---|---|---|
| 1 | `base` | reference |
| 2 | **`base-p1`** | **NEW — layout control** |
| 3 | **`base-p2`** | **NEW — layout control** |
| 4 | `h17` | the only layout-clean semantic probe (§5) |
| 5 | `h16` | demoted: a layout draw that also shrinks the frame (§3) |
| 6 | `prev` | a layout draw (§6 #8) |

Six arms, same window. `h13b`/`h13` (refuted) are traded for two layout controls.

### The new arms — **built and verified during this session**

`base-p1` and `base-p2` are built from **unmodified source at `4c6c83f`**. No source
file was touched; the perturbation is entirely in `RUSTFLAGS`, so `dispatch.rs` is not
edited and the concurrent U-CTOR sessions are unaffected.

| arm | how | `run_until_inner` move | align mod 64 |
|---|---|---|---|
| `base` | — | — | 40 |
| **`base-p1`** | `-C llvm-args=-align-all-functions=5` | **+12,952 B** | **0** |
| **`base-p2`** | `-C link-arg=-Wl,-order_file,…` (300 of 3,138 `__text` symbols, seed 20260715) | **+120,624 B** | **24** |

Validity, checked before use:

- **`run_until_inner`'s instruction sequence is identical to base in both.** The only
  disassembly differences are `adrp` page immediates (data moved), trailing alignment
  `nop`s, and — in p2 — `bl` targets whose *mangled hash suffix* changed because
  `RUSTFLAGS` feeds rustc's `-C metadata`. **No instruction, register, or ordering
  differs.** Verified by normalized diff: the non-`adrp`, non-`nop` difference set is
  empty for p1 and is exactly one symbol-hash class for p2.
- **Byte-identical stdout to base on all 8 `full` rows.** Verified.
- Installed to `~/phalcom-perf-phaseA/`, added to `MANIFEST.sha256` (all **10** binaries
  verify), provenance in `~/phalcom-perf-phaseA/BUILD-PROVENANCE.txt`.

Build cost: two `cargo build -r`, already paid. Window cost: 2/6 of the run — **paid for
by B7**.

**A caveat that must be recorded, because it is p1's weakness.** `-align-all-functions=5`
is not a *random* draw — it aligns every function to 32 B, which is a systematic
intervention that could plausibly help on its own. `base-p2`'s order-file permutation is
the more neutral draw and is the one to weight. Reading them together is the point: two
mechanisms, two very different displacements (+13 KB, +121 KB), two different alignments
(0, 24). If both are flat, layout is flat. If they disagree with each other, layout is a
lottery and that is the finding.

### The reproducible-build result, which re-reads F21

While building the controls: **a rebuild of unmodified `4c6c83f` reproduces `ph-A-base`
byte-for-byte** (sha256 `6832e576…`, verified). Builds here are deterministic.

This retires one of F21's arguments. F21 says of its four slow builds: *"Systematic, not
a per-build coin flip."* But **no build here is a coin flip** — the same source always
gives the same binary. "Not a coin flip" is guaranteed by determinism and carries **zero
information** about whether the effect is layout or arm code. The layout distribution can
only be sampled by changing the source, the flags, or the link — which is exactly what
`base-p1`/`base-p2` do, and exactly what F21 never did.

An unrelated but load-bearing artifact of the same exercise: a first attempt added a
`#[unsafe(no_mangle)] #[inline(never)]` pad function to `phalcom-common`. It was
**dead-stripped** and the binary came out byte-identical to base. Source-level padding in
a library crate does not work as a layout probe on macOS; use `RUSTFLAGS`.

---

## 8. Pre-registration addendum (recorded before any number exists)

### H18 — is the layout term ±5% or ±0.5%?

**Hypothesis.** The +5.2%…+6.1% F21 attributes to "an arm's code" is a draw from the
build-layout distribution, not a property of the arm.

**Mechanism.** `codegen-units=16`, `lto=false`, no `target-cpu` (G4) ⇒ function
placement is a global function of source. Mytkowicz et al. (ASPLOS 2009) show link order
and environment size alone produce effects larger than the optimizations under study,
**with a consistent sign**. Curtsinger & Berger (STABILIZER, ASPLOS 2013): the remedy is
to randomize layout and measure the distribution, converting a systematic bias into a
random effect with a confidence interval.

**Prediction.** `base-p1` and `base-p2` execute **byte-identical semantics** to `base`.
Their true effect is **exactly zero**. Any `d%` they show is layout + noise.

**Falsifier — and this is the point: it fires in both directions.**

- **|d%| ≥ ~3% on any execution-bound row** ⇒ layout is worth several percent, **F21's
  +5–6% is not evidence about arms**, H16 is unreadable in principle, cut 008's −8.1%
  and cut 009's +5% are part-lottery in unknown ratio, and **every cut in SCOREBOARD §1
  carries an uncontrolled layout term**. That re-prices the whole log and is the largest
  finding the round can produce.
- **|d%| ≤ ~1% on every row** ⇒ layout is *not* a several-percent term on this machine,
  F21's effect is **real and needs a mechanism**, and M-btb (§2.2) becomes the leading
  candidate — under which h16's measured **delta-0 target count** predicts null and the
  fusion list is gated on target count, not bytes.
- **1% < |d%| < 3%** ⇒ layout is a real but sub-fusion term; every future A/B needs ≥2
  layout draws per arm, and the log's sub-2% rows are unsafe.

**What a null buys.** A null (**≤1%**) is a *positive* result: it retires M-align, makes
every past cut's headline defensible for the first time, and promotes M-btb from
speculation to the standing hypothesis. There is no outcome of H18 that fails to change
what happens next. Contrast H16 (§2.5), which has no such outcome.

**Cost.** Two builds (free, load-insensitive). 2/6 of the window, paid for by B7.

### H16 — rewritten

**The I-cache framing is withdrawn (G1, §2.2). H16 as posed cannot be answered by this
arm.** Retained at 1/6 window as a **third layout draw**, not as a test of footprint.

**Prediction:** under M-btb, **null** (h16 does not change the target set — measured,
delta 0). Under M-align, a draw indistinguishable from `base-p1`/`base-p2` — except that
h16 is the only arm that drew **64-byte alignment** (§3), so a positive is attributable
to alignment before it is attributable to outlining.

**Falsifier:** if h16 moves and `base-p1`/`base-p2` do not, outlining did something and
the mechanism hunt narrows to frame size / regalloc (§3) — **not** to footprint.
If h16 and the controls move together, h16 measured the lottery.

**010 §2's "split sign" falsifier is withdrawn.** With M-icache refuted, a split sign is
the *expected* signature of M-align, not a surprise.

### H17 — unchanged, and promoted to headline

010 §3's prediction stands **exactly as written** and needs no amendment: direction only
(`for` improves, other rows flat), **no number predicted**, do not derive wall-clock from
the instruction count (F15/F18/F19). This is the batch's best-specified hypothesis.

Strengthened by §5: h17 is **byte-identical machine code at identical addresses**, so
`for` improving is attributable to the `.ph` cut and nothing else. It is the only arm in
the batch of which that is true.

**Added, free:** h17's **seven non-`for` rows are a layout-matched base-vs-base noise
measurement** in the same session. Record them as the noise floor; the log has never had
one better than F21's single-pair −0.6%.

**Landing blocker unchanged**: ADR-0026 class reopening. The green corpus is evidence the
corpus does not reopen `List`, nothing more.

---

## 9. Ranked backlog

Rank = (expected effect × confidence) ÷ (window cost + landing cost).

### T0 — measurement validity (gates everything)

| item | cost | note |
|---|---|---|
| B1 wait threshold 1.5→0.5 | 1 line | **without this the window is probably lost** |
| B2 raw vectors | ~5 lines | already computed, thrown away |
| B3 per-rep checkpoint + resume | ~15 lines | turns a lost window into a lost rep |
| B4 randomized order, even reps | ~3 lines | kills §6 #1 at every moment |
| B5 RSS via `os.wait4` | ~5 lines | SCOREBOARD's own law |
| B6 machine state per rep | ~10 lines | makes F22's confound a covariate |
| **H18 layout control arms** | 2 builds + 2/6 window | **re-prices the entire log** |
| PMU counters | unknown | see §10 — feasibility unestablished |

### T1 — the interpreter loop (VM level)

| # | item | ceiling + basis | conf | preclusion |
|---|---|---|---|---|
| **1** | **Hoist the top-frame pointer.** Every dispatch recomputes `frames.ptr + len*96` — emptiness check, rematerialized `#0x60`, **integer multiply**, then a **load-add-store round trip** to `frame.ip` (`dispatch.rs:568`, §2.3) | ~8 instructions + a store→load dependency out of a ~3.3 ns dispatch (F19). **Not derivable** (L6) — must be measured | **high** that it is real; **unknown** size | Cut 007's `closure_id` guard is sound *because* `ip` is not hoisted (`s1a-guard-is-closure-not-frame`). Needs a frame-identity guard first. Fights `&mut self` |
| 2 | Threaded / computed-goto dispatch | **one `br x`, 37 slots, 36 targets** (measured). Ertl & Gregg (JILP 2003): replication gives each opcode its own predictor history | high mechanism | **Rust cannot express it.** No computed goto; `become` (RFC 3407) status unverified — **check, do not assume**. An implementation-language limit, not a design flaw |
| 3 | **PGO** (`cargo-pgo`) | The *controlled* form of the variable H16 probes by hand: hot/cold splitting + basic-block ordering | medium | Re-baselines the whole log. Do it **before or after** the batch, never between. BOLT is ELF-oriented — verify Mach-O before proposing |
| 4 | Build knobs: `lto=fat`, `codegen-units=1`, `panic=abort`, `target-cpu=native` | Unclaimed (G4: **no profile section exists**). `codegen-units=1` also **shrinks the layout distribution**, i.e. partly *fixes* H18's problem | medium | Same re-baselining hazard as PGO. Interacts with every layout measurement |

### T2 — representation

| # | item | ceiling + basis | conf | preclusion |
|---|---|---|---|---|
| 5 | **`Value` 16 B → 8 B** (NaN-box / niche-pack) | `Value` = 16 B, `CallFrame` = 96 B (G5, from the binary). Halves operand-stack traffic, frame size, arg-buffer. **Subsumes H13 entirely** | medium | NaN-box ⊗ moving GC; ⊗ `Option` niche absence. ADR-0010 names NaN-boxing as the successor and `Value::obj_ref` is already the GC's sole seam — the escape hatch is **built**. Precedent: LuaJIT/SpiderMonkey NaN-box and pay in pointer-width assumptions and platform lock-in; Wren tagged-unions and stays portable and slower |
| 6 | `CallFrame` 96 B — what is derivable? | 96 B × every call; `for` pushes **4 frames/element** | medium | `caller_source: Option<SourceRange>` and `generation: u64` are stack-trace/identity infrastructure. Shrinking costs diagnostics |

### T3/T4 — dispatch and calls

| # | item | ceiling + basis | conf | preclusion |
|---|---|---|---|---|
| 7 | **`memcpy` stub call for `arity*16` bytes** (§4). Three in `call_method`; the common case is `arity == 1` = **16 bytes through a symbol stub** | Plausibly ≫ the 8-byte init H13 targeted. **A new static observation, not F22's void round** | medium | `match arity { 1 => …, 2 => …, _ => copy_from_slice }` needs no opcodes and no const generics. F22 killed `CALL_0..16` on *mechanism* — this route does not touch the opcode set. **Re-read F22 before acting** |
| 8 | Monomorphic inline cache per send site | F12 landed the global slot cache; sends are the other half | medium | **IC ⊗ mutable hierarchy** — every IC needs an invalidation epoch unless classes are sealed. Fuses with T5 |

### T6 — compiler / bytecode

| # | item | status |
|---|---|---|
| 9 | Superinstruction fusion | **Gated on H18, not H16.** F21's ~5% tax exceeds every remaining fusion's ceiling (best ~5.1%, `GetLocal -> InvokeConst`). If H18 says layout is ±5%, the "tax" may not exist and the list is **alive**; if H18 says ±0.5%, the tax is real, and M-btb says it is priced in **live target count**, so fusion (which *removes* a target) may be **free or negative-cost** — the opposite of the current reading |
| 10 | Quickening (Brunthaler) | Adds arms. Same gate |
| 11 | Register bytecode (Lua 5.0) | Enormous ceiling, enormous landing cost, invalidates every number in the log. **Named so a reader need not ask. Defer** |

### T7 — GC

| # | item |
|---|---|
| 12 | **Main has a full GC + Fiber implementation and the GC's cost is not in SCOREBOARD.** That is an unmeasured hole worth a row, not a cut |

---

## 9b. The spec side — H17's blocker is mis-stated

Cuts 001–009 are all Rust. The `.ph` seam is priced for the first time by H17, and
010's framing of what H17 unlocks is **a false dilemma**, in the same shape as weaknesses
#1 and #3.

010 §3 and SCOREBOARD H17 both say: *"Landing needs kernel-collection sealing (the
`8d401f4` `Option` precedent) or its own ADR."* Two options. **There is a third, and the
repo already accepted, built, and shipped it.**

### The axis

Not "iteration". Precisely: **the deoptimization mechanism for a speculatively-direct
cursor protocol on a kernel collection whose methods are, by committed policy, open.**

### The hazard, and what Phalcom already committed to

The live hazard is **speculative optimization ⊗ late binding**: H17's `List.iterate`
calls `self.length_` directly, so a user who reopens `List` to override `size` no longer
sees `for` honor it. 010 is right that the green corpus is not evidence of invariance.

But **ADR-0026 is "Methods are open; superclass *reparenting* is sealed"** (Accepted,
`docs/adr/STATUS.md:56`). Methods being open is not an accident awaiting a sealing ADR —
**it is the committed position.** Sealing kernel collections would *reverse* it.

And **ADR-0018 — "Sacred-selector inliner + override-epoch guard" — is Accepted and
built** (`STATUS.md:48`, marked ✅). It exists in the tree, right now:

```
universe/mod.rs:41   pub bool_sacred_pristine: bool,
universe/mod.rs:45   pub block_sacred_pristine: bool,
universe/mod.rs      BOOL_SACRED_SELECTORS  = ["and(_)","or(_)","not()","ifTrue(_)","ifFalse(_)","ifTrue(_,ifFalse)"]
universe/mod.rs      BLOCK_SACRED_SELECTORS = ["whileTrue(_)"]
dispatch.rs:931      // Sacred-selector override-epoch tracking (ADR-0018)
bytecode.rs:237      // the override-epoch half of the deopt guard
```

The design: a monotone `bool`, `true` from bootstrap, flipped the moment a watched
selector is installed on the kernel class — flagged from the `Bytecode::Method` handler,
*"the only place user code can (re)install a method on a class row"* — at which point
`GuardBool` deopts every inlined site back to a real send.

**That is exactly H17's problem, already solved, for a different receiver family.**
Phalcom has already ruled that speculating on kernel-class method identity is acceptable
*provided the speculation is guarded by an override epoch and deopts to the real send*.
H17 is the same ruling applied to `List`.

### The recommendation

**Extend the override epoch — do not seal.** Add `list_sacred_pristine` and
`LIST_SACRED_SELECTORS = ["size()", "at(_)"]`, mirroring the shipped Bool/Block design.

**Deopt without a hot-path guard (PLAUSIBLE, unverified — this is a design sketch, not a
measurement).** H17 differs from ADR-0018 in one way that matters: ADR-0018 guards a
*compile-time-known inlined site* with a `GuardBool` opcode, whereas H17's change is a
`core.ph` method body. So the deopt need not be a per-iteration guard at all — when the
epoch flips, **swap `List.iterate`/`List.iteratorValue`'s installed `Callable` back to
the wrapper version**. A method-dictionary swap at override time costs the hot loop
nothing, adds no opcode arm, and therefore pays no F21 tax whatever H18 concludes about
it. Cost is one-time and paid only by programs that actually reopen `List`. This wants
its own ADR and a check against ADR-0048 (bare-cursor sentinel + `Iterable` root, which
amends ADR-0035) before anyone writes it.

### What it precludes (mandatory)

- **The epoch is coarse and one-way.** Any override of `size`/`at` on `List` deopts
  **all** `List` iteration, **permanently, process-wide** — there is no re-optimization,
  exactly as `bool_sacred_pristine` has no path back to `true`. That is the price, and
  it is the price ADR-0018 already agreed to pay.
- **It precludes per-site tiering.** A monotone `bool` is the wrong shape if Phalcom
  ever wants tiered reopt or per-callsite deopt. Adopting it for `List` deepens a
  commitment to coarse, global, one-way invalidation. That is a real cost and it should
  be paid knowingly.
- **Reflection must stay honest.** If the deopt is a dictionary swap, `List.methods`,
  stack traces, and reflective lookup must not be able to observe which `iterate` is
  installed, or the optimization is surface-visible and becomes a spec change.
- **It does not preclude sealing later.** This is the argument's strongest point: the
  epoch is reversible, sealing is not.

### Precedent, with consequence

- **Ruby** keeps classes open unconditionally, which forces megamorphic call sites and a
  global method cache that must be invalidated on every definition — the openness is
  free to the user and permanently expensive to the implementer.
- **Java** made `final`/sealed plus CHA the basis of devirtualization; HotSpot speculates
  on a monomorphic hierarchy and deopts when a class loads that breaks it. The lesson is
  the *deopt*, not the sealing: the speed comes from speculating and having a way back.
- **Smalltalk's `become:`** is precisely the mutability that forced Self to invent maps —
  when the object model can be reshaped at runtime, the implementation pays for it
  forever unless there is a guard.

Phalcom already picked Java's answer once, in ADR-0018. **Picking it again for `List` is
cheaper and less precluding than picking Ruby's problem's solution (sealing).**

### The generalisation is bigger than the arm — and it is the real prize

010 notes it and then drops it: `Iterable.iterate` calls `self.size` **per element**, and
`size` is a `.ph` wrapper over a native on **every** collection (ADR-0048 makes
`Iterable` the root). **Fixing `List` is one row; fixing `Iterable` is language-wide
iteration.**

The general lever is **native-alias / frame elision**: a `.ph` method whose body is
*exactly* a forward to a native could compile to the native itself, so the frame push
**disappears** rather than getting faster. `for` costs **4 `.ph` frames per element**, two
of them pure forwarding wrappers (F23) — this deletes a whole class of them at once.

- **Axis:** the primitive/library boundary for kernel classes.
- **Hazard: primitive/library boundary ⊗ bootstrap order.** The kernel classes the
  runtime secretly depends on are on the critical path; aliasing during bootstrap is
  where this breaks.
- **Preclusion — answer before proposing, per the skill:** does an aliased method still
  appear in the method dictionary? Still get overridden (epoch again)? Still show in a
  stack trace? Still answer reflection? If any answer is "no", it is a **spec change**
  and must be labelled one, not smuggled in as an optimization.

**Do not start this until H18 reports.** If layout is a ±5% term, the `.ph` seam is the
*only* lever in the log currently measurable above the noise — which raises its priority
further. If layout is ±0.5%, H17's `for` number is trustworthy on its own and sizes the
whole tier. Either way H17 is the arm that decides it, which is why it is the headline.

---

## 10. Instrumentation — what we should record and are not

- **Raw per-rep vectors.** Computed today; discarded today. Zero cost. **Had F22's round
  emitted them, "min improved while median did not" would have been a query, not a
  post-mortem.**
- **RSS** (`os.wait4`). Zero extra processes. SCOREBOARD's own law.
- **Machine state per rep**: `load1`, elapsed, **executing core cluster**, thermal
  pressure. §6 #9 shows `load1` is partly *self-inflicted*; recording it separates
  ambient from own.
- **P/E cluster (G2).** Nothing pins QoS. On a 6P+2E machine an unpinned benchmark that
  lands on an E-core is **a different experiment**, not a slow one — E-core L1i is
  131,072 B vs P-core 196,608 B, and L2 is 4 MB vs 12 MB. This is a candidate root cause
  for F22 *and* for `min`'s behaviour ("the run that got a P-core") in one shot.
  `taskpolicy -c` or a QoS class would remove it.
- **Build provenance per binary**: SHA, profile knobs, **`run_until_inner`'s address,
  size, and alignment mod 64**. This session found h16's 64-byte alignment and prev's
  32 KB relocation with three `nm` calls. **Layout should be a recorded covariate in
  every row of the log**; it currently is not recorded anywhere.
- **Per-row opcode histogram, checked in.** H16's prediction that "`for`/`fib`/
  `string_equals` execute these five opcodes ~never" is an **assertion**. The counting
  build can make it a fact at zero window cost. It is load-bearing for H16's falsifier.
- **CPU counters** (cycles, branch-misses, L1i-misses). Would turn mechanism from
  inference into observation, and would make M-btb vs M-align *directly* separable
  instead of inferable. **Feasibility NOT established this session** — Apple Silicon PMU
  access is restricted; `xctrace record --template 'CPU Counters'` is the candidate path
  and was not tested. **Reported as unverified, not as available.** If it works it
  changes the plan; if not, the wall-clock design stands.

---

## 11. Miri

**The handoff's premise was false: there is no miri lane and miri was not installed.**
`rustup component list --installed` showed no miri. The only `miri` hits in the tree are
skill documentation and agent prompts — no CI job, no script, no `Makefile` target.
Nightly *was* installed.

`rustup +nightly component add miri` succeeded (miri `0.1.0 (375b1431b7 2026-07-10)`),
and this was launched against the `h13b` worktree:

```sh
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test -p phalcom-core --test invariants
```

(`-Zmiri-disable-isolation` is required because the VM reads `core.ph` from disk during
bootstrap.)

**Status: STOPPED at 51 minutes, INCOMPLETE — 8 of 42 tests passed, 0 failures, 0 UB,
0 `Miri caught`, 0 unsupported operations.** Partial log preserved at
`~/phalcom-perf-phaseA/miri-h13b-partial-8of42.log`. Killed deliberately: §11a shows the
remaining 34 tests cannot reach an arity the first 8 did not.

Green under miri (each bootstraps all of `core.ph` through the probe's `unsafe`):

```
behavior_class_exists_in_tower
callable_tower_and_reflection_protocol
class_is_instance_of_class_class_not_metaclass_directly
core_classes_have_correct_metaclass_and_superclass
cross_fiber_non_local_return_raises_dead_frame_error
error_raise_unwinds_through_the_shared_raise_payload
expression_result_absence_surfaces_to_none
floor_census_matches_installed_bindings
```

`cross_fiber_non_local_return_raises_dead_frame_error` is the one worth naming: it drives
the **fiber-switch path through `call_method`** — the case whose own source comment warns
that *"`self.frames`/`self.stack` were just repointed to a different fiber by the
primitive itself"*. That is the nastiest aliasing interaction the probe has, and miri saw
no UB in it.

### 11a. What the lane could and could not ever reach

Measured with a throwaway arity histogram on the primitive fast path (built on **base** —
the distribution is a property of the program, not of the probe; probe reverted, worktree
clean).

**Static bound — decisive.** All **126** primitives are registered in one file,
`universe/primitives.rs`. Declared arities:

| `SignatureKind` | count |
|---|---|
| `Getter` / `Setter` | 41 |
| `Method(0)` | 19 |
| `Method(1)` | 57 |
| `Method(2)` | 10 |
| **`Method(3)`** | **1** — `Range.new`, the only one |

**No variadic primitives** (`callWith(_:)` takes a list, so variadic collapses to
arity 1 — which is why `variadic_send` retires only arity 0 and 1). Therefore **arity at
the primitive fast path ∈ {0,1,2,3} by construction.**

Two consequences:

- **`call_method`'s `else { Vec }` fallback is DEAD CODE.** It is guarded by `arity > 8`
  and the maximum declared arity is 3. It cannot execute. The comment above it —
  *"only a rare wider call falls back to a heap `Vec`"* — describes a case that does not
  exist.
- **`INLINE_ARGS = 8` is 2.7× oversized.** The buffer reserves 8 slots for a language
  whose widest primitive takes 3.

**Empirical bound.** 61 `.ph` files (examples + fixtures + benchmarks),
**121,613,008 primitive sends**:

| arity | calls | |
|---|---|---|
| 0 | 14,601,417 | |
| 1 | 104,851,577 | |
| 2 | 2,160,014 | |
| **3** | **0** | `Range.new(_,_,_)` **is called by nothing in the repo** |
| >8 | 0 | the `Vec` branch, never taken |

Arities 0+1 are **98.2%** of every primitive send in the repo.

**So the 4-hour lane was buying volume, not coverage.** Bootstrap alone is 47 sends at
arity 0 and 1 at arity 1; the 8 completed tests already cover arities 0 and 1, and the
34 remaining ones cannot reach arity 3 (nothing calls it) and cannot reach 4–8 or the
`Vec` branch (they do not exist). At best they add arity 2.

**Recorded as a separate defect, independent of h13b: `Range.new(_,_,_)` is registered
and exercised by no test, benchmark, or example in the tree.**

**Say precisely what this clears and what it does not.** Miri detects UB **on the paths
it executes**. It does **not** detect miscompilation, and it cannot: a green miri run is
evidence about the interpreted execution of those tests, not about what LLVM emitted for
`call_method` at `opt-level=3`. Nor is a green `invariants` lane evidence about every
`arity` — the probe's `unsafe` is arity-parametric and `invariants` does not sweep it.

**None of this gates the decision.** `h13b` is dropped from the batch on §4's grounds —
its premise is refuted — and those grounds are independent of its soundness. The miri
lane is worth finishing anyway, because **the repo should have one** and now can: it took
one `rustup` command.

---

## 12. The dead list — do not re-derive

- **F22's void round (`INLINE_ARGS`).** No result, in either direction.
- **`CALL_0..16` / `LOAD_LOCAL_0..8`.** Killed on **mechanism** (F20/F22), not only
  footprint: `invoke_at`/`call_method` take arity as a **runtime parameter**, so the
  constant cannot reach the buffer build. Re-read F22 before touching it **even if H18
  comes back saying the footprint tax never existed**.
- **Per-fiber eager allocation.** Pool (F10) and presize (F18) both measured negative.
  The lever is the shell's **lifetime**, not buffer size.
- **F3/H9's memmove.** Spent — 20.6% → 3.0%.
- **Tier-0's "19–20× Wren" and "allocation is #1".** Dead narrative.
- **New, from this document:** *the 128 B arg-buffer init does not exist.* It is
  **8 bytes** (§4). Do not re-propose removing it, and fix the claim wherever it appears
  — it is currently stated in `010` §4, in `SCOREBOARD` H13, and **in the h13b probe's
  own source comment**.
