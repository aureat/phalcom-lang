from universe.random import Distribution, Generator
from universe.collections.iterator_laws import Iterator

/// Infinite, single-pass sampler retaining the caller's descriptor and RNG.
class SampleIterator<D, G> {
  _distribution: D
  mut _generator: G

  @constructor
  private create(_ distribution: D, _ generator: G) {
    _distribution = distribution
    _generator = generator
  }

  @class
  new(_ distribution: D, _ generator: G) -> SampleIterator<D, G> {
    SampleIterator.create(distribution, generator)
  }

  next -> Option<<D as Distribution>::Output> {
    Option::Some(_distribution.sample(using: _generator))
  }
}

impl<D, G> Iterator for SampleIterator<D, G>
  where D |= Distribution, G |= Generator
{
  type Item = <D as Distribution>::Output
}

class Sampling {
  @class
  sampleMany<D, G>(_ distribution: D, count n: Int, using generator: G) -> List<<D as Distribution>::Output>
    where D |= Distribution, G |= Generator
  {
    if n < 0 { ArgumentError.new("sampleMany count must be nonnegative").raise() }
    // Reserve and validate the complete result buffer before the first sample
    // can consume generator state.
    let result = List<(<D as Distribution>::Output)>.withCapacity(n)
    let mut remaining = n
    while remaining > 0 {
      result.append(distribution.sample(using: generator))
      remaining = remaining - 1
    }
    return result
  }

  @class
  samples<D, G>(of distribution: D, using generator: G) -> SampleIterator<D, G>
    where D |= Distribution, G |= Generator
  { SampleIterator.new(distribution, generator) }
}

export Sampling, SampleIterator
