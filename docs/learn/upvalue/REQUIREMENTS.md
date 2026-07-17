# `docs/learn` — upvalues: requirements, approach, checklist

Working folder. Scratch. The shipped doc is `docs/learn/<part>/upvalues.md`; everything
here is state used to build it.

## 0. The obligation

One test, and it is the whole spec:

> **After reading, the reader could re-derive Phalcom's choice from the constraints alone.**

Delete the source. Hand the reader the pressures. Could they rebuild it? If the doc only
describes what `closure.rs` does, it fails, no matter how accurate.

Corollary: every branch not taken must be made **genuinely tempting** before it is rejected.
A strawman teaches Phalcom's answer without teaching the question, which is the failure mode
that produces a reader who can recite and not reason.

## 1. Reader

Knows what a closure is. Has written them in several languages. Not fluent in runtime
implementation. Specific stated weakness: **cannot hold moving-state mechanisms in their head** —
lacks a stable notation/reframe, so complexity accretes until the thread is lost. The doc's
job is to hand over a **grip**, not to be complete.

## 2. Doc kind

Determined by content, not template:

- **Fork** at its core — capture strategy is a live decision with real occupants on every branch.
- **Mechanism** in its middle — open/closed transition + the open-upvalue list is machinery
  that must be understood exactly, not chosen.
- **Stateful** — therefore a trace **earns its place here**. This is one of the few docs where
  it does. Trace the close operation. Do not trace anything else.

## 3. The grip — **SUPERSEDED. The original was wrong about Phalcom.** ⚠

### 3a. What I originally wrote (WRONG — kept as the record of the error)

> ~~**An upvalue is a pointer that knows how to become an owner.** … **the read path never
> branches.** `*uv.location` is correct in both states … Self-pointing struct.~~

This describes **Lua**, not Phalcom, and I asserted it before reading the source. §10 flagged
exactly this risk and it fired. It also contaminated sonnet A's brief, so `draft-concept.md`'s
thesis is confidently wrong about the target language — through no fault of A, which had no
source access by design.

**Phalcom branches on every upvalue read**, twice when cross-fiber:
`Upvalue::Open { fiber: ObjRef, slot: usize } / Closed(Value)`, matched in `GetUpvalue`
(`vm/dispatch.rs` ~L1052). There is no `location`. There is no pointer. There is no
self-pointing struct.

### 3b. The real grip

> **A captured variable must be reachable after its frame dies. Everything follows from what
> you reach it with: an address, or a name.**
>
> Lua reaches with an **address** (`Value *location`) and buys a branchless read — paid for by
> every subsystem that moves memory (stack growth, moving GC, coroutine stacks), each of which
> must find and repair every open upvalue.
>
> Phalcom reaches with a **name** — `(fiber, slot)`, resolved fresh per access. It pays two
> branches per read and buys: a `Vec` stack that may reallocate freely, a collector that only
> marks and never fixes, cross-fiber capture that is *expressible* rather than accidental, and
> zero `unsafe`.
>
> The two states were never "pointer vs owner" — that is Lua's framing, forced by Lua's
> representation. They are **where the variable lives**: still in a stack (nameable by slot), or
> moved to the heap (nameable only by itself). Lua's self-pointing trick is a representation
> trick to *hide* that distinction from the read path. Phalcom declines the trick and shows it.

### 3c. The rhyme — the actual mental tool

**Phalcom never holds an address for something that can die. It holds a name, and checks.**

| Refers to | Named by | Dead handle yields |
|---|---|---|
| Any heap object (`ObjRef`) | index + generation | `None` |
| A block's home frame (`FrameToken`) | frame index + generation | `DeadFrameError` |
| A captured variable (`Upvalue::Open`) | fiber handle + slot index | resolved fresh; nothing to dangle |

One idea, applied three times. This is what serves the reader's stated weakness: three
mechanisms collapse into one they already understand. The codebase looks like many moving parts;
it is one part, moved.

## 4. The design space (must be walked, not listed)

The problem: a closure captures a local. Locals live on the stack. The stack dies at return.
The closure outlives it. This is the **upward funarg problem** (Weizenbaum, 1968) — name it,
it is a handle.

