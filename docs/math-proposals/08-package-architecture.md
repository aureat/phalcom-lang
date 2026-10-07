# 8. Package architecture and extension seams

## 8.1 Recommended dependency direction

```text
language/runtime primitives
    ├── Int / Float / Number behavior
    ├── numeric semantic kernel and policy
    └── static-value support
          ↓
universe.numbers
    ├── numeric capabilities and conversions
    ├── Rational
    └── Complex<T> core arithmetic
          ↓
universe.math
    ├── Float elementary facade (current)
    ├── exact integer algorithms (current)
    └── later dispatch to explicitly supported numeric domains
          ↓
optional packages
    ├── math.special
    ├── statistics
    ├── probability/distributions
    ├── linear algebra
    └── scientific computing
```

The arrows show dependency direction, not import syntax. In particular,
`statistics` and `probability` may depend on stable `universe.math` and
`universe.numbers`, but the numerical foundation must not depend on them.

## 8.2 Keep `universe.math` as a facade

The live package already demonstrates the right shape:

```text
universe.math/package.ph       public constants, captures, exports
universe.math/float_kernel.ph  hidden native FloatMathKernel
universe.math/integer.ph       hidden IntegerMath algorithms
```

The current math audit recommends hidden children and exact Family captures. Keep
that boundary. Do not create a public `Math` class merely to hold functions; it

## 8.3 Proposed future children

When implementation begins, a bounded topology could be:

```text
universe.numbers/package.ph       capability exports and documentation
universe.numbers/rational.ph      Rational data owner and exact operations
universe.numbers/complex.ph       Complex<T> data owner and core arithmetic
universe.numbers/conversion.ph    explicit conversion helpers
universe.numbers/classification.ph FloatClass / numeric predicates

universe.math/package.ph          current public facade
universe.math/float_kernel.ph     Float native kernels
universe.math/integer.ph          gcd/lcm/isqrt and exact helpers
universe.math/special.ph          future special-function facade
```

Do not expose implementation children unless third-party packages need a stable
import path. Public consumers should receive values and captured Families, not
native kernel owner classes.

## 8.4 Third-party extension contract

A third-party numeric type should be able to:

1. declare a nominal `class` or `data` type;
2. provide ordinary operators and methods using exact selectors;
3. explicitly `impl` approved numeric capabilities;
4. expose conversion methods with stated exactness and failure behavior;
5. participate in generic algorithms through conformance evidence;
6. retain its own representation and backend policy;
7. avoid compiler changes for ordinary library-level participation.

It should not need to subclass `Number`, modify the numeric kernel, or register in
a global compiler promotion table merely to support a generic algorithm.

## 8.5 Native boundary

The existing native boundary is a good model: source-owned declarations define
the public semantic owner, while Rust attaches representation-sensitive kernels.
Rational normalization and Complex core arithmetic should remain source-authored
where current Telos can express them and use native helpers only when the
representation or algorithm cannot be implemented cleanly in `.ph`.

Native special-function backends must carry an explicit accuracy/portability
contract. The current Rust `f64` backend's documented precision limits are part
of the design, not a testing detail.
