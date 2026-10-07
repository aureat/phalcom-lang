from .generator import Generator

trait Distribution {
  type Output
  sample<G>(using generator: G) -> Self::Output where G |= Generator
}

export Distribution
