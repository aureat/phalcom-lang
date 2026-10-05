data Query<*Goals> {
  goals: Pack<*Goals>,
}

impl<*Goals> Query<*Goals> {
  @class
  of(
    *goals: *Goals
  ) -> Self {
    Query<*Goals> {
      goals: goals,
    }
  }

  size -> Int { goals.size }
}

export Query
