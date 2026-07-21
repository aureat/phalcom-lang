# U-FIBER — Implementation Spec: bare cooperative `Fiber` on the restricted re-entrant loop

> **Status:** Normative work order for a `phalcom-implementer`. Realizes
> [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §1–§7 and the
> deepened [specification.md](specification.md) (which extends
> [`concurrency.md`](../../../spec/current/concurrency.md) §1). `file:line` anchors were
> verified against source; **re-confirm at dispatch** (spine files shift under concurrent
> sessions).
>
> **Baseline: grounded at HEAD `9d3b7e1`** (U-CORE track closed, floor **88**; graph
> rebuilt 16:54). **Reviewer ON** (deep VM change) — hand the diff to `phalcom-reviewer`;
> never self-approve. **Worktree isolation** (mutates `vm.rs`/`heap.rs`/`core.ph` while
> U-ITER and U-CORE units are live). Green gate: `./scripts/verify.sh` exits 0 **and**
> `cargo doc --workspace --no-deps` clean; run the **miri** lane for the stack-swap
> plumbing if U0 wired it.
>
> **Scope in one line (user, 2026-07-12): BARE `Fiber` ONLY** —
> `new`/`call`/`try`/`yield`/`current`/`abort`, enough for generators and the
> `for`-yield ergonomic. **No `Future`, no ready-queue, no `System` scheduler hooks** —
> those are the Deferred [[U-FUTURE]](../U-FUTURE/implementation-spec.md) track.

---

## §0. Prerequisites + scope gate

### Phase 0 — the forward-compat §7.3 pre-fiber audit (do this FIRST, on real HEAD)

**Decision (user): the §7.3 audit + D5/D7 hardening live INSIDE U-FIBER as Phase 0.** The
typed switch signal has no consumer to verify against until fibers exist, so it is not a
separable unit. Phase 0 is a **read + targeted-fix** gate; do not proceed to Phase 1 until
all five hold on HEAD. **Return a one-paragraph verdict per row before any Phase 1 edit.**

| Invariant | Check on HEAD (verified this pass) | Verdict / action |
|---|---|---|
| **D4 — `next_frame_generation` stays VM-global** | `vm.rs:72` — `pub(crate) next_frame_generation: u64` is a **`VM` field**, not in any per-fiber struct. | **Verified.** Keep it global. A per-fiber counter is a silent cross-fiber miscompile (spec §4.4). Pin an invariant test (§4). |
| **D2 — no new `Value` arm for heap types** | `value.rs:30-45` — `Value` = `Nil / Bool / Number / Symbol / Obj(ObjRef)`; **no `Fiber` arm**. `List` proves the `Object::` pattern. `concurrency.md` §1's `Value::Fiber(PhRef<…>)` is **stale** (predates the handle heap). | **Verified.** Add `Object::Fiber`, reach via `Value::Obj`. Do **not** add `Value::Fiber`. |
| **D3 — `stack_offset` frame-relative** | `frame.rs:75` — `stack_offset` is a window-relative index; `CallFrame` is `Copy` (`frame.rs:65`). | **Verified.** Per-fiber stacks starting at 0 need no rebasing; the O(1) switch (§3.2) depends on it. |
| **D5 — the typed fiber-switch signal** | `vm.rs:442-469` — the `call_method` **Primitive arm** reconciles a non-local return by a `frames.len()` **shrink heuristic** (`frames_before = self.frames.len()` `:442`; `if self.frames.len() >= frames_before` `:445` else re-push `:468`). A fiber switch **also** moves `frames.len()`. | **Confirmed as the only frame-count consumer at this site.** **Build the typed signal in Phase 1 §3.3.** Phase 0 just scopes it. |
| **D7 — the U-CORE-6 unwind is fiber-local** ⚠️ **the live risk (DEC-FIB-A)** | **Read on HEAD:** the landed U-CORE-6 raise returns `Err(RuntimeError::Raise { error, rendered })` (`error.rs:86-94`) that propagates **via Rust `?`** through `run_until` (`vm.rs:818`). `run_until` drains to `base_frames` **only on a normal `Return`** (`vm.rs:820-826`, `:1149-1151`); it has **no error catch** and does **not** truncate `self.frames` on `Err` — the error rides straight to the top-level driver. **There is no fiber floor and no floor-parameter.** | **DEC-FIB-A RESOLVED (verify-on-HEAD): U-FIBER OWNS THE FIX.** No change to `error_raise`/`RuntimeError::Raise` is needed. U-FIBER adds the fiber-floor **capture** at the resume boundary (spec §5.1, §3.4 below): catch `PhError::Runtime(RuntimeError::Raise{..})` (+ other terminal `RuntimeError`s) where the `call`/`try` primitive runs the fiber body, store `error` in the result slot, set `failed`, clear the fiber's frames, resume the resumer. **Re-read this code on HEAD at dispatch** — if a later change adds a floor stop-point to `run_until`, reuse it instead. |

### Already landed (do not rebuild) — verified at HEAD

| Dep | What it gives U-FIBER | Ground truth |
|---|---|---|
| **U1 arena heap** | `Object` enum (`heap.rs:68`: `Instance/Class/Method/Module/Closure/Str/Block/BoundMethod/Upvalue/List`) — `Object::Fiber` is one more variant. | `heap.rs:68`; ADR-0009 |
| **U4 blocks/closures** | `block_call` (`primitive/block.rs:117`) with the re-entrant `vm.run_until(base_frames)` (`:153`) — the native frame the guard refuses to yield across. | `primitive/block.rs:117-154` |
| **U10 non-local return** | `Bytecode::ReturnNonLocal` handler (`vm.rs:1154-1205`) — eager, one-shot, fiber-local unwind template; `FrameToken{frame_index, generation}` (`frame.rs:19-24`); `DeadFrameError` (`error.rs:150`). | `vm.rs:1154`; `frame.rs`; `error.rs` |
| **U-CORE-6 error root** | `RuntimeError::Raise { error, rendered }` (`error.rs:86-94`); `PhError::Runtime` (`error.rs:13`); the unified unwind the fiber floor captures. | `error.rs` |
| **Class-tower machinery** | `create_core_classes`/`make_core_class` (`universe.rs:95`), `install_primitives` (`universe.rs:260`, `primitive!`/`primitive_static!`), `verify_invariants` (`universe.rs:485`), `add_class!` in `install_core` (`vm.rs:388-400`). | `universe.rs`, `vm.rs` |

### Explicitly OUT of scope (RESERVE, do not implement)

- **`Future`/`async`/`await`/`then`/scheduler/ready-queue/`System.sleep`** — the Deferred
  [[U-FUTURE]](../U-FUTURE/implementation-spec.md) track. Keep the `resumer` link + result
  slot **general** so `await` layers over them (spec §7.2). Ship none of it.
- **ADR-0033 `CallBlock` trampoline** — Deferred. U-FIBER **builds** the typed switch
  signal (§3.3) ADR-0033 depends on, but does **not** trampoline the block call-site.
- **`iterate`/`for` machinery** — [[U-ITER]](../U-ITER/implementation-spec.md).
- **`ensure`-on-abandoned-fiber, `Fiber.finish`, resource caps** —
  `experimental/fiber-ensure-and-limits.md`, post-v0.2.

### Non-negotiables carried in

- **`unsafe` FORBIDDEN** — the whole point of Option A over stackful C is zero `unsafe`
  (ADR-0030 §Alternatives C). If a slice seems to need it, **stop and flag**.
- **Floor extension is an ADR-0019 amendment authorised by ADR-0030 §Consequences** — no
  new ADR; **bump the floor census** for the new bindings. Class-side selectors pass
  `verify_invariants()`.
- No native fiber stacks (GC roots stay in the arena, spec §7.3). Full rustdoc on every
  new item, citing ADR-0030 §.

---

## §1. What exists vs what is missing (grounded)

### Exists (verified at HEAD)

- **Single VM value+frame stack:** `VM { frames: Vec<CallFrame> (vm.rs:52), stack:
  Vec<Value> (vm.rs:54), next_frame_generation: u64 (vm.rs:72), … }`. U-FIBER relocates
  "current stack/frames" behind a `current: ObjRef`.
- **Re-entrant dispatch:** `run_until(base_frames)` (`vm.rs:818`) with the normal-drain at
  `vm.rs:820-826`; `Return` handler `vm.rs:1137-1153` (`return Ok` when `frames.len() <=
  base_frames`, `:1149`); `ReturnNonLocal` handler `vm.rs:1154-1205` (eager fiber-local
  unwind, `DeadFrameError` `:1179`, `frames.truncate(token.frame_index)` `:1204`). The
  Primitive-arm heuristic to replace: `vm.rs:442-469`.
- **`block_call`** `primitive/block.rs:117` → the re-entrant `vm.run_until(base_frames)`
  `:153` (the native frame yield refuses to cross).
- **Disasm** renders opcodes via `{:?}` Debug (`bin/phalcom/disasm.rs:18`) — **a new
  `Yield` opcode needs NO disasm arm** (correction to plan §3, which listed a disasm arm).
- **Bootstrap:** `add_class!` block `vm.rs:388-400`; `install_primitives` List/Error
  blocks `universe.rs:445-469`; `verify_invariants` field-count fences `universe.rs:615-625`.

### Missing (this unit adds)

| Missing | Add in |
|---|---|
| `Object::Fiber(FiberObject)` variant + the `FiberObject` struct (spec §2.1) | `heap.rs` |
| `current: ObjRef`; read current `stack`/`frames` through it; a default "main" fiber | `vm.rs` (SPINE) |
| typed `ControlFlow`/switch enum replacing the `frames.len()` heuristic (`vm.rs:442-469`) | `vm.rs` (SPINE) |
| `Yield` opcode + handler; the restricted-yield guard | `bytecode.rs` + `vm.rs` |
| `fiber_new`/`call`/`try`/`yield`/`current`/`abort` primitives + the fiber-floor capture (spec §5.1) | `primitive/fiber.rs` (**new**) |
| `Fiber` class row + primitive registration + floor-census bump | `universe.rs` |
| `class Fiber` surface wiring (class-side `yield`/`current`/`abort`) | `core.ph` |
| `CannotYieldAcrossNativeFrame` error | `error.rs` (new `RuntimeError` variant or a surface `< Error` class — D-FIB-1) |
| goldens + invariants + graduate the U-ITER PENDING | `tests/lang/concurrency/`, `tests/invariants.rs`, `MANIFEST.md` |

**`value.rs` — NO change** (D2, no `Value::Fiber` arm); listed to make the non-edit explicit.

---

## §2. The native/`.ph` split + exact insertion points

**Decision: native `FiberObject` + primitives (mirroring `List`), thin `.ph` surface
wiring.** The switch machinery, per-fiber stacks, `Yield` opcode, and fiber-floor capture
are native; `class Fiber`'s class-side selectors are `.ph` over the primitives.

| Concern | Native (Rust) | `.ph` |
|---|---|---|
| `Object::Fiber(FiberObject)` + struct | ✅ `heap.rs` | — |
| `current: ObjRef`; stack/frame relocation; typed switch; `Yield` handler + guard | ✅ `vm.rs` | — |
| `Yield` opcode | ✅ `bytecode.rs` (no disasm arm) | — |
| `fiber_new`/`call`/`try`/`current`/`abort` + `Fiber.yield` + fiber-floor capture | ✅ `primitive/fiber.rs` (new) | — |
| `class Fiber` name + class-side `yield`/`current`/`abort` wiring | — | ✅ `core.ph` |
| `CannotYieldAcrossNativeFrame` | ✅ `error.rs` (+ maybe a surface class, D-FIB-1) | optional reopen |

### Insertion points (exact — re-confirm at dispatch)

1. **`heap.rs`** — add `Fiber(FiberObject)` to `enum Object` (`heap.rs:68`) and define
   `FiberObject { stack: Vec<Value>, frames: Vec<CallFrame>, status: FiberStatus, resumer:
   Option<ObjRef>, result: Value, entry: ObjRef /* closure */ }` with an accessor helper
   (`as_fiber`/`fiber_mut`) mirroring `as_instance`/`as_list`. Full rustdoc (ADR-0030 §2).
2. **`vm.rs` VM struct** (`vm.rs:47-…`) — add `current: ObjRef` (the running fiber). In
   `VM::new`, allocate a **default "main" `FiberObject`** wrapping today's behaviour, set
   `current` to it, and route the existing `frames`/`stack` **through** `current` (either
   move them into the main fiber and read via `current`, or keep `VM.frames`/`stack` as
   the *live mirror* of `current`'s — D-FIB-4). **Phase 1 is a pure refactor: existing
   suite must stay green.**
3. **`vm.rs` `call_method` Primitive arm** (`vm.rs:434-471`) — replace the `frames.len()`
   heuristic with a typed `ControlFlow` match (§3.3). `native_fn` returns
   `PhResult<ControlFlow>` (or the loop inspects a VM-set switch flag — D-FIB-5); the loop
   honors `Switch` / `Return` / ordinary distinctly.
4. **`vm.rs` `run_until`** (`vm.rs:818`) — teach the loop to honor a `Switch` signal
   (repoint to `current`, resume its `ip`). The normal-drain (`:820-826`) and
   `Return`/`ReturnNonLocal` handlers (`:1137`/`:1154`) are otherwise unchanged.
5. **`bytecode.rs`** — add `Yield` after `WrapSome` (`bytecode.rs:194`). Handler in
   `run_until` near `ReturnNonLocal`. **No `disasm.rs` arm** (renders via `Debug`).
6. **`primitive/fiber.rs`** *(new module)* — `fiber_new`, `fiber_call`, `fiber_try`,
   `fiber_yield`, `fiber_current`, `fiber_abort` (§3). Register `pub mod fiber;` in
   `primitive/mod.rs`.
7. **`universe.rs`** — create the `Fiber` class row in `create_core_classes` (`:95`) via
   `make_core_class`; add it to `CoreClasses`; register primitives in `install_primitives`
   (append after the Error block, `universe.rs:469`); add `add_class!(fiber_class)` in
   `install_core` (`vm.rs:388-400`); add a `verify_invariants` parallel-rule check
   (`universe.rs:485`); **bump the floor census** (`floor-census.md`, no new ADR).
8. **`core.ph`** — `class Fiber` reopen wiring class-side `yield(_)`/`current`/`abort(_)`
   to the primitives. **Serialize against U-ITER (`List`) and every U-CORE `core.ph`
   editor** (§6 collision).
9. **`error.rs`** — `CannotYieldAcrossNativeFrame` (D-FIB-1); if native `RuntimeError`, add
   a `#[error("…")]` variant; if a surface class, wire it like U-CORE-6's MNU.

---

## §3. Concrete bodies / pseudocode

### 3.1 `fiber_new` (class-side `Fiber.new(_)`)

```
fiber_new(vm, class_receiver, [entry_fn]):
    require entry_fn is a Function/Block closure       # else RuntimeError::Type
    let fib = FiberObject { stack: [], frames: [], status: Suspended,
                            resumer: None, result: None-value, entry: closure_of(entry_fn) }
    return Value::Obj(vm.heap.alloc(Object::Fiber(fib)))
```

### 3.2 `fiber_call` / `fiber_try` (the O(1) switch + fiber-floor capture)

```
fiber_call(vm, fiber_receiver, args):           # args: 0 or 1 (call / call(_))
    let callee = as_fiber(fiber_receiver)
    if callee.status in {Done, Failed}:  return Err(<cannot resume finished fiber>)   # D-FIB-2
    let resumer = vm.current
    # 1. Park the resumer: store live stack/frames back into it, mark suspended.
    store_live_into(vm, resumer); set_status(resumer, Suspended)
    # 2. Repoint current to the callee; load its stack/frames as live.
    callee.resumer = Some(resumer); set_status(callee, Running); vm.current = callee_ref
    load_live_from(vm, callee)
    # 3. Deliver the transferred value (arg → the value of the callee's suspended yield,
    #    or the entry parameter on first resume). Push it where the callee expects it.
    deliver_resume_value(vm, callee, args)
    # 4. Return a TYPED SWITCH to the dispatch loop — do NOT start a nested run_until.
    return ControlFlow::Switch                    # §3.3
```

Failure capture (spec §5.1, DEC-FIB-A) happens at the **fiber floor** — the boundary that
owns the fiber body's `run_until`. Whether that boundary is the top-level `run_until`
(pure Option A, switch-driven) or a scoped `run_until` the resume primitive owns is
**D-FIB-6**; either way:

```
on the fiber body's terminal result at its floor:
    Ok(v)            → callee.status=Done;   callee.result=v;     switch back to resumer, deliver v
    Err(Raise{error})→ callee.status=Failed; callee.result=error; callee.frames.clear()
                       under call → re-raise error to resumer ; under try → deliver error/None
    Err(other RuntimeError) → wrap as surface Error, same capture path
```

`fiber_try` is `fiber_call` with the failure branch delivering the `Error`/`None` instead
of re-raising (spec §5.2).

### 3.3 The typed `ControlFlow` signal (replaces `vm.rs:442-469`)

```rust
/// How a primitive/opcode reconciled with the dispatch loop (ADR-0030 §5, D5).
enum ControlFlow {
    /// Ordinary primitive return: land `value` in the receiver slot.
    Return(Value),
    /// A fiber switch occurred: the loop must resume `vm.current` at its saved ip,
    /// delivering the transferred value cross-fiber. NOT a frames.len() delta.
    Switch,
}
```

The Primitive arm becomes an explicit match on this, **not** the `frames.len() >=
frames_before` test (`vm.rs:445`). `ReturnNonLocal` keeps its own eager-unwind path
(`vm.rs:1154`); it is a distinct third cause and must remain distinguishable from `Switch`.
*(A minimal alternative keeps `native_fn -> PhResult<Value>` and sets a `vm.pending_switch`
flag the loop checks — D-FIB-5. The typed return is cleaner and is recommended.)*

### 3.4 `Yield` opcode + `fiber_yield` (class-side `Fiber.yield(_)`)

```
fiber_yield(vm, class_receiver, [value]):
    let me = vm.current
    if me.resumer is None:  return Err(<cannot yield the root fiber>)          # §2 rule 7
    if native_reentry_between_floor_and_here(vm):                              # the guard, D-FIB-3
        return Err(CannotYieldAcrossNativeFrame)                              # catchable
    me.result = value ; set_status(me, Suspended)
    let resumer = me.resumer ; store_live_into(vm, me)
    vm.current = resumer ; set_status(resumer, Running) ; load_live_from(vm, resumer)
    deliver_yield_value(vm, resumer, value)         # lands as the value of resumer's call()
    return ControlFlow::Switch
```

- The **`Yield` opcode** is the compiled form of a `Fiber.yield(_)` send if the compiler
  chooses to specialize it; otherwise `Fiber.yield` is an ordinary class-side send to
  `fiber_yield` and the opcode is unnecessary. **D-FIB-7:** decide whether `Yield` is a
  real opcode or `Fiber.yield` is a plain primitive send. ADR-0030 §Consequences names a
  `Yield` opcode; if kept, it drives `fiber_yield` and needs no disasm arm.
- `native_reentry_between_floor_and_here` is the restricted-yield guard: track a VM-level
  **native re-entrancy marker** (incremented by `block_call`/`send_dynamic`/
  `forward_does_not_understand` around their recursive `run_until`), recorded at the
  fiber's floor on resume; yield is legal iff the current marker equals the floor's
  (D-FIB-3).

### 3.5 `fiber_current` / `fiber_abort`

```
fiber_current(vm, _):  return Value::Obj(vm.current)
fiber_abort(vm, _, [err]):                        # class-side; fails the current fiber
    # equivalent to `throw err` at the fiber floor: capture into result, resume resumer
    return Err(RuntimeError::Raise { error: err, rendered: render(err) })   # caught at the fiber floor (§3.2)
```

### 3.6 `class Fiber` in `core.ph`

Thin wiring; class-side selectors route to the primitives (mirror how other kernel classes
expose class-side methods). Keep it minimal — the mechanism is native.

---

## §4. Test strategy — `concurrency` corpus label

Graduate `tests/lang/concurrency/pending/concurrency_fiber_yield_resume`; add the guard
negative; bump `tests/lang/MANIFEST.md`.

| ID | Test | Kind |
|---|---|---|
| **C-FIB-1** | counter generator: `Fiber.new { let n=0; while(true){ Fiber.yield(n); n=n+1 } }` → `call`s yield `0,1,2` (ADR-0030 §4). | golden |
| **C-FIB-2** | resume value: the argument to `call(_)` becomes the value of the suspended `yield`. | golden |
| **C-FIB-3** | restricted-yield guard: `Fiber.new { [1,2].each { x => Fiber.yield(x) } }` raises a **catchable** `CannotYieldAcrossNativeFrame` (not a host abort). | negative |
| **C-FIB-4** | failure capture (D7): a fiber that `throw`s ends `failed`, `try` yields the `Error`, the host keeps running (proves the unwind stops at the fiber floor — the DEC-FIB-A fix). | golden |
| **C-FIB-5** | fiber-local non-local return (D4): a `return` token from fiber A used in fiber B → `DeadFrameError`; **`next_frame_generation` stays VM-global**. | golden + `invariants.rs` |
| **C-FIB-7** | typed switch, not length delta: existing non-local-return goldens stay **green** after the Primitive-arm swap. | regression |
| **C-FIB-8** | **no `unsafe`** in fiber machinery; no native fiber stacks. | grep + review |
| **C-FIB-9** | `Fiber` passes `verify_invariants()`; floor census bumped. | `invariants.rs` + census |
| **C-FIB-6** *(cross-unit, PENDING → graduates)* | [[U-ITER]](../U-ITER/implementation-spec.md)'s `pending/for_generator_suspends`: `Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }` suspends, yields `1,2,3`. Flip to PASS once **both** land. | pending golden |

**Invariants (`tests/invariants.rs`):** `next_frame_generation` is a `VM` field (compile-
level + a runtime cross-fiber `DeadFrameError` assertion); a parked (non-`current`,
non-`done`) fiber's stacks remain reachable arena roots (spec §7.3).

---

## §5. Must-not-preclude

| Hazard | How this design clears it |
|---|---|
| **native-stack ⊗ suspendable control (CROWN JEWEL).** | Handled by **restriction, not machinery**: the guard (§3.4) refuses the unsafe case (`CannotYieldAcrossNativeFrame`) rather than corrupting it. No native fiber stacks ⇒ nothing new for a future moving GC to scan (ADR-0009 preserved). |
| **`next_frame_generation` VM-global (D4).** | Kept a `VM` field (`vm.rs:72`); pinned by an invariant test (§4 C-FIB-5). A per-fiber counter would be a silent cross-fiber miscompile. |
| **Option-B additivity (ADR-0033).** | The switch is a **typed signal** (§3.3), not baked into a length heuristic; the guard (§3.4) is **lift-by-deletion**. A→B only *removes* the guard — do not bake the restriction anywhere it can't be deleted. |
| **`Future`/`await` (Deferred [[U-FUTURE]](../U-FUTURE/implementation-spec.md)).** | The `resumer` link + result slot stay **general** (§2.1 spec) — `await` suspends through exactly them. Not generator-specialized. |
| **`ensure`-on-abandoned-fiber + limits (post-v0.2).** | The fiber-local unwind + fiber-floor capture (§3.2) is the seam that layer builds on; nothing here forecloses it. |
| **Moving/tracing GC (ADR-0009).** | No native fiber stacks; parked-fiber stacks are arena roots (§4). Do not introduce native-memory fiber state. |

---

## §6. Open sub-decisions, build order, traceability

### Sub-decisions (recommend; flag if deviating)

- **DEC-FIB-A ✅ RESOLVED (verify-on-HEAD, this pass).** The landed U-CORE-6 unwind is
  **not** floor-parameterised — it propagates via Rust `?` with no fiber floor
  (`vm.rs:818-826`, `error.rs:86-94`). **U-FIBER owns the fiber-floor capture** (§3.2,
  spec §5.1). Re-read on HEAD at dispatch; if a floor parameter has since landed, reuse it.
- **DEC-FIB-scope ✅ (user).** Bare `Fiber` only; `Future`/scheduler = Deferred track.
- **DEC-FIB-0033 ✅ (user).** Keep ADR-0033 Deferred; build the switch signal, do not
  trampoline the block call-site.
- **D-FIB-1 — `CannotYieldAcrossNativeFrame` shape.** *Recommended:* a **surface `Error`
  subclass** (catchable via the U-CORE-6 unwind / `try`), so C-FIB-3 asserts catchability
  cleanly; a native `RuntimeError` variant is the lighter alternative but is only catchable
  once the on/ensure protocol lands. Prefer the surface class (spec §6 requires catchable).
- **D-FIB-2 — resume-a-finished-fiber error.** *Recommended:* a distinct catchable error
  (e.g. "cannot resume a `done`/`failed` fiber"); pick a clear message.
- **D-FIB-3 — the restricted-yield guard representation.** *Recommended:* a VM-level
  **native re-entrancy depth counter** incremented around each recursive `run_until`
  (`block_call:153`, `send_dynamic:595`, `forward_dnu:558`), with the fiber's floor depth
  recorded on resume; yield legal iff equal. *Alternative:* a sentinel frame at the floor.
- **D-FIB-4 — where the live stack/frames live.** *Recommended:* the running fiber owns
  them; `VM.frames`/`stack` are the live mirror of `current`, stored back on switch.
  Whether to keep `VM.frames`/`stack` as fields or move them wholesale into the `current`
  `FiberObject` is a refactor-shape choice — keep Phase 1 a **pure refactor** either way.
- **D-FIB-5 — typed return vs. VM flag.** *Recommended:* `native_fn ->
  PhResult<ControlFlow>` (§3.3). *Alternative:* keep `-> PhResult<Value>` + a
  `vm.pending_switch` flag. Typed return is cleaner and matches ADR-0030 §5's "typed
  signal" language.
- **D-FIB-6 — fiber-floor placement.** *Recommended:* the resume primitive scopes a
  `run_until` (or an equivalent boundary) it owns, catching the terminal result there
  (§3.2). Confirm it composes with the pure-switch top-loop model (§3.3) on HEAD.
- **D-FIB-7 — `Yield` opcode vs. plain send.** *Recommended:* keep the `Yield` opcode
  (ADR-0030 §Consequences names it); no disasm arm needed (`Debug`). If a plain
  `Fiber.yield` primitive send suffices, drop the opcode and note the deviation.

### Build order (small, independently-green diffs — from plan §4)

0. **Phase 0 audit** (§0) — five verdicts + the DEC-FIB-A finding (already resolved:
   U-FIBER owns the capture). Green (existing suite passes).
1. **`Object::Fiber` + `current: ObjRef` plumbing** — VM reads current stack/frames
   through `current`; a single default "main" fiber wraps today's behaviour. **Pure
   refactor, no surface change — verify the whole existing suite stays green. Highest-risk
   diff; land it alone.**
2. **Typed switch signal** — replace the `frames.len()` heuristic (`vm.rs:442-469`) with
   `ControlFlow`; existing non-local-return goldens stay green (C-FIB-7).
3. **`Yield` + `new`/`call`/`yield`** — the minimal generator; graduate
   `pending/concurrency_fiber_yield_resume`; pin the `counter` + resume-value goldens
   (C-FIB-1/2).
4. **Restricted-yield guard** — `.each { yield }` → `CannotYieldAcrossNativeFrame`
   (catchable); negative golden (C-FIB-3).
5. **`try`/`current`/`abort` + failure capture** — the fiber-floor capture (§3.2, the
   DEC-FIB-A fix); a failing fiber stores its `Error`, host survives (C-FIB-4, D7).

### Collision risk (flag, don't resolve — plan §3.1)

- **`vm.rs` / `heap.rs`** — spine files; **worktree isolation** is mandated. Confirm no
  concurrent holder.
- **`core.ph`** — never two editors; serialize `class Fiber` against U-ITER (`List`) and
  every U-CORE `core.ph` unit.

### Traceability

| Claim / requirement | Source |
|---|---|
| Execution model (Option A), typed switch, O(1) swap, GC roots, fiber-local return, floor amendment | [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §1–§7 |
| Surface (`new`/`call`/`try`/`yield`/`current`/`abort`), state machine | [concurrency.md](../../../spec/current/concurrency.md) §1; [specification.md](specification.md) §1–§2 |
| D1–D7 audit; re-entrant-loop finding; pre-fiber invariants | [forward-compat.md](../../../spec/current/core/forward-compat.md) §7 |
| DEC-FIB-A: U-CORE-6 unwind not floor-parameterised → U-FIBER owns capture | `error.rs:86-94`, `vm.rs:818-826`/`:1149-1151` (this pass); [ADR-0008](../../../adr/0008-layered-exceptions-and-result.md); [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §6 |
| No `Value::Fiber` (D2); `Object` enum | `value.rs:30-45`; `heap.rs:68`; [ADR-0009](../../../adr/0009-handle-arena-heap.md)/[ADR-0010](../../../adr/0010-tagged-value-enum.md) |
| `next_frame_generation` VM-global (D4); `stack_offset` relative (D3) | `vm.rs:72`; `frame.rs:75` |
| Primitive-arm heuristic to replace (D5) | `vm.rs:442-469` |
| `block_call` re-entrant frame (the crown jewel) | `primitive/block.rs:117-154` |
| `ReturnNonLocal` fiber-local unwind template | `vm.rs:1154-1205` |
| ADR-0033 Deferred; U-FIBER builds only the switch | [ADR-0033](../../../adr/0033-amend-fiber-execution-trampolined-block-callsite.md) §Decision 4 |
| `for`-generator seam | [[U-ITER]](../U-ITER/specification.md#fiber-generator-seam); [ADR-0035](../../../adr/0035-iteration-protocol-cursor.md) §5 |
| disasm renders via `Debug` (no Yield arm) | `bin/phalcom/disasm.rs:18` |
