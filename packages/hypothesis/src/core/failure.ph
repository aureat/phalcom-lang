import .phase as Phases

/// A falsification and the stable identity used to preserve it while shrinking.
data Failure(
  exampleIndex: Int,
  phase: Phases.Phase,
  origin: Symbol,
  message: String,
)

impl Failure {
  /// Compares the failure identity without comparing generated values.
  sameOrigin(other: Failure) -> Bool {
    origin == other.origin
  }
}

export Failure
