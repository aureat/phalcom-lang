# 17. Distribution descriptors and mathematical contracts

Status: recommendation. This file fixes model semantics independently of sampler
algorithm choices. Capabilities are in file 14; backend prerequisites are in 13.

## 17.1 Representation and construction

Use transparent immutable data descriptors for scalar-parameter distributions,
with private primary constructors and public validated associated factories.
For example the recommended Normal shape is:

```telos
// Proposed library declaration in current data/impl syntax.
data Normal private(mean: Float, stddev: Float)

impl Normal {
  @class
  new(mean: Float, stddev: Float) -> Normal {
    if not mean.isFinite or not stddev.isFinite or stddev <= 0.0 {
      throw ArgumentError.new("Normal requires finite mean and positive finite stddev")
    }
    Normal(mean: mean, stddev: stddev)
  }
}
```

This is a factory named new, not a replacement for the data primary constructor.
Direct `Normal(mean: ..., stddev: ...)` outside the owner is deliberately
inaccessible. E24 and its constructor-access follow-up ground this distinction.
Factories for finite literals raise ordinary errors on invalid parameters.
Parsing/untrusted configuration may use a named `tryNew` returning
Result<Descriptor,ArgumentError>; it shares validation, not a second invariant.
No public unchecked constructor, and no @constructor on data's primary factory.

Scalar descriptors store canonical parameters without RNG or mutable sampler
caches. A Categorical table should be an externally immutable class with private
copied storage and no writable accessors; shallowly immutable data around a
public List would expose a mutation hole. Cached CDF/log tables are private and
owned by the descriptor. Label values themselves may be ordinary references;
their equality/hash must remain stable while the categorical model is used.

## 17.2 V1 family matrix

Nine families form the smallest useful set covering the requested workflows and
two conjugate models. `new` names below are proposed associated factories.

| Family / canonical construction | Valid parameters | Sample / mass or density observation | Ordered operations / moments |
|---|---|---|---|
| Bernoulli.new(probability: p) | finite p in [0,1] | Bool / Bool | Tails use false<true; quantile Bool; mean=p and variance=p(1-p) of numeric indicator |
| Binomial.new(trials: n, probability: p) | Int 0<=n<=2^53-1, p in [0,1] | Int / Int | CDF/SF cut Int; quantile Int; mean=np, variance=np(1-p) |
| Categorical.fromWeights(labels: xs, weights: ws), plus fromProbabilities/fromLogWeights | Nonempty equal-sized tables; unique stable labels; valid weights | T / T | No CDF, quantile, mean, or variance for arbitrary labels |
| Poisson.new(rate: lambda) | finite lambda>=0, backend range checked | Int / Int | CDF/SF cut Int; quantile Int; mean=variance=lambda |
| Uniform.new(lower: a, upper: b) | finite a<b | Float / Float | Tails and quantile Float; mean=(a+b)/2, variance=(b-a)^2/12 via scaled paths |
| Normal.new(mean: mu, stddev: sigma) | finite mu, finite sigma>0 | Float / Float | Tails and quantile Float; mean=mu, variance=sigma² |
| Exponential.new(rate: r) | finite r>0 | Float / Float | Tails and quantile Float; mean=1/r, variance=1/r² |
| Gamma.new(shape: a, rate: b) | finite a,b>0 | Float / Float | Tails and quantile Float; mean=a/b, variance=a/b² |
| Beta.new(alpha: a, beta: b) | finite a,b>0 | Float / Float | Tails and quantile Float; mean=a/(a+b), variance=ab/((a+b)²(a+b+1)) via scaled ratios |

Variance means the model's population variance. It is distinct from a sample
variance estimator. No HasMoments conformance is inferred from these getters.
Parameter validity does not guarantee that every moment is representable;
moment overflow raises NumericalRangeError. Beta's ratios must avoid a+b overflow
where the ratio is representable.

Named alternate factories: `Normal.fromVariance(mean:, variance:)` and
`Gamma.fromScale(shape:, scale:)`. Both validate before converting to canonical
parameters; a reciprocal overflow is a numerical-range error. Defer precision
parameterization and implicit positional constructors. “Gaussian” is a
documentation synonym, not a separate descriptor identity.

