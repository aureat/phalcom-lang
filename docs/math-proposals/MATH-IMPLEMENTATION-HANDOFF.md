# Math proposal implementation handoff — 2026-10-07

## Current main continuation status — 2026-10-11

The consolidation remains on `main`; Universe has not been relocated. The
attached audit's source-level corrections are present in the current work:
probability consumers use the structured `numerics.special` families, the
foreign inherent Bernoulli methods are owned by Universe, `Rational.inverse`
uses the normalized constructor, `stats.mean` delegates to the streaming math
kernel, the shared `UndefinedStatisticError` has one numerics owner, and
`monte_carlo` uses a hyphenated source directory while preserving its module
name. Focused numerical and package-client regressions have been added.

This is not source acceptance or numerical certification. Current evidence and
open blockers:

- `RUSTFLAGS='' cargo check -p phalcom-core` passes after the module-export
  Family gateway began forwarding the original call shape and generic recipe.
- `audit_probes::math::public_statistics_family_applies_its_recipe_after_live_member_selection`
  reaches the real `stats.mean` Family method and obtains the positive result,
  but its negative control still succeeds after the recipe's dispatch side is
  changed. Canonical runtime identity validation is open.
- The Float list-literal Iterable/associated-item discriminator still reports
  `GenericConstraintUnsatisfied`; the recent exact-conformance normalization
  fallback did not fix it. Typed `List<Float>` and its iterator pass the
  bounded discriminator. Inspect the materialized projection, normalization
  result, rewrite-progress flag, and residual on canonical retry before another
  semantic correction.
- The stable Float/reduction boundary test and the consolidated numerics
  end-to-end client have not passed their full focused gates. Do not infer
  distributional accuracy, RNG-consumption, moment, or workspace certification
  from the source edits or Rust compilation.

Keep the generic `Iterable<Item = Float>` API and evidence authority intact.
The next bounded work is a debugger-led root-cause pass for canonical literal
inference and runtime Family identity, followed by the smallest tests that
establish each correction. Update the STDL C7/C8/C9 records only when their
specific acceptance evidence is complete. Preserve the decision not to move
Universe in this workstream.

## Consolidation into main — 2026-10-08

The complete worktree snapshot was committed as `568c545a` on
`feat/math-proposals-consolidation` and merged into `main` as `c361793a`.
Git reported no conflicts. It automatically combined `builtin.rs`, `call.rs`,
and `context.rs`, preserving main's random APIs and callable-owner corrections.

Post-merge verification on main:

- inference unit tests: 16 passed;
- generic conformance tests: 21 passed;
- Family invocation tests: 13 passed;
- special-function reference matrix: 1 test passed, covering 219 cases;
- merge diff whitespace check: passed.

The earlier probability acceptance failures remain documented below. Those
tests and workspace/release gates were not rerun for consolidation. Probability
remains draft/UNVERIFIED, and the List reduction signature remains temporary
containment. Follow-up implementation should inherit main's merged compiler
prerequisites and consult `MATH-GENERIC-ITERABLE-INCIDENT.md`.

The source worktree and its branch are retained. No push was requested.
The sections below preserve the pre-consolidation implementation history.

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