| Branch | Occupants | The bill |
|---|---|---|
| Refuse it | Java (`effectively final`), early C++ | No shared mutable capture. Array-of-one hack. Not a solution — a refusal. |
| Static link / display | Algol, Pascal | Only correct for *downward* funargs. Escape ⇒ dangling parent frame. The historical dead end. |
| Box everything captured, at declaration | Scheme (assignment conversion), naive JS engines | Allocation for a possibility that usually doesn't happen. Pay always, benefit sometimes. |
| Heap-allocate *every* activation | Smalltalk (contexts are objects) | Problem vanishes; cost is a context allocation per activation. **Phalcom is Smalltalk-styled and did NOT take this.** That refusal is a doc beat. |
| Stack slot + upvalue that closes | **Lua**, Wren, (Phalcom?) | The open-upvalue list, and the identity invariant it exists to protect. |
| Flat closure, copy at creation | ML/Chez | No list, no close — but needs a separate mutable-box analysis, and nested capture must copy through each level. |

## 5. Comparison filter

A language enters **only** if it does one of these. Otherwise cut. Expect ~6 to survive, not 10.

1. **Took the other branch, with the bill attached.** ("Ruby does X, which forces Y.")
2. **Has scars** — shipped bug, perf loss, spec change.
3. **Names something Phalcom does anonymously.** ← highest value for this reader. Names are
   the mental tools they said they lack.
4. **Ancestor** — explains shape that otherwise reads as arbitrary.

