# Collection Literal Surface

## Authority

This document records the accepted active literal behavior for collections.
The type contracts live in [`collections/`](collections/); structural Tuple
and Record semantics live in [`structural-products.md`](structural-products.md).
An implementation plan cannot change these source rules. A change requires an
accepted superseding decision and an update here.

## List literals

`[a, b, c]` is a primary expression that constructs a fresh mutable `List`.
Elements evaluate eagerly from left to right. A trailing comma is allowed and
`[]` constructs an empty List. Literal syntax constructs a value; it does not
define indexed access or indexed assignment on arbitrary expressions.

The list literal is semantically equivalent to starting with `List.new()` and
adding each evaluated element in order. Implementations may use a different
lowering if the observable evaluation order and fresh mutable List result are
preserved.

## Map literals

`{key: value, ...}` is a map literal when its contents are one or more
comma-separated key/value pairs. `{}` and brace groups containing statements or
block parameters remain blocks; construct an empty map with `Map.new()`.

A bare identifier key denotes the corresponding Symbol. String and numeric
literal keys denote those values. A parenthesized expression key is evaluated
as an expression. Keys and values evaluate left to right, and the expression
returns a fresh mutable `Map`.

## Tuple literals

`(a)` is a parenthesized expression. A comma marks a Tuple: `(a,b)`, `(a,b,c)`,
and `(a,)` are Tuple literals; `()` is the empty Tuple. Elements evaluate left
to right and the result is immutable.

## Reserved collection sigils

The `#{...}` Set sigil and `..` / `...` Range operators are reserved but
inactive. Their future meanings are recorded by
[`collections/set.md`](collections/set.md) and
[`collections/range.md`](collections/range.md). Until they are activated,
construct Set and Range values through their explicit APIs.

## Array and Vector relationship

`Array<T>` and `Vector<T,N>` type contracts do not by themselves select either
type from `[...]`. The accepted active rule remains List construction. Contextual
bracket defaults, explicit literal heads, and phase-sensitive Array/Vector
selection are a separate language change and must be specified here only after
the existing decision is formally superseded.
