# 10. Selective external prior art

This is a decision aid, not a language survey. Four sources were consulted only
for concrete numerical design questions.

## Julia: explicit promotion can be powerful but assumes a mature dispatch model

Julia's documentation treats conversion and promotion as a coordinated system and
uses parametric `Rational{T}` and `Complex{T}` families. Its Rational constructor
normalizes signs and reduces numerator/denominator by `gcd`; its promotion model
lets an operation find a common type before performing arithmetic.

Source: [Julia constructors, conversion/promotion, and Rational case study](https://docs.julialang.org/en/v1/manual/constructors/#Case-Study:-Rational).

**Lesson for Telos:** parametric exact values and promotion are coherent when
multiple dispatch, constructor methods, and conversion rules are designed as one
system. Telos currently has selector-identity dispatch and explicit nominal trait
evidence instead. It should adopt the normalization law and generic-family idea,
but defer a global promotion lattice until it can state ambiguity and exactness
rules in Telos terms.

## Python PEP 3141: useful vocabulary, expensive reverse-operation protocol

Python's `numbers` module names `Number`, `Complex`, `Real`, `Rational`, and
`Integral` and explains mixed-mode operations with forward and reverse methods.
The documentation also warns implementers to coordinate equality and hashing.

Source: [Python `numbers` documentation](https://docs.python.org/3/library/numbers.html).

**Lesson for Telos:** the vocabulary is useful as a taxonomy, but Telos should
not import the entire nominal tower. Python's reverse-method fallback is a poor
primary abstraction for Telos because Telos already has a compiler-defined
bilateral operator path and exact selector identity. Use capabilities for the
algorithm's laws and keep mixed operator behavior finite and explicit.

## Rust `f64`: representation metadata and backend honesty

Rust's current `f64` documentation exposes binary64 constants and classification
predicates such as `is_nan`, `is_infinite`, `is_subnormal`, and `classify`. It also
marks several elementary operations, including angle conversion, as having
unspecified precision that may vary by platform or toolchain.

Source: [Rust `f64` API](https://doc.rust-lang.org/stable/core/primitive.f64.html).

**Lesson for Telos:** Float should expose classification and machine metadata as
Float-owned facilities, while `universe.math` owns elementary functions. The
public contract must distinguish correctly-rounded operations from backend-
dependent transcendental results.

## Rust `num-traits`: small operation traits are more reusable than one tower

`num-traits` separates identities (`Zero`, `One`), basic operations (`Num`),
sign, bounds, checked operations, Euclidean operations, multiplication-addition,
and Float-specific capabilities. It provides a useful example of decomposing
generic algorithms into operation sets rather than requiring one universal
numeric class.

Source: [num-traits 0.2.19 API](https://docs.rs/num-traits/0.2.19/num_traits/).

**Lesson for Telos:** adopt the decomposition principle, not Rust trait syntax.
Telos's current traits are nominal, explicit, associated-type-capable, and do not
support trait inheritance or trait objects. Its numeric vocabulary should be a
small set of Telos traits justified by actual algorithms.

## Synthesis

The external evidence supports four recommendations already reflected elsewhere
in this report:

1. normalize exact quotients canonically;
2. keep Float classification and backend guarantees explicit;
3. decompose generic requirements into small capabilities;
4. do not introduce a global promotion lattice until its ambiguity and precision
   rules are a first-class Telos specification.
