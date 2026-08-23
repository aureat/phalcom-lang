# Phalcom Callable Body and Return Semantics

**Status:** Normative design specification
**Scope:** Brace-bodied named callables, expression-bodied methods using existing `method => expression` syntax, closures, empty bodies, explicit return, fallthrough, constructors, and interaction with `Never`.

---

## 1. Purpose

Phalcom deliberately distinguishes three body forms:

1. brace-bodied named callables;
2. expression-bodied named callables using the existing `=>` form;
3. closures and blocks.

The distinction is syntactic and semantic. Return behavior must not depend on optional type annotations.

---

## 2. Brace-bodied named callables

A named callable written with braces uses statement-body semantics:

```phalcom
method(...) {
  ...
}
```

Its rules are:

- `return expression` returns the expression's value;
- bare `return` returns `()`;
- reaching the end of the body returns `()`;
- the final expression is not returned implicitly.

Example:

```phalcom
square(value: Int) -> Int {
  return value * value
}
```

The following does not return the multiplication result:

```phalcom
square(value: Int) -> Int {
  value * value
}
```

It evaluates `value * value`, discards the result, and falls through with `()`.

A checker shall diagnose that the declared `Int` result is not satisfied.

---

## 3. Unit fallthrough

Brace-bodied procedures naturally fall through with unit:

```phalcom
log(message: String) -> () {
  System.print(message)
}
```

This is equivalent to:

```phalcom
log(message: String) -> () {
  System.print(message)
  return ()
}
```

and:

```phalcom
log(message: String) -> () {
  System.print(message)
  return
}
```

A missing return annotation does not change these runtime semantics.

---

## 4. Expression-bodied named callables

Phalcom already supports expression-bodied named callables:

```phalcom
method(...) => expression
```

The expression's value is the callable's result.

Example:

```phalcom
square(value: Int) -> Int =>
  value * value
```

Equivalent brace-bodied form:

```phalcom
square(value: Int) -> Int {
  return value * value
}
```

### 4.1 No fallthrough

An expression body cannot fall through. It always evaluates one expression, unless that expression transfers control with type `Never`.

### 4.2 Unit expression body

A unit-returning expression body is valid:

```phalcom
noop() -> () =>
  ()
```

### 4.3 Control-flow expressions

Expression bodies compose naturally with expression-oriented control flow:

```phalcom
absolute(value: Int) -> Int =>
  if value < 0 {
    -value
  } else {
    value
  }
```

```phalcom
describe(result: Result<Int, Error>) -> String =>
  match result {
    Ok(value) => value.toString
    Err(error) => error.message
  }
```

The exact expression semantics of `if` and `match` are specified separately. This specification requires only that an expression body returns the produced value.

### 4.4 Explicit `return` inside expression bodies

The expression-body form itself does not need an explicit `return`.

An explicit `return` appearing inside a nested closure retains the nested closure's own semantics. A direct `return` expression in the top-level expression body may be rejected as redundant or accepted as a `Never`-typed control-flow expression; the language should prefer a diagnostic discouraging it.

---

## 5. Semantics do not depend on annotations

These two brace-bodied methods have the same runtime body semantics:

```phalcom
compute() {
  42
}
```

```phalcom
compute() -> Int {
  42
}
```

Both evaluate `42`, discard it, and fall through with `()`.

The second receives a static diagnostic because `()` does not satisfy `Int`.

Annotations describe and constrain semantics. They do not decide whether the final expression is returned.

---

## 6. Closures and blocks

Closures remain expression-oriented.

```phalcom
values.map { value =>
  value * value
}
```

The final expression is the closure result.

A multi-statement closure returns its final expression:

```phalcom
values.map { value =>
  const doubled = value * 2
  doubled + 1
}
```

An empty closure returns unit:

```phalcom
const callback = {
}
```

Its inferred callable type is conceptually:

```phalcom
() -> ()
```

### 6.1 Why closures differ

Closures are commonly embedded as value-producing expressions in transformations, predicates, callbacks, and higher-order APIs.

Named brace-bodied methods commonly describe longer procedural workflows where implicit final-expression returns are more error-prone.

The distinction gives Phalcom:

- concise value-producing closures;
- concise named expression bodies through `=>`;
- explicit control flow in ordinary method bodies.

---

## 7. `return` inside closures

A closure needs a defined return target.

Recommended rule:

- the closure's final expression normally determines its result;
- a `return` written inside a closure returns from that closure, not from the lexically enclosing named method;
- nonlocal return is not implicit and would require a separate explicit language feature.

Example:

```phalcom
const classify = { value =>
  if value < 0 {
    return #negative
  }

  #nonnegative
}
```

The `return` exits `classify`, not the method that created it.

This rule avoids hidden nonlocal control transfer in higher-order APIs.

---

## 8. `Never` and expression joins

Control-flow expressions such as `throw` and `return` have type `Never` in the position they occupy.

Example:

```phalcom
requireValue(option: Option<Int>) -> Int =>
  match option {
    Some(value) => value
    None => throw MissingValue.new()
  }
```

The `None` arm has type `Never`. The complete match expression has type `Int`.

Likewise:

```phalcom
fail(message: String) -> Never =>
  throw Failure.new(message)
```

---

## 9. Constructors

Constructors remain brace-bodied initially:

```phalcom
@constructor
new(value: Int) {
  _value = value
}
```

Expression-bodied constructor syntax is prohibited:

```phalcom
@constructor
new(value: Int) => expression
// invalid
```

Rationale:

- the constructor's externally visible result is the allocated `Self`;
- the instance initialization body conceptually completes with `()`;
- an arbitrary expression result must not appear to replace the constructed object;
- brace bodies make initialization and early-exit rules explicit.

The generated class-side constructor wrapper returns the allocated instance.

---

## 10. Protocol and abstract requirements

Signature-only protocol or abstract declarations contain no body and therefore no fallthrough behavior:

```phalcom
@protocol
class Writer {
  write(value: String) -> ()
}
```

A concrete implementation may choose either body form where legal:

```phalcom
write(value: String) -> () {
  stream.send(value)
}
```

```phalcom
size(value: String) -> Int =>
  value.size
```

---

## 11. Diagnostics

A checker should report missing non-unit returns in brace-bodied methods:

```text
error: callable declares result Int but this path completes with ()
```

It should distinguish:

- explicit `return ()`;
- bare `return`;
- body fallthrough;
- `Never` paths that do not complete normally.

Unreachable code after `return`, `throw`, or another `Never` expression should receive an unreachable-code diagnostic.

---

## 12. Normative decisions summary

- Brace-bodied named callables use statement-body semantics.
- Their final expression is not returned implicitly.
- Brace-body fallthrough returns `()`.
- Bare `return` returns `()`.
- `return expression` returns the expression value.
- Existing `method => expression` syntax defines an expression-bodied named callable.
- An expression body's value is the callable result.
- Return semantics never depend on optional type annotations.
- Closures return their final expression.
- Empty closures return `()`.
- `return` inside a closure returns from that closure, not the enclosing named callable.
- Constructors remain brace-bodied and cannot use expression-body syntax initially.
- `Never` branches do not widen normal expression results.
