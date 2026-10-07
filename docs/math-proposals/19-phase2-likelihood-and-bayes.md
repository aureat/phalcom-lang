# 19. Likelihood and bounded Bayesian foundations

Status: recommendation. Current Result, typed callable, data, and impl mechanisms
are E11–E19 and E24–E26. No generic Bayes object or inference runtime is proposed.

## 19.1 Evaluation and independence are explicit

One observation of a mass law uses pmf/logPmf; one continuous observation uses
pdf/logPdf. Batch helpers are `iidLogMassLikelihood(model, observations)` and
`iidLogDensityLikelihood(model, observations)`, generic over the corresponding
capability and Iterable with matching Observation type. They sum direct log
contributions with compensation and extended-value handling from file 16.

The `iid` name asserts the model assumption: observations are independent and
identically distributed conditional on these fixed parameters. There is no
automatic conditional-independence proof. Empty observations yield log 0.0
(likelihood 1). A matching-type impossible outcome contributes -infinity.
Invalid input or an indeterminate infinity combination raises the stated error.

A future `logLikelihoodAt(parameters:, observations:)` is an ordinary typed
callable over parameter values. No new Likelihood type is needed. `exp(logL)`
is an explicitly approximate conversion which may overflow/underflow; V1 does
not advertise product accumulation as the default likelihood path. Densities
and log likelihoods may exceed 1 and 0 respectively, and are not probabilities.

Model-specific sufficient-statistic kernels may omit parameter-independent
constants only when named/documented as unnormalized log kernels. Never use
those kernels for evidence or comparisons between different observation spaces.

## 19.2 Sufficient statistics earn concrete types

| Type | Invariant / collection builder / merge | Uses |
|---|---|---|
| BernoulliCounts | successes, failures are nonnegative Int; total within approximate count range; fromBools; componentwise merge | Beta posterior, likelihood kernel, sequential updates |
| BinomialObservation | trials>=0, 0<=successes<=trials; fromCount factory | Beta posterior for one grouped count; retains grouping for evidence |
| PoissonCounts | observations>=0, totalCount>=0; both zero when observations=0; fromCounts over nonnegative Int; merge | Unit-exposure Gamma posterior and streaming count likelihood kernel |
| MomentSummary | n/mean/scaled M2 with validated private state | Future Normal parameter fitting and Gaussian class features |

These types represent information with invariants and different model roles.
They justify allocation/typing more strongly than generic Observation<T> or
Statistic<T>. Builders consume iterables once. Raw constructors are private;
zero counts produce valid empty sufficient statistics. Public aggregates do not
retain the source observations. Validated numeric-range checks precede merge.

No common SufficientStatistic protocol in V1: the update law and adequate
evidence constants differ. Existing fold and concrete merge already support at
least Bernoulli, Poisson, and Normal-summary use cases. A future generic protocol
must serve an additional generic inference algorithm, not just group these names.

## 19.3 Beta–Bernoulli and Beta–Binomial

Canonical prior: Beta(alpha=a,beta=b), a,b>0. For successes s and failures f:

```text
posterior = Beta(alpha=a+s, beta=b+f)
posterior predictive next Bool = Bernoulli(p=(a+s)/(a+b+s+f))
```

V1 public methods in probability.bayes:

```text
betaBernoulli(prior: Beta, counts: BernoulliCounts) -> Beta
betaBinomial(prior: Beta, observation: BinomialObservation) -> Beta
bernoulliPredictive(posterior: Beta) -> Bernoulli
logBernoulliSequenceEvidence(prior: Beta, counts: BernoulliCounts) -> Float
logBinomialEvidence(prior: Beta, observation: BinomialObservation) -> Float
```

These are contract notation for captured method Families, not free-function
declarations. Evidence:

```text
ordered Bernoulli sequence: logBeta(a+s,b+f) - logBeta(a,b)
one Binomial(n,p) count k: logChoose(n,k) + logBeta(a+k,b+n-k) - logBeta(a,b)
```

The same posterior can come from different observed events. Summarizing an
ordered Bool sequence into counts does not change that event into “all sequences
with s successes.” The latter adds logChoose(s+f,s). Evidence names fix the
observation model and prevent that error.

Batch Bernoulli updates build counts then update once. Sequential updates may
update from singleton counts or merged chunks; all preserve Beta return type.
Real arithmetic is associative here, but Float parameter rounding makes chunked
and sequential results only numerically equivalent. Very large totals and
unrepresentable shape increments raise NumericalRangeError. Empty counts return
the same prior law. No mutation of prior descriptors.

