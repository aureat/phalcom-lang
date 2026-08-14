# Phalcom callables

This is the canonical reference for executable values, message application,
argument transport, dispatch, reflection, and callable activation in Phalcom.
It specifies the language model first, then describes the VM mechanisms that
realize it. An implementation detail never changes the observable language
rule stated in its surrounding section.

Older callable documents are not silently reconciled here. Where they conflict
with this folder, this folder defines the callable model.

## Reading map

Start here for the object model and the distinction between a callable protocol
and a `Function`. Then follow the path that matches the question:

| Question | Read |
| --- | --- |
| How does `f(x)` become a message send? | [Dispatch and lowering](dispatch.md) |
| How are labels, packs, rest, and spread represented? | [Arguments and rest](arguments.md) |
| How does the VM enter a callable without recursive interpreter re-entry? | [Runtime and activation](runtime.md) |
| What do `self`, `super`, return, closures, and constructors mean? | [Execution contexts](execution.md) |
| How do `methodFor`, `bind`, `invokeOn`, and `perform` relate? | [Reflection and exact invocation](reflection.md) |
| What must compiler and VM changes preserve? | [Callable conformance requirements](conformance.md) |

The class chapters are deliberately separate:

| Callable object | Chapter |
| --- | --- |
| Reified behavior that still needs a receiver | [Method](method.md) |
| Complete VM-backed callable root | [Function](function.md) |
| Lexically captured executable value | [Closure](closure.md) |
| Exact Method paired with a receiver | [BoundMethod](bound-method.md) |
| Bound `::` late-dispatch reference | [Family](family.md) |

## Normative vocabulary

A **block** is a brace-delimited syntactic and lexical region. A block is not,
by itself, a runtime callable value.

A **Closure** is a first-class executable value carrying compiled code and its
lexical captures. A **Method** is reified holder-owned behavior requiring an
explicit receiver. A **BoundMethod** is an exact Method paired with a captured
receiver. A **Function** is a sealed abstract VM-backed callable
whose remaining runtime inputs are only explicitly supplied call arguments. A
**Family** is a bound `::` method-family reference that performs lookup when
called.

`()` is Unit: successful completion with no payload and the empty product.
`None` is absence. They are different values with different meanings.

## Callable hierarchy

```text
Object
├── Method                         sealed/final core class
└── Function                       abstract, sealed core class
    ├── Closure                    sealed/final
    ├── BoundMethod                sealed/final
    └── Family                     sealed/final
```

`Method` is deliberately outside `Function`. It carries exact behavior but is
not complete: a receiver must still be supplied. `Closure` captures its own
lexical environment, `BoundMethod` already stores a receiver, and `Family`
already stores reference context; each therefore needs only explicit call
arguments and is a Function.

The sealed hierarchy is not the whole callable universe. Any ordinary object
may define a method named `call` and then participate in application syntax.
That protocol conformance does not make the object a `Function`, does not give
it VM Function representation, and does not permit it to subclass the sealed
core hierarchy.

## Application is an ordinary send

For a value application, Phalcom specifies:

```phalcom
f(a, b)
```

as observationally equivalent to:

```phalcom
f.call(a, b)
```

This is a message rule, not a second invocation language. The compiler may use
special bytecode for the common case, but that bytecode encodes the same `call`
selector and enters the same lookup, authorization, error, and activation
paths. [Dispatch and lowering](dispatch.md) gives the precise exception that
matters to readers of source code: an unqualified `name(...)` with no lexical
or global value binding is an implicit-`self` method send, not value
application.

The common Function gateway is one rest-capable method:

```phalcom
call(***arguments)
```

It transports a complete argument shape. It does not promise that every
concrete Function accepts every shape; each concrete representation performs
its own parameter acceptance after the gateway is selected. See
[Function](function.md), [Arguments and rest](arguments.md), and
[Runtime and activation](runtime.md).

## Rest and spread notation

Phalcom has exactly three rest/spread markers:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

```phalcom
target(*values)
target(**labels)
target(***arguments)

method(*rest) { body }
method(**rest) { body }
method(***rest) { body }

|head, *tail| { body }
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator. [Arguments and rest](arguments.md) defines evaluation,
normalization, matching, and capture.

## Semantic layers

The model is easiest to reason about as six separate layers. Keeping them
separate prevents familiar but incorrect shortcuts such as treating a Method as
a Function or treating a complete pack as a parameter binder.

```text
source application / method send
        ↓
selector construction from actual argument shape
        ↓
exact Method lookup, then rest-family fallback
        ↓
visibility authorization and parameter acceptance
        ↓
bytecode or native Method activation
        ↓
result, throw, or explicit control transfer
```

1. **Source and name resolution** decide whether syntax is an explicit receiver
   send, an implicit-`self` send, or a call to a value binding.
2. **Selector construction** preserves positional slots and labels as dispatch
   identity.
3. **Lookup** selects a Method; it is distinct from Method execution.
4. **Acceptance** compares actual argument shape to the selected Method or
   Closure parameter shape.
5. **Activation** installs receiver, locals, lexical authority, and a frame or
   native context.
6. **Execution** applies final-expression, Unit, local-return, `self`, and
   lexical-`super` rules.

## Normative specification and implementation notes

The language rules in this folder are normative. Sections titled
**Implementation note** describe the current Rust VM, usually with a source
excerpt and a link. They explain why a rule is inexpensive or why two surface
forms share a path; they are not a public Rust API.

The current checkout contains callable-migration work. In particular, some
internal carrier names and compatibility primitives remain while the canonical
surface is being completed. Such material is identified as transitional. It
does not create an additional public callable class or alter the hierarchy
above.

## Source map

The implementation-oriented chapters refer mainly to these locations:

- [`compiler/lib/expr.rs`](../../../phalcom-core/src/compiler/lib/expr.rs) —
  name resolution, selector construction, `Invoke`, and `InvokePack` emission.
- [`vm/dispatch.rs`](../../../phalcom-core/src/vm/dispatch.rs) — bytecode
  execution, rest lookup, pack sends, class installation, and family creation.
- [`vm/send.rs`](../../../phalcom-core/src/vm/send.rs) — method authorization,
  shaped dispatch, Function activation, and flat forwarding.
- [`method/object.rs`](../../../phalcom-core/src/method/object.rs) —
  `ArgumentView`, `CallOutcome`, Method representation, and native ABI.
- [`primitive/block.rs`](../../../phalcom-core/src/primitive/block.rs),
  [`primitive/method.rs`](../../../phalcom-core/src/primitive/method.rs), and
  [`primitive/object.rs`](../../../phalcom-core/src/primitive/object.rs) —
  Function, exact-Method, and reflective gateways.

## Related chapters

- [Dispatch and lowering](dispatch.md)
- [Arguments and rest](arguments.md)
- [Runtime and activation](runtime.md)
- [Execution contexts](execution.md)
- [Reflection and exact invocation](reflection.md)
- [Callable conformance requirements](conformance.md)
- [Method](method.md)
- [Function](function.md)
- [Closure](closure.md)
- [BoundMethod](bound-method.md)
- [Family](family.md)
