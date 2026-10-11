# Nonlinear and contextual units

`LinearUnit` is deliberately not a universal "anything called a unit" type.

- absolute Celsius/Fahrenheit are `AffineUnit<Temperature>`;
- logarithmic scales such as decibels should define their own callable values;
- currency exchange must use explicit exchange-rate context rather than a
  `LinearUnit<Money>` conversion graph;
- irrational scales use `ApproximateUnit<K>` and remain outside exact
  Rational-backed structural composition.

The reusable language pattern is callable values plus nominal semantic types,
not forcing every domain through physical linear-unit algebra.
