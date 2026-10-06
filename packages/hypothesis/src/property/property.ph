import ..core.settings as SettingsModule
from ..core.result import PropertyResult
from ..choices.provider import ChoiceProvider, ChoiceProviderFactory
import .specification as Specifications
from .search import SearchEngine
import ..strategies.strategy as StrategyModule

class Property {
  /// Runs a typed property with the supplied settings and choice source.
  @class
  check<T, S, P>(
    specification: Specifications.PropertySpec<T, S>,
    settings: SettingsModule.Settings,
    providers: ChoiceProviderFactory<P>
  ) -> PropertyResult<T>
      where
        S |= StrategyModule.Strategy<T>,
        P |= ChoiceProvider
  {
    SearchEngine.new(specification: specification).run(
      settings: settings,
      factory: providers
    )
  }
}

export Property
