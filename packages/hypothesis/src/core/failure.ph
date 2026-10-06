import .phase as Phases

/// Identifies a property failure independently of its message or example.
data FailureOrigin(
  kind: Symbol,
  property: Symbol
)

impl FailureOrigin {
  /// Compares both the failure category and stable property identity.
  sameOrigin(other: FailureOrigin) -> Bool {
    kind == other.kind and property == other.property
  }
}

/// A falsification and the stable identity used to preserve it while shrinking.
data Failure(
  exampleIndex: Int,
  phase: Phases.Phase,
  origin: FailureOrigin,
  message: String,
)

impl Failure {
  /// Compares the failure identity without comparing generated values.
  sameOrigin(other: Failure) -> Bool {
    origin.sameOrigin(other: other.origin)
  }
}

export (FailureOrigin, Failure)
