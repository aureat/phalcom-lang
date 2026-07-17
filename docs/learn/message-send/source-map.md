# Message send / dynamic dispatch — source map

Read-only source map at HEAD. Oriented via `graphify query "how does message
send and method dispatch work"`, `graphify explain "invoke_at"`, `graphify
explain "call_method"`, `graphify explain "lookup_method_in_hierarchy"`,
`graphify affected "invoke_at"` / `"call_method"` before reading; regions
below are the ones those queries pointed at, then read directly.

## THE QUESTION THAT DOMINATES EVERYTHING

Candidates considered for what a compiled call site names:
- (a) a **method** — the call site already holds a resolved method handle
- (b) a **vtable offset** — a fixed slot index into a per-class dispatch table
- (c) a **selector** — an interned name+labels symbol, resolved at send time

**The code shows (c).** `phalcom-core/src/bytecode.rs` L108-111 — VERIFIED,
quoted in full:

```rust
/// Calls a method directly on a receiver, bypassing property lookup.
/// 0: number of arguments
/// 1: index of selector constant
Invoke(u8, u16),
```

The operand shape is `(u8, u16)`: an arity count and a `u16` index into the
chunk's **constant pool**, where the constant is a `Symbol` (a selector), not
a method handle and not a table offset. Recon's claim is confirmed exactly:
`Invoke` carries a `u8` arity and a `u16` selector-constant index. It also
owns an inline-cache slot, but the slot is *keyed by* `cache_ip` (the
instruction's own index into `Chunk::caches`/`Chunk::spans`, parallel arrays
outside the instruction encoding) — the cache is a side table the send
consults, not part of what the operand itself names. See `invoke_at` (§2)
for confirmation the cache is populated by, not a substitute for, a selector
lookup.

**Nothing is resolved at compile time.** The compiler emits the selector
string via `encode_selector` (§5) and interns it into the constant pool; no
method handle, class handle, or slot offset is baked into the instruction.
Resolution — selector → method — happens **at send time**, inside
`invoke_at` (dispatch.rs L398, §2): first an inline-cache probe (send-time,
populated by a prior send at this exact call site), then a hierarchy walk
(`lookup_method`, §4) if the cache misses. There is no compile-time dispatch
path in this VM at all — every `Invoke`/`InvokeLocal`/`InvokeConst`/
`SuperSend` reaches `invoke_at` or the `SuperSend` arm, both of which resolve
at runtime.

## 1. The `Invoke` opcode operand shape

`Bytecode::Invoke(u8, u16)` — `phalcom-core/src/bytecode.rs` L108-111,
VERIFIED, quoted above. Comment: "0: number of arguments. 1: index of
selector constant." The `u16` indexes `Chunk::constants`, and the value read
out at that index is a `Value::Symbol` (confirmed at `invoke_at` L419-420:
`let selector_val = callable.chunk.constants[selector_idx as usize]; let
selector_sym = selector_val.as_symbol().unwrap();`). No method handle is
present anywhere in the operand.

Fused super-instructions exist and carry the same selector-constant
convention — **this is Doc-5 (inline-cache/fusion) territory, flagged, not
dwelt on here**:

- `InvokeLocal(u16, u8, u16)` — `bytecode.rs` L344 — "0: local slot index. 1:
  number of arguments. 2: selector constant index." Fuses `GetLocal` +
  `Invoke` (cut 008).
- `InvokeConst(u16, u8, u16)` — `bytecode.rs` L354 — "0: constant-pool index
  of the pushed value. 1: number of arguments. 2: selector constant index."
  Fuses `Constant` + `Invoke`.

Both fused forms' third operand is a selector-constant index — same shape as
plain `Invoke`, not a method or offset. Both are handled in
`vm/dispatch.rs` (L1036, L1046) by pushing their first half's value, then
calling the *same* `invoke_at` at `ip + 1` (the dead `Invoke` the fusion left
behind) — so fusion is purely a dispatch-loop-iteration optimization, not a
different resolution strategy.

## 2. `invoke_at` full body — the send on the miss path