Vocabulary to import (this is a deliverable in itself): *upward funarg problem*; *open* vs
*closed* upvalue; *assignment conversion*; *flat* vs *linked* closure; *display* / *static link*;
*effectively final*; *cell variable* (`co_cellvars` / `co_freevars`); *display class* (C#);
*escaping* (Swift).

Provisional cast, subject to earning it:

- **Lua** — ancestor + name-giver. Wren copies it; Phalcom likely copies Wren. **Deep.**
- **JavaScript** — `for (var i…)` loop capture: the most famous closure bug in history, and
  `let` per-iteration binding as the fix. Scars + a bug the reader has personally hit. **Deep.**
- **Java** — the refusal branch, honestly argued. **Medium-deep.**
- **Smalltalk** — the pole where the problem doesn't exist. Load-bearing because of Phalcom's lineage. **Medium.**
- **C#** — display classes, *and changed the language spec in C# 5* to fix `foreach` capture.
  Breaking compat to fix closures is rare and instructive. **Medium.**
- **Swift** — `@escaping` puts in the **type system** what Lua puts in the **runtime**. Same
  distinction, different layer. One sharp paragraph, high payoff. **Short, deep.**
- **Python** — cells; `nonlocal` added in 3.0 *because there was no way to write*. **Short.**
- **C++** — `[&]` + return = UB. Shows precisely what Lua's `close` buys. **Short.**
- **Rust** — `Fn`/`FnMut`/`FnOnce`, `move`. Earns it only via the implementation angle: Phalcom
  is written in Rust and the impl must fight this. Fold into the source section or cut.

## 6. Tensions to surface

- **Escape ⊗ non-local return** — once a closure outlives its frame, `^` through a dead frame
  must trap, not corrupt. Adjacent; pointer, not a detour.
- **Loop scoping ⊗ capture** — fresh binding per iteration, or one slot reused? *This has a real
  answer in Phalcom's source and it is the JS `var` bug.* Must be answered, not hand-waved.
- **Open upvalue ⊗ moving GC** — an open upvalue points into the stack; the collector must cope.
- **Open upvalue ⊗ fibers** — a fiber owns a stack. Closing at fiber death. Phalcom has U-FIBER planned.
- **Upvalue ⊗ the call-chain seam** — capture cost is on the `for`-loop path (4 `.ph` frames/element).

## 7. Structural rules (constraints, not a skeleton)

- **Structure follows the theory.** No imposed heading set. It bottoms out where the theory
  bottoms out.
- **No comparative table as a checkbox.** Comparison is a weapon aimed at a named confusion.
- **Trace the close operation, and only that.** Stateful ⇒ trace earns its place *here*.
- **Mermaid where the shape is the point** (open vs closed pointer topology; the list). Not decoration.
- **Source anchors: symbol first, line second** (`closure.rs::Upvalue::close` @ ~L120). Bare line
  numbers rot; symbol names are checkable.
- **HEAD as-implemented.** Where v0.2 is unfinished, say so and cite the spec's intent as intent.
- Mark simplifications as lies with a forward pointer. Unmarked lies destroy trust on contact.

## 8. Checklist (gate before shipping)

- [ ] Grip stated early, in one sentence, and *earned* by the end.
- [ ] Read path shown to be branchless — explicitly, not implied.
- [ ] Every rejected branch made tempting before it is killed.
- [ ] Identity invariant explained: same slot ⇒ same upvalue object. **Why the list exists.**
- [ ] `close` traced step by step, from real structure.
- [ ] Loop-per-iteration question answered from source.
- [ ] Every language present passes the §5 filter. Named cut list for those that didn't.
- [ ] Vocabulary imported and marked.
- [ ] Anchors are symbol-first and exist at HEAD.
- [ ] Reader could re-derive the design. (§0)

## 9. Build sequence

| # | Deliverable | Who | Path |
|---|---|---|---|
| 1 | This file | me | `REQUIREMENTS.md` |
| 2 | Theory draft — no source access | sonnet A | `draft-concept.md` |
| 3 | Source map — graphify-led | sonnet B | `source-map.md` |
| 4 | The doc — synthesis, my judgment over A's bulk + B's ground truth | me | `../<part>/upvalues.md` |

**Division of labour.** Sonnet A supplies research recall and prose bulk — the history, the
precedents, the exact scars — which I would otherwise burn context re-deriving. I supply
judgment: what's subtly wrong, what earns its place, what the grip is. Sonnet B supplies ground
truth I must not guess at.

**Is source exploration necessary?** Yes. §0 demands anchors; §6's loop question has an answer
only the code holds; and whether Phalcom even *has* open/closed upvalues is unknown to me right now.

**Is spec reading necessary?** Marginal. Upvalues are mechanism, not policy. Bounded check for a
closure/capture ADR only — no spec-tree sweep.

## 10. Open risk — **FIRED. Resolved.** ✅

> ~~I do not yet know whether Phalcom implements Lua-style upvalues … §3's grip is written
> *assuming* Lua-style. If sonnet B reports otherwise, §3 through §6 are wrong and get
> rewritten.~~

**Outcome: half right, and the wrong half was the load-bearing half.**

Phalcom *is* Lua-style **architecturally** — two states, find-or-create map, recursive compiler
resolve, `is_local` descriptors, ADR-0013 accepted. So §4's design-space walk survived intact.

Phalcom is Lua's **inverse representationally** — name, not address. So §3's grip was wrong and
is rewritten at §3b. The lesson generalizes past this doc: *"X-style" is a claim about
architecture and says nothing about representation, and representation is where the
consequences live.* Two systems can share every structural feature and still make opposite
trades.

**Corrections applied to `draft-concept.md` during synthesis** (A was right about Lua
throughout; wrong only where the brief pointed it at Phalcom):

| A's claim | Phalcom's reality |
|---|---|
| read path is branchless | branches twice (`Open`/`Closed`, then fiber identity) |
| self-pointing `location` on close | `*upvalue_mut(cell) = Closed(v)` — variant replacement in the arena |
| sorted **linked list**, prefix walk | `BTreeMap<usize, ObjRef>`, `range(from..)` |
| open list is per-thread, holds everything on that stack | live map is the **running** fiber's; each `FiberObject` parks a mirror — because a slot key is meaningless across stacks and slot 5 would collide |
| "an open upvalue is a pointer the GC must fix" | no pointer exists; the hazard is **dissolved, not mitigated**. Kept in the doc as Lua's bill, not Phalcom's. |
| suspended fiber needs nothing done | Phalcom swaps the map in/out on switch (see collision reason above) |
| loop policy is "an empirical question theory can't answer" | correct, and honest — B answered it: `for` fresh, `while` shared |
| Rust cut as "belongs with the source doc" | **Rust is the forcing function** and belongs in the doc. Reinstated. |

**Synthesis neither agent could reach alone** (this is the argument for the two-track split):
A researched C# 5 breaking `foreach` while deliberately leaving `for` alone; B found `for` fresh
/ `while` shared. Joined: two languages, no shared lineage, drew the same line —
*the construct that hands you an element gets a fresh binding; the construct where you visibly
mutate your own counter does not.* That reframes Phalcom's split from apparent inconsistency
into the more defensible position. It exists only in the join.
