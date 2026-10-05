/// The stage of property execution that produced an observation.
data Phase {
  /// A user-supplied example was evaluated.
  Explicit,

  /// A previously saved example was replayed.
  Reuse,

  /// A fresh example was generated.
  Generate,

  /// A smaller candidate example was evaluated.
  Shrink,

  /// A minimized example was replayed to confirm stability.
  Verify,
}

export Phase
