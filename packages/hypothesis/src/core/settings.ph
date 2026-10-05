/// Limits and seed used for one property search.
data Settings(
  maxExamples: Int,
  maxShrinks: Int,
  seed: Int,
)

impl Settings {
  /// Provides the default example limit, shrink limit, and seed.
  @class
  defaults -> Settings {
    Settings(maxExamples: 100, maxShrinks: 500, seed: 0)
  }
}

export Settings