`phalcom-core/src/vm/dispatch.rs::VM::invoke_at` @ L398. VERIFIED, quoted in
full:

```rust
fn invoke_at(&mut self, callable: &Callable, cache_ip: usize, arity: u8, selector_idx: u16) -> PhResult<()> {
    let arity = arity as usize;
    let receiver_idx = self.stack.len() - 1 - arity;
    let receiver = self.stack[receiver_idx];
    let receiver_class = receiver.class(self);

    let (cached, source_range) = {
        let chunk = &callable.chunk;
        let cached = chunk.caches[cache_ip].get().filter(|slot| {
            slot.class == receiver_class && slot.version == self.world_version
        }).map(|slot| slot.method);
        (cached, chunk.spans[cache_ip])
    };

    if let Some(method) = cached {
        self.call_method(&receiver, method, arity, source_range)?;
    } else {
        let selector_val = callable.chunk.constants[selector_idx as usize];
        let selector_sym = selector_val.as_symbol().unwrap();

        if let Some(method) = receiver.lookup_method(self, selector_sym) {
            let entry = crate::chunk::InlineCache { class: receiver_class, method, version: self.world_version };
            callable.chunk.caches[cache_ip].set(Some(entry));
            self.call_method(&receiver, method, arity, source_range)?;
        } else {
            let variadic_selector_opt = if let Some(&cached_opt) = self.variadic_selector_cache.get(&selector_sym) {
                cached_opt
            } else {
                let (name, labels, kind) = decode_selector(self.resolve_symbol(selector_sym));
                let eligible = matches!(kind, SignatureKind::Method(_)) && labels.iter().all(Option::is_none);
                let derived = eligible.then(|| self.interner.intern(&format!("{name}(*)")));
                self.variadic_selector_cache.insert(selector_sym, derived);
                derived
            };
            let variadic_hit = variadic_selector_opt
                .and_then(|variadic_selector| receiver.lookup_method(self, variadic_selector))
                .and_then(|m| {
                    let sig = &self.heap.method(m).signature;
                    (arity >= sig.positional_arity as usize).then_some(m)
                });
            if let Some(method) = variadic_hit {
                self.call_method(&receiver, method, arity, source_range)?;
            } else {
                self.forward_does_not_understand(receiver_idx, selector_sym, source_range)?;
            }
        }
    }
    Ok(())
}
```

**Miss order, in order (VERIFIED by reading the arms above), matching the
function's own doc comment "IC probe, exact-selector lookup + refill,
variadic probe, then `doesNotUnderstand(_)` forward":**

1. **Inline-cache probe** (L410-412) — `chunk.caches[cache_ip]`, valid only
   if `slot.class == receiver_class && slot.version == self.world_version`.
   *Doc-5 territory — flagged, not analyzed here.*
2. **Exact-selector probe** — `receiver.lookup_method(self, selector_sym)`
   (L422), i.e. the ordinary hierarchy walk (§4). A hit refills the cache
   (L425-426) before dispatching. *Cache refill is Doc-5 territory.*
3. **Variadic probe** (L442-456) — only reachable if the exact probe missed.
   Only eligible for an all-positional `SignatureKind::Method` selector
   (never labelled/getter/setter/subscript — L446); derives `name(*)` and
   does one more ordinary `lookup_method` walk, gated on `arity >=
   positional_arity`. *Also effectively a cache (`variadic_selector_cache`)
   — Doc-5 territory, flagged.*
4. **`doesNotUnderstand(_)` forward** (L460) — only if all three above miss.

Every branch reaches `self.call_method(...)` or
`self.forward_does_not_understand(...)`, both of which propagate `PhResult`
with `?` — confirming ADR-0012's F1 fix (the `Result` is threaded, not
discarded) is live at HEAD.

## 3. `call_method` full body — the fork

`phalcom-core/src/vm/send.rs::VM::call_method` @ L19. VERIFIED, quoted in
full (the argument-buffer/fiber-switch commentary is original, not
paraphrased — it is load-bearing to the "not every send pushes a frame"
claim):

