# REQUIREMENTS — `frames.md` (VM track Doc 3)

## The one obligation

> After reading, the reader can re-derive Phalcom's frame representation from the constraints
> alone: given "objects are `Copy` handles into an arena" (ADR-0009) and "the VM must own its call
> stack to support fibers and GC," the reader should land on **frame-as-`Copy`-value-in-a-`Vec`**
> without being told, and see why that forecloses a caller pointer.

## Reader

Knows PL design; not fluent in Rust runtime implementation. Has an imported intuition to dislodge:
*frames are heap objects you can grab* (Python `sys._getframe`, Smalltalk `thisContext`) **or**
*frames are just the native C stack* (tree-walkers). Cannot hold moving-state in their head, so the
push/pop/unwind trace must use stable notation and trace ONE hard case, not a survey.

## Doc kind

**Fork fused with a short mechanism.** The fork — *how do you represent an activation record?* —
is the spine (three branches, §"design space"). The mechanism — push on call, pop on return,
truncate on unwind — is the payoff trace that shows the chosen branch in motion. Lean fork: the
mechanism earns space only where it reveals the representation (truncate-not-unlink).

## The grip (from recon §2, grounded)

> **A frame is a value, not an object.** A flat `Copy` record in a `Vec`, so the call stack is an
> array you **truncate**, not a linked list you unlink — and the caller is the record one slot
> down, never a pointer the frame carries.

## The design space (fork — make each branch tempting, then bill it)

| Branch | Who | Buys | Costs / forecloses |
|---|---|---|---|
| **Heap frame object + parent (`f_back`) pointer** | CPython (`PyFrameObject`), Ruby | Reifiable, reflectable (`sys._getframe`), debuggable; frame outlives call trivially | Per-call heap alloc; a self-referential parent chain the borrow checker hates; GC pressure |
| **Flat array of value-records / register windows** | Lua (`CallInfo`), Wren, **Phalcom** | No per-call alloc; caller chain is array order; unwind = truncate; cache-friendly | Frame is not a first-class object (no cheap `thisContext`); needs a side mechanism for identity across reuse (→ generation counter, Doc 6) |
| **Native machine stack** | Tree-walkers, many JITs | Zero bookkeeping; fastest calls | Cannot reify frames; fights green threads / fibers (can't swap a native stack cheaply); no `System.gc` visibility into locals |

Phalcom took the middle branch — **but as a consequence of ADR-0009, not a bake-off.** State that
(honesty pass). The Copy-ness (not just "an array") is the ADR-0009 payoff that makes the middle
branch borrow-clean in Rust.

## Comparison filter (a language enters only on bill / scar / names-something / ancestor)

- **Lua** — CallInfo register-window frames in a flat array. *Ancestor + took-the-same-branch.*
  Names the array-of-records representation. **Keep, deep.**
- **CPython** — `PyFrameObject` with `f_back` linked parent + `sys._getframe` reification. *Took the
  other branch, with the bill (per-call alloc).* Names *reification* and the parent pointer Phalcom
  omits. Note 3.11 "zero-cost frames" softened the alloc bill — **flag, don't overclaim.** **Keep.**
- **Smalltalk** — reified `MethodContext`/`BlockContext`, `thisContext` first-class. *Ancestor of the
  semantics; the extreme of frame-as-object.* Phalcom keeps Smalltalk semantics with a Lua
  representation — the recurring theme. **Keep, medium.**
- **Cut:** JVM (frame representation not reified the same way; JIT-specific), Ruby MRI (redundant with
  CPython for this axis), generic tree-walkers (folded into the native-stack branch). Name them cut.

## Tensions to surface

1. **Smalltalk semantics vs Lua representation.** Phalcom's object model is Smalltalk (everything an
   object, contexts conceptually first-class), but frames are deliberately *not* objects — they are
   Copy values. The reader must see this is intentional, not an omission.
2. **`Copy` frame vs the heap it points into.** The frame is Copy, but its `closure`/`context`
   handles refer to heap objects the GC owns. So frames must be GC *roots* even though they are not
   heap objects. One breath.
3. **One receiver, four contexts.** `CallContext` has to cover instance / class / module / immediate
   receivers; the `Immediate` variant (no `ObjRef`) is where the uniform "receiver is a handle"
   assumption breaks. The counterintuitive case to trace.

