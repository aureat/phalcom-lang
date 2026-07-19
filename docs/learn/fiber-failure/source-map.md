# What happens when a Phalcom fiber fails — source map (Agent B)

HEAD verified against: `cdd2117` (fix(vm): commit temp_roots GC escape-hatch paired with vm/mod.rs). Build: `cargo build -p phalcom-core --bin phalcom` — clean (one pre-existing `dead_code` warning on `init_selector_cache`, unrelated). All programs run via `./target/debug/phalcom <file.ph>`, scratch files under the session scratchpad, `var` rewritten to `let` throughout.

---

## Headline: architecture vs representation of a fiber failure

**Architecture** (the shape `run_until`'s `Err` arm implements) is a clean state machine: mark `Failed`, store the captured error as `result`, clear the three parked buffers, and either deliver to a `Try`-mode resumer or step to the next `Call`-mode resumer. Read as prose this sounds like an orderly teardown.

**Representation** (what actually happens to the live buffers at the moment of failure) is not a walk — it is two different bulk operations, and for the *first* fiber in the cascade (the one that actually raised) even the "clear" is a no-op that hides where the real state goes:

1. `phalcom-core/src/vm/dispatch.rs:319-321`, inside the cascade loop:
   ```rust
   self.heap.fiber_mut(failed).frames.clear();
   self.heap.fiber_mut(failed).stack.clear();
   self.heap.fiber_mut(failed).open_upvalues.clear();
   ```
   `Vec::clear`/`BTreeMap::clear` on the **parked mirror fields of the `FiberObject`**. For the fiber that is `self.current` at the moment of failure (the first iteration), these fields are already empty — per `heap/fiber.rs:53` ("empty while running — mirrored by `VM::frames`/`stack`/`open_upvalues`"), the *live* state sits in `self.frames`/`self.stack`/`self.open_upvalues` (the VM's own fields), not here. So this triple-clear is a genuine no-op for the failing fiber itself, and only does real work for a `Call`-mode intermediate resumer walked later in the cascade (whose `FiberObject` fields really do hold what it parked when it resumed the callee that ultimately failed).

2. The failing fiber's *actual* live state (still sitting in `self.frames`/`self.stack`/`self.open_upvalues`) is disposed of by straight Rust drop-on-reassignment inside `load_live_from`, called from `switch_to_fiber_and_deliver` (`dispatch.rs:352-359`) via `crate::primitive::fiber::load_live_from` (`primitive/fiber.rs:49-59`):
   ```rust
   pub(crate) fn load_live_from(vm: &mut VM, fiber_ref: ObjRef) {
       let fiber = vm.heap.fiber_mut(fiber_ref);
       let frames = std::mem::take(&mut fiber.frames);
       let stack = std::mem::take(&mut fiber.stack);
       let open_upvalues = std::mem::take(&mut fiber.open_upvalues);
       let checking = std::mem::take(&mut fiber.checking);
       vm.frames = frames;
       vm.stack = stack;
       vm.open_upvalues = open_upvalues;
       vm.checking = checking;
   }
   ```
   `vm.open_upvalues = open_upvalues;` **drops the old value of `vm.open_upvalues`** — the failing fiber's `BTreeMap<usize, ObjRef>` of live open-upvalue cells — as an ordinary Rust assignment. No entry is iterated, no `Upvalue::Open` is converted to `Upvalue::Closed`. Nothing is walked. It is dropped in bulk.

**Contrast with `unwind_to`** (`dispatch.rs:88-114`), the non-fiber error path used by `Block::on(_)` (error-handling.md §2) once a handler has decided to catch a `Raise`:
```rust
/// `run_until` deliberately leaves a throwing block's frames and stack
/// live on an `Err` (so the top-level renderer can source-map the full
/// trace on an *uncaught* `throw`); once a matching `on(_)` handler has
/// decided to catch it, those abandoned frames must be torn down before
/// the handler runs. Order matters: **close upvalues first**, then
/// truncate — mirroring [`Bytecode::Return`]/[`Bytecode::ReturnNonLocal`]'s
/// own unwind order exactly (`close_upvalues_from` before
/// `stack.truncate`) — so a closure that escaped the throwing block still
/// observes its captured locals rather than a use-after-free once its
/// stack slot is reclaimed.
pub(crate) fn unwind_to(&mut self, stack_len: usize, frames_len: usize) {
    self.close_upvalues_from(stack_len);
    self.frames.truncate(frames_len);
    self.stack.truncate(stack_len);
}
```
`unwind_to` explicitly *walks* (`close_upvalues_from`, `dispatch.rs:79-86`, promotes every `Open` cell at or above `from` into a heap-owned `Closed(value)` copy) before it ever truncates. Its own doc names the exact hazard — "so a closure that escaped the throwing block still observes its captured locals rather than a use-after-free" — and the fiber-failure path skips precisely that step on the same kind of buffer.

**These are two different machines**, not one. `unwind_to` is reachable from exactly one call site, `primitive/block.rs:274` inside `block_on`, and is not called, directly or indirectly, from the `Err` arm of `run_until` (full trace under Q2). The fiber-failure path's "clear" (`.clear()` on `FiberObject` fields) and "drop-via-reassignment" (`load_live_from`) are a different mechanism entirely, one that never routes through `close_upvalues_from`.

**This is not a documentation gap — it is a live, reproducible crash**, and it already has an open ticket: `docs/errors/E002-fiber-floor-upvalue-crash.md` ("Fiber-floor failure capture drops the live stack without closing open upvalues", status OPEN, confirmed 2026-07-19). I independently reproduced it (own program, `Try` mode instead of E002's `Fiber.abort`, to show it isn't mode-specific):

```phalcom
let leak = None
let f = Fiber.new {
  let x = "captured"
  let blk = { x }
  leak = blk
  throw Error.new("fiber failed")
}
let r = f.try()
System.print("try result class: " + r.class.name)
System.print("calling leaked closure now:")
System.print(leak.call())
```
Verbatim output:
```
try result class: Error
calling leaked closure now:

thread 'main' (323000) panicked at phalcom-core/src/vm/dispatch.rs:1062:61:
index out of bounds: the len is 0 but the index is 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
`dispatch.rs:1058-1064` (`Bytecode::GetUpvalue`) is exactly where this dies: `Upvalue::Open { fiber, slot }` with `fiber != self.current` resolves via `self.heap.fiber(fiber).stack[slot]` — and that `stack` is the `Vec` that was `.clear()`-ed at line 320 and never refilled (the failing fiber never becomes `current` again; it is terminal). OBSERVED, not inferred — I ran it.

---

## Q1 — `Err` arm of `run_until`, `capture_error_value`, `FiberStatus`/`FiberResumeMode`

**`run_until`, full `Err` arm** (`phalcom-core/src/vm/dispatch.rs:290-338`, inside the `base_frames == 0` loop that starts at line 237):
```rust
290                Err(e) => {
291                    // Fiber-floor capture, failure path (spec §3.2, the
292                    // DEC-FIB-A fix): the U-CORE-6 unwind reached the top of
293                    // the current fiber's own activation uncaught. Capture it
294                    // into its result slot instead of propagating past the
295                    // fiber boundary. Under `call`, cascade the same capture
296                    // straight up the resumer chain — without executing any
297                    // of an intermediate `call`-mode resumer's own bytecode,
298                    // exactly as if `e` had been raised at each `call()` site
299                    // in turn with no handler — until a `try`-mode resumer
300                    // (which gets the `Error` delivered as a value instead)
301                    // or the root fiber (which ends the whole run) is
302                    // reached. The host, and every fiber the failure doesn't
303                    // reach, survives.
304                    let error_value = self.capture_error_value(&e);
305                    let mut failed = self.current;
306                    loop {
307                        self.heap.fiber_mut(failed).status = crate::heap::FiberStatus::Failed;
308                        self.heap.fiber_mut(failed).result = error_value;
309                        // Spec §5.1: a `Failed` fiber can never resume, so its
310                        // parked state is pure retention — clear all three
311                        // parked fields here (not just `frames`). The
312                        // originating fiber's own `FiberObject` fields are
313                        // already empty (they were `vm.frames`/`stack`/
314                        // `open_upvalues`, the live mirror, when it raised);
315                        // this matters for an intermediate `Call`-mode
316                        // resumer walked by this cascade, whose fields still
317                        // hold the state it parked when it resumed the fiber
318                        // that ultimately failed.
319                        self.heap.fiber_mut(failed).frames.clear();
320                        self.heap.fiber_mut(failed).stack.clear();
321                        self.heap.fiber_mut(failed).open_upvalues.clear();
322                        let mode = self.heap.fiber(failed).resume_mode;
323                        let Some(resumer) = self.heap.fiber(failed).resumer else {
324                            return Err(e);
325                        };
326                        match mode {
327                            crate::heap::FiberResumeMode::Try => {
328                                self.switch_to_fiber_and_deliver(resumer, error_value);
329                                break;
330                            }
331                            crate::heap::FiberResumeMode::Call => {
332                                failed = resumer;
333                            }
334                        }
335                    }
336                    // Loop again: keep draining, now as the fiber the
337                    // cascade stopped at.
338                }
```

**`capture_error_value`** (`dispatch.rs:361-379`):
```rust
361    /// Extracts the surface `Error` [`Value`] a fiber-floor capture stores in
362    /// [`crate::heap::FiberObject::result`] (ADR-0030 §6, spec §3.2).
363    ///
364    /// [`RuntimeError::Raise`]'s `error` is already the surface instance
365    /// (U-CORE-6); any other terminal error (a native VM error with no
366    /// surface reification) is wrapped in a bare [`crate::heap::Object::Instance`]
367    /// of the kernel `Error` class carrying its rendered message, so a
368    /// fiber's failure is always a catchable `Error` value, never a native
369    /// Rust error type leaking across the fiber boundary.
370    fn capture_error_value(&mut self, e: &PhError) -> Value {
371        if let PhError::Runtime(RuntimeError::Raise { error, .. }) = e {
372            return *error;
373        }
374        let error_class = self.universe.classes.error_class;
375        let field_count = self.heap.class(error_class).field_count;
376        let mut inst = crate::heap::InstanceObject::new(error_class, field_count);
377        inst.slots[0] = self.alloc_string_value(e.to_string());
378        Value::Obj(self.heap.alloc(Object::Instance(inst)))
379    }
```

**`FiberStatus`/`FiberResumeMode`** (`phalcom-core/src/heap/fiber.rs:11-44`):
```rust
11 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
12 pub enum FiberStatus {
13     /// Created but not yet started, or suspended at a `yield` — resumable via
14     /// `Fiber#call`/`Fiber#try`.
15     Suspended,
16     /// Currently executing on the VM: this is the `vm.current` fiber, whose
17     /// live stacks are mirrored in [`VM::frames`](crate::vm::VM)/`stack`.
18     Running,
19     /// The entry function returned normally; [`FiberObject::result`] holds the
20     /// return value and the fiber can no longer be resumed.
21     Done,
22     /// The entry raised an uncaught error; [`FiberObject::result`] holds the
23     /// captured `Error` value (the fiber-floor capture, ADR-0030 §6) and the
24     /// fiber can no longer be resumed.
25     Failed,
26 }
...
36 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
37 pub enum FiberResumeMode {
38     /// Resumed via `Fiber#call`/`call(_:)` — an uncaught failure re-raises
39     /// into the resumer as if it had been raised at the `call` site.
40     Call,
41     /// Resumed via `Fiber#try`/`try(_:)` — a failure is captured and
42     /// delivered as the `Error` value instead of raised.
43     Try,
44 }
```

**Sequence of writes, in my own words:**

*(a) The failing fiber* (`failed = self.current` on entry to the loop): `status ← Failed`, `result ← error_value` (`capture_error_value`'s output — the raw `Value` from a `RuntimeError::Raise`, or a freshly built `Error` instance for anything else). Then `.frames.clear()/.stack.clear()/.open_upvalues.clear()` on its `FiberObject` — a **no-op**, since this fiber's real state lives in `self.frames`/`self.stack`/`self.open_upvalues` (the VM's own fields), not its `FiberObject`, at the moment it fails. Those live fields are never touched by anything in the `Err` arm itself; they are silently overwritten later (dropped, not walked — see Headline) the moment `switch_to_fiber_and_deliver`/`load_live_from` runs for the `Try`-mode delivery, or on the *next* fiber's turn through `run_until`'s own `loop` if `Call`-mode cascades past it.

*(b) Each intermediate `Call`-mode resumer walked by the cascade*: exactly the same two writes (`status ← Failed`, `result ← error_value` — **the same `error_value` object**, unwrapped-and-shared, not re-captured or re-rendered per hop) — but for these fibers the subsequent `.clear()` calls are **not** no-ops: their `FiberObject.frames`/`.stack`/`.open_upvalues` genuinely held the state they parked when they resumed the callee that eventually failed, and that real, non-empty state (including any `Upvalue::Open` cells referencing their own stack slots) is dropped via `Vec::clear()`/`BTreeMap::clear()` — again bulk drop, not a walk, and again no `close_upvalues_from` call anywhere in this loop.

The loop stops (a) at the first `Try`-mode fiber, delivering `error_value` as an ordinary value via `switch_to_fiber_and_deliver`, or (b) at the root fiber (`resumer == None`), which re-raises `e` out of `run_until` entirely, ending the whole program (top-level renderer prints it).

---

## Q2 — REFUTE ASK: "`close_upvalues_from` is never reached from the `Err` arm, direct or indirect"

**All callers of `close_upvalues_from`** (`grep -n "close_upvalues_from" phalcom-core/src/vm/dispatch.rs`):
- `dispatch.rs:79` — the definition itself.
- `dispatch.rs:100` — a doc-comment reference (not a call).
- `dispatch.rs:111` — called from `unwind_to`.
- `dispatch.rs:1091` — `Bytecode::CloseUpvalue(slot)` handler, inside `run_until_inner`'s ordinary dispatch loop (line 477's function, confirmed `fn run_until_inner` at `dispatch.rs:477`, whose own doc says "this function's own behavior is otherwise exactly the pre-U-FIBER `run_until`" — i.e. it is unaware of fibers and unaware of the outer `run_until`'s `Err` arm).
- `dispatch.rs:1103` — `Bytecode::Return` handler, same loop.
- `dispatch.rs:1151` — `Bytecode::ReturnNonLocal` handler, same loop.

**All callers of `unwind_to`** (`grep -rn "unwind_to" phalcom-core/src/`):
- `dispatch.rs:110` — the definition.
- `primitive/block.rs:274` — the **only** call site, inside `block_on` (`Block::on(_)(_)`), and only on the branch where a caught `Raise` matched `isA(class_arg)` — a completely separate primitive from the fiber machinery.

**Tracing every route from the `Err` arm** (`dispatch.rs:290-338`): it calls `self.capture_error_value(&e)` (builds a `Value`, no upvalue interaction, no calls into anything else) and `self.switch_to_fiber_and_deliver(resumer, error_value)` (`dispatch.rs:352-359`), which calls `crate::primitive::fiber::load_live_from` (`primitive/fiber.rs:49-59`) — a pure `mem::take`/assign, no call to `close_upvalues_from` or `unwind_to` anywhere in its body. Nothing else is called in the `Err` arm. I could not construct a route, direct or indirect, from the `Err` arm to `close_upvalues_from`.

**Verdict: VERIFIED-TRUE.** Confirmed both by static trace (above) and empirically — the crash reproduced under Headline (`index out of bounds: the len is 0 but the index is 1` at `dispatch.rs:1062`) is exactly the symptom that would not exist if `close_upvalues_from` ran anywhere on this path. This is not a new finding — it matches `docs/errors/E002-fiber-floor-upvalue-crash.md` (status OPEN at HEAD) almost exactly; my repro used `Try` mode + `throw` instead of E002's `Fiber.abort`, to check the bug isn't mode- or API-specific, and it isn't.

---

## Q3 — REFUTE ASK: E001 is fixed at HEAD

**Read `vm/gc.rs`.** `push_temp_root` (`gc.rs:148-152`), `temp_root_depth` (`gc.rs:161-163`), `truncate_temp_roots` (`gc.rs:169-171`) all exist and are wired into `collect_roots` (`gc.rs:32-116`, `temp_roots` field destructured at line 53 and enumerated at line 108: `out.extend(temp_roots.iter().copied());`). `git show cdd2117 --stat` confirms these landed in exactly this commit (`fix(vm): commit temp_roots GC escape-hatch paired with vm/mod.rs`, touching `primitive/block.rs`, `vm/bootstrap.rs`, `vm/gc.rs`) — the commit's own message says the *previous* commit had already staged `vm/mod.rs`'s `temp_roots` field as a side effect of an unrelated change, and this commit is "its required pair," restoring buildability.

**Read `primitive/block.rs::block_ensure`** (`block.rs:303-340`):
```rust
303 pub fn block_ensure(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
304     let cleanup = args[0];
305     let outcome = block_call(vm, receiver, &[]);
306
307     // The pending outcome lives only in this Rust local while the cleanup block
308     // runs, and that block re-enters the interpreter — so its back-edge
309     // safepoint can collect. Neither `vm.stack` nor `vm.frames` describes
310     // `outcome`, so without a temp root the collector frees it and `ensure`
311     // returns a dangling handle (ADR-0050 §7; memory-management.md §4).
312     //
313     // Both arms carry a handle: `Ok` is the protected block's value, and a
314     // `Raise` error is the surface `Error` instance an enclosing `on` will
315     // receive.
316     let roots = vm.temp_root_depth();
317     match &outcome {
318         Ok(value) => vm.push_temp_root(*value),
319         Err(PhError::Runtime(RuntimeError::Raise { error, .. })) => vm.push_temp_root(*error),
320         Err(_) => {}
321     }
322
323     let frames_before_cleanup = vm.frames.len();
324     let cleanup_outcome = block_call(vm, &cleanup, &[]);
325     vm.truncate_temp_roots(roots);
326
327     match cleanup_outcome {
328         Err(cleanup_err) => Err(cleanup_err),
329         Ok(cleanup_value) => {
330             if vm.frames.len() < frames_before_cleanup {
331                 Ok(cleanup_value)
332             } else {
333                 outcome
334             }
335         }
336     }
337 }
```
Both the `Ok` path and the `Raise` (error-carrying) path push a temp root before the re-entrant cleanup call, and `truncate_temp_roots(roots)` runs unconditionally right after `block_call(vm, &cleanup, &[])` returns — **before** the `match cleanup_outcome` that decides what to return, so there is no branch (`Err(cleanup_err)`, non-shrunk `Ok`, or shrunk `Ok`/non-local-return) that skips the truncate.

**Ran every repro + control from E001, `var` rewritten to `let`:**
```
--- A (explicit collect) ---
freshstring
--- C (allocating cleanup, no explicit gc) ---
freshstring
--- control (no collection) ---
freshstring
```
All three now pass cleanly — no panic, correct value. (E001 documented these as panicking pre-fix; that panic no longer reproduces.)

**Error-carrying path** (E001 flagged as "plausible, not yet independently gated") — I gated it myself:
```phalcom
let r = { { throw Error.new("boom") }.ensure { System.gc } }.on(Error) { e => e.message }
System.print(r)
```
Output: `boom`, exit 0. The `Raise`-carrying `Error` instance survives the `System.gc` inside the cleanup and is delivered to `.on(Error)` with its correct `message` — the error-carrying branch (`block.rs:319`) is exercised and correct, not merely "not crashing."

**Trying to break the fix — pop discipline under nesting:**
```phalcom
let r = { "outer" + "val" }.ensure {
  let x = { "clean-inner" + "val" }.ensure { System.gc }
  System.print(x)
  System.gc
}
System.print(r)
```
Output:
```
clean-innerval
outerval
```
Both the outer and inner protected-block results survive two separate GC triggers under nesting, and each is truncated back to the correct depth: the depth-and-truncate design (`gc.rs:154-163`, "Depth-and-truncate rather than push-and-pop … correct on all of them without the caller counting its own pushes") behaves exactly as documented — LIFO, no leak, no over-truncation. I found no early-return path in `block_ensure` that skips `truncate_temp_roots` (it runs unconditionally before the outcome match, not gated behind `?` or any early branch).

**Verdict: VERIFIED-FIXED.** All of E001's repros (A, C, control) and the error-carrying path pass; the fix is a real `temp_roots` mechanism (not just the value-path special case E001 worried might be all that shipped), and I could not construct a leak or an over-pop under nesting.

**Unexplored lead (flagged, not chased — would be a 7th area):** `block_on` (`primitive/block.rs:239-281`) computes `error` (sometimes a **freshly allocated** `Object::Instance`, `block.rs:259-261`) and then calls `vm.send_dynamic(error, isa_sym, &[class_arg])?` (`block.rs:272`) — a re-entrant call — **without** pushing a temp root on `error` first. This looks like the same hazard class E001 was patched for, in a sibling primitive, and E001's own "Doc debt" section explicitly names `.on(_)` as a primitive that "may share it." I did not reproduce or confirm this; I only noticed it while reading the fix. Flagging per the budget rule rather than opening a full audit.

---

## Q4 — REFUTE ASK: the cascade's "no intermediate bytecode runs" behaviour is unobservable because the guard always fires first

**The guard**, `fiber_resume` (`primitive/fiber.rs:247-250`):
```rust
fn fiber_resume(vm: &mut VM, receiver: &Value, args: &[Value], mode: FiberResumeMode) -> PhResult<Value> {
    if vm.native_reentry_depth != 0 {
        return Err(cannot_resume_across_native_frame(vm));
    }
```
`ensure`/`on` both compile down to `Block::ensure(_)`/`Block::on(_)(_)`, which run their protected block via `block_call` — a native re-entrant call that increments `native_reentry_depth` for its duration (the whole point of the restricted-yield guard, `fiber.rs:1-13`'s module doc: "Every switch is legal only when `VM::native_reentry_depth` is `0`"). So a fiber resume (`.call()`/`.try()`) attempted from *inside* an active `ensure`/`on` protected block is rejected by this guard **before any switch happens** — the callee fiber never starts, so it can never later fail "underneath" that handler.

**Tried to break it — case 1: does an intermediate resumer's own bytecode run at all past a `.call()` that fails deeper down?**
```phalcom
let sideEffectRan = false
let c = Fiber.new { throw Error.new("c failed") }
let b = Fiber.new {
  c.call()
  sideEffectRan = true
}
let outcome = b.try()
System.print(sideEffectRan)
System.print(b.isDone)
System.print(outcome.message)
```
Output:
```
false
true
c failed
```
`sideEffectRan` never flips — confirms no bytecode after the failing `.call()` runs in the intermediate resumer. (This much is unsurprising and true of ordinary exception propagation too, fiber or not — it doesn't by itself demonstrate that a *cleanup handler* is skipped.)

**Tried harder — case 2: wrap the resuming call itself in an active `ensure`, to see whether the cascade can be made to skip it:**
```phalcom
let cleanupRan = false
let c = Fiber.new { throw Error.new("c failed") }
let b = Fiber.new {
  { c.call() }.ensure { cleanupRan = true }
}
let outcome = b.try()
System.print(cleanupRan)
System.print(outcome.class.name)
System.print(outcome.message)
```
Output:
```
true
CannotYieldAcrossNativeFrame
cannot resume a fiber across a native call frame (e.g. inside .each { })
```
`cleanupRan` is **true** — but not because the cascade ran `b`'s cleanup. `c.call()` never even starts: `b`'s own `.ensure`'s `block_call` had already bumped `native_reentry_depth` before running the protected block, so `c.call()` is rejected synchronously by the guard quoted above. `b`'s `ensure` then runs its cleanup as an *ordinary*, synchronous, single-fiber error (a `CannotYieldAcrossNativeFrame` instance) — this never touches the cascade machinery in the `Err` arm's cascade loop at all (it's a ordinary in-fiber raise/catch, one hop, no `resumer` walk). I could not construct a program where a genuine `Call`-mode cascade (the loop at `dispatch.rs:306-335` iterating more than zero times) walks *through* a fiber with a still-pending `ensure`/`on`, because setting that up requires the exact call the guard forbids.

**Verdict: VERIFIED-TRUE**, for the reason the claim gives: every cleanup-installing construct is a native re-entrant frame, so the guard (`primitive/fiber.rs:248-250`) always fires before a resume that could later cascade past an active handler. What *is* observable (case 1) is unremarkable — ordinary skip-on-error — not the distinguishing "cleanup handler specifically skipped" behavior the claim is about.

---

## Q5 — `Fiber.abort(_)` vs `throw`

**`Fiber.abort(_)`** (`primitive/fiber.rs:199-216`):
```rust
199 /// Signature: `Fiber::abort(_)` — raises `args[0]` at the fiber floor, caught
200 /// by `VM::run_until`'s fiber-floor capture exactly like any other raise.
...
208 pub fn fiber_abort(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
209     let me = vm.current;
210     if vm.heap.fiber(me).resumer.is_none() {
211         return Err(RuntimeError::NotAllowed("cannot abort the root fiber".to_string()).into());
212     }
213     let error = args[0];
214     let rendered = error.to_string(vm);
215     Err(RuntimeError::Raise { error, rendered }.into())
216 }
```
No type check whatsoever on `args[0]` — any `Value` is raised as-is.

**`throw`** desugars to `expr.raise()` (ADR-0031 §1), landing in `error_raise` (`primitive/error.rs:61-65`):
```rust
pub fn error_raise(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let message_sym = vm.get_or_intern("message");
    let rendered = vm.send_dynamic(*receiver, message_sym, &[])?.to_string(vm);
    Err(RuntimeError::Raise { error: *receiver, rendered }.into())
}
```
`raise` is installed **only on `Error`** (`primitive/error.rs:44-46`: "Only `Error` and its subclasses respond to `raise` … a non-`Error` receiver misses (dNU → `MessageNotUnderstood`)"). The compiler additionally rejects a literal non-`Error` operand to `throw` at compile time (observed, see below) — but only for literals; a non-literal expression is not statically checked and reaches `.raise()` as an ordinary dynamic send.

**Same route?** Yes: both terminate in `PhError::Runtime(RuntimeError::Raise { error, rendered })`, which `capture_error_value` (`dispatch.rs:370-373`) special-cases identically: `return *error` — no wrapping, no distinction between the two call sites. The fiber-floor capture cannot tell `Fiber.abort` and `throw` apart; they are the same payload shape at that point.

**Do they produce the same result-value shape for a non-`Error` argument?** No — they diverge *before* reaching that shared route:

```phalcom
let fa = Fiber.new { Fiber.abort(123) }
System.print(fa.try())

let n = 123
let ft = Fiber.new { throw n }
System.print(ft.try())
```
Output:
```
123
<MessageNotUnderstood>
```
`Fiber.abort(123)` delivers the raw `123` verbatim (matching the named fixtures below). `throw n` (non-literal, so not compile-rejected) compiles to `n.raise()`; `Number` doesn't understand `raise`, so it dNUs, and the *dNU's own* `MessageNotUnderstood` instance — not `123` — is what reaches the fiber floor. (`throw 123` as a literal is rejected outright at compile time: `Error: `throw` of a non-`Error` literal is a compile error; only `Error` subclasses are throwable.` — observed directly, not inferred.)

**Named Wren-ported fixtures for `Fiber.abort` + non-`Error` values** (found via directory listing, not fully censused): `concurrency_fiber_wren_abort_number_captured.ph` ("Ported from Wren `test/core/fiber/abort_not_string.wren`: `Fiber.abort(_)` accepts any value, not only strings — `try()` hands back the raw `123`.", `.expected` = `123`) and `concurrency_fiber_wren_abort_string_captured.ph` ("Ported from Wren `test/core/fiber/abort.wren` … `try()` captures it and hands back the exact value passed to `abort`, unwrapped, not wrapped in an `Error` instance.", `.expected` = `Error message.`). Both match my program's output exactly.

---

## Q6 — Named fixtures + the multi-fiber cascade question

**`concurrency_fiber_is_done_and_error_once_failed.ph`** (`.expected`: `None` / `true` / `true`): asserts `Fiber#isDone` is `false`-then-`true` around a failure and `Fiber#error` is `Some(e)` wrapping the **identical** captured `Error` instance `try()` already delivered — checked via `Object#==` identity through `match`, not a re-wrapped copy. Single fiber, no cascade (`f.error` read before `try()`, then after).

**`concurrency_fiber_abort_then_resume_fails.ph`** (`.expected`: `Error` / `Error` / `cannot resume a finished fiber`): two parts. Part 1 — `f.try()` on a fiber that `Fiber.abort`s: ordinary single-hop capture, `r1.class.name == "Error"`. Part 2 — a *second*, different fiber `driver` calls `f.call()` where `f` is already `Failed`: this hits the **ordinary** "cannot resume a finished fiber" `NotAllowed` guard in `fiber_resume`'s status match (`primitive/fiber.rs:252-260`, not the fiber-floor `Err`-arm cascade), which then propagates as `driver`'s own single-fiber failure, caught by `driver.try()` at the top level. Despite involving two `Fiber.new`s and a `.call()`, this is **not** a live cascade through the fiber-floor capture — it demonstrates `Failed`/`Done` share one terminal-resume guard, nothing about the cascade mechanism itself.

**`concurrency_fiber_try_abort_current.ph`** (`.expected`: `Error` / `true`): single fiber `failing` aborts, captured by `.try()` directly (`result.class.name == "Error"`); a second, unrelated fiber `g` checks `Fiber.current == g` from inside itself. No interaction between the two fibers — no cascade.

**`concurrency_fiber_wren_try_value_error_capture.ph`** (`.expected`: `before` / `value` / `MessageNotUnderstood` / `true does not understand 'unknownMethod'` / `after try`): single fiber, ported from Wren `try_value.wren` — delivers the first-resume argument (`"value"`) at entry, then fails partway through via a genuine dNU (`true.unknownMethod`), captured directly by the caller's `try()`. No second fiber, no cascade; demonstrates the caller can keep running normally ("after try") post-capture.

**Is there any fixture, anywhere in the tree, exercising a genuine multi-fiber `Call`-mode failure cascade (a fiber failing while resumed by a fiber that was itself resumed)?**

I checked every `concurrency/*.ph` file combining `Fiber.new` with `.call()` (16 candidates) and specifically inspected the two structurally closest to a nested-resume shape: `concurrency_fiber_nested_current_identity.ph` (an `outer` fiber drives an `inner` fiber via `.call()`/`.yield()` — but neither fiber ever fails; both complete normally, testing `Fiber.current` identity only) and `concurrency_sched_raising_fiber_does_not_abort_host.ph` (a fiber scheduled via `System.schedule` raises uncaught, captured by the root-drive pump's own direct `fiber_try` call — a single hop from the failing fiber straight to the root, not through an intermediate resumer fiber).

**No fixture in the tree drives a three-(or-more)-fiber chain where the middle fiber resumed the failing one in `Call` mode and is itself resumed by another fiber** — the exact shape the `Err` arm's cascade `loop` (`dispatch.rs:306-335`, the `FiberResumeMode::Call => { failed = resumer; }` branch iterating more than once) is written to handle. This is a genuine coverage gap, not merely an unnamed test: every existing failure fixture is a single hop (fiber fails → its one direct resumer captures via `try`), and the one program that superficially looks like a chain (`concurrency_fiber_abort_then_resume_fails.ph`) resolves through the ordinary "already-finished" guard instead of the cascade loop.
