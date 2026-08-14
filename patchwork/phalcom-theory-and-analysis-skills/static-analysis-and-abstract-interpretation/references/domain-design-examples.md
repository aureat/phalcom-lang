# Abstract Domain Design Examples

## Constant domain

```text
Bottom | Const(v) | Top
```

Join:

```text
Const(1) ⊔ Const(1) = Const(1)
Const(1) ⊔ Const(2) = Top
Bottom ⊔ x = x
```

## Class/shape set domain

```text
Bottom | Classes({ClassId...}) | Top
```

Useful for receiver dispatch. Bound set size for advisory LSP; correctness consumers need conservative behavior when widened.

## Interval domain

```text
[-∞, +∞]
[0, 10]
[3, 3]
```

Transfer arithmetic approximately. Useful for range checks, loop reasoning and numeric contracts.

## Sign domain

```text
Negative | Zero | Positive | combinations | Top
```

Cheaper than intervals.

## Option presence domain

```text
Bottom
NoneOnly
Some(value-domain)
Maybe(value-domain)
Top/Dynamic as required
```

Can refine branch handling without using `nil` semantics.

## String domain

Possible abstractions:

- exact short literal;
- length interval;
- prefix/suffix set;
- encoding validity (Bytes -> String decode);
- unknown.

Useful for path/process/static-security checks, but regex automata analysis can become expensive.

## Collection domain

Track:

```text
kind
length interval
joined element type/shape
known tuple lanes/record labels
mutability/escape
```

Keep structural tuples/records separate from homogeneous list/set approximations.

## Map domain

Could track key/value joins plus exact known keys for small literal maps. Weak updates required when aliases/dynamic keys exist.

## Product domains

Combine orthogonal facts:

```text
ValueFact = Shape × Const × Presence × Provenance
```

Do not create a combinatorial enum of every combination.
