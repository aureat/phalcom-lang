import ..core.settings as SettingsModule
import ..core.result as Results
import ..choices.provider as Providers
import .specification as Specifications
import .search as Search
import ..strategies.strategy as StrategyModule

class Property {
  /// Runs a typed property with the supplied settings and choice source.
  @class
  check<T, S, F>(
    specification: Specifications.PropertySpec<T, S>,
    settings: SettingsModule.Settings,
    providers: F
  ) -> Results.PropertyResult<T>
      where
        S |= StrategyModule.Strategy<T>,
        F |= Providers.ChoiceProviderFactory,
        F::Provider |= Providers.ChoiceProvider
  {
    Search.SearchEngine<T, S>.new().run(specification, settings, providers)
  }
}

export Property
