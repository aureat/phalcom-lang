# Execution contexts, returns, and constructors

[Callables](README.md) · [Dispatch and lowering](dispatch.md) · [Runtime and activation](runtime.md) · [Method](method.md) · [Closure](closure.md) · [Function](function.md)

This chapter defines the execution context shared by Method and Closure bodies:
lexical scope, `self`, `super`, return, Unit, and compiler-generated
constructors. It is separate from [Arguments and rest](arguments.md): argument
binding decides how an activation begins; these rules decide what executing its
body means.

## 1. Brace regions and Closure values

Every brace-delimited executable region introduces lexical scope. Bindings
declared there are visible in nested lexical regions and may be captured by a
Closure created within that scope.

A brace region is not automatically a Closure. A Closure is a first-class
runtime value created by `||`, `|parameters|`, or the contextual trailing
Closure sugar described in [Closure](closure.md#2-closure-is-a-value-a-block-is-syntax).

## 2. `self`

`self` is an execution-context value, not an argument supplied by ordinary
source call syntax.

- A Method receives dynamic `self` from the receiver on which it is activated.
- A BoundMethod supplies its stored receiver as that `self`.
- A Closure created in a Method or Closure lexical environment captures the
  current `self` when one exists.
- An ordinary send inside a Method dispatches dynamically on its current
  `self`, even when the entry Method was selected exactly.

```phalcom
class Base {
    describe { self.label }
}

class Derived is Base {
    label { #derived }
}

const method = Base.methodFor(#describe())
method.invokeOn(Derived.new(), ***()) // invokes Base#describe; self.label is Derived#label
```

The precise `methodFor` surface is described in [Reflection](reflection.md).
The example illustrates the split: entry behavior is exact `Base#describe`,
but an ordinary send inside it is dynamically dispatched on the Derived
receiver.

## 3. `super`

`super` keeps the current dynamic receiver but changes only the lookup origin.
It starts lookup above the lexically defining Method holder.

```text
exact Method body selected     → remains the reified Method body
self                           → supplied receiver; field access may require holder layout
ordinary receiver sends        → lookup from receiver's runtime class
super sends                    → lookup above lexical defining holder
```

Binding a Method to a subclass receiver and calling it through a BoundMethod
does not move its lexical `super` anchor. Capturing `self` in a Closure also
does not rewrite the anchor; a Closure's capture preserves an environment, not
the definition site of `super`.

The compiler emits a dedicated `SuperSend`/`SuperSendPack` path rather than an
ordinary `Invoke`, carrying the defining holder as bytecode metadata. See
[Dispatch and lowering](dispatch.md#7-super-is-a-different-send-origin).

## 4. Final-expression result

Ordinary Method and Closure bodies return the semantic value of their final
executable construct.

```phalcom
answer {
    42
}

const double = |x| x * 2
```

Both return their final expression. An empty body returns Unit:

```phalcom
noop { }
const empty = || { }
```

Both produce `()`.

Assignments and executable declarations evaluate their operands and perform
their side effects exactly once, but their own semantic value is Unit. A
one-armed `if` and normal loop completion also produce Unit. `None` never means
"no result"; it remains the language's absence value.

### 4.1 Assignment and declaration

Assignment evaluates its right-hand side once, performs the store, and returns
Unit even when an underlying setter Method returns another value:

```phalcom
const result = (x = compute())
// result == ()
```

This applies to local, captured, field, property, and subscript assignment.
The compiler may omit a physical Unit push when it can prove the result is
immediately discarded, but it must not omit side effects or change the value in
a value-needed context.

Executable declarations also produce Unit:

```phalcom
const result = {
    const x = compute()
}
// result == ()
```

The initializer still runs exactly once. This rule prevents declarations from
accidentally leaking initializer values as body results.

### 4.2 Conditionals

A two-armed conditional returns the selected branch value:

```phalcom
const sign = if value < 0 {
    #negative
} else {
    #nonnegative
}
```

A one-armed conditional always returns Unit, regardless of whether its body
executes. Its body value is discarded:

```phalcom
const result = if ready {
    start()
}
// result == ()
```

### 4.3 Loops and `break` values

The normal result of a source loop is Unit. Per-iteration body values are
discarded. Bare `break` is equivalent to `break ()`; `break value` makes the
enclosing loop evaluate to `value`.

```phalcom
const found = while search.hasNext {
    const item = search.next()
    if matches(item) {
        break item
    }
}
```

`found` is `item` when the break executes and `()` when the loop finishes
normally. The break operand is evaluated once. These are lexical control-flow
rules, not a cross-Closure message protocol.

## 5. Local `return`

```phalcom
return
```

means:

```phalcom
return ()
```

`return value` exits only the current Method or Closure activation. There is
no implicit non-local return through an enclosing Method or Closure.

```phalcom
make {
    const f = || {
        return 10
    }

    f()
    20
}
```

The `return 10` exits `f`; `make` returns `20`. An escaping Closure with a
local `return` remains valid after the creating Method has finished.

**Implementation note.** The current runtime still contains transitional
home-frame bookkeeping from an older callable representation. It must not be
used to define public return semantics. The canonical compiler/VM contract is
that every ordinary return has one explicit result value, including Unit for a
bare return or an empty body.

## 6. Loops and Closure boundaries

Source loops are lexical control flow. Normal completion yields Unit; `break`
yields Unit; `break value` yields `value` as the enclosing loop's result.
Per-iteration values are discarded.

A real Closure establishes a new callable activation. Therefore `break` and
`continue` inside that Closure do not target a loop in an outer Method or
Closure:

```phalcom
while ready {
    const later = || {
        break 42 // invalid: no enclosing loop in this Closure activation
    }
}
```

This rule avoids cross-Closure control transfer and keeps loop value semantics
local to compiled lexical control flow.

## 7. Constructors

An `@constructor` source declaration is conceptually compiler generation over
ordinary Methods:

```text
@constructor source declaration
    → generated class-side factory
    → allocate instance
    → call generated/hidden instance initializer as an ordinary Method
    → ignore initializer return value
    → return allocated instance
```

The generated initializer obeys normal Method runtime semantics. Its final
expression has a value and bare `return` returns Unit from that initializer.
The generated factory discards that value and returns the allocated instance.

The source `@constructor` marker adds one compile-time restriction:

```phalcom
return value
```

is rejected in a constructor initializer body. Bare `return` is permitted. The
restriction belongs to constructor generation, not to a special Method return
opcode. An ordinary class-side Method should be used when source intends to
return a non-instance value.

## 8. Related chapters

- [Method](method.md) — exact activation and lexical ownership
- [Closure](closure.md) — capture and local Closure return
- [Dispatch and lowering](dispatch.md) — ordinary versus `super` sends
- [Runtime and activation](runtime.md) — frame entry and transitional notes
- [Function](function.md) — complete callable activation
