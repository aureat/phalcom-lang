# Orders, Lattices, and Fixed Points

## Precision order

Let `a ⊑ b` mean "a is at least as precise as / approximated by b" according to the chosen convention. Pick one convention and use it consistently.

A common may-analysis convention represents sets of possible concrete states:

```text
{Int} ⊑ {Int, String} ⊑ AnyPossible
```

Smaller sets are more precise.

## Partial order laws

A precision/order relation should satisfy:

- reflexivity: `a ⊑ a`;
- antisymmetry: `a ⊑ b` and `b ⊑ a` implies semantic equality;
- transitivity: `a ⊑ b` and `b ⊑ c` implies `a ⊑ c`.

If your relation violates these, it may be a compatibility relation rather than an order.

## Join

`a ⊔ b` is the least abstract state covering both branch possibilities.

Required laws for a standard lattice join:

```text
a ⊔ a = a
 a ⊔ b = b ⊔ a
(a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
```

These laws are ideal targets for property tests.

## Bottom

`⊥` often means no reachable concrete states / impossible program point.

For may-analysis:

```text
⊥ ⊔ a = a
```

This is not the same as "we know nothing."

## Top

`⊤` often means all possible states allowed by the domain / total loss of precision.

For may-analysis:

```text
a ⊔ ⊤ = ⊤
```

## Complete lattices and finite height

Tarski's fixed-point theorem gives existence of least fixed points for monotone functions over complete lattices. Practical solvers rely on finite domains or widening so iteration terminates.

## Monotonicity

Transfer `F` should preserve order:

```text
a ⊑ b  =>  F(a) ⊑ F(b)
```

Non-monotone transfer functions can make worklist iteration oscillate or invalidate fixed-point reasoning.

## Ascending chains

A loop solver repeatedly grows information:

```text
a0 ⊑ a1 ⊑ a2 ...
```

If the domain admits infinite chains (numeric intervals expanding forever), use widening.

## Product domains

Combine domains component-wise:

```text
State = Bindings × PathFacts × Effects
```

Join each component according to its own algebra. Avoid one mega-enum that loses independent precision.

## Reduced products

Separate domains can exchange information to improve precision:

```text
interval says x in [0,0]
congruence says x even
```

Reduction tightens one using the other. This is powerful but increases complexity; introduce only when useful.
