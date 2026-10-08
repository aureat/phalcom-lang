/// Nominal basis identities used by contextual algebraic realization.
data I()
data J()
data K()

/// A basis value retains its axis identity until a numeric target is selected.
data Basis<Axis>()

// Runtime bindings are used here because structural data constants are still
// outside the current static-materialization contract. These bindings remain
// immutable at the source level and preserve the canonical data values.
let i = Basis<I>()
let j = Basis<J>()
let k = Basis<K>()

export I, J, K, Basis, i, j, k
