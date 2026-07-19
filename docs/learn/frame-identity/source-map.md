# Frame identity and non-local return — source map

Read-only research pass at HEAD. Anchors are `file.rs::Symbol` (`~Lnn`); line
numbers rot, symbols don't. Every claim below is tagged **VERIFIED** (read the
line, or ran the program and matched output) or **INFERRED** (reasoned from
verified pieces but not captured in a single runtime trace).

Method note up front: `graphify query`/`explain`/`affected` were run three
times against this topic (`"frame identity FrameToken generation non-local
return"`, `explain "FrameToken"`, `affected "next_frame_generation"`) before
any raw read. All three came back weak or empty for the load-bearing symbols
(`next_frame_generation`, `FrameToken::{frame_index,generation}`,
`home_frame_token`, `BlockObject`, the fiber swap functions) — confirming the
task brief's own expectation that the graph is weak here. Everything after
that point is targeted `Read`/`grep`, per the brief's explicit fallback
permission.

---

## THE DOMINATING QUESTION

**Claim:** `FrameToken { frame_index, generation }`'s two halves have
different scopes, and it's deliberate.

**CONFIRMED.**

`VM::frames` is swapped per-fiber. `phalcom-core/src/primitive/fiber.rs::store_live_into` (~L29-43):

```rust
pub(crate) fn store_live_into(vm: &mut VM, fiber_ref: ObjRef) {
    let frames = std::mem::take(&mut vm.frames);
    let stack = std::mem::take(&mut vm.stack);
    let open_upvalues = std::mem::take(&mut vm.open_upvalues);
    let checking = std::mem::take(&mut vm.checking);
    let fiber = vm.heap.fiber_mut(fiber_ref);
    fiber.frames = frames;
    fiber.stack = stack;
    fiber.open_upvalues = open_upvalues;
    fiber.checking = checking;
}
```

`load_live_from` (~L49-59) is the exact mirror. Only four fields move:
`frames`, `stack`, `open_upvalues`, `checking`. **`next_frame_generation` is
not among them.**