Future BetaBinomial predictive for m trials:

```text
P(K=k|data) = choose(m,k)*B(a'+k,b'+m-k)/B(a',b')
```

Do not approximate this by Binomial(m,posteriorMean): marginalizing a shared
uncertain p induces dependence between future trials. BetaBinomial's public
descriptor is designed now and implemented later.

## 19.4 Gamma–Poisson

Canonical Gamma prior uses shape a and rate b. Unit-exposure Poisson observations
x_1,...,x_m with total s yield:

```text
posterior = Gamma(shape=a+s, rate=b+m)
```

V1 `gammaPoisson(prior: Gamma, counts: PoissonCounts) -> Gamma` fixes unit
exposure per observation. Empty counts return the prior law. Total counts and
observation counts remain exact until explicit validated Float conversion.

For varying known exposures t_i>=0, future update uses rate b+sum(t_i), with
x_i~Poisson(lambda*t_i); t_i=0 requires x_i=0. ExposedPoissonSummary should retain
total count and total exposure, plus constants when evidence is requested.
Ordinary PoissonCounts cannot imply an exposure or evidence constant that was
never stored.

Evidence needs more than s and m: for unit exposure it includes
`-sum(logFactorial(x_i))` in addition to
`a*ln(b)-logGamma(a)+logGamma(a+s)-(a+s)*ln(b+m)`.
Varying exposure also contributes sum(x_i*ln(t_i)), with zero cases handled.
Therefore Gamma–Poisson evidence is not a V1 operation on bare PoissonCounts.

Future posterior predictive at exposure t is a Negative Binomial count of
failures with shape a' and success probability b'/(b'+t):

```text
P(K=k) = Gamma(a'+k)/(Gamma(a')*k!)
         * (b'/(b'+t))^a' * (t/(b'+t))^k
```

This fixes a convention for the later descriptor. Poisson(posterior mean*t)
is a plug-in approximation, not the marginalized predictive law.

## 19.5 Dirichlet and categorical/multinomial extension

Design later Dirichlet(alphaVector), all finite positive concentrations, immutable
dimension/order. Observed category counts add componentwise; total category
count and stable labels must align explicitly. Posterior remains Dirichlet.
Next categorical predictive has normalized alpha' values; future batch count
predictive is DirichletMultinomial, not Multinomial at posterior mean.

Dirichlet is not a V1 blocker for Categorical sampling or class prior smoothing.
Its simplex sample/result type and log-density reference measure must be fixed
before it implements any capability. Do not force it into scalar CDF/quantiles.

## 19.6 Point estimates and interval vocabulary

Posterior mean is a concrete distribution getter. It is a Bayes point estimate
under squared-error loss, not universally optimal. MAP is a different quantity:
Beta has unique interior mode (a-1)/(a+b-2) only when a,b>1; for boundary or
multiple-mode cases there is no universal single Float result. Gamma's interior
mode (a-1)/b requires a>1; at smaller shape a boundary/singularity matters.
Defer a generic MAP helper and loss-independent Estimate wrapper.

V1 offers `equalTailedInterval(d, mass: c)` via HasQuantile, with 0<c<1 and
quantiles at (1-c)/2 and 1-(1-c)/2; use upperQuantile for the upper endpoint.
Result is an ordinary labeled tuple `(lower:, upper:)`. For a continuous
posterior this has posterior probability c within numerical accuracy. For a
discrete posterior its coverage may exceed c. The helper is a distribution
interval; calling it a credible interval requires a posterior interpretation.

Confidence intervals describe a repeated-sampling procedure and are outside V1.
Highest-density intervals require optimization/multimodal set semantics and are
deferred. Prediction intervals require an actual predictive law, not posterior
parameter quantiles. No generic `interval`, `confidence`, or “95% certainty” alias.

## 19.7 Future inference seams

Preserve direct normalized log evaluation, explicit parameter names/support,
concrete summary reuse, observation measure, and typed callbacks. These support
future likelihood fitting, standard errors, sampling distributions, hypothesis
tests/p-values, and differentiable log evaluators. StudentT/ChiSquared and
additional quantile backends are later requirements. Optimizers, autodiff,
MCMC/HMC, and probabilistic program graphs have no runtime or package obligation
in this V1.
