# Closures

> **Status:** Draft normative specification  
> **Semantic ownership:** Closure literal forms, lexical capture, Closure parameter restrictions, `self` and `super` capture, local return, and Closure activation.  
> **Related specifications:** [Functions](functions.md), [Arguments and Parameters](arguments.md), [Execution Contexts](execution-contexts.md), [Dispatch](dispatch.md).

A `Closure` is a first-class `Function` containing executable code together with the lexical environment required by that code.

A brace-delimited region is not, by itself, a Closure value.

Closure creation is explicit through Closure literal syntax or another syntactic form defined to produce a Closure.

## Literal forms

Canonical Closure forms include:

```phalcom
|| { body }

|value| { body }

|first, second| {
  body
}

|value| expression

|head, *tail| {
  body
}
```

The parameter list is part of the Closure literal.

A body written as a single expression produces that expression's semantic value unless control transfers earlier.

A braced Closure body follows ordinary callable-body completion rules.

## Block syntax and Closure values

A brace-delimited block may introduce lexical scope without producing a first-class callable value.

For example, ordinary control-flow bodies are lexical blocks but are not automatically Closures.

A Closure literal explicitly allocates or denotes a first-class callable value whose execution may be deferred, stored, returned, or invoked later.

This distinction prevents lexical grouping from being conflated with first-class deferred execution.

## Trailing Closure syntax

Where the grammar admits contextual trailing Closure syntax:

```phalcom
resource.withLock {
  work()
}
```

the trailing block is syntax for passing a zero-argument Closure in that position.

It does not make every brace expression a Closure.

Explicit parameterized trailing Closures may likewise be written where the grammar permits:

```phalcom
users.any where: |user| {
  user.active
}
```

Ambiguous collection/block literals must be disambiguated according to the expression grammar rather than by treating braces as universally callable.

## Closure parameters

Closure parameter syntax is intentionally narrower than ordinary Method parameter syntax.

A Closure supports:

```text
fixed positional parameters
optional terminal positional *rest
```

It does not currently support:

```text
labeled parameters
**rest
***rest
multiple *rest parameters
fixed parameters after *rest
```

Examples:

```phalcom
|| { ... }
|x| { ... }
|x, y| { ... }
|head, *tail| { ... }
```

The general argument-shape and rest-capture laws are defined by [Arguments and Parameters](arguments.md).

A Closure without rest accepts exactly its fixed positional count and requires an empty labeled lane.

A Closure with terminal `*rest` accepts at least the fixed positional prefix and still requires an empty labeled lane.

Residual rest capture is:

```text
no residual values
    → ()

one or more residual values
    → Tuple
```

Closure rest does not produce a List.

## Lexical capture

A Closure captures the lexical bindings referenced by its body.

Captured immutable values remain available after the creating activation returns.

Captured mutable bindings retain shared binding-cell semantics: mutation through one lexical path is observed by other references to the same captured binding.

**Invariant — `CLO-CAPTURE-LEXICAL`**

> A Closure retains the lexical bindings required by its body, and mutable captured bindings preserve shared binding identity rather than being copied into independent value snapshots.

The exact storage strategy for captured bindings is implementation-private.

## Captured `self`

When a Closure is created in an execution context with a current `self`, the Closure captures that value when its body refers to `self`.

For example:

```phalcom
class Box {
  callback() {
    || { self }
  }
}
```

the returned Closure continues to yield the original `Box` receiver after `callback()` has returned.

Captured `self` is lexical Closure state, not an explicit Closure argument.

## `super` in a Closure

A Closure does not establish a new Method holder merely because it is callable.

Where Closure code contains a `super` send lexically nested within a Method context, the send retains the corresponding lexical Method super anchor.

Capturing a different runtime `self` does not rewrite that anchor.

This follows the execution-context model in [Execution Contexts](execution-contexts.md).

## Closure activation

A Closure is a complete `Function`.

Application transports the complete incoming argument shape through the common Function gateway.

The Closure then validates that shape against its positional-only parameter model and binds fixed/rest parameters.

A non-empty labeled lane is rejected.

After binding, the Closure executes in its captured lexical environment.

## Return

`return` inside a Closure is local to that Closure activation.

```phalcom
const f = || {
  return 10
}
```

calling `f()` returns `10`.

It does not return from the Method or Closure that created `f`.

**Invariant — `CLO-RETURN-LOCAL`**

> A Closure `return` exits only the current Closure activation and never implicitly unwinds an enclosing callable activation.

Bare:

```phalcom
return
```

returns `Unit`.

An empty Closure body also produces `Unit`.

## Closure boundaries and control flow

A real Closure creates a new callable boundary.

Lexical control-flow operations inside the Closure cannot target loops or lexical control constructs in an outer callable activation.

For example, a `break` inside a Closure cannot break a loop executing in the creating Method.

The detailed loop semantics belong to the control-flow specification.

## Closure identity

Two Closure values created by separate evaluations of the same literal are not required to be the same runtime object merely because they share source code.

Their captured environments may differ.

Structural equality of Closures is not implied by identical code or parameter shape.

Any equality rule beyond ordinary object identity must be specified elsewhere.

## Invariants defined by this specification

**`CLO-CAPTURE-LEXICAL`**

> A Closure retains the lexical bindings required by its body, and mutable captured bindings preserve shared binding identity rather than being copied into independent value snapshots.

**`CLO-RETURN-LOCAL`**

> A Closure `return` exits only the current Closure activation and never implicitly unwinds an enclosing callable activation.
