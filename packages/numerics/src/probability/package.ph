@!documentation("Explicit-RNG probability descriptors, checked Float evaluations, and direct log-tail operations. No hidden random state or implicit numeric conversion.")

expose .bayes
expose .monte_carlo

from .probability_helpers import ProbabilityHelpers, UndefinedNormalizationError
from .capabilities import HasMass, HasDensity, HasOrderedTails, HasQuantile
from .bernoulli import Bernoulli, bernoulliSupport
from .continuous import Uniform, Normal, Exponential
from .discrete import Binomial, Poisson
from .gamma_beta import Gamma, Beta
from .categorical import Categorical
from .errors import UndefinedStatisticError
from .sampling import Sampling, SampleIterator
from .likelihood import Likelihood
from .support import RealSupport, IntegerSupport, FiniteSupport
from universe.random import Distribution, Generator

let logit = &ProbabilityHelpers.logit(_)
let logOdds = &ProbabilityHelpers.logit(_)
let odds = &ProbabilityHelpers.odds(_)
let invLogit = &ProbabilityHelpers.invLogit(_)
let normalizeWeights = &ProbabilityHelpers.normalizeWeights(_)
let normalizeLogWeights = &ProbabilityHelpers.normalizeLogWeights(_)
let normalizedLogWeights = &ProbabilityHelpers.normalizedLogWeights(_)
let sampleMany = &Sampling.sampleMany(_, count, using)
let samples = &Sampling.samples(of, using)
let iidLogMassLikelihood = &Likelihood.iidLogMassLikelihood(_, observations)
let iidLogDensityLikelihood = &Likelihood.iidLogDensityLikelihood(_, observations)
let tailDiagnostics = &Likelihood.tailDiagnostics(_, at)

export (
  logit, logOdds, odds, invLogit,
  normalizeWeights, normalizeLogWeights, normalizedLogWeights,
  Bernoulli, bernoulliSupport, Binomial, Poisson, Categorical, Uniform, Normal, Exponential, Gamma, Beta,
  Distribution, Generator, HasMass, HasDensity, HasOrderedTails, HasQuantile,
  RealSupport, IntegerSupport, FiniteSupport,
  Sampling, SampleIterator, sampleMany, samples,
  iidLogMassLikelihood, iidLogDensityLikelihood, tailDiagnostics,
  UndefinedNormalizationError, UndefinedStatisticError
)
