from universe.functional.chain import Chain
from ..buffer import Sized
from .laws import Semiring

data Edge<R>(_ source: Int, _ target: Int, _ weight: R)

/// Sparse weighted walk evaluation. Parallel edges represent alternatives.
/// Edge order is encounter order; multiplication is prefix then edge weight.
class Paths {
  @class
  private validate<R, N: Int>(_ edges: Chain<Edge<R>>) -> Unit {
    if N < 0 { ArgumentError.new("Paths: vertex extent must be nonnegative").raise() }
    let mut cursor = edges
    while not cursor.isEmpty {
      let edge = cursor.first.unwrap
      if edge.source < 0 or edge.source >= N or edge.target < 0 or edge.target >= N {
        ArgumentError.new("Paths: edge endpoint outside vertex extent").raise()
      }
      cursor = cursor.tail
    }
    ()
  }
  @class
  advance<R, N: Int>(_ state: Sized<R, N>, along edges: Chain<Edge<R>>) -> Sized<R, N>
    where R |= Semiring
  {
    Paths.validate<R, N>(edges)
    let result = List<R>.new()
    let mut index = 0
    while index < N { result.append(R.zero); index = index + 1 }
    let mut cursor = edges
    while not cursor.isEmpty {
      let edge = cursor.first.unwrap
      let contribution = state.at(edge.source).sequential(edge.weight)
      result[edge.target] = result.at(edge.target).alternative(contribution)
      cursor = cursor.tail
    }
    Sized<R, N>.from(Chain<R>.fromIterable(result))
  }
  /// Exactly length edges, including the empty walk at length zero.
  @class
  walks<R, N: Int>(from source: Int, length: Int, along edges: Chain<Edge<R>>) -> Sized<R, N>
    where R |= Semiring
  {
    if source < 0 or source >= N or length < 0 { ArgumentError.new("Paths.walks: invalid source or length").raise() }
    Paths.validate<R, N>(edges)
    let mut state = Sized<R, N>.tabulate(|index| { if index == source { R.one } else { R.zero } })
    let mut step = 0
    while step < length {
      state = Paths.advance(state, along: edges)
      step = step + 1
    }
    state
  }
  /// All walks with 0..maximum edges. Each length is counted once, which is
  /// essential for non-idempotent alternatives such as counts and weights.
  @class
  bounded<R, N: Int>(from source: Int, maximum: Int, along edges: Chain<Edge<R>>) -> Sized<R, N>
    where R |= Semiring
  {
    if source < 0 or source >= N or maximum < 0 { ArgumentError.new("Paths.bounded: invalid source or maximum").raise() }
    Paths.validate<R, N>(edges)
    let mut frontier = Sized<R, N>.tabulate(|index| { if index == source { R.one } else { R.zero } })
    let mut total = frontier
    let mut step = 0
    while step < maximum {
      frontier = Paths.advance(frontier, along: edges)
      total = total.zipWith(frontier, combine: |left, right| { left.alternative(right) })
      step = step + 1
    }
    total
  }
}
export Edge, Paths