## 17.3 Discrete mathematical semantics

- Bernoulli is a law over Bool. A single Bool is an observation, never a
  distribution. An explicit indicator map (false->0, true->1) supplies mean or
  count arithmetic; Bool is not silently a Number.
- Binomial is a success count from n independent equal-p Bernoulli trials.
  n=0 is a valid point mass at 0. p=0 and p=1 are valid degenerate laws.
- Poisson is a count at a stated interval rate lambda. lambda=0 is a point mass
  at 0. Gamma–Poisson exposure is supplied by the model update, not hidden in
  the Poisson descriptor.
- Categorical is one label from a finite ordered label table. Labels need not
  be ordered mathematically. Repeated labels are rejected rather than silently
  combining weights. The table's order fixes sampling/reproducibility, not a CDF.

Integer mass queries below zero or outside finite bounds return 0/log -infinity.
Typed mass APIs do not accept fractional Float and round it into an Int. A
future real-valued count-cut convenience must explicitly floor its cut.

### Multinomial and geometric: design now, later implementation

Multinomial is a joint count vector with k components c_i>=0 and sum c_i=n.
It is neither k independent binomial counts nor a collection of categorical
labels. Proposed construction is `Multinomial.new(trials:, probabilities:)`,
with stable category order owned by an immutable table, sample List<Int>,
mass on a validated count vector. Wrong length/negative count is invalid input;
nonnegative counts with wrong total describe an impossible event (mass zero).
It has no scalar CDF/quantile. Moments are vectors/covariance, and no future
scalar HasMoments contract should force them into Float.

Geometric counts failures before the first success: k=0,1,...; 0<p<=1;
pmf(k)=(1-p)^k*p, mean=(1-p)/p, variance=(1-p)/p². p=0 is invalid because no
finite count occurs with probability one. Name a later trials-until-success
variant explicitly, with support starting at 1. Negative Binomial must likewise
fix successes/failures conventions before publication.

## 17.4 Support: sample domain, positive mass, and closure

Expose concrete `support` runtime metadata and `contains(_)` on that metadata.
Metadata can use small immutable Bool/count interval/real interval/finite-label
descriptors; there is no new general Set algebra or Support trait in V1. An
ordered support descriptor records finite versus unbounded endpoints and closure
flags. Categories expose a copied/immutable label view, never the live storage.

| Family | Mathematical/topological support metadata | Positive mass or endpoint density distinction |
|---|---|---|
| Bernoulli | {false,true}, reduced for p=0/1 | A degenerate zero-probability outcome is excluded |
| Binomial | Integers [0,n], reduced for p=0/1 | Discrete support equals positive mass outcomes |
| Categorical | Labels from positive source weights or finite source log weights | Zero/-infinity-weight labels remain in parameter table but are outside support |
| Poisson | Nonnegative integers; {0} at lambda=0 | Unbounded support when lambda>0 |
| Uniform | Closed real interval [a,b] | Point probabilities still zero; sampler returns [a,b) representable values |
| Normal | Real line | +/-infinity are CDF/quantile limits, not sampled real outcomes |
| Exponential / Gamma | Closed [0,+infinity) | Density convention at zero is specified below; no atom |
| Beta | Closed [0,1] | Endpoint density can be zero, finite, or singular; no endpoint atom |

Support membership describes the law, not whether a binary64 computation can
represent a particular tail or sample. Continuous density positivity and
mathematical support closure differ. Runtime parameters determine endpoints;
do not use Uniform<a,b> or Binomial<n,p> classifiers.

Categorical retains positivity flags and normalized log masses independently of
rounded Float pmf/prefix entries. A tiny positive mass may underflow to pmf=0
without disappearing from support; logPmf still gives the direct logarithm.
This is the same zero-versus-underflow distinction required by other laws.
Finite-resolution sampling may fail to resolve such a tiny category, within its
documented approximation; it never invents mass for a genuinely zero weight.

