data Query<*Goals> {
  goals: Pack<*Goals>,
}

impl<*Goals> Query<*Goals> {
  @class
  of(
    *goals: Pack<*Goals>
  ) -> Self {
    Query {
      goals: goals,
    }
  }

  size -> Int { goals.size }
}

export Query
