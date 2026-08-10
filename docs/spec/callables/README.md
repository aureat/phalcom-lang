# Callable model

This folder is the canonical specification for Phalcom callable values. It
defines the object model, execution contexts, argument transport, returns, and
constructor generation. Older callable documents remain outside this migration
until a later documentation task explicitly updates them.

## Hierarchy

```text
Object
├── Method                         sealed/final core class
└── Function                       abstract, sealed core class
    ├── Closure                    sealed/final
    ├── BoundMethod                sealed/final
    └── Family                     sealed/final
```

The hierarchy is deliberately split at `Method`. A `Method` is reified,
holder-owned behavior that still requires a compatible receiver. `Function` is
the abstract, sealed, VM-backed root for complete callables whose remaining
runtime inputs are explicitly supplied call arguments. `Closure` is a
first-class executable value carrying compiled code and lexical captures.
`BoundMethod` is an exact `Method` paired with a compatible receiver. `Family`
is a bound `::` method-family reference that performs lookup when called.

`Method` is not a subtype of `Function`: it is incomplete until a receiver is
provided. `BoundMethod` and `Family` are `Function`s because their receiver or
reference context is already present. No user class may subclass any class in
this sealed callable hierarchy.

An arbitrary user object that defines `call` is still callable without being a
`Function`. Application syntax is ordinary message syntax:

```phalcom
f(a, b)
```

means:

```phalcom
f.call(a, b)
```

The `Function` hierarchy supplies one final `call` family for its own concrete
representations; it does not turn every object answering `call` into a
`Function`.

## Argument notation

Phalcom uses one rest/spread notation everywhere:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

Examples:

```phalcom
foo(*values)
foo(**labels)
foo(***arguments)

method(*rest) { body }
method(**rest) { body }
method(***rest) { body }

|head, *tail| { body }
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

## Execution contexts

`Method` still needs a receiver. `Function` needs only explicit arguments.
That difference controls both inheritance and activation:

| Value | Receiver state at call time | Call result |
| --- | --- | --- |
| `Method` | Missing; supplied by binding or exact invocation | Not a `Function` until bound |
| `Closure` | Lexically captured when applicable | Executes captured code |
| `BoundMethod` | Stored and validated | Executes its exact `Method` |
| `Family` | Stored in its reference context | Performs family lookup, then sends |

All concrete `Function`s enter through the common `call(***arguments)` gateway.
The gateway can transport a complete argument shape, but the concrete callable
still validates its own parameter shape. For example:

```phalcom
const f = |head, *tail| { tail }
```

accepts positional arguments and positional spread. It rejects a non-empty
labeled lane, including one transported by a complete pack.

The VM may represent this gateway with allocation-free argument views. That is
an implementation detail; the public shape remains `***arguments`.

## `self` and lexical context

- A `Method` receives dynamic `self` from its activation.
- A `BoundMethod` supplies its stored receiver as that `self`.
- A `Closure` created in a `Method` or another `Closure` lexical environment
  captures the current `self` value when one exists.
- Ordinary sends inside a `Method` dispatch dynamically on `self`.
- `super` remains lexically anchored to the defining `Method` holder.
- Capturing `self` in a `Closure` does not change `super` semantics.

Consequently, binding changes the receiver supplied to a `Method`, while
lexical capture preserves the closure environment in which a `Closure` was
created.

## Return model

Ordinary `Method`s and `Closure`s return the final expression of their body.
Empty bodies return `()`. Bare `return` means `return ()`. `return value`
returns from the current `Method` or `Closure` activation only; there is no
implicit non-local return through a closure boundary.

`None` is absence. It is distinct from `()`, the successful no-result Unit
value. Assignment, declarations, empty bodies, and other constructs with no
result use Unit rather than `None`.

## Constructor generation

An `@constructor` source declaration has this conceptual compiler-generated
shape:

```text
@constructor source declaration
    → generated class-side factory
    → allocate instance
    → call generated/hidden instance initializer as an ordinary Method
    → ignore initializer return value
    → return allocated instance
```

The initializer itself obeys ordinary `Method` runtime return semantics.
Because its source declaration is marked `@constructor`, the compiler rejects
`return value` in the constructor initializer body. Bare `return` is allowed
and returns `()` from the initializer; the generated factory still returns the
allocated instance. This is a compiler restriction attached to generated
constructor semantics, not a special `Method` return opcode.

## Callable specifications

- [`Method`](method.md) — exact holder-owned behavior and receiver validation
- [`Function`](function.md) — common gateway and sealed callable protocol
- [`Closure`](closure.md) — lexical executable values and positional rest
- [`BoundMethod`](bound-method.md) — exact method plus validated receiver
- [`Family`](family.md) — bound `::` method-family references
