import .core.settings as SettingsModule

// This standalone source mirrors the legacy Settings test while using the
// smaller, typed settings surface in the active package.
let defaults = SettingsModule.Settings.defaults

if defaults.maxExamples != 100 or
  defaults.maxShrinks != 500 or
  defaults.seed != 0 {
  throw Error.new("Hypothesis settings defaults changed unexpectedly")
}

System.print("PASS Hypothesis settings smoke")
