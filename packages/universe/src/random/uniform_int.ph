from .generator import Generator
from .distribution import Distribution

data UniformInt private(_ lower: Int, _ upperExclusive: Int)

impl UniformInt {
  @class
  from(_ lower: Int, below upper: Int) -> UniformInt {
    if upper <= lower { ArgumentError.new("UniformInt.from: upper bound must be greater than lower bound").raise() }
    UniformInt(lower, upper)
  }
}

impl Distribution for UniformInt {
  type Output = Int
  sample<G>(using generator: G) -> Int where G |= Generator {
    generator.nextInt(from: self.lower, below: self.upperExclusive)
  }
}

export UniformInt