## 17.5 CDF, survival, log tails, and inverse contracts

For ordered laws, `cdf(c)=P(X<=c)` and `survival(c)=P(X>c)`. Therefore for an
integer law `P(X>=k)=survival(k-1)`. This off-by-one is part of the API contract.
For Bernoulli, false<true supplies ordered cuts without numeric coercion.

All four tail methods evaluate directly in the favorable branch. Discrete and
continuous distributions both expose them where meaningful; Categorical and
Multinomial do not. Continuous CDF at -infinity is 0, at +infinity 1; survival
is complementary; logs have matching 0/-infinity limits. NaN query is invalid.

Quantile for 0<q<1 is the generalized inverse: least support point with F>=q
for discrete laws, the inverse where continuous F is strictly increasing.
q=0 returns the lower support endpoint; q=1 the upper endpoint. For unbounded
Float laws, extended Float infinities represent these limits. Poisson/geometric
quantile(1) has no Int result and raises UndefinedStatisticError; degenerate
Poisson(0) returns 0 at every q. This avoids imposing an extended-integer type
on ordinary samples. Out-of-range or NaN q raises ArgumentError.

`upperQuantile(t)` computes the equivalent 1-t quantile without forming 1-t
when t is tiny; it has the corresponding reversed endpoints. Discrete values
satisfy the generalized-inverse convention rather than a continuous equality
to t. Backend nonconvergence raises NumericalEvaluationError. Future inverse
log-tail inputs can extend this contract without changing quantile probability
scale.

## 17.6 Density and stable formulas

Formulas below define semantics. Backend evaluation must use region-dependent
scaling and direct logarithmic/complementary methods.

| Law | Interior log density/mass and stable tail requirement |
|---|---|
| Bernoulli | true: ln(p), false: ln1p(-p), with p=0/1 branches |
| Binomial | logChoose(n,k)+k*ln(p)+(n-k)*ln1p(-p); use deviance/large-count branches; F(k)=I_(1-p)(n-k,k+1), S(k)=I_p(k+1,n-k) for 0<=k<n |
| Poisson | k*ln(lambda)-lambda-logFactorial(k), direct k=0/lambda=0 handling; F(k)=Q(k+1,lambda), S(k)=P(k+1,lambda) for k>=0 |
| Uniform | -ln(b-a), scaled width; CDF ratio and complementary ratio computed directly |
| Normal | -0.5*z²-ln(sigma)-0.5*ln(2*pi), z=(x-mu)/sigma with scaled differences; S(x)=0.5*erfc(z/sqrt2) |
| Exponential | ln(r)-r*x, x>=0; F=-expm1(-r*x), logS=-r*x |
| Gamma | a*ln(b)-logGamma(a)+(a-1)*ln(x)-b*x, x>0; P/Q(a,b*x) |
| Beta | (a-1)*ln(x)+(b-1)*ln1p(-x)-logBeta(a,b), 0<x<1; incomplete-beta pair |

Continuous values strictly outside support: pdf=0/logPdf=-infinity. Uniform
defines density 1/(b-a) at both finite endpoints; this chosen endpoint version
does not affect its law. Exponential pdf(0)=r. Gamma pdf(0): +infinity if shape<1,
rate if shape=1, zero if shape>1. Beta pdf(0): +infinity if alpha<1, beta if
alpha=1, zero if alpha>1; at 1 use beta and alpha symmetrically. Never let an
unhandled 0*ln(0) create NaN for a specified finite endpoint.

Normal/Exponential/Gamma density at +infinity is zero; Normal density at
-infinity is zero. Infinite values outside bounded support return zero. All
NaN inputs are rejected. These evaluation limits do not authorize infinite
observations in descriptive statistics.

Near-term: Multinomial, Geometric, Dirichlet and BetaBinomial, NegativeBinomial
predictive laws, StudentT/ChiSquared for future inference. Specialist laws such
as F, Weibull, Cauchy, Pareto, and generalized families remain deferred. Cauchy
is a useful reminder that a common distribution interface cannot promise a
finite mean or variance.
