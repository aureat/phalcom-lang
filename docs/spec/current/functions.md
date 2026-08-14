# Functions, Blocks & Methods

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

This part specifies the **callable tower**: concrete callable values share
one shape-aware activation gateway over compiled code and captured state.

```
Object
 ├─ Function   (A)   the common shape-aware call protocol
 │   ├─ Closure            anonymous lexical closure; non-local return
 │   ├─ BoundMethod        exact method plus stored receiver
 │   ├─ Family             exact-selector or structural-pattern reference
 │   └─ BoundMethodFamily  immutable reflection snapshot plus receiver
 ├─ Method                reified selector/holder; applied through invokeOn
 └─ MethodFamily         immutable reflection snapshot; bind(_) makes it callable
```

> **Amendment to [Blocks](blocks.md).** Blocks are first-class Function
> values. Reified `Method` values remain selector-bound reflection objects;
> `Method#invokeOn(_,***)` validates and applies them through the same flat
> activation machinery, without making bare `Method` a raw-call receiver.

---

## 0. Declaration parameters and selector identity

Method declarations separate external selector labels from body-local names:

```phalcom
foo(_ x)                    // selector foo(_), local x
foo(label)                  // selector foo(label), local label
foo(label local)            // selector foo(label), local local
foo(_ x, label y)           // selector foo(_,label), locals x and y
foo(_ x, *rest)             // variadic tail; local rest
```

Standalone `_` marks a positional selector slot; it is not a local binding.
Call-site labeled syntax is unchanged: `foo(label: value)`. Declaration forms
such as `foo(x)` for a positional parameter, `foo(label:)`, and
`foo(label: local)` are invalid.

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
subclasses. It gives concrete callable values one place to hang function-application
sugar and keeps activation independent of each concrete representation.

### Interface

| Signature | Meaning |
|-----------|---------|
| `call(***)` | apply one complete positional/labeled argument shape |
| `callWith(_)` | apply a complete `Unit` or `Tuple` argument pack |
| `arity` | declared parameter count (`Int`) |
| `name` | a display `Symbol`/`String` for diagnostics |

**Application sugar.** `f(a, b)` desugars to `f.call(a, b)` ([Blocks §7](blocks.md)).
This is the *only* place a value other than through a selector is "called": the
parser lowers postfix `(...)` on any expression to a `call(_,…)` send.

The concrete callable validates its accepted shape. **Arity mismatch** raises
`ArgumentError` ([Object Model §4](object-model.md)).

### Implementation

The VM already carries the shared substrate
([`callable.rs`](../../../phalcom-core/src/callable.rs),
[`closure.rs`](../../../phalcom-core/src/closure.rs)):

```rust
struct Callable { chunk: Chunk, max_slots: usize, num_upvalues: usize,
                  arity: usize, name_sym: Symbol }
struct ClosureObject { callable: Callable, module: PhRef<ModuleObject>,
                       upvalues: Vec<Value> }
```

`Function` is **not** a Rust variant of its own; it is the abstract class that
Closure, BoundMethod, and Family values answer to. `x.class` returns the
concrete callable class, never `Function`; a bare `Method` is reified metadata
and uses `invokeOn(_,***)` for application.

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
| `call(***)` | apply accepted shape; binds parameters and runs the body |
| `isClosed` | whether the home frame is still live (see below) |

- **Non-local `return`** unwinds to the home frame ([Blocks §5](blocks.md)).
- Applying a block whose home frame is dead raises `DeadFrameError`.
- No `break` / `continue` — those exist only in loop sugar
  ([Control Flow](control-flow.md)).

### Implementation

Implemented by the VM's `Object::Block`/`Object::Closure` representations,
home-frame tokens, and the flat Function gateway. Positional rest binds to
a `Tuple`, or `Unit` when empty; labeled arguments are rejected by Closure
activation. Escaping-block dead-frame fencing remains enforced.

Blocks and methods run through the **same** `CallFrame`
([`frame.rs`](../../../phalcom-core/src/frame.rs)) and the same VM value stack; a
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

Reflection and receiver-binding:

| Signature | Meaning |
|-----------|---------|
| `signature` | the `Signature` (selector + kind) |
| `selector` | the interned selector `Symbol` |
| `holder` | the defining `Class` (or its metaclass, for class-side methods) |
| `isPrimitive` | native vs. Phalcom-compiled |
| `bind(_)` | close over a receiver → a zero-`self` `Function` (a `Block`) |
| `invokeOn(_,***)` | apply to an explicit receiver plus a complete argument shape |

`recv.methodFor(_)` ([Object Model §8](object-model.md), via `perform`) reifies
the method a selector resolves to, so methods can be extracted and passed as
values: `let g = 3.methodFor(#+(_))`; `g.invokeOn(3, ***(4,))` → `7`. (`#+(_)` is a
bare selector-symbol literal, comma form — see
[Selectors, Symbols & References §2](selectors.md#2-symbol-literals-).)

**Relationship to `::` families.** `Method.bind`/`methodFor`/`invokeOn` above and
[Selectors, Symbols & References §3](selectors.md#3-method-references-) (`::`
`Family`) produce different concrete Function values, but both enter through
Function's shape-aware `call(***)` gateway. Family routing is explicit runtime
activation; it does not rely on an intentional `doesNotUnderstand(_)` miss.

`m.bind(receiver)` produces a `BoundMethod` Function value that supplies
`receiver` as `self`. Its `call(***)` activation and `invokeOn(_,***)` use
the same exact method target; neither path redispatches by selector.

### Implementation

Implemented by `MethodObject`, `BoundMethodObject`, and the shape-aware
`invokeOn(_,***)`/`bind(_)` gateways. Class-side methods continue to register
on the receiver metaclass; receiver compatibility and visibility are checked
before stack mutation or BoundMethod allocation.

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
