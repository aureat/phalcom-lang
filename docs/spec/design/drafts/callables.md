# Callables — the integrated model (`Function`, `Block`, `Method`, `Family`)

- Status: **Proposed** (experimental; consolidates + resolves — not ratified)
- Date: 2026-07-13
- Consolidates:
  [functions.md](../functions.md) (the `Function`/`Block`/`Method` tower) ·
  [blocks.md](../blocks.md) (block forms, non-local return, protected execution) ·
  [selectors.md §3](../selectors.md) (`::` `Family`, base-name index) ·
  [bound-callable-unification.md](../experimental/bound-callable-unification.md) (the `Family` ↔ `Method.bind` decision, adopted here)
- Depends on:
  [method-lookup.md](../method-lookup.md) (send, `perform`, `doesNotUnderstand`) ·
  [messages-and-selectors.md](../messages-and-selectors.md) (`SignatureKind`) ·
  [object-model.md](../object-model.md) (`self`, frame slot 0, metaclass) ·
  ADR-0006 (`Function` as abstract callable root)
- Related:
  [implicit-self.md](implicit-self.md) (bare `name()` self-send; `obj::m` as a read) ·
  [concurrency.md](../concurrency.md) (a `Function` as a coroutine entry)

This document is the **single integrated picture** of everything callable in Phalcom.
The pieces are specified separately across four docs; here they are one model, with
the two coexisting bound-callable routes ([functions.md §3](../functions.md) open
question) resolved into one. Where this doc takes a position beyond the sources, it
says so.

---

## 1. The one invariant

**There is exactly one compiled-code representation and exactly one closure
representation. Everything callable is a view over that closure, differing only in
what it wraps the closure *with*.**

```
Object
 └─ Function   (abstract)   the apply protocol — a code unit + an environment
     ├─ Block              closure + home-frame token           (no selector, no receiver)
     └─ Method             closure|primitive + signature + holder (receives self)

Family  (a *reference* to a Method-to-be, not a subclass of Function —
         convertible to/from a bound Method; §5–6)
```

