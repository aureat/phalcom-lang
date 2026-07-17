# `docs/learn` — message send: requirements, approach, checklist

Working folder. Scratch. The shipped doc is `docs/learn/vm/message-send.md` (VM track, Doc 4);
everything here is state used to build it. Grip and design space copied from `recon.md`, grounded.

## 0. The obligation

One test, and it is the whole spec:

> **After reading, the reader could re-derive Phalcom's choice from the constraints alone.**

Delete the source. Hand the reader the pressures. Could they rebuild it? A doc that only describes
what `send.rs` and `invoke_at` do has failed, however accurate.

Corollary: every branch not taken must be made **genuinely tempting** before it is rejected. A
strawman teaches Phalcom's answer without teaching the question.

## 1. Reader

Knows PL design — has used dynamic dispatch, virtual methods, maybe Smalltalk or Ruby. Not fluent
in runtime implementation. Stated weakness: **cannot hold moving-state mechanisms in their head**,
lacks stable notation, so complexity accretes until the thread is lost. The doc hands over a
**grip**, not completeness.

Inherited truth from Docs 1–3 (assume the reader has them): the VM is one `while` loop over a
`match` (Doc 1); *what* runs is a `Callable` recipe instantiated into a `ClosureObject` (Doc 2);
*where* it runs is a `CallFrame` — a `Copy` value pushed onto `VM::frames`, whose caller is the slot
one below, no pointer (Doc 3). Doc 3 already established the **method push site** (`send.rs` ~L113)
and that `new_call_frame` builds-but-does-not-push, with four push sites. **This doc owns the site
Doc 3 pointed forward to: what a call site *does* to select and enter a method.**

## 2. Doc kind

**Fork + mechanism** (recon §7).

- **Fork** — how to bind a selector to a method: static/early binding, vtable-offset binding,
  dynamic-dictionary lookup. Live decision, real occupants on each branch. This is the spine.
  Honesty caveat (§5.2 territory): the *coarse* fork is **lineage, not a bake-off** — Phalcom is
  Smalltalk/Wren-lineage dictionary dispatch, the same way Doc 1's stack-machine was lineage. The
  **deliberated** fork is finer: *given* dictionary dispatch, key on arity-only or on full
  label-encoded selectors? That is ADR-0012, and it carries the real scar. The doc presents the
  coarse fork as pedagogical scaffolding and lands the deliberated choice on the finer axis.
