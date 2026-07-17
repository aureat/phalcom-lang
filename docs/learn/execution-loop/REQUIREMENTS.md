# `docs/learn` — the execution loop: requirements, approach, checklist

Working folder. Scratch. The shipped doc is `docs/learn/vm/execution-loop.md`; everything here
is state used to build it. This is **VM Doc 1** — the group entry and the opcode map. Grounded by
[recon.md](recon.md).

## 0. The obligation

One test, and it is the whole spec:

> **After reading, the reader could re-derive Phalcom's execution model from the constraints alone.**

Delete the source. Hand the reader the pressures. Could they rebuild it? For a *mechanism* doc the
re-derive test is sharper than for a fork: the reader must be able to say *why the loop is shaped
this way* — why one stack, why a drain check and not an end-of-code check, why "decode" is free.

Corollary: the one real branch not taken (register machine) must be made **genuinely tempting**
before it is set aside. A strawman register machine teaches nothing.

## 1. Reader

Knows what a bytecode interpreter *is* — has seen "compile to bytecode, run it on a VM" as a phrase.
Not fluent in implementation. Specific stated weakness: **cannot hold moving-state mechanisms in
their head** — lacks a stable notation, so complexity accretes until the thread is lost. The loop is
the worst possible place for that to happen, because everything else in the VM is *one arm* of it.
The doc's job is to hand over the **grip** — "it is just a loop over a match" — and the **map** —
"here is every arm and which later doc owns it" — so the reader has a frame to hang all six docs on.

## 2. Doc kind

**Mechanism (the cycle).** No fork at its core: fetch-decode-execute is not a decision, it is the
shape of every bytecode interpreter. One genuine fork sits to the side — **stack machine vs register
machine** — and earns real design-space depth (recon §3: deliberated in the bytecode-representation
design note). Everything else is machinery to be understood exactly, not chosen.

**Stateful?** Yes, but the state is *shallow per turn* (`ip`, the stack, the frame count). A trace
earns its place only for the **predict-then-check** beat (§5.4): trace `Constant`/`Dup`/`Pop` stack
deltas from real disassembler output. Do not trace a method call here — that is Doc 3/4's job.

## 3. The grip — grounded (recon §2)

> **"Running a program" is one `while` loop matching a flat array of typed opcodes against a stack.**
> Fetch the opcode at `ip`, `match` on it, push/pop `self.stack`, advance, repeat — until the *frame
> stack* drains to its floor. `run_until_inner` (`dispatch.rs:477`). Nothing magic runs underneath:
> a method call, a closure, garbage collection — each is *one arm* of that `match`, and the loop is
> the whole engine.

Two grounded sharpenings the doc must land, both from recon §1:

1. **The halt is a drain, not an end.** The loop never asks "did `ip` reach the end of the code?" It
   asks "did the frame stack shrink to `base_frames`?" (`dispatch.rs:491`). A program halts because
   its last frame *returned*, not because `ip` fell off the array. This is why the same loop drives
   the whole program (`base=0`) and a single re-entrant block activation (`base≠0`) with no change.

2. **"Decode" is free.** Because the code array is `Vec<Bytecode>` (a typed enum), not a byte
   stream, one indexed load pulls the whole instruction into registers and `match` switches on the
   discriminant. There is no width-parse, no operand-decode step. This is the representation fact
   that the word "bytecode" actively misleads on — see §3a.

### 3a. The representation trap (this doc's [[x-style-is-architecture-not-representation]] beat)

Phalcom is a **stack machine** (architecture) whose **bytecode is a name-typed enum vector, not
bytes** (representation). The reader will hear "bytecode VM" and picture a `Vec<u8>` with a decode
loop. That picture is wrong here and the doc must correct it explicitly, early. The design note
`bytecode-representation-and-borrowed-techniques.md` §B1 is the anchor: "`Vec<Bytecode>` is not a
bytestream, and the difference eats a class of techniques."

## 4. The design space (one real fork; walk it, don't survey)

The mechanism is not chosen; the *machine model under it* is. The problem: a compiled program is a
linear list of primitive operations plus a way to pass values between them. Where do the operands
live between operations?

