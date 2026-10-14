from universe.functional import Chain
from experiments.semirings import Edge, Paths, Count, Reachability, Tropical

class Graphs {
  @class run() -> Unit {
    // Two parallel alternatives reach vertex 1, then one edge reaches vertex 2.
    let edges = Chain<Edge<Count>>::Empty
      .prepend(Edge<Count>(1, 2, Count.one))
      .prepend(Edge<Count>(0, 1, Count.one))
      .prepend(Edge<Count>(0, 1, Count.one))
    System.print(Paths.walks<Count, 3>(from: 0, length: 2, along: edges).at(2).value)
    let reachable = edges.map(|edge| { Edge<Reachability>(edge.source, edge.target, Reachability.one) })
    System.print(Paths.bounded<Reachability, 3>(from: 0, maximum: 2, along: reachable).at(2).value)
    let costs = Chain<Edge<Tropical>>::Empty
      .prepend(Edge<Tropical>(0, 2, Tropical::Cost(10)))
      .prepend(Edge<Tropical>(1, 2, Tropical::Cost(3)))
      .prepend(Edge<Tropical>(0, 1, Tropical::Cost(2)))
    System.print(Paths.bounded<Tropical, 3>(from: 0, maximum: 2, along: costs).at(2))
    ()
  }
}
export Graphs