- **Mechanism** — the two-move spine: **resolve** (walk the receiver's class chain) then **enter**
  (`call_method` forks on `MethodKind`). And the miss order.
- **Stateful trace earns its place** — but trace the **counterintuitive** case: not the textbook
  hit (reader's intuition already handles it), but the **miss path** — a send resolving to nothing,
  reified as a `Message` and forwarded to `doesNotUnderstand(_)`.

## 3. The grip (grounded — from recon §2)

> **A call site names a selector, not a method. Sending is *resolve then enter* — find the method
> by walking the receiver's class chain, then run it — and both "which code runs" and "does a frame
> even appear" are decided *after* the call site, by the receiver, never by the caller.**

Corollary that ties to Doc 3: **not every send pushes a frame.** A primitive send (`1 + 2`) runs
native Rust in place and pushes zero `CallFrame`s. The frame push is one arm of `call_method`, not a
property of "calling." This is the secondary predict-then-check.

The one-line representation fact that settles it: *the compiled call site holds a selector-constant
index and an empty inline-cache slot — no method handle, no vtable offset, no resolved address.
Which code runs, and whether a frame appears, is decided at runtime from the receiver's class chain
and the method's kind.*

## 4. The design space (walked, not listed)

The problem: source says `p foo`. Between that text and the code that runs sits **selector→method
binding**. When is it bound, and to what?

| Branch | Occupants | The bill |
|---|---|---|
| **Static / early binding** — resolved at compile time to a fixed target | C (direct call), non-virtual C++, most function calls everywhere | Fastest possible: a direct jump, inlinable. But the target is frozen at compile time — no polymorphism, no overriding by receiver, no defining the method after the call site is compiled. |
| **Vtable / offset binding** — compile a fixed *slot number*; receiver carries a table; call `table[slot]` | C++ virtual, Java (`invokevirtual`), Go interfaces | One indirection, still very fast, real subtype polymorphism. Bill: the slot layout is fixed at class-compile time — you cannot add a method to an existing class at runtime (no monkeypatch), and a genuine miss is a *type/link error*, not a runtime event the program can catch. |
| **Dynamic dictionary lookup** — compile only the *selector*; at send time hash it against the receiver-class's method dictionary, walk superclasses on miss | Smalltalk, Ruby, Objective-C, Python (roughly), **Phalcom** | Latest binding possible: define/redefine methods at runtime, receiver fully decides, a miss is a *first-class runtime event* (`doesNotUnderstand`/`method_missing`/`__getattr__`). Bill: a hash + chain-walk per send unless cached; invalidation cost when the world changes. |

The coarse three-way choice above is **lineage** for Phalcom (Smalltalk/Wren), not a deliberated
bake-off — the doc must say so. The deliberated fork (ADR-0012) is *inside* the dictionary branch:

| Finer branch (given dictionary dispatch) | Bill |
|---|---|
| **Arity-only key** — `SignatureKind::Method(u8)`, the pre-ADR tree | Cannot distinguish `move(to,duration)` from `move(_,_)`; forces malformed encodings. **Rejected.** Scar: shipped defects F1 (Invoke dropped `call_method`'s `Result`, swallowing primitive errors), F7 (0-arg `new()` mis-tagged `Method(1)`), F8 (divergent encoder interned `">( _)"` with a stray space). |
| **Full label-encoded selector** — name + arg labels baked into the interned `Symbol` | One hashmap probe per class; `move(to,duration)` and `move(_,_)` are distinct keys. **Chosen** (ADR-0012). |

## 5. Comparison filter

A language enters **only** if it (recon §4 cast): (1) took another branch with the bill attached;
(2) has a scar; (3) names something Phalcom does anonymously; (4) is an ancestor. Expect ~5–6 to
survive. Name the cut list.

Vocabulary to import (deliverable in itself): *selector*; *dynamic dispatch* / *late binding*;
*method dictionary*; *`doesNotUnderstand`* / *method_missing* / *`__getattr__`*; *reified message*
(`Message` object); *vtable* / *itable*; *`objc_msgSend`* (send-is-a-C-function); *monomorphic
inline cache* (name it, defer mechanism → Doc 5); *MRO / C3 linearization* (name it to say we don't
need it).

Provisional cast, subject to earning it:

- **Smalltalk** — the ancestor. Names `doesNotUnderstand`, `selector`, `Message`, `perform:`.
  Everything-is-a-send. **Deep.**
- **Objective-C** — `objc_msgSend`, selector→IMP, the *send is literally a C function call* reframe;
  message forwarding as the miss path in a static-typed host. **Deep, one sharp section.**
- **C++ virtual** — the vtable branch, with the monkeypatch bill (cannot add a method to a compiled
  class) and the miss-is-a-link-error contrast. **Medium.**
- **Ruby** — `method_missing`, open classes → the invalidation cost that motivates caches. **Medium.**
- **CPython** — MRO/C3 and `__getattr__`; single-vs-multiple-inheritance contrast (Phalcom is single,
  so the walk is a trivial climb — establish *why* MRO exists so the doc can say "we don't need it").
  **Short.**
- **Wren** — direct lineage (Phalcom copies its dispatch shape). Fold in where it earns it; likely
  **short** or cut into the lineage note.

Likely **cut** (name them): Java (redundant with C++ vtable for this axis unless the `invokevirtual`
vs `invokedynamic` distinction earns a line); JS prototype-chain lookup (a different axis —
delegation, not class dictionary — risks muddying the fork).

## 6. Tensions to surface

- **Selector encoding ⊗ overloading** — labels baked into the symbol mean `foo(_)` and `foo(_,_)`
  coexist as distinct methods; there is no arity-overload resolution step because arity *is* part of
  the key. Ties the deliberated ADR-0012 choice to a visible language feature.
- **Send ⊗ frame push** — the corollary grip: primitive sends push zero frames. `call_method`'s
  `MethodKind` fork is where "run a method" splits into "native in place" vs "push a `CallFrame`."
  Direct tie to Doc 3.
- **Miss ⊗ recursion guard** — `doesNotUnderstand` is itself a send; sending it must not re-enter
  `doesNotUnderstand` on a class that lacks it. The guard is the mechanism.
- **Send ⊗ super** — `SuperSend` (ADR-0040) starts the walk *above* the receiver's class; own
  opcode, bypasses the receiver-class start. Mention, defer full mechanism.
- **Constructors ⊗ dispatch** — `Foo.new()` is an ordinary class-side method resolved by the same
  walk (ADR-0063); no constructor special-case in `lookup_method`. Reinforces "everything is a
  selector."

## 7. Structural rules (constraints, not a skeleton)

- **Structure follows the theory.** No imposed heading set. Bottoms out where the theory bottoms out.
- **No checkbox comparative table.** Comparison is a weapon aimed at one named confusion.
- **Trace the miss path, and that as the hard case.** The hit is the reader's intuition; the miss
  (reify → `doesNotUnderstand`) is the strange one. Trace from real observed output.
- **Present the coarse fork as scaffolding**, land the deliberated choice on the finer axis (§5.2).
- **Mermaid only where the shape is the point** — the resolve-then-enter two-move, or the miss
  ladder. Not decoration; do not draw a vtable Phalcom doesn't have.
- **Source anchors: symbol first, line second** (`vm/send.rs::VM::call_method` @ ~L19). Bare line
  numbers rot.
- **HEAD as-implemented.** Cite spec/ADR intent as intent.
- **Mark every simplification as a lie with a forward pointer.** (Recon §6 list below.)

## 8. Lies to mark forward (recon §6)

1. **Inline cache** — `caches[cache_ip]` / `world_version` probe + refill in `invoke_at` short-
   circuits the chain walk after the first send. Present lookup as "walk every send," mark the cache
   → **Doc 5 (caches-and-fusion)**.
2. **Fusion** — `InvokeLocal`/`InvokeConst` superinstructions fold a preceding load into the send
   → **Doc 5**.
3. **SuperSend** (ADR-0040) — starts the walk above the receiver's class; own opcode. Mention, defer.
4. **Fiber-switch return path** — `switch_pending` branch after a primitive returns → **concurrency
   doc**. Mark.
5. **Non-local-return guard** — the frame-identity branch in the primitive-return handler → **Doc 6**
   (and Doc 3). Mark.

## 9. Checklist (gate before shipping — maps to AUTHORING §6)

- [ ] Grip stated early, one sentence, *earned* by the end.
- [ ] Call site shown to hold a **selector**, not a method — explicitly, from the `Invoke` operand.
- [ ] Every rejected branch made tempting before it is killed.
- [ ] The **deliberated** finer fork (ADR-0012) landed, with the F1/F7/F8 scar, and the coarse fork
      labelled pedagogical scaffolding.
- [ ] ≥1 predict-then-check moment (primary: what does `Invoke` hold; secondary: does every send
      push a frame).
- [ ] The **miss path** traced step by step, from real observed output (`doesNotUnderstand`).
- [ ] `call_method`'s `MethodKind` fork explained — primitive in place vs closure pushes a frame.
- [ ] Single-inheritance chain-walk shown; MRO named only to say we don't need it.
- [ ] Every language present passes §5; named cut list.
- [ ] Vocabulary imported and findable.
- [ ] Anchors symbol-first and exist at HEAD.
- [ ] Every lie marked with a forward pointer.
- [ ] Claims ledger clean: perf/comparative claims cited, labelled unverified, or cut; links resolve.
- [ ] Reader could re-derive the design. (§0)

## 10. Build sequence

| # | Deliverable | Who | Path |
|---|---|---|---|
| 1 | `recon.md` | me | done |
| 2 | This file | me | `REQUIREMENTS.md` |
| 3 | Theory draft — no source access | sonnet A | `draft-concept.md` |
| 4 | Source map — graphify-led, fixtures run live | sonnet B | `source-map.md` |
| 5 | The doc — synthesis, my judgment over A's bulk + B's ground truth | me | `../vm/message-send.md` |

## 11. Open risk

Recon is bounded; Agent B goes deeper. Assumptions the doc rests on that B must confirm or the doc
bends:

1. **`Invoke` operand carries a selector-constant index, not a method/offset.** Grip §3 assumes it.
   If B finds the operand resolves anything at compile time, §3 is wrong. *(Recon read the operand
   shape `Invoke(arity, selector_idx)` at dispatch.rs ~L1024 — high confidence, but B verifies.)*
2. **`call_method` forks on `MethodKind::{Primitive,Closure}` and only the closure arm pushes a
   frame.** The corollary grip and the Doc-3 tie both rest on this. If a primitive *also* pushes a
   frame, the "not every send pushes a frame" beat dies. B must read the full body.
3. **The miss path fires a *user-overridable* `doesNotUnderstand(_)` at runtime**, not a Rust panic
   or a compile error. The whole "miss is a first-class event" fork-payoff depends on it. B must
   **run a fixture** and report the observed message, not read it off code.
4. **The coarse static/vtable/dictionary fork was *not* deliberated; only the finer arity-vs-label
   axis (ADR-0012) was.** If B finds an ADR deliberating the coarse choice, §5.2's honesty framing
   flips. *(Recon found only ADR-0012 on the finer axis — B does a bounded confirm.)*
5. **A method defined/monkeypatched after compile takes effect on an already-compiled call site.**
   The "late binding" claim rests on it. B runs a fixture (define-after-use, or reopen a class).
