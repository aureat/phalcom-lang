# Phalcom Type Syntax

> Sources: Phalcom repository, 2026-09-07
> Raw: [phalcom-type-syntax source snapshot](../raw/type-system/2026-09-07-phalcom-type-syntax.md)
> Updated: 2026-09-07

## Overview

`phalcom-type-syntax` is the VM-free symbolic syntax layer for Phalcom native metadata. It defines an owned AST for type expressions, parameter tuples, callable signatures, generic constraints, and parser errors, then exposes small entry points for parsing complete type and callable strings.

## Type-expression model

`TypeExpr` represents the syntax-level vocabulary: `Unknown`, `Never`, `Self`, named types, `universe.<name>` references, named parameters, applied types, unions, and tuple-shaped parameter groups. Applied types use an origin plus a vector of argument expressions; unions retain their alternatives in source order.

The parser recognizes the reserved words `Self`, `Never`, `Unknown`, and `universe`. A named identifier is parsed as `TypeExpr::Named`; the `Parameter` variant is available in the AST but is not selected by the standalone parser based only on identifier spelling. A trailing-token check makes the top-level `parse_type_expr` entry point reject otherwise valid prefixes followed by extra syntax.

## Callable and parameter syntax

A `ParameterTuple` separates positional parameters, labeled parameters, and an optional rest parameter. Labeled parameters use `label: Type`, and once a label has appeared, a later positional parameter is rejected. Rest syntax is `...` or `...Type` and is stored as `RestParameter { ty: None }` or `Some(TypeExpr)`.

`CallableType` combines optional type parameters, a parameter tuple, a return type, and zero or more constraints. The accepted shape is:

```
<T, U>(T, using: U) -> U where T <: Object, U == U
```

The `where` clause supports subtype (`<:`) and equivalence (`==`) relations. The public `parse_callable_type` function parses the whole input and rejects trailing tokens; `parse_param_list` exposes tuple parsing when a callable arrow is not needed.

## Parsing and display behavior

The lexer is intentionally small: it handles identifiers, punctuation, arrows, generic delimiters, the pipe union operator, and ellipsis rest markers. Generic arguments and unions recurse through the same type parser. Labeled-parameter detection uses a cloned parser state so an identifier can still be interpreted as a positional named type when it is not followed by a colon.

Failures are returned as `TypeSyntaxError`: unexpected end of input, unexpected character, an expected token mismatch, or a contextual invalid-syntax message. Display implementations reconstruct the principal surface forms, including comma-separated generic arguments, unions, labeled/rest parameters, callable type parameters, and generic constraints.

## Evidence and tests

The crate source includes focused tests for basic and universe-qualified types, applied types, unions, callable signatures, labeled parameters, generic parameters, and both supported constraint relations. The manifest describes the crate as VM-free and lists `thiserror` as its dependency.

## See Also

- [Phalcom Type Metadata](phalcom-type-meta.md)