## Structural rules

- Grip stated early, earned by the truncate-not-unlink trace.
- ONE predict-then-check: the `f_back` / caller-pointer question (recon §4).
- Trace the HARD case: the `truncate` unwind and/or the `Immediate` receiver — not a vanilla call.
- Mark every lie with a forward pointer (generation/token → Doc 6; fiber mirror → concurrency).
- Anchors symbol-first (`frame.rs::CallFrame` @ ~L66), never bare line numbers.
- Diagram: draw the array-of-records stack with the caller-is-slot-below relation — the thing whose
  *shape* is the point. Do NOT draw parent-pointer arrows (the thesis is that there are none).
- Pay off Doc 2 (a frame is one activation *of* a ClosureObject) and set up Doc 4 (a message-send is
  what pushes the next frame) and Doc 6 (frame identity).

## Must-cover checklist

- [ ] `CallFrame` is `Copy`; the seven fields; why Copy → plain `Vec`, no `Rc<RefCell>`, no borrow panic.
- [ ] No caller/parent pointer; Vec order is the chain; `frames[i-1]` is the caller.
- [ ] Push = `new_call_frame` (+ generation bump, marked as Doc-6 machinery); pop = `frames.pop`;
      unwind = `frames.truncate`.
- [ ] `CallContext`'s four variants; the `Immediate` scar (ADR-0018/U5).
- [ ] `stack_offset` = the frame's window into the shared value stack (fiber-relative — bounded).
- [ ] Frames are GC roots (`collect_roots` walks `vm.frames`; `trace_frame`).
- [ ] ADR-0009 is the deliberated decision; frame-as-value is its consequence, not its own ADR.
- [ ] Lies marked: generation/`home_frame_token` → Doc 6; per-fiber mirror → concurrency.

## Agent B question list (verification targets)

1. Quote `CallFrame` (L66) + `Copy` derive; confirm NO parent field. (load-bearing negative)
2. Quote all four `CallContext` variants + the `Immediate` doc (L51–61).
3. Confirm module-doc claim L1–6 (Copy → Vec, no Rc/RefCell) and tie to ADR-0009.
4. `new_call_frame` @ ~L29: construction + push + generation bump.
5. Pop @ ~L1099, `unwind_to` truncate @ ~L110; caller resumes as `frames[len-1]`.
6. `VM::frames` (live) vs `FiberObject::frames` (per-fiber backing); mirror + swap. Bounded.
7. `stack_offset` semantics; fiber-relative note (fiber.rs L77).
8. GC: `trace_frame` @ ~L37, `collect_roots` @ ~L32. One line.
9. RUN: nested-call stack trace (order visible); a `DeadFrameError` if reachable (identity check
   exists — mechanism deferred to Doc 6).

## Open risks (R1–R5)

- **R1 — "frame is Copy" over-simplifies if a field is secretly not Copy.** Mitigation: B quotes the
  full field list; all confirmed Copy in recon. If wrong, the grip's "truncate not unlink" still
  holds but the "no borrow surface" claim weakens.
- **R2 — the fiber mirror is more load-bearing than "one breath."** If `stack_offset` semantics can't
  be explained without the mirror, promote it from a footnote to a bounded subsection — but never to
  the spine. Watch during synthesis.
- **R3 — CPython 3.11 zero-cost frames may make the "heap alloc per call" bill look stale.** Must
  flag that modern CPython lazily materializes frame objects; keep the historical bill but date it.
- **R4 — the design-space "bake-off" framing risks flattering the codebase** (recon §3). The middle
  branch fell out of ADR-0009; the honesty pass must say Phalcom did not deliberate frame
  representation as such.
- **R5 — bleeding into Doc 6.** `generation`/token/`DeadFrameError`/non-local-return mechanics belong
  to frame-identity. Doc 3 may *name* them as fields and *show* a frame gets a fresh generation, but
  must not explain the identity check or unwinding-through-a-token. Hard boundary.

## §11 Reconciliation record (Phase 4)

### A-vs-Phalcom table (theory → reality, with the settling line)

