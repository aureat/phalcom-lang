# Recon — `message-send.md` (VM track, Doc 4)

Phase-1 scout. Not the survey (that is Agent B). Answers the four required questions and arms
both briefs. All line anchors verified at HEAD this session.

---

## 1. Architecture vs representation

**Architecture (the shape).** Late-bound, Smalltalk-lineage *dynamic dispatch by dictionary
lookup*. A send is **two decoupled moves**:

1. **Resolve** — turn a selector into a method by walking the receiver's class chain, matching a
   per-class method dictionary. `lookup_method_in_hierarchy` (`heap/class.rs::lookup_method_in_hierarchy`
   @ ~L74) is a plain loop: `current.methods.get(&selector)`, else follow `current.superclass`,
   else `None`. Single inheritance → the walk is a trivial linked climb, no C3/MRO.
2. **Enter** — hand the resolved method to `call_method` (`vm/send.rs::VM::call_method` @ ~L19),
   which forks on `MethodKind`: a **`Primitive`** runs its native Rust fn *in place* (pushes **no**
   frame); a **`Closure`** builds a `CallFrame` and pushes it (`new_call_frame` then
   `self.frames.push`, send.rs @ ~L113–114 — this is Doc 3's "method push site").

The miss path is the Smalltalk hallmark: no method found ⇒ reify the send as a `Message` object
and forward to `doesNotUnderstand(_)` (`VM::new_message` @ ~L138, `VM::forward_does_not_understand`
@ ~L181; ADR-0012, method-lookup.md §2).

**Representation (what the live state holds — the axis where consequences live).**

- A **selector is an interned `Symbol`** encoding *name + argument labels* (`add(_,_)`,
  `move(to,duration)`, `+(_)`, `name=(_)`, variadic `sum(*)`). Labels are baked into the symbol, so
  `move(to,duration)` and `move(_,_)` are **different keys** — lookup stays one hashmap probe per
  class (ADR-0012 Decision).
- **The compiled call site names the *selector*, not the method.** `Bytecode::Invoke(arity,
  selector_idx)` (dispatch.rs @ ~L1024) carries an `arity: u8` and a `selector_idx: u16` into the
  chunk's constant pool — plus an inline-cache slot it owns. It does **not** hold a method handle,
  a vtable offset, or a resolved address. Which code runs is decided at runtime, by the receiver.
- The method dictionary is `ClassObject.methods: MethodsMap` keyed by selector `Symbol`; the chain
  is `superclass: Option<ClassId>` handles (ADR-0009 arena, no `Rc` cycle).

**The one-line representation fact that settles the doc:** *the call site holds a selector and an
empty cache slot; the method — and whether a frame even appears — is resolved after the site, from
the receiver's class chain and the method's kind.*

## 2. The grip, grounded

> **A call site names a selector, not a method. Sending is *resolve then enter* — find the method
> by walking the receiver's class chain, then run it — and both "which code runs" and "does a frame
> appear" are decided *after* the call site, by the receiver, never by the caller.**

Corollary that ties to Doc 3: **not every send pushes a frame.** A primitive send (`1 + 2`) runs
native Rust in place and pushes zero `CallFrame`s. The frame push is one arm of `call_method`, not a
property of "calling."

## 3. What was actually deliberated (ADR-0012, the one real ADR)

ADR-0012 *selector-signature-encoding-and-dispatch* is the deliberated core. Its **Alternatives
considered**:

- **Arity-only dispatch** (`SignatureKind::Method(u8)`, the pre-ADR tree). Cannot tell
  `move(to,duration)` from `move(_,_)`; forced malformed encodings; **rejected**. Scar attached:
  audit defects F1 (Invoke dropped `call_method`'s `Result` → swallowed primitive errors), F7
  (0-arg `new()` mis-tagged `Method(1)`), F8 (a divergent encoder interned `">( _)"` with a stray
  space). Real shipped bugs — this is the doc's honesty scar.
- **Populate the inline cache now** vs defer. **Deferred** at ADR time ("IC-ready shape at zero
  present cost"); the IC has *since* landed (U-IC — the `caches[cache_ip]` probe + `world_version`
  is live in `invoke_at`, dispatch.rs @ ~L398). → **that is Doc 5's territory; mark as a lie here.**

**Honesty flag for synthesis (§5.2).** The *big* fork a design-space walk wants to present —
static/early binding vs vtable-offset vs dynamic-dictionary — was **not** deliberated. Phalcom is
dictionary-dispatch by Smalltalk/Wren lineage, the same way Doc 1's stack-machine choice was
lineage, not a bake-off. The **finer** fork *was* deliberated: *given* dictionary dispatch, key it
on arity-only or on full label-encoded selectors? That is ADR-0012, and it carries the F1/F7/F8
scar. The doc must present the coarse fork as pedagogical scaffolding and land the real deliberated
choice on the finer axis.

Secondary ADRs (mention, defer mechanism): ADR-0040 (SuperSend is its own opcode — bypasses the
receiver-class start, begins the walk above `self`'s class); ADR-0060 (index `[]` is a real
selector, reinforces "everything is a selector"); ADR-0063 (constructors are ordinary class-side
methods, so `Foo.new()` resolves by the same walk — no constructor special-case in `lookup_method`,
value/mod.rs @ ~L165 comment confirms).

## 4. Brief-steering

**Agent A (theory, no source, not told Phalcom's branch).** Emphasis:
- Go **deep** on the selector→method binding fork: static/early binding, vtable/offset binding,
  dynamic dictionary lookup — each genuinely tempting, each with its bill (what it buys, what it
  forecloses: polymorphism, monkeypatching, speed). This is the doc's spine.
- Go **deep** on the *miss* semantics: what a language does when the selector resolves to nothing —
  compile error (static), `NoSuchMethodError` (vtable/Java), `doesNotUnderstand`/`method_missing`/
  `__getattr__` (dynamic). Name the reification of a failed send as a first-class object.
- **One sentence** on multiple-vs-single inheritance lookup order (C3/MRO) — Phalcom is single, so
  the chain walk is trivial; A should establish *why* MRO exists so the doc can say "we don't need
  it," but not dwell.
- Distinguishing program: the exact small program separating "the method name is bound at compile
  time" from "bound at send time" — i.e. define a method *after* the call site is compiled, or
  monkeypatch, and show it takes effect. And: a send to an absent selector, caught by the miss
  handler at runtime.
- Cast to consider (A names the cut list): Smalltalk (ancestor; names `doesNotUnderstand`,
  `selector`, `Message`), Objective-C (`objc_msgSend`, selector→IMP, the send-is-a-C-function
  reframe), C++ virtual (vtable, the other branch, monkeypatch bill), Ruby (`method_missing`, open
  classes → the invalidation cost), CPython (MRO/C3, `__getattr__` — single-vs-multiple contrast).

**Agent B (source map, graphify-led).** Headline question first: *does a Phalcom call site name a
method, a vtable offset, or a selector — and where is selector→method actually resolved?* Must
confirm with lines:
- `Bytecode::Invoke(arity, selector_idx)` operand shape (dispatch.rs @ ~L1024) + the fused
  `InvokeLocal`/`InvokeConst` (dispatch.rs @ ~L1036/L1046) — note fusion is Doc 5, but confirm the
  operand carries a selector-constant index, not a method.
- `invoke_at` full body (dispatch.rs @ ~L398): the miss order IC → exact-probe → variadic probe →
  `doesNotUnderstand`. Flag the IC/variadic-cache parts as Doc-5 lies but quote the lookup+call
  spine.
- `call_method` full body (send.rs @ ~L19): the `MethodKind::{Primitive,Closure}` fork; primitive
  runs `native_fn` in place with the on-stack arg buffer (INLINE_ARGS, ADR-0051) and the **three**
  post-return paths (ordinary / fiber-switch / non-local-return); closure builds+pushes the frame.
- `lookup_method` (value/mod.rs @ ~L170) → `lookup_method_in_hierarchy` (class.rs @ ~L74): the
  chain walk; confirm single-inheritance simple loop, selector-keyed `methods.get`.
- The miss path: `new_message` (send.rs @ ~L138) 4-slot `Message` reification; `forward_does_not_understand`
  (send.rs @ ~L181) and the "dNU never re-sent as dNU" recursion guard.
- The reflective twin `send_dynamic` (send.rs @ ~L218) and `invoke_method_object` (send.rs @ ~L259)
  — same resolve/enter but re-entering `run_until`; note these are the `perform(_)` / `invokeOn`
  surface, not the compiled `Invoke` path.
- **Run fixtures live:** a plain method send; a primitive send (show it works with zero user
  frames); a `doesNotUnderstand` miss (does the default handler fire? what message?); a
  method defined/monkeypatched and then sent; if a `perform:`-style reflective send exists in
  `.ph`, run it. Report observed output verbatim.
- Bounded ADR read: ADR-0012 Decision + Alternatives only (already summarized here — B verifies).

## 5. Predict-then-check candidates (pick in synthesis)

- **Primary:** "You write `p foo`. The compiler sees the selector `foo`. What does the emitted
  `Invoke` operand hold — the method, a vtable slot, or the selector?" → the selector constant index
  (+ empty cache). Method found at runtime by walking `p`'s class chain.
- **Secondary (ties Doc 3):** "Every send runs a method body. Does every send push a `CallFrame`?"
  → No. Primitive sends push zero; `1 + 2` allocates no frame.

## 6. Lies to mark forward

1. **Inline cache** — the `caches[cache_ip]`/`world_version` probe + refill in `invoke_at` short-
   circuits the chain walk after the first send. Present lookup as "walk every send," mark the cache
   → **Doc 5 (caches-and-fusion)**.
2. **Fusion** — `InvokeLocal`/`InvokeConst` superinstructions fold a preceding load into the send →
   **Doc 5**.
3. **SuperSend** (ADR-0040) — starts the walk above the receiver's class; own opcode. Mention,
   defer mechanism.
4. **Fiber-switch return path** — `switch_pending` branch after a primitive returns → **concurrency
   doc**. Mark.
5. **Non-local-return guard** — the `frames.len() >= frames_before` branch in the primitive-return
   handler is frame-identity territory → **Doc 6** (and already Doc 3). Mark.

## 7. Doc kind

**Fork + mechanism.** Fork: how to bind selector→method (static / vtable / dictionary), with the
honest note that Phalcom's coarse position is lineage and the *deliberated* fork is the finer
arity-vs-label-encoded axis (ADR-0012). Mechanism spine: the two-move resolve-then-enter and the
miss order. Hard case to trace (§5.5): **the miss path** — a send that resolves to nothing, reified
as a `Message` and forwarded to `doesNotUnderstand(_)` — not the textbook hit, which the reader's
intuition already handles.
