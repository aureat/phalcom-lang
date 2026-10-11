from .core import BernoulliCounts, BinomialObservation, PoissonCounts, Bayes
from ..gamma_beta import Beta, Gamma
from ..bernoulli import Bernoulli
from ..likelihood import Likelihood

let betaBernoulli = &Bayes.betaBernoulli(prior, counts)
let betaBinomial = &Bayes.betaBinomial(prior, observation)
let bernoulliPredictive = &Bayes.bernoulliPredictive(_)
let logBernoulliSequenceEvidence = &Bayes.logBernoulliSequenceEvidence(prior, counts)
let logBinomialEvidence = &Bayes.logBinomialEvidence(prior, observation)
let gammaPoisson = &Bayes.gammaPoisson(prior, counts)
let equalTailedInterval = &Likelihood.equalTailedInterval(_, mass)

export (
  Beta, Gamma, Bernoulli, BernoulliCounts, BinomialObservation, PoissonCounts,
  betaBernoulli, betaBinomial, bernoulliPredictive,
  logBernoulliSequenceEvidence, logBinomialEvidence, gammaPoisson,
  equalTailedInterval
)