| Branch | Occupants | The bill |
|---|---|---|
| **Stack machine** — operands on an implicit operand stack | JVM, CPython, Wren, Smalltalk-80, **Phalcom** | Tiny, dense instructions (`Add` has no operands). More instructions per expression ⇒ more dispatches ⇒ more dispatch overhead. Trivial compiler. |
| **Register machine** — operands named by virtual-register index | Lua 5.0+, Dalvik, LuaJIT | Fewer, fatter instructions; fewer dispatches. Harder compiler (register allocation). Lua *switched* stack→register at 5.0 for exactly the dispatch win — the scar to cite. |
| Tree-walker (no bytecode) | Ruby pre-1.9, early interpreters | No compile step; slowest. The pole Phalcom left when it chose bytecode at all. One sentence. |

Second axis, **representation of dispatch** (recon §1, the vocabulary the reader lacks):

| Technique | What it is | Bill |
|---|---|---|
| `switch`/`match` on opcode | one branch per fetch | portable, simplest; branch-predictor pressure. **Phalcom.** |
| Direct/token threading | each handler jumps to the next via a table | fewer branch mispredicts; needs `goto`-into-table, non-portable C |
| Computed goto (labels-as-values) | threading via `&&label` | fastest classic technique; a GCC/Clang extension; **precondition: a bytestream you index by width** — which `Vec<Bytecode>` is *not*, so it is foreclosed here (design note §B1) |

Agent A walks these blind to Phalcom's pick. Agent B reports the pick with the line.

## 5. Comparison filter

A language/VM enters **only** if it does one of these. Otherwise cut. Expect ~5 to survive.

1. **Took the other branch, with the bill.** (Lua's stack→register switch.)
2. **Has a scar** — shipped change, perf loss. (Lua 5.0; CPython's computed-goto opt-in.)
3. **Names something Phalcom does anonymously.** ← highest value. (threading, safepoint, the drain.)
4. **Ancestor** — explains otherwise-arbitrary shape. (Smalltalk-80: the loop *is* `bytecodeAt:`.)

Vocabulary to import (a deliverable in itself): *fetch-decode-execute*; *operand stack*; *dispatch*;
*direct/token threading*; *computed goto / labels-as-values*; *superinstruction* (forward-ref to
Doc 5); *safepoint*; *drain / frame floor*; *tree-walker vs bytecode*.

Provisional cast, subject to earning it:

- **Lua** — the register-machine branch, *with the 5.0 switch as the scar*. **Deep.**
- **CPython** — the canonical `switch`-based stack VM; `USE_COMPUTED_GOTOS`, opt-in threading as a
  measured win. Names Phalcom's dispatch technique and its rejected alternative. **Deep.**
- **Wren** — closest lineage; stack VM Phalcom's shape most resembles; the direct source of several
  choices. **Medium.**
- **Smalltalk-80** — the ancestor where the loop *is* the object model (`Interpreter»interpret`,
  `bytecodeAt:`). Explains the Phalcom loop's send-centricity. **Medium.**
- **JVM** — the stack machine everyone has met; `wide` prefix as what a real bytestream costs.
  One sharp paragraph on why enum-vector Phalcom never needs `wide`. **Short.**
- **JavaScript V8 / tracing JITs** — the thing Phalcom is *not* (pure interpreter at HEAD).
  Named and cut with the reason. **Cut list.**

## 6. Tensions to surface

- **One stack ⊗ locals** — locals are not a separate array; they are a window into `self.stack` by
  offset. Named here, *owned by Doc 3*. One forward-pointing sentence only.
- **The loop ⊗ GC** — `service_gc_safepoint` (`:505`) is the *only* place collection runs, on the
  back-edge where `stack`/`frames` are coherent. Why *here* is a real beat (recon §1); depth on the
  collector itself is the GC doc's.
- **The loop ⊗ fibers** — `run_until_inner` is fiber-unaware; `run_until` wraps it. The hoist guard
  keys on `closure_id` not `ip` because a fiber switch swaps `self.frames` wholesale. **One honest
  paragraph, zero depth, forward-pointed** (recon §4).
- **The loop ⊗ the hoisted `Callable`** — the per-instruction `Rc` hoist (cut 004/F14). A *marked
  lie* here (the loop "just reads the chunk"); owned by Doc 2. Do not explain the `Rc` mechanics.

## 7. Structural rules (constraints, not a skeleton)

- **Structure follows the mechanism.** No imposed heading set. Grip → the cycle → the map → the one
  fork → the coherence points (safepoint, drain) → forward pointers. Bottoms out at the opcode tour.
- **The opcode tour is a map, not a catalogue.** Every arm named once, each tagged with the doc that
  owns its detail. It is the reader's index into the whole six-doc course. This is a load-bearing
  deliverable, not filler.
- **One trace, for the predict-then-check only** (`Constant`/`Dup`/`Pop`), from real disassembler
  output. No method-call trace.
- **Mermaid only where the shape is the point** — the fetch-decode-execute cycle as a labelled ring,
  and/or the drain check. Not a pointer diagram (there are no pointers here to draw).
- **Source anchors: symbol first, line second** (`dispatch.rs::VM::run_until_inner` @ ~L477). Bare
  line numbers rot.
- **HEAD as-implemented.** Fibers/GC/caches cited as intent-of-later-docs, not re-derived.
- **Mark every simplification as a lie with a forward pointer.** The four in recon §4.

## 8. Checklist (gate before shipping)

- [ ] Grip stated early, one sentence, *earned* by the end (loop → every arm is one branch).
- [ ] "Bytecode is a typed enum vector, not a byte stream" stated explicitly, early (§3a).
- [ ] The halt-is-a-drain point made, and shown to unify `base=0` and `base≠0` runs.
- [ ] Register-machine branch made tempting (Lua 5.0 switch) before set aside.
- [ ] Dispatch-technique vocabulary imported (threading, computed goto) and computed-goto's
      foreclosure explained via the enum-vector precondition.
- [ ] Opcode tour complete — every `Bytecode::` arm named, each tagged to its owning doc.
- [ ] `service_gc_safepoint` placement explained (why the back-edge, why the *only* point).
- [ ] Entry chain traced: `interpret_source` → `compile_closure` → `run_in_module` → `run` →
      `run_until` → `run_until_inner`.
- [ ] ≥1 predict-then-check moment (stack deltas, or the "ip hits end but program runs" puzzle).
- [ ] Fiber touch is one honest paragraph, forward-pointed, zero depth.
- [ ] Four marked lies present with forward pointers (chunk detail→Doc 2, call→Doc 3, hoist→Doc 2,
      fiber wrapper→fibers doc).
- [ ] Every language present passes the §5 filter; named cut list (V8/JITs).
- [ ] Anchors symbol-first and exist at HEAD.
- [ ] Reader could re-derive the model. (§0)

## 9. Build sequence

| # | Deliverable | Who | Path |
|---|---|---|---|
| 1 | recon | me | `recon.md` |
| 2 | This file | me | `REQUIREMENTS.md` |
| 3 | Theory draft — no source access | sonnet A | `draft-concept.md` |
| 4 | Source map — graphify-led, runs `.ph` live | sonnet B | `source-map.md` |
| 5 | The doc — my judgment over A's bulk + B's ground truth | me | `../vm/execution-loop.md` |

**Division of labour.** A supplies the history and the design-space bill (Lua's switch, CPython's
threading, Smalltalk's loop) I would otherwise burn context re-deriving, blind to Phalcom's answer so
it cannot flatter it. B supplies ground truth — the enum-vector representation, the drain line, the
live disassembler output — that I must not guess. I reconcile: where A's stack-machine theory meets
Phalcom's enum-vector representation is exactly where the doc teaches (§5.1).

## 10. Open risk

Recon settled the representation question (enum vector, `match` dispatch — the error that hit the
upvalue doc cannot repeat here; the type was read first). Residual risks for B to close:

- **Is the `match` truly the whole dispatch**, or is there a threaded/fused fast path in the hot loop
  that changes the "just a match" grip? (Recon saw fusion — `InvokeLocal`/`InvokeConst` — but that is
  Doc 5. B must confirm the *base* loop is a plain `match` so the grip holds, and that fusion is a
  peephole over it, not a different engine.) If B reports a threaded core, §3's grip is wrong.
- **Does `base_frames` really unify the two run modes**, or does `run` do more than `run_until(0)`?
  B confirms the drain check is the sole halt condition of the inner loop.
- **Is `service_gc_safepoint` genuinely the only collection point?** The "one coherent point" claim
  rests on it. B confirms no `alloc`-triggered collection (memory-management.md Invariant L).

**Outcome: settled. All three residual risks closed by B, grip held.** No representation
surprise (enum vector, plain `match` — recon read the type first, so the upvalue-doc error did
not recur). The base loop is a plain `match`; fusion is a peephole over it (Doc 5), not a
different engine. `run()` = `run_until(0)` literally; the drain is the sole inner-loop halt.
`service_gc_safepoint` is the sole automatic collection site (grep-verified two hits);
`Heap::insert` latches only (Invariant L).

## 11. Reconciliation record (§5.1) — A's blind theory vs Phalcom's ground truth (B)

The centerpiece is row 2: A, with no source access, wrote the general principle that computed
goto "buys nothing for a materialized variant-type instruction array" as a *hypothetical*. B
found Phalcom **is** exactly that array. The doc poses A's hypothetical as a puzzle and reveals
it is Phalcom — the reader learns the principle and its instance at once (the predict-then-check).

| A's claim (pure theory) | Phalcom's reality (B, with anchor) | How the doc teaches it |
|---|---|---|
| dispatch is an open space: switch / threading / computed-goto | plain Rust `match`, one indirect branch (`dispatch.rs:570`); threading/computed-goto *rejected by precondition* (design note §B1/§B3) | present the spectrum, then foreclose the fast end |
| computed goto "buys nothing" for a variant-type array — hypothetical | Phalcom's `code` **is** `Vec<Bytecode>` (`chunk.rs:45`); `code[ip]` pulls the whole instruction (`:544`) | the puzzle + reveal; "discovered, not designed" (findings note) |
| decode is free *via threading* | decode is free *via representation* — the enum discriminant already decoded it | teach the different route to the same "free decode" |
| halt: model (a) end-of-array vs (b) frame-drain — both real | Phalcom is (b): `frames.len() <= base_frames` (`:491`); no end-of-code check exists (whole loop read) | (b) is what unifies `base=0` and `base≠0`; `import_module` shown live |
| stack vs register — open fork | stack machine, confirmed by disasm of `1 + 2 * 3` → `Constant`/`Constant`/send | honesty pass: no bake-off; lineage, not deliberation |
| Smalltalk is "send-centric," `3 + 4` is a send | Phalcom's `1 + 2 * 3` compiles to `Invoke` — arithmetic **is** a send, seen live | the A+B join: the ancestor's shape, made concrete in Phalcom's output |
| GC safepoint at the back-edge is "the one coherent point" | exactly Phalcom: sole `service_gc_safepoint` call (`:505`), `Heap::insert` latches only (Invariant L) | stated as "half a GC design," verified |

**Honesty corrections applied (§5.2):** the stack-vs-register walk is labelled *pedagogical
reconstruction* (no ADR, no bake-off — lineage choice); `match`-not-threading is labelled *fit to
the representation*, and the computed-goto foreclosure *discovered in a findings note*, not a
principled rejection; `Loop`-vs-`Jump` noted as a disassembly-readability distinction, not perf.
**Claims ledger (§5.3):** Lua 5.0 register switch (2003, "first in wide use") cited to *The
Implementation of Lua 5.0*; CPython computed-goto (build option 3.1, default 3.2) and wordcode
(2-byte since 3.6) web-verified; every Phalcom claim carries a source anchor verified at HEAD; no
invented numbers; every link resolves. **Weight (§5.5):** cut A's subroutine-threading deep dive,
the freestanding Wren/CLR/YARV specimens, the full JVM `wide` treatment, and the standalone
one-stack-or-two section (folded to a forward pointer) — ~30%+ trimmed toward Phalcom's choice.
