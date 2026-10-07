# Math proposal implementation handoff — 2026-10-07

## Continuation update — 2026-10-08

The supplied projection-residual progress defect is corrected in
`phalcom-semantic/src/checker/inference.rs`. Constraint normalization reports
rewrites as progress; canonical propagation owns proof retry and residual
removal. Both the solver-level and local-binding source reproducer failed
before the patch and pass afterward. Known-binding equality proof failure now
receives `GenericConstraintUnsatisfied` without publishing specialization.

New semantic coverage checks both bound spellings, rigid caller bindings,
Forwarded conformance provenance, insufficient/contradictory caller constraints,
normalization conflict provenance, and local/imported/re-exported generic
Families. The exported identity discriminator passes.

Completed verification:

- inference unit tests: 16 passed;
- inference foundation laws: 61 passed;
- generic associated constraints: 14 passed;
- generic conformance: 21 passed;
- Family invocation: 13 passed;
- math audit: 16 passed, 2 probability acceptance tests failed;
- scoped Rust formatting and `git diff --check`: passed.

One earlier parallel generic-conformance run failed because a metadata test
selected Iterable instead of Printable. An isolated discriminator and the
final completed module run both passed. No test expectation was weakened.
The first math audit attempt timed out; the later completed run above owns
the result. Workspace formatting reports existing unrelated formatting drift.
Workspace build/test/clippy and release certification were deferred.

The generic `logSumExp` restoration was tested but remains blocked. A typed
`List<Float>.new()` and its iterator pass the canonical constraint discriminator,
while a Float list literal in the same call fails despite final `List<Float>`
knowledge. The generic empty-list reduction reached execution without usable
Iterable evidence. The detailed follow-up incident and decision request are in
`MATH-GENERIC-ITERABLE-INCIDENT.md`.

The existing List reduction signature is restored as temporary containment.
Probability remains draft/UNVERIFIED. No runtime patch or Dynamic workaround
is retained. Compiler correction: IMPLEMENTED + FOCUSED_TESTED. Overall math
program: PARTIAL. Continue with the incident's contextual collection inference
and executable evidence decision before restoring the generic reduction.

All original uncommitted work is preserved. No commit, push, or merge.

The sections below retain the predecessor's implementation history.

Worktree: `/Users/altunhasanli/.codex/worktrees/math-proposals/phalcom`.
Base: c4db36bb, detached HEAD. Concurrent main checkout was not modified.
Input proposals remain in the main checkout at
`/Users/altunhasanli/dev/phalcom/phalcom/docs/math-proposals/`.

## Implemented numerical slices

- Proposal 13 A3: exact factorial, choose, fallingFactorial, risingFactorial;
  ordinary checked arbitrary-precision Int arithmetic and domain validation.
  Binomial counts exploit symmetry and cancel before multiplication.
- Proposal 16: List<Float> logSumExp with maximum shifting, Neumaier residual
  compensation, ln1p, NaN precedence, and extended-value endpoints.
- Proposal 13 A2: optional universe.math.special source backend, hidden kernel,
  shared canonical numerical errors, and embedded Universe registration.
  Public functions: logGamma, logBeta, logFactorial, logChoose, erfc, logErfc,
  regularizedGammaP/Q, logRegularizedGammaP/Q, regularizedBeta/complement,
  logRegularizedBeta/complement, inverseRegularizedBeta,
  inverseRegularizedGammaP, inverseNormal.
- Independent mpmath reference generator plus checked-in binary64 constants.
  Production and Cargo do not depend on Python/mpmath.

Backend details, domains, bounds, algorithm references, and numerical limitations
are in `phalcom-core/core/universe/src/math/special/README.md`.

## Verification acquired

- Initial math audit suite: 14 passed before the later special-function work.
- Initial special-function reference matrix: 161 cases passed.
- Expanded matrix: 219 reference cases passed, including extreme and subnormal
  tails, inverse references, exact endpoint laws, and stable log combinatorics.
- Whitespace and scoped Rust formatting checks passed.
- Special-function domain/range regression passed (7 invalid/range cases).
- Restored List logSumExp regression passed (1 test, 9 expressions).

A broad focused run during the failed generic experiment returned 2 passed,
15 failed. The special-function matrix and native descriptor test passed; other
failures came from the generic reduction signature and probability package.
This run is NOT a successful owning-suite gate. The reduction signature has
since been restored to its previously tested List surface.

## Active compiler boundary / incomplete work

`MATH-GENERIC-ITERABLE-INCIDENT.md` owns the concrete failure and decision request.
Generic helper delegation under `I |= Iterable, Item == Float` reports
GenericInferenceUnderconstrained. Explicit method type argument syntax is also
not accepted. No compiler fix, Dynamic workaround, or weakened test was added.

`packages/probability` is a draft containing logit/logOdds/odds/invLogit and
linear/log normalization; its required generic iterator acceptance tests are
currently failing. It is not a completed or verified package. A user decision
is pending between investigating canonical compiler inference and explicitly
implementing partial List entry points in this pass.

Overall: PARTIAL. Numerical slices are IMPLEMENTED + FOCUSED_TESTED; generic probability
package remains UNVERIFIED. No workspace/release certification. No commit, push,
or merge. Rational/Complex, robust moments/statistics, distributions, sampling,
Bayesian updates, and Monte Carlo remain unimplemented.

## Numerical correction record

Observed: the first small-shape upper-tail implementation made P(1,1e-300)
round to zero when it complemented an upper-tail log rounded to zero.
Cause: using the direct upper expression even when the LOWER tail was tiny.
Correction: choose the lower series when Q >= 1/2; use the factored direct upper
series when Q < 1/2. The expanded relative-error reference matrix passed.

Also added local logGamma and erf Taylor series to preserve near-one/subnormal
arguments, and lower-tail asymptotic inverse seeds to avoid unnecessary full
exponent-range bisection on tiny probabilities.
