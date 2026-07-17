# Recon — `execution-loop.md` (VM Doc 1)

Scout, not survey. Arms the two briefs and arms synthesis. Grounded at HEAD.

Entry symbols (graphify): `VM::run_until_inner` @ `phalcom-core/src/vm/dispatch.rs:477`
(the loop), `VM::run` @ `:204`, `VM::run_until` @ `:221` (the fiber wrapper),
`VM::invoke_at` @ `:398`, `Bytecode` enum @ `bytecode.rs:48`, `Chunk` @ `chunk.rs:44`.

---

## 1. Architecture vs representation (the read that prevents the error)

**Architecture (the shape).** A **stack machine**. Concretely:
- One flat instruction array per activation, walked by an index `ip`.
- One operand stack, `self.stack` (a `Vec<Value>`), that every arm pushes to / pops from.
- A `while`-style `loop` (`run_until_inner`, `dispatch.rs:477`) doing fetch → decode → execute,
  one opcode per turn.
- Halt condition is a **drain check**, not an end-of-array check:
  `if self.frames.len() <= base_frames { return … }` (`dispatch.rs:491`). The loop stops when the
  *frame stack* shrinks to a floor, not when `ip` runs off the code.

**Representation (where the consequences live).** The instruction array is **`Vec<Bytecode>` — a
typed-enum vector, not a byte stream.** `Bytecode` is a Rust enum (`bytecode.rs:48`), variants like
`GetLocal(u16)`, `Invoke(u8, u16)`, `SuperSend(u8, u16, u16)`. `Chunk.code: Vec<Bytecode>`. One
indexed load `chunk.code[ip]` pulls the **whole** instruction — discriminant *and* operands — into
registers, sized to the widest variant (~8 bytes, `SuperSend`). `match opcode` switches on the
discriminant; the operand is already in a register from the same load. There is **no byte decoding
step**: "decode" is a Rust `match`, not a width-parse.

> This is the [[x-style-is-architecture-not-representation]] trap in the flesh: Phalcom is a
> "bytecode VM" architecturally, but its "bytecode" is a name-typed enum vector, not bytes. Do not
> let the reader infer a bytestream from the word "bytecode." The design note
> `docs/design-notes/bytecode-representation-and-borrowed-techniques.md` §B1 is explicit: "`Vec<Bytecode>`
> is not a bytestream, and the difference eats a class of techniques" (operand-free superinstructions,
> computed-goto threading — the precondition doesn't hold here).

**Dispatch technique:** a plain Rust `match` on the enum discriminant. Not a computed-goto /
direct-threaded interpreter. (Agent A must present switch vs threading vs computed-goto as the
representation axis without being told Phalcom picks the simplest one.)

Cite: `bytecode.rs:48` (enum), `chunk.rs` (`code: Vec<Bytecode>`), `dispatch.rs:544` (`let opcode =
callable.chunk.code[ip];`), `dispatch.rs:570` (`match opcode`).

---

## 2. The grip, grounded

> **"Running a program" is one `while` loop matching a flat array of typed opcodes against a stack —
> fetch the opcode at `ip`, `match` it, push/pop `self.stack`, repeat until the frame stack drains.
> Nothing magic runs underneath it.**

Earned by: `run_until_inner:477` (the loop), the drain check `:491`, the single `self.stack`, the
`match opcode` at `:570`. Every "advanced" thing (a method call, a closure, GC) is *one arm* of that
match — the doc's opcode tour names each and points at the doc that owns its detail.

---

## 3. What was actually deliberated (vs pedagogical reconstruction)

There is **no dedicated ADR for "the execution loop."** It is foundational machinery, not a fork with
occupied branches in an ADR. Two things *were* genuinely deliberated, both to be handled honestly:

1. **Stack- vs register-machine** — deliberated in the design note
   `bytecode-representation-and-borrowed-techniques.md` §B1 (not an ADR). Phalcom is a stack machine;
   the note's framing is about which borrowed techniques the `Vec<Bytecode>` representation forecloses.
   This is the doc's **one real fork** and earns the design-space depth.
2. **The hoisted `Callable` `Rc`** (perf cut 004 / F14 S1a) and **safepoint placement** (the loop
   back-edge as the *only* GC point, memory-management.md §4, "Invariant L") — real perf decisions,
   governed by ADR-0051 (measure-first). But these are **Doc 2 / GC-doc detail**. In Doc 1 they are
   *marked lies* with forward pointers, not design-space branches.

