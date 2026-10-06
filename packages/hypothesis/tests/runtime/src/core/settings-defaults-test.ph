import hypothesis.core.settings as SettingsModule
import hypothesis_tests.support.test as Testing

let settings = SettingsModule.Settings.defaults

Testing.Test.expect(
  settings.maxExamples == 100,
  "Settings.defaults.maxExamples must be 100"
)
Testing.Test.expect(
  settings.maxShrinkSteps == 500,
  "Settings.defaults.maxShrinkSteps must be 500"
)
Testing.Test.pass("core.settings_defaults")
