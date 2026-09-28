// A query is just a conjunction of goals.
// It has no execution semantics yet.
//
// Keeping it as a value is useful because later Solver.query(query)
// does not need any special query syntax.
data Query<*Goals> {
  goals: Tuple<*Goals>,
}

impl<*Goals> Query<*Goals> {
  class.of(
    *goals: *Goals
  ) -> Query<*Goals> {
    Query {
      goals: goals,
    }
  }

  size -> Int {
    goals.size
  }
}

export Query