Honesty note for synthesis (§5.2): the loop is a **mechanism**, not a fork. The stack-vs-register
walk is a genuine deliberation; everything else ("why one stack", "why a drain check") is the
mechanism explained, and the doc should not dress plain machinery as if it were a decision someone
agonized over. The register-machine branch must still be made tempting (Lua/Dalvik took it) before
it is set aside.

---

## 4. Brief-steering notes

**Doc kind:** mechanism (the cycle). The one fork worth full design-space depth = stack vs register.

**Agent A (theory) — emphasis:**
- Go deep: the fetch-decode-execute cycle from first principles; stack-machine vs register-machine as
  a real fork (who took each, the bill — code density vs instruction count vs dispatch overhead;
  Lua 5's register VM and *why* Lua switched from stack in 5.0; JVM/CPython/Wren as stack machines).
- Go deep: the **dispatch-technique** axis as representation — `switch`/`match`, direct threading,
  token threading, computed goto (labels-as-values), subroutine threading — with the bill each pays
  (branch prediction, portability, the `Vec<Bytecode>`-vs-bytestream precondition). This is where the
  reader gains vocabulary. **Do not tell A that Phalcom uses a plain `match`.**
- One sentence, no more: JIT / tracing / superoptimization (out of scope — Phalcom is a pure
  interpreter at HEAD), and the operand-encoding minutiae of real bytestreams.
- The halt condition: how interpreters know when to stop (end-of-code vs a return-driven frame drain).
  Keep honest — present both; do not reveal Phalcom drains on frame count.

**Agent B (source map) — must-confirm symbols (headline question first):**
- **HEADLINE:** Is the opcode stream a byte array or a typed-enum vector, and is dispatch a
  `switch`/`match`, a threaded loop, or computed-goto? Answer first, with the line.
- Confirm + quote: `run_until_inner` @ `dispatch.rs:477`; the drain check `frames.len() <= base_frames`
  @ `:491`; the single `self.stack`; `service_gc_safepoint` @ `:505` (and its body in `gc.rs:152`) as
  the *sole* GC point and *why here*; the hoist guard keyed on `closure_id` **not** `ip` @ `:518-536`;
  `Bytecode` enum @ `bytecode.rs:48`; `Chunk.code: Vec<Bytecode>`.
- Confirm the entry chain: `interpret_source:186` → `compile_closure:142` → `run_in_module:163`
  (pushes frame 0) → `run:204` → `run_until:221` → `run_until_inner:477`.
- Confirm the fiber wrapper: `run_until:221` wraps `run_until_inner` with fiber-floor capture; the
  hoist guard keys on `closure_id` because a fiber switch swaps `self.frames` wholesale (`:521-529`).
- **Run it live:** compile a tiny `.ph` program and dump its bytecode with the disassembler
  (`bin/phalcom/disasm.rs`, or the CLI) to *show* the flat typed-opcode array; trace `Constant` /
  `Pop` / `Dup` stack deltas from real output. Give the exact command.
- Enumerate every opcode arm (`grep "Bytecode::" dispatch.rs`) as the tour, each tagged with the doc
  that owns its detail (send→Doc 4, frame push/pop→Doc 3, unwind→Doc 6, upvalue ops→upvalues doc).

**Marked lies Doc 1 tells (each needs a forward pointer):**
- "the opcode array + a constants table" is the whole story → **Doc 2** (Chunk also holds `spans`,
  `caches`, `gcaches`; Callable/Closure split).
- "a call just appears / an arm runs a method" → **Doc 3** (a send pushes a resumable frame).
- the hoisted `Callable` / `Rc` and why the guard is on `closure_id` → **Doc 2** (+ one honest fiber
  para forward-pointing the fibers doc).
- the fiber wrapper around the loop → **fibers doc**.

**Fiber touch (one paragraph, zero depth):** the loop documented is `run_until_inner`; the outer
`run_until:221` wraps it with fiber-floor capture. The hoist guard keys on `closure_id` **not** `ip`
precisely because a fiber switch swaps `self.frames` wholesale — one sentence, deferred to the fibers
doc.

**Predict-then-check candidate (§5.4):** give the reader a 3-line `.ph` snippet's opcode listing and
have them predict the stack after `Constant`, `Dup`, `Pop` — then show the traced deltas. Second
candidate: "the code array has 40 opcodes; `ip` reaches the end at opcode 12 and the program keeps
running. How?" (answer: the drain check is on frames, and opcode 12 was a `Jump`/`Return`, not the
end — the loop never checks for end-of-array).
