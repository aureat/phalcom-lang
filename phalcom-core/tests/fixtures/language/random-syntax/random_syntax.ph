trait ProbeGenerator {
  nextU64 -> Int
  choose<T>(_ values: List<T>) -> Option<T> {
    if values.size == 0 { return None }
    Some(values.at(0))
  }
  nextInt(below upper: Int) -> Int { upper - 1 }
  nextInt(from lower: Int, below upper: Int) -> Int { lower }
}

trait ProbeDistribution {
  type Output
  sample<G>(using generator: G) -> Self::Output
    where G |= ProbeGenerator
}

data ProbeUniform private(_ lower: Int, _ upper: Int)
impl ProbeUniform {
  @class
  from(_ lower: Int, below upper: Int) -> ProbeUniform {
    ProbeUniform(lower, upper)
  }
}
impl ProbeDistribution for ProbeUniform {
  type Output = Int
  sample<G>(using generator: G) -> Int
    where G |= ProbeGenerator
  { generator.nextInt(from: self.lower, below: self.upper) }
}

class ProbeSequence {
  @constructor
  new() { self }
}
impl ProbeGenerator for ProbeSequence {
  nextU64 -> Int { 0 }
}
let sequence = ProbeSequence.new()
let selected: Option<Int> = sequence.choose([1, 2])
let sampled: Int = ProbeUniform.from(1, below: 3).sample(using: sequence)
System.print(sampled)
