/// Example and accepted-shrink-step limits for one property search.
data Settings(
  maxExamples: Int,
  maxShrinkSteps: Int,
)

impl Settings {
  /// Provides the default example and accepted-shrink-step limits.
  @class
  defaults -> Settings {
    Settings(maxExamples: 100, maxShrinkSteps: 500)
  }
}

export Settings
