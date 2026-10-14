//! Composable access and replacement for nested immutable data.

/// A typed focus from a whole S to a part T.
/// Supply a getter and a setter that returns a new whole. For a lawful lens:
/// get(set(s, t)) == t; set(s, get(s)) == s; and
/// set(set(s, a), b) == set(s, b). These are caller obligations, not proofs.
/// The wrapper is immutable; callbacks must avoid mutation for pure updates.
data Lens<S, T> private(_ read: (S) -> T, _ replace: (S, T) -> S)

impl<S, T> Lens<S, T> {
  @class
  new(get: (S) -> T, set: (S, T) -> S) -> Lens<S, T> {
    Lens<S, T>(get, set)
  }

  get(_ source: S) -> T { self.read.call(source) }

  set(_ source: S, to value: T) -> S { self.replace.call(source, value) }

  /// Reads once, transforms once, then replaces once.
  modify(_ source: S, using f: (T) -> T) -> S {
    self.set(source, to: f.call(self.get(source)))
  }

  /// Focuses through this lens and then other.
  /// A composed read invokes each getter once. A composed replacement reads
  /// the outer part once, replaces the inner part once, then replaces the
  /// outer part once. It never reads the previous inner value.
  compose<U>(_ other: Lens<T, U>) -> Lens<S, U> {
    Lens<S, U>.new(
      get: |source| { other.get(self.get(source)) },
      set: |source, value| { self.set(source, to: other.set(self.get(source), to: value)) }
    )
  }
}

export Lens
