from .core import MonteCarlo, MonteCarloEstimate

let expectation = &MonteCarlo.expectation(_, count, using, evaluate)

export MonteCarloEstimate, expectation