```rust
pub(super) fn call_method(&mut self, callee: &Value, method: ObjRef, arity: usize, source_range: SourceRange) -> PhResult<()> {
    let kind = self.heap.method(method).kind;
    match kind {
        MethodKind::Primitive(native_fn) => {
            let receiver_idx = self.stack.len() - 1 - arity;
            let receiver = self.stack[receiver_idx];
            let frames_before = self.frames.len();
            self.switch_pending = false;
            const INLINE_ARGS: usize = 8;
            let result = if arity <= INLINE_ARGS {
                let mut args = [Value::Nil; INLINE_ARGS];
                args[..arity].copy_from_slice(&self.stack[receiver_idx + 1..]);
                native_fn(self, &receiver, &args[..arity])
            } else {
                let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
                native_fn(self, &receiver, &args)
            };
            result.map(|result| {
                if self.switch_pending {
                    self.switch_pending = false;
                } else if self.frames.len() >= frames_before {
                    self.stack.truncate(receiver_idx);
                    self.stack.push(result);
                } else {
                    self.stack.push(result);
                }
            })
        }
        MethodKind::Closure(closure_id) => {
            let context = callee.to_context(&self.heap);
            let receiver_idx = self.stack.len() - arity - 1;
            let (variadic, fixed_arity) = {
                let sig = &self.heap.method(method).signature;
                (sig.variadic, sig.positional_arity as usize)
            };
            if variadic {
                let rest = self.stack.split_off(receiver_idx + 1 + fixed_arity);
                let list_id = self.heap.alloc_list(rest);
                self.stack.push(Value::Obj(list_id));
            }
            let stack_offset = receiver_idx;
            let new_frame = self.new_call_frame(closure_id, context, 0, stack_offset, Some(source_range));
            self.frames.push(new_frame);
            Ok(())
        }
    }
}
```

**The fork is `MethodKind::{Primitive(PrimitiveFn), Closure(ObjRef)}`** —
`phalcom-core/src/method/object.rs` L15-22, VERIFIED, quoted in full:

```rust
/// The implementation strategy behind a [`MethodObject`].
#[derive(Debug, Clone, Copy)]
pub enum MethodKind {
    /// Phalcom code compiled to bytecode, by [`ClosureObject`](crate::heap::ClosureObject) handle.
    Closure(ObjRef),
    /// A native Rust function for a core-library method.
    Primitive(PrimitiveFn),
}
```

**"Not every send pushes a frame" — CONFIRMED by reading the body above.**
The `Primitive` arm calls `native_fn(self, &receiver, &args[..arity])`
directly (in place, on the Rust call stack) and never touches
`self.frames` at all in its own body — it only *reads* `self.frames.len()`
afterward (`frames_before`) to detect whether the primitive itself triggered
frame changes (a non-local return unwinding through it, or a fiber switch).
No `CallFrame` is constructed or pushed on the primitive path. The `Closure`
arm is the opposite: it builds a `CallFrame` via `self.new_call_frame(...)`
(L113) and pushes it onto `self.frames` (L114) — this is the "method push
site" a prior doc (frames/source-map.md, Doc 3) names — and returns without
running the method body itself; `run_until`'s loop drains it later.

**Three post-return paths on the primitive arm (L53-93)**, quoted above,
branch on:

1. **`self.switch_pending`** (L54) — a fiber switch fired *inside* the
   primitive (`fiber_call`/`fiber_try`/`fiber_yield`); neither `result` nor
   the stack is touched, because the primitive already repointed
   `self.frames`/`self.stack` to a different fiber. *→ concurrency doc
   (ADR-0030), flagged, not analyzed here.*
2. **`self.frames.len() >= frames_before`** (L68) — the ordinary case: no
   frame-count change, so the receiver+args window is truncated and the
   result pushed in its place.
3. **else** (L73, `self.frames.len() < frames_before`) — a
   `Bytecode::ReturnNonLocal` fired inside `native_fn` (e.g. a block passed
   to a primitive like `block_call` executed a `return` that unwound past
   this call site), popping frames out from under `call_method`; the result
   is re-pushed rather than truncated-and-pushed, to avoid mis-placing it
   against a stack that already moved. *→ Doc 6 (non-local return), flagged,
   not analyzed here.*

