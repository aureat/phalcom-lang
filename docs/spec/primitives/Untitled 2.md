For syntax and LSP testing, you want an ADT that exercises:

- multiple variants
- unit variants
- tuple/product payloads
- labeled fields
- nested ADTs
- recursive structure
- exhaustive matching
- field inference/autocomplete
- constructor completion

A good candidate is a small **compiler diagnostic / syntax tree model**. It naturally exercises almost every feature.

```
@sealed
class SyntaxNode {

  @variant
  class Literal is SyntaxNode {
    value
    kind
  }

  @variant
  class Identifier is SyntaxNode {
    name
  }

  @variant
  class Binary is SyntaxNode {
    left
    operator
    right
  }

  @variant
  class Call is SyntaxNode {
    callee
    arguments
  }

  @variant
  class Block is SyntaxNode {
    statements
  }

  @variant
  object Missing is SyntaxNode {
  }
}
```

This gives the algebra:

```
SyntaxNode =
    Literal(value, kind)
  | Identifier(name)
  | Binary(left, operator, right)
  | Call(callee, arguments)
  | Block(statements)
  | Missing
```

A parser could produce:

```
const expression =
  SyntaxNode.Binary(
    left: SyntaxNode.Identifier("x"),
    operator: #+,
    right: SyntaxNode.Literal(
      value: 42,
      kind: #integer
    )
  )
```

Pattern matching:

```
match expression {

  SyntaxNode.Literal(value, kind) => {
    print("literal {value}")
  }

  SyntaxNode.Identifier(name) => {
    print("identifier {name}")
  }

  SyntaxNode.Binary(left, operator, right) => {
    print("binary {operator}")
  }

  SyntaxNode.Call(callee, arguments) => {
    print("call")
  }

  SyntaxNode.Block(statements) => {
    print("block")
  }

  SyntaxNode.Missing => {
    print("missing")
  }
}
```

The LSP can test:

## Constructor completion

Typing:

```
SyntaxNode.
```

should suggest:

```
Literal
Identifier
Binary
Call
Block
Missing
```

Typing:

```
SyntaxNode.Binary(
```

should suggest:

```
left:
operator:
right:
```

---

## Variant field inference

Given:

```
node = SyntaxNode.Binary(...)
```

then:

```
node.
```

should know:

```
left
operator
right
```

but:

```
node.value
```

should be invalid because not every variant has `value`.

---

## Exhaustiveness checking

This should produce no warning:

```
match node {
  SyntaxNode.Literal(v, k) => ...
  SyntaxNode.Identifier(n) => ...
  SyntaxNode.Binary(l, op, r) => ...
  SyntaxNode.Call(c, args) => ...
  SyntaxNode.Block(stmts) => ...
  SyntaxNode.Missing => ...
}
```

But:

```
match node {
  SyntaxNode.Literal(v, k) => ...
  SyntaxNode.Identifier(n) => ...
}
```

should report:

```
non-exhaustive match

missing variants:
  SyntaxNode.Binary
  SyntaxNode.Call
  SyntaxNode.Block
  SyntaxNode.Missing
```

---

For testing labeled payloads, I would add a richer example:

```
@sealed
class Diagnostic {

  @variant
  class Error is Diagnostic {
    message
    location
    cause
  }

  @variant
  class Warning is Diagnostic {
    message
    location
  }

  @variant
  class Info is Diagnostic {
    message
  }

  @variant
  object None is Diagnostic {
  }
}
```

Usage:

```
Diagnostic.Error(
  message: "Unexpected token",
  location: span,
  cause: None
)
```

This tests:

- named construction
- required fields
- optional-looking recursive values
- variant-specific fields

---

For recursive ADT testing, a type-expression tree is even better:

```
@sealed
class TypeExpr {

  @variant
  object Int is TypeExpr {
  }

  @variant
  object String is TypeExpr {
  }

  @variant
  class List is TypeExpr {
    element
  }

  @variant
  class Function is TypeExpr {
    parameters
    returns
  }

  @variant
  class Union is TypeExpr {
    members
  }
}
```

Example:

```
const type =
  TypeExpr.Function(
    parameters: [
      TypeExpr.Int,
      TypeExpr.String
    ],
    returns: TypeExpr.List(
      element: TypeExpr.Int
    )
  )
```

This is excellent for testing recursive pattern matching:

```
match type {

  TypeExpr.Int =>
    "Int"

  TypeExpr.String =>
    "String"

  TypeExpr.List(element) =>
    "List<{element}>"

  TypeExpr.Function(params, result) =>
    "Function"

  TypeExpr.Union(types) =>
    "Union"
}
```

For Phalcom specifically, I would use `SyntaxNode` as the primary compiler/LSP fixture because it exercises the same concepts Phalcom itself will need:

- recursive variants
- source locations
- selectors as fields
- symbols as payloads
- nested pattern matching
- reflection
- exhaustiveness analysis
- IDE navigation

It is almost a self-hosting test case for the language tooling.