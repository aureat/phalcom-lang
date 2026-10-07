@!documentation("Checked binary64 probability helpers. No hidden random state or implicit numeric conversion.")

from .probability_helpers import ProbabilityHelpers, UndefinedNormalizationError

let logit = &ProbabilityHelpers.logit(_)
let logOdds = &ProbabilityHelpers.logit(_)
let odds = &ProbabilityHelpers.odds(_)
let invLogit = &ProbabilityHelpers.invLogit(_)
let normalizeWeights = &ProbabilityHelpers.normalizeWeights(_)
let normalizeLogWeights = &ProbabilityHelpers.normalizeLogWeights(_)
let normalizedLogWeights = &ProbabilityHelpers.normalizedLogWeights(_)

export (
  logit, logOdds, odds, invLogit,
  normalizeWeights, normalizeLogWeights, normalizedLogWeights,
  UndefinedNormalizationError
)
