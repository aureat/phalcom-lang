# REQUIREMENTS — `docs/learn/vm/compiled-artifact.md` (VM track, Doc 2)

Working spec for the doc. Derived from `recon.md`. Agent A is judged against §§3–8; Agent B
against §9. Synthesis (§10 gate) is mine.

## 1. The obligation

> After reading, the reader can re-derive **why a Phalcom compiled function is three layered
> objects and not one** — from the pressures (share immutable code across many live closures;
> capture different cells per materialization; carry the home-frame identity a non-local return
> needs) alone. Delete the types; hand the reader those three pressures; they rebuild the split.

## 2. Reader & kind

- **Reader:** knows PL design; has met "closure" and "bytecode"; has *not* internalized that a
  closure implementation routinely splits a code template from its per-instance capture. Cannot
  hold moving state; needs a stable vocabulary (recipe / instantiation / stamp).
- **Kind:** **mechanism/vocabulary** with one live fork inside it (*one object or three?*). No
  Overview/Conclusion skeleton. Structure = the layer stack, built bottom-up (Chunk → Callable →
  Closure → Block), with `MethodKind` as the cap that answers "where does native live."
- **Grip (from recon §2):** *recipe / instantiation / stamp — shared, minted, identified.*

## 3. The design space (walk as a space; Agent A owns this, blind to the answer)

The one fork, plus two sub-forks. Make each branch tempting before rejecting.

