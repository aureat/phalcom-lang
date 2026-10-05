import ..strategies.strategy as StrategyModule
import ..choices.data as ChoiceData

/// A generated value type paired with its property predicate.
data PropertySpec<T>(
  strategy: StrategyModule.Strategy<T>,
  property: (T) -> Bool
)

impl<T> PropertySpec<T> {
  /// Draws one value and applies the predicate.
  check(drawData: ChoiceData.DrawData) -> Bool {
    property(strategy.draw(drawData))
  }
}

export PropertySpec