| Agent A's theory claim | Phalcom reality (Agent B) |
|---|---|
| Branch (b) frame is "register-window": a small frame array + one shared value array, `stack_offset` windows | Exact: `VM::frames: Vec<CallFrame>` (`vm/mod.rs` ~L53) + `VM::stack: Vec<Value>`; `stack_offset` field indexes the value stack (`frame.rs` ~L76) |
| Branch (b) "never stores a caller pointer — array position IS the caller" | Confirmed negative: `CallFrame` (`frame.rs` ~L66) has **no** parent field; `frames[i-1]` is the caller |
| Unwind = bulk truncate, O(1) frame-bookkeeping vs branch (a)'s O(N) unlink | Confirmed: `unwind_to` = `frames.truncate` (~L110); Return = `frames.pop()` (~L1099); no relink |
| Branch (b) owes a "generation/token identity mechanism" (named, deferred) | Confirmed present: `generation` bumped every `new_call_frame`; `FrameToken` — **but that is Doc 6, inherit A's deferral** |
| Branch (b) frame is "not a first-class value the language can hand around; introspection is a copy-out side channel (cf. Lua `debug.getinfo`)" | True *and stronger at HEAD*: the copy-out path (`runtime_error`/`print_rt` walking `frames.rev()`) exists but is **not wired to the CLI** — `cli.rs::cmd_run` only `eprintln!("{e}")`. Phalcom is currently *below* Lua in reflection surface |
| Frame is a GC root regardless of branch; branch (b) needs an *explicit* root walk | Confirmed: `collect_roots` (`gc.rs` ~L93) explicitly walks `vm.frames` → `trace_frame` (`trace.rs` ~L37) |
| CPython 3.11 lazily materializes heap frames (semantics vs representation split) | Not a Phalcom fact — A's comparison, kept **flagged/dated** per A's own confidence note |

### Honesty corrections (§5.2)

1. **Frame-as-`Copy`-value is a *consequence* of ADR-0009, not a frame-representation bake-off.** ADR-0009 deliberated `Rc<RefCell>` vs handle-arena vs immediate `Gc`; "the frame is `Copy`" falls out of "every link is a `Copy` handle." The doc states the design-space walk is pedagogical scaffolding.
2. **The LIFO-stack-of-records framing is textbook**, not a Phalcom decision. Label as such.
3. **The missing CLI traceback is an absence at HEAD, not designed minimalism.** The machinery exists and looks correct; it is simply unwired from `cmd_run`. Present as current-state fact (Agent B verified the call chain), never as a principle.
4. **`CallContext::Immediate` is real and deliberated (ADR-0018/U5) but narrow**: it exists so the sacred-selector inliner's override-epoch deopt guard is *exercisable* by a closure method on an immediate receiver. Present accurately, not as grand receiver-uniformity design.

### Claims ledger (§5.3)

- "No per-call allocator call on the hot path" — cite: push/pop on `Vec<CallFrame>`, `new_call_frame` does no alloc. **Hedge:** the `Vec` itself grows rarely (amortized), so "no allocator traffic per call" not "zero allocation ever." ✓ cited.
- "Fiber switch is an O(1) `Vec` swap" — cite: `VM::frames` doc, "an O(1) pointer-free copy (a `Vec` swap)". ✓ verbatim quote.
- "`DeadFrameError` fires" — cite: Agent B ran `runtime_non_local_return_dead_frame.ph`, verbatim stderr. ✓ ran.
- "CLI prints only the message, no traceback" — cite: Agent B ran a 3-deep throw → `boom-from-inner`; traced `cli.rs::cmd_run` eprintln + `interpret_source`-only callers. ✓ ran + traced.
- "cache-local activation" — qualitative array-branch property (A's theory), **NOT a Phalcom measurement**. Frame it as design reasoning, not a measured Phalcom claim. ✓ labelled.
- CPython 3.11 lazy frames / Lua `CallInfo` linked-list — A's **[flagged]** items; keep only the high-confidence cores (CPython moved rep rightward while keeping semantics; Lua locals live in one shared array), drop the low-confidence internals. ✓ trimmed.
- Links: only `execution-loop.md` and `compiled-artifact.md` (both exist in `docs/learn/vm/`). ADRs/spec as plain text, no bare directory links. ✓