| Question | Branch A | Branch B | Who sits where |
|---|---|---|---|
| One object or several? | Fuse code + captures into one "closure" object | Split shared code template from per-instance closure | Naive/pedagogical vs Lua (`Proto`/`Closure`), CPython (`code`/`function`) |
| Literals: where? | Bake immediates into the instruction stream | Side **constant pool**, opcodes carry indices | asm/byte-immediates vs JVM/CPython/Lua constant arrays |
| Native code: how represented? | A "native" variant *of the code object* | A **sibling** to the whole bytecode stack (fn ptr beside a closure handle) | (do not reveal Phalcom's pick — this is a predict-then-check) |

Sub-question the doc must answer but that is **not** a live fork (state it as reconstruction): given
you split, *how many* layers and which axis each owns — code vs recipe-metadata vs
instance-capture vs home-identity.

## 4. Comparison filter (≤ ~5 survive; name the cut list)

A language enters only on one of the four tests (bill / scar / names-something-Phalcom-does-
anonymously / ancestor). Candidates:

- **Lua** — `Proto` (shared prototype: code + constants + upvalue descriptors) vs `Closure`/
  `LClosure` (proto + upvalue cells). This is *exactly* Phalcom's Callable/ClosureObject line and
  **names it** — highest value. Keep.
- **CPython** — `code object` (`co_code`, `co_consts`) vs `function object` (code + `__closure__`
  cells + `__globals__`). Names the constant pool (`co_consts`) and the module link. Keep.
- **JVM** — constant pool as an ancestor/vocabulary source (the *term* "constant pool" is JVM's);
  no per-instance closure object (pre-lambda) — the contrast that shows why Phalcom *needs* a second
  layer where Java did not. Keep, but tightly.
- **Smalltalk** — `CompiledMethod` (bytecode + literal frame) and block context/home identity — the
  ancestor of `home_frame_token` and the `Method`/`Block` sibling framing (ADR-0006). Keep for the
  home-frame stamp only; one paragraph.
- **Cut:** JS engine hidden-class/shape machinery (different problem), Wren (same lineage but adds
  no *name* Lua doesn't already give), any JIT tier (out of scope, like Doc 1). Name them cut.

## 5. Tensions to surface

1. **Sharing vs capture.** The recipe must be shared (many closures, one code body) *and* each
   closure must capture its own cells. Resolved by putting the `Rc` on the recipe and the `Vec<cell
   handle>` on the instance. This is the spine of the doc.
2. **Immutability vs the mutable side tables.** The `Callable`/`Chunk` are conceptually immutable
   ("frozen after compile"), yet `caches`/`gcaches` are `Cell<Option<..>>` and mutate at runtime.
   Surface the tension; resolve it as "the *code* is frozen; the caches are interior-mutable
   memo slots on a shared borrow" — and forward the *why* to Doc 5. (Lie A.)
3. **Where native lives.** A method can be bytecode or a raw Rust fn, but the *bytecode* layers know
   nothing of native. The tension resolves one layer up in `MethodKind`. (Predict-then-check.)

## 6. Structural rules

- Build bottom-up; each layer introduced only when the previous one's limitation forces it
  ("a Chunk can't be shared safely and also carry per-call capture — so…").
- Every simplification marked as a **lie** with a forward pointer (recon §8: A→Doc 5, B→Doc 5,
  C→Doc 6; D pays off Doc 1's Lie #1).
- Anchors symbol-first: `callable.rs::Callable` (~L21), `heap/closure.rs::ClosureObject` (~L24),
  `heap/block.rs::BlockObject` (~L18), `method/object.rs::MethodKind` (~L17), `chunk.rs::Chunk`
  (~L44).
- At least one **predict-then-check** (recon §6.1, the loop-materialization question), traced from
  a **real disasm** of a block-in-loop program (Agent B produces the disasm).
- A **diagram earns its place** only if it draws the layer split (the thing whose *shape is the
  point*): recipe shared by an `Rc` arrow, two instances hanging off it with distinct cell vectors.
  Do not draw bytecode boxes.

## 7. Must-cover checklist (content)

- [ ] `Chunk`: `code`, `constants` (the pool), `spans`; `caches`/`gcaches` named and **deferred to
      Doc 5** (Lie A).
- [ ] The constant pool: what an opcode index buys; which opcodes read it (`Constant`, and the
      selector/name indices on `Invoke`/`GetGlobal`/`Class`/`Method`). Dedup-or-not stated from B.
- [ ] `Callable`: chunk-by-value + `max_slots`/`arity`/`name_sym` + `num_upvalues`/`upvalues`
      (descriptors). Purely bytecode; **no native variant** (correct the plan).
- [ ] Why the recipe is a shared `Rc` (`ClosureObject.callable: Rc<Callable>`) — cite perf/U-HOTPATH
      as the driver (honesty: an *optimization*, not an object-model axiom). **Pays off Doc 1 Lie #1.**
- [ ] `ClosureObject`: recipe + module handle + filled upvalue cells. The `Bytecode::Closure`
      opcode that materializes template→instance (real site from B).
- [ ] `BlockObject`: closure handle + `home_frame_token`; block-literal opcode that stamps it.
      Bridge to Doc 6 (Lie C).
- [ ] `MethodKind::{Closure, Primitive}` as the cap: native lives here, sibling to the whole stack.
- [ ] Upvalue tie: descriptors-on-recipe vs cells-on-instance (one paragraph; defer mechanics to the
      upvalue doc). Fiber touch: `Upvalue::Open{fiber, slot}` (one sentence).
- [ ] GC touch: closure/block are traced heap objects; constants are GC roots reachable through the
      shared recipe (one paragraph).
- [ ] "What you can now re-derive" close + symbol-first Anchors section.

## 8. What Agent A must NOT be told

Phalcom's picks: that the share is by `Rc` on the `Callable`, that native is a separate `MethodKind`
enum (not a Callable variant), the layer count, and the U-HOTPATH provenance. A gets the *space*.

## 9. Agent B — the questions source must settle (headline first)

Headline (answer first, with the line): **Is a Phalcom "compiled function" one object or a layered
stack — and if layered, what does each layer hold and which is shared how?** State the candidate
answers (one fused object / two layers / three / four) so B cannot pattern-match.

Then, each with `file:line` + quoted def, and live output where behavioural:
- The four/five type defs (`Chunk`, `Callable`, `UpvalueDescriptor`, `ClosureObject`, `BlockObject`,
  `MethodKind`) in full.
- The load-bearing representation lines: `Rc<Callable>` (`closure.rs:28`), `chunk: Chunk` by value
  (`callable.rs:23`).
- Native/bytecode fork is `MethodKind`, not `Callable` — confirm `Callable` has no native variant.
- Constant-pool read opcodes: enumerate the arms that index `chunk.constants` (+ selector/name
  indices). Lines.
- `Bytecode::Closure` materialization site (template→live instance, fills cells). Quote dispatch arm.
- `BlockObject` mint/stamp site (block-literal opcode + `home_frame_token`).
- **Constant dedup at HEAD?** `add_constant` — resolve 5050 (no dedup) vs 5964 (ConstKey specced).
  Disasm a program with a repeated literal.
- Perf provenance of the `Rc<Callable>` share (U-HOTPATH / perf-log) — a citation the honesty pass
  can quote.
- Live disasm of a **block-literal-inside-a-loop** program (for the predict-then-check trace).
- Bounded ADR: 0006 (Function root / Block·Method siblings) + 0013 (frame-token return) Decisions
  only.

## 10. Open risks (name each; state the failure if wrong)

- **R1 — dedup direction.** If `add_constant` *does* dedup at HEAD (5964 shipped), the "pool stores
  duplicate literals" line is false. Failure: a smuggled wrong claim in a grounded doc. → B settles
  by disasm; do not assert either way until then.
- **R2 — is `home_frame_token` on `BlockObject` only, or also on plain method closures?** If methods
  also carry a token, the "stamp is the block's distinguishing feature" framing weakens. Failure:
  overstated Block/Closure distinction. → B confirms `BlockObject` is the sole carrier at HEAD.
- **R3 — `MethodKind::Closure(ObjRef)` points at a `ClosureObject` specifically** (not some other
  wrapper). If the handle is to a different type, the "native is a sibling of *this* stack" claim
  mis-wires. → B confirms the handle target.
- **R4 — Rc-share provenance.** If no perf-log/U-HOTPATH citation exists, the `Rc` share must be
  labelled "reason unverified," not "for the hot path." → B settles; honesty pass §5.2 gates it.
- **R5 — layer-count reconstruction.** The "why exactly these layers" walk is pedagogical, not an
  ADR decision (recon §3). Must be labelled reconstruction, or it flatters the codebase. → §5.5.

## 11. Reconciliation record (§5.1)

### A's blind theory → Phalcom reality (from B, with the line)

| A's theory claim | Phalcom reality at HEAD |
|---|---|
| "the box wants to become two boxes" (template + instance) | **Four** types, three object boundaries: `Chunk` ⊂ `Callable` (Rc-shared recipe) ← `ClosureObject` (instance) ← `BlockObject` (stamp). `callable.rs:21`, `closure.rs:24`, `block.rs:18` |
| template ref is "a pointer, handle, or Rc" | specifically `Rc<Callable>` (`closure.rs:28`) — a **measured perf cut** (perf-log 004), not a neutral choice |
| a "MakeClosure-style instruction" mints the closure | `Bytecode::Closure(idx)` (`dispatch.rs:577`) — and it does **double duty**: materializes the `ClosureObject` *and* wraps it in a `BlockObject` in one arm. No separate block opcode exists |
| identity/home-frame is "a small number of languages," a bridge | Phalcom **took it**: `BlockObject.home_frame_token` (`block.rs:22`), ADR-0013 |
| module link "typically per-instance (CPython `__globals__`)" | matches: `ClosureObject.module: ObjRef` (`closure.rs:31`), per-instance |
| constant pool "MAY dedup — compiler policy"; Lua/CPython dedup within a code object | Phalcom does **not** dedup (`chunk.rs:85` unconditional push; verified: two `"hello"` → two `ObjRef`s, `1+1` → two slots). Diverges from both |
| fork (c) native: variant-of-code-object (Ruby MRI) vs sibling (CPython) | Phalcom = **sibling**: `MethodKind::{Closure(ObjRef), Primitive(fn)}` (`method/object.rs:17`); `Callable` has no native variant |
| capture descriptors `Local`/`Upvalue` live on the template | `UpvalueDescriptor{is_local, index}` on `Callable` (`callable.rs:10`); the filled cells `Vec<ObjRef>` on `ClosureObject` |
| immutability vs mutable memo: "a side table of interior-mutable memo cells" | exactly: `caches`/`gcaches: Vec<Cell<Option<..>>>` on `Chunk` (`chunk.rs:50,55`) — the *why* is Doc 5 |

### Honesty corrections (§5.2)

- **Rc share = optimization, not principle.** perf-log/004: before, `Closure` deep-copied the whole
  `ClosureObject`/chunk per block eval (1.1M copies in Skynet). `Rc::clone` → Skynet user −30%,
  RSS −63%, at a 5-7% regression on non-block sends (extra pointer hop). Cite the file, do not
  assert "designed shared."
- **No-dedup is an absence, not a choice.** A `ConstKey` dedup was *specced* (U-COMPILE) but did not
  land. State HEAD behaviour; do not dress the absence as a rationale.
- **Layer count is reconstruction.** ADR-0006 frames Block/Method as siblings (deliberated);
  ADR-0013 the frame token (deliberated, with rejected alternatives). But *Chunk-by-value inside
  Callable and the Rc on Callable* is how the types fell out + a perf cut — not an ADR. Label the
  "why exactly these layers" walk pedagogical.

### Claims ledger (§5.3)

- Rc numbers (−30%/−63%/5-7%) → perf-log/004-hotpath-rc-callable.md (B, verified). Cite.
- no constant dedup → `chunk.rs:85` + B's disasm. Cite.
- Lua `Proto`/`Closure`, CPython `co_consts`/`PyFunctionObject` vs `PyCFunctionObject`, JVM = origin
  of "constant pool" → keep (earn a slot); Lua 5.0 paper already cited in Doc 1. Ruby MRI variant
  (A flagged tag names) → keep as one lineage's *shape* only, no tag names. Smalltalk home-context
  (A flagged class names) → mechanism only, forward to Doc 6, no exact class names. FUNARG → one
  sentence, upvalue doc owns it.
- Forward links to Doc 5/6 (not yet written) → plain text, no bare-file links.
