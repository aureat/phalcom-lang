import ..choices.data as ChoiceData
import ..choices.provider as Providers

/// Produces a value of `T` by requesting typed choices from a draw context.
trait Strategy<T> {
  draw<P>(drawData: ChoiceData.DrawData<P>) -> T
      where P |= Providers.ChoiceProvider
}

export Strategy