## 4. The chain walk

`phalcom-core/src/value/mod.rs::Value::lookup_method` @ L170 — VERIFIED,
quoted in full:

```rust
pub fn lookup_method(&self, vm: &VM, selector: Symbol) -> Option<ObjRef> {
    let class = self.class(vm);
    lookup_method_in_hierarchy(&vm.heap, class, selector)
}
```

It delegates immediately to
`phalcom-core/src/heap/class.rs::lookup_method_in_hierarchy` @ L74 —
VERIFIED, quoted in full:

```rust
pub fn lookup_method_in_hierarchy(heap: &Heap, mut class: ClassId, selector: Symbol) -> Option<ObjRef> {
    loop {
        let current = heap.class(class);
        if let Some(&method) = current.methods.get(&selector) {
            return Some(method);
        }
        match current.superclass {
            Some(superclass) => class = superclass,
            None => return None,
        }
    }
}
```

**Confirmed: single-inheritance simple loop** — exactly `current.methods.get(&selector)`,
else follow `current.superclass`, else `None`. No multi-parent walk, no
MRO/C3 linearization, no caching inside this function (caching is the
caller's — `invoke_at`'s — job).

**Method dictionary type** — `phalcom-core/src/heap/class.rs` L16-17,
VERIFIED:

```rust
/// Selector → [`MethodObject`](crate::method::MethodObject) handle table.
type MethodsMap = IndexMap<Symbol, ObjRef>;
```

`ClassObject.methods: MethodsMap` (`IndexMap<Symbol, ObjRef>`), keyed by
selector `Symbol`, valued by an `ObjRef` handle to the `MethodObject` (not
the method inline).

