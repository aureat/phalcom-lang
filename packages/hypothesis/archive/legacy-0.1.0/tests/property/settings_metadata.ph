// Phase 06: @WithSettings overrides runner defaults for one property only.

import Assert from hypothesis
import Given from hypothesis
import WithSettings from hypothesis
import Settings from hypothesis
import PropertySuite from hypothesis
import StrategyRegistry from hypothesis
import discovery from "property/discovery"

class SettingsProperties is PropertySuite {
  @WithSettings(
    Settings.standard
      .examples(7)
      .seed(42)
  )
  @Given
  locallyConfigured(value: Int) {
    self.assertTrue(value.isA(Int))
  }
}

const receiver = SettingsProperties.new()
const definitions = discovery.PropertyDiscovery.discover(
  suiteClass: SettingsProperties,
  receiver: receiver,
  defaults: Settings.standard.examples(100),
  registry: StrategyRegistry.standard
)
const definition = definitions.at(0)
Assert.equal(7, definition.settings.maxExamples)
Assert.equal(Some.new(42), definition.settings.seedValue)
