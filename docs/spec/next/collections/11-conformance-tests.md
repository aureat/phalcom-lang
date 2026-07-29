# Conformance and Acceptance Tests

## 1. Test conventions

Examples use conceptual helpers:

```phalcom
assertEqual(actual, expected)
assertTrue(condition)
assertType(value, type)
assertCompileError(source, code: Symbol)
assertRuntimeError(block, code: Symbol)
```

A conforming implementation may express these through its existing test framework.

## 2. Tuple values

```phalcom
const empty = ()
assertEqual(empty.size, 0)

const pair = (1, "a")
assertEqual(pair[0], 1)
assertEqual(pair[1], "a")
```

## 3. Symbolic tuple labels

```phalcom
const value = (
  *: 1,
  **: 2,
  ?: 3,
  +: 4
)

assertEqual(value[#*], 1)
assertEqual(value[#**], 2)
assertEqual(value[#?], 3)
assertEqual(value[#+], 4)
```

## 4. Selector-valued labels

```phalcom
const operations = (
  +(): "unary",
  +(_): "binary"
)

assertEqual(operations[#+()], "unary")
assertEqual(operations[#+(_)], "binary")
```

Negative:

```phalcom
assertCompileError(
  `( +(_): first, [#+(_)]: second )`,
  code: #duplicateTupleLabel
)
```

## 5. Tuple Type exactness

```phalcom
type Pair = (Int, String)

assertTrue(Pair.satisfiedBy((1, "a")))
assertTrue(not Pair.satisfiedBy((1,)))
assertTrue(not Pair.satisfiedBy((1, "a", true)))
```

## 6. Repeated tails

```phalcom
type Ints = (Int, ...)

assertTrue(Ints.satisfiedBy(()))
assertTrue(Ints.satisfiedBy((1,)))
assertTrue(Ints.satisfiedBy((1, 2, 3)))
assertTrue(not Ints.satisfiedBy((1, "a")))
```

```phalcom
type Requests = (Context, Request, ...)
```

Acceptance MUST require exactly one `Context` followed by zero or more `Request` values.

## 7. Record values and types

```phalcom
const user = {
  name: "Ada",
  age: 36
}

assertEqual(user.name, "Ada")
assertEqual(user[#age], 36)
```

```phalcom
type User = {
  name: String,
  age: Int
}

assertTrue(User.satisfiedBy(user))
assertTrue(not User.satisfiedBy({ name: "Ada" }))
```

## 8. Set semantics

```phalcom
const values = Set.new(1, 1, 2)
assertEqual(values.size, 2)
assertTrue(values.contains(1))
assertTrue(values.contains(2))
```

```phalcom
assertTrue(Set<Int>.satisfiedBy(Set.new(1, 2, 3)))
assertTrue(not Set<Int>.satisfiedBy(Set.new(1, "a")))
```

## 9. Positional rest capture

```phalcom
capture(*args: Int) {
  return args
}

assertEqual(capture(), ())
assertEqual(capture(1, 2, 3), (1, 2, 3))
```

Equivalent explicit schema:

```phalcom
captureExplicit(*args: (*: Int)) {
  return args
}

assertEqual(captureExplicit(1, 2), (1, 2))
```

## 10. Labeled rest capture

```phalcom
capture(**labels: String) {
  return labels
}

assertEqual(
  capture(name: "Ada", mode: "strict"),
  (name: "Ada", mode: "strict")
)
```

## 11. Complete rest capture

```phalcom
capture(***arguments: Any) {
  return arguments
}

assertEqual(capture(), ())
assertEqual(capture(1, 2), (1, 2))
assertEqual(capture(name: "Ada"), (name: "Ada"))
assertEqual(
  capture(1, 2, name: "Ada"),
  (1, 2, name: "Ada")
)
```

## 12. Exact complete capture

```phalcom
capture(
  ***arguments: (
    Int,
    name: String
  )
) {
  return arguments
}

assertEqual(capture(1, name: "Ada"), (1, name: "Ada"))
```

Negative calls:

```phalcom
capture()
capture(1)
capture(1, 2, name: "Ada")
capture(1, name: "Ada", debug: true)
```

Each MUST be rejected by static checking or explicit runtime validation at a checked boundary.

## 13. Lane mismatch diagnostics

```phalcom
assertCompileError(
  `method(**labels: (*: Int)) {}`,
  code: #restLaneMismatch
)
```

```phalcom
assertCompileError(
  `method(*args: (**: String)) {}`,
  code: #restLaneMismatch
)
```

```phalcom
assertCompileError(
  `method(*args: (*: Int, **: String)) {}`,
  code: #restLaneMismatch
)
```

## 14. Mutual exclusion

```phalcom
assertCompileError(
  `method(*args: Int, ***remaining: P) {}`,
  code: #conflictingRestModes
)
```

```phalcom
assertCompileError(
  `target(*args, ***pack)`,
  code: #conflictingExpansionModes
)
```

## 15. Split expansion

```phalcom
collect(***arguments: Any) {
  return arguments
}

const positionals = (1, 2)
const labels = (name: "Ada", mode: "strict")

assertEqual(
  collect(*positionals, **labels),
  (1, 2, name: "Ada", mode: "strict")
)
```

Multiple expansions:

```phalcom
assertEqual(
  collect(*(1, 2), *(3, 4), **(x: 5), **(y: 6)),
  (1, 2, 3, 4, x: 5, y: 6)
)
```

## 16. Complete expansion

```phalcom
const pack = (1, 2, name: "Ada")
assertEqual(collect(***pack), pack)
```

## 17. Duplicate labels

```phalcom
assertRuntimeError(
  { collect(x: 1, **(x: 2)) },
  code: #duplicateCallLabel
)
```

```phalcom
assertRuntimeError(
  { collect(**(x: 1), **(x: 2)) },
  code: #duplicateCallLabel
)
```

## 18. Strict ordering

```phalcom
assertCompileError(
  `target(timeout: second, *args)`,
  code: #positionalAfterLabeled
)
```

```phalcom
assertCompileError(
  `target(**labels, timeout: second)`,
  code: #fixedLabelAfterLabeledExpansion
)
```

## 19. Type-level projections

```phalcom
type P = (Request, timeout: Duration)

type Positional = (*P,) -> R
type Labeled = (**P,) -> R
type Complete = (***P,) -> R

assertEqual(Positional, (Request) -> R)
assertEqual(Labeled, (timeout: Duration) -> R)
assertEqual(Complete, (Request, timeout: Duration) -> R)
```

## 20. Generic forwarding

```phalcom
forward<P: Tuple, R>(
  callable: (***P,) -> R,
  ***arguments: P
) -> R {
  return callable(***arguments)
}
```

Acceptance tests MUST cover:

```phalcom
forward(positionalCallable, 1, 2)
forward(labeledCallable, name: "Ada")
forward(mixedCallable, 1, name: "Ada")
```

and verify that no lane is lost.

## 21. TupleType versus ArgumentPackType

```phalcom
type Literal = (*: Int)
const Domain = ((*: Int) -> R).domain

assertTrue(Literal.class == TupleType)
assertTrue(Domain.class == ArgumentPackType)
assertTrue(Literal != Domain)
```

## 22. Record expansion

```phalcom
const fields = {
  name: "Ada",
  mode: "strict"
}

assertEqual(
  collect(**fields),
  (name: "Ada", mode: "strict")
)
```

Negative:

```phalcom
assertRuntimeError(
  { collect(*fields) },
  code: #missingPositionalLane
)
```

## 23. Selector call-label boundary

Provisional negative test:

```phalcom
const operations = (+(_): handler)

assertRuntimeError(
  { collect(**operations) },
  code: #invalidCallLabel
)
```

This test must be removed or inverted if Selector-valued call labels are later ratified.

## 24. Evaluation order

```phalcom
var events = []

record(value) {
  events.add(value)
  return value
}

collect(
  record(#first),
  *record((#second,)),
  option: record(#third),
  **record((extra: #fourth))
)

assertEqual(events, [#first, #second, #third, #fourth])
```

Every operand MUST be evaluated exactly once.
