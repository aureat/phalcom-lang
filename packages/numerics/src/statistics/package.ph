@!documentation("Finite Float descriptive statistics with explicit estimator names and mergeable summaries.")

from .descriptive import Descriptive, EmptyInputError, UndefinedStatisticError, MomentSummary
from .moments import RunningMoments
from .covariance import CovarianceSummary, RunningCovariance

let sum = &Descriptive.sum(_)
let sumInts = &Descriptive.sumInts(_)
let mean = &Descriptive.mean(_)
let meanExact = &Descriptive.meanExact(_)
let minimum = &Descriptive.minimum(_)
let maximum = &Descriptive.maximum(_)
let extrema = &Descriptive.extrema(_)
let span = &Descriptive.span(_)
let populationVariance = &Descriptive.populationVariance(_)
let sampleVariance = &Descriptive.sampleVariance(_)
let populationStddev = &Descriptive.populationStddev(_)
let sampleStddev = &Descriptive.sampleStddev(_)
let populationCovariance = &Descriptive.populationCovariance(_, with)
let sampleCovariance = &Descriptive.sampleCovariance(_, with)
let correlation = &Descriptive.correlation(_, with)
let median = &Descriptive.median(_)
let quantile = &Descriptive.quantile(_, probability)
let weightedMean = &Descriptive.weightedMean(_, weights)

export (
  sum, sumInts, mean, meanExact, minimum, maximum, extrema, span,
  populationVariance, sampleVariance, populationStddev, sampleStddev,
  populationCovariance, sampleCovariance, correlation,
  RunningMoments, RunningCovariance,
  median, quantile, weightedMean, MomentSummary,
  EmptyInputError, UndefinedStatisticError
)
