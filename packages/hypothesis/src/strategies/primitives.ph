import .strategy as StrategyModule
import ..choices.data as ChoiceData

/// Generates integers from an inclusive range.
data IntStrategy(min: Int, max: Int, shrinkTowards: Int)

/// Generates booleans with a preferred shrink value.
data BoolStrategy(shrinkTowards: Bool)

impl StrategyModule.Strategy<Int> for IntStrategy {
  /// Draws an integer from the configured inclusive range.
  draw(drawData: ChoiceData.DrawData) -> Int {
    drawData.drawInt(min, max, shrinkTowards)
  }
}

impl StrategyModule.Strategy<Bool> for BoolStrategy {
  /// Draws a boolean with the configured shrink target.
  draw(drawData: ChoiceData.DrawData) -> Bool {
    drawData.drawBool(shrinkTowards)
  }
}

export (IntStrategy, BoolStrategy)