`Function` is **abstract** ([object-model.md §2](../object-model.md)): no value has it
as direct class. `x.class` for any callable is `Block` or `Method`, never `Function`;
`x.is(Function)` is true for both. It exists only to give the two a shared protocol
and one place to hang application sugar (ADR-0006; amends blocks.md §7's "a method
*is* a Block" to "siblings under `Function`").

### The shared substrate (VM)

Both concrete forms wrap the same two Rust structs
([callable.rs](../../../phalcom-core/src/callable.rs),
[closure.rs](../../../phalcom-core/src/closure.rs)):

```rust
struct Callable      { chunk: Chunk, max_slots: usize, num_upvalues: usize,
                       arity: usize, name_sym: Symbol }       // the code unit
struct ClosureObject { callable: Callable, module: PhRef<ModuleObject>,
                       upvalues: Vec<Value> }                 // code + environment
```

`Block` and `Method` run through the **same** `CallFrame`
([frame.rs](../../../phalcom-core/src/frame.rs)) and the same VM value stack.

### The protocol every callable answers

| Signature | Meaning |
|---|---|
| `call` / `call(_)` / `call(_,_)` … | apply with N positional arguments |
| `callWith(_)` | apply with a `List` of arguments (reflective, variable arity) |
| `arity` | declared parameter count (`Int`) |
| `name` | display `Symbol`/`String` for diagnostics |

**Application sugar.** `f(a, b)` desugars to `f.call(a, b)` — the *only* place a value
is applied other than through a selector. The parser lowers postfix `(...)` on any
expression to a `call(_,…)` send. Arity mismatch raises `ArgumentError`.

---

## 2. `Block` — anonymous lexical closure

The value of `{ x => … }` and of an unbraced arrow `n => n * 2` ([blocks.md](../blocks.md)).

### Structure

- the shared `ClosureObject` (code + module + captured upvalues);
- a **home-frame token** — a frame pointer plus a generation counter — naming the
  method activation the block was created in.

A block has **no receiver and no selector**. `self` inside a block is the `self` of
its home method, captured as an ordinary upvalue.

### Forms (blocks.md §1–3)

```phalcom
n => n * 2                 // unbraced: single param, single-expression body, no return
{ acc, n => acc + n }      // braced: any arity; the brace is what makes the multi-param comma safe
{ System.print("hi") }     // braced, zero params
```

`=>` means exactly one thing everywhere: **"yields."** Same token in a block header
and a method expression body. The unbraced form is expression-only and
single-parameter — both restrictions exist to keep the argument/tuple comma
unambiguous and to make non-local return safe *by construction* (you cannot write a
JS-looking arrow containing `return`).

### Non-local return & escape (blocks.md §5)

`return` inside a block unwinds to the **home method's** frame and returns from *that*
method, not the block. A block may outlive its home frame (stored, returned, sent to
another fiber); on non-local return the home-frame token's generation is compared
against the live frame, and a mismatch raises `DeadFrameError` — a cheap integer
compare converting a memory-safety hazard into a clean runtime error. This is the
**escape ⊗ non-local-return** discipline: the dead-frame trap is designed in at the
block, not bolted on with `return`.

No `break` / `continue` in a block — those exist only in `while`/`for` sugar and
compile to jumps ([control-flow.md](../control-flow.md)).

### Blocks are the unit of protected execution (blocks.md §7)

```phalcom
blk.on(TypeError) { e => … }   // install a typed handler
blk.ensure { … }               // run on every exit path (finally)
blk.attempt()                  // run, capturing a throw into a Result
```

---

## 3. `Method` — selector-bound callable

The value stored in a class's method dictionary and the target a send resolves to.

### Structure

- a **kind** — either a `ClosureObject` (Phalcom code) or a native `PrimitiveFn` (Rust);
- a **signature** — the interned selector `Symbol` + a `SignatureKind`;
- a **holder** — a weak reference to the defining class (or its metaclass, for
  class-side `static`/`construct` methods).

A method **receives `self` in slot 0** of its frame. It is bound to a class but *not*
to an instance. Already present as `MethodObject`/`MethodKind`
([method.rs](../../../phalcom-core/src/method.rs)), surfaced as `Value::Method`.

### Signature & selector identity (selectors.md §1, messages-and-selectors.md)

A **selector** is the full method identity: base name + argument labels in declared
order, interned to one `Symbol`, the sole key for lookup (one hashmap hit, no overload
resolution at dispatch).

```
move(_,to,duration)     size()     +(_)     [](_)
```

`SignatureKind` distinguishes the callable's *shape*:
`Method(n)` · `Getter` · `Setter` · `Initializer(n)` · `SubscriptGet(n)` · `SubscriptSet(n)`.

| Rule | Statement |
|---|---|
| **R1** | Labels are identity — `move(_,to,duration)` ≠ `move(_,_)`; both may coexist on one class. |
| **R2** | Positionals precede labels; no interior positionals (`move(to,_)` illegal). |
| **R3** | Label *order* is identity — `move(to,duration)` ≠ `move(duration,to)`; no reordering. |
| **R4** | No label normalization (follows from R3 — reordering would need callee knowledge before dispatch, which is circular). |
| **R5** | Arity is implied = slot count; not stored separately. |

R3/R4 are structural: under dynamic dispatch the selector must be computable from the
call site alone. This is the **keyword-selector ⊗ evaluation-order** commitment — the
label field is identity-bearing, decided before dispatch became load-bearing.

### Interface (reflection + receiver-binding)

| Signature | Meaning |
|---|---|
| `signature` | the `Signature` (selector + kind) |
| `selector` | the interned selector `Symbol` |
| `holder` | the defining `Class` / metaclass |
| `isPrimitive` | native vs. Phalcom-compiled |
| `invokeOn(recv, args)` | apply to an explicit receiver + argument `List` (a **raw activation** — never a send, see §7) |
| `bind(recv)` | close over a receiver → a `Family` (§6) |

`recv.methodFor(sel)` ([object-model.md §8](../object-model.md), via `perform`) reifies
the method a selector resolves to, so methods are extractable as values:

```phalcom
let g = 3.methodFor(#+(_))
g.invokeOn(3, [4])          // => 7
```

---

## 4. Symbols name selectors *or* families (selectors.md §2)

Two literal types, both backed by an interned `Symbol`, distinguished by whether
labels are present:

| Literal | Type | Identifies | Used for |
|---|---|---|---|
| `#move` | **Name symbol** | a *family* (base name, all arities/kinds) | `respondsTo`, map keys, reflection |
| `#move(_,to,duration)`, `#+`, `#[]` | **Selector symbol** | one complete method identity | `perform`, pinned references |

`perform` accepts **only** selector symbols; a name symbol is a type error. This is the
seam between "the whole family under a name" and "one specific method."

---

## 5. `Family` — the callable reference (`::`)

`::` produces a **`Family`**: a callable value that is a *reference* to a method-to-be,
not a reified `Method`. Two axes — open vs pinned, bound vs unbound.

```phalcom
obj::move                    // Open,   bound   — receiver fixed, name only
obj::#move(_,to,duration)    // Pinned, bound   — receiver fixed, selector fixed
Point::move                  // Open,   unbound — receiver is the first argument
Point::#move(_,to,duration)  // Pinned, unbound
```

Grammar is LR(1)-clean: after `::`, peek for `#`.

```rust
enum Family {
    Open   { recv: Option<Value>, name: Symbol },      // recv: None = unbound
    Pinned { recv: Option<Value>, selector: Symbol },
}
```

### Semantics — a family call *is* a send

**Open** families resolve **at call time**: the call site knows its own labels
statically, so the selector is built from `family.name` + the call's label suffix,
then dispatched as an ordinary send.

```phalcom
let f = obj::move
f(to: p, duration: 2)   // dispatches move(to,duration)
f(p, 2)                 // dispatches move(_,_)
```

Consequences: an Open family is **never stale** (the table is consulted every call =
**late-bound**), and an unbound `Point::move` dispatches on the *actual receiver
passed in*, so subclass overrides work.

**Pinned** families have the selector fixed at compile time — no re-interning, straight
to the send. The fast path, and the way to name one specific overload.

There is **no second dispatch mechanism**: an Open call builds its selector then enters
the ordinary send path; a Pinned call skips straight to it; a miss is a normal
`doesNotUnderstand`, enriched with the family's candidate list.

### Base-name index (selectors.md §3.1)

Built per class at finalization, flattened through inheritance like a vtable:

```rust
base_names: HashMap<Symbol /* "move" */, SmallVec<[Symbol; 2]> /* full selectors */>
```

Serves three jobs: the empty-family check, the candidate list in error messages, and
reflection.

### Error behavior

| Situation | Behavior |
|---|---|
| Empty family (`obj::typo`, no such base name) | Error **at reference time**, naming the class (checked against the base-name index). |
| Empty family but class defines `doesNotUnderstand` | **Not** an error — routes to DNU. Check is `empty && no DNU hook`. |
| Call-time miss (labels match no member) | Ordinary `doesNotUnderstand`, enriched with the candidate list. |
| Miss where labels are a strict subset of exactly one candidate | Report the *specific* missing label (`missing label 'duration'`). |

---

## 6. The unification — `Family` and `Method.bind` are two views of one concept

[functions.md §3](../functions.md) flagged that two routes to a receiver-closed
callable coexist without unifying. **This doc adopts
[bound-callable-unification.md](../experimental/bound-callable-unification.md): two
views of one concept, each convertible — not two types.** It mirrors the
Block/Method-under-`Function` move.

| | `Family` (the reference) | `Method.bind(recv)` (the reified form) |
|---|---|---|
| What it holds | lazy `(receiver, name|selector)` | an actual reified `Method` + pinned receiver |
| Binding time | resolves on **call** — **late-bound** | fixed to **today's definition** — **early-bound** |
| Redefinition | **survives** it (re-dispatches) | pinned to the definition captured at `bind` |
| Cost | cheap; no `Method` reified | reifies a `Method` |
| Bridge | `family.reify → Method` | `method.bind → Family` |

Both answer `Function`'s `call`/`arity` protocol, so **every higher-order API takes a
`Function` and never branches on which route produced it**. The late/early-bound
distinction stays *observable* (redefinition semantics differ) — collapsing it away
would be wrong, so the two forms are kept, unified by protocol rather than merged into
one type.

Rule of thumb: reach for `obj::m` (Family) for a live reference that tracks
redefinition; reach for `method.bind(obj)` when you want a snapshot pinned to the
current definition.

---

## 7. Calling — every route funnels to one send

Phalcom has exactly **two** ways to invoke, and the first is sugar for a send:

1. **Selector send** — `recv.sel(args)`, and its sugar forms:
   - bare `sel(args)` inside a method → `self.sel(args)` ([implicit-self.md](implicit-self.md));
   - `f(args)` on a callable value → `f.call(args)` (§1), which is itself a `call`
     *send* to the `Function`;
   - a `Family` call (§5) → builds/uses a selector, then an ordinary send;
   - trailing block sugar — `xs.map { n => … }` passes the block as the final argument.
2. **Raw activation** — `method.invokeOn(recv, args)`: applies a *specific* reified
   `Method` to `recv` **without lookup**.

The one invariant that keeps decorators/proxies sound: **`invokeOn` must never miss.**
A decorator's `wrap(m)` calls `m.invokeOn(self, args)` on the *original* method; if
that went through a send it would re-trigger the very interception it wraps
(reentrancy — see [proxy.md](proxy.md), [decorators.md](decorators.md)). `invokeOn` is
therefore a direct frame activation, not a `perform`.

```phalcom
xs.reduce(0) { acc, n => acc + n }   // send `reduce(_)` + trailing block
f.call(1, 2)  //  ≡  f(1, 2)          // apply a callable value  (a `call` send)
obj::move(to: p, duration: 2)         // family → ordinary send of move(to,duration)
m.invokeOn(obj, [1, 2])               // raw activation of a specific Method, no lookup
```

---

## 8. Reflection surface (the metaobject gate)

The receiver-binding and reflection entry points (`invokeOn`, `bind`, `methodFor`,
`signature`, `holder`, `Behavior.defineMethod`) are the same **object-model §8
metaobject surface** the decorator/proxy/attribute specs gate on
([decorators.md D-2](decorators.md)). Ship the read+execute half (`invokeOn` as raw
activation, `signature`, `bind`, `methodFor`, base-name reflection) first; scope
`defineMethod`'s inline-cache cost to the class-definition path. Until §8 is ratified,
`Family` and `bind` may *function* (compiler/definition-driven) but the reflective
read API stays behind that gate.

---

## 9. One representation, four views

| | shares `ClosureObject` | selector + holder | home frame | receives `self` | binding |
|---|:--:|:--:|:--:|:--:|:--:|
| `Function` (abstract) | — | — | — | — | — |
| `Block` | ✓ | — | ✓ | inherited (upvalue) | n/a |
| `Method` | ✓ (or primitive) | ✓ | — | ✓ (slot 0) | early (its definition) |
| `Family` | — (references one) | name **or** selector | — | supplies / takes recv | **late** (re-dispatches) |

`Block` and `Method` differ only in what they wrap the closure with (a home frame vs. a
selector + holder). `Family` is not a fourth closure — it is a *reference* that
resolves to a `Method` via a send, convertible to a reified bound `Method` and back.

A `Function` also becomes a coroutine entry point ([concurrency.md](../concurrency.md)):
a fiber captures the fiber-floor for non-local return across the coroutine boundary,
the same home-frame discipline as §2.

---

## 10. Hazards

- **Escape ⊗ non-local return.** A block that outlives its home frame must trap a
  `return` through the dead frame (`DeadFrameError`), not corrupt the stack. Designed
  in via the home-frame generation token (§2), not retrofitted.
- **`invokeOn` reentrancy.** If `invokeOn` were a send it would re-enter decorator /
  proxy interception. It must be a raw activation that never misses (§7).
- **Open family staleness vs speed.** Open families re-dispatch every call (never
  stale, subclass-correct) at the cost of building the selector each time; a
  monomorphic IC keyed by `(call_site, class_id)` collapses the intern step after the
  first hit, so an Open call costs the same as a normal send.
- **Late/early-bound confusion.** `obj::m` tracks redefinition; `m.bind(obj)` does not.
  Two *observably different* semantics unified by protocol, not merged — a user who
  wants one and gets the other sees a redefinition behave "wrong." Document which each
  API returns.
- **`Value`-arm gap (implementation).** First-class `Block` is not yet realized: no
  `Value::Block` arm, no `Closure`/`GetUpvalue`/`SetUpvalue` opcodes, no home-frame-
  carrying `Return` ([functions.md §2 impl](../functions.md)). `Method` exists;
  `Block`/`Family` are design-ahead.

---

## 11. What this precludes

- **A second dispatch mechanism.** A family call, an `f(...)` apply, and a bare
  `name()` self-send all funnel to the one send path (or, for `invokeOn`, one raw
  activation). No callable introduces a parallel lookup.
- **Merging `Family` into `Method`.** The late/early-bound distinction is load-bearing;
  §6 keeps them as convertible views, never one type. Collapsing them would erase
  observable redefinition semantics.
- **Overload resolution at dispatch.** Labels-as-identity (R1–R5) means the selector is
  the key; there is no arity-family search or keyword reordering at send time. A
  feature needing callee-known label order before dispatch is foreclosed.
- **`Function` as a concrete class.** It stays abstract; no value is directly a
  `Function`. Anything wanting to *be* callable is a `Block` or `Method`, or references
  one via `Family`.

---

## 12. Open questions

| # | Question |
|---|---|
| C-1 | Does `Family.reify` on an **Open** (name-only) family require a call-site label suffix to pick a selector, or reify the whole candidate set? Pinned reifies unambiguously; Open is a family, not a method. |
| C-2 | `bind` returns a `Family` (§6) — but a `Pinned`-bound one (early, per the table) or does `bind` stay the sole *early* form with `Family` always *late*? The bridge direction needs one fixed answer. |
| C-3 | Should `f(args)` apply-sugar and a bare `name()` self-send ([implicit-self.md](implicit-self.md)) ever collide — i.e. is a local callable `name` invoked as `name()` a `call` send or a self-send? (Resolved by implicit-self tier-1: local wins. Cross-reference, don't re-decide.) |
| C-4 | First-class `Block` realization order: `Value::Block` arm + upvalue opcodes + home-frame `Return` — one unit, or split capture (opcodes) from escape (dead-frame trap)? |
| C-5 | Does `invokeOn` on a **primitive** `Method` (native `PrimitiveFn`) share the raw-activation path, or need a distinct native-call entry that still never misses? |