**Superclass link type** — `ClassObject.superclass: Option<ClassId>`
(`heap/class.rs` L33, VERIFIED: "Handle to this class's superclass, or
`None` at the tower's apex (`Object`)."). `ClassId` is a `Copy` arena handle
(ADR-0009), not a pointer/`Rc`.

## 5. The selector representation

Selectors are built by `encode_selector` /
`phalcom-core/src/method/mod.rs` L102-118, VERIFIED, quoted in full:

```rust
pub fn encode_selector(name: &str, labels: &[Option<String>], kind: SignatureKind) -> String {
    match kind {
        SignatureKind::Initializer(0) => format!("init {name}()"),
        SignatureKind::Initializer(_) => format!("init {name}({})", comma_form_slots(labels)),
        SignatureKind::Method(0) => format!("{name}()"),
        SignatureKind::Method(_) => format!("{name}({})", comma_form_slots(labels)),
        SignatureKind::Getter => name.to_string(),
        SignatureKind::Setter => format!("{name}=(_)"),
        SignatureKind::Subscript(_) => format!("[{}]", comma_form_slots(labels)),
        SignatureKind::Variadic(_) => format!("{name}(*)"),
    }
}
```

`comma_form_slots` (L122-124) joins each label as `_` for a positional
argument or the label text for a keyword argument, comma-joined. The encoded
string is then interned to a `Symbol` at the call site (compiler) and at
every runtime selector-building path (`perform`, `doesNotUnderstand`
forwarding, `new_message`) — ADR-0012 requires this be the *only* encoder, to
close a prior divergent-encoder defect (F8, §11).

**`move(to,duration)` and `move(_,_)` are confirmed different keys.** Given
`labels = [Some("to"), Some("duration")]` vs `labels = [None, None]`,
`comma_form_slots` produces `"to,duration"` vs `"_,_"`, so `encode_selector`
returns the distinct strings `"move(to,duration)"` and `"move(_,_)"` — which
intern to distinct `Symbol`s and therefore distinct `MethodsMap` keys. This
is exactly ADR-0012's core claim (§11).

The inverse, `decode_selector` (`method/mod.rs` L156-207, read in full), is
total (never panics — an unparseable string decodes to `SignatureKind::Getter`,
documented at L137-141) and is used on the miss path to reify a `Message`
(§6) and by the variadic-probe derivation in `invoke_at` (§2).

## 6. The miss path

**`new_message`** — `phalcom-core/src/vm/send.rs` L138-161, VERIFIED. Doc
comment (L120-137) states the `Message` instance has **four slots**,
matching recon exactly:

```
0. `selector` — the interned Symbol as sent;
1. `name` — the bare method name String;
2. `labels` — a List of String, one per argument, "" for positional;
3. `args` — a List of the argument values.
```

Body confirms the four-slot construction directly:

```rust
let mut instance = crate::heap::InstanceObject::new(message_class, 4);
instance.slots[0] = Value::Symbol(selector);
instance.slots[1] = name_val;
instance.slots[2] = labels_list;
instance.slots[3] = args_list;
```

**`forward_does_not_understand`** — `phalcom-core/src/vm/send.rs` L181-195,
VERIFIED, quoted in full:

```rust
pub(super) fn forward_does_not_understand(&mut self, receiver_idx: usize, selector: Symbol, source_range: SourceRange) -> PhResult<()> {
    let receiver = self.stack[receiver_idx];
    let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
    self.stack.truncate(receiver_idx + 1);
    let message = self.new_message(selector, &args);
    self.stack.push(message);

    let dnu_str = crate::method::encode_selector("doesNotUnderstand", &[None], crate::method::SignatureKind::Method(1));
    let dnu_sym = self.get_or_intern(&dnu_str);
    match receiver.lookup_method(self, dnu_sym) {
        Some(method) => self.call_method(&receiver, method, 1, source_range),
        None => Err(RuntimeError::Internal("doesNotUnderstand(_) missing from Object — kernel invariant violated".into()).into()),
    }
}
```

It builds the selector `doesNotUnderstand(_)` and sends it — via the same
`lookup_method` + `call_method` machinery as any ordinary send, not a
separate code path.

**Recursion guard — VERIFIED by reading the `None` arm.** If
`doesNotUnderstand(_)` itself is missing from the receiver's chain, the
function does **not** re-forward to `doesNotUnderstand` again (which would
recurse) — it raises `RuntimeError::Internal("doesNotUnderstand(_) missing
from Object — kernel invariant violated")` and stops. The doc comment (L171-175)
states this explicitly: "a missing dNU is never itself re-sent as a dNU."
The guard is structural (a distinct `Err` arm with no further send), not a
depth counter or re-entrancy flag.

## 7. The reflective twin

**`send_dynamic`** — `phalcom-core/src/vm/send.rs` L218-237. Pushes
`receiver`+`args` onto the stack at a fresh window, does the *same*
`lookup_method` → `call_method`/`forward_does_not_understand` dispatch as
`invoke_at`, then **re-enters `run_until`** (`self.run_until(base_frames)`,
guarded by `native_reentry_depth`) to drain the pushed activation and
recover a synchronous `Value`. Consumers: `Object::perform(_)` /
`Object::perform(_,_)` (`primitive/object.rs`, §10e) and the
`doesNotUnderstand(_)` forward indirectly.

**`invoke_method_object`** — `phalcom-core/src/vm/send.rs` L259-280. Same
re-entrant `run_until` pattern, but **no lookup at all** — `method_id` is
already resolved (arity-validated first, L264-267), so it goes straight to
`self.call_method(...)`. Backs `Method#invokeOn(_,_)` / `Method#bind(_)`'s
`call` (reflection on an already-resolved `Method` object).

Both are **not** the compiled `Invoke` path — they are the runtime-callable
surface behind `perform`/reflection, reachable from a native primitive
(`send_dynamic`/`invoke_method_object` are `pub fn`, called from
`primitive/object.rs`), never emitted by the compiler for an ordinary `.`
send.

## 8. `SuperSend`

**Confirmed: its own opcode**, not a flag on `Invoke`. `SuperSend(u8, u16, u16)`
— `bytecode.rs` L138, doc comment: "Sends `selector` starting the method walk
**above** a statically-known class, with the original receiver (`self`)... the
lowering of a `super.sel(…)` send (method-lookup.md §1.14, U-INH, ADR-0040)."
The dispatch arm (`vm/dispatch.rs` L863-916) resolves the *defining class*
by name at dispatch time (`self.classes.get(&defining_sym)`), reads its
`superclass` fresh each send, and walks `lookup_method_in_hierarchy` starting
there — never touching the receiver's own class for the walk's start point.
*→ deferred mechanism (ADR-0040), not analyzed further here.*

## 9. Constructors

**Confirmed: no constructor special-case in `lookup_method`.** The comment
at `value/mod.rs` L165-169 (VERIFIED, quoted in full above §THE QUESTION's
neighboring read) states it directly:

```rust
/// A class receiver needs no constructor-specific fallback here:
/// constructors install on the metaclass under the ordinary selector their
/// call sites encode, so the plain hierarchy walk resolves `Foo.new()` to
/// `Foo`'s constructor — shadowing the bare allocator `Class >> new()` at
/// the tower root — exactly as it resolves any other class-side method.
```

`Foo.new()` resolves through the identical `lookup_method_in_hierarchy` walk
(§4) as any other send — `new` is an ordinary selector on `Foo`'s metaclass
that happens to shadow `Class`'s bare allocator at the tower root, per
ADR-0063 ("Constructors are ordinary class-side methods").

## 10. Fixtures run live

CLI: `cargo run -p phalcom-core --bin phalcom -- <file.ph>`. Build was clean
(`cargo build -p phalcom-core --bin phalcom`, one pre-existing unrelated dead-code
warning for `init_selector_cache`, no errors). All five fixtures below are
OBSERVED, not inferred — run in this session against HEAD, output stripped
of cargo's warning banner.

**(a) plain method send that resolves and runs:**

```phalcom
class Greeter {
  greet(name) {
    return "Hello, " + name;
  }
}
let g = Greeter.new();
System.print(g.greet("World"));
```

Output: `Hello, World` — CONFIRMED works.

**(b) primitive send (`1 + 2`):**

```phalcom
System.print(1 + 2);
```

Output: `3` — CONFIRMED the native path produces a correct result. "Zero
frames pushed" is **read-from-code** (§3), not independently observable from
stdout — noted, not re-claimed as observed.

**(c) miss — `42.flibbertigibbet()`, no class implements it:**

```phalcom
System.print(42.flibbertigibbet());
```

Output, VERBATIM:

```
42 does not understand 'flibbertigibbet()'
```

The default `doesNotUnderstand` handler fires and produces exactly this
message. This is the single strongest piece of live evidence for §6's miss
path: an unrecognized selector reaches `forward_does_not_understand`, builds
a `Message`, and the kernel's default handler formats it as
`"<receiver> does not understand '<selector-text>'"`.

**(d) define-after / monkeypatch (class reopened, method redefined):**

```phalcom
class Widget {
  label() {
    return "v1";
  }
}
System.print(Widget.new().label());

class Widget {
  label() {
    return "v2";
  }
}
System.print(Widget.new().label());
```

Output, VERBATIM:

```
v1
v2
```

Late binding confirmed: reopening `Widget` and redefining `label()`
re-finalizes the class (`Bytecode::FinalizeClass`, `dispatch.rs` L856-861,
which rebuilds the base-name index "from scratch, not accumulated" per its
own doc comment) and every subsequent send resolves to the new
implementation — consistent with no method handle ever being baked into a
call site (§THE QUESTION).

**(e) reflective `perform`-style send:**

`Object::perform(_)` / `Object::perform(_,_)` exist in `.ph` surface
(`primitive/object.rs` L129-166, `object_perform`/`object_perform_with`,
thin wrappers over `VM::send_dynamic`, §7).

```phalcom
class Adder {
  add(x, y) {
    return x + y;
  }
}
let a = Adder.new();
System.print(a.perform(Symbol.new("add(_,_)"), [3, 4]));
```

Output: `7` — CONFIRMED the reflective surface resolves and runs through the
same lookup/dispatch machinery as a static send.

## 11. Bounded ADR/spec read

`docs/adr/accepted/` selectors/dispatch-relevant set (grepped, not swept):
0012, 0040, 0060, 0063 (plus 0009, 0011, 0018 as supporting handle/slot/inliner
ADRs, not read here).

**ADR-0012 — "Label-encoded selectors and inline-cache-ready dispatch"**
(Accepted, 2026-07-11). Decision: replace arity-only `SignatureKind::Method(u8)`
dispatch with label-encoded selector symbols (`add(_,_)`, `move(to,duration)`,
`name=(_)`, `+(_)`, variadic `sum(*)`), one `encode_selector` helper shared by
compiler and every runtime selector builder, `Invoke` keeping its
selector-constant operand, and the dispatch shape built inline-cache-ready
(population deferred). **Alternatives considered — "Arity-only dispatch"
(the prior `SignatureKind::Method(u8)`)**, rejected explicitly because it
"cannot tell `move(to,duration)` from `move(_,_)`, forced the F7/F8 metadata
mismatches, and contradicts the spec's selector identity." The defect tags
**F1, F7, F8 do appear in the ADR** (Context section, "the forge audit
pinned three defects in exactly this code"): F1 = `Invoke` swallowing
`call_method`'s `Result`; F7 = static `new()` mis-registered as `Method(1)`
for a 0-arg selector; F8 = a malformed selector `">( _)"` from a divergent
encoder. All three are folded into this ADR's decision rather than patched
separately.

**ADR-0040 — "SuperSend opcode"** (Accepted, verified against the U-INH
implementation 2026-07-14, per its own status line). Decision: a third
send opcode, `SuperSend(argc: u8, selector: u16, defining_class_name: u16)`;
receiver stays the original `self`; walk starts at `defining.superclass`,
computed fresh at dispatch time; a super-construct miss retries against the
superclass's metaclass chain; a full-chain miss routes to the same
`doesNotUnderstand` path as an ordinary `Invoke` miss, never a panic.

**ADR-0063 — "Constructors are ordinary class-side methods"** (Accepted,
ratified 2026-07-15). Establishes that `new`'s only special treatment is two
hardcoded compiler string checks (`class_decl.rs`, `expr.rs`) needed solely
because `new` collides with the tower-root's bare allocator; named
constructors (`Ref.at`, `Cell.of`, etc.) have no such collision and dispatch
as ordinary class-side sends. `construct`/`static` are pure syntactic
metadata, changing zero grammar.

**ADR-0060 — "Index operator as a real selector"** (Accepted). Decision:
`expr[idx]` / `expr[idx] = value` / `expr[]` / `expr[] = value` compile
directly to sends against dedicated bracket selectors `[_]` / `[_,put]` /
`[]` / `[put]` — no `at(_)`/`at(_,put:)` lowering. Confirms `[]` is a real
selector kind (`SignatureKind::Subscript`, §5), dispatched through the exact
same `Invoke`/`lookup_method` machinery, not special VM support.

## 12. Use sites / blast radius

`graphify affected "invoke_at"` (depth 2):

| Caller | Location |
|---|---|
| `.run_until_inner()` | `vm/dispatch.rs:L477` (the `Bytecode::Invoke` arm at L1024-1026 calls it) |
| `.run_until()` | `vm/dispatch.rs:L221` (wraps `run_until_inner`) |

`graphify affected "call_method"` (via its node ID, depth 2):

| Caller | Location |
|---|---|
| `.forward_does_not_understand()` | `vm/send.rs:L181` |
| `.invoke_method_object()` | `vm/send.rs:L259` |
| `.send_dynamic()` | `vm/send.rs:L218` |

**Gap in the graph, found by direct reading, not by the tool:** `invoke_at`
itself calls `self.call_method(...)` three times (dispatch.rs L417, L427,
L458) — this edge did **not** appear in `graphify affected "call_method"`'s
output. The extraction likely missed it because `invoke_at` lives in a
different file (`dispatch.rs`) from `call_method`'s definition (`send.rs`)
and the call is through `self.` on a shared `impl VM` split across files.
Flagging this as a known blind spot in the graph rather than silently
trusting its "3 callers" count — `invoke_at` is in fact `call_method`'s
primary, highest-volume caller (every ordinary `.` send that misses its
inline cache), not merely absent from the list by insignificance.
