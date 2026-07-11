# Functions, Blocks & Methods

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

This part specifies the **callable tower**: the three classes that share one
runtime representation — a compiled code unit plus a captured environment.

```
Object
 └─ Function   (A)   the call protocol; a code unit + an environment
     ├─ Block  (U)   anonymous lexical closure; non-local return
     └─ Method (U)   selector-bound, holder-bound, receiver-taking
```

> **Amendment to [Blocks](blocks.md).** Blocks §7 said "a method *is* a `Block`."
> The precise relationship is: `Block` and `Method` are **siblings** under the
> abstract `Function`, sharing the closure representation. A `Method` is not a
> `Block` — it carries a selector, a holder, and a receiver a `Block` does not.
> What they share is `Function`'s protocol and the VM's `ClosureObject`. See
> [ADR 0006](../adr/0006-function-as-abstract-callable-root.md).

---

## 1. `Function` — the abstract callable

`Function` is abstract ([Object Model §2](object-model.md)): no value has
`Function` as its direct class. It defines the one thing common to everything
callable — you can **apply** it.

### Structure

A `Function` is, conceptually, a pair:

- a **code unit** — a compiled bytecode `Chunk` with fixed `arity`, slot count,
  and upvalue count (the VM's `Callable`);
- an **environment** — the enclosing `Module` plus captured `upvalues`.

`Function` stores no fields of its own; the pair is materialized by its concrete
subclasses. It exists to give `Block` and `Method` a shared protocol and a single
place to hang function-application sugar.

### Interface

| Signature | Meaning |
|-----------|---------|
| `call` | apply with no arguments |
| `call(_:)`, `call(_:_:)`, … | apply with N positional arguments |
| `callWith(_:)` | apply with a `List` of arguments (reflective, variable arity) |
| `arity` | declared parameter count (`Number`) |
| `name` | a display `Symbol`/`String` for diagnostics |

**Application sugar.** `f(a, b)` desugars to `f.call(a, b)` ([Blocks §7](blocks.md)).
This is the *only* place a value other than through a selector is "called": the
parser lowers postfix `(...)` on any expression to a `call(_:…)` send.

**Arity mismatch** raises `ArgumentError` ([Object Model §4](object-model.md)).

### Implementation

The VM already carries the shared substrate
([`callable.rs`](../../phalcom-core/src/callable.rs),
[`closure.rs`](../../phalcom-core/src/closure.rs)):

```rust
struct Callable { chunk: Chunk, max_slots: usize, num_upvalues: usize,
                  arity: usize, name_sym: Symbol }
struct ClosureObject { callable: Callable, module: PhRef<ModuleObject>,
                       upvalues: Vec<Value> }
```

`Function` is **not** a Rust variant of its own; it is the abstract class the two
concrete `Value` representations answer to. `x.class` for any callable returns
`Block` or `Method`, never `Function`. `isA(Function)` is true for both.

---

## 2. `Block` — anonymous lexical closure

A `Block` is a first-class closure literal ([Blocks](blocks.md)): the value of
`{ x => … }` and of an unbraced arrow `n => n * 2`.

### Structure

- the shared `ClosureObject` (code + module + captured upvalues);
- a **home-frame token** — a frame pointer plus a generation counter — naming the
  method activation the block was created in ([Blocks §5](blocks.md)). This is
  what a `return` inside the block unwinds to.

A block has **no receiver and no selector.** `self` inside a block is the `self`
of its home method, captured like any other upvalue.

### Interface

Everything on `Function`, plus:

| Signature | Meaning |
|-----------|---------|
| `call` … | apply; binds parameters, runs the body, yields the last expression |
| `isClosed` | whether the home frame is still live (see below) |

- **Non-local `return`** unwinds to the home frame ([Blocks §5](blocks.md)).
- Applying a block whose home frame is dead raises `DeadFrameError`.
- No `break` / `continue` — those exist only in loop sugar
  ([Control Flow](control-flow.md)).

### Implementation

Currently unrealized: closures exist in the VM only *inside* a `MethodObject`
(`MethodKind::Closure`), and the `Value` enum has no `Block` arm
([`value.rs`](../../phalcom-core/src/value.rs)). Making blocks first-class
requires:

1. a `Value::Block(PhRef<BlockObject>)` arm, where `BlockObject` wraps a
   `ClosureObject` plus the home-frame token;
2. `Closure`, `GetUpvalue`, and `SetUpvalue` opcodes
   ([`bytecode.rs`](../../phalcom-core/src/bytecode.rs) has none yet) so the
   compiler can capture upvalues at the block-literal site;
3. a `Return` variant that carries the home-frame token and unwinds the
   `CallFrame` stack to it, comparing the generation counter and raising
   `DeadFrameError` on mismatch.

Blocks and methods run through the **same** `CallFrame`
([`frame.rs`](../../phalcom-core/src/frame.rs)) and the same VM value stack; a
block pushes a frame whose `context` is inherited from its home activation.

---

## 3. `Method` — selector-bound callable

A `Method` is a reified compiled method: the value stored in a class's method
dictionary and the target a message send resolves to
([Method Lookup](method-lookup.md)).

### Structure

- a **kind** — either a `ClosureObject` (Phalcom code) or a native
  `PrimitiveFn` (Rust);
- a **signature** — the interned selector `Symbol` plus a `SignatureKind`
  (`Method(n)`, `Getter`, `Setter`, `Initializer(n)`, `SubscriptGet/Set(n)`)
  ([Messages & Selectors](messages-and-selectors.md));
- a **holder** — a weak reference to the defining class.

A method **receives `self`** in slot 0 of its frame; it is bound to a class but
*not* to an instance.

### Interface

Everything on `Function`, plus reflection and receiver-binding:

| Signature | Meaning |
|-----------|---------|
| `signature` | the `Signature` (selector + kind) |
| `selector` | the interned selector `Symbol` |
| `holder` | the defining `Class` (or its metaclass, for class-side methods) |
| `isPrimitive` | native vs. Phalcom-compiled |
| `bind(_:)` | close over a receiver → a zero-`self` `Function` (a `Block`) |
| `invokeOn(_:_:)` | apply to an explicit receiver + argument `List` |

`recv.methodFor(_:)` ([Object Model §8](object-model.md), via `perform`) reifies
the method a selector resolves to, so methods can be extracted and passed as
values: `let g = 3.methodFor(#"+(_)")`; `g.invokeOn(3, [4])` → `7`.

`m.bind(receiver)` is how "a method becomes a callable value": it yields a
`Function` that supplies `receiver` as `self`, reusing the `Block` machinery. This
is the precise meaning of the old "a method bound to a class."

### Implementation

Already present as `MethodObject` / `MethodKind`
([`method.rs`](../../phalcom-core/src/method.rs)) and surfaced as
`Value::Method(PhRef<MethodObject>)`. What the spec adds beyond today's tree:

- `bind(_:)` (needs first-class `Block`, §2);
- `invokeOn(_:_:)` and `methodFor(_:)` reflective entry points;
- class-side methods (`static`, `construct`) already register on the metaclass
  ([Classes §1](classes.md)); no change to storage, only reflection surface.

---

## 4. One representation, three views

| | shares `ClosureObject` | has selector + holder | has home frame | receives `self` |
|---|:---:|:---:|:---:|:---:|
| `Function` (A) | — (abstract) | — | — | — |
| `Block` | ✓ | — | ✓ | inherited |
| `Method` | ✓ (or primitive) | ✓ | — | ✓ (slot 0) |

The invariant: **there is exactly one compiled-code representation** (`Callable`)
and exactly one closure representation (`ClosureObject`). `Block` and `Method`
differ only in what they wrap that closure *with* — a home frame vs. a selector
and holder — and `Function` is the protocol both answer.

See [Fibers & Futures](concurrency.md) for how a `Function` becomes a coroutine
entry point, and [Implementation Status](implementation-status.md) for the gap
between this design and the current tree.
