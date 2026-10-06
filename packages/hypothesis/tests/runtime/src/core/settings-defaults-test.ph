import hypothesis.core.settings as SettingsModule
import hypothesis_tests.support.test as Testing

let settings = SettingsModule.Settings.defaults

Testing.Test.expect(
  settings.maxExamples == 100,
  "Settings.defaults.maxExamples must be 100"
)
Testing.Test.expect(
  settings.maxShrinks == 500,
  "Settings.defaults.maxShrinks must be 500"
)
Testing.Test.expect(
  settings.seed == 0,
  "Settings.defaults.seed must be 0"
)

Testing.Test.pass("core.settings_defaults")