`VM::next_frame_generation` (`phalcom-core/src/vm/mod.rs` ~L109, doc: "Monotonically-assigned
generation counter for frame tokens.") is never swapped anywhere. **Exhaustive
grep for every write site** (the "decisive negative check" the brief asked
for):

```
phalcom-core/src/interpret.rs:171   frame.generation = self.next_frame_generation;
phalcom-core/src/interpret.rs:172   self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
phalcom-core/src/vm/dispatch.rs:37  let generation = self.next_frame_generation;
phalcom-core/src/vm/dispatch.rs:38  self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
phalcom-core/src/vm/bootstrap.rs:38 next_frame_generation: 0,   (VM::new init)
phalcom-core/src/vm/mod.rs:109      pub(crate) next_frame_generation: u64,  (declaration)
phalcom-core/src/vm/gc.rs:83        next_frame_generation: _,  (GC root destructure — a compile-time "did you forget me" guard, not a write)
```

That is every occurrence of the identifier in `phalcom-core/src`. None of them
is inside `store_live_into`/`load_live_from`, `fiber_call`/`fiber_try`/
`fiber_yield`, or any other fiber-switch path. **Absence confirmed —
`next_frame_generation` is never swapped, parked, or reset per-fiber.**

So: `frame_index` is meaningful only relative to whichever fiber's `frames`
Vec is currently mounted at `VM::frames` — the same index means a different
activation depending on which fiber owns the buffer at read time. `generation`
is a single counter shared by the whole VM across every fiber that has ever
existed, monotonically increasing, never reused, never reset.

**Deliberate, not emergent** — named explicitly in
`docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md` §6
("Non-local return and unwind stay fiber-local"), ~L107-117:

> Once `self.frames` is the *current* fiber's vector, [ADR-0013]'s
> `ReturnNonLocal` searches only that fiber; a token whose home is on another
> fiber fails the generation check → `DeadFrameError`. **Invariant:** the
> VM-global monotonic `next_frame_generation` counter **must not** be
> relocated into `FiberObject` — it is the only thing making a cross-fiber
> token globally non-matching.

And the ADR's own motivating section (~L36-38): the pre-fiber audit
"surfaced invariants (e.g. the VM-global frame generation counter) that a
pre-fiber refactor must not break." This is a load-bearing, named design
invariant, not something that fell out unnamed. The golden
`tests/lang/concurrency/negative/fiber_cross_fiber_non_local_return_dead_frame.ph`
and the Rust unit test `cross_fiber_non_local_return_raises_dead_frame_error`
(`phalcom-core/tests/invariants.rs` ~L1249) exist specifically to pin this
behavior. **VERIFIED** by reading the ADR text and the swap functions above.

---

## 1. The data structures

`phalcom-core/src/frame.rs::FrameToken` (~L19-24), in full:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameToken {
    /// The VM frame index the token refers to.
    pub frame_index: usize,
    /// The generation associated with that frame activation.
    pub generation: u64,
}
```

`CallFrame`'s identity fields (~L66-95), full struct with the load-bearing
doc on `home_frame_token`:

```rust
pub struct CallFrame {
    pub closure: ObjRef,
    pub context: CallContext,
    pub ip: usize,
    pub stack_offset: usize,
    pub caller_source: Option<SourceRange>,
    /// Monotonically-assigned generation for this activation.
    pub generation: u64,
    /// The home-frame token this activation returns *through* on a non-local
    /// `return`, or `None` for an ordinary method/closure activation.
    ///
    /// Populated **only** when the frame is a block invocation: [`block_call`]
    /// copies it from the invoked [`BlockObject`]'s `home_frame_token`, so the
    /// [`Bytecode::ReturnNonLocal`] handler can read "what activation am I
    /// unwinding to" straight off the currently-executing frame (the
    /// `BlockObject` that carried the token is not otherwise reachable from a
    /// live `CallFrame`, which only stores the `ClosureObject` handle).
    /// Ordinary method/closure calls leave this `None`; their `return`
    /// compiles to [`Bytecode::Return`] and never reads it (ADR-0013,
    /// blocks.md §5). Because [`FrameToken`] is `Copy`, `Option<FrameToken>`
    /// keeps [`CallFrame`] `Copy`.
    pub home_frame_token: Option<FrameToken>,
}
```

`CallFrame::new` (~L100-110) leaves `generation: 0, home_frame_token: None` —
the constructor itself never stamps a real generation; every caller must do
that explicitly (see §3 below for whether they all do).

`phalcom-core/src/heap/block.rs::BlockObject` (~L18-23), in full:

```rust
pub struct BlockObject {
    /// Handle to the underlying [`ClosureObject`].
    pub closure: ObjRef,
    /// The frame token of the activation in which this block was created.
    pub home_frame_token: FrameToken,
}
```

Note `BlockObject::home_frame_token` is a bare `FrameToken` (not
`Option<FrameToken>`) — every block, by construction, has a home. It is
`CallFrame::home_frame_token` that is `Option`, because only a block
*activation* (not an ordinary method activation) carries one. **VERIFIED**
(all four read in full).

---

## 2. The four lifecycle points

**Mint** — `phalcom-core/src/vm/dispatch.rs::VM::new_call_frame` (~L29-42):

```rust
pub(crate) fn new_call_frame(&mut self, closure: ObjRef, context: CallContext,
    ip: usize, stack_offset: usize, caller_source: Option<SourceRange>) -> CallFrame {
    let generation = self.next_frame_generation;
    self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
    let mut frame = CallFrame::new(closure, context, ip, stack_offset, caller_source);
    frame.generation = generation;
    frame
}
```

Every pushed activation is supposed to go through here (or duplicate its
logic — see §3) so it gets its own generation.

**Stamp** — `Bytecode::Closure` handler, `vm/dispatch.rs` (~L577-605), and
`VM::current_frame_token` (~L50-52):

```rust
fn current_frame_token(&self) -> Option<crate::frame::FrameToken> {
    self.frames.last().map(|frame| frame.token(self.frames.len() - 1))
}
```

```rust
Bytecode::Closure(idx) => {
    // Materialize a fresh instance whose upvalue cells are captured...
    ...
    let new_closure = self.heap.alloc(Object::Closure(Box::new(ClosureObject { callable, module, upvalues })));
    let token = self.current_frame_token().expect("closure created inside a frame");
    let block = self.heap.alloc(Object::Block(BlockObject::new(new_closure, token)));
    self.stack.push(Value::Obj(block));
}
```

`frame_index` here is computed as `self.frames.len() - 1` at the *instant the
block literal executes* — i.e., the index of whatever frame is innermost
right then. This is why the home frame recorded is always the *lexically
enclosing activation that was running when the `{ ... }` literal was
evaluated*, not necessarily the outermost method — if a block literal is
itself created inside another block's activation, its home token points at
that inner activation, not transitively further out. **VERIFIED**.

**Carry** — `phalcom-core/src/primitive/block.rs::block_call` (~L125,~L143-152):

```rust
let (closure_id, home_frame_token) = resolve_callable(vm, receiver)?;
...
let mut frame = vm.new_call_frame(closure_id, context, 0, stack_offset, None);
// Stamp the block activation with its lexical home frame...
frame.home_frame_token = home_frame_token;
vm.frames.push(frame);
```

`primitive/fiber.rs` fiber-entry path (~L272-273, ~L298-306) — carrying a
token across onto a **different fiber's** frame stack, verbatim, unchanged:

```rust
let (closure_id, home_frame_token) = match vm.heap.get(entry) {
    Object::Block(block) => (block.closure, Some(block.home_frame_token)),
    Object::Closure(_) => (entry, None),
    _ => unreachable!("fiber_new only accepts Block/Closure entries"),
};
...
let mut frame = vm.new_call_frame(closure_id, CallContext::Instance { instance: entry }, 0, stack_offset, None);
frame.home_frame_token = home_frame_token;
vm.frames.push(frame);
```

This is the literal mechanism behind the cross-fiber `DeadFrameError` golden:
a block's `home_frame_token` was minted on fiber A's now-defunct frame stack;
it is carried, byte-for-byte, into a fresh `CallFrame` on fiber B's `vm.frames`
(post-swap, so `vm.frames` now IS fiber B's Vec); the token's `frame_index`
means nothing there, but the `generation` — being VM-global — still correctly
fails to match. **VERIFIED**.

**Check + unwind** — `Bytecode::ReturnNonLocal`, `vm/dispatch.rs` (~L1110-1161),
essentially in full:

```rust
Bytecode::ReturnNonLocal => {
    let token = self.frames.last()
        .and_then(|frame| frame.home_frame_token)
        .ok_or_else(|| RuntimeError::Internal("ReturnNonLocal in a frame with no home-frame token".to_string()))?;

    // Locate the still-live home frame by (index, generation).
    let is_live = self.frames.get(token.frame_index)
        .is_some_and(|home| home.generation == token.generation);
    if !is_live {
        return Err(RuntimeError::DeadFrameError.into());
    }
    let home_stack_offset = self.frames[token.frame_index].stack_offset;

    let return_value = self.stack.pop().unwrap_or(Value::Nil);
    let return_value = self.surface_absence(return_value);

    self.close_upvalues_from(home_stack_offset);
    self.stack.truncate(home_stack_offset);
    self.stack.push(return_value);
    // Remove the home frame and everything above it. ... Do NOT
    // `return Ok(_)` here — let the loop continue.
    self.frames.truncate(token.frame_index);
}
```

**VERIFIED** — read verbatim.

---

## 3. The duplicated stamp

**Confirmed.** `interpret.rs::run_in_module` (~L163-176) open-codes the
mint instead of calling `new_call_frame`:

```rust
pub fn run_in_module(&mut self, module: ObjRef, closure: ObjRef) -> PhResult<()> {
    ...
    self.frames.clear();
    self.stack.clear();

    let mut frame = CallFrame::new(closure, CallContext::Module { module }, 0, 0, None);
    frame.generation = self.next_frame_generation;
    self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
    self.frames.push(frame);
    self.run()?;
    Ok(())
}
```

Read/bump/stamp logic is byte-for-byte identical to `new_call_frame`'s body,
just inlined rather than called. **Duplicated, not omitted** — no bug here,
just a missed refactor (this is the outermost program-entry path, called
once per top-level run, so it predates or was never routed through the
helper).

`interpret.rs::import_module`'s frame push (~L265-268) **does** go through
`new_call_frame`:

```rust
let base_frames = self.frames.len();
let stack_offset = self.stack.len();
let frame = self.new_call_frame(closure, CallContext::Module { module: module_id }, 0, stack_offset, None);
self.frames.push(frame);
```

**Exhaustive check for any frame left at the `CallFrame::new` default
(`generation: 0`, i.e. never re-stamped):** grepped every `CallFrame::new(`
and every `frames.push(`/`.frames.push(` site in `phalcom-core/src`:

```
interpret.rs:170   CallFrame::new(...)         -> interpret.rs:171-172 manually stamps (run_in_module)
interpret.rs:267   self.new_call_frame(...)    -> import_module
primitive/block.rs:143   vm.new_call_frame(...) -> block_call
primitive/fiber.rs:304   vm.new_call_frame(...) -> fiber entry
vm/send.rs:113     self.new_call_frame(...)    -> ordinary Invoke → Closure dispatch
vm/dispatch.rs:39  CallFrame::new(...)          -> inside new_call_frame itself (immediately re-stamped, line 40)
```

Every site either calls `new_call_frame` or (in the single `run_in_module`
case) manually duplicates its exact stamping logic. **No frame is ever
pushed and left at the constructor's `generation: 0` default** — that would
have been a real finding; it does not exist at HEAD. **VERIFIED** (exhaustive
grep, all six sites read).

---

## 4. `wrapping_add` / exhaustion

**None at HEAD.** No guard, comment, test, or ADR text discusses
`next_frame_generation` wraparound. It is a bare `u64` incremented by
`wrapping_add(1)` with no overflow check, no saturating behavior, and no
documented mitigation. Stating this plainly per the brief's instruction, not
editorializing further.

---

## 5. The ADR's forward claim — shipped, partial, or aspirational

ADR-0013 Consequences: *"The frame token also unifies with `throw` and fiber
`abort` as one stack-unwinding primitive."*

**Partial.** What exists at HEAD:

- `VM::unwind_to` (`vm/dispatch.rs` ~L110-114) is a **shared low-level
  primitive** used by both non-local-return-adjacent code and `throw`/`on`
  recovery:

  ```rust
  pub(crate) fn unwind_to(&mut self, stack_len: usize, frames_len: usize) {
      self.close_upvalues_from(stack_len);
      self.frames.truncate(frames_len);
      self.stack.truncate(stack_len);
  }
  ```

  Its doc explicitly says it "mirror[s] `Bytecode::Return`/
  `Bytecode::ReturnNonLocal`'s own unwind order exactly." Its only caller is
  `primitive/block.rs::block_on` (~L274), for restoring VM state before
  running a caught-`throw` handler. So `unwind_to` **is** a second unwinding
  primitive that shares the same truncate-order discipline as
  `ReturnNonLocal` — but it takes raw lengths, not a `FrameToken`; it is not
  literally the *same* mechanism, just the same *shape*.
- `block_on` and `block_ensure` (`primitive/block.rs` ~L239-323) both treat a
  non-local return as a first-class outcome alongside `throw`/normal
  completion by comparing `vm.frames.len()` before/after — i.e., they are
  aware of and coordinate with `ReturnNonLocal`'s frame-truncation, but this
  is a `frames.len()` comparison, not a shared `FrameToken`-typed unwind call.
- There is **no evidence of fiber `abort`** using `FrameToken` or
  `unwind_to` at all in the current source — `abort` as a concept doesn't
  appear as a fiber primitive at HEAD; fiber failure instead goes through the
  `run_until` top-level `Err` branch (`vm/dispatch.rs` ~L290-338), which
  captures the error into `FiberObject::result` and cascades to a resumer —
  a related but textually separate mechanism, not routed through
  `FrameToken`/`unwind_to`.

So: `throw`'s catch path (`on`) and non-local return share a *disciplined
unwind order* and a real shared helper (`unwind_to`), but they are not
literally "one primitive" keyed on `FrameToken`, and fiber-`abort`-as-token-
unwind does not exist. **Report: partial, leaning aspirational on the fiber
side.**

---

## 6. `DeadFrameError` through a native window

Three source sites (plus the actual raise site already quoted in §2):

- `error.rs::RuntimeError::DeadFrameError` (~L138-151, full doc + variant):
  states the mechanism precisely — "the `ReturnNonLocal` handler compares the
  executing block frame's `home_frame_token` against the live frame stack,
  and raises this variant when no frame matches the token's `(frame_index,
  generation)`... Detail beyond the fixed message is intentionally omitted."
- `vm/send.rs::call_method`'s `Primitive` arm (~L68-92) — the frame-count
  guard for when a primitive (e.g. `block_call`, called from `f.call(x)`)
  ran a block whose `return` unwound past the primitive's own call site:

  ```rust
  } else if self.frames.len() >= frames_before {
      // Ordinary primitive return...
  } else {
      // A `Bytecode::ReturnNonLocal` fired *inside* `native_fn` ...
      // `result` must be re-pushed here to re-establish it for the outer
      // frame that resumes next.
      self.stack.push(result);
  }
  ```

  This does not itself raise or catch `DeadFrameError` — it is the "did a
  non-local return happen underneath this native call" bookkeeping that keeps
  the stack consistent whether the unwind succeeded *or* (indirectly, since a
  `DeadFrameError` is a plain `Err` that just propagates through `result.map`
  without reaching this arm at all) failed.
- `vm/send.rs::invoke_method_object`'s doc (~L256-258) documents that this
  entry point (behind `Method#invokeOn`/`bound.call`) can propagate a
  `DeadFrameError` from an escaping block's non-local return, same as an
  ordinary send — proven by the Rust unit test
  `invoke_on_preserves_dead_frame_fencing_for_escaping_blocks`
  (`phalcom-core/tests/invariants.rs` ~L1218-1246), which asserts
  `Err(PhError::Runtime(RuntimeError::DeadFrameError))` from exactly this
  path.
- `primitive/block.rs::block_on`'s doc (~L226-227) lists `DeadFrameError` as
  one of the outcomes `on` does **not** treat as a catchable `throw` — "`on`
  catches only `Raise`; re-propagated unchanged." **This claim is
  contradicted by the actual code, see below — a real, verified
  discrepancy.**

**Is `DeadFrameError` catchable by a user-level handler? Ran the program —
answer: YES, and this contradicts the source's own doc comment.**

Wrote and ran (via `cargo run -p phalcom-core --bin phalcom --`) two `.ph`
files in the scratchpad, using the real catch syntax confirmed from
`phalcom-core/tests/lang/errors/errors_try_on_typed_and_catch_all.ph`
(`try { } on Type e { } catch e { }`, per ADR-0031 — `catch` desugars to
`.on(Error)`):

**Test 1** — `/private/tmp/.../scratchpad/dead_frame_catch_test.ph`:

```phalcom
class Maker {
  make() { return { return 1 } }
}
let escaped = Maker.new().make()

try {
  System.print(escaped.call())
} catch e {
  System.print("caught: " + e.message)
}
System.print("after")
```

Actual output (`cargo run -q -p phalcom-core --bin phalcom -- <file>`):

```
caught: non-local return from a block whose home method frame is no longer alive (DeadFrameError)
after
```

Exit code 0. **The catch-all handler caught it.**

Why: `block_on`'s actual `Err(err)` arm (`primitive/block.rs` ~L253-263)
wraps **any** `Err`, not just `Raise`, into a generic `Error` instance when it
isn't already a `Raise`:

```rust
Err(err) => {
    let error = match &err {
        PhError::Runtime(RuntimeError::Raise { error, .. }) => *error,
        _ => {
            let error_class = vm.universe.classes.error_class;
            let field_count = vm.heap.class(error_class).field_count;
            let mut inst = crate::heap::InstanceObject::new(error_class, field_count);
            inst.slots[0] = vm.alloc_string_value(err.to_string());
            Value::Obj(vm.heap.alloc(crate::heap::Object::Instance(inst)))
        }
    };
    ...
    let matched = vm.send_dynamic(error, isa_sym, &[class_arg])?;
    if matches!(matched, Value::Bool(true)) { ... catch ... } else { Err(err) }
}
```

`DeadFrameError` is a plain `RuntimeError`, not a `Raise`, so it falls into
the wildcard arm, gets wrapped as a bare kernel `Error` instance, and a
catch-all `catch e { }` (which desugars to `.on(Error)`) always matches
`isA(Error)` on it. **The code catches it; the rustdoc directly above it
claims it doesn't.** This is a verified doc/code mismatch, not a bug in
behavior per se — but the doc comment at `primitive/block.rs` ~L226-227 is
stale/wrong and should not be repeated as fact in the new doc.

**Test 2** (control — does a *non-matching typed* handler let it through
uncaught, as the doc's "any other Err... re-propagated unchanged" would
predict for a mismatch case?) —
`/private/tmp/.../scratchpad/dead_frame_typed_no_match.ph`, same setup but
`on ParseError e { ... }` (a class that isn't `Error`'s root). Actual output:
non-local return from a block whose home method frame is no longer alive (DeadFrameError)` printed to stderr, exit code 1 — **uncaught**, confirming the
`isA` check is what gates catching, not a `Raise`-vs-other-error distinction.
So the real rule is: **any wrapped error, including `DeadFrameError`, is
catchable by any handler whose class matches via `isA`** — `Error`/`catch`
always matches; a narrower typed class only matches if `DeadFrameError`'s
synthetic wrapper class (`vm.universe.classes.error_class`, i.e. plain kernel
`Error`) `isA` that narrower class, which it normally won't unless the
handler's class *is* `Error`.

Commands run, verbatim:
```
cargo run -q -p phalcom-core --bin phalcom -- "<scratchpad>/dead_frame_catch_test.ph"
cargo run -q -p phalcom-core --bin phalcom -- "<scratchpad>/dead_frame_typed_no_match.ph"
```

---

## 7. The GC relationship

`heap/trace.rs::trace_frame` doc (~L35-36):

> `home_frame_token` is **not** an edge: a [`FrameToken`] is an index plus a
> generation counter, not a handle.

And at the `Object::Block` trace arm (~L141-144):

```rust
Object::Block(block) => {
    // The block *is* the only retainer of its closure once passed around.
    // `home_frame_token` is an index+generation, not a handle.
    push(block.closure);
}
```

`vm/gc.rs::collect_roots` (~L18-110) is written as an **exhaustive
destructure of `VM`** specifically so a newly added field fails to compile
until classified root/non-root (doc, ~L25-31): "the original hand-audited
root table missed `sealed_classes`, `checking` **and** `ready_queue`... Do
not replace this with field accesses." `next_frame_generation: _` (~L83) is
one line in that destructure, classified as a non-root alongside
`world_version`, `switch_pending`, `native_reentry_depth`, etc. — plain `u64`
counters, not object handles, so nothing to trace.

**Confirmed: a `FrameToken` (whether sitting in a live `CallFrame` or a
`BlockObject`) does not keep its frame alive.** It is pure data (an index and
a counter), never traced as a GC edge; only the closure a `BlockObject` wraps
is a root. **VERIFIED** (all three sites read in full).

---

## 8. The hard trace — `blocks_non_local_return_two_deep.ph`

Source (`phalcom-core/tests/lang/blocks/blocks_non_local_return_two_deep.ph`,
expected output `8`):

```phalcom
class Finder {
  findFirstEven(numbers) {
    numbers.each { n =>
      (n > 0).ifTrue {
        (n % 2 == 0).ifTrue { return n }
      }
    }
    return None
  }
}
```

**Attempted the `vm-trace` feature first, as instructed.** Built and ran:

```
cargo run -q -p phalcom-core --bin phalcom --features vm-trace -- <file>
```

Output: just `8` — **zero trace lines**, even though the crate compiles
`debug!` calls under `#[cfg(feature = "vm-trace")]`
(`vm/dispatch.rs` ~L1197-1198). Root cause, **VERIFIED** by reading
`phalcom-core/bin/phalcom/main.rs` (~L12-15):

```rust
let stdout_log = fmt::layer().pretty();
tracing_subscriber::registry().with(stdout_log.with_filter(filter::LevelFilter::OFF)).init();
```

The tracing subscriber's level filter is **hardcoded to `OFF`** in the CLI
binary regardless of the `vm-trace` feature flag or `RUST_LOG` — so
`vm-trace` currently produces no observable output from this binary at all
(not "too noisy," literally silent). Reporting this plainly per the brief's
instruction, without editorializing on whether it's a bug.

**Fallback: the disassembler.** Checked `bin/phalcom/cli.rs` — the real
subcommand is `disasm` (`Commands::Disasm`/`cmd_disasm`). Ran it:

```
cargo run -q -p phalcom-core --bin phalcom -- disasm <file>
```

This **only disassembles the top-level module chunk**
(`bin/phalcom/disasm.rs`, in full — it calls `compile_closure` once and
prints `chunk.code`/`chunk.constants` for that one closure; it does not walk
into class method or block closures at all). So it shows the module-level
`class Finder { ... }` / driver code, not `findFirstEven`'s or the blocks'
own bytecode. **Confirmed limitation, stated plainly**: neither `vm-trace`
nor `disasm` gives a usable runtime frame trace for this program as shipped.

**Falling back to reasoning from source**, as the brief allows. This part is
**INFERRED** (grounded in verified mechanisms, not a captured trace):

- `List` has no `each` of its own in `core.ph`; it inherits `Iterable#each`
  (`core.ph` ~L654-658, **VERIFIED** by reading it):

  ```phalcom
  each(f) {
    for (x in self) {
      f.call(x)
    }
  }
  ```

  This is a real `.ph` closure method, invoked as an ordinary message send
  (`Invoke` → `call_method`'s `MethodKind::Closure` arm, `vm/send.rs` ~L95+),
  pushing a genuine `CallFrame` with `home_frame_token: None` (it's an
  ordinary method, not a block).

- `ifTrue { literal block }` is **sacred-inlined** — confirmed by
  `compiler/inliner.rs`'s module doc (~L18: "directly into the enclosing
  function's bytecode — no `ClosureObject`") and the inlining condition
  (~L141: `("ifTrue", 1) if is_literal_block(&call.args[0])`). So neither
  `(n > 0).ifTrue { ... }` nor the nested `(n % 2 == 0).ifTrue { return n }`
  creates a `BlockObject` or a `CallFrame` — both compile straight into
  `GuardBool`/jump bytecode inside whatever frame is already running.
  `return n`, though lexically nested two `ifTrue`s deep, is really just
  more bytecode in the **one** enclosing block's own chunk, compiled to
  `ReturnNonLocal` because it is lexically inside the `{ n => ... }` block
  literal.

So the live frame stack at the moment `ReturnNonLocal` fires is (inferred,
four frames, indices 0-3):

| idx | frame | pushed by | `home_frame_token` |
|---|---|---|---|
| 0 | module top level | VM init / `run_in_module` | — |
| 1 | `findFirstEven(numbers)` | ordinary `Invoke` (`vm/send.rs`) | `None` (ordinary method) |
| 2 | `Iterable#each(f)` | ordinary `Invoke` from inside frame 1 | `None` (ordinary method) |
| 3 | `{ n => ... }` block body | `block_call` from inside frame 2's `f.call(x)` | `Some(token{frame_index:1, generation:g1})` |

The token's `frame_index` is `1` because it was minted by the `Closure`
opcode's `current_frame_token()` at the point the `{ n => ... }` literal was
evaluated — which happens directly inside `findFirstEven`'s own bytecode,
*before* `.each` is ever called, when `self.frames.len() - 1 == 1`.

At `ReturnNonLocal` (running inside frame 3, `self.frames.len() == 4`):

- `is_live`: `self.frames.get(1)` is still frame 1 (`findFirstEven`, never
  popped, same generation) → **true**.
- `home_stack_offset = self.frames[1].stack_offset`.
- `self.frames.truncate(1)` keeps indices `[0, 1)` — **only frame 0
  survives.** Frames 1 (`findFirstEven` itself), 2 (`each`), and 3 (the
  block) are all dropped.

**Does it remove the home frame itself? Yes — and that is correct, not an
off-by-one.** `token.frame_index` is the home frame's own absolute index
(`1`), not "one past the home frame" or a child count. `Vec::truncate(n)`
keeps `[0, n)`, i.e. drops index `n` and everything above — so
`truncate(1)` drops exactly index `1` onward. That is precisely what an
ordinary `Bytecode::Return` executed *by frame 1 itself* would have done:
`self.frames.pop()` removes exactly frame 1, leaving `[frame 0]`. Truncating
to `token.frame_index` (rather than `token.frame_index + 1`, which would
incorrectly leave the home frame permanently un-popped) is what makes the
comment true: "the value is now exactly where an ordinary `Return` from the
home method would have left it." **VERIFIED** the truncate semantics and the
`Return` handler's own `frames.pop()` (`vm/dispatch.rs` ~L1099); the specific
assembled 4-frame picture above is **INFERRED**, not captured by a trace.

Control resumes in whichever Rust `run_until`/`run_until_inner` invocation's
own floor (`base_frames`) is now `>= self.frames.len()` (`== 1`) — see §9.

---

## 9. The termination hand-off

Verified against the actual loop. `run_until_inner`'s halt check,
`vm/dispatch.rs` (~L490-497):

```rust
loop {
    if self.frames.len() <= base_frames {
        // A drained frame stack with nothing left to yield means the
        // top-level program (or a block activation) fell off its end:
        // surface that absence as `None`, never the private sentinel.
        let result = self.stack.pop().unwrap_or(Value::Nil);
        return Ok(self.surface_absence(result));
    }
    ...
```

This check is **completely unmodified** by `ReturnNonLocal` — it is the same
top-of-loop drain check every `run_until_inner` (nested or not) always runs.
Mechanical confirmation for the two-deep trace: three re-entrant
`run_until`/`run_until_inner` calls are on the native Rust stack when
`ReturnNonLocal` fires (one per native call boundary that pushed a frame
re-entrantly: `each`'s own `Invoke`-based call does *not* re-enter natively —
only `block_call`'s `f.call(x)` does, via `vm.native_reentry_depth += 1;
vm.run_until(base_frames)`, `primitive/block.rs` ~L158-160). That inner
`run_until(base_frames=2)` (frame 2, `each`, was on top when `f.call(x)`
ran, so `base_frames` was `self.frames.len()` at that point, i.e. `2`) is
the first nested loop to notice, on its very next top-of-loop check after
`ReturnNonLocal` truncates to length `1`: `self.frames.len() (1) <= base_frames (2)`
→ **true**, so it pops the pushed `return_value` off `self.stack` and
returns it as `Ok(value)` through the `block_call` → `Iterable#each`'s own
Rust call chain, which itself returns through `call_method`'s `Primitive`
arm and back up. The outer `run_until_inner(base_frames=1)` — the one
directly driving `findFirstEven`'s own frame — sees, on its next iteration,
`self.frames.len() (1) <= base_frames (1)` → also true, and it too drains
and returns. Eventually the outermost `run(base_frames=0)` sees `0 <= 0` and
completes the top-level program. **The comment is not stale — the mechanism
is exactly as described**: no special-cased return path, just the
unmodified drain check, now finding its floor early because `frames.truncate`
already did the real work. **VERIFIED** the halt check and the general
mechanism; the specific nesting depths quoted (`base_frames` values 2, 1, 0)
are **INFERRED** from the source-level trace in §8, not captured live.

---

## 10. Spec/ADR — bounded

ADR-0013 (`docs/adr/accepted/0013-closure-upvalues-and-frame-token-return.md`)
**Alternatives considered** (~L57-66) — exactly two entries, confirmed:

1. "By-value snapshot capture" — rejected: breaks shared mutation, fights
   non-local return.
2. "Raw frame pointer with no generation counter" — rejected: a reused frame
   slot would alias a stale pointer, silently returning to the wrong method.

`docs/spec/v0.2/blocks.md` §5 "Non-local return" (~L62-85) promises: `return`
inside a block unwinds to the frame it was created in and returns from the
enclosing method; escaping blocks get a frame token (pointer + generation);
generation mismatch raises `DeadFrameError`; explicitly notes Smalltalk's
equivalent is `BlockCannotReturn`. Matches the implementation exactly.
**VERIFIED**.

---

## 11. Tests/fixtures

All under `phalcom-core/tests/lang/...` (not `tests/lang/...` — the crate-local
test corpus; note this path correction for future doc work).

- `blocks/blocks_non_local_return.ph` → `-5`. Multi-level unwind through
  `List.each`'s native-`block_call` re-entry (the note in the file says this
  is deliberately *not* a single-level `{ return x }.call()`, to exercise
  the real U10 unwind).
- `blocks/blocks_non_local_return_bare.ph` → `None`. Bare `return` (no
  expression) inside a block still non-local-returns and surfaces `None`,
  mirroring `Bytecode::Return`'s bare-return handling.
- `blocks/blocks_non_local_return_two_deep.ph` → `8`. Two nested `ifTrue`
  blocks (both sacred-inlined, no extra frames) plus one native `block_call`
  re-entry via `each`; see §8.
- `blocks/blocks_non_local_return_in_loop.ph` → `5` then `after`. `return`
  inside a `for` loop body unwinds the whole method, not just the loop; the
  file's own comment stresses `"after"` must never print for the *inner*
  case (it does print, correctly, once execution truly returns to top level).
- `control-flow/control_flow_inline_non_local_return.ph` → `3`. `return`
  inside an *inlined* `if`/`while` (not a block at all — sacred-inlined
  control flow) is an ordinary same-frame return, not `ReturnNonLocal` — a
  useful contrast case.
- `runtime-errors/runtime_non_local_return_dead_frame.ph` → `DeadFrameError`
  (NEGATIVE lane). `Maker.new().make()` returns an escaping block without
  calling it; a later `.call()` finds the home frame gone.
- `concurrency/negative/fiber_cross_fiber_non_local_return_dead_frame.ph` →
  `DeadFrameError` (NEGATIVE lane). Same escaped block, but invoked from a
  **different fiber** via `Fiber.new(escaped); f.call()` — proves the
  generation check is fiber-agnostic (see the dominating question above).

Rust unit tests touching `FrameToken`/`generation` directly —
`phalcom-core/tests/invariants.rs`:

- `invoke_on_preserves_dead_frame_fencing_for_escaping_blocks` (~L1218-1246):
  drives the escaping-block scenario through `VM::invoke_method_object`
  (the `Method#invokeOn`/`bound.call` engine) instead of an ordinary send,
  and asserts `Err(PhError::Runtime(RuntimeError::DeadFrameError))`.
- `cross_fiber_non_local_return_raises_dead_frame_error` (~L1248-1270):
  the Rust-level mirror of the cross-fiber golden above, same assertion,
  explicitly commented as proving "the frame-token generation check is
  fiber-agnostic."

Neither test touches `FrameToken`'s fields directly (no `frame.generation ==`
assertions in Rust) — both test the *observable* behavior (the `Err` variant)
rather than poking the struct. **VERIFIED** (both read in full).

---

## 12. What `docs/learn/vm/upvalues.md` already covered

Read only its "Non-local return: trap, don't corrupt" section (~L752-781).
It already spent:

- The framing: non-local return "has a failure mode: the method may already
  be gone," and the fix must **trap**, not corrupt.
- The exact `is_live` code snippet (the same `is_some_and(|home|
  home.generation == token.generation)` check quoted in §2 above).
- The exact error string from running
  `tests/lang/runtime-errors/runtime_non_local_return_dead_frame.ph`:
  `non-local return from a block whose home method frame is no longer alive
  (DeadFrameError)`.
- The Smalltalk name-check: "matching Smalltalk-80's `BlockCannotReturn`."
- The rhyme line: "Index plus generation. Name, then check." — tying this
  back to the chapter's broader open/closed-upvalue "name, not address"
  theme.
- Immediately after (still in the file, next section "The garbage
  collector"), the `Upvalue::Open` GC-rooting snippet — a **different**
  mechanism (open upvalues root their fiber) from `FrameToken` GC-non-rooting
  (§7 above); worth being careful not to conflate the two in the new doc.

What it did **not** cover (all novel to this doc): the `frame_index` vs
`generation` scope asymmetry and its fiber implications (the dominating
question); the mint/stamp/carry lifecycle across `new_call_frame`,
`Bytecode::Closure`, `block_call`, and the fiber-entry path; the duplicated
stamp in `run_in_module`; the `wrapping_add` non-guard; the ADR-0013
"unifies with throw/abort" claim's actual (partial) shipped status; the
`DeadFrameError`-is-actually-catchable discrepancy between `block_on`'s doc
comment and its code; the concrete truncate-removes-the-home-frame mechanics
via `Vec::truncate`; and the drain-check hand-off across nested `run_until`
calls. **VERIFIED** (full section read).
