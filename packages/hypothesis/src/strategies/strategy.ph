import ..choices.data as ChoiceData

/// Produces a value of `T` by requesting typed choices from a draw context.
trait Strategy<T> {
  draw(drawData: ChoiceData.DrawData) -> T
}

export Strategy
