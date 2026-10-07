# Telos Numerical, Statistical, and Probability Foundations

This folder contains the Phase 1 numerical architecture and its Phase 2
statistics, probability, and Bayesian continuation. The live repository identifies the language as
**Phalcom** (`docs/specs/README.md` and the `phalcom-*` crates); the user brief
calls it **Telos**. This report uses **Telos** for the proposed language and
**Phalcom** when referring to repository evidence.

## Report map

### Phase 1 — numerical and mathematical substrate

1. [Executive recommendation](01-executive-recommendation.md)
2. [Repository findings and evidence ledger](02-repository-findings.md)
3. [Semantic model and numerical ontology](03-semantic-model.md)
4. [Exact arithmetic: Int, Rational, conversion, and algorithms](04-exact-arithmetic.md)
5. [Float and numerical robustness](05-float-semantics.md)
6. [Complex numbers](06-complex-numbers.md)
7. [Generic abstractions and mathematical APIs](07-generic-abstractions-and-math-api.md)
8. [Package architecture and extension seams](08-package-architecture.md)
9. [API sketches and usage examples](09-api-sketches.md)
10. [Selective external prior art](10-prior-art.md)
11. [Gaps, rejected alternatives, V1 boundary, and future phases](11-boundary-and-open-questions.md)

### Phase 2 — statistics, probability, and Bayesian foundations

12. [Executive recommendation and current repository evidence](12-phase2-recommendation-and-evidence.md)
13. [Phase 1 dependencies and foundation amendments](13-phase2-foundation-amendments.md)
14. [Ontology, capabilities, alternatives, and extension contracts](14-phase2-ontology-and-capabilities.md)
15. [Collection statistics, results, and streaming summaries](15-phase2-statistics.md)
16. [Numerical algorithms and probability-domain semantics](16-phase2-stability-and-probability.md)
17. [Distribution families, support, and parameter contracts](17-phase2-distributions.md)
18. [RNG integration, sampling, and reproducibility](18-phase2-random-and-sampling.md)
19. [Likelihoods, sufficient statistics, and Bayesian updates](19-phase2-likelihood-and-bayes.md)
20. [Monte Carlo, simulation, and classifier stress tests](20-phase2-monte-carlo-and-classification.md)
21. [Telos API sketches and complete workflow examples](21-phase2-api-and-workflows.md)
22. [Packages, prior art, gaps, and decisive V1 boundary](22-phase2-packages-and-boundary.md)

Start Phase 2 at [12](12-phase2-recommendation-and-evidence.md). Its evidence
ledger separates live implementations from specifications and proposed members.
The input brief is [phase2-prompt.md](phase2-prompt.md); continuation context is
[HANDOFF.md](HANDOFF.md).

## Status vocabulary

- **Observed current behavior** — verified in live source or tests.
- **Existing intended architecture** — stated by a current specification or accepted decision, whether or not fully implemented.
- **Recommendation** — this report's proposed architecture.
- **Future direction** — deliberately deferred work or a dependency for later packages.

## Scope

Both phases are design reports. Phase 2 specifies a bounded statistical and
probability library, including limited Bayesian and Monte Carlo modules. Source,
tests, language specifications, and manifests are outside this deliverable.
Linear algebra, tensors, automatic differentiation, optimization, symbolic
algebra, dataframes, and general ML remain later consumers.

Phase 1's “future phases” in file 11 describe anticipated numerical work streams;
they do not renumber the user-requested Phase 2. File 13 records the specific
foundation amendments required by this continuation. The existing Phase 1
recommendations remain the baseline except where those amendments say otherwise.

All Phase 2 Telos blocks are **proposed library API in current syntax**, unless
explicitly labeled as current API or non-executable contract notation. They have
been compared with live source and test precedents, not compiled against a new
statistics implementation. Proposal status does not ratify a language change.
