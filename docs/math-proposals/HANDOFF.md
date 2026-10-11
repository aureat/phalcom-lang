# Handoff for Phase 2

Continue the Telos numerical/mathematical design in this folder only.

- Treat the existing files as the Phase 1 baseline; extend them rather than creating a competing report.
- Preserve the distinction between **observed current behavior**, **existing intended architecture**, **recommendation**, and **future direction**.
- Ground every claim in live repository evidence; cite files, symbols, tests, or specifications.
- The repository is named Phalcom while the requested language is Telos. Preserve that documented naming assumption.
- Use current Telos/Phalcom syntax only. Mark unsupported or proposed syntax explicitly.
- Keep the design capability-oriented: avoid Rust-style trait inheritance, global implicit promotion, and unsupported language features.
- The original report was design-only. The user authorized implementation continuation on 2026-10-10; current implementation status is recorded below and in `docs/implementation/`.
- Maintain the bounded package structure and the explicit V1/deferred boundaries.
- Add new material as focused numbered Markdown files and update `README.md` when the report map changes.

## Phase 2 report state — 2026-10-07

The Phase 2 architecture report is in files 12–22. Start at
[12-phase2-recommendation-and-evidence.md](12-phase2-recommendation-and-evidence.md)
and use the updated README report map. Files 01–11 remain the Phase 1 baseline;
file 13 records the explicit foundation amendments.

The recommendation uses two optional consumer packages (`statistics` and
`probability`), independent capabilities, Float statistical kernels with explicit
conversion, immutable validated descriptors, explicit seeded RNG state, typed
conjugate updates, and bounded Monte Carlo. The V1 distribution set is nine
families; Multinomial/Geometric/Dirichlet and generalized inference remain later
work. File 22 owns the decisive boundary and bounded unresolved questions.

## Live implementation reconciliation — 2026-10-10

The earlier report statement that `universe.random` has no generator is stale
for `feat/complex-math`. The live tree contains `Generator`,
`Xoshiro256PlusPlus`, `Distribution`, `UniformInt`, `Bernoulli`,
`SecureRandom`, and deterministic rejection-sampling helpers. The proposal's
`RandomBits`/`Pcg32` is therefore an alternative contract to reconcile with the
existing API, not a missing-generator implementation task. Do not introduce a
second canonical sampling source without an explicit compatibility design.

`universe.math.special` already contains gamma/beta, complementary and log-tail
kernels, regularized incomplete functions, and inverse functions. The math
facade also has exact integer helpers and `logSumExp`. The optional
`universe.math` exposes source-defined compensated `sum` and range-aware
`mean`; signed Neumaier corrections compare operand magnitudes. The optional
`statistics` package contains finite-Float descriptive reductions, Type 7
quantiles, weighted means, scaled Welford/Chan moment and covariance summaries,
mutable running owners, and an exact Int-to-Rational `meanExact`. Its Phalcom
source is not yet compiled or behaviorally verified. The canonical `Rational`
source slice now includes exact `Complex<Rational>` division and integer powers;
the Float representability API is also present. These source interfaces remain
unverified. The
`probability` package now has generic single-pass normalization, independent
mass/density/tail/quantile capabilities, V1 scalar-law source, categorical
tables with copied finite support metadata, explicit RNG sampling, likelihood helpers, bounded Beta/Gamma
conjugate updates, and fixed-budget Monte Carlo under its own sub-package. The
existing `universe.random::Generator` and `Distribution` identities remain
canonical; this branch does not add competing `RandomBits` or `Sampleable`
traits. Binomial/Poisson sampling currently uses inverse-CDF source drafts.
`List.withCapacity` now enforces the approved shared 64 MiB backing-storage
budget, and `sampleMany` reserves before its first draw. The probability source
has not passed acceptance.

The Phase 2 API remains a recommendation. Its live-source evidence and package
ownership notes must be refreshed as implementation advances. Rust checks and
native-surface generation do not verify Phalcom source behavior. CLI source
acceptance was attempted in this continuation but is blocked by a native stack
overflow, including on a dependency-free project; see the
[implementation incident](../implementation/STDL001-numeric-library/stack-overflow-incident-2026-10-11.md).

## Package namespace reconciliation — 2026-10-11

The optional numerical sources now share the `packages/numerics` project and
are public as `numerics.special`, `numerics.statistics`, and
`numerics.probability`. `numerics` exposes these sibling modules; probability
uses the sibling special and statistics modules internally. Standalone
`statistics` and `probability` project manifests were removed. This records the
current package boundary; source acceptance remains unverified.

This handoff tracks authorized implementation. Focused Rust crate checks pass
for the current semantic and compiler changes; they do not establish Phalcom
source behavior. The new Rational/Complex, statistics, and probability source
remains unverified. Source acceptance is paused until the stack-overflow phase
and owner are identified. Focused generic Family/evidence checks and
independent numerical/sampler validation remain required before behavioral
claims are made.
