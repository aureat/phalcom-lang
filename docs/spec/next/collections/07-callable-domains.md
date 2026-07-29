# Callable-Domain Specification

## 1. Definition

A callable Type consists of:

```text
CallableType = ArgumentPackType → ResultType
```

The domain classifies accepted call packs. It does not prescribe local parameter names or whether an implementation uses split or complete rest capture.

## 2. Tuple-shaped domain syntax

**RATIFIED:** Callable domains always use tuple syntax.

```phalcom
() -> R
(Int,) -> R
(Int, String) -> R
(Int, timeout: Duration) -> R
(***P,) -> R
```

There is no `Int -> R` shorthand.

## 3. One argument versus multiple arguments

```phalcom
(Int,) -> R
```

accepts one positional `Int`.

```phalcom
((Int, String),) -> R
```

accepts one positional Tuple.

```phalcom
(Int, String) -> R
```

accepts two positional arguments.

## 4. Open lanes

Callable-domain context interprets reserved tuple keys as open lanes:

```phalcom
(*: Int) -> R
(**: String) -> R
(*: Int, **: String) -> R
```

With fixed entries:

```phalcom
(
  Request,
  *: Bytes,
  timeout: Duration,
  **: Metadata
) -> Response
```

## 5. Type-level pack unpacking

Given:

```phalcom
type P = (
  Request,
  timeout: Duration
)
```

then:

```phalcom
(*P,) -> R
```

projects only `P`'s positional lane.

```phalcom
(**P,) -> R
```

projects only `P`'s labeled lane.

```phalcom
(***P,) -> R
```

preserves the complete domain.

This is the canonical form for generic forwarding.

```phalcom
forward<P: Tuple, R>(
  callable: (***P,) -> R,
  ***arguments: P
) -> R {
  return callable(***arguments)
}
```

## 6. Arbitrary callable shorthand

**AMENDED:** Earlier discussion equated `(...) -> R` with a complete Tuple unpack using `*`. Under the three-operator model it MUST normalize to complete unpack:

```phalcom
(...) -> R
```

is sugar for:

```phalcom
(***Tuple,) -> R
```

or an equivalent canonical `ArgumentPackType.any` representation.

The result accepts any well-formed call pack.

## 7. Method declaration elaboration

These declarations may expose the same callable domain:

```phalcom
method(
  *args: Int,
  **labels: String
) -> R
```

```phalcom
method(
  ***arguments: (
    *: Int,
    **: String
  )
) -> R
```

Both elaborate to:

```phalcom
(*: Int, **: String) -> R
```

Their local bindings differ, but callable type identity does not.

## 8. Exact domains

```phalcom
(Int, name: String) -> R
```

accepts exactly one positional `Int` and one labeled `name: String` unless defaults or optional-domain expansion explicitly alter acceptance.

## 9. Defaults and optional parameters

**OPEN:** Default values create multiple accepted call shapes.

Example:

```phalcom
connect(host: String, port: Int = 443)
```

Possible models:

1. callable domain is a union of `(host: String)` and `(host: String, port: Int)`;
2. domain slots carry optionality metadata;
3. reflection stores one declaration domain and a separate accepted-domain expansion.

This suite recommends model 3, but it is not ratified.

## 10. Callable subtyping

Let `Calls(D)` be the set of call packs accepted by domain `D`.

```text
(D₁ → R₁) <: (D₂ → R₂)
```

iff:

```text
Calls(D₂) ⊆ Calls(D₁)
R₁ <: R₂
```

Thus parameters are contravariant by accepted-call-set inclusion and results are covariant.

Example:

```phalcom
(*: Any) -> String
```

is a subtype of:

```phalcom
(*: Int) -> Any
```

because it accepts at least every positional-`Int` call and returns a more specific result.

## 11. Labels in subtyping

Labels are part of the accepted call shape.

```phalcom
(timeout: Duration) -> R
```

and:

```phalcom
(deadline: Duration) -> R
```

are not substitutable merely because their value types match.

Open labeled domains compare through accepted pack inclusion:

```phalcom
(**: Any) -> R
```

accepts at least the calls accepted by:

```phalcom
(**: String) -> R
```

## 12. Reflection

A `CallableType` SHOULD expose:

```phalcom
callable.domain -> ArgumentPackType
callable.result -> Type
callable.accepts(pack) -> Bool
```

Source spelling MAY be retained separately:

```phalcom
callable.sourceDomain
callable.normalizedDomain
```
