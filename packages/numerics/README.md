# Numerics

Optional numerical modules built on `universe.math` and the canonical
collection and random APIs:

- [`numerics.special`](src/special/README.md) provides advanced special
  functions and numerical kernels.
- [`numerics.statistics`](src/statistics/README.md) provides finite-Float
  descriptive statistics and mergeable running summaries.
- [`numerics.probability`](src/probability/README.md) provides checked
  probability operations, distribution descriptors, sampling, Bayesian
  helpers, and Monte Carlo utilities.

The three modules share the `numerics` project and each has a public import
path. Probability reuses `special`, and Monte Carlo reuses `statistics`.
Source and runtime acceptance remains unverified for these APIs.
