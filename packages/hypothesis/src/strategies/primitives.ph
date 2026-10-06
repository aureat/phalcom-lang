import .strategy as StrategyModule
import ..choices.data as ChoiceData
import ..choices.provider as Providers

/// Generates integers from an inclusive range.
data IntStrategy(min: Int, max: Int, shrinkTowards: Int)

/// Generates booleans with a preferred shrink value.
data BoolStrategy(shrinkTowards: Bool)

impl StrategyModule.Strategy<Int> for IntStrategy {
  /// Draws an integer from the configured inclusive range.
  draw<P>(drawData: ChoiceData.DrawData<P>) -> Int
      where P |= Providers.ChoiceProvider
  {
    drawData.drawInt(min: min, max: max, shrinkTowards: shrinkTowards)
  }
}

impl StrategyModule.Strategy<Bool> for BoolStrategy {
  /// Draws a boolean with the configured shrink target.
  draw<P>(drawData: ChoiceData.DrawData<P>) -> Bool
      where P |= Providers.ChoiceProvider
  {
    drawData.drawBool(shrinkTowards: shrinkTowards)
  }
}

export (IntStrategy, BoolStrategy)
