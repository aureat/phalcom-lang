import ..core.settings as SettingsModule
import ..core.result as Results
import ..choices.provider as Providers
import .specification as Specifications
import .search as Search

class Property {
  /// Runs a typed property with the supplied settings and choice source.
  @class
  check<T>(
    specification: Specifications.PropertySpec<T>,
    settings: SettingsModule.Settings,
    providers: Providers.ChoiceProviderFactory
  ) -> Results.PropertyResult<T> {
    Search.SearchEngine<T>.new().run(specification, settings, providers)
  }
}

export Property